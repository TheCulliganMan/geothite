//! Full inverse, native-wire and complete exterior geometry checks.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use super::{BOUNDARY, CEDAR, COAST, LOWER, OWNED, RETIRED, TOWN, UPPER};
use crate::{CompiledGamePack, MapModule};

pub(super) fn validate(
    base: &CompiledGamePack,
    output: &CompiledGamePack,
    revised: &BTreeMap<String, MapModule>,
) -> Result<()> {
    ensure!(
        output.data.maps.len() == 689 && RETIRED.iter().all(|n| !output.data.maps.contains_key(*n)),
        "compact map retirement differs"
    );
    for (name, expected) in revised {
        ensure!(
            output.data.maps.get(name) == Some(expected),
            "compact native module differs: {name}"
        );
    }
    inverse(base, output)?;
    for name in OWNED.into_iter().filter(|n| !RETIRED.contains(n)) {
        let module = &output.data.maps[name];
        let constant = module
            .attributes
            .map_constant
            .as_ref()
            .context("compact constant")?;
        let before = &base.data.runtime_map_metadata[constant];
        let after = &output.data.runtime_map_metadata[constant];
        ensure!(
            before.group_id == after.group_id && before.map_id == after.map_id,
            "retained native numeric identity changed"
        );
        let mut old_wire = base.data.clone();
        old_wire.maps.remove(name);
        let mut baseline = old_wire.assemble_map_module_from_compiled_payloads(name)?;
        let mut old_cached = base.data.maps[name].clone();
        let mut uncached = output.data.clone();
        uncached.maps.remove(name);
        let mut rebuilt = uncached.assemble_map_module_from_compiled_payloads(name)?;
        let mut expected = module.clone();
        for map in [&mut baseline, &mut old_cached, &mut rebuilt, &mut expected] {
            for label in map
                .attributes
                .map_scripts_label
                .iter()
                .chain(map.attributes.map_events_label.iter())
            {
                map.scripts.remove(label);
            }
        }
        let baseline = normalize(&baseline)?;
        let before = normalize(&old_cached)?;
        let after = normalize(&expected)?;
        if std::env::var_os("CRYSTAL_COMPACT_WIRE_TRACE").is_some() && baseline != before {
            eprintln!(
                "compact inherited wire/cache difference {name}: {}",
                first_difference(&baseline, &before, "/").unwrap_or_default()
            );
        }
        let mut expected_wire = baseline;
        apply_wire_delta(&mut expected_wire, &before, &after)?;
        let actual = normalize(&rebuilt)?;
        ensure!(
            actual == expected_wire,
            "compact cached/uncached behavioral delta differs: {name}: {}",
            first_difference(&actual, &expected_wire, "/").unwrap_or_default()
        );
    }
    graph(output)?;
    water_boundaries(output)?;
    ports(output)?;
    Ok(())
}
fn normalize(map: &MapModule) -> Result<Value> {
    let mut value = serde_json::to_value(map)?;
    for (key, field) in value.as_object_mut().context("module value")? {
        if key.starts_with("script_")
            && let Some(rows) = field.as_array_mut()
        {
            rows.sort_by_key(Value::to_string);
        }
    }
    Ok(value)
}
fn restore_key(target: &mut Value, original: &Value, field: &str, key: &str) -> Result<()> {
    let table = target["data"][field]
        .as_object_mut()
        .with_context(|| format!("native wire table {field}"))?;
    if let Some(value) = original["data"][field].get(key) {
        table.insert(key.into(), value.clone());
    } else {
        table.remove(key);
    }
    Ok(())
}
fn inverse(base: &CompiledGamePack, output: &CompiledGamePack) -> Result<()> {
    let original = serde_json::to_value(base)?;
    let mut restored = serde_json::to_value(output)?;
    let mut labels = BTreeSet::new();
    for name in OWNED {
        let map = &base.data.maps[name];
        for field in ["maps", "map_attributes", "npcs", "wild_encounters"] {
            restore_key(&mut restored, &original, field, name)?;
        }
        let constant = map
            .attributes
            .map_constant
            .as_ref()
            .context("inverse constant")?;
        for field in ["map_dimensions", "runtime_map_metadata"] {
            restore_key(&mut restored, &original, field, constant)?;
        }
        restore_key(
            &mut restored,
            &original,
            "map_blocks",
            map.attributes
                .blocks_label
                .as_ref()
                .context("inverse block label")?,
        )?;
        let landmark = original["data"]["pokegear_landmarks"]["map_to_landmark"]
            .get(name)
            .cloned();
        let table = restored["data"]["pokegear_landmarks"]["map_to_landmark"]
            .as_object_mut()
            .context("landmark table")?;
        if let Some(value) = landmark {
            table.insert(name.into(), value);
        } else {
            table.remove(name);
        }
        labels.extend(map.scripts.keys().cloned());
        labels.extend(map.script_text_bodies.keys().cloned());
        labels.extend(map.attributes.map_scripts_label.iter().cloned());
        labels.extend(map.attributes.map_events_label.iter().cloned());
    }
    let transports = OWNED
        .iter()
        .flat_map(|n| {
            base.data.maps[*n]
                .attributes
                .map_scripts_label
                .iter()
                .chain(base.data.maps[*n].attributes.map_events_label.iter())
        })
        .collect::<BTreeSet<_>>();
    for label in &labels {
        if !transports.contains(label) {
            ensure!(
                output.data.map_scripts.get(label) == base.data.map_scripts.get(label),
                "existing native program/text changed: {label}"
            );
        }
        restore_key(&mut restored, &original, "map_scripts", label)?;
    }
    restore_key(&mut restored, &original, "tilesets", "mn_winter_gunflint")?;
    for path in super::art::paths() {
        restored["runtime_files"][&path] = original["runtime_files"][&path].clone();
    }
    let mut expected_report = base.report.clone();
    expected_report.maps = 689;
    expected_report
        .reachable_maps
        .retain(|n| !RETIRED.contains(&n.as_str()));
    expected_report
        .solvable_maps
        .retain(|n| !RETIRED.contains(&n.as_str()));
    ensure!(
        output.report == expected_report,
        "compact report differs outside explicit retirement"
    );
    restored["report"] = original["report"].clone();
    restored["identity"] = original["identity"].clone();
    ensure!(
        restored == original,
        "compact revision changed native wire/art/audio/recovery outside exact scope"
    );
    Ok(())
}
fn graph(pack: &CompiledGamePack) -> Result<()> {
    let maps = &pack.data.maps;
    let mut positions = BTreeMap::from([("MnMinneapolis".to_string(), [0i32, 0i32])]);
    let mut queue = VecDeque::from(["MnMinneapolis".to_string()]);
    while let Some(name) = queue.pop_front() {
        let a = &maps[&name].attributes;
        let [x, y] = positions[&name];
        ensure!(
            a.connections
                .iter()
                .map(|c| &c.direction)
                .collect::<BTreeSet<_>>()
                .len()
                == a.connections.len(),
            "duplicate compact connection direction"
        );
        for edge in &a.connections {
            let b = &maps
                .get(&edge.target_map)
                .context("dangling compact connection")?
                .attributes;
            let reverse = match edge.direction.as_str() {
                "north" => "south",
                "south" => "north",
                "east" => "west",
                "west" => "east",
                _ => anyhow::bail!("invalid connection direction"),
            };
            ensure!(
                b.connections.iter().any(|c| c.direction == reverse
                    && c.target_map == name
                    && c.offset == -edge.offset),
                "missing reciprocal compact connection"
            );
            let offset = i32::from(edge.offset);
            let next = match edge.direction.as_str() {
                "north" => [x + offset, y - i32::from(b.height)],
                "south" => [x + offset, y + i32::from(a.height)],
                "west" => [x - i32::from(b.width), y + offset],
                "east" => [x + i32::from(a.width), y + offset],
                _ => unreachable!("direction checked"),
            };
            if let Some(old) = positions.get(&edge.target_map) {
                ensure!(*old == next, "compact graph has an inconsistent cycle");
            } else {
                positions.insert(edge.target_map.clone(), next);
                queue.push_back(edge.target_map.clone());
            }
        }
    }
    for (name, expected) in [
        (TOWN, [226, -400]),
        (COAST, [222, -400]),
        (CEDAR, [222, -412]),
        (UPPER, [190, -432]),
        (LOWER, [190, -408]),
        (BOUNDARY, [158, -430]),
    ] {
        ensure!(
            positions.get(name) == Some(&expected),
            "compact derived position differs: {name}"
        );
    }
    let names = positions.keys().collect::<Vec<_>>();
    ensure!(
        names.len() == 96,
        "complete compact exterior graph count differs"
    );
    for (i, name) in names.iter().enumerate() {
        let [x, y] = positions[*name];
        let a = &maps[*name].attributes;
        for target in &names[i + 1..] {
            let [u, v] = positions[*target];
            let b = &maps[*target].attributes;
            ensure!(
                x.max(u) >= (x + i32::from(a.width)).min(u + i32::from(b.width))
                    || y.max(v) >= (y + i32::from(a.height)).min(v + i32::from(b.height)),
                "compact exteriors overlap: {name}/{target}"
            );
        }
    }
    for module in maps.values() {
        for warp in &module.events.warps {
            ensure!(
                !RETIRED
                    .iter()
                    .any(|n| base_constant(n) == warp.target_map_constant),
                "warp retains retired map target"
            );
        }
    }
    Ok(())
}
fn base_constant(name: &str) -> String {
    if name == super::NECK {
        "MN_MARAIS_INLAND_NECK".into()
    } else {
        name.replacen("MnR12C", "MN_R12_C", 1)
    }
}
fn ports(pack: &CompiledGamePack) -> Result<()> {
    let floor = |name: &str, at: [u16; 2]| {
        crate::authored_town_geometry::kind(pack, &pack.data.maps[name], at)
    };
    for y in [8, 9] {
        ensure!(
            floor(COAST, [7, y])? == "FLOOR" && floor(TOWN, [0, y])? == "FLOOR",
            "compact warm Foot seam differs"
        );
    }
    for y in 16..36 {
        ensure!(
            floor(COAST, [7, y])? == "WATER" && floor(TOWN, [0, y])? == "WATER",
            "compact shared Superior seam differs"
        );
    }
    for (a, b) in [([63, 6], [0, 14]), ([63, 7], [0, 15])] {
        ensure!(
            floor(LOWER, a)? == "FLOOR" && floor(CEDAR, b)? == "FLOOR",
            "compact Gunflint/Cedar Foot seam differs"
        );
    }
    for (a, b) in [([26, 23], [18, 0]), ([27, 23], [19, 0])] {
        ensure!(
            floor(CEDAR, a)? == "FLOOR" && floor(TOWN, b)? == "FLOOR",
            "compact Cedar/town Foot seam differs"
        );
    }
    Ok(())
}

