use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_wild_challenge_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base_path = PathBuf::from(
        args.next()
            .context("usage: pack_wild_challenge <base-pack> <output-pack>")?,
    )
    .canonicalize()?;
    let output = PathBuf::from(args.next().context("output pack is required")?);
    ensure!(
        args.next().is_none(),
        "usage: pack_wild_challenge <base-pack> <output-pack>"
    );
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    ensure!(
        output.canonicalize().ok().as_ref() != Some(&base_path),
        "output must differ from base pack"
    );
    let base = read_verified_compiled_game_pack(base_path)?;
    let pack = build_wild_challenge_modpack(&base)?;
    pack.write_preserving_storage(&output)?;
    let loaded = read_verified_compiled_game_pack(&output)?;
    ensure!(
        pack.identity()? == loaded.identity()?,
        "written pack identity changed"
    );
    println!("exported {}", output.display());
    Ok(())
}
