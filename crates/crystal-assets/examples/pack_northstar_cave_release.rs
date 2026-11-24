use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{
    build_northstar_cave_release, read_loaded_verified_compiled_game_pack,
    read_verified_compiled_game_pack,
};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [raw_parent, hosted_parent, reviewed, raw, hosted] = args.as_slice() else {
        anyhow::bail!(
            "usage: pack_northstar_cave_release RAW_PARENT HOSTED_PARENT REVIEWED_PACK RAW_OUTPUT HOSTED_OUTPUT"
        );
    };
    ensure!(raw != hosted, "separate output paths required");
    for path in [raw, hosted] {
        ensure!(
            !path.exists()
                && std::process::Command::new("git")
                    .args(["check-ignore", "--quiet", "--no-index", "--"])
                    .arg(path)
                    .status()?
                    .success(),
            "choose fresh ignored output"
        );
    }
    let editions = build_northstar_cave_release(
        &read_loaded_verified_compiled_game_pack(raw_parent)?,
        &read_loaded_verified_compiled_game_pack(hosted_parent)?,
        &read_loaded_verified_compiled_game_pack(reviewed)?,
    )?;
    for (pack, path) in editions.into_iter().zip([raw, hosted]) {
        pack.write_preserving_storage(path)?;
        ensure!(
            read_verified_compiled_game_pack(path)? == pack,
            "pack changed on readback"
        );
        println!("{} {}", path.display(), pack.identity()?.content_hash);
    }
    Ok(())
}
