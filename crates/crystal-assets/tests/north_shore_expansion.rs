//! Installed North Shore expansion must preserve earned world content.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, expand_north_shore_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    previous_layout: String,
    layout: String,
    previous_dossier: String,
    dossier: String,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("real repository root")
}
fn source(variable: &str, fallback: &str) -> String {
    std::fs::read_to_string(
        std::env::var_os(variable)
            .map(PathBuf::from)
            .unwrap_or_else(|| root().join(fallback)),
    )
    .expect("frozen actual source input")
}
fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let path = std::env::var_os("CRYSTAL_NORTH_SHORE_EXPANSION_PACK").map(PathBuf::from)
            .unwrap_or_else(|| root().join(".local/minnesota-city-revisions/cities-37a09cd52d95b7425711237a4014ec387555a27cc3d4df61e246581b4d108cf6.crystalpack"));
        let base = read_verified_compiled_game_pack(path.canonicalize().expect("actual baseline path")).expect("completed installed North Shore baseline");
        let previous_layout = source("CRYSTAL_NORTH_SHORE_EXPANSION_PREVIOUS_LAYOUT", "planning/minnesota/revisions/northwoods-expansion/before/MN_R11.layout.json");
        let previous_dossier = source("CRYSTAL_NORTH_SHORE_EXPANSION_PREVIOUS_DOSSIER", "planning/minnesota/revisions/northwoods-expansion/before/MN_R11.json");
        let layout = source("CRYSTAL_NORTH_SHORE_EXPANSION_LAYOUT", "planning/minnesota/revisions/northwoods-expansion/after/MN_R11.layout.json");
        let dossier = source("CRYSTAL_NORTH_SHORE_EXPANSION_DOSSIER", "planning/minnesota/revisions/northwoods-expansion/after/MN_R11.json");
        let result = expand_north_shore_modpack(&base,&previous_layout,&layout,&previous_dossier,&dossier)
            .expect("exact frozen installed North Shore expansion");
        Inputs {base,result,previous_layout,layout,previous_dossier,dossier}
    })
}
fn expand(i: &Inputs, old: &str, layout: &str, dossier: &str) -> anyhow::Result<CompiledGamePack> {
    expand_north_shore_modpack(&i.base, old, layout, &i.previous_dossier, dossier)
}

#[test]
#[ignore = "requires completed external North Shore baseline and frozen authoring inputs"]
fn north_shore_expansion_preserves_old_ids_rooms_scripts_and_unrelated_content() {
    let i = inputs();
    let b = i.base.data();
    let r = i.result.data();
    assert_eq!(r.maps.len(), b.maps.len() + 5);
    let exteriors = BTreeSet::from(["MnR11C00R00", "MnR11C00R01", "MnR11C00R02"]);
    for (name, old) in &b.maps {
        let new = &r.maps[name];
        if exteriors.contains(name.as_str()) {
            let mut normalized = serde_json::to_value(new).expect("new native module");
            let mut previous = serde_json::to_value(old).expect("old native module");
            for field in [
                "blocks",
                "attributes",
                "objects",
                "events",
                "scripts",
                "script_text_bodies",
                "script_text_commands",
                "script_control_commands",
                "script_field_pickups",
                "map_event_section_commands",
            ] {
                normalized
                    .as_object_mut()
                    .expect("new fields")
                    .remove(field);
                previous.as_object_mut().expect("old fields").remove(field);
            }
            assert_eq!(
                normalized, previous,
                "unrelated executable field drift: {name}"
            );
            assert!(new.objects.starts_with(&old.objects));
            assert!(new.events.bg_events.starts_with(&old.events.bg_events));
            assert_eq!(new.events.warps, old.events.warps);
            assert_eq!(new.events.coord_events, old.events.coord_events);
            for (label, script) in &old.scripts {
                assert_eq!(
                    new.scripts.get(label),
                    Some(script),
                    "old script changed: {label}"
                );
            }
            for (label, text) in &old.script_text_bodies {
                assert_eq!(
                    new.script_text_bodies.get(label),
                    Some(text),
                    "old text changed: {label}"
                );
            }
            for command in &old.script_text_commands {
                assert!(new.script_text_commands.contains(command));
            }
            for command in &old.script_control_commands {
                assert!(new.script_control_commands.contains(command));
            }
            for pickup in &old.script_field_pickups {
                assert!(new.script_field_pickups.contains(pickup));
            }
            let commands = |m: &crystal_assets::MapModule| {
                m.map_event_section_commands
                    .iter()
                    .map(|c| (c.command.clone(), c.args.clone()))
                    .collect::<Vec<_>>()
            };
            let after = commands(new);
            for command in commands(old) {
                assert!(
                    after.contains(&command),
                    "original wire event changed: {command:?}"
                );
            }
        } else {
            assert_eq!(new, old, "unrelated or lighthouse map changed: {name}");
        }
    }
    let mut globals = r.story_event_script_constants.clone();
    for flag in ["EVENT_MN_R11_ITEM_05", "EVENT_MN_R11_ITEM_06"] {
        assert!(!b.story_event_script_constants.global.contains_key(flag));
        assert!(globals.global.remove(flag).is_some());
    }
    assert_eq!(globals, b.story_event_script_constants);
    assert_eq!(b.runtime_spawn_points, r.runtime_spawn_points);
    assert_eq!(b.trainers, r.trainers);
    assert_eq!(b.pokemon, r.pokemon);
    assert_eq!(b.items, r.items);
    assert_eq!(b.marts, r.marts);
    assert_eq!(b.moves, r.moves);
    assert_eq!(b.field_moves, r.field_moves);
    assert_eq!(i.base.compiled_audio(), i.result.compiled_audio());
    assert_eq!(
        i.base.audio_manifest().expect("base audio"),
        i.result.audio_manifest().expect("result audio")
    );
    for (name, old) in &b.wild_encounters {
        assert_eq!(
            r.wild_encounters.get(name),
            Some(old),
            "encounter drift: {name}"
        );
    }
    for (name, old) in &b.tilesets {
        if name != "mn_north_shore" {
            assert_eq!(r.tilesets.get(name), Some(old));
        }
    }
    let allowed = BTreeSet::from([
        "data/tilesets/mn_north_shore_metatiles.bin",
        "data/tilesets/mn_north_shore.json",
        "data/tilesets/mn_north_shore_palette_map.json",
        "gfx/tilesets/mn_north_shore.2bpp",
        "gfx/tilesets/mn_north_shore.png",
    ]);
    assert_eq!(i.base.runtime_files().len(), i.result.runtime_files().len());
    for (name, old) in i.base.runtime_files() {
        if !allowed.contains(name.as_str()) {
            assert_eq!(
                i.result.runtime_files().get(name),
                Some(old),
                "unrelated source file drift: {name}"
            );
        }
    }
}

