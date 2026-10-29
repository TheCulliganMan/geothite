use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, repair_minnesota_script_commands};

fn main() -> Result<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 2,
        "usage: repair_minnesota_scripts BASE OUTPUT"
    );
    ensure!(
        args[0].canonicalize()? != args[1].canonicalize().unwrap_or_else(|_| args[1].clone()),
        "output must differ from the base pack"
    );
    let base = read_verified_compiled_game_pack(&args[0])?;
    let repaired = repair_minnesota_script_commands(&base)?;
    repaired.write_preserving_storage(&args[1])?;
    println!("Wrote repaired Minnesota scripts to {}", args[1].display());
    Ok(())
}
