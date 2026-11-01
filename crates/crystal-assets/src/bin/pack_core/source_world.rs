use super::{
    radio_text_export::{code_part, source_arguments},
    source_map_geometry, source_scripts,
};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn write(data: &Path, category: &str, values: &BTreeMap<String, Value>) -> Result<()> {
    let directory = data.join("content-packs/core-modular").join(category);
    std::fs::create_dir_all(&directory)?;
    for (id, value) in values {
        std::fs::write(
            directory.join(format!("{id}.json")),
            serde_json::to_vec_pretty(&json!({id:value}))?,
        )?;
    }
    std::fs::write(
        data.join(format!("{category}.json")),
        serde_json::to_vec_pretty(values)?,
    )?;
    Ok(())
}
pub(super) fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let geometry = source_map_geometry::read(&vendor)?;
    let data = root.join("apps/web/assets/data");
    let constants: Value = serde_json::from_slice(&std::fs::read(data.join("story_event_script_constants.json"))?)?;
    let constants: BTreeMap<String,i64> = serde_json::from_value(constants["global"].clone())?;
    let mut attributes = BTreeMap::new();
    let mut blocks = BTreeMap::new();
    let mut dimensions = BTreeMap::new();
    let mut metadata = BTreeMap::new();
    let mut scripts = BTreeMap::new();
    let mut map_ids = BTreeMap::new();
    let mut group = String::new();
    let mut group_id = 0;
    let mut map_id = 0;
    for line in std::fs::read_to_string(vendor.join("constants/map_constants.asm"))?
        .lines()
        .map(code_part)
    {
        if let Some(name) = line.strip_prefix("newgroup ") {
            group = name.trim().to_owned();
            group_id += 1;
            map_id = 0;
        }
        if let Some(args) = line.strip_prefix("map_const ") {
            let args = source_arguments(args)?;
            map_id += 1;
            map_ids.insert(args[0].clone(), (group.clone(), group_id, map_id));
        }
    }
    for (name, map) in geometry {
        let mut attr = map.attributes;
        if attr.music.as_deref() == Some("RADIO_TOWER_MUSIC | MUSIC_GOLDENROD_CITY") {
            let source = std::fs::read_to_string(vendor.join("home/map.asm"))?;
            ensure!(source.contains("bit STATUSFLAGS2_ROCKETS_IN_RADIO_TOWER_F, a") && source.contains("ld de, MUSIC_ROCKET_OVERTURE"), "authored Radio Tower music routing");
            attr.music = Some("MUSIC_GOLDENROD_CITY".to_owned());
        } else if attr.music.as_deref() == Some("MUSIC_MAHOGANY_MART") {
            let source = std::fs::read_to_string(vendor.join("home/map.asm"))?;
            ensure!(source.contains("bit STATUSFLAGS2_ROCKETS_IN_MAHOGANY_F, a") && source.contains("ld de, MUSIC_CHERRYGROVE_CITY"), "authored Mahogany music routing");
            attr.music = Some("MUSIC_CHERRYGROVE_CITY".to_owned());
        }
        let palette = match attr.palette.as_deref().context("source palette")? {
            "PALETTE_AUTO" => (0, Value::Null),
            "PALETTE_DAY" => (1, json!("day")),
            "PALETTE_NITE" => (2, json!("nite")),
            "PALETTE_MORN" => (3, json!("morn")),
            "PALETTE_DARK" => (4, json!("dark")),
            other => anyhow::bail!("unsupported palette {other}"),
        };
        attr.phone_service = (u8::from(attr.phone_flag) << 4) | palette.0;
        attr.time_of_day = palette.1.as_str().map(str::to_owned);
        let constant = attr.map_constant.as_ref().context("source map constant")?;
        let (group, group_id, map_id) = map_ids
            .get(constant)
            .context("source numeric map identity")?;
        metadata.insert(constant.clone(),json!({"constant":constant,"name":name,"groupName":group,"groupId":group_id,"mapId":map_id,"width":attr.width,"height":attr.height,"environment":attr.environment,"phoneService":attr.phone_service}));
        dimensions.insert(
            constant.clone(),
            json!({"width":attr.width,"height":attr.height}),
        );
        let bytes = map
            .blocks
            .into_iter()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        blocks.insert(
            attr.blocks_label
                .as_ref()
                .context("source blocks label")?
                .clone(),
            json!(STANDARD.encode(bytes)),
        );
        attributes.insert(name.clone(), serde_json::to_value(&attr)?);
        let source = std::fs::read_to_string(vendor.join("maps").join(format!("{name}.asm")))
            .with_context(|| format!("read map scripts {name}"))?;
        let mut parsed = source_scripts::parse(&source).with_context(|| format!("parse map scripts {name}"))?;
        source_scripts::materialize_event_fallthroughs(&vendor, &source, &mut parsed)
            .with_context(|| format!("preserve authored script fallthrough for {name}"))?;
        super::source_expressions::normalize(&source, &mut parsed, &constants).with_context(|| format!("resolve source expressions for {name}"))?;
        for (label, body) in parsed {
            ensure!(
                scripts.insert(label.clone(), body).is_none(),
                "duplicate map script {label}"
            );
        }
    }
    write(&data, "map_attributes", &attributes)?;
    write(&data, "map_blocks", &blocks)?;
    write(&data, "map_dimensions", &dimensions)?;
    write(&data, "runtime_map_metadata", &metadata)?;
    let mut spawns = BTreeMap::new();
    for line in std::fs::read_to_string(vendor.join("data/maps/spawn_points.asm"))?.lines().map(code_part) {
        let Some(raw) = line.strip_prefix("spawn ") else {continue};
        let args = source_arguments(raw)?;
        ensure!(args.len() == 3, "invalid source spawn row");
        if args[0] == "N_A" { continue; }
        let info = metadata.get(&args[0]).context("spawn map metadata")?;
        let x: i16 = args[1].parse()?; let y: i16 = args[2].parse()?;
        let id = spawns.len();
        let spawn = json!({"identifier":id,"mapConstant":args[0],"mapName":info["name"],"groupId":info["groupId"],"mapId":info["mapId"],"tileX":x,"tileY":y,"groupName":info["groupName"],"metatileX":x/2,"metatileY":y/2,"subtileX":x%2,"subtileY":y%2});
        let _: crystal_core::systems::special_routines::RuntimeSpawnPointRef = serde_json::from_value(spawn.clone())?;
        spawns.insert(id.to_string(),spawn);
    }
    ensure!(spawns.len() == usize::from(crystal_core::systems::special_routines::CRYSTAL_NUM_SPAWN_POINTS), "source spawn table incomplete");
    write(&data, "runtime_spawn_points", &spawns)?;
    write(&data, "map_scripts", &scripts)?;
    Ok(())
}
