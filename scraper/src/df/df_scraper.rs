use super::df_type::DfType;
use super::models::DfTypeInfo;
use itertools::Itertools;
use log::{debug, info};
use scraper::{ElementRef, Html, Selector};
use std::str::FromStr;

pub trait DfScrapable {
    async fn get_dftype_info(&self) -> reqwest::Result<Vec<DfTypeInfo>>;
}

#[derive(Debug)]
pub struct DfScraper {
    base_url: String,
    client: reqwest::Client,
}

impl DfScraper {
    pub fn new(base_url: String, client: reqwest::Client) -> Self {
        Self { base_url, client }
    }
}

impl DfScrapable for DfScraper {
    async fn get_dftype_info(&self) -> reqwest::Result<Vec<DfTypeInfo>> {
        let url = format!("{}/Devil_Fruit", self.base_url);
        let response_htm = self.client.get(url).send().await?.text().await?;

        let doc = Html::parse_document(&response_htm);
        let row_selector = &Selector::parse(
            "table.wikitable:nth-of-type(1) tr:nth-of-type(n+2):nth-of-type(-n+5)",
        )
        .unwrap();
        let td_selector = &Selector::parse("td").unwrap();

        let df_infos = doc
            .select(row_selector)
            .map(|row| {
                let cells = row.select(td_selector).collect_vec();
                let dft = cells[0].text().into_iter().collect_vec()[0].trim();
                let cc: u32 = cells[1].text().into_iter().collect_vec()[0]
                    .trim()
                    .parse()
                    .unwrap();
                let ncc: u32 = cells[2].text().into_iter().collect_vec()[0]
                    .trim()
                    .parse()
                    .unwrap();
                let obj = DfTypeInfo {
                    df_type: DfType::from_str(dft).unwrap(),
                    cannon_count: cc,
                    non_cannon_count: ncc,
                };
                info!("obj: {}", &obj);
                obj
            })
            .collect_vec();

        Ok(df_infos)
    }
}
