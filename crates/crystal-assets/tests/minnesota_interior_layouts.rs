//! Terrain preimages, native access and preservation over an external story pack.
use std::path::PathBuf;

use crystal_assets::{build_minnesota_interior_layout_modpack, read_verified_compiled_game_pack};
use serde_json::Value;

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_LAYOUT_BASE and CRYSTAL_INTERIOR_PACK"]
fn interior_layouts_change_only_named_terrain_and_append_readable_furniture() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_LAYOUT_BASE").expect("story parent"),
    ))
    .expect("verified parent");
    let pack = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("layout output"),
    ))
    .expect("verified output");
    let source = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_INTERIOR_LAYOUT_SOURCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../planning/minnesota/interior-life/layouts.json")
            }),
    )
    .expect("explicit layouts");
    let plan: Value = serde_json::from_str(&source).expect("layout contract");
    assert_eq!(base.runtime_files(), pack.runtime_files());
    assert_eq!(base.compiled_audio(), pack.compiled_audio());
    let before = base.data();
    let after = pack.data();
    let mut old_data = serde_json::to_value(before).expect("old data");
    let mut new_data = serde_json::to_value(after).expect("new data");
    for key in [
        "maps",
        "map_scripts",
        "npcs",
        "map_blocks",
        "map_attributes",
    ] {
        old_data.as_object_mut().expect("old fields").remove(key);
        new_data.as_object_mut().expect("new fields").remove(key);
    }
    assert_eq!(
        old_data, new_data,
        "all other content, flags, recovery and audio metadata"
    );
    for (name, old) in &before.maps {
        let new = &after.maps[name];
        let Some(room) = plan["rooms"]
            .as_array()
            .expect("rooms")
            .iter()
            .find(|r| r["map"] == *name)
        else {
            assert_eq!(old, new, "unrelated map, including every gym: {name}");
            assert_eq!(before.map_attributes[name], after.map_attributes[name]);
            if let Some(label) = &old.attributes.blocks_label {
                assert_eq!(before.map_blocks[label], after.map_blocks[label]);
            }
            continue;
        };
        let mut attrs = old.attributes.clone();
        attrs.tileset_name = room["tileset"].as_str().expect("tileset").into();
        attrs.border_block = room["border_block"].as_u64().expect("border") as u8;
        assert_eq!(
            attrs, new.attributes,
            "only palette/source bank and border change: {name}"
        );
        let rows = room["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .flat_map(|v| v.as_str().expect("row").split_whitespace())
            .map(|s| u16::from_str_radix(s, 16).expect("native block"))
            .collect::<Vec<_>>();
        assert_eq!(new.blocks, rows);
        assert_eq!(old.objects, new.objects);
        assert_eq!(old.events.warps, new.events.warps);
        assert_eq!(old.events.coord_events, new.events.coord_events);
        assert_eq!(
            old.events.bg_events,
            new.events.bg_events[..old.events.bg_events.len()]
        );
        for (label, body) in &old.scripts {
            assert_eq!(
                Some(body),
                new.scripts.get(label),
                "retained script {label}"
            );
        }
        // Strip only the authorized room data and appended furniture programs.
        // Every original typed command table and map identity must remain exact.
        let mut restored = serde_json::to_value(new).expect("new map fields");
        let original = serde_json::to_value(old).expect("old map fields");
        for (key, value) in original.as_object().expect("old fields") {
            if matches!(
                key.as_str(),
                "blocks" | "attributes" | "events" | "map_event_section_commands"
            ) {
                restored[key] = value.clone();
                continue;
            }
            match value {
                Value::Array(values) => {
                    restored[key]
                        .as_array_mut()
                        .expect("array table")
                        .truncate(values.len());
                }
                Value::Object(values) => {
                    restored[key]
                        .as_object_mut()
                        .expect("object table")
                        .retain(|label, _| values.contains_key(label));
                }
                _ => {}
            }
        }
        assert_eq!(restored, original, "complete original programs: {name}");
    }
    assert_eq!(
        build_minnesota_interior_layout_modpack(&base, &source)
            .expect("deterministic rebuild")
            .identity()
            .expect("identity"),
        pack.identity().expect("identity")
    );
    assert!(build_minnesota_interior_layout_modpack(&pack, &source).is_err());
    let mut invalid = plan.clone();
    invalid["rooms"][0]["before_rows"][0] = "00".into();
    assert!(
        build_minnesota_interior_layout_modpack(&base, &invalid.to_string()).is_err(),
        "stale preimage rejected"
    );
    let first = &plan["rooms"][0];
    let map = &base.data().maps[first["map"].as_str().expect("first room")];
    let wall = format!("{:02x}", first["border_block"].as_u64().expect("wall"));
    let blocked = |tile: [u16; 2]| {
        let mut invalid = plan.clone();
        let y = usize::from(tile[1] / 2);
        let mut row = first["rows"][y]
            .as_str()
            .expect("native row")
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        row[usize::from(tile[0] / 2)] = wall.clone();
        invalid["rooms"][0]["rows"][y] = row.join(" ").into();
        invalid
    };
    invalid = blocked([map.objects[0].x, map.objects[0].y]);
    assert!(
        build_minnesota_interior_layout_modpack(&base, &invalid.to_string()).is_err(),
        "buried residents rejected"
    );
    invalid = blocked([map.events.warps[0].x, map.events.warps[0].y]);
    assert!(
        build_minnesota_interior_layout_modpack(&base, &invalid.to_string()).is_err(),
        "blocked carpet rejected"
    );
    // A read target must be furniture. Merely turning toward a floor event can
    // appear to work, then step onto it when the player already faces that way.
    invalid = plan.clone();
    invalid["rooms"][0]["exhibits"][0]["tile"] = first["exhibits"][0]["approach"].clone();
    let error = build_minnesota_interior_layout_modpack(&base, &invalid.to_string())
        .expect_err("walkable furniture target rejected");
    assert!(
        format!("{error:#}").contains("must target solid furniture"),
        "reject the target itself before testing its approach: {error:#}"
    );
    let (room_index, tile) = plan["rooms"]
        .as_array()
        .expect("rooms")
        .iter()
        .enumerate()
        .find_map(|(index, room)| {
            let collision = &base.data().tilesets[room["tileset"].as_str()?].collision;
            room["rows"]
                .as_array()?
                .iter()
                .enumerate()
                .find_map(|(y, row)| {
                    row.as_str()?
                        .split_whitespace()
                        .enumerate()
                        .find_map(|(x, block)| {
                            collision[block]
                                .iter()
                                .position(|kind| matches!(kind.as_str(), "TOWN_MAP" | "RADIO"))
                                .map(|q| (index, [x * 2 + q % 2, y * 2 + q / 2]))
                        })
                })
        })
        .expect("retained native wall map or radio");
    invalid = plan.clone();
    invalid["rooms"][room_index]["exhibits"][0]["tile"] = serde_json::json!(tile);
    invalid["rooms"][room_index]["exhibits"][0]["approach"] =
        serde_json::json!([tile[0], tile[1] + 1]);
    let error = build_minnesota_interior_layout_modpack(&base, &invalid.to_string())
        .expect_err("native wall-map/radio service protected");
    assert!(format!("{error:#}").contains("must target solid furniture"));
    if let Some(counter) = first["actor_approaches"][0].get("counter_tile") {
        let mut invalid = plan.clone();
        invalid["rooms"][0]["actor_approaches"][0]["counter_tile"] =
            first["actor_approaches"][0]["tile"].clone();
        let error = build_minnesota_interior_layout_modpack(&base, &invalid.to_string())
            .expect_err("a floor cell cannot be declared as a counter");
        assert!(format!("{error:#}").contains("native counter approach"));
        invalid = plan.clone();
        invalid["rooms"][0]["exhibits"][0]["tile"] = counter.clone();
        invalid["rooms"][0]["exhibits"][0]["approach"] =
            first["actor_approaches"][0]["tile"].clone();
        let error = build_minnesota_interior_layout_modpack(&base, &invalid.to_string())
            .expect_err("readable furniture cannot intercept a resident's counter");
        assert!(format!("{error:#}").contains("furniture interaction overlaps"));
    }
    invalid = plan.clone();
    invalid["rooms"][0]["map"] = "MnDuluthGym".into();
    assert!(
        build_minnesota_interior_layout_modpack(&base, &invalid.to_string()).is_err(),
        "gyms excluded"
    );
}
