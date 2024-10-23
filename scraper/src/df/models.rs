use std::cmp::Ordering;

use serde::Serialize;

use super::df_type::{DfSubType, DfType};
use crate::types::UrlTyped;

#[derive(Debug, Serialize)]
pub struct DfTypeInfo {
    pub df_type: DfType,
    pub cannon_count: u32,
    pub non_cannon_count: u32,
    pub description: String,
}

impl std::fmt::Display for DfTypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "(df_type: {}, cannon: {}, non-cannon: {}, description: {})",
            self.df_type, self.cannon_count, self.non_cannon_count, self.description
        )
    }
}

#[derive(Debug, Serialize)]
pub struct DevilFruit {
    pub df_type: DfType,
    pub df_sub_type: Option<DfSubType>,
    pub name: String,
    pub en_name: String,
    pub description: String,
    pub pic_url: String,
    pub df_url: String,
}

impl std::fmt::Display for DevilFruit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "(df_type: {}, df_sub_type: {:?}, name: {}, english name: {}, pic: {}, url: {}, description: {})",
            self.df_type, self.df_sub_type, self.name, self.en_name, self.pic_url, self.df_url, self.description,
        )
    }
}

impl Ord for DevilFruit {
    fn cmp(&self, other: &Self) -> Ordering {
        self.df_url.cmp(&other.df_url)
    }
}

impl PartialOrd for DevilFruit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.df_url.partial_cmp(&other.df_url)
    }
}

impl Eq for DevilFruit {}

impl PartialEq for DevilFruit {
    fn eq(&self, other: &Self) -> bool {
        self.df_type == other.df_type
            && self.df_sub_type == other.df_sub_type
            && self.name == other.name
            && self.en_name == other.en_name
            && self.description == other.description
            && self.df_url == other.df_url
    }
}

#[derive(Debug)]
pub struct DfUser {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub pic_url: String,
}

impl UrlTyped for Character {
    fn get_path(&self) -> String {
        format!("/wiki/Character:{}", self.id)
    }
}
