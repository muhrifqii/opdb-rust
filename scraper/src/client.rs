use std::fmt::Debug;

use async_trait::async_trait;
use reqwest::Client;

use crate::types::Error;

#[async_trait]
pub trait FetchHtml: Send + Sync + Debug {
    async fn fetch(&self, url: String) -> Result<String, Error>;
}

#[derive(Debug, Clone)]
pub struct HttpClientWrapper(pub Client);

#[async_trait]
#[cfg_attr(test, mockall::automock)]
impl FetchHtml for HttpClientWrapper {
    async fn fetch(&self, url: String) -> Result<String, Error> {
        self.0
            .get(&url)
            .send()
            .await
            .map_err(|r| Error::RequestError(format!("{:?} on url: {}", r, &url)))?
            .text()
            .await
            .map_err(|e| Error::RequestError(format!("{:?} on url: {}", e, &url)))
    }
}
