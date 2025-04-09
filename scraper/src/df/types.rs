use crate::types::UrlTyped;
use serde::Serialize;
use strum::{Display, EnumIter, EnumString};

#[derive(
    Debug,
    Clone,
    Copy,
    EnumString,
    PartialEq,
    Display,
    EnumIter,
    Serialize,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub enum DfType {
    Logia,
    Zoan,
    Paramecia,
    #[default]
    Undetermined,
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

#[derive(
    Debug, Clone, Copy, EnumString, PartialEq, Display, EnumIter, Serialize, Eq, PartialOrd, Ord,
)]
pub enum DfSubType {
    AncientZoan,
    MythicalZoan,
}

impl UrlTyped for DfSubType {
    fn get_path(&self) -> String {
        match self {
            DfSubType::AncientZoan | DfSubType::MythicalZoan => "/wiki/Zoan".to_string(),
        }
    }
}

impl DfSubType {
    pub fn id_for_fruit_list(&self) -> String {
        match self {
            DfSubType::AncientZoan => "#Ancient_Zoan".to_string(),
            DfSubType::MythicalZoan => "#Mythical_Zoan".to_string(),
        }
    }
}

pub trait HasDevilFruit {
    fn df_type(&self) -> DfType;
}

#[cfg(test)]
mod tests {
    use crate::{df::types::DfSubType, types::UrlTyped};

    use super::DfType;

    #[test]
    fn dftype_has_valid_impl() {
        let dft = DfType::Zoan;
        assert_eq!(dft.get_path(), "/wiki/Zoan");
        assert_eq!(dft.id_for_fruit_list(), "#List_of_Zoan-Type_Fruits");
        let dft = DfType::Logia;
        assert_eq!(dft.get_path(), "/wiki/Logia");
        assert_eq!(dft.id_for_fruit_list(), "#Logia-Types");
        let dft = DfType::Paramecia;
        assert_eq!(dft.get_path(), "/wiki/Paramecia");
        assert_eq!(dft.id_for_fruit_list(), "#Paramecia-Type_Fruits");
        let dft = DfType::Undetermined;
        assert_eq!(dft.get_path(), "");
        assert_eq!(dft.id_for_fruit_list(), "");

        let dfsub = DfSubType::AncientZoan;
        assert_eq!(dfsub.get_path(), "/wiki/Zoan");
        assert_eq!(dfsub.id_for_fruit_list(), "#Ancient_Zoan");
        let dfsub = DfSubType::MythicalZoan;
        assert_eq!(dfsub.get_path(), "/wiki/Zoan");
        assert_eq!(dfsub.id_for_fruit_list(), "#Mythical_Zoan");
    }
}
