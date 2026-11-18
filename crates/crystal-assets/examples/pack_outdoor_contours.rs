//! Build and strictly reread an outdoor terrain review using the native pack writer.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_outdoor_contours};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 3,
        "usage: pack_outdoor_contours PARENT SOURCE_DIR FRESH_PACK"
    );
    ensure!(!args[2].try_exists()?, "choose a fresh output pack");
    let pack = revise_outdoor_contours(&read_verified_compiled_game_pack(&args[0])?, &args[1])?;
    pack.write_preserving_storage(&args[2])?;
    let written = read_verified_compiled_game_pack(&args[2])?;
    ensure!(written == pack, "outdoor review failed readback");
    println!("{} {}", args[2].display(), written.identity()?.content_hash);
    Ok(())
}
