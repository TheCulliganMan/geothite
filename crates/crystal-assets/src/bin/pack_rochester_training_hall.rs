//! Build a local, ignored Rochester training hall review pack.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "usage: pack_rochester_training_hall PARENT SOURCE FRESH_OUTPUT"
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
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let source = std::fs::read_to_string(&args[1])?;
    let result = crystal_assets::build_rochester_training_hall(&base, &source)?;
    result.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == result,
        "pack readback differs"
    );
    println!("{} {}", output.display(), result.identity()?.content_hash);
    Ok(())
}
