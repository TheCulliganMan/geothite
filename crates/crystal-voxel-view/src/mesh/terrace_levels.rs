//! Shared ground levels constrained by authored directed drops. Connected
//! corners share one terrace; consecutive drops retain separate lower levels.
use bevy::prelude::{IVec2, Vec2};
use crystal_render_api::VisualJumpLedge;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

pub(super) struct Levels {
    origin: IVec2,
    width: usize,
    height: usize,
    corners: Vec<[f32; 4]>,
}
static CACHE: OnceLock<Mutex<VecDeque<(Vec<VisualJumpLedge>, bool, Arc<Levels>)>>> =
    OnceLock::new();

pub(super) fn get(edges: &[VisualJumpLedge]) -> Arc<Levels> {
    get_for_family(edges, false)
}

pub(super) fn get_for_family(edges: &[VisualJumpLedge], kanto: bool) -> Arc<Levels> {
    let mut cache = CACHE
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .unwrap();
    if let Some(i) = cache
        .iter()
        .position(|(key, family, _)| key.as_slice() == edges && *family == kanto)
    {
        let item = cache.remove(i).unwrap();
        let answer = item.2.clone();
        cache.push_back(item);
        return answer;
    }
    let _trace=crate::loading_trace::span("region.resolve_legacy_terraces", || serde_json::json!({"edges":edges.len(),"kanto":kanto}));
    let levels = Arc::new(Levels::build(edges, kanto));
    if cache.len() == 4 {
        cache.pop_front();
    }
    cache.push_back((edges.to_vec(), kanto, levels.clone()));
    levels
}

pub(super) fn corner_domain(
    vertex: IVec2,
    owner: usize,
    index: &impl Fn(IVec2) -> Option<usize>,
    cuts: &HashSet<(usize, usize)>,
) -> Vec<usize> {
    let candidates = [
        vertex - IVec2::ONE,
        vertex - IVec2::Y,
        vertex,
        vertex - IVec2::X,
    ]
    .map(index);
    let mut visited = [false; 4];
    let mut pending = Vec::new();
    if let Some(i) = candidates.iter().position(|&c| c == Some(owner)) {
        visited[i] = true;
        pending.push(i);
    }
    while let Some(i) = pending.pop() {
        for j in [(i + 1) % 4, (i + 3) % 4] {
            if visited[j] {
                continue;
            }
            let Some(b) = candidates[j] else {
                continue;
            };
            let a = candidates[i].unwrap();
            if cuts.contains(&(a.min(b), a.max(b))) {
                continue;
            }
            visited[j] = true;
            pending.push(j);
        }
    }
    (0..4)
        .filter(|&i| visited[i])
        .filter_map(|i| candidates[i])
        .collect()
}

struct Union {
    parent: Vec<usize>,
    rank: Vec<u8>,
    /// Integer level relative to the parent.
    offset: Vec<i32>,
}
impl Union {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
            offset: vec![0; n],
        }
    }
    fn find(&mut self, i: usize) -> (usize, i32) {
        if self.parent[i] == i {
            return (i, 0);
        }
        let (root, shift) = self.find(self.parent[i]);
        self.offset[i] += shift;
        self.parent[i] = root;
        (root, self.offset[i])
    }
    fn join(&mut self, a: usize, b: usize, delta: i32) -> bool {
        let (ra, pa) = self.find(a);
        let (rb, pb) = self.find(b);
        if ra == rb {
            return pa - pb == delta;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
            self.offset[ra] = delta - pa + pb;
        } else {
            self.parent[rb] = ra;
            self.offset[rb] = pa - pb - delta;
            if self.rank[ra] == self.rank[rb] {
                self.rank[ra] += 1;
            }
        }
        true
    }
}

