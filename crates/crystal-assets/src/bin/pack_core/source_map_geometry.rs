//! Source-authenticated map geometry, independent of scripts and event export.
//! No incomplete MapModule is fabricated here. All source data stays external.
use anyhow::{Context, Result, ensure};
use crystal_core::map::{MapAttributes, MapConnection};
use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

#[derive(Debug)]
pub struct SourceMapGeometry {
    pub attributes: MapAttributes,
    pub blocks: Vec<u16>,
}

fn statements(source: &str) -> Vec<&str> {
    let mut macro_body = false;
    source
        .lines()
        .filter_map(|raw| {
            let line = raw.split(';').next().unwrap_or("").trim();
            if line.starts_with("MACRO ") {
                macro_body = true;
                return None;
            }
            if line == "ENDM" {
                macro_body = false;
                return None;
            }
            (!macro_body && !line.is_empty()).then_some(line)
        })
        .collect()
}

fn arguments<'a>(line: &'a str, command: &str, count: usize) -> Result<Vec<&'a str>> {
    let values: Vec<_> = line
        .strip_prefix(command)
        .context("missing source command")?
        .trim()
        .split(',')
        .map(str::trim)
        .collect();
    ensure!(
        values.len() == count && values.iter().all(|v| !v.is_empty()),
        "invalid {command} arguments: {line}"
    );
    Ok(values)
}

fn integer(value: &str) -> Result<i32> {
    let (negative, value) = value
        .strip_prefix('-')
        .map_or((false, value), |v| (true, v));
    let number = if let Some(v) = value.strip_prefix('$') {
        i32::from_str_radix(v, 16)
    } else if let Some(v) = value.strip_prefix('%') {
        i32::from_str_radix(v, 2)
    } else {
        value.parse()
    }
    .with_context(|| format!("unsupported source integer {value}"))?;
    Ok(if negative { -number } else { number })
}

