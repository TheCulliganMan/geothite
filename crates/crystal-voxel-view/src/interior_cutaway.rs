//! Presentation-only player reveal shared by authored indoor occluders.
//! A perspective-projected capsule cuts only marked foreground fragments.
//! The mesh, navigation, support heights, and shadow geometry stay intact.
use bevy::{prelude::*, render::render_resource::ShaderType};

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect, ShaderType)]
pub(super) struct CutawayUniform {
    /// World foot point and clear radius; zero radius disables the reveal.
    pub bottom_radius: Vec4,
    /// World head point and the width of the stippled boundary in world units.
    pub top_feather: Vec4,
}

impl CutawayUniform {
    pub(super) fn for_player(foot: Vec3, model_scale: f32) -> Option<Self> {
        if !foot.is_finite() || !model_scale.is_finite() || model_scale <= 0.0 {
            return None;
        }
        Some(Self {
            bottom_radius: foot.extend(model_scale * 0.65),
            top_feather: (foot + Vec3::Y * model_scale * 1.35).extend(model_scale * 0.10),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // CPU reference for the shader's perspective calculation. This exercises
    // depth order and screen projection, not a world-space proximity shortcut.
    fn reveal_at(uniform: CutawayUniform, world_from_view: Mat4, world: Vec3) -> f32 {
        let view = world_from_view.inverse();
        let bottom = view.transform_point3(uniform.bottom_radius.truncate());
        let top = view.transform_point3(uniform.top_feather.truncate());
        let fragment = view.transform_point3(world);
        let radius = uniform.bottom_radius.w;
        if radius <= 0.0 || bottom.z >= -0.001 || top.z >= -0.001 || fragment.z >= -0.001 {
            return 0.0;
        }
        let a = bottom.truncate() / -bottom.z;
        let b = top.truncate() / -top.z;
        let p = fragment.truncate() / -fragment.z;
        let axis = b - a;
        let t = ((p - a).dot(axis) / axis.length_squared().max(0.000001)).clamp(0.0, 1.0);
        let depth = 1.0 / ((1.0 - t) / -bottom.z + t / -top.z);
        if -fragment.z >= depth - radius * 0.01 {
            return 0.0;
        }
        let distance = p.distance(a.lerp(b, t)) * depth;
        let u = ((distance - radius) / uniform.top_feather.w).clamp(0.0, 1.0);
        1.0 - u * u * (3.0 - 2.0 * u)
    }

    #[test]
    fn invalid_player_support_disables_cutaway() {
        assert_eq!(CutawayUniform::default().bottom_radius.w, 0.0);
        for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(CutawayUniform::for_player(Vec3::ZERO, scale).is_none());
        }
        assert!(CutawayUniform::for_player(Vec3::splat(f32::NAN), 16.0).is_none());
    }

    #[test]
    fn player_capsule_scales_from_the_real_foot_without_moving_it() {
        let foot = Vec3::new(200.0, 32.0, -48.0);
        let u = CutawayUniform::for_player(foot, 16.0).unwrap();
        assert_eq!(u.bottom_radius.truncate(), foot);
        assert_eq!(u.top_feather.truncate(), foot + Vec3::Y * 21.6);
        assert_eq!(u.bottom_radius.w, 10.4);
        assert_eq!(u.top_feather.w, 1.6);
    }

    #[test]
    fn only_foreground_obstruction_reveals_at_every_orbit_and_zoom() {
        let foot = Vec3::new(18.0, 7.0, -12.0);
        let u = CutawayUniform::for_player(foot, 16.0).unwrap();
        let center = foot + Vec3::Y * 10.8;
        for distance in [72.0, 144.0, 288.0] {
            for step in 0..32 {
                let yaw = step as f32 * std::f32::consts::TAU / 32.0;
                let direction = Quat::from_rotation_y(yaw) * Vec3::new(0.0, 1.0, 1.0).normalize();
                let eye = center + direction * distance;
                let camera = Transform::from_translation(eye).looking_at(center, Vec3::Y);
                let matrix = camera.compute_matrix();
                // Exact same perspective screen point, before/after the player.
                assert_eq!(reveal_at(u, matrix, center.lerp(eye, 0.3)), 1.0);
                assert_eq!(reveal_at(u, matrix, center - direction * 12.0), 0.0);
                assert_eq!(reveal_at(u, matrix, center), 0.0);
                let side = camera.right().as_vec3() * 40.0;
                assert_eq!(reveal_at(u, matrix, center.lerp(eye, 0.3) + side), 0.0);
            }
        }
    }

    #[test]
    fn scrolling_world_translation_preserves_reveal() {
        let foot = Vec3::new(12.0, 0.0, 4.0);
        let eye = Vec3::new(15.0, 110.0, 130.0);
        let center = foot + Vec3::Y * 10.8;
        let obstruction = center.lerp(eye, 0.35);
        let camera = Transform::from_translation(eye).looking_at(center, Vec3::Y);
        let offset = Vec3::new(200.0, 0.0, -160.0);
        let shifted =
            Transform::from_translation(eye + offset).looking_at(center + offset, Vec3::Y);
        let first = reveal_at(
            CutawayUniform::for_player(foot, 16.0).unwrap(),
            camera.compute_matrix(),
            obstruction,
        );
        let second = reveal_at(
            CutawayUniform::for_player(foot + offset, 16.0).unwrap(),
            shifted.compute_matrix(),
            obstruction + offset,
        );
        assert_eq!(first, 1.0);
        assert_eq!(second, first);
    }
}
