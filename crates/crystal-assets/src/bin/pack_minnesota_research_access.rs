//! Publish free early starters and a complete paired-fossil path.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 2,
        "usage: pack_minnesota_research_access PARENT FRESH_OUTPUT"
    );
    let output = Path::new(&args[1]);
    ensure!(
        !output.exists(),
        "preserve completed packs; choose a fresh output"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated pack must remain ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let pack = crystal_assets::build_minnesota_research_access(&base)?;
    pack.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == pack,
        "written research pack differs"
    );
    println!("{} {}", output.display(), pack.identity()?.content_hash);
    Ok(())
}
