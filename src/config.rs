use std::path::Path;
use std::collections::HashMap;
use crate::termination::TimeTermination;
use crate::server::PathExecutableServer;
use crate::request_generator::SimpleJsonGetRequest;
use std::sync::Arc;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
#[serde(rename_all="camelCase")]
struct BenchmarkDescription {
    executable_path: String,
    executable_args: Vec<String>,
    url: String,
    json_path: Option<String>,
    method: Option<String>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct FileConfig {
    concurrency_levels: Vec<usize>,
    time_termination_in_seconds: i64,
    benchmarks: Vec<BenchmarkDescription>
}

impl FileConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<FileConfig, Box<dyn Error>> {
        let file = std::fs::File::open(path)?;
        let buf_reader = std::io::BufReader::new(file);
        let config = serde_json::from_reader(buf_reader)?;

        Ok(config)
    }
}

pub struct BenchmarkFactory {
    file_config: FileConfig,
    files_cache: HashMap<String, bytes::Bytes>,
    client: reqwest::Client
}

impl BenchmarkFactory {

    pub fn new(file_config: FileConfig) -> Result<BenchmarkFactory, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let mut files_cache = HashMap::default();
        let files= file_config.benchmarks
            .iter()
            .filter_map(|x| x.json_path.as_deref());
        for file in files {
            if !files_cache.contains_key(file) {
                let file_contents = std::fs::read_to_string(file)?;
                files_cache.insert(file.to_owned(), bytes::Bytes::from(file_contents));
            }
        }
        Ok(BenchmarkFactory { files_cache, file_config, client })
    }

    pub fn create_concurrency_levels(&self) -> Vec<usize> {
        self.file_config.concurrency_levels.clone()
    }

    pub fn create_termination(&self) -> TimeTermination {
        let termination = if self.file_config.time_termination_in_seconds <= 0 {
            1
        } else {
            self.file_config.time_termination_in_seconds
        };
        TimeTermination::new(chrono::Duration::seconds(termination))
    }

    pub fn create_executables(&self) -> Result<Vec<(PathExecutableServer, Vec<SimpleJsonGetRequest>)>, Box<dyn Error>> {
        self.file_config
            .benchmarks
            .iter()
            .map(|x| {
                let server = PathExecutableServer::new(&x.executable_path, x.executable_args.as_ref());
                let url = Arc::new(x.url.clone());
                let client = self.client.clone();
                let method = x.method.as_deref().unwrap_or("GET");
                let json_content = x.json_path
                    .as_deref()
                    .and_then(|x| self.files_cache.get(x))
                    .cloned();

                Ok((server, vec![SimpleJsonGetRequest::new(json_content, url, client, method)?]))
            })
            .collect()
    }
}
