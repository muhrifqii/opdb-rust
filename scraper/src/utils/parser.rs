use itertools::Itertools as _;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use crate::types::{Error, NamedUrl};

lazy_static! {
    static ref REX_SUP: Regex = Regex::new(r"<sup.*?>.*?</sup>").unwrap();
}

pub(crate) fn cleanup_html(html_source: String) -> String {
    REX_SUP.replace_all(&html_source, "").to_string()
}

pub(crate) fn parse_selector(selector: &str) -> Result<Selector, Error> {
    Selector::parse(selector)
        .map_err(|_| Error::InvalidStructure(format!("Invalid selector: {}", selector)))
}

pub(crate) fn extract_href(el: &ElementRef, selector: &str) -> Result<String, Error> {
    el.select(&parse_selector(selector)?)
        .next()
        .and_then(|e| e.value().attr("href"))
        .map(String::from)
        .ok_or(Error::InvalidStructure(format!(
            "Missing href attribute. found: {}",
            el.html().as_str()
        )))
}

pub(crate) fn extract_all_href(el: &ElementRef, selector: &str) -> Result<Vec<String>, Error> {
    Ok(el
        .select(&parse_selector(selector)?)
        .filter_map(|e| e.value().attr("href"))
        .map(String::from)
        .collect())
}

pub(crate) async fn get_first_parents_sibling_text(
    html_doc: &Html,
    selector: &str,
) -> Result<String, Error> {
    html_doc
        .select(&parse_selector(selector)?)
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

pub(crate) fn parse_picture_url(html_doc: &Html) -> Result<Vec<String>, Error> {
    let selector = parse_selector("aside.portable-infobox figure.pi-image>a.image")?;
    Ok(html_doc
        .select(&selector)
        .filter_map(|e| e.value().attr("href"))
        .filter_map(|s| s.split("?cb=").next())
        .map(String::from)
        .collect_vec())
}

pub(crate) fn parse_main_page_title(html_doc: &Html) -> Result<String, Error> {
    html_doc
        .select(&parse_selector(".mw-page-title-main")?)
        .next()
        .map(|e| e.text())
        .and_then(|mut t| t.next())
        .map(String::from)
        .ok_or(Error::InvalidStructure(format!(
            "invalid title page element",
        )))
}

/// need to manually clean the HTML from sup first
pub(crate) fn parse_main_page_first_paragraph(html_doc: &Html) -> Result<String, Error> {
    html_doc
        .select(&parse_selector("main #mw-content-text p:nth-of-type(3)")?)
        .next()
        .map(|e| e.text().join("").replace("\n", ""))
        .ok_or(Error::InvalidStructure(format!(
            "invalid first paragraph element"
        )))
}

pub(crate) fn parse_infobox_single_data_text(el: &ElementRef) -> Option<String> {
    el.child_elements().last().map(|e| e.text().join(""))
}

pub(crate) fn parse_infobox_single_data_named_urls(el: &ElementRef) -> Vec<NamedUrl> {
    el.select(&parse_selector("a").unwrap())
        .filter_map(|a| {
            let name = a.text().collect::<String>().trim().to_string();
            a.value()
                .attr("href")
                .map(String::from)
                .map(|url| NamedUrl::new(name, url))
        })
        .collect()
}

pub(crate) fn parse_is_non_cannon(html_doc: &Html) -> Result<bool, Error> {
    let selector = parse_selector("page-header__categories a")?;
    let non_cannon = html_doc
        .select(&selector)
        .filter_map(|e| e.value().attr("href"))
        .any(|url| url.contains("Category:Non-Canon"));
    Ok(non_cannon)
}
