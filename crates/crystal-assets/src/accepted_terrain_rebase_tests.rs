//! Three-way native preservation and conflicting revision rejection.
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::{CompiledGamePack, read_verified_compiled_game_pack, rebase_accepted_terrain};

fn read(variable: &str) -> CompiledGamePack {
    read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os(variable).expect("explicit completed terrain fixture"),
    ))
    .expect("strict terrain fixture")
}

fn update_identity(pack: &mut CompiledGamePack) {
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )
    .expect("valid changed fixture identity");
}

#[test]
#[ignore = "requires CRYSTAL_TERRAIN_CURRENT/BASE/CHILD/CONTRACT/WRITTEN external fixtures"]
fn accepted_terrain_rebase_preserves_newer_rooms_and_rejects_conflicts() {
    let current = read("CRYSTAL_TERRAIN_CURRENT");
    let before = read("CRYSTAL_TERRAIN_BASE");
    let accepted = read("CRYSTAL_TERRAIN_CHILD");
    let written = read("CRYSTAL_TERRAIN_WRITTEN");
    let source = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_TERRAIN_CONTRACT").expect("authored terrain contract"),
    )
    .expect("explicit contract");
    let result = rebase_accepted_terrain(&current, &before, &accepted, &source)
        .expect("exact nonoverlapping accepted terrain");
    assert!(
        result == written,
        "written pack equals deterministic Rust rebase"
    );
    assert!(result.runtime_files == current.runtime_files);
    assert!(result.compiled_audio() == current.compiled_audio());
    assert_eq!(result.data.maps.len(), current.data.maps.len());
    let mut changed_cells = 0;
    for (name, original) in &current.data.maps {
        let mut expected = original.clone();
        if let (Some(old), Some(new)) = (before.data.maps.get(name), accepted.data.maps.get(name)) {
            for (index, (a, b)) in old.blocks.iter().zip(&new.blocks).enumerate() {
                if a != b {
                    assert_eq!(expected.blocks[index], *a, "exact cell preimage");
                    expected.blocks[index] = *b;
                    changed_cells += 1;
                }
            }
            if old.attributes.border_block != new.attributes.border_block {
                expected.attributes.border_block = new.attributes.border_block;
            }
        }
        assert!(
            result.data.maps[name] == expected,
            "all other map fields stay exact: {name}"
        );
    }
    assert_eq!(changed_cells, 87, "34 grass cells and 53 cave cells");
    let mut old = serde_json::to_value(&current.data).expect("original data");
    let mut new = serde_json::to_value(&result.data).expect("rebased data");
    for key in ["maps", "map_attributes", "map_blocks"] {
        old.as_object_mut().expect("data").remove(key);
        new.as_object_mut().expect("data").remove(key);
    }
    assert!(
        old == new,
        "all other catalogs, identities, services and programs stay exact"
    );
    for name in ["MnR03C00R00", "MnR03C00R01", "MnR13Cave1F"] {
        assert_eq!(
            result
                .data
                .assemble_map_module_from_compiled_payloads(name)
                .expect("native mirrors"),
            result.data.maps[name]
        );
    }

    let contract: Value = serde_json::from_str(&source).expect("contract");
    for (field, value) in [
        ("schema_version", json!(2)),
        ("current_content_hash", json!("stale")),
        ("accepted_base_content_hash", json!("stale")),
        ("accepted_child_content_hash", json!("stale")),
        ("maps", json!(["MnR13Cave1F"])),
        (
            "maps",
            json!(["MnR03C00R00", "MnR03C00R01", "MnR13Cave1F", "MnStPaul"]),
        ),
    ] {
        let mut bad = contract.clone();
        bad[field] = value;
        assert!(
            rebase_accepted_terrain(&current, &before, &accepted, &bad.to_string()).is_err(),
            "reject {field}"
        );
    }
    let mut conflict = current.clone();
    let mut map = conflict.data.maps["MnR03C00R00"].clone();
    let index = before.data.maps[&map.id]
        .blocks
        .iter()
        .zip(&accepted.data.maps[&map.id].blocks)
        .position(|(a, b)| a != b)
        .expect("accepted grass delta");
    map.blocks[index] = if map.blocks[index] == 1 { 2 } else { 1 };
    crate::authored_town_publication::replace(&mut conflict, map)
        .expect("valid conflicting current cell");
    update_identity(&mut conflict);
    let mut bad = contract.clone();
    bad["current_content_hash"] = json!(conflict.identity.content_hash);
    let error = rebase_accepted_terrain(&conflict, &before, &accepted, &bad.to_string())
        .expect_err("overlapping current work must be rejected");
    assert!(
        error.to_string().contains("terrain cell conflicts"),
        "{error:#}"
    );

    let mut repriced = accepted.clone();
    repriced
        .data
        .items
        .get_mut("POTION")
        .expect("native item")
        .price += 1;
    update_identity(&mut repriced);
    let mut bad = contract.clone();
    bad["accepted_child_content_hash"] = json!(repriced.identity.content_hash);
    assert!(
        rebase_accepted_terrain(&current, &before, &repriced, &bad.to_string()).is_err(),
        "never import accepted catalog changes as terrain"
    );
    let mut duplicate = contract;
    duplicate["current_content_hash"] = json!(result.identity.content_hash);
    assert!(rebase_accepted_terrain(&result, &before, &accepted, &duplicate.to_string()).is_err());
}

