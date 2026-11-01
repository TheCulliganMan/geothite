//! Opt-in measurement of conservative receiver/caster depth, without changing
//! cascade distances or excluding any geometry from rendering.
use crate::{VOXEL_RENDER_LAYER, VoxelSun, VoxelWorldCamera};
use bevy::{
    camera::{
        primitives::{Aabb, Frustum},
        visibility::RenderLayers,
    },
    light::{CascadeShadowConfig, NotShadowCaster},
    math::DVec3,
    prelude::*,
};

pub(super) fn register(app: &mut App) {
    if std::env::var("CRYSTAL_SHADOW_BOUNDS").as_deref() != Ok("on") {
        return;
    }
    app.add_systems(
        PostUpdate,
        report
            .after(bevy::transform::TransformSystems::Propagate)
            .after(bevy::camera::visibility::VisibilitySystems::CheckVisibility)
            .after(bevy::light::SimulationLightSystems::UpdateLightFrusta),
    );
}

// For a directional light, a caster point q lies on p + t * toward_sun for
// some visible receiver p and t >= 0. Its view depth is therefore bounded by
// receiver_far + max(0, forward.dot(toward_sun)) * (caster_top-receiver_bottom)
// / toward_sun.y. Include every caster's height, including off-screen ones.
fn conservative_far(
    receiver_far: f32,
    receiver_bottom: f32,
    caster_top: f32,
    forward: Vec3,
    toward_sun: Vec3,
) -> Option<f32> {
    if !receiver_far.is_finite()
        || !receiver_bottom.is_finite()
        || !caster_top.is_finite()
        || !forward.is_finite()
        || !toward_sun.is_finite()
        || toward_sun.y <= 0.001
    {
        return None;
    }
    let far = receiver_far
        + forward.dot(toward_sun).max(0.0) * (caster_top - receiver_bottom).max(0.0) / toward_sun.y;
    far.is_finite().then_some(far)
}

pub(super) fn corners(bounds: &Aabb, transform: &GlobalTransform) -> [Vec3; 8] {
    std::array::from_fn(|i| {
        let sign = Vec3::new(
            if i & 1 == 0 { -1.0 } else { 1.0 },
            if i & 2 == 0 { -1.0 } else { 1.0 },
            if i & 4 == 0 { -1.0 } else { 1.0 },
        );
        transform
            .affine()
            .transform_point3(Vec3::from(bounds.center) + Vec3::from(bounds.half_extents) * sign)
    })
}

// Animated terrain retains its CPU asset even for empty flower domains. Those
// zero-vertex placeholders draw nothing; distinguish them from missing bounds
// on a real mesh. An unavailable CPU asset is never proof of an empty mesh.
pub(super) fn proven_empty(mesh: Option<&Mesh>) -> bool {
    mesh.is_some_and(|mesh| {
        // RENDER_WORLD meshes can still have an Assets<Mesh> entry after
        // extraction, but their CPU buffers are no longer accessible. The
        // fallible accessors distinguish that state from a known empty mesh.
        mesh.try_attributes().is_ok_and(|mut attributes| {
            attributes.all(|(_, values)| values.is_empty())
        }) && mesh.try_indices_option().is_ok_and(|indices| {
            indices.is_none_or(|indices| indices.is_empty())
        })
    })
}

