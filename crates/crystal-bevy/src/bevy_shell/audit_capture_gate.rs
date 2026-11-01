//! Keep native profiler attachment and its first submissions outside measurement.

pub(super) enum Action {
    PublishReady,
    Wait,
    Measure,
}

#[derive(Clone, Copy)]
enum Phase {
    Unannounced,
    Waiting,
    Draining(u8),
    Running,
}

pub(super) struct CaptureGate(Phase);

impl CaptureGate {
    pub(super) fn new(enabled: bool) -> Self {
        Self(if enabled {
            Phase::Unannounced
        } else {
            Phase::Running
        })
    }

    // Called only after image validation and ordinary renderer readiness.
    pub(super) fn advance(&mut self, started: bool) -> Action {
        match self.0 {
            Phase::Unannounced => {
                self.0 = Phase::Waiting;
                Action::PublishReady
            }
            Phase::Waiting => {
                if started {
                    self.0 = Phase::Draining(30);
                }
                Action::Wait
            }
            Phase::Draining(remaining) => {
                if remaining > 1 {
                    self.0 = Phase::Draining(remaining - 1);
                    Action::Wait
                } else {
                    self.0 = Phase::Running;
                    Action::Measure
                }
            }
            Phase::Running => Action::Measure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, CaptureGate};

    #[test]
    fn attachment_requires_a_start_signal_and_thirty_ready_submissions() {
        let mut gate = CaptureGate::new(true);
        assert!(matches!(gate.advance(false), Action::PublishReady));
        for _ in 0..100 {
            assert!(matches!(gate.advance(false), Action::Wait));
        }
        assert!(matches!(gate.advance(true), Action::Wait));
        for _ in 0..29 {
            assert!(matches!(gate.advance(true), Action::Wait));
        }
        assert!(matches!(gate.advance(true), Action::Measure));
    }

    #[test]
    fn cpu_controls_do_not_wait_for_an_external_profiler() {
        assert!(matches!(
            CaptureGate::new(false).advance(false),
            Action::Measure
        ));
    }
}
