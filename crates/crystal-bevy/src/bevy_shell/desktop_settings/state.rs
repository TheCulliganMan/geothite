//! Presentation preferences stay separate from the cartridge's game options.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Camera {
    pub zoom: f32,
    pub rotation: f32,
    pub pitch: f32,
    pub first_person: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            rotation: 0.0,
            pitch: 45.0,
            first_person: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Preferences {
    pub volume: u8,
    pub muted: bool,
    pub view_3d: bool,
    pub camera: Camera,
    pub fullscreen: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            volume: 100,
            muted: false,
            view_3d: false,
            camera: Camera::default(),
            fullscreen: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Preset {
    Default,
    Close,
    Wide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Volume(u8),
    Mute,
    Toggle3d,
    ToggleFirstPerson,
    Camera(Preset),
    ResetCamera,
    Fullscreen,
    Reset,
}

impl Preferences {
    pub fn gain(self) -> f32 {
        if self.muted {
            0.0
        } else {
            f32::from(self.volume.min(100)) / 100.0
        }
    }

    pub fn sanitize(&mut self) {
        let finite = |value: f32, default: f32| if value.is_finite() { value } else { default };
        self.volume = self.volume.min(100);
        self.camera.zoom = finite(self.camera.zoom, 1.0).clamp(0.0, 5.0);
        self.camera.rotation = finite(self.camera.rotation, 0.0).rem_euclid(8.0);
        self.camera.pitch = finite(
            self.camera.pitch,
            if self.camera.first_person { 0.0 } else { 45.0 },
        )
        .clamp(if self.camera.first_person { -85.0 } else { 5.0 }, 85.0);
    }

    pub fn apply(&mut self, action: Action, facing: [f32; 2]) {
        match action {
            Action::Volume(volume) => {
                self.volume = volume.min(100);
                self.muted = false;
            }
            Action::Mute => self.muted = !self.muted,
            Action::Toggle3d => self.view_3d = !self.view_3d,
            Action::ToggleFirstPerson => {
                self.camera.first_person = !(self.view_3d && self.camera.first_person);
                self.view_3d = true;
                self.camera.pitch = if self.camera.first_person { 0.0 } else { 45.0 };
                if self.camera.first_person
                    && facing.iter().all(|value| value.is_finite())
                    && facing[0].hypot(facing[1]) > 0.0
                {
                    self.camera.rotation = (-facing[0])
                        .atan2(facing[1])
                        .rem_euclid(std::f32::consts::TAU)
                        / std::f32::consts::FRAC_PI_4;
                }
            }
            Action::Camera(preset) => {
                self.view_3d = true;
                self.camera.first_person = false;
                self.camera.zoom = match preset {
                    Preset::Close => 3.0,
                    Preset::Wide => 0.0,
                    Preset::Default => 1.0,
                };
                self.camera.pitch = 45.0;
            }
            Action::ResetCamera => self.camera = Camera::default(),
            Action::Fullscreen => self.fullscreen = !self.fullscreen,
            Action::Reset => *self = Self::default(),
        }
        self.sanitize();
    }

    pub fn selected_camera(self) -> Option<Preset> {
        if self.camera.first_person {
            return None;
        }
        if (self.camera.pitch - 45.0).abs() > 0.01 {
            return None;
        }
        match self.camera.zoom {
            zoom if (zoom - 1.0).abs() < 0.01 => Some(Preset::Default),
            zoom if (zoom - 3.0).abs() < 0.01 => Some(Preset::Close),
            zoom if zoom.abs() < 0.01 => Some(Preset::Wide),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mute_restores_the_selected_volume_and_zero_survives_unmute() {
        let mut prefs = Preferences::default();
        prefs.apply(Action::Volume(25), [0.0, -1.0]);
        assert_eq!(prefs.gain(), 0.25);
        prefs.apply(Action::Mute, [0.0, -1.0]);
        assert_eq!(prefs.gain(), 0.0);
        prefs.apply(Action::Mute, [0.0, -1.0]);
        assert_eq!(prefs.gain(), 0.25);
        prefs.apply(Action::Volume(0), [0.0, -1.0]);
        prefs.apply(Action::Mute, [0.0, -1.0]);
        prefs.apply(Action::Mute, [0.0, -1.0]);
        assert_eq!(prefs.gain(), 0.0);
    }

    #[test]
    fn camera_presets_align_first_person_and_preserve_audio_preferences() {
        let mut prefs = Preferences {
            volume: 50,
            muted: true,
            ..Preferences::default()
        };
        prefs.apply(Action::ToggleFirstPerson, [1.0, 0.0]);
        assert!(prefs.view_3d && prefs.camera.first_person);
        assert_eq!(prefs.camera.rotation, 6.0);
        assert_eq!(prefs.camera.pitch, 0.0);
        prefs.apply(Action::Camera(Preset::Wide), [1.0, 0.0]);
        assert_eq!(prefs.selected_camera(), Some(Preset::Wide));
        assert_eq!(prefs.camera.rotation, 6.0);
        assert_eq!((prefs.volume, prefs.muted), (50, true));
        prefs.apply(Action::ResetCamera, [1.0, 0.0]);
        assert_eq!(prefs.camera, Camera::default());
        assert_eq!((prefs.volume, prefs.muted), (50, true));
    }

    #[test]
    fn first_person_toggle_returns_to_orbit_and_enables_3d_from_classic() {
        let mut prefs = Preferences::default();
        prefs.apply(Action::Camera(Preset::Close), [1.0, 0.0]);
        prefs.apply(Action::ToggleFirstPerson, [1.0, 0.0]);
        assert!(prefs.view_3d && prefs.camera.first_person);
        assert_eq!(prefs.camera.zoom, 3.0);
        assert_eq!(prefs.camera.rotation, 6.0);
        assert_eq!(prefs.selected_camera(), None);
        prefs.apply(Action::ToggleFirstPerson, [0.0, -1.0]);
        assert!(prefs.view_3d && !prefs.camera.first_person);
        assert_eq!(prefs.camera.zoom, 3.0);
        assert_eq!(prefs.camera.pitch, 45.0);
        assert_eq!(prefs.selected_camera(), Some(Preset::Close));
        prefs.apply(Action::ToggleFirstPerson, [0.0, -1.0]);
        prefs.apply(Action::Toggle3d, [0.0, -1.0]);
        assert!(!prefs.view_3d && prefs.camera.first_person);
        prefs.apply(Action::ToggleFirstPerson, [0.0, -1.0]);
        assert!(prefs.view_3d && prefs.camera.first_person);
        assert!((prefs.camera.rotation - 4.0).abs() < 0.00001);
    }

    #[test]
    fn corrupt_camera_values_are_bounded_and_custom_views_are_not_mislabelled() {
        let mut prefs = Preferences::default();
        prefs.camera = Camera {
            zoom: f32::NAN,
            rotation: -9.0,
            pitch: f32::INFINITY,
            first_person: true,
        };
        prefs.sanitize();
        assert_eq!(
            prefs.camera,
            Camera {
                zoom: 1.0,
                rotation: 7.0,
                pitch: 0.0,
                first_person: true
            }
        );
        prefs.camera = Camera {
            zoom: 2.25,
            ..Camera::default()
        };
        assert_eq!(prefs.selected_camera(), None);
    }
}
