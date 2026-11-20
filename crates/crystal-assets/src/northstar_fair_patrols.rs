//! The six State Fair trainer circuits on the Fair Table editions.
#[path = "northstar_cabin_cut.rs"]
mod cabin_cut;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_publication;
use crate::state_fair_patrols::{self, Patrol};

const PATROLS: &[u8] =
    include_bytes!("../../../planning/minnesota/revisions/fair-patrols-v01/patrols.json");
const PARENTS: [&str; 2] = [
    "301905c7887d716871827436db7204e61c4b6db96db7f197ad4f895b18ea7772",
    "c7b7cde4020895fd8062c50b67ab9be3e984be5b34cb980e77c1ca4eeeb62abf",
];

/// Builds the Minnesota edition with active Fair patrols and updated local services.
///
/// # Errors
/// Returns an error for an unexpected Fair Table input, blocked patrol, missing
/// trainer or inconsistent resulting archive.
pub fn revise_fair_patrols(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        PARENTS.contains(&pack.identity.content_hash.as_str())
            && pack.report.manifests.last().map(String::as_str) == Some("northstar-fair-table-v1"),
        "fair patrols require a matching Fair Table edition"
    );
    let mut room = pack
        .data
        .maps
        .get("MnStateFairgrounds")
        .context("fairgrounds")?
        .clone();
    let routes: [Patrol; 6] = serde_json::from_slice(PATROLS)?;
    state_fair_patrols::install(&pack, &mut room, &routes)?;
    for route in routes {
        let trainer = room
            .objects
            .iter_mut()
            .find(|actor| actor.script == route.id)
            .context("fair trainer")?;
        let row = room
            .map_event_section_commands
            .iter_mut()
            .find(|row| row.command == "object_event" && row.args.get(11) == Some(&route.id))
            .context("fair trainer object row")?;
        ensure!(
            row.args.len() == 13
                && row.args[3].starts_with("SPRITEMOVEDATA_STANDING_")
                && row.args[4..6] == ["0", "0"]
                && row.args[9] == "OBJECTTYPE_TRAINER"
                && row.args[10] == trainer.radius.to_string(),
            "unexpected event row for {}",
            route.id
        );
        trainer.radius = 8;
        row.args[3].clone_from(&trainer.spritemovedata);
        row.args[4] = trainer.move_range_x.to_string();
        row.args[5] = trainer.move_range_y.to_string();
        row.args[10] = trainer.radius.to_string();
    }
    authored_town_publication::replace(&mut pack, room)?;
    cabin_cut::open_shore_path(&mut pack)?;
    crate::minnesota_research_access::refresh_adopted_starter_benches(&mut pack)?;
    crate::minnesota_dex_roster_sync::install_regional_numbers(&mut pack)?;
    crate::fair_baker_picnic::give_ada_pie(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-fair-patrols-v1".into());
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
