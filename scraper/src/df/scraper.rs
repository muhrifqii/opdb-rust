use itertools::Itertools;
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use scraper::selectable::Selectable;
use scraper::Html;
use std::collections::HashMap;
use std::str::FromStr as _;
use strum::IntoEnumIterator;

use super::models::{DevilFruit, DfTypeInfo};
use crate::df::parser::{get_parser, Utils};
use crate::df::types::DfType;
use crate::fetcher::HtmlFetcher;
use crate::types::{Error, UrlTyped};

pub trait DfScrapable {
    async fn get_dftype_info(&self) -> Result<Vec<DfTypeInfo>, Error>;
    async fn get_df_list(&self) -> Result<Vec<DevilFruit>, Error>;
}

lazy_static! {
    static ref REX_EN_NAME: Regex = Regex::new(r"English version: (.+)").unwrap();
}

#[derive(Debug)]
pub struct DfScraper {
    fetcher: HtmlFetcher,
    base_url: String,
}

impl DfScraper {
    pub fn new(fetcher: HtmlFetcher, base_url: &str) -> Self {
        Self {
            fetcher,
            base_url: base_url.to_string(),
        }
    }
}

impl DfScrapable for DfScraper {
    async fn get_dftype_info(&self) -> Result<Vec<DfTypeInfo>, Error> {
        let url = format!("{}/wiki/Devil_Fruit", self.base_url);
        let html = self.fetcher.fetch(&url).await?;
        let doc = Html::parse_document(&html);

        let desc = tokio::try_join!(
            Utils::get_first_parents_sibling_text(&doc, "#Paramecia"),
            Utils::get_first_parents_sibling_text(&doc, "#Zoan"),
            Utils::get_first_parents_sibling_text(&doc, "#Logia")
        );
        let (p_desc, z_desc, l_desc) = desc?;

        let row_selector = Utils::parse_selector(
            "table.wikitable:nth-of-type(1) tr:nth-of-type(n+2):nth-of-type(-n+5)",
        )?;
        let td_selector = Utils::parse_selector("td")?;

        doc.select(&row_selector)
            .map(|row| {
                let cells = row.select(&td_selector).collect_vec();
                if cells.len() < 3 {
                    let msg = format!(
                        "Expected at least 3 cells, found {}: {:?}",
                        cells.len(),
                        row.html()
                    );
                    return Err(Error::InvalidStructure(msg));
                }

                let dft = cells[0].text().collect_vec()[0].trim();
                let cc = cells[1].text().collect_vec()[0]
                    .trim()
                    .parse::<u32>()
                    .unwrap();
                let ncc = cells[2].text().collect_vec()[0]
                    .trim()
                    .parse::<u32>()
                    .unwrap();
                let df_type = DfType::from_str(dft).map_err(|_| {
                    Error::InvalidStructure(format!("Unknown DfType '{}': {:?}", dft, row.html()))
                })?;
                let desc = match df_type {
                    DfType::Paramecia => p_desc.trim(),
                    DfType::Zoan => z_desc.trim(),
                    DfType::Logia => l_desc.trim(),
                    _ => "",
                };
                let obj = DfTypeInfo {
                    df_type,
                    cannon_count: cc,
                    non_cannon_count: ncc,
                    description: desc.to_string(),
                };
                info!("obj: {}", &obj);
                Ok(obj)
            })
            .collect()
    }

    async fn get_df_list(&self) -> Result<Vec<DevilFruit>, Error> {
        let mut pic_tasks = tokio::task::JoinSet::new();
        let mut devil_fruits_map = HashMap::new();
        info!("collecting df...");
        // Step 1: For each DfType (Paramecia, Zoan, Logia)
        for df_type in DfType::iter().filter(|t| !t.get_path().is_empty()) {
            let url = format!("{}{}", &self.base_url, df_type.get_path());
            let html = self.fetcher.fetch(&url).await?;
            let doc = Html::parse_document(&html);

            let df_list = get_parser(&df_type, true).parse(&doc)?;

            // Step 2: Store each DevilFruit and prepare to fetch their pictures
            for df in df_list {
                let df_url = format!("{}{}", &self.base_url, &df.df_url);
                devil_fruits_map.insert(df_url.clone(), df);

                let fetcher = self.fetcher.clone();

                pic_tasks.spawn(async move {
                    let html = fetcher.fetch_only(&df_url).await?;
                    let doc = Html::parse_document(&html);
                    let pic_url = Utils::parse_picture_url(&doc)?;
                    let pic = pic_url.first().cloned().unwrap_or_default();

                    Ok::<(String, String), Error>((df_url, pic))
                });
            }
        }
        // Step 3: Await all picture tasks
        info!("collecting df pictures...");
        while let Some(res) = pic_tasks.join_next().await {
            match res {
                Ok(Ok((url, pic_url))) => {
                    if let Some(df) = devil_fruits_map.get_mut(&url) {
                        df.pic_url = pic_url;
                    }
                }
                Ok(Err(e)) => error!("Error parsing picture {}", e),
                Err(e) => error!("Error parsing picture {}", e),
            }
        }

        Ok(devil_fruits_map.into_values().sorted().collect_vec())
    }
}
