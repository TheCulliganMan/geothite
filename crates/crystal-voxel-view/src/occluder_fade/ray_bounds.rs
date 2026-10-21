//! Conservative broad phase for complete occluder sight-line tests.
use super::{Bounds, CollisionSurface, interior_npc_reveal};
use bevy::prelude::*;

// Bound the complete ray bundle once, then reject distant objects before
// transforming nine or more endpoints or touching their triangle trees.
// Unknown/nonfinite bounds always retain the exact original collision path.
pub(super) fn obstruction_ray_bounds(
    rays: Option<&(Vec3, [Vec3; 9])>,
    pit: &[Vec3],
    people: &[Vec3],
    furniture: &[interior_npc_reveal::Person],
) -> Option<Bounds> {
    let (eye, body) = rays?;
    let mut result = Bounds::empty();
    for point in std::iter::once(eye)
        .chain(body.iter())
        .chain(pit)
        .chain(people)
        .chain(furniture.iter().flat_map(|person| person.rays[3..].iter()))
    {
        if !point.is_finite() {
            return None;
        }
        result.include(*point);
    }
    padded_ray_bounds(result)
}
fn padded_ray_bounds(mut bounds: Bounds) -> Option<Bounds> {
    if !bounds.min.is_finite() || !bounds.max.is_finite() {
        return None;
    }
    // Include transform rounding at large panorama coordinates. Looser bounds
    // can only retain extra exact tests; they cannot hide an obstruction.
    let margin = bounds.min.abs().max(bounds.max.abs()).max(Vec3::ONE) * 0.0001;
    bounds.min -= margin;
    bounds.max += margin;
    (bounds.min.is_finite() && bounds.max.is_finite()).then_some(bounds)
}
pub(super) fn transformed_occluder_bounds(
    surface: &CollisionSurface,
    overlay: Option<&CollisionSurface>,
    pose: bevy::math::Affine3A,
) -> Option<Bounds> {
    if !pose.is_finite() {
        return None;
    }
    let mut source = Bounds::empty();
    for bounds in std::iter::once(surface)
        .chain(overlay)
        .filter_map(|surface| surface.bounds)
    {
        if !bounds.min.is_finite() || !bounds.max.is_finite() {
            return None;
        }
        source.include(bounds.min);
        source.include(bounds.max);
    }
    if !source.min.is_finite() || !source.max.is_finite() {
        return None;
    }
    let mut world = Bounds::empty();
    for i in 0..8 {
        let corner = Vec3::new(
            if i & 1 == 0 {
                source.min.x
            } else {
                source.max.x
            },
            if i & 2 == 0 {
                source.min.y
            } else {
                source.max.y
            },
            if i & 4 == 0 {
                source.min.z
            } else {
                source.max.z
            },
        );
        let point = pose.transform_point3(corner);
        if !point.is_finite() {
            return None;
        }
        world.include(point);
    }
    padded_ray_bounds(world)
}
pub(super) fn conservative_overlap(object: Option<Bounds>, rays: Option<Bounds>) -> bool {
    match (object, rays) {
        (Some(object), Some(rays)) => {
            object.min.cmple(rays.max).all() && rays.min.cmple(object.max).all()
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{interior_cutaway::CutawayUniform, mesh::SurfaceMeshData};
    fn make_surface(x: f32) -> CollisionSurface {
        CollisionSurface::deferred(&SurfaceMeshData {
            positions: vec![
                [x - 16., 0., 20.],
                [x + 16., 0., 20.],
                [x + 16., 32., 20.],
                [x - 16., 32., 20.],
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            ..Default::default()
        })
    }
    #[test]
    fn conservative_bounds_keep_exact_affine_and_overlay_hits() {
        let surface = make_surface(0.);
        let overlay = make_surface(96.);
        let poses = [
            bevy::math::Affine3A::IDENTITY,
            GlobalTransform::from(
                Transform::from_xyz(100000., -320., -64000.).with_scale(Vec3::new(-2., 0.75, 3.)),
            )
            .mul_transform(Transform::from_rotation(Quat::from_rotation_y(0.6)))
            .affine(),
            GlobalTransform::from(
                Transform::from_rotation(Quat::from_rotation_x(0.7))
                    .with_scale(Vec3::new(1., 2., 0.5)),
            )
            .mul_transform(Transform::from_rotation(Quat::from_rotation_z(-0.4)))
            .affine(),
        ];
        let mut hits = 0;
        for pose in poses {
            let bounds = transformed_occluder_bounds(&surface, Some(&overlay), pose);
            for x in [
                -100., -16., -8., 0., 8., 16., 80., 88., 96., 104., 112., 200.,
            ] {
                for y in [-1., 0., 8., 16., 32., 40.] {
                    let start = Vec3::new(x, y, 80.);
                    let end = Vec3::new(x, y, 0.);
                    if !(surface.blocks(start, end) || overlay.blocks(start, end)) {
                        continue;
                    }
                    hits += 1;
                    let world_start = pose.transform_point3(start);
                    let world_end = pose.transform_point3(end);
                    let rays =
                        obstruction_ray_bounds(Some(&(world_start, [world_end; 9])), &[], &[], &[]);
                    assert!(
                        conservative_overlap(bounds, rays),
                        "an exact triangle hit cannot be rejected"
                    );
                }
            }
        }
        assert!(hits > 20);
        let distant = obstruction_ray_bounds(
            Some(&(Vec3::splat(1000.), [Vec3::splat(1100.); 9])),
            &[],
            &[],
            &[],
        );
        assert!(!conservative_overlap(
            transformed_occluder_bounds(&surface, None, poses[0]),
            distant
        ));
    }
    #[test]
    fn pit_people_and_furniture_rays_remain_in_the_bundle() {
        let rays = (Vec3::ZERO, [Vec3::ONE; 9]);
        let pit = [Vec3::new(-300., 20., 40.)];
        let people = [Vec3::new(50., -600., 30.)];
        let furniture = [interior_npc_reveal::Person {
            body: CutawayUniform::default(),
            rays: [Vec3::new(80., 90., 1200.); 9],
        }];
        let bounds = obstruction_ray_bounds(Some(&rays), &pit, &people, &furniture).unwrap();
        for point in [rays.0, rays.1[0], pit[0], people[0], furniture[0].rays[3]] {
            assert!(point.cmpge(bounds.min).all() && point.cmple(bounds.max).all());
        }
    }
    #[test]
    fn missing_and_nonfinite_bounds_keep_the_exact_path() {
        assert!(conservative_overlap(None, None));
        let object =
            transformed_occluder_bounds(&make_surface(0.), None, bevy::math::Affine3A::IDENTITY);
        let rays = obstruction_ray_bounds(Some(&(Vec3::ZERO, [Vec3::NAN; 9])), &[], &[], &[]);
        assert!(rays.is_none());
        assert!(conservative_overlap(object, rays));
        assert!(
            transformed_occluder_bounds(
                &make_surface(0.),
                None,
                bevy::math::Affine3A::from_translation(Vec3::NAN)
            )
            .is_none()
        );
    }
}