#[test]
#[ignore = "requires the R17 relocation in CRYSTAL_TERRAIN_CURRENT/BASE/CHILD/CONTRACT/WRITTEN"]
fn accepted_terrain_rebase_keeps_the_lichen_story_and_rejects_unsafe_actor_moves() {
    let current = read("CRYSTAL_TERRAIN_CURRENT");
    let before = read("CRYSTAL_TERRAIN_BASE");
    let accepted = read("CRYSTAL_TERRAIN_CHILD");
    let written = read("CRYSTAL_TERRAIN_WRITTEN");
    let source = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_TERRAIN_CONTRACT").expect("R17 contract"),
    )
    .expect("source");
    let result = rebase_accepted_terrain(&current, &before, &accepted, &source)
        .expect("safe explicit resident relocation");
    assert!(
        result == written,
        "written R17 synchronization is deterministic"
    );
    assert!(result.runtime_files == current.runtime_files);
    assert!(result.compiled_audio() == current.compiled_audio());
    for (name, original) in &current.data.maps {
        let actual = &result.data.maps[name];
        if name != "MnR17Cave1F" {
            assert!(actual == original, "all other maps stay exact: {name}");
            continue;
        }
        assert_eq!(actual.blocks, accepted.data.maps[name].blocks);
        assert_eq!(actual.attributes.border_block, 0x2e);
        let mut normalized = actual.clone();
        normalized.blocks.clone_from(&original.blocks);
        normalized.attributes.border_block = original.attributes.border_block;
        normalized
            .map_event_section_commands
            .clone_from(&original.map_event_section_commands);
        let actor = normalized
            .objects
            .iter_mut()
            .find(|o| o.script == "MN_INTERIOR_R17_LICHEN_READER")
            .expect("retained resident");
        assert_eq!([actor.x, actor.y], [6, 10]);
        actor.x = 8;
        actor.y = 10;
        assert!(
            normalized == *original,
            "only accepted terrain and the explicit resident position change"
        );
    }
    let contract: Value = serde_json::from_str(&source).expect("contract");
    for (field, value) in [
        ("before", json!([7, 10])),
        ("tile", json!([0, 0])),
        ("tile", json!([9, 13])),
        ("approach", json!([65535, 65535])),
        ("arrival", json!([5, 2])),
        ("script", json!("MN_R17_ITEM_04")),
    ] {
        let mut bad = contract.clone();
        bad["actor_relocations"][0][field] = value;
        assert!(
            rebase_accepted_terrain(&current, &before, &accepted, &bad.to_string()).is_err(),
            "reject unsafe resident {field}"
        );
    }
}
