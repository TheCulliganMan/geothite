//! Build and strictly reread an outdoor terrain review using the native pack writer.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{
    read_verified_compiled_game_pack, revise_cliff_faces, revise_outdoor_contours,
};

fn main() -> Result<()> {
    let mut args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let cliffs = args
        .first()
        .is_some_and(|arg| arg.as_os_str() == "--cliffs");
    if cliffs {
        args.remove(0);
    }
    ensure!(
        args.len() == 3,
        "usage: pack_outdoor_contours [--cliffs] PARENT SOURCE_DIR FRESH_PACK"
    );
    ensure!(!args[2].try_exists()?, "choose a fresh output pack");
    let base = read_verified_compiled_game_pack(&args[0])?;
    let pack = if cliffs {
        revise_cliff_faces(&base, &args[1])?
    } else {
        revise_outdoor_contours(&base, &args[1])?
    };
    drop(base);
    pack.write_preserving_storage(&args[2])?;
    let written = read_verified_compiled_game_pack(&args[2])?;
    ensure!(written == pack, "outdoor review failed readback");
    println!("{} {}", args[2].display(), written.identity()?.content_hash);
    Ok(())
}
