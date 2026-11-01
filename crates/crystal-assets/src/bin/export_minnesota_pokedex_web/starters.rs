//! Static starter labels from the authored family role.

use serde_json::Value;

pub(super) fn role(family: &Value) -> Option<(&'static str, &'static str)> {
    match family["starter_role"].as_str()? {
        "grass" => Some(("grass", "Grass starter")),
        "fire" => Some(("fire", "Fire starter")),
        "water" => Some(("water", "Water starter")),
        _ => None,
    }
}

pub(super) fn badge(family: &Value) -> String {
    role(family)
        .map(|(key, label)| {
            format!("<span class=\"starter-label\" data-starter-role=\"{key}\">{label}</span>")
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{badge, role};

    #[test]
    fn only_explicit_valid_roles_receive_starter_labels() {
        for (key, label) in [
            ("grass", "Grass starter"),
            ("fire", "Fire starter"),
            ("water", "Water starter"),
        ] {
            let family = json!({"starter_role": key});
            assert_eq!(role(&family), Some((key, label)));
            assert!(badge(&family).contains(label));
        }
        for family in [
            json!({"id": "MN_F003"}),
            json!({"starter_role": null}),
            json!({"starter_role": "water\"><img>"}),
        ] {
            assert_eq!(role(&family), None);
            assert!(badge(&family).is_empty());
        }
    }
}
