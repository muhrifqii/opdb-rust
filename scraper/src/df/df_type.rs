use crate::types::UrlTyped;
use strum::{Display, EnumIter, EnumString};

#[derive(Debug, EnumString, PartialEq, Display, EnumIter)]
pub enum DfType {
    Logia,
    Zoan,
    Paramecia,
    Undetermined,
}

#[derive(Debug, EnumString, PartialEq, Display, EnumIter)]
pub enum DfSubType {
    AncientZoan,
    MythicalZoan,
    Artificial,
}

impl UrlTyped for DfType {
    fn get_path(&self) -> String {
        match self {
            DfType::Logia => "/wiki/Logia".to_string(),
            DfType::Zoan => "/wiki/Zoan".to_string(),
            DfType::Paramecia => "/wiki/Paramecia".to_string(),
            DfType::Undetermined => String::new(),
        }
    }
}

impl UrlTyped for DfSubType {
    fn get_path(&self) -> String {
        match self {
            DfSubType::AncientZoan => "/wiki/Zoan#Ancient_Zoan".to_string(),
            DfSubType::MythicalZoan => "/wiki/Zoan#Mythical_Zoan".to_string(),
            DfSubType::Artificial => "/wiki/Artificial_Devil_Fruit".to_string(),
        }
    }
}
