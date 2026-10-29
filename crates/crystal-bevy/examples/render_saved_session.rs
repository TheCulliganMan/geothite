//! Capture a real saved session through the production renderer, including
//! Surf users and party order. No game state or source scripts are synthesized.
use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::{BevyShellConfig, BevyShellStart};
use crystal_runtime::CrystalRuntime;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: render_saved_session PACK SAVE OUTPUT [--2d | --pair] [--walk ROUTE] [--quick-save PATH]")?,
    );
    let save_path = PathBuf::from(args.next().context("missing SAVE")?);
    let output = PathBuf::from(args.next().context("missing OUTPUT")?);
    let mut pair = false;
    let mut two_d = false;
    let mut walk = None;
    let mut quick_save_path = None;
    while let Some(option) = args.next() {
        match option.as_str() {
            "--pair" => pair = true,
            "--2d" => two_d = true,
            "--walk" => walk = Some(args.next().context("--walk requires a route")?),
            "--quick-save" => {
                quick_save_path = Some(PathBuf::from(
                    args.next().context("--quick-save requires a path")?,
                ));
            }
            _ => anyhow::bail!("unexpected option {option}"),
        }
    }
    anyhow::ensure!(!(two_d && pair), "--2d captures only the native 2D view");
    let root = AssetRoot::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    );
    let loaded = read_loaded_verified_compiled_game_pack(pack)?;
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    // Movement QA measures the published 3D scene as well as authoritative
    // steps, so run the route there before freezing the matched source view.
    let start_in_3d = !two_d && (!pair || walk.is_some());
    let first = if pair {
        output.with_file_name(format!(
            "{}-{}.png",
            output
                .file_stem()
                .context("output filename")?
                .to_string_lossy(),
            if start_in_3d { "3d" } else { "2d" },
        ))
    } else {
        output.clone()
    };
    let second = pair.then(|| {
        output.with_file_name(format!(
            "{}-{}.png",
            output.file_stem().unwrap().to_string_lossy(),
            if start_in_3d { "2d" } else { "3d" },
        ))
    });
    crystal_bevy::run_bevy_shell(
        root,
        runtime,
        BevyShellStart::LoadSave { save_path },
        BevyShellConfig {
            voxel_view_enabled: Some(start_in_3d),
            render_test_screenshot: Some(first),
            render_test_second_screenshot: second,
            render_test_walk: walk,
            // Explicit opt-in for authored save transactions during a review.
            // Omit it to keep ordinary read-only captures unchanged.
            quick_save_path,
            window_title: Some("Geothite | saved companion session capture".into()),
            ..Default::default()
        },
    )?;
    Ok(())
}
