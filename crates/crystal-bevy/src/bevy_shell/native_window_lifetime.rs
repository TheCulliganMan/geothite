//! Keep native window destruction on main after pipelined rendering shuts down.
use anyhow::{Context, Result};
use bevy::{
    app::App,
    prelude::With,
    window::{PrimaryWindow, RawHandleWrapperHolder},
};

/// WindowPlugin installs this holder before Winit creates the actual window.
/// Winit fills the shared holder with its owning RawHandleWrapper. Keeping the
/// existing holder outside App::run prevents the renderer from dropping the
/// last native-window Arc while main waits for RenderAppChannels to return.
pub(super) fn retain_primary(app: &mut App) -> Result<RawHandleWrapperHolder> {
    let mut windows = app
        .world_mut()
        .query_filtered::<&RawHandleWrapperHolder, With<PrimaryWindow>>();
    windows
        .single(app.world())
        .cloned()
        .context("retain the existing primary window handle through native renderer shutdown")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{app::AppExit, window::WindowPlugin};
    use std::sync::Arc;

    #[test]
    fn existing_primary_holder_outlives_runner_world_and_releases_after_run() {
        // No native event loop or fabricated UI handles: test the actual
        // public holder and runner ownership boundary used in production.
        let mut app = App::new();
        app.add_plugins(WindowPlugin::default());
        let retained = retain_primary(&mut app).unwrap();
        let weak = Arc::downgrade(&retained.0);
        {
            let mut windows = app
                .world_mut()
                .query_filtered::<&RawHandleWrapperHolder, With<PrimaryWindow>>();
            assert!(Arc::ptr_eq(
                &retained.0,
                &windows.single(app.world()).unwrap().0
            ));
        }
        let runner_weak = weak.clone();
        app.set_runner(move |mut owned_app| {
            // Winit's exiting callback clears the main world while render
            // shutdown can still be waiting on the last extracted frame.
            owned_app.world_mut().clear_all();
            assert_eq!(runner_weak.strong_count(), 1);
            assert!(runner_weak.upgrade().is_some());
            AppExit::Success
        });
        assert!(app.run().is_success());
        assert_eq!(weak.strong_count(), 1);
        drop(retained);
        assert!(
            weak.upgrade().is_none(),
            "the keepalive must not leak ownership"
        );
    }
}
