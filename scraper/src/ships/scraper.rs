use std::sync::Arc;

use log::info;
use scraper::Html;

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

        let ships = vec![];
        Ok(ships)
    }
}

async fn parse_ship_detail(
    fetcher: HtmlFetcher,
    ship_url: String,
    base_url: String,
) -> Result<Ship, Error> {
    let html = fetcher
        .fetch(&format!("{}{}", base_url, &ship_url))
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
    ))
}
