//! External source and parent inputs remain runtime prerequisites.
use std::path::PathBuf;

use serde_json::Value;

use super::{ROOM, rebuild_iron_crosscut};

#[test]
#[ignore = "requires CRYSTAL_IRON_CROSSCUT_PARENT and current authored source files"]
fn iron_crosscut_preserves_the_complete_parent_and_rejects_bad_stairs_lips_or_mouths() {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_IRON_CROSSCUT_PARENT").expect("current parent"),
    )
    .expect("verified parent");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/caves/iron-crosscut-v01");
    let before = std::fs::read_to_string(root.join("before.json")).expect("before");
    let after = std::fs::read_to_string(root.join("after.json")).expect("after");
    let contract = std::fs::read_to_string(root.join("contract.json")).expect("contract");
    let child = rebuild_iron_crosscut(&base, &before, &after, &contract).expect("fitted crosscut");
    let mut expected = base.data.maps[ROOM].clone();
    expected.blocks.clone_from(&child.data.maps[ROOM].blocks);
    assert_eq!(child.data.maps[ROOM], expected);
    assert_eq!(child.data.trainers, base.data.trainers);
    assert_eq!(child.data.wild_encounters, base.data.wild_encounters);
    assert_eq!(child.runtime_files, base.runtime_files);
    for (name, map) in &base.data.maps {
        if name != ROOM {
            assert_eq!(child.data.maps[name], *map);
        }
    }
    for (label, x, y, value) in [
        ("wrong stair", 3, 4, "36"),
        ("reverse ledge", 6, 9, "3c"),
        ("cliff adjacent lip", 6, 8, "0a"),
        ("unpaired mouth", 4, 9, "07"),
        ("open edge", 0, 8, "02"),
    ] {
        let mut bad: Value = serde_json::from_str(&after).expect("source");
        let mut row = bad["rows"][y]
            .as_str()
            .expect("row")
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        row[x] = value.into();
        bad["rows"][y] = row.join(" ").into();
        assert!(
            rebuild_iron_crosscut(&base, &before, &bad.to_string(), &contract).is_err(),
            "{label}"
        );
    }
    let mut stale: Value = serde_json::from_str(&contract).expect("binding");
    stale["native_module_sha256"] = "stale".into();
    assert!(rebuild_iron_crosscut(&base, &before, &after, &stale.to_string()).is_err());
}
