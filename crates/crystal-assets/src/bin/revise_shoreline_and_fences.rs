//! Publish exact native shore and route layout revisions onto a guarded parent.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crystal_assets::{
    clear_duluth_inner_buoys, read_verified_compiled_game_pack, revise_authored_route_terrain,
    revise_duluth_shoreline, validate_compact_superior_geography,
};

#[derive(Deserialize)]
struct Plan {
    #[serde(default)]
    clear_inner_buoys: bool,
    parent_pack: PathBuf,
    parent_pack_sha256: String,
    shore_before: PathBuf,
    shore_after: PathBuf,
    routes: Vec<Route>,
}

#[derive(Deserialize)]
struct Route {
    before: PathBuf,
    after: PathBuf,
    dossier: PathBuf,
}

fn read(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| path.display().to_string())
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let plan: Plan = serde_json::from_str(&read(Path::new(
        &args
            .next()
            .context("usage: revise_shoreline_and_fences PLAN OUTPUT")?,
    ))?)?;
    let output = PathBuf::from(args.next().context("fresh ignored output pack")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "fresh output and exact arguments required"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "pack output must be ignored"
    );
    let digest = || -> Result<String> {
        Ok(format!(
            "{:x}",
            Sha256::digest(std::fs::read(&plan.parent_pack)?)
        ))
    };
    ensure!(
        digest()? == plan.parent_pack_sha256,
        "parent SHA256 differs"
    );
    let base = read_verified_compiled_game_pack(&plan.parent_pack)?;
    let revise = if plan.clear_inner_buoys {
        clear_duluth_inner_buoys
    } else {
        revise_duluth_shoreline
    };
    let mut revised = revise(&base, &read(&plan.shore_before)?, &read(&plan.shore_after)?)?;
    for route in plan.routes {
        revised = revise_authored_route_terrain(
            &revised,
            &read(&route.before)?,
            &read(&route.after)?,
            &read(&route.dossier)?,
        )
        .with_context(|| format!("route terrain {}", route.after.display()))?;
    }
    validate_compact_superior_geography(&revised)?;
    ensure!(
        digest()? == plan.parent_pack_sha256,
        "parent changed during publication"
    );
    revised.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("publication readback")?;
    println!("{}", output.display());
    Ok(())
}
