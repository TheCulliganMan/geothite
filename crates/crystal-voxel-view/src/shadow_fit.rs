//! Native shadow-coverage experiment. Disabled until visual and timing checks
//! establish a gain. It never changes geometry, map resolution, or cascade count.
use crate::{VOXEL_RENDER_LAYER, VoxelSun, VoxelWorldCamera, shadow_bounds};
use bevy::{
    camera::{
        primitives::{Aabb, Frustum},
        visibility::RenderLayers,
    },
    light::{
        CascadeShadowConfig, CascadeShadowConfigBuilder, Cascades, NotShadowCaster,
        SimulationLightSystems, cascade::Cascade,
    },
    prelude::*,
};
use crystal_render_api::VisualWorldFrame;

pub(super) fn register(app: &mut App) {
    if std::env::var("CRYSTAL_SHADOW_FIT").as_deref() != Ok("on") {
        return;
    }
    app.init_resource::<Coverage>().add_systems(
        PostUpdate,
        (
            fit.after(bevy::camera::visibility::VisibilitySystems::CheckVisibility)
                .before(SimulationLightSystems::UpdateDirectionalLightCascades),
            extend_depth
                .after(SimulationLightSystems::UpdateDirectionalLightCascades)
                .before(SimulationLightSystems::UpdateLightFrusta),
        ),
    );
}

#[derive(Resource, Default)]
struct Coverage {
    camera: Option<Entity>,
    caster_z: Option<(f32, f32)>,
}

// A linear depth function reaches its extrema at vertices of the convex
// frustum/height-slab intersection: retained frustum corners and edge cuts.
// This bounds all visible geometry without scanning its individual triangles.
fn slab_far(points: [Vec3; 8], bottom: f32, top: f32, eye: Vec3, forward: Vec3) -> Option<f32> {
    if !bottom.is_finite()
        || !top.is_finite()
        || bottom > top
        || !eye.is_finite()
        || !forward.is_finite()
        || points.iter().any(|p| !p.is_finite())
    {
        return None;
    }
    let bottom = f64::from(bottom) - 0.1;
    let top = f64::from(top) + 0.1;
    let mut far = f64::NEG_INFINITY;
    let mut include = |p: bevy::math::DVec3| {
        far = far.max(forward.as_dvec3().dot(p - eye.as_dvec3()));
    };
    for p in points {
        if (bottom..=top).contains(&f64::from(p.y)) {
            include(p.as_dvec3());
        }
    }
    for (a, b) in [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ] {
        let a = points[a].as_dvec3();
        let b = points[b].as_dvec3();
        let dy = b.y - a.y;
        if dy == 0.0 {
            continue;
        }
        for y in [bottom, top] {
            let t = (y - a.y) / dy;
            if (0.0..=1.0).contains(&t) {
                include(a + (b - a) * t);
            }
        }
    }
    far.is_finite().then(|| (far as f32).next_up())
}

