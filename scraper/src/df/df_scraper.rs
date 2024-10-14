use super::df_type::DfType;
use super::models::DfTypeInfo;
use crate::types::Error;
use itertools::Itertools;
use log::{debug, info};
use scraper::{ElementRef, Html, Selector};
use std::str::FromStr;

pub trait DfScrapable {
    async fn get_dftype_info(&self) -> Result<Vec<DfTypeInfo>, Error>;
    async fn get_df_list(&self) -> Result<Vec<DfType>, Error>;
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
    async fn get_dftype_info(&self) -> Result<Vec<DfTypeInfo>, Error> {
        let url = format!("{}/Devil_Fruit", self.base_url);
        let response_htm = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|r| Error::RequestError(r.to_string()))?
            .text()
            .await
            .map_err(|r| Error::RequestError(r.to_string()))?;

        let doc = Html::parse_document(&response_htm);
        let row_selector = &Selector::parse(
            "table.wikitable:nth-of-type(1) tr:nth-of-type(n+2):nth-of-type(-n+5)",
        )
        .unwrap();
        let td_selector = &Selector::parse("td").unwrap();

        fn scrap_description(doc: &Html, selector: &Selector) -> Result<String, Error> {
            let desc = doc
                .select(selector)
                .next()
                .and_then(|e| e.parent())
                .map(|n| n.next_siblings())
                .ok_or(Error::InvalidStructure(String::from(
                    "invalid sibling node",
                )))?
                .find(|n| n.value().is_element())
                .and_then(|n| ElementRef::wrap(n))
                .map(|e| e.text().join(""))
                .ok_or(Error::InvalidStructure(String::from("invalid element")))?;
            Ok(desc)
        }

        let p_desc = scrap_description(&doc, &Selector::parse("#Paramecia").unwrap())?;
        let z_desc = scrap_description(&doc, &Selector::parse("#Zoan").unwrap())?;
        let l_desc = scrap_description(&doc, &Selector::parse("#Logia").unwrap())?;

        let df_infos: Result<Vec<_>, _> = doc
            .select(row_selector)
            .map(|row| {
                let cells = row.select(td_selector).collect_vec();
                let dft = cells[0].text().into_iter().collect_vec()[0].trim();
                let cc: u32 = cells[1].text().into_iter().collect_vec()[0]
                    .trim()
                    .parse::<u32>()
                    .unwrap();
                let ncc: u32 = cells[2].text().into_iter().collect_vec()[0]
                    .trim()
                    .parse()
                    .unwrap();
                let df_type = DfType::from_str(dft).unwrap();
                let desc = match df_type {
                    DfType::Paramecia => p_desc.trim().to_string(),
                    DfType::Zoan => z_desc.trim().to_string(),
                    DfType::Logia => l_desc.trim().to_string(),
                    _ => String::new(),
                };
                let obj = DfTypeInfo {
                    df_type,
                    cannon_count: cc,
                    non_cannon_count: ncc,
                    description: desc,
                };
                info!("obj: {}", &obj);
                Ok(obj)
            })
            .collect();

        df_infos
    }

    async fn get_df_list(&self) -> Result<Vec<DfType>, Error> {
        todo!()
    }
}
