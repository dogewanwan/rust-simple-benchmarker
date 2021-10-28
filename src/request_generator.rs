use std::future::Future;
use std::sync::Arc;
use std::pin::Pin;
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use reqwest::StatusCode;
use thiserror::Error;
use std::error::Error;
use std::str::FromStr;

pub trait RequestGenerator {
    type Error: std::error::Error;
    type Request: Future<Output = Result<(), Self::Error>> + Send;

    fn generate_request(&self) -> Self::Request;
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Status is incorrect")]
    StatusError()
}

#[derive(Clone)]
pub struct SimpleJsonGetRequest {
    json_content: Option<bytes::Bytes>,
    url: Arc<String>,
    client: reqwest::Client,
    method: reqwest::Method
}

impl SimpleJsonGetRequest {
    pub fn new(json_content: Option<bytes::Bytes>, url: Arc<String>, client: reqwest::Client, method: &str) -> Result<SimpleJsonGetRequest, Box<dyn Error>> {
        let method = reqwest::Method::from_str(method)?;
        Ok(SimpleJsonGetRequest { json_content, url, client, method })
    }
}

impl RequestGenerator for SimpleJsonGetRequest {
    type Error = RequestError;
    type Request = Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send>>;

    fn generate_request(&self) -> Self::Request {
        let cloned_client = self.client.clone();
        let content = self.json_content.clone();
        let url = self.url.clone();
        let method = self.method.clone();

        Box::pin(async move {
            let req = cloned_client.request(method, url.as_str());
            let req = if let Some(x) = content {
                req
                    .body(x)
                    .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            } else {
                req
            };

            let result = req.send().await?;
            let status = result.status();
            let bytes = result.bytes().await?;

            if bytes.len() == 0 || status != StatusCode::OK {
                return Err(RequestError::StatusError())
            }

            Ok(())
        })
    }
}