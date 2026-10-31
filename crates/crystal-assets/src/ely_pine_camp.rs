//! A sheltered, inhabited Ely clearing and a small woodland supply loop.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_route_design::Pickup;
use crate::authored_town_design::Actor;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnEly";
const FLAG: &str = "EVENT_MN_ELY_PINE_SUPPLY";
const MARKER: &str = "minnesota-ely-pine-camp-v1";

#[derive(Deserialize)]
struct Terrain {
    schema_version: u8,
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    border_block: u16,
    rows: Vec<String>,
}

impl Terrain {
    fn blocks(&self) -> Result<Vec<u16>> {
        ensure!(
            self.schema_version == 1
                && self.id == MAP
                && self.tileset == "mn_ely_snow"
                && self.dimensions_metatiles == [20, 18]
                && self.border_block == 131
                && self.rows.len() == 18,
            "complete current snowy Ely source required"
        );
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(words.len() == 20, "complete Ely columns required");
            for word in words {
                let block = u16::from_str_radix(word, 16).context("native snowy town metatile")?;
                ensure!(block <= 0x83, "existing Ely art required");
                blocks.push(block);
            }
        }
        Ok(blocks)
    }
}

#[derive(Deserialize)]
struct Activities {
    schema_version: u8,
    id: String,
    residents: Vec<Actor>,
    items: Vec<Pickup>,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Refines the installed snowy town using its native pines and original art.
///
/// Adds three ordinary residents and one once-only Full Restore in a northern
/// supply nook. Original League, clinic, mart, Fly callback, north/east links,
/// weather identity and all other maps remain intact. Existing saved terrain
/// overrides are not migrated. Publication and gameplay review are separate.
///
/// # Errors
/// Rejects stale parent/source bindings, duplicate activities, changed original
/// anchors, unsafe borders, blocked walking approaches or an invalid pack.
pub fn rebuild_ely_pine_camp(
    base: &CompiledGamePack,
    before: &str,
    after: &str,
    activities: &str,
    contract: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let binding: Value = serde_json::from_str(contract)?;
    let original = base.data.maps.get(MAP).context("installed snowy Ely")?;
    ensure!(
        binding["schema_version"] == 1
            && binding["map"] == MAP
            && binding["native_module_sha256"]
                == hash(&serde_json::to_vec(&serde_json::to_value(original)?)?)
            && binding["source_sha256"] == hash(after.as_bytes())
            && binding["activities_sha256"] == hash(activities.as_bytes())
            && binding["preserved_warps"] == serde_json::to_value(&original.events.warps)?
            && original.blocks == serde_json::from_str::<Terrain>(before)?.blocks()?
            && original.attributes.tileset_name == "mn_ely_snow"
            && original.attributes.width == 20
            && original.attributes.height == 18
            && original.attributes.border_block == 131
            && original.objects.len() == 1
            && original.objects[0].script == "MnElyRanger"
            && original.script_block_changes.is_empty()
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "complete current Ely preimage and reviewed sources required"
    );
    let source: Activities = serde_json::from_str(activities)?;
    ensure!(
        source.schema_version == 1
            && source.id == "MN_ELY_PINE_CAMP_V1"
            && source.residents.len() == 3
            && source.residents.iter().all(|a| a.service.is_none())
            && source.items.len() == 1
            && source.items[0].id == "MN_ELY_PINE_SUPPLY"
            && source.items[0].flag == FLAG
            && source.items[0].item == "FULL_RESTORE"
            && source.items[0].quantity == 1
            && source.items[0].tile == [35, 5],
        "reviewed residents and independent supply nook required"
    );
    let mut map = original.clone();
    map.blocks = serde_json::from_str::<Terrain>(after)?.blocks()?;
    // Native facades, original actor/signs and every active/passive return stay
    // on exactly the same metatiles, including their other three quadrants.
    let mut protected = original
        .events
        .warps
        .iter()
        .map(|w| [w.x / 2, w.y / 2])
        .collect::<BTreeSet<_>>();
    protected.extend(original.objects.iter().map(|o| [o.x / 2, o.y / 2]));
    protected.extend(original.events.bg_events.iter().map(|e| [e.x / 2, e.y / 2]));
    for y in 0..18 {
        for x in 0..20 {
            let index = y * 20 + x;
            if protected.contains(&[x as u16, y as u16])
                || !matches!(original.blocks[index], 0x0a | 0x31 | 0x83)
            {
                ensure!(
                    map.blocks[index] == original.blocks[index],
                    "original Ely anchor changed"
                );
            }
            ensure!(
                map.blocks[index] == original.blocks[index]
                    || matches!(map.blocks[index], 0x0a | 0x31 | 0x83),
                "refinement may only fit snow paths and pine windbreaks"
            );
        }
    }
    let mut pack = base.clone();
    crate::authored_city_expansion_events::flags(&mut pack, &[FLAG.into()])?;
    crate::authored_town_scripts::ordinary(&pack, &mut map, &source.residents, &[], "", "")?;
    crate::authored_city_expansion_events::activities(&mut pack, &mut map, &[], &source.items)?;
    let names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect::<BTreeMap<_, _>>();
    crate::authored_city_expansion_events::finish(&pack, &mut map, &names)?;
    for (name, program) in &original.scripts {
        ensure!(
            map.scripts.get(name) == Some(program),
            "original Ely program changed: {name}"
        );
    }
    validate(&pack, &map, &source)?;
    crate::authored_town_publication::replace(&mut pack, map)?;
    for (name, map) in &base.data.maps {
        if name != MAP {
            ensure!(
                pack.data.maps.get(name) == Some(map),
                "unrelated map changed: {name}"
            );
        }
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.data.trainers == base.data.trainers
            && pack.data.wild_encounters == base.data.wild_encounters
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio,
        "native art, wildlife, trainer teams and audio must remain exact"
    );
    pack.report.manifests.push(MARKER.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn validate(pack: &CompiledGamePack, map: &MapModule, source: &Activities) -> Result<()> {
    let mut floor = BTreeSet::new();
    let bodies = map
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    for y in 0..36 {
        for x in 0..40 {
            let kind = crate::authored_town_geometry::kind(pack, map, [x, y])?;
            if x == 0 || x == 39 || y == 0 || y == 35 {
                let port = y == 0 && matches!(x, 24 | 25) || x == 39 && matches!(y, 14 | 15);
                ensure!(
                    kind == if port { "FLOOR" } else { "WALL" },
                    "original town border required"
                );
            }
            if kind == "FLOOR" && !bodies.contains(&[x, y]) {
                floor.insert([x, y]);
            }
        }
    }
    let mut reached = BTreeSet::from([[24u16, 0]]);
    let mut queue = VecDeque::from([[24u16, 0]]);
    while let Some([x, y]) = queue.pop_front() {
        for next in [
            x.checked_sub(1).map(|x| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if floor.contains(&next) && reached.insert(next) {
                queue.push_back(next);
            }
        }
    }
    ensure!(
        reached == floor,
        "town floor must not leave isolated snow pockets"
    );
    for at in [
        [11, 22],
        [27, 28],
        [25, 14],
        [26, 14],
        [39, 14],
        [39, 15],
        [34, 5],
    ] {
        ensure!(reached.contains(&at), "dry Ely anchor unreachable: {at:?}");
    }
    for actor in &source.residents {
        let approach = actor.approach.context("explicit resident approach")?;
        ensure!(
            !floor.contains(&actor.tile)
                && crate::authored_town_geometry::kind(pack, map, actor.tile)? == "FLOOR"
                && reached.contains(&approach)
                && actor.tile[0].abs_diff(approach[0]) + actor.tile[1].abs_diff(approach[1]) == 1,
            "resident needs reachable ordinary ground and adjacent approach"
        );
    }
    for sign in &map.events.bg_events {
        for below in [1, 2] {
            ensure!(
                reached.contains(&[sign.x, sign.y + below]),
                "original sign needs its clear approach, including actor bodies"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "ely_pine_camp_tests.rs"]
mod tests;
