//! One level per complete native pond, following its lowest dry shore.
//! Open/clipped seas, cascades and custom terrain retain their original datum.
use super::*;

fn wet(tile: &VisualTile) -> bool {
    tile.source.tileset_id.as_ref() == "kanto"
        && tile.source.tile_index == 0x14
        && shape_for_source(&tile.source) == CellShape::Water
}
fn shore(tile: &VisualTile, claim: Option<&str>) -> bool {
    let s = &tile.source;
    if s.tileset_id.as_ref() != "kanto" || shape_for_source(s) != CellShape::Flat {
        return false;
    }
    match claim {
        None => matches!(
            new_bark::ground_material(s),
            Some(
                new_bark::GroundMaterial::Lawn
                    | new_bark::GroundMaterial::Path
                    | new_bark::GroundMaterial::Pavers
            )
        ),
        Some("outdoor/kanto-shore") => matches!(s.tile_index, 0x31 | 0x33 | 0x54),
        Some("outdoor/kanto-brick-path") => s.metatile_id == 0x55 && s.tile_index == 0x5b,
        _ => false,
    }
}

pub(super) fn finish_with_datum(mesh:&mut TerrainMeshData,cells:&[&VisualTile],g:&GridGeometry,
    edges:&[crystal_render_api::VisualJumpLedge],origin:[i32;2],datum:Option<&crystal_render_api::VisualTerraceDatum>) {
    let field = terraces::Field::with_source(edges, g, origin, cells).with_datum(datum);
    finish_with_field(mesh, cells, g, Some(&field));
}
#[cfg(test)]
fn finish(mesh: &mut TerrainMeshData, cells: &[&VisualTile], g: &GridGeometry) {
    finish_with_field(mesh, cells, g, None);
}
fn finish_with_field(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    field: Option<&terraces::Field<'_>>,
) {
    let count = g.width * g.height;
    if g.width == 0 || g.height == 0 || cells.len() != count || mesh.footing_heights.len() != count
    {
        return;
    }
    let mut visited = vec![false; count];
    let mut offsets = vec![0.; count];
    for start in 0..count {
        if visited[start] || !wet(cells[start]) {
            continue;
        }
        let mut body = Vec::new();
        let mut pending = vec![start];
        visited[start] = true;
        let mut closed = true;
        let mut rim = f32::INFINITY;
        while let Some(i) = pending.pop() {
            body.push(i);
            // Complete-source mineral ponds have already moved with their
            // plateau, including Surf support. The fallback pond pass must
            // not apply that same offset a second time.
            if field.is_some_and(|f|f.water_offset(i,cells)>0.) {closed=false;}

            if mesh.authored_cells.get(i).is_some_and(|c| c.is_some()) {
                closed = false;
            }
            let (x, y) = (i % g.width, i / g.width);
            for j in [
                x.checked_sub(1).map(|x| y * g.width + x),
                (x + 1 < g.width).then_some(i + 1),
                y.checked_sub(1).map(|y| y * g.width + x),
                (y + 1 < g.height).then_some(i + g.width),
            ] {
                let Some(j) = j else {
                    closed = false;
                    continue;
                };
                if wet(cells[j]) {
                    if !visited[j] {
                        visited[j] = true;
                        pending.push(j);
                    }
                } else if shore(cells[j], mesh.authored_cells.get(j).copied().flatten())
                    && mesh.footing_heights[j].is_finite()
                {
                    rim = rim.min(mesh.footing_heights[j]);
                } else {
                    closed = false;
                }
            }
        }
        if closed && rim.is_finite() && rim > 0.001 {
            for i in body {
                offsets[i] = rim;
            }
        }
    }
    if !offsets.iter().any(|&v| v > 0.) {
        return;
    }
    let water = crate::profile::WATER_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
    for surface in [
        &mut mesh.solid,
        &mut mesh.textured,
        &mut mesh.animated_solid,
        &mut mesh.animated_textured,
    ] {
        let mut lifts = vec![0f32; surface.positions.len()];
        for tri in surface.indices.chunks_exact(3) {
            let ids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
            if ids.iter().any(|&i| {
                surface.normals[i][1] < 0.99
                    || (surface.positions[i][1] - water).abs() > g.tile_height * 0.04
            }) {
                continue;
            }
            let center = ids
                .map(|i| Vec3::from_array(surface.positions[i]))
                .into_iter()
                .sum::<Vec3>()
                / 3.;
            let Some(i) = new_bark::cell_at(g, center.x, center.z) else {
                continue;
            };
            if offsets[i] > 0. {
                for v in ids {
                    lifts[v] = offsets[i];
                }
            }
        }
        for (p, lift) in surface.positions.iter_mut().zip(lifts) {
            p[1] += lift;
        }
    }
    // Retain the native rocky bank, but grade its inner edge down to the
    // pond rim. The outer contact stays on the existing terrace. Submerged
    // wall geometry stays submerged instead of leaving a tall retaining wall.
    for surface in [
        &mut mesh.solid,
        &mut mesh.textured,
        &mut mesh.animated_solid,
        &mut mesh.animated_textured,
    ] {
        let mut shifts = vec![None; surface.positions.len()];
        for tri in surface.indices.chunks_exact(3) {
            let ids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
            let center = ids
                .map(|i| Vec3::from_array(surface.positions[i]))
                .into_iter()
                .sum::<Vec3>()
                / 3.;
            let normal = Vec3::from_array(surface.normals[ids[0]]);
            // Sculpted support faces have relief into the water. Sample half
            // a source cell behind near-vertical faces to recover their bank.
            let owner = center
                - normal
                    * if normal.y.abs() < 0.7 {
                        g.tile_width.min(g.tile_height) * 0.5
                    } else {
                        0.001
                    };
            let Some(i) = new_bark::cell_at(g, owner.x, owner.z) else {
                continue;
            };
            if mesh.authored_cells.get(i).copied().flatten() != Some("outdoor/kanto-shore") {
                continue;
            }
            for v in ids {
                let p = surface.positions[v];
                if let Some((rim, weight)) = bank_grade(g, i, p[0], p[2], |j| {
                    (offsets[j] > 0.).then_some(offsets[j])
                }) {
                    let ground = field.map_or(mesh.footing_heights[i], |f| {
                        f.height(bevy::prelude::Vec2::new(p[0], p[2]))
                    });
                    shifts[v] = Some((rim - ground) * weight);
                }
            }
        }
        for (p, shift) in surface.positions.iter_mut().zip(shifts.iter()) {
            p[1] += shift.unwrap_or(0.);
        }
        let mut normals = vec![Vec3::ZERO; surface.positions.len()];
        for tri in surface.indices.chunks_exact(3) {
            let ids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
            let [a, b, c] = ids.map(|i| Vec3::from_array(surface.positions[i]));
            let n = (b - a).cross(c - a);
            for i in ids {
                normals[i] += n;
            }
        }
        for (i, n) in normals.into_iter().enumerate() {
            if shifts[i].is_some() && n.length_squared() > 1e-8 {
                surface.normals[i] = n.normalize().to_array();
            }
        }
    }
    // Surf and other visible actors consume this same authoritative render
    // support. Collision, source coordinates and movement permissions stay put.
    for (height, offset) in mesh.footing_heights.iter_mut().zip(offsets) {
        *height += offset;
    }
}

