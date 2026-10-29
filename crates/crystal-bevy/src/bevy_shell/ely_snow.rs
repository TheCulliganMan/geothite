//! Bounded presentation-only snowfall over the authored Ely winter exterior.
use bevy::prelude::{
    Color, Commands, Component, Local, Projection, Query, Res, ResMut, Sprite, Time, Transform,
    Vec2, Visibility, With,
};

use super::{
    BevyRuntimeShell, MainCameraMarker, PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH, RenderedViewport,
};

const FLAKES: usize = 48;
#[derive(Component)]
pub(super) struct Snowflake(usize);

fn snowy_exterior(map: &str, tileset: &str) -> bool {
    map == "MnEly" && tileset == "mn_ely_snow"
}

fn position(slot: usize, seconds: f32, extent: Vec2) -> Vec2 {
    let phase = slot as f32 * 2.399_963;
    let x = ((slot * 73 % 257) as f32 / 257.0 * extent.x
        + seconds * (3.0 + phase.sin())
        + phase.sin() * 4.0)
        .rem_euclid(extent.x);
    let y = ((slot * 47 % 251) as f32 / 251.0 * extent.y + seconds * (13.0 + slot as f32 % 5.0))
        .rem_euclid(extent.y);
    Vec2::new(x - extent.x * 0.5, extent.y * 0.5 - y)
}

pub(super) fn animate(
    mut commands: Commands,
    mut shell: ResMut<BevyRuntimeShell>,
    rendered: Res<RenderedViewport>,
    time: Res<Time>,
    cameras: Query<&Projection, With<MainCameraMarker>>,
    mut initialized: Local<bool>,
    mut particles: Query<(&Snowflake, &mut Transform, &mut Visibility)>,
    #[cfg(feature = "voxel-view")] settings: Option<Res<crystal_voxel_view::VoxelViewSettings>>,
) {
    if !*initialized {
        for slot in 0..FLAKES {
            commands.spawn((
                Snowflake(slot),
                Sprite {
                    color: Color::srgb(0.95, 0.98, 1.0),
                    custom_size: Some(Vec2::splat(if slot % 3 == 0 { 8.0 } else { 4.0 })),
                    ..Default::default()
                },
                Transform::default(),
                Visibility::Hidden,
            ));
        }
        *initialized = true;
    }
    #[cfg(feature = "voxel-view")]
    let source_view = settings.is_none_or(|s| !s.enabled);
    #[cfg(not(feature = "voxel-view"))]
    let source_view = true;
    let active = source_view
        && !rendered.title_active
        && super::cached_runtime_snapshot(&mut shell).is_ok_and(|snapshot| {
            snapshot.battle.is_none()
                && rendered.map_name.as_deref() == Some(snapshot.overworld.map_name.as_str())
                && snapshot.maps.iter().any(|m| {
                    m.map_name == snapshot.overworld.map_name
                        && snowy_exterior(&m.map_name, &m.attributes.tileset_name)
                })
        });
    let extent = cameras
        .single()
        .ok()
        .and_then(|p| match p {
            Projection::Orthographic(p) => Some(p.area.size()),
            _ => None,
        })
        .filter(|size| size.min_element() > 0.0)
        .unwrap_or(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT));
    for (flake, mut transform, mut visibility) in &mut particles {
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if active {
            let at = position(flake.0, time.elapsed_secs(), extent);
            // Below battle/fullscreen/modal canvases, above ordinary world sprites.
            *transform = Transform::from_xyz(at.x, at.y, 2.65);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FLAKES, PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH, position, snowy_exterior};
    use bevy::prelude::Vec2;
    #[test]
    fn snow_is_confined_to_the_new_ely_exterior_and_moves_within_the_viewport() {
        assert!(snowy_exterior("MnEly", "mn_ely_snow"));
        assert_eq!(
            super::super::authored_visual_sources::tileset_for_block("mn_ely_snow", 0x7f),
            "kanto"
        );
        for (map, art) in [
            ("MnEly", "kanto"),
            ("MnElyPokecenter1F", "mn_ely_snow"),
            ("MnR11C00R02", "mn_north_shore"),
        ] {
            assert!(!snowy_exterior(map, art));
        }
        for slot in 0..FLAKES {
            let extent = Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT);
            let start = position(slot, 0.0, extent);
            let end = position(slot, 2.0, extent);
            assert_ne!(start, end);
            for p in [start, end] {
                assert!(p.x.abs() <= extent.x * 0.5);
                assert!(p.y.abs() <= extent.y * 0.5);
            }
        }
    }
}
