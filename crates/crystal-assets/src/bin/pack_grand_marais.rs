use anyhow::{Context, Result, ensure};
use crystal_assets::{
    build_grand_marais_modpack, build_grand_marais_region_extension,
    read_verified_compiled_game_pack,
};
use std::path::PathBuf;
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let base = PathBuf::from(
        args.next()
            .context("usage: pack_grand_marais BASE LAYOUT OUTPUT")?,
    );
    let layout = std::fs::read_to_string(args.next().context("town layout file")?)?;
    let output = PathBuf::from(
        args.next()
            .context("usage: pack_grand_marais BASE LAYOUT OUTPUT")?,
    );
    let preserve_home = match args.next().as_deref() {
        None => false,
        Some("--preserve-home") => true,
        Some(_) => anyhow::bail!("usage: pack_grand_marais BASE LAYOUT OUTPUT [--preserve-home]"),
    };
    ensure!(args.next().is_none(), "unexpected additional argument");
    ensure!(
        base.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from base pack"
    );
    let pack = read_verified_compiled_game_pack(base)?;
    let town = if preserve_home {
        build_grand_marais_region_extension(&pack, &layout)?
    } else {
        build_grand_marais_modpack(&pack, &layout)?
    };
    town.write_preserving_storage(&output)?;
    println!("exported {}", output.display());
    Ok(())
}
