//! An exhausted deterministic divider must fail rather than spin in AI sampling.
use crate::battle::ai::trainer_ai_damage;
use crate::battle::damage::{
    DamageCalculationError, TypeCategories, TypeEffectivenessTable, WeatherModifiers,
};
use crate::battle::stats::BattleStatMultiplierTables;
use crate::battle::turn::BattleCombatState;
use crate::models::{BaseStats, Dv, Move, Pokemon, PokemonSpecies, pokemon_type};
use crate::random::{CrystalRandomState, ExactBattleRandom, ReplayDivider};

#[test]
fn ai_psywave_exhausted_divider_returns_error_without_spinning() {
    let species = PokemonSpecies::new_for_tests("PARTNER", BaseStats::new(40, 40, 40, 40, 40, 40));
    let player = Pokemon::new_for_tests(species.clone(), 20, Dv::default());
    let enemy = Pokemon::new_for_tests(species, 20, Dv::default());
    let state = BattleCombatState::new(player, enemy);
    let movement = Move {
        source_index: 149,
        name: "PSYWAVE".into(),
        move_type: pokemon_type("PSYCHIC"),
        power: 1,
        accuracy: 80,
        pp: 15,
        effect: "PSYWAVE".into(),
        effect_chance: 0,
        stat: None,
        amount: None,
    };
    let mut divider = ReplayDivider::new(std::iter::empty::<u8>());
    let mut random = ExactBattleRandom::new(CrystalRandomState::default(), &mut divider);
    let result = trainer_ai_damage(
        &state,
        &movement,
        &BattleStatMultiplierTables::default(),
        &TypeCategories::default(),
        &TypeEffectivenessTable::default(),
        &WeatherModifiers::default(),
        0,
        &mut random,
    );
    let Err(DamageCalculationError::RandomSource { error }) = result else {
        panic!("exhausted timing input must reject the damage estimate");
    };
    assert!(error.contains("divider replay exhausted"));
    assert!(random.divider_error().is_some());
}
