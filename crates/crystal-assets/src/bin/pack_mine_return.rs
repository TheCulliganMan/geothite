//! Build an unselected review pack for the mine return drift.
use anyhow::{Result, ensure};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 2,
        "usage: pack_mine_return BASE FRESH_IGNORED_OUTPUT"
    );
    ensure!(
        !args[1].exists()
            && std::process::Command::new("git")
                .args(["check-ignore", "--quiet", "--no-index", "--"])
                .arg(&args[1])
                .status()?
                .success(),
        "output must be fresh and ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let pack = crystal_assets::build_mine_return(&base)?;
    crystal_assets::verify_compiled_game_pack_for_runtime(&pack)?;
    pack.write_preserving_storage(&args[1])?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&args[1])? == pack,
        "mine review readback differs"
    );
    println!("{}", args[1].display());
    Ok(())
}
