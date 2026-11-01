//! Keep recently traversed windows ready for a reversal of travel.
use std::collections::VecDeque;

use bevy::prelude::{AssetId, Commands, Image};
use crystal_render_api::VisualWorldFrame;

use super::{cached_compatible, hide, roots};
use crate::{TerrainCacheKey, TerrainRevisionCache, live_profiles, loading_trace, world_stream};

// Four complete windows cover a recent return journey while the independent
// forward lane builds. This bounds live GPU residency, not audit artifact size.
const RECENT_WINDOWS: usize = 4;
// Expanded windows already cover a long return journey. Keep their aggregate
// source acreage within the old four-window budget instead of multiplying GPU
// residency by the new view distance. Always retain the most recent window.
const RECENT_SOURCE_CELLS: u64 = 4 * 120 * 120;

type Compatibility = Option<(TerrainCacheKey, AssetId<Image>, bool)>;

struct Window {
    cache: TerrainRevisionCache,
    compatibility: Compatibility,
}

#[derive(Default)]
pub(super) struct RetainedWindows {
    windows: VecDeque<Window>,
}

impl RetainedWindows {
    pub(super) fn take_ready(
        &mut self,
        resident: &VisualWorldFrame,
        live: &VisualWorldFrame,
        profiles: &live_profiles::LiveProfiles,
    ) -> Option<TerrainRevisionCache> {
        let selected = self
            .windows
            .iter_mut()
            .enumerate()
            .filter_map(|(index, window)| {
                if !window
                    .cache
                    .key
                    .as_ref()
                    .is_some_and(|key| key.profiles_revision == profiles.revision)
                {
                    return None;
                }
                let built = window.cache.built_frame.as_ref()?;
                (world_stream::advances_coverage(resident, built, live)
                    && cached_compatible(built, live, &mut window.compatibility))
                .then_some((index, distance(built, live)?))
            })
            .min_by_key(|(_, distance)| *distance)?
            .0;
        self.windows.remove(selected).map(|window| window.cache)
    }

    pub(super) fn remember(
        &mut self,
        cache: TerrainRevisionCache,
        reason: &str,
        commands: &mut Commands,
    ) {
        let _trace = loading_trace::span("world.window_retain", || {
            serde_json::json!({
                "map": cache.built_frame.as_ref().map(|frame| frame.map_id.as_ref()),
                "origin": cache.built_frame.as_ref().map(|frame| frame.grid_origin.to_array()),
                "roots": roots(&cache).map(|entity| entity.to_bits()).collect::<Vec<_>>(),
                "reason": reason,
            })
        });
        hide(&cache, commands);
        self.windows.push_back(Window {
            cache,
            compatibility: None,
        });
        while self.windows.len() > RECENT_WINDOWS
            || (self.windows.len() > 1
                && self
                    .windows
                    .iter()
                    .filter_map(|window| window.cache.built_frame.as_ref())
                    .map(|frame| u64::from(frame.grid_size.x) * u64::from(frame.grid_size.y))
                    .sum::<u64>()
                    > RECENT_SOURCE_CELLS)
        {
            if let Some(window) = self.windows.pop_front() {
                for entity in roots(&window.cache) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn distance(built: &VisualWorldFrame, live: &VisualWorldFrame) -> Option<i32> {
    built.connected_source_shift(live).map(|shift| {
        (live.grid_origin - built.grid_origin - shift)
            .abs()
            .max_element()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Resource, Default)]
    struct History(RetainedWindows);

    #[test]
    fn larger_windows_keep_the_return_cache_within_its_original_acreage_budget() {
        for (size, expected) in [(120, 4), (216, 1), (512, 1)] {
            let mut app = App::new();
            app.init_resource::<History>().add_systems(
                Update,
                move |mut history: ResMut<History>,
                      mut count: Local<i32>,
                      mut commands: Commands| {
                    history.0.remember(
                        TerrainRevisionCache {
                            built_frame: Some(VisualWorldFrame {
                                grid_size: UVec2::splat(size),
                                grid_origin: IVec2::new(*count, 0),
                                ..default()
                            }),
                            ..default()
                        },
                        "coverage test",
                        &mut commands,
                    );
                    *count += 1;
                },
            );
            for _ in 0..6 {
                app.update();
            }
            let windows = &app.world().resource::<History>().0.windows;
            assert_eq!(windows.len(), expected);
            assert_eq!(
                windows
                    .back()
                    .expect("return window")
                    .cache
                    .built_frame
                    .as_ref()
                    .expect("source coverage")
                    .grid_origin
                    .x,
                5
            );
        }
    }
}
