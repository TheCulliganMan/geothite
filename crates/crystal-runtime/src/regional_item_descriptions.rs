//! Contextual regional help without changing original item catalogs.
use crate::RuntimeShellSnapshot;

impl RuntimeShellSnapshot {
    /// Returns the item's displayed help for the current map.
    ///
    /// Minnesota items describe their local uses. Other regions and items
    /// retain the original catalog text. Returns `None` for an unknown item.
    pub fn item_description(&self, item_id: &str) -> Option<&str> {
        let item = self.items.iter().find(|item| item.item_id == item_id)?;
        Some(description(
            &self.overworld.map_name,
            item_id,
            &item.description,
        ))
    }
}

fn description<'a>(map: &str, item: &str, original: &'a str) -> &'a str {
    if map.starts_with("Mn") {
        match item {
            "CARD_KEY" => return "Opens ICE holding\nroom doors.",
            "MACHINE_PART" => return "A part for the\nMARAIS lighthouse.",
            "EXP_SHARE" => return "Full bench EXP.\nwhen held.",
            _ => {}
        }
    }
    original
}

#[cfg(test)]
mod tests {
    use super::description;

    #[test]
    fn regional_help_preserves_originals_outside_minnesota_and_for_unrelated_items() {
        for map in ["RadioTower1F", "PowerPlant", "Route29", ""] {
            for item in ["CARD_KEY", "MACHINE_PART", "EXP_SHARE"] {
                assert_eq!(description(map, item, "original help"), "original help");
            }
        }
        assert_eq!(
            description("MnMinneapolis", "POTION", "original potion"),
            "original potion"
        );
        assert_eq!(
            description("MnWinona", "EXP_SHARE", "original share"),
            "Full bench EXP.\nwhen held."
        );
        for map in ["MnMinneapolisDispatchOffice", "MnGrandMaraisLighthouse"] {
            assert!(!description(map, "CARD_KEY", "RADIO TOWER").contains("RADIO TOWER"));
            assert!(!description(map, "MACHINE_PART", "POWER PLANT").contains("POWER PLANT"));
        }
    }
}
