//! Navigation keys have semantic meaning independent of physical keyboard layout.
//! Some X11/remote desktops expose non-evdev physical codes for named arrows.
use bevy::input::{
    ButtonState, InputSystems,
    keyboard::{Key, KeyboardInput},
};
use bevy::prelude::*;

pub(super) fn install(app: &mut App) {
    app.add_systems(PreUpdate, normalize_navigation.after(InputSystems));
}

fn navigation_code(key: &Key) -> Option<KeyCode> {
    match key {
        Key::ArrowUp => Some(KeyCode::ArrowUp),
        Key::ArrowDown => Some(KeyCode::ArrowDown),
        Key::ArrowLeft => Some(KeyCode::ArrowLeft),
        Key::ArrowRight => Some(KeyCode::ArrowRight),
        _ => None,
    }
}

fn normalize_navigation(
    mut events: MessageReader<KeyboardInput>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    for event in events.read() {
        let Some(code) = navigation_code(&event.logical_key) else {
            continue;
        };
        if code == event.key_code {
            continue;
        }
        match event.state {
            ButtonState::Pressed => keys.press(code),
            ButtonState::Released => keys.release(code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn semantic_arrows_keep_matching_press_and_release() {
        let mut app = App::new();
        app.add_message::<KeyboardInput>()
            .init_resource::<ButtonInput<KeyCode>>();
        install(&mut app);
        let window = app.world_mut().spawn_empty().id();
        for state in [ButtonState::Pressed, ButtonState::Released] {
            app.world_mut().write_message(KeyboardInput {
                key_code: KeyCode::Convert,
                logical_key: Key::ArrowLeft,
                state,
                window,
                text: None,
                repeat: false,
            });
            app.update();
            assert_eq!(
                app.world()
                    .resource::<ButtonInput<KeyCode>>()
                    .pressed(KeyCode::ArrowLeft),
                state == ButtonState::Pressed
            );
        }
    }
    #[test]
    fn character_and_conversion_keys_are_not_rebound() {
        assert_eq!(navigation_code(&Key::Character("w".into())), None);
        assert_eq!(navigation_code(&Key::Convert), None);
        assert_eq!(navigation_code(&Key::ArrowRight), Some(KeyCode::ArrowRight));
    }
}
