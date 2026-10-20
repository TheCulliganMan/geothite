//! One source-owned floating course for Johto and Johto Modern.
//! Float paint comes from the existing atlas; its bed uses ordinary water.
use super::*;

pub(super) const LABEL: &str = "outdoor/buoy-course";

pub(super) fn source(s: &VisualTileSource) -> bool {
    matches!(s.tileset_id.as_ref(), "johto" | "johto_modern")
        && s.tile_index == 0x58
        && s.subtile_column < 4
        && s.subtile_row < 4
        && (((0x30..=0x39).contains(&s.metatile_id)
            && matches!(shape_for_source(s),CellShape::PlaneAt {height} if height == crate::profile::WATER_HEIGHT))
            || (s.tileset_id.as_ref()=="johto" && s.metatile_id==0x07
                && s.subtile_column>=2 && matches!(shape_for_source(s),CellShape::Flat))
            || (s.metatile_id==0x3a && (s.subtile_column>=2 || s.subtile_row>=2)
                && matches!(shape_for_source(s),CellShape::Flat)))
}

// The dominant color is the water background. Keep only the largest connected
// foreground component: isolated wave marks must not become extra floats.
fn silhouette(pixels: &[u8]) -> Option<([usize; 4], [usize; 2])> {
    if pixels.len() != 256 {
        return None;
    }
    let mut counts = std::collections::BTreeMap::new();
    for p in pixels.chunks_exact(4) {
        let p: [u8; 4] = p.try_into().ok()?;
        if p[3] != 255 {
            return None;
        }
        *counts.entry(p).or_insert(0usize) += 1;
    }
    let background = counts.into_iter().max_by_key(|(_, count)| *count)?.0;
    let mut seen = [false; 64];
    let mut largest = Vec::new();
    for start in 0..64 {
        if seen[start] || pixels[start * 4..start * 4 + 4] == background {
            continue;
        }
        let mut component = Vec::new();
        let mut pending = vec![start];
        seen[start] = true;
        while let Some(i) = pending.pop() {
            component.push(i);
            let x = i % 8;
            let z = i / 8;
            for (nx, nz) in [
                (x as isize - 1, z as isize),
                (x as isize + 1, z as isize),
                (x as isize, z as isize - 1),
                (x as isize, z as isize + 1),
            ] {
                if !(0..8).contains(&nx) || !(0..8).contains(&nz) {
                    continue;
                }
                let next = nz as usize * 8 + nx as usize;
                if !seen[next] && pixels[next * 4..next * 4 + 4] != background {
                    seen[next] = true;
                    pending.push(next);
                }
            }
        }
        if component.len() > largest.len() {
            largest = component;
        }
    }
    if largest.len() < 4 || largest.len() > 48 {
        return None;
    }
    let b = [
        largest.iter().map(|i| i % 8).min()?,
        largest.iter().map(|i| i % 8 + 1).max()?,
        largest.iter().map(|i| i / 8).min()?,
        largest.iter().map(|i| i / 8 + 1).max()?,
    ];
    let luminance = |i: usize| -> u32 {
        let p = &pixels[i * 4..i * 4 + 4];
        u32::from(p[0]) * 3 + u32::from(p[1]) * 6 + u32::from(p[2])
    };
    let bright = *largest.iter().max_by_key(|i| luminance(**i))?;
    let mid = largest
        .iter()
        .copied()
        .filter(|i| luminance(*i) < luminance(bright))
        .max_by_key(|i| luminance(*i))
        .unwrap_or(bright);
    Some((b, [bright, mid]))
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    images: &TerrainImageSamples,
    reserved: &[bool],
) {
    let water = CellShape::Water.surface_height(g.tile_height);
    // Only a native, complete, unclaimed horizontal tile is replaceable.
    // Borrowed samples, cropped planes and custom live profiles retain their art.
    let mut planes = Vec::new();
    for (q, (p, uv)) in mesh
        .textured
        .positions
        .chunks_exact(4)
        .zip(mesh.textured.uvs.chunks_exact(4))
        .enumerate()
    {
        let Some(i) = new_bark::sampled_cell(uv, g) else {
            continue;
        };
        if !source(&cells[i].source)
            || reserved[i]
            || mesh.authored_cells.get(i).is_some_and(|c| c.is_some())
        {
            continue;
        }
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        let original_height=shape_for_source(&cells[i].source).surface_height(g.tile_height);
        let expected = [[w, original_height, s], [e, original_height, s],
            [e, original_height, n], [w, original_height, n]];
        if !p.iter().all(|v| expected.contains(v)) || expected.iter().any(|v| !p.contains(v)) {
            continue;
        }
        if mesh.textured.normals[q * 4..q * 4 + 4]
            .iter()
            .any(|n| n[1] < 0.999)
        {
            continue;
        }
        planes.push((q, i));
    }
    let mut consumed = std::collections::HashSet::new();
    for (q, i) in planes {
        if consumed.contains(&i) {
            continue;
        }
        let Some(donor) = cells.iter().position(|t| {
            t.source.tileset_id == cells[i].source.tileset_id
                && t.source.tile_index == 0x14
                && matches!(shape_for_source(&t.source), CellShape::Water)
        }) else {
            continue;
        };
        let Ok(pixels) = tile_rgba(images, cells[i]) else {
            continue;
        };
        let Some((b, paint)) = silhouette(pixels) else {
            continue;
        };
        let (u0, u1, v0, v1) = g.uv(i % g.width, i / g.width);
        let swatches = paint.map(|p| {
            [
                u0 + (u1 - u0) * (p % 8) as f32 / 8. + (u1 - u0) / 16.,
                v0 + (v1 - v0) * (p / 8) as f32 / 8. + (v1 - v0) / 16.,
            ]
        });
        let (w, _, n, _) = g.bounds(i % g.width, i / g.width);
        let center = Vec3::new(
            w + (b[0] + b[1]) as f32 / 16. * g.tile_width,
            water,
            n + (b[2] + b[3]) as f32 / 16. * g.tile_height,
        );
        let radius =
            ((b[1] - b[0]) as f32 * g.tile_width).min((b[3] - b[2]) as f32 * g.tile_height) / 16.
                * 0.90;
        float(&mut mesh.textured, center, radius, g.tile_height, swatches);
        let (a, b, c, d) = g.uv(donor % g.width, donor / g.width);
        mesh.textured.uvs[q * 4..q * 4 + 4].copy_from_slice(&[[a, d], [b, d], [b, c], [a, c]]);
        for p in &mut mesh.textured.positions[q*4..q*4+4] {p[1]=water;}
        mesh.authored_cells[i] = Some(LABEL);
        consumed.insert(i);
    }
    let raised=consumed.iter().copied().filter(|&i|
        shape_for_source(&cells[i].source).surface_height(g.tile_height)>water+0.001)
        .collect::<std::collections::HashSet<_>>();
    retire_raised_banks(&mut mesh.textured,&raised,cells,g,water);
    if !raised.is_empty() {
        let old=std::mem::take(&mut mesh.solid.indices);
        for tri in old.chunks_exact(3) {
            let p=[mesh.solid.positions[tri[0] as usize],mesh.solid.positions[tri[1] as usize],
                mesh.solid.positions[tri[2] as usize]];
            if !raised_bank_face(&p,mesh.solid.normals[tri[0] as usize],&raised,cells,g,water) {
                mesh.solid.indices.extend_from_slice(tri);
            }
        }
    }
}

