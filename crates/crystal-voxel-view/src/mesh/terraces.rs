//! Cumulative one-way terraces. The authored jump direction always descends;
//! the upper approach is a shared ground level, never a repeated rising strip.
use super::*;
use bevy::prelude::{IVec2, Vec2};

pub(super) fn support_correction(
    frame: &VisualWorldFrame,
    cell: bevy::prelude::UVec2,
    foot: Vec2,
) -> f32 {
    if frame.jump_ledges.is_empty() {
        return 0.;
    }
    let g = GridGeometry {
        width: frame.grid_size.x as usize,
        height: frame.grid_size.y as usize,
        tile_width: frame.tile_size.x,
        tile_height: frame.tile_size.y,
        origin_x: 0.,
        origin_z: 0.,
    };
    let field = Field::new(&frame.jump_ledges, &g, [0, 0]);
    let start = frame.grid_origin.as_vec2() + cell.as_vec2();
    let grid_size = frame.grid_size.as_vec2() * frame.tile_size;
    let grid_point = Vec2::new(
        foot.x - (frame.center.x - grid_size.x * 0.5),
        frame.center.y + grid_size.y * 0.5 - foot.y,
    ) / frame.tile_size
        + frame.grid_origin.as_vec2();
    let t = (grid_point - start).clamp(Vec2::ZERO, Vec2::ONE);
    // The floor triangles interpolate the source-corner field. Sample on
    // the owning side of a lip, just as mesh generation does.
    let eps = 0.0001;
    let corners = [
        Vec2::new(eps, eps),
        Vec2::new(eps, 1. - eps),
        Vec2::new(1. - eps, 1. - eps),
        Vec2::new(1. - eps, eps),
    ]
    .map(|p| field.map_height(start + p));
    let support = if t.x <= t.y {
        corners[0] * (1. - t.y) + corners[1] * (t.y - t.x) + corners[2] * t.x
    } else {
        corners[0] * (1. - t.x) + corners[2] * t.y + corners[3] * (t.x - t.y)
    };
    support - field.map_height(start + Vec2::splat(0.5))
}
use crystal_render_api::VisualJumpLedge;

pub(super) struct Field<'a> {
    edges: &'a [VisualJumpLedge],
    g: &'a GridGeometry,
    origin: [i32; 2],
    levels: std::sync::Arc<super::terrace_levels::Levels>,
}
impl<'a> Field<'a> {
    pub(super) fn new(edges: &'a [VisualJumpLedge], g: &'a GridGeometry, origin: [i32; 2]) -> Self {
        let levels = super::terrace_levels::get(edges);
        Self {
            edges,
            g,
            origin,
            levels,
        }
    }

    /// A finite ledge raises its entire upstream terrace. Beyond its ends,
    /// a gentle lateral grade joins the bypass path to the lower ground.
    pub(super) fn height(&self, p: Vec2) -> f32 {
        self.map_height(self.map_point(p))
    }
    fn map_point(&self, p: Vec2) -> Vec2 {
        (p - Vec2::new(self.g.origin_x, self.g.origin_z))
            / Vec2::new(self.g.tile_width, self.g.tile_height)
            + Vec2::new(self.origin[0] as f32, self.origin[1] as f32)
    }
    fn map_height(&self, p: Vec2) -> f32 {
        let rise = crate::profile::JUMP_LEDGE_HEIGHT * self.g.tile_height / SOURCE_TILE_HEIGHT;
        self.levels.height(p) * rise
    }

    fn jump_cell(&self, index: usize, cells: &[&VisualTile]) -> bool {
        let s = &cells[index].source;
        if !matches!(
            s.tileset_id.as_ref(),
            "johto" | "johto_modern" | "kanto" | "forest"
        ) || !matches!(shape_for_source(s), CellShape::RaisedTop {height,solid:SolidKind::Bank}
                |CellShape::LedgeBand {height,..} if height==crate::profile::JUMP_LEDGE_HEIGHT)
        {
            return false;
        }
        let block = IVec2::new(
            self.origin[0] + (index % self.g.width) as i32 - s.subtile_column as i32,
            self.origin[1] + (index / self.g.width) as i32 - s.subtile_row as i32,
        );
        self.edges.iter().any(|edge| {
            if edge.descent.x == 0 {
                (edge.start.y == block.y || edge.start.y == block.y + 4)
                    && edge.start.x < block.x + 4
                    && edge.end.x > block.x
            } else {
                (edge.start.x == block.x || edge.start.x == block.x + 4)
                    && edge.start.y < block.y + 4
                    && edge.end.y > block.y
            }
        })
    }
    fn drop_face(&self, p: Vec3, normal: Vec3) -> bool {
        let p = self.map_point(Vec2::new(p.x, p.z));
        self.edges.iter().any(|edge| {
            let d = edge.descent.as_vec2();
            let start = edge.start.as_vec2();
            let (along, a, b) = if edge.descent.x == 0 {
                (p.x, start.x, edge.end.x as f32)
            } else {
                (p.y, start.y, edge.end.y as f32)
            };
            Vec2::new(normal.x, normal.z).dot(d) > 0.7
                && (p - start).dot(d).abs() < 0.25
                && along >= a - 0.05
                && along <= b + 0.05
        })
    }
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    edges: &[VisualJumpLedge],
) {
    let field = Field::new(edges, g, origin);
    let h = crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
    for (i, height) in mesh.footing_heights.iter_mut().enumerate() {
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        *height += field.height(Vec2::new((w + e) * 0.5, (n + s) * 0.5))
            - if field.jump_cell(i, cells) { h } else { 0. };
    }
    let rigid = (!mesh.rigid_buildings.is_empty()).then(||
        super::rigid_buildings::owners(mesh.solid.positions.len(), &mesh.rigid_buildings));
    for (domain, surface) in [
        &mut mesh.solid,
        &mut mesh.textured,
        &mut mesh.animated_solid,
        &mut mesh.animated_textured,
    ].into_iter().enumerate() {
        shape_surface_excluding(surface, cells, &field,
            if domain == 0 { rigid.as_deref() } else { None });
    }
    super::rigid_buildings::lift(&mut mesh.solid, &mesh.rigid_buildings, &field);
    append_drop_faces(&mut mesh.solid, cells, &field);
}

