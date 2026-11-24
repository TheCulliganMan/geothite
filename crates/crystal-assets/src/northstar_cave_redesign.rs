//! Publish complete replacement Minnesota cave maps against an exact parent.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::is_warp_permission;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;

#[path = "northstar_cave_event_relocation.rs"]
mod relocation;
use relocation::{EventMove, ScriptPatch, WarpMove};

const PARENT: &str = "d09a77f306578627e661a0811d2bacc12d22552e5e84041e0f002f6635db46c2";
const MANIFEST: &str = "northstar-cave-replacement-v2";
const REVIEW_PARENT: &str = "768c0244886c6ecdc609336c04409d21b27941aea90175d943440a7166090440";
const REVIEW_MANIFEST: &str = "northstar-cave-audit-v5";
const SAVE_POLICY: &str = "data/northstar-save-compatibility.json";
const MINE_DUNGEONS: [&str; 5] = [
    "MnIronDrillCore",
    "MnIronHaulage",
    "MnIronPumpGallery",
    "MnIronSump",
    "MnIronVentilation",
];

#[derive(Deserialize)]
struct Layout {
    map: String,
    dimensions_metatiles: [u16; 2],
    before_blocks_sha256: String,
    rows: Vec<String>,
    arrivals: Vec<[u16; 2]>,
    time_of_day: Option<String>,
    border_block: Option<u8>,
    #[serde(default)]
    warps: Vec<WarpMove>,
    #[serde(default)]
    actors: Vec<EventMove>,
    #[serde(default)]
    backgrounds: Vec<EventMove>,
    #[serde(default)]
    coordinates: Vec<EventMove>,
    #[serde(default)]
    script_patches: Vec<ScriptPatch>,
}

/// Replaces Minnesota cave footprints and authored placements in the Fair pack.
///
/// Exterior connections, map identities, progression programs and reward flags
/// carry over. Inside a cave, its terrain, warp landings, and event placements
/// are authored together. The local preview uses a separate save slot.
///
/// # Errors
/// Rejects a stale parent, incomplete room inventory, unsupported terrain,
/// invalid anchors or geometry changes that break native warp activation.
pub fn build_northstar_cave_redesign(
    parent: &CompiledGamePack,
    sources: &[&str],
) -> Result<CompiledGamePack> {
    compile(parent, sources, &[], PARENT, MANIFEST)
}

/// Compiles the room plans and native art from the comparative cave audit.
/// Existing event identities and destination warp records carry over.
///
/// # Errors
/// Rejects a different parent, incomplete region coverage, stale terrain or
/// event preimages, and layouts that invalidate native event collision.
pub fn build_northstar_cave_audit(
    parent: &CompiledGamePack,
    sources: &[&str],
    art: &[&str],
) -> Result<CompiledGamePack> {
    compile(parent, sources, art, REVIEW_PARENT, REVIEW_MANIFEST)
}

