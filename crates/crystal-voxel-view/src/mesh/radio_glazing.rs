//! Explicit radio glass roles share the source atlas and the scenery blend path.
//! Source ownership, wall framing, openings and support stay with the mesher.
use super::*;

pub(super) fn mark(surface: &mut SurfaceMeshData, first: usize, opacity: f32) {
    debug_assert!(opacity == 0.05 || opacity == 0.12);
    for color in &mut surface.colors[first..] {
        color[3] = opacity;
    }
}
fn pane(color: [f32; 4]) -> bool {
    color[3] == 0.05 || color[3] == 0.12
}

pub(super) fn publish(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    samples: Option<&TerrainImageSamples>,
) {
    if !matches!(
        map,
        "RadioTower1F"
            | "RadioTower2F"
            | "RadioTower3F"
            | "RadioTower4F"
            | "RadioTower5F"
            | "LavRadioTower1F"
    ) || !mesh.textured.colors.iter().any(|&c| pane(c))
    {
        return;
    }
    let atlas = samples.and_then(|samples| {
        let mut atlas = vec![0; g.width * g.height * 8 * 8 * 4];
        for (i, tile) in cells.iter().enumerate() {
            let pixels = tile_rgba(samples, tile).ok()?;
            for y in 0..8 {
                let at = (((i / g.width) * 8 + y) * g.width * 8 + (i % g.width) * 8) * 4;
                atlas[at..at + 32].copy_from_slice(&pixels[y * 32..y * 32 + 32]);
            }
        }
        Some(crate::scenery_surfaces::SceneryTexture::live_glazing(
            (g.width * 8) as u32,
            (g.height * 8) as u32,
            atlas,
        ))
    });
    let Some(texture) = atlas else {
        // An incomplete live atlas must retain an opaque source drawing.
        for c in &mut mesh.textured.colors {
            if pane(*c) {
                c[3] = 1.;
            }
        }
        return;
    };
    let mut glass = SurfaceMeshData::default();
    let mut opaque = Vec::with_capacity(mesh.textured.indices.len());
    for face in mesh.textured.indices.chunks_exact(3) {
        if face.iter().all(|&i| pane(mesh.textured.colors[i as usize])) {
            for &i in face {
                let i = i as usize;
                glass.indices.push(glass.positions.len() as u32);
                glass.positions.push(mesh.textured.positions[i]);
                glass.normals.push(mesh.textured.normals[i]);
                glass.uvs.push(mesh.textured.uvs[i]);
                glass.colors.push(mesh.textured.colors[i]);
            }
        } else {
            opaque.extend_from_slice(face);
        }
    }
    mesh.textured.indices = opaque;
    // Keep vertex ranges stable for the existing whole-object fade ownership.
    // Only referenced glass triangles move to the transparent draw group.
    if !glass.indices.is_empty() {
        mesh.scenery.push(crate::scenery_surfaces::ScenerySurface {
            mesh: glass,
            texture,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        TerrainMeshData,
        VisualTile,
        GridGeometry,
        TerrainImageSamples,
    ) {
        let mut mesh = TerrainMeshData::default();
        mesh.textured = SurfaceMeshData {
            positions: vec![
                [0., 0., 0.],
                [1., 0., 0.],
                [0., 1., 0.],
                [2., 0., 0.],
                [3., 0., 0.],
                [2., 1., 0.],
            ],
            normals: vec![[0., 0., 1.]; 6],
            uvs: vec![[0.5, 0.5]; 6],
            colors: vec![[1.; 4]; 6],
            indices: vec![0, 1, 2, 3, 4, 5],
            cutaway_ranges: vec![0..6],
        };
        mark(&mut mesh.textured, 3, 0.12);
        mesh.footing_heights = vec![7.];
        let tile = VisualTile {
            column: 0,
            row: 0,
            texture: Default::default(),
            priority: false,
            animation_frames: None,
            source: VisualTileSource {
                tileset_id: "radio_tower".into(),
                metatile_id: 0x15,
                subtile_column: 0,
                subtile_row: 0,
                tile_index: 0x30,
            },
        };
        let mut samples = TerrainImageSamples::default();
        let mut pixels = vec![0; 8 * 8 * 4];
        for (i, p) in pixels.chunks_exact_mut(4).enumerate() {
            p.copy_from_slice(&[i as u8, 180, 220, 255]);
        }
        samples
            .pixels
            .insert(tile.texture.id(), TileImageSample::Rgba(pixels));
        (
            mesh,
            tile,
            GridGeometry {
                width: 1,
                height: 1,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
            samples,
        )
    }
    #[test]
    fn named_panes_move_to_live_blend_without_moving_framing_or_support() {
        let (mut mesh, tile, g, samples) = fixture();
        let original = mesh.textured.clone();
        publish(&mut mesh, "LavRadioTower1F", &[&tile], &g, Some(&samples));
        assert_eq!(mesh.textured.indices, vec![0, 1, 2]);
        assert_eq!(mesh.textured.positions, original.positions);
        assert_eq!(mesh.textured.cutaway_ranges, original.cutaway_ranges);
        assert_eq!(mesh.footing_heights, vec![7.]);
        assert_eq!(mesh.scenery.len(), 1);
        let glass = &mesh.scenery[0];
        assert_eq!(glass.mesh.positions, original.positions[3..]);
        assert_eq!(glass.mesh.indices, vec![0, 1, 2]);
        assert!(glass.mesh.colors.iter().all(|c| c[3] == 0.12));
        assert!(matches!(
            glass.texture.alpha_mode(),
            bevy::prelude::AlphaMode::Blend
        ));
        assert!(glass.texture.double_sided);
        assert!(
            glass
                .texture
                .image()
                .data
                .as_ref()
                .unwrap()
                .starts_with(tile_rgba(&samples, &tile).unwrap()),
            "base atlas pixels stay exact before generated mip levels"
        );
        assert_eq!(
            tile_rgba(&samples, &tile).unwrap()[3],
            255,
            "source pixels remain immutable"
        );
    }
    #[test]
    fn incomplete_pixels_retain_opaque_source_and_other_rooms_keep_their_policy() {
        for invalid in [false, true] {
            let (mut mesh, tile, g, mut samples) = fixture();
            let indices = mesh.textured.indices.clone();
            if invalid {
                samples
                    .pixels
                    .insert(tile.texture.id(), TileImageSample::Invalid);
            }
            publish(
                &mut mesh,
                "RadioTower5F",
                &[&tile],
                &g,
                invalid.then_some(&samples),
            );
            assert_eq!(mesh.textured.indices, indices);
            assert!(mesh.scenery.is_empty());
            assert!(mesh.textured.colors.iter().all(|c| c[3] == 1.));
        }
        let (mut mesh, tile, g, samples) = fixture();
        let original = mesh.textured.clone();
        publish(&mut mesh, "PlayersHouse1F", &[&tile], &g, Some(&samples));
        assert_eq!(mesh.textured, original);
        assert!(mesh.scenery.is_empty());
    }
}
