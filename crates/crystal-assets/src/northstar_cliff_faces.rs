//! Authored follow-up to the open-water outdoor revision.
use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};
use serde::Deserialize;

use super::{
    ArtIds, Room, append_metatiles, apply_room, northstar_public_piers, outdoor_dock_dialogue,
    outdoor_shore_access, source,
};
use crate::{CompiledGamePack, outdoor_contour_paths};

#[path = "cliff_south_bridges.rs"]
mod cliff_south_bridges;

pub(super) const MANIFEST: &str = "northstar-cliff-faces-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Checks {
    drops: Vec<FacingTile>,
    walks: Vec<WalkCheck>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FacingTile {
    map: String,
    from: [i16; 2],
    direction: Direction,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WalkCheck {
    map: String,
    from: [u16; 2],
    to: [u16; 2],
    max_steps: u32,
}

impl WalkCheck {
    pub(super) fn verify(self, pack: &CompiledGamePack) -> Result<()> {
        outdoor_contour_paths::check_walk(pack, &self.map, self.from, self.to, self.max_steps)
    }
}

/// Repairs Minnesota outdoor cliff faces and verifies their installed walking routes.
///
/// # Errors
/// Returns an error for an unexpected parent, stale or repeated placement,
/// invalid native art, blocked shortcut, or unreachable stair landing.
pub fn revise_cliff_faces(base: &CompiledGamePack, dir: &Path) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.iter().any(|m| m == super::MANIFEST)
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "expected the unmodified outdoor contours revision as a parent"
    );
    let mut pack = base.clone();
    northstar_public_piers::install_timber_in_banks(
        &mut pack,
        &["mn_duluth_expanded", "mn_boundary_safari"],
    )?;
    let mut ids = ArtIds::new();
    let mut rooms = BTreeSet::new();
    for region in ["north", "south", "places", "waterfronts"] {
        append_metatiles(
            &mut pack,
            &mut ids,
            source(dir, &format!("{region}-art.json"))?,
            northstar_public_piers::is_timber_glyph,
        )?;
        let mut layouts: Vec<Room> = source(dir, &format!("{region}.json"))?;
        if region == "south" {
            let variant = if base.data.maps.contains_key("MnMothlightLodge") {
                "south-mothlight.json"
            } else {
                "south-field.json"
            };
            for cell_group in source::<Vec<Room>>(dir, variant)? {
                let room = layouts
                    .iter_mut()
                    .find(|r| r.map == cell_group.map && r.tileset == cell_group.tileset)
                    .context("missing shoreline on selected parent")?;
                room.placements.extend(cell_group.placements);
            }
        }
        for room in layouts {
            let old = base.data.maps.get(&room.map).context("missing cliff map")?;
            ensure!(
                room.map.starts_with("Mn")
                    && matches!(
                        old.attributes.environment.as_deref(),
                        Some("ROUTE" | "TOWN")
                    )
                    && rooms.insert(room.map.clone()),
                "unexpected or duplicate outdoor cliff map {}",
                room.map
            );
            apply_room(&mut pack, base, room, &ids)?;
        }
    }
    verify_installed_paths(&pack, dir)?;
    outdoor_dock_dialogue::install(&mut pack, dir)?;
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

pub(super) fn verify_installed_paths(pack: &CompiledGamePack, dir: &Path) -> Result<()> {
    for region in ["north", "south", "places", "waterfronts"] {
        verify_shores(pack, dir, &format!("{region}-surf.json"))?;
        let checks: Checks = source(dir, &format!("{region}-checks.json"))?;
        for drop in checks.drops {
            outdoor_contour_paths::check_drop(pack, &drop.map, drop.from, drop.direction)?;
        }
        for walk in checks.walks {
            walk.verify(pack)?;
        }
    }
    cliff_south_bridges::verify(pack)?;
    outdoor_contour_paths::verify(pack)?;
    outdoor_shore_access::verify(pack)?;
    Ok(())
}

pub(super) fn verify_shores(pack: &CompiledGamePack, dir: &Path, name: &str) -> Result<()> {
    let partner = outdoor_shore_access::surf_partner(pack)?;
    for shore in source::<Vec<FacingTile>>(dir, name)? {
        outdoor_shore_access::verify_dock_edge(
            pack,
            &partner,
            &shore.map,
            TilePosition::new(shore.from[0], shore.from[1]),
            shore.direction,
        )?;
    }
    Ok(())
}
