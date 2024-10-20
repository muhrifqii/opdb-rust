use super::models::DfTypeInfo;
use super::{df_type::DfType, models::DevilFruit};
use crate::types::{Error, UrlTyped};
use itertools::Itertools;
use log::info;
use regex::Regex;
use scraper::selectable::Selectable;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
use std::str::FromStr as _;
use std::sync::Arc;
use strum::IntoEnumIterator;
use tokio::sync::Mutex;

pub type ArcMapHtml = Arc<Mutex<HashMap<String, String>>>;

pub trait DfScrapable {
    async fn get_dftype_info(&self) -> Result<Vec<DfTypeInfo>, Error>;
    async fn get_df_list(&self) -> Result<Vec<DfType>, Error>;
    async fn get_df(&self) -> Result<DevilFruit, Error>;
}

#[derive(Debug)]
pub struct DfScraper {
    base_url: String,
    client: reqwest::Client,
    html_cache: ArcMapHtml,
}

impl DfScraper {
    pub fn new(base_url: &str, client: reqwest::Client) -> Self {
        Self {
            base_url: base_url.to_string(),
            client,
            html_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn fetch_cached_html(&self, url: &str) -> Result<String, Error> {
        fetch_cached_html(&self.html_cache, &url, &self.client).await
    }
}

impl DfScrapable for DfScraper {
    async fn get_dftype_info(&self) -> Result<Vec<DfTypeInfo>, Error> {
        let url = format!("{}/wiki/Devil_Fruit", self.base_url);
        let response_htm = self.fetch_cached_html(&url).await?;

        let doc = Html::parse_document(&response_htm);
        let row_selector = &Selector::parse(
            "table.wikitable:nth-of-type(1) tr:nth-of-type(n+2):nth-of-type(-n+5)",
        )
        .unwrap();
        let td_selector = &Selector::parse("td").unwrap();

        let p_desc = Selector::parse("#Paramecia").unwrap();
        let z_desc = Selector::parse("#Zoan").unwrap();
        let l_desc = Selector::parse("#Logia").unwrap();
        let desc = tokio::try_join!(
            get_first_parents_sibling_text(&doc, &p_desc),
            get_first_parents_sibling_text(&doc, &z_desc),
            get_first_parents_sibling_text(&doc, &l_desc)
        );
        let (p_desc, z_desc, l_desc) = desc?;

        let df_infos: Result<Vec<_>, _> = doc
            .select(row_selector)
            .map(|row| {
                let cells = row.select(td_selector).collect_vec();
                let dft = cells[0].text().collect_vec()[0].trim();
                let cc = cells[1].text().collect_vec()[0]
                    .trim()
                    .parse::<u32>()
                    .unwrap();
                let ncc = cells[2].text().collect_vec()[0]
                    .trim()
                    .parse::<u32>()
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
        let df_type = DfType::iter()
            .filter(|df| !df.get_path().is_empty())
            .collect_vec();
        let mut tasks = Vec::with_capacity(df_type.len());
        for t in df_type {
            let client = self.client.clone();
            let cache = self.html_cache.clone();
            let base_url = self.base_url.to_string();
            tasks.push(tokio::spawn(async move {
                get_canon(t, &cache, &client, &base_url).await
            }));
        }
        for task in tasks {
            task.await
                .map_err(|e| Error::RequestError(e.to_string()))??;
        }

        // let tasks = DfType::iter()
        //     .filter(|df| !df.get_path().is_empty())
        //     .filter(|df| df == &DfType::Paramecia)
        //     .map(|df| {
        //         let url = format!("{}{}", self.base_url, df.get_path());
        //         let client = self.client.clone();
        //         tokio::spawn(async move {
        //             let res_htm = fetch_html(&url, &client).await?;
        //             let doc = Html::parse_document(&res_htm);
        //             let siblings: Result<Vec<_>, _> = doc
        //                 .select(&Selector::parse(&df.id_for_fruit_list()).unwrap())
        //                 .next()
        //                 .and_then(|e| e.parent())
        //                 .map(|n| n.next_siblings())
        //                 .ok_or(Error::InvalidStructure(String::from(
        //                     "invalid sibling node",
        //                 )))?
        //                 .filter_map(|n| {
        //                     if n.value().is_element() {
        //                         ElementRef::wrap(n)
        //                             .filter(|e| e.value().name() == "dl")
        //                             .and_then(|e| e.next_sibling())
        //                             .and_then(|e| e.next_sibling())
        //                     } else {
        //                         None
        //                     }
        //                 })
        //                 .map(|n| {
        //                     ElementRef::wrap(n)
        //                         .ok_or(Error::InvalidStructure("invalid element node".to_string()))
        //                 })
        //                 // .filter_ok(|e| e.value().name() == "ul")
        //                 .collect();

        //             for x in siblings.unwrap() {
        //                 info!("siblings {}: {:?}", df, &x.html());
        //             }

        //             Ok(vec![DevilFruit {
        //                 df_type: df,
        //                 name: String::new(),
        //                 description: String::new(),
        //                 pic_url: String::new(),
        //             }])
        //         })
        //     })
        //     .collect_vec();
        // for task in tasks {
        //     let res = task
        //         .await
        //         .map_err(|e| Error::RequestError(e.to_string()))??;
        //     // info!("res: {:?}", &res);
        // }

        Ok(Vec::new())
    }

    async fn get_df(&self) -> Result<DevilFruit, Error> {
        todo!()
    }
}

async fn fetch_html(url: &str, client: &reqwest::Client) -> Result<String, Error> {
    client
        .get(url)
        .send()
        .await
        .map_err(|r| Error::RequestError(r.to_string()))?
        .text()
        .await
        .map_err(|r| Error::RequestError(r.to_string()))
}

async fn get_first_parents_sibling_text(doc: &Html, selector: &Selector) -> Result<String, Error> {
    doc.select(selector)
        .next()
        .and_then(|e| e.parent())
        .map(|n| n.next_siblings())
        .ok_or(Error::InvalidStructure(String::from(
            "invalid sibling node",
        )))?
        .find(|n| n.value().is_element())
        .and_then(|n| ElementRef::wrap(n))
        .map(|e| e.text().join(""))
        .ok_or(Error::InvalidStructure(String::from("invalid element")))
}

async fn fetch_cached_html(
    cache: &ArcMapHtml,
    url: &str,
    client: &reqwest::Client,
) -> Result<String, Error> {
    let mut cache = cache.lock().await;
    if let Some(html) = cache.get(url) {
        Ok(html.to_string())
    } else {
        let response_htm = fetch_html(&url, &client).await?;
        cache.insert(url.to_string(), response_htm.to_string());
        Ok(response_htm)
    }
}

async fn get_canon(
    df_typpe: DfType,
    cache: &ArcMapHtml,
    client: &reqwest::Client,
    base_url: &str,
) -> Result<Vec<DevilFruit>, Error> {
    match df_typpe {
        DfType::Paramecia => get_canon_paramecia(cache, client, base_url).await,
        _ => Ok(Vec::new()),
    }
}

async fn get_canon_paramecia(
    cache: &ArcMapHtml,
    client: &reqwest::Client,
    base_url: &str,
) -> Result<Vec<DevilFruit>, Error> {
    let url = format!("{}{}", base_url, DfType::Paramecia.get_path());
    let htm = fetch_cached_html(&cache, &url, &client).await?;

    let doc = Html::parse_document(&htm);
    let fruits: Result<Vec<_>, _> = doc
        .select(&Selector::parse(&DfType::Paramecia.id_for_fruit_list()).unwrap())
        .next()
        .and_then(|e| e.parent())
        .map(|n| n.next_siblings())
        .ok_or(Error::InvalidStructure(String::from(
            "invalid sibling node",
        )))?
        .filter_map(|n| {
            if n.value().is_element() {
                ElementRef::wrap(n)
                    .filter(|e| e.value().name() == "dl")
                    .and_then(|e| e.next_sibling())
                    .and_then(|e| e.next_sibling())
            } else {
                None
            }
        })
        .map(|n| {
            ElementRef::wrap(n).ok_or(Error::InvalidStructure("invalid element node".to_string()))
        })
        .filter_ok(|e| e.value().name() == "ul")
        .take(1)
        .map_ok(|e| e.child_elements().collect_vec())
        .flatten_ok()
        .collect();
    let fruits = fruits?;
    let mut df_list = Vec::with_capacity(fruits.len());
    let rex_en_name = Regex::new(r"English version: (.+)").unwrap();
    let rex_desc = Regex::new(r"\): (.+)").unwrap();
    for el in fruits {
        let path = el
            .select(&Selector::parse("a:nth-of-type(1)").unwrap())
            .next()
            .and_then(|e| e.value().attr("href"))
            .unwrap();

        let mut en_name = "";
        let mut desc = "".to_string();
        let mut iter = el.text().into_iter();
        let name = iter.next().unwrap();

        while let Some(txt) = iter.next() {
            if rex_en_name.is_match(txt) {
                en_name = rex_en_name.captures(txt).unwrap().get(1).unwrap().as_str();
                continue;
            }
            if rex_desc.is_match(txt) {
                desc = rex_desc
                    .captures(txt)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .to_string();
                break;
            }
        }
        desc += &iter.join("").replace("\n", "").to_string();

        // info!("fruit: {:?}", &el.html());
        let df = DevilFruit {
            df_type: DfType::Paramecia,
            name: name.to_string(),
            en_name: en_name.to_string(),
            description: desc.to_string(),
            pic_url: String::new(),
            df_url: path.to_string(),
        };
        info!("fruit name: {}", &df);
        df_list.push(df);
    }

    Ok(df_list)
}
