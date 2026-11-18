//! Hand-fitted outdoor cliffs, paths and stair landings.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::authored_town_geometry::kind;
use crate::northstar_trail_caches::{self, Landmark};
use crate::{CompiledGamePack, authored_town_publication};

#[path = "northstar_public_piers.rs"]
mod northstar_public_piers;
#[path = "outdoor_dock_dialogue.rs"]
mod outdoor_dock_dialogue;
#[path = "outdoor_shore_access.rs"]
mod outdoor_shore_access;
#[path = "shiner_dock_water.rs"]
mod shiner_dock_water;

#[path = "northstar_cliff_faces.rs"]
mod cliff_faces;
pub use cliff_faces::revise_cliff_faces;

const MANIFEST: &str = "northstar-outdoor-contours-v4";
type ArtIds = BTreeMap<(String, u16), u8>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ink {
    first_glyph: u8,
    design: Landmark,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Metatile {
    tileset: String,
    id: u16,
    collision_from: u8,
    collision: Option<[String; 4]>,
    tiles: [u8; 16],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Room {
    map: String,
    tileset: String,
    placements: Vec<Placement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    at: [u16; 2],
    from: u16,
    to: u16,
}

fn source<T: DeserializeOwned>(dir: &Path, name: &str) -> Result<T> {
    let path = dir.join(name);
    serde_json::from_slice(&std::fs::read(&path).with_context(|| path.display().to_string())?)
        .with_context(|| path.display().to_string())
}

fn metatile_path(tileset: &str) -> String {
    format!("data/tilesets/{tileset}_metatiles.bin")
}

fn allocate(pack: &CompiledGamePack, ids: &mut ArtIds, tileset: &str, logical: u16) -> Result<u8> {
    ensure!(
        tileset.starts_with("mn_")
            || (tileset == "kanto_glacier_rail"
                && pack
                    .data
                    .maps
                    .values()
                    .all(|m| m.attributes.tileset_name != tileset || m.id == "MnStPaul")),
        "expected Minnesota tileset"
    );
    let len = pack
        .runtime_files
        .get(&metatile_path(tileset))
        .context("missing outdoor art")?
        .len();
    ensure!(len % 16 == 0, "incomplete native metatile");
    let native = u8::try_from(len / 16).context("full native metatile bank")?;
    ensure!(
        ids.insert((tileset.to_owned(), logical), native).is_none(),
        "duplicate terrain recipe"
    );
    Ok(native)
}

fn install_art(pack: &mut CompiledGamePack, dir: &Path) -> Result<ArtIds> {
    // A parent can already contain a regional hidden-item tile in this bank.
    // Append after it; map sources use the recipe's logical ID only on output.
    let mut ids = ArtIds::new();
    for mut ink in source::<Vec<Ink>>(dir, "iron-ink.json")? {
        let tileset = ink.design.tileset.clone();
        ink.design.block = allocate(pack, &mut ids, &tileset, u16::from(ink.design.block))?;
        northstar_trail_caches::install_art(pack, &tileset, &[ink.design], ink.first_glyph)?;
    }
    for name in [
        "iron-art.json",
        "northern-art.json",
        "driftless-art.json",
        "southern-shore-art.json",
        "lake-pier-art.json",
    ] {
        let additions: Vec<Metatile> = source(dir, name)?;
        for entry in &additions {
            if let Some(quadrants) = &entry.collision {
                ensure!(
                    quadrants
                        .iter()
                        .all(|q| matches!(q.as_str(), "WATER" | "FLOOR")),
                    "unexpected dock collision"
                );
            }
        }
        append_metatiles(pack, &mut ids, additions)?;
    }
    Ok(ids)
}

fn append_metatiles(
    pack: &mut CompiledGamePack,
    ids: &mut ArtIds,
    art: Vec<Metatile>,
) -> Result<()> {
    for art in art {
        let native = allocate(pack, ids, &art.tileset, art.id)?;
        let layout = pack
            .runtime_files
            .get_mut(&metatile_path(&art.tileset))
            .context("metatile bank")?;
        ensure!(
            art.tiles
                .iter()
                .all(|g| layout.contains(g)
                    || northstar_public_piers::is_timber_glyph(&art.tileset, *g)),
            "recipe references unused tile in {}",
            art.tileset
        );
        let def = pack
            .data
            .tilesets
            .get_mut(&art.tileset)
            .context("terrain collision bank")?;
        let donor = def
            .collision
            .get(&format!("{:02x}", art.collision_from))
            .context("collision donor")?;
        let collision = if let Some(quadrants) = art.collision {
            for quadrant in &quadrants {
                crate::resolve_collision_token(quadrant)?;
            }
            quadrants.to_vec()
        } else {
            donor.clone()
        };
        ensure!(
            def.collision
                .insert(format!("{native:02x}"), collision)
                .is_none(),
            "occupied terrain slot"
        );
        layout.extend(art.tiles);
        pack.runtime_files.insert(
            format!("data/tilesets/{}.json", art.tileset),
            serde_json::to_vec(&def.collision)?,
        );
    }
    Ok(())
}

fn apply_room(
    pack: &mut CompiledGamePack,
    base: &CompiledGamePack,
    changes: Room,
    ids: &ArtIds,
) -> Result<()> {
    let old = base
        .data
        .maps
        .get(&changes.map)
        .context("missing terrain map")?;
    ensure!(
        old.attributes.tileset_name == changes.tileset && !changes.placements.is_empty(),
        "unexpected terrain: {}",
        changes.map
    );
    let mut map = old.clone();
    let a = &map.attributes;
    let mut edited = BTreeSet::new();
    for cell in changes.placements {
        let [x, y] = cell.at;
        ensure!(
            edited.insert(cell.at) && x < a.width && y < a.height,
            "invalid placement on {}",
            changes.map
        );
        ensure!(
            !map.script_block_changes
                .iter()
                .any(|c| [c.x / 2, c.y / 2] == cell.at),
            "scripted terrain on {} at {x},{y}",
            changes.map
        );
        let at = usize::from(y) * usize::from(a.width) + usize::from(x);
        let dest = ids
            .get(&(changes.tileset.clone(), cell.to))
            .copied()
            .map(u16::from)
            .unwrap_or(cell.to);
        ensure!(
            map.blocks[at] == cell.from && dest != cell.from,
            "stale terrain on {} at {x},{y}",
            changes.map
        );
        ensure!(
            pack.data.tilesets[&changes.tileset]
                .collision
                .contains_key(&format!("{dest:02x}")),
            "missing terrain at {x},{y}"
        );
        map.blocks[at] = dest;
    }
    for tile in map
        .events
        .warps
        .iter()
        .map(|w| [w.x, w.y])
        .chain(map.objects.iter().map(|o| [o.x, o.y]))
    {
        let old_kind = kind(base, old, tile)?;
        let new_kind = kind(pack, &map, tile)?;
        ensure!(
            old_kind == new_kind,
            "actor or warp terrain changed on {} at {tile:?}",
            changes.map
        );
    }
    authored_town_publication::replace(pack, map)
}

/// Applies the authored outdoor-map revisions to a verified Minnesota parent.
/// New art appends to each installed bank. The review has its own save namespace.
///
/// # Errors
/// Rejects changed map preimages, occupied new artwork, modified scripted or
/// entrance cells, duplicate placements, or invalid native game data.
pub fn revise_outdoor_contours(base: &CompiledGamePack, dir: &Path) -> Result<CompiledGamePack> {
    ensure!(
        !base.report.manifests.iter().any(|s| s == MANIFEST),
        "outdoor revision already installed"
    );
    let mut pack = base.clone();
    northstar_public_piers::install_timber(&mut pack)?;
    let ids = install_art(&mut pack, dir)?;
    let mut rooms = BTreeSet::new();
    for file in [
        "driftless.json",
        "river-bluffs.json",
        "iron-range.json",
        "iron-descent.json",
        "northern-routes.json",
        "shore-north.json",
        "lake-country.json",
        "southern-shore.json",
        "lake-piers.json",
    ] {
        for changes in source::<Vec<Room>>(dir, file)? {
            ensure!(
                rooms.insert(changes.map.clone()) && changes.map.starts_with("MnR"),
                "duplicate or unowned route"
            );
            let old = base.data.maps.get(&changes.map).context("missing route")?;
            ensure!(
                old.attributes.environment.as_deref() == Some("ROUTE"),
                "expected route"
            );
            apply_room(&mut pack, base, changes, &ids)?;
        }
    }
    northstar_public_piers::install_brainerd(&mut pack)?;
    shiner_dock_water::install(&mut pack)?;
    crate::outdoor_contour_paths::verify(&pack)?;
    outdoor_shore_access::verify(&pack)?;
    outdoor_dock_dialogue::install(&mut pack, dir)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.runtime_files
        .remove("data/northstar-save-compatibility.json");
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
