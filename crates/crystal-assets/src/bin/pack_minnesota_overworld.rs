//! Add reviewed native overworld art without modifying species or audio.
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::minnesota_overworld_sprites::OverworldManifest;

fn main() -> Result<()> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        arguments.len() >= 3,
        "usage: pack_minnesota_overworld BASE_PACK OUTPUT_PACK MANIFEST_JSON..."
    );
    let base = Path::new(&arguments[0]).canonicalize()?;
    let output = PathBuf::from(&arguments[1]);
    fs::create_dir_all(output.parent().context("output directory")?)?;
    let resolved = output
        .parent()
        .context("output directory")?
        .canonicalize()?
        .join(output.file_name().context("output filename")?);
    ensure!(resolved != base, "use a separate output pack");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&resolved)
            .status()?
            .success(),
        "output pack must remain ignored"
    );
    let mut pack = crystal_assets::read_verified_compiled_game_pack(base)?;
    for file in &arguments[2..] {
        let path = Path::new(file).canonicalize()?;
        let manifest = OverworldManifest::from_json(&fs::read(&path)?)?;
        let compiled = manifest.compile(path.parent().context("manifest directory")?)?;
        pack = compiled.build_pack(&pack)?;
    }
    pack.write_preserving_storage(&resolved)?;
    let verified = crystal_assets::read_verified_compiled_game_pack(&resolved)?;
    ensure!(
        verified.identity()? == pack.identity()?,
        "written pack differs"
    );
    println!("Verified overworld art pack: {}", resolved.display());
    Ok(())
}
