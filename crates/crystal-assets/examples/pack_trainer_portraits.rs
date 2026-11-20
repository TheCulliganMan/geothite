//! Publish a battle-only trainer portrait revision to a fresh external pack.
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let arguments: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [parent, plan, output] = arguments.as_slice() else {
        anyhow::bail!("usage: pack_trainer_portraits PARENT PLAN_JSON FRESH_OUTPUT");
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
    let plan = std::fs::read_to_string(plan)?;
    let pack = crystal_assets::revise_minnesota_trainer_portraits(&base, &plan)?;
    ensure!(pack.data() == base.data(), "preserve all gameplay data");
    ensure!(
        pack.compiled_audio() == base.compiled_audio(),
        "preserve all audio"
    );
    let changed = base
        .runtime_files()
        .iter()
        .filter(|(key, bytes)| pack.runtime_files().get(*key) != Some(*bytes))
        .count();
    drop(base);
    pack.write_preserving_storage(output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(written == pack, "portrait pack changed during readback");
    println!(
        "{} {} ({changed} battle resources changed)",
        output.display(),
        written.identity()?.content_hash
    );
    Ok(())
}
