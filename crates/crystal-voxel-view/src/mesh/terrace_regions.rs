//! Native-map height ownership. Connected scenery is context, not additional
//! uphill courses for the current map. Only adjoining land corners are joined.
use super::*;
use bevy::prelude::{IVec2, UVec2, Vec2};
use crystal_render_api::{VisualJumpLedge, VisualOpenWaterDatum, VisualTerraceDatum};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone, Debug, PartialEq)]
pub struct TerraceSourceRegion {
    pub map: String,
    pub edges: Vec<VisualJumpLedge>,
    pub cells: Vec<(IVec2, VisualTileSource, bool)>,
}

struct Cached {
    map: String,
    regions: Vec<TerraceSourceRegion>,
    edges: Vec<VisualJumpLedge>,
    datum: Option<Arc<VisualTerraceDatum>>,
}

/// Regions arrive in the same ownership order as the rendered source maps.
/// Complete native inputs keep the result independent of the camera crop.
pub fn connected_source_terrace_datum(
    map: &str,
    regions: &[TerraceSourceRegion],
    edges: &[VisualJumpLedge],
) -> Option<Arc<VisualTerraceDatum>> {
    static CACHE: OnceLock<Mutex<VecDeque<Cached>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().unwrap();
    if let Some(i) = cache
        .iter()
        .position(|v| v.map == map && v.regions == regions && v.edges == edges)
    {
        let entry = cache.remove(i).unwrap();
        let result = entry.datum.clone();
        cache.push_back(entry);
        return result;
    }
    let datum = build(map, regions, edges);
    if regions.iter().map(|r| r.cells.len()).sum::<usize>() <= 65_536 {
        if cache.len() == 4 {
            cache.pop_front();
        }
        cache.push_back(Cached {
            map: map.into(),
            regions: regions.to_vec(),
            edges: edges.to_vec(),
            datum: datum.clone(),
        });
    }
    datum
}