fn compile(
    parent: &CompiledGamePack,
    sources: &[&str],
    art: &[&str],
    parent_identity: &str,
    manifest: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(parent)?;
    ensure!(
        parent.identity()?.content_hash == parent_identity,
        "unexpected cave parent"
    );
    let required: BTreeSet<_> = parent
        .data
        .maps
        .values()
        .filter(|room| {
            (room.id.starts_with("Mn") && room.attributes.environment.as_deref() == Some("CAVE"))
                || MINE_DUNGEONS.contains(&room.id.as_str())
        })
        .map(|room| room.id.as_str())
        .collect();
    let mut layouts = Vec::new();
    for source in sources {
        layouts.extend(serde_json::from_str::<Vec<Layout>>(source)?);
    }
    ensure!(
        layouts.len() == required.len()
            && layouts
                .iter()
                .map(|layout| layout.map.as_str())
                .collect::<BTreeSet<_>>()
                == required,
        "replacement requires every Minnesota cave and Iron Range chamber"
    );
    let mut result = parent.clone();
    let allowed = required
        .iter()
        .map(|&name| parent.data.maps[name].attributes.tileset_name.as_str())
        .collect();
    crate::northstar_cave_detail_art::install(&mut result, art, &allowed)?;
    for layout in &layouts {
        install_room(parent, &mut result, layout)
            .with_context(|| format!("replacing cave {}", layout.map))?;
    }
    result.report.manifests.push(manifest.into());
    result.runtime_files.remove(SAVE_POLICY);
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

fn install_room(
    parent: &CompiledGamePack,
    result: &mut CompiledGamePack,
    layout: &Layout,
) -> Result<()> {
    let before = parent
        .data
        .maps
        .get(&layout.map)
        .context("installed cave")?;
    ensure!(
        before.attributes.connections.is_empty(),
        "replace caves with discrete entrances"
    );
    let label = before
        .attributes
        .blocks_label
        .as_ref()
        .context("cave block label")?;
    ensure!(
        parent
            .data
            .maps
            .values()
            .filter(|map| map.attributes.blocks_label.as_ref() == Some(label))
            .count()
            == 1,
        "shared cave terrain payload"
    );
    let original = crate::decode_base64_bytes(
        parent
            .data
            .map_blocks
            .get(label)
            .context("cave block payload")?,
    )?;
    ensure!(
        original
            .iter()
            .copied()
            .map(u16::from)
            .eq(before.blocks.iter().copied())
            && format!("{:x}", Sha256::digest(&original)) == layout.before_blocks_sha256,
        "stale cave terrain preimage"
    );
    let [width, height] = layout.dimensions_metatiles;
    ensure!(
        (4..=64).contains(&width)
            && (4..=64).contains(&height)
            && layout.rows.len() == usize::from(height),
        "invalid cave dimensions"
    );
    let mut room = before.clone();
    room.attributes.width = width;
    room.attributes.height = height;
    room.blocks.clear();
    for row in &layout.rows {
        let cells = row
            .split_whitespace()
            .map(|cell| u8::from_str_radix(cell, 16).map(u16::from))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(cells.len() == usize::from(width), "incomplete terrain row");
        room.blocks.extend(cells);
    }
    ensure!(
        room.blocks != before.blocks,
        "replacement cannot reuse the original floorplan"
    );
    if let Some(period) = &layout.time_of_day {
        ensure!(
            matches!(period.as_str(), "day" | "nite" | "dark"),
            "invalid cave light period"
        );
        room.attributes.time_of_day = Some(period.clone());
        room.attributes.palette = Some(format!("PALETTE_{}", period.to_ascii_uppercase()));
    }
    if let Some(block) = layout.border_block {
        let collision = &result.data.tilesets[&room.attributes.tileset_name].collision;
        ensure!(
            collision
                .get(&format!("{block:02x}"))
                .is_some_and(|permissions| permissions.iter().all(|kind| kind == "WALL")),
            "cave border must be solid in every quadrant"
        );
        room.attributes.border_block = block;
    }
    relocation::apply(result, before, &mut room, layout)?;
    crate::northstar_cave_overhaul::validate_native_art(result, &room)?;
    for (old, new) in before.events.warps.iter().zip(&room.events.warps) {
        let before_kind = kind(parent, before, [old.x, old.y])?;
        let after_kind = kind(result, &room, [new.x, new.y])?;
        let both_active = is_warp_permission(crate::resolve_collision_token(before_kind)?)
            && is_warp_permission(crate::resolve_collision_token(after_kind)?);
        ensure!(
            before_kind == after_kind || (both_active && (old.x, old.y) != (new.x, new.y)),
            "warp {} at ({}, {}) changed activation from {before_kind} to {after_kind}",
            new.index,
            new.x,
            new.y
        );
    }
    ensure!(
        !layout.arrivals.is_empty(),
        "at least one clear arrival required"
    );
    for &at in &layout.arrivals {
        ensure!(
            kind(result, &room, at)? == "FLOOR" && room.objects.iter().all(|o| [o.x, o.y] != at),
            "obstructed arrival {at:?}"
        );
    }
    crate::authored_town_publication::replace(result, room)
}