fn raised_bank_face(p: &[[f32;3]], normal: [f32;3], raised: &std::collections::HashSet<usize>,
    cells: &[&VisualTile], g: &GridGeometry, water: f32) -> bool {
    if normal[1].abs()>0.0001 {return false;}
    let center=p.iter().map(|p|Vec3::from_array(*p)).sum::<Vec3>()/p.len() as f32
        -Vec3::from_array(normal)*0.001;
    let col=((center.x-g.origin_x)/g.tile_width).floor() as i32;
    let row=((center.z-g.origin_z)/g.tile_height).floor() as i32;
    if col<0 || row<0 || col>=g.width as i32 || row>=g.height as i32 {return false;}
    let i=row as usize*g.width+col as usize;
    if !raised.contains(&i) {return false;}
    let (w,e,n,s)=g.bounds(col as usize,row as usize);
    let top=shape_for_source(&cells[i].source).surface_height(g.tile_height);
    p.iter().all(|p|p[0]>=w-0.001 && p[0]<=e+0.001 && p[2]>=n-0.001 && p[2]<=s+0.001
        && p[1]>=water-0.001 && p[1]<=top+0.001)
        && p.iter().any(|p|p[1]>water+0.001)
}

fn retire_raised_banks(surface: &mut SurfaceMeshData, raised: &std::collections::HashSet<usize>,
    cells: &[&VisualTile], g: &GridGeometry, water: f32) {
    if raised.is_empty() {return;}
    let mut remove=std::collections::HashSet::new();
    for (q,p) in surface.positions.chunks_exact(4).enumerate() {
        if raised_bank_face(p,surface.normals[q*4],raised,cells,g,water) {remove.insert(q);}
    }
    if remove.is_empty() {return;}
    let old=std::mem::take(surface);
    let mut remap=vec![u32::MAX;old.positions.len()];
    for (i,index) in remap.iter_mut().enumerate() {
        if remove.contains(&(i/4)) {continue;}
        *index=surface.positions.len() as u32;
        surface.positions.push(old.positions[i]);surface.normals.push(old.normals[i]);
        surface.uvs.push(old.uvs[i]);surface.colors.push(old.colors[i]);
    }
    for tri in old.indices.chunks_exact(3) {
        let ids=[remap[tri[0] as usize],remap[tri[1] as usize],remap[tri[2] as usize]];
        if ids.iter().all(|&i|i!=u32::MAX) {surface.indices.extend(ids);}
    }
    for range in old.cutaway_ranges {
        let mut kept=remap[range].iter().copied().filter(|&i|i!=u32::MAX);
        if let Some(start)=kept.next() {
            let end=kept.last().unwrap_or(start)+1;
            surface.cutaway_ranges.push(start as usize..end as usize);
        }
    }
}

