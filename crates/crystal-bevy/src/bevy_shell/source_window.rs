//! Retain the wide source atlas while the smaller LCD viewport scrolls.
use super::VisualWorldWindow;

// Keep this below the resident halo's 32-cell movement reserve. The renderer
// retains another 16 cells while a replacement window finishes publication.
const SCROLL_RESERVE: i32 = 16;

pub(super) fn select(
    previous: Option<VisualWorldWindow>,
    desired: VisualWorldWindow,
    same_content: bool,
) -> (VisualWorldWindow, bool) {
    if same_content
        && desired.size.min_element() >= 192
        && let Some(previous) = previous
        && previous.size == desired.size
        && (previous.origin - desired.origin).abs().max_element() < SCROLL_RESERVE
    {
        (previous, true)
    } else {
        (desired, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{IVec2, UVec2};

    #[test]
    fn scrolling_reuses_the_wide_window_until_its_guard_band_is_used() {
        let previous = VisualWorldWindow {
            origin: IVec2::new(-96, -96),
            size: UVec2::splat(216),
        };
        for direction in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y, IVec2::ONE] {
            for steps in [0, 4, 8, 12] {
                let desired = VisualWorldWindow {
                    origin: previous.origin + direction * steps,
                    ..previous
                };
                assert_eq!(select(Some(previous), desired, true), (previous, true));
                assert_eq!(select(Some(previous), desired, false), (desired, false));
            }
            let desired = VisualWorldWindow {
                origin: previous.origin + direction * 16,
                ..previous
            };
            assert_eq!(select(Some(previous), desired, true), (desired, false));
        }
        let room = VisualWorldWindow {
            size: UVec2::splat(120),
            ..previous
        };
        assert_eq!(select(Some(room), room, true), (room, false));
        assert_eq!(select(None, previous, true), (previous, false));
    }
}
