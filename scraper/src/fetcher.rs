use async_trait::async_trait;
use reqwest::Client;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::types::Error;

pub type ArcMapHtml = Arc<Mutex<HashMap<String, String>>>;

#[async_trait]
pub trait FetchHtml {
    async fn fetch(&self, url: &str) -> Result<String, Error>;
}

#[async_trait]
impl FetchHtml for Client {
    async fn fetch(&self, url: &str) -> Result<String, Error> {
        Ok(self
            .get(url)
            .send()
            .await
            .map_err(|r| Error::RequestError(format!("{:?} on url: {}", r, url)))?
            .text()
            .await
            .unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct HtmlFetcher<T>
where
    T: FetchHtml,
{
    client: T,
    cache: ArcMapHtml,
}

impl<T: FetchHtml> HtmlFetcher<T> {
    pub fn new(client: T) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn fetch(&self, url: &str) -> Result<String, Error> {
        let mut cache = self.cache.lock().await;
        if let Some(html) = cache.get(url) {
            return Ok(html.clone());
        }

        let html = self.client.fetch(url).await?;
        cache.insert(url.to_string(), html.clone());
        Ok(html)
    }

    pub async fn fetch_only(&self, url: &str) -> Result<String, Error> {
        self.client.fetch(url).await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_trait::async_trait;
    use mockito::Server;

    use super::{FetchHtml, HtmlFetcher};
    use crate::types::Error;

    struct MockClient {
        res_req: HashMap<String, Result<String, Error>>,
    }

    #[async_trait]
    impl FetchHtml for MockClient {
        async fn fetch(&self, url: &str) -> Result<String, Error> {
            self.res_req.get(url).cloned().unwrap()
        }
    }

    #[tokio::test]
    async fn valid_fetcher() {
        let mock_fetch = MockClient {
            res_req: HashMap::from([("url".to_string(), Ok("htmls".to_string()))]),
        };
        let fetcher = HtmlFetcher::new(mock_fetch);

        let resp = fetcher.fetch_only("url").await;
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap(), "htmls");
        assert!(!fetcher.cache.lock().await.contains_key("url"));

        let resp = fetcher.fetch("url").await;
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap(), "htmls");
        assert!(fetcher.cache.lock().await.contains_key("url"));

        let resp = fetcher.fetch("url").await;
        assert_eq!(resp.unwrap(), "htmls");
    }

    #[tokio::test]
    async fn fetcher_with_mock_server() {
        let mut server = Server::new_async().await;
        let mocked = server
            .mock("GET", "/a_path")
            .with_body("Will of D")
            .create_async()
            .await;

        let fetcher = HtmlFetcher::new(reqwest::Client::builder().build().unwrap());

        let url = format!("{}/a_path", server.url());
        let resp = fetcher.fetch(&url).await;
        mocked.assert_async().await;
        assert_eq!(resp.unwrap(), "Will of D");

        let resp = fetcher.fetch("/404").await;
        assert!(resp.is_err());
    }
}
