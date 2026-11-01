//! Bounded native 2D rain/snow particles and a subtle world-only rain shade.
use bevy::ecs::system::SystemParam;
use bevy::prelude::{
    Color, Commands, Component, Local, Projection, Quat, Query, Res, ResMut, Sprite, Time,
    Transform, Vec2, Visibility, With, Without,
};

use super::{
    BevyRuntimeShell, MainCameraMarker, PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH, RenderedViewport,
    RuntimeTickTimer, TILE_SIZE, outdoor_weather_policy, visible_overworld_camera_offset,
};

const SNOWFLAKES: usize = 48;
const RAINDROPS: usize = 80;

#[derive(Component)]
pub(super) struct Particle {
    index: usize,
    rain: bool,
}

#[derive(Component)]
pub(super) struct CloudShade;

type ShadeSprites<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Sprite,
        &'static mut Transform,
        &'static mut Visibility,
    ),
    (With<CloudShade>, Without<Particle>),
>;

#[derive(SystemParam)]
pub(super) struct WeatherView<'w, 's> {
    rendered: Res<'w, RenderedViewport>,
    time: Res<'w, Time>,
    timer: Res<'w, RuntimeTickTimer>,
    cameras: Query<'w, 's, &'static Projection, With<MainCameraMarker>>,
    #[cfg(feature = "fullscreen-scaling")]
    world_root: Query<
        'w,
        's,
        &'static Transform,
        (
            With<super::FullscreenWorldRoot>,
            Without<Particle>,
            Without<CloudShade>,
        ),
    >,
    #[cfg(feature = "voxel-view")]
    settings: Option<Res<'w, crystal_voxel_view::VoxelViewSettings>>,
}

fn position(index: usize, seconds: f32, extent: Vec2, rain: bool, scroll_fall: f32) -> Vec2 {
    let phase = index as f32 * 2.399_963;
    let (drift, speed) = if rain {
        (22.0, 180.0 + (index % 5) as f32 * 7.0)
    } else {
        (3.0 + phase.sin(), 13.0 + (index % 5) as f32)
    };
    let x = ((index * 73 % 257) as f32 / 257.0 * extent.x + seconds * drift + phase.sin() * 4.0)
        .rem_euclid(extent.x);
    let y = ((index * 47 % 251) as f32 / 251.0 * extent.y
        + seconds * speed
        + if rain { scroll_fall } else { 0.0 })
    .rem_euclid(extent.y);
    Vec2::new(x - extent.x * 0.5, extent.y * 0.5 - y)
}

#[derive(Default)]
pub(super) struct ScrollSample {
    map: Option<String>,
    camera_y: f32,
    scale: f32,
}

impl ScrollSample {
    fn downward_world_motion(&mut self, map: Option<&str>, camera_y: f32, scale: f32) -> f32 {
        let motion = if self.map.as_deref() == map && self.scale == scale {
            // Map coordinates grow southward; a decreasing camera coordinate
            // moves the ground down in the screen's upward-positive space.
            (self.camera_y - camera_y).max(0.0) * scale
        } else {
            0.0
        };
        if self.map.as_deref() != map {
            self.map = map.map(str::to_owned);
        }
        self.camera_y = camera_y;
        self.scale = scale;
        motion
    }
}

