//! Source-complete flowerbed edging; native flowers retain their own ownership.
use super::*;
use super::modeled_exteriors::park_flowers;
use crate::live_profiles::Document;
#[path = "park_flowerbed_geometry.rs"]
mod geometry;
#[path = "park_flowerbed_source.rs"]
mod source;

pub(super) struct Placement {
    column: usize,
    row: usize,
    width: usize,
    ground: [usize; 2],
    source_origin: [i32; 2],
}
impl Placement {
    pub(super) fn indices(&self, grid_width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..4).flat_map(move |y| {
            (0..self.width).filter_map(move |x| {
                source::border(x, y, self.width)
                    .then_some((self.row + y) * grid_width + self.column + x)
            })
        })
    }
    fn coherent(&self, cells: &[&VisualTile], g: &GridGeometry) -> bool {
        self.column + self.width <= g.width
            && self.row + 4 <= g.height
            && (0..4).all(|y| {
                (0..self.width).all(|x| {
                    let s = &cells[(self.row + y) * g.width + self.column + x].source;
                    source::matches_cell(
                        self.width,
                        x,
                        y,
                        (
                            &s.tileset_id,
                            s.metatile_id,
                            s.subtile_column,
                            s.subtile_row,
                            s.tile_index,
                        ),
                    )
                })
            })
    }
}

pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if !matches!(map, "NationalPark" | "NationalParkBugContest")
        || g.width.checked_mul(g.height) != Some(cells.len())
        || reserved.len() != cells.len()
    {
        return vec![];
    }
    let (Some(paving), Some(lawn)) = (
        park_side_fence::ground_sample(cells, true),
        park_side_fence::ground_sample(cells, false),
    ) else {
        return vec![];
    };
    let mut out = vec![];
    for c in cells {
        let s = &c.source;
        if s.tileset_id.as_ref() != "park"
            || s.metatile_id != 0x3c
            || s.subtile_column != 0
            || s.subtile_row != 0
        {
            continue;
        }
        let column = c.column as usize;
        let row = c.row as usize;
        if row + 4 > g.height
            || column + 8 > g.width
            || (origin[0] + column as i32).rem_euclid(4) != 0
            || (origin[1] + row as i32).rem_euclid(4) != 0
        {
            continue;
        }
        let mut end = column + 4;
        while end + 4 <= g.width
            && cells[row * g.width + end].source.tileset_id.as_ref() == "park"
            && cells[row * g.width + end].source.metatile_id == 0x3d
        {
            end += 4;
        }
        if end + 4 > g.width || cells[row * g.width + end].source.metatile_id != 0x3e {
            continue;
        }
        let p = Placement {
            column,
            row,
            width: end + 4 - column,
            ground: [paving, lawn],
            source_origin: [origin[0] + column as i32, origin[1] + row as i32],
        };
        if !p.coherent(cells, g) || p.indices(g.width).any(|i| reserved[i]) {
            continue;
        }
        if (0..4).any(|y| {
            (0..p.width).any(|x| {
                park_scenery::profile_owns_source(
                    map,
                    &cells[(row + y) * g.width + column + x].source,
                    profiles,
                )
            })
        }) {
            continue;
        }
        out.push(p);
    }
    out
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
) -> bool {
    if claimed.len() != cells.len()
        || !p.coherent(cells, g)
        || p.indices(g.width).any(|i| claimed[i])
        || park_side_fence::ground_sample(cells, true) != Some(p.ground[0])
        || park_side_fence::ground_sample(cells, false) != Some(p.ground[1])
    {
        return false;
    }
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let b = [
        w,
        w + p.width as f32 * g.tile_width,
        n,
        n + 4. * g.tile_height,
    ];
    let inner = [
        b[0] + g.tile_width * 0.45,
        b[1] - g.tile_width * 0.45,
        b[2] + g.tile_height * 0.45,
        b[3] - g.tile_height * 0.45,
    ];
    for i in p.indices(g.width) {
        ground(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            inner,
            g,
            p.ground,
        );
    }
    geometry::append(&mut mesh.solid, b, g.tile_height);
    // Only a complete native bed permits the low planting between its
    // flower samples. Heads and floor ownership remain on their own cells.
    for y in 1..3 { for x in 1..p.width - 1 {
        let i = (p.row + y) * g.width + p.column + x;
        if !claimed[i] {
            super::modeled_exteriors::append_park_underplanting(&mut mesh.solid,
                g.bounds(p.column + x, p.row + y).into(),
                [p.source_origin[0] + x as i32, p.source_origin[1] + y as i32]);
        }
    }}
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some("park/flowerbed-edge");
        }
    }
    true
}

