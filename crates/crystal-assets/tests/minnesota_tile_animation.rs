use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use crystal_assets::{AuthoredTileAnimation, MinnesotaTileAnimationRecipe, MinnesotaTilePixels};

fn inputs() -> Result<(MinnesotaTileAnimationRecipe, Vec<MinnesotaTilePixels>)> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/tiles/campfire-animation-v1");
    let recipe: MinnesotaTileAnimationRecipe =
        serde_json::from_slice(&fs::read(directory.join("animation.json"))?)?;
    let variants = recipe
        .sources
        .iter()
        .map(|frame| fs::read(directory.join(&frame.path)).map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;
    let frames = recipe.compile(
        &fs::read(directory.join(&recipe.base_catalog))?,
        &fs::read(directory.join(&recipe.base_source))?,
        &variants,
    )?;
    Ok((recipe, frames))
}

#[test]
fn original_flames_flicker_without_moving_stones_or_logs() -> Result<()> {
    let (recipe, frames) = inputs()?;
    assert_eq!(recipe.frame_order, [0, 1, 0, 2]);
    assert_eq!(recipe.frame_ticks, 8);
    assert_eq!(frames.len(), 3);
    for frame in &frames[1..] {
        let changes = frame
            .indices
            .iter()
            .zip(&frames[0].indices)
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            changes >= 8,
            "flame keyframe must visibly change at native scale"
        );
        for y in 0..32 {
            for x in 0..32 {
                if !(8..24).contains(&x) || y >= 14 {
                    assert_eq!(
                        frame.indices[y * 32 + x],
                        frames[0].indices[y * 32 + x],
                        "static campfire moved at{x},{y}"
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn ambient_clips_reject_zero_duration_and_unbounded_frames() {
    assert!(
        AuthoredTileAnimation {
            frame_ticks: 0,
            frames: vec![[0; 16]; 4]
        }
        .validate()
        .is_err()
    );
    assert!(
        AuthoredTileAnimation {
            frame_ticks: 8,
            frames: vec![]
        }
        .validate()
        .is_err()
    );
    assert!(
        AuthoredTileAnimation {
            frame_ticks: 8,
            frames: vec![[0; 16]; 17]
        }
        .validate()
        .is_err()
    );
}

#[test]
#[ignore = "requires CRYSTAL_TILE_TEST_PACK with the reviewed static campfire"]
fn native_flame_clips_preserve_static_pack_and_never_animate_shared_ground() -> Result<()> {
    let path = std::env::var_os("CRYSTAL_TILE_TEST_PACK").context("CRYSTAL_TILE_TEST_PACK")?;
    let base = crystal_assets::read_verified_compiled_game_pack(path)?;
    let (recipe, frames) = inputs()?;
    let pack = crystal_assets::with_minnesota_tile_animation(
        &base,
        "mn_campfire",
        128,
        &frames,
        &recipe.frame_order,
        recipe.frame_ticks,
    )?;
    assert_eq!(
        serde_json::to_value(pack.data())?,
        serde_json::to_value(base.data())?
    );
    for (path, bytes) in base.runtime_files() {
        assert_eq!(
            pack.runtime_files().get(path),
            Some(bytes),
            "static asset changed: {path}"
        );
    }
    let clips: BTreeMap<u8, AuthoredTileAnimation> =
        serde_json::from_slice(&pack.runtime_files()["data/tilesets/mn_campfire_animations.json"])?;
    assert!(!clips.is_empty());
    let layout = &pack.runtime_files()["data/tilesets/mn_campfire_metatiles.bin"];
    for (glyph, clip) in clips {
        clip.validate()?;
        assert!(
            ![3, 20].contains(&glyph),
            "native flower/water glyph overwritten"
        );
        let tile = layout[128 * 16..129 * 16]
            .iter()
            .position(|value| *value == glyph)
            .context("owned glyph")?;
        assert_eq!(layout.iter().filter(|value| **value == glyph).count(), 1);
        for (step, &frame) in recipe.frame_order.iter().enumerate() {
            for y in 0..8 {
                let lo = clip.frames[step][y * 2];
                let hi = clip.frames[step][y * 2 + 1];
                for x in 0..8 {
                    let pixel = ((lo >> (7 - x)) & 1) | (((hi >> (7 - x)) & 1) << 1);
                    let expected =
                        frames[frame].indices[(tile / 4 * 8 + y) * 32 + tile % 4 * 8 + x];
                    assert_eq!(pixel, expected);
                }
            }
        }
    }
    let mut shared = frames[0].indices.clone();
    shared[0] = 1;
    let bad = vec![
        MinnesotaTilePixels {
            indices: frames[0].indices.clone(),
        },
        MinnesotaTilePixels { indices: shared },
    ];
    let error =
        crystal_assets::with_minnesota_tile_animation(&base, "mn_campfire", 128, &bad, &[0, 1], 8)
            .expect_err("shared native floor must not animate");
    assert!(error.to_string().contains("shared"));
    Ok(())
}
