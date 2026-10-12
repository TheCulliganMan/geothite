//! Crystal palette variants of immutable authored geometry. Palette anchors
//! arrive from the local pack; no duplicate species palette catalog lives here.
use crystal_render_api::VisualPokemonModelPalette;
use crate::mesh::SurfaceMeshData;

fn linear(value: u8) -> f32 {
    let value = f32::from(value) / 255.;
    if value <= 0.04045 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) }
}

/// Match a modeled region to its closest normal Crystal palette anchor, then
/// transfer that anchor's appearance. Keep authored tone variations, ink,
/// cream and highlights rather than tinting the entire creature one color.
fn color(color: [f32; 4], palette: VisualPokemonModelPalette) -> [f32; 4] {
    if palette.normal == palette.appearance { return color; }
    let normal = palette.normal.map(|rgb| rgb.map(linear));
    let index = (0..4).min_by(|a, b| {
        let distance = |i: usize| (0..3).map(|c| (color[c] - normal[i][c]).powi(2)).sum::<f32>();
        distance(*a).total_cmp(&distance(*b))
    }).unwrap();
    if palette.normal[index] == palette.appearance[index] { return color; }
    let appearance = palette.appearance[index].map(linear);
    let luminance = |rgb: [f32; 3]| rgb[0] * 0.2126 + rgb[1] * 0.7152 + rgb[2] * 0.0722;
    let tone = (luminance([color[0], color[1], color[2]]) / luminance(normal[index]).max(0.001)).clamp(0.65, 1.35);
    [ (appearance[0] * tone).min(1.), (appearance[1] * tone).min(1.),
      (appearance[2] * tone).min(1.), color[3] ]
}

pub(crate) fn mesh(data: &SurfaceMeshData, palette: VisualPokemonModelPalette) -> SurfaceMeshData {
    let mut variant = data.clone();
    variant.colors = data.colors.iter().map(|c| color(*c, palette)).collect();
    variant
}

#[cfg(test)]
mod tests {
    use super::*;
    fn palette() -> VisualPokemonModelPalette {
        VisualPokemonModelPalette {
            normal: [[255;3], [90,160,230], [24,65,140], [0;3]],
            appearance: [[255;3], [230,70,65], [140,25,24], [0;3]],
        }
    }
    #[test]
    fn pack_palette_changes_blue_regions_but_preserves_ink_cream_alpha_and_geometry() {
        let mut source = SurfaceMeshData::default();
        source.positions = vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]];
        source.indices = vec![0,1,2];
        source.normals = vec![[0.,0.,1.];3];
        source.colors = vec![[0.01,0.01,0.01,0.5], [0.9,0.85,0.7,1.], [90,160,230].map(linear).into_iter().chain([1.]).collect::<Vec<_>>().try_into().unwrap()];
        let original = source.clone();
        let variant = mesh(&source, palette());
        assert_eq!(source, original);
        assert_eq!(variant.positions, source.positions);
        assert_eq!(variant.normals, source.normals);
        assert_eq!(variant.indices, source.indices);
        assert_eq!(variant.colors[..2], source.colors[..2]);
        assert!(variant.colors[2][0] > variant.colors[2][2] * 3.);
        assert_eq!(variant.colors[0][3], 0.5);
        let mut neutral = palette(); neutral.appearance = neutral.normal;
        assert_eq!(mesh(&source, neutral), source);
    }
}
