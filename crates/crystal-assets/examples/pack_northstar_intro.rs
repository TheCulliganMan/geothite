//! Build and read back a scoped Northstar opening revision.
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [parent, source, output] = args.as_slice() else {
        anyhow::bail!("usage: pack_northstar_intro PARENT STORY_JSON FRESH_OUTPUT");
    };
    ensure!(!output.try_exists()?, "choose a fresh output pack");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "pack output must be ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(parent)?;
    let story = std::fs::read_to_string(source)?;
    let pack = crystal_assets::polish_northstar_intro(&base, &story)?;
    drop(base);
    pack.write_preserving_storage(output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(
        written == pack,
        "Northstar intro pack changed during readback"
    );
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
