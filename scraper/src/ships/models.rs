use serde::Serialize;

use crate::types::{NamedJpEn, NamedUrl, UrlTyped};

#[derive(Debug, Default, Serialize)]
pub struct Ship {
    pub name: String,
    pub en_name: String,
    pub description: String,
    pub affiliation: NamedUrl,
    pub status: String,
    pub pic_url: String,
    pub non_cannon: bool,
    url: String,
}

impl Ship {
    pub fn new(
        named_detail: NamedJpEn,
        url: String,
        pic_url: String,
        affiliation: NamedUrl,
        status: String,
        non_cannon: bool,
    ) -> Self {
        Self {
            name: named_detail.name,
            en_name: named_detail.en_name,
            description: named_detail.description,
            affiliation,
            status,
            pic_url,
            url,
            non_cannon,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NamedJpEn, NamedUrl};

    fn create_test_ship(
        name: &str,
        en_name: &str,
        description: &str,
        url: &str,
        pic_url: &str,
        affiliation_name: &str,
        affiliation_url: &str,
        status: &str,
    ) -> Ship {
        Ship::new(
            NamedJpEn::new(
                name.to_string(),
                en_name.to_string(),
                description.to_string(),
            ),
            url.to_string(),
            pic_url.to_string(),
            NamedUrl::new(affiliation_name.to_string(), affiliation_url.to_string()),
            status.to_string(),
            false,
        )
    }

    #[test]
    fn ship_has_valid_traits() {
        let ship = create_test_ship(
            "Thousand Sunny",
            "Thousand Sunny",
            "The ship of the Straw Hat Pirates",
            "ship/thousand-sunny",
            "https://example.com/thousand-sunny.jpg",
            "Straw Hat Pirates",
            "crew/straw-hat-pirates",
            "Active",
        );

        assert_eq!(ship.get_path(), "ship/thousand-sunny");
        assert_eq!(ship.name, "Thousand Sunny");
        assert_eq!(ship.en_name, "Thousand Sunny");
        assert_eq!(ship.description, "The ship of the Straw Hat Pirates");
    }

    #[test]
    fn ship_ordering() {
        let ship1 = create_test_ship(
            "Thousand Sunny",
            "Thousand Sunny",
            "The ship of the Straw Hat Pirates",
            "ship/thousand-sunny",
            "",
            "Straw Hat Pirates",
            "crew/straw-hat-pirates",
            "Active",
        );

        let ship2 = create_test_ship(
            "Going Merry",
            "Going Merry",
            "The first ship of the Straw Hat Pirates",
            "ship/going-merry",
            "",
            "Straw Hat Pirates",
            "crew/straw-hat-pirates",
            "Inactive",
        );

        assert!(ship1 > ship2);
        assert_ne!(ship1, ship2);
    }
}