// Whole-box far corners can be kilometres outside the visible view for a
// large ground/backdrop draw. Measure the convex box/frustum intersection.
// Its extrema occur on clipped box faces or at frustum corners inside the
// box. This changes only diagnostics, never mesh vertices or draw visibility.
// Invalid/singular input returns None so the caller retains the loose bound.
fn clipped_receiver_bounds(
    bounds: &Aabb,
    transform: &GlobalTransform,
    frustum: &Frustum,
    forward: Vec3,
    eye: Vec3,
) -> Option<(f32, f32)> {
    let frustum_corners = frustum.corners()?;
    if frustum_corners.iter().any(|point| !point.is_finite())
        || frustum
            .half_spaces
            .iter()
            .any(|plane| !plane.normal_d().is_finite())
    {
        return None;
    }
    let inverse = transform.affine().inverse();
    if !inverse.is_finite() {
        return None;
    }
    let points = corners(bounds, transform);
    if points.iter().any(|point| !point.is_finite()) {
        return None;
    }
    let mut far = f64::NEG_INFINITY;
    let mut bottom = f64::INFINITY;
    let mut include = |point: DVec3| {
        far = far.max(forward.as_dvec3().dot(point - eye.as_dvec3()));
        bottom = bottom.min(point.y);
    };
    for corner in frustum_corners {
        let local = inverse.transform_point3(corner);
        let relative = (local - Vec3::from(bounds.center)).abs();
        if relative
            .cmple(Vec3::from(bounds.half_extents) + Vec3::splat(0.01))
            .all()
        {
            include(corner.as_dvec3());
        }
    }
    for face in [
        [0, 2, 6, 4],
        [1, 5, 7, 3],
        [0, 4, 5, 1],
        [2, 3, 7, 6],
        [0, 1, 3, 2],
        [4, 6, 7, 5],
    ] {
        let mut polygon = [DVec3::ZERO; 16];
        for (i, index) in face.into_iter().enumerate() {
            polygon[i] = points[index].as_dvec3();
        }
        let mut count = 4;
        for plane in frustum.half_spaces {
            if count == 0 {
                break;
            }
            let normal = Vec3::from(plane.normal()).as_dvec3();
            // Expand the measurement planes slightly; f64 intersections avoid
            // rounding visible boundary points inward at large coordinates.
            let distance = f64::from(plane.d()) + 0.01;
            let mut output = [DVec3::ZERO; 16];
            let mut output_count = 0;
            let mut previous = polygon[count - 1];
            let mut previous_distance = normal.dot(previous) + distance;
            for &current in &polygon[..count] {
                let current_distance = normal.dot(current) + distance;
                if (previous_distance >= 0.0) != (current_distance >= 0.0) {
                    if output_count == output.len() {
                        return None;
                    }
                    let t = previous_distance / (previous_distance - current_distance);
                    output[output_count] = previous + (current - previous) * t;
                    output_count += 1;
                }
                if current_distance >= 0.0 {
                    if output_count == output.len() {
                        return None;
                    }
                    output[output_count] = current;
                    output_count += 1;
                }
                previous = current;
                previous_distance = current_distance;
            }
            polygon = output;
            count = output_count;
        }
        for point in &polygon[..count] {
            include(*point);
        }
    }
    if !far.is_finite() || !bottom.is_finite() {
        return None;
    }
    // Round outward, including conversion back to the renderer's f32 units.
    Some(((far as f32).next_up(), (bottom as f32).next_down()))
}

