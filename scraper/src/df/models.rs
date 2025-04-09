use serde::Serialize;
use std::cmp::Ordering;

use super::types::{DfSubType, DfType, HasDevilFruit};
use crate::types::UrlTyped;

#[derive(Debug, Serialize, Default)]
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
pub struct DevilFruitName {
    name: String,
    en_name: String,
    description: String,
}

impl DevilFruitName {
    pub fn new(name: String, en_name: String, description: String) -> Self {
        Self {
            name,
            en_name,
            description,
        }
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

impl DevilFruit {
    pub fn zoan(
        sub_type: Option<DfSubType>,
        name_detail: DevilFruitName,
        pic_url: String,
        df_url: String,
    ) -> Self {
        Self {
            df_type: DfType::Zoan,
            df_sub_type: sub_type,
            name: name_detail.name,
            en_name: name_detail.en_name,
            description: name_detail.description,
            pic_url,
            df_url,
        }
    }

    pub fn non_zoan(
        df_type: DfType,
        name_detail: DevilFruitName,
        pic_url: String,
        df_url: String,
    ) -> Self {
        Self {
            df_type,
            df_sub_type: None,
            name: name_detail.name,
            en_name: name_detail.en_name,
            description: name_detail.description,
            pic_url,
            df_url,
        }
    }
}

impl HasDevilFruit for DevilFruit {
    fn df_type(&self) -> DfType {
        self.df_type
    }
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
        Some(self.cmp(other))
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

#[cfg(test)]
mod tests {
    use crate::{
        df::{
            models::DfTypeInfo,
            types::{DfSubType, DfType, HasDevilFruit},
        },
        types::UrlTyped,
    };

    use super::{Character, DevilFruit, DevilFruitName};

    #[test]
    fn df_has_valid_traits() {
        let dftype = DfTypeInfo {
            df_type: DfType::Logia,
            cannon_count: 10,
            non_cannon_count: 1,
            description: "logia".to_string(),
        };
        assert_eq!(
            format!("{}", dftype),
            "(df_type: Logia, cannon: 10, non-cannon: 1, description: logia)"
        );
        let df1 = DevilFruit {
            df_type: DfType::Zoan,
            df_sub_type: Some(DfSubType::MythicalZoan),
            name: "Nika".to_string(),
            en_name: "Nika".to_string(),
            description: "Used to Gomu".to_string(),
            pic_url: "pic".to_string(),
            df_url: "nika".to_string(),
        };
        let df2 = DevilFruit {
            df_type: DfType::Zoan,
            df_sub_type: Some(DfSubType::MythicalZoan),
            name: "Zeus".to_string(),
            en_name: "Zeus".to_string(),
            description: "Greek".to_string(),
            pic_url: "pic".to_string(),
            df_url: "zeus".to_string(),
        };
        let df3 = DevilFruit {
            df_type: DfType::Zoan,
            df_sub_type: Some(DfSubType::MythicalZoan),
            name: "Nika".to_string(),
            en_name: "Nika".to_string(),
            description: "Used to Gomu".to_string(),
            pic_url: "pic".to_string(),
            df_url: "nika".to_string(),
        };
        assert_ne!(df1, df2);
        assert_eq!(df1, df3);
        assert!(df1 < df2);
        assert_eq!(
            format!("{}", df1),
            format!("(df_type: {}, df_sub_type: {:?}, name: {}, english name: {}, pic: {}, url: {}, description: {})",
                df1.df_type, df1.df_sub_type, df1.name, df1.en_name, df1.pic_url, df1.df_url, df1.description
            ));
        assert_eq!(DfType::Zoan, df3.df_type());
    }

    #[test]
    fn char_has_valid_traits() {
        let character = Character {
            id: "1234".to_string(),
            name: "Foo".to_string(),
            pic_url: "pic".to_string(),
        };
        assert_eq!(character.get_path(), "/wiki/Character:1234");
    }

    #[test]
    fn df_constructor_tests() {
        let df = DevilFruit::zoan(
            Some(DfSubType::MythicalZoan),
            DevilFruitName::new(
                "fruito".to_string(),
                "fruit".to_string(),
                "lorem".to_string(),
            ),
            "".to_string(),
            "".to_string(),
        );
        assert_eq!("fruit", df.en_name);
        let df = DevilFruit::non_zoan(
            DfType::Logia,
            DevilFruitName::new(
                "ananas".to_string(),
                "pinaple".to_string(),
                "lorem".to_string(),
            ),
            "".to_string(),
            "".to_string(),
        );
        assert_eq!("ananas", df.name);
    }
}
