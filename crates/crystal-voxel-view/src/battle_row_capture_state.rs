//! Render receipts, independent of Bevy so lifecycle/race tests need no GPU.
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Debug, Default, Clone, Copy)]
struct Outcome {
    revision: u64,
    attempt: u64,
    success: bool,
}

#[derive(Debug, Default)]
struct Receipt {
    // One bounded outcome; no per-frame queue and no GPU wait.
    outcome: Mutex<Outcome>,
}

#[derive(Clone, Debug)]
pub(super) struct CaptureTicket {
    revision: u64,
    attempt: u64,
    // Per attempted render frame, never inherited from an earlier attempt.
    drawn: Arc<AtomicBool>,
    output: Arc<AtomicBool>,
    receipt: Arc<Receipt>,
}

impl CaptureTicket {
    /// Called only after the ordinary actor draw commands all succeed.
    pub(super) fn mark_drawn(&self) {
        self.drawn.store(true, Ordering::Release);
    }

    /// Called after this view's verified output blit runs in the render graph.
    pub(super) fn mark_output(&self) {
        self.output.store(true, Ordering::Release);
    }

    /// Called only after the frame containing that draw and output blit submits.
    pub(super) fn mark_submitted(&self) {
        let mut outcome = self
            .receipt
            .outcome
            .lock()
            .expect("capture receipt poisoned");
        if (self.revision, self.attempt) > (outcome.revision, outcome.attempt) {
            *outcome = Outcome {
                revision: self.revision,
                attempt: self.attempt,
                success: self.drawn.load(Ordering::Acquire) && self.output.load(Ordering::Acquire),
            };
        }
    }
}

pub(super) struct CaptureCache<K> {
    key: Option<K>,
    revision: u64,
    attempt: u64,
    receipt: Arc<Receipt>,
}

impl<K> Default for CaptureCache<K> {
    fn default() -> Self {
        Self {
            key: None,
            revision: 0,
            attempt: 0,
            receipt: Arc::default(),
        }
    }
}

impl<K: PartialEq> CaptureCache<K> {
    pub(super) fn invalidate(&mut self) {
        self.key = None;
    }

    /// None stops issuing draws after a successful output. Any later already
    /// issued attempt drains naturally; `settled` proves the final writer.
    /// Missing readiness never expires into success after a fixed frame count.
    pub(super) fn request(&mut self, key: K) -> Option<CaptureTicket> {
        if self.key.as_ref() != Some(&key) {
            self.revision = self
                .revision
                .checked_add(1)
                .expect("capture revision overflow");
            self.key = Some(key);
        }
        let outcome = *self
            .receipt
            .outcome
            .lock()
            .expect("capture receipt poisoned");
        if outcome.revision == self.revision && outcome.success {
            return None;
        }
        self.attempt = self
            .attempt
            .checked_add(1)
            .expect("capture attempt overflow");
        Some(CaptureTicket {
            revision: self.revision,
            attempt: self.attempt,
            drawn: Arc::default(),
            output: Arc::default(),
            receipt: self.receipt.clone(),
        })
    }

