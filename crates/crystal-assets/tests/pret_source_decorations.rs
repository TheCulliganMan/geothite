#[path = "../src/bin/pack_core/source_decorations.rs"]
mod source_decorations;

#[test]
fn decoration_names_follow_attribute_type_even_when_numeric_domains_overlap() {
    let constants = serde_json::json!({"SUPER_NES":1,"PIKACHU":1,"_POSTER":2,"_DOLL":3,"BIG_":4,"PIKACHU_BED":5,"_BED":6});
    let names = [
        "CANCEL",
        "SUPER NES",
        " POSTER",
        " DOLL",
        "BIG ",
        "PIKACHU",
        " BED",
    ]
    .map(str::to_owned);
    let pokemon = ["PIKACHU".to_owned()];
    let name = |kind, token| {
        source_decorations::display_name(kind, token, &constants, &names, &pokemon).unwrap()
    };
    assert_eq!(name("DECO_PLANT", "SUPER_NES"), "SUPER NES");
    assert_eq!(name("DECO_POSTER", "PIKACHU"), "PIKACHU POSTER");
    assert_eq!(name("DECO_DOLL", "PIKACHU"), "PIKACHU DOLL");
    assert_eq!(name("DECO_BIGDOLL", "PIKACHU"), "BIG PIKACHU");
    assert_eq!(name("DECO_BED", "PIKACHU_BED"), "PIKACHU BED");
    assert!(
        source_decorations::display_name("DECO_POSTER", "MISSING", &constants, &names, &pokemon)
            .is_err()
    );
}

#[test]
fn pinned_decoration_catalog_preserves_names_indices_and_source_sprite() {
    let Some(root) = std::env::var_os("CRYSTAL_SOURCE_EXPORT_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let constants: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json"))
            .unwrap(),
    )
    .unwrap();
    let catalog =
        source_decorations::export(&root.join("vendor/pokecrystal"), &constants["global"]).unwrap();
    assert_eq!(catalog.decorations.len(), 45);
    let entry = |id| {
        catalog
            .decorations
            .iter()
            .find(|entry| entry.id == id)
            .unwrap()
    };
    assert_eq!(entry("DECO_SNES").display_name, "SUPER NES");
    assert_eq!(entry("DECO_TOWN_MAP").display_name, "TOWN MAP");
    assert_eq!(
        entry("DECO_SURF_PIKACHU_DOLL").display_name,
        "SURF PIKACHU DOLL"
    );
    assert_eq!(entry("DECO_BIG_SNORLAX_DOLL").display_name, "BIG SNORLAX");
    assert_eq!(entry("DECO_STARYU_DOLL").sprite, "SPRITE_STARMIE");
    assert_eq!(entry("DECO_FEATHERY_BED").index, 2);
    assert_eq!(
        entry("DECO_GOLD_TROPHY_DOLL").event_flag,
        "EVENT_DECO_GOLD_TROPHY"
    );
}
