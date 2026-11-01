#[path = "../src/bin/pack_core/source_field_encounters.rs"]
mod source_field_encounters;
#[path = "../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;

#[test]
fn full_source_field_catalog_exports_when_workspace_is_available() {
    let Ok(root) = std::env::var("CRYSTAL_SOURCE_EXPORT_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    source_field_encounters::export(root).unwrap();
    let catalog: std::collections::BTreeMap<
        String,
        crystal_core::world::encounters::FieldEncounterData,
    > = serde_json::from_slice(
        &std::fs::read(root.join("apps/web/assets/data/field_encounters.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(catalog.len(), 36);
    assert!(catalog["NewBarkTown"].tables.is_empty());
    assert_eq!(catalog["IlexForest"].tables["headbutt"].rare.len(), 6);
    assert_eq!(
        catalog
            .values()
            .filter(|data| data.tables.contains_key("rock_smash"))
            .count(),
        4
    );
    assert!(catalog["CianwoodCity"].tables.get("headbutt").is_none());
}

#[test]
fn source_tables_keep_none_alias_rare_weights_and_tree_only_sleep() {
    let source = "TreeMonSet_None:\nTreeMonSet_Route:\ndb 90, HOOTHOOT, 10\ndb 10, EXEGGCUTE, 10\ndb -1\ndb 100, PINECO, 10\ndb -1\nTreeMonSet_Rock:\ndb 90, KRABBY, 15\ndb 10, SHUCKLE, 15\ndb -1";
    let sleep = "AsleepTreeMonsMorn:\ndb HOOTHOOT\ndb -1\nAsleepTreeMonsDay:\ndb HOOTHOOT\ndb -1\nAsleepTreeMonsNite:\ndb EXEGGCUTE\ndb KRABBY\ndb -1";
    let sets = source_field_encounters::tables(source, sleep, 7).unwrap();
    assert!(!sets.contains_key("TreeMonSet_None"));
    let route = &sets["TreeMonSet_Route"];
    assert_eq!(route.common[0].weight, 90);
    assert_eq!(route.rare[0].species, "PINECO");
    assert_eq!(
        serde_json::to_value(&route.common[0].sleep_turns_by_time).unwrap(),
        serde_json::json!({"morning":7,"day":7})
    );
    assert!(
        sets["TreeMonSet_Rock"].common[0]
            .sleep_turns_by_time
            .is_empty()
    );
    assert!(sets["TreeMonSet_Rock"].rare.is_empty());
    let names = [("ROUTE_40".to_string(), "Route40".to_string())].into();
    let maps = source_field_encounters::map_catalog("TreeMonMaps:\ntreemon_map ROUTE_40, TREEMON_SET_NONE\ndb -1\nRockMonMaps:\ntreemon_map ROUTE_40, TREEMON_SET_ROCK\ndb -1", &names, &sets).unwrap();
    assert_eq!(maps["Route40"].tables.len(), 1);
    assert!(maps["Route40"].tables.contains_key("rock_smash"));
    assert!(
        source_field_encounters::tables(
            &source.replace("db 90, KRABBY", "db 89, KRABBY"),
            sleep,
            7
        )
        .is_err()
    );
    assert!(
        source_field_encounters::tables(
            &source.replace("db 10, SHUCKLE, 15\ndb -1", "db 10, SHUCKLE, 15"),
            sleep,
            7
        )
        .is_err()
    );
}
