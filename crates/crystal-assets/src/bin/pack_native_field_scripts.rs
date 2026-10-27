use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_native_field_scripts_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let args = std::env::args()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "usage: pack_native_field_scripts BASE SOURCE OUTPUT"
    );
    ensure!(
        args[0].canonicalize()? != args[2].canonicalize().unwrap_or_else(|_| args[2].clone()),
        "output must differ from the base pack"
    );
    let base = read_verified_compiled_game_pack(&args[0])?;
    let source = read_verified_compiled_game_pack(&args[1])?;
    build_native_field_scripts_modpack(&base, &source)?.write_preserving_storage(&args[2])?;
    println!(
        "Published {}: restored native field visual definitions and retained Waterfall loop; maps, audio and regional progression are preserved.",
        args[2].display()
    );
    Ok(())
}
