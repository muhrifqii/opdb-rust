use std::collections::HashSet;

use async_trait::async_trait;
use log::info;
use scraper::Html;

use crate::{fetcher::HtmlFetcher, types::Error, utils};

#[async_trait]
pub trait UrlCrawler {
    async fn get_href(&self, path: &str) -> Result<Vec<String>, Error>;

    async fn get_nested_href(&self, path: &str) -> Result<Vec<String>, Error>;
}

#[derive(Debug)]
pub struct CategoryScraper {
    fetcher: HtmlFetcher,
    base_url: String,
}

impl CategoryScraper {
    pub fn new(fetcher: HtmlFetcher, base_url: &str) -> Self {
        Self {
            fetcher,
            base_url: base_url.to_string(),
        }
    }
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
impl UrlCrawler for CategoryScraper {
    async fn get_href(&self, path: &str) -> Result<Vec<String>, Error> {
        let url = format!("{}{}", self.base_url, path);
        let html = self.fetcher.fetch_only(&url).await?;
        let doc = Html::parse_document(&html);

        let links = utils::extract_all_href(
            &doc.root_element(),
            "li.category-page__member a.category-page__member-link",
        )?;
        // info!("links: {:?}", links);
        Ok(links)
    }

    /// DFS crawling
    async fn get_nested_href(&self, path: &str) -> Result<Vec<String>, Error> {
        let selector =
            &utils::parse_selector("li.category-page__member a.category-page__member-link")?;
        let root = path.to_string();

        let mut visited = HashSet::new();
        let mut stack = vec![root];
        let mut urls = Vec::<String>::new();

        while !stack.is_empty() {
            let next_path = stack.pop().unwrap();
            if visited.contains(&next_path) {
                continue;
            }
            info!("DFS crawling on {:?}", next_path);
            visited.insert(next_path);

            let url = format!("{}{}", self.base_url, path);
            let html = self.fetcher.fetch_only(&url).await?;
            let doc = Html::parse_document(&html);
            for a in doc
                .select(selector)
                .filter_map(|e| e.value().attr("href"))
                .map(String::from)
            {
                if a.contains("Category:") && !visited.contains(&a) {
                    stack.push(a);
                } else if !a.contains("Category:") {
                    urls.push(a);
                }
            }
        }

        Ok(urls)
    }
}
