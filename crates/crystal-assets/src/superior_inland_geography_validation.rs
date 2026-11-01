//! Exact inverse residual and native geometry checks for the west-entry stage.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};

use super::{CEDAR, CONSTANT, Contract, FLAG, MANIFEST, NECK, OWNED, SIGN};
use crate::{CompiledGamePack, MapModule};

pub(super) fn validate(
    base: &CompiledGamePack,
    output: &CompiledGamePack,
    plan: &Contract,
) -> Result<()> {
    ensure!(
        output.data.maps.len() == base.data.maps.len() + 1,
        "one new inland map required"
    );
    let revised = super::revisions(base, plan)?;
    for (name, expected) in &revised {
        ensure!(
            output.data.maps.get(name) == Some(expected),
            "existing geography scope drift: {name}"
        );
        ensure!(
            output.data.map_attributes.get(name) == Some(&expected.attributes),
            "geography attribute mirror drift: {name}"
        );
        let label = expected
            .attributes
            .blocks_label
            .as_ref()
            .context("owned blocks")?;
        let old = &base.data.maps[name];
        let payload = if expected.blocks == old.blocks {
            base.data.map_blocks[label].clone()
        } else {
            super::encode(expected)?
        };
        ensure!(
            output.data.map_blocks.get(label) == Some(&payload),
            "geography block mirror drift"
        );
    }
    let expected_neck = super::new_map(output, plan)?;
    ensure!(
        output.data.maps.get(NECK) == Some(&expected_neck),
        "new inland module differs"
    );
    let mut expected_encounters = base.data.wild_encounters[CEDAR].clone();
    expected_encounters.map_name = NECK.into();
    ensure!(
        output.data.wild_encounters.get(NECK) == Some(&expected_encounters),
        "inland encounters differ beyond map name"
    );
    ensure!(
        output.data.story_event_script_constants.global.get(FLAG) == Some(&2387),
        "camp requires exactly its fresh persistent event"
    );
    ensure!(
        !output.data.runtime_spawn_points.contains_key("280"),
        "temporary recovery leaked"
    );
    let metadata = output
        .data
        .runtime_map_metadata
        .get(CONSTANT)
        .context("new map registration")?;
    ensure!(
        metadata.group_id == 40
            && metadata.map_id == 8
            && metadata.width == 65
            && metadata.height == 19
            && metadata.group_name == "MINNESOTA",
        "native connector registration differs"
    );
    let landmark = base.data.pokegear_landmark_for_map("MnGrandMarais")?;
    ensure!(
        output.data.pokegear_landmarks.map_to_landmark.get(NECK) == Some(&landmark.constant),
        "new map landmark differs"
    );
    let mut expected_report = base.report.clone();
    expected_report.maps += 1;
    expected_report.manifests.push(MANIFEST.into());
    expected_report.reachable_maps.push(NECK.into());
    expected_report.solvable_maps.push(NECK.into());
    ensure!(
        output.report == expected_report,
        "geography report changed beyond the new map"
    );
    let mut expected_sign = base.data.map_scripts[SIGN].clone();
    super::correct_directions(&mut expected_sign)?;
    ensure!(
        output.data.map_scripts.get(SIGN) == Some(&expected_sign),
        "global sign mirror differs"
    );

    // Restore the precise inherited entries, including Value-shaped NPC/script
    // payloads. Comparing the normalized parent to its own wire representation
    // avoids replacing old object wrappers with generated arrays as a side effect.
    let mut normalized = output.clone();
    for name in OWNED {
        let old = &base.data.maps[name];
        normalized.data.maps.insert(name.into(), old.clone());
        normalized
            .data
            .map_attributes
            .insert(name.into(), old.attributes.clone());
        let label = old
            .attributes
            .blocks_label
            .as_ref()
            .context("original block label")?;
        normalized
            .data
            .map_blocks
            .insert(label.clone(), base.data.map_blocks[label].clone());
    }
    normalized
        .data
        .map_scripts
        .insert(SIGN.into(), base.data.map_scripts[SIGN].clone());
    let new = normalized
        .data
        .maps
        .remove(NECK)
        .context("remove new map for inverse proof")?;
    normalized.data.map_attributes.remove(NECK);
    normalized.data.map_dimensions.remove(CONSTANT);
    normalized.data.runtime_map_metadata.remove(CONSTANT);
    normalized.data.map_blocks.remove(
        new.attributes
            .blocks_label
            .as_ref()
            .context("new block label")?,
    );
    normalized.data.npcs.remove(NECK);
    normalized.data.wild_encounters.remove(NECK);
    normalized
        .data
        .pokegear_landmarks
        .map_to_landmark
        .remove(NECK);
    normalized
        .data
        .story_event_script_constants
        .global
        .remove(FLAG);
    for label in new
        .scripts
        .keys()
        .chain(new.script_text_bodies.keys())
        .chain(new.attributes.map_scripts_label.iter())
        .chain(new.attributes.map_events_label.iter())
    {
        ensure!(
            !base.data.map_scripts.contains_key(label),
            "new map replaced original script label"
        );
        normalized.data.map_scripts.remove(label);
    }
    normalized.report = base.report.clone();
    normalized.identity = base.identity.clone();
    ensure!(
        serde_json::to_value(&normalized.data)? == serde_json::to_value(&base.data)?,
        "geography changed inherited native wire data outside its exact scope"
    );
    ensure!(
        normalized == *base,
        "geography changed unrelated art/audio/catalogs/recovery/report"
    );
    ensure!(
        output.identity == super::identity(output)?,
        "geography output identity stale"
    );

    uncached(base, output, plan)?;
    ports(output)?;
    exterior_geometry(output)?;
    Ok(())
}