    pub(super) fn settled(&self) -> bool {
        let outcome = self
            .receipt
            .outcome
            .lock()
            .expect("capture receipt poisoned");
        self.key.is_some()
            && outcome.revision == self.revision
            && outcome.attempt == self.attempt
            && outcome.success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_inputs_need_a_draw_and_submission_not_elapsed_frames() {
        let mut cache = CaptureCache::default();
        for _ in 0..1000 {
            cache.request(42).unwrap().mark_submitted();
        }
        let ticket = cache.request(42).unwrap();
        ticket.mark_drawn();
        ticket.mark_output();
        assert!(cache.request(42).is_some());
        ticket.mark_submitted();
        assert!(cache.request(42).is_none());
    }

    #[test]
    fn dirty_generation_ignores_a_late_old_render() {
        let mut cache = CaptureCache::default();
        let old = cache.request("old palette").unwrap();
        let new = cache.request("new palette").unwrap();
        old.mark_drawn();
        old.mark_output();
        old.mark_submitted();
        assert!(cache.request("new palette").is_some());
        new.mark_drawn();
        new.mark_output();
        new.mark_submitted();
        old.mark_drawn();
        old.mark_output();
        old.mark_submitted();
        assert!(cache.request("new palette").is_none());
    }

    #[test]
    fn a_draw_without_submission_cannot_certify_a_later_blank_attempt() {
        let mut cache = CaptureCache::default();
        let failed_output = cache.request(4).unwrap();
        failed_output.mark_drawn();
        failed_output.mark_output();
        let next_frame = cache.request(4).unwrap();
        next_frame.mark_submitted();
        assert!(cache.request(4).is_some());
        let next_frame = cache.request(4).unwrap();
        next_frame.mark_drawn();
        next_frame.mark_output();
        next_frame.mark_submitted();
        assert!(cache.request(4).is_none());
    }

    #[test]
    fn same_handle_resize_and_in_place_assets_require_a_new_render() {
        let mut cache = CaptureCache::default();
        let original = cache.request(7).unwrap();
        original.mark_drawn();
        original.mark_output();
        original.mark_submitted();
        for _ in 0..4 {
            cache.invalidate();
            original.mark_submitted();
            let next = cache.request(7).unwrap();
            assert_ne!(next.revision, original.revision);
            next.mark_submitted();
            assert!(cache.request(7).is_some());
            let next = cache.request(7).unwrap();
            next.mark_drawn();
            next.mark_output();
            next.mark_submitted();
            assert!(cache.request(7).is_none());
        }
    }

    #[test]
    fn f3_interruption_and_phase_reentry_cannot_reuse_an_old_receipt() {
        let mut cache = CaptureCache::default();
        for _ in 0..20 {
            let ticket = cache.request("same held source frame").unwrap();
            ticket.mark_drawn();
            ticket.mark_output();
            ticket.mark_submitted();
            assert!(cache.request("same held source frame").is_none());
            cache.invalidate();
        }
    }

    #[test]
    fn actors_have_independent_dirty_generations() {
        let mut actors = [CaptureCache::default(), CaptureCache::default()];
        for cache in &mut actors {
            let ticket = cache.request(1).unwrap();
            ticket.mark_drawn();
            ticket.mark_output();
            ticket.mark_submitted();
        }
        assert!(actors[0].request(2).is_some());
        assert!(actors[1].request(1).is_none());
    }

    #[test]
    fn pose_camera_material_and_viewport_each_invalidate() {
        let mut cache = CaptureCache::default();
        let mut key = [0; 6];
        for index in 0..6 {
            let ticket = cache.request(key).unwrap();
            ticket.mark_drawn();
            ticket.mark_output();
            ticket.mark_submitted();
            assert!(cache.request(key).is_none());
            key[index] += 1;
        }
        assert!(cache.request(key).is_some());
    }

    #[test]
    fn draw_without_output_and_output_without_draw_cannot_certify_a_target() {
        let mut cache = CaptureCache::default();
        let draw_only = cache.request(1).unwrap();
        draw_only.mark_drawn();
        draw_only.mark_submitted();
        assert!(!cache.settled());
        let output_only = cache.request(1).unwrap();
        output_only.mark_output();
        output_only.mark_submitted();
        assert!(!cache.settled());
        assert!(cache.request(1).is_some());
    }

    #[test]
    fn a_later_failed_writer_revokes_success_without_accepting_a_late_old_success() {
        let mut cache = CaptureCache::default();
        let first = cache.request(1).unwrap();
        let later = cache.request(1).unwrap();
        first.mark_drawn();
        first.mark_output();
        first.mark_submitted();
        assert!(
            cache.request(1).is_none(),
            "stop issuing while later attempt drains"
        );
        assert!(!cache.settled());
        later.mark_output(); // The target clears, but the actor pipeline fails.
        later.mark_submitted();
        first.mark_submitted(); // Cannot overwrite the newer failed writer.
        assert!(cache.request(1).is_some());
        assert!(!cache.settled());
        let recovered = cache.request(1).unwrap();
        recovered.mark_drawn();
        recovered.mark_output();
        recovered.mark_submitted();
        assert!(cache.settled());
        assert!(cache.request(1).is_none());
    }

    #[test]
    fn a_late_earlier_failure_cannot_revoke_the_final_successful_writer() {
        let mut cache = CaptureCache::default();
        let earlier = cache.request(1).unwrap();
        let final_writer = cache.request(1).unwrap();
        final_writer.mark_drawn();
        final_writer.mark_output();
        final_writer.mark_submitted();
        earlier.mark_submitted();
        assert!(cache.settled());
        assert!(cache.request(1).is_none());
    }
}
