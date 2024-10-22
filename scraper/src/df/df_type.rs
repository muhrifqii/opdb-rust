use crate::types::UrlTyped;
use serde::Serialize;
use strum::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, Copy, EnumString, PartialEq, Display, EnumIter, Serialize)]
pub enum DfType {
    Logia,
    Zoan,
    Paramecia,
    Undetermined,
}

impl DfType {
    pub fn id_for_fruit_list(&self) -> String {
        match self {
            DfType::Logia => "#Logia-Types".to_string(),
            DfType::Zoan => "#List_of_Zoan-Type_Fruits".to_string(),
            DfType::Paramecia => "#Paramecia-Type_Fruits".to_string(),
            DfType::Undetermined => String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, EnumString, PartialEq, Display, EnumIter, Serialize)]
pub enum DfSubType {
    AncientZoan,
    MythicalZoan,
}

impl DfSubType {
    pub fn id_for_fruit_list(&self) -> String {
        match self {
            DfSubType::AncientZoan => "#Ancient_Zoan".to_string(),
            DfSubType::MythicalZoan => "#Mythical_Zoan".to_string(),
        }
    }
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
            DfSubType::AncientZoan | DfSubType::MythicalZoan => "/wiki/Zoan".to_string(),
        }
    }
}
