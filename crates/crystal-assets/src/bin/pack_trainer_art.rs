//! Bind one existing character's private native art without replacing its gameplay.
use anyhow::{Context, Result, ensure};
use crystal_assets::{bind_minnesota_trainer_art, read_verified_compiled_game_pack};
use std::collections::BTreeMap;
use std::path::PathBuf;
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 4,
        "usage: pack_trainer_art BASE BINDING NATIVE_ART_ROOT NEW_PACK"
    );
    ensure!(!args[3].exists(), "preserve existing packs");
    let source = std::fs::read_to_string(&args[1])?;
    let value: serde_json::Value = serde_json::from_str(&source)?;
    let stem = value["trainer_class"]
        .as_str()
        .context("private class")?
        .to_ascii_lowercase();
    ensure!(
        stem.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'),
        "native resource stem"
    );
    let mut art = BTreeMap::new();
    for kind in ["sprites", "trainers"] {
        for extension in ["png", "2bpp", "gbcpal"] {
            let path = format!("gfx/{kind}/{stem}.{extension}");
            art.insert(path.clone(), std::fs::read(args[2].join(path))?);
        }
    }
    let base = read_verified_compiled_game_pack(&args[0])?;
    let pack = bind_minnesota_trainer_art(&base, &source, &art)?;
    ensure!(
        pack.compiled_audio() == base.compiled_audio(),
        "preserve all audio"
    );
    for (path, bytes) in base.runtime_files() {
        ensure!(
            pack.runtime_files().get(path) == Some(bytes),
            "preserve resource: {path}"
        );
    }
    pack.write_preserving_storage(&args[3])?;
    let loaded = read_verified_compiled_game_pack(&args[3])?;
    ensure!(loaded.identity()? == pack.identity()?, "written identity");
    println!("Published one existing character: {}", args[3].display());
    Ok(())
}
