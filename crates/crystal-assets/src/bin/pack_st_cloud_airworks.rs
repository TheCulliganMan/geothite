//! Build an ignored five-gym review pack.
use anyhow::{Result, ensure};
use std::path::Path;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "usage: pack_st_cloud_airworks PARENT SOURCE FRESH_OUTPUT"
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
    let result =
        crystal_assets::build_st_cloud_airworks(&base, &std::fs::read_to_string(&args[1])?)?;
    result.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == result,
        "pack readback differs"
    );
    println!("{} {}", output.display(), result.identity()?.content_hash);
    Ok(())
}
