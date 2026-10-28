//! Publish only the bounded full-state connector contract through native helpers.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_state_cohesion, read_verified_compiled_game_pack};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn source_path(contract: &Path, name: &str) -> PathBuf {
    let path = PathBuf::from(name);
    if path.is_file() {
        path
    } else {
        contract
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path)
    }
}
fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: build_state_cohesion PACK CONTRACT OUTPUT")?,
    );
    let contract_path = PathBuf::from(args.next().context("cohesion contract")?);
    let output = PathBuf::from(args.next().context("new output pack")?);
    ensure!(args.next().is_none(), "unexpected argument");
    ensure!(
        !output.exists(),
        "cohesion publication must use a fresh output path"
    );
    let contract = std::fs::read_to_string(&contract_path)?;
    let plan: Value = serde_json::from_str(&contract)?;
    ensure!(
        plan["base_pack_sha256"].as_str()
            == Some(format!("{:x}", Sha256::digest(std::fs::read(&input)?)).as_str()),
        "actual parent file SHA differs"
    );
    let mut sources = BTreeMap::new();
    let patches = plan["native_port_patches"]
        .as_str()
        .context("port patch path")?;
    sources.insert(
        patches.to_owned(),
        std::fs::read_to_string(source_path(&contract_path, patches))?,
    );
    for map in plan["new_maps"]
        .as_array()
        .context("four new source maps")?
    {
        for field in ["layout_file", "dossier_file"] {
            let path = map[field].as_str().context("source path")?;
            ensure!(
                sources
                    .insert(
                        path.to_owned(),
                        std::fs::read_to_string(source_path(&contract_path, path))?
                    )
                    .is_none(),
                "duplicate source path"
            );
        }
    }
    let base = read_verified_compiled_game_pack(input.canonicalize()?)?;
    let revised = build_state_cohesion(&base, &contract, &sources)?;
    revised.write_preserving_storage(&output)?;
    let checked = read_verified_compiled_game_pack(output.canonicalize()?)
        .context("strict written cohesion readback")?;
    ensure!(
        checked.data().maps.len() == 681,
        "written cohesion map count differs"
    );
    println!(
        "published bounded state cohesion: {} ({})",
        output.display(),
        checked
            .identity()
            .context("published identity")?
            .content_hash
    );
    Ok(())
}
