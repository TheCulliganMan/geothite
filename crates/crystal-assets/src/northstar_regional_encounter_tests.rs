//! Explicit native wild stages survive binding; imported legacy stages do not.
use std::path::PathBuf;

use serde_json::json;

use crate::read_verified_compiled_game_pack;

#[test]
#[ignore = "requires CRYSTAL_REGIONAL_WILD_PACK with the active Minnesota roster"]
fn regional_encounters_keep_authored_forms_without_relaxing_legacy_or_trainer_rules() {
    let pack = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_REGIONAL_WILD_PACK").expect("regional fixture"),
    ))
    .expect("strict regional pack");
    let policy = crate::authored_regional_approach::policy().expect("active regional policy");
    let mut value = json!({
        "grass": [{"species":"REGALARCH","level":3}, {"species":"BUTTERFREE","level":3}],
        "time": {"day_species":"REGALARCH","day_level":3,
            "night_species":"BUTTERFREE","night_level":3}
    });
    super::rewrite_encounters(&pack, &policy, &mut value).expect("curated and imported wild forms");
    assert_eq!(
        value,
        json!({
            "grass": [{"species":"REGALARCH","level":3}, {"species":"STRIPILLAR","level":3}],
            "time": {"day_species":"REGALARCH","day_level":3,
                "night_species":"STRIPILLAR","night_level":3}
        })
    );
    let before = value.clone();
    super::rewrite_encounters(&pack, &policy, &mut value).expect("idempotent encounter audit");
    assert_eq!(value, before);
    assert_eq!(
        policy
            .target(&pack, "REGALARCH", Some(3))
            .expect("trainer/gift level rule"),
        "STRIPILLAR"
    );
    for mut bad in [
        json!({"species":"UNMAPPED_LEGACY","level":3}),
        json!({"species":"REGALARCH","level":0}),
        json!({"species":"REGALARCH","level":256}),
    ] {
        assert!(
            super::rewrite_encounters(&pack, &policy, &mut bad).is_err(),
            "unmapped species and invalid level remain errors"
        );
    }
}
