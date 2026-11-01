//! Continuous input must not wait for the work whose latency it measures.
use super::FrameSample;

pub(super) fn ready(frame: &FrameSample, continuous: bool, rendered: bool) -> bool {
    if !continuous {
        return rendered;
    }
    !frame.battle
        && !frame.dialogue
        && frame.warp_phase == "None"
        && frame.walk_remaining == 0
        && frame.natural_offset.iter().all(|offset| offset.is_finite())
}

#[cfg(test)]
mod tests {
    use super::{FrameSample, ready};

    #[test]
    fn continuous_reversal_preserves_fractional_movement_without_renderer_wait() {
        let mut frame = FrameSample {
            warp_phase: "None".into(),
            natural_offset: [4., 0.],
            terrain_pending: true,
            ..Default::default()
        };
        assert!(ready(&frame, true, false));
        frame.natural_offset = [0., 0.];
        assert!(ready(&frame, true, false));
        assert!(!ready(&frame, false, false));
        assert!(ready(&frame, false, true));
        frame.walk_remaining = 1;
        assert!(!ready(&frame, true, true));
        frame.walk_remaining = 0;
        frame.warp_phase = "FadeOut".into();
        assert!(!ready(&frame, true, true));
    }

    #[test]
    fn dialogue_battle_or_invalid_motion_cannot_complete_a_movement_leg() {
        let mut frame = FrameSample {
            warp_phase: "None".into(),
            ..Default::default()
        };
        for offset in [f32::NAN, f32::INFINITY] {
            frame.natural_offset = [offset, 0.];
            assert!(!ready(&frame, true, true));
        }
        frame.natural_offset = [0., 0.];
        frame.dialogue = true;
        assert!(!ready(&frame, true, true));
        frame.dialogue = false;
        frame.battle = true;
        assert!(!ready(&frame, true, true));
    }
}
