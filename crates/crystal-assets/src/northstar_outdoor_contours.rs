//! Hand-fitted outdoor cliffs, paths and stair landings.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::authored_town_geometry::kind;
use crate::northstar_trail_caches::{self, Landmark};
use crate::{CompiledGamePack, authored_town_publication};

const MANIFEST: &str = "northstar-outdoor-contours-v1";
type ArtIds = BTreeMap<(String, u16), u8>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ink {
    first_glyph: u8,
    design: Landmark,
    #[serde(default)]
    additional_banks: Vec<InkBank>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InkBank {
    tileset: String,
    id: u8,
    first_glyph: u8,
    background: u8,
    foreground: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Metatile {
    tileset: String,
    id: u16,
    collision_from: u8,
    tiles: [u8; 16],
    cut_to: Option<u16>,
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
    ensure!(tileset.starts_with("mn_"), "expected Minnesota tileset");
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
    for ink in source::<Vec<Ink>>(dir, "iron-ink.json")? {
        let mut designs = Vec::new();
        for bank in ink.additional_banks {
            let mut local = ink.design.clone();
            local.tileset = bank.tileset;
            local.block = bank.id;
            local.background = bank.background;
            local.foreground = Some(bank.foreground);
            designs.push((local, bank.first_glyph));
        }
        designs.push((ink.design, ink.first_glyph));
        for (mut local, first_glyph) in designs {
            let tileset = local.tileset.clone();
            local.block = allocate(pack, &mut ids, &tileset, u16::from(local.block))?;
            northstar_trail_caches::install_art(pack, &tileset, &[local], first_glyph)?;
        }
    }
    for name in [
        "iron-art.json",
        "northern-art.json",
        "driftless-art.json",
        "descent-art.json",
    ] {
        for art in source::<Vec<Metatile>>(dir, name)? {
            let native = allocate(pack, &mut ids, &art.tileset, art.id)?;
            let layout = pack
                .runtime_files
                .get_mut(&metatile_path(&art.tileset))
                .context("metatile bank")?;
            ensure!(
                art.tiles.iter().all(|g| layout.contains(g)),
                "recipe references unused tile in {}",
                art.tileset
            );
            let def = pack
                .data
                .tilesets
                .get_mut(&art.tileset)
                .context("terrain collision bank")?;
            let collision = def
                .collision
                .get(&format!("{:02x}", art.collision_from))
                .context("collision donor")?
                .clone();
            ensure!(
                def.collision
                    .insert(format!("{native:02x}"), collision)
                    .is_none(),
                "occupied terrain slot"
            );
            if let Some(bare) = art.cut_to {
                let bare = ids
                    .get(&(art.tileset.clone(), bare))
                    .copied()
                    .map(u16::from)
                    .unwrap_or(bare);
                let ground = def
                    .collision
                    .get(&format!("{bare:02x}"))
                    .context("bare quarry ground")?;
                let grass = &def.collision[&format!("{native:02x}")];
                ensure!(
                    grass.iter().any(|tile| tile == "TALL_GRASS")
                        && grass.iter().zip(ground).all(|(a, b)| if a == "TALL_GRASS" {
                            b == "FLOOR"
                        } else {
                            a == b
                        }),
                    "Cut would alter a cliff on {}",
                    art.tileset
                );
                let rules = pack
                    .data
                    .field_moves
                    .cut
                    .replacements
                    .get_mut(&art.tileset)
                    .context("outdoor Cut rules")?;
                let mut rule = rules.get(&3).context("ordinary tall grass Cut")?.clone();
                ensure!(rule.variant == "grass", "unexpected native Cut variant");
                rule.replacement_block_id = bare;
                ensure!(
                    rules.insert(u16::from(native), rule).is_none(),
                    "duplicate grass Cut rule"
                );
            }
            layout.extend(art.tiles);
            pack.runtime_files.insert(
                format!("data/tilesets/{}.json", art.tileset),
                serde_json::to_vec(&def.collision)?,
            );
        }
    }
    Ok(ids)
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
    let ids = install_art(&mut pack, dir)?;
    let mut rooms = BTreeSet::new();
    for file in [
        "driftless.json",
        "river-bluffs.json",
        "iron-range.json",
        "iron-descent.json",
        "northern-routes.json",
    ] {
        for changes in source::<Vec<Room>>(dir, file)? {
            ensure!(
                rooms.insert(changes.map.clone()) && changes.map.starts_with("MnR"),
                "duplicate or unowned route"
            );
            let old = base.data.maps.get(&changes.map).context("missing route")?;
            let mut map = old.clone();
            let a = &map.attributes;
            ensure!(
                a.environment.as_deref() == Some("ROUTE")
                    && a.tileset_name == changes.tileset
                    && !changes.placements.is_empty(),
                "unexpected route terrain: {}",
                changes.map
            );
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
                let new_kind = kind(&pack, &map, tile)?;
                ensure!(
                    old_kind == new_kind
                        || matches!(
                            (old_kind, new_kind),
                            ("FLOOR", "TALL_GRASS") | ("TALL_GRASS", "FLOOR")
                        ),
                    "actor or warp terrain changed on {} at {tile:?}",
                    changes.map
                );
            }
            authored_town_publication::replace(&mut pack, map)?;
        }
    }
    crate::outdoor_contour_paths::verify(&pack)?;
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
