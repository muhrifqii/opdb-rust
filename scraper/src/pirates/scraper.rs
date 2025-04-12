use itertools::Itertools as _;
use log::{debug, error, info};
use reqwest::Client;
use scraper::Html;
use tokio::task::JoinSet;

use crate::{
    category::UrlCrawler,
    fetcher::{FetchHtml, HtmlFetcher},
    types::{Error, NamedJpEn, NamedUrl, UrlTyped},
    utils,
};

use super::models::{Pirate, Ship};

pub struct PirateScraper<T = Client>
where
    T: FetchHtml + Clone,
{
    fetcher: HtmlFetcher<T>,
    base_url: String,
    category_crawler: Box<dyn UrlCrawler>,
}

impl<T: FetchHtml + Clone + Send + Sync + 'static> PirateScraper<T> {
    pub fn new(
        fetcher: HtmlFetcher<T>,
        category_crawler: Box<dyn UrlCrawler>,
        base_url: &str,
    ) -> Self {
        Self {
            fetcher,
            base_url: base_url.to_string(),
            category_crawler,
        }
    }

    pub async fn scrape(&self) -> Result<(Vec<Pirate>, Vec<Ship>), Error> {
        info!("crawling categories...");
        let category_by_sea_iter = self
            .category_crawler
            .get_href("/wiki/Category:Pirate_Crews_by_Sea")
            .await?
            .into_iter()
            .filter(|path| !path.contains("Category:Non-Canon"));
        let mut ships = vec![];
        let mut pirates = vec![];
        info!("crawling categories by sea");
        for sea_url in category_by_sea_iter {
            let mut pirate_urls = self
                .category_crawler
                .get_href(&sea_url)
                .await?
                .into_iter()
                .filter(|path| !path.contains("Category:Non-Canon"))
                .collect_vec();

            if let Some(i) = pirate_urls
                .iter()
                .position(|path| path.contains("Category:"))
            {
                let nested_cat_url = pirate_urls[i].clone();
                pirate_urls.swap_remove(i);
                pirate_urls.extend(
                    self.category_crawler
                        .get_href(&nested_cat_url)
                        .await?
                        .into_iter()
                        .filter(|path| {
                            !path.contains("Category:Non-Canon")
                                && path != "/wiki/New_Donquixote_Family"
                        }),
                );
            }

            let mut ship_tasks = JoinSet::new();
            info!("collecting pirates...");
            for pirate_url in pirate_urls {
                let fetcher = self.fetcher.clone();
                let html = fetcher
                    .fetch(&format!("{}{}", &self.base_url, &pirate_url))
                    .await
                    .map(utils::cleanup_html)?;
                let doc = Html::parse_document(&html);
                let pic_url = utils::parse_picture_url(&doc)?
                    .first()
                    .cloned()
                    .unwrap_or_default();
                let description = utils::parse_main_page_first_paragraph(&doc)?;
                let en_name = utils::parse_main_page_title(&doc)?;
                let stat_selector =
                    utils::parse_selector("aside.portable-infobox>section .pi-data")?;
                let mut name_detail = NamedJpEn::new(String::new(), en_name, description);
                let mut captain = vec![];
                let mut ship = vec![];
                for el in doc.select(&stat_selector) {
                    if let Some(kind) = el.attr("data-source") {
                        match kind {
                            "rname" => {
                                name_detail.name =
                                    utils::parse_infobox_single_data_text(&el).unwrap_or_default()
                            }
                            "captain" | "extra1" => {
                                captain.extend(utils::parse_infobox_single_data_named_urls(&el))
                            }
                            "ship" => {
                                // edge case for strawhat, take 0 & 5 only, preventing unrelated <a> tag
                                let named_urls = utils::parse_infobox_single_data_named_urls(&el);
                                ship.extend(named_urls.into_iter().enumerate().filter_map(
                                    |(i, _named)| {
                                        {
                                            if i == 0 || i == 5 {
                                                Some(_named)
                                            } else {
                                                None
                                            }
                                        }
                                        .map(|named| {
                                            let fetcher = fetcher.clone();
                                            let base_url = self.base_url.clone();
                                            ship_tasks.spawn(parse_ship_detail(
                                                fetcher,
                                                named.clone(),
                                                base_url,
                                            ));
                                            named
                                        })
                                    },
                                ));
                            }
                            _ => debug!("unknown: .pi-data[data-source={}]", kind),
                        }
                    }
                }
                let pirate = Pirate::new(name_detail, pirate_url, ship, captain, pic_url);
                pirates.push(pirate);
            }
            info!("collecting ships...");
            while let Some(res) = ship_tasks.join_next().await {
                match res {
                    Ok(Ok(ship)) => ships.push(ship),
                    Ok(Err(e)) => error!("Error parsing ship detail {}", e),
                    Err(e) => error!("Error parsing ship detail {}", e),
                }
            }
        }
        pirates.sort();
        ships.sort();
        Ok((pirates, ships))
    }
}

