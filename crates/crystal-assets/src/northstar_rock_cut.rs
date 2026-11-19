//! The two trail entrances and the limestone shelf inside Rock Cut.
use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_core::input::GameButton;
use crystal_core::random::ReplayDivider;
use crystal_core::state::GameState;
use crystal_core::world::map::{Direction, TilePosition};

use super::{ArtIds, Room, append_metatiles, apply_room, brown_shores, paint_blocks, source};
use crate::authored_town_geometry::kind;
use crate::{
    CompiledGamePack, CompiledTilesetExtension, authored_town_publication, outdoor_contour_paths,
};

const CAVE: &str = "MnR05RockCut";
const STONE: &str = "mn_r05_rock_cut";
const PORTALS: [([u16; 2], &str, [i16; 2]); 2] = [
    ([5, 17], "MnR05C00R01", [16, 30]),
    ([11, 5], "MnR05C00R00", [30, 28]),
];

fn cave_stone(pack: &mut CompiledGamePack) -> Result<()> {
    let file = |path: &str| {
        pack.runtime_files
            .get(path)
            .cloned()
            .with_context(|| path.to_owned())
    };
    let extension = CompiledTilesetExtension {
        manifest_id: "northstar-rock-cut-stone-v1".into(),
        tileset_id: STONE.into(),
        behavior_source_tileset_id: None,
        definition: pack
            .data
            .tilesets
            .get("cave")
            .context("native cave tileset")?
            .clone(),
        metatiles: file("data/tilesets/cave_metatiles.bin")?,
        tile_graphics_2bpp: file("gfx/tilesets/cave.2bpp")?,
        tile_graphics_png: file("gfx/tilesets/cave.png")?,
    };
    // The installed Crystal sheet uses direct native indices. The validator for new
    // two-bank sheets splits it in half, which would reject its original ladders.
    ensure!(
        extension
            .metatiles
            .iter()
            .all(|&glyph| usize::from(glyph) < extension.tile_graphics_2bpp.len() / 16),
        "invalid original cave glyph"
    );
    crate::insert_compiled_tileset_extension(pack, extension)
}

fn install_interior(
    pack: &mut CompiledGamePack,
    base: &CompiledGamePack,
    dir: &Path,
) -> Result<()> {
    let old = base.data.maps.get(CAVE).context("missing Rock Cut")?;
    ensure!(
        old.attributes.tileset_name == "cave" && old.attributes.border_block == 0x2e,
        "unexpected Rock Cut stone"
    );
    let mut room = old.clone();
    room.attributes.tileset_name = STONE.into();
    room.attributes.border_block = 0;
    let mut ids = ArtIds::new();
    append_metatiles(pack, &mut ids, source(dir, "interior-art.json")?, |_, _| {
        false
    })?;
    let [changes]: [Room; 1] = source(dir, "interior.json")?;
    ensure!(changes.map == CAVE, "expected Rock Cut");
    let room = paint_blocks(pack, &room, changes, &ids)?;
    for obj in &old.objects {
        let at = [obj.x, obj.y];
        ensure!(
            kind(base, old, at)? == kind(pack, &room, at)?,
            "Rock Cut resident or pickup terrain changed at {at:?}"
        );
    }
    ensure!(
        room.events.warps.len() == PORTALS.len(),
        "unexpected Rock Cut exit count"
    );
    for (at, _, _) in PORTALS {
        ensure!(
            old.events.warps.iter().any(|w| [w.x, w.y] == at)
                && kind(base, old, at)? == "LADDER"
                && kind(pack, &room, at)? == "WARP_CARPET_DOWN"
                && kind(pack, &room, [at[0], at[1] + 1])? == "WALL",
            "missing directional threshold at {at:?}"
        );
    }
    authored_town_publication::replace(pack, room)
}

fn check_thresholds(pack: &CompiledGamePack) -> Result<()> {
    for ([x, y], outside, [road_x, road_y]) in PORTALS {
        let threshold = TilePosition::new(i16::try_from(x)?, i16::try_from(y)?);
        let clear = threshold.moved(Direction::Up);
        let mut session = pack.data.overworld_session(CAVE, clear, 0)?;
        session.player.facing = Direction::Down;
        let mut state = GameState::default();
        let mut divider = ReplayDivider::new([0; 64]);
        let music_ids = BTreeSet::new();
        let mut press = |session: &mut _, button| {
            pack.data
                .apply_overworld_input(&mut state, session, [button], &music_ids, &mut divider)
        };
        let enter_light = press(&mut session, GameButton::Down)?;
        ensure!(
            enter_light.warp.is_none()
                && session.map.name == CAVE
                && session.player.tile == threshold,
            "Rock Cut threshold activated on the landing step"
        );
        let exit = press(&mut session, GameButton::Down)?;
        ensure!(
            exit.warp.is_some()
                && session.map.name == outside
                && session.player.tile == TilePosition::new(road_x, road_y),
            "Rock Cut exit {x},{y} did not return to its trail"
        );
        for _ in 0..2 {
            let entry = press(&mut session, GameButton::Up)?;
            if entry.warp.is_some() {
                break;
            }
        }
        ensure!(
            session.map.name == CAVE && session.player.tile == threshold,
            "Rock Cut trail entrance did not reach {x},{y}"
        );
        // Arrivals can inherit either facing. A turn is allowed before walking clear.
        for _ in 0..2 {
            if session.player.tile == clear {
                break;
            }
            let frame = press(&mut session, GameButton::Up)?;
            ensure!(frame.warp.is_none(), "Rock Cut arrival loop");
        }
        ensure!(
            session.map.name == CAVE && session.player.tile == clear,
            "cannot clear Rock Cut arrival"
        );
    }
    Ok(())
}

/// Fits Rock Cut's two surface openings and its interior using the installed native art.
///
/// # Errors
/// Rejects mismatched parent geometry, altered actors or exits, and broken route, stair,
/// ledge, or two-way cave access.
pub fn revise_rock_cut(base: &CompiledGamePack, dir: &Path) -> Result<CompiledGamePack> {
    ensure!(
        base.report
            .manifests
            .last()
            .is_some_and(|id| id == brown_shores::MANIFEST),
        "expected brown-shore parent"
    );
    let mut pack = base.clone();
    cave_stone(&mut pack)?;
    install_interior(&mut pack, base, dir)?;
    let mut ids = ArtIds::new();
    append_metatiles(
        &mut pack,
        &mut ids,
        source(dir, "exterior-art.json")?,
        |_, _| false,
    )?;
    let [north, south]: [Room; 2] = source(dir, "exteriors.json")?;
    ensure!(
        north.map == "MnR05C00R00" && south.map == "MnR05C00R01",
        "expected Rock Cut approaches"
    );
    for room in [north, south] {
        apply_room(&mut pack, base, room, &ids)?;
    }
    let revisions = dir.parent().context("expected revisions directory")?;
    brown_shores::verify_paths(
        base,
        &pack,
        &revisions.join("brown-shore-edges-v01"),
        &revisions.join("shore-rocks-cave-mouths-v01"),
    )?;
    for x in [6, 7] {
        outdoor_contour_paths::check_drop(&pack, CAVE, [x, 6], Direction::Down)?;
    }
    outdoor_contour_paths::check_walk(&pack, CAVE, [9, 8], [9, 6], 2)?;
    check_thresholds(&pack)?;
    pack.report.manifests.push("northstar-rock-cut-v1".into());
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
