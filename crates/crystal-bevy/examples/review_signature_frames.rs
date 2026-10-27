//! Select actual visible effect poses from retained production captures.
//! This creates review sheets; it never redraws or approximates an effect.
//! Run with one argument: the root NORTHWOODS_ANIMATION_RENDER_DIR.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use image::RgbaImage;
use image::imageops::{self, FilterType};

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("usage: review_signature_frames CAPTURE_ROOT")?,
    );
    let mut reviewed = 0;
    for movement in [
        "magnetic_rail",
        "bog_siphon",
        "tip_up_strike",
        "slipper_waltz",
        "pike_drill",
    ] {
        for variant in [
            "player-normal",
            "player-shiny",
            "enemy-normal",
            "enemy-shiny",
        ] {
            let directory = root.join(movement).join(variant);
            let mut paths = fs::read_dir(&directory)?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<Result<Vec<_>, _>>()?;
            paths.retain(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("frame-") && name.ends_with(".png"))
            });
            paths.sort();
            let baseline = image::open(paths.first().ok_or("no production frames")?)?.into_rgba8();
            let mut seen = BTreeSet::new();
            let mut poses = Vec::new();
            for path in paths {
                let frame = image::open(path)?.into_rgba8();
                if frame.dimensions() != (160, 144) {
                    return Err("expected native 160×144 frames".into());
                }
                if frame != baseline && seen.insert(frame.as_raw().clone()) {
                    poses.push(frame);
                }
            }
            if poses.is_empty() {
                return Err("no visible effect poses".into());
            }
            let mut sheet = RgbaImage::new(160 * 6, 144);
            for column in 0..6 {
                let index = column * (poses.len() - 1) / 5;
                imageops::overlay(&mut sheet, &poses[index], column as i64 * 160, 0);
            }
            sheet.save(
                root.join(movement)
                    .join(format!("{variant}-effects-filmstrip.png")),
            )?;
            imageops::resize(&sheet, 1920, 288, FilterType::Nearest).save(
                root.join(movement)
                    .join(format!("{variant}-effects-2x.png")),
            )?;
            reviewed += 1;
        }
    }
    println!("Retained native and 2x effect review sheets for {reviewed} variants.");
    Ok(())
}
