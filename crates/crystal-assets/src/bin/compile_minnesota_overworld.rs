//! Compile an original family sheet without exporting or modifying any pack.
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::minnesota_overworld_sprites::OverworldManifest;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 4 && args[0] == "--manifest" && args[2] == "--output-dir",
        "usage: compile_minnesota_overworld --manifest MANIFEST.json --output-dir IGNORED_DIRECTORY"
    );
    let manifest_path = PathBuf::from(&args[1]).canonicalize()?;
    let output = PathBuf::from(&args[3]);
    let manifest = OverworldManifest::from_json(&fs::read(&manifest_path)?)?;
    let compiled = manifest.compile(manifest_path.parent().context("manifest directory")?)?;
    compiled.write_to(&output)?;
    println!(
        "Compiled {}: {} species, {} literal OBJ runtime files; retained reviews in {}",
        manifest.family_id,
        manifest.species.len(),
        compiled.runtime_files.len(),
        output.display()
    );
    Ok(())
}
