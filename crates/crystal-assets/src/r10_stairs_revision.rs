//! Bounded outdoor stone stairs on the installed western R10 carrier ramp.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{
    OccupiedTile, PlayerMovementState, StepOptions, StepOutcome, attempt_step_with_occupied_tiles,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::authored_route_design::RouteDesign;
use crate::authored_route_layout::Layout;
use crate::authored_route_revision::{clipped_blocks, installed_chunks, validate_anchors};
use crate::{CompiledGamePack, CompiledTilesetExtension};

const TILESET: &str = "mn_r10_rockstairs";
const ART_MANIFEST: &str = "minnesota-r10-rockstairs-art";
const STAIR_MANIFEST: &str = "minnesota-r10-rockstairs-native-stair";
const SEAMS: [(usize, usize, u16, u16); 5] = [
    (0, 5, 0x68, 1),
    (47, 10, 2, 5),
    (47, 11, 2, 5),
    (47, 12, 2, 1),
    (47, 13, 2, 5),
];

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn certify_sources(previous: &str, revised: &str, dossier: &str) -> Result<()> {
    ensure!(
        sha(previous.as_bytes())
            == "2db59b63a1184b5f0975c30f3ff8fec1bebbce2bf6cf18c81895410f3ff94e5a"
            && sha(dossier.as_bytes())
                == "0fd088086912e9a13ec64bae47750e856ac01d1faaedad34f80720b89a268ffa",
        "R10 frozen source layout or dossier differs"
    );
    let old: Value = serde_json::from_str(previous)?;
    let mut new: Value = serde_json::from_str(revised)?;
    ensure!(
        new["compiled_tileset"] == TILESET,
        "R10 stair alias differs"
    );
    let recipe = json!([{
        "id":"80", "source_tileset":"cave", "source_block":"12",
        "collision":"FLOOR/FLOOR/WALL/FLOOR",
        "required_typed_extension":"r10_stairs_revision::revise_r10_rock_stairs"
    }]);
    ensure!(
        new["derived_blocks"] == recipe,
        "R10 exact stair recipe differs"
    );
    let layers = new["layers"]
        .as_array_mut()
        .context("R10 authored layers")?;
    let old_layers = old["layers"].as_array().context("R10 original layers")?;
    ensure!(
        layers.len() == old_layers.len() + 2,
        "R10 stair stroke count differs"
    );
    ensure!(
        layers[old_layers.len()..]
            == [
                json!({"kind":"point","name":"Exact native outdoor stone stair at retained western climb","position":[5,10],"block":"80"}),
                json!({"kind":"point","name":"Attached rock shoulder below western carrier bench","position":[4,11],"block":"0a"})
            ],
        "R10 bounded stair strokes differ"
    );
    layers.truncate(old_layers.len());
    new["compiled_tileset"] = old["compiled_tileset"].clone();
    new.as_object_mut()
        .context("R10 source object")?
        .remove("derived_blocks");
    ensure!(new == old, "R10 retained source field differs");
    Ok(())
}

fn certify_art(base: &CompiledGamePack) -> Result<()> {
    for (path, expected) in [
        (
            "data/tilesets/johto_metatiles.bin",
            "4af44f22edf06b8c4dd949c0acfad0a74a05554d63278d8f5791ba0e7bf13017",
        ),
        (
            "gfx/tilesets/johto.2bpp",
            "da7c80d2d52999f84deb38789eba3758a8a3aba25aaf24d1d2da699683cfd019",
        ),
        (
            "data/tilesets/cave_metatiles.bin",
            "01b0d299482c98724420b4dc0642a20ac57c4b3662754325f098dceef4de391e",
        ),
        (
            "gfx/tilesets/cave.2bpp",
            "d6c5cdb0fb28859d5025b83f15e4150c037c194cd2d496373d4e5224cb34029e",
        ),
        (
            "gfx/tilesets/bg_tiles.pal",
            "83ff1f3f9ab57e9d9cb021acb7689c169a7b1d6ebe78da4a294b8f4f7dc11bed",
        ),
    ] {
        ensure!(
            sha(base
                .runtime_files
                .get(path)
                .with_context(|| format!("native art {path}"))?)
                == expected,
            "R10 certified native art differs: {path}"
        );
    }
    ensure!(
        !base.data.tilesets.contains_key(TILESET)
            && !base
                .report
                .manifests
                .iter()
                .any(|id| id == ART_MANIFEST || id == STAIR_MANIFEST),
        "R10 stair alias is already installed"
    );
    Ok(())
}

fn apply_seams(blocks: &mut [u16]) -> Result<()> {
    for (x, y, original, installed) in SEAMS {
        let cell = blocks.get_mut(y * 48 + x).context("R10 seam cell")?;
        ensure!(*cell == original, "R10 source seam preimage differs");
        *cell = installed;
    }
    Ok(())
}