fn ground(
    mesh: &mut SurfaceMeshData,
    b: [f32; 4],
    inner: [f32; 4],
    g: &GridGeometry,
    samples: [usize; 2],
) {
    let [w, e, n, s] = b;
    let [iw, ie, inn, is] = [
        w.max(inner[0]),
        e.min(inner[1]),
        n.max(inner[2]),
        s.min(inner[3]),
    ];
    // Split, rather than overlay, the lawn/paving join beneath the rail.
    for (rect, sample) in [
        ([w, e, n, inn], samples[0]),
        ([w, e, is, s], samples[0]),
        ([w, iw, inn, is], samples[0]),
        ([ie, e, inn, is], samples[0]),
        ([iw, ie, inn, is], samples[1]),
    ] {
        if rect[1] > rect[0] && rect[3] > rect[2] {
            append_top(mesh, rect, 0., g.uv(sample % g.width, sample / g.width));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (VisualWorldFrame, GridGeometry) {
        let left = [
            0x37, 0x38, 0x38, 0x38, 0x47, 3, 1, 3, 0x47, 3, 3, 3, 0x57, 0x58, 0x58, 0x58,
        ];
        let middle = [
            0x38, 0x38, 0x38, 0x38, 3, 3, 1, 3, 3, 1, 3, 3, 0x58, 0x58, 0x58, 0x58,
        ];
        let right = [
            0x38, 0x38, 0x38, 0x39, 1, 3, 3, 0x49, 3, 1, 3, 0x49, 0x58, 0x58, 0x58, 0x59,
        ];
        let sources = (0..88)
            .map(|i| {
                let x = i % 22;
                let y = i / 22;
                let (block, tile) = if x < 4 {
                    (0x3c, left[y * 4 + x])
                } else if x < 16 {
                    (0x3d, middle[y * 4 + x % 4])
                } else if x < 20 {
                    (0x3e, right[y * 4 + x % 4])
                } else if x == 20 {
                    (1, 0)
                } else {
                    (crate::park::PARK_TREE_GROUND_METATILE, 1)
                };
                let mut s = super::super::tests::source_with_tile(
                    block,
                    if x < 20 { (x % 4) as u8 } else { 0 },
                    y as u8,
                    tile,
                );
                s.tileset_id = "park".into();
                s
            })
            .collect();
        (
            super::super::tests::frame(22, 4, sources),
            GridGeometry {
                width: 22,
                height: 4,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
        )
    }
    #[test]
    fn park_flowerbed_replaces_only_the_complete_border_and_keeps_native_flowers() {
        let (frame, g) = fixture();
        let cells: Vec<_> = frame.tiles.iter().collect();
        let out = resolve("NationalPark", &cells, &g, [0, 0], None, &[false; 88]);
        assert_eq!(out.len(), 1);
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; 88],
            footing_heights: vec![0.; 88],
            ..Default::default()
        };
        let mut claimed = vec![false; 88];
        assert!(append(&mut mesh, &cells, &g, &out[0], &mut claimed));
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 44);
        assert_eq!(mesh.footing_heights, vec![0.; 88]);
        let foliage: Vec<_> = mesh.solid.positions.iter().zip(&mesh.solid.colors)
            .filter(|(_, c)| c[1] > c[0] * 1.7 && c[1] > c[2] * 3.)
            .map(|(p, _)| p).collect();
        assert!(!foliage.is_empty(), "the complete bed must receive its low planting");
        assert!(foliage.iter().all(|p| p[0] > 8. && p[0] < 152.
            && p[2] > 8. && p[2] < 24. && p[1] > 0. && p[1] < 2.4));
        for i in 0..88 {
            if matches!(cells[i].source.tile_index, 1 | 3) {
                assert!(!claimed[i]);
            }
        }
        assert!(!append(&mut mesh, &cells, &g, &out[0], &mut claimed));
        for y in 0..4 {
            for x in 0..20 {
                let mut changed = frame.tiles.clone();
                changed[y * 22 + x].source.tile_index ^= 1;
                let cells: Vec<_> = changed.iter().collect();
                assert!(resolve("NationalPark", &cells, &g, [0, 0], None, &[false; 88]).is_empty());
                let mut empty = TerrainMeshData::default();
                assert!(!append(&mut empty, &cells, &g, &out[0], &mut [false; 88]));
                assert!(empty.solid.positions.is_empty() && empty.textured.positions.is_empty());
            }
        }
        assert!(resolve("NationalPark", &cells, &g, [1, 0], None, &[false; 88]).is_empty());
        let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"My flower patch","map":"NationalPark","tileset":"park","metatile":60,"origin":[1,1],"tiles":[[3]],"ground":65535,"top_pixels":8,"depth_pixels":9}]}"#).unwrap();
        assert!(
            resolve(
                "NationalPark",
                &cells,
                &g,
                [0, 0],
                Some(&custom),
                &[false; 88]
            )
            .is_empty()
        );
    }
    #[test]
    fn park_flowerbed_ground_meets_without_overlapping_or_raising_the_floor() {
        let g = GridGeometry {
            width: 20,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let inner = [3.6, 156.4, 3.6, 28.4];
        let mut mesh = SurfaceMeshData::default();
        for y in 0..4 {
            for x in 0..20 {
                if !source::border(x, y, 20) {
                    continue;
                }
                ground(&mut mesh, g.bounds(x, y).into(), inner, &g, [0, 1]);
            }
        }
        assert!(mesh.positions.iter().all(|p| p[1] == 0.));
        let area: f32 = mesh
            .indices
            .chunks_exact(3)
            .map(|t| {
                let a = Vec3::from_array(mesh.positions[t[0] as usize]);
                let b = Vec3::from_array(mesh.positions[t[1] as usize]);
                let c = Vec3::from_array(mesh.positions[t[2] as usize]);
                (b - a).cross(c - a).length() * 0.5
            })
            .sum();
        assert!(
            (area - 44. * 64.).abs() < 0.01,
            "partitioned backing must cover the 44 border cells exactly once"
        );
    }
}
