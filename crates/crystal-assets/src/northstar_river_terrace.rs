//! A visible supply shelf and one-way return on the Minnesota River terrace.
use std::collections::{BTreeMap, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};
use serde::Deserialize;

use crate::authored_dialogue::AuthoredDialoguePage;
use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule, authored_dialogue, authored_town_publication};

const MAP: &str = "MnR03C00R00";
const SIGN: &str = "MN_R03_TERRACE_SIGNText";
const SOURCE: &str =
    include_str!("../../../planning/minnesota/routes/r03-terrace-return-v01/layout.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Layout {
    map: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    placements: Vec<Placement>,
    item: Item,
    sign_pages: Vec<AuthoredDialoguePage>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    at: [u16; 2],
    from: u16,
    to: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Item {
    script: String,
    from: [u16; 2],
    to: [u16; 2],
}

/// Applies the bounded terrace revision for a combined Minnesota release.
///
/// # Errors
/// Returns an error when the installed terrain, supply or native route contract
/// differs from the reviewed parent. Does not select a pack or open a game.
pub fn revise_northstar_river_terrace(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let mut pack = base.clone();
    install(&mut pack)?;
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

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let layout: Layout = serde_json::from_str(SOURCE)?;
    let mut map = pack
        .data
        .maps
        .get(MAP)
        .context("River Valley terrace")?
        .clone();
    ensure!(
        layout.map == MAP
            && map.attributes.tileset_name == layout.tileset
            && [map.attributes.width, map.attributes.height] == layout.dimensions_metatiles,
        "unexpected terrace dimensions or art"
    );
    for placement in layout.placements {
        let [x, y] = placement.at;
        ensure!(
            x < map.attributes.width && y < map.attributes.height,
            "terrace placement outside map"
        );
        let index = usize::from(y) * usize::from(map.attributes.width) + usize::from(x);
        ensure!(
            map.blocks[index] == placement.from,
            "terrace preimage changed at ({x}, {y})"
        );
        map.blocks[index] = placement.to;
    }
    let item = map
        .objects
        .iter_mut()
        .find(|o| o.script == layout.item.script)
        .context("existing terrace supply")?;
    ensure!(
        [item.x, item.y] == layout.item.from && item.object_type == "OBJECTTYPE_ITEMBALL",
        "terrace supply changed"
    );
    [item.x, item.y] = layout.item.to;
    ensure!(map.scripts.contains_key(SIGN), "existing terrace sign");
    map.scripts
        .insert(SIGN.into(), authored_dialogue::compile(&layout.sign_pages)?);
    let mut bodies = crate::parse_script_text_bodies(MAP, &map.scripts)?;
    map.script_text_bodies.insert(
        SIGN.into(),
        bodies.remove(SIGN).context("terrace sign body")?,
    );
    // Rebind the original ball at its new coordinate without changing its item,
    // quantity or collected flag. Existing saves keep their collected state.
    map.script_field_pickups = crate::parse_script_field_pickups(MAP, &map.scripts, &map.objects)?;
    authoring::refresh_event_section(&mut map);
    verify_geometry(pack, &map)?;
    authored_town_publication::replace(pack, map)
}

/// Static collision proof using core movement functions, without a game session.
fn path(
    pack: &CompiledGamePack,
    room: &MapModule,
    start: [u16; 2],
    target: [u16; 2],
    hops: bool,
) -> Result<Vec<[u16; 2]>> {
    let mut map = OverworldMapData::from_attributes(MAP, &room.attributes, room.blocks.clone());
    map.connections.clear();
    let collision = pack.data.tileset_collision(&room.attributes.tileset_name)?;
    let bodies = room
        .objects
        .iter()
        .map(|o| {
            Ok(OccupiedTile {
                tile: TilePosition {
                    x: i16::try_from(o.x)?,
                    y: i16::try_from(o.y)?,
                },
                object_identifier: o.object_identifier.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let options = StepOptions {
        force_step_after_turn: true,
        ..Default::default()
    };
    let mut pending = VecDeque::from([start]);
    let mut parents = BTreeMap::from([(start, None)]);
    while let Some(at) = pending.pop_front() {
        if at == target {
            let mut route = vec![at];
            let mut current = at;
            while let Some(previous) = parents[&current] {
                route.push(previous);
                current = previous;
            }
            route.reverse();
            return Ok(route);
        }
        for direction in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ] {
            let mut state = PlayerMovementState::new(TilePosition {
                x: i16::try_from(at[0])?,
                y: i16::try_from(at[1])?,
            });
            let jump = hops.then(|| {
                attempt_ledge_jump_with_occupied_tiles(
                    &mut state, direction, &map, &collision, options, &bodies,
                )
            });
            let to = if let Some(LedgeJumpOutcome::Jumped { to, .. }) = jump {
                Some(to)
            } else {
                match attempt_step_with_occupied_tiles(
                    &mut state, direction, &map, &collision, options, &bodies,
                ) {
                    StepOutcome::Moved { to, .. } => Some(to),
                    _ => None,
                }
            };
            let Some(to) = to else {
                continue;
            };
            let to = [u16::try_from(to.x)?, u16::try_from(to.y)?];
            if let std::collections::btree_map::Entry::Vacant(entry) = parents.entry(to) {
                entry.insert(Some(at));
                pending.push_back(to);
            }
        }
    }
    anyhow::bail!("terrace route missing: {start:?} to {target:?}")
}

fn verify_geometry(pack: &CompiledGamePack, map: &MapModule) -> Result<()> {
    for x in [36, 37] {
        ensure!(
            kind(pack, map, [x, 40])? == "HOP_DOWN"
                && kind(pack, map, [x, 41])? == "WALL"
                && kind(pack, map, [x, 42])? == "FLOOR",
            "genuine south ledge and safe landing"
        );
        ensure!(
            path(pack, map, [x, 40], [x, 42], true)? == [[x, 40], [x, 42]],
            "native south jump must span the lip"
        );
        let reverse = path(pack, map, [x, 42], [x, 40], true)?;
        ensure!(
            reverse.len() > 10 && reverse.contains(&[41, 41]),
            "return must climb the east stair"
        );
    }
    ensure!(
        kind(pack, map, [35, 41])? == "WALL" && kind(pack, map, [38, 41])? == "WALL",
        "ledge sides stay closed"
    );
    // The full local north/south journey remains two-way with every native
    // actor present. No water, warps, gameplay inputs or battle simulation.
    for (start, end) in [
        ([20, 8], [38, 47]),
        ([38, 47], [20, 8]),
        ([37, 43], [40, 39]),
        ([40, 39], [37, 43]),
    ] {
        path(pack, map, start, end, false)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MAP, path, revise_northstar_river_terrace};

    #[test]
    #[ignore = "requires the external selected Minnesota pack"]
    fn terrace_return_collision_and_preservation() {
        let source =
            std::env::var_os("CRYSTAL_TERRACE_PARENT").expect("set CRYSTAL_TERRACE_PARENT");
        let base = crate::read_verified_compiled_game_pack(std::path::PathBuf::from(source))
            .expect("verified parent");
        let revised = revise_northstar_river_terrace(&base).expect("guarded terrace overlay");
        let map = &revised.data.maps[MAP];
        for (name, start, end, hops) in [
            ("uphill", [37, 42], [37, 40], true),
            ("downhill", [37, 40], [37, 42], true),
            ("supply approach", [37, 43], [40, 39], false),
            ("supply return", [40, 39], [37, 43], true),
        ] {
            let route = path(&revised, map, start, end, hops).expect("native route");
            println!("{name}: {} inputs; {route:?}", route.len() - 1);
        }
        assert_eq!(revised.runtime_files, base.runtime_files);
        assert_eq!(revised.audio_manifest, base.audio_manifest);
        assert_eq!(revised.compiled_audio, base.compiled_audio);
        assert_eq!(revised.report, base.report);
        let mut restored = revised.clone();
        crate::authored_town_publication::replace(&mut restored, base.data.maps[MAP].clone())
            .expect("restore owned map");
        assert!(
            restored.data == base.data,
            "all unrelated data must remain exact"
        );
        assert!(
            revise_northstar_river_terrace(&revised).is_err(),
            "duplicate application rejected"
        );
        let mut drifted = base.clone();
        drifted.data.maps.get_mut(MAP).expect("map").blocks[20 * 24 + 18] = 1;
        assert!(
            revise_northstar_river_terrace(&drifted).is_err(),
            "unexpected terrain rejected"
        );
    }
}