fn report(
    time: Res<Time<Real>>,
    mut next: Local<f64>,
    cameras: Query<(&Camera, &GlobalTransform, &Frustum), With<VoxelWorldCamera>>,
    suns: Query<(&GlobalTransform, &CascadeShadowConfig), With<VoxelSun>>,
    assets: Res<Assets<Mesh>>,
    meshes: Query<
        (
            &Mesh3d,
            &GlobalTransform,
            &InheritedVisibility,
            Option<&Aabb>,
            Option<&RenderLayers>,
            Has<NotShadowCaster>,
        ),
        With<Mesh3d>,
    >,
) {
    let now = time.elapsed_secs_f64();
    if now < *next {
        return;
    }
    *next = now + 2.0;
    let Some((_, camera, frustum)) = cameras.iter().find(|(camera, _, _)| camera.is_active) else {
        return;
    };
    let Ok((sun, cascade)) = suns.single() else {
        return;
    };
    let layer = RenderLayers::layer(VOXEL_RENDER_LAYER);
    let forward = *camera.forward();
    let toward_sun = *sun.back();
    let mut missing = 0usize;
    let mut empty = 0usize;
    let mut receivers = 0usize;
    let mut casters = 0usize;
    let mut clipped = 0usize;
    let mut loose_fallbacks = 0usize;
    let mut receiver_far = f32::NEG_INFINITY;
    let mut loose_receiver_far = f32::NEG_INFINITY;
    let mut receiver_bottom = f32::INFINITY;
    let mut caster_top = f32::NEG_INFINITY;
    for (mesh, transform, visibility, bounds, layers, no_shadow) in &meshes {
        if !visibility.get() || !layers.is_some_and(|layers| layers.intersects(&layer)) {
            continue;
        }
        let Some(bounds) = bounds else {
            if proven_empty(assets.get(&mesh.0)) {
                empty += 1;
                continue;
            }
            missing += 1;
            continue;
        };
        let points = corners(bounds, transform);
        if points.iter().any(|point| !point.is_finite()) {
            missing += 1;
            continue;
        }
        if !no_shadow {
            casters += 1;
            caster_top = points
                .iter()
                .fold(caster_top, |top, point| top.max(point.y));
        }
        if frustum.intersects_obb(bounds, &transform.affine(), true, true) {
            receivers += 1;
            let loose_far = points.iter().fold(f32::NEG_INFINITY, |far, point| {
                far.max(forward.dot(*point - camera.translation()))
            });
            let loose_bottom = points
                .iter()
                .fold(f32::INFINITY, |bottom, point| bottom.min(point.y));
            loose_receiver_far = loose_receiver_far.max(loose_far);
            let (far, bottom) = match clipped_receiver_bounds(
                bounds, transform, frustum, forward, camera.translation(),
            ) {
                Some(bounds) => { clipped += 1; bounds }
                None => { loose_fallbacks += 1; (loose_far, loose_bottom) }
            };
            receiver_far = receiver_far.max(far);
            receiver_bottom = receiver_bottom.min(bottom);
        }
    }
    let far = (missing == 0)
        .then(|| {
            conservative_far(
                receiver_far,
                receiver_bottom,
                caster_top,
                forward,
                toward_sun,
            )
        })
        .flatten();
    eprintln!(
        "shadow_bounds receivers={receivers} casters={casters} missing={missing} empty={empty} clipped={clipped} loose_fallbacks={loose_fallbacks} frustum_corners_finite={} cascade_far={:?} loose_receiver_far={loose_receiver_far:.2} receiver_far={receiver_far:.2} receiver_bottom={receiver_bottom:.2} caster_top={caster_top:.2} conservative_far={far:?}",
        frustum.corners().is_some_and(|points| points.iter().all(|point| point.is_finite())),
        cascade.bounds.last()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn unit_frustum() -> Frustum {
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
        frustum
    }

    #[test]
    fn huge_ground_draws_use_the_visible_intersection_not_offscreen_corners() {
        let bounds = Aabb {
            center: Vec3::ZERO.into(),
            half_extents: Vec3::new(1000.0, 0.0, 1000.0).into(),
        };
        let (far, bottom) = clipped_receiver_bounds(
            &bounds,
            &GlobalTransform::IDENTITY,
            &unit_frustum(),
            Vec3::Z,
            Vec3::ZERO,
        )
        .unwrap();
        assert!((1.0..1.02).contains(&far));
        assert!(bottom <= 0.0 && bottom > -0.01);
        // Frustum corners, rather than clipped box faces, bound an enclosing box.
        let enclosing = Aabb {
            half_extents: Vec3::splat(1000.0).into(),
            ..bounds
        };
        let (far, bottom) = clipped_receiver_bounds(
            &enclosing,
            &GlobalTransform::IDENTITY,
            &unit_frustum(),
            Vec3::Z,
            Vec3::ZERO,
        )
        .unwrap();
        assert!(far >= 1.0 && bottom <= -1.0);
    }

    #[test]
    fn clipped_rotated_bounds_keep_every_visible_sample_and_fail_closed_when_singular() {
        let bounds = Aabb {
            center: Vec3::ZERO.into(),
            half_extents: Vec3::new(3.0, 2.0, 1.0).into(),
        };
        let transform = GlobalTransform::from(
            Transform::from_xyz(0.3, 0.4, 0.2)
                .with_rotation(Quat::from_rotation_y(0.7))
                .with_scale(Vec3::new(1.3, 0.8, 2.0)),
        );
        let frustum = unit_frustum();
        let forward = Vec3::new(0.4, -0.3, 0.8).normalize();
        let (far, bottom) =
            clipped_receiver_bounds(&bounds, &transform, &frustum, forward, Vec3::ZERO).unwrap();
        let mut visible = 0;
        for x in 0..=10 {
            for y in 0..=10 {
                for z in 0..=10 {
                    let local = Vec3::from(bounds.half_extents)
                        * (Vec3::new(x as f32, y as f32, z as f32) * 0.2 - Vec3::ONE);
                    let point = transform.affine().transform_point3(local);
                    if frustum
                        .half_spaces
                        .iter()
                        .all(|plane| plane.normal_d().dot(point.extend(1.0)) >= 0.0)
                    {
                        visible += 1;
                        assert!(forward.dot(point) <= far);
                        assert!(point.y >= bottom);
                    }
                }
            }
        }
        assert!(visible > 0);
        let singular = GlobalTransform::from(Transform::from_scale(Vec3::ZERO));
        assert!(
            clipped_receiver_bounds(&bounds, &singular, &frustum, forward, Vec3::ZERO).is_none()
        );
        assert!(
            clipped_receiver_bounds(
                &bounds,
                &transform,
                &Frustum::default(),
                forward,
                Vec3::ZERO
            )
            .is_none()
        );
    }

    #[test]
    fn only_a_present_zero_vertex_asset_proves_an_empty_draw() {
        let empty = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default(),
        );
        assert!(proven_empty(Some(&empty)));
        assert!(!proven_empty(None));
        assert!(!proven_empty(Some(&Mesh::from(Cuboid::default()))));
        let mut invalid = empty;
        invalid.insert_indices(bevy::mesh::Indices::U32(vec![0, 1, 2]));
        assert!(!proven_empty(Some(&invalid)));
    }
    #[test]
    fn meshes_moved_to_the_gpu_are_unknown_not_empty_and_never_panic() {
        let mut drawn = Mesh::from(Cuboid::default());
        let gpu = drawn.take_gpu_data().unwrap();
        assert!(!proven_empty(Some(&drawn)));
        assert!(!proven_empty(Some(&gpu)));
        let mut empty = Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::RENDER_WORLD);
        assert!(proven_empty(Some(&empty)));
        let _gpu = empty.take_gpu_data().unwrap();
        assert!(!proven_empty(Some(&empty)));
    }
    #[test]
    fn depth_bound_includes_offscreen_casters_for_every_camera_direction() {
        let sun = Vec3::new(180.0, 320.0, 220.0).normalize();
        for forward in [
            Vec3::X,
            Vec3::NEG_X,
            Vec3::Z,
            Vec3::NEG_Z,
            Vec3::new(1.0, -1.0, 1.0).normalize(),
            Vec3::new(-1.0, -1.0, -1.0).normalize(),
        ] {
            let eye = Vec3::new(300.0, 500.0, 100.0);
            let receivers = [Vec3::new(-100.0, -5.0, 30.0), Vec3::new(100.0, 70.0, -80.0)];
            let receiver_far = receivers
                .iter()
                .map(|p| forward.dot(*p - eye))
                .fold(f32::NEG_INFINITY, f32::max);
            let bottom = -5.0;
            let top = 180.0;
            let far = conservative_far(receiver_far, bottom, top, forward, sun).unwrap();
            for p in receivers {
                for step in 0..=100 {
                    let t = (top - p.y) / sun.y * step as f32 / 100.0;
                    let caster = p + sun * t;
                    assert!(forward.dot(caster - eye) <= far + 0.0001);
                }
            }
        }
    }
    #[test]
    fn unknown_or_non_upward_light_bounds_fail_closed() {
        for sun in [Vec3::NEG_Y, Vec3::X, Vec3::NAN] {
            assert!(conservative_far(300.0, 0.0, 100.0, Vec3::Z, sun).is_none());
        }
        assert!(conservative_far(f32::INFINITY, 0.0, 100.0, Vec3::Z, Vec3::Y).is_none());
    }
    #[test]
    fn transformed_bounds_cover_scaled_rotated_and_translated_geometry() {
        let bounds = Aabb {
            center: Vec3::new(2.0, 3.0, 4.0).into(),
            half_extents: Vec3::new(5.0, 2.0, 1.0).into(),
        };
        let transform = GlobalTransform::from(
            Transform::from_xyz(8.0, 10.0, -20.0)
                .with_rotation(Quat::from_rotation_y(0.7))
                .with_scale(Vec3::new(2.0, 3.0, 0.5)),
        );
        let points = corners(&bounds, &transform);
        let low = points
            .iter()
            .copied()
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let high = points
            .iter()
            .copied()
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        for x in 0..=4 {
            for y in 0..=4 {
                for z in 0..=4 {
                    let local = Vec3::from(bounds.center)
                        + Vec3::from(bounds.half_extents)
                            * (Vec3::new(x as f32, y as f32, z as f32) * 0.5 - Vec3::ONE);
                    let world = transform.affine().transform_point3(local);
                    assert!(world.cmpge(low - Vec3::splat(0.0001)).all());
                    assert!(world.cmple(high + Vec3::splat(0.0001)).all());
                }
            }
        }
    }
}
