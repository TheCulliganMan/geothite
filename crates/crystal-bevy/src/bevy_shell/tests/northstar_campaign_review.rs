//! User-authorized level boost for story review; never earned-party evidence.
use crystal_core::models::pokemon::Stat;
use crystal_core::systems::experience::calculate_experience;
use crystal_runtime::CrystalRuntime;

use super::{VisibleShellController, mark_runtime_snapshot_dirty};

pub(super) fn boost(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let Some(value) = std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL") else {
        return;
    };
    let level = value
        .to_str()
        .expect("review level text")
        .parse::<u8>()
        .expect("explicit review level");
    assert!((1..=100).contains(&level));
    let before = c.shell.shell.session().state().clone();
    let state = c.shell.shell.session_mut().state_mut();
    for p in state.storage.party.pokemon.iter_mut().flatten() {
        if p.level >= level {
            continue;
        }
        let missing_hp = p.max_hp - p.hp;
        let fainted = p.hp == 0;
        p.level = level;
        p.experience = calculate_experience(r.growth_rates(), &p.species.growth_rate, level)
            .expect("native growth curve for boosted review");
        p.max_hp = p.calculate_stat(Stat::Hp).expect("native boosted HP");
        p.hp = if fainted { 0 } else { p.max_hp - missing_hp };
        p.attack = p
            .calculate_stat(Stat::Attack)
            .expect("native boosted attack");
        p.defense = p
            .calculate_stat(Stat::Defense)
            .expect("native boosted defense");
        p.speed = p.calculate_stat(Stat::Speed).expect("native boosted speed");
        p.special_attack = p
            .calculate_stat(Stat::SpecialAttack)
            .expect("native boosted special attack");
        p.special_defense = p
            .calculate_stat(Stat::SpecialDefense)
            .expect("native boosted special defense");
        p.validate_saved_state()
            .expect("valid boosted review partner");
    }
    assert_eq!(state.flags, before.flags);
    assert_eq!(state.badges, before.badges);
    assert_eq!(state.money, before.money);
    assert_eq!(state.bag, before.bag);
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    eprintln!(
        "BOOSTED STORY REVIEW: user-authorized minimum level {level}; progression remains unchanged"
    );
}

pub(super) fn require_unboosted() {
    assert!(
        std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_none(),
        "run the explicitly named boosted story review for supplied levels"
    );
}

#[test]
#[ignore = "user-authorized boosted review; requires a real same-pack checkpoint and explicit review level"]
fn northstar_campaign_boosted_story_review_ice_to_brainerd() {
    assert!(std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_some());
    let _ = super::northstar_campaign_wind::wind();
}
