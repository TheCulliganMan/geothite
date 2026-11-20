//! Matchups at the levels where these League and rematch parties are met.
use crystal_core::battle::damage::{DamageContext, calculate_damage};
use crystal_core::battle::start::materialize_trainer_party;
use crystal_core::battle::turn::{
    BattleAction, BattleCombatState, BattleTurnInput, resolve_battle_turn_with_items,
};
use crystal_core::models::{Dv, Pokemon, Stat};
use crystal_core::random::BattleRandomSource;

use crate::{GameDataSet, read_verified_compiled_game_pack};

fn partner(data: &GameDataSet, species: &str, level: u8) -> Pokemon {
    data.create_pokemon(species, level, Dv::from_non_hp(8, 8, 8, 8))
        .expect("create a regional partner with no stat experience")
}

fn opponent(
    data: &GameDataSet,
    trainer_id: &str,
    species: &str,
    slot: usize,
    name: &str,
) -> Pokemon {
    let trainer = &data.trainers.trainers[trainer_id];
    let mon = materialize_trainer_party(
        trainer,
        &data.pokemon,
        &data.learnsets,
        &data.moves,
        &data.growth_rates,
    )
    .expect("instantiate the compiled trainer party")
    .into_iter()
    .find(|mon| mon.species.id == species)
    .expect("authored opponent");
    assert_eq!(mon.moves[slot].name, name);
    assert_eq!(mon.moves[slot].current_pp, data.moves[name].pp);
    assert!(
        data.learnsets[species]
            .iter()
            .any(|entry| entry.0 <= mon.level && entry.1 == name)
    );
    mon
}

fn hit(data: &GameDataSet, attacker: &Pokemon, target: &Pokemon, name: &str) -> u16 {
    calculate_damage(
        attacker,
        target,
        &data.moves[name],
        &data.battle_stat_multipliers,
        &data.type_categories,
        &data.type_effectiveness,
        &data.weather_modifiers,
        DamageContext {
            random_roll: 236,
            ..DamageContext::default()
        },
    )
    .expect("ordinary hit in clear weather")
    .damage
}

struct GlareHits;

impl BattleRandomSource for GlareHits {
    fn battle_random_byte(&mut self) -> u8 {
        // Below Glare's 75% accuracy threshold, above full-paralysis odds.
        0xb9
    }
}

#[test]
#[ignore = "requires a badge-battles edition in CRYSTAL_MINNESOTA_BATTLE_TEST_PACK"]
fn league_and_rematch_moves_change_their_authored_matchups() {
    let path = std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK")
        .expect("provide the authored badge-battles pack");
    let pack = read_verified_compiled_game_pack(path).expect("verified Minnesota battle pack");
    let data = pack.data();

    // This Surf encounter on the League approach is an available Steel/Water pivot.
    let approach = &data.wild_encounters["MnR13C00R01"];
    assert!(approach.water_rate.is_some_and(|rate| rate > 0));
    assert!(
        approach
            .water
            .as_ref()
            .expect("daytime Surf table")
            .day
            .iter()
            .any(|entry| entry.species == "MUSKETUNGE" && entry.level == 47)
    );
    let steel_fish = partner(data, "MUSKETUNGE", 47);
    let flower = partner(data, "CYPRIELLE", 50);
    let mut blade = opponent(data, "MN_LEAGUE_3", "DOUBLEHEX", 0, "IRON_HEAD");
    assert_eq!(blade.moves[2].name, "SWORDS_DANCE");
    let plain = hit(data, &blade, &flower, "IRON_HEAD");
    assert!(plain > hit(data, &blade, &flower, "CRUNCH"));
    blade.stat_boosts.insert(Stat::Attack, 2);
    assert!(hit(data, &blade, &flower, "IRON_HEAD") > plain);
    assert!(hit(data, &blade, &steel_fish, "IRON_HEAD") < steel_fish.max_hp / 2);

    let knot = opponent(data, "MN_LEAGUE_4", "KNOTITAN", 2, "GLARE");
    assert!(
        data.learnsets["CYPRIELLE"]
            .iter()
            .any(|entry| entry.0 == 46 && entry.1 == "MOONBLAST")
    );
    assert!(hit(data, &flower, &knot, "MOONBLAST") >= knot.max_hp);
    let state = BattleCombatState::new(flower.clone(), knot.clone())
        .with_parties(vec![flower, steel_fish], vec![knot]);
    // Resolve an actual switch. Paralysis belongs to the arriving Pokémon.
    let turn = resolve_battle_turn_with_items(
        state,
        BattleTurnInput {
            player: BattleAction::Switch { party_index: 1 },
            enemy: BattleAction::Move { slot: 2 },
        },
        &data.moves,
        &data.items,
        &data.move_priorities,
        &data.battle_stat_multipliers,
        &data.type_categories,
        &data.type_effectiveness,
        &data.weather_modifiers,
        &mut GlareHits,
    )
    .expect("Halden uses Glare as the player switches");
    assert_eq!(turn.state.player.species.id, "MUSKETUNGE");
    assert_eq!(turn.state.player.status.as_deref(), Some("PARALYSIS"));
    assert!(turn.state.player_party[0].status.is_none());
    assert_eq!(
        turn.state.enemy.moves[2].current_pp,
        data.moves["GLARE"].pp - 1
    );

    let guard = opponent(data, "MN_GYM_8_REMATCH", "RIMEGUARD", 0, "ICE_PUNCH");
    let flower = partner(data, "CYPRIELLE", guard.level);
    let otter = partner(data, "VOYAGEAR", guard.level);
    assert!(hit(data, &guard, &flower, "ICE_PUNCH") > hit(data, &guard, &flower, "ICE_BEAM"));
    assert!(hit(data, &guard, &otter, "ICE_PUNCH") < otter.max_hp / 2);
    assert!(
        data.learnsets["VOYAGEAR"]
            .iter()
            .any(|entry| entry.0 == 38 && entry.1 == "DRAIN_PUNCH")
    );
    assert!(hit(data, &otter, &guard, "DRAIN_PUNCH") > hit(data, &guard, &otter, "ICE_PUNCH"));
}