#[test]
#[ignore = "requires completed external North Shore baseline and frozen authoring inputs"]
fn north_shore_expansion_has_exact_grid_stable_numeric_ids_and_true_split_reload() {
    let i = inputs();
    let d = i.result.data();
    let mut data = d.clone();
    let ids = [[1, 5], [2, 6], [3, 7], [8, 9]];
    let mut directed = 0;
    for (row, row_ids) in ids.iter().enumerate() {
        for (col, map_id) in row_ids.iter().enumerate() {
            let name = format!("MnR11C{col:02}R{row:02}");
            let constant = format!("MN_R11_C{col:02}R{row:02}");
            let m = &d.maps[&name];
            let meta = &d.runtime_map_metadata[&constant];
            assert_eq!(meta.group_id, 41);
            assert_eq!(meta.map_id, *map_id);
            assert_eq!(
                [m.attributes.width, m.attributes.height],
                [if col == 0 { 32 } else { 24 }, 24]
            );
            assert_eq!(m.blocks.len(), usize::from(m.attributes.width) * 24);
            assert_eq!(
                m.attributes.map_group_constant.as_deref(),
                Some("MAPGROUP_MINNESOTA_NORTH_SHORE")
            );
            data.maps.remove(&name);
            let mut expected = m.clone();
            for label in [
                m.attributes
                    .map_scripts_label
                    .as_ref()
                    .expect("owned script section"),
                m.attributes
                    .map_events_label
                    .as_ref()
                    .expect("owned event section"),
            ] {
                expected
                    .scripts
                    .entry(label.clone())
                    .or_insert_with(|| d.map_scripts[label].clone());
            }
            assert_eq!(
                data.assemble_map_module_from_compiled_payloads(&name)
                    .expect("actual uncached split native assembly"),
                expected
            );
            directed += m.attributes.connections.len();
            for c in &m.attributes.connections {
                let opposite = match c.direction.as_str() {
                    "north" => "south",
                    "south" => "north",
                    "east" => "west",
                    "west" => "east",
                    _ => panic!("cardinal connection"),
                };
                assert!(
                    d.maps[&c.target_map]
                        .attributes
                        .connections
                        .iter()
                        .any(|back| back.direction == opposite
                            && back.target_map == name
                            && back.offset == 0)
                );
            }
            assert!(d.wild_encounters.contains_key(&name));
        }
    }
    assert_eq!(directed, 20);
    assert_eq!(
        d.runtime_map_metadata["MN_R11_SPLIT_ROCK_LIGHTHOUSE"].map_id,
        4
    );
    assert!(
        expand_north_shore_modpack(
            &i.result,
            &i.previous_layout,
            &i.layout,
            &i.previous_dossier,
            &i.dossier
        )
        .is_err(),
        "must reject double application"
    );
}

