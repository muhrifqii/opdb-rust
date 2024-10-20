use super::df_type::DfType;
use crate::types::UrlTyped;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct DevilFruit {
    pub df_type: DfType,
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
            "(df_type: {}, name: {}, english name: {}, pic: {}, url: {}, description: {})",
            self.df_type, self.name, self.en_name, self.pic_url, self.df_url, self.description,
        )
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
