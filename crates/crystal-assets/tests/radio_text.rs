use crystal_assets::AssetRoot;

#[test]
fn radio_selection_tables_preserve_source_order_and_fallthrough() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let mut data = AssetRoot::new(&root)
        .load_verified_compiled_game_data("content-packs/core-modular.crystalpack")
        .unwrap();
    data.materialize_global_scripts().unwrap();
    let module = data.global_scripts.as_ref().unwrap();
    for (label, expected) in [
        (
            "OaksPKMNTalkRoutes",
            "ROUTE_29 ROUTE_46 ROUTE_30 ROUTE_32 ROUTE_34 ROUTE_35 ROUTE_37 ROUTE_38 ROUTE_39 ROUTE_42 ROUTE_43 ROUTE_44 ROUTE_45 ROUTE_36 ROUTE_31",
        ),
        (
            "PnP_Places",
            "PALLET_TOWN ROUTE_22 PEWTER_CITY CERULEAN_POLICE_STATION ROUTE_12 ROUTE_11 ROUTE_16 ROUTE_14 CINNABAR_POKECENTER_2F_BETA",
        ),
        (
            "PnP_HiddenPeople",
            "WILL BRUNO KAREN KOGA CHAMPION BROCK MISTY LT_SURGE ERIKA JANINE SABRINA BLAINE BLUE RIVAL1 POKEMON_PROF CAL RIVAL2 RED -1",
        ),
        (
            "PnP_HiddenPeople_BeatE4",
            "BROCK MISTY LT_SURGE ERIKA JANINE SABRINA BLAINE BLUE RIVAL1 POKEMON_PROF CAL RIVAL2 RED -1",
        ),
        (
            "PnP_HiddenPeople_BeatKanto",
            "RIVAL1 POKEMON_PROF CAL RIVAL2 RED -1",
        ),
    ] {
        let body = module
            .definitions
            .get(label)
            .unwrap_or_else(|| {
                panic!("radio selection table {label} is absent from the runtime catalog")
            })
            .as_array()
            .unwrap();
        assert!(
            !module.scripts.contains_key(label),
            "data table {label} must not be an executable root"
        );
        assert_eq!(
            body.iter()
                .map(|row| row["args"][0].as_str().unwrap())
                .collect::<Vec<_>>(),
            expected.split_whitespace().collect::<Vec<_>>(),
            "{label}"
        );
        let command = if label.starts_with("PnP_Hidden") {
            "db"
        } else {
            "map_id"
        };
        assert!(
            body.iter()
                .all(|row| row["command"] == command && row["args"].as_array().unwrap().len() == 1)
        );
    }
}

#[test]
fn source_radio_commands_reach_the_runtime_text_catalog() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let mut data = AssetRoot::new(&root)
        .load_verified_compiled_game_data("content-packs/core-modular.crystalpack")
        .unwrap();
    data.materialize_global_scripts().unwrap();
    let texts = &data.global_scripts.as_ref().unwrap().script_text_bodies;
    let source =
        std::fs::read_to_string(root.join("vendor/pokecrystal/engine/pokegear/radio.asm")).unwrap();
    let labels = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("text_far ").map(str::trim))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(labels.len(), 113);
    for label in labels {
        assert!(
            texts.contains_key(label),
            "radio text {label} is absent from the runtime catalog"
        );
    }
    let rocket = &texts["_RocketRadioText7"].commands;
    assert_eq!(
        rocket
            .iter()
            .map(|command| command.command.as_str())
            .collect::<Vec<_>>(),
        ["text_start", "line", "text_pause", "text", "done"]
    );
    assert_eq!(rocket[1].args, ["\"GIOVANNI! @\""]);
    assert_eq!(rocket[3].args, ["\"Can you\""]);
}
