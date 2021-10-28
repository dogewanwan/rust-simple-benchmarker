use crate::observation_logger::SimpleLogger;
use crate::config::{FileConfig, BenchmarkFactory};
use chrono::{DateTime, Utc};

mod server;
mod termination;
mod observation_logger;
mod request_generator;
mod config;
mod runner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = FileConfig::from_file("config.json")?;
    let factory = BenchmarkFactory::new(config)?;
    let levels = factory.create_concurrency_levels();
    let termination = factory.create_termination();
    let executables = factory.create_executables()?;

    runner::run_servers_benchmarks::<_, SimpleLogger<DateTime<Utc>>, _, _>(
        &levels,
        termination,
        executables)
        .await?;

    Ok(())
}