//! Preserve native move rejection sampling while surfacing failed input.
use std::collections::BTreeSet;

use anyhow::{Result, bail, ensure};
use crystal_core::random::BattleRandomSource;

pub(crate) fn select_scored_move_slot(
    best: &BTreeSet<usize>,
    random: &mut dyn BattleRandomSource,
) -> Result<usize> {
    ensure!(
        !best.is_empty() && best.iter().all(|slot| *slot < 4),
        "trainer AI has no valid scored move slot"
    );
    loop {
        let slot = usize::from(random.battle_random_byte() & 3);
        if let Some(error) = random.source_error() {
            bail!("trainer AI random source failed: {error}");
        }
        if best.contains(&slot) {
            return Ok(slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, VecDeque};

    use crystal_core::random::{
        BattleRandomSource, CrystalRandomState, ExactBattleRandom, ReplayDivider,
    };

    use super::select_scored_move_slot;

    #[test]
    fn exhausted_replay_is_rejected_even_when_placeholder_matches_a_best_slot() {
        for best in [0, 3] {
            let mut divider = ReplayDivider::new(std::iter::empty::<u8>());
            let mut random = ExactBattleRandom::new(CrystalRandomState::default(), &mut divider);
            let error = select_scored_move_slot(&BTreeSet::from([best]), &mut random)
                .expect_err("finite replay exhaustion is recoverable");
            assert!(error.to_string().contains("divider replay exhausted"));
        }
    }

    #[test]
    fn valid_rejection_sampling_preserves_draw_order_and_remaining_bytes() {
        struct Bytes(VecDeque<u8>);
        impl BattleRandomSource for Bytes {
            fn battle_random_byte(&mut self) -> u8 {
                self.0.pop_front().expect("declared native random draws")
            }
        }
        let mut random = Bytes(VecDeque::from([0, 2, 1, 99]));
        assert_eq!(
            select_scored_move_slot(&BTreeSet::from([1]), &mut random)
                .expect("eligible native slot"),
            1
        );
        assert_eq!(random.0, VecDeque::from([99]));
    }
}
