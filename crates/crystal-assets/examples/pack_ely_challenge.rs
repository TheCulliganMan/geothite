//! Build a Minnesota campaign revision and verify the written external archive.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{
    CompiledGamePack, read_verified_compiled_game_pack, revise_ely_challenge, revise_ely_return,
    revise_finder_lesson, revise_marais_supplies, revise_research_choices, revise_trail_signs,
};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    const USAGE: &str = "usage: pack_ely_challenge [--finder-lesson|--ely-return|--marais-supplies|--research-choices|--trail-signs] PARENT FRESH_OUTPUT";
    let [options @ .., parent, output] = args.as_slice() else {
        anyhow::bail!(USAGE);
    };
    let revise: fn(CompiledGamePack) -> Result<CompiledGamePack> = match options {
        [] => revise_ely_challenge,
        [mode] if mode.as_os_str() == "--finder-lesson" => revise_finder_lesson,
        [mode] if mode.as_os_str() == "--ely-return" => revise_ely_return,
        [mode] if mode.as_os_str() == "--marais-supplies" => revise_marais_supplies,
        [mode] if mode.as_os_str() == "--research-choices" => revise_research_choices,
        [mode] if mode.as_os_str() == "--trail-signs" => revise_trail_signs,
        _ => anyhow::bail!(USAGE),
    };
    ensure!(!output.try_exists()?, "choose a fresh output pack");
    let pack = revise(read_verified_compiled_game_pack(parent)?)?;
    pack.write_preserving_storage(output)?;
    let written = read_verified_compiled_game_pack(output)?;
    ensure!(written == pack, "content changed during readback");
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
