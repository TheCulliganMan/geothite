//! Complete native land domains prevent inland ledges from lifting other islands.
use super::*;
use bevy::prelude::{IVec2, UVec2, Vec2};
use crystal_render_api::{VisualJumpLedge, VisualTerraceDatum};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

#[path = "water_barrier_domains.rs"]
mod water_barrier_domains;
#[path = "mountain_ponds.rs"]
pub(super) mod mountain_ponds;

type SourceCell = (IVec2, VisualTileSource, bool);

struct CachedDatum {
    map: String,
    edges: Vec<VisualJumpLedge>,
    sources: Vec<SourceCell>,
    datum: Option<Arc<VisualTerraceDatum>>,
}

#[derive(Default)]
struct DatumCache {
    entries: VecDeque<CachedDatum>,
}

impl DatumCache {
    fn resolve(
        &mut self,
        map: &str,
        edges: &[VisualJumpLedge],
        sources: &[SourceCell],
    ) -> Option<Arc<VisualTerraceDatum>> {
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.map == map && entry.edges == edges && entry.sources == sources)
        {
            let entry = self.entries.remove(index).unwrap();
            let datum = entry.datum.clone();
            self.entries.push_back(entry);
            return datum;
        }
        let datum = build_source_terrace_datum(map, edges, sources);
        // Keep complete input evidence, including ordering/overlap ownership,
        // rather than a collision-prone hash or a camera-dependent crop.
        // Oversized inputs still use the identical uncached calculation.
        if sources.len() <= 65_536 && edges.len() <= 65_536 {
            if self.entries.len() == 4 {
                self.entries.pop_front();
            }
            self.entries.push_back(CachedDatum {
                map: map.to_owned(),
                edges: edges.to_vec(),
                sources: sources.to_vec(),
                datum: datum.clone(),
            });
        }
        datum
    }
}

/// Samples must cover complete native maps/connections, never a camera crop.
/// Earlier maps own overlapping coordinates, matching production selection.
/// The boolean is read-only authored land-collision evidence, including hops.
pub fn source_terrace_datum(
    map: &str,
    edges: &[VisualJumpLedge],
    sources: &[(IVec2, VisualTileSource, bool)],
) -> Option<Arc<VisualTerraceDatum>> {
    static CACHE: OnceLock<Mutex<DatumCache>> = OnceLock::new();
    CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .resolve(map, edges, sources)
}