fn float(out: &mut SurfaceMeshData, center: Vec3, radius: f32, tile: f32, paint: [[f32; 2]; 2]) {
    // A rounded floating collar with water visible through its center. Retain
    // the source silhouette, waterline envelope and live white/blue pigment.
    const SIDES: usize = 16;
    const SECTIONS: usize = 8;
    let sample = |side: usize, section: usize| {
        // Wrap indices before trig so both seams have identical float bits.
        let (s, c) = (std::f32::consts::TAU * (side % SIDES) as f32 / SIDES as f32).sin_cos();
        let (v, r) =
            (std::f32::consts::TAU * (section % SECTIONS) as f32 / SECTIONS as f32).sin_cos();
        let p = center
            + Vec3::new(
                c * radius * (0.70 + 0.30 * r),
                tile * (0.05 + 0.15 * v),
                s * radius * (0.70 + 0.30 * r),
            );
        // Elliptical cross-section: normals follow its actual world dimensions.
        let n = Vec3::new(
            c * r / (radius * 0.30),
            v / (tile * 0.15),
            s * r / (radius * 0.30),
        )
        .normalize();
        (p.to_array(), n.to_array())
    };
    for side in 0..SIDES {
        for section in 0..SECTIONS {
            let vertices = [
                (side, section),
                (side, section + 1),
                (side + 1, section + 1),
                (side + 1, section),
            ]
            .map(|(s, t)| sample(s, t));
            let first = out.positions.len();
            let swatch = if side == 0 || side == SIDES / 2 || section >= SECTIONS / 2 {
                paint[1]
            } else {
                paint[0]
            };
            let colors = vertices.map(|(_, n)| {
                let n = Vec3::from_array(n);
                let x = direction_shade(if n.x < 0. {
                    Direction::West
                } else {
                    Direction::East
                });
                let z = direction_shade(if n.z < 0. {
                    Direction::North
                } else {
                    Direction::South
                });
                let y = if n.y > 0. {
                    1.
                } else {
                    direction_shade(Direction::North)
                };
                let shade = (x * n.x.abs() + z * n.z.abs() + y * n.y.abs())
                    / (n.x.abs() + n.y.abs() + n.z.abs());
                [shade * 0.90, shade * 0.90, shade * 0.90, 1.]
            });
            append_quad_colors(
                out,
                vertices.map(|v| v.0),
                vertices[0].1,
                [crate::scenery_finish::source_atlas_uv(swatch); 4],
                colors,
            );
            for (n, v) in out.normals[first..].iter_mut().zip(vertices) {
                *n = v.1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec2;
    fn pixels() -> Vec<u8> {
        let mut p = [20, 70, 120, 255].repeat(64);
        for z in 2..6 {
            for x in 2..6 {
                p[(z * 8 + x) * 4..(z * 8 + x + 1) * 4].copy_from_slice(&[170, 170, 170, 255]);
            }
        }
        p[(3 * 8 + 3) * 4..(3 * 8 + 4) * 4].copy_from_slice(&[255, 255, 255, 255]);
        p[0..4].copy_from_slice(&[255, 255, 255, 255]); // detached wave highlight
        p
    }
    #[test]
    fn silhouette_ignores_detached_waves_and_preserves_source_swatches() {
        let (bounds, paint) = silhouette(&pixels()).unwrap();
        assert_eq!(bounds, [2, 6, 2, 6]);
        assert_eq!(paint[0], 27);
        assert_ne!(paint[1], 0);
        assert!(silhouette(&[0, 0, 255, 255].repeat(64)).is_none());
        assert!(silhouette(&[0; 255]).is_none());
    }
    fn fixture() -> (
        Vec<VisualTile>,
        GridGeometry,
        TerrainImageSamples,
        TerrainMeshData,
    ) {
        let f = super::super::tests::frame(
            2,
            1,
            vec![
                super::super::tests::source_with_tile(0x34, 0, 0, 0x58),
                super::super::tests::source_with_tile(0x34, 1, 0, 0x14),
            ],
        );
        let g = GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut samples = TerrainImageSamples::default();
        samples
            .pixels
            .insert(f.tiles[0].texture.id(), TileImageSample::Rgba(pixels()));
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; 2],
            footing_heights: vec![0., crate::profile::WATER_HEIGHT],
            ..Default::default()
        };
        append_top(&mut mesh.textured, [0., 8., 0., 8.], crate::profile::WATER_HEIGHT, g.uv(0, 0));
        append_top(&mut mesh.textured, [8., 16., 0., 8.], crate::profile::WATER_HEIGHT, g.uv(1, 0));
        (f.tiles, g, samples, mesh)
    }
    #[test]
    fn final_buoy_corner_block_uses_both_verified_johto_atlases() {
        for atlas in ["johto","johto_modern"] {
            let (mut tiles,g,samples,mut mesh)=fixture();
            tiles[0].source.tileset_id=atlas.into();
            tiles[1].source.tileset_id=atlas.into();
            tiles[0].source.metatile_id=0x3a;
            tiles[0].source.subtile_row=2;
            assert!(source(&tiles[0].source));
            for p in &mut mesh.textured.positions[..4] {p[1]=0.;}
            let footing=mesh.footing_heights.clone();
            append(&mut mesh,&tiles.iter().collect::<Vec<_>>(),&g,&samples,&[false;2]);
            assert_eq!(mesh.authored_cells[0],Some(LABEL));
            assert_eq!(mesh.footing_heights,footing);
            assert!(mesh.textured.positions[..4].iter().all(|p|p[1]==crate::profile::WATER_HEIGHT));
            tiles[0].source.subtile_row=0;
            assert!(!source(&tiles[0].source),"the water quadrant is not a float");
        }
    }

    #[test]
    fn whirlpool_block_buoys_replace_their_flat_art_without_moving_footing() {
        let (mut tiles,g,samples,mut mesh)=fixture();
        tiles[0].source.metatile_id=0x07;
        tiles[0].source.subtile_column=2;
        assert!(source(&tiles[0].source));
        for p in &mut mesh.textured.positions[..4] {p[1]=0.;}
        append_textured_shoreline(&mut mesh.textured,&g,0,0,Direction::East,
            [0.,8.,0.,8.],crate::profile::WATER_HEIGHT,0.);
        append_top(&mut mesh.textured,[24.,32.,0.,8.],3.,g.uv(1,0));
        mesh.textured.cutaway_ranges.push(12..16);
        append_quad_colors(&mut mesh.solid,[[0.,crate::profile::WATER_HEIGHT,8.],[0.,0.,8.],[0.,0.,0.],[0.,crate::profile::WATER_HEIGHT,0.]],
            [-1.,0.,0.],[[0.;2];4],[[0.3,0.2,0.1,1.];4]);
        append_quad_colors(&mut mesh.solid,[[24.,3.,8.],[32.,3.,8.],[32.,3.,0.],[24.,3.,0.]],
            [0.,1.,0.],[[0.;2];4],[[0.4,0.4,0.4,1.];4]);
        mesh.solid.cutaway_ranges.push(4..8);
        let solid_kept=mesh.solid.indices[6..].to_vec();
        let footing=mesh.footing_heights.clone();
        append(&mut mesh,&tiles.iter().collect::<Vec<_>>(),&g,&samples,&[false;2]);
        assert_eq!(mesh.authored_cells[0],Some(LABEL));
        assert_eq!(mesh.footing_heights,footing);
        assert!(mesh.textured.positions[..4].iter().all(|p|p[1]==crate::profile::WATER_HEIGHT));
        assert!(mesh.textured.positions.len()>8,"a native float replaces the source glyph");
        assert_eq!(mesh.textured.cutaway_ranges,vec![8..12],"unrelated cutaways survive compaction");
        assert_eq!(mesh.solid.indices,solid_kept,"only the false solid bank is retired");
        assert_eq!(mesh.solid.cutaway_ranges,vec![4..8]);
        assert!(mesh.textured.positions.contains(&[24.,3.,8.]));
        assert!(!mesh.textured.positions.chunks_exact(4).enumerate().any(|(q,p)|
            mesh.textured.normals[q*4][1].abs()<0.0001
                && p.iter().all(|p|p[0]==8.) && p.iter().any(|p|p[1]>crate::profile::WATER_HEIGHT)),
            "the old buoy tile cannot leave a vertical raster bank");
        tiles[0].source.subtile_column=0;
        assert!(!source(&tiles[0].source),"the whirlpool quadrant is not a buoy");
        tiles[0].source.subtile_column=2;
        tiles[0].source.tileset_id="johto_modern".into();
        assert!(!source(&tiles[0].source),"do not infer Modern block07 art");
    }

    #[test]
    fn native_course_reuses_water_bed_and_source_paint_without_moving_footing() {
        let (tiles, g, samples, mut mesh) = fixture();
        let refs: Vec<_> = tiles.iter().collect();
        append(&mut mesh, &refs, &g, &samples, &[false; 2]);
        assert_eq!(mesh.footing_heights, vec![0., crate::profile::WATER_HEIGHT]);
        assert_eq!(mesh.authored_cells, vec![Some(LABEL), None]);
        assert_eq!(new_bark::sampled_cell(&mesh.textured.uvs[..4], &g), Some(1));
        let float = &mesh.textured.positions[8..];
        assert!(!float.is_empty());
        assert!(
            float
                .iter()
                .all(|p| p[0] > 0. && p[0] < 8. && p[2] > 0. && p[2] < 8.)
        );
        assert!(float.iter().any(|p| p[1] < crate::profile::WATER_HEIGHT));
        assert!(float.iter().any(|p| p[1] > -1.5),"the float emerges above its water bed");
        assert!(
            mesh.textured.uvs[8..]
                .iter()
                .all(|uv| crate::scenery_finish::source_atlas_coordinates(*uv)
                    .is_some_and(|uv|new_bark::sampled_cell(&[uv], &g) == Some(0)))
        );
        assert_eq!(mesh.textured.positions.len() % 4, 0);
        let before = mesh.textured.positions.len();
        append(&mut mesh, &refs, &g, &samples, &[false; 2]);
        assert_eq!(
            mesh.textured.positions.len(),
            before,
            "the same cell cannot be claimed twice"
        );
        new_bark::polish_world_surfaces(&mut mesh, "CherrygroveCity", &refs, &g, [0, 0]);
        assert!(mesh.solid.positions.contains(&[0., crate::profile::WATER_HEIGHT, 8.]));
        assert!(
            !mesh
                .textured
                .positions
                .chunks_exact(4)
                .any(|q| q.iter().all(|p| p[1] == crate::profile::WATER_HEIGHT)),
            "old buoy and water planes must use the shared water finish"
        );
        assert_eq!(mesh.footing_heights, vec![0., crate::profile::WATER_HEIGHT]);
        assert!(
            mesh.textured.positions.iter().any(|p| p[1] > -1.5),
            "the float survives ground finishing"
        );
        assert!(mesh.textured.positions.iter().zip(&mesh.textured.uvs)
            .filter(|(p,_)|p[1]>-1.5)
            .all(|(_,uv)|crate::scenery_finish::source_atlas_coordinates(*uv).is_some()),
            "exposed float paint retains its live palette through ground finishing");
    }
    #[test]
    fn cropped_reserved_unknown_and_missing_art_retain_the_original_plane() {
        for kind in 0..5 {
            let (mut tiles, g, samples, mut mesh) = fixture();
            let mut reserved = [false; 2];
            match kind {
                0 => reserved[0] = true,
                1 => mesh.textured.positions[0][0] += 1.,
                2 => tiles[0].source.tile_index = 0x57,
                3 => tiles[1].source.tileset_id = std::sync::Arc::from("kanto"),
                _ => {}
            }
            let samples = if kind == 4 {
                TerrainImageSamples::default()
            } else {
                samples
            };
            let refs: Vec<_> = tiles.iter().collect();
            let original = mesh.textured.positions.clone();
            let uv = mesh.textured.uvs.clone();
            append(&mut mesh, &refs, &g, &samples, &reserved);
            assert_eq!(mesh.textured.positions, original);
            assert_eq!(mesh.textured.uvs, uv);
            assert_eq!(mesh.authored_cells, vec![None; 2]);
        }
    }
    #[test]
    fn float_retains_its_envelope_and_native_paint_with_a_real_open_center() {
        let mut mesh = SurfaceMeshData::default();
        let center = Vec3::new(4., crate::profile::WATER_HEIGHT, 4.);
        let paint = [[0.1, 0.2], [0.3, 0.4]];
        float(&mut mesh, center, 2., 8., paint);
        let low = mesh
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min);
        let high = mesh
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            (low + 2.8).abs() < 0.0001 && (high + 0.4).abs() < 0.0001,
            "the collar retains the original waterline envelope"
        );
        assert!(
            mesh.positions
                .iter()
                .all(|p| p[0] >= 2. && p[0] <= 6. && p[2] >= 2. && p[2] <= 6.),
            "the native silhouette still owns the whole footprint"
        );
        assert!(
            mesh.positions
                .iter()
                .all(|p| Vec2::new(p[0] - center.x, p[2] - center.z).length() >= 0.799),
            "open water must remain visible through the center"
        );
        for swatch in paint {
            assert!(
                mesh.positions
                    .iter()
                    .zip(&mesh.uvs)
                    .any(|(p, uv)| p[1] > center.y
                        && crate::scenery_finish::source_atlas_coordinates(*uv).is_some_and(|p| p
                            .into_iter()
                            .zip(swatch)
                            .all(|(a, b)| (a - b).abs() < 0.0001))),
                "both native paint roles remain visible above the water"
            );
        }
    }

    #[test]
    fn float_is_closed_consistently_wound_and_non_degenerate() {
        let mut mesh = SurfaceMeshData::default();
        float(
            &mut mesh,
            Vec3::new(4., crate::profile::WATER_HEIGHT, 4.),
            2.,
            8.,
            [[0.1, 0.2], [0.3, 0.4]],
        );
        let key = |p: [f32; 3]| p.map(|v| (v * 10000.).round() as i32);
        let mut edges = std::collections::BTreeMap::new();
        let mut volume = 0.;
        for triangle in mesh.indices.chunks_exact(3) {
            let ids: [u32; 3] = triangle.try_into().unwrap();
            let p = ids.map(|i| Vec3::from_array(mesh.positions[i as usize]));
            assert!((p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 1e-10);
            volume += p[0].dot(p[1].cross(p[2])) / 6.;
            for j in 0..3 {
                let a = key(p[j].to_array());
                let b = key(p[(j + 1) % 3].to_array());
                let (edge, direction) = if a < b { ((a, b), 1) } else { ((b, a), -1) };
                let e = edges.entry(edge).or_insert((0, 0));
                e.0 += 1;
                e.1 += direction;
            }
        }
        assert!(volume > 0.);
        assert!(edges.values().all(|e| *e == (2, 0)));
    }
}
