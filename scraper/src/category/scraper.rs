use async_trait::async_trait;
use reqwest::Client;
use scraper::Html;

use crate::{
    fetcher::{FetchHtml, HtmlFetcher},
    types::Error,
    utils,
};

#[async_trait]
pub trait UrlCrawler {
    async fn get_href(&self, path: &str) -> Result<Vec<String>, Error>;
}

pub struct CategoryScraper<T = Client>
where
    T: FetchHtml + Clone,
{
    fetcher: HtmlFetcher<T>,
    base_url: String,
}

impl<T: FetchHtml + Clone> CategoryScraper<T> {
    pub fn new(fetcher: HtmlFetcher<T>, base_url: &str) -> Self {
        Self {
            fetcher,
            base_url: base_url.to_string(),
        }
    }
}

#[async_trait]
impl<T: FetchHtml + Clone + std::marker::Send + std::marker::Sync + 'static> UrlCrawler
    for CategoryScraper<T>
{
    async fn get_href(&self, path: &str) -> Result<Vec<String>, Error> {
        let url = format!("{}{}", self.base_url, path);
        let html = self.fetcher.fetch_only(&url).await?;
        let doc = Html::parse_document(&html);

        let links = utils::extract_all_href(
            &doc.root_element(),
            "li.category-page__member a.category-page__member-link",
        )?;
        // info!("links: {:?}", links);
        return Ok(links);
    }
}
