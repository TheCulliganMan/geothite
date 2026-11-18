//! Write an unselected Route 3 review pack without opening a game session.
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 2,
        "usage: pack_river_terrace BASE FRESH_IGNORED_OUTPUT"
    );
    ensure!(
        !args[1].try_exists()?
            && std::process::Command::new("git")
                .args(["check-ignore", "--quiet", "--no-index", "--"])
                .arg(&args[1])
                .status()?
                .success(),
        "output must be fresh and ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let pack = crystal_assets::revise_northstar_river_terrace(&base)?;
    pack.write_preserving_storage(&args[1])?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&args[1])? == pack,
        "terrace readback differs"
    );
    println!("{}", args[1].display());
    Ok(())
}
