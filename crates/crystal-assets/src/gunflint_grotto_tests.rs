//! Runtime-only current-pack fixtures for the resized native cave.
use std::path::PathBuf;

use serde_json::Value;

use super::{ROOM, rebuild_gunflint_grotto};

#[test]
#[ignore = "requires CRYSTAL_GUNFLINT_GROTTO_PARENT and authored grotto sources"]
fn gunflint_grotto_retains_rest_and_habitat_and_rejects_bypasses_or_stale_inputs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/caves/gunflint-grotto-v01");
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_GUNFLINT_GROTTO_PARENT").expect("completed current parent"),
    )
    .expect("verified current pack");
    let before = std::fs::read_to_string(root.join("before.json")).expect("original source");
    let after = std::fs::read_to_string(root.join("after.json")).expect("grotto source");
    let contract = std::fs::read_to_string(root.join("contract.json")).expect("native contract");
    let child =
        rebuild_gunflint_grotto(&base, &before, &after, &contract).expect("complete grotto");
    assert_eq!(child.data.maps.len(), base.data.maps.len());
    assert_eq!(child.runtime_files, base.runtime_files);
    assert_eq!(child.data.wild_encounters, base.data.wild_encounters);
    assert_eq!(child.audio_manifest, base.audio_manifest);
    let old = &base.data.maps[ROOM];
    let new = &child.data.maps[ROOM];
    assert_eq!(new.events.warps, old.events.warps);
    assert_eq!(new.objects[0], old.objects[0]);
    assert_eq!(new.objects[1].event_flag, old.objects[1].event_flag);
    assert_eq!((new.objects[1].x, new.objects[1].y), (35, 3));
    for (label, program) in &old.scripts {
        assert_eq!(new.scripts.get(label), Some(program));
    }
    for (name, map) in &base.data.maps {
        if name != ROOM {
            assert_eq!(child.data.maps[name], *map);
        }
    }
    let source: Value = serde_json::from_str(&after).expect("source JSON");
    for (label, x, y, replacement) in [
        ("pillar dry bypass", 13, 8, "02"),
        ("wrong corner stair", 17, 4, "30"),
        ("unpaired cave mouth", 2, 7, "07"),
        ("outer water", 19, 9, "0f"),
    ] {
        let mut bad = source.clone();
        let mut row = bad["rows"][y]
            .as_str()
            .expect("row")
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        row[x] = replacement.into();
        bad["rows"][y] = row.join(" ").into();
        assert!(
            rebuild_gunflint_grotto(&base, &before, &bad.to_string(), &contract).is_err(),
            "{label}"
        );
    }
    let mut stale: Value = serde_json::from_str(&contract).expect("contract JSON");
    stale["native_module_sha256"] = "stale".into();
    assert!(rebuild_gunflint_grotto(&base, &before, &after, &stale.to_string()).is_err());
}
