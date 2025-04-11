use itertools::Itertools;
use log::{error, info};
use reqwest::Client;
use scraper::selectable::Selectable;
use scraper::Html;
use std::collections::HashMap;
use std::str::FromStr as _;
use strum::IntoEnumIterator;

use super::models::{DevilFruit, DfTypeInfo};
use crate::df::parser::get_parser;
use crate::df::types::DfType;
use crate::fetcher::{FetchHtml, HtmlFetcher};
use crate::types::{Error, UrlTyped};
use crate::utils;

pub trait DfScrapable {
    async fn get_dftype_info(&self) -> Result<Vec<DfTypeInfo>, Error>;
    async fn get_df_list(&self) -> Result<Vec<DevilFruit>, Error>;
}

#[derive(Debug)]
pub struct DfScraper<T = Client>
where
    T: FetchHtml + Clone,
{
    fetcher: HtmlFetcher<T>,
    base_url: String,
}

impl<T: FetchHtml + Clone> DfScraper<T> {
    pub fn new(fetcher: HtmlFetcher<T>, base_url: &str) -> Self {
        Self {
            fetcher,
            base_url: base_url.to_string(),
        }
    }
}

impl<T: FetchHtml + Clone + std::marker::Send + std::marker::Sync + 'static> DfScrapable
    for DfScraper<T>
{
    async fn get_dftype_info(&self) -> Result<Vec<DfTypeInfo>, Error> {
        let url = format!("{}/wiki/Devil_Fruit", self.base_url);
        let html = self.fetcher.fetch(&url).await?;
        let doc = Html::parse_document(&html);

        let desc = tokio::try_join!(
            utils::get_first_parents_sibling_text(&doc, "#Paramecia"),
            utils::get_first_parents_sibling_text(&doc, "#Zoan"),
            utils::get_first_parents_sibling_text(&doc, "#Logia")
        );
        let (p_desc, z_desc, l_desc) = desc?;

        let row_selector = utils::parse_selector(
            "table.wikitable:nth-of-type(1) tr:nth-of-type(n+2):nth-of-type(-n+5)",
        )?;
        let td_selector = utils::parse_selector("td")?;

        doc.select(&row_selector)
            .map(|row| {
                let cells = row.select(&td_selector).collect_vec();
                if cells.len() < 3 {
                    let msg = format!("Expected at least 3 cells, found {}", cells.len());
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
                let obj = DfTypeInfo::new(df_type, cc, ncc, desc.to_string());
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
                    let pic_url = utils::parse_picture_url(&doc)?;
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_trait::async_trait;

    use crate::{
        df::scraper::{DfScrapable, DfScraper},
        fetcher::{FetchHtml, HtmlFetcher},
        types::Error,
    };

    #[derive(Clone)]
    struct MockClient {
        res_req: HashMap<String, Result<String, Error>>,
    }

    #[async_trait]
    impl FetchHtml for MockClient {
        async fn fetch(&self, url: &str) -> Result<String, Error> {
            self.res_req.get(url).cloned().unwrap()
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
    async fn get_type_info() {
        let fetcher = prepare_fetcher([(
            "/wiki/Devil_Fruit".to_string(),
            Ok(r#"<html><body>
                <h4><span id="Paramecia">Paramecia</span></h4>
                <p>Paramecia Text</p>
                <h4><span id="Zoan">Zoan</span></h4>
                <p>Zoan Text</p>
                <h4><span id="Logia">Logia</span></h4>
                <p>Logia Text</p>
                <table class="wikitable">
                    <tbody>
                    <tr><th></th><th>Canon</th><th>Non-Canon</th><th>Total</th></tr>
                    <tr><td>Paramecia</td><td>94 </td><td>48</td><td>142</td></tr>
                    <tr><td>Zoan</td><td>55</td><td>7 </td><td> 62</td></tr>
                    <tr><td>Logia</td><td>13</td><td>3</td><td>16  </td></tr>
                    <tr><td>Undetermined</td><td>3</td><td>2</td><td>5</td></tr>
                    <tr><td>Last</td><td></td><td></td><td></td></tr>
                    </tbody></table>
            </body></html>"#
                .to_string()),
        )]);
        let scrape = DfScraper::new(fetcher, "");
        let result = scrape.get_dftype_info().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 4);
    }

    #[tokio::test]
    async fn get_df_list() {
        let fetcher = prepare_fetcher([
            (
                "/wiki/Logia".to_string(),
                Ok(r#"<html><body>
                <h4><span id="Logia-Types">Logia</span></h4>
                <dl></dl>
                <h3>Some Text</h3>
                <dl></dl>
                <ul>
                    <li><a href="/dfpath_logia" title="Some Logia Df">Some Logia Devil Fruit</a> (<i>English versions: Some En Logia Df</i>): Some description. Eaten by <a href="/character">Df User</a>.</li>
                </ul>
                </body></html>"#
                    .to_string()),
            ),
            (
                "/wiki/Paramecia".to_string(),
                Ok(r#"<html><body>
                <h4><span id="Paramecia-Type_Fruits">Paramecia</span></h4>
                <dl></dl>
                <h3>Some Text</h3>
                <dl></dl>
                <ul>
                    <li><a href="/dfpath_paramecia" title="Some Paramecia Df">Some Paramecia Devil Fruit</a> (<i>English versions: Some En Paramecia Df</i>): Some description. Eaten by <a href="/character">Df User</a>.</li>
                </ul>
                </body></html>"#
                    .to_string()),
            ),
            (
                "/wiki/Zoan".to_string(),
                Ok(r#"<html><body>
                <h4><span id="Ancient_Zoan">Ancient Zoan</span></h4>
                <dl></dl>
                <p>wow</p>
                <ul>
                    <li><a href="/dfpath-ancient-zoan" title="Some Ancient Zoan">Some ancient zoan</a>: Eaten by <a href="/characterx" title="X">X</a>.</li>
                </ul>
                <h4><span id="Mythical_Zoan">Mythical Zoan</span></h4>
                <dl></dl>
                <p>wow</p>
                <ul>
                    <li><a href="/dfpath-mythical-zoan" title="Some Mythical Zoan">Some mythical zoan</a>: Eaten by <a href="/characterx" title="X">X</a>.</li>
                </ul>
                <h4><span id="List_of_Zoan-Type_Fruits">Zoan</span></h4>
                <h3>Some Text</h3>
                <dl></dl>
                <ul>
                    <li><a href="/dfpath-zoan" title="Some Zoan Df">Some Zoan Devil Fruit</a> (<i>English versions: Some En Zoan Df</i>): Some description. Eaten by <a href="/character">Df User</a>.</li>
                    <li><a href="/dfpath-ancient-zoan" title="Some Ancient Zoan">Some ancient zoan</a> (<i>English versions: Some En Ancient Zoan Df</i>): Some description. Eaten by <a href="/characterx">X</a>.</li>
                </ul>
                </body></html>"#
                    .to_string()),
            ),
            (
                "/dfpath-zoan".to_string(),
                Ok(r#"<html><body>
                <aside><figure class="pi-image piapia"><a href="/picurl" class="image"><img src="/picurlsrc"/></a></figure></aside>
                </body></html>"#
                    .to_string()),
            ),
        ]);
        let scrape = DfScraper::new(fetcher, "");
        let df_list = scrape.get_df_list().await.unwrap();
        assert_eq!(df_list.len(), 4);
    }
}