fn append_drop_faces(out: &mut SurfaceMeshData, cells: &[&VisualTile], field: &Field<'_>) {
    let g = field.g;
    let first = out.indices.len();
    for edge in field.edges {
        let vertical = edge.descent.x != 0;
        let plane = if vertical { edge.start.x } else { edge.start.y };
        let limit = if vertical {
            (field.origin[0], field.origin[0] + g.width as i32)
        } else {
            (field.origin[1], field.origin[1] + g.height as i32)
        };
        if plane < limit.0 || plane > limit.1 {
            continue;
        }
        let (a, b, low, high) = if vertical {
            (
                edge.start.y,
                edge.end.y,
                field.origin[1],
                field.origin[1] + g.height as i32,
            )
        } else {
            (
                edge.start.x,
                edge.end.x,
                field.origin[0],
                field.origin[0] + g.width as i32,
            )
        };
        let normal = Vec3::new(edge.descent.x as f32, 0., edge.descent.y as f32);
        let to_world = |v: i32| {
            let map = if vertical {
                Vec2::new(plane as f32, v as f32)
            } else {
                Vec2::new(v as f32, plane as f32)
            };
            (map - Vec2::new(field.origin[0] as f32, field.origin[1] as f32))
                * Vec2::new(g.tile_width, g.tile_height)
                + Vec2::new(g.origin_x, g.origin_z)
        };
        for v in a.max(low)..b.min(high) {
            let points = [to_world(v), to_world(v + 1)];
            let tangent = (points[1] - points[0]).normalize_or_zero();
            let levels = std::array::from_fn::<_, 2, _>(|endpoint| {
                // At a joined corner, sampling exactly at the endpoint can
                // select the landing beyond the perpendicular course. Both
                // lip limits must stay inside this unit's owning span.
                let p = points[endpoint] + tangent * if endpoint == 0 { 0.001 } else { -0.001 };
                let toward = Vec2::new(normal.x, normal.z) * 0.001;
                let lower = field.height(p + toward)
                    + cell_at(g, p.x + toward.x, p.y + toward.y)
                        .filter(|&i| matches!(shape_for_source(&cells[i].source), CellShape::Water))
                        .map_or(0., |_| CellShape::Water.surface_height(g.tile_height));
                (lower, field.height(p - toward))
            });
            if levels.iter().all(|&(l, h)| h <= l + 0.001) {
                continue;
            }
            let reverse = edge.descent.y > 0 || edge.descent.x < 0;
            let [a, b] = if reverse { [1, 0] } else { [0, 1] };
            let p = [
                [points[a].x, levels[a].0, points[a].y],
                [points[a].x, levels[a].1, points[a].y],
                [points[b].x, levels[b].1, points[b].y],
                [points[b].x, levels[b].0, points[b].y],
            ];
            let middle = (points[0] + points[1]) * 0.5 - Vec2::new(normal.x, normal.z) * 0.001;
            let timber = cell_at(g, middle.x, middle.y).is_some_and(|i| {
                let source = &cells[i].source;
                source.tileset_id.as_ref() == "tower"
                    && matches!(source.metatile_id, 0x2b | 0x2c | 0x3c..=0x3f)
                    && matches!(source.tile_index, 0x0b | 0x1a | 0x1b)
            });
            if timber {
                append_solid_quad(out, p, normal.to_array(), [0.38, 0.22, 0.085, 1.]);
            } else if !super::new_bark::ledge_finish::append(
                out,
                p,
                normal.to_array(),
                g,
                field.origin,
            ) {
                append_solid_quad(out, p, normal.to_array(), [0.29, 0.19, 0.08, 1.]);
            }
        }
    }
    // A taper can close one endpoint of a drop quad. Keep its visible
    // triangle, while removing the coincident triangle at the closed end.
    let added = out.indices[first..]
        .chunks_exact(3)
        .filter(|tri| {
            let [a, b, c] =
                [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(out.positions[i as usize]));
            (b - a).cross(c - a).length_squared() > 0.00000001
        })
        .flat_map(|tri| tri.iter().copied())
        .collect::<Vec<_>>();
    out.indices.truncate(first);
    out.indices.extend(added);
}

fn shape_surface(surface: &mut SurfaceMeshData, cells: &[&VisualTile], field: &Field<'_>) {
    shape_surface_excluding(surface, cells, field, None);
}

