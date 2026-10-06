//! Presentation-only player reveal shared by authored indoor occluders.
//! The player capsule identifies actual blockers for whole-object fading.
//! The mesh, navigation, support heights, and shadow geometry stay intact.
use bevy::{prelude::*, render::render_resource::ShaderType};

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect, ShaderType)]
pub(super) struct CutawayUniform {
    /// World foot point and body radius; zero radius disables fading.
    pub bottom_radius: Vec4,
    /// World head point and retained body margin in world units.
    pub top_feather: Vec4,
    /// Forward-pass fade amount (zero is opaque); shadow alpha stays original.
    pub fade: Vec4,
}

impl CutawayUniform {
    pub(super) fn for_player(foot: Vec3, model_scale: f32) -> Option<Self> {
        if !foot.is_finite() || !model_scale.is_finite() || model_scale <= 0.0 {
            return None;
        }
        Some(Self {
            bottom_radius: foot.extend(model_scale * 0.65),
            top_feather: (foot + Vec3::Y * model_scale * 1.35).extend(model_scale * 0.10),
            fade: Vec4::ZERO,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

}

/// Native opt-in A/B instrumentation; never changes the default or web reveal.
/// `zero-radius` disables blocker detection; `omit-uv1` removes only the legacy
/// eligibility attribute. Whole-object fading no longer reads UV1 in the shader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DiagnosticMode {
    Normal,
    ZeroRadius,
    OmitUv1,
}

pub(super) fn diagnostic_mode() -> DiagnosticMode {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static MODE: std::sync::OnceLock<DiagnosticMode> = std::sync::OnceLock::new();
        *MODE.get_or_init(|| {
            match std::env::var("CRYSTAL_CUTAWAY_DIAGNOSTIC").as_deref() {
                Ok("zero-radius") => DiagnosticMode::ZeroRadius,
                Ok("omit-uv1") => DiagnosticMode::OmitUv1,
                _ => DiagnosticMode::Normal,
            }
        })
    }
    #[cfg(target_arch = "wasm32")]
    { DiagnosticMode::Normal }
}
