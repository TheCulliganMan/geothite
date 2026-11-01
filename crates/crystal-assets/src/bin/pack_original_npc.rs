//! Publish one original NPC with native art and a real Minnesota encounter.
use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{
    build_original_npc_modpack, read_verified_compiled_game_pack, revise_original_npc_overworld,
};

fn main() -> Result<()> {
    let mut args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let revise = args
        .first()
        .is_some_and(|arg| arg.to_str() == Some("--revise-overworld"));
    if revise {
        args.remove(0);
    }
    ensure!(
        args.len() == 4,
        "usage: pack_original_npc [--revise-overworld] BASE PLACEMENT NATIVE_ART_ROOT OUTPUT"
    );
    ensure!(!args[3].exists(), "preserve existing packs");
    let source = std::fs::read_to_string(&args[1])?;
    let value: serde_json::Value = serde_json::from_str(&source)?;
    let class = value["trainers"][0]["trainer_class"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing NPC class"))?;
    let stem = class.to_ascii_lowercase();
    ensure!(
        stem.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'),
        "invalid art stem"
    );
    let mut art = BTreeMap::new();
    for kind in ["sprites", "trainers"] {
        for extension in ["png", "2bpp", "gbcpal"] {
            let path = format!("gfx/{kind}/{stem}.{extension}");
            art.insert(path.clone(), std::fs::read(args[2].join(path))?);
        }
    }
    let base = read_verified_compiled_game_pack(&args[0])?;
    let pack = if revise {
        revise_original_npc_overworld(&base, &source, &art)?
    } else {
        build_original_npc_modpack(&base, &source, &art)?
    };
    pack.write_preserving_storage(&args[3])?;
    let loaded = read_verified_compiled_game_pack(&args[3])?;
    ensure!(
        pack.identity()? == loaded.identity()?,
        "pack readback identity changed"
    );
    ensure!(
        loaded.data().minnesota_trainer_scaling == base.data().minnesota_trainer_scaling,
        "one NPC must preserve existing trainer scaling"
    );
    ensure!(
        loaded.data().trainers.trainers.len()
            == base.data().trainers.trainers.len() + usize::from(!revise),
        "trainer count must match publication mode"
    );
    for (id, trainer) in &base.data().trainers.trainers {
        ensure!(
            loaded.data().trainers.trainers.get(id) == Some(trainer),
            "existing trainer changed: {id}"
        );
    }
    if revise {
        ensure!(
            loaded.data() == base.data(),
            "overworld revision changed gameplay data"
        );
        ensure!(
            loaded.compiled_audio() == base.compiled_audio(),
            "overworld revision changed audio"
        );
    }
    let mutable_paths = [
        format!("gfx/sprites/{stem}.png"),
        format!("gfx/sprites/{stem}.2bpp"),
    ];
    for (path, bytes) in base.runtime_files() {
        if revise && mutable_paths.contains(path) {
            continue;
        }
        ensure!(
            loaded.runtime_files().get(path) == Some(bytes),
            "existing runtime art changed: {path}"
        );
    }
    let mode = if revise {
        "hand-edited overworld"
    } else {
        "one original NPC"
    };
    println!("Published {mode}: {}", args[3].display());
    Ok(())
}
