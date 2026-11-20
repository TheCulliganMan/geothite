//! One copy of each Minnesota leader's reward and Brainerd's optional Surf HM.
use crystal_core::state::GameState;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::systems::script_items::ScriptItemGrantOutcome;

use crate::GameDataSet;

const REWARDS: [(&str, &str); 8] = [
    ("MnRochesterGym", "TM_HEADBUTT"),
    ("MnMankatoGym", "TM_MUD_SLAP"),
    ("MnMinneapolisGym", "TM_THUNDERPUNCH"),
    ("MnStPaulGym", "TM_STEEL_WING"),
    ("MnStCloudGym", "TM_SWIFT"),
    ("MnBrainerdGym", "TM_RAIN_DANCE"),
    ("MnBemidjiGym", "TM_GIGA_DRAIN"),
    ("MnDuluthGym", "TM_ICY_WIND"),
];

fn collect(data: &GameDataSet, state: &mut GameState, map: &str, script: &str, id: &str) {
    let grant = data.maps[map]
        .script_item_grants
        .iter()
        .find(|g| g.source_script == script && g.item_id == id && g.quantity == 1)
        .expect("installed machine gift");
    let item = &data.items[id];
    assert_eq!(state.bag.quantity(item), 0);
    let receipt = data
        .grant_script_item(state, map, map, script, grant.command_index)
        .expect("collect the machine");
    assert!(matches!(
        receipt,
        ScriptItemGrantOutcome::Granted { quantity: 1, .. }
    ));
    assert_eq!(state.bag.quantity(item), 1);
}

fn teach(data: &GameDataSet, state: &mut GameState, species: &str, machine: &str, forget: &str) {
    let party = &state.storage.party.pokemon;
    let (slot, partner) = party
        .iter()
        .enumerate()
        .find_map(|(i, p)| {
            let p = p.as_ref()?;
            (p.species.id == species).then_some((i, p))
        })
        .expect("preselected traveling partner");
    let item = &data.items[machine];
    let new_move = item.tmhm_move.as_deref().expect("machine move");
    assert!(partner.species.tmhm_learnset.iter().any(|m| m == new_move));
    let replace = partner
        .moves
        .iter()
        .position(|m| m.name == forget)
        .unwrap_or_else(|| panic!("{species} should know {forget}: {:?}", partner.moves));
    let (used, learned) = data
        .use_bag_tmhm_on_party_pokemon(state, machine, slot, Some(replace))
        .expect("teach through the Pack");
    assert_eq!(learned.replaced_move.as_deref(), Some(forget));
    assert_eq!(learned.learned_move, new_move);
    assert_eq!(used.consumed, item.consumable);
    assert_eq!(state.bag.quantity(item), u16::from(!item.consumable));
    let partner = state.storage.party.pokemon[slot]
        .as_ref()
        .expect("taught partner");
    assert_eq!(partner.moves[replace].current_pp, data.moves[new_move].pp);
    println!("ely-machine,{species},{machine},{forget},{new_move}");
}

/// The player saved each badge TM, accepted the ranger's HM and chose these
/// permanent lessons in town before the first battle. Every seed uses them.
pub(super) fn prepare(data: &GameDataSet, state: &mut GameState, starter: usize) {
    let wallet = state.money;
    for ((map, id), flag) in REWARDS.into_iter().zip(BADGE_ENGINE_FLAGS) {
        assert!(
            state
                .flags
                .is_engine_flag_set(flag)
                .expect("earned gym badge")
        );
        collect(data, state, map, &format!("{map}LeaderReward"), id);
    }
    collect(
        data,
        state,
        "MnBrainerd",
        "MnFlowRangerOfferSurf",
        "HM_SURF",
    );

    // Parafloss gets immediate special coverage and recovery in place of
    // two-turn Razor Wind. The Birchit party gives that coverage to Vernacape.
    if starter == 2 {
        teach(data, state, "VERNACAPE", "TM_GIGA_DRAIN", "SAFEGUARD");
    } else {
        teach(data, state, "PARAFLOSS", "TM_GIGA_DRAIN", "RAZOR_WIND");
    }
    if starter == 1 {
        teach(data, state, "CHOIRBOG", "HM_SURF", "AMNESIA");
    } else {
        teach(data, state, "FURROWBANK", "TM_HEADBUTT", "SUBSTITUTE");
    }
    assert_eq!(state.money, wallet);
}
