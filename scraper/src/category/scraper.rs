use std::collections::HashSet;

use async_trait::async_trait;
use log::{debug, error, info};
use scraper::Html;

use crate::{fetcher::HtmlFetcher, types::Error, utils};

#[async_trait]
pub trait UrlCrawler {
    async fn get_href(&self, path: &str) -> Result<Vec<String>, Error>;

    async fn get_nested_href(&self, path: &str, strict: bool) -> Result<Vec<String>, Error>;
}

#[derive(Debug)]
pub struct CategoryScraper {
    fetcher: HtmlFetcher,
}

impl CategoryScraper {
    pub fn new(fetcher: HtmlFetcher) -> Self {
        Self { fetcher }
    }
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
impl UrlCrawler for CategoryScraper {
    async fn get_href(&self, path: &str) -> Result<Vec<String>, Error> {
        let html = self.fetcher.fetch_only(path).await?;
        let doc = Html::parse_document(&html);

        let links = utils::extract_all_href(
            &doc.root_element(),
            "li.category-page__member a.category-page__member-link",
        )?;
        // info!("links: {:?}", links);
        Ok(links)
    }

    /// DFS crawling
    async fn get_nested_href(&self, path: &str, strict: bool) -> Result<Vec<String>, Error> {
        let root = path.to_string();

        let mut visited = HashSet::new();
        let mut stack = vec![root];
        let mut urls = Vec::<String>::new();
        let mut err_collection = vec![];

        while !stack.is_empty() {
            let next_path = stack.pop().unwrap();
            if visited.contains(&next_path) {
                continue;
            }
            debug!("DFS crawling on {:?}", next_path);
            visited.insert(next_path.clone());

            let html = self.fetcher.fetch_only(&next_path).await;
            if html.is_err() {
                err_collection.push(html.err().unwrap());
                continue;
            }
            let doc = Html::parse_fragment(&html.unwrap());
            let selector =
                &utils::parse_selector("li.category-page__member a.category-page__member-link")?;
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
        if err_collection.is_empty() {
            Ok(urls)
        } else if strict {
            err_collection.iter().for_each(|err| error!("{:?}", err));
            Err(Error::RequestError(format!(
                "%{} Error happened while crawling categories",
                err_collection.len()
            )))
        } else {
            info!("non-strict mode category crawler having some errors");
            err_collection.iter().for_each(|err| info!("{:?}", err));
            Ok(urls)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        category::{CategoryScraper, UrlCrawler},
        fetcher::mocks::prepare_fetcher,
    };

    #[tokio::test]
    async fn get_category_should_crawl_href_only() {
        let fetcher = prepare_fetcher([
            (
                "/wiki/Category:Pirate_Crews_by_Sea".to_string(),
                Ok(r##"
<div>
    <ul>
        <li class="category-page__member">
            <a href="/wiki/Category:Grand_Line_Pirate_Crews" class="category-page__member-link" title="Category:Grand Line Pirate Crews">
                Category:Grand Line Pirate Crews
            </a>
        </li>
        <li class="category-page__member">
            <a href="/wiki/Category:New_World_Pirate_Crews" class="category-page__member-link" title="Category:New World Pirate Crews">
                Category:New World Pirate Crews
            </a>
        </li>
    </ul>
</div>"##.to_string()),
            ),
        ]);

        let crawler = CategoryScraper::new(fetcher);
        let result = crawler
            .get_href("/wiki/Category:Pirate_Crews_by_Sea")
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn nested_category_should_crawl_all() {
        let fetcher = prepare_fetcher([
                (
                    "/wiki/Category:Pirate_Crews_by_Sea".to_string(),
                    Ok(r##"
    <div>
        <ul>
            <li class="category-page__member">
                <a href="/wiki/Category:Grand_Line_Pirate_Crews" class="category-page__member-link" title="Category:Grand Line Pirate Crews">
                    Category:Grand Line Pirate Crews
                </a>
            </li>
        </ul>
    </div>
    "##.to_string()),
                ),
                (
                    "/wiki/Category:Grand_Line_Pirate_Crews".to_string(),
                    Ok(r##"
    <div>
        <div>
            <ul>
                <li class="category-page__member">
                    <a href="/wiki/Category:Non-Canon_Grand_Line_Pirate_Crews" class="category-page__member-link" title="Category:Non-Canon Grand Line Pirate Crews">
                       Category:Non-Canon Grand Line Pirate Crews
                    </a>
                </li>
            </ul>
        </div>
        <div>
            <ul>
                <li class="category-page__member">
                    <a href="/wiki/Fallen_Monk_Pirates" class="category-page__member-link" title="Fallen Monk Pirates">
                        Fallen Monk Pirates
                    </a>
                </li>
            </ul>
        </div>
        <div>
            <ul>
                <li class="category-page__member">
                    <a href="/wiki/Category:New_World_Pirate_Crews" class="category-page__member-link" title="Category:New World Pirate Crews">
                    Category:New World Pirate Crews</a>
                </li>
            </ul>
        </div>
    </div>"##.to_string())
                ),
                (
                    "/wiki/Category:New_World_Pirate_Crews".to_string(), // having a looping child-parent
                    Ok(r##"
    <div>
        <ul>
            <li class="category-page__member">
                <a href="/wiki/Rocks_Pirates" class="category-page__member-link" title="Rocks Pirates">
                    Rocks Pirates
                </a>
            </li>
            <li class="category-page__member">
                <a href="/wiki/Category:Grand_Line_Pirate_Crews" class="category-page__member-link" title="Category:Grand Line Pirate Crews">
                    Category:Grand Line Pirate Crews
                </a>
            </li>
        </ul>
    </div>
                    "##.to_string())
                ),
                (
                    "/wiki/Category:Some-categories".to_string(),
                    Ok(r#"<ul><li class="category-page__member"><a href="/wiki/somepages" class="category-age__member-link" title="wow">a page</a></li></ul>"#.to_string()),
                )
        ]);
        let crawler = CategoryScraper::new(fetcher);
        let result = crawler
            .get_nested_href("/wiki/Category:Pirate_Crews_by_Sea", true)
            .await;
        assert!(result.is_err());
        let result = crawler
            .get_nested_href("/wiki/Category:Pirate_Crews_by_Sea", false)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
        let result = crawler
            .get_nested_href("/wiki/Category:Some-categories", true)
            .await;
        assert!(result.is_ok());
    }
}
