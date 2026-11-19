//! Restore native water edges and repair the elevation changes on Minnesota Route 5.
use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{MovementMode, StepOptions, StepOutcome};
use serde::Deserialize;

use super::{
    ArtIds, Metatile, Room, append_metatiles, cliff_faces, metatile_path, paint_room, publish_room,
    rocky_shores, source,
};
use crate::{CompiledGamePack, outdoor_contour_paths};

const MANIFEST: &str = "northstar-brown-shore-edges-v1";

fn restore_edges(
    pack: &mut CompiledGamePack,
    base: &CompiledGamePack,
    previous: &Path,
) -> Result<()> {
    for section in ["routes", "places", "route-landings", "places-landings"] {
        let mut recipes = BTreeMap::new();
        for tile in source::<Vec<Metatile>>(previous, &format!("{section}-art.json"))? {
            recipes.insert((tile.tileset, tile.id), tile.tiles);
        }
        for mut room in source::<Vec<Room>>(previous, &format!("{section}.json"))? {
            let map = pack.data.maps.get(&room.map).context("missing shore map")?;
            let bank = &map.attributes.tileset_name;
            let layout = &pack.runtime_files[&metatile_path(bank)];
            let collision = &pack.data.tilesets[bank].collision;
            for cell in &mut room.placements {
                let [x, y] = cell.at;
                ensure!(
                    x < map.attributes.width && y < map.attributes.height,
                    "shore outside {}",
                    map.id
                );
                let index = usize::from(y) * usize::from(map.attributes.width) + usize::from(x);
                let installed = map.blocks[index];
                let original = cell.from;
                if let Some(tiles) = recipes.get(&(bank.clone(), cell.to)) {
                    let start = usize::from(installed) * 16;
                    ensure!(
                        layout.get(start..start + 16) == Some(tiles.as_slice()),
                        "shore art changed on {} at {x},{y}",
                        map.id
                    );
                } else {
                    ensure!(
                        installed == cell.to,
                        "shore block changed on {} at {x},{y}",
                        map.id
                    );
                }
                let was = collision
                    .get(&format!("{installed:02x}"))
                    .context("missing shore collision")?;
                ensure!(
                    collision.get(&format!("{original:02x}")) == Some(was),
                    "restored water edge would change access on {} at {x},{y}",
                    map.id
                );
                cell.from = installed;
                cell.to = original;
            }
            let map = paint_room(pack, pack, room, &ArtIds::new())?;
            publish_room(pack, base, map)?;
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteStep {
    map: String,
    from: [i16; 2],
    direction: Direction,
    allowed: bool,
}

impl RouteStep {
    fn verify(&self, pack: &CompiledGamePack) -> Result<()> {
        let origin = TilePosition::new(self.from[0], self.from[1]);
        let mut session = pack.data.overworld_session(&self.map, origin, 0)?;
        let dest = origin.moved(self.direction);
        let outcome = session
            .step_and_check_warp_checked(
                self.direction,
                StepOptions {
                    force_step_after_turn: true,
                    ..Default::default()
                },
            )?
            .outcome;
        let matches = match outcome {
            StepOutcome::Moved { to, .. } => self.allowed && to == dest,
            StepOutcome::Blocked { at, .. } => !self.allowed && at == dest,
            _ => false,
        };
        ensure!(
            matches,
            "unexpected {:?} step on {} at {:?}",
            self.direction,
            self.map,
            origin
        );
        Ok(())
    }
}

/// Restores the original brown shoreline metatiles and installs the authored Route 5 repairs.
///
/// # Errors
/// Rejects a mismatched parent, stale shore or route cell, changed shoreline collision,
/// blocked stair or cave approach, or an impassable public landing.
pub fn restore_brown_shores(base: &CompiledGamePack, dir: &Path) -> Result<CompiledGamePack> {
    ensure!(
        base.report
            .manifests
            .last()
            .is_some_and(|s| s == rocky_shores::MANIFEST),
        "expected the rocky-shores revision"
    );
    let previous = dir
        .parent()
        .context("missing revisions directory")?
        .join("shore-rocks-cave-mouths-v01");
    let mut pack = base.clone();
    restore_edges(&mut pack, base, &previous)?;
    let mut ids = ArtIds::new();
    append_metatiles(
        &mut pack,
        &mut ids,
        source(dir, "route5-art.json")?,
        |_, _| false,
    )?;
    for room in source::<Vec<Room>>(dir, "route5.json")? {
        ensure!(
            room.map.starts_with("MnR05"),
            "unexpected map in Route 5 repairs"
        );
        let map = paint_room(&pack, &pack, room, &ids)?;
        publish_room(&mut pack, base, map)?;
    }
    rocky_shores::verify_paths(base, &pack, &previous)?;
    for walk in source::<Vec<cliff_faces::WalkCheck>>(dir, "route5-walks.json")? {
        walk.verify(&pack)?;
    }
    for step in source::<Vec<RouteStep>>(dir, "route5-steps.json")? {
        step.verify(&pack)?;
    }
    // The south crossing includes the walking deck and the offset into Minneapolis.
    for ((north, x, y), (south, arrival_x, arrival_y), steps) in [
        (("MnStCloud", 24, 35), ("MnR05C00R00", 24, 0), 1),
        (("MnR05C00R00", 28, 47), ("MnR05C00R01", 28, 0), 1),
        (("MnR05C00R01", 22, 47), ("MnR05C00R02", 22, 0), 1),
        (("MnR05C00R02", 22, 9), ("MnMinneapolis", 40, 0), 7),
    ] {
        for lane in [0, 1] {
            let start = (north, x + lane, y);
            let end = (south, arrival_x + lane, arrival_y);
            outdoor_contour_paths::check_passage(
                &pack,
                start,
                Direction::Down,
                MovementMode::Normal,
                steps,
                end,
            )?;
            outdoor_contour_paths::check_passage(
                &pack,
                end,
                Direction::Up,
                MovementMode::Normal,
                steps,
                start,
            )?;
        }
    }
    pack.report.manifests.push(MANIFEST.to_owned());
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
