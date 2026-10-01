//! Retain original hidden maze faces in cached leaf draws. Only draw selection
//! changes as the view/player moves; the mesh and shared material stay intact.
use crate::{
    TerrainRevisionCache, VoxelMaterial, VoxelViewStatus, VoxelWorldCamera,
    interior_cutaway::CutawayUniform,
};
use bevy::prelude::*;

pub(super) fn register(app: &mut App) {
    app.add_systems(
        PostUpdate,
        sync.after(bevy::render::camera::CameraUpdateSystem)
            .after(bevy::transform::TransformSystem::TransformPropagate)
            .before(bevy::render::view::VisibilitySystems::VisibilityPropagate),
    );
}

#[derive(Component, Clone, Copy)]
pub(super) struct MazeRevealBatch {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Selection {
    Hidden,
    All,
    Bounds {
        view_from_world: Mat4,
        min: Vec2,
        max: Vec2,
    },
}

fn reveal_bounds(uniform: CutawayUniform, world_from_view: Mat4, padding: Vec2) -> Selection {
    if uniform.bottom_radius.w <= 0.0 {
        return Selection::Hidden;
    }
    if !world_from_view.is_finite()
        || !uniform.bottom_radius.is_finite()
        || !uniform.top_feather.is_finite()
        || !padding.is_finite()
    {
        return Selection::All;
    }
    let view_from_world = world_from_view.inverse();
    if !view_from_world.is_finite() {
        return Selection::All;
    }
    let bottom = view_from_world.transform_point3(uniform.bottom_radius.truncate());
    let top = view_from_world.transform_point3(uniform.top_feather.truncate());
    // Exactly the shader's disabled/behind-eye conditions. With no possible
    // reveal, source-proven buried faces cannot be seen in the ordinary union.
    if bottom.z >= -0.001 || top.z >= -0.001 {
        return Selection::Hidden;
    }
    let a = bottom.truncate() / -bottom.z;
    let b = top.truncate() / -top.z;
    // Harmonic capsule depth is no smaller than its nearer endpoint. Thus any
    // fragment the original shader can discard lies in this conservative box.
    let radius =
        (uniform.bottom_radius.w + uniform.top_feather.w).max(0.0) / (-bottom.z).min(-top.z);
    let margin = Vec2::splat(radius * 1.0001 + 0.0001) + padding.abs();
    let min = a.min(b) - margin;
    let max = a.max(b) + margin;
    if !min.is_finite() || !max.is_finite() {
        return Selection::All;
    }
    Selection::Bounds {
        view_from_world,
        min,
        max,
    }
}

fn may_expose(selection: Selection, batch: MazeRevealBatch, world_from_local: Mat4) -> bool {
    let Selection::Bounds {
        view_from_world,
        min,
        max,
    } = selection
    else {
        return selection == Selection::All;
    };
    let view_from_local = view_from_world * world_from_local;
    if !view_from_local.is_finite() || !batch.min.is_finite() || !batch.max.is_finite() {
        return true;
    }
    let mut projected_min = Vec2::splat(f32::INFINITY);
    let mut projected_max = Vec2::splat(f32::NEG_INFINITY);
    for x in [batch.min.x, batch.max.x] {
        for y in [batch.min.y, batch.max.y] {
            for z in [batch.min.z, batch.max.z] {
                let p = view_from_local.transform_point3(Vec3::new(x, y, z));
                // A box crossing the eye plane has unbounded projection. Keep
                // it whole; do not guess clipping or discard it by player depth.
                if !p.is_finite() || p.z >= -0.001 {
                    return true;
                }
                let p = p.truncate() / -p.z;
                projected_min = projected_min.min(p);
                projected_max = projected_max.max(p);
            }
        }
    }
    // Perspective extrema on a convex box in front of the eye occur at corners.
    // Include every depth, especially internal faces beyond the player that the
    // original reveal leaves visible after discarding a nearer wall surface.
    projected_min.cmple(max).all() && projected_max.cmpge(min).all()
}

fn pixel_padding(camera: &Camera, projection: &Projection) -> Option<Vec2> {
    let Projection::Perspective(projection) = projection else {
        return None;
    };
    let size = camera.physical_viewport_size()?.as_vec2();
    let half_height = (projection.fov * 0.5).tan();
    if !size.is_finite()
        || size.cmple(Vec2::ZERO).any()
        || !half_height.is_finite()
        || half_height <= 0.0
        || !projection.aspect_ratio.is_finite()
        || projection.aspect_ratio <= 0.0
    {
        return None;
    }
    // Two actual viewport pixels for raster sample and arithmetic boundary
    // padding. This is not a world-distance guess or a zoom-dependent cutoff.
    Some(Vec2::new(half_height * projection.aspect_ratio, half_height) * 4.0 / size)
}

pub(super) fn sync(
    status: Res<VoxelViewStatus>,
    cache: Res<TerrainRevisionCache>,
    materials: Res<Assets<VoxelMaterial>>,
    cameras: Query<(&Camera, &Projection, &GlobalTransform), With<VoxelWorldCamera>>,
    terrain_visibility: Query<&Visibility, Without<MazeRevealBatch>>,
    mut batches: Query<
        (&MazeRevealBatch, Ref<GlobalTransform>, &mut Visibility),
        With<MazeRevealBatch>,
    >,
    mut previous: Local<Option<Selection>>,
) {
    // The solid terrain entity is an unparented root (spawn_terrain_entity), so
    // its current Visibility is authoritative before propagation. The batch
    // additionally inherits its own terrain root's visibility in that pass.
    let terrain_visible = cache
        .solid_entity
        .and_then(|entity| terrain_visibility.get(entity).ok())
        .is_some_and(|visibility| *visibility != Visibility::Hidden);
    let selection = if !status.active || !terrain_visible {
        Selection::Hidden
    } else if let Ok((camera, projection, transform)) = cameras.get_single() {
        if !camera.is_active {
            Selection::Hidden
        } else if let Some(uniform) = cache
            .solid_material
            .as_ref()
            .and_then(|handle| materials.get(handle))
            .map(|material| material.extension.cutaway)
        {
            if uniform.bottom_radius.w <= 0.0 {
                Selection::Hidden
            } else if let Some(padding) = pixel_padding(camera, projection) {
                reveal_bounds(uniform, transform.compute_matrix(), padding)
            } else {
                Selection::All
            }
        } else {
            Selection::All
        }
    } else {
        Selection::All
    };
    let changed = previous.as_ref() != Some(&selection);
    *previous = Some(selection);
    for (&batch, transform, mut visibility) in &mut batches {
        if !changed && !transform.is_changed() && !visibility.is_added() {
            continue;
        }
        let next = if may_expose(selection, batch, transform.compute_matrix()) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_batch_selection_keeps_behind_player_and_uncertain_projection() {
        let uniform = CutawayUniform::for_player(Vec3::new(0.0, 0.0, -50.0), 16.0).unwrap();
        let selection = reveal_bounds(uniform, Mat4::IDENTITY, Vec2::splat(0.001));
        // These original internal faces survive the reveal's depth cutoff and
        // must return even though they lie behind the player's capsule.
        let behind = MazeRevealBatch {
            min: Vec3::new(-2.0, 2.0, -70.0),
            max: Vec3::new(2.0, 12.0, -60.0),
        };
        assert!(may_expose(selection, behind, Mat4::IDENTITY));
        let distant = MazeRevealBatch {
            min: Vec3::new(200.0, 2.0, -70.0),
            max: Vec3::new(210.0, 12.0, -60.0),
        };
        assert!(!may_expose(selection, distant, Mat4::IDENTITY));
        let crossing_eye = MazeRevealBatch {
            min: Vec3::new(200.0, 0.0, -1.0),
            max: Vec3::new(210.0, 12.0, 1.0),
        };
        assert!(may_expose(selection, crossing_eye, Mat4::IDENTITY));
        assert!(may_expose(Selection::All, distant, Mat4::IDENTITY));
        assert!(!may_expose(Selection::Hidden, behind, Mat4::IDENTITY));
    }

    #[test]
    fn reveal_batch_projection_preserves_scrolling_orbit_zoom_and_box_interiors() {
        for zoom in [0.0, 1.0, 2.5, 5.0] {
            for orbit in [0.0, 0.25, 1.0, 2.0, 3.5, 4.0, 5.25, 6.0, 7.0] {
                for viewport in [Vec2::new(160.0, 144.0), Vec2::new(480.0, 640.0)] {
                    let camera = crate::camera::VoxelCameraControls::new(zoom, orbit)
                        .pose(viewport)
                        .transform();
                    let world_from_view = camera.compute_matrix();
                    let uniform = CutawayUniform::for_player(Vec3::ZERO, 16.0).unwrap();
                    let selection = reveal_bounds(uniform, world_from_view, Vec2::splat(0.002));
                    let Selection::Bounds {
                        view_from_world,
                        min,
                        max,
                    } = selection
                    else {
                        panic!("expected finite reveal");
                    };
                    for x in -5..=5 {
                        for z in -5..=5 {
                            let batch = MazeRevealBatch {
                                min: Vec3::new(x as f32 * 8.0, 0.0, z as f32 * 8.0),
                                max: Vec3::new(x as f32 * 8.0 + 8.0, 12.0, z as f32 * 8.0 + 8.0),
                            };
                            let selected = may_expose(selection, batch, Mat4::IDENTITY);
                            // Independent interior samples must never enter the
                            // reveal rectangle when the whole batch is rejected.
                            if !selected {
                                for u in [0.0, 0.25, 0.5, 0.75, 1.0] {
                                    for v in [0.0, 0.5, 1.0] {
                                        for w in [0.0, 0.25, 0.5, 0.75, 1.0] {
                                            let p = batch.min
                                                + (batch.max - batch.min) * Vec3::new(u, v, w);
                                            let p = view_from_world.transform_point3(p);
                                            assert!(p.z < -0.001);
                                            let p = p.truncate() / -p.z;
                                            assert!(!(p.cmpge(min).all() && p.cmple(max).all()));
                                        }
                                    }
                                }
                            }
                            let shift = Vec3::new(200.0, 32.0, -160.0);
                            let translated = reveal_bounds(
                                CutawayUniform::for_player(shift, 16.0).unwrap(),
                                Mat4::from_translation(shift) * world_from_view,
                                Vec2::splat(0.002),
                            );
                            assert_eq!(
                                selected,
                                may_expose(translated, batch, Mat4::from_translation(shift))
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn reveal_batches_use_current_propagated_camera_scroll_and_visibility_without_uploads() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::transform::TransformPlugin,
            bevy::render::view::VisibilityPlugin,
        ))
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<VoxelMaterial>>()
        .init_resource::<TerrainRevisionCache>()
        .init_resource::<VoxelViewStatus>();
        register(&mut app);
        app.world_mut().resource_mut::<VoxelViewStatus>().active = true;
        let mut material = crate::solid_terrain_material();
        material.extension.cutaway =
            CutawayUniform::for_player(Vec3::new(0.0, 0.0, -50.0), 16.0).unwrap();
        let handle = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(material);
        let root = app.world_mut().spawn(SpatialBundle::default()).id();
        let main = app.world_mut().spawn(SpatialBundle::default()).id();
        {
            let mut cache = app.world_mut().resource_mut::<TerrainRevisionCache>();
            cache.solid_entity = Some(main);
            cache.solid_material = Some(handle.clone());
        }
        let camera = app
            .world_mut()
            .spawn((
                Camera3dBundle {
                    camera: Camera {
                        viewport: Some(bevy::render::camera::Viewport {
                            physical_size: UVec2::new(1180, 812),
                            ..default()
                        }),
                        ..default()
                    },
                    projection: Projection::Perspective(PerspectiveProjection {
                        aspect_ratio: 1180.0 / 812.0,
                        ..default()
                    }),
                    ..default()
                },
                VoxelWorldCamera,
            ))
            .id();
        let batch = app
            .world_mut()
            .spawn((
                SpatialBundle::default(),
                MazeRevealBatch {
                    min: Vec3::new(-2.0, 2.0, -70.0),
                    max: Vec3::new(2.0, 12.0, -65.0),
                },
            ))
            .id();
        app.world_mut().entity_mut(root).add_child(batch);
        let visible = |app: &App| app.world().get::<InheritedVisibility>(batch).unwrap().get();
        app.update();
        assert!(
            visible(&app),
            "behind-player batch must be restored on its first frame"
        );
        assert_eq!(
            app.world()
                .get::<GlobalTransform>(batch)
                .unwrap()
                .translation(),
            Vec3::ZERO
        );
        // Change only the local camera transform. A stale GlobalTransform would
        // leave the batch visible for one frame, which this assertion rejects.
        *app.world_mut().get_mut::<Transform>(camera).unwrap() =
            Transform::from_xyz(50.0, 0.0, -50.0).looking_at(Vec3::new(0.0, 0.0, -50.0), Vec3::Y);
        app.update();
        assert!(!visible(&app));
        app.world_mut()
            .get_mut::<Transform>(root)
            .unwrap()
            .translation
            .z = 20.0;
        app.update();
        assert!(
            visible(&app),
            "the current propagated scrolling root must restore the batch"
        );
        assert_eq!(
            app.world()
                .get::<GlobalTransform>(batch)
                .unwrap()
                .translation()
                .z,
            20.0
        );
        let visibility_tick = app
            .world()
            .entity(batch)
            .get_ref::<Visibility>()
            .unwrap()
            .last_changed();
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        app.update();
        assert_eq!(
            app.world()
                .entity(batch)
                .get_ref::<Visibility>()
                .unwrap()
                .last_changed(),
            visibility_tick,
            "idle view changed draw state"
        );
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
        *app.world_mut().get_mut::<Visibility>(main).unwrap() = Visibility::Hidden;
        app.update();
        assert!(
            !visible(&app),
            "hidden main terrain cannot leave floating joins"
        );
        *app.world_mut().get_mut::<Visibility>(main).unwrap() = Visibility::Visible;
        app.update();
        assert!(visible(&app));
        *app.world_mut().get_mut::<Visibility>(root).unwrap() = Visibility::Hidden;
        app.update();
        assert!(
            !visible(&app),
            "secondary batches must inherit transition visibility"
        );
        *app.world_mut().get_mut::<Visibility>(root).unwrap() = Visibility::Visible;
        app.update();
        assert!(visible(&app));
        app.world_mut().get_mut::<Camera>(camera).unwrap().is_active = false;
        app.update();
        assert!(!visible(&app));
        app.world_mut().get_mut::<Camera>(camera).unwrap().is_active = true;
        app.world_mut().resource_mut::<VoxelViewStatus>().active = false;
        app.update();
        assert!(
            !visible(&app),
            "classic/battle/failure state cannot leave joins active"
        );
        app.world_mut().resource_mut::<VoxelViewStatus>().active = true;
        app.world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .get_mut(&handle)
            .unwrap()
            .extension
            .cutaway = CutawayUniform::default();
        app.update();
        assert!(!visible(&app), "disabled reveal needs no internal faces");
        app.world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .get_mut(&handle)
            .unwrap()
            .extension
            .cutaway = CutawayUniform::for_player(Vec3::new(0.0, 0.0, -50.0), 16.0).unwrap();
        // An unsupported projection restores all faces rather than guessing.
        *app.world_mut().get_mut::<Projection>(camera).unwrap() =
            Projection::Orthographic(default());
        app.update();
        assert!(visible(&app));
        assert_eq!(
            app.world().resource::<Assets<Mesh>>().len(),
            mesh_count,
            "view selection uploaded geometry"
        );
    }
}
