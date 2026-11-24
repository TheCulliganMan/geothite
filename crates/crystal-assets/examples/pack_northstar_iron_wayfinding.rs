use std::path::PathBuf;
use std::process::Command;

use anyhow::{Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_northstar_iron_wayfinding};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [source, output] = args.as_slice() else {
        anyhow::bail!("usage: pack_northstar_iron_wayfinding FAIR_PRESENTATION PREVIEW_OUTPUT");
    };
    ensure!(
        !output.exists()
            && Command::new("git")
                .args(["check-ignore", "--quiet", "--no-index", "--"])
                .arg(output)
                .status()?
                .success(),
        "choose fresh ignored output"
    );
    let preview = revise_northstar_iron_wayfinding(&read_verified_compiled_game_pack(source)?)?;
    preview.write_preserving_storage(output)?;
    ensure!(
        read_verified_compiled_game_pack(output)? == preview,
        "mine preview changed on readback"
    );
    println!("{} {}", output.display(), preview.identity()?.content_hash);
    Ok(())
}
