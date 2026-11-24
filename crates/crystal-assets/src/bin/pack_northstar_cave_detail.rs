//! Build the Northstar cave layout pack.
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    build_northstar_cave_audit, build_northstar_cave_detail, read_verified_compiled_game_pack,
};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let parent = PathBuf::from(
        args.next()
            .context("usage: pack_northstar_cave_detail PARENT SOURCES OUTPUT [--audit]")?,
    );
    let sources = PathBuf::from(args.next().context("missing cave layout directory")?);
    let output = PathBuf::from(args.next().context("missing cave pack output")?);
    let audit = match args.next().as_deref() {
        None => false,
        Some(flag) if flag == OsStr::new("--audit") => true,
        Some(flag) => anyhow::bail!("unexpected cave build option {flag:?}"),
    };
    ensure!(
        args.next().is_none() && !output.exists(),
        "expected a new pack path"
    );
    ensure!(
        Command::new("git")
            .args(["check-ignore", "--quiet", "--"])
            .arg(&output)
            .status()?
            .success(),
        "pack must stay ignored"
    );
    let layouts = ["south.json", "north.json", "mines.json", "ice.json"]
        .map(|name| fs::read_to_string(sources.join(name)));
    let [south, north, mines, ice] = layouts;
    let art = [
        "mines-art.json",
        "south-art.json",
        "north-art.json",
        "ice-art.json",
    ]
    .map(|name| fs::read_to_string(sources.join(name)));
    let [mines_art, south_art, north_art, ice_art] = art;
    let parent = read_verified_compiled_game_pack(parent)?;
    let rooms: [&str; 4] = [&south?, &north?, &mines?, &ice?];
    let [mines_art, south_art, north_art, ice_art] =
        [&mines_art?, &south_art?, &north_art?, &ice_art?];
    let candidate = if audit {
        build_northstar_cave_audit(&parent, &rooms, &[north_art, south_art, mines_art, ice_art])?
    } else {
        build_northstar_cave_detail(&parent, &rooms, &[mines_art, south_art, north_art, ice_art])?
    };
    candidate.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == candidate,
        "compiled cave pack differs on readback"
    );
    println!("{}", output.display());
    Ok(())
}
