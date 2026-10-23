//! Keep recently traversed windows ready for a reversal of travel.
use std::collections::VecDeque;

use bevy::prelude::{AssetId, Commands, Image};
use crystal_render_api::VisualWorldFrame;

use super::{cached_compatible, hide, roots};
use crate::{TerrainCacheKey, TerrainRevisionCache, live_profiles, loading_trace, world_stream};

// Four complete windows cover a recent return journey while the independent
// forward lane builds. This bounds live GPU residency, not audit artifact size.
const RECENT_WINDOWS: usize = 4;

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
        while self.windows.len() > RECENT_WINDOWS {
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