/// Adds one exact native stone stair and its attached shoulder to installed R10.
///
/// Certifies the frozen 48×20 source epoch, every original event/party/door anchor,
/// the installed five city-seam changes, and native Johto/cave art. Appends cave12
/// as block80 to a dedicated Johto clone; shared Johto files remain unchanged.
/// Only two exterior modules' blocks/tileset attributes and their split mirrors,
/// five new alias runtime files, one tileset, two art manifests and the derived
/// identity change. The 18×14 cave and all gameplay programs/catalogs/audio remain
/// exact. Field-move tables remain exact: neither old nor new R10 terrain contains
/// Cut or Whirlpool collision. Ordinary Surf access remains unchanged.
///
/// Native movement verifies the retained right stair lane both ways and its
/// blocked side. This does not publish a file, migrate saves, or establish visual
/// approval, production-controller input or earned regional progression.
///
/// # Errors
/// Returns an error for source/art/native preimage drift, repeated application,
/// changed events, unsafe geometry, shared payloads or failed pack verification.
/// Borrowed inputs remain unchanged on failure.
pub fn revise_r10_rock_stairs(
    base: &CompiledGamePack,
    previous_layout_json: &str,
    revised_layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    certify_sources(previous_layout_json, revised_layout_json, dossier_json)?;
    certify_art(base)?;
    let old = Layout::parse(previous_layout_json)?;
    let new = Layout::parse(revised_layout_json)?;
    let design = RouteDesign::parse(dossier_json)?;
    let mut before = old.compile()?;
    let mut after = new.compile()?;
    ensure!(
        before
            .iter()
            .zip(&after)
            .enumerate()
            .filter_map(|(i, (&a, &b))| (a != b).then_some((i % 48, i / 48, a, b)))
            .collect::<Vec<_>>()
            == [(5, 10, 0x74, 0x80), (4, 11, 0x71, 0x0a)],
        "R10 terrain must retain the exact two-cell refinement"
    );
    apply_seams(&mut before)?;
    apply_seams(&mut after)?;
    let chunks = installed_chunks(base, &design, &old, &before)?;
    validate_anchors(base, &design)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let files = &base.runtime_files;
    let alias = base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: ART_MANIFEST.into(),
        tileset_id: TILESET.into(),
        behavior_source_tileset_id: None,
        definition: base
            .data
            .tilesets
            .get("johto")
            .context("Johto tileset")?
            .clone(),
        metatiles: files["data/tilesets/johto_metatiles.bin"].clone(),
        tile_graphics_2bpp: files["gfx/tilesets/johto.2bpp"].clone(),
        tile_graphics_png: files["gfx/tilesets/johto.png"].clone(),
    })?;
    let mut result = crate::north_shore_art::append_native_cave_stairs(
        &alias,
        TILESET,
        0x80,
        &[0x12],
        STAIR_MANIFEST,
    )?;
    for (blocks, tileset) in [(&before, "johto"), (&after, TILESET)] {
        for &block in blocks {
            let collision = result.data.tilesets[tileset]
                .collision
                .get(&format!("{block:02x}"))
                .context("R10 native collision")?;
            ensure!(
                collision
                    .iter()
                    .all(|token| token != "CUT_TREE" && !token.starts_with("WHIRLPOOL")),
                "R10 cannot omit inherited Cut or Whirlpool behavior"
            );
        }
    }
    crate::authored_route_modpack::validate_geometry(&result, &design, &after, TILESET)?;
    for chunk in chunks {
        let blocks = clipped_blocks(&after, 48, &chunk);
        let bytes = blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let module = result
            .data
            .maps
            .get_mut(&chunk.name)
            .context("R10 exterior chunk")?;
        module.blocks = blocks;
        module.attributes.tileset_name = TILESET.into();
        result
            .data
            .map_attributes
            .insert(chunk.name, module.attributes.clone());
        result
            .data
            .map_blocks
            .insert(chunk.label, crate::encode_base64_bytes(&bytes));
    }
    validate_native_stair(&result)?;
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

fn validate_native_stair(pack: &CompiledGamePack) -> Result<()> {
    let module = &pack.data.maps["MnR10C00R00"];
    let map = pack.data.overworld_map("MnR10C00R00")?;
    let tileset = pack.data.tileset_collision(TILESET)?;
    let occupied = module
        .objects
        .iter()
        .map(|object| {
            Ok(OccupiedTile {
                tile: TilePosition::new(i16::try_from(object.x)?, i16::try_from(object.y)?),
                object_identifier: object.object_identifier.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    for (start, direction, expected) in [(19, Direction::Down, 22), (22, Direction::Up, 19)] {
        let mut player = PlayerMovementState::new(TilePosition::new(11, start));
        for _ in 0..3 {
            player.facing = direction;
            ensure!(
                matches!(
                    attempt_step_with_occupied_tiles(
                        &mut player,
                        direction,
                        &map,
                        &tileset,
                        StepOptions::default(),
                        &occupied
                    ),
                    StepOutcome::Moved { .. }
                ),
                "R10 native stair lane blocked"
            );
        }
        ensure!(
            player.tile == TilePosition::new(11, expected),
            "R10 native climb target differs"
        );
    }
    let mut player = PlayerMovementState::new(TilePosition::new(11, 21));
    player.facing = Direction::Left;
    ensure!(
        matches!(
            attempt_step_with_occupied_tiles(
                &mut player,
                Direction::Left,
                &map,
                &tileset,
                StepOptions::default(),
                &occupied
            ),
            StepOutcome::Blocked { .. }
        ),
        "R10 stair wall permits side entry"
    );
    Ok(())
}
