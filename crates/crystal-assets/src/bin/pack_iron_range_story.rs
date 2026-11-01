use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    build_iron_range_story_modpack, read_verified_compiled_game_pack,
    refresh_boundary_basecamp_story_dialogue, refresh_grand_marais_story_dialogue,
};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base = PathBuf::from(
        args.next()
            .context("usage: pack_iron_range_story PACK DOSSIER STORY OUTPUT [--refresh-finale] [--grand-marais-dialogue TOWN]")?,
    );
    let dossier = std::fs::read_to_string(args.next().context("Iron Range dossier file")?)?;
    let story = std::fs::read_to_string(args.next().context("Minnesota story file")?)?;
    let output = PathBuf::from(args.next().context("output pack file")?);
    let mut refresh_finale = false;
    let mut town_dialogue = None;
    while let Some(flag) = args.next() {
        if flag == "--refresh-finale" && !refresh_finale {
            refresh_finale = true;
        } else if flag == "--grand-marais-dialogue" && town_dialogue.is_none() {
            town_dialogue = Some(std::fs::read_to_string(
                args.next().context("Grand Marais dialogue source")?,
            )?);
        } else {
            anyhow::bail!("expected unique --refresh-finale or --grand-marais-dialogue TOWN");
        }
    }
    ensure!(
        base.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from base pack"
    );
    let base = read_verified_compiled_game_pack(base)?;
    let pack = build_iron_range_story_modpack(&base, &dossier, &story)?;
    let pack = if refresh_finale {
        refresh_boundary_basecamp_story_dialogue(&pack)?
    } else {
        pack
    };
    let pack = if let Some(source) = town_dialogue {
        refresh_grand_marais_story_dialogue(&pack, &source)?
    } else {
        pack
    };
    pack.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("verify published Q06 story pack")?;
    println!("exported {}", output.display());
    Ok(())
}
