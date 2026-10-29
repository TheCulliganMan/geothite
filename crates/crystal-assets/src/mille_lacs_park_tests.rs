//! Ignored external-pack publication and meaningful negative geometry cases.
use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::{Value, json};

use crate::{CompiledGamePack, read_verified_compiled_game_pack};

use super::{MAP, ROUTE, build_mille_lacs_park, hash};

fn inputs() -> (&'static CompiledGamePack, String, String, String) {
    static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
    let base = BASE.get_or_init(|| {
        read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_MILLE_LACS_PARK_BASE").expect("exact reviewed parent"),
        ))
        .expect("strict parent pack")
    });
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/towns/mille-lacs-park-v01");
    let read = |name| std::fs::read_to_string(root.join(name)).expect("authored park source");
    (
        base,
        read("town.json"),
        read("layout.json"),
        read("native.contract.json"),
    )
}

#[test]
#[ignore = "requires the exact external Mille Lacs park parent and reviewed source"]
fn park_publication_preserves_original_shore_quest_and_complete_native_world() {
    let (base, town, layout, contract) = inputs();
    let output =
        build_mille_lacs_park(base, &town, &layout, &contract).expect("valid reviewed park");
    assert_eq!(output.data.maps.len(), base.data.maps.len() + 1);
    for (name, map) in &base.data.maps {
        if name != ROUTE {
            assert_eq!(&output.data.maps[name], map, "original map {name}");
        }
    }
    assert_eq!(output.runtime_files, base.runtime_files);
    assert_eq!(output.audio_manifest, base.audio_manifest);
    assert_eq!(output.data.tilesets, base.data.tilesets);
    assert_eq!(output.data.trainers, base.data.trainers);
    assert_eq!(output.data.maps[MAP].objects.len(), 6);
    assert_eq!(output.data.maps[MAP].script_field_pickups.len(), 1);
    assert_eq!(output.data.maps[ROUTE].blocks, base.data.maps[ROUTE].blocks);
    assert_eq!(
        output.data.maps[ROUTE].scripts,
        base.data.maps[ROUTE].scripts
    );
    let mut expected_habitat = base.data.wild_encounters[ROUTE].clone();
    expected_habitat.map_name = MAP.into();
    assert_eq!(output.data.wild_encounters[MAP], expected_habitat);
    let replay =
        build_mille_lacs_park(base, &town, &layout, &contract).expect("deterministic replay");
    assert_eq!(replay, output);
    assert!(build_mille_lacs_park(&output, &town, &layout, &contract).is_err());
}

#[test]
#[ignore = "requires the exact external Mille Lacs park parent and reviewed source"]
fn park_rejects_open_dock_water_edge_and_free_grass_bypass() {
    let (base, town, layout, contract) = inputs();
    for (label, patch) in [
        (
            "missing dock rail",
            json!({"kind":"point","name":"invalid rail","position":[11,10],"block":"01"}),
        ),
        (
            "open eastern water edge",
            json!({"kind":"point","name":"invalid edge","position":[23,10],"block":"35"}),
        ),
        (
            "free lookout bypass",
            json!({"kind":"point","name":"invalid grass","position":[7,8],"block":"01"}),
        ),
        (
            "open dock end",
            json!({"kind":"point","name":"invalid dock cap","position":[14,11],"block":"71"}),
        ),
    ] {
        let mut value: Value = serde_json::from_str(&layout).expect("editor source");
        value["layers"].as_array_mut().expect("strokes").push(patch);
        let changed = serde_json::to_string(&value).expect("negative source");
        let mut bound: Value = serde_json::from_str(&contract).expect("contract");
        bound["layout_sha256"] = hash(changed.as_bytes()).into();
        assert!(
            build_mille_lacs_park(base, &town, &changed, &bound.to_string()).is_err(),
            "unsafe editor variant accepted: {label}"
        );
    }
    assert!(build_mille_lacs_park(base, &town, &(layout + " "), &contract).is_err());
}
