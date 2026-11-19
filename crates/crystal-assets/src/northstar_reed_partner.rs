//! Fix two trainer moves whose battle effects did not serve their partners.
use anyhow::{Context, Result, ensure};
use crystal_core::models::LearnedMove;
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::northstar_battle_balance::{AuthoredMember, build_member};
use crate::northstar_regional_roster::RegionalRoster;

/// Restore an effective Normal attack for Reed and a rain follow-up for Kite.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    fix_reed_partner(pack)?;
    fix_kite_partner(pack)
}

fn fix_reed_partner(pack: &mut CompiledGamePack) -> Result<()> {
    let species = pack.data.pokemon.get("GREYTAG").context("Greytag")?;
    ensure!(
        species.tmhm_learnset.iter().any(|name| name == "STRENGTH"),
        "Greytag cannot learn Strength"
    );
    let strength = pack.data.moves.get("STRENGTH").context("Strength")?;
    let reed = pack
        .data
        .trainers
        .trainers
        .get_mut("MN_REED_REFINEMENT")
        .context("Reed's Champion rematch")?;
    let partner = reed.party.get_mut(3).context("Reed's fourth partner")?;
    ensure!(
        partner.species == "GREYTAG" && partner.level == 57,
        "unexpected partner in Reed's fourth slot"
    );
    let [attack, low_kick, priority, encore] = partner.moves.as_mut_slice() else {
        anyhow::bail!("unexpected number of moves on Reed's Greytag");
    };
    ensure!(
        matches!(attack.name.as_str(), "RETURN" | "STRENGTH")
            && low_kick.name == "LOW_KICK"
            && priority.name == "QUICK_ATTACK"
            && encore.name == "ENCORE",
        "unexpected moves on Reed's Greytag"
    );
    *attack = LearnedMove {
        name: "STRENGTH".into(),
        current_pp: strength.pp,
        pp_ups: 0,
    };
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteTrainer {
    trainer: String,
    party_slot: usize,
    member: AuthoredMember,
}

fn fix_kite_partner(pack: &mut CompiledGamePack) -> Result<()> {
    let design: RouteTrainer = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/trail-discoveries-v01/route-trainer.json"
    ))?;
    let index = design.party_slot.checked_sub(1).context("party slot")?;
    let original = pack
        .data
        .trainers
        .get(&design.trainer)
        .and_then(|trainer| trainer.party.get(index))
        .context("Kite's rain partner")?;
    ensure!(
        original.species == design.member.species
            && original.level == design.member.level
            && original.item == design.member.item,
        "unexpected partner in Kite's second slot"
    );
    let reviewed = build_member(
        pack,
        &RegionalRoster::current()?,
        &design.member,
        original.dvs,
    )?;
    let [amnesia, sludge, psychic, drum] = original.moves.as_slice() else {
        anyhow::bail!("unexpected number of moves on Kite's Chorip");
    };
    ensure!(
        [amnesia, sludge, psychic]
            .into_iter()
            .eq(&reviewed.moves[..3])
            && matches!(drum.name.as_str(), "BELLY_DRUM" | "WATER_GUN"),
        "unexpected moves on Kite's Chorip"
    );
    pack.data
        .trainers
        .trainers
        .get_mut(&design.trainer)
        .context("Kite")?
        .party[index] = reviewed;
    Ok(())
}
