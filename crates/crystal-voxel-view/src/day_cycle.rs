//! Presentation-only outdoor lighting, sampled from the authoritative clock.
//! Smoothstep between authored stops avoids jumps at palette-period boundaries.
use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct DayCycle {
    pub ambient: Color,
    pub direct: Color,
    pub ambient_gain: f32,
    pub direct_gain: f32,
    pub regional_tint_gain: f32,
    pub direction: Vec3,
    pub horizon: Color,
}

impl DayCycle {
    fn stop(
        ambient: [f32; 3],
        direct: [f32; 3],
        gains: [f32; 2],
        direction: [f32; 3],
        horizon: [f32; 3],
    ) -> Self {
        fn tint(rgb: [f32; 3]) -> Color {
            let luminance = rgb[0] * 0.2126 + rgb[1] * 0.7152 + rgb[2] * 0.0722;
            Color::linear_rgb(rgb[0] / luminance, rgb[1] / luminance, rgb[2] / luminance)
        }
        Self {
            ambient: tint(ambient),
            direct: tint(direct),
            ambient_gain: gains[0],
            direct_gain: gains[1],
            regional_tint_gain: 1.0,
            direction: Vec3::from_array(direction),
            horizon: Color::srgb_from_array(horizon),
        }
    }

    #[cfg(test)]
    pub fn at(minute: u16) -> Self {
        Self::at_seconds(u32::from(minute) * 60)
    }