#[test]
#[ignore = "requires completed external North Shore baseline and frozen authoring inputs"]
fn north_shore_expansion_stairs_copy_exact_source_pixels_palette_and_quadrants() {
    let i = inputs();
    let old = i.base.runtime_files();
    let new = i.result.runtime_files();
    let path = "data/tilesets/mn_north_shore_metatiles.bin";
    let before = &old[path];
    let after = &new[path];
    let recipe: Value = serde_json::from_str(&i.layout).expect("source art");
    let count = recipe["derived_blocks"].as_array().expect("blocks").len() - 1;
    assert_eq!(&after[..before.len()], before);
    assert_eq!(after.len(), before.len() + count * 16);
    let pm = &i.result.data().tilesets["mn_north_shore"].palette_map;
    let source_pm = &i.base.data().tilesets["cave"].palette_map;
    let physical = |glyph: u8, palette: u8| {
        if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            usize::from(glyph & 127) + 96
        }
    };
    for (offset, source) in [0x12usize, 0x30, 0x36].into_iter().take(count).enumerate() {
        let src = &old["data/tilesets/cave_metatiles.bin"][source * 16..(source + 1) * 16];
        let dst = &after[(0x81 + offset) * 16..(0x82 + offset) * 16];
        assert_eq!(
            i.result.data().tilesets["mn_north_shore"].collision[&format!("{:02x}", 0x81 + offset)],
            i.base.data().tilesets["cave"].collision[&format!("{source:02x}")]
        );
        for (&a, &b) in src.iter().zip(dst) {
            let ap = source_pm[usize::from(a)];
            let bp = pm[usize::from(b)];
            assert_eq!(ap & 7, bp & 7);
            let a = physical(a, ap);
            let b = physical(b, bp);
            assert_eq!(
                &old["gfx/tilesets/cave.2bpp"][a * 16..(a + 1) * 16],
                &new["gfx/tilesets/mn_north_shore.2bpp"][b * 16..(b + 1) * 16]
            );
        }
    }
    for &glyph in before {
        let a = i.base.data().tilesets["mn_north_shore"].palette_map[usize::from(glyph)];
        let b = pm[usize::from(glyph)];
        assert_eq!(a, b, "referenced old palette changed");
        let slot = physical(glyph, a);
        assert_eq!(
            &old["gfx/tilesets/mn_north_shore.2bpp"][slot * 16..(slot + 1) * 16],
            &new["gfx/tilesets/mn_north_shore.2bpp"][slot * 16..(slot + 1) * 16],
            "bank-aliased old art overwritten"
        );
    }
}

#[test]
#[ignore = "requires completed external North Shore baseline and frozen authoring inputs"]
fn north_shore_expansion_rejects_existing_event_party_story_and_source_art_drift() {
    let i = inputs();
    for field in [
        "trainers",
        "residents",
        "items",
        "signs",
        "doors",
        "interiors",
        "encounters",
        "required_flags",
    ] {
        let mut v: Value = serde_json::from_str(&i.dossier).expect("dossier");
        match field {
            "trainers" => v[field][0]["party"][0]["level"] = json!(1),
            "residents" => v[field][0]["lines"][0] = json!("CHANGED"),
            "items" => v[field][0]["quantity"] = json!(99),
            "signs" => v[field][0]["tile"][0] = json!(1),
            "doors" => v[field][0]["target_warp"] = json!(99),
            "interiors" => v[field][0]["rows"][0] = json!("01"),
            "encounters" => v[field]["water_rate"] = json!(99),
            _ => v[field][0] = json!("EVENT_UNEARNED"),
        }
        assert!(
            expand(i, &i.previous_layout, &i.layout, &v.to_string()).is_err(),
            "accepted changed {field}"
        );
    }
    for field in ["palette_override", "tile_row_order", "tile_row_sources"] {
        let mut v: Value = serde_json::from_str(&i.layout).expect("layout");
        v["derived_blocks"][1][field] = json!(0);
        assert!(
            expand(i, &i.previous_layout, &v.to_string(), &i.dossier).is_err(),
            "accepted render-only {field}"
        );
    }
    let mut old: Value = serde_json::from_str(&i.previous_layout).expect("previous");
    old["layers"]
        .as_array_mut()
        .expect("strokes")
        .push(json!({"name":"false preimage","kind":"point","position":[0,0],"block":"02"}));
    assert!(
        expand(i, &old.to_string(), &i.layout, &i.dossier).is_err(),
        "accepted stale installed preimage"
    );
}

#[test]
#[ignore = "requires completed external North Shore baseline and frozen authoring inputs"]
fn north_shore_expansion_rejects_required_grass_damage_and_roundtrips_real_pack() {
    let i = inputs();
    let d: Value = serde_json::from_str(&i.dossier).expect("dossier");
    let bounds = &d["required_grass_crossings"][0]["bounds_core"];
    let mut v: Value = serde_json::from_str(&i.layout).expect("layout");
    v["layers"].as_array_mut().expect("layers").push(json!({"name":"damage actual habitat","kind":"point","position":[bounds[0].as_u64().expect("x")/2,bounds[1].as_u64().expect("y")/2],"block":"01"}));
    assert!(expand(i, &i.previous_layout, &v.to_string(), &i.dossier).is_err());
    let path = std::env::temp_dir().join(format!(
        "geothite-north-shore-expanded-{}.crystalpack",
        std::process::id()
    ));
    i.result
        .write_preserving_storage(&path)
        .expect("actual pack serialization");
    let result =
        read_verified_compiled_game_pack(&path).expect("strict written native pack reload");
    assert_eq!(result, i.result);
    std::fs::remove_file(path).expect("remove temporary pack, retain render histories");
}
