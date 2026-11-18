//! Small native terrain clues for the existing hidden discoveries on Routes 7–11.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_route_surface::walking_map_distances;
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, authored_town_publication, northstar_trail_caches};

const CONTRACTS: &str =
    include_str!("../../../planning/minnesota/discoveries/route-07-11-clues-v01/caches.json");
const ART: [&[u8]; 5] = [
    include_bytes!("../../../planning/minnesota/discoveries/route-07-11-clues-v01/r07-art.json"),
    include_bytes!("../../../planning/minnesota/discoveries/route-07-11-clues-v01/r08-art.json"),
    include_bytes!("../../../planning/minnesota/discoveries/route-07-11-clues-v01/r09-art.json"),
    include_bytes!("../../../planning/minnesota/discoveries/route-07-11-clues-v01/r10-art.json"),
    include_bytes!("../../../planning/minnesota/discoveries/route-07-11-clues-v01/r11-art.json"),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cache {
    route: u8,
    map: String,
    tile: [u16; 2],
    approach: [u16; 2],
    from_block: u16,
    to_block: u16,
    first_glyph: u8,
    item: String,
    script: String,
    flag: String,
}

/// Adds original terrain clues without moving or replacing any collectible.
///
/// The release owner must finalize the resulting pack's identity after composing
/// this leaf with other revisions. Existing maps, flags and item rewards retain
/// their identities; this function does not select or publish a pack.
///
/// # Errors
/// Returns an error if a source tile, pickup, free art slot or accessible approach
/// differs from the reviewed parent. Earlier edits may remain on error; discard
/// the candidate in that case.
pub fn install_minnesota_hidden_route_clues(pack: &mut CompiledGamePack) -> Result<()> {
    let caches: Vec<Cache> = serde_json::from_str(CONTRACTS)?;
    ensure!(caches.len() == ART.len(), "five route clues required");
    for (cache, art) in caches.iter().zip(ART) {
        ensure!((7..=11).contains(&cache.route), "route outside owned range");
        let mut map = pack
            .data
            .maps
            .get(&cache.map)
            .context("clue route")?
            .clone();
        let event = map
            .events
            .bg_events
            .iter()
            .find(|e| e.script == cache.script)
            .context("original hidden event")?;
        ensure!(
            event.event_type == "BGEVENT_ITEM" && [event.x, event.y] == cache.tile,
            "hidden event identity differs"
        );
        let pickup = map
            .script_field_pickups
            .iter()
            .find(|p| p.source_script == cache.script)
            .context("original hidden pickup")?;
        ensure!(
            pickup.command == "hiddenitem"
                && pickup.quantity == 1
                && pickup.item_id.as_deref() == Some(cache.item.as_str())
                && pickup.event_flag.as_deref() == Some(cache.flag.as_str())
                && pack.data.items.contains_key(&cache.item),
            "original reward or flag differs"
        );
        let [x, y] = cache.tile;
        let [ax, ay] = cache.approach;
        ensure!(
            x.abs_diff(ax) + y.abs_diff(ay) == 1
                && matches!(kind(pack, &map, cache.approach)?, "FLOOR" | "TALL_GRASS"),
            "clue requires an adjacent standing square"
        );
        let occupied: BTreeSet<_> = map.objects.iter().map(|o| [o.x, o.y]).collect();
        ensure!(
            !occupied.contains(&cache.approach)
                && !map
                    .events
                    .warps
                    .iter()
                    .any(|e| [e.x, e.y] == cache.approach)
                && !map
                    .events
                    .coord_events
                    .iter()
                    .any(|e| [e.x, e.y] == cache.approach),
            "clue approach is occupied"
        );
        // Check both directions: a one-way ledge must not make an isolated
        // approach look reachable merely because the player can leave it.
        let reach = walking_map_distances(pack, &map, cache.approach, &occupied)?;
        let seam = reach
            .keys()
            .find(|p| {
                map.attributes
                    .connections
                    .iter()
                    .any(|c| match c.direction.as_str() {
                        "north" => p[1] == 0,
                        "south" => p[1] + 1 == map.attributes.height * 2,
                        "west" => p[0] == 0,
                        "east" => p[0] + 1 == map.attributes.width * 2,
                        _ => false,
                    })
            })
            .context("clue approach cannot reach a route seam")?;
        ensure!(
            walking_map_distances(pack, &map, *seam, &occupied)?.contains_key(&cache.approach),
            "route seam cannot reach clue approach"
        );
        let index = usize::from(y / 2) * usize::from(map.attributes.width) + usize::from(x / 2);
        ensure!(
            map.blocks[index] == cache.from_block,
            "clue terrain preimage differs"
        );
        northstar_trail_caches::install_landmark_art_at(pack, art, cache.first_glyph)?;
        let collision = &pack.data.tilesets[&map.attributes.tileset_name].collision;
        ensure!(
            collision[&format!("{:02x}", cache.from_block)]
                == collision[&format!("{:02x}", cache.to_block)],
            "clue changed collision"
        );
        map.blocks[index] = cache.to_block;
        authored_town_publication::replace(pack, map)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_core::state::GameState;
    use crystal_core::systems::field_items::{
        FieldItemPickupOutcome, find_itemfinder_hidden_item, pickup_field_item,
    };
    use crystal_core::world::map::{Direction, TilePosition};
    use crystal_core::world::session::OverworldSession;

    #[test]
    #[ignore = "requires the external selected Crossings pack"]
    fn route_clues_preserve_rewards_flags_and_native_itemfinder() -> Result<()> {
        let path = std::env::var_os("MN_CLUES_PARENT").context("set MN_CLUES_PARENT")?;
        let base = crate::read_verified_compiled_game_pack(path)?;
        let mut candidate = base.clone();
        install_minnesota_hidden_route_clues(&mut candidate)?;
        let caches: Vec<Cache> = serde_json::from_str(CONTRACTS)?;
        let owned: BTreeSet<_> = caches.iter().map(|c| c.map.as_str()).collect();
        for (name, map) in &base.data.maps {
            let revised = &candidate.data.maps[name];
            if !owned.contains(name.as_str()) {
                assert_eq!(map, revised, "unowned map changed: {name}");
            } else {
                let mut expected = map.clone();
                expected.blocks.clone_from(&revised.blocks);
                assert_eq!(&expected, revised, "non-art map data changed: {name}");
                assert_eq!(
                    map.blocks
                        .iter()
                        .zip(&revised.blocks)
                        .filter(|(a, b)| a != b)
                        .count(),
                    1
                );
            }
        }
        assert_eq!(base.audio_manifest, candidate.audio_manifest);
        for cache in caches {
            let map = &candidate.data.maps[&cache.map];
            let pickup = map
                .script_field_pickups
                .iter()
                .find(|p| p.source_script == cache.script)
                .context("installed pickup")?;
            let mut state = GameState::default();
            let at = TilePosition::new(cache.approach[0] as i16, cache.approach[1] as i16);
            let mut session = OverworldSession::with_events(
                candidate.data.overworld_map(&map.id)?,
                map.events.clone(),
                candidate
                    .data
                    .tileset_collision(&map.attributes.tileset_name)?,
                at,
            );
            session.player.facing = match (
                cache.tile[0].cmp(&cache.approach[0]),
                cache.tile[1].cmp(&cache.approach[1]),
            ) {
                (std::cmp::Ordering::Less, _) => Direction::Left,
                (std::cmp::Ordering::Greater, _) => Direction::Right,
                (_, std::cmp::Ordering::Less) => Direction::Up,
                _ => Direction::Down,
            };
            let interaction = session
                .check_interaction_checked(1)
                .expect("native interaction query")
                .context("hidden clue interaction")?;
            assert_eq!(interaction.script, cache.script);
            let find = |state: &GameState| {
                find_itemfinder_hidden_item(
                    state,
                    &map.id,
                    &map.events.bg_events,
                    &map.script_field_pickups,
                    at,
                )
                .expect("native Itemfinder")
            };
            let found = find(&state).context("Itemfinder must find clue")?;
            assert_eq!(found.item_id, cache.item);
            assert_eq!(found.event_flag, cache.flag);
            let collect = |state: &mut GameState| {
                pickup_field_item(
                    state,
                    &candidate.data.items,
                    pickup.to_field_item_pickup().expect("native pickup"),
                )
                .expect("native collection")
            };
            assert!(matches!(
                collect(&mut state),
                FieldItemPickupOutcome::Collected { quantity: 1, .. }
            ));
            assert_eq!(state.bag.quantity(&candidate.data.items[&cache.item]), 1);
            assert!(
                state
                    .flags
                    .is_event_flag_set(&cache.flag)
                    .expect("collection flag")
            );
            assert!(find(&state).is_none());
            assert!(matches!(
                collect(&mut state),
                FieldItemPickupOutcome::AlreadyCollected { .. }
            ));
            assert_eq!(state.bag.quantity(&candidate.data.items[&cache.item]), 1);
        }
        if let Some(output) = std::env::var_os("MN_CLUES_OUTPUT") {
            let output = std::path::PathBuf::from(output);
            ensure!(
                !output.exists()
                    && std::process::Command::new("git")
                        .args(["check-ignore", "--quiet", "--no-index", "--"])
                        .arg(&output)
                        .status()?
                        .success(),
                "review output must be fresh and ignored"
            );
            candidate.identity = crate::derive_compiled_game_pack_identity_from_manifest(
                candidate.format_version,
                &candidate.data,
                &candidate.audio_manifest,
                &candidate.runtime_files,
                &candidate.report,
            )?;
            crate::verify_compiled_game_pack_for_runtime(&candidate)?;
            candidate.write_preserving_storage(&output)?;
            assert_eq!(crate::read_verified_compiled_game_pack(&output)?, candidate);
        }
        Ok(())
    }
}
