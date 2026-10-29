//! Publish ordinary trainer art assignments on existing routes and fairgrounds.
use anyhow::{Result, ensure};
use crystal_assets::{distribute_minnesota_trainer_art, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ensure!(
        args.len() == 3,
        "usage: pack_trainer_distribution BASE ASSIGNMENTS NEW_PACK"
    );
    ensure!(
        !std::path::Path::new(&args[2]).exists(),
        "preserve previous packs"
    );
    let base = read_verified_compiled_game_pack(&args[0])?;
    let pack = distribute_minnesota_trainer_art(&base, &std::fs::read_to_string(&args[1])?)?;
    ensure!(
        base.runtime_files() == pack.runtime_files()
            && base.compiled_audio() == pack.compiled_audio(),
        "preserve all art/audio"
    );
    pack.write_preserving_storage(&args[2])?;
    let restored = read_verified_compiled_game_pack(&args[2])?;
    ensure!(
        restored.identity()? == pack.identity()?,
        "readback identity"
    );
    println!(
        "Published regular trainer distribution: {}",
        std::path::Path::new(&args[2]).display()
    );
    Ok(())
}
