//! Publish per-trainer battle artwork while preserving gameplay and shared art.
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, ensure};

const SAVE_POLICY: &str = "data/northstar-save-compatibility.json";

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [parent, plan, output] = args.as_slice() else {
        anyhow::bail!("usage: pack_trainer_portrait_overrides PARENT PLAN_JSON FRESH_OUTPUT");
    };
    ensure!(!output.try_exists()?, "choose a fresh output pack");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated pack output must be ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(parent)?;
    let pack = crystal_assets::add_minnesota_trainer_portrait_overrides(
        &base,
        &fs::read_to_string(plan)?,
    )?;
    ensure!(
        pack.data() == base.data() && pack.compiled_audio() == base.compiled_audio(),
        "preserve gameplay and audio"
    );
    ensure!(
        base.runtime_files().iter().all(|(path, bytes)| {
            path == crystal_assets::TRAINER_PORTRAIT_OVERRIDES_PATH
                || path == SAVE_POLICY
                || pack.runtime_files().get(path) == Some(bytes)
        }),
        "preserve every existing stock, private and field resource"
    );
    ensure!(
        pack.runtime_modpack_id()? != base.runtime_modpack_id()?
            && !pack.runtime_files().contains_key(SAVE_POLICY),
        "portrait preview must have an isolated save slot"
    );
    let added = pack
        .runtime_files()
        .keys()
        .filter(|path| !base.runtime_files().contains_key(*path))
        .count();
    drop(base);
    pack.write_preserving_storage(output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(
        written == pack,
        "portrait override pack changed during readback"
    );
    println!(
        "{} {} ({added} resources added)",
        output.display(),
        written.identity()?.content_hash
    );
    Ok(())
}
