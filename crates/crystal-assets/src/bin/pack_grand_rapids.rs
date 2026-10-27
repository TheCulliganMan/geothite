use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    build_grand_rapids_modpack, connect_grand_rapids, read_verified_compiled_game_pack,
};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base = PathBuf::from(
        args.next()
            .context("usage: pack_grand_rapids PACK TOWN LAYOUT STORY OUTPUT [--connect]")?,
    );
    let town = std::fs::read_to_string(args.next().context("town source")?)?;
    let layout = std::fs::read_to_string(args.next().context("layout source")?)?;
    let story = std::fs::read_to_string(args.next().context("story source")?)?;
    let output = PathBuf::from(args.next().context("output pack")?);
    let connect = match args.next() {
        None => false,
        Some(flag) if flag == "--connect" => true,
        Some(_) => anyhow::bail!("expected --connect"),
    };
    ensure!(args.next().is_none(), "unexpected publisher argument");
    ensure!(
        base.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from source pack"
    );
    let pack = build_grand_rapids_modpack(
        &read_verified_compiled_game_pack(base)?,
        &town,
        &layout,
        &story,
    )?;
    let pack = if connect {
        connect_grand_rapids(&pack, &town)?
    } else {
        pack
    };
    pack.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("read back Grand Rapids publication")?;
    println!("exported {}", output.display());
    Ok(())
}
