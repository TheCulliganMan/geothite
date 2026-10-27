use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    build_boundary_basecamp_modpack, connect_boundary_basecamp, read_verified_compiled_game_pack,
};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base = PathBuf::from(
        args.next()
            .context("usage: pack_boundary_basecamp BASE TOWN LAYOUT OUTPUT [--connect-routes]")?,
    );
    let town = std::fs::read_to_string(args.next().context("town source file")?)?;
    let layout = std::fs::read_to_string(args.next().context("town layout file")?)?;
    let output = PathBuf::from(args.next().context("output pack file")?);
    let connect_routes = match args.next().as_deref() {
        None => false,
        Some(flag) if flag == "--connect-routes" => true,
        Some(_) => anyhow::bail!("expected optional --connect-routes"),
    };
    ensure!(args.next().is_none(), "unexpected additional argument");
    ensure!(
        base.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from base pack"
    );
    let pack = read_verified_compiled_game_pack(base)?;
    let result = build_boundary_basecamp_modpack(&pack, &town, &layout)?;
    let result = if connect_routes {
        connect_boundary_basecamp(&result)?
    } else {
        result
    };
    result.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("verify written basecamp pack")?;
    println!("exported {}", output.display());
    Ok(())
}
