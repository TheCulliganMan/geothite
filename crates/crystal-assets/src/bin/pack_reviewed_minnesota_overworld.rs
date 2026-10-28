//! Publish the complete visually reviewed Northwoods roster to a separate pack.
use std::path::Path;

use anyhow::{Result, ensure};

#[path = "pack_reviewed_minnesota_overworld/reviewed_banks.rs"]
mod reviewed_banks;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 4,
        "usage: pack_reviewed_minnesota_overworld BASE_PACK POKEMON_ROOT OVERWORLD_ROOT OUTPUT_PACK"
    );
    if args[0] == "--verify-reviewed" {
        let base = crystal_assets::read_verified_compiled_game_pack(Path::new(&args[3]))?;
        let banks = reviewed_banks::load(
            Path::new(&args[1]),
            Path::new(&args[2]),
            &base,
            reviewed_banks::Coverage::Reviewed,
        )?;
        let overlay = banks.build_pack(&base)?;
        reviewed_banks::verify_preservation(&base, &overlay, &banks)?;
        println!(
            "Verified {} reviewed six-pose banks against the supplied pack; no pack written.",
            banks.runtime_files.len() / 4
        );
        return Ok(());
    }
    let source = Path::new(&args[0]).canonicalize()?;
    let destination = Path::new(&args[3]);
    ensure!(
        destination.canonicalize().ok().as_ref() != Some(&source),
        "use a separate output pack"
    );
    ensure!(!destination.exists(), "output pack must be fresh");
    let base = crystal_assets::read_verified_compiled_game_pack(&source)?;
    let banks = reviewed_banks::load(
        Path::new(&args[1]),
        Path::new(&args[2]),
        &base,
        reviewed_banks::Coverage::Complete,
    )?;
    let count = banks.runtime_files.len() / 4;
    let pack = banks.build_pack(&base)?;
    reviewed_banks::verify_preservation(&base, &pack, &banks)?;
    pack.write_preserving_storage(destination)?;
    let reopened = crystal_assets::read_verified_compiled_game_pack(destination)?;
    reviewed_banks::verify_preservation(&base, &reopened, &banks)?;
    println!(
        "Published {count} reviewed six-pose banks: {}",
        destination.display()
    );
    Ok(())
}
