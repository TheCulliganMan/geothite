use anyhow::{Context, Result, ensure};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: certify_intro PACK TRACE EXTERNAL_WORKSPACE")?,
    );
    let trace = PathBuf::from(args.next().context("independent ROM trace required")?);
    let root =
        PathBuf::from(args.next().context("external source workspace required")?).canonicalize()?;
    ensure!(args.next().is_none(), "unexpected certification argument");
    let checkout = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    ensure!(
        !root.starts_with(checkout),
        "certified game content must remain external"
    );
    let raw_path = root.join("apps/web/assets/data/runtime_title_screen.unvalidated.json");
    let raw: crystal_assets::RuntimeTitleScreen =
        serde_json::from_slice(&std::fs::read(&raw_path)?)?;
    let loaded =
        crystal_assets::read_unverified_compiled_game_pack_for_intro_timing_diagnostic(&pack)?;
    let (_, _, loaded) = loaded.into_parts();
    ensure!(
        loaded.data().runtime_title_screen == raw,
        "diagnostic pack differs from the newly authored title catalog"
    );
    let title = crystal_bevy::bevy_shell::certify_source_intro_timeline(&pack, &trace)?;
    let bytes = serde_json::to_vec_pretty(&title)?;
    let data = root.join("apps/web/assets/data");
    let catalog = data.join("content-packs/core-modular/runtime_title_screen");
    std::fs::create_dir_all(&catalog)?;
    std::fs::write(data.join("runtime_title_screen.json"), &bytes)?;
    std::fs::write(catalog.join("runtime_title_screen.json"), bytes)?;
    println!("certified the continuous production intro against all 28 pinned ROM scenes");
    Ok(())
}
