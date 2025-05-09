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
    pub non_cannon: bool,
    url: String,
}

impl Pirate {
    pub fn new(
        name_detail: NamedJpEn,
        url: String,
        ship: Vec<NamedUrl>,
        captain: Vec<NamedUrl>,
        pic_url: String,
        non_cannon: bool,
    ) -> Self {
        Self {
            name: name_detail.name,
            en_name: name_detail.en_name,
            description: name_detail.description,
            ship,
            captain,
            pic_url,
            url,
            non_cannon,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NamedJpEn, NamedUrl};

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
            false,
        )
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