pub(super) fn animate(
    mut commands: Commands,
    mut shell: ResMut<BevyRuntimeShell>,
    view: WeatherView,
    mut initialized: Local<bool>,
    mut scroll_fall: Local<f32>,
    mut previous_scroll: Local<ScrollSample>,
    mut particles: Query<
        (&Particle, &mut Transform, &mut Visibility, &mut Sprite),
        Without<CloudShade>,
    >,
    mut shade: ShadeSprites,
) {
    if !*initialized {
        for (rain, count) in [(false, SNOWFLAKES), (true, RAINDROPS)] {
            for index in 0..count {
                commands.spawn((
                    Particle { index, rain },
                    Sprite {
                        custom_size: Some(if rain {
                            Vec2::new(3.0, 16.0)
                        } else {
                            Vec2::splat(if index % 3 == 0 { 8.0 } else { 4.0 })
                        }),
                        ..Default::default()
                    },
                    Transform::default(),
                    Visibility::Hidden,
                ));
            }
        }
        commands.spawn((
            CloudShade,
            Sprite::default(),
            Transform::default(),
            Visibility::Hidden,
        ));
        *initialized = true;
    }
    #[cfg(feature = "voxel-view")]
    let native = view.settings.is_none_or(|s| !s.enabled);
    #[cfg(not(feature = "voxel-view"))]
    let native = true;
    #[cfg(feature = "fullscreen-scaling")]
    let battle_presented = super::fullscreen_battle_active(&shell);
    #[cfg(not(feature = "fullscreen-scaling"))]
    let battle_presented = shell.battle_lcd_animation_active
        || shell
            .battle_message_scene
            .as_ref()
            .is_some_and(|scene| scene.battle.is_some());
    let weather = if native
        && !view.rendered.title_active
        && !battle_presented
        && shell.visible_battle_transition.is_none()
    {
        super::cached_runtime_snapshot(&mut shell)
            .ok()
            .and_then(|s| {
                if s.battle.is_some()
                    || view.rendered.map_name.as_deref() != Some(s.overworld.map_name.as_str())
                {
                    return None;
                }
                let map = s.maps.iter().find(|m| m.map_name == s.overworld.map_name)?;
                Some(outdoor_weather_policy::sample(
                    &map.map_name,
                    &map.attributes.tileset_name,
                    map.attributes.environment.as_deref()?,
                    s.overworld.tile.x,
                    s.overworld.tile.y,
                    &s.progression.time,
                ))
            })
            .unwrap_or_default()
    } else {
        Default::default()
    };
    let extent = view
        .cameras
        .single()
        .ok()
        .and_then(|p| match p {
            Projection::Orthographic(p) => Some(p.area.size()),
            _ => None,
        })
        .filter(|size| size.min_element() > 0.0)
        .unwrap_or(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT));
    #[cfg(feature = "fullscreen-scaling")]
    let world_scale = view.world_root.single().map_or(1.0, |root| root.scale.y);
    #[cfg(not(feature = "fullscreen-scaling"))]
    let world_scale = 1.0;
    let camera_offset =
        visible_overworld_camera_offset(&view.rendered, &shell, view.timer.presentation_subframe());
    let camera_y = view
        .rendered
        .viewport_origin
        .map_or(0.0, |(_, y)| f32::from(y) * TILE_SIZE + camera_offset.y);
    let map = view
        .rendered
        .viewport_origin
        .and(view.rendered.map_name.as_deref());
    let world_motion = previous_scroll.downward_world_motion(map, camera_y, world_scale);
    *scroll_fall = (*scroll_fall + world_motion).rem_euclid(extent.y);
    for (particle, mut transform, mut visibility, mut sprite) in &mut particles {
        let strength = if particle.rain {
            weather.rain
        } else {
            weather.snow
        };
        *visibility = if strength > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if strength > 0.0 {
            let at = position(
                particle.index,
                view.time.elapsed_secs(),
                extent,
                particle.rain,
                *scroll_fall,
            );
            *transform = Transform::from_xyz(at.x, at.y, 2.65).with_rotation(
                Quat::from_rotation_z(if particle.rain { -0.15 } else { 0.0 }),
            );
            sprite.color = if particle.rain {
                Color::srgba(0.66, 0.80, 0.94, strength * 0.72)
            } else {
                Color::srgba(0.95, 0.98, 1.0, strength)
            };
        }
    }
    if let Ok((mut sprite, mut transform, mut visibility)) = shade.single_mut() {
        *visibility = if weather.rain > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        sprite.custom_size = Some(extent);
        sprite.color = Color::srgba(0.04, 0.07, 0.12, weather.rain * 0.13);
        // Same world overlay plane as the old Ely snowfall; modal canvases and
        // text stay above it. Rain shading never changes native source assets.
        *transform = Transform::from_xyz(0.0, 0.0, 2.60);
    }
}

