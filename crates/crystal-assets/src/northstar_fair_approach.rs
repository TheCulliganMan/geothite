//! Give the St. Paul fair annex and the public fairgrounds distinct directions.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_dialogue;
use crate::authored_town_publication;

/// Updates the existing speakers and signs along the public approach.
///
/// Returns an error if an expected room, installed text or authored page is missing.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let directions: &[(&str, &[&str])] = &[
        (
            "MnStPaul",
            &[
                "MN_ST_PAUL_FAIR_GUARD_EASTText",
                "MN_ST_PAUL_FAIR_GUARD_WESTText",
                "MN_ST_PAUL_FAIR_STEWARDText",
                "MN_ST_PAUL_SIGN_FAIRText",
                "MN_ST_PAUL_CIVIC_GUIDEText",
            ],
        ),
        ("MnStPaulCapitolHall", &["MN_ST_PAUL_RECORDS_CLERKText"]),
        ("MnStPaulFairInfoHouse", &["MN_SPX_FAIRINFOHOUSE_HOSTText"]),
        ("MnR05C00R02", &["MN_R05_SOUTH_SIGNText"]),
    ];
    for &(name, labels) in directions {
        let mut room = pack
            .data
            .maps
            .get(name)
            .with_context(|| format!("fair directions in {name}"))?
            .clone();
        for &label in labels {
            ensure!(
                room.script_text_bodies.contains_key(label),
                "missing existing fair directions at {name}/{label}"
            );
            authored_dialogue::install_canonical(&mut room, label)?;
        }
        room.script_text_bodies = crate::parse_script_text_bodies(name, &room.scripts)?;
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}
