//! Assemble the reviewed Northstar playtest repairs into an external pack.
use std::path::Path;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "usage: assemble_northstar_playtest_repairs STORY_PACK PASSAGE_SOURCE FRESH_OUTPUT"
    );
    let output = Path::new(&args[2]);
    ensure!(!output.exists(), "choose a fresh output");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated pack must remain ignored"
    );
    let mut pack = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    pack = crystal_assets::repair_northstar_mom_door(&pack).context("Mom's door scene")?;
    pack = crystal_assets::repair_rochester_gym_progress(&pack).context("Rochester gym")?;
    pack = crystal_assets::correct_minnesota_mart_arrivals(&pack).context("Mart arrivals")?;
    pack = crystal_assets::revise_minnesota_fossil_lab_dialogue(&pack)
        .context("fossil lab dialogue")?;
    pack = crystal_assets::repair_rochester_dialogue(&pack).context("Rochester dialogue")?;
    pack =
        crystal_assets::revise_minnesota_ordinary_challenges(&pack).context("ordinary trainers")?;
    pack =
        crystal_assets::add_northstar_driftless_passage(&pack, &std::fs::read_to_string(&args[1])?)
            .context("Driftless passage")?;
    pack = crystal_assets::restore_minnesota_center_nurses(&pack).context("center restoration")?;
    pack.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == pack,
        "pack readback differs"
    );
    println!("{} {}", output.display(), pack.identity()?.content_hash);
    Ok(())
}