impl Levels {
    fn build(edges: &[VisualJumpLedge], kanto: bool) -> Self {
        if kanto && !edges.is_empty() && edges.iter().all(|e| e.descent == IVec2::Y) {
            return Self::south_courses(edges);
        }
        let mut min = IVec2::ZERO;
        let mut max = IVec2::ZERO;
        for e in edges {
            min = min.min(e.start).min(e.end);
            max = max.max(e.start).max(e.end);
        }
        min -= IVec2::splat(8);
        max += IVec2::splat(8);
        let width = (max.x - min.x) as usize;
        let height = (max.y - min.y) as usize;
        let n = width * height;
        let index = |p: IVec2| -> Option<usize> {
            let p = p - min;
            (p.x >= 0 && p.y >= 0 && p.x < width as i32 && p.y < height as i32)
                .then(|| p.y as usize * width + p.x as usize)
        };
        let pair = |a: usize, b: usize| if a < b { (a, b) } else { (b, a) };
        let mut drops = Vec::new();
        let mut cuts = HashSet::new();
        for edge in edges {
            let length = (edge.end - edge.start).abs().element_sum();
            let tangent = if edge.descent.x == 0 {
                IVec2::X
            } else {
                IVec2::Y
            };
            for i in 0..length {
                let plane = edge.start + tangent * i;
                let high = plane - IVec2::new(edge.descent.x.max(0), edge.descent.y.max(0));
                let low = high + edge.descent;
                if let (Some(a), Some(b)) = (index(high), index(low)) {
                    drops.push((high, low, edge.descent, a, b));
                    cuts.insert(pair(a, b));
                }
            }
        }
        let mut union = Union::new(n);
        let directions = drops
            .iter()
            .map(|&(p, _, d, _, _)| (p.x, p.y, d.x, d.y))
            .collect::<HashSet<_>>();
        for &(_, _, _, a, b) in &drops {
            assert!(
                union.join(a, b, 1),
                "authored ledge directions impose contradictory ground levels"
            );
        }
        // Along a course, join adjacent equal-side cells unless another
        // directed edge cuts that shared boundary.
        for &(high, low, d, _, _) in &drops {
            let tangent = if d.x == 0 { IVec2::X } else { IVec2::Y };
            let next = high + tangent;
            if !directions.contains(&(next.x, next.y, d.x, d.y)) {
                continue;
            }
            for p in [high, low] {
                if let (Some(a), Some(b)) = (index(p), index(p + tangent)) {
                    if !cuts.contains(&pair(a, b)) {
                        union.join(a, b, 0);
                    }
                }
            }
        }
        let mut foot_anchors = std::collections::HashMap::<IVec2, usize>::new();
        // Anchor both standing-foot positions of a two-runtime-tile jump.
        // Each collision quadrant spans two source cells. Bind the two pixels
        // inside the standing sprite footprint to the ledge level. A blocked
        // neighboring row must not create a conflicting equality. The floor
        // vertex below the foot then uses this anchor on its connected side
        // of the authored cuts. Surrounding ground is extended afterward.
        for edge in edges {
            let d = edge.descent;
            let tangent = if d.x == 0 { IVec2::X } else { IVec2::Y };
            let length = (edge.end - edge.start).abs().element_sum();
            for offset in (0..length).step_by(2) {
                let plane = edge.start + tangent * offset;
                let high = plane - d.max(IVec2::ZERO);
                let low = high + d;
                let (Some(a), Some(b)) = (index(high), index(low)) else {
                    continue;
                };
                for depth in [1, 3] {
                    let takeoff = if d.x == 0 {
                        IVec2::new(plane.x + 1, plane.y + 1 - d.y * depth)
                    } else {
                        IVec2::new(plane.x - d.x * depth, plane.y + 2)
                    };
                    for (point, level) in [(takeoff, a), (takeoff + d * 4, b)] {
                        let Some(owner) = index(point - IVec2::Y) else {
                            continue;
                        };
                        for pixel in [point - IVec2::ONE, point - IVec2::Y] {
                            if let Some(cell) = index(pixel) {
                                assert!(
                                    union.join(level, cell, 0),
                                    "authored ledge standing-foot domains impose contradictory levels"
                                );
                            }
                        }
                        foot_anchors.insert(point, owner);
                    }
                }
            }
        }
        // Protect local takeoff/landing ground before remote approach rays.
        // A two-runtime-tile jump spans four source cells. Reserve its ending
        // boundary too: sprite feet sit there, and the floor interpolation
        // includes the neighboring cell. Remote continuations are secondary.
        let extend_approaches = |union: &mut Union, limit: Option<i32>| {
            for &(high, low, d, _, _) in &drops {
                for (start, step) in [(high, -d), (low, d)] {
                    let mut p = start;
                    let mut distance = 0;
                    while let (Some(a), Some(b)) = (index(p), index(p + step)) {
                        if limit.is_some_and(|limit| distance >= limit)
                            || cuts.contains(&pair(a, b))
                            || !union.join(a, b, 0)
                        {
                            break;
                        }
                        p += step;
                        distance += 1;
                    }
                }
            }
        };
        extend_approaches(&mut union, Some(4));
        // Sprite feet lie at the tangent end of a collision quad. Keep that
        // boundary's nearby ground on its owning level; a remote terrace
        // must not lift the landing through the next source row's corners.
        for &(high, low, d, _, _) in &drops {
            let tangent = if d.x == 0 { IVec2::X } else { IVec2::Y };
            for side in [-tangent, tangent] {
                if directions.contains(&(high.x + side.x, high.y + side.y, d.x, d.y)) {
                    continue;
                }
                for (start, step) in [(high, -d), (low, d)] {
                    let Some(owner) = index(start) else {
                        continue;
                    };
                    for distance in 0..=4 {
                        let p = start + step * distance;
                        let (Some(a), Some(b)) = (index(p), index(p + side)) else {
                            continue;
                        };
                        if union.find(a) != union.find(owner) || cuts.contains(&pair(a, b)) {
                            continue;
                        }
                        union.join(a, b, 0);
                    }
                }
            }
        }
        extend_approaches(&mut union, None);
        let mut roots = vec![0; n];
        let mut potential = vec![0.; n];
        let mut ids = vec![usize::MAX; n];
        let mut count = 0;
        for i in 0..n {
            let (root, p) = union.find(i);
            if ids[root] == usize::MAX {
                ids[root] = count;
                count += 1;
            }
            roots[i] = ids[root];
            potential[i] = p as f32;
        }
        let mut neighbors = vec![Vec::<usize>::new(); count];
        let mut rhs = vec![0.; count];
        for y in 0..height {
            for x in 0..width {
                let a = y * width + x;
                for b in [
                    (x + 1 < width).then_some(a + 1),
                    (y + 1 < height).then_some(a + width),
                ]
                .into_iter()
                .flatten()
                {
                    let (ra, rb) = (roots[a], roots[b]);
                    if ra == rb {
                        continue;
                    }
                    neighbors[ra].push(rb);
                    neighbors[rb].push(ra);
                    rhs[ra] += potential[b] - potential[a];
                    rhs[rb] += potential[a] - potential[b];
                }
            }
        }
        let pinned = drops.first().map_or(0, |drop| roots[drop.3]);
        let mut base = vec![0.; count];
        // Only unconstrained bypass ground grades between the fixed terraces.
        // The map-aligned solution is cached independently of camera crops.
        for _ in 0..800 {
            let mut change = 0_f32;
            for i in 0..count {
                if i == pinned || neighbors[i].is_empty() {
                    continue;
                }
                let next = (rhs[i] + neighbors[i].iter().map(|&j| base[j]).sum::<f32>())
                    / neighbors[i].len() as f32;
                change = change.max((next - base[i]).abs());
                base[i] = next;
            }
            if change < 0.00001 {
                break;
            }
        }
        let mut levels = (0..n)
            .map(|i| base[roots[i]] + potential[i])
            .collect::<Vec<_>>();
        let lowest = levels.iter().copied().fold(f32::INFINITY, f32::min);
        for level in &mut levels {
            *level -= lowest;
        }
        let corner = |vertex: IVec2, owner: usize| {
            if let Some(&anchor) = foot_anchors.get(&vertex) {
                if corner_domain(vertex, anchor, &index, &cuts).contains(&owner) {
                    return levels[anchor];
                }
            }
            let domain = corner_domain(vertex, owner, &index, &cuts);
            if domain.is_empty() {
                levels[owner]
            } else {
                domain.iter().map(|&i| levels[i]).sum::<f32>() / domain.len() as f32
            }
        };
        let mut corners = Vec::with_capacity(n);
        for y in 0..height {
            for x in 0..width {
                let p = min + IVec2::new(x as i32, y as i32);
                let i = y * width + x;
                corners.push([
                    corner(p, i),
                    corner(p + IVec2::Y, i),
                    corner(p + IVec2::ONE, i),
                    corner(p + IVec2::X, i),
                ]);
            }
        }
        Self {
            origin: min,
            width,
            height,
            corners,
        }
    }

