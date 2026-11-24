//! Write a standalone local cave redesign pack from Fair Presentation.
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_northstar_cave_redesign, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1).map(PathBuf::from);
    let base = args
        .next()
        .context("usage: pack_northstar_cave_redesign BASE SOURCES OUTPUT")?;
    let sources = args.next().context("missing cave source directory")?;
    let output = args.next().context("missing candidate pack destination")?;
    ensure!(
        args.next().is_none() && !output.exists(),
        "expected a new candidate path"
    );
    ensure!(
        Command::new("git")
            .args(["check-ignore", "--quiet", "--"])
            .arg(&output)
            .status()?
            .success(),
        "candidate pack must stay ignored"
    );
    let sources = ["south.json", "north.json", "mines.json", "ice.json"]
        .map(|name| {
            fs::read_to_string(sources.join(name)).with_context(|| format!("cave source {name}"))
        })
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    let source_text: Vec<_> = sources.iter().map(String::as_str).collect();
    let base = read_verified_compiled_game_pack(&base)?;
    let candidate = build_northstar_cave_redesign(&base, &source_text)?;
    candidate.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == candidate,
        "cave pack readback differs"
    );
    println!("{}", output.display());
    Ok(())
}
