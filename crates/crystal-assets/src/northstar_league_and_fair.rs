//! Restore the Minnesota League cast and distinguish Flint's final team member.

use anyhow::{Context, Result, ensure};
use crystal_core::models::LearnedMove;

use crate::CompiledGamePack;
use crate::northstar_regional_roster::RegionalRoster;

const ELITE_ART: [(&str, &str, &str); 4] = [
    (
        include_str!("../../../planning/minnesota/npcs/cast/10-fern/native-pixels-v2.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/10-fern/native-pixels-v3.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/10-fern.json"
        ),
    ),
    (
        include_str!("../../../planning/minnesota/npcs/cast/11-flint/native-pixels-v3.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/11-flint/native-pixels-v2.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/11-flint.json"
        ),
    ),
    (
        include_str!("../../../planning/minnesota/npcs/cast/12-veil/native-pixels-v3.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/12-veil/native-pixels-v3.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/12-veil.json"
        ),
    ),
    (
        include_str!("../../../planning/minnesota/npcs/cast/13-halden/native-pixels-v2.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/13-halden/native-pixels-v4.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/13-halden.json"
        ),
    ),
];

/// Builds the League and State Fair revision of either Minnesota edition.
///
/// # Errors
/// Rejects an unexpected parent, actor binding, team, learnset or invalid pack.
pub fn revise_league_and_fair(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let manifests = &base.report.manifests;
    let parent = manifests.last().map(String::as_str);
    let cafe = parent == Some("mothlight-battle-cafe-v1")
        && manifests.iter().rev().nth(1).map(String::as_str)
            == Some("northstar-northern-trails-v1")
        && base.identity()?.content_hash
            == "701440cf37077155a8c690d54fddc48d91aadb3bf6f7ffcee52525dc034f04a8";
    ensure!(
        parent == Some("northstar-northern-trails-v1") || cafe,
        "expected a northern trails edition or its reviewed Mothlight Battle Café child"
    );
    let mut pack = base.clone();
    for (portrait, field, binding) in ELITE_ART {
        let art = crate::minnesota_native_npc::compile(portrait, Some(field))?;
        pack = crate::bind_minnesota_trainer_art(&pack, binding, &art)?;
    }
    revise_flints_ace(&mut pack)?;
    crate::northstar_champion_invitation::revise_champion_invitation(&mut pack)?;
    crate::northstar_fair_approach::apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-league-and-fair-v1".into());
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

fn revise_flints_ace(pack: &mut CompiledGamePack) -> Result<()> {
    let species = "FORTIGARD";
    let level = 49;
    ensure!(
        RegionalRoster::current()?.target(pack, species, Some(level))? == species,
        "expected an active regional Fortigard"
    );
    let learnset = pack
        .data
        .learnsets
        .get(species)
        .context("Fortigard learnset")?;
    let team = &mut pack
        .data
        .trainers
        .trainers
        .get_mut("MN_LEAGUE_2")
        .context("Flint")?
        .party;
    let ace = team.last_mut().context("Flint's ace")?;
    ensure!(
        ace.species == species
            && ace.level == level
            && ace.moves.iter().map(|m| m.name.as_str()).eq([
                "SANDSTORM",
                "ROCK_SLIDE",
                "BARRIER",
                "PSYCHIC_M"
            ]),
        "expected Flint's original level-49 ace"
    );
    for (slot, name) in [(0, "DEFENSE_CURL"), (2, "ROLLOUT")] {
        ensure!(
            learnset
                .iter()
                .any(|entry| entry.0 <= level && entry.1 == name),
            "{species} cannot learn {name} by level {level}"
        );
        ace.moves[slot] = LearnedMove {
            name: name.into(),
            current_pp: pack.data.moves.get(name).context("Fortigard move")?.pp,
            pp_ups: 0,
        };
    }
    Ok(())
}
