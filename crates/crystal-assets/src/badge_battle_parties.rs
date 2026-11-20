//! Daytime catch histories for the Minnesota badge battle audit.
use std::sync::OnceLock;

use crystal_core::models::{Dv, LearnedMove, Pokemon};
use crystal_core::systems::battle_rewards::apply_experience_level_ups;
use crystal_core::systems::evolution::{EvolutionContext, LinkMode, check_and_evolve};
use crystal_core::systems::experience::calculate_experience;
use crystal_core::world::encounters::TimeOfDay;

use crate::GameDataSet;
use crate::northstar_regional_roster::RegionalRoster;

const STARTERS: [&str; 3] = ["POUCHID", "KINDLIT", "BIRCHIT"];
const LEVELS: [u8; 9] = [11, 15, 21, 24, 29, 34, 39, 44, 47];
const R1: &str = "MnR01C00R00";
const R2: &str = "MnR02C00R00";
const R3: &str = "MnR03C00R00";
const R6: &str = "MnR06C00R00";

struct Catch {
    route: &'static str,
    day_slot: usize,
    species: &'static str,
}

impl Catch {
    const fn new(route: &'static str, day_slot: usize, species: &'static str) -> Self {
        Self {
            route,
            day_slot,
            species,
        }
    }
}

// Preserve the same companions as the party grows. The three teams sample
// different common rolls on the roads and never change for a particular boss.
const CATCHES: [[Catch; 4]; 3] = [
    [
        Catch::new(R1, 0, "NIBBIN"),
        Catch::new(R1, 1, "PODLET"),
        Catch::new(R2, 2, "PIPBOOM"),
        Catch::new(R6, 2, "BRISTUB"),
    ],
    [
        Catch::new(R1, 1, "PODLET"),
        Catch::new(R1, 2, "TESSLARV"),
        Catch::new(R3, 0, "CHORIP"),
        Catch::new(R6, 1, "PITCHWICK"),
    ],
    [
        Catch::new(R1, 2, "TESSLARV"),
        Catch::new(R1, 0, "NIBBIN"),
        Catch::new(R3, 2, "VELVAWN"),
        Catch::new(R6, 3, "PASQUIB"),
    ],
];

fn accept_offers(data: &GameDataSet, pokemon: &mut Pokemon, offered: Vec<LearnedMove>) {
    for new in offered {
        // Accept the newest move and forget the oldest, retaining a damaging
        // attack when the remaining three and the new offer would all be status.
        if data.moves[&new.name].power == 0
            && pokemon.moves[1..]
                .iter()
                .all(|m| data.moves[&m.name].power == 0)
        {
            continue;
        }
        pokemon.moves.remove(0);
        pokemon.moves.push(new);
    }
}

pub(super) fn train(data: &GameDataSet, pokemon: &mut Pokemon, target: u8) {
    static ROSTER: OnceLock<RegionalRoster> = OnceLock::new();
    let roster = ROSTER.get_or_init(|| RegionalRoster::current().expect("Minnesota active roster"));
    assert!(
        roster.allowed.contains(&pokemon.species.id),
        "retired acquisition"
    );
    assert!(pokemon.level <= target, "fixture catch above party ceiling");
    let context = EvolutionContext {
        species: &data.pokemon,
        moves: &data.moves,
        learnsets: &data.learnsets,
        time_of_day: TimeOfDay::Day,
        current_item: None,
        force_evolution: false,
        link_mode: LinkMode::None,
    };
    while pokemon.level < target {
        pokemon.experience = calculate_experience(
            &data.growth_rates,
            &pokemon.species.growth_rate,
            pokemon.level + 1,
        )
        .expect("next experience threshold");
        let leveled = apply_experience_level_ups(
            pokemon,
            &data.moves,
            &data.learnsets,
            &data.growth_rates,
            &data.battle_reward_rules,
        )
        .expect("normal level-up offers and stats");
        accept_offers(data, pokemon, leveled.pending_move_learns);
        let evolved = check_and_evolve(pokemon, &data.evolutions, &context, false)
            .expect("normal daytime evolution");
        assert!(
            roster.allowed.contains(&pokemon.species.id),
            "retired evolution"
        );
        accept_offers(data, pokemon, evolved.pending_move_learns);
    }
    pokemon.hp = pokemon.max_hp;
    assert!(
        pokemon.moves.iter().any(|m| data.moves[&m.name].power > 0),
        "party member without a damaging move"
    );
}

/// Returns a healed party for badge 1 through 8 or the entire League at stage 9.
/// The starter index follows the Field Station's Grass, Fire and Water choices.
pub(super) fn party(data: &GameDataSet, starter_index: usize, stage: usize) -> Vec<Pokemon> {
    assert!((1..=9).contains(&stage) && starter_index < STARTERS.len());
    let level = LEVELS[stage - 1];
    let dvs = Dv::from_non_hp(8, 8, 8, 8);
    let starter = STARTERS[starter_index];
    let gift = data.maps["MnWinonaFieldStation"]
        .gift_pokemon_scripts
        .iter()
        .find(|g| g.species_id == starter && !g.egg)
        .expect("available Field Station starter");
    let mut lead = data
        .create_pokemon(starter, gift.level, dvs)
        .expect("starter");
    train(data, &mut lead, level);
    let mut team = vec![lead];
    for (index, catch) in CATCHES[starter_index].iter().enumerate() {
        let arrives = match catch.route {
            R1 => 1,
            R2 => 2,
            R3 => 3,
            R6 => 6,
            _ => unreachable!("unscheduled route"),
        };
        if stage < arrives {
            continue;
        }
        let encounters = data.wild_encounters[catch.route]
            .grass
            .as_ref()
            .expect("route grass");
        let slot = &encounters.day[catch.day_slot];
        assert_eq!(
            slot.species, catch.species,
            "daytime slot on {}",
            catch.route
        );
        let mut member = data
            .create_pokemon(catch.species, slot.level, dvs)
            .expect("wild party member");
        data.equip_wild_challenge_moves(&mut member, catch.route)
            .expect("actual capture moves");
        train(data, &mut member, level - [1, 2, 1, 2][index]);
        team.push(member);
    }
    team
}
