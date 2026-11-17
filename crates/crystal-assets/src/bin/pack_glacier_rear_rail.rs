//! Publish the five rear rail approaches as a separate Minnesota pack.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ensure!(
        args.len() == 2,
        "usage: pack_glacier_rear_rail BASE_PACK OUTPUT_PACK"
    );
    let output = Path::new(&args[1]);
    ensure!(!output.exists(), "output pack must be fresh");
    let base = crystal_assets::read_verified_compiled_game_pack(Path::new(&args[0]))?;
    let candidate = crystal_assets::revise_glacier_rear_rail(&base)?;
    candidate.write_preserving_storage(output)?;
    let reopened = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(
        reopened == candidate,
        "rear rail pack changed during serialization"
    );
    println!("Published {}", output.display());
    Ok(())
}
