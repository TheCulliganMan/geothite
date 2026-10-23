//! Held mouse-button locomotion shares the field controller's camera-local axes.

use bevy::ecs::system::SystemParam;
use bevy::prelude::{ButtonInput, Local, MouseButton, Query, Res, Vec2, Window, With};
// Match the component used by the existing shell buttons.
use bevy::ui::DeprecatedInteraction;
use bevy::window::PrimaryWindow;

#[derive(SystemParam)]
pub(super) struct MouseMovement<'w, 's> {
    buttons: Option<Res<'w, ButtonInput<MouseButton>>>,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    interactions: Query<'w, 's, &'static DeprecatedInteraction>,
    wait_for_release: Local<'s, bool>,
}

impl MouseMovement<'_, '_> {
    pub(super) fn buttons_held(&self) -> bool {
        self.buttons
            .as_ref()
            .is_some_and(|buttons| buttons.any_pressed([MouseButton::Left, MouseButton::Right]))
    }

    pub(super) fn forward(&mut self, enabled: bool, blocked: bool) -> Vec2 {
        let left = self
            .buttons
            .as_ref()
            .is_some_and(|b| b.pressed(MouseButton::Left));
        let right = self
            .buttons
            .as_ref()
            .is_some_and(|b| b.pressed(MouseButton::Right));
        if !left && !right {
            *self.wait_for_release = false;
        }
        if !enabled
            || blocked
            || self.windows.iter().any(|window| !window.focused)
            || self
                .interactions
                .iter()
                .any(|interaction| *interaction != DeprecatedInteraction::None)
        {
            *self.wait_for_release = left || right;
            return Vec2::ZERO;
        }
        if left && right && !*self.wait_for_release {
            Vec2::NEG_Y
        } else {
            Vec2::ZERO
        }
    }
}