    /// A south-facing course lowers the route; an opening is a ramp between
    /// its two levels. Independent bypass solves must not make the ground dip
    /// and rise again between authored drops.
    fn south_courses(edges: &[VisualJumpLedge]) -> Self {
        let mut min = IVec2::ZERO;
        let mut max = IVec2::ZERO;
        let mut courses = std::collections::BTreeMap::<i32, Vec<(i32, i32)>>::new();
        for edge in edges {
            min = min.min(edge.start).min(edge.end);
            max = max.max(edge.start).max(edge.end);
            courses
                .entry(edge.start.y)
                .or_default()
                .push((edge.start.x, edge.end.x));
        }
        min -= IVec2::splat(8);
        max += IVec2::splat(8);
        let width = (max.x - min.x) as usize;
        let height = (max.y - min.y) as usize;
        let mut corners = Vec::with_capacity(width * height);
        for y in min.y..max.y {
            for x in min.x..max.x {
                let mut h = [0.; 4];
                for (&plane, spans) in &courses {
                    let cut = spans.iter().any(|&(a, b)| x >= a && x < b);
                    for (i, dy) in [0, 1, 1, 0].into_iter().enumerate() {
                        h[i] += if cut {
                            if y < plane { 1. } else { 0. }
                        } else {
                            (plane - y - dy).clamp(0, 1) as f32
                        };
                    }
                }
                corners.push(h);
            }
        }
        Self {
            origin: min,
            width,
            height,
            corners,
        }
    }

