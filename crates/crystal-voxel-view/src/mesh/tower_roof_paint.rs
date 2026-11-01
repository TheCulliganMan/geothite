//! Live ceramic pigment for the already authenticated native pitched roof.
//! Geometry, source ownership and the ridge's walking datum stay unchanged.
use super::*;

fn roof(p: &Placement) -> bool {
    matches!(p.form, Form::Special(extension::Detail::RoofSlope { .. } | extension::Detail::RoofRidge))
}

fn pigment(p: &Placement, cells: &[&VisualTile], g: &GridGeometry,
    images: &TerrainImageSamples) -> Option<[f32; 4]> {
    if !roof(p) || g.width == 0 || cells.len() != g.width * g.height
        || p.column + p.width > g.width || p.row + p.height > g.height { return None; }
    let source = &cells[p.row * g.width + p.column].source;
    let (x, y, width, height) = match p.form {
        Form::Special(extension::Detail::RoofSlope { .. }) => {
            let x = p.column.checked_sub(source.subtile_column as usize)?;
            let y = p.row.checked_sub(source.subtile_row as usize)?;
            let art = match source.metatile_id {
                0x1c => [0x16, 0x24, 0x16, 0x06],
                0x23 => [0x25, 0x35, 0x34, 0x35],
                _ => return None,
            };
            if x + 4 > g.width || y + 4 > g.height || !(0..16).all(|i| {
                let (dx, dy) = (i % 4, i / 4);
                let s = &cells[(y + dy) * g.width + x + dx].source;
                s.tileset_id.as_ref() == "tower" && s.metatile_id == source.metatile_id
                    && s.subtile_column as usize == dx && s.subtile_row as usize == dy
                    && s.tile_index == art[dx]
            }) { return None; }
            (x, y, 4, 4)
        }
        Form::Special(extension::Detail::RoofRidge) => {
            if p.width != 2 || p.height != 1 || !matches!(source.metatile_id, 0x1b | 0x2d)
                || !(0..2).all(|dx| {
                    let s = &cells[p.row * g.width + p.column + dx].source;
                    s.tileset_id.as_ref() == "tower" && s.metatile_id == source.metatile_id
                        && s.subtile_row == source.subtile_row
                        && s.subtile_column as usize == source.subtile_column as usize + dx
                        && s.tile_index == 0x50 + dx as u16
                }) { return None; }
            (p.column, p.row, 2, 1)
        }
        _ => return None,
    };
    let mut colors = std::collections::BTreeSet::new();
    for row in y..y + height { for column in x..x + width {
        let rgba = tile_rgba(images, cells[row * g.width + column]).ok()?;
        if rgba.len() != 256 || rgba.chunks_exact(4).any(|p| p[3] != 255) { return None; }
        for pixel in rgba.chunks_exact(4) { colors.insert([pixel[0], pixel[1], pixel[2]]); }
    }}
    if !(2..=4).contains(&colors.len()) { return None; }
    let mut colors: Vec<_> = colors.into_iter().collect();
    colors.sort_by_key(|c| u32::from(c[0]) * 2126 + u32::from(c[1]) * 7152 + u32::from(c[2]) * 722);
    let linear = |c: [u8; 3]| c.map(|v| {
        let s = f32::from(v) / 255.;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    });
    // The darkest ink belongs to joints, and a fourth, brightest entry to
    // source highlights. Keep the ceramic field between its two body tones.
    let mid = linear(colors[if colors.len() >= 3 { 1 } else { 0 }]);
    let light = linear(colors[if colors.len() == 4 { 2 } else { colors.len() - 1 }]);
    Some([mid[0] * 0.4 + light[0] * 0.6, mid[1] * 0.4 + light[1] * 0.6,
        mid[2] * 0.4 + light[2] * 0.6, 1.])
}

