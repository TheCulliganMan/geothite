//! Keep regional research membership and its final award aligned with the roster.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::northstar_rival_arc::{map, publish};

const PREFIX: &str = "MN_DEX_MEMBER_";
const MANIFEST: &str = "minnesota-dex-roster-sync-v1";

/// Numbers active pack members in the same family order as the published Pokédex.
pub(crate) fn install_regional_numbers(pack: &mut CompiledGamePack) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct Entry {
        id: String,
        number: u16,
    }
    #[derive(serde::Deserialize)]
    struct Order {
        species: Vec<Entry>,
    }
    let order: Order = serde_json::from_str(include_str!(
        "../../../planning/minnesota/pokemon/pokedex-order.json"
    ))?;
    let source_numbers: BTreeMap<_, _> = order
        .species
        .into_iter()
        .map(|e| (e.id, e.number))
        .collect();
    let roster: crate::minnesota_pokemon_profiles::Roster = serde_json::from_str(include_str!(
        "../../../planning/minnesota/pokemon/roster.json"
    ))?;
    let mut families = BTreeMap::new();
    for family in roster.families {
        let first = family.members.first().context("empty regional family")?;
        let number = source_numbers
            .get(&first.id)
            .context("unlisted regional family")?;
        ensure!(
            families.insert(*number, family.members).is_none(),
            "duplicate family order"
        );
    }
    let expected = pack.data.minnesota_dex_members().count();
    let mut count = 0;
    for member in families.into_values().flatten() {
        let key = format!("{PREFIX}{}", member.name.to_ascii_uppercase());
        if let Some(marker) = pack.data.story_event_script_constants.global.get_mut(&key) {
            count += 1;
            *marker = i64::try_from(count)?;
        }
    }
    ensure!(
        count == expected && count > 0,
        "regional numbering omitted an active species"
    );
    Ok(())
}

