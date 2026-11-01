//! Decision caches must preserve the complete sampled response values.

use std::path::PathBuf;

use crystal_core::battle::turn::{BattleAction, BattleCombatState, BattleSide};
use crystal_core::models::{Dv, LearnedMove};

use super::{TrainerBattleProfile, compact_planning_species, competitive_move_slots};
use crate::read_verified_compiled_game_pack;

#[test]
fn cached_continuations_match_uncached_search_across_profiles_and_repeated_choices() {
    let path = std::env::var_os("CRYSTAL_MODPACK_TEST_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/core-modular.browser.crystalpack")
        });
    let path = path.canonicalize().expect("resolve external battle pack");
    let pack = read_verified_compiled_game_pack(path).expect("load external battle pack");
    let data = pack.data();
    let pokemon = |id, moves: &[&str]| {
        let mut pokemon = data
            .create_pokemon(id, 50, Dv::from_non_hp(15, 15, 15, 15))
            .expect("build legal planner fixture");
        pokemon.moves = moves
            .iter()
            .map(|name| LearnedMove {
                name: (*name).to_string(),
                current_pp: data.moves[*name].pp,
                pp_ups: 0,
            })
            .collect();
        pokemon
    };
    let state = BattleCombatState::new(
        pokemon(
            "CYNDAQUIL",
            &["SWIFT", "EMBER", "SMOKESCREEN", "QUICK_ATTACK"],
        ),
        pokemon("SUDOWOODO", &["ROCK_THROW", "LOW_KICK", "MIMIC", "FLAIL"]),
    );
    let original = state.clone();
    let mut cached = data.competitive_search();
    let planning = data
        .competitive_planning_state(&state, &mut cached)
        .expect("prepare public planning information");
    let mut compact = planning.clone();
    compact_planning_species(&mut compact);
    for profile in TrainerBattleProfile::ALL {
        let mut uncached = data.competitive_search();
        uncached.continuation_limit = 0;
        let slots = competitive_move_slots(&planning, BattleSide::Enemy);
        for slot in slots.iter().copied().chain(slots.iter().rev().copied()) {
            let action = BattleAction::Move { slot };
            let expected = data
                .competitive_action_values(&planning, &action, true, profile, &mut uncached)
                .expect("uncached response values");
            let actual = data
                .competitive_action_values(&compact, &action, true, profile, &mut cached)
                .expect("cached response values");
            assert_eq!(actual, expected, "{profile:?}, move slot {slot}");
            assert_eq!(
                state, original,
                "hypothetical turns must not change live combat"
            );
        }
    }
    assert!(!cached.continuations.is_empty());
    assert!(cached.continuations.len() <= cached.continuation_limit);
}
