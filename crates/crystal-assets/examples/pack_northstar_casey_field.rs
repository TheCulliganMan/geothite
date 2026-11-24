//! Build a local Northstar pack with Casey's original field sprite.

use std::path::PathBuf;

use anyhow::{Result, bail, ensure};
use crystal_assets::{build_northstar_casey_field, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [parent, output] = args.as_slice() else {
        bail!("usage: pack_northstar_casey_field PARENT_PACK FRESH_IGNORED_OUTPUT");
    };
    ensure!(
        !output.exists()
            && std::process::Command::new("git")
                .args(["check-ignore", "--quiet", "--no-index", "--"])
                .arg(output)
                .status()?
                .success(),
        "choose a fresh ignored output path"
    );
    let pack = build_northstar_casey_field(&read_verified_compiled_game_pack(parent)?)?;
    pack.write_preserving_storage(output)?;
    ensure!(
        read_verified_compiled_game_pack(output)? == pack,
        "Casey field pack changed on readback"
    );
    Ok(())
}
