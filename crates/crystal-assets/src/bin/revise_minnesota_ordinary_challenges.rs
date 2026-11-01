//! Write a fresh, ignored pack with ordinary Minnesota trainer challenges revised.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 2,
        "usage: revise_minnesota_ordinary_challenges PARENT FRESH_OUTPUT"
    );
    let output = Path::new(&args[1]);
    ensure!(!output.exists(), "choose a fresh output");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated pack must remain ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let revised = crystal_assets::revise_minnesota_ordinary_challenges(&base)?;
    revised.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == revised,
        "pack readback differs"
    );
    println!("{} {}", output.display(), revised.identity()?.content_hash);
    Ok(())
}
