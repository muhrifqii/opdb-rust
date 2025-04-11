use serde::Serialize;

use crate::types::{NamedJpEn, NamedUrl, UrlTyped};

#[derive(Debug, Serialize)]
pub struct Pirate {
    pub name: String,
    pub en_name: String,
    pub description: String,
    pub ship: Vec<NamedUrl>,
    pub captain: Vec<NamedUrl>,
    pub pic_url: String,
    url: String,
}

impl Pirate {
    pub fn new(
        name_detail: NamedJpEn,
        url: String,
        ship: Vec<NamedUrl>,
        captain: Vec<NamedUrl>,
        pic_url: String,
    ) -> Self {
        Self {
            name: name_detail.name,
            en_name: name_detail.en_name,
            description: name_detail.description,
            ship,
            captain,
            pic_url,
            url,
        }
    }
}

impl UrlTyped for Pirate {
    fn get_path(&self) -> String {
        self.url.clone()
    }
}

impl Eq for Pirate {}

impl PartialEq for Pirate {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Ord for Pirate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.url.cmp(&other.url)
    }
}

impl PartialOrd for Pirate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Ship {
    pub name: String,
    pub en_name: String,
    pub description: String,
    pub affiliation: NamedUrl,
    pub status: String,
    pub pic_url: String,
    url: String,
}

impl Ship {
    pub fn new(
        named_detail: NamedJpEn,
        url: String,
        pic_url: String,
        affiliation: NamedUrl,
        status: String,
    ) -> Self {
        Self {
            name: named_detail.name,
            en_name: named_detail.en_name,
            description: named_detail.description,
            affiliation,
            status,
            pic_url,
            url,
        }
    }
}

impl UrlTyped for Ship {
    fn get_path(&self) -> String {
        self.url.clone()
    }
}

impl Eq for Ship {}

impl PartialEq for Ship {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Ord for Ship {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.url.cmp(&other.url)
    }
}

impl PartialOrd for Ship {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
