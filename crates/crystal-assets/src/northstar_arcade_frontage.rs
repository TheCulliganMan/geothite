//! Make the Cedar Inlet fork and Minneapolis arcade readable on approach.

#[path = "northstar_arcade_front.rs"]
mod arcade;
#[path = "northstar_cedar_sign.rs"]
mod cedar;
#[path = "northstar_arrival_desk.rs"]
mod desk;
#[path = "northstar_mill_inn.rs"]
mod inn;
#[path = "northstar_arcade_prize.rs"]
mod prize;

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, authored_dialogue};

const ROOM: &str = "MnMinneapolisCornerHome";
const PARENTS: [&str; 2] = [
    "2737fc05be7dee021808e6fd2e93bd44baa7b61645d058cd8a87468614a866af",
    "b6708b138ecdda7848e9d4865b0acb2c04f807ee0a5a65d8983baf7b0ff78313",
];

/// Installs a freestanding trail sign, a reel marquee and directions to the arcade counters.
///
/// # Errors
/// Returns an error for an unexpected parent, tile layout, reading point or
/// dialogue program, or if the resulting pack fails runtime verification.
pub fn revise_arcade_frontage(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        PARENTS.contains(&pack.identity.content_hash.as_str())
            && pack.report.manifests.last().map(String::as_str) == Some("northstar-wind-lesson-v1"),
        "arcade frontage requires a verified Wind Lesson edition"
    );
    cedar::apply(&mut pack)?;
    arcade::install(&mut pack)?;
    prize::install(&mut pack)?;
    desk::install(&mut pack)?;
    inn::install(&mut pack)?;
    let room = pack
        .data
        .maps
        .get_mut(ROOM)
        .context("Minneapolis Game Corner")?;
    let mut texts = BTreeMap::new();
    for label in [
        "MnCornerWelcomeText",
        "MnCornerRulesText",
        "MnCornerNeedCase",
        "MnCornerCoinsBoughtText",
        "MnCornerItemsText",
        "MnCornerItemsConfirm0",
        "MnCornerItemsConfirm1",
        "MnCornerItemsConfirm2",
        "MnCornerTmsConfirm0",
        "MnCornerTmsConfirm1",
        "MnCornerTmsConfirm2",
        "MnMinneapolisCornerHomePatronText",
    ] {
        let previous = room
            .scripts
            .get(label)
            .with_context(|| format!("missing {label}"))?;
        ensure!(
            Some(previous) == pack.data.map_scripts.get(label),
            "{label} differs between indexes"
        );
        authored_dialogue::install_canonical(room, label)?;
        texts.insert(label.to_owned(), room.scripts[label].clone());
    }
    room.script_text_bodies
        .extend(crate::parse_script_text_bodies(ROOM, &texts)?);
    pack.data.map_scripts.extend(texts);
    pack.report
        .manifests
        .push("northstar-arcade-frontage-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
