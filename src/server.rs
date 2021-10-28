use tokio::process::{Child, Command};
use std::error::Error;
use std::pin::Pin;
use std::future::Future;
use std::fmt::{Display, Formatter};

pub struct PathExecutableServer {
    name: String,
    args: Vec<String>,
    child: Option<Child>,
    start_duration: std::time::Duration
}

impl PathExecutableServer {
    pub fn new<N: AsRef<str>, R: AsRef<str>>(name: N, args: &[R]) -> PathExecutableServer {
        PathExecutableServer {
            name: name.as_ref().to_string(),
            args: args.iter().map(|x| x.as_ref().to_string()).collect::<Vec<_>>(),
            child: None,
            start_duration: std::time::Duration::from_secs(1)
        }
    }

    async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.child = Some(Command::new(&self.name)
            .args(self.args.iter())
            .spawn()?);
        Ok(tokio::time::sleep(self.start_duration).await)
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(mut child) = self.child.take() {
            child.kill().await?
        }

        Ok(())
    }
}

impl Display for PathExecutableServer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'a> Server<'a> for PathExecutableServer {
    type StartFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + 'a>>;
    type StopFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + 'a>>;

    fn start(&'a mut self) -> Self::StartFuture {
        Box::pin(self.start())
    }

    fn stop(&'a mut self) -> Self::StopFuture {
        Box::pin(self.stop())
    }
}

pub trait Server<'a> {
    type StartFuture: Future<Output = Result<(), Box<dyn Error>>> + 'a;
    type StopFuture: Future<Output = Result<(), Box<dyn Error>>> + 'a;

    fn start(&'a mut self) -> Self::StartFuture;
    fn stop(&'a mut self) -> Self::StopFuture;
}