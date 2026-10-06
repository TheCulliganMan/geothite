//! Explicit art support for a stationary water presentation. These are model
//! waterlines, not species heights, type inference, or swimming animation.
use crate::battle_layout::BattleBody;

pub(super) fn supported_bodies(
    bodies: [Option<BattleBody>; 2],
    species: [Option<&str>; 2],
) -> Option<[Option<BattleBody>; 2]> {
    let mut result = bodies;
    for index in 0..2 {
        let body = result[index].as_mut()?;
        if !body.modeled {
            return None;
        }
        // Authored on the current neutral sculptures: Totodile's torso root
        // (0.218493 on its 1.02-high model), Poliwag's lower rounded body above
        // the feet, and Remoraid's fish-body centerline below the dorsal fin.
        // Keep every original vertex/clip and the pack's physical scale.
        let fraction = match species[index]? {
            "TOTODILE" => 0.218_493 / 1.02,
            "POLIWAG" => 0.36,
            "REMORAID" => 0.34,
            _ => return None,
        };
        body.support_y = body.min.y + (body.max.y - body.min.y) * fraction;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle_layout::{
        model_scale, reference_span, BattleSceneLayout, WORLD_UNITS_PER_METER,
    };
    use bevy::prelude::*;

    fn body(species: &str) -> BattleBody {
        let rig = crate::species_rig::for_species(species);
        let mesh = rig
            .map(|r| r.neutral.clone())
            .or_else(|| crate::battle_species_models::mesh(species))
            .or_else(|| crate::new_bark_actors::actor_props::battle_species_mesh(species))
            .unwrap();
        let min = mesh
            .positions
            .iter()
            .map(|p| Vec3::from_array(*p))
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let max = mesh
            .positions
            .iter()
            .map(|p| Vec3::from_array(*p))
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        let mut body = BattleBody::modeled(
            species,
            min,
            max,
            model_scale(species, Some(0.6096), min, max).unwrap(),
        );
        body.visual_bounds = rig.map(|r| r.animated_bounds);
        body
    }

    #[test]
    fn real_water_stances_preserve_geometry_scale_rigs_and_exact_surface_contact() {
        for enemy in ["POLIWAG", "REMORAID"] {
            let species = [Some("TOTODILE"), Some(enemy)];
            let original = [Some(body("TOTODILE")), Some(body(enemy))];
            let supported = supported_bodies(original, species).unwrap();
            let anchors = [Vec3::new(-4.0, -0.5, 0.0), Vec3::new(4.0, -0.5, 0.0)];
            let layout =
                BattleSceneLayout::for_anchored_bodies(supported, anchors, Vec2::new(800.0, 600.0))
                    .unwrap();
            for index in 0..2 {
                let before = original[index].unwrap();
                let after = supported[index].unwrap();
                assert_eq!(
                    (
                        before.min,
                        before.max,
                        before.scale,
                        before.visual_bounds,
                        before.reference_height
                    ),
                    (
                        after.min,
                        after.max,
                        after.scale,
                        after.visual_bounds,
                        after.reference_height
                    )
                );
                assert!(after.support_y > after.min.y && after.support_y < after.max.y);
                assert!(
                    (reference_span(species[index].unwrap(), after.min, after.max) * after.scale.y
                        / WORLD_UNITS_PER_METER
                        - 0.6096)
                        .abs()
                        < 0.00001
                );
                let pose = layout.body_poses[index].unwrap();
                assert!(pose
                    .transform_point(Vec3::Y * after.support_y)
                    .abs_diff_eq(anchors[index], 0.00001));
                assert!(pose.transform_point(Vec3::Y * after.min.y).y < anchors[index].y);
            }
            assert_eq!(layout.origins, anchors);
            let ground =
                BattleSceneLayout::for_anchored_bodies(original, anchors, Vec2::new(800.0, 600.0))
                    .unwrap();
            for index in 0..2 {
                assert!(ground.body_poses[index]
                    .unwrap()
                    .transform_point(Vec3::Y * original[index].unwrap().min.y)
                    .abs_diff_eq(anchors[index], 0.00001));
            }
        }
    }

    #[test]
    fn water_support_never_infers_swimming_from_a_type_or_ground_model() {
        let body = body("TOTODILE");
        for species in ["CYNDAQUIL", "GYARADOS", "SURF", "", "totodile"] {
            assert!(supported_bodies([Some(body); 2], [Some(species), Some("POLIWAG")]).is_none());
        }
        assert!(
            supported_bodies([None, Some(body)], [Some("TOTODILE"), Some("POLIWAG")]).is_none()
        );
        assert!(supported_bodies(
            [Some(BattleBody::source_card(1.0)), Some(body)],
            [Some("TOTODILE"), Some("POLIWAG")]
        )
        .is_none());
    }
}
