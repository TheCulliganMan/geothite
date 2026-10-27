//! Apply either stage of the opt-in Northwoods Fairy rules to a local pack.
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        (3..=4).contains(&args.len()),
        "usage: pack_fairy_type types|moves BASE_PACK OUTPUT_PACK [MOVES_JSON]"
    );
    let source = PathBuf::from(&args[1]).canonicalize()?;
    let output = PathBuf::from(&args[2]);
    let parent = output.parent().context("output parent")?;
    fs::create_dir_all(parent)?;
    let resolved = parent
        .canonicalize()?
        .join(output.file_name().context("output filename")?);
    ensure!(resolved != source, "output must differ from source");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&resolved)
            .status()?
            .success(),
        "output must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&resolved)
            .output()?
            .status
            .success(),
        "output must not be tracked"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&source)?;
    let result = match args[0].as_str() {
        "types" => {
            ensure!(args.len() == 3, "type stage does not take a move catalog");
            crystal_assets::build_fairy_type_modpack(&base)?
        }
        "moves" => {
            if let Some(path) = args.get(3) {
                let json = fs::read_to_string(path)
                    .with_context(|| format!("read move catalog {path}"))?;
                crystal_assets::build_fairy_moves_modpack_from_json(&base, &json)?
            } else {
                crystal_assets::build_fairy_moves_modpack(&base)?
            }
        }
        _ => anyhow::bail!("stage must be types or moves"),
    };
    result.write_preserving_storage(&resolved)?;
    let verified = crystal_assets::read_verified_compiled_game_pack(&resolved)?;
    ensure!(
        verified.identity()? == result.identity()?,
        "written Fairy pack differs"
    );
    println!(
        "Verified {}: {} species, {} moves",
        resolved.display(),
        verified.data().pokemon.len(),
        verified.data().moves.len()
    );
    Ok(())
}
