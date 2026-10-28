//! Publish explicit additive river sources through the native Rust compiler.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_authored_river_connections, read_verified_compiled_game_pack};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn source_path(manifest: &Path, name: &str) -> PathBuf {
    let path = PathBuf::from(name);
    if path.exists() {
        path
    } else {
        manifest
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path)
    }
}
fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: build_mississippi_connections PACK MANIFEST OUTPUT")?,
    );
    let manifest_path = PathBuf::from(args.next().context("river manifest")?);
    let output = PathBuf::from(args.next().context("output pack")?);
    ensure!(args.next().is_none(), "unexpected argument");
    ensure!(
        input.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from input"
    );
    let manifest = std::fs::read_to_string(&manifest_path)?;
    let value: Value = serde_json::from_str(&manifest)?;
    ensure!(
        value["base_pack_sha256"].as_str()
            == Some(format!("{:x}", Sha256::digest(std::fs::read(&input)?)).as_str()),
        "actual input file SHA differs from manifest"
    );
    let mut layouts = BTreeMap::new();
    for map in value["new_maps"].as_array().context("river source maps")? {
        let name = map["layout"].as_str().context("river layout path")?;
        if !layouts.contains_key(name) {
            layouts.insert(
                name.to_owned(),
                std::fs::read_to_string(source_path(&manifest_path, name))?,
            );
        }
    }
    let inventory = value["bridge_inventory"]
        .as_str()
        .context("native bridge inventory path")?;
    let bridges = std::fs::read_to_string(source_path(&manifest_path, inventory))?;
    let base = read_verified_compiled_game_pack(&input)?;
    let revised = build_authored_river_connections(&base, &manifest, &layouts, &bridges)?;
    revised.write_preserving_storage(&output)?;
    let checked =
        read_verified_compiled_game_pack(&output).context("strict river publication readback")?;
    println!(
        "published additive Mississippi connections: {} ({})",
        output.display(),
        checked
            .identity()
            .context("published identity")?
            .content_hash
    );
    Ok(())
}
