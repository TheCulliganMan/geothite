//! Preparation available to the three established parties before League registration.
use std::collections::BTreeSet;

use crystal_core::models::{Dv, Pokemon};
use crystal_core::state::GameState;
use crystal_core::systems::shop::close_active_shop;

use super::badge_battle_parties;
use crate::northstar_regional_roster::RegionalRoster;
use crate::{GameDataSet, RuntimeMoveReminderCommand};

const SIXTH_CATCH: [(&str, usize, &str, u8); 3] = [
    ("MnR13C00R01", 3, "FORTIGARD", 46),
    ("MnR13C00R02", 2, "LATCHQUILL", 45),
    ("MnR13C00R02", 3, "PYROPINE", 46),
];

fn workshop_choices(species: &str) -> &'static [(&'static str, &'static str)] {
    match species {
        "CYPRIELLE" => &[("SAFEGUARD", "SYNTHESIS")],
        "FURROWBANK" => &[("SLAM", "DIG")],
        "PARAFLOSS" => &[("WHIRLWIND", "STUN_SPORE")],
        "PRAIRONDO" => &[("MIRROR_MOVE", "MAGNITUDE")],
        "LATCHQUILL" => &[("IRON_TAIL", "HEADBUTT")],
        "BOREALYNX" => &[("SLASH", "BITE")],
        "MOTHOSAIC" => &[("WING_ATTACK", "SIGNAL_BEAM")],
        "PYROPINE" => &[("FIRE_SPIN", "LEECH_SEED")],
        "VOYAGEAR" => &[("SEISMIC_TOSS", "WATERFALL"), ("VITAL_THROW", "AQUA_JET")],
        "SHEDSTAG" => &[("TAKE_DOWN", "RAZOR_LEAF")],
        "VERNACAPE" => &[("FUTURE_SIGHT", "PSYBEAM")],
        "CHOIRBOG" => &[],
        _ => panic!("unexpected raised League partner {species}"),
    }
}

/// Returns a single, fully healed six-member squad for the entire first challenge.
/// Starter indices are 0 for Pouchid, 1 for Kindlit and 2 for Birchit.
pub(super) fn party(data: &GameDataSet, starter_index: usize) -> Vec<Pokemon> {
    let raised = badge_battle_parties::party(data, starter_index, 9);
    let mut state = GameState::reset_wram_for_new_game();
    state.money = 3_060; // Up to six lessons at ₽500 and six Gold Berries at ₽10.
    for member in raised {
        assert!(state.storage.party.add_pokemon(member));
    }
    state.sync_party_from_storage();
    let workshop = &data.maps["MnMinneapolisToolWorkshop"];
    assert!(
        workshop
            .objects
            .iter()
            .any(|o| o.script == "MN_MP_INTERIOR_TOOL_APPRENTICE")
    );
    for slot in 0..5 {
        let choices = workshop_choices(
            &state.storage.party.pokemon[slot]
                .as_ref()
                .expect("raised partner")
                .species
                .id,
        );
        for &(old, new) in choices {
            let moves = &state.storage.party.pokemon[slot]
                .as_ref()
                .expect("workshop student")
                .moves;
            if moves.iter().any(|m| m.name == new) {
                continue;
            }
            let forget = moves
                .iter()
                .position(|m| m.name == old)
                .expect("scheduled move to replace");
            data.relearn_party_move(
                &mut state,
                RuntimeMoveReminderCommand {
                    party_index: slot,
                    move_id: new.into(),
                    forget_index: Some(forget),
                },
                &BTreeSet::new(),
            )
            .expect("paid, legal move reminder lesson");
        }
    }

    let mart = "MnGrandMaraisMart";
    let clerk = data.maps[mart]
        .script_shop_commands
        .iter()
        .find(|command| command.mart_id == "MART_MN_GRAND_MARAIS")
        .expect("Grand Marais counter");
    data.open_script_shop(
        &mut state,
        mart,
        mart,
        &clerk.source_script,
        clerk.command_index,
    )
    .expect("open Grand Marais stock");
    let berry = &data.items["GOLD_BERRY"];
    assert_eq!(berry.price, 10);
    assert!(
        data.buy_shop_item(&mut state, "GOLD_BERRY", 6)
            .expect("Grand Marais purchase")
            .success
    );
    close_active_shop(&mut state).expect("leave the counter");

    let (route, slot, species, floor) = SIXTH_CATCH[starter_index];
    let wild = data.wild_encounters[route]
        .grass
        .as_ref()
        .expect("Route 13 daytime grass");
    let catch = &wild.day[slot];
    assert_eq!((catch.species.as_str(), catch.level), (species, floor));
    let level = data.ordinary_challenge_level(&state, route, catch.level, 2, 0);
    assert_eq!(level, floor);
    assert!(
        RegionalRoster::current()
            .expect("active Minnesota roster")
            .allowed
            .contains(species)
    );
    let mut recruit = data
        .create_pokemon(species, level, Dv::from_non_hp(8, 8, 8, 8))
        .expect("Route 13 catch");
    data.equip_wild_challenge_moves(&mut recruit, route)
        .expect("ordinary wild encounter moves");
    assert!(state.storage.party.add_pokemon(recruit));

    let mut party = Vec::with_capacity(6);
    for mut member in state.storage.party.pokemon.into_iter().flatten() {
        assert!(
            state
                .bag
                .remove_item(berry, 1)
                .expect("one purchased berry per partner")
        );
        assert!(
            member.item.replace("GOLD_BERRY".into()).is_none(),
            "held slot must be free"
        );
        party.push(member);
    }
    assert_eq!(party.len(), 6);
    party
}
