//! Build and audit a private field-review pack without changing any selection.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let paths: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        paths.len() == 3,
        "usage: pack_northstar_field_review LAKESIDE_RAW FRESH_PACK FRESH_MANIFEST"
    );
    for output in &paths[1..] {
        ensure!(
            !output.try_exists()?
                && std::process::Command::new("git")
                    .args(["check-ignore", "--quiet", "--no-index", "--"])
                    .arg(output)
                    .status()?
                    .success(),
            "outputs must be fresh and ignored"
        );
    }
    let base = crystal_assets::read_loaded_verified_compiled_game_pack(&paths[0])?;
    let pack = crystal_assets::build_northstar_field_review(&base)?;
    pack.write_preserving_storage(&paths[1])?;
    let written = crystal_assets::read_verified_compiled_game_pack(&paths[1])?;
    ensure!(written == pack, "private review readback differs");
    let mut manifest = crystal_assets::field_review_manifest(base.pack(), &written)?;
    manifest["content_identity"] = written.identity()?.content_hash.into();
    manifest["pack_sha256"] = format!("{:x}", Sha256::digest(std::fs::read(&paths[1])?)).into();
    std::fs::write(&paths[2], serde_json::to_vec_pretty(&manifest)?)?;
    println!("{}\n{}", paths[1].display(), paths[2].display());
    Ok(())
}
