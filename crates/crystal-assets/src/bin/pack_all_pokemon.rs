use anyhow::{Context, Result, ensure};
use crystal_assets::{build_all_pokemon_modpack, read_verified_compiled_game_pack};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_all_pokemon <base.crystalpack> <output.crystalpack>")?,
    )
    .canonicalize()?;
    let output = PathBuf::from(args.next().context("output .crystalpack path required")?);
    ensure!(args.next().is_none(), "unexpected argument");
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let output = parent
        .canonicalize()?
        .join(output.file_name().context("output filename required")?);
    ensure!(
        output != input && (!output.exists() || output.canonicalize()? != input),
        "output must differ from source"
    );
    let base = read_verified_compiled_game_pack(input)?;
    let pack = build_all_pokemon_modpack(&base)?;
    pack.write_preserving_storage(&output)?;
    let reloaded = read_verified_compiled_game_pack(&output)?;
    ensure!(
        reloaded.identity()? == pack.identity()?,
        "written pack identity mismatch"
    );
    println!(
        "exported {}: {}",
        output.display(),
        pack.runtime_modpack_id()?
    );
    Ok(())
}
