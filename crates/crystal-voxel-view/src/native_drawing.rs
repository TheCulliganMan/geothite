//! Native-only direct-drawing comparison. Registering the requirement during
//! plugin construction attaches it at camera spawn, as required by Bevy.
use bevy::{prelude::*, render::view::NoIndirectDrawing};

pub(super) fn register(app: &mut App) {
    let mode = std::env::var("CRYSTAL_NATIVE_DRAWING").ok();
    configure(app, mode.as_deref());
}

fn configure(app: &mut App, mode: Option<&str>) {
    if mode == Some("direct") {
        app.register_required_components::<Camera3d, NoIndirectDrawing>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_is_attached_at_initial_spawn_to_every_3d_camera() {
        let mut app = App::new();
        configure(&mut app, Some("direct"));
        // World, battle, row, and grass-mask cameras all originate in Camera3d.
        for _ in 0..4 {
            let camera = app.world_mut().spawn(Camera3d::default()).id();
            assert!(app.world().get::<NoIndirectDrawing>(camera).is_some());
            assert!(app.world().get::<Camera3d>(camera).is_some());
            assert!(app.world().get::<Camera>(camera).is_some());
        }
        let camera_2d = app.world_mut().spawn(Camera2d).id();
        assert!(app.world().get::<NoIndirectDrawing>(camera_2d).is_none());
    }

    #[test]
    fn unset_and_unknown_preserve_indirect_camera_spawn() {
        for mode in [None, Some("indirect"), Some("DIRECT"), Some(""), Some("unknown")] {
            let mut app = App::new();
            configure(&mut app, mode);
            let camera = app.world_mut().spawn(Camera3d::default()).id();
            assert!(app.world().get::<NoIndirectDrawing>(camera).is_none());
        }
    }
}