fn water_boundaries(pack: &CompiledGamePack) -> Result<()> {
    for name in [COAST, super::LOWER_COAST, UPPER, LOWER, CEDAR] {
        let module = &pack.data.maps[name];
        let a = &module.attributes;
        let width = a.width * 2;
        let height = a.height * 2;
        for (direction, length) in [
            ("north", width),
            ("south", width),
            ("west", height),
            ("east", height),
        ] {
            for lane in 0..length {
                let tile = match direction {
                    "north" => [lane, 0],
                    "south" => [lane, height - 1],
                    "west" => [0, lane],
                    "east" => [width - 1, lane],
                    _ => unreachable!("fixed direction"),
                };
                if crate::authored_town_geometry::kind(pack, module, tile)? != "WATER" {
                    continue;
                }
                let connection = a
                    .connections
                    .iter()
                    .find(|c| c.direction == direction)
                    .with_context(|| format!("unconnected WATER edge {name} {direction} {lane}"))?;
                let target = &pack.data.maps[&connection.target_map].attributes;
                let mapped = i32::from(lane) - i32::from(connection.offset) * 2;
                let extent = if ["north", "south"].contains(&direction) {
                    target.width * 2
                } else {
                    target.height * 2
                };
                ensure!(
                    (0..i32::from(extent)).contains(&mapped),
                    "WATER edge overhangs clipped neighbor: {name} {direction} {lane}"
                );
            }
        }
    }
    Ok(())
}

