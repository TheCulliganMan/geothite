use anyhow::{Context, Result, ensure};
use crystal_assets::{build_competitive_battle_modpack, read_verified_compiled_game_pack};
use std::path::PathBuf;

struct PendingPack(PathBuf);
impl Drop for PendingPack {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let source = PathBuf::from(
        args.next()
            .context("usage: pack_competitive_battles <base-pack> <output-pack>")?,
    )
    .canonicalize()?;
    let output = PathBuf::from(args.next().context("output pack is required")?);
    ensure!(args.next().is_none(), "unexpected extra argument");
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    ensure!(
        output.canonicalize().ok().as_ref() != Some(&source),
        "output must differ from base pack"
    );
    let base = read_verified_compiled_game_pack(source)?;
    let pack = build_competitive_battle_modpack(&base)?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let temporary_path = output.with_file_name(format!(
        "competitive-{}-{nonce}.pending.crystalpack",
        std::process::id()
    ));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)?;
    let temporary = PendingPack(temporary_path);
    pack.write_preserving_storage(&temporary.0)?;
    let loaded = read_verified_compiled_game_pack(&temporary.0)?;
    ensure!(
        loaded.identity()? == pack.identity()?,
        "written pack identity changed"
    );
    std::fs::rename(&temporary.0, &output)?;
    println!("exported {}", output.display());
    Ok(())
}
