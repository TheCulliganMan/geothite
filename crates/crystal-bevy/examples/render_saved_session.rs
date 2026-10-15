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
            .context("usage: render_saved_session PACK SAVE OUTPUT [--pair]")?,
    );
    let save_path = PathBuf::from(args.next().context("missing SAVE")?);
    let output = PathBuf::from(args.next().context("missing OUTPUT")?);
    let pair = match args.next().as_deref() {
        None => false,
        Some("--pair") => true,
        _ => anyhow::bail!("only --pair is supported after OUTPUT"),
    };
    anyhow::ensure!(args.next().is_none(), "unexpected extra arguments");
    let root = AssetRoot::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    );
    let loaded = read_loaded_verified_compiled_game_pack(pack)?;
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    let first = if pair {
        output.with_file_name(format!(
            "{}-2d.png",
            output
                .file_stem()
                .context("output filename")?
                .to_string_lossy()
        ))
    } else {
        output.clone()
    };
    let second = pair.then(|| {
        output.with_file_name(format!(
            "{}-3d.png",
            output.file_stem().unwrap().to_string_lossy()
        ))
    });
    crystal_bevy::run_bevy_shell(
        root,
        runtime,
        BevyShellStart::LoadSave { save_path },
        BevyShellConfig {
            voxel_view_enabled: Some(!pair),
            render_test_screenshot: Some(first),
            render_test_second_screenshot: second,
            window_title: Some("Geothite | saved companion session capture".into()),
            ..Default::default()
        },
    )?;
    Ok(())
}
