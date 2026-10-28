//! Publish the bounded Grand Marais west-coast/north-inland stage.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_grand_marais_superior_pack, read_verified_compiled_game_pack};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 4,
        "usage: build_grand_marais_superior BASE EXPECTED_INPUT_FILE_SHA CONTRACT FRESH_OUTPUT"
    );
    let input = PathBuf::from(&args[0]);
    let expected = args[1].to_str().context("explicit input SHA UTF-8")?;
    let output = PathBuf::from(&args[3]);
    ensure!(
        expected.len() == 64 && expected.bytes().all(|c| c.is_ascii_hexdigit()),
        "explicit 64-character input file SHA required"
    );
    ensure!(!output.exists(), "output must be fresh");
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "caller-bound parent file SHA differs"
    );
    let source =
        std::fs::read_to_string(PathBuf::from(&args[2])).context("frozen harbor contract")?;
    let base = read_verified_compiled_game_pack(input)?;
    let result = build_grand_marais_superior_pack(&base, &source)?;
    result.write_preserving_storage(&output)?;
    let written =
        read_verified_compiled_game_pack(&output).context("strict written harbor readback")?;
    ensure!(
        written == result,
        "written harbor differs from validated stage"
    );
    println!(
        "Published Grand Marais stage: {} ({})",
        output.display(),
        written.identity()?.content_hash
    );
    Ok(())
}