async fn parse_ship_detail(
    fetcher: HtmlFetcher<impl FetchHtml>,
    named_ship: NamedUrl,
    base_url: String,
) -> Result<Ship, Error> {
    let html = fetcher
        .fetch(&format!("{}{}", base_url, &named_ship.get_path()))
        .await
        .map(utils::cleanup_html)?;
    let doc = Html::parse_document(&html);
    let pic_url = utils::parse_picture_url(&doc)?
        .first()
        .cloned()
        .unwrap_or_default();
    let en_name = utils::parse_main_page_title(&doc)?;
    let description = utils::parse_main_page_first_paragraph(&doc)?;
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
                _ => debug!("unknown: .pi-data[data-source={}]", kind),
            }
        }
    }
    Ok(Ship::new(
        name_detail,
        named_ship.get_path(),
        pic_url,
        affiliation,
        status,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_trait::async_trait;

    use crate::{
        category::CategoryScraper,
        fetcher::{FetchHtml, HtmlFetcher},
        pirates::scraper::PirateScraper,
        types::Error,
    };

    #[derive(Clone)]
    struct MockClient {
        res_req: HashMap<String, Result<String, Error>>,
    }

    #[async_trait]
    impl FetchHtml for MockClient {
        async fn fetch(&self, url: &str) -> Result<String, Error> {
            self.res_req
                .get(url)
                .cloned()
                .ok_or(Error::InvalidStructure(url.to_string()))
                .unwrap()
        }
    }

    fn prepare_fetcher<const N: usize>(
        arr: [(String, Result<String, Error>); N],
    ) -> HtmlFetcher<MockClient> {
        let client = MockClient {
            res_req: HashMap::from(arr),
        };
        HtmlFetcher::new(client)
    }

    #[tokio::test]
    async fn test_get() {
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
</div>"##
                    .to_string()),
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
                "/wiki/Category:New_World_Pirate_Crews".to_string(),
                Ok(r##"
<div>
    <ul>
        <li class="category-page__member">
            <a href="/wiki/New_Donquixote_Family" class="category-page__member-link" title="New Donquixote Family">New Donquixote Family</a>
        </li>
        <li class="category-page__member">
            <a href="/wiki/Rocks_Pirates" class="category-page__member-link" title="Rocks Pirates">
                Rocks Pirates
            </a>
        </li>
    </ul>
</div>
                "##.to_string())
            ),
            (
                "/wiki/Fallen_Monk_Pirates".to_string(),
                Ok(r##"
<main>
    <span class="mw-page-title-main">Fallen Monk Pirates</span>
    <div id="mw-content-text">
        <p></p>
        <aside class="portable-infobox">
            <figure class="pi-image">
                <a href="/image-path-212" class="image"/>
            </figure>
            <section>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="rname">
                    <div class="pi-data-value pi-font"><i>Hakaisō Kaizokudan</i></div>
                </div>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="captain">
                    <div class="pi-data-value pi-font"><a href="/wiki/Urouge" title="Urouge">Urouge</a></div>
                </div>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="ship">
                    <div class="pi-data-value pi-font"><a href="/wiki/Hanjomaru" title="Hanjomaru">Hanjomaru</a></div>
                </div>
            </section>
        </aside>
        <p></p>
        <p>The Fallen Monk Pirates are an infamous and notable rookie pirate crew</p>
    </div>
</main>
                "##.to_string()),
            ),
            (
                "/wiki/Rocks_Pirates".to_string(),
                Ok(r##"
<main>
    <span class="mw-page-title-main">Rocks Pirates</span>
    <div id="mw-content-text">
        <p></p>
        <aside class="portable-infobox">
            <figure class="pi-image">
                <a href="/image-path-213" class="image"/>
            </figure>
            <section>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="rname">
                    <div class="pi-data-value pi-font"><i>Rokkusu Kaizokudan</i></div>
                </div>
                <div class="pi-item pi-data pi-item-spacing pi-border-color" data-source="captain">
                    <div class="pi-data-value pi-font"><a href="/wiki/Rocks_D._Xebec" title="Rocks D. Xebec">Rocks D. Xebec</a></div>
                </div>
            </section>
        </aside>
        <p></p>
        <p>The Rocks Pirates were a legendary and powerful pirate crew that sailed the seas until their defeat at God Valley 38 years ago.</p>
    </div>
</main>
                "##.to_string()),
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
            ),]);

        let cat_crawler = CategoryScraper::new(fetcher.clone(), "");
        let scraper = PirateScraper::new(fetcher, Box::new(cat_crawler), "");
        let (pirates, ships) = scraper.scrape().await.unwrap();
        assert_eq!(pirates.len(), 2);
        assert_eq!(ships.len(), 1);
    }
}
