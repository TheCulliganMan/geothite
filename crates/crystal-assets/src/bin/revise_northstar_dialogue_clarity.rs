//! Write a fresh ignored dialogue review pack from the current Fair release.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (ely_preparation, parent, source, output) = match args.as_slice() {
        [parent, source, output] => (false, parent, source, output),
        [flag, parent, source, output] if flag == "--ely-preparation" => {
            (true, parent, source, output)
        }
        _ => anyhow::bail!(
            "usage: revise_northstar_dialogue_clarity [--ely-preparation] PARENT STORY_JSON FRESH_OUTPUT"
        ),
    };
    let output = Path::new(output);
    ensure!(
        !output.exists()
            && std::process::Command::new("git")
                .args(["check-ignore", "--quiet", "--no-index", "--"])
                .arg(output)
                .status()?
                .success(),
        "choose a fresh ignored output"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(parent)?;
    let source = std::fs::read_to_string(source)?;
    let revised = if ely_preparation {
        crystal_assets::revise_northstar_ely_preparation(&base, &source)?
    } else {
        crystal_assets::revise_northstar_dialogue_clarity(&base, &source)?
    };
    revised.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == revised,
        "pack readback differs"
    );
    println!("{} {}", output.display(), revised.identity()?.content_hash);
    Ok(())
}
