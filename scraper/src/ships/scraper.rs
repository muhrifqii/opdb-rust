use std::sync::Arc;

use log::{error, info};
use scraper::Html;
use tokio::{sync::Semaphore, task::JoinSet};

use crate::{
    category::UrlCrawler,
    fetcher::HtmlFetcher,
    types::{Error, NamedJpEn, NamedUrl},
    utils,
};

use super::models::Ship;

pub struct ShipScraper {
    fetcher: HtmlFetcher,
    category_crawler: Arc<dyn UrlCrawler>,
}

impl ShipScraper {
    pub fn new(fetcher: HtmlFetcher, category_crawler: Arc<dyn UrlCrawler>) -> Self {
        Self {
            fetcher,
            category_crawler,
        }
    }

    pub async fn scrape(&self) -> Result<Vec<Ship>, Error> {
        info!("crawling ship categories");
        let urls = self
            .category_crawler
            .get_nested_href("/wiki/Category:Ships", true)
            .await?;

        let concurrency_limit = Arc::new(Semaphore::new(20));
        let mut ships = vec![];
        let mut ship_tasks = JoinSet::new();
        for url in urls {
            let permit = concurrency_limit.clone().acquire_owned().await.unwrap();
            let fetcher = self.fetcher.clone();
            ship_tasks.spawn(async move {
                let _permit = permit; // keep permit alive during task
                let result = parse_ship_detail(fetcher, url.clone()).await;
                (url, result)
            });
        }
        info!("collecting ships");
        while let Some(res) = ship_tasks.join_next().await {
            match res {
                Ok((_, Ok(ship))) => {
                    ships.push(ship);
                }
                Ok((url, Err(e))) => error!("Error parsing ship detail at {}: {}", url, e),
                Err(e) => error!("JoinSet error {}", e),
            }
        }
        ships.sort();
        Ok(ships)
    }
}

async fn parse_ship_detail(fetcher: HtmlFetcher, ship_url: String) -> Result<Ship, Error> {
    let html = fetcher.fetch(&ship_url).await.map(utils::cleanup_html)?;
    let doc = Html::parse_document(&html);
    let pic_url = utils::parse_picture_url(&doc)?
        .first()
        .cloned()
        .unwrap_or_default();
    let en_name = utils::parse_main_page_title(&doc)?;
    let description = utils::parse_main_page_first_paragraph(&doc)?;
    let non_cannon = utils::parse_is_non_cannon(&doc)?;
    let mut name_detail = NamedJpEn::new(String::new(), en_name, description);
    let mut status = String::new();
    let mut affiliation = NamedUrl::default();
    let stat_selector = utils::parse_selector("aside.portable-infobox>section .pi-data")?;
    for el in doc.select(&stat_selector) {
        if let Some(kind) = el.attr("data-source") {
            match kind {
                "rname" => {
                    name_detail.name =
                        utils::parse_infobox_single_data_text(&el).unwrap_or_default();
                }
                "status" => {
                    status = utils::parse_infobox_single_data_text(&el).unwrap_or_default();
                }
                "affiliation" => {
                    affiliation = utils::parse_infobox_single_data_named_urls(&el)
                        .first()
                        .cloned()
                        .unwrap_or_default();
                }
                _ => {}
            }
        }
    }
    Ok(Ship::new(
        name_detail,
        ship_url,
        pic_url,
        affiliation,
        status,
        non_cannon,
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        category::CategoryScraper, fetcher::mocks::prepare_fetcher, ships::scraper::ShipScraper,
    };

    #[tokio::test]
    async fn test_get() {
        let fetcher = prepare_fetcher([
            (
                "/wiki/Category:Ships".to_string(),
                Ok(r##"
<div>
    <ul>
        <li class="category-page__member">
            <a href="/wiki/Category:someone" class="category-page__member-link" title="Category:someone">
                Category:Some Category
            </a>
        </li>
        <li class="category-page__member">
            <a href="/wiki/Hanjomaru" class="category-page__member-link" title="Hanjomaru">
                Hanjomaru
            </a>
        </li>
    </ul>
</div>"##
                    .to_string()),
            ),
            (
                "/wiki/Category:someone".to_string(),
                Ok(r##"
<div>
    <ul>
        <li class="category-page__member">
            <a href="/wiki/Northheim" class="category-page__member-link" title="Northheim">
                Northheim
            </a>
        </li>
    </ul>
</div>"##
                    .to_string()),
            ),
            (
                "/wiki/Hanjomaru".to_string(),
                Ok(r##"
<main>
    <span class="mw-page-title-main">Hanjomaru</span>
    <div id="mw-content-text">
        <p></p>
        <aside class="portable-infobox">
            <figure class="pi-image">
                <a href="/image-path-213" class="image"/>
            </figure>
            <section>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="rname">
                    <div class="pi-data-value pi-font"><i>Hanjōmaru</i></div>
                </div>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="affiliation">
                    <div class="pi-data-value pi-font"><a href="/wiki/Fallen_Monk_Pirates" title="Fallen Monk Pirates">Fallen Monk Pirates</a></div>
                </div>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="status">
                    <div class="pi-data-value pi-font">Active</div>
                </div>
            </section>
        </aside>
        <p></p>
        <p>Hanjomaru is the Fallen Monk Pirates ship.</p>
    </div>
</main>
                "##.to_string()),
            ),
            (
                "/wiki/Northheim".to_string(),
                Ok(r##"
<main>
    <span class="mw-page-title-main">Northheim</span>
    <div id="mw-content-text">
        <p></p>
        <aside class="portable-infobox">
            <figure class="pi-image">
                <a href="/image-path-213" class="image"/>
            </figure>
            <section>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="rname">
                    <div class="pi-data-value pi-font"><i>Northheim</i></div>
                </div>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="affiliation">
                        <h3 class="pi-data-label pi-secondary-font">Affiliation:</h3>
                    <div class="pi-data-value pi-font"><a href="/wiki/Mont_Blanc_Noland" title="Mont Blanc Noland">Mont Blanc Noland</a></div>
                </div>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="status">
                    <div class="pi-data-value pi-font">Unknown</div>
                </div>
            </section>
        </aside>
        <p></p>
        <p>The Northheim[2] was the ship Mont Blanc Noland used to explore the Grand Line.</p>
    </div>
</main>
                "##.to_string()),
            )
        ]);
        let crawler = Arc::new(CategoryScraper::new(fetcher.clone()));
        let scraper = ShipScraper::new(fetcher, crawler);
        let ships = scraper.scrape().await.unwrap();
        assert_eq!(ships.len(), 2);
    }
}
