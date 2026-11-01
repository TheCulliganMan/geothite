//! Real-pack preservation and stale-input checks for regular trainer art reuse.
use crystal_assets::{distribute_minnesota_trainer_art, read_verified_compiled_game_pack};
use serde_json::Value;

#[test]
#[ignore = "requires CRYSTAL_DISTRIBUTION_PARENT and CRYSTAL_DISTRIBUTION_SOURCE"]
fn trainer_distribution_preserves_programs_and_rejects_stale_inputs() {
    let parent = std::env::var_os("CRYSTAL_DISTRIBUTION_PARENT").expect("explicit parent");
    let source = std::env::var_os("CRYSTAL_DISTRIBUTION_SOURCE").expect("explicit assignments");
    let base = read_verified_compiled_game_pack(parent).expect("verified parent");
    let mut selected: Value =
        serde_json::from_slice(&std::fs::read(source).expect("source")).expect("assignment JSON");
    let row = selected["assignments"]
        .as_array()
        .expect("assignments")
        .iter()
        .find(|r| r["map"] == "MnR12C00R00")
        .expect("route with authored programs")
        .clone();
    selected["assignments"] = serde_json::json!([row]);
    let pack = distribute_minnesota_trainer_art(&base, &selected.to_string()).expect("valid reuse");
    assert_eq!(base.runtime_files(), pack.runtime_files());
    assert_eq!(base.compiled_audio(), pack.compiled_audio());
    for (id, map) in &base.data().maps {
        let updated = &pack.data().maps[id];
        assert_eq!(map.blocks, updated.blocks, "terrain {id}");
        assert_eq!(map.events, updated.events, "events {id}");
        assert_eq!(
            map.script_control_commands, updated.script_control_commands,
            "control {id}"
        );
        assert_eq!(
            map.script_text_commands, updated.script_text_commands,
            "text commands {id}"
        );
        assert_eq!(
            map.script_text_bodies, updated.script_text_bodies,
            "dialogue {id}"
        );
    }
    let mut stale = selected.clone();
    stale["assignments"][0]["previous_sprite"] = "SPRITE_STALE".into();
    assert!(distribute_minnesota_trainer_art(&base, &stale.to_string()).is_err());
    let mut duplicate = selected.clone();
    let row = duplicate["assignments"][0].clone();
    duplicate["assignments"]
        .as_array_mut()
        .expect("rows")
        .push(row);
    assert!(distribute_minnesota_trainer_art(&base, &duplicate.to_string()).is_err());
    for (key, forbidden) in [
        ("trainer_class", "MN_ELITE_FLINT"),
        ("trainer_id", "MN_RIVAL_REED"),
    ] {
        let mut invalid = selected.clone();
        invalid["assignments"][0][key] = forbidden.into();
        assert!(distribute_minnesota_trainer_art(&base, &invalid.to_string()).is_err());
    }
}
