//! Build the local native cave wall finish.
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_northstar_cave_wall_finish, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1).map(PathBuf::from);
    let path = args
        .next()
        .context("usage: pack_northstar_cave_wall_finish PARENT SOURCES OUTPUT")?;
    let source = args
        .next()
        .context("missing cave finish source directory")?;
    let output = args.next().context("missing cave finish pack path")?;
    ensure!(
        args.next().is_none() && !output.exists(),
        "expected a new output path"
    );
    ensure!(
        Command::new("git")
            .args(["check-ignore", "--quiet", "--"])
            .arg(&output)
            .status()?
            .success(),
        "pack must stay ignored"
    );
    let natural = fs::read_to_string(source.join("natural-edges.json"))?;
    let mine = fs::read_to_string(source.join("mine-edges.json"))?;
    let parent = read_verified_compiled_game_pack(path)?;
    let candidate = build_northstar_cave_wall_finish(&parent, &[&natural, &mine])?;
    candidate.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == candidate,
        "cave finish differs on readback"
    );
    println!("{}", output.display());
    Ok(())
}
