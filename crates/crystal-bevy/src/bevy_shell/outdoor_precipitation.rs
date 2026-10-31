//! Bounded native 2D rain/snow particles and a subtle world-only rain shade.
use bevy::ecs::system::SystemParam;
use bevy::prelude::{
    Color, Commands, Component, Local, Projection, Quat, Query, Res, ResMut, Sprite, Time,
    Transform, Vec2, Visibility, With, Without,
};

use super::{
    BevyRuntimeShell, MainCameraMarker, PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH, RenderedViewport,
    outdoor_weather_policy,
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
    cameras: Query<'w, 's, &'static Projection, With<MainCameraMarker>>,
    #[cfg(feature = "voxel-view")]
    settings: Option<Res<'w, crystal_voxel_view::VoxelViewSettings>>,
}

fn position(index: usize, seconds: f32, extent: Vec2, rain: bool) -> Vec2 {
    let phase = index as f32 * 2.399_963;
    let (drift, speed) = if rain {
        (22.0, 180.0 + (index % 5) as f32 * 7.0)
    } else {
        (3.0 + phase.sin(), 13.0 + (index % 5) as f32)
    };
    let x = ((index * 73 % 257) as f32 / 257.0 * extent.x + seconds * drift + phase.sin() * 4.0)
        .rem_euclid(extent.x);
    let y = ((index * 47 % 251) as f32 / 251.0 * extent.y + seconds * speed).rem_euclid(extent.y);
    Vec2::new(x - extent.x * 0.5, extent.y * 0.5 - y)
}

pub(super) fn animate(
    mut commands: Commands,
    mut shell: ResMut<BevyRuntimeShell>,
    view: WeatherView,
    mut initialized: Local<bool>,
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
    let weather = if native && !view.rendered.title_active {
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
    use super::{RAINDROPS, SNOWFLAKES, position};
    use bevy::prelude::Vec2;

    #[test]
    fn precipitation_moves_inside_a_bounded_pool_at_different_window_sizes() {
        assert_eq!(RAINDROPS + SNOWFLAKES, 128);
        for extent in [Vec2::new(640.0, 576.0), Vec2::new(1280.0, 720.0)] {
            for (rain, count) in [(false, SNOWFLAKES), (true, RAINDROPS)] {
                for index in 0..count {
                    let start = position(index, 0.0, extent, rain);
                    let after = position(index, 2.0, extent, rain);
                    assert_ne!(start, after);
                    for at in [start, after] {
                        assert!(at.x.abs() <= extent.x * 0.5 && at.y.abs() <= extent.y * 0.5);
                    }
                }
            }
        }
    }
}