// A shared distance field gives coincident bank vertices the same grade,
// including diagonal corners, without moving the outer dry contact.
fn bank_grade(
    g: &GridGeometry,
    i: usize,
    x: f32,
    z: f32,
    level: impl Fn(usize) -> Option<f32>,
) -> Option<(f32, f32)> {
    let (cx, cy) = (i % g.width, i / g.width);
    let mut result = None;
    let mut nearest = f32::INFINITY;
    for y in cy.saturating_sub(1)..=(cy + 1).min(g.height - 1) {
        for c in cx.saturating_sub(1)..=(cx + 1).min(g.width - 1) {
            let j = y * g.width + c;
            let Some(rim) = level(j) else { continue };
            let (w, e, n, s) = g.bounds(c, y);
            let d = ((x - x.clamp(w, e)) / g.tile_width).hypot((z - z.clamp(n, s)) / g.tile_height);
            if d < nearest {
                nearest = d;
                result = Some((rim, (1. - d).clamp(0., 1.)));
            }
        }
    }
    result
}

pub(super) fn support_correction(
    frame: &VisualWorldFrame,
    cell: bevy::prelude::UVec2,
    foot: bevy::prelude::Vec2,
    resolved: &[f32],
) -> f32 {
    let g = GridGeometry {
        width: frame.grid_size.x as usize,
        height: frame.grid_size.y as usize,
        tile_width: frame.tile_size.x,
        tile_height: frame.tile_size.y,
        origin_x: 0.,
        origin_z: 0.,
    };
    let i = cell.y as usize * g.width + cell.x as usize;
    let Some(tile) = frame.tiles.get(i) else {
        return 0.;
    };
    if !kanto_shore::source_bank(&tile.source) {
        return 0.;
    }
    let left = frame.center.x - frame.grid_size.x as f32 * frame.tile_size.x * 0.5;
    let top = frame.center.y + frame.grid_size.y as f32 * frame.tile_size.y * 0.5;
    let Some((rim, weight)) = bank_grade(&g, i, foot.x - left, top - foot.y, |j| {
        frame
            .tiles
            .get(j)
            .filter(|t| wet(t))
            .and_then(|_| resolved.get(j))
            .copied()
            .filter(|h| *h > 0.001)
    }) else {
        return 0.;
    };
    let Some(base) = resolved.get(i) else {
        return 0.;
    };
    (rim - base - terraces::support_correction(frame, cell, foot)) * weight
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Vec<VisualTile>, GridGeometry, TerrainMeshData) {
        let g = GridGeometry {
            width: 7,
            height: 7,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = (0..49)
            .map(|i| {
                let mut s = super::super::tests::source_with_tile(
                    0x31,
                    (i % 4) as u8,
                    (i / 7 % 4) as u8,
                    0x39,
                );
                s.tileset_id = "kanto".into();
                if (2..5).contains(&(i % 7)) && (2..5).contains(&(i / 7)) {
                    s.metatile_id = 0x43;
                    s.tile_index = 0x14;
                }
                VisualTile {
                    column: (i % 7) as u32,
                    row: (i / 7) as u32,
                    source: s,
                    texture: Default::default(),
                    priority: false,
                    animation_frames: None,
                }
            })
            .collect::<Vec<_>>();
        let mut mesh = TerrainMeshData {
            footing_heights: tiles
                .iter()
                .map(|t| if wet(t) { 0. } else { 16. })
                .collect(),
            authored_cells: vec![None; 49],
            ..Default::default()
        };
        for surface in [
            &mut mesh.solid,
            &mut mesh.textured,
            &mut mesh.animated_solid,
            &mut mesh.animated_textured,
        ] {
            for (i, t) in tiles.iter().enumerate() {
                if wet(t) {
                    let (w, e, n, s) = g.bounds(i % 7, i / 7);
                    append_top(surface, [w, e, n, s], -2., g.uv(i % 7, i / 7));
                }
            }
        }
        (tiles, g, mesh)
    }
    #[test]
    fn enclosed_pond_has_one_shore_level_for_geometry_and_surf_support() {
        let (tiles, g, mut mesh) = fixture();
        let original = mesh.clone();
        let source = tiles.clone();
        // Different surrounding terraces must never tilt or step the water.
        mesh.footing_heights[1 * 7 + 3] = 24.;
        finish(&mut mesh, &tiles.iter().collect::<Vec<_>>(), &g);
        for surface in [
            &mesh.solid,
            &mesh.textured,
            &mesh.animated_solid,
            &mesh.animated_textured,
        ] {
            assert!(surface.positions.iter().all(|p| p[1] == 14.));
            assert_eq!(surface.indices, original.solid.indices);
            assert_eq!(surface.uvs, original.solid.uvs);
            assert_eq!(surface.normals, original.solid.normals);
            assert_eq!(surface.colors, original.solid.colors);
        }
        for (i, t) in tiles.iter().enumerate() {
            assert_eq!(
                mesh.footing_heights[i],
                if wet(t) {
                    16.
                } else if i == 10 {
                    24.
                } else {
                    16.
                }
            );
        }
        assert_eq!(tiles, source);
        assert_eq!(mesh.authored_cells, original.authored_cells);
    }
    #[test]
    fn source_resolved_pond_is_not_lifted_a_second_time() {
        let (tiles,g,mut mesh)=fixture();
        let mut levels=vec![[0.;4];tiles.len()];
        for (i,tile) in tiles.iter().enumerate() {if wet(tile) {
            levels[i]=[16./crate::profile::JUMP_LEDGE_HEIGHT;4];
            mesh.footing_heights[i]+=16.;
        }}
        for surface in [&mut mesh.solid,&mut mesh.textured,&mut mesh.animated_solid,&mut mesh.animated_textured] {
            for p in &mut surface.positions {p[1]+=16.;}
        }
        let datum=crystal_render_api::VisualTerraceDatum {origin:bevy::prelude::IVec2::ZERO,
            size:bevy::prelude::UVec2::splat(7),levels:levels.into(),open_water:None,water_barriers:std::sync::Arc::from([])};
        let original=mesh.clone();
        finish_with_datum(&mut mesh,&tiles.iter().collect::<Vec<_>>(),&g,&[],[0,0],Some(&datum));
        assert_eq!(mesh,original);
    }

    #[test]
    fn rocky_bank_grades_to_water_without_moving_the_outer_contact() {
        let (mut tiles, g, mut mesh) = fixture();
        let bank = 10;
        tiles[bank].source.metatile_id = 0x1d;
        tiles[bank].source.subtile_column = 3;
        tiles[bank].source.subtile_row = 0;
        tiles[bank].source.tile_index = 0x33;
        mesh.authored_cells[bank] = Some("outdoor/kanto-shore");
        mesh.footing_heights[bank] = 24.;
        let old_len = mesh.solid.positions.len();
        let (w, e, n, s) = g.bounds(3, 1);
        append_top(&mut mesh.solid, [w, e, n, s], 24., g.uv(3, 1));
        let cap_end = mesh.solid.positions.len();
        // Terrace support relief protrudes into the water beyond the exact
        // source-cell border. It must follow the same inner bank grade.
        append_solid_quad(
            &mut mesh.solid,
            [
                [w, -2., s + 0.3],
                [w, 24., s + 0.3],
                [e, 24., s + 0.3],
                [e, -2., s + 0.3],
            ],
            [0., 0., 1.],
            [0.3, 0.2, 0.1, 1.],
        );
        let old = mesh.solid.clone();
        finish(&mut mesh, &tiles.iter().collect::<Vec<_>>(), &g);
        for (p, before) in mesh.solid.positions[old_len..cap_end]
            .iter()
            .zip(&old.positions[old_len..cap_end])
        {
            assert_eq!([p[0], p[2]], [before[0], before[2]]);
            assert_eq!(p[1], if p[2] == n { 24. } else { 16. });
        }
        assert!(
            mesh.solid.positions[cap_end..]
                .iter()
                .all(|p| p[1] <= 16.001),
            "relieved support wall must not remain above the new rocky rim"
        );
        assert_eq!(
            mesh.footing_heights[bank], 24.,
            "cached bank retains outer datum"
        );
        let frame = VisualWorldFrame { terrace_datum: None,
            active: true,
            map_id: "Route22".into(),
            source_map_size_core_tiles: None,
            jump_ledges: Default::default(),
            boundary_warps: Default::default(),
            terrain_revision: 1,
            grid_origin: Default::default(),
            map_texture: Default::default(),
            center: bevy::prelude::Vec2::ZERO,
            viewport_size: bevy::prelude::Vec2::splat(56.),
            tile_size: bevy::prelude::Vec2::splat(8.),
            grid_size: bevy::prelude::UVec2::splat(7),
            tiles,
            actors: Vec::new(),
        };
        for (z, height) in [(8.1, 23.9), (12., 20.), (15.9, 16.1)] {
            let foot = bevy::prelude::Vec2::new(0., 28. - z);
            let actual =
                crate::footing::resolved_footing_height(&frame, foot, &mesh.footing_heights)
                    .unwrap();
            assert!((actual - height).abs() < 0.02, "{z}: {actual} vs {height}");
        }
    }
    #[test]
    fn ocean_clipped_water_custom_banks_and_cascades_keep_their_original_level() {
        for fault in 0..5 {
            let (mut tiles, g, mut mesh) = fixture();
            match fault {
                0 => {
                    for i in [0 * 7 + 3, 1 * 7 + 3] {
                        tiles[i].source.metatile_id = 0x43;
                        tiles[i].source.tile_index = 0x14;
                    }
                }
                1 => mesh.authored_cells[1 * 7 + 3] = Some("custom"),
                2 => mesh.authored_cells[3 * 7 + 3] = Some("custom water"),
                3 => {
                    let s = &mut tiles[1 * 7 + 3].source;
                    s.tileset_id = "cave".into();
                    s.metatile_id = 0x2c;
                    s.tile_index = 0x40;
                    assert_eq!(shape_for_source(s), CellShape::Waterfall);
                }
                _ => {
                    for h in &mut mesh.footing_heights {
                        *h = 0.;
                    }
                }
            }
            let original = mesh.clone();
            finish(&mut mesh, &tiles.iter().collect::<Vec<_>>(), &g);
            assert_eq!(
                mesh, original,
                "unproven or sea-level body must remain unchanged: {fault}"
            );
        }
    }
}