fn uncached(base: &CompiledGamePack, output: &CompiledGamePack, plan: &Contract) -> Result<()> {
    let mut before = base.data.clone();
    let mut after = output.data.clone();
    for name in OWNED {
        before.maps.remove(name);
        after.maps.remove(name);
    }
    for name in OWNED {
        let original = before.assemble_map_module_from_compiled_payloads(name)?;
        let expected = super::revise(&original, plan)?;
        ensure!(
            after.assemble_map_module_from_compiled_payloads(name)? == expected,
            "existing geography uncached behavior differs: {name}"
        );
    }
    after.maps.remove(NECK);
    let mut rebuilt = after.assemble_map_module_from_compiled_payloads(NECK)?;
    let expected = &output.data.maps[NECK];
    for label in [
        &expected.attributes.map_scripts_label,
        &expected.attributes.map_events_label,
    ] {
        rebuilt
            .scripts
            .remove(label.as_ref().context("new map transport header")?);
    }
    ensure!(
        &rebuilt == expected,
        "new connector cached/uncached typed behavior differs"
    );
    Ok(())
}

fn ports(pack: &CompiledGamePack) -> Result<()> {
    for (name, expected) in [
        (NECK, BTreeSet::from([[0, 8], [0, 9], [129, 12], [129, 13]])),
        (
            CEDAR,
            BTreeSet::from([[0, 12], [0, 13], [56, 37], [57, 37]]),
        ),
    ] {
        let map = &pack.data.maps[name];
        let mut actual = BTreeSet::new();
        for y in 0..map.attributes.height * 2 {
            for x in 0..map.attributes.width * 2 {
                if x != 0
                    && y != 0
                    && x != map.attributes.width * 2 - 1
                    && y != map.attributes.height * 2 - 1
                {
                    continue;
                }
                if crate::authored_town_geometry::kind(pack, map, [x, y])? != "WALL" {
                    actual.insert([x, y]);
                }
            }
        }
        ensure!(actual == expected, "undeclared inland outer lane: {name}");
    }
    for y in 16..36 {
        ensure!(
            crate::authored_town_geometry::kind(pack, &pack.data.maps["MnR11C01R00"], [47, y])?
                == "WATER",
            "closed R11 east Superior aperture"
        );
    }
    for (name, tile) in [
        ("MnR11C01R00", [47, 8]),
        ("MnR11C01R00", [47, 9]),
        ("MnGrandMarais", [0, 8]),
        ("MnGrandMarais", [0, 9]),
    ] {
        ensure!(
            crate::authored_town_geometry::kind(pack, &pack.data.maps[name], tile)? == "FLOOR",
            "west coastal walking port differs"
        );
    }
    Ok(())
}

