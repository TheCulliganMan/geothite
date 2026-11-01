//! Publish a native Crystal-style Minnesota State Fairgrounds candidate.
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        (4..=5).contains(&args.len()),
        "usage: pack_state_fair BASE LAYOUT DOSSIER OUTPUT [CITY_CONNECTION_JSON]"
    );
    ensure!(!args[3].exists(), "choose a new immutable output pack");
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let mut design: serde_json::Value = serde_json::from_str(&fs::read_to_string(&args[2])?)?;
    if let Some(path) = args.get(4) {
        *design
            .as_object_mut()
            .and_then(|fields| fields.get_mut("native_city_connection"))
            .context("fair dossier city connection")? =
            serde_json::from_str(&fs::read_to_string(path)?)?;
    }
    let pack = crystal_assets::build_state_fair_modpack(
        &base,
        &fs::read_to_string(&args[1])?,
        &serde_json::to_string(&design)?,
    )?;
    pack.write_preserving_storage(&args[3])?;
    let restored = crystal_assets::read_verified_compiled_game_pack(&args[3])?;
    ensure!(
        restored.identity()? == pack.identity()?,
        "fair pack readback differs"
    );
    println!(
        "Published native State Fairgrounds with separate counter NPCs: {}",
        args[3].display()
    );
    Ok(())
}
