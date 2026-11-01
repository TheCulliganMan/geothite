//! Guarded installation of the optional three-wake investigation on the river.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc as arc;

const MANIFEST: &str = "minnesota-pepin-three-wakes-v1";

/// Adds two native shoreline people, two observations and optional Pepivern.
/// All existing terrain, connections, actors, events and programs are retained.
/// No custom props, new maps, paid admission or main-story gate are introduced.
/// The caller must verify actual land/Surf access, dialogue, capture/storage,
/// return journeys and native presentation before selecting the returned pack.
///
/// # Errors
/// Rejects duplicate publication, noncanonical sites, missing original species
/// or field art, changed collisions, occupied anchors, script collisions, and
/// invalid compiled output. Neither the input pack nor disk is mutated.
pub fn build_pepin_discovery_pack(
    base: &CompiledGamePack,
    quests: &str,
    site: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|marker| marker == MANIFEST),
        "Pepin investigation already installed"
    );
    let source: Value = serde_json::from_str(site)?;
    ensure!(
        source["schema_version"] == 1
            && source["id"] == "minnesota-pepin-three-wakes-site-v1"
            && source["terrain_changes"] == false
            && source["custom_art"] == false,
        "canonical native shoreline contract required"
    );
    ensure!(
        base.data.pokemon.contains_key("PEPIVERN")
            && base
                .data
                .story_event_script_constants
                .global
                .contains_key("MN_DEX_MEMBER_PEPIVERN")
            && base
                .runtime_files
                .contains_key("gfx/pokemon/pepivern/overworld.2bpp"),
        "active original Pepivern and native field art required"
    );
    let programs = crate::build_pepin_discovery_programs(quests)?;
    let mut pack = base.clone();
    for flag in &programs.event_flags {
        a::flag(&mut pack, flag)?;
    }
    let mut maps = BTreeMap::new();
    let mut roots: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (key, expected_map, expected_at, expected_approach, collision, label, sprite) in [
        (
            "observer",
            "MnMississippiLowerWoodsC00R01",
            [22, 10],
            [22, 11],
            "FLOOR",
            "MnPepinDock",
            Some("SPRITE_FISHER"),
        ),
        (
            "eddy",
            "MnMississippiLowerWoodsC00R01",
            [24, 10],
            [23, 10],
            "WATER",
            "MnPepinEddy",
            None,
        ),
        (
            "operator",
            "MnMississippiLowerWoodsC01R01",
            [50, 16],
            [51, 16],
            "FLOOR",
            "MnPepinOperator",
            Some("SPRITE_SAILOR"),
        ),
        (
            "reeds",
            "MnMississippiLowerWoodsC01R01",
            [50, 20],
            [49, 20],
            "WALL",
            "MnPepinReeds",
            None,
        ),
        (
            "pepivern",
            "MnMississippiLowerWoodsC01R00",
            [24, 21],
            [24, 22],
            "WATER",
            "MN_PEPIN_PEPIVERN",
            Some("SPRITE_PEPIVERN"),
        ),
    ] {
        let anchor = &source["anchors"][key];
        let at: [u16; 2] = serde_json::from_value(anchor["at"].clone())?;
        let approach: [u16; 2] = serde_json::from_value(anchor["approach"].clone())?;
        ensure!(
            anchor["map"] == expected_map && at == expected_at && approach == expected_approach,
            "reviewed Pepin anchor changed: {key}"
        );
        ensure!(
            at[0].abs_diff(approach[0]) + at[1].abs_diff(approach[1]) == 1,
            "adjacent observation approach required"
        );
        if !maps.contains_key(expected_map) {
            maps.insert(expected_map.to_string(), arc::map(base, expected_map)?);
        }
        let map = maps.get_mut(expected_map).context("bound river map")?;
        ensure!(
            map.attributes.tileset_name == "johto"
                && map.attributes.width == 32
                && map.attributes.height == 24,
            "reviewed river dimensions required"
        );
        ensure!(
            crate::authored_town_geometry::kind(base, map, at)? == collision,
            "river anchor collision changed: {key}"
        );
        let expected_approach_kind = if key == "reeds" || key == "pepivern" {
            "WATER"
        } else {
            "FLOOR"
        };
        ensure!(
            crate::authored_town_geometry::kind(base, map, approach)? == expected_approach_kind,
            "river approach collision changed: {key}"
        );
        for point in [at, approach] {
            ensure!(
                !map.objects
                    .iter()
                    .any(|object| [object.x, object.y] == point)
                    && !map
                        .events
                        .warps
                        .iter()
                        .any(|event| [event.x, event.y] == point)
                    && !map
                        .events
                        .coord_events
                        .iter()
                        .any(|event| [event.x, event.y] == point)
                    && !map
                        .events
                        .bg_events
                        .iter()
                        .any(|event| [event.x, event.y] == point),
                "occupied Pepin anchor/approach: {key}"
            );
        }
        if let Some(sprite) = sprite {
            a::actor(base, map, label, sprite, at)?;
        } else {
            a::read(map, at, label);
        }
        roots
            .entry(expected_map.into())
            .or_default()
            .push(label.into());
    }
    let creature_map = source["callbacks_map"]
        .as_str()
        .context("creature callback map")?;
    ensure!(
        creature_map == "MnMississippiLowerWoodsC01R00",
        "reviewed creature map required"
    );
    let room = maps.get_mut(creature_map).context("creature map")?;
    for (kind, label) in ["MAPCALLBACK_OBJECTS", "MAPCALLBACK_NEWMAP"]
        .into_iter()
        .zip(&programs.callbacks)
    {
        a::callback(room, kind, label);
        roots
            .entry(creature_map.into())
            .or_default()
            .push(label.clone());
    }
    for (name, mut map) in maps {
        // Copy only the reachable local graph. Other chunks must not acquire
        // an unused wild-battle definition merely because they host a clue.
        let labels = crate::discovery_dialogue::script_closure(&programs.scripts, &roots[&name])?;
        for label in labels {
            ensure!(
                !map.scripts.contains_key(&label),
                "occupied Pepin script: {name}/{label}"
            );
            map.scripts
                .insert(label.clone(), programs.scripts[&label].clone());
        }
        map.scripted_wild_battles = crate::parse_scripted_wild_battles(&name, &map.scripts)?;
        let original = base.data.maps.get(&name).context("original river map")?;
        ensure!(
            map.attributes == original.attributes && map.blocks == original.blocks,
            "Pepin may not alter river terrain or geography"
        );
        arc::publish(&mut pack, map)?;
    }
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    Ok(pack)
}
