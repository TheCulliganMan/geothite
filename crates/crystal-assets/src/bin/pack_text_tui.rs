use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use crystal_assets::{build_text_tui_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let repository_root = PathBuf::from(
        args.next()
            .context("usage: pack_text_tui <repository-root> [base-pack] [output-pack]")?,
    )
    .canonicalize()
    .context("resolve repository root")?;
    let base_pack = args.next().map(PathBuf::from).unwrap_or_else(|| {
        repository_root.join("content-packs/realtime-clock.browser.crystalpack")
    });
    let output_pack = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| repository_root.join("content-packs/text-tui.crystalpack"));
    if args.next().is_some() {
        anyhow::bail!("usage: pack_text_tui <repository-root> [base-pack] [output-pack]");
    }
    pack_text_tui(&base_pack, &output_pack)
}

fn pack_text_tui(base_pack: &Path, output_pack: &Path) -> Result<()> {
    let base_pack = base_pack.canonicalize().context("resolve base pack path")?;
    let base = read_verified_compiled_game_pack(&base_pack)
        .with_context(|| format!("load verified base pack {}", base_pack.display()))?;
    let text_tui = build_text_tui_modpack(&base)?;
    let filename = output_pack
        .file_name()
        .context("output pack requires a filename")?;
    let parent = output_pack
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).context("create output pack directory")?;
    let output_pack = parent
        .canonicalize()
        .context("resolve output directory")?
        .join(filename);
    text_tui
        .write_preserving_storage(&output_pack)
        .with_context(|| format!("write text TUI pack {}", output_pack.display()))?;
    println!("exported {}", output_pack.display());
    Ok(())
}