    pub fn at_seconds(second: u32) -> Self {
        let night = Self::stop(
            // Cool moonlight preserves the 3D materials' warm/cool separation.
            // A saturated palette-blue light erased their pigment and turned
            // pale rock and people into the same flat violet silhouette.
            [0.72, 0.82, 1.08],
            [0.78, 0.88, 1.15],
            [0.42, 0.40],
            [-180.0, 260.0, 220.0],
            [0.055, 0.085, 0.17],
        );
        let mut dawn = Self::stop(
            [1.05, 1.02, 0.94],
            [1.18, 1.03, 0.70],
            [0.78, 0.72],
            [300.0, 380.0, 220.0],
            [0.86, 0.76, 0.62],
        );
        // The source morning temperature already supplies yellow warmth.
        // Do not multiply it by a second regional golden-day tint.
        dawn.regional_tint_gain = 0.0;
        let day = Self::stop(
            [1.0; 3],
            [1.0; 3],
            [1.0, 1.0],
            [180.0, 320.0, 220.0],
            [0.72, 0.83, 0.89],
        );
        let evening = Self::stop(
            [0.96, 0.98, 1.10],
            [1.38, 0.90, 0.55],
            [0.75, 0.70],
            [-240.0, 340.0, 300.0],
            [0.85, 0.53, 0.35],
        );
        let late_morning = Self {
            direction: Vec3::new(260.0, 380.0, 220.0),
            ..day
        };
        let afternoon = Self {
            direction: Vec3::new(-30.0, 280.0, 280.0),
            ..day
        };
        let stops = [
            (0.0, night),
            (4.0, night),
            (6.0, dawn),
            (9.0, late_morning),
            (12.0, day),
            (15.0, afternoon),
            (18.0, evening),
            (20.0, night),
            (24.0, night),
        ];
        let hour = (second % 86_400) as f32 / 3600.0;
        let pair = stops.windows(2).find(|pair| hour < pair[1].0).unwrap();
        let t = (hour - pair[0].0) / (pair[1].0 - pair[0].0);
        let t = t * t * (3.0 - 2.0 * t);
        let (a, b) = (pair[0].1, pair[1].1);
        Self {
            ambient: a.ambient.mix(&b.ambient, t),
            direct: a.direct.mix(&b.direct, t),
            ambient_gain: a.ambient_gain + (b.ambient_gain - a.ambient_gain) * t,
            direct_gain: a.direct_gain + (b.direct_gain - a.direct_gain) * t,
            regional_tint_gain: a.regional_tint_gain
                + (b.regional_tint_gain - a.regional_tint_gain) * t,
            direction: a.direction.lerp(b.direction, t),
            horizon: a.horizon.mix(&b.horizon, t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cycle_is_continuous_across_palette_boundaries_and_midnight() {
        for boundary in [240, 600, 1080, 1200, 1440] {
            let a = DayCycle::at(boundary - 1);
            let b = DayCycle::at(boundary);
            assert!((a.ambient_gain - b.ambient_gain).abs() < 0.01);
            assert!((a.direct_gain - b.direct_gain).abs() < 0.01);
            assert!(a.direction.distance(b.direction) < 6.0);
            for (x, y) in a
                .direct
                .to_linear()
                .to_f32_array()
                .into_iter()
                .zip(b.direct.to_linear().to_f32_array())
            {
                assert!((x - y).abs() < 0.02);
            }
        }
    }
    #[test]
    fn lighting_progresses_within_a_minute_and_wraps_smoothly_over_a_day() {
        let a = DayCycle::at_seconds(7 * 3600 + 30 * 60);
        let b = DayCycle::at_seconds(7 * 3600 + 30 * 60 + 30);
        let c = DayCycle::at_seconds(7 * 3600 + 31 * 60);
        assert!(a.ambient_gain < b.ambient_gain && b.ambient_gain < c.ambient_gain);
        assert!(a.direction.x > b.direction.x && b.direction.x > c.direction.x);
        for second in 1..=86_400 {
            let previous = DayCycle::at_seconds(second - 1);
            let current = DayCycle::at_seconds(second);
            assert!((previous.direct_gain - current.direct_gain).abs() < 0.0002);
            assert!((previous.ambient_gain - current.ambient_gain).abs() < 0.0002);
            assert!(previous.direction.distance(current.direction) < 0.11);
        }
    }

    #[test]
    fn sun_travels_and_night_retains_fill_without_daytime_shadow_energy() {
        let morning = DayCycle::at(360);
        let noon = DayCycle::at(720);
        let sunset = DayCycle::at(1080);
        let night = DayCycle::at(0);
        assert!(morning.direction.x > 0.0 && sunset.direction.x < 0.0);
        assert!(
            Vec2::new(morning.direction.x, morning.direction.z).length() / morning.direction.y
                < 1.0,
            "morning shadows must be shorter than the caster height"
        );
        assert!(
            noon.direction.normalize().y > morning.direction.normalize().y
                && noon.direction.normalize().y > sunset.direction.normalize().y
        );
        assert!(sunset.direct.to_linear().red > sunset.direct.to_linear().blue);
        assert!(night.ambient.to_linear().blue > night.ambient.to_linear().red);
        assert!(night.ambient_gain > night.direct_gain);
        assert_eq!(noon.ambient_gain, 1.0);
        assert_eq!(noon.direct_gain, 1.0);
        for minute in 0..1440 {
            let sample = DayCycle::at(minute);
            assert!((0.4199..=1.0001).contains(&sample.ambient_gain));
            assert!((0.3999..=1.0001).contains(&sample.direct_gain));
            assert!(sample.direction.y >= 260.0);
            assert!(
                Vec2::new(sample.direction.x, sample.direction.z).length() / sample.direction.y
                    <= 1.25,
                "sun angle must not stretch house/tree shadows across the whole path"
            );
        }
    }
    #[test]
    fn clock_updates_real_lights_camera_and_shadows_without_mutating_the_world() {
        use crate::{VoxelSun, VoxelViewSettings, VoxelWorldCamera, sync_voxel_atmosphere};
        use crystal_render_api::{VisualWorldClock, VisualWorldFrame};
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<VisualWorldClock>()
            .init_resource::<crate::live_profiles::LiveProfiles>()
            .init_resource::<VoxelViewSettings>()
            .init_resource::<GlobalAmbientLight>()
            .add_systems(Update, sync_voxel_atmosphere);
        let sun = app
            .world_mut()
            .spawn((
                DirectionalLight::default(),
                Transform::default(),
                bevy::light::CascadeShadowConfig::default(),
                VoxelSun,
            ))
            .id();
        let camera = app
            .world_mut()
            .spawn((
                Camera::default(),
                DistanceFog::default(),
                bevy::anti_alias::fxaa::Fxaa::default(),
                VoxelWorldCamera,
            ))
            .id();
        let frame = app.world().resource::<VisualWorldFrame>().clone();
        for minute in [360, 720, 1080, 0] {
            app.world_mut()
                .resource_mut::<VisualWorldClock>()
                .minute_of_day = Some(minute);
            app.update();
            let expected = DayCycle::at(minute);
            let light = app.world().get::<DirectionalLight>(sun).unwrap();
            assert!((light.illuminance - 3000.0 * expected.direct_gain).abs() < 0.01);
            assert_eq!(light.color, expected.direct);
            assert_eq!(
                light.shadow_maps_enabled,
                minute != 0,
                "night uses authored contact shading rather than large moon-cast silhouettes"
            );
            let transform = app.world().get::<Transform>(sun).unwrap();
            assert_eq!(transform.translation, expected.direction);
            assert!(
                matches!(app.world().get::<Camera>(camera).unwrap().clear_color,
                bevy::camera::ClearColorConfig::Custom(color) if color == expected.horizon)
            );
            assert_eq!(*app.world().resource::<VisualWorldFrame>(), frame);
        }
        // Leaving an automatic outdoor palette restores the original room light.
        app.world_mut()
            .resource_mut::<VisualWorldClock>()
            .minute_of_day = None;
        app.update();
        assert_eq!(
            app.world()
                .get::<DirectionalLight>(sun)
                .unwrap()
                .illuminance,
            3000.0
        );
        assert_eq!(
            app.world().get::<Transform>(sun).unwrap().translation,
            Vec3::new(180.0, 320.0, 220.0)
        );
        assert_eq!(
            app.world().resource::<GlobalAmbientLight>().brightness,
            80.0
        );
    }

    #[test]
    fn enclosed_rooms_ignore_an_outdoor_clock() {
        use crystal_render_api::{
            VisualTile, VisualTileSource, VisualWorldClock, VisualWorldFrame,
        };
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .insert_resource(VisualWorldClock {
                minute_of_day: Some(0),
                second_of_minute: 0,
            })
            .init_resource::<crate::live_profiles::LiveProfiles>()
            .init_resource::<crate::VoxelViewSettings>()
            .init_resource::<GlobalAmbientLight>()
            .add_systems(Update, crate::sync_voxel_atmosphere);
        app.world_mut()
            .resource_mut::<crate::VoxelViewSettings>()
            .enabled = true;
        let sun = app
            .world_mut()
            .spawn((
                DirectionalLight::default(),
                Transform::default(),
                bevy::light::CascadeShadowConfig::default(),
                crate::VoxelSun,
            ))
            .id();
        for (map, tileset, energy) in [
            ("PlayersHouse1F", "house", 3000.0),
            ("UnionCave1F", "cave", 1440.0),
        ] {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.active = true;
            frame.map_id = map.into();
            frame.tiles = vec![VisualTile {
                column: 0,
                row: 0,
                priority: false,
                texture: Default::default(),
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: tileset.into(),
                    metatile_id: 0,
                    tile_index: 0,
                    subtile_column: 0,
                    subtile_row: 0,
                },
            }];
            drop(frame);
            app.update();
            assert_eq!(
                app.world()
                    .get::<DirectionalLight>(sun)
                    .unwrap()
                    .illuminance,
                energy
            );
            assert_eq!(
                app.world().get::<Transform>(sun).unwrap().translation,
                Vec3::new(180.0, 320.0, 220.0)
            );
        }
    }
}
