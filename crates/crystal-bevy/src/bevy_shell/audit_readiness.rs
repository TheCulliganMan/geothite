//! Latch the first ready source frame while awaiting its GPU completion.

#[derive(Default)]
pub(super) struct Readiness {
    signature: Option<(String, [i16; 2], bool)>,
    source_frame: u64,
    previous_render_ready: bool,
    stable: usize,
}

impl Readiness {
    pub(super) fn observe(
        &mut self,
        signature: (String, [i16; 2], bool),
        scene_ready: bool,
        frame: u64,
        pipelines_ready: bool,
        completed_frame: u64,
    ) -> bool {
        let render_ready = scene_ready && pipelines_ready;
        if self.signature.as_ref() != Some(&signature)
            || !render_ready
            || !self.previous_render_ready
        {
            self.signature = Some(signature);
            self.source_frame = frame;
            self.stable = 0;
        }
        self.previous_render_ready = render_ready;
        self.stable = if render_ready && completed_frame >= self.source_frame {
            self.stable + 1
        } else {
            0
        };
        self.stable >= 3
    }

    pub(super) fn restart_stability(&mut self) {
        self.stable = 0;
    }

    // A changed result can share its map/tile/battle signature with the old
    // state. Its own extracted frame must complete before accepting a checkpoint.
    pub(super) fn require_frame(&mut self, frame: u64) {
        self.source_frame = self.source_frame.max(frame);
        self.stable = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::Readiness;

    fn signature(map: &str) -> (String, [i16; 2], bool) {
        (map.into(), [1, 2], false)
    }

    #[test]
    fn queued_completion_can_catch_up_to_a_latched_frame() {
        let mut ready = Readiness::default();
        assert!(!ready.observe(signature("field"), true, 1, true, 0));
        assert!(!ready.observe(signature("field"), true, 2, true, 1));
        assert!(!ready.observe(signature("field"), true, 3, true, 2));
        assert!(ready.observe(signature("field"), true, 4, true, 3));
        ready.restart_stability();
        assert!(!ready.observe(signature("field"), true, 5, true, 4));
    }

    #[test]
    fn destination_and_pipeline_changes_require_new_evidence() {
        let mut ready = Readiness::default();
        for frame in 1..=4 {
            ready.observe(signature("field"), true, frame, true, frame);
        }
        assert!(!ready.observe(signature("inside"), true, 5, true, 4));
        assert!(!ready.observe(signature("inside"), true, 6, false, 5));
        assert!(!ready.observe(signature("inside"), false, 7, true, 6));
        assert!(!ready.observe(signature("inside"), true, 8, true, 7));
        assert!(!ready.observe(signature("inside"), true, 9, true, 8));
        assert!(!ready.observe(signature("inside"), true, 10, true, 9));
        assert!(ready.observe(signature("inside"), true, 11, true, 10));
    }

    #[test]
    fn changed_battle_result_cannot_reuse_completion_from_the_command_menu() {
        let mut ready = Readiness::default();
        for frame in 1..=4 {
            ready.observe(signature("battle"), true, frame, true, frame);
        }
        ready.require_frame(10);
        for frame in 11..=15 {
            assert!(!ready.observe(signature("battle"), true, frame, true, 9));
        }
        assert!(!ready.observe(signature("battle"), true, 16, true, 10));
        assert!(!ready.observe(signature("battle"), true, 17, true, 10));
        assert!(ready.observe(signature("battle"), true, 18, true, 10));
    }

    #[test]
    fn newly_ready_pipeline_requires_a_submission_with_that_pipeline() {
        let mut ready = Readiness::default();
        for frame in 1..=4 {
            ready.observe(signature("field"), true, frame, true, frame);
        }
        assert!(!ready.observe(signature("field"), true, 5, false, 4));
        assert!(!ready.observe(signature("field"), true, 6, true, 5));
        for frame in 7..=10 {
            assert!(!ready.observe(signature("field"), true, frame, true, 5));
        }
        assert!(!ready.observe(signature("field"), true, 11, true, 6));
        assert!(!ready.observe(signature("field"), true, 12, true, 6));
        assert!(ready.observe(signature("field"), true, 13, true, 6));
    }
}