fn exterior_geometry(pack: &CompiledGamePack) -> Result<()> {
    let maps = pack
        .data
        .maps
        .iter()
        .filter(|(name, m)| {
            name.starts_with("Mn")
                && m.attributes.environment.as_deref().is_some_and(|e| {
                    matches!(
                        e.to_ascii_lowercase().as_str(),
                        "town" | "route" | "outdoor"
                    )
                })
        })
        .map(|(n, m)| (n.as_str(), m))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        maps.len() == 101,
        "exact101 Minnesota outdoor maps required"
    );
    let mut positions = BTreeMap::from([("MnMinneapolis", [0i32, 0i32])]);
    let mut queue = VecDeque::from(["MnMinneapolis"]);
    while let Some(name) = queue.pop_front() {
        let map = maps[name];
        let [x, y] = positions[name];
        let mut sides = BTreeSet::new();
        for edge in &map.attributes.connections {
            ensure!(
                sides.insert(edge.direction.as_str()),
                "duplicate native connection side"
            );
            let target = *maps
                .get(edge.target_map.as_str())
                .context("outdoor connection target")?;
            let (origin, opposite) = match edge.direction.as_str() {
                "north" => (
                    [x + edge.offset, y - i32::from(target.attributes.height)],
                    "south",
                ),
                "south" => (
                    [x + edge.offset, y + i32::from(map.attributes.height)],
                    "north",
                ),
                "west" => (
                    [x - i32::from(target.attributes.width), y + edge.offset],
                    "east",
                ),
                "east" => (
                    [x + i32::from(map.attributes.width), y + edge.offset],
                    "west",
                ),
                _ => anyhow::bail!("unknown connection direction"),
            };
            ensure!(
                super::has_connection(target, opposite, name, -edge.offset),
                "nonreciprocal geography connection"
            );
            if let Some(old) = positions.get(edge.target_map.as_str()) {
                ensure!(
                    *old == origin,
                    "inconsistent whole-state native connection cycle"
                );
            } else {
                let key = maps
                    .get_key_value(edge.target_map.as_str())
                    .context("outdoor name")?
                    .0;
                positions.insert(*key, origin);
                queue.push_back(*key);
            }
        }
    }
    ensure!(
        positions.len() == maps.len(),
        "disconnected native Minnesota exterior"
    );
    for (name, expected) in [
        ("MnGrandMarais", [246, -400]),
        (CEDAR, [227, -419]),
        (NECK, [162, -419]),
        ("MnEly", [71, -339]),
        ("MnBoundaryWaters", [66, -427]),
        ("MnDuluth", [181, -304]),
        ("MnR12C01R01", [130, -419]),
    ] {
        ensure!(
            positions.get(name) == Some(&expected),
            "native exterior anchor differs: {name}"
        );
    }
    let placed = positions.keys().copied().collect::<Vec<_>>();
    for (i, name) in placed.iter().copied().enumerate() {
        let [x, y] = positions[name];
        let first: &MapModule = maps[name];
        for other in placed[i + 1..].iter().copied() {
            let [xx, yy] = positions[other];
            let second: &MapModule = maps[other];
            ensure!(
                !(x < xx + i32::from(second.attributes.width)
                    && xx < x + i32::from(first.attributes.width)
                    && y < yy + i32::from(second.attributes.height)
                    && yy < y + i32::from(first.attributes.height)),
                "overlapping native exterior footprints: {name}/{other}"
            );
        }
    }
    Ok(())
}