    pub(super) fn height(&self, p: Vec2) -> f32 {
        let p = p - self.origin.as_vec2();
        let x = (p.x.floor() as i32).clamp(0, self.width as i32 - 1) as usize;
        let y = (p.y.floor() as i32).clamp(0, self.height as i32 - 1) as usize;
        let u = (p.x - x as f32).clamp(0., 1.);
        let v = (p.y - y as f32).clamp(0., 1.);
        let c = self.corners[y * self.width + x];
        if u <= v {
            c[0] * (1. - v) + c[1] * (v - u) + c[2] * u
        } else {
            c[0] * (1. - u) + c[2] * v + c[3] * (u - v)
        }
    }
}

#[cfg(test)]
mod stair_tests {
    use super::*;

    #[test]
    fn staggered_south_courses_never_dip_when_travelling_north() {
        let edges = [
            (12, 4, 14),
            (12, 16, 24),
            (20, 4, 14),
            (28, 8, 16),
            (40, 4, 6),
            (40, 8, 14),
            (40, 16, 32),
            (48, 28, 32),
            (56, 4, 8),
            (56, 16, 32),
        ]
        .map(|(y, a, b)| VisualJumpLedge {
            start: IVec2::new(a, y),
            end: IVec2::new(b, y),
            descent: IVec2::Y,
        });
        let field = get_for_family(&edges, true);
        for x in 0..40 {
            for quarter in 0..(72 * 4 - 1) {
                let north = field.height(Vec2::new(x as f32 + 0.5, quarter as f32 / 4.));
                let south = field.height(Vec2::new(x as f32 + 0.5, (quarter + 1) as f32 / 4.));
                assert!(
                    north + 0.0001 >= south,
                    "northward descent at x{x}, quarter{quarter}: {south} -> {north}"
                );
            }
        }
        for edge in &edges {
            let x = (edge.start.x + edge.end.x) as f32 / 2.;
            let y = edge.start.y as f32;
            assert!(
                (field.height(Vec2::new(x, y - 0.001))
                    - field.height(Vec2::new(x, y + 0.001))
                    - 1.)
                    .abs()
                    < 0.0001
            );
        }
    }

    #[test]
    fn interrupted_kanto_shelf_is_level_with_descent_confined_to_stair_row() {
        // Route 1's three adjacent banks, interrupted by two half-block stairs.
        let edges = [(4, 6), (8, 14), (16, 32)].map(|(a, b)| VisualJumpLedge {
            start: IVec2::new(a, 40),
            end: IVec2::new(b, 40),
            descent: IVec2::Y,
        });
        let field = get_for_family(&edges, true);
        let high = field.height(Vec2::new(5., 38.5));
        let low = field.height(Vec2::new(5., 40.5));
        assert!((high - low - 1.).abs() < 0.0001);
        for x in 5..31 {
            assert!(
                (field.height(Vec2::new(x as f32 + 0.5, 38.5)) - high).abs() < 0.0001,
                "shelf bends at column {x}"
            );
            assert!((field.height(Vec2::new(x as f32 + 0.5, 40.5)) - low).abs() < 0.0001);
        }
        for x in [6.5, 7.5, 14.5, 15.5] {
            for t in [0., 0.25, 0.5, 0.75, 0.999] {
                assert!(
                    (field.height(Vec2::new(x, 39. + t)) - (high + (low - high) * t)).abs()
                        < 0.0001
                );
            }
        }
        // Family-scoped cache entries cannot change Johto or forest terrain.
        assert!(Arc::ptr_eq(&get(&edges), &get_for_family(&edges, false)));
        assert!(!Arc::ptr_eq(&get(&edges), &field));
    }
}
