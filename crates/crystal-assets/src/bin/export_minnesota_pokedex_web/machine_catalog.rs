//! Numbered machine compatibility derived from the supplied game's item catalog.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::models::{ITEM_POCKET_TM_HM, Item, Move};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct Machine {
    pub item_id: String,
    pub code: String,
    pub move_id: String,
}

pub(super) struct MachineCatalog {
    pub entries: Vec<Machine>,
}

impl MachineCatalog {
    pub fn load(items: &BTreeMap<String, Item>, moves: &BTreeMap<String, Move>) -> Result<Self> {
        let mut numbered = BTreeMap::new();
        let mut seen_moves = BTreeSet::new();
        for (id, item) in items {
            let Some(index) = item.tmhm_index else {
                continue;
            };
            let move_id = item.tmhm_move.as_deref().context("machine lacks a move")?;
            ensure!(
                item.pocket == ITEM_POCKET_TM_HM && (1..=57).contains(&index),
                "invalid machine item {id}"
            );
            ensure!(
                moves.contains_key(move_id),
                "unknown machine move {move_id}"
            );
            ensure!(
                seen_moves.insert(move_id),
                "duplicate machine move {move_id}"
            );
            let code = code(index);
            ensure!(
                numbered
                    .insert(
                        index,
                        Machine {
                            item_id: id.clone(),
                            code,
                            move_id: move_id.to_owned(),
                        }
                    )
                    .is_none(),
                "duplicate machine number {index}"
            );
        }
        ensure!(numbered.len() == 57, "expected all 50 TMs and seven HMs");
        Ok(Self {
            entries: numbered.into_values().collect(),
        })
    }

    pub fn learnset<'a>(&'a self, compatibility: &[String]) -> Vec<&'a Machine> {
        self.entries
            .iter()
            .filter(|machine| compatibility.contains(&machine.move_id))
            .collect()
    }

    pub fn tutors<'a>(&self, compatibility: &'a [String]) -> Vec<&'a str> {
        compatibility
            .iter()
            .filter(|id| !self.entries.iter().any(|machine| &machine.move_id == *id))
            .map(String::as_str)
            .collect()
    }
}

fn code(index: usize) -> String {
    if index <= 50 {
        format!("TM{index:02}")
    } else {
        format!("HM{:02}", index - 50)
    }
}

pub(super) fn validate_compatibility(
    authored: &[String],
    packed: &[String],
    catalog: &MachineCatalog,
    moves: &BTreeMap<String, Move>,
) -> Result<()> {
    let authored_set: BTreeSet<_> = authored.iter().collect();
    ensure!(
        authored_set.len() == authored.len(),
        "duplicate compatibility move"
    );
    ensure!(
        authored_set == packed.iter().collect(),
        "authored compatibility differs from the game"
    );
    ensure!(
        !catalog.learnset(packed).is_empty(),
        "species has no TM/HM learnset"
    );
    for id in packed {
        ensure!(moves.contains_key(id), "unknown compatibility move {id}");
    }
    for id in catalog.tutors(packed) {
        ensure!(
            matches!(id, "FLAMETHROWER" | "THUNDERBOLT" | "ICE_BEAM"),
            "unrecognized active tutor move {id}"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (MachineCatalog, BTreeMap<String, Move>) {
        let catalog = MachineCatalog {
            entries: vec![
                Machine {
                    item_id: "TM_PROTECT".into(),
                    code: code(17),
                    move_id: "PROTECT".into(),
                },
                Machine {
                    item_id: "HM_SURF".into(),
                    code: code(53),
                    move_id: "SURF".into(),
                },
            ],
        };
        let moves = ["PROTECT", "SURF", "ICE_BEAM", "TACKLE"]
            .into_iter()
            .map(|id| {
                (
                    id.into(),
                    Move {
                        source_index: 1,
                        name: id.into(),
                        move_type: "NORMAL".into(),
                        power: 0,
                        accuracy: 100,
                        pp: 10,
                        effect: "NORMAL_HIT".into(),
                        effect_chance: 0,
                        stat: None,
                        amount: None,
                    },
                )
            })
            .collect();
        (catalog, moves)
    }

    #[test]
    fn machine_numbers_and_tutors_stay_separate_in_catalog_order() {
        assert_eq!(code(1), "TM01");
        assert_eq!(code(50), "TM50");
        assert_eq!(code(51), "HM01");
        assert_eq!(code(57), "HM07");
        let (catalog, moves) = fixture();
        let compatible = vec!["SURF".into(), "ICE_BEAM".into(), "PROTECT".into()];
        validate_compatibility(&compatible, &compatible, &catalog, &moves)
            .expect("valid mixed learnset");
        assert_eq!(
            catalog
                .learnset(&compatible)
                .iter()
                .map(|m| m.code.as_str())
                .collect::<Vec<_>>(),
            ["TM17", "HM03"]
        );
        assert_eq!(catalog.tutors(&compatible), ["ICE_BEAM"]);
    }

    #[test]
    fn stale_empty_duplicate_and_unknown_compatibility_fail_export() {
        let (catalog, moves) = fixture();
        let valid = vec!["PROTECT".into()];
        for invalid in [
            vec![],
            vec!["SURF".into()],
            vec!["PROTECT".into(), "PROTECT".into()],
        ] {
            assert!(validate_compatibility(&invalid, &valid, &catalog, &moves).is_err());
        }
        for invalid in [
            vec![],
            vec!["ICE_BEAM".into()],
            vec!["PROTECT".into(), "TACKLE".into()],
            vec!["PROTECT".into(), "TYPO".into()],
        ] {
            assert!(validate_compatibility(&invalid, &invalid, &catalog, &moves).is_err());
        }
        assert!(MachineCatalog::load(&BTreeMap::new(), &moves).is_err());
    }
}
