//! Conservative coarse bounds for explicitly owned panorama occluders.
//! Render meshes, source collision and per-object fade behavior remain intact.
use super::{Bounds, OccluderGroup, ray_bounds::conservative_overlap};
use bevy::ecs::entity::EntityHashSet;
use bevy::{math::Affine3A, prelude::*};

#[derive(Component)]
pub(super) struct Member(pub(super) Entity);

#[derive(Component, Default)]
pub(super) struct Region {
    pub(super) members: Vec<Entity>,
    bounds: Option<Bounds>,
    unbounded: bool,
    awake: bool,
    selected: bool,
}
impl Region {
    pub(super) fn include(&mut self, bounds: Option<Bounds>) {
        match bounds {
            Some(next) => {
                if let Some(bounds) = &mut self.bounds {
                    bounds.include(next.min);
                    bounds.include(next.max);
                } else {
                    self.bounds = Some(next);
                }
            }
            None => self.unbounded = true,
        }
    }
}

pub(super) struct Selection {
    pub(super) entities: Vec<Entity>,
    pub(super) waking: EntityHashSet,
    pub(super) changed: bool,
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Regions<'w, 's> {
    ordinary: Query<'w, 's, Entity, (With<OccluderGroup>, Without<Member>)>,
    roots: Query<'w, 's, (&'static GlobalTransform, &'static mut Region)>,
    members: Query<'w, 's, (Entity, &'static Member)>,
    lost_parents: RemovedComponents<'w, 's, ChildOf>,
    lost_regions: RemovedComponents<'w, 's, Region>,
    orphans: Local<'s, EntityHashSet>,
    moved: Query<
        'w,
        's,
        (
            Entity,
            &'static Member,
            &'static ChildOf,
            &'static GlobalTransform,
        ),
        (
            With<OccluderGroup>,
            Or<(Changed<Transform>, Changed<ChildOf>)>,
        ),
    >,
}
impl Regions<'_, '_> {
    pub(super) fn select(
        &mut self,
        rays: Option<Bounds>,
        mut bounds_for: impl FnMut(Entity, Affine3A) -> Option<Bounds>,
    ) -> Selection {
        for entity in self.lost_parents.read() {
            if let Ok((_, member)) = self.members.get(entity) {
                if let Ok((_, mut region)) = self.roots.get_mut(member.0) {
                    region.unbounded = true;
                }
            }
        }
        let lost: EntityHashSet = self.lost_regions.read().collect();
        if !lost.is_empty() {
            self.orphans.extend(
                self.members
                    .iter()
                    .filter_map(|(entity, member)| lost.contains(&member.0).then_some(entity)),
            );
        }
        self.orphans.retain(|entity| {
            self.members
                .get(*entity)
                .is_ok_and(|(_, member)| self.roots.get(member.0).is_err())
        });
        // Only local transforms trigger this scan's yielded updates. A shared
        // region may translate/rotate without touching each member's bounds.
        // Grow conservatively on member motion; never discard its old bounds.
        for (entity, member, parent, world) in &self.moved {
            if let Ok((root, mut region)) = self.roots.get_mut(member.0) {
                if parent.parent() != member.0 {
                    // A reparented member no longer satisfies direct ownership.
                    // Keep its region on the original exact processing path.
                    region.unbounded = true;
                } else {
                    region.include(bounds_for(entity, root.affine().inverse() * world.affine()));
                }
            } else {
                self.orphans.insert(entity);
            }
        }
        let mut selected = Selection {
            entities: self.ordinary.iter().collect(),
            waking: default(),
            changed: false,
        };
        selected.entities.extend(self.orphans.iter().copied());
        for (pose, mut region) in &mut self.roots {
            let bounds = if region.unbounded {
                None
            } else {
                region
                    .bounds
                    .and_then(|bounds| transformed(bounds, pose.affine()))
            };
            let included = region.awake || conservative_overlap(bounds, rays);
            if included != region.selected {
                selected.changed = true;
                if included {
                    selected.waking.extend(region.members.iter().copied());
                }
                region.selected = included;
            }
            if included {
                selected.entities.extend(region.members.iter().copied());
            }
        }
        // Structural edits can move an entity to the ordinary path while an
        // old region still retains its ID. Never advance a fade twice.
        let mut unique = EntityHashSet::default();
        selected.entities.retain(|entity| unique.insert(*entity));
        selected
    }

    pub(super) fn settle(&mut self, mut unsettled: impl FnMut(Entity) -> bool) {
        for (_, mut region) in &mut self.roots {
            if region.selected {
                // A formerly intersecting region must finish all visible fades
                // before sleeping. Hidden fades retain their original state.
                region.awake = region.members.iter().copied().any(&mut unsettled);
            }
        }
    }
}

fn transformed(bounds: Bounds, pose: Affine3A) -> Option<Bounds> {
    if !pose.is_finite() {
        return None;
    }
    let mut world = Bounds::empty();
    for corner in 0..8 {
        let point = pose.transform_point3(Vec3::new(
            if corner & 1 == 0 {
                bounds.min.x
            } else {
                bounds.max.x
            },
            if corner & 2 == 0 {
                bounds.min.y
            } else {
                bounds.max.y
            },
            if corner & 4 == 0 {
                bounds.min.z
            } else {
                bounds.max.z
            },
        ));
        if !point.is_finite() {
            return None;
        }
        world.include(point);
    }
    let margin = world.min.abs().max(world.max.abs()).max(Vec3::ONE) * 0.0001;
    world.min -= margin;
    world.max += margin;
    (world.min.is_finite() && world.max.is_finite()).then_some(world)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transformed_region_encloses_rotated_scaled_and_large_coordinate_points() {
        let source = Bounds {
            min: Vec3::new(-7., -3., 2.),
            max: Vec3::new(12., 8., 11.),
        };
        for scale in [Vec3::ONE, Vec3::new(-2., 0.3, 4.), Vec3::new(0., 2., -1.)] {
            for angle in [0., 0.7, -2.1] {
                let pose = Affine3A::from_scale_rotation_translation(
                    scale,
                    Quat::from_euler(EulerRot::XYZ, angle, angle * 0.7, angle * -0.3),
                    Vec3::new(1_000_000., -250., -800_000.),
                );
                let result = transformed(source, pose).unwrap();
                for x in 0..5 {
                    for y in 0..5 {
                        for z in 0..5 {
                            let local = source.min
                                + (source.max - source.min)
                                    * Vec3::new(x as f32, y as f32, z as f32)
                                    / 4.;
                            let world = pose.transform_point3(local);
                            assert!(result.min.cmple(world).all() && world.cmple(result.max).all());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn invalid_region_proofs_never_reject_a_possible_obstruction() {
        let source = Bounds {
            min: Vec3::ZERO,
            max: Vec3::ONE,
        };
        for translation in [Vec3::splat(f32::NAN), Vec3::splat(f32::INFINITY)] {
            let result = transformed(source, Affine3A::from_translation(translation));
            assert!(result.is_none());
            assert!(conservative_overlap(result, Some(source)));
        }
        assert!(conservative_overlap(Some(source), None));
        let mut region = Region::default();
        region.include(None);
        region.include(Some(source));
        assert!(
            region.unbounded,
            "later bounded members cannot erase an unknown member"
        );
    }
}
