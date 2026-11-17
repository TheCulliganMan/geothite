//! Legal, level-bound choices for the paid move reminder.

use std::collections::BTreeSet;

use thiserror::Error;

use crate::models::Pokemon;
use crate::systems::learnsets::{LearnsetError, SpeciesLearnsets, level_up_moves_for_species};

/// Money charged only when a forgotten move is successfully learned.
pub const MOVE_REMINDER_PRICE: u32 = 500;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MoveReminderError {
    #[error(transparent)]
    Learnset(#[from] LearnsetError),
    #[error("an Egg cannot relearn moves")]
    Egg,
    #[error("'{move_id}' is not an eligible forgotten move")]
    IneligibleMove { move_id: String },
    #[error("not enough money for the move reminder")]
    InsufficientMoney,
    #[error("a full move list requires a valid replacement slot")]
    InvalidReplacement,
    #[error("a replacement is unnecessary with fewer than four moves")]
    UnexpectedReplacement,
    #[error("HM move '{move_id}' cannot be forgotten")]
    CannotForgetHm { move_id: String },
}

/// Return unique forgotten moves in the current species' learnset order.
///
/// Only moves at or below the current level qualify; Eggs return no choices.
/// Missing or invalid species learnsets return an error.
pub fn relearnable_moves(
    learnsets: &SpeciesLearnsets,
    pokemon: &Pokemon,
) -> Result<Vec<String>, MoveReminderError> {
    if pokemon.is_egg {
        return Ok(Vec::new());
    }
    let mut seen = BTreeSet::new();
    Ok(level_up_moves_for_species(learnsets, &pokemon.species.id)?
        .iter()
        .filter(|entry| entry.0 <= pokemon.level)
        .filter(|entry| !pokemon.moves.iter().any(|known| known.name == entry.1))
        .filter(|entry| seen.insert(entry.1.as_str()))
        .map(|entry| entry.1.clone())
        .collect())
}

/// Validate a complete choice before any money or move mutation.
///
/// Returns an error for ineligible moves, insufficient funds, malformed move
/// lists, invalid replacement slots or attempts to forget an HM.
pub fn validate_relearning(
    learnsets: &SpeciesLearnsets,
    pokemon: &Pokemon,
    move_id: &str,
    forget_index: Option<usize>,
    hm_moves: &BTreeSet<String>,
    money: u32,
) -> Result<(), MoveReminderError> {
    if pokemon.is_egg {
        return Err(MoveReminderError::Egg);
    }
    if !relearnable_moves(learnsets, pokemon)?
        .iter()
        .any(|name| name == move_id)
    {
        return Err(MoveReminderError::IneligibleMove {
            move_id: move_id.to_owned(),
        });
    }
    if money < MOVE_REMINDER_PRICE {
        return Err(MoveReminderError::InsufficientMoney);
    }
    if pokemon.moves.len() < 4 {
        if forget_index.is_some() {
            return Err(MoveReminderError::UnexpectedReplacement);
        }
    } else {
        if pokemon.moves.len() != 4 {
            return Err(MoveReminderError::InvalidReplacement);
        }
        let existing = forget_index
            .and_then(|index| pokemon.moves.get(index))
            .ok_or(MoveReminderError::InvalidReplacement)?;
        if hm_moves.contains(&existing.name) {
            return Err(MoveReminderError::CannotForgetHm {
                move_id: existing.name.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BaseStats, Dv, LearnedMove, PokemonSpecies};
    use crate::systems::learnsets::LearnsetEntry;

    fn subject() -> (SpeciesLearnsets, Pokemon) {
        let species =
            PokemonSpecies::new_for_tests("TESTMON", BaseStats::new(50, 50, 50, 50, 50, 50));
        let mut pokemon = Pokemon::new_for_tests(species, 12, Dv::default());
        pokemon.moves = vec![LearnedMove {
            name: "TACKLE".into(),
            current_pp: 3,
            pp_ups: 1,
        }];
        let learnsets = [(
            "TESTMON".into(),
            vec![
                LearnsetEntry(1, "TACKLE".into()),
                LearnsetEntry(5, "GROWL".into()),
                LearnsetEntry(12, "GROWL".into()),
                LearnsetEntry(13, "BITE".into()),
            ],
        )]
        .into();
        (learnsets, pokemon)
    }

    #[test]
    fn only_forgotten_current_level_moves_are_offered_once() {
        let (learnsets, mut pokemon) = subject();
        assert_eq!(
            relearnable_moves(&learnsets, &pokemon).expect("eligible moves"),
            ["GROWL"]
        );
        pokemon.is_egg = true;
        assert!(
            relearnable_moves(&learnsets, &pokemon)
                .expect("Egg choices")
                .is_empty()
        );
    }

    #[test]
    fn rejects_future_foreign_known_moves_and_insufficient_funds() {
        let (learnsets, pokemon) = subject();
        for name in ["BITE", "SURF", "TACKLE"] {
            assert!(matches!(
                validate_relearning(&learnsets, &pokemon, name, None, &BTreeSet::new(), 500),
                Err(MoveReminderError::IneligibleMove { .. })
            ));
        }
        assert_eq!(
            validate_relearning(&learnsets, &pokemon, "GROWL", None, &BTreeSet::new(), 499),
            Err(MoveReminderError::InsufficientMoney)
        );
        assert!(
            validate_relearning(&learnsets, &pokemon, "GROWL", None, &BTreeSet::new(), 500).is_ok()
        );
    }

    #[test]
    fn replacement_requires_four_moves_and_preserves_hms() {
        let (learnsets, mut pokemon) = subject();
        let hms = ["SURF".to_owned()].into();
        assert_eq!(
            validate_relearning(&learnsets, &pokemon, "GROWL", Some(0), &hms, 500),
            Err(MoveReminderError::UnexpectedReplacement)
        );
        for name in ["SURF", "CUT", "FLASH"] {
            pokemon.moves.push(LearnedMove {
                name: name.into(),
                current_pp: 5,
                pp_ups: 0,
            });
        }
        for slot in [None, Some(4)] {
            assert_eq!(
                validate_relearning(&learnsets, &pokemon, "GROWL", slot, &hms, 500),
                Err(MoveReminderError::InvalidReplacement)
            );
        }
        assert!(matches!(
            validate_relearning(&learnsets, &pokemon, "GROWL", Some(1), &hms, 500),
            Err(MoveReminderError::CannotForgetHm { .. })
        ));
        assert!(validate_relearning(&learnsets, &pokemon, "GROWL", Some(0), &hms, 500).is_ok());
    }
}
