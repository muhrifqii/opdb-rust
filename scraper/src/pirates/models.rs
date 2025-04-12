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
        )
    }

    fn create_test_pirate(
        name: &str,
        en_name: &str,
        description: &str,
        url: &str,
        pic_url: &str,
        ship_name: &str,
        ship_url: &str,
        captain_name: &str,
        captain_url: &str,
    ) -> Pirate {
        Pirate::new(
            NamedJpEn::new(
                name.to_string(),
                en_name.to_string(),
                description.to_string(),
            ),
            url.to_string(),
            vec![NamedUrl::new(ship_name.to_string(), ship_url.to_string())],
            vec![NamedUrl::new(
                captain_name.to_string(),
                captain_url.to_string(),
            )],
            pic_url.to_string(),
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

    #[test]
    fn pirate_has_valid_traits() {
        let pirate = create_test_pirate(
            "Akagami Kaizokudan",
            "Red-Haired Pirates",
            "A powerful pirate crew led by Shanks",
            "pirate/red-haired-pirates",
            "https://example.com/red-haired-pirates.jpg",
            "Red Force",
            "ship/red-force",
            "Shanks",
            "pirate/shanks",
        );

        assert_eq!(pirate.get_path(), "pirate/red-haired-pirates");
        assert_eq!(pirate.name, "Akagami Kaizokudan");
        assert_eq!(pirate.en_name, "Red-Haired Pirates");
        assert_eq!(pirate.description, "A powerful pirate crew led by Shanks");
    }

    #[test]
    fn pirate_ordering() {
        let pirate1 = create_test_pirate(
            "Akagami Kaizokudan",
            "Red-Haired Pirates",
            "A powerful pirate crew led by Shanks",
            "pirate/red-haired-pirates",
            "",
            "Red Force",
            "ship/red-force",
            "Shanks",
            "pirate/shanks",
        );

        let pirate2 = create_test_pirate(
            "Mugiwara Kaizokudan",
            "Straw Hat Pirates",
            "A pirate crew led by Monkey D. Luffy",
            "pirate/straw-hat-pirates",
            "",
            "Thousand Sunny",
            "ship/thousand-sunny",
            "Monkey D. Luffy",
            "pirate/monkey-d-luffy",
        );

        assert!(pirate2 > pirate1);
        assert_ne!(pirate1, pirate2);
    }
}