#[cfg(test)]
mod tests {
    use super::{RAINDROPS, SNOWFLAKES, ScrollSample, position};
    use crate::bevy_shell::{
        RenderedViewport, TILE_SIZE, overworld_walk_camera_offset_for_duration_with_subframe,
    };
    use bevy::prelude::Vec2;

    #[test]
    fn precipitation_moves_inside_a_bounded_pool_at_different_window_sizes() {
        assert_eq!(RAINDROPS + SNOWFLAKES, 128);
        for extent in [Vec2::new(640.0, 576.0), Vec2::new(1280.0, 720.0)] {
            for (rain, count) in [(false, SNOWFLAKES), (true, RAINDROPS)] {
                for index in 0..count {
                    let start = position(index, 0.0, extent, rain, 0.0);
                    let after = position(index, 2.0, extent, rain, 0.0);
                    assert_ne!(start, after);
                    for at in [start, after] {
                        assert!(at.x.abs() <= extent.x * 0.5 && at.y.abs() <= extent.y * 0.5);
                    }
                }
            }
        }
    }

    #[test]
    fn upward_walking_adds_downward_rain_motion_without_affecting_snow() {
        let extent = Vec2::new(640.0, 576.0);
        for scale in [1.0, 2.0] {
            let mut scroll = ScrollSample::default();
            assert_eq!(
                scroll.downward_world_motion(Some("route"), 100.0, scale),
                0.0
            );
            let dt = 0.02;
            let world_scroll = scroll.downward_world_motion(Some("route"), 92.0, scale);
            assert_eq!(world_scroll, 8.0 * scale);
            let before = position(10, 0.0, extent, true, 0.0);
            let after = position(10, dt, extent, true, world_scroll);
            // Upward travel moves the ground down. Rain must descend faster
            // than that ground in screen coordinates as well as on screen.
            assert!(after.y < before.y - world_scroll);
            assert_eq!(
                scroll.downward_world_motion(Some("route"), 100.0, scale),
                0.0
            );
            assert_eq!(
                scroll.downward_world_motion(Some("other"), 120.0, scale),
                0.0
            );
            assert_eq!(
                scroll.downward_world_motion(Some("other"), -500.0, scale),
                620.0 * scale
            );
            assert_eq!(
                scroll.downward_world_motion(Some("other"), -500.0, scale),
                0.0
            );
            assert_eq!(
                scroll.downward_world_motion(Some("other"), -510.0, scale * 2.0),
                0.0
            );
            assert_eq!(scroll.downward_world_motion(None, 0.0, scale), 0.0);
        }
        assert_eq!(
            position(10, 0.02, extent, false, 0.0),
            position(10, 0.02, extent, false, 9.0)
        );
    }

    #[test]
    fn rain_tracks_the_rendered_camera_through_an_upward_step() {
        let rendered = RenderedViewport {
            walk_viewport_origin: Some((10, 20)),
            viewport_origin: Some((10, 18)),
            ..Default::default()
        };
        let mut scroll = ScrollSample::default();
        let extent = Vec2::new(640.0, 576.0);
        let mut fall = 0.0;
        let mut previous_rain = position(1, 0.0, extent, true, fall).y;
        for frame in 0..=8 {
            let remaining = 8 - frame;
            let offset = overworld_walk_camera_offset_for_duration_with_subframe(
                &rendered, remaining, 8, 0.0,
            );
            let camera_y = 18.0 * TILE_SIZE + offset.y;
            let world_motion = scroll.downward_world_motion(Some("route"), camera_y, 1.0);
            fall += world_motion;
            let rain = position(1, frame as f32 * 0.01, extent, true, fall).y;
            if frame > 0 {
                assert!(rain < previous_rain - world_motion);
            }
            previous_rain = rain;
        }
        assert_eq!(fall, 2.0 * TILE_SIZE);
    }
}
