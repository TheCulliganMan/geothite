//! Publish a disposable art-review map without changing any campaign maps.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::CompiledMapExtension;
use serde_json::json;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 2,
        "usage: minnesota_tile_gallery ART_PACK FIXTURE_PACK"
    );
    ensure!(!args[1].exists(), "choose a new fixture output");
    let pack = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    ensure!(
        pack.data().tilesets.contains_key("mn_state_fair_props"),
        "fair art missing"
    );
    let name = "MnFairArtGallery";
    let constant = "MN_FAIR_ART_GALLERY";
    let mut module = serde_json::to_value(
        pack.data()
            .maps
            .get("NewBarkTown")
            .context("source map attributes")?,
    )?;
    for (key, value) in module.as_object_mut().context("map module")? {
        match key.as_str() {
            "id" | "attributes" => {}
            "scenes" => *value = json!({"scenes": []}),
            "events" => *value = json!({"warps": [], "coord_events": [], "bg_events": []}),
            _ if value.is_array() => *value = json!([]),
            _ if value.is_object() => *value = json!({}),
            _ => anyhow::bail!("unexpected map field {key}"),
        }
    }
    module["id"] = json!(name);
    let attributes = &mut module["attributes"];
    for (key, value) in [
        ("tileset_name", json!("mn_state_fair_props")),
        ("width", json!(8)),
        ("height", json!(5)),
        ("border_block", json!(10)),
        ("connections", json!([])),
        ("connection_flags", json!("0")),
        ("map_constant", json!(constant)),
        ("map_group_constant", json!("MAPGROUP_FAIR_ART_GALLERY")),
        ("blocks_label", json!("MnFairArtGallery_Blocks")),
        ("map_scripts_label", json!("MnFairArtGallery_MapScripts")),
        ("map_events_label", json!("MnFairArtGallery_MapEvents")),
    ] {
        attributes[key] = value;
    }
    let mut blocks = vec![1_u16; 40];
    for y in 0..5 {
        for x in 0..8 {
            if x == 0 || x == 7 || y == 0 || y == 4 {
                blocks[y * 8 + x] = 10;
            }
        }
    }
    for (x, y, block) in [
        (1, 1, 128),
        (2, 1, 129),
        (1, 2, 130),
        (2, 2, 131),
        (4, 1, 132),
        (6, 1, 133),
        (4, 3, 134),
    ] {
        blocks[y * 8 + x] = block;
    }
    module["blocks"] = json!(blocks);
    let mut metadata = pack
        .data()
        .runtime_map_metadata
        .get("NEW_BARK_TOWN")
        .context("map metadata")?
        .clone();
    metadata.name = name.into();
    metadata.constant = constant.into();
    metadata.group_name = "FAIR_ART_GALLERY".into();
    metadata.group_id = 250;
    metadata.map_id = 1;
    metadata.width = 8;
    metadata.height = 5;
    let mut spawn = pack
        .data()
        .runtime_spawn_points
        .values()
        .next()
        .context("spawn template")?
        .clone();
    spawn.identifier = 9000;
    spawn.map_name = name.into();
    spawn.map_constant = constant.into();
    spawn.group_name = metadata.group_name.clone();
    spawn.group_id = 250;
    spawn.map_id = 1;
    spawn.tile_x = 3;
    spawn.tile_y = 7;
    spawn.metatile_x = 1;
    spawn.metatile_y = 3;
    spawn.subtile_x = 1;
    spawn.subtile_y = 1;
    pack.with_minnesota_tile_gallery(CompiledMapExtension {
        manifest_id: "minnesota-fair-art-gallery-fixture-v1".into(),
        map_name: name.into(),
        map_constant: constant.into(),
        module: serde_json::from_value(module)?,
        metadata,
        spawn_key: "9000".into(),
        spawn,
        wild_encounters: None,
        start_new_game_here: false,
    })?
    .write_preserving_storage(&args[1])?;
    println!(
        "Published disposable {name} fixture in {}",
        args[1].display()
    );
    Ok(())
}
