//! Build an ignored review pack with a connecting Driftless passage.
use anyhow::{Result, ensure};
use std::path::Path;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "usage: pack_northstar_driftless_passage PARENT SOURCE FRESH_OUTPUT"
    );
    let output = Path::new(&args[2]);
    ensure!(!output.exists(), "choose a fresh output");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated pack must remain ignored"
    );
    let parent = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let source = std::fs::read_to_string(&args[1])?;
    let pack = crystal_assets::add_northstar_driftless_passage(&parent, &source)?;
    pack.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == pack,
        "pack readback differs"
    );
    println!("{} {}", output.display(), pack.identity()?.content_hash);
    Ok(())
}