// Preserve inherited parser/cache distinctions only where this revision made no
// change. Every changed native field must still match its exact authored delta.
fn apply_wire_delta(wire: &mut Value, before: &Value, after: &Value) -> Result<()> {
    if before == after {
        return Ok(());
    }
    if let (Some(old), Some(new)) = (before.as_object(), after.as_object()) {
        let target = wire
            .as_object_mut()
            .context("changed native wire object absent")?;
        for key in old.keys().chain(new.keys()).collect::<BTreeSet<_>>() {
            match (old.get(key), new.get(key)) {
                (Some(a), Some(b)) if a != b => {
                    let value = target
                        .get_mut(key)
                        .with_context(|| format!("changed native wire field absent: {key}"))?;
                    apply_wire_delta(value, a, b)?;
                }
                (None, Some(b)) => {
                    ensure!(
                        target.insert(key.clone(), b.clone()).is_none(),
                        "new native wire field occupied"
                    );
                }
                (Some(_), None) => {
                    ensure!(
                        target.remove(key).is_some(),
                        "removed native wire field absent"
                    );
                }
                _ => {}
            }
        }
    } else {
        *wire = after.clone();
    }
    Ok(())
}
fn first_difference(a: &Value, b: &Value, path: &str) -> Option<String> {
    if a == b {
        return None;
    }
    if let (Some(x), Some(y)) = (a.as_object(), b.as_object()) {
        for key in x.keys().chain(y.keys()).collect::<BTreeSet<_>>() {
            if let Some(diff) = first_difference(
                x.get(key).unwrap_or(&Value::Null),
                y.get(key).unwrap_or(&Value::Null),
                &format!("{path}{key}/"),
            ) {
                return Some(diff);
            }
        }
    }
    if let (Some(x), Some(y)) = (a.as_array(), b.as_array()) {
        for index in 0..x.len().max(y.len()) {
            if let Some(diff) = first_difference(
                x.get(index).unwrap_or(&Value::Null),
                y.get(index).unwrap_or(&Value::Null),
                &format!("{path}{index}/"),
            ) {
                return Some(diff);
            }
        }
    }
    Some(format!(
        "{path}: {:?} versus {:?}",
        a.to_string().chars().take(96).collect::<String>(),
        b.to_string().chars().take(96).collect::<String>()
    ))
}
