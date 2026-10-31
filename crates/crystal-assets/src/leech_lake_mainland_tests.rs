//! Current external parent and source inputs are runtime prerequisites.
use std::path::PathBuf;

use serde_json::Value;

use super::{LAKE, rebuild_leech_lake_mainland};

#[test]
#[ignore = "requires CRYSTAL_LEECH_MAINLAND_PARENT and reviewed mainland source files"]
fn leech_mainland_preserves_island_services_and_rejects_dry_bypass_or_shore_edits() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/towns/leech-lake-mainland-v01");
    let parent = std::env::var_os("CRYSTAL_LEECH_MAINLAND_PARENT")
        .expect("completed current mainland parent");
    let base = crate::read_verified_compiled_game_pack(parent).expect("verified parent");
    let source = std::fs::read_to_string(root.join("after.json")).expect("reviewed source");
    let contract = std::fs::read_to_string(root.join("contract.json")).expect("native preimage");
    let child = rebuild_leech_lake_mainland(&base, &source, &contract).expect("native mainland");
    assert_eq!(child.data.maps.len(), base.data.maps.len());
    assert_eq!(child.runtime_files, base.runtime_files);
    assert_eq!(child.audio_manifest, base.audio_manifest);
    let mut encounters = child.data.wild_encounters[LAKE].clone();
    assert_eq!(
        encounters.zones.pop().expect("new zone").id,
        "MN_LEECH_CABIN_PINES"
    );
    assert_eq!(encounters, base.data.wild_encounters[LAKE]);
    for (name, map) in &base.data.maps {
        if name != LAKE {
            assert_eq!(child.data.maps[name], *map);
        }
    }
    let old = &base.data.maps[LAKE];
    let new = &child.data.maps[LAKE];
    assert_eq!(new.events.warps, old.events.warps);
    assert_eq!(new.events.bg_events, old.events.bg_events);
    assert_eq!(new.events.coord_events, old.events.coord_events);
    assert!(new.objects.starts_with(&old.objects));
    for (label, script) in &old.scripts {
        if label != "MN_LEECH_SHORE_WALKERText" {
            assert_eq!(new.scripts.get(label), Some(script));
        }
    }
    let valid: Value = serde_json::from_str(&source).expect("source JSON");
    let mut dry = valid.clone();
    for y in 10..=13 {
        let row = dry["rows"][y]
            .as_str()
            .expect("row")
            .split_whitespace()
            .map(|v| if v == "03" { "02" } else { v })
            .collect::<Vec<_>>()
            .join(" ");
        dry["rows"][y] = row.into();
    }
    assert!(rebuild_leech_lake_mainland(&base, &dry.to_string(), &contract).is_err());
    let mut shore = valid.clone();
    let mut row = shore["rows"][12]
        .as_str()
        .expect("shore row")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    row[18] = "02".into();
    shore["rows"][12] = row.join(" ").into();
    assert!(rebuild_leech_lake_mainland(&base, &shore.to_string(), &contract).is_err());
    let mut orphan = valid.clone();
    let mut row = orphan["rows"][4]
        .as_str()
        .expect("pine row")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    row[3] = "02".into();
    orphan["rows"][4] = row.join(" ").into();
    assert!(rebuild_leech_lake_mainland(&base, &orphan.to_string(), &contract).is_err());
    let mut foreign = valid;
    foreign["grass_zone"]["grass"]["day"][0]["species"] = "MAGIKARP".into();
    assert!(rebuild_leech_lake_mainland(&base, &foreign.to_string(), &contract).is_err());
    let mut stale: Value = serde_json::from_str(&contract).expect("contract JSON");
    stale["native_module_sha256"] = "stale".into();
    assert!(rebuild_leech_lake_mainland(&base, &source, &stale.to_string()).is_err());
    let mut foreign_parent = base.clone();
    foreign_parent
        .data
        .wild_encounters
        .get_mut(LAKE)
        .expect("lake wildlife")
        .water
        .as_mut()
        .expect("native Surf fallback")
        .morning[0]
        .species = "MAGIKARP".into();
    foreign_parent.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        foreign_parent.format_version,
        &foreign_parent.data,
        &foreign_parent.audio_manifest,
        &foreign_parent.runtime_files,
        &foreign_parent.report,
    )
    .expect("declared foreign-parent fixture");
    assert!(rebuild_leech_lake_mainland(&foreign_parent, &source, &contract).is_err());
}
