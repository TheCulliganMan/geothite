//! Publish reviewed encounters and character bindings in one external pack.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    bind_minnesota_trainer_art, build_original_npc_modpack, distribute_minnesota_trainer_art,
    read_verified_compiled_game_pack,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema_version: u8,
    #[serde(default)]
    placements: Vec<PlacementFiles>,
    #[serde(default)]
    bindings: Vec<BindingFiles>,
    #[serde(default)]
    distribution: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlacementFiles {
    placement: PathBuf,
    native_art: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingFiles {
    binding: PathBuf,
    native_art: PathBuf,
}

fn native_art(integration: &str, root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let value: serde_json::Value = serde_json::from_str(integration)?;
    let stem = value["trainer_class"]
        .as_str()
        .or_else(|| value["trainers"][0]["trainer_class"].as_str())
        .context("private trainer class")?
        .to_ascii_lowercase();
    ensure!(
        stem.starts_with("mn_") && stem.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'),
        "native Minnesota resource stem required"
    );
    let mut art = BTreeMap::new();
    for kind in ["sprites", "trainers"] {
        for extension in ["png", "2bpp", "gbcpal"] {
            let path = format!("gfx/{kind}/{stem}.{extension}");
            let bytes = std::fs::read(root.join(&path)).with_context(|| path.clone())?;
            art.insert(path, bytes);
        }
    }
    Ok(art)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 3,
        "usage: pack_trainer_art_batch BASE PLAN_JSON NEW_PACK"
    );
    ensure!(!args[2].exists(), "preserve previous packs");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&args[2])
            .status()?
            .success(),
        "generated output must be ignored"
    );
    let plan: Plan = serde_json::from_str(&std::fs::read_to_string(&args[1])?)?;
    ensure!(
        plan.schema_version == 1
            && (!plan.placements.is_empty()
                || !plan.bindings.is_empty()
                || plan.distribution.is_some()),
        "nonempty version-one binding plan required"
    );
    let base = read_verified_compiled_game_pack(&args[0])?;
    let mut pack = base.clone();
    for entry in &plan.placements {
        let source = std::fs::read_to_string(&entry.placement)?;
        let art = native_art(&source, &entry.native_art)?;
        pack = build_original_npc_modpack(&pack, &source, &art)
            .with_context(|| format!("placement {}", entry.placement.display()))?;
    }
    for entry in &plan.bindings {
        let source = std::fs::read_to_string(&entry.binding)?;
        let art = native_art(&source, &entry.native_art)?;
        pack = bind_minnesota_trainer_art(&pack, &source, &art)
            .with_context(|| format!("binding {}", entry.binding.display()))?;
    }
    if let Some(path) = &plan.distribution {
        pack = distribute_minnesota_trainer_art(&pack, &std::fs::read_to_string(path)?)
            .with_context(|| format!("distribution {}", path.display()))?;
    }
    ensure!(
        base.compiled_audio() == pack.compiled_audio(),
        "preserve audio"
    );
    ensure!(
        pack.data().trainers.trainers.len()
            == base.data().trainers.trainers.len() + plan.placements.len(),
        "one additional trainer per placement"
    );
    ensure!(
        pack.data().minnesota_trainer_scaling == base.data().minnesota_trainer_scaling,
        "preserve trainer scaling"
    );
    for (path, bytes) in base.runtime_files() {
        ensure!(
            pack.runtime_files().get(path) == Some(bytes),
            "preserve original resource: {path}"
        );
    }
    pack.write_preserving_storage(&args[2])?;
    let restored = read_verified_compiled_game_pack(&args[2])?;
    ensure!(restored.identity()? == pack.identity()?, "written identity");
    println!(
        "Published {} encounters and {} reviewed character bindings: {}",
        plan.placements.len(),
        plan.bindings.len(),
        args[2].display()
    );
    Ok(())
}
