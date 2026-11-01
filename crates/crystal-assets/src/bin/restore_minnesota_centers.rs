//! Write a fresh, unselected regional Center restoration candidate.
use anyhow::{Result, ensure};
use std::path::Path;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 2,
        "usage: restore_minnesota_centers PARENT FRESH_OUTPUT"
    );
    let output = Path::new(&args[1]);
    ensure!(
        !output.exists(),
        "preserve existing packs; choose a fresh output"
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
    let candidate = crystal_assets::restore_minnesota_center_nurses(&base)?;
    candidate.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == candidate,
        "written candidate differs"
    );
    println!(
        "{} {}",
        output.display(),
        candidate.identity()?.content_hash
    );
    Ok(())
}
