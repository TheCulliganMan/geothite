use anyhow::{Result, ensure};
use crystal_assets::{
    build_northstar_fair_presentation, read_loaded_verified_compiled_game_pack,
    read_verified_compiled_game_pack,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [fair_raw, fair_hosted, art, story, raw, hosted] = args.as_slice() else {
        anyhow::bail!(
            "usage: pack_northstar_fair_presentation FAIR_RAW FAIR_HOSTED TRAINER_ART STORY RAW_OUTPUT HOSTED_OUTPUT"
        );
    };
    ensure!(raw != hosted, "separate output paths required");
    for path in [raw, hosted] {
        ensure!(
            !path.exists()
                && std::process::Command::new("git")
                    .args(["check-ignore", "--quiet", "--no-index", "--"])
                    .arg(path)
                    .status()?
                    .success(),
            "choose fresh ignored output"
        );
    }
    let editions = build_northstar_fair_presentation(
        &read_loaded_verified_compiled_game_pack(fair_raw)?,
        &read_loaded_verified_compiled_game_pack(fair_hosted)?,
        &read_verified_compiled_game_pack(art)?,
        &std::fs::read_to_string(story)?,
    )?;
    for (pack, path) in editions.into_iter().zip([raw, hosted]) {
        pack.write_preserving_storage(path)?;
        ensure!(
            read_verified_compiled_game_pack(path)? == pack,
            "pack changed on readback"
        );
        println!("{} {}", path.display(), pack.identity()?.content_hash);
    }
    Ok(())
}
