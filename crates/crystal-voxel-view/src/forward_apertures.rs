//! QA-only removal of provably inactive forward fragment apertures.
//! The legacy shader remains the default until GPU and active-aperture checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct ForwardApertures {
    pub cave: bool,
    pub grass: bool,
}

impl ForwardApertures {
    pub const fn legacy() -> Self { Self { cave: true, grass: true } }

    pub fn for_values(enabled: bool, cave: f32, radius: f32, clearance: [f32; 2]) -> Self {
        if !enabled { return Self::legacy(); }
        Self {
            // Match the shader's early-return predicate, including NaNs. A
            // positive-form >= comparison would incorrectly suppress NaNs.
            cave: !(cave < 0.5 || radius <= 0.0),
            grass: clearance.into_iter().any(|radius| radius > 0.0),
        }
    }
}

pub(super) fn enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("CRYSTAL_FORWARD_APERTURES").as_deref() == Ok("specialized"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_runtime_predicates_preserve_infinities_nan_and_active_apertures() {
        let values = [f32::NEG_INFINITY, -1.0, -0.0, 0.0, 0.49, 0.5, 1.0,
            f32::INFINITY, f32::from_bits(0x7fc01234)];
        for cave in values { for radius in values {
            for a in values { for b in values {
                let specialized = ForwardApertures::for_values(true, cave, radius, [a, b]);
                assert_eq!(specialized.cave, !(cave < 0.5 || radius <= 0.0));
                assert_eq!(specialized.grass, a > 0.0 || b > 0.0);
                assert_eq!(ForwardApertures::for_values(false, cave, radius, [a, b]), ForwardApertures::legacy());
            }}
        }}
    }

    #[test]
    fn actual_shader_keeps_original_predicates_inside_specialized_regions() {
        let shader = include_str!("voxel_surface.wesl");
        assert!(shader.contains("if terrain.cave_reveal.x < 0.5 || terrain.cutaway.bottom_radius.w <= 0.0 { return; }"));
        assert_eq!(shader.matches("@if(VOXEL_CAVE_APERTURE)").count(), 1);
        assert_eq!(shader.matches("@if(VERTEX_UVS_A && VOXEL_GRASS_APERTURE)").count(), 1);
        let cave = shader.split("@if(VOXEL_CAVE_APERTURE) {").nth(1).unwrap()
            .split("// A flat transparent shadow").next().unwrap();
        assert_eq!(cave.matches("reveal_cave_player(").count(), 3);
        let grass = shader.split("@if(VERTEX_UVS_A && VOXEL_GRASS_APERTURE) {").nth(1).unwrap()
            .split("@if(VOXEL_SOURCE_ALPHA_CUTOFF)").next().unwrap();
        assert!(grass.contains("if clearance.w > 0.0 {"));
        assert!(grass.contains("if grass_view.z > center_view.z - depth {"));
        assert_eq!(grass.matches("discard;").count(), 1);
    }
}
