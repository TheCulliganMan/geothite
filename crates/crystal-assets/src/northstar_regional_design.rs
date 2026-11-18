//! Regional route, cave and discovery revisions over the released Minnesota story.
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_town_geometry::kind;
use crate::authored_town_publication;
use crate::{CompiledGamePack, LoadedCompiledGamePack, MapModule};

type Tile = [u16; 2];
const MANIFEST: &str = "northstar-regional-design-v1";
const POLICY: &str = "data/northstar-save-compatibility.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Room {
    map: String,
    #[serde(default)]
    blocks: Vec<Block>,
    #[serde(default)]
    caches: Vec<Cache>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    at: Tile,
    from: u16,
    to: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cache {
    script: String,
    from: Tile,
    to: Tile,
    entry: Tile,
    approach: Tile,
}

fn terrain(pack: &CompiledGamePack, room: &mut MapModule, blocks: &[Block]) -> Result<()> {
    let [w, h] = [room.attributes.width, room.attributes.height];
    let mut edited = BTreeSet::new();
    for b in blocks {
        let [x, y] = b.at;
        ensure!(
            x < w && y < h && edited.insert(b.at) && b.from != b.to,
            "invalid block at {:?}",
            b.at
        );
        let index = usize::from(y) * usize::from(w) + usize::from(x);
        ensure!(
            room.blocks.get(index) == Some(&b.from),
            "block preimage differs at {:?}",
            b.at
        );
        ensure!(
            !room
                .script_block_changes
                .iter()
                .any(|change| [change.x / 2, change.y / 2] == b.at),
            "scripted terrain at {:?}",
            b.at
        );
        let tiles = [
            [2 * x, 2 * y],
            [2 * x + 1, 2 * y],
            [2 * x, 2 * y + 1],
            [2 * x + 1, 2 * y + 1],
        ];
        let old: Vec<_> = tiles
            .iter()
            .map(|&p| kind(pack, room, p))
            .collect::<Result<_>>()?;
        room.blocks[index] = b.to;
        for (tile, before) in tiles.into_iter().zip(old) {
            let after = kind(pack, room, tile)?;
            // Retain all terrain on which a prior save can stand, surf, slide or hop.
            // Walkable ground can gain or lose grass without changing movement.
            ensure!(
                before == after
                    || before == "WALL"
                    || matches!(
                        (before, after),
                        ("FLOOR", "TALL_GRASS") | ("TALL_GRASS", "FLOOR")
                    ),
                "changed existing traversal at {tile:?}: {before} to {after}"
            );
        }
    }
    Ok(())
}

fn move_cache(pack: &CompiledGamePack, room: &mut MapModule, cache: &Cache) -> Result<()> {
    let [x, y] = cache.to;
    ensure!(
        cache.to != cache.from
            && u32::from(x.abs_diff(cache.approach[0])) + u32::from(y.abs_diff(cache.approach[1]))
                == 1
            && matches!(kind(pack, room, cache.approach)?, "FLOOR" | "TALL_GRASS")
            && !room
                .objects
                .iter()
                .any(|o| [o.x, o.y] == cache.to || [o.x, o.y] == cache.approach)
            && !room.events.bg_events.iter().any(|e| [e.x, e.y] == cache.to)
            && !room
                .events
                .warps
                .iter()
                .any(|e| [e.x, e.y] == cache.to || [e.x, e.y] == cache.approach),
        "cache approach for {}",
        cache.script
    );
    kind(pack, room, cache.to)?;
    let reached = crate::northstar_cave_geometry::resting_tiles(pack, room, cache.entry, false)?;
    ensure!(
        reached.contains(&cache.approach),
        "unreachable cache approach for {}",
        cache.script
    );
    let event = room
        .events
        .bg_events
        .iter_mut()
        .find(|ev| {
            ev.script == cache.script
                && ev.event_type == "BGEVENT_ITEM"
                && [ev.x, ev.y] == cache.from
        })
        .context("existing cache")?;
    let row = room
        .map_event_section_commands
        .iter_mut()
        .find(|r| r.command == "bg_event" && r.args.get(3) == Some(&cache.script))
        .context("published cache event")?;
    ensure!(
        row.args[..2] == [event.x.to_string(), event.y.to_string()]
            && room
                .script_field_pickups
                .iter()
                .filter(|p| p.source_script == cache.script && p.command == "hiddenitem")
                .count()
                == 1,
        "cache identity"
    );
    [event.x, event.y] = cache.to;
    row.args[0] = x.to_string();
    row.args[1] = y.to_string();
    Ok(())
}

