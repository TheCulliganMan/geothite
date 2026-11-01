//! Campaign power bands. These are regression limits, not encounter placement.
use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Result, ensure};
use crystal_assets::GameDataSet;
use crystal_core::battle::damage::MoveCategory;
use serde::Deserialize;

use super::{Family, Profile, read, total};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Policy {
    schema_version: u16,
    ruleset: String,
    gym_ace_levels: Vec<u8>,
    league_levels: Vec<u8>,
    ordinary_base_bst: [u16; 2],
    ordinary_middle_bst: [u16; 2],
    ordinary_final_bst: [u16; 2],
    ordinary_two_stage_evolution: [u8; 2],
    ordinary_three_stage_evolutions: [[u8; 2]; 2],
    starter_bst: [u16; 3],
    starter_evolutions: [u8; 2],
    pseudo_legendary_bst: [u16; 3],
    pseudo_legendary_evolutions: [u8; 2],
    legendary_bst: [u16; 2],
    pub(super) type_access_level: u8,
    pub(super) stat_matched_attack_level: u8,
    early_finishers: BTreeMap<String, EarlyFinisher>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EarlyFinisher {
    bst: [u16; 3],
    evolutions: [u8; 2],
}

#[derive(Deserialize)]
struct Story {
    gyms: Vec<Gym>,
    league_levels: Vec<u8>,
}

#[derive(Deserialize)]
struct Gym {
    party: Vec<TrainerPokemon>,
}

#[derive(Deserialize)]
struct TrainerPokemon {
    level: u8,
}

#[derive(Deserialize)]
struct Split {
    moves: BTreeMap<String, MoveCategory>,
}

impl Policy {
    pub(super) fn load(root: &Path, data: &GameDataSet) -> Result<Self> {
        let policy: Self = read(&root.join("balance-policy.json"))?;
        ensure!(
            policy.schema_version == 1 && policy.ruleset == "modern-move-split",
            "unsupported balance policy"
        );
        let story: Story = read(&root.join("../story.json"))?;
        let aces: Vec<_> = story
            .gyms
            .iter()
            .filter_map(|gym| gym.party.iter().map(|p| p.level).max())
            .collect();
        ensure!(
            aces == policy.gym_ace_levels && story.league_levels == policy.league_levels,
            "campaign level curve changed; review the balance policy"
        );
        ensure!(
            data.type_categories.moves.len() == data.moves.len(),
            "Northwoods requires exhaustive per-move categories"
        );
        let split: Split = serde_json::from_str(include_str!(
            "../../../../../modpacks/modern-move-split/data.json"
        ))?;
        for (id, category) in split.moves {
            ensure!(
                data.type_categories.moves.get(&id) == Some(&category),
                "incorrect physical/special split for {id}"
            );
        }
        Ok(policy)
    }

    pub(super) fn findings(&self, family: &Family, profile: &Profile, stage: usize) -> Vec<String> {
        let mut issues = Vec::new();
        let bst = total(&profile.base_stats);
        let (exact_bst, exact_evolutions, growth) = if family.starter_role.is_some() {
            (
                Some(&self.starter_bst),
                Some(&self.starter_evolutions),
                Some("GROWTH_MEDIUM_SLOW"),
            )
        } else if family.classification.as_deref() == Some("pseudo_legendary") {
            (
                Some(&self.pseudo_legendary_bst),
                Some(&self.pseudo_legendary_evolutions),
                Some("GROWTH_SLOW"),
            )
        } else if let Some(early) = self.early_finishers.get(&family.id) {
            (Some(&early.bst), Some(&early.evolutions), None)
        } else {
            (None, None, None)
        };
        if let Some(totals) = exact_bst {
            if totals[stage] != bst {
                issues.push(format!("role stat budget {}", totals[stage]));
            }
        } else {
            let range = if family.rarity.is_some() {
                self.legendary_bst
            } else if stage + 1 == family.members.len() {
                self.ordinary_final_bst
            } else if stage == 0 {
                self.ordinary_base_bst
            } else {
                self.ordinary_middle_bst
            };
            if !(range[0]..=range[1]).contains(&bst) {
                issues.push(format!("role stat budget {}–{}", range[0], range[1]));
            }
        }
        if let Some(growth) = growth
            && profile.growth_rate != growth
        {
            issues.push(format!("role experience curve {growth}"));
        }
        if let Some(evolution) = &profile.evolution {
            if let Some(levels) = exact_evolutions {
                if levels[stage] != evolution.level {
                    issues.push(format!("role evolution at {}", levels[stage]));
                }
            } else {
                let range = if family.members.len() == 2 {
                    self.ordinary_two_stage_evolution
                } else {
                    self.ordinary_three_stage_evolutions[stage]
                };
                if !(range[0]..=range[1]).contains(&evolution.level) {
                    issues.push(format!("evolution window {}–{}", range[0], range[1]));
                }
            }
        }
        issues
    }
}
