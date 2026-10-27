use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    build_minnesota_gym_review_pack, build_minnesota_gym_review_room,
    read_verified_compiled_game_pack,
};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base = PathBuf::from(
        args.next()
            .context("usage: pack_minnesota_gyms BASE DESIGN OUTPUT [GYM_SLOT]")?,
    );
    let design = std::fs::read_to_string(args.next().context("gym design JSON")?)?;
    let output = PathBuf::from(args.next().context("output pack")?);
    let slot = args
        .next()
        .map(|arg| {
            arg.to_str()
                .context("gym slot must be UTF-8")?
                .parse::<u8>()
                .context("gym slot must be 1..=8")
        })
        .transpose()?;
    ensure!(args.next().is_none(), "unexpected argument");
    ensure!(
        base.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from input"
    );
    let pack = read_verified_compiled_game_pack(base)?;
    let result = match slot {
        Some(slot) => build_minnesota_gym_review_room(&pack, &design, slot)?,
        None => build_minnesota_gym_review_pack(&pack, &design)?,
    };
    result.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("read back gym pack")?;
    println!(
        "exported {} gym interior(s) and review entrance to {}",
        if slot.is_some() { 1 } else { 8 },
        output.display()
    );
    Ok(())
}
