use itertools::Itertools as _;
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use scraper::{ElementRef, Html};
use std::collections::HashMap;
use strum::IntoEnumIterator as _;

use crate::{
    types::{Error, NamedJpEn},
    utils,
};

use super::{
    models::DevilFruit,
    types::{DfSubType, DfType},
};

lazy_static! {
    static ref REX_EN_NAME: Regex = Regex::new(r"English version: (.+)").unwrap();
    static ref REX_DESCRIPTION_ZOAN: Regex = Regex::new(r"\) \- (.+)").unwrap();
}

pub trait DfTypeParser {
    fn parse(&self, html: &Html) -> Result<Vec<DevilFruit>, Error>;
}

pub struct CanonZoanParser;
pub struct CanonParameciaParser;
pub struct CanonLogiaParser;

// traverse h3 "Canon" before h3 "Non-Canon"
impl DfTypeParser for CanonZoanParser {
    fn parse(&self, html: &Html) -> Result<Vec<DevilFruit>, Error> {
        let sibling_iter = html
            .select(&utils::parse_selector(&DfType::Zoan.id_for_fruit_list())?)
            .next()
            .and_then(|e| e.parent())
            .map(|n| n.next_siblings())
            .ok_or(Error::InvalidStructure(String::from(
                "invalid sibling node",
            )))?
            .filter_map(|n| {
                if n.value().is_element() {
                    ElementRef::wrap(n)
                } else {
                    None
                }
            });
        let fruits: Vec<ElementRef> = sibling_iter
            .take_while(|el| {
                !(el.value().name() == "h3"
                    && el
                        .first_child()
                        .and_then(|n| ElementRef::wrap(n))
                        .and_then(|el| el.value().id())
                        .is_some_and(|s| s != "Canon"))
            })
            .filter(|el| el.value().name() == "ul")
            .flat_map(|el| el.child_elements().collect_vec())
            .collect();
        let df_sub_map = Utils::parse_sub_type(html)?;
        // info!("df_sub_map content: {:?}", df_sub_map);
        let df_list: Vec<_> = fruits
            .iter()
            .map(|el| {
                let path = utils::extract_href(el, "a:nth-of-type(1)")?;

                let name_detail = Utils::parse_df_name(el, &REX_EN_NAME, &REX_DESCRIPTION_ZOAN);
                let sub_type = df_sub_map.get(&path);
                // info!("fruit: {:?}", &el.html());
                let df = DevilFruit::zoan(sub_type.copied(), name_detail, String::new(), path);
                // info!("fruit name: {}", &df);
                Ok(df)
            })
            .collect::<Result<_, _>>()?;

        info!("total Zoan: {}", df_list.len());

        Ok(df_list)
    }
}

// first ul after dl
macro_rules! impl_canon_paramecia_logia_parser {
    ($T:ident, $df_type:expr) => {
        impl DfTypeParser for $T {
            fn parse(&self, html: &Html) -> Result<Vec<DevilFruit>, Error> {
                let fruits: Result<Vec<_>, _> = html
                    .select(&utils::parse_selector(&$df_type.id_for_fruit_list())?)
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
                        ElementRef::wrap(n)
                            .ok_or(Error::InvalidStructure("invalid element node".to_string()))
                    })
                    .filter_ok(|e| e.value().name() == "ul")
                    .take(1)
                    .map_ok(|e| e.child_elements().collect_vec())
                    .flatten_ok()
                    .collect();
                let fruits = fruits?;
                let rex_desc = Regex::new(r"\): (.+)").unwrap();
                let rex_en_name = Regex::new(r"English version: (.+)").unwrap();
                let df_list: Vec<_> = fruits
                    .iter()
                    .map(|el| {
                        let path = utils::extract_href(el, &"a:nth-of-type(1)")?;
                        let name_detail = Utils::parse_df_name(el, &rex_en_name, &rex_desc);
                        let df = DevilFruit::non_zoan($df_type, name_detail, String::new(), path);

                        Ok(df)
                    })
                    .collect::<Result<_, _>>()?;

                info!("total {}: {}", $df_type, df_list.len());

                Ok(df_list)
            }
        }
    };
}

impl_canon_paramecia_logia_parser!(CanonParameciaParser, DfType::Paramecia);
impl_canon_paramecia_logia_parser!(CanonLogiaParser, DfType::Logia);

pub fn get_parser(df_type: &DfType, canon: bool) -> Box<dyn DfTypeParser> {
    match (df_type, canon) {
        (DfType::Zoan, true) => Box::new(CanonZoanParser),
        (DfType::Logia, true) => Box::new(CanonLogiaParser),
        (DfType::Paramecia, true) => Box::new(CanonParameciaParser),
        _ => unimplemented!(),
    }
}

pub struct Utils;

impl Utils {
    fn parse_df_name(el: &ElementRef, rex_en_name: &Regex, rex_desc: &Regex) -> NamedJpEn {
        let mut en_name = String::new();
        let mut description = String::new();
        let mut iter = el.text().into_iter();
        let name = iter.next().unwrap_or_default().to_string();

        while let Some(txt) = iter.next() {
            if rex_en_name.is_match(txt) {
                en_name = rex_en_name
                    .captures(txt)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .to_string();
                continue;
            }
            if rex_desc.is_match(txt) {
                description = rex_desc
                    .captures(txt)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .to_string();
                break;
            }
        }
        description += &iter.join("").replace("\n", "").to_string();
        NamedJpEn::new(name, en_name, description)
    }

    fn parse_sub_type(html_doc: &Html) -> Result<HashMap<String, DfSubType>, Error> {
        let mut sub_type_map = HashMap::new();

        for df_sub in DfSubType::iter() {
            let sub_type_selector = &utils::parse_selector(&df_sub.id_for_fruit_list())?;
            let res: Result<(), Error> = html_doc
                .select(sub_type_selector)
                .next()
                .and_then(|e| e.parent())
                .map(|n| n.next_siblings())
                .ok_or(Error::InvalidStructure(String::from(
                    "invalid sibling node",
                )))?
                .filter_map(|n| {
                    if n.value().is_element() {
                        ElementRef::wrap(n)
                    } else {
                        None
                    }
                })
                .filter(|e| e.value().name() == "ul")
                .take(1)
                .flat_map(|e| e.child_elements().collect_vec())
                .try_for_each(|e| {
                    let path = utils::extract_href(&e, "a:nth-of-type(1)")?;
                    sub_type_map.insert(path, df_sub);
                    Ok(())
                });
            res?;
        }

        Ok(sub_type_map)
    }
}

#[cfg(test)]
mod tests {
    use crate::df::types::DfType;

    use super::get_parser;

    #[test]
    #[should_panic]
    fn test_get_parser() {
        get_parser(&DfType::Undetermined, true);
    }
}
