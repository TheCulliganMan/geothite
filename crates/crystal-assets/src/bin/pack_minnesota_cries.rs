//! Publish original voices to a separate local pack; no game source extraction.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let usage = "usage: pack_minnesota_cries BASE_PACK POKEMON_ROOT OUTPUT_PACK";
    let base = PathBuf::from(args.next().context(usage)?).canonicalize()?;
    let root = PathBuf::from(args.next().context(usage)?).canonicalize()?;
    let output = PathBuf::from(args.next().context(usage)?);
    ensure!(args.next().is_none(), "{usage}");
    ensure!(
        output.canonicalize().ok().as_ref() != Some(&base),
        "use a separate output pack"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&base)?;
    let pack = crystal_assets::build_minnesota_cries(&base, &root)?;
    pack.write_preserving_storage(&output)?;
    let pack = crystal_assets::read_verified_compiled_game_pack(&output)?;
    let count = pack
        .data()
        .audio
        .iter()
        .filter(|asset| asset.id.starts_with("CRY_MN_"))
        .count();
    println!(
        "published {count} original Minnesota cries: {}",
        output.display()
    );
    Ok(())
}