/// Read the four authored map tables and exact INCBIN layouts from a pret root.
pub fn read(root: &Path) -> Result<BTreeMap<String, SourceMapGeometry>> {
    let root = root
        .canonicalize()
        .context("resolve external map source root")?;
    let load = |path: &str| {
        std::fs::read_to_string(root.join(path))
            .with_context(|| format!("read source map table {path}"))
    };
    let constants = load("constants/map_constants.asm")?;
    let mut dimensions = BTreeMap::new();
    let mut group = String::new();
    for line in statements(&constants) {
        if let Some(name) = line.strip_prefix("newgroup ") {
            group = format!("MAPGROUP_{}", name.trim());
        }
        if line.starts_with("map_const ") {
            let a = arguments(line, "map_const", 3)?;
            let width = u16::try_from(integer(a[1])?)?;
            let height = u16::try_from(integer(a[2])?)?;
            ensure!(
                width > 0 && height > 0 && !group.is_empty(),
                "invalid dimensions for {}",
                a[0]
            );
            ensure!(
                dimensions
                    .insert(a[0].to_owned(), (width, height, group.clone()))
                    .is_none(),
                "duplicate map constant {}",
                a[0]
            );
        }
    }
    let headers = load("data/maps/maps.asm")?;
    let mut metadata = BTreeMap::new();
    for line in statements(&headers) {
        if line.starts_with("map ") {
            let a = arguments(line, "map", 8)?;
            ensure!(
                metadata
                    .insert(
                        a[0].to_owned(),
                        a.iter().skip(1).map(|s| s.to_string()).collect::<Vec<_>>()
                    )
                    .is_none(),
                "duplicate map header {}",
                a[0]
            );
        }
    }
    let attributes = load("data/maps/attributes.asm")?;
    let mut maps = BTreeMap::new();
    let mut constant_maps = BTreeMap::new();
    let mut connection_constants = Vec::new();
    let mut active = None;
    for line in statements(&attributes) {
        if line.starts_with("map_attributes ") {
            let a = arguments(line, "map_attributes", 3)?;
            let m = metadata
                .get(a[0])
                .with_context(|| format!("missing map header {}", a[0]))?;
            let (width, height, group) = dimensions
                .get(a[1])
                .with_context(|| format!("missing map dimensions {}", a[1]))?;
            ensure!(
                constant_maps
                    .insert(a[1].to_owned(), a[0].to_owned())
                    .is_none(),
                "duplicate map identity {}",
                a[1]
            );
            let phone = match m[4].as_str() {
                "TRUE" => 1,
                "FALSE" => 0,
                other => anyhow::bail!("unknown phone flag {other}"),
            };
            // maps.asm encodes `dn phone, palette`; AUTO follows the live clock.
            let (palette_id, time_of_day) = match m[5].as_str() {
                "PALETTE_AUTO" => (0, None),
                "PALETTE_DAY" => (1, Some("day")),
                "PALETTE_NITE" => (2, Some("nite")),
                "PALETTE_MORN" => (3, Some("morn")),
                "PALETTE_DARK" => (4, Some("dark")),
                other => anyhow::bail!("unknown palette constant {other}"),
            };
            let map = SourceMapGeometry {
                attributes: MapAttributes {
                    tileset_name: m[0]
                        .strip_prefix("TILESET_")
                        .context("invalid tileset constant")?
                        .to_ascii_lowercase(),
                    border_block: u8::try_from(integer(a[2])?)?,
                    width: *width,
                    height: *height,
                    connections: vec![],
                    time_of_day: time_of_day.map(str::to_owned),
                    phone_service: (phone << 4) | palette_id,
                    phone_flag: phone != 0,
                    environment: Some(m[1].clone()),
                    location: Some(m[2].clone()),
                    music: Some(m[3].clone()),
                    palette: Some(m[5].clone()),
                    fishing_group: Some(m[6].clone()),
                    map_constant: Some(a[1].into()),
                    map_group_constant: Some(group.clone()),
                    blocks_label: Some(format!("{}_Blocks", a[0])),
                    map_scripts_label: Some(format!("{}_MapScripts", a[0])),
                    map_events_label: Some(format!("{}_MapEvents", a[0])),
                    connection_flags: None,
                },
                blocks: vec![],
            };
            ensure!(
                maps.insert(a[0].to_owned(), map).is_none(),
                "duplicate map attributes {}",
                a[0]
            );
            active = Some(a[0].to_owned());
        } else if line.starts_with("connection ") {
            let a = arguments(line, "connection", 4)?;
            ensure!(
                matches!(a[0], "north" | "south" | "west" | "east"),
                "invalid connection direction {}",
                a[0]
            );
            ensure!(
                dimensions.contains_key(a[2]),
                "missing connection constant {}",
                a[2]
            );
            ensure!(
                metadata.contains_key(a[1]),
                "missing connection target {}",
                a[1]
            );
            connection_constants.push((a[1].to_owned(), a[2].to_owned()));
            let map = maps
                .get_mut(
                    active
                        .as_ref()
                        .context("connection without map attributes")?,
                )
                .unwrap();
            ensure!(
                !map.attributes
                    .connections
                    .iter()
                    .any(|c| c.direction == a[0]),
                "duplicate connection direction"
            );
            map.attributes.connections.push(MapConnection {
                direction: a[0].into(),
                target_map: a[1].into(),
                offset: integer(a[3])?,
            });
        }
    }
    ensure!(
        maps.len() == metadata.len() && maps.len() == dimensions.len(),
        "map header/attribute/dimension catalogs differ"
    );
    for (name, constant) in connection_constants {
        ensure!(
            constant_maps.get(&constant) == Some(&name),
            "connection target name/constant mismatch for {name}"
        );
    }
    let block_table = load("data/maps/blocks.asm")?;
    let mut labels = Vec::new();
    let mut blocks = BTreeMap::new();
    for line in statements(&block_table) {
        if line.ends_with(':') {
            labels.push(line.trim_end_matches(':').to_owned());
        } else if let Some(value) = line.strip_prefix("INCBIN ") {
            let relative = value
                .trim()
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .context("invalid block INCBIN")?;
            let path = Path::new(relative);
            ensure!(
                path.components().all(|c| matches!(c, Component::Normal(_))),
                "block path escapes source root"
            );
            let path = root
                .join(path)
                .canonicalize()
                .context("resolve authored block layout")?;
            ensure!(path.starts_with(&root), "block path escapes source root");
            let bytes = std::fs::read(&path)
                .with_context(|| format!("read block layout {}", path.display()))?;
            for label in labels.drain(..) {
                ensure!(
                    blocks.insert(label.clone(), bytes.clone()).is_none(),
                    "duplicate block label {label}"
                );
            }
        }
    }
    for (name, map) in &mut maps {
        let data = blocks
            .get(map.attributes.blocks_label.as_ref().unwrap())
            .with_context(|| format!("missing block layout {name}"))?;
        ensure!(
            data.len() == usize::from(map.attributes.width) * usize::from(map.attributes.height),
            "block count differs from dimensions for {name}"
        );
        map.blocks = data.iter().copied().map(u16::from).collect();
        if !map.attributes.connections.is_empty() {
            map.attributes.connection_flags = Some(
                map.attributes
                    .connections
                    .iter()
                    .map(|c| c.direction.to_ascii_uppercase())
                    .collect::<Vec<_>>()
                    .join(" | "),
            );
        }
    }
    ensure!(!maps.is_empty(), "empty source map catalog");
    Ok(maps)
}
