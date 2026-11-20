//! Northstar's source-movie opt-in and presentation-only title clock.
use crystal_runtime::CrystalRuntime;

pub(super) const MARKER: &str = "data/title/northstar-opening-v1.json";
pub(super) const REVEAL_FRAMES: u32 = 180;
pub(super) const LOOP_FRAMES: u32 = 480;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct Opening {
    pub(super) frame: u32,
    pub(super) reduced_motion: bool,
}

impl Opening {
    pub(super) fn new(reduced_motion: bool) -> Self {
        Self {
            frame: 0,
            reduced_motion,
        }
    }

    /// Advances only the title artwork; the original movie owns its own clock.
    pub(super) fn tick(&mut self) {
        if !self.reduced_motion {
            self.frame += 1;
            if self.frame == REVEAL_FRAMES + LOOP_FRAMES {
                self.frame = REVEAL_FRAMES;
            }
        }
    }

    pub(super) fn image_frame(&self) -> u32 {
        if self.reduced_motion {
            return REVEAL_FRAMES;
        }
        // The authored pixel animation is 15fps; input still samples every frame.
        self.frame / 4 * 4
    }

    /// Shows the finished composition when the player opens the menu early.
    pub(super) fn finish_reveal(&mut self) {
        if !self.reduced_motion {
            self.frame = self.frame.max(REVEAL_FRAMES);
        }
    }
}

pub(super) fn enabled(runtime: &CrystalRuntime) -> bool {
    runtime
        .data()
        .runtime_title_screen
        .static_title_image
        .as_deref()
        == Some("gfx/title/northstar.png")
        && runtime.runtime_file(MARKER).is_some()
}

pub(super) fn prefers_reduced_motion() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var("GEOTHITE_REDUCED_MOTION").is_ok_and(|value| value == "1")
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| {
                window
                    .match_media("(prefers-reduced-motion: reduce)")
                    .ok()
                    .flatten()
            })
            .is_some_and(|query| query.matches())
    }
}

#[cfg(test)]
mod tests {
    use super::{LOOP_FRAMES, Opening, REVEAL_FRAMES};

    #[test]
    fn title_cycles_without_advancing_any_movie() {
        let mut opening = Opening::new(false);
        for expected in 1..=4800 {
            opening.tick();
            let frame = if expected < REVEAL_FRAMES {
                expected
            } else {
                REVEAL_FRAMES + (expected - REVEAL_FRAMES) % LOOP_FRAMES
            };
            assert_eq!(opening.frame, frame);
        }
    }

    #[test]
    fn reduced_motion_keeps_the_title_still_and_ready() {
        let mut still = Opening::new(true);
        for _ in 0..1800 {
            still.tick();
        }
        assert_eq!(still.frame, 0);
        assert_eq!(still.image_frame(), REVEAL_FRAMES);
    }
}
