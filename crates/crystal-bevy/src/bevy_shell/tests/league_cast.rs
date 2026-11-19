//! Read the League cast through the native field and battle renderers without gameplay.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use bevy::prelude::{Assets, Image};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_runtime::CrystalRuntime;

use super::super::{
    SpriteFrame, load_gbcpal_palette, load_oak_intro_frame, load_sprite_art,
    normalize_battle_trainer_sprite_id,
};

fn pixels(images: &Assets<Image>, frame: &SpriteFrame) -> image::RgbaImage {
    let image = images.get(&frame.handle).expect("native image");
    image::RgbaImage::from_raw(
        image.width(),
        image.height(),
        image.data.clone().expect("native pixels"),
    )
    .expect("native frame dimensions")
}

#[test]
#[ignore = "requires an external MN_LEAGUE_FAIR_PACK"]
fn elite_four_portraits_and_walking_frames_resolve_from_installed_classes() -> Result<()> {
    let path = PathBuf::from(std::env::var_os("MN_LEAGUE_FAIR_PACK").context("League pack")?);
    let game = CrystalRuntime::from_loaded_compiled_pack(
        &AssetRoot::new(path.parent().context("pack directory")?),
        read_loaded_verified_compiled_game_pack(&path)?,
    )?;
    let root = game.materialize_runtime_files()?;
    let mut images = Assets::<Image>::default();
    for (index, name) in ["FERN", "FLINT", "VEIL", "HALDEN"].iter().enumerate() {
        let number = index + 1;
        let trainer = &game.data().trainers.trainers[&format!("MN_LEAGUE_{number}")];
        ensure!(
            trainer.trainer_class == format!("MN_ELITE_{name}"),
            "League identity"
        );
        let stem = normalize_battle_trainer_sprite_id(&trainer.trainer_class);
        let portrait = load_oak_intro_frame(&root, &format!("battle-trainer:{stem}"), &mut images)?;
        let picture = pixels(&images, &portrait);
        let palette = load_gbcpal_palette(
            &root
                .runtime_assets()
                .join(format!("gfx/trainers/{stem}.gbcpal")),
        )?;
        ensure!(picture.dimensions() == (56, 56), "battle portrait size");
        for rgb in &palette[1..] {
            ensure!(
                picture
                    .pixels()
                    .any(|p| p.0 == [rgb[0], rgb[1], rgb[2], 255]),
                "missing visible {name} ink"
            );
        }
        let room_name = format!("MnLeagueRoom{number}");
        let actor = game.data().maps[&room_name]
            .objects
            .iter()
            .find(|actor| actor.script == format!("{room_name}Challenge"))
            .context("League challenger")?;
        let field_id = actor
            .sprite
            .strip_prefix("SPRITE_")
            .context("field sprite")?
            .to_ascii_lowercase();
        ensure!(field_id == stem, "field and battle identity");
        let art = load_sprite_art(&root, &field_id, 0, "day", &mut images)?;
        let mut sheet = image::RgbaImage::new(64, 88);
        image::imageops::replace(&mut sheet, &picture, 4, 0);
        for (x, dir) in [&art.down, &art.up, &art.left, &art.right]
            .iter()
            .enumerate()
        {
            let idle = pixels(&images, &dir.standing);
            let step = pixels(&images, dir.walking.as_ref().context("walking frame")?);
            ensure!(
                idle.dimensions() == (16, 16) && step.dimensions() == (16, 16) && idle != step,
                "distinct {name} walking frames"
            );
            image::imageops::replace(&mut sheet, &idle, 16 * x as i64, 56);
            image::imageops::replace(&mut sheet, &step, 16 * x as i64, 72);
        }
        if let Some(dir) = std::env::var_os("MN_LEAGUE_RENDER_DIR") {
            let dir = PathBuf::from(dir);
            std::fs::create_dir_all(&dir)?;
            sheet.save(dir.join(format!("{stem}.png")))?;
        }
    }
    Ok(())
}