// Exercise the production background-event resolver on the installed coordinates.
// Once its saved receipt is set the ordinary contextual field interaction is free.
fn check_cache_receipt(pack: &CompiledGamePack, map: &str, cache: &Cache) -> Result<()> {
    use crystal_core::state::GameState;
    use crystal_core::world::map::{Direction, TilePosition};
    use crystal_core::world::session::{OverworldInteraction, OverworldInteractionTarget};
    let position = |[x, y]: Tile| -> Result<TilePosition> {
        Ok(TilePosition::new(i16::try_from(x)?, i16::try_from(y)?))
    };
    let player = position(cache.approach)?;
    let target = position(cache.to)?;
    let facing = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ]
    .into_iter()
    .find(|&d| player.moved(d) == target)
    .context("adjacent cache")?;
    let event = OverworldInteraction {
        map_name: map.to_owned(),
        player_tile: player,
        target_tile: target,
        facing,
        script: cache.script.clone(),
        target: OverworldInteractionTarget::Background {
            event_type: "BGEVENT_ITEM".into(),
        },
    };
    let mut state = GameState::reset_wram_for_new_game();
    ensure!(
        pack.data
            .resolve_overworld_interaction(&state, &event)?
            .is_some(),
        "unclaimed cache is unavailable"
    );
    let flag = pack.data.maps[map]
        .script_field_pickups
        .iter()
        .find(|p| p.source_script == cache.script)
        .and_then(|p| p.event_flag.as_deref())
        .context("installed cache receipt")?;
    state.flags.set_event_flag(flag, true)?;
    ensure!(
        pack.data
            .resolve_overworld_interaction(&state, &event)?
            .is_none(),
        "claimed cache intercepts the field action"
    );
    Ok(())
}

/// Builds the audited regional revision from the exact released story and hosted packs.
///
/// # Errors
/// Rejects mismatched parents, terrain or event preimages, unsafe changes to
/// previous traversal, and invalid native content.
pub fn revise_northstar_regional_design(
    raw: &LoadedCompiledGamePack,
    hosted: &LoadedCompiledGamePack,
    root: &Path,
) -> Result<CompiledGamePack> {
    let raw_id = raw.save_modpack_identity()?;
    let hosted_id = hosted.save_modpack_identity()?;
    ensure!(
        raw_id.hash() == "02e18ba71203d8aa2e10658c6babc53512daeef592ee53fb2900630f91aa57cb"
            && hosted_id.hash()
                == "f4dd4819f039631c81372078d8640f7929114d46bdef90f5ae90d42612a789a0",
        "expected the released Minnesota story pack pair"
    );
    let mut pack = raw.pack().clone();
    crate::northstar_regional_art::install(&mut pack)?;
    let dir = root.join("revisions/regional-design-v01");
    for file in ["caches.json", "roads.json", "caves.json"] {
        let rooms: Vec<Room> = serde_json::from_slice(&fs::read(dir.join(file))?)?;
        ensure!(!rooms.is_empty(), "no designs in {file}");
        let mut ids = BTreeSet::new();
        for patch in rooms {
            ensure!(
                patch.map.starts_with("Mn")
                    && ids.insert(patch.map.clone())
                    && !(patch.blocks.is_empty() && patch.caches.is_empty()),
                "empty or duplicate room in {file}"
            );
            let mut room = pack
                .data
                .maps
                .get(&patch.map)
                .context("Minnesota room")?
                .clone();
            terrain(&pack, &mut room, &patch.blocks)
                .with_context(|| format!("terrain in {}", patch.map))?;
            for cache in &patch.caches {
                move_cache(&pack, &mut room, cache)?;
            }
            crate::northstar_cave_overhaul::validate_native_art(&pack, &room)?;
            authored_town_publication::replace(&mut pack, room)?;
            for cache in &patch.caches {
                check_cache_receipt(&pack, &patch.map, cache)?;
            }
        }
    }
    crate::northstar_carrier_ledge::install(&mut pack)?;
    crate::northstar_road_trainers::install(&mut pack, &dir)?;
    crate::northstar_fair_circulation::install(&mut pack, &dir)?;
    pack.report.manifests.push(MANIFEST.into());
    let inherited: Value = serde_json::from_slice(
        pack.runtime_files
            .get(POLICY)
            .context("story save policy")?,
    )?;
    ensure!(
        inherited["schema_version"] == 4
            && inherited["target_manifest"] == "northstar-story-flow-v1",
        "unexpected story save policy"
    );
    let id = pack.runtime_modpack_id()?;
    let metadata = json!({
        "schema_version":5, "target_manifest":MANIFEST,
        "target_modpack_ids":[id, format!("{id}+realtime-clock+player-customization")],
        "sources":[
            {"modpack_id":raw_id.id(),"modpack_hash":raw_id.hash(),"content_hash":raw.pack_content_hash()?},
            {"modpack_id":hosted_id.id(),"modpack_hash":hosted_id.hash(),"content_hash":hosted.pack_content_hash()?}
        ], "inherited":inherited
    });
    pack.runtime_files
        .insert(POLICY.into(), serde_json::to_vec(&metadata)?);
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
