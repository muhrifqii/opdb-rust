use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::{client::FetchHtml, types::Error};

pub type ArcMapHtml = Arc<Mutex<HashMap<String, String>>>;

#[derive(Debug, Clone)]
pub struct HtmlFetcher {
    base_url: String,
    client: Arc<dyn FetchHtml>,
    cache: ArcMapHtml,
}

impl HtmlFetcher {
    pub fn new(
        client: impl FetchHtml + Send + Sync + std::fmt::Debug + 'static,
        base_url: &str,
    ) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Arc::new(client),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn fetch(&self, url_path: &str) -> Result<String, Error> {
        let mut cache = self.cache.lock().await;
        if let Some(html) = cache.get(url_path) {
            return Ok(html.clone());
        }

        let html = self
            .client
            .fetch(format!("{}{}", &self.base_url, &url_path))
            .await?;
        cache.insert(url_path.to_string(), html.clone());
        Ok(html)
    }

    pub async fn fetch_only(&self, url_path: &str) -> Result<String, Error> {
        self.client
            .fetch(format!("{}{}", &self.base_url, url_path))
            .await
    }
}

#[cfg(test)]
pub mod mocks {
    use std::collections::HashMap;

    use async_trait::async_trait;

    use crate::{
        client::{FetchHtml, MockHttpClientWrapper},
        types::Error,
    };

    use super::HtmlFetcher;

    #[derive(Clone, Debug)]
    pub struct MockClient {
        res_req: HashMap<String, Result<String, Error>>,
    }

    #[async_trait]
    impl FetchHtml for MockClient {
        async fn fetch(&self, url: String) -> Result<String, Error> {
            self.res_req
                .get(&url)
                .cloned()
                .ok_or(Error::RequestError(url))
                .unwrap()
        }
    }

    pub fn prepare_fetcher<const N: usize>(
        arr: [(String, Result<String, Error>); N],
    ) -> HtmlFetcher {
        let client = MockClient {
            res_req: HashMap::from(arr),
        };

        HtmlFetcher::new(client, "")
    }
}

#[cfg(test)]
mod tests {
    use mockito::Server;

    use super::HtmlFetcher;
    use crate::{client::HttpClientWrapper, fetcher::mocks::prepare_fetcher};

    #[tokio::test]
    async fn valid_fetcher() {
        let fetcher = prepare_fetcher([("url".to_string(), Ok("htmls".to_string()))]);

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

        let client = HttpClientWrapper(reqwest::Client::builder().build().unwrap());
        let fetcher = HtmlFetcher::new(client, &server.url());

        let resp = fetcher.fetch("/a_path").await;
        mocked.assert_async().await;
        assert_eq!(resp.unwrap(), "Will of D");

        server.reset();
        let resp = fetcher.fetch("/404").await;
        assert!(resp.is_err());
    }
}
