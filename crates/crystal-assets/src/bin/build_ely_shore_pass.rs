//! Publish the reviewed Ely back route to a fresh external content pack.
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
fn main() -> Result<()> {
    let p = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        p.len() == 4,
        "usage: build_ely_shore_pass BASE CONTRACT CONTRACT_SHA256 NEW_OUTPUT"
    );
    ensure!(!p[3].exists(), "output must be fresh");
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&p[0])?))
            == "032304d0fd1d51863a2c3bf2562b20728e22c7535d1c194555621bc8e85b4899",
        "exact compact parent required"
    );
    let source = std::fs::read_to_string(&p[1])?;
    let pack = crystal_assets::build_ely_shore_pass_pack(
        &crystal_assets::read_verified_compiled_game_pack(&p[0])?,
        &source,
        p[2].to_str().context("ASCII contract hash")?,
    )?;
    pack.write_preserving_storage(&p[3])?;
    let written = crystal_assets::read_verified_compiled_game_pack(&p[3])
        .context("strict Ely pass readback")?;
    ensure!(written == pack, "written pack differs");
    println!(
        "Published Ely snow, coastal back route and Split Rock clinic: {}",
        p[3].display()
    );
    Ok(())
}