fn build(
    map: &str,
    regions: &[TerraceSourceRegion],
    edges: &[VisualJumpLedge],
) -> Option<Arc<VisualTerraceDatum>> {
    let primary = regions.first()?;
    let all = regions
        .iter()
        .flat_map(|r| r.cells.iter().cloned())
        .collect::<Vec<_>>();
    if !primary
        .cells
        .iter()
        .any(|(_, s, _)| matches!(s.tileset_id.as_ref(), "kanto" | "johto" | "johto_modern"))
    {
        return terrace_datum::source_terrace_datum(map, edges, &all);
    }
    let foreign = regions.iter().any(|r| {
        r.cells
            .iter()
            .any(|(_, s, _)| !matches!(s.tileset_id.as_ref(), "kanto" | "johto" | "johto_modern"))
    });
    let legacy = foreign
        .then(|| terrace_datum::source_terrace_datum(map, edges, &all))
        .flatten();
    let mut native = BTreeMap::new();
    for (owner, r) in regions.iter().enumerate() {
        for (p, s, land) in &r.cells {
            native.entry((p.y, p.x)).or_insert((owner, s, *land));
        }
    }
    let lo = native
        .keys()
        .fold(IVec2::splat(i32::MAX), |a, &(y, x)| a.min(IVec2::new(x, y)));
    let hi = native
        .keys()
        .fold(IVec2::splat(i32::MIN), |a, &(y, x)| a.max(IVec2::new(x, y)))
        + IVec2::ONE;
    let size = hi - lo;
    if size.x <= 0 || size.y <= 0 || i64::from(size.x) * i64::from(size.y) > 2_000_000 {
        return None;
    }
    let (w, h) = (size.x as usize, size.y as usize);
    let index = |p: IVec2| {
        let p = p - lo;
        (p.x >= 0 && p.y >= 0 && p.x < size.x && p.y < size.y)
            .then(|| p.y as usize * w + p.x as usize)
    };
    let point = |i: usize| lo + IVec2::new((i % w) as i32, (i / w) as i32);
    let mut levels = vec![[0.; 4]; w * h];
    let mut owners = vec![usize::MAX; w * h];
    let mut wet = vec![false; w * h];
    let mut native_outdoor_cells = vec![false; w * h];
    let mut barriers = Vec::new();
    let mut river = vec![false; w * h];
    for (owner, r) in regions.iter().enumerate() {
        let is_kanto = r
            .cells
            .iter()
            .all(|(_, s, _)| s.tileset_id.as_ref() == "kanto");
        let native_outdoor = r.cells.iter().all(|(_, s, _)|
            matches!(s.tileset_id.as_ref(), "kanto" | "johto" | "johto_modern"));
        let origin = if native_outdoor {
            r.cells
                .iter()
                .fold(IVec2::splat(i32::MAX), |lo, (p, _, _)| lo.min(*p))
        } else {
            IVec2::ZERO
        };
        let local_cells = r
            .cells
            .iter()
            .map(|(p, s, land)| (*p - origin, s.clone(), *land))
            .collect::<Vec<_>>();
        let local_edges = if jump_changes_floor_level(&r.map) {
            r.edges
                .iter()
                .map(|e| VisualJumpLedge {
                    start: e.start - origin,
                    end: e.end - origin,
                    ..*e
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let datum = if native_outdoor {
            terrace_datum::source_terrace_datum(&r.map, &local_edges, &local_cells)
        } else {
            legacy.clone()
        };
        let fallback =
            terrace_levels::get_for_family(if native_outdoor { &local_edges } else { edges }, is_kanto);
        for (p, s, _) in &r.cells {
            let i = index(*p)?;
            if native[&(p.y, p.x)].0 != owner {
                continue;
            }
            owners[i] = owner;
            native_outdoor_cells[i] = native_outdoor;
            let barrier = datum
                .as_ref()
                .is_some_and(|d| d.water_barrier_at(*p - origin));
            if barrier {
                barriers.push(*p);
            }
            wet[i] = terraces::water_owns_height(s)
                || barrier
                || elevated_piers::water_under_deck(&r.map, s, None);
            river[i] = datum
                .as_ref()
                .is_some_and(|d| d.open_water_at(*p - origin).is_some());
            levels[i] = if !jump_changes_floor_level(&r.map) {
                [0.; 4]
            } else {
                [IVec2::ZERO, IVec2::Y, IVec2::ONE, IVec2::X].map(|d| {
                    // Stay inside the native owner at a real jump boundary.
                    let q = (*p - origin + d).as_vec2() + (Vec2::splat(0.5) - d.as_vec2()) * 0.0001;
                    terrace_datum::sample(datum.as_deref(), q)
                        .unwrap_or_else(|| if wet[i] { 0. } else { fallback.height(q) })
                })
            };
        }
    }
    // A connected standing-water body has one level across native map joins.
    // Preserve separately authored waterfall pools and enclosed mineral ponds.
    let mut visited = vec![false; w * h];
    for start in 0..w * h {
        if !wet[start] || !native_outdoor_cells[start] || visited[start] {
            continue;
        }
        let mut body = vec![start];
        visited[start] = true;
        let mut cursor = 0;
        let mut waterfall = false;
        while cursor < body.len() {
            let i = body[cursor];
            cursor += 1;
            for d in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
                let p = point(i) + d;
                if let Some((_, s, _)) = native.get(&(p.y, p.x)) {
                    waterfall |= shape_for_source(s) == CellShape::Waterfall;
                }
                if let Some(j) = index(p) {
                    if wet[j] && native_outdoor_cells[j] && !visited[j] {
                        visited[j] = true;
                        body.push(j);
                    }
                }
            }
        }
        if !waterfall {
            let level = body
                .iter()
                .flat_map(|&i| levels[i])
                .fold(f32::INFINITY, f32::min);
            for i in body {
                levels[i] = [level; 4];
            }
        }
    }
    // Source-local courses stay flat away from joins. Average only dry native
    // map seams; never average an upper ledge through its authored cut or water.
    let mut cuts = HashSet::new();
    for edge in edges {
        let tangent = if edge.descent.x == 0 {
            IVec2::X
        } else {
            IVec2::Y
        };
        for step in 0..(edge.end - edge.start).abs().element_sum() {
            let high = edge.start + tangent * step - edge.descent.max(IVec2::ZERO);
            if let Some((a, b)) = index(high).zip(index(high + edge.descent)) {
                cuts.insert((a.min(b), a.max(b)));
            }
        }
    }
    let original = levels.clone();
    for i in 0..w * h {
        if wet[i] || !native_outdoor_cells[i] || owners[i] == usize::MAX {
            continue;
        }
        for (corner, d) in [IVec2::ZERO, IVec2::Y, IVec2::ONE, IVec2::X]
            .into_iter()
            .enumerate()
        {
            let vertex = point(i) + d;
            let group = terrace_levels::corner_domain(vertex, i, &index, &cuts)
                .into_iter()
                .filter(|&j| !wet[j] && native_outdoor_cells[j] && owners[j] != usize::MAX)
                .collect::<Vec<_>>();
            if !group.iter().any(|&j| owners[j] != owners[i]) {
                continue;
            }
            let fixed = group
                .iter()
                .copied()
                .filter(|&j| !jump_changes_floor_level(&regions[owners[j]].map))
                .collect::<Vec<_>>();
            let samples = if fixed.is_empty() { &group } else { &fixed };
            let mut sum = 0.;
            for &j in samples {
                let d = vertex - point(j);
                let c = match (d.x, d.y) {
                    (0, 0) => 0,
                    (0, 1) => 1,
                    (1, 1) => 2,
                    (1, 0) => 3,
                    _ => unreachable!(),
                };
                sum += original[j][c];
            }
            levels[i][corner] = sum / samples.len() as f32;
        }
    }
    let known=(0..w*h).map(|i|owners[i]!=usize::MAX && native.get(&(point(i).y,point(i).x))
        .is_some_and(|(_,source,_)|shape_for_source(source)!=CellShape::Waterfall)).collect::<Vec<_>>();
    terrace_datum::fit_shore_corners(w,h,&known,&wet,&mut levels);
    barriers.sort_unstable_by_key(|p| (p.y, p.x));
    barriers.dedup();
    let river_level = (0..w * h)
        .filter(|&i| river[i])
        .map(|i| levels[i][0])
        .reduce(f32::min);
    let open_water = river_level.map(|level| {
        let cells = (0..w * h)
            .filter(|&i| {
                owners[i] == usize::MAX || (river[i] && (levels[i][0] - level).abs() < 0.0001)
            })
            .map(point)
            .collect::<Vec<_>>();
        VisualOpenWaterDatum {
            level,
            cells: cells.into(),
        }
    });
    Some(Arc::new(VisualTerraceDatum {
        origin: lo,
        size: UVec2::new(w as u32, h as u32),
        levels: levels.into(),
        water_barriers: barriers.into(),
        open_water,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(
        map: &str,
        origin: IVec2,
        height: i32,
        edges: Vec<VisualJumpLedge>,
    ) -> TerraceSourceRegion {
        let cells = (0..height)
            .flat_map(|y| {
                (0..16).map(move |x| {
                    let wet = x >= 14;
                    let mut s = crate::mesh::tests::source_with_tile(
                        if wet { 0x18 } else { 0x0a },
                        (x % 4) as u8,
                        (y % 4) as u8,
                        if wet {
                            [[0x2a, 0x2b], [0x3a, 0x3b]][y as usize % 2][x as usize % 2]
                        } else {
                            0x2c
                        },
                    );
                    s.tileset_id = "kanto".into();
                    (origin + IVec2::new(x, y), s, !wet)
                })
            })
            .collect();
        TerraceSourceRegion {
            map: map.into(),
            edges,
            cells,
        }
    }

    #[test]
    fn johto_town_shore_does_not_inherit_connected_route_courses() {
        let main = region("NewBarkTown", IVec2::ZERO, 16, Vec::new());
        let edges = vec![VisualJumpLedge {
            start: IVec2::new(0, 24), end: IVec2::new(14, 24), descent: IVec2::Y,
        }];
        let remote = region("Route27", IVec2::new(0, 16), 32, edges.clone());
        let mut regions = vec![main, remote];
        for region in &mut regions {
            for (_, source, land) in &mut region.cells {
                source.tileset_id = "johto".into();
                source.metatile_id = if *land {1} else {0x54};
                source.tile_index = if *land {6} else {0x14};
            }
        }
        let datum = connected_source_terrace_datum("NewBarkTown", &regions, &edges).unwrap();
        for y in 0..14 { for x in 0..16 {
            assert_eq!(datum.levels[y*16+x], [0.;4],
                "a remote ledge must not turn the town shore into a ramp");
        }}
        let high = terrace_datum::sample(Some(&datum), Vec2::new(6.5,23.99)).unwrap();
        let low = terrace_datum::sample(Some(&datum), Vec2::new(6.5,24.01)).unwrap();
        assert!((high-low-1.).abs()<0.002, "the route retains its own ledge");
    }

    #[test]
    fn remote_route_courses_cannot_raise_a_flat_native_bank_and_seams_become_local_ramps() {
        let main = region("Route24", IVec2::ZERO, 16, Vec::new());
        let edges = (0..7)
            .map(|i| VisualJumpLedge {
                start: IVec2::new(0, 24 + i * 8),
                end: IVec2::new(14, 24 + i * 8),
                descent: IVec2::Y,
            })
            .collect::<Vec<_>>();
        let remote = region("RemoteRoute", IVec2::new(0, 16), 64, edges.clone());
        let regions = vec![main, remote];
        let cells = regions
            .iter()
            .flat_map(|r| r.cells.iter().cloned())
            .collect::<Vec<_>>();
        let old = terrace_datum::source_terrace_datum("Route24", &edges, &cells).unwrap();
        assert!(
            terrace_datum::sample(Some(&old), Vec2::new(6.5, 8.5)).unwrap() > 6.,
            "the combined solve must reproduce the seven-course false rise"
        );
        let datum = connected_source_terrace_datum("Route24", &regions, &edges).unwrap();
        for y in 0..14 {
            for x in 0..14 {
                assert_eq!(
                    datum.levels[y * 16 + x],
                    [0.; 4],
                    "the native map has no ledge here: ({x},{y})"
                );
            }
        }
        for y in 0..80 {
            for x in 14..16 {
                assert_eq!(datum.levels[y * 16 + x], [0.; 4]);
            }
        }
        for x in 1..13 {
            assert_eq!(
                datum.levels[15 * 16 + x][1],
                datum.levels[16 * 16 + x][0],
                "join the native seam"
            );
            assert_eq!(datum.levels[15 * 16 + x][2], datum.levels[16 * 16 + x][3]);
        }
        assert!(
            datum.levels[15 * 16 + 6][1] > 0.,
            "height changes are confined to the connection approach"
        );
        for edge in &edges {
            let x = 6.5;
            let y = edge.start.y as f32;
            let high = terrace_datum::sample(Some(&datum), Vec2::new(x, y - 0.01)).unwrap();
            let low = terrace_datum::sample(Some(&datum), Vec2::new(x, y + 0.01)).unwrap();
            assert!(
                (high - low - 1.).abs() < 0.002,
                "preserve the real directed drop at {y}: {high}/{low}"
            );
        }
        let delta = IVec2::new(-48, 32);
        let shifted = regions
            .iter()
            .cloned()
            .map(|mut r| {
                for (p, _, _) in &mut r.cells {
                    *p += delta;
                }
                for e in &mut r.edges {
                    e.start += delta;
                    e.end += delta;
                }
                r
            })
            .collect::<Vec<_>>();
        let shifted_edges = edges
            .iter()
            .map(|e| VisualJumpLedge {
                start: e.start + delta,
                end: e.end + delta,
                ..*e
            })
            .collect::<Vec<_>>();
        let other = connected_source_terrace_datum("Route24", &shifted, &shifted_edges).unwrap();
        assert_eq!(other.origin, datum.origin + delta);
        assert_eq!(
            other.levels, datum.levels,
            "native map heights cannot depend on the active map's coordinate frame"
        );
    }

    #[test]
    fn flat_city_keeps_its_border_flat_and_the_neighbor_owns_the_approach() {
        let main = region("SaffronCity", IVec2::ZERO, 16, Vec::new());
        let edges = vec![VisualJumpLedge {
            start: IVec2::new(0, 24),
            end: IVec2::new(14, 24),
            descent: IVec2::Y,
        }];
        let neighbor = region("Route6", IVec2::new(0, 16), 16, edges.clone());
        let datum =
            connected_source_terrace_datum("SaffronCity", &[main, neighbor], &edges).unwrap();
        assert!(
            datum.levels[..16 * 16].iter().all(|v| *v == [0.; 4]),
            "Saffron includes its complete border"
        );
        assert_eq!(
            datum.levels[16 * 16 + 6][0],
            0.,
            "the approach starts on the same city plane"
        );
        assert!(
            datum.levels[17 * 16 + 6][0] > 0.9,
            "the neighbor still retains its higher native course"
        );
    }
}
