//! Apply the roster and opening balance pass to an installed Northstar game.
use anyhow::{Context, Result, ensure};
use crystal_core::world::fishing::{FishingGroup, ROD_OLD};

use crate::CompiledGamePack;
use crate::northstar_regional_cast;
use crate::northstar_regional_roster::RegionalRoster;
use crate::northstar_story_revision::{self, InvestigationAccess};

const REPLACEMENTS: &[(&str, &str)] = &[
    ("AUGRUB", "SODCALF"),
    ("BOBCAST", "PINGLET"),
    ("BOUGHLASH", "CROOKVEIL"),
    ("BUOYOSTIC", "AURORILL"),
    ("DENSY", "TOTLIT"),
    ("DENUMBRA", "CASSEROGRE"),
    ("DOZEBEAR", "STACKAJACK"),
    ("DRIFTILE", "JOTLET"),
    ("FLARKINDLE", "MITWIST"),
    ("GILLPUP", "LOOBIT"),
    ("GLISNARE", "MUFFIEND"),
    ("GOOSNIDE", "VANECHICK"),
    ("GOUGECREST", "GREYTAG"),
    ("HELIKNOCK", "RIFTMAUL"),
    ("MUDMENDER", "CHORIP"),
    ("NOTCHBILL", "BOOMRUFF"),
    ("PECKIT", "TAPDUCK"),
    ("PEPIVERN", "BARBELISK"),
    ("REGALARCH", "DAKOTASKIP"),
    ("RIVVIT", "CHORIP"),
    ("SLIDEWAKE", "PORTABEAU"),
    ("SOLSTORT", "CHORIP"),
    ("STRANDRAK", "KENSIGARD"),
    ("STRIPILLAR", "DEWDANT"),
    ("TICKLET", "SCUTLET"),
    ("TILTAWISP", "RIFFBIT"),
    ("TIPULORD", "MUSKETUNGE"),
    ("WHIRLNUT", "FLAPIP"),
    ("WINKFIN", "SCUTLET"),
];

/// Applies the current dialogue, curated cast and opening Old Rod balance.
///
/// Returns the revised pack and dialogue labels that need separate staging.
///
/// # Errors
/// Rejects invalid authored dialogue, missing roster bindings, or invalid pack content.
pub fn polish_northstar_campaign(
    base: &CompiledGamePack,
    story: &str,
) -> Result<(CompiledGamePack, Vec<String>)> {
    polish_with_access(base, story, InvestigationAccess::PreserveBadgeChecks)
}

pub(crate) fn polish_with_access(
    base: &CompiledGamePack,
    story: &str,
    access: InvestigationAccess,
) -> Result<(CompiledGamePack, Vec<String>)> {
    let (mut pack, deferred) =
        northstar_story_revision::refresh_installed_story(base, story, access)?;
    let policy = RegionalRoster::current()?.with_bindings(REPLACEMENTS)?;
    crate::icemyth_discovery_pack::retire_exhibit(&mut pack)?;

    let trainers: Vec<_> = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|trainer| {
            trainer.trainer_id.starts_with("MN_")
                && trainer
                    .party
                    .iter()
                    .any(|member| !policy.allowed.contains(&member.species))
        })
        .cloned()
        .collect();
    for mut trainer in trainers {
        let has_boomruff = trainer
            .party
            .iter()
            .any(|member| member.species == "BOOMRUFF");
        let has_sodcalf = trainer
            .party
            .iter()
            .any(|member| member.species == "SODCALF");
        for member in &mut trainer.party {
            if !policy.allowed.contains(&member.species) {
                // Keep an existing Flying or Ground ace distinct from its teammates.
                let replacement = match member.species.as_str() {
                    "NOTCHBILL" if has_boomruff => "BANKWING",
                    "AUGRUB" if has_sodcalf => "SETTKIT",
                    original => original,
                };
                member.species = policy.target(&pack, replacement, Some(member.level))?;
                member.moves = pack
                    .data
                    .create_pokemon(&member.species, member.level, member.dvs)?
                    .moves;
            }
        }
        pack.data
            .trainers
            .trainers
            .insert(trainer.trainer_id.clone(), trainer);
    }

    // The campaign bosses have authored roles and recurring partners. Apply
    // those teams after ordinary roster repair so replacement species do not
    // silently inherit a retired partner's moves or lose their intended role.
    let teams = serde_json::from_str(include_str!(
        "../../../planning/minnesota/story-pokemon.json"
    ))?;
    crate::northstar_faction_teams::install(&mut pack, &policy, &teams)?;

    let maps = pack
        .data
        .maps
        .keys()
        .filter(|name| name.starts_with("Mn"))
        .cloned()
        .collect();
    northstar_regional_cast::bind_map_species(&mut pack, &policy, maps)?;
    for map in ["MnWinona", "MnR01C00R00", "MnRochester"] {
        ease_first_cast(&mut pack, map)?;
    }
    northstar_regional_cast::audit(&pack, &policy)?;

    pack.report
        .manifests
        .push("northstar-campaign-polish-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok((pack, deferred))
}

fn ease_first_cast(pack: &mut CompiledGamePack, map: &str) -> Result<()> {
    let mut map = pack
        .data
        .maps
        .get(map)
        .context("opening fishing map")?
        .clone();
    let id = map
        .attributes
        .fishing_group
        .as_deref()
        .context("opening fishing group")?;
    let mut group = pack
        .data
        .fishing
        .groups
        .get(id)
        .context("installed fishing group")?
        .clone();
    lower_old_rod_levels(&mut group, id)?;
    let swarm_groups: Vec<_> = pack
        .data
        .fishing
        .swarm_rules
        .values()
        .filter(|rule| rule.base_group == id)
        .map(|rule| rule.swarm_group.clone())
        .collect();
    if pack
        .data
        .maps
        .values()
        .any(|other| other.id != map.id && other.attributes.fishing_group.as_deref() == Some(id))
    {
        ensure!(
            swarm_groups.is_empty(),
            "shared opening fishing group has a swarm"
        );
        group.source_index = pack
            .data
            .fishing
            .groups
            .values()
            .map(|group| group.source_index)
            .max()
            .and_then(|last| last.checked_add(1))
            .context("free fishing group index")?;
        let new_id = format!("FISHGROUP_MN_FIRST_{}", map.id.to_ascii_uppercase());
        ensure!(
            !pack.data.fishing.groups.contains_key(&new_id),
            "first cast group already exists"
        );
        pack.data.fishing.groups.insert(new_id.clone(), group);
        map.attributes.fishing_group = Some(new_id);
        crate::authored_town_publication::replace(pack, map)?;
    } else {
        pack.data.fishing.groups.insert(id.into(), group);
        for name in swarm_groups {
            let group = pack
                .data
                .fishing
                .groups
                .get_mut(&name)
                .context("opening swarm group")?;
            lower_old_rod_levels(group, &name)?;
        }
    }
    Ok(())
}

fn lower_old_rod_levels(group: &mut FishingGroup, name: &str) -> Result<()> {
    let old_rod = group
        .rod_tables
        .get_mut(ROD_OLD)
        .context("opening Old Rod table")?;
    ensure!(
        old_rod.slots.len() == 3,
        "expected three opening Old Rod slots in {}",
        name
    );
    for (slot, level) in old_rod.slots.iter_mut().zip([4, 5, 6]) {
        ensure!(
            slot.species.is_some() && slot.time_group.is_none(),
            "expected direct opening Old Rod encounters"
        );
        slot.level = level;
    }
    Ok(())
}
