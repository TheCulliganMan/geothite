use crystal_core::systems::script_blocks::{ScriptBlockChange, CHANGE_BLOCK_COORD_STRIDE};
use std::collections::{BTreeMap, BTreeSet};

type Edits = BTreeMap<(usize, usize), u16>;

pub(super) struct VariantPlan {
    pub states: Vec<Edits>,
    pub coverage: &'static str,
    pub possible_combinations: u64,
    pub binary_choices: bool,
}

fn position(change: &ScriptBlockChange) -> (usize, usize) {
    (usize::from(change.x / CHANGE_BLOCK_COORD_STRIDE) * 4,
     usize::from(change.y / CHANGE_BLOCK_COORD_STRIDE) * 4)
}

/// Geometry fixtures only. Neither enumeration strategy proves script reachability.
pub(super) fn plan(changes: &[ScriptBlockChange]) -> VariantPlan {
    let mut alternatives = BTreeMap::<_, BTreeSet<_>>::new();
    for change in changes {
        alternatives.entry(position(change)).or_default().insert(change.block_id);
    }
    let combinations = alternatives.values().fold(1u64, |n, values|
        n.saturating_mul(values.len() as u64 + 1)).saturating_sub(1);
    let binary_choices = alternatives.values().all(|values| values.len() == 1);
    // Retain every formerly supported binary combination, and enumerate all
    // alternatives for small rooms. A switch room's millions of Cartesian
    // products include contradictory/unreachable door configurations.
    if combinations <= 255 {
        let alternatives: Vec<_> = alternatives.into_iter()
            .map(|(xy, values)| (xy, values.into_iter().collect::<Vec<_>>())).collect();
        let states = (1..=combinations).map(|mut ordinal| {
            let mut state = Edits::new();
            for (xy, values) in &alternatives {
                let radix = values.len() as u64 + 1;
                let choice = (ordinal % radix) as usize;
                ordinal /= radix;
                if choice != 0 { state.insert(*xy, values[choice - 1]); }
            }
            state
        }).collect();
        return VariantPlan { states, coverage: "all-declared-alternative-combinations",
            possible_combinations: combinations, binary_choices };
    }
    // Inspect each declared script's ordered prefixes, preserving replacements
    // at the same coordinate. Explicit coverage metadata prevents confusing
    // this bounded geometry review with exhaustive or reachable-state coverage.
    let mut scripts = BTreeMap::<_, Vec<_>>::new();
    for change in changes { scripts.entry(&change.source_script).or_default().push(change); }
    let mut states = BTreeSet::new();
    for commands in scripts.values_mut() {
        commands.sort_by_key(|change| change.command_index);
        let mut state = Edits::new();
        for change in commands {
            state.insert(position(change), change.block_id);
            states.insert(state.clone());
        }
    }
    VariantPlan { states: states.into_iter().collect(), coverage: "declared-script-prefixes",
        possible_combinations: combinations, binary_choices }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn change(x: u16, block_id: u16, script: &str, command_index: usize) -> ScriptBlockChange {
        ScriptBlockChange { x, y: 0, block_id, source_script: script.into(), command_index }
    }
    #[test]
    fn alternatives_at_one_coordinate_are_separate_choices_and_binary_masks_survive() {
        let plan = plan(&[change(0, 1, "a", 0), change(0, 2, "b", 0), change(2, 3, "a", 1)]);
        assert_eq!(plan.states.len(), 5);
        assert!(plan.states.contains(&BTreeMap::from([((0, 0), 2), ((4, 0), 3)])));
        let binary = super::plan(&[change(0, 1, "a", 0), change(2, 3, "a", 1)]);
        assert!(binary.binary_choices);
        assert_eq!(binary.states, vec![BTreeMap::from([((0,0),1)]),
            BTreeMap::from([((4,0),3)]), BTreeMap::from([((0,0),1),((4,0),3)])]);
    }
    #[test]
    fn large_plan_covers_every_declared_alternative_and_preserves_script_order() {
        let mut changes = Vec::new();
        for x in 0..16 { for block in 1..=2 {
            changes.push(change(x * 2, block, &format!("door{x}"), (2 - block) as usize));
        }}
        let plan = plan(&changes);
        assert_eq!(plan.coverage, "declared-script-prefixes");
        assert_eq!(plan.possible_combinations, 43_046_720);
        assert_eq!(plan.states.len(), 32);
        for c in changes {
            assert!(plan.states.iter().any(|state| state.get(&position(&c)) == Some(&c.block_id)));
        }
    }
}
