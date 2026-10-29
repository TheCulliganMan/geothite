//! Preserve the Grand Marais partner offer when native delivery fails.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_rival_arc::{map, publish, put};

const MANIFEST: &str = "grand-marais-partner-retry-v1";
const ORIGINAL: &str = "MnRewardMachinePartOriginal";
const FULL: &str = "MnMaraisPartnerFull";

/// Guards the existing Birchit welcome against failed native delivery.
/// Successful delivery retains the original supplies and Machine Part. Failure
/// leaves the welcome flag and supplies untouched, allowing another attempt.
/// A full party still uses the native selected-PC-box fallback.
///
/// # Errors
/// Rejects duplicate installation, incompatible scripts, missing Minnesota
/// species, occupied labels, invalid geography or invalid compiled output.
pub fn build_grand_marais_partner_retry(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "partner retry already installed"
    );
    ensure!(
        base.data.pokemon.contains_key("BIRCHIT")
            && base
                .data
                .story_event_script_constants
                .global
                .contains_key("MN_DEX_MEMBER_BIRCHIT"),
        "active Minnesota partner required"
    );
    let mut pack = base.clone();
    let mut room = map(&pack, "MnGrandMaraisFieldStation")?;
    ensure!(
        !room.scripts.contains_key(FULL) && !room.scripts.contains_key("MnMaraisPartnerFullText"),
        "occupied partner retry label"
    );
    let original = room
        .scripts
        .get(ORIGINAL)
        .and_then(|program| program.as_array())
        .context("installed recovery wrapper")?;
    ensure!(
        original.len() >= 3
            && original[0] == c("checkevent", &["EVENT_MN_GRAND_MARAIS_STARTER"])
            && original[1] == c("iftrue", &["MnGrandMaraisWelcomeAfter"]),
        "existing once-only partner guard required"
    );
    let mut patched = original[..2].to_vec();
    let mut gifts = 0;
    for command in &original[2..] {
        patched.push(command.clone());
        if command["command"] == "givepoke" {
            ensure!(
                command == &c("givepoke", &["BIRCHIT", "8", "BERRY"]),
                "original native partner required"
            );
            patched.push(c("ifequal", &["2", FULL]));
            gifts += 1;
        }
    }
    ensure!(gifts == 1, "exactly one existing partner delivery required");
    put(&mut room, ORIGINAL, patched);
    put(
        &mut room,
        FULL,
        vec![c("jumptextfaceplayer", &["MnMaraisPartnerFullText"])],
    );
    authoring::text(
        &mut room,
        "MnMaraisPartnerFullText",
        &[
            "Your party and PC",
            "box are full.",
            "Make room, return.",
            "The part can wait.",
        ],
    );
    publish(&mut pack, room)?;
    pack.report.manifests.push(MANIFEST.into());
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