fn paint(mesh: &mut SurfaceMeshData, first: usize, pigment: [f32; 4],
    p: &Placement, cells: &[&VisualTile], g: &GridGeometry) {
    let source = &cells[p.row * g.width + p.column].source;
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let base = 0.29 * 0.2126 + 0.40 * 0.7152 + 0.43 * 0.0722;
    for i in first..mesh.positions.len() {
        let old = mesh.colors[i];
        let shade = (old[0] * 0.2126 + old[1] * 0.7152 + old[2] * 0.0722) / base;
        mesh.colors[i] = [pigment[0] * shade, pigment[1] * shade, pigment[2] * shade, old[3]];
        let point = mesh.positions[i];
        mesh.uvs[i] = [
            (source.subtile_column as f32 + (point[0] - w) / g.tile_width) * 0.5,
            crate::scenery_finish::TOWN_PAINT_BASE
                + (source.subtile_row as f32 + (point[2] - n) / g.tile_height) * 0.5,
        ];
    }
}

pub(in crate::mesh) fn append_all_with_samples(mesh: &mut TerrainMeshData, g: &GridGeometry,
    placements: &[Placement], cells: &[&VisualTile], origin: [i32; 2],
    images: Option<&TerrainImageSamples>) {
    let Some(images) = images.filter(|_| placements.iter().any(roof)) else {
        append_all(mesh, g, placements, cells, origin); return;
    };
    // Preserve the connected-body path for all ordinary scenery. Only native
    // roof volumes get a separate, bounded vertex range for their live finish.
    let others: Vec<_> = placements.iter().filter(|p| !roof(p)).cloned().collect();
    append_all(mesh, g, &others, cells, origin);
    for p in placements.iter().filter(|p| roof(p)) {
        let first = mesh.solid.positions.len();
        append(mesh, g, p, cells, origin);
        if let Some(pigment) = pigment(p, cells, g, images) {
            paint(&mut mesh.solid, first, pigment, p, cells, g);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Vec<VisualTile>, GridGeometry, Vec<Placement>, TerrainImageSamples) {
        let g = GridGeometry { width: 5, height: 4, tile_width: 8., tile_height: 8., origin_x: -20., origin_z: -16. };
        let mut assets = Assets::<Image>::default();
        let mut images = TerrainImageSamples::default();
        let mut cells = Vec::new();
        for y in 0..4 { for x in 0..5 {
            let texture = assets.add(Image::default());
            let colors = [[25, 20, 8, 255], [110, 65, 12, 255], [212, 158, 48, 255], [245, 220, 120, 255]];
            images.pixels.insert(texture.id(), TileImageSample::Rgba(
                (0..64).flat_map(|i| colors[i % 4]).collect()));
            cells.push(VisualTile { column: x as u32, row: y as u32, texture,
                priority: false, animation_frames: None,
                source: VisualTileSource { tileset_id: "tower".into(),
                    metatile_id: if x < 4 { 0x1c } else { 2 },
                    tile_index: if x < 4 { [0x16, 0x24, 0x16, 6][x] } else { 2 },
                    subtile_column: (x % 4) as u8, subtile_row: y as u8 } });
        }}
        let placements = resolve("TinTowerRoof", &cells.iter().collect::<Vec<_>>(), &g, None)
            .into_iter().filter(roof).collect::<Vec<_>>();
        assert_eq!(placements.len(), 16);
        (cells, g, placements, images)
    }

    #[test]
    fn tower_roof_live_paint_preserves_geometry_prefix_and_standing_support() {
        let (cells, g, placements, images) = fixture();
        let refs = cells.iter().collect::<Vec<_>>();
        let mut initial = TerrainMeshData { footing_heights: vec![2.5; cells.len()], ..Default::default() };
        append(&mut initial, &g, &placements[0], &refs, [0, 0]);
        let prefix = initial.solid.positions.len();
        let mut before = initial.clone();
        let mut after = initial.clone();
        append_all_with_samples(&mut before, &g, &placements, &refs, [0, 0], None);
        append_all_with_samples(&mut after, &g, &placements, &refs, [0, 0], Some(&images));
        assert_eq!(after.solid.positions, before.solid.positions);
        assert_eq!(after.solid.normals, before.solid.normals);
        assert_eq!(after.solid.indices, before.solid.indices);
        assert_eq!(after.solid.cutaway_ranges, before.solid.cutaway_ranges);
        assert_eq!(after.textured, before.textured);
        assert_eq!(after.footing_heights, before.footing_heights);
        assert_eq!(after.solid.colors[..prefix], initial.solid.colors);
        assert_eq!(after.solid.uvs[..prefix], initial.solid.uvs);
        assert!(after.solid.colors[prefix..].iter().all(|c| c[0] > c[1] && c[1] > c[2]));
        assert!(after.solid.uvs[prefix..].iter().all(|&uv| crate::scenery_finish::town_paint_kind(uv) == Some(0)));
        let mut translated = g.clone(); translated.origin_x += 128.; translated.origin_z -= 64.;
        let mut moved = TerrainMeshData::default();
        append_all_with_samples(&mut moved, &translated, &placements, &refs, [16, -8], Some(&images));
        assert_eq!(moved.solid.colors, after.solid.colors[prefix..]);
        let drift = moved.solid.uvs.iter().zip(&after.solid.uvs[prefix..])
            .flat_map(|(a, b)| [ (a[0] - b[0]).abs(), (a[1] - b[1]).abs() ])
            .fold(0.0_f32, f32::max);
        assert!(drift < 0.00001, "translated ceramic coordinates drifted by {drift}");
    }

    #[test]
    fn tower_roof_pigment_tracks_live_art_and_rejects_incomplete_or_unknown_paint() {
        let (cells, g, placements, images) = fixture();
        let refs = cells.iter().collect::<Vec<_>>();
        let day = pigment(&placements[0], &refs, &g, &images).unwrap();
        let mut changed = images.clone();
        for sample in changed.pixels.values_mut() {
            if let TileImageSample::Rgba(rgba) = sample {
                for pixel in rgba.chunks_exact_mut(4) { pixel[0] /= 2; pixel[1] /= 2; }
            }
        }
        assert_ne!(pigment(&placements[0], &refs, &g, &changed).unwrap(), day);
        for i in 0..16 {
            let index = i / 4 * g.width + i % 4;
            let mut bad = cells.clone(); bad[index].source.tile_index ^= 0x80;
            assert!(pigment(&placements[0], &bad.iter().collect::<Vec<_>>(), &g, &images).is_none());
        }
        for fault in 0..4 {
            let mut bad = cells.clone(); let mut samples = images.clone();
            match fault {
                0 => bad[0].source.tileset_id = "mansion".into(),
                1 => { samples.pixels.remove(&cells[0].texture.id()); },
                2 => if let TileImageSample::Rgba(p) = samples.pixels.get_mut(&cells[0].texture.id()).unwrap() { p[3] = 0; },
                _ => if let TileImageSample::Rgba(p) = samples.pixels.get_mut(&cells[0].texture.id()).unwrap() { p[..4].copy_from_slice(&[5, 40, 70, 255]); },
            }
            assert!(pigment(&placements[0], &bad.iter().collect::<Vec<_>>(), &g, &samples).is_none());
        }
    }

    #[test]
    fn tower_roof_ridge_uses_its_own_live_pigment_and_keeps_its_closed_volume() {
        let (mut cells, mut g, _, images) = fixture();
        g.width = 2; g.height = 1; cells.truncate(2);
        for (i, tile) in cells.iter_mut().enumerate() {
            tile.source.metatile_id = 0x1b; tile.source.tile_index = 0x50 + i as u16;
        }
        let p = Placement { column: 0, row: 0, width: 2, height: 1, ground: 0,
            form: Form::Special(extension::Detail::RoofRidge), rise_pixels: 3., depth_pixels: 8., front_rows: 1. };
        let refs = cells.iter().collect::<Vec<_>>();
        let mut original = TerrainMeshData::default();
        let mut painted = TerrainMeshData::default();
        append(&mut original, &g, &p, &refs, [0, 0]);
        append_all_with_samples(&mut painted, &g, &[p.clone()], &refs, [0, 0], Some(&images));
        assert_eq!(painted.solid.positions, original.solid.positions);
        assert_eq!(painted.solid.normals, original.solid.normals);
        assert_eq!(painted.solid.indices, original.solid.indices);
        assert_ne!(painted.solid.colors, original.solid.colors);
        assert!(painted.solid.colors.iter().all(|c| c[0] > c[1] && c[1] > c[2]));
        cells[1].source.tile_index = 2;
        assert!(pigment(&p, &cells.iter().collect::<Vec<_>>(), &g, &images).is_none());
    }
}
