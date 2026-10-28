//! Resolve a gym collection or its neighboring revision index through Rust.
use std::collections::BTreeSet;
use std::path::{Component, Path};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::gym_design::GymDesign;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    schema_version: u8,
    base: String,
    overrides: Vec<Revision>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    map: String,
    source: String,
}

fn read_peer(directory: &Path, name: &str) -> Result<Value> {
    let path = Path::new(name);
    ensure!(
        path.components().count() == 1
            && matches!(path.components().next(), Some(Component::Normal(_))),
        "gym index sources must be neighboring files"
    );
    let path = directory.join(path);
    serde_json::from_str(
        &std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
}

/// Reads a complete eight-gym design, resolving an optional local revision index.
///
/// A collection retains its original text. An index names a neighboring base
/// collection and zero or more single-room revision files. Map identities must
/// match exactly, and each map and revision file can be selected only once.
/// The complete merged design receives the same gameplay contract validation as
/// the pack builder. This function does not write files or generate game content.
///
/// # Errors
/// Returns an error for I/O or JSON failures, unknown index/design fields, paths
/// outside the index directory, duplicates, mismatched rooms or invalid designs.
pub fn read_minnesota_gym_design(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read gym selection {}", path.display()))?;
    let value: Value = serde_json::from_str(&text).context("gym collection or index")?;
    if value.get("base").is_none() {
        serde_json::from_value::<GymDesign>(value)?.validate()?;
        return Ok(text);
    }
    let selection: Selection = serde_json::from_value(value)?;
    ensure!(selection.schema_version == 1, "gym selection schema");
    let directory = path.parent().context("gym index directory")?;
    let mut merged = read_peer(directory, &selection.base)?;
    serde_json::from_value::<GymDesign>(merged.clone())?.validate()?;
    let mut maps = BTreeSet::new();
    let mut files = BTreeSet::new();
    for revision in selection.overrides {
        ensure!(
            maps.insert(revision.map.clone()) && files.insert(revision.source.clone()),
            "a gym map or revision file was selected twice"
        );
        let source = read_peer(directory, &revision.source)?;
        let design: GymDesign = serde_json::from_value(source.clone())?;
        ensure!(
            design.schema_version == 1
                && !design.status.is_empty()
                && design.gyms.len() == 1
                && design.gyms[0].id == revision.map,
            "revision must contain its one named gym: {}",
            revision.map
        );
        let gyms = merged["gyms"]
            .as_array_mut()
            .context("base gym collection")?;
        let room = gyms
            .iter_mut()
            .find(|gym| gym["id"] == revision.map)
            .with_context(|| format!("unknown gym selection {}", revision.map))?;
        *room = source["gyms"][0].clone();
    }
    serde_json::from_value::<GymDesign>(merged.clone())?.validate()?;
    serde_json::to_string(&merged).context("resolved gym collection")
}