/// Adds newly compiled active species to regional research and updates only the
/// final caught-count threshold and its displayed number. Existing award flags,
/// starter wrappers and all unrelated content remain. The obsolete 50-species
/// EXP Share award becomes ten Ultra Balls using its existing claim flag.
///
/// # Errors
/// Rejects missing species, retired existing members, duplicate installation,
/// malformed roster/curation, or an ambiguous final award program/text.
pub fn build_minnesota_dex_roster_sync(
    base: &CompiledGamePack,
    roster: &str,
    curation: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "Dex sync already installed"
    );
    let roster: Value = serde_json::from_str(roster)?;
    let curation: Value = serde_json::from_str(curation)?;
    let retired: BTreeSet<_> = curation["retired_families"]
        .as_array()
        .context("retired families")?
        .iter()
        .filter_map(|f| f["id"].as_str())
        .collect();
    let mut active = BTreeSet::new();
    for family in roster["families"].as_array().context("roster families")? {
        if retired.contains(family["id"].as_str().context("family ID")?) {
            continue;
        }
        for member in family["members"].as_array().context("family members")? {
            let name = member["name"]
                .as_str()
                .context("species name")?
                .to_ascii_uppercase();
            ensure!(
                base.data.pokemon.contains_key(&name),
                "compile active species first: {name}"
            );
            ensure!(active.insert(name), "duplicate active species");
        }
    }
    let previous: BTreeSet<_> = base
        .data
        .story_event_script_constants
        .global
        .keys()
        .filter_map(|key| key.strip_prefix(PREFIX).map(str::to_owned))
        .collect();
    ensure!(
        !previous.is_empty() && previous.is_subset(&active) && previous.len() < active.len(),
        "requires an expanded, compiled roster without retiring existing members"
    );
    let old_count = previous.len().to_string();
    let new_count = active.len().to_string();
    let mut pack = base.clone();
    for name in active.difference(&previous) {
        pack.data
            .story_event_script_constants
            .global
            .insert(format!("{PREFIX}{name}"), 1);
    }
    let mut room = map(&pack, "MnWinonaFieldStation")?;
    let mut claim = None;
    let mut guards = 0;
    for (label, program) in &mut room.scripts {
        if label != "MnRewardDex" && !label.starts_with("MnRewardDexTier") {
            continue;
        }
        let commands = program.as_array_mut().context("research program")?;
        let mut changed = false;
        for command in commands.iter_mut() {
            if command["command"] == "ifless"
                && command["args"][0] == old_count
                && command["args"][1] == "MnRewardDexNeed"
            {
                command["args"][0] = new_count.clone().into();
                guards += 1;
                changed = true;
            }
        }
        if changed {
            claim = commands
                .iter()
                .find(|c| c["command"] == "sjump")
                .and_then(|c| c["args"][0].as_str())
                .map(str::to_owned);
        }
    }
    ensure!(guards == 1, "exactly one final reward threshold required");
    let claim = claim.context("final award branch")?;
    let text = room.scripts[&claim]
        .as_array()
        .context("final claim program")?
        .iter()
        .find(|c| c["command"] == "writetext")
        .and_then(|c| c["args"][0].as_str())
        .context("final claim text")?
        .to_owned();
    let mut replacements = 0;
    for command in room
        .scripts
        .get_mut(&text)
        .context("claim text body")?
        .as_array_mut()
        .context("claim text instructions")?
    {
        for operand in command["args"].as_array_mut().context("text operands")? {
            if let Some(row) = operand.as_str().filter(|row| row.contains(&old_count)) {
                let updated = row.replace(&old_count, &new_count);
                ensure!(
                    updated.trim_matches('"').chars().count() <= 18,
                    "final reward text exceeds native width"
                );
                *operand = updated.into();
                replacements += 1;
            }
        }
    }
    ensure!(
        replacements == 1,
        "exactly one displayed final count required"
    );
    let mut fifty_text = None;
    for (label, program) in &mut room.scripts {
        if !label.starts_with("MnRewardDexClaim") {
            continue;
        }
        let Some(commands) = program.as_array_mut() else {
            continue;
        };
        let mut replaced = false;
        for command in commands.iter_mut() {
            if command["command"] == "verbosegiveitem" && command["args"][0] == "EXP_SHARE" {
                command["args"][0] = "ULTRA_BALL".into();
                command["args"][1] = "10".into();
                replaced = true;
            }
        }
        if replaced {
            ensure!(fifty_text.is_none(), "ambiguous obsolete research reward");
            fifty_text = commands
                .iter()
                .find(|c| c["command"] == "writetext")
                .and_then(|c| c["args"][0].as_str())
                .map(str::to_owned);
        }
    }
    if let Some(label) = fifty_text {
        authoring::text(
            &mut room,
            &label,
            &[
                "You've caught 50 Minnesota species! Your field notes earned this research award.",
                "ULTRA BALLs help catch elusive Pokemon. Keep exploring!",
                "Each award is given once.",
            ],
        );
        authoring::text(
            &mut room,
            "MnRewardDexNeedText",
            &[
                "I award research supplies for 10, 25, 50, 100 and all Minnesota species caught.",
                "Seen entries don't count. Partners may stay in the PC.",
                "All healthy party Pokemon already share battle experience. No item is needed.",
            ],
        );
    }
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

#[cfg(test)]
mod regional_number_tests {
    use std::collections::BTreeSet;

    #[test]
    #[ignore = "requires the original Fair Table pack in MN_STARTER_BENCH_PARENT"]
    fn fair_table_dex_numbers_are_dense_and_follow_the_published_starter_families()
    -> anyhow::Result<()> {
        let mut pack = crate::read_verified_compiled_game_pack(
            std::env::var_os("MN_STARTER_BENCH_PARENT").expect("external Fair Table pack"),
        )?;
        let expected: BTreeSet<_> = (1..=225).collect();
        let before = pack.data.minnesota_dex_numbers();
        assert_eq!(before.values().copied().collect::<BTreeSet<_>>(), expected);
        assert!(!before.contains_key("CHIKORITA"));
        super::install_regional_numbers(&mut pack)?;
        let after = pack.data.minnesota_dex_numbers();
        assert_eq!(after.len(), 225);
        assert_eq!(after.values().copied().collect::<BTreeSet<_>>(), expected);
        for (species, number) in [
            ("POUCHID", 1),
            ("SLIPPERINA", 2),
            ("KINDLIT", 4),
            ("BIRCHIT", 7),
            ("VOYAGEAR", 9),
        ] {
            assert_eq!(after.get(species), Some(&number));
        }
        assert!(!after.contains_key("ICEMYTH"));
        assert!(!after.contains_key("CHIKORITA"));
        Ok(())
    }
}
