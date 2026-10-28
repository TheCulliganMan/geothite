//! Publish the exact reviewed Grand Marais native south-edge buoy line.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_grand_marais_harbor_boundary};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: revise_grand_marais_harbor BASE CONTRACT LAYOUT NEW_OUTPUT"
    );
    ensure!(
        !paths[3].exists(),
        "harbor publication requires a fresh output path"
    );
    let contract = std::fs::read_to_string(&paths[1]).context("accepted harbor contract")?;
    let value: Value = serde_json::from_str(&contract)?;
    ensure!(
        value["base_pack_sha256"].as_str()
            == Some(format!("{:x}", Sha256::digest(std::fs::read(&paths[0])?)).as_str()),
        "actual harbor parent file SHA differs"
    );
    let layout = std::fs::read_to_string(&paths[2]).context("accepted complete town layout")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let result = revise_grand_marais_harbor_boundary(&base, &contract, &layout)?;
    result.write_preserving_storage(&paths[3])?;
    let checked = read_verified_compiled_game_pack(&paths[3])
        .context("strict written harbor revision readback")?;
    ensure!(checked == result, "strict written harbor content differs");
    println!(
        "Published Grand Marais buoy boundary: {} ({})",
        paths[3].display(),
        checked.identity()?.content_hash
    );
    Ok(())
}
