//! Build and strictly reread an outdoor terrain review using the native pack writer.
use std::path::{Path, PathBuf};

use anyhow::{Result, ensure};
use crystal_assets::{
    CompiledGamePack, read_verified_compiled_game_pack, restore_brown_shores, revise_cliff_faces,
    revise_outdoor_contours, revise_rock_cut, revise_rock_cut_north_face, revise_rocky_shores,
};

type Builder = fn(&CompiledGamePack, &Path) -> Result<CompiledGamePack>;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let (builder, args): (Builder, &[PathBuf]) = match args.as_slice() {
        [flag, inputs @ ..] if flag.as_os_str() == "--rock-cut-north-face" => {
            (revise_rock_cut_north_face, inputs)
        }
        [flag, inputs @ ..] if flag.as_os_str() == "--rock-cut" => (revise_rock_cut, inputs),
        [flag, inputs @ ..] if flag.as_os_str() == "--brown-shores" => {
            (restore_brown_shores, inputs)
        }
        [flag, inputs @ ..] if flag.as_os_str() == "--rocky-shores" => {
            (revise_rocky_shores, inputs)
        }
        [flag, inputs @ ..] if flag.as_os_str() == "--cliffs" => (revise_cliff_faces, inputs),
        inputs => (revise_outdoor_contours, inputs),
    };
    ensure!(
        args.len() == 3,
        "usage: pack_outdoor_contours [--cliffs|--rocky-shores|--brown-shores|--rock-cut|--rock-cut-north-face] PARENT SOURCE_DIR FRESH_PACK"
    );
    ensure!(!args[2].try_exists()?, "choose a fresh output pack");
    let base = read_verified_compiled_game_pack(&args[0])?;
    let pack = builder(&base, args[1].as_path())?;
    drop(base);
    pack.write_preserving_storage(&args[2])?;
    let written = read_verified_compiled_game_pack(&args[2])?;
    ensure!(written == pack, "outdoor review failed readback");
    println!("{} {}", args[2].display(), written.identity()?.content_hash);
    Ok(())
}