fn shape_surface_excluding(surface: &mut SurfaceMeshData, cells: &[&VisualTile], field: &Field<'_>, rigid: Option<&[bool]>) {
    let old = surface.positions.clone();
    let h = crate::profile::JUMP_LEDGE_HEIGHT * field.g.tile_height / SOURCE_TILE_HEIGHT;
    let mut changed = vec![false; old.len()];
    let mut obsolete = vec![false; surface.indices.len() / 3];
    let mut contact_tops = HashMap::<(IVec2, IVec2), Option<f32>>::new();
    let source_point = |uv: [f32; 2]| Vec2::new(
        (-uv[0] - 10000.) / field.g.tile_width,
        (-uv[1] - if uv[1] < -30000. { 40000. } else { 20000. }) / field.g.tile_height);
    for (number, tri) in surface.indices.chunks_exact(3).enumerate() {
        let ids = [tri[0], tri[1], tri[2]].map(|i| i as usize);
        if rigid.is_some_and(|owned| ids.iter().all(|&i| owned[i])) {
            continue;
        }
        let center = ids.iter().map(|&i| Vec3::from_array(old[i])).sum::<Vec3>() / 3.;
        let normal = Vec3::from_array(surface.normals[ids[0]]);
        let marked = ids
            .iter()
            .all(|&i| surface.uvs[i][0] == super::ledge_approach::BANK_OWNER_UV)
            .then(|| surface.uvs[ids[0]][1] as usize)
            .filter(|&i| i < cells.len());
        let inside = center - normal * 0.001;
        let owner = marked.or_else(|| cell_at(field.g, inside.x, inside.z));
        // Mineral faces carry their shader's world-coordinate tag. Their
        // fractured relief can protrude into a neighboring shallow-bank cell;
        // that cannot turn the tall cliff into the bank's six-pixel cap.
        let mineral = ids.iter().all(|&i| surface.uvs[i][0] < -9000.);
        let jump = !mineral && owner.is_some_and(|i| field.jump_cell(i, cells));
        let geometric_normal = (Vec3::from_array(old[ids[1]]) - Vec3::from_array(old[ids[0]]))
            .cross(Vec3::from_array(old[ids[2]]) - Vec3::from_array(old[ids[0]]))
            .normalize_or_zero();
        let mineral_wall = if mineral {
            // Sculpting/rounded contacts can move a facet into the adjoining
            // cell and tilt its normal. The mineral shader's original map
            // coordinates retain the source face independently of that relief.
            let source = ids.map(|i| {
                source_point(surface.uvs[i])
            });
            let extent = source.iter().copied().fold(
                (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                |(lo, hi), p| (lo.min(p), hi.max(p)),
            );
            let span = extent.1 - extent.0;
            // Relief gives the source wall a narrow, nonzero UV thickness.
            // Its dominant source axis still identifies the face when a
            // sculpted shoulder tilts almost horizontal; shading orientation
            // must not discard that original contact evidence.
            let mut point = source.iter().copied().sum::<Vec2>() / 3.;
            let axis = if span.y > span.x * 1.5 && geometric_normal.x.abs() > 0.0001 {
                point.x = point.x.round();
                Some(Vec2::new(geometric_normal.x.signum(), 0.))
            } else if span.x > span.y * 1.5 && geometric_normal.z.abs() > 0.0001 {
                point.y = point.y.round();
                Some(Vec2::new(0., geometric_normal.z.signum()))
            } else {
                None
            };
            let index = |p: Vec2| {
                let p = p.floor().as_ivec2() - IVec2::new(field.origin[0], field.origin[1]);
                (p.x >= 0 && p.y >= 0 && p.x < field.g.width as i32 && p.y < field.g.height as i32)
                    .then(|| p.y as usize * field.g.width + p.x as usize)
            };
            axis.and_then(|a| index(point - a * 0.05).zip(index(point + a * 0.05)))
                .and_then(|(rock, ground)| {
                    if !field.jump_cell(ground, cells) {
                        return None;
                    }
                    let top =
                        shape_for_source(&cells[rock].source).surface_height(field.g.tile_height);
                    (top > h + 0.05).then_some((h, top))
                })
        } else {
            None
        };
        if jump && normal.y.abs() < 0.9 && field.drop_face(center, normal) {
            obsolete[number] = true;
            continue;
        }
        if jump && normal.y > 0.9 && ids.iter().all(|&i| old[i][1].abs() < 0.001) {
            obsolete[number] = true;
            continue;
        }
        for &i in &ids {
            if changed[i] {
                continue;
            }
            changed[i] = true;
            let p = Vec3::from_array(old[i]);
            let position = Vec2::new(p.x, p.z);
            // Separate vertices at a drop's shared XZ edge remain on their
            // owning side. This is stable across tessellation and scrolling.
            let toward = (Vec2::new(inside.x, inside.z) - position).normalize_or_zero() * 0.001;
            let upper = field.height(position + toward);
            // Both faces of a mineral contact must remove the same legacy
            // bank rise. A zero-floor side may bury its toe, but cannot tear
            // away from the neighboring bank side above that ground contact.
            let contact = mineral.then(|| source_point(surface.uvs[i])).and_then(|p| {
                let key = mineral_contact_cells(p)?;
                let top = *contact_tops.entry(key).or_insert_with(|| {
                    mineral_contact_top(key, cells, field, h)
                });
                top.map(|top| (h, top))
            });
            let y = if let Some((old_floor, old_top)) = contact.or(mineral_wall) {
                // The cliff's cap keeps its authored height, while its foot
                // follows the bank whose old local rise has been removed.
                let t = ((p.y - old_floor) / (old_top - old_floor)).clamp(0., 1.);
                p.y + upper - old_floor * (1. - t)
            } else if jump && normal.y.abs() < 0.9 && p.y >= -0.01 && p.y <= h + 0.05 {
                let lower = field.height(position + Vec2::new(normal.x, normal.z) * 0.01);
                lower + (upper - lower) * (p.y / h).clamp(0., 1.)
            } else if jump && normal.y > 0.1 && p.y >= -0.01 && p.y <= h + 0.05 {
                upper
            } else {
                p.y + upper - if jump { h } else { 0. }
            };
            surface.positions[i][1] = y;
            if surface.uvs[i][0] == super::ledge_approach::BANK_OWNER_UV {
                surface.uvs[i] = [0., 0.];
            }
        }
    }
    let _contact_trace = crate::loading_trace::span("mineral.terrace_shared_points", || {
        let mut active=vec![false;old.len()];
        for &i in &surface.indices {active[i as usize]=true;}
        let mut groups=HashMap::<[i64;3],(f32,f32,usize,[f32;2])>::new();
        for (i,p) in old.iter().enumerate() {
            if !active[i] || surface.uvs[i][0]>=-9000. {continue;}
            let source=source_point(surface.uvs[i]);
            if (source-source.round()).abs().min_element()>0.001 {continue;}
            let y=surface.positions[i][1];
            let key=p.map(|v|(v*10000.).round() as i64);
            let entry=groups.entry(key).or_insert((y,y,0,source.to_array()));
            entry.0=entry.0.min(y);entry.1=entry.1.max(y);entry.2+=1;
        }
        let mut split=groups.into_iter().filter(|(_,v)|v.2>1 && v.1-v.0>0.01).collect::<Vec<_>>();
        split.sort_by_key(|(p,_)|*p);
        serde_json::json!({"map_origin":field.origin,"geometry_origin":[field.g.origin_x,field.g.origin_z],
            "split_count":split.len(),"first_points":split.iter().take(8)
            .map(|(p,v)|serde_json::json!({"before":p.map(|q|q as f64/10000.),
                "after_height":[v.0,v.1],"copies":v.2,"source":v.3})).collect::<Vec<_>>()})
    });
    let mut indices = Vec::with_capacity(surface.indices.len());
    let mut normals = vec![Vec3::ZERO; surface.positions.len()];
    let mut deformed = vec![false; surface.positions.len()];
    for (number, tri) in surface.indices.chunks_exact(3).enumerate() {
        if obsolete[number] {
            continue;
        }
        if rigid.is_some_and(|owned| tri.iter().all(|&i| owned[i as usize])) {
            indices.extend_from_slice(tri);
            continue;
        }
        let [a, b, c] =
            [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(surface.positions[i as usize]));
        let n = (b - a).cross(c - a);
        if n.length_squared() < 0.00000001 {
            continue;
        }
        let shift =
            [tri[0], tri[1], tri[2]].map(|i| surface.positions[i as usize][1] - old[i as usize][1]);
        if (shift[0] - shift[1]).abs() > 0.0001 || (shift[0] - shift[2]).abs() > 0.0001 {
            for &i in tri {
                deformed[i as usize] = true;
            }
        }
        indices.extend_from_slice(tri);
        for &i in tri {
            normals[i as usize] += n;
        }
    }
    surface.indices = indices;
    for (i, n) in normals.into_iter().enumerate() {
        if deformed[i] && n.length_squared() > 0.00000001 {
            surface.normals[i] = n.normalize().to_array();
        }
    }
}

fn mineral_contact_cells(point: Vec2) -> Option<(IVec2, IVec2)> {
    let rounded = point.round();
    let near = (point-rounded).abs();
    let boundary_x = near.x <= 0.05;
    let boundary_y = near.y <= 0.05;
    if !boundary_x && !boundary_y { return None; }
    let mut lo = point.floor().as_ivec2();
    let mut hi = lo;
    if boundary_x { hi.x=rounded.x as i32; lo.x=hi.x-1; }
    if boundary_y { hi.y=rounded.y as i32; lo.y=hi.y-1; }
    Some((lo,hi))
}

fn mineral_contact_top((lo,hi): (IVec2,IVec2), cells: &[&VisualTile], field: &Field<'_>, rise: f32) -> Option<f32> {
    let neighbors = [lo,IVec2::new(hi.x,lo.y),IVec2::new(lo.x,hi.y),hi]
        .map(|point| {
            let coord = point - IVec2::from_array(field.origin);
            if coord.x < 0 || coord.y < 0 || coord.x >= field.g.width as i32
                || coord.y >= field.g.height as i32 { return None; }
            let index = coord.y as usize * field.g.width + coord.x as usize;
            let shape = shape_for_source(&cells[index].source);
            let top = shape.surface_height(field.g.tile_height);
            Some((coord, (shape.solid_kind()==SolidKind::Bank && top>rise+0.05)
                .then_some(top), field.jump_cell(index,cells)))
        });
    let contact = neighbors.iter().flatten().any(|(rock,top,_)| top.is_some()
        && neighbors.iter().flatten().any(|(ground,_,jump)| *jump
            && (*rock-*ground).abs().element_sum()==1));
    if !contact { return None; }
    neighbors.iter().flatten().filter_map(|(_,top,_)|*top)
        .min_by(f32::total_cmp)
}

fn cell_at(g: &GridGeometry, x: f32, z: f32) -> Option<usize> {
    let x = ((x - g.origin_x) / g.tile_width).floor() as isize;
    let z = ((z - g.origin_z) / g.tile_height).floor() as isize;
    (x >= 0 && z >= 0 && x < g.width as isize && z < g.height as isize)
        .then_some(z.max(0) as usize * g.width + x.max(0) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_authored_jump_descends_and_reverse_side_is_lower() {
        let g = GridGeometry {
            width: 20,
            height: 20,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        for descent in [IVec2::Y, IVec2::NEG_Y, IVec2::X, IVec2::NEG_X] {
            let edge = VisualJumpLedge {
                start: IVec2::new(8, 8),
                end: if descent.x == 0 {
                    IVec2::new(12, 8)
                } else {
                    IVec2::new(8, 12)
                },
                descent,
            };
            let f = Field::new(std::slice::from_ref(&edge), &g, [0, 0]);
            let middle = (edge.start + edge.end).as_vec2() * 0.5;
            let high = f.map_height(middle - descent.as_vec2() * 0.5);
            let low = f.map_height(middle + descent.as_vec2() * 0.5);
            assert_eq!(
                high - low,
                6.,
                "the permitted direction must drop in {descent:?}"
            );
        }
    }
    #[test]
    fn opposing_route38_courses_keep_actual_eastward_jumps_descending() {
        let g = GridGeometry {
            width: 80,
            height: 36,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let edges = [
            ([20, 16], [20, 28], [-1, 0]),
            ([20, 28], [44, 28], [0, 1]),
            ([8, 32], [16, 32], [0, 1]),
            ([16, 20], [16, 32], [1, 0]),
            ([44, 24], [44, 28], [1, 0]),
        ]
        .map(|(start, end, descent)| VisualJumpLedge {
            start: IVec2::from_array(start),
            end: IVec2::from_array(end),
            descent: IVec2::from_array(descent),
        });
        let f = Field::new(&edges, &g, [0, 0]);
        for y in 10..=14 {
            let high = f.map_height(Vec2::new(13., (y * 2 + 2) as f32 - 0.0001));
            let low = f.map_height(Vec2::new(17., (y * 2 + 2) as f32 - 0.0001));
            assert!(
                high > low + 0.01,
                "actual eastward takeoff at (6,{y}) must be higher: {high} -> {low}"
            );
        }
    }

    #[test]
    fn silver_cave_local_takeoffs_survive_neighboring_remote_courses() {
        let g = GridGeometry {
            width: 80,
            height: 72,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let edges = [
            ([40, 40], [40, 48], [-1, 0]),
            ([52, 40], [52, 44], [-1, 0]),
            ([60, 56], [60, 64], [-1, 0]),
            ([52, 44], [58, 44], [0, 1]),
            ([60, 44], [64, 44], [0, 1]),
            ([40, 48], [46, 48], [0, 1]),
            ([48, 48], [56, 48], [0, 1]),
            ([8, 56], [10, 56], [0, 1]),
            ([12, 56], [24, 56], [0, 1]),
            ([48, 56], [50, 56], [0, 1]),
            ([52, 56], [56, 56], [0, 1]),
            ([64, 56], [68, 56], [0, 1]),
            ([60, 64], [72, 64], [0, 1]),
            ([24, 48], [24, 56], [1, 0]),
            ([72, 60], [72, 64], [1, 0]),
        ]
        .map(|(start, end, descent)| VisualJumpLedge {
            start: IVec2::from_array(start),
            end: IVec2::from_array(end),
            descent: IVec2::from_array(descent),
        });
        let f = Field::new(&edges, &g, [0, 0]);
        for x in [24, 26, 27] {
            let high = f.map_height(Vec2::new((x * 2 + 1) as f32, 54. - 0.0001));
            let low = f.map_height(Vec2::new((x * 2 + 1) as f32, 58. - 0.0001));
            assert!(
                high > low + 0.01,
                "actual southward takeoff at ({x},26) must be higher: {high} -> {low}"
            );
        }
    }

    #[test]
    fn landing_reservation_includes_the_sprite_foot_boundary() {
        let g = GridGeometry {
            width: 40,
            height: 120,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let edges = [
            ([16, 100], [16, 108], [-1, 0]),
            ([12, 72], [24, 72], [0, 1]),
            ([20, 96], [28, 96], [0, 1]),
        ]
        .map(|(start, end, descent)| VisualJumpLedge {
            start: IVec2::from_array(start),
            end: IVec2::from_array(end),
            descent: IVec2::from_array(descent),
        });
        let f = Field::new(&edges, &g, [0, 0]);
        for x in [21., 23., 25., 27.] {
            let high = f.map_height(Vec2::new(x, 96. - 0.0001));
            let low = f.map_height(Vec2::new(x, 100. - 0.0001));
            assert!(
                high > low + 0.01,
                "the landing boundary must be below its takeoff: {high} -> {low}"
            );
        }
    }

    #[test]
    fn short_eastward_course_keeps_the_foot_at_its_tangent_end_above_landing() {
        let g = GridGeometry {
            width: 40,
            height: 40,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let edges = [
            ([32, 4], [32, 8], [-1, 0]),
            ([26, 8], [28, 8], [0, 1]),
            ([32, 8], [36, 8], [0, 1]),
            ([28, 0], [28, 2], [1, 0]),
            ([32, 0], [32, 2], [1, 0]),
        ]
        .map(|(start, end, descent)| VisualJumpLedge {
            start: IVec2::from_array(start),
            end: IVec2::from_array(end),
            descent: IVec2::from_array(descent),
        });
        let f = Field::new(&edges, &g, [0, 0]);
        let high = f.map_height(Vec2::new(29., 2. - 0.0001));
        let low = f.map_height(Vec2::new(33., 2. - 0.0001));
        assert!(
            high > low + 0.01,
            "the actor foot at a short course end must descend: {high} -> {low}"
        );
    }

    #[test]
    fn a_jump_landing_that_is_another_takeoff_keeps_both_drops() {
        let g = GridGeometry {
            width: 40,
            height: 40,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let edges = [
            ([20, 28], [22, 28], [0, 1]),
            ([16, 20], [16, 26], [1, 0]),
            ([20, 24], [20, 26], [1, 0]),
        ]
        .map(|(start, end, descent)| VisualJumpLedge {
            start: IVec2::from_array(start),
            end: IVec2::from_array(end),
            descent: IVec2::from_array(descent),
        });
        let f = Field::new(&edges, &g, [0, 0]);
        let heights = [
            Vec2::new(13., 26.),
            Vec2::new(17., 26.),
            Vec2::new(21., 26.),
            Vec2::new(21., 30.),
        ]
        .map(|point| f.map_height(point - Vec2::Y * 0.0001));
        assert!(
            heights.windows(2).all(|h| h[0] > h[1] + 0.01),
            "each real jump in the right/right/down chain must descend: {heights:?}"
        );
    }

    #[test]
    fn tower_wooden_drop_has_no_outdoor_turf_and_keeps_its_floor_levels() {
        let g = GridGeometry {
            width: 4,
            height: 8,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let f = super::super::tests::frame(
            4,
            8,
            (0..8)
                .flat_map(|y| {
                    (0..4).map(move |x| {
                        let mut source = super::super::tests::source_with_tile(
                            if y < 4 { 0x3e } else { 3 },
                            x,
                            y % 4,
                            if y == 3 && x < 2 { 0x0b } else { 2 },
                        );
                        source.tileset_id = "tower".into();
                        source
                    })
                })
                .collect(),
        );
        let cells = f.tiles.iter().collect::<Vec<_>>();
        let edges = [VisualJumpLedge {
            start: IVec2::new(0, 4),
            end: IVec2::new(2, 4),
            descent: IVec2::Y,
        }];
        let field = Field::new(&edges, &g, [0, 0]);
        let mut surface = SurfaceMeshData::default();
        append_drop_faces(&mut surface, &cells, &field);
        assert!(!surface.indices.is_empty());
        assert!(
            surface.colors.iter().all(|c| c[0] > c[1] && c[1] > c[2]),
            "an indoor timber support cannot sprout a green grass rim"
        );
        assert!(surface.positions.iter().any(|p| (p[1] - 6.).abs() < 0.001));
        assert!(surface.positions.iter().any(|p| p[1].abs() < 0.001));
        assert!(surface.normals.iter().all(|n| n[2] > 0.99));
    }

    #[test]
    fn consecutive_south_ledges_form_three_descending_levels_without_an_uphill_toe() {
        let g = GridGeometry {
            width: 8,
            height: 16,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let edges = [4, 12].map(|y| VisualJumpLedge {
            start: IVec2::new(0, y),
            end: IVec2::new(8, y),
            descent: IVec2::Y,
        });
        let f = Field::new(&edges, &g, [0, 0]);
        let heights = (0..16)
            .map(|y| f.map_height(Vec2::new(4., y as f32 + 0.5)))
            .collect::<Vec<_>>();
        assert!(heights.windows(2).all(|p| p[0] >= p[1]));
        assert_eq!((heights[0], heights[7], heights[15]), (12., 6., 0.));
        let shifted = Field::new(&edges, &g, [-8, -10]);
        assert_eq!(
            f.height(Vec2::new(32., 56.)),
            shifted.height(Vec2::new(96., 136.)),
            "the same source point cannot change height when the viewport scrolls"
        );
    }

    #[test]
    fn native_mesh_and_actor_support_share_descending_ground_across_both_ledges() {
        let mut f = super::super::tests::frame(
            4,
            16,
            (0..16)
                .flat_map(|y| {
                    (0..4).map(move |x| {
                        let bank = y < 4 || (8..12).contains(&y);
                        let mut source = super::super::tests::source_with_tile(
                            if bank { 0x07 } else { 0x0a },
                            x,
                            y % 4,
                            if bank && y % 4 == 3 { 0x37 } else { 0x2c },
                        );
                        source.tileset_id = "kanto".into();
                        source
                    })
                })
                .collect(),
        );
        f.jump_ledges = vec![
            VisualJumpLedge {
                start: IVec2::new(0, 4),
                end: IVec2::new(4, 4),
                descent: IVec2::Y,
            },
            VisualJumpLedge {
                start: IVec2::new(0, 12),
                end: IVec2::new(4, 12),
                descent: IVec2::Y,
            },
        ]
        .into();
        let mesh = build_terrain_mesh(&f).unwrap();
        let heights = (0..16)
            .map(|y| mesh.footing_heights[y * 4 + 2])
            .collect::<Vec<_>>();
        assert_eq!(heights, [vec![12.; 4], vec![6.; 8], vec![0.; 4]].concat());
        assert!(
            heights.windows(2).all(|pair| pair[0] >= pair[1]),
            "walking south cannot rise onto the next bank"
        );
        for s in [&mesh.solid, &mesh.textured] {
            for triangle in s.indices.chunks_exact(3) {
                let [a, b, c] = [triangle[0], triangle[1], triangle[2]]
                    .map(|i| Vec3::from_array(s.positions[i as usize]));
                assert!(
                    (b - a).cross(c - a).length_squared() > 0.00000001,
                    "obsolete back walls must be removed, not collapsed triangles"
                );
            }
        }
    }

    #[test]
    fn forest_jump_caps_use_the_shared_level_without_an_extra_local_rise() {
        let mut frame = super::super::tests::frame(
            4,
            8,
            (0..8)
                .flat_map(|y| {
                    (0..4).map(move |x| {
                        let mut source = super::super::tests::source_with_tile(
                            if y < 4 { 0x1a } else { 1 },
                            x,
                            y % 4,
                            if y == 3 { 0x2a } else { 5 },
                        );
                        source.tileset_id = "forest".into();
                        source
                    })
                })
                .collect(),
        );
        frame.jump_ledges = vec![VisualJumpLedge {
            start: IVec2::new(0, 4),
            end: IVec2::new(4, 4),
            descent: IVec2::Y,
        }]
        .into();
        let mesh = build_terrain_mesh(&frame).unwrap();
        for row in 0..8 {
            assert_eq!(
                mesh.footing_heights[row * 4 + 2],
                if row < 4 { 6. } else { 0. },
                "the forest lip cannot add a second rise at source row {row}"
            );
        }
    }

    #[test]
    fn sculpted_mineral_cap_and_wall_keep_their_shared_mid_edge() {
        let g=GridGeometry {width:8,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let f=super::super::tests::frame(8,4,(0..4).flat_map(|y|(0..8).map(move|x| {
            let mut s=super::super::tests::source_with_tile(if y==0 {0x2c} else {0x58},
                x%4,y,if y==0 {0x11} else {0x2c});
            s.tileset_id="kanto".into(); s
        })).collect());
        let cells=f.tiles.iter().collect::<Vec<_>>();
        let edges=[VisualJumpLedge {start:IVec2::new(4,0),end:IVec2::new(4,4),descent:IVec2::NEG_X}];
        let field=Field::new(&edges,&g,[0,0]);
        let mut surface=SurfaceMeshData::default();
        super::super::outdoor_stone::append_canyon(&mut surface,
            [[40.,16.,0.],[40.,16.,8.],[48.,16.,8.],[48.,16.,0.]],[0.,1.,0.],&g,[0,0]);
        super::super::outdoor_stone::append_canyon(&mut surface,
            [[48.,6.,8.],[48.,16.,8.],[40.,16.,8.],[40.,6.,8.]],[0.,0.,1.],&g,[0,0]);
        let common=surface.positions.iter().enumerate().filter_map(|(i,p)|
            ((p[0]-44.).abs()<0.001 && (p[1]-16.).abs()<0.001 && (p[2]-8.).abs()<0.001)
                .then_some(i)).collect::<Vec<_>>();
        assert!(common.len()>=2,"fixture shares a cap/wall mid-edge");
        // The native mineral finish erodes crests before terrace removal.
        // Preserve the same authored UV coordinate on both sculpted faces.
        for p in &mut surface.positions {if (p[1]-16.).abs()<0.001 {p[1]-=0.4;}}
        shape_surface(&mut surface,&cells,&field);
        let heights=common.iter().map(|&i|surface.positions[i][1]).collect::<Vec<_>>();
        let low=heights.iter().copied().fold(f32::INFINITY,f32::min);
        let high=heights.iter().copied().fold(f32::NEG_INFINITY,f32::max);
        assert!(high-low<0.001,"sculpted cap/wall mid-edge tears: {heights:?}");
    }

    #[test]
    fn mineral_corner_shares_one_height_mapping_between_bank_and_plain_ground() {
        let g = GridGeometry { width:8,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0. };
        let f = super::super::tests::frame(8,4,(0..4).flat_map(|y|(0..8).map(move|x| {
            let (block,tile) = if y==0 && x<6 {(0x2c,0x11)}
                else if y==0 {(0,0x2c)} else {(0x58,0x2c)};
            let mut s=super::super::tests::source_with_tile(block,x%4,y,tile);
            s.tileset_id="kanto".into(); s
        })).collect());
        let cells=f.tiles.iter().collect::<Vec<_>>();
        assert!(matches!(shape_for_source(&cells[6].source),CellShape::Flat));
        let edges=[VisualJumpLedge{start:IVec2::new(4,0),end:IVec2::new(4,4),descent:IVec2::NEG_X}];
        let field=Field::new(&edges,&g,[0,0]);
        let mut surface=SurfaceMeshData::default();
        super::super::outdoor_stone::append_canyon(&mut surface,
            [[48.,6.,8.],[48.,16.,8.],[40.,16.,8.],[40.,6.,8.]], [0.,0.,1.],&g,[0,0]);
        super::super::outdoor_stone::append_canyon(&mut surface,
            [[48.,0.,8.],[48.,0.,0.],[48.,16.,0.],[48.,16.,8.]], [1.,0.,0.],&g,[0,0]);
        let common=surface.positions.iter().enumerate().filter_map(|(i,p)|
            ((p[0]-48.).abs()<0.0001 && (p[1]-8.).abs()<0.0001 && (p[2]-8.).abs()<0.0001)
                .then_some(i)).collect::<Vec<_>>();
        assert!(common.len()>=2,"fixture must share its original corner");
        shape_surface(&mut surface,&cells,&field);
        let heights=common.iter().map(|&i|surface.positions[i][1]).collect::<Vec<_>>();
        let low=heights.iter().copied().fold(f32::INFINITY,f32::min);
        let high=heights.iter().copied().fold(f32::NEG_INFINITY,f32::max);
        assert!(high-low<0.001,"shared mineral corner tears: {heights:?}");
    }

    #[test]
    fn neighboring_mineral_wall_keeps_its_height_above_a_jump_terrace() {
        let g = GridGeometry {
            width: 8,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = (0..4)
            .flat_map(|y| {
                (0..8).map(move |x| {
                    let mut source = super::super::tests::source_with_tile(
                        if y == 0 { 0x2c } else { 0x58 },
                        x % 4,
                        y,
                        if y == 0 { 0x11 } else { 0x2c },
                    );
                    source.tileset_id = "kanto".into();
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        source,
                        texture: Default::default(),
                        animation_frames: None,
                        priority: false,
                    }
                })
            })
            .collect::<Vec<_>>();
        let cells = tiles.iter().collect::<Vec<_>>();
        let edges = [VisualJumpLedge {
            start: IVec2::new(4, 0),
            end: IVec2::new(4, 4),
            descent: IVec2::NEG_X,
        }];
        let field = Field::new(&edges, &g, [0, 0]);
        let mut wall = SurfaceMeshData::default();
        super::super::outdoor_stone::append_canyon(
            &mut wall,
            [[48., 6., 8.], [48., 16., 8.], [40., 16., 8.], [40., 6., 8.]],
            [0., 0., 1.],
            &g,
            [0, 0],
        );
        assert!(
            wall.normals.iter().all(|n| n[2] > 0.9),
            "fixture faces the neighboring bank"
        );
        // Simulate the production stone sculpt/normal pass crossing the
        // source-cell border. The unchanged mineral coordinates still identify
        // the original wall, including its grass-floor contact.
        for shoulder_slope in [0., 4.] {
            let mut sculpted = wall.clone();
            for p in &mut sculpted.positions {
                p[2] += 2. + (p[1] - 6.) * shoulder_slope;
            }
            sculpted.normals.fill([0., 1., 0.]);
            let old = sculpted.positions.clone();
            shape_surface(&mut sculpted, &cells, &field);
            for (before, after) in old.iter().zip(&sculpted.positions) {
                // The tilted fixture extends beyond the course's tangent
                // end, where the shared terrace field grades down. Compare
                // against that actual field rather than a constant six rise.
                let expected = field.height(Vec2::new(before[0], before[2]))
                    - 6. * (1. - ((before[1] - 6.) / 10.).clamp(0., 1.));
                assert!(
                    (after[1] - before[1] - expected).abs() < 0.001,
                    "mineral join mismatch at slope {shoulder_slope}: old={before:?}, new={after:?}"
                );
            }
        }
    }

    #[test]
    fn joined_drop_faces_meet_the_upper_cap_at_the_corner() {
        let g = GridGeometry {
            width: 8,
            height: 8,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = (0..8)
            .flat_map(|y| {
                (0..8).map(move |x| {
                    let mut source =
                        super::super::tests::source_with_tile(0x0a, x % 4, y % 4, 0x2c);
                    source.tileset_id = "kanto".into();
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        source,
                        texture: Default::default(),
                        animation_frames: None,
                        priority: false,
                    }
                })
            })
            .collect::<Vec<_>>();
        let cells = tiles.iter().collect::<Vec<_>>();
        let edges = [
            VisualJumpLedge {
                start: IVec2::new(0, 4),
                end: IVec2::new(4, 4),
                descent: IVec2::Y,
            },
            VisualJumpLedge {
                start: IVec2::new(4, 0),
                end: IVec2::new(4, 4),
                descent: IVec2::X,
            },
        ];
        let field = Field::new(&edges, &g, [0, 0]);
        let mut surface = SurfaceMeshData::default();
        append_drop_faces(&mut surface, &cells, &field);
        for normal in [Vec3::X, Vec3::Z] {
            let top = surface
                .positions
                .iter()
                .zip(&surface.normals)
                .filter(|(p, n)| {
                    (p[0] - 32.).abs() < 0.0001
                        && (p[2] - 32.).abs() < 0.0001
                        && Vec3::from_array(**n).dot(normal) > 0.99
                })
                .map(|(p, _)| p[1])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(
                (top - 6.).abs() < 0.01,
                "joined face must meet its upper cap, got {top}"
            );
        }
    }

    #[test]
    fn a_corner_is_one_shared_upper_level_for_both_permitted_directions() {
        let g = GridGeometry {
            width: 16,
            height: 16,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let edges = [
            VisualJumpLedge {
                start: IVec2::new(0, 4),
                end: IVec2::new(4, 4),
                descent: IVec2::Y,
            },
            VisualJumpLedge {
                start: IVec2::new(4, 0),
                end: IVec2::new(4, 4),
                descent: IVec2::X,
            },
        ];
        let field = Field::new(&edges, &g, [0, 0]);
        assert!(
            (field.map_height(Vec2::new(2., 2.)) - 6.).abs() < 0.001,
            "a corner cannot double the upper level by adding two ramps"
        );
        assert!(field.map_height(Vec2::new(2., 6.)).abs() < 0.001);
        assert!(field.map_height(Vec2::new(6., 2.)).abs() < 0.001);
        assert_eq!(
            field.map_height(Vec2::new(1., -2.)),
            field.map_height(Vec2::new(1., 2.)),
            "the southward approach remains on the upper terrace"
        );
    }
}