fn fit(
    frame: Res<VisualWorldFrame>,
    mut coverage: ResMut<Coverage>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &Frustum), With<VoxelWorldCamera>>,
    assets: Res<Assets<Mesh>>,
    meshes: Query<(
        &Mesh3d,
        &GlobalTransform,
        &InheritedVisibility,
        Option<&Aabb>,
        Option<&RenderLayers>,
        Has<NotShadowCaster>,
    )>,
    mut suns: Query<(&GlobalTransform, &mut CascadeShadowConfig), With<VoxelSun>>,
) {
    coverage.camera = None;
    coverage.caster_z = None;
    if !frame.active || !matches!(frame.map_id.as_ref(), "NewBarkTown" | "Route29") {
        return;
    }
    let Some((entity, _, camera, frustum)) =
        cameras.iter().find(|(_, camera, _, _)| camera.is_active)
    else {
        return;
    };
    let Ok((sun, mut config)) = suns.single_mut() else {
        return;
    };
    // Always restore the stock coverage if any bound is unknown or invalid.
    let stock = CascadeShadowConfigBuilder {
        num_cascades: 3,
        maximum_distance: 2048.0,
        first_cascade_far_bound: 256.0,
        ..default()
    }
    .build();
    if config.bounds != stock.bounds {
        config.bounds = stock.bounds;
    }
    let Some(frustum_corners) = frustum.corners() else {
        return;
    };
    let layer = RenderLayers::layer(VOXEL_RENDER_LAYER);
    let light_from_world = Mat4::from_quat(sun.rotation()).transpose();
    let mut bottom = f32::INFINITY;
    let mut top = f32::NEG_INFINITY;
    let mut caster_min = f32::INFINITY;
    let mut caster_max = f32::NEG_INFINITY;
    for (mesh, transform, visible, bounds, layers, no_shadow) in &meshes {
        if !visible.get() || !layers.is_some_and(|l| l.intersects(&layer)) {
            continue;
        }
        if shadow_bounds::proven_empty(assets.get(&mesh.0)) {
            continue;
        }
        let Some(bounds) = bounds else {
            return;
        };
        for p in shadow_bounds::corners(bounds, transform) {
            if !p.is_finite() {
                return;
            }
            bottom = bottom.min(p.y);
            top = top.max(p.y);
            if !no_shadow {
                let z = light_from_world.transform_point3(p).z;
                caster_min = caster_min.min(z);
                caster_max = caster_max.max(z);
            }
        }
    }
    let Some(far) = slab_far(
        frustum_corners,
        bottom,
        top,
        camera.translation(),
        *camera.forward(),
    ) else {
        return;
    };
    if !caster_min.is_finite() || !caster_max.is_finite() {
        return;
    }
    // Outward bins avoid tiny fit fluctuations, with a generous boundary margin.
    let far = ((far + 128.0) / 128.0)
        .ceil()
        .mul_add(128.0, 0.0)
        .clamp(512.0, 2048.0);
    let fitted = CascadeShadowConfigBuilder {
        num_cascades: 3,
        maximum_distance: far,
        first_cascade_far_bound: 256.0,
        ..default()
    }
    .build();
    config.bounds = fitted.bounds;
    coverage.camera = Some(entity);
    coverage.caster_z = Some((caster_min - 16.0, caster_max + 16.0));
}

// Preserve the stock XY projection and texel snapping exactly. Extend only Z
// to include every caster, including those above/outside the camera frustum.
fn expand_depth(cascade: &mut Cascade, min_z: f32, max_z: f32) -> Option<()> {
    let rotation = Mat4::from_cols(
        cascade.world_from_cascade.x_axis,
        cascade.world_from_cascade.y_axis,
        cascade.world_from_cascade.z_axis,
        Vec4::W,
    );
    let mut center = rotation
        .transpose()
        .transform_point3(cascade.world_from_cascade.w_axis.truncate());
    let old_range = cascade.clip_from_cascade.z_axis.z.recip();
    if !center.is_finite()
        || !old_range.is_finite()
        || old_range <= 0.0
        || !min_z.is_finite()
        || !max_z.is_finite()
        || min_z > max_z
    {
        return None;
    }
    let bottom = (center.z - old_range).min(min_z).next_down();
    let top = center.z.max(max_z).next_up();
    center.z = top;
    let range = top - bottom;
    if !range.is_finite() || range <= 0.0 {
        return None;
    }
    let inverse = rotation.transpose();
    let cascade_from_world = Mat4::from_cols(
        inverse.x_axis,
        inverse.y_axis,
        inverse.z_axis,
        (-center).extend(1.0),
    );
    cascade.world_from_cascade.w_axis = rotation * center.extend(1.0);
    cascade.clip_from_cascade.z_axis.z = range.recip();
    let expanded = cascade.clip_from_cascade * cascade_from_world;
    // Retain the original world-space XY rows verbatim to avoid new shadow swim.
    cascade.clip_from_world.x_axis.z = expanded.x_axis.z;
    cascade.clip_from_world.y_axis.z = expanded.y_axis.z;
    cascade.clip_from_world.z_axis.z = expanded.z_axis.z;
    cascade.clip_from_world.w_axis.z = expanded.w_axis.z;
    Some(())
}