fn build_source_terrace_datum(
    map: &str,
    edges: &[VisualJumpLedge],
    sources: &[SourceCell],
) -> Option<Arc<VisualTerraceDatum>> {
    if !crate::cave_water::outdoor(map) || map.contains("Forest") {
        return None;
    }
    let mut native = BTreeMap::new();
    for (p, s, land) in sources {
        native.entry((p.y, p.x)).or_insert((s, *land));
    }
    if native.is_empty() {
        return None;
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
    let n = w * h;
    let water_barriers=water_barrier_domains::resolve(&native);
    let flat=|mineral:bool| {
        let mut levels=vec![[0.;4];n];
        let ponds=mineral && mountain_ponds::raise(&native,lo,w,&mut levels);
        (ponds || !water_barriers.is_empty()).then(||Arc::new(VisualTerraceDatum {
            origin:lo,size:UVec2::new(w as u32,h as u32),levels:levels.into(),
            open_water:None,water_barriers:water_barriers.clone(),
        }))
    };
    if edges.is_empty() {return flat(true);}
    let index = |p: IVec2| {
        let p = p - lo;
        (p.x >= 0 && p.y >= 0 && p.x < size.x && p.y < size.y)
            .then(|| p.y as usize * w + p.x as usize)
    };
    let point = |i: usize| lo + IVec2::new((i % w) as i32, (i / w) as i32);
    let neighbors = |i: usize| {
        [
            (i % w > 0).then(|| i - 1),
            (i % w + 1 < w).then(|| i + 1),
            (i >= w).then(|| i - w),
            (i / w + 1 < h).then(|| i + w),
        ]
    };
    let kanto = native
        .values()
        .any(|(s, _)| s.tileset_id.as_ref() == "kanto")
        && !native
            .values()
            .any(|(s, _)| matches!(s.tileset_id.as_ref(), "johto" | "johto_modern" | "forest"));
    let mut dry = vec![false; n];
    let mut wet = vec![false; n];
    let mut sea = vec![false; n];
    let mut ground = vec![false; n];
    let mut known = vec![false; n];
    let mut kanto_water = vec![false; n];
    for (&(y, x), (s, land)) in &native {
        let i = index(IVec2::new(x, y)).unwrap();
        known[i] = true;
        kanto_water[i] = s.tileset_id.as_ref() == "kanto";
        let barrier=water_barriers.binary_search_by_key(&(y,x),|p|(p.y,p.x)).is_ok();
        wet[i] = barrier || terraces::water_owns_height(s) || elevated_piers::level_surface(map, s);
        // Native water-backed rocks are part of the same coastal contact as
        // their water underlay. Leaving this strip out separates the land
        // domain from the river and extrudes an unrelated inland terrace
        // into a retaining wall behind otherwise sea-level shoreline rocks.
        sea[i] = barrier || shape_for_source(s) == CellShape::Water
            || crate::kanto_cliff::shoreline_round_barrier_local(s).is_some();
        dry[i] = *land && !wet[i];
        ground[i] = dry[i] && support_height(s, SOURCE_TILE_HEIGHT).abs() < 0.001;
    }
    // Open water remains sea level. Complete Johto lakes instead meet their
    // lowest surrounding terrace; a camera crop must never choose this level.
    // Ordinary Kanto ponds retain their independent producer; sealed mineral
    // pockets are calibrated against their plateau after the land field.
    let mut lakes = Vec::new();
    let mut rivers=Vec::new();
    let mut river_openness=Vec::new();
    let inland=!crate::environment_fx::coastal_map(map)
        && !matches!(map,"CinnabarIsland"|"PalletTown"|"VermilionCity"|"VermilionPort"|"OlivinePort"|"Route12"|"Route13");
    let mut seen = vec![false; n];
    for start in 0..n {
        if !wet[start] || seen[start] {
            continue;
        }
        let mut body = vec![start];
        seen[start] = true;
        let mut cursor = 0;
        let mut open = false;
        while cursor < body.len() {
            let i = body[cursor];
            cursor += 1;
            for j in neighbors(i) {
                let Some(j) = j else {
                    open = true;
                    continue;
                };
                open |= !known[j];
                if wet[j] && !seen[j] {
                    seen[j] = true;
                    body.push(j);
                }
            }
        }
        let johto = body.iter().all(|&i| native.get(&(point(i).y, point(i).x))
            .is_some_and(|(s, _)| matches!(s.tileset_id.as_ref(), "johto" | "johto_modern")
                && shape_for_source(s) == CellShape::Water));
        let waterfall = body.iter().any(|&i| neighbors(i).into_iter().flatten().any(|j|
            native.get(&(point(j).y, point(j).x)).is_some_and(|(s, _)|
                shape_for_source(s) == CellShape::Waterfall)));
        if inland && !waterfall && body.iter().all(|&i|kanto_water[i])
            && body.iter().any(|&i|native.get(&(point(i).y,point(i).x)).is_some_and(|(s,_)|
                crate::kanto_cliff::shoreline_round_barrier_local(s).is_some())) {
            for &i in &body {sea[i]=false;}
            river_openness.push(open);
            rivers.push(body.clone());
        }
        if !open && (body.iter().all(|&i| kanto_water[i]) || (johto && !waterfall)) {
            for &i in &body { sea[i] = false; }
            if johto && !waterfall { lakes.push(body); }
        }
    }
    // A pool attached to a waterfall owns its authored level separately.
    let mut falls = VecDeque::new();
    for (&(y, x), (s, _)) in &native {
        if shape_for_source(s) == CellShape::Waterfall {
            falls.extend(
                neighbors(index(IVec2::new(x, y)).unwrap())
                    .into_iter()
                    .flatten(),
            );
        }
    }
    while let Some(i) = falls.pop_front() {
        if sea[i] {
            sea[i] = false;
            falls.extend(neighbors(i).into_iter().flatten());
        }
    }
    let mut domain = vec![usize::MAX; n];
    let mut links = vec![Vec::new(); n];
    let ends = |edge: &VisualJumpLedge, step: i32| {
        let tangent = if edge.descent.x == 0 {
            IVec2::X
        } else {
            IVec2::Y
        };
        let plane = edge.start + tangent * (step + 1);
        index(plane - edge.descent * 3).zip(index(plane + edge.descent))
    };
    for edge in edges {
        for step in (0..(edge.end - edge.start).abs().element_sum()).step_by(2) {
            if let Some((a, b)) = ends(edge, step).filter(|&(a, b)| dry[a] && dry[b]) {
                links[a].push(b);
                links[b].push(a);
            }
        }
    }
    let mut groups = Vec::new();
    let mut coastal = Vec::new();
    for start in 0..n {
        if !dry[start] || domain[start] != usize::MAX {
            continue;
        }
        let id = groups.len();
        let mut cells = vec![start];
        domain[start] = id;
        let mut cursor = 0;
        let mut coast = false;
        while cursor < cells.len() {
            let i = cells[cursor];
            cursor += 1;
            for j in neighbors(i)
                .into_iter()
                .flatten()
                .chain(links[i].iter().copied())
            {
                coast |= sea[j];
                if dry[j] && domain[j] == usize::MAX {
                    domain[j] = id;
                    cells.push(j);
                }
            }
        }
        groups.push(cells);
        coastal.push(coast);
    }
    if groups.is_empty() {
        return flat(false);
    }
    // Associate blocked shoreline art and scenery with the nearest authored
    // land domain. They cannot connect two domains through a wall or mountain.
    let mut queue = VecDeque::new();
    for i in 0..n {
        if domain[i] != usize::MAX {
            queue.push_back(i);
        }
    }
    while let Some(i) = queue.pop_front() {
        for j in neighbors(i).into_iter().flatten() {
            if domain[j] == usize::MAX {
                domain[j] = domain[i];
                queue.push_back(j);
            }
        }
    }
    for (&(y, x), _) in &native {
        let i = index(IVec2::new(x, y)).unwrap();
        if !wet[i] && neighbors(i).into_iter().flatten().any(|j| sea[j]) {
            coastal[domain[i]] = true;
        }
    }
    if !coastal.iter().any(|&v| v) && lakes.is_empty() && rivers.is_empty() {
        return flat(false);
    }
    let mut fields = Vec::new();
    let mut baselines = Vec::new();
    for (id, cells) in groups.iter().enumerate() {
        // Inland water separates land domains just as the sea does. A ledge
        // across a river cannot lift this bank merely because both maps were
        // included in the complete source halo. Keep only real takeoff and
        // landing contacts belonging to this domain, in its own coordinates.
        let local = coastal[id] || !rivers.is_empty();
        let attached = if local {
            edges
                .iter()
                .copied()
                .filter(|edge| {
                    let length = (edge.end - edge.start).abs().element_sum();
                    (0..length).step_by(2).any(|step| {
                        ends(edge, step).is_some_and(|(a, b)| {
                            dry[a] && dry[b] && domain[a] == id && domain[b] == id
                        })
                    })
                })
                .collect::<Vec<_>>()
        } else {
            edges.to_vec()
        };
        // A connected source halo can include a distant forest atlas. Its
        // presence cannot switch a Kanto bank from flat directed courses to
        // the generic slope solve. Resolve the family from this domain's real
        // takeoff/landing contacts, before translating its attached ledges.
        let kanto_courses=if local {
            let mut contacts=0;
            let all_kanto=attached.iter().all(|edge| {
                let length=(edge.end-edge.start).abs().element_sum();
                (0..length).step_by(2).filter_map(|step|ends(edge,step))
                    .filter(|&(a,b)|dry[a] && dry[b] && domain[a]==id && domain[b]==id)
                    .all(|(a,b)| {
                        contacts+=1;
                        [a,b].into_iter().all(|i| {
                            let p=point(i);
                            native.get(&(p.y,p.x)).is_some_and(|(s,_)|s.tileset_id.as_ref()=="kanto")
                        })
                    })
            });
            all_kanto && contacts>0
        } else {kanto};
        // TEMP_TERRACE_BANK_OWNERSHIP
        if map=="Route24" && std::env::var_os("CRYSTAL_TERRACE_BANK_DIAGNOSTIC").is_some()
            && [IVec2::new(23,16),IVec2::new(27,16)].into_iter()
                .any(|p|index(p).is_some_and(|i|domain[i]==id)) {
            eprintln!("BANK_DOMAIN id={id} local={local} kanto={kanto_courses} coastal={} edge_count={} edges={attached:?}",coastal[id],attached.len());
        }
        // END_TEMP_TERRACE_BANK_OWNERSHIP
        let origin = if local {
            point(cells[0])
        } else {
            IVec2::ZERO
        };
        let attached = attached
            .into_iter()
            .map(|mut edge| {
                edge.start -= origin;
                edge.end -= origin;
                edge
            })
            .collect::<Vec<_>>();
        // Each land domain uses its own authenticated drops. Kanto keeps
        // the same flat south-facing courses and bypass ramps as Route 26;
        // an inland river must not switch its bank to the dense slope solve.
        let field = terrace_levels::get_for_family(&attached, kanto_courses);
        let mut base = f32::INFINITY;
        if coastal[id] {
            for &i in cells.iter().filter(|&&i| ground[i]) {
                for d in [
                    Vec2::splat(0.0001),
                    Vec2::new(0.0001, 0.9999),
                    Vec2::splat(0.9999),
                    Vec2::new(0.9999, 0.0001),
                ] {
                    base = base.min(field.height((point(i) - origin).as_vec2() + d));
                }
            }
        }
        fields.push((field, origin));
        baselines.push(if base.is_finite() { base.max(0.) } else { 0. });
    }
    if fields.is_empty() {
        return flat(false);
    }
    // An inland river reaching the source-map border is still a river,
    // not an ocean. Calibrate its complete open body against real walkable
    // shores before zero-sea contact flattens their shared corners.
    let mut river_levels=Vec::new();
    for body in &rivers {
        let mut rim=f32::INFINITY;
        for &i in body {
            for j in neighbors(i).into_iter().flatten().filter(|&j|ground[j]) {
                let id=domain[j];
                let p=(point(i).as_vec2()+point(j).as_vec2())*0.5+Vec2::splat(0.5);
                rim=rim.min((fields[id].0.height(p-fields[id].1.as_vec2())-baselines[id]).max(0.));
            }
        }
        river_levels.push(rim);
    }
    let river_rim=river_levels.iter().zip(&river_openness).filter_map(|(&level,&open)|open.then_some(level))
        .fold(f32::INFINITY,f32::min);
    let open_water=river_rim.is_finite().then(|| {
        let mut river_cells=rivers.iter().zip(&river_openness).filter(|(_,open)|**open)
            .flat_map(|(body,_)|body.iter()).map(|&i|point(i)).collect::<Vec<_>>();
        river_cells.extend((0..n).filter(|&i|!known[i]).map(point));
        river_cells.sort_unstable_by_key(|p|(p.y,p.x));river_cells.dedup();
        crystal_render_api::VisualOpenWaterDatum {level:river_rim,cells:river_cells.into()}
    });
    let mut lake_level = vec![None; n];
    for ((body,&level),&open) in rivers.iter().zip(&river_levels).zip(&river_openness) {
        let level=if open {river_rim}else{level};
        if level.is_finite() {for &i in body {lake_level[i]=Some(level);}}
    }
    for body in lakes {
        let mut rim = f32::INFINITY;
        for &i in &body {
            for j in neighbors(i).into_iter().flatten().filter(|&j| !wet[j] && known[j]) {
                let id = domain[j];
                let p = (point(i).as_vec2() + point(j).as_vec2()) * 0.5 + Vec2::splat(0.5);
                rim = rim.min((fields[id].0.height(p - fields[id].1.as_vec2())
                    - baselines[id]).max(0.));
            }
        }
        if rim.is_finite() { for i in body { lake_level[i] = Some(rim); } }
    }
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
    let mut levels = (0..n)
        .map(|i| {
            // The bounding rectangle also contains gaps between connected
            // native maps. Borrowed camera-halo water in those gaps has no
            // authored land domain and must retain the sea-level baseline.
            if !known[i] { return [0.;4]; }
            [IVec2::ZERO, IVec2::Y, IVec2::ONE, IVec2::X].map(|d| {
                if wet[i] { return lake_level[i].unwrap_or(0.); }
                let vertex = point(i) + d;
                let owners = terrace_levels::corner_domain(vertex, i, &index, &cuts);
                // Each complete enclosed body has one level, shared by its
                // entire water plane and shore contacts, even in partial views.
                if let Some(level) = owners.iter().filter_map(|&j| lake_level[j])
                    .reduce(f32::min) { return level; }
                // Grade the land-side shelf into its fixed sea contact.
                if owners.iter().any(|&j| sea[j]) {
                    return 0.;
                }
                let sum = owners
                    .iter()
                    .map(|&j| {
                        let id = domain[j];
                        let inside = point(j).as_vec2() + Vec2::splat(0.5);
                        let toward = (inside - vertex.as_vec2()).normalize_or_zero() * 0.0001;
                        (fields[id]
                            .0
                            .height((vertex - fields[id].1).as_vec2() + toward)
                            - baselines[id])
                            .max(0.)
                    })
                    .sum::<f32>();
                sum / owners.len().max(1) as f32
            })
        })
        .collect::<Vec<_>>();
    // TEMP_TERRACE_BANK_CELLS
    if map=="Route24" && std::env::var_os("CRYSTAL_TERRACE_BANK_DIAGNOSTIC").is_some() {
        for x in 18..32 {let p=IVec2::new(x,16);if let Some(i)=index(p) {
            eprintln!("BANK_CELL p={p:?} native={:?} dry={} wet={} domain={} level={:?}",native.get(&(p.y,p.x)),dry[i],wet[i],domain[i],levels[i]);
        }}
    }
    // END_TEMP_TERRACE_BANK_CELLS
    mountain_ponds::raise(&native,lo,w,&mut levels);
    Some(Arc::new(VisualTerraceDatum {
        origin: lo,
        size: UVec2::new(w as u32, h as u32),
        levels: levels.into(),
        water_barriers,
        open_water,
    }))
}

pub(super) fn sample(datum: Option<&VisualTerraceDatum>, p: Vec2) -> Option<f32> {
    let d = datum?;
    if d.size.x == 0 || d.size.y == 0 || d.levels.len() != d.size.x as usize * d.size.y as usize {
        return None;
    }
    let p = p - d.origin.as_vec2();
    let x = (p.x.floor() as i32).clamp(0, d.size.x as i32 - 1) as usize;
    let y = (p.y.floor() as i32).clamp(0, d.size.y as i32 - 1) as usize;
    let (u, v) = (
        (p.x - x as f32).clamp(0., 1.),
        (p.y - y as f32).clamp(0., 1.),
    );
    let c = d.levels[y * d.size.x as usize + x];
    Some(if u <= v {
        c[0] * (1. - v) + c[1] * (v - u) + c[2] * u
    } else {
        c[0] * (1. - u) + c[2] * v + c[3] * (u - v)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(atlas: &str) -> (Vec<(IVec2, VisualTileSource, bool)>, Vec<VisualJumpLedge>) {
        let mut cells = (0..256)
            .map(|i| {
                let (x, y) = (i % 16, i / 16);
                let land = (x < 4 && y < 4) || ((8..12).contains(&x) && y < 12);
                let mut source = super::super::tests::source_with_tile(
                    if land { 1 } else { 0x35 },
                    (x % 4) as u8,
                    (y % 4) as u8,
                    if land { 6 } else { 0x14 },
                );
                source.tileset_id = atlas.into();
                (IVec2::new(x as i32, y as i32), source, land)
            })
            .collect::<Vec<_>>();
        // A blocked dry strip must not join the islands. Conversely the
        // authored jump must connect its two standing landings across a wall.
        for x in 4..8 {
            cells[x].1.tile_index = 6;
        }
        for y in 4..7 {
            for x in 8..12 {
                cells[y * 16 + x].2 = false;
            }
        }
        (
            cells,
            vec![VisualJumpLedge {
                start: IVec2::new(8, 6),
                end: IVec2::new(12, 6),
                descent: IVec2::Y,
            }],
        )
    }
    #[test]
    fn native_shore_rock_belt_keeps_open_water_and_land_on_the_same_coastal_datum() {
        let (mut sources, edges)=fixture("kanto");
        for (p,source,land) in &mut sources {
            let island=(8..12).contains(&p.x) && p.y<12;
            let belt=(6..14).contains(&p.x) && p.y<14 && !island;
            *land=island;
            source.metatile_id=if belt {0x14}else if island {1}else{0x35};
            source.tile_index=if belt {
                [[0x2a,0x2b],[0x3a,0x3b]][p.y as usize%2][p.x as usize%2]
            }else if island {6}else{0x14};
        }
        let datum=source_terrace_datum("CoastalTown",&edges,&sources).unwrap();
        for y in 0..12 {
            let west=&datum.levels[y*16+8];
            let east=&datum.levels[y*16+11];
            assert_eq!([west[0],west[1],east[2],east[3]],[0.;4],
                "the water-backed rock belt must anchor the dry shore at row {y}");
        }
        for (p,source,_) in &sources {
            if terraces::water_owns_height(source) {
                assert_eq!(datum.levels[p.y as usize*16+p.x as usize],[0.;4],
                    "open water and its source-authenticated rocks stay at sea level");
            }
        }
        assert!(datum.levels.iter().any(|v|v.iter().any(|&h|h>0.)),
            "coastal calibration must preserve the actual inland terrace");
    }

    #[test]
    fn inland_river_border_and_source_gaps_share_the_real_shore_level_but_seas_stay_zero() {
        let (mut sources,_)=fixture("kanto");
        let edges=vec![VisualJumpLedge {start:IVec2::new(0,6),end:IVec2::new(16,6),descent:IVec2::Y}];
        for (p,s,land) in &mut sources {
            let pool=p.x<4 && p.y<2;
            let rocks=p.x<6 && p.y<4 && !pool;
            *land=!pool && !rocks;
            s.metatile_id=if rocks {0x14}else if pool {0x35}else{1};
            s.tile_index=if rocks {[[0x2a,0x2b],[0x3a,0x3b]][p.y as usize%2][p.x as usize%2]}
                else if pool {0x14}else{6};
        }
        sources.retain(|(p,_,_)|*p!=IVec2::new(1,1));
        let river=source_terrace_datum("RiverTown",&edges,&sources).unwrap();
        let level=river.open_water.as_ref().expect("an inland source edge is not an ocean").level;
        assert!((level-1.).abs()<0.001,"river follows its upstream shore: {level}");
        for p in [IVec2::new(0,0),IVec2::new(4,1),IVec2::new(1,1),IVec2::new(-10,1)] {
            assert_eq!(river.open_water_at(p),Some(level),"native water, rocks, gaps and continuation share one plane");
        }
        assert_eq!(river.open_water_at(IVec2::new(8,8)),None,"dry source never acquires river ownership");
        for map in ["CinnabarIsland","Route26","PalletTown","VermilionCity"] {
            let sea=source_terrace_datum(map,&edges,&sources).unwrap();
            assert!(sea.open_water.is_none(),"{map} ocean stays at sea level");
            assert_eq!(sea.levels[0],[0.;4]);
        }
        let (_,fall,land)=sources.iter_mut().find(|(p,_,_)|*p==IVec2::new(6,1)).unwrap();
        fall.tileset_id="cave".into();fall.metatile_id=0x2c;fall.tile_index=0x40;*land=false;
        assert!(source_terrace_datum("RiverTown",&edges,&sources).is_none_or(|d|d.open_water.is_none()),
            "a waterfall's separately authored pools cannot inherit an inland continuation");
    }

    #[test]
    fn a_ledge_across_an_inland_river_cannot_raise_the_unrelated_bank() {
        let (mut sources,_)=fixture("kanto");
        for (p,s,land) in &mut sources {
            let water=(4..8).contains(&p.x);
            let rocks=water && matches!(p.x,4|7);
            *land=!water;
            s.metatile_id=if rocks {0x14}else if water {0x35}else{1};
            s.tile_index=if rocks {[[0x2a,0x2b],[0x3a,0x3b]][p.y as usize%2][p.x as usize%2]}
                else if water {0x14}else{6};
        }
        let edges=[VisualJumpLedge {start:IVec2::new(8,8),end:IVec2::new(16,8),descent:IVec2::Y}];
        let datum=source_terrace_datum("RiverTown",&edges,&sources).unwrap();
        for y in 0..16 {for x in 0..4 {
            assert_eq!(datum.levels[y*16+x],[0.;4],
                "an unrelated ledge across the river cannot create a retaining wall at ({x},{y})");
        }}
        assert_eq!(datum.open_water.as_ref().unwrap().level,0.,"one level for the connected water body");
        assert!(sample(Some(&datum),Vec2::new(12.5,6.5)).unwrap()
            >sample(Some(&datum),Vec2::new(12.5,9.5)).unwrap()+0.9,
            "the actual ledge on its own bank keeps its directed drop");
        let delta=IVec2::new(-32,24);
        let shifted=sources.iter().map(|(p,s,land)|(*p+delta,s.clone(),*land)).collect::<Vec<_>>();
        let shifted_edges=edges.map(|edge|VisualJumpLedge {
            start:edge.start+delta,end:edge.end+delta,..edge
        });
        let other=source_terrace_datum("RiverTown",&shifted_edges,&shifted).unwrap();
        assert_eq!(other.origin,datum.origin+delta);
        assert_eq!(other.levels,datum.levels,"changing the active map cannot move either bank's height");
        assert_eq!(other.open_water.as_ref().unwrap().level,datum.open_water.as_ref().unwrap().level);
    }

    #[test]
    fn inland_kanto_courses_stay_flat_with_a_foreign_atlas_in_the_source_halo() {
        let (mut sources,_)=fixture("kanto");
        for (p,s,land) in &mut sources {
            let water=(4..8).contains(&p.x);
            let rocks=water && matches!(p.x,4|7);
            *land=!water;
            s.metatile_id=if rocks {0x14}else if water {0x35}else{1};
            s.tile_index=if rocks {[[0x2a,0x2b],[0x3a,0x3b]][p.y as usize%2][p.x as usize%2]}
                else if water {0x14}else{6};
        }
        let edges=[VisualJumpLedge {start:IVec2::new(8,6),end:IVec2::new(12,6),descent:IVec2::Y},
            VisualJumpLedge {start:IVec2::new(12,12),end:IVec2::new(16,12),descent:IVec2::Y}];
        let native=source_terrace_datum("Route24",&edges,&sources).unwrap();
        let mut foreign=sources[0].1.clone();foreign.tileset_id="forest".into();
        sources.push((IVec2::new(-16,-16),foreign,false));
        let mixed=source_terrace_datum("Route24",&edges,&sources).unwrap();
        for y in [2.5,8.5,14.5] {
            let mut heights=Vec::new();
            for x in [9.5,10.5,11.5,12.5,13.5,14.5] {
                let a=sample(Some(&native),Vec2::new(x,y)).unwrap();
                let b=sample(Some(&mixed),Vec2::new(x,y)).unwrap();
                assert!((a-b).abs()<0.0001,"a foreign halo cannot change the bank at ({x},{y}): {a} vs {b}");
                heights.push(b);
            }
            let expected=if y<6. {2.}else if y<12. {1.}else{0.};
            assert!(heights.iter().all(|h|(h-expected).abs()<0.0001),
                "ground between authored drops is a flat course, not a sideways slope: {heights:?}");
        }
        assert_eq!(mixed.open_water.as_ref().unwrap().level,0.);
    }

    #[test]
    fn connected_maps_closing_a_rock_lined_river_do_not_drop_it_to_the_legacy_sea_plane() {
        let (mut sources,_)=fixture("kanto");
        let edges=vec![VisualJumpLedge {start:IVec2::new(0,10),end:IVec2::new(16,10),descent:IVec2::Y}];
        for (p,s,land) in &mut sources {
            let water=(4..8).contains(&p.x) && (3..5).contains(&p.y);
            let rocks=(2..10).contains(&p.x) && (2..6).contains(&p.y) && !water;
            *land=!water && !rocks;
            s.metatile_id=if rocks {0x14}else if water {0x35}else{1};
            s.tile_index=if rocks {[[0x2a,0x2b],[0x3a,0x3b]][p.y as usize%2][p.x as usize%2]}
                else if water {0x14}else{6};
        }
        let datum=source_terrace_datum("RiverTown",&edges,&sources)
            .expect("the source-complete river needs a datum, not a global route-height fallback");
        assert!(datum.open_water.is_none(),"a genuinely closed river does not lift unrelated border water");
        for (p,s,_) in &sources {if terraces::water_owns_height(s) {
            assert_eq!(datum.levels[p.y as usize*16+p.x as usize],[1.;4],
                "water and native shore rocks must meet their real upstream rim at {p:?}");
        }}
    }

    #[test]
    fn gaps_between_complete_maps_cannot_extrapolate_a_raised_terrace() {
        for atlas in ["johto", "johto_modern"] {
            let (mut sources, edges) = fixture(atlas);
            let gaps = [IVec2::new(12,1), IVec2::new(13,1),
                IVec2::new(12,2), IVec2::new(13,2)];
            sources.retain(|(point,_,_)| !gaps.contains(point));
            let datum = source_terrace_datum("Route42", &edges, &sources).unwrap();
            for point in gaps {
                let cell=point-datum.origin;
                let i=cell.y as usize*datum.size.x as usize+cell.x as usize;
                assert_eq!(datum.levels[i],[0.;4],"{atlas}: absent native cell {point:?}");
                assert_eq!(sample(Some(&datum),point.as_vec2()+Vec2::splat(0.5)),Some(0.));
            }
            assert!(datum.levels.iter().any(|corners|corners.iter().any(|&v|v>0.)),
                "the actual island must retain its authored elevation");
        }
    }
    #[test]
    fn enclosed_johto_lake_meets_its_rim_without_steps_or_camera_dependence() {
        for atlas in ["johto", "johto_modern"] {
            let (mut sources, edges) = fixture(atlas);
            for (p, source, land) in &mut sources {
                let lake = (9..11).contains(&p.x) && (1..4).contains(&p.y);
                source.tile_index = if lake { 0x14 } else { 6 };
                source.metatile_id = if lake { 0x35 } else { 1 };
                *land = !lake;
            }
            let datum = source_terrace_datum("LakeTown", &edges, &sources).unwrap();
            let g = GridGeometry {width:16,height:16,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let mut frame = super::super::tests::frame(16,16,
                sources.iter().map(|(_,s,_)|s.clone()).collect());
            frame.map_id="LakeTown".into();
            let mut mesh = TerrainMeshData {authored_cells:vec![None;256],footing_heights:vec![0.;256],..Default::default()};
            for y in 1..4 { for x in 9..11 {
                assert!((sample(Some(&datum),Vec2::new(x as f32+0.4,y as f32+0.6)).unwrap()-1.).abs()<0.002);
                let (w,e,n,s)=g.bounds(x,y);
                append_top(&mut mesh.animated_textured,[w,e,n,s],crate::profile::WATER_HEIGHT,(0.,1.,0.,1.));
            } }
            let old=mesh.animated_textured.clone();
            terraces::finish_with_datum(&mut mesh,"LakeTown",&frame.tiles.iter().collect::<Vec<_>>(),&g,[0,0],&edges,Some(&datum));
            for (before,after) in old.positions.iter().zip(&mesh.animated_textured.positions) {
                assert!((after[1]-before[1]-crate::profile::JUMP_LEDGE_HEIGHT).abs()<0.002);
                assert_eq!([before[0],before[2]],[after[0],after[2]]);
            }
            assert_eq!(old.indices,mesh.animated_textured.indices);
            assert_eq!(old.normals,mesh.animated_textured.normals);
            assert_eq!(old.uvs,mesh.animated_textured.uvs);
            for y in 1..4 { for x in 9..11 {
                assert!((mesh.footing_heights[y*16+x]-crate::profile::JUMP_LEDGE_HEIGHT).abs()<0.002);
            } }
            // A view containing just part of the lake samples the complete
            // source body's same level, rather than inventing a local rim.
            let cropped=terraces::Field::new(&edges,&g,[9,1]).with_datum(Some(&datum));
            assert!((cropped.height(Vec2::new(4.,4.))-crate::profile::JUMP_LEDGE_HEIGHT).abs()<0.002);
            let (_, waterfall, land)=&mut sources[2*16+8];
            waterfall.tileset_id="cave".into();waterfall.metatile_id=0x2c;waterfall.tile_index=0x40;*land=false;
            assert!(source_terrace_datum("LakeTown",&edges,&sources).is_none(),
                "authored waterfall pools retain their separate level");
        }
    }

    #[test]
    fn datum_cache_reuses_complete_evidence_and_invalidates_waterfall_map_and_edges() {
        let (sources, edges) = fixture("johto");
        let mut cache = DatumCache::default();
        let original = cache.resolve("Route27", &edges, &sources).unwrap();
        let repeated = cache.resolve("Route27", &edges, &sources).unwrap();
        assert!(Arc::ptr_eq(&original, &repeated));
        assert_eq!(
            Some(original.as_ref()),
            build_source_terrace_datum("Route27", &edges, &sources).as_deref()
        );

        let mut waterfall = sources.clone();
        let (_, tile, land) = &mut waterfall[255];
        tile.tileset_id = "cave".into();
        tile.metatile_id = 0x2c;
        tile.tile_index = 0x40;
        *land = false;
        assert!(
            cache.resolve("Route27", &edges, &waterfall).is_none(),
            "cached coastal heights must not replace a waterfall pool's own level"
        );
        assert!(cache.resolve("Route27", &edges, &waterfall).is_none());
        assert!(cache.resolve("IlexForest", &edges, &sources).is_none());
        assert!(cache.resolve("Route27", &[], &sources).is_none());
        let restored = cache.resolve("Route27", &edges, &sources).unwrap();
        assert_eq!(original, restored);
        assert!((sample(Some(&restored), Vec2::new(9., 2.)).unwrap() - 1.).abs() < 0.002);
        assert_eq!(sample(Some(&restored), Vec2::new(2., 2.)), Some(0.));
    }

    #[test]
    fn datum_cache_preserves_source_overlap_priority_and_bounds_retention() {
        let (sources, edges) = fixture("johto");
        let mut cache = DatumCache::default();
        for i in 0..7 {
            let map = format!("Cache{i}Town");
            let mut overlapping = sources.clone();
            let mut tile = overlapping[0].clone();
            tile.1.tile_index = 0x14;
            tile.1.metatile_id = 0x35;
            tile.2 = false;
            if i % 2 == 0 {
                overlapping.insert(0, tile);
            } else {
                overlapping.push(tile);
            }
            let actual = cache.resolve(&map, &edges, &overlapping);
            let expected = build_source_terrace_datum(&map, &edges, &overlapping);
            assert!(
                actual.is_some(),
                "the overlap check must exercise an outdoor height field"
            );
            assert_eq!(actual, expected);
            assert!(cache.entries.len() <= 4);
        }
        let expected = build_source_terrace_datum("Route27", &edges, &sources);
        assert_eq!(cache.resolve("Route27", &edges, &sources), expected);
        assert_eq!(cache.entries.len(), 4);
    }

    #[test]
    fn remote_island_ledge_cannot_raise_a_flat_coast_and_authored_drop_survives() {
        for atlas in ["johto", "johto_modern", "kanto"] {
            let (sources, edges) = fixture(atlas);
            let datum = source_terrace_datum("UnseenTown", &edges, &sources).unwrap();
            let old = terrace_levels::get_for_family(&edges, atlas == "kanto");
            assert!(
                old.height(Vec2::new(2., 2.)) > 0.01,
                "the original field reproduces island bleed"
            );
            for y in 0..4 {
                for x in 0..4 {
                    assert_eq!(
                        sample(Some(&datum), Vec2::new(x as f32 + 0.5, y as f32 + 0.5)),
                        Some(0.)
                    );
                }
            }
            let high = sample(Some(&datum), Vec2::new(9., 5.99)).unwrap();
            let low = sample(Some(&datum), Vec2::new(9., 6.01)).unwrap();
            assert!(
                (high - low - 1.).abs() < 0.002,
                "the actual island keeps its one-level drop: {atlas} {high}/{low}"
            );
            assert_eq!(
                sample(Some(&datum), Vec2::new(12., 2.)),
                Some(0.),
                "the sea contact cannot inherit an inland wall"
            );
            let mut reordered = sources.clone();
            reordered.reverse();
            let mut overlap = sources.clone();
            overlap.extend(sources.iter().map(|(p, s, _)| {
                let mut s = s.clone();
                s.tile_index = 0x14;
                s.metatile_id = 0x35;
                (*p, s, false)
            }));
            assert_eq!(
                source_terrace_datum("UnseenTown", &edges, &overlap),
                Some(datum.clone()),
                "the selected map's native cells own overlapping connection coordinates"
            );
            assert_eq!(
                source_terrace_datum("UnseenTown", &edges, &reordered),
                Some(datum)
            );
        }
    }
    #[test]
    fn one_complete_source_field_stays_registered_across_camera_crops() {
        let (sources, edges) = fixture("johto");
        let datum = source_terrace_datum("Route27", &edges, &sources).unwrap();
        let g = GridGeometry {
            width: 16,
            height: 16,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let first = terraces::Field::new(&edges, &g, [0, 0]).with_datum(Some(&datum));
        let moved = terraces::Field::new(&edges, &g, [4, -2]).with_datum(Some(&datum));
        for p in [
            Vec2::new(2.5, 2.5),
            Vec2::new(9.5, 2.5),
            Vec2::new(9.5, 8.5),
        ] {
            assert!(
                (first.height(p * 8.) - moved.height((p - Vec2::new(4., -2.)) * 8.)).abs() < 0.0001
            );
        }
        let delta = IVec2::new(-40, 20);
        let shifted = sources
            .iter()
            .map(|(p, s, land)| (*p + delta, s.clone(), *land))
            .collect::<Vec<_>>();
        let shifted_edges = edges
            .iter()
            .map(|e| VisualJumpLedge {
                start: e.start + delta,
                end: e.end + delta,
                descent: e.descent,
            })
            .collect::<Vec<_>>();
        let other = source_terrace_datum("UnseenTown", &shifted_edges, &shifted).unwrap();
        assert_eq!(other.origin, datum.origin + delta);
        assert_eq!(
            other.levels, datum.levels,
            "changing the active-map coordinate origin cannot change land heights"
        );
    }
    #[test]
    fn forest_indoor_empty_and_inland_sources_retain_legacy_elevation() {
        let (mut sources, edges) = fixture("johto");
        for map in ["IlexForest", "PlayersHouse1F"] {
            assert!(source_terrace_datum(map, &edges, &sources).is_none());
        }
        assert!(source_terrace_datum("Route27", &[], &sources).is_none());
        assert!(source_terrace_datum("Route27", &edges, &[]).is_none());
        for (_, s, land) in &mut sources {
            *land = true;
            s.metatile_id = 1;
            s.tile_index = 6;
        }
        assert!(source_terrace_datum("Route27", &edges, &sources).is_none());
    }

    #[test]
    fn waterfall_connected_water_keeps_its_authored_level_ownership() {
        let (mut sources, edges) = fixture("johto");
        let flat = source_terrace_datum("Route27", &edges, &sources).unwrap();
        assert_eq!(sample(Some(&flat), Vec2::new(12., 2.)), Some(0.));
        let (_, s, land) = &mut sources[255];
        s.tileset_id = "cave".into();
        s.metatile_id = 0x2c;
        s.tile_index = 0x40;
        *land = false;
        assert_eq!(shape_for_source(s), CellShape::Waterfall);
        assert!(
            source_terrace_datum("Route27", &edges, &sources).is_none(),
            "the zero-sea constraint must not replace a waterfall pool's own height"
        );
    }

    #[test]
    fn closed_water_preserves_the_kanto_pond_producer_and_johto_low_water_datum() {
        for atlas in ["johto", "kanto"] {
            let (mut sources, edges) = fixture(atlas);
            for (p, s, land) in &mut sources {
                if p.x == 0 || p.y == 0 || p.x == 15 || p.y == 15 {
                    s.tile_index = 6;
                    s.metatile_id = 1;
                    *land = false;
                }
            }
            let field = source_terrace_datum("Route27", &edges, &sources);
            if atlas == "kanto" {
                assert!(field.is_none());
            } else {
                assert_eq!(
                    sample(field.as_deref(), Vec2::new(12., 2.)),
                    Some(0.),
                    "a closed Johto bay still needs a low shoreline"
                );
            }
        }
    }
    #[test]
    fn source_domains_move_terrain_and_production_support_together_but_not_water() {
        let (sources, edges) = fixture("johto");
        let mut frame =
            super::super::tests::frame(16, 16, sources.iter().map(|(_, s, _)| s.clone()).collect());
        frame.map_id = "Route27".into();
        frame.jump_ledges = edges.clone().into();
        frame.terrace_datum = source_terrace_datum("Route27", &edges, &sources);
        let g = GridGeometry {
            width: 16,
            height: 16,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; 256],
            footing_heights: vec![0.; 256],
            ..Default::default()
        };
        for i in 0..256 {
            let (w, e, n, s) = g.bounds(i % 16, i / 16);
            if shape_for_source(&frame.tiles[i].source) == CellShape::Water {
                append_top(
                    &mut mesh.animated_textured,
                    [w, e, n, s],
                    -2.,
                    g.uv(i % 16, i / 16),
                );
            } else {
                append_top(&mut mesh.solid, [w, e, n, s], 0., (0., 1., 0., 1.));
            }
        }
        let water = mesh.animated_textured.clone();
        terraces::finish_with_datum(
            &mut mesh,
            "Route27",
            &frame.tiles.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            &edges,
            frame.terrace_datum.as_deref(),
        );
        assert_eq!(
            mesh.animated_textured, water,
            "positions, UVs, normals, colors and topology of water stay exact"
        );
        for i in 0..256 {
            if shape_for_source(&frame.tiles[i].source) == CellShape::Water {
                assert_eq!(mesh.footing_heights[i], 0.);
            }
        }
        for (x, y, want) in [(2, 2, 0.), (9, 2, 6.), (9, 8, 0.)] {
            let i = y * 16 + x;
            assert!((mesh.footing_heights[i] - want).abs() < 0.002);
            let foot = Vec2::new(
                frame.center.x - 64. + x as f32 * 8. + 4.,
                frame.center.y + 64. - y as f32 * 8. - 4.,
            );
            let actual =
                crate::footing::resolved_footing_height(&frame, foot, &mesh.footing_heights)
                    .unwrap();
            assert!(
                (actual - want).abs() < 0.002,
                "production actor support: {actual}/{want}"
            );
        }
    }
}
