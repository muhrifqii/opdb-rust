use crate::types::UrlTyped;
use strum::{Display, EnumString};

#[derive(Debug, EnumString, PartialEq, Display)]
pub enum DfType {
    Logia,
    Zoan,
    Paramecia,
    Undetermined,
}

#[derive(Debug)]
pub enum DfSubType {
    AncientZoan,
    MythicalZoan,
    Artificial,
}

impl UrlTyped for DfType {
    fn get_path(&self) -> &'static str {
        match self {
            DfType::Logia => "/Logia",
            DfType::Zoan => "/Zoan",
            DfType::Paramecia => "/Paramecia",
            DfType::Undetermined => "",
        }
    }
}

impl UrlTyped for DfSubType {
    fn get_path(&self) -> &'static str {
        match self {
            DfSubType::AncientZoan => "/Zoan#Ancient_Zoan",
            DfSubType::MythicalZoan => "/Zoan#Mythical_Zoan",
            DfSubType::Artificial => "/Artificial_Devil_Fruit",
        }
    }
}
