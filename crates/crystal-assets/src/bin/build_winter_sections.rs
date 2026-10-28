//! Strict publication of exact frozen winter sections onto a chosen art parent.
use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_winter_sections_modpack, read_verified_compiled_game_pack};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 4,
        "usage: build_winter_sections PARENT CONTRACT EXPECTED_PARENT_SHA256 NEW_OUTPUT"
    );
    let parent = PathBuf::from(&args[0])
        .canonicalize()
        .context("completed winter art/region/mixed-edge parent")?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&parent)?)) == args[2],
        "actual chosen parent file SHA differs"
    );
    let contract = std::fs::read_to_string(&args[1]).context("frozen winter-section contract")?;
    let value: Value = serde_json::from_str(&contract)?;
    let mut sources = BTreeMap::new();
    for route in value["routes"].as_array().context("winter routes")? {
        for name in route["sources"]
            .as_object()
            .context("winter exact input files")?
            .keys()
        {
            sources.insert(
                name.clone(),
                std::fs::read_to_string(name).with_context(|| format!("winter input: {name}"))?,
            );
        }
    }
    let requested = PathBuf::from(&args[3]);
    ensure!(
        !requested.exists(),
        "winter sections require a fresh output path"
    );
    let output = requested
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(requested.file_name().context("output filename")?);
    let base = read_verified_compiled_game_pack(&parent)?;
    let pack = build_winter_sections_modpack(&base, &contract, &sources)?;
    pack.write_preserving_storage(&output)?;
    let written = read_verified_compiled_game_pack(&output)
        .context("strict written winter-section readback")?;
    ensure!(
        written == pack,
        "written winter sections differ from validated in-memory pack"
    );
    println!(
        "Published winter sections: {} ({})",
        output.display(),
        written.identity()?.content_hash
    );
    Ok(())
}
