//! Low beveled stone at the exact paired exit motif; no invented flight.
use super::*;
pub(super) fn append(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    dark: bool,
    frozen: bool,
    origin: [i32; 2],
) {
    let first = out.positions.len();
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + 2. * g.tile_width;
    let s = n + g.tile_height;
    let u = g.tile_height / SOURCE_TILE_HEIGHT;
    let rim = 0.7 * u;
    let h = 1.5 * u;
    let bottom = [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]];
    let top = [
        [w + rim, h, n + rim],
        [w + rim, h, s - rim],
        [e - rim, h, s - rim],
        [e - rim, h, n + rim],
    ];
    let color = if frozen {
        [0.43, 0.61, 0.66, 1.]
    } else if dark {
        [0.35, 0.42, 0.45, 1.]
    } else {
        [0.51, 0.47, 0.36, 1.]
    };
    append_solid_quad(out, top, [0., 1., 0.], color);
    for i in 0..4 {
        let j = (i + 1) % 4;
        let points = [bottom[i], bottom[j], top[j], top[i]];
        let normal = (Vec3::from_array(points[1]) - Vec3::from_array(points[0]))
            .cross(Vec3::from_array(points[2]) - Vec3::from_array(points[0]))
            .normalize()
            .to_array();
        append_solid_quad(
            out,
            points,
            normal,
            [color[0] * 0.83, color[1] * 0.83, color[2] * 0.83, 1.],
        );
    }
    // Thresholds share the surrounding mineral finish in every cavern.
    // Their low complete form stays opaque during the enclosing-wall reveal.
    for i in first..out.positions.len() {
        out.uvs[i] = cave_cliff::stone_uv(Vec3::from_array(out.positions[i]), g, origin, frozen);
        out.colors[i] = cave_cliff::pigment(
            out.colors[i],
            Vec3::from_array(out.positions[i]),
            g,
            origin,
            frozen,
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ice_exit_threshold_claims_only_the_exact_pair_and_keeps_zero_support() {
        let mut tiles = super::super::tests::tiles(4, 4, "ice_path");
        for t in &mut tiles {
            t.source.tile_index = 0x19;
        }
        for (i, tile_index) in [(0, 0x42), (1, 0x43)] {
            tiles[i].source.metatile_id = 0x2a;
            tiles[i].source.tile_index = tile_index;
        }
        let g = super::super::tests::grid(4, 4);
        let refs = tiles.iter().collect::<Vec<_>>();
        let placements = resolve("IcePath1F", &refs, &g, None);
        assert_eq!(placements.len(), 1);
        let p = &placements[0];
        assert_eq!(p.kind_label(), "dungeon:ice-exit-sill");
        assert_eq!(p.indices(4).collect::<Vec<_>>(), vec![0, 1]);
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; 16],
            ..Default::default()
        };
        super::super::append(&mut mesh, &g, p, &refs, [10, 20]);
        assert_eq!(mesh.footing_heights, vec![0.; 16]);
        assert!(mesh.solid.positions.iter().all(|p| p[0] >= 0.
            && p[0] <= 16.
            && p[2] >= 0.
            && p[2] <= 8.
            && p[1] >= 0.
            && p[1] <= 1.5));
        assert!(
            mesh.solid
                .uvs
                .iter()
                .all(|uv| (uv[1] / 100000.).round() == 7.)
        );
        assert!(
            mesh.solid
                .normals
                .iter()
                .all(|n| Vec3::from_array(*n).is_finite())
        );
        let original = tiles[1].source.clone();
        for defect in 0..4 {
            tiles[1].source = original.clone();
            match defect {
                0 => tiles[1].source.tile_index = 0xff,
                1 => tiles[1].source.metatile_id = 0x07,
                2 => tiles[1].source.subtile_row = 1,
                _ => tiles[1].source.subtile_column = 3,
            }
            assert!(resolve("IcePath1F", &tiles.iter().collect::<Vec<_>>(), &g, None).is_empty());
        }
    }
    #[test]
    fn paired_sill_keeps_support_and_rejects_partial_or_repainted_motifs() {
        let mut t = super::super::tests::tiles(4, 4, "cave");
        for cell in &mut t {
            cell.source.tile_index = 0x16;
        }
        for (i, tile) in [(2, 0x1a), (3, 0x1b)] {
            t[i].source.metatile_id = 0x23;
            t[i].source.tile_index = tile;
        }
        let refs = t.iter().collect::<Vec<_>>();
        let g = super::super::tests::grid(4, 4);
        let p = resolve("DiglettsCave", &refs, &g, None);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].kind_label(), "dungeon:cave-exit-sill");
        assert_eq!(p[0].indices(4).collect::<Vec<_>>(), vec![2, 3]);
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; 16],
            ..Default::default()
        };
        super::super::append(&mut mesh, &g, &p[0], &refs, [0, 0]);
        assert_eq!(mesh.footing_heights, vec![0.; 16]);
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| (0. ..=1.5).contains(&p[1]))
        );
        let substrate=p[0].indices(g.width).count()*64;
        assert_eq!(mesh.solid.positions.len(), substrate+20);
        assert!(mesh.solid.positions[..substrate].iter().all(|p|p[1]==0.));
        assert!(mesh.solid.uvs[..substrate].iter().all(|uv|(uv[1]/100000.).round()==4.));
        assert!(mesh.solid.uvs[substrate..].iter()
            .all(|uv| (uv[1] / 100000.).round() == 6.));
        t[3].source.tile_index = 0xff;
        assert!(resolve("DiglettsCave", &t.iter().collect::<Vec<_>>(), &g, None).is_empty());
    }
}
