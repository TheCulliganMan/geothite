//! Build the eight-Gym revision from the published refinement without playing it.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectoryRevision {
    schema_version: u8,
    base: PathBuf,
}

fn main() -> Result<()> {
    let paths: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        (6..=7).contains(&paths.len()),
        "usage: pack_northstar_gyms REFINEMENT_RAW REFINEMENT_HOSTED DESIGN_DIRECTORY STORY_JSON POKEMON_JSON FRESH_OUTPUT [CAVE_DESIGN_JSON]"
    );
    let output = &paths[5];
    ensure!(!output.exists(), "choose a fresh candidate output");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated candidate must remain ignored"
    );
    let raw = crystal_assets::read_loaded_verified_compiled_game_pack(&paths[0])?;
    let hosted = crystal_assets::read_loaded_verified_compiled_game_pack(&paths[1])?;
    let revision_path = paths[2].join("revision.json");
    let fallback = if revision_path.exists() {
        let revision: DirectoryRevision = serde_json::from_slice(&std::fs::read(revision_path)?)?;
        ensure!(
            revision.schema_version == 1 && revision.base.is_relative(),
            "relative Gym source revision"
        );
        Some(paths[2].join(revision.base))
    } else {
        None
    };
    let read_source = |name: &str| -> Result<Value> {
        let local = paths[2].join(name);
        let path = if local.try_exists()? {
            local
        } else {
            fallback
                .as_ref()
                .with_context(|| format!("missing Gym source {name}"))?
                .join(name)
        };
        Ok(serde_json::from_slice(
            &std::fs::read(&path).with_context(|| path.display().to_string())?,
        )?)
    };
    let mut rooms = Vec::new();
    let mut contracts = BTreeMap::new();
    for name in [
        "rochester",
        "mankato",
        "minneapolis",
        "st-paul",
        "st-cloud",
        "brainerd",
        "bemidji",
        "duluth",
    ] {
        let document = read_source(&format!("{name}.json"))?;
        let room = document["gyms"]
            .as_array()
            .and_then(|rooms| (rooms.len() == 1).then(|| &rooms[0]))
            .context("one room per authored source")?
            .clone();
        let id = room["id"].as_str().context("room ID")?.to_owned();
        let contract = read_source(&format!("{name}-contract.json"))?;
        rooms.push(room);
        contracts.insert(id, contract);
    }
    let design =
        serde_json::to_string(&json!({"schema_version":1,"rooms":rooms,"contracts":contracts}))?;
    let caves = paths.get(6).map(std::fs::read_to_string).transpose()?;
    let candidate = crystal_assets::overhaul_northstar_gyms(
        &raw,
        &hosted,
        &design,
        &std::fs::read_to_string(&paths[3])?,
        &std::fs::read_to_string(&paths[4])?,
        caves.as_deref(),
    )?;
    candidate.write_preserving_storage(output)?;
    let readback = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(readback == candidate, "Gym pack readback mismatch");
    println!("{} {}", output.display(), readback.identity()?.content_hash);
    Ok(())
}
