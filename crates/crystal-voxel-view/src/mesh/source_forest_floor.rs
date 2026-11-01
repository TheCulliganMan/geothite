//! Live Forest ground pigment on the existing authored floor, without changing
//! source occupancy, contact shade, map-space grain or litter geometry.
use super::*;

fn source_color(cells: &[&VisualTile], images: &TerrainImageSamples) -> Option<[f32; 3]> {
    for tile in cells {
        if tile.source.tileset_id.as_ref() != "forest" || tile.source.tile_index != 0x05
            || !matches!(shape_for_source(&tile.source), CellShape::Flat) {
            continue;
        }
        let Ok(pixels) = tile_rgba(images, tile) else { continue; };
        if pixels.len() != SOURCE_TILE_PIXELS * SOURCE_TILE_PIXELS * 4
            || pixels.chunks_exact(4).any(|p| p[3] != 255) {
            continue;
        }
        let mut color = [0.; 3];
        for pixel in pixels.chunks_exact(4) {
            for channel in 0..3 {
                let s = f32::from(pixel[channel]) / 255.;
                color[channel] += if s <= 0.04045 { s / 12.92 }
                    else { ((s + 0.055) / 1.055).powf(2.4) };
            }
        }
        for c in &mut color { *c /= (SOURCE_TILE_PIXELS * SOURCE_TILE_PIXELS) as f32; }
        return Some(color);
    }
    None
}

pub(super) fn finish(mesh: &mut TerrainMeshData, cells: &[&VisualTile], images: &TerrainImageSamples) {
    let Some(paint) = source_color(cells, images) else { return; };
    for (color, uv) in mesh.solid.colors.iter_mut().zip(&mut mesh.solid.uvs) {
        if !(65000. ..69000.).contains(&uv[0]) || (-uv[1] / 10000.).round() != 7. {
            continue;
        }
        // Preserve the existing stable contact/weather field, while replacing
        // its brown/green hue. This datum is independent of the camera crop.
        let shade = ((color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722)
            / 0.25).clamp(0.35, 1.3);
        for channel in 0..3 { color[channel] = paint[channel] * shade; }
        // Separate live pigment from unsampled floors, retaining the exact
        // existing grain coordinates. The shader supplies neutral source light.
        uv[0] += 10000.;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn forest_floor_uses_live_pigment_and_keeps_geometry_grain_and_other_roles() {
        let mut tile = VisualTile { animation_frames: None, column: 0, row: 0,
            source: VisualTileSource { tileset_id: Arc::from("forest"), metatile_id: 0,
                subtile_column: 0, subtile_row: 0, tile_index: 5 },
            texture: Handle::default(), priority: false };
        let mut images = TerrainImageSamples::default();
        images.pixels.insert(tile.texture.id(), TileImageSample::Rgba(
            [123,115,198,255].repeat(SOURCE_TILE_PIXELS * SOURCE_TILE_PIXELS)));
        let mut mesh = TerrainMeshData::default();
        mesh.solid.positions = vec![[0.,0.,0.],[1.,0.,0.],[0.,0.,1.]];
        mesh.solid.normals = vec![[0.,1.,0.];3];
        mesh.solid.indices = vec![0,2,1];
        mesh.solid.colors = vec![[0.2,0.25,0.1,1.];3];
        mesh.solid.uvs = vec![[66012.,-69980.],[66013.,-59980.],[85000.,0.]];
        mesh.footing_heights = vec![0.;3];
        let before = mesh.clone();
        finish(&mut mesh, &[&tile], &images);
        assert_eq!(mesh.solid.positions,before.solid.positions);
        assert_eq!(mesh.solid.normals,before.solid.normals);
        assert_eq!(mesh.solid.indices,before.solid.indices);
        assert_eq!(mesh.footing_heights,before.footing_heights);
        assert_eq!(mesh.solid.uvs[0],[76012.,-69980.]);
        assert!(mesh.solid.colors[0][2] > mesh.solid.colors[0][0] * 2.);
        assert_eq!(&mesh.solid.colors[1..],&before.solid.colors[1..]);
        assert_eq!(&mesh.solid.uvs[1..],&before.solid.uvs[1..]);
        let once = mesh.clone(); finish(&mut mesh, &[&tile], &images);
        assert_eq!(mesh,once);
        for source in ["johto","park","kanto"] {
            tile.source.tileset_id=Arc::from(source);
            let mut other=before.clone(); finish(&mut other,&[&tile],&images);
            assert_eq!(other,before);
        }
        tile.source.tileset_id=Arc::from("forest");
        let mut missing=before.clone(); finish(&mut missing,&[&tile],&TerrainImageSamples::default());
        assert_eq!(missing,before);
    }
}
