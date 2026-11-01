//! Native scroll and pinch camera controls.
use crate::{VoxelViewSettings, VoxelViewStatus};
use bevy::prelude::*;

pub(super) fn register(app: &mut App) {
    app.add_systems(Update, input.in_set(crate::VoxelCameraInput).before(crate::sync_voxel_view));
}

fn input(
    status: Res<VoxelViewStatus>,
    mut settings: ResMut<VoxelViewSettings>,
    scroll: Option<Res<bevy::input::mouse::AccumulatedMouseScroll>>,
    pinch: Option<Res<Messages<bevy::input::gestures::PinchGesture>>>,
    mut pinch_cursor: Local<bevy::ecs::message::MessageCursor<bevy::input::gestures::PinchGesture>>,
) {
    if status.active && settings.enabled {
        if let Some(scroll) = scroll {
            let scale = match scroll.unit {
                bevy::input::mouse::MouseScrollUnit::Line => 0.5,
                bevy::input::mouse::MouseScrollUnit::Pixel => 0.005,
            };
            if scroll.delta.x.is_finite() && scroll.delta.x != 0.0 {
                settings.camera.steer(-scroll.delta.x * scale * 0.2, 0.);
            }
            if scroll.delta.y.is_finite() && scroll.delta.y != 0.0 {
                settings.camera.zoom_step = (settings.camera.zoom_step
                    + (scroll.delta.y * scale).clamp(-1., 1.))
                .clamp(0., 5.);
            }
        }
    }
    if let Some(pinch) = pinch {
        for event in pinch_cursor.read(&pinch) {
            if status.active && settings.enabled && event.0.is_finite() {
                settings.camera.zoom_step =
                    (settings.camera.zoom_step + event.0 * 4.).clamp(0., 5.);
            }
        }
    }
}