fn extend_depth(coverage: Res<Coverage>, mut suns: Query<&mut Cascades, With<VoxelSun>>) {
    let (Some(camera), Some((min, max))) = (coverage.camera, coverage.caster_z) else {
        return;
    };
    for mut cascades in &mut suns {
        if let Some(cascades) = cascades.cascades.get_mut(&camera) {
            for cascade in cascades {
                let _ = expand_depth(cascade, min, max);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::CameraProjection;
    fn box_points() -> [Vec3; 8] {
        [
            (-1., 1., -1.),
            (1., 1., -1.),
            (1., -1., -1.),
            (-1., -1., -1.),
            (-3., 3., 5.),
            (3., 3., 5.),
            (3., -3., 5.),
            (-3., -3., 5.),
        ]
        .map(|(x, y, z)| Vec3::new(x, y, z))
    }
    #[test]
    fn height_slab_covers_visible_points_for_every_view_direction() {
        let points = box_points();
        for forward in [
            Vec3::X,
            Vec3::Z,
            Vec3::NEG_Z,
            Vec3::new(1., -1., 1.).normalize(),
        ] {
            let far = slab_far(points, -0.4, 0.7, Vec3::ZERO, forward).unwrap();
            for z in 0..=40 {
                let t = z as f32 / 40.0;
                let width = 1.0 + 2.0 * t;
                for x in 0..=20 {
                    for y in 0..=20 {
                        let p = Vec3::new(
                            width * (x as f32 / 10. - 1.),
                            -0.4 + 1.1 * y as f32 / 20.,
                            -1. + 6. * t,
                        );
                        assert!(forward.dot(p) <= far + 0.00001);
                    }
                }
            }
        }
        assert!(slab_far(points, 10., 20., Vec3::ZERO, Vec3::Z).is_none());
        assert!(slab_far(points, f32::NAN, 20., Vec3::ZERO, Vec3::Z).is_none());
    }

    #[test]
    fn perspective_height_fit_covers_receivers_across_all_orbits_and_zoom_levels() {
        let projection = PerspectiveProjection {
            fov: 2.0 * 0.5_f32.atan(),
            near: 24.0,
            far: 6016.0,
            aspect_ratio: 4.0 / 3.0,
            ..default()
        };
        for orbit in 0..8 {
            for zoom in 0..=5 {
                let target = Vec3::new(0., 32., 0.);
                let yaw = Quat::from_rotation_y(orbit as f32 * std::f32::consts::FRAC_PI_4);
                let offset = yaw * Vec3::new(0., 480. / 2.0_f32.sqrt(), 480. / 2.0_f32.sqrt())
                    / (0.75 + zoom as f32 * 0.25);
                let camera = GlobalTransform::from(
                    Transform::from_translation(target + offset).looking_at(target, Vec3::Y),
                );
                let frustum = projection.compute_frustum(&camera);
                let corners = frustum
                    .corners()
                    .expect("production perspective has finite corners");
                let far = slab_far(
                    corners,
                    -8.,
                    267.07,
                    camera.translation(),
                    *camera.forward(),
                )
                .expect("visible height slab intersects the camera view");
                let mut visible = 0;
                for x in -20..=20 {
                    for z in -20..=20 {
                        for y in [-8., 0., 100., 267.07] {
                            let point = Vec3::new(x as f32 * 50., y, z as f32 * 50.);
                            if frustum
                                .half_spaces
                                .iter()
                                .all(|plane| plane.normal_d().dot(point.extend(1.)) >= 0.)
                            {
                                visible += 1;
                                assert!(
                                    camera.forward().dot(point - camera.translation())
                                        <= far + 0.001
                                );
                            }
                        }
                    }
                }
                assert!(visible > 0);
            }
        }
    }
    #[test]
    fn extending_caster_depth_preserves_every_xy_projection_bit() {
        let rotation = Mat4::from_quat(Quat::from_rotation_y(0.6) * Quat::from_rotation_x(-0.8));
        let center = Vec3::new(120., -70., 300.);
        let view = Mat4::from_cols(
            rotation.transpose().x_axis,
            rotation.transpose().y_axis,
            rotation.transpose().z_axis,
            (-center).extend(1.),
        );
        let projection = Mat4::from_cols(
            Vec4::new(0.01, 0., 0., 0.),
            Vec4::new(0., 0.02, 0., 0.),
            Vec4::new(0., 0., 0.01, 0.),
            Vec4::new(0., 0., 1., 1.),
        );
        let mut cascade = Cascade {
            world_from_cascade: rotation * Mat4::from_translation(center),
            clip_from_cascade: projection,
            clip_from_world: projection * view,
            texel_size: 0.125,
        };
        let before = cascade.clip_from_world;
        expand_depth(&mut cascade, -1000., 2000.).unwrap();
        for z in [-1000., 200., 300., 2000.] {
            let world = rotation.transform_point3(Vec3::new(120., -70., z));
            let old = before * world.extend(1.);
            let new = cascade.clip_from_world * world.extend(1.);
            assert_eq!(old.x.to_bits(), new.x.to_bits());
            assert_eq!(old.y.to_bits(), new.y.to_bits());
            assert!((-0.000001..=1.000001).contains(&new.z));
        }
        assert_eq!(cascade.texel_size, 0.125);
    }
    #[test]
    fn invalid_depth_expansion_leaves_cascade_untouched() {
        let mut cascade = Cascade::default();
        let before = cascade.clip_from_world;
        assert!(expand_depth(&mut cascade, f32::NAN, 1.).is_none());
        assert_eq!(cascade.clip_from_world, before);
    }

    #[test]
    fn fitting_keeps_offscreen_casters_and_unknown_geometry_restores_stock() {
        let mut app = App::new();
        let mut frame = VisualWorldFrame::default();
        frame.active = true;
        frame.map_id = "NewBarkTown".into();
        app.insert_resource(frame)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Coverage>()
            .add_systems(Update, fit);
        let mut frustum = Frustum::default();
        frustum.half_spaces = [
            Vec3::X,
            Vec3::NEG_X,
            Vec3::NEG_Y,
            Vec3::Y,
            Vec3::Z,
            Vec3::NEG_Z,
        ]
        .map(|normal| bevy::shape::HalfSpace::new(normal.extend(1.0)));
        let camera = app
            .world_mut()
            .spawn((
                VoxelWorldCamera,
                Camera::default(),
                GlobalTransform::IDENTITY,
                frustum,
            ))
            .id();
        let sun = app
            .world_mut()
            .spawn((
                VoxelSun,
                GlobalTransform::IDENTITY,
                CascadeShadowConfigBuilder {
                    num_cascades: 3,
                    maximum_distance: 2048.,
                    first_cascade_far_bound: 256.,
                    ..default()
                }
                .build(),
            ))
            .id();
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::default());
        app.world_mut().spawn((
            Mesh3d(mesh.clone()),
            GlobalTransform::IDENTITY,
            InheritedVisibility::VISIBLE,
            RenderLayers::layer(VOXEL_RENDER_LAYER),
            Aabb {
                center: Vec3::ZERO.into(),
                half_extents: Vec3::ONE.into(),
            },
        ));
        // Far outside the receiver view; it must still bound shadow depth.
        app.world_mut().spawn((
            Mesh3d(mesh.clone()),
            GlobalTransform::from(Transform::from_xyz(5000., 100., 1000.)),
            InheritedVisibility::VISIBLE,
            RenderLayers::layer(VOXEL_RENDER_LAYER),
            Aabb {
                center: Vec3::ZERO.into(),
                half_extents: Vec3::ONE.into(),
            },
        ));
        app.update();
        let coverage = app.world().resource::<Coverage>();
        assert_eq!(coverage.camera, Some(camera));
        let (min, max) = coverage.caster_z.unwrap();
        assert!(min <= 999. && max >= 1001.);
        assert_eq!(
            app.world()
                .get::<CascadeShadowConfig>(sun)
                .unwrap()
                .bounds
                .len(),
            3
        );
        // A drawable asset lacking a conservative bound must disable fitting.
        app.world_mut().spawn((
            Mesh3d(mesh),
            GlobalTransform::IDENTITY,
            InheritedVisibility::VISIBLE,
            RenderLayers::layer(VOXEL_RENDER_LAYER),
        ));
        app.update();
        assert!(app.world().resource::<Coverage>().caster_z.is_none());
        let stock = CascadeShadowConfigBuilder {
            num_cascades: 3,
            maximum_distance: 2048.,
            first_cascade_far_bound: 256.,
            ..default()
        }
        .build();
        assert_eq!(
            app.world().get::<CascadeShadowConfig>(sun).unwrap().bounds,
            stock.bounds
        );
    }
}
