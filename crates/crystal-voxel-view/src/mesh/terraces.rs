//! Cumulative one-way terraces. The authored jump direction always descends;
//! the upper approach is a shared ground level, never a repeated rising strip.
use super::*;
use bevy::prelude::{IVec2, Vec2};

// Standing water uses one complete-body datum, never a varying land contour.
// Authored waterfalls/upper pools own their existing geometry separately.
fn level_water(source: &VisualTileSource) -> bool {
    crate::cave_water::standing_water_source(source)
}
pub(super) fn water_owns_height(source: &VisualTileSource) -> bool {
    level_water(source) || matches!(shape_for_source(source), CellShape::Waterfall)
        || crate::kanto_cliff::shoreline_round_barrier_local(source).is_some()
        || super::buoy_course::source(source)
}

// Preserve Johto/forest terrain even when a connected source halo contains Kanto.
fn kanto_family<'a>(tiles: impl Iterator<Item=&'a VisualTile>) -> bool {
    let mut kanto=false;
    for tile in tiles {match tile.source.tileset_id.as_ref() {
        "johto"|"johto_modern"|"forest"=>return false,
        "kanto"=>kanto=true,
        _=>{},
    }}
    kanto
}

pub(super) fn support_correction(
    frame: &VisualWorldFrame,
    cell: bevy::prelude::UVec2,
    foot: Vec2,
) -> f32 {
    if frame.jump_ledges.is_empty() || !super::jump_changes_floor_level(&frame.map_id) {
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
    if frame.tiles.get(cell.y as usize*g.width+cell.x as usize)
        .is_some_and(|t|water_owns_height(&t.source)
            ||super::elevated_piers::level_surface(&frame.map_id,&t.source)) {return 0.;}
    let mut field = Field::new(&frame.jump_ledges, &g, [0, 0]).with_datum(frame.terrace_datum.as_deref());
    if kanto_family(frame.tiles.iter()) {
        field.levels=super::terrace_levels::get_for_family(&frame.jump_ledges,true);
    }
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
#[path = "town_ledge_lip.rs"]
mod town_ledge_lip;

pub(super) struct Field<'a> {
    datum: Option<&'a crystal_render_api::VisualTerraceDatum>,
    edges: &'a [VisualJumpLedge],
    g: &'a GridGeometry,
    origin: [i32; 2],
    levels: std::sync::Arc<super::terrace_levels::Levels>,
}
impl<'a> Field<'a> {
    pub(super) fn new(edges: &'a [VisualJumpLedge], g: &'a GridGeometry, origin: [i32; 2]) -> Self {
        let levels = super::terrace_levels::get(edges);
        Self {
            datum: None,
            edges,
            g,
            origin,
            levels,
        }
    }

    pub(super) fn with_kanto_levels(mut self) -> Self {
        self.levels=super::terrace_levels::get_for_family(self.edges,true);
        self
    }

    pub(super) fn with_source(edges:&'a [VisualJumpLedge],g:&'a GridGeometry,
        origin:[i32;2],cells:&[&VisualTile])->Self {
        let mut field=Self::new(edges,g,origin);
        if kanto_family(cells.iter().copied()) {
            field.levels=super::terrace_levels::get_for_family(edges,true);
        }
        field
    }
    pub(super) fn with_datum(mut self, datum:Option<&'a crystal_render_api::VisualTerraceDatum>)->Self {
        self.datum=datum;self
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
        super::terrace_datum::sample(self.datum,p).unwrap_or_else(||self.levels.height(p)) * rise
    }

    fn water_owned(&self,i:usize,cells:&[&VisualTile])->bool {
        water_owns_height(&cells[i].source) || crate::kanto_cliff::water_backed_barrier_local(
            &cells[i].source,IVec2::new(self.origin[0]+(i%self.g.width)as i32,
                self.origin[1]+(i/self.g.width)as i32),self.datum).is_some()
    }
    pub(super) fn water_surface_offset(&self,p:Vec2)->f32 {
        let point=self.map_point(p);
        let Some(datum)=self.datum else {return 0.;};
        let level=if let Some(level)=datum.open_water_at(point.floor().as_ivec2()) {level}
        else {
            let local=point-datum.origin.as_vec2();
            // Native deck corners include the exact outer boundary, but a
            // repeated offshore deck cannot borrow a raised land-edge sample.
            if local.x<0. || local.y<0. || local.x>datum.size.x as f32 || local.y>datum.size.y as f32 {return 0.;}
            super::terrace_datum::sample(Some(datum),point).unwrap_or(0.)
        };
        level*crate::profile::JUMP_LEDGE_HEIGHT*self.g.tile_height/SOURCE_TILE_HEIGHT
    }
    pub(super) fn water_offset(&self, i: usize, cells: &[&VisualTile]) -> f32 {
        if self.datum.is_none() || !self.water_owned(i,cells)
            || matches!(shape_for_source(&cells[i].source),CellShape::Waterfall)
            || !matches!(cells[i].source.tileset_id.as_ref(), "johto" | "johto_modern" | "kanto") {
            return 0.;
        }
        let (w,e,n,s) = self.g.bounds(i % self.g.width, i / self.g.width);
        let point = self.map_point(Vec2::new((w+e)*0.5, (n+s)*0.5));
        let datum = self.datum.unwrap();
        if let Some(level)=datum.open_water_at(point.floor().as_ivec2()) {
            return level*crate::profile::JUMP_LEDGE_HEIGHT*self.g.tile_height/SOURCE_TILE_HEIGHT;
        }
        let local = point - datum.origin.as_vec2();
        // The camera halo can repeat water beyond the complete native maps.
        // Land sampling clamps to the nearest authored edge, but that edge
        // cannot authorize a raised ocean outside its source evidence.
        if local.x < 0. || local.y < 0.
            || local.x >= datum.size.x as f32 || local.y >= datum.size.y as f32 {
            return 0.;
        }
        super::terrace_datum::sample(Some(datum), point).unwrap_or(0.)
            * crate::profile::JUMP_LEDGE_HEIGHT * self.g.tile_height / SOURCE_TILE_HEIGHT
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
    finish_on_map(mesh,"",cells,g,origin,edges);
}

pub(super) fn finish_on_map(mesh: &mut TerrainMeshData, map: &str,
    cells: &[&VisualTile], g: &GridGeometry, origin: [i32;2], edges: &[VisualJumpLedge]) {
    finish_with_datum(mesh,map,cells,g,origin,edges,None);
}
pub(super) fn finish_with_datum(mesh:&mut TerrainMeshData,map:&str,cells:&[&VisualTile],
    g:&GridGeometry,origin:[i32;2],edges:&[VisualJumpLedge],datum:Option<&crystal_render_api::VisualTerraceDatum>) {
    // Tin Tower's one-way hops cross timber barriers on a level indoor floor.
    // They do not describe cumulative outdoor hills or changes of floor level.
    if !super::jump_changes_floor_level(map) {
        // Flat-map policy also ignores any imported neighbor elevation, but
        // a recessed modeled rock still needs its dry-side closure.
        close_flat_rock_contacts(mesh,map,cells,g,origin,None);
        return;
    }
    let mut field = Field::new(edges, g, origin).with_datum(datum);
    if kanto_family(cells.iter().copied()) {
        field.levels=super::terrace_levels::get_for_family(edges,true);
    }
    let h = crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
    let water_continuation=mesh.authored_cells.iter().map(|owner|
        *owner==Some("outdoor/border-water")).collect::<Vec<_>>();
    for (i, height) in mesh.footing_heights.iter_mut().enumerate() {
        if level_water(&cells[i].source) {
            *height += field.water_offset(i,cells);
            continue;
        }
        if water_continuation.get(i).copied().unwrap_or(false) || field.water_owned(i,cells)
            ||super::elevated_piers::level_surface(map,&cells[i].source) {continue;}
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
        shape_surface_excluding_with_lips_on_map(surface, cells, &field,
            if domain == 0 { rigid.as_deref() } else { None },map=="BlackthornCity",map);
    }
    super::rigid_buildings::lift(&mut mesh.solid, &mesh.rigid_buildings, &field);
    append_drop_faces_with_lips_on_map(&mut mesh.solid, cells, &field,map=="BlackthornCity",map);
    if crate::cave_water::outdoor(map)||super::elevated_piers::has_piers(map) {
        let recessed_rocks = recessed_rock_cells(mesh);
        append_low_water_shore_with_continuation(&mut mesh.solid,map,cells,&field,&water_continuation,Some(&recessed_rocks));
    }
}

fn recessed_rock_cells(mesh: &TerrainMeshData) -> Vec<bool> {
    mesh.authored_cells.iter().map(|owner|
        *owner==Some("world-exterior/kanto_boundary_shore")).collect()
}

/// Replacing a flat rock sprite with a recessed sea bed exposes the side of
/// adjacent level ground. Close that hidden side even on maps with no ledges.
/// Existing bank tops, rock models, water planes and actor support stay intact.
pub(super) fn close_flat_rock_contacts(mesh:&mut TerrainMeshData,map:&str,cells:&[&VisualTile],
    g:&GridGeometry,origin:[i32;2],datum:Option<&crystal_render_api::VisualTerraceDatum>) {
    if !crate::cave_water::outdoor(map) {return;}
    let recessed_rocks=recessed_rock_cells(mesh);
    if !recessed_rocks.iter().any(|&rock|rock) {return;}
    let field=Field::with_source(&[],g,origin,cells).with_datum(datum);
    append_low_water_shore_with_continuation(&mut mesh.solid,map,cells,&field,&[],Some(&recessed_rocks));
}

/// Add only the missing mineral below an already-fitted native cave bank.
/// Walking support and the source's level pools/cascades stay untouched.
pub(super) fn close_cave_water_banks(mesh:&mut TerrainMeshData,map:&str,cells:&[&VisualTile],
    g:&GridGeometry,edges:&[VisualJumpLedge],origin:[i32;2],datum:Option<&crystal_render_api::VisualTerraceDatum>) {
    if crate::cave_water::outdoor(map) || !cells.first().is_some_and(|t|
        matches!(t.source.tileset_id.as_ref(),"cave"|"dark_cave")) {return;}
    let field=Field::with_source(edges,g,origin,cells).with_datum(datum);
    append_low_water_shore_with_continuation(&mut mesh.solid,map,cells,&field,&[],None);
}

fn cave_wet_underlay(cells:&[&VisualTile],i:usize,map:&str)->bool {
    matches!(cells[i].source.tileset_id.as_ref(),"cave"|"dark_cave")
        && ((crate::cave_water::role_for_map(cells[i],map)[0]>0)
            || super::cave_shore::water_underlay_source(cells,i).is_some())
}

// Preserve the complete native upper bank. A backing below it seals the
// distance to level water without stretching or tearing its sculpted faces.
#[cfg(test)]
fn append_low_water_shore(out:&mut SurfaceMeshData,map:&str,cells:&[&VisualTile],field:&Field<'_>) {
    append_low_water_shore_with_continuation(out,map,cells,field,&[],None);
}
fn append_low_water_shore_with_continuation(out:&mut SurfaceMeshData,map:&str,cells:&[&VisualTile],
    field:&Field<'_>,water_continuation:&[bool],recessed_rocks:Option<&[bool]>) {
    let g=field.g;
    for (i,tile) in cells.iter().enumerate() {
        let cave=cave_wet_underlay(cells,i,map);
        let water=if cave {0.} else {crate::profile::WATER_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT
            +field.water_offset(i,cells)};
        // Modeled sea rocks and floating courses retain a level water
        // underlay too. Its contact with raised land needs the same backing,
        // while two neighboring water owners must never acquire a wall.
        let water_backed_barrier=crate::kanto_cliff::water_backed_barrier_local(&tile.source,
            IVec2::new(field.origin[0]+(i%g.width)as i32,field.origin[1]+(i/g.width)as i32),field.datum).is_some()
            ||super::buoy_course::source(&tile.source);
        if !water_backed_barrier && !cave
            && (tile.source.tile_index!=0x14||!matches!(shape_for_source(&tile.source),CellShape::Water)) {continue;}
        let x=(i%g.width)as isize;let z=(i/g.width)as isize;
        let (w,e,n,s)=g.bounds(i%g.width,i/g.width);
        for (dx,dz,ends) in [(-1,0,[[w,n],[w,s]]),(1,0,[[e,s],[e,n]]),
            (0,-1,[[e,n],[w,n]]),(0,1,[[w,s],[e,s]])] {
            let nx=x+dx;let nz=z+dz;
            if nx<0||nz<0||nx>=g.width as isize||nz>=g.height as isize {continue;}
            let neighbor=nz as usize*g.width+nx as usize;
            let source=&cells[neighbor].source;
            if water_continuation.get(neighbor).copied().unwrap_or(false)
                || super::elevated_piers::level_surface(map,source)
                ||field.water_owned(neighbor,cells) || (cave && cave_wet_underlay(cells,neighbor,map)) {continue;}
            let normal=Vec3::new(-dx as f32,0.,-dz as f32);
            let inset=Vec2::new(dx as f32,dz as f32)*g.tile_width.min(g.tile_height)*0.0002;
            let [a,b]=ends.map(|p|Vec2::from_array(p)+inset);
            // A sealed mineral pond meets the plateau's native base as well
            // as its terrace offset. Ignoring that base leaves no front bank
            // after the water rises, exposing individual eroded cap fingers.
            let native = if field.water_offset(i,cells)>0.
                && super::terrace_datum::mountain_ponds::plateau(source) {
                shape_for_source(source).surface_height(g.tile_height)
            } else {0.};
            // Endpoints can lie exactly on the next cell's bridge or water
            // datum. The cap was fitted from inside this owning bank cell;
            // its backing must use the same side at both terminal corners.
            let (nw,ne,nn,ns)=g.bounds(neighbor%g.width,neighbor/g.width);
            let eps=g.tile_width.min(g.tile_height)*0.0001;
            let heights=[a,b].map(|p|field.height(p.clamp(
                Vec2::new(nw+eps,nn+eps),Vec2::new(ne-eps,ns-eps)))+native);
            // Native shoreline caps are dry bank geometry too, even though
            // their glyph is not an ordinary lawn/path material donor.
            let flat_rock_contact = water_backed_barrier
                && recessed_rocks.is_some_and(|mask|mask.get(i)==Some(&true))
                && matches!(shape_for_source(source),CellShape::Flat|CellShape::PlaneAt {height:0.})
                && (super::kanto_shore::source_bank(source) || matches!(super::new_bark::ground_material(source),Some(
                    super::new_bark::GroundMaterial::Lawn|super::new_bark::GroundMaterial::Path
                    |super::new_bark::GroundMaterial::Pavers)));
            if heights.iter().all(|h|*h<=0.001) && !flat_rock_contact {continue;}
            let top=heights.map(|h|h.max(water));
            let p=[[a.x,water,a.y],[a.x,top[0],a.y],[b.x,top[1],b.y],[b.x,water,b.y]];
            // This is the side of the existing cliff, never a second
            // shoreline crown. Forest presentation remains outside this pass.
            let first=out.positions.len();
            if cave {
                let mut uvs=p.map(|v|super::cave_cliff::rock_uv(Vec3::from_array(v),g,field.origin,false));
                // Private pending-bank role; the source-pigment finish below
                // converts only these new faces, never an existing native cap.
                for uv in &mut uvs {uv[1]+=3_400_000.;}
                let colors=p.map(|v|super::cave_cliff::pigment([0.295,0.355,0.38,1.],
                    Vec3::from_array(v),g,field.origin,false));
                append_quad_colors(out,p,normal.to_array(),uvs,colors);
                continue;
            }
            let keep_existing_finish=source.tileset_id.as_ref()=="forest"
                || tile.source.tileset_id.as_ref()=="forest";
            let backed=if keep_existing_finish {
                super::new_bark::ledge_finish::append(out,p,normal.to_array(),g,field.origin)
            } else if matches!(super::new_bark::ground_material(source),
                Some(super::new_bark::GroundMaterial::Lawn)) {
                super::new_bark::ledge_finish::append_support_face(out,p,normal.to_array(),g,field.origin)
            } else {
                super::new_bark::ledge_finish::append_bare_support_face(out,p,normal.to_array(),g,field.origin)
            };
            if !backed {
                append_solid_quad(out,p,normal.to_array(),[0.29,0.19,0.08,1.]);
            }
            if keep_existing_finish {out.uvs[first..].fill([0.;2]);}
        }
    }
}

// Refreshes replace the retained blooms. Fit outdoor flowers to the same
// existing field as their static lawn; forest/interior vertices keep their
// legacy path even when they occur in a connected town halo.
pub(super) fn fit_animated_outdoor(surface: &mut SurfaceMeshData, map:&str,cells: &[&VisualTile],
    g: &GridGeometry, origin: [i32; 2], edges: &[VisualJumpLedge],datum:Option<&crystal_render_api::VisualTerraceDatum>) {
    let kanto=kanto_family(cells.iter().copied());
    if !kanto && !super::new_bark::ordinary_lawn_map(map) {return;}
    let mut field = Field::new(edges, g, origin).with_datum(datum);
    if kanto {field.levels=super::terrace_levels::get_for_family(edges,true);}
    let preserved=(!kanto).then(||surface.positions.iter().map(|p|
        cell_at(g,p[0],p[2]).is_none_or(|i|!super::source_bound_outdoor_ground(&cells[i].source)))
        .collect::<Vec<_>>());
    shape_surface_excluding_with_lips_on_map(surface, cells, &field, preserved.as_deref(), false,map);
}

fn append_drop_faces(out: &mut SurfaceMeshData, cells: &[&VisualTile], field: &Field<'_>) {
    append_drop_faces_with_lips(out,cells,field,false);
}

fn append_drop_faces_with_lips(out: &mut SurfaceMeshData, cells: &[&VisualTile],
    field: &Field<'_>, closed_lips: bool) {
    append_drop_faces_with_lips_on_map(out,cells,field,closed_lips,"");
}
fn append_drop_faces_with_lips_on_map(out: &mut SurfaceMeshData, cells: &[&VisualTile],
    field: &Field<'_>, closed_lips: bool,map:&str) {
    let g = field.g;
    let deck_floors = (super::elevated_piers::has_piers(map)
        && cells.iter().any(|t|super::elevated_piers::walkway(map,&t.source)))
        .then(|| super::elevated_piers::bank_contact_floors(map, cells, g, field));
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
            let middle = (points[0] + points[1]) * 0.5;
            let across = Vec2::new(normal.x, normal.z) * 0.001;
            if [middle - across, middle + across].into_iter().all(|p|
                cell_at(g,p.x,p.y).is_some_and(|i|field.water_owned(i,cells))) {
                // A propagated land contour cannot become a wall or step
                // through a connected pool. Its bank belongs to land only.
                continue;
            }
            let levels = std::array::from_fn::<_, 2, _>(|endpoint| {
                // At a joined corner, sampling exactly at the endpoint can
                // select the landing beyond the perpendicular course. Both
                // lip limits must stay inside this unit's owning span.
                let p = points[endpoint] + tangent * if endpoint == 0 { 0.001 } else { -0.001 };
                let toward = Vec2::new(normal.x, normal.z) * 0.001;
                let lower_cell=cell_at(g,p.x+toward.x,p.y+toward.y);
                let lower=if lower_cell.is_some_and(|i|level_water(&cells[i].source)) {
                    crate::profile::WATER_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT
                        +field.water_offset(lower_cell.unwrap(),cells)
                } else if lower_cell.is_some_and(|i|super::elevated_piers::level_surface(map,&cells[i].source)) {
                    if lower_cell.is_some_and(|i|!super::elevated_piers::walkway(map,&cells[i].source)) {
                        crate::profile::WATER_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT
                    } else if let Some(corners) = deck_floors.as_ref()
                        .and_then(|v| v[lower_cell.unwrap()]) {
                        let i=lower_cell.unwrap();
                        let (w,e,n,s)=g.bounds(i%g.width,i/g.width);
                        let t=((p-Vec2::new(w,n))/Vec2::new(e-w,s-n)).clamp(Vec2::ZERO,Vec2::ONE);
                        // Interpolate the same boundary edge as the fitted
                        // deck, seating the bank below its recessed mortar.
                        (corners[0]*(1.-t.y)+corners[1]*t.y)*(1.-t.x)
                            +(corners[3]*(1.-t.y)+corners[2]*t.y)*t.x
                    } else {crate::port::PIER_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT}
                } else {field.height(p+toward)+lower_cell
                    .filter(|&i|matches!(shape_for_source(&cells[i].source),CellShape::Water))
                    .map_or(0., |_|CellShape::Water.surface_height(g.tile_height))};
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
            } else if cell_at(g,middle.x,middle.y).is_some_and(|i|
                cells[i].source.tileset_id.as_ref()=="kanto" && field.jump_cell(i,cells)) {
                town_ledge_lip::append_kanto(out,p,normal.to_array(),g,field.origin);
            } else if closed_lips {
                town_ledge_lip::append(out,p,normal.to_array(),g,field.origin);
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
    shape_surface_excluding_with_lips(surface,cells,field,rigid,false);
}

fn shape_surface_excluding_with_lips(surface: &mut SurfaceMeshData, cells: &[&VisualTile],
    field: &Field<'_>, rigid: Option<&[bool]>, closed_lips: bool) {
    shape_surface_excluding_with_lips_on_map(surface,cells,field,rigid,closed_lips,"");
}
/// A water sheet borrowed by a wet rock corner still belongs to the live
/// pool sampled by its atlas UVs, not the dry terrain under that corner.
fn cave_water_uv_source(surface:&SurfaceMeshData,ids:[usize;3],cells:&[&VisualTile],g:&GridGeometry,map:&str)->Option<usize> {
    let mut source=None;
    for i in ids {
        let uv=surface.uvs[i];
        if !uv.iter().all(|v|v.is_finite() && *v>=0. && *v<1.) {return None;}
        let index=(uv[1]*g.height as f32).floor() as usize*g.width
            +(uv[0]*g.width as f32).floor() as usize;
        let tile=cells.get(index)?;
        if !matches!(tile.source.tileset_id.as_ref(),"cave"|"dark_cave")
            || crate::cave_water::role_for_map(tile,map)[0]==0
            || source.is_some_and(|old|old!=index) {return None;}
        source=Some(index);
    }
    source
}

fn shape_surface_excluding_with_lips_on_map(surface: &mut SurfaceMeshData, cells: &[&VisualTile],
    field: &Field<'_>, rigid: Option<&[bool]>, closed_lips: bool,map:&str) {
    let old = surface.positions.clone();
    let h = crate::profile::JUMP_LEDGE_HEIGHT * field.g.tile_height / SOURCE_TILE_HEIGHT;
    let mut changed = vec![false; old.len()];
    let mut obsolete = vec![false; surface.indices.len() / 3];
    let mut contact_tops = HashMap::<(IVec2, IVec2), Option<f32>>::new();
    // A source-authenticated bank of a resolved mineral pond has one floor
    // offset. Its sculpted faces can protrude over the adjacent plateau;
    // sampling that lower offset per vertex stretches them into needles.
    let mut resolved_banks=Vec::new();
    if field.datum.is_some() && surface.uvs.iter().any(|uv|
        uv[0]>87000. && uv[0]<90000. && uv[1]< -65000.) {
        for (i,tile) in cells.iter().enumerate() {
            if !super::kanto_shore::source_bank(&tile.source) {continue;}
            let (x,z)=(i%field.g.width,i/field.g.width);
            // Raised inland rivers also have positive water offsets. Only
            // a native mineral rim needs the pond's rigid fitting; ordinary
            // banks must keep the land's corner heights to close their joins.
            let mineral_rim=(z.saturating_sub(1)..=(z+1).min(field.g.height-1)).any(|r|
                (x.saturating_sub(1)..=(x+1).min(field.g.width-1)).any(|c|
                    super::terrace_datum::mountain_ponds::plateau(&cells[r*field.g.width+c].source)));
            let resolved=mineral_rim && (z.saturating_sub(1)..=(z+1).min(field.g.height-1)).any(|r|
                (x.saturating_sub(1)..=(x+1).min(field.g.width-1)).any(|c| {
                    let j=r*field.g.width+c;
                    cells[j].source.tileset_id.as_ref()=="kanto" && field.water_offset(j,cells)>0.
                }));
            if resolved {
                let (w,e,n,s)=field.g.bounds(x,z);
                let center=Vec2::new((w+e)*0.5,(n+s)*0.5);
                resolved_banks.push((center,field.height(center)));
            }
        }
    }
    let source_point = |uv: [f32; 2]| Vec2::new(
        (-uv[0] - 10000.) / field.g.tile_width,
        (-uv[1] - if uv[1] < -50000. {60000.} else if uv[1] < -30000. { 40000. } else { 20000. }) / field.g.tile_height);
    for (number, tri) in surface.indices.chunks_exact(3).enumerate() {
        let ids = [tri[0], tri[1], tri[2]].map(|i| i as usize);
        // Native brick bevels may be clipped exactly onto a land boundary.
        // Their producer has fitted them to the pier's own corner heights;
        // geometric inward nudges must not assign them to the adjoining hill.
        if ids.iter().all(|&i|surface.uvs[i]==crate::scenery_finish::LEVEL_PAVING_UV) {
            continue;
        }
        if rigid.is_some_and(|owned| ids.iter().all(|&i| owned[i])) {
            continue;
        }
        let center = ids.iter().map(|&i| Vec3::from_array(old[i])).sum::<Vec3>() / 3.;
        let normal = Vec3::from_array(surface.normals[ids[0]]);
        if normal.y>0.99 && ids.iter().all(|&i|(old[i][1]-old[ids[0]][1]).abs()<0.0001)
            && cave_water_uv_source(surface,ids,cells,field.g,map).is_some() {
            continue;
        }
        if !resolved_banks.is_empty() && ids.iter().all(|&i| {
            let uv=surface.uvs[i];uv[0]>87000. && uv[0]<90000. && uv[1]< -65000.
        }) {
            let point=Vec2::new(center.x,center.z);
            let nearest=resolved_banks.iter().filter_map(|&(bank,offset)| {
                let d=((point-bank)/Vec2::new(field.g.tile_width,field.g.tile_height)).length_squared();
                (d<2.25).then_some((d,offset))
            }).min_by(|a,b|a.0.total_cmp(&b.0));
            if let Some((_,offset))=nearest {
                for &i in &ids {if !changed[i] {surface.positions[i][1]=old[i][1]+offset;changed[i]=true;}}
                continue;
            }
        }
        let marked = ids
            .iter()
            .all(|&i| surface.uvs[i][0] == super::ledge_approach::BANK_OWNER_UV)
            .then(|| surface.uvs[ids[0]][1] as usize)
            .filter(|&i| i < cells.len());
        let inside = center - normal * 0.001;
        let owner = marked.or_else(|| cell_at(field.g, inside.x, inside.z));
        if owner.is_some_and(|i|field.water_owned(i,cells))
            && !ids.iter().all(|&i|surface.uvs[i][0]< -9000.) {
            // A complete enclosed lake receives one constant offset, including
            // ripple accents. Open seas and authored waterfall pools stay put.
            let offset=field.water_offset(owner.unwrap(),cells);
            if offset != 0. { for &i in &ids {
                if !changed[i] { surface.positions[i][1]=old[i][1]+offset; changed[i]=true; }
            } }
            continue;
        }
        // Piers, planted islands and their low water form a level structure.
        // Hills elsewhere on a route must not bend its decks or flower roots.
        let pier=owner.is_some_and(|i|super::elevated_piers::walkway(map,&cells[i].source)
            ||super::elevated_piers::structural_edge(map,&cells[i].source));
        let water=crate::profile::WATER_HEIGHT*field.g.tile_height/SOURCE_TILE_HEIGHT;
        let flat_water=normal.y>0.99
            &&!ids.iter().all(|&i|surface.uvs[i][0]< -9000.)
            &&(ids.iter().all(|&i|(old[i][1]-water).abs()<0.0001)
                || (owner.is_some_and(|i|level_water(&cells[i].source))
                    &&ids.iter().all(|&i|(old[i][1]-old[ids[0]][1]).abs()<0.0001)));
        // The suspended bridge keeps the same inland river beneath its
        // deck. Its source cells belong to paving, so the water underlay
        // needs explicit height ownership rather than the land contour.
        if flat_water && pier {
            let offset=field.water_surface_offset(Vec2::new(center.x,center.z));
            for &i in &ids {if !changed[i] {
                surface.positions[i][1]=old[i][1]+offset;changed[i]=true;
            }}
            continue;
        }
        let rooted_water=ids.iter().all(|&i|surface.uvs[i][0]==super::shore_ecology::ROOTED_WATER_UV);
        // Proven deck structures were fitted by PierLevels already. A
        // raised river can place them above the old fixed-height cutoff;
        // applying land height again lifts their beams away from the paving.
        if rooted_water || flat_water || (pier && !ids.iter().all(|&i|surface.uvs[i][0]< -9000.)) {
            continue;
        }
        // Mineral faces carry their shader's world-coordinate tag. Their
        // fractured relief can protrude into a neighboring shallow-bank cell;
        // that cannot turn the tall cliff into the bank's six-pixel cap.
        let mineral = ids.iter().all(|&i| surface.uvs[i][0] < -9000.);
        let jump = !mineral && owner.is_some_and(|i| field.jump_cell(i, cells));
        let kanto_ground=owner.is_some_and(|i|cells[i].source.tileset_id.as_ref()=="kanto");
        // Mineral relief can protrude past its original cell. Its calibrated
        // offset belongs to the source footprint, not the water or bank under
        // the protrusion. Preserve the owning side at discontinuous contacts.
        let geometric_normal = (Vec3::from_array(old[ids[1]]) - Vec3::from_array(old[ids[0]]))
            .cross(Vec3::from_array(old[ids[2]]) - Vec3::from_array(old[ids[0]]))
            .normalize_or_zero();
        let mineral_cell = (mineral && field.datum.is_some()).then(|| {
            let source=ids.map(|i|source_point(surface.uvs[i]));
            let mut center=source.into_iter().sum::<Vec2>()/3.;
            let lo=source.into_iter().fold(Vec2::splat(f32::INFINITY),Vec2::min);
            let hi=source.into_iter().fold(Vec2::splat(f32::NEG_INFINITY),Vec2::max);
            let span=hi-lo;
            // Folded courses retain a narrow UV thickness across the wall.
            // Recover their authored grid plane before choosing its inside;
            // a tiny centroid bias can otherwise remain in the water cell.
            let axis=if span.y>span.x*1.5 && geometric_normal.x.abs()>0.0001 {
                center.x=center.x.round();Vec2::new(geometric_normal.x.signum(),0.)
            } else if span.x>span.y*1.5 && geometric_normal.z.abs()>0.0001 {
                center.y=center.y.round();Vec2::new(0.,geometric_normal.z.signum())
            } else {Vec2::new(normal.x,normal.z).normalize_or_zero()};
            (center-axis*0.01).floor()
        });
        // The shared terrace field already replaces this drawing's local
        // bank rise. Retire the old fronts, including their tail beyond the
        // authored cut. Keep lateral returns: they seal the bank against a
        // descending stair opening rather than becoming floating end cards.
        let kanto_bank=marked.is_some_and(|i|cells[i].source.tileset_id.as_ref()=="kanto");
        if (closed_lips || (kanto_bank && (normal.z>0.7 || field.drop_face(center,normal)))) && jump && marked.is_some() {
            obsolete[number]=true;
            continue;
        }
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
        if jump && normal.y > 0.9 && ids.iter().all(|&i| old[i][1].abs()<0.001 || (kanto_ground && old[i][1]>=0. && old[i][1]<h*0.5)) {
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
            // Relieved bank returns protrude outside their source cell. Their
            // height belongs to the original wall plane, not the lower ramp
            // beneath that relief. Sampling the protrusion tears open the end.
            let bank_contact = marked.filter(|_|kanto_ground).map(|owner| {
                let (w,e,n,s)=field.g.bounds(owner%field.g.width,owner/field.g.width);
                let eps=field.g.tile_width.min(field.g.tile_height)*0.0001;
                let mut p=position.clamp(Vec2::new(w+eps,n+eps),Vec2::new(e-eps,s-eps));
                let axis=if normal.x.abs()>normal.z.abs() {
                    p.x=if normal.x>0. {e}else{w};Vec2::new(normal.x.signum(),0.)
                } else {
                    p.y=if normal.z>0. {s}else{n};Vec2::new(0.,normal.z.signum())
                };
                (p-axis*eps,p+axis*eps)
            });
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
            let upper = if let Some(cell)=mineral_cell.filter(|_|contact.or(mineral_wall).is_none()) {
                let source=source_point(surface.uvs[i]).clamp(cell+Vec2::splat(0.0001),cell+Vec2::splat(0.9999));
                field.map_height(source)
            } else {field.height(bank_contact.map_or(position+toward,|p|p.0))};
            let y = if let Some((old_floor, old_top)) = contact.or(mineral_wall) {
                // The cliff's cap keeps its authored height, while its foot
                // follows the bank whose old local rise has been removed.
                let t = ((p.y - old_floor) / (old_top - old_floor)).clamp(0., 1.);
                p.y + upper - old_floor * (1. - t)
            } else if jump && normal.y.abs() < 0.9 && p.y >= -0.01
                && p.y <= h + if kanto_ground {0.001}else{0.05} {
                let lower = field.height(bank_contact.map_or(
                    position + Vec2::new(normal.x, normal.z) * 0.01,|p|p.1));
                lower + (upper - lower) * (p.y / h).clamp(0., 1.)
            } else if jump && normal.y > 0.1 && p.y >= -0.01 && p.y <= h + 0.05 {
                upper + if kanto_ground {(p.y-h).max(0.)}else{0.}
            } else {
                p.y + upper - if jump { h } else { 0. }
            };
            surface.positions[i][1] = y;
            if !kanto_ground && surface.uvs[i][0] == super::ledge_approach::BANK_OWNER_UV {
                surface.uvs[i] = super::new_bark::ledge_finish::cliff_material_uv(surface.positions[i],field.g,field.origin);
            }
        }
        if kanto_ground && marked.is_some() {
            let (low,high)=ids.iter().map(|&i|surface.positions[i][1])
                .fold((f32::INFINITY,f32::NEG_INFINITY),|(lo,hi),y|(lo.min(y),hi.max(y)));
            // A relieved wall between equal levels becomes a horizontal card.
            // The real cap already owns that floor; retain only actual returns.
            if high-low<0.001 {obsolete[number]=true;}
        }
    }
    // Keep Kanto provenance until both triangles sharing each quad have used
    // it. Clearing the first triangle's vertices made the second lose ownership.
    for (position,uv) in surface.positions.iter().zip(&mut surface.uvs) {
        if uv[0]==super::ledge_approach::BANK_OWNER_UV {
            *uv=super::new_bark::ledge_finish::cliff_material_uv(*position,field.g,field.origin);
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
    fn cave_pool_corner_fixture()->(Vec<VisualTile>,GridGeometry,crystal_render_api::VisualTerraceDatum) {
        let handle=bevy::prelude::Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(619001),std::marker::PhantomData);
        let make=|column,index,block,x|VisualTile {column,row:0,texture:handle.clone(),priority:false,
            animation_frames:(index==0x14).then(||vec![handle.clone(),handle.clone()].into()),
            source:VisualTileSource {tileset_id:"dark_cave".into(),metatile_id:block,
                subtile_column:x,subtile_row:1,tile_index:index}};
        let mut corner=make(1,0x26,0x27,0);corner.source.subtile_row=0;
        let tiles=vec![make(0,0x14,0x2c,0),corner,make(2,0x16,2,0)];
        let g=GridGeometry {width:3,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let datum=crystal_render_api::VisualTerraceDatum {origin:IVec2::ZERO,size:bevy::prelude::UVec2::new(3,1),
            levels:vec![[2.;4];3].into(),water_barriers:vec![].into(),open_water:None};
        (tiles,g,datum)
    }

    #[test]
    fn borrowed_cave_water_stays_level_while_the_rock_corner_follows_its_land() {
        let (mut tiles,g,datum)=cave_pool_corner_fixture();
        let mut sheet=SurfaceMeshData::default();
        append_top(&mut sheet,[8.,16.,0.,8.],0.,cave_pool_uv(&tiles[0],g.uv(0,0)));
        let original=sheet.clone();
        let cells=tiles.iter().collect::<Vec<_>>();
        let field=Field::with_source(&[],&g,[0,0],&cells).with_datum(Some(&datum));
        assert_eq!(field.height(Vec2::new(12.,4.)),12.);
        shape_surface_excluding_with_lips_on_map(&mut sheet,&cells,&field,None,false,"DarkCaveBlackthornEntrance");
        assert_eq!(sheet,original,"the borrowed live pool must not acquire a dry-land slope");
        let mut dry=SurfaceMeshData::default();
        append_top(&mut dry,[8.,16.,0.,8.],0.,g.uv(1,0));
        shape_surface_excluding_with_lips_on_map(&mut dry,&cells,&field,None,false,"DarkCaveBlackthornEntrance");
        assert!(dry.positions.iter().all(|p|p[1]==12.),"only the water's own source UVs select its level");
        tiles[0].animation_frames=None;
        let cells=tiles.iter().collect::<Vec<_>>();
        let field=Field::with_source(&[],&g,[0,0],&cells).with_datum(Some(&datum));
        let mut unknown=original;
        shape_surface_excluding_with_lips_on_map(&mut unknown,&cells,&field,None,false,"DarkCaveBlackthornEntrance");
        assert!(unknown.positions.iter().all(|p|p[1]==12.),"unproven art cannot claim a pool datum");
    }

    #[test]
    fn cave_banks_close_the_drop_at_dry_land_and_never_wall_off_the_wet_rim() {
        let (tiles,g,datum)=cave_pool_corner_fixture();let cells=tiles.iter().collect::<Vec<_>>();
        let field=Field::with_source(&[],&g,[0,0],&cells).with_datum(Some(&datum));
        let mut wall=SurfaceMeshData::default();
        append_low_water_shore(&mut wall,"DarkCaveBlackthornEntrance",&cells,&field);
        assert_eq!(wall.positions.len(),4);assert_eq!(wall.indices.len(),6);
        assert!(wall.positions.iter().all(|p|(p[0]-16.).abs()<0.002));
        assert!(wall.positions.iter().any(|p|p[1]==0.));assert!(wall.positions.iter().any(|p|p[1]==12.));
        assert!(wall.normals.iter().all(|n|*n==[-1.,0.,0.]));
        assert!(wall.uvs.iter().all(|uv|(uv[1]/100000.).round()==36.));
    }

    #[test]
    fn bank_closure_meets_the_fitted_frame_on_a_sloping_pier_approach() {
        use super::*;
        for scale in [1.,2.] {
            let g=GridGeometry {width:8,height:8,tile_width:8.*scale,tile_height:8.*scale,
                origin_x:-11.,origin_z:17.};
            let frame=super::super::tests::frame(8,8,(0..64).map(|i| {
                let mut s=super::super::tests::source_with_tile(if i/8<4 {0x55}else{5},
                    (i%4)as u8,(i/8%4)as u8,if i/8<4 {0x5b}else{0x4b});
                s.tileset_id="kanto".into();s
            }).collect());
            let cells=frame.tiles.iter().collect::<Vec<_>>();
            let edges=[VisualJumpLedge {start:IVec2::new(0,4),end:IVec2::new(8,4),descent:IVec2::Y}];
            let datum=crystal_render_api::VisualTerraceDatum {
                origin:IVec2::ZERO,size:bevy::prelude::UVec2::splat(8),
                levels:(0..64).map(|i| {
                    let a=0.25+(i%8)as f32*0.75/8.;
                    let b=a+0.75/8.;
                    if i/8<4 {[a,a,b,b]}else{[0.;4]}
                }).collect::<Vec<_>>().into(),open_water:None,water_barriers:Default::default(),
            };
            let field=Field::with_source(&edges,&g,[0,0],&cells).with_datum(Some(&datum));
            let mut deck=TerrainMeshData {authored_cells:vec![None;64],footing_heights:vec![0.;64],..Default::default()};
            super::super::elevated_piers::append(&mut deck,"Route13",&cells,&g,
                &mut vec![false;64],None,Some(&field),[0,0]);
            let mut bank=SurfaceMeshData::default();
            append_drop_faces_with_lips_on_map(&mut bank,&cells,&field,false,"Route13");
            // Compare against the actual fitted structural frame, including
            // approach points below the former fixed four-pixel cutoff.
            let plane=g.origin_z+4.*g.tile_height;
            for x in 1..8 {
                let px=g.origin_x+x as f32*g.tile_width;
                let bottom=deck.solid.positions.iter().filter(|p|
                    (p[0]-px).abs()<0.001 && (p[2]-plane).abs()<0.001)
                    .map(|p|p[1]).fold(f32::INFINITY,f32::min);
                assert!(bottom.is_finite());
                assert!(bank.positions.iter().any(|p|(p[0]-px).abs()<0.001
                    && (p[2]-plane).abs()<0.001 && (p[1]-bottom).abs()<0.003*scale),
                    "bank must meet the fitted deck underside at x={x}, y={bottom}");
                let top=field.height(Vec2::new(px,plane-0.001));
                assert!(bank.positions.iter().any(|p|(p[0]-px).abs()<0.001
                    && (p[2]-plane).abs()<0.001 && (p[1]-top).abs()<0.003*scale),
                    "the original bank lip must remain at x={x}, y={top}");
            }
        }
    }

    #[test]
    fn retained_river_plane_continues_outside_bounds_and_beneath_the_bridge() {
        use super::*;
        let frame=super::super::tests::frame(4,4,(0..16).map(|i| {
            let mut s=super::super::tests::source_with_tile(if i==5 {5}else{0x43},
                (i%4)as u8,(i/4)as u8,if i==5 {0x4b}else{0x14});
            s.tileset_id="kanto".into();s
        }).collect());
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let datum=crystal_render_api::VisualTerraceDatum {origin:IVec2::ONE,
            size:bevy::prelude::UVec2::splat(2),levels:vec![[0.;4];4].into(),water_barriers:Default::default(),
            open_water:Some(crystal_render_api::VisualOpenWaterDatum {level:2.,cells:vec![IVec2::ONE].into()})};
        let field=Field::with_source(&[],&g,[0,0],&cells).with_datum(Some(&datum));
        assert_eq!(field.water_offset(0,&cells),12.,"the visible source-water border continues the river");
        assert_eq!(field.water_offset(10,&cells),0.,"an unrelated native pond retains its own level");
        let mut surface=SurfaceMeshData::default();
        let (w,e,n,s)=g.bounds(1,1);
        append_top(&mut surface,[w,e,n,s],crate::profile::WATER_HEIGHT,g.uv(1,1));
        shape_surface_excluding_with_lips_on_map(&mut surface,&cells,&field,None,false,"CeruleanCity");
        assert!(surface.positions.iter().all(|p|p[1]==crate::profile::WATER_HEIGHT+12.),
            "a bridge cannot hide a second sea-level water plane beneath the raised river");
        let mut closed=datum.clone();closed.open_water=None;closed.levels=vec![[2.;4];4].into();
        let closed_field=Field::with_source(&[],&g,[0,0],&cells).with_datum(Some(&closed));
        assert_eq!(closed_field.water_surface_offset(Vec2::new(-8.,0.)),0.,
            "ocean continuation cannot inherit a native land-edge elevation");
        let mut bed=SurfaceMeshData::default();
        append_top(&mut bed,[w,e,n,s],crate::profile::WATER_HEIGHT,g.uv(1,1));
        shape_surface_excluding_with_lips_on_map(&mut bed,&cells,&closed_field,None,false,"CeruleanCity");
        assert!(bed.positions.iter().all(|p|p[1]==crate::profile::WATER_HEIGHT+12.),
            "connected source maps closing the river must retain its bridge water level too");
        let mut high=closed.clone();high.levels=vec![[8.;4];4].into();
        let high_field=Field::with_source(&[],&g,[0,0],&cells).with_datum(Some(&high));
        let mut frame=SurfaceMeshData::default();
        append_top(&mut frame,[w,e,n,s],52.,g.uv(1,1));
        let fitted=frame.clone();
        shape_surface_excluding_with_lips_on_map(&mut frame,&cells,&high_field,None,false,"CeruleanCity");
        assert_eq!(frame,fitted,"high-river deck frames must not receive a second land-height lift");
    }

    #[test]
    fn repeated_water_outside_native_datum_cannot_inherit_the_edge_terrace() {
        use super::*;
        for atlas in ["kanto", "johto"] {
            let frame = super::super::tests::frame(6, 6, (0..36).map(|i| {
                let mut source = super::super::tests::source_with_tile(0x43,
                    (i % 4) as u8, (i / 6 % 4) as u8, 0x14);
                source.tileset_id = atlas.into();
                source
            }).collect());
            let cells = frame.tiles.iter().collect::<Vec<_>>();
            let g = GridGeometry { width: 6, height: 6, tile_width: 8., tile_height: 8.,
                origin_x: -17., origin_z: 23. };
            let datum = crystal_render_api::VisualTerraceDatum {
                origin: IVec2::new(7, 11), size: bevy::prelude::UVec2::splat(2),
                levels: vec![[1.;4];4].into(), open_water:None,water_barriers: Default::default(),
            };
            let field = Field::with_source(&[], &g, [6,10], &cells).with_datum(Some(&datum));
            let mut surface = SurfaceMeshData::default();
            for i in 0..36 {
                let (w,e,n,s) = g.bounds(i%6,i/6);
                append_top(&mut surface,[w,e,n,s],crate::profile::WATER_HEIGHT,g.uv(i%6,i/6));
            }
            let original = surface.clone();
            shape_surface(&mut surface,&cells,&field);
            for i in 0..36 {
                let inside = (1..3).contains(&(i%6)) && (1..3).contains(&(i/6));
                let expected = if inside {crate::profile::JUMP_LEDGE_HEIGHT} else {0.};
                assert_eq!(field.water_offset(i,&cells),expected,"{atlas} cell {i}");
                for j in i*4..i*4+4 {
                    assert_eq!(surface.positions[j][1],original.positions[j][1]+expected,
                        "{atlas} water geometry must agree with its support at cell {i}");
                }
            }
            assert_eq!(surface.indices,original.indices);
            assert_eq!(surface.uvs,original.uvs);
            assert_eq!(field.height(bevy::prelude::Vec2::new(-16.,24.)),
                crate::profile::JUMP_LEDGE_HEIGHT,"land continuation keeps its edge sampling");
        }
    }
    #[test]
    fn shoreline_backing_keeps_its_bank_height_at_a_bridge_corner() {
        let make=|block,column,row,tile| {
            let mut s=super::super::tests::source_with_tile(block,column,row,tile);
            s.tileset_id="kanto".into();s
        };
        let frame=super::super::tests::frame(2,2,vec![make(0x1f,0,0,0x33),make(5,0,0,0x4b),
            make(0x1f,0,1,0x14),make(5,0,1,0x4b)]);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:2,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut levels=vec![[0.;4];4];levels[0]=[8.;4];
        let datum=crystal_render_api::VisualTerraceDatum {origin:IVec2::ZERO,size:bevy::prelude::UVec2::splat(2),
            levels:levels.into(),open_water:None,water_barriers:Default::default()};
        let field=Field::new(&[],&g,[0,0]).with_datum(Some(&datum));
        let mut surface=SurfaceMeshData::default();
        append_low_water_shore(&mut surface,"Route25",&cells,&field);
        for x in [0.,8.] {
            let high=surface.positions.iter().filter(|p|(p[0]-x).abs()<0.001
                && (p[2]-8.).abs()<0.01).map(|p|p[1]).fold(f32::NEG_INFINITY,f32::max);
            assert!((high-48.).abs()<0.001,"bank endpoint {x} must meet its cap instead of borrowing the bridge datum");
        }
        assert!(surface.positions.iter().any(|p|(p[1]-crate::profile::WATER_HEIGHT).abs()<0.001),"water stays at its original level");
        let mut flat=datum.clone();flat.levels=vec![[0.;4];4].into();
        let mut control=SurfaceMeshData::default();
        append_low_water_shore(&mut control,"Route25",&cells,&Field::new(&[],&g,[0,0]).with_datum(Some(&flat)));
        assert!(control.positions.is_empty(),"an unraised bank retains its existing native lip");
    }

    #[test]
    fn raised_river_bank_follows_its_land_corners_instead_of_a_pond_center() {
        let mut ground=super::super::tests::source_with_tile(0x23,0,0,0x01);
        ground.tileset_id="kanto".into();
        let mut bank=super::super::tests::source_with_tile(0x1f,1,0,0x33);
        bank.tileset_id="kanto".into();
        let mut water=super::super::tests::source_with_tile(0x43,1,1,0x14);
        water.tileset_id="kanto".into();
        let frame=super::super::tests::frame(3,2,vec![ground.clone(),bank,ground.clone(),ground.clone(),water,ground]);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:3,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut levels=vec![[1.;4];6];levels[1]=[2.,1.,1.,2.];
        let datum=crystal_render_api::VisualTerraceDatum {origin:IVec2::ZERO,size:bevy::prelude::UVec2::new(3,2),
            levels:levels.into(),open_water:None,water_barriers:Default::default()};
        let field=Field::new(&[],&g,[0,0]).with_datum(Some(&datum));
        let mut bank=SurfaceMeshData::default();
        append_top(&mut bank,[8.,16.,0.,8.],0.,g.uv(1,0));
        let mut land=bank.clone();
        for (uv,p) in bank.uvs.iter_mut().zip(&bank.positions) {*uv=[88000.+p[0]/8.,-70000.-p[2]/8.];}
        shape_surface(&mut bank,&cells,&field);
        shape_surface(&mut land,&cells,&field);
        assert_eq!(bank.positions,land.positions,"the bank cap must meet the graded land along both edges");
        assert!(bank.positions.iter().any(|p|p[1]>11.9));
        assert!(bank.positions.iter().any(|p|p[1]<6.1));
    }

    #[test]
    fn sealed_pond_bank_keeps_one_offset_across_its_sculpted_source_edge() {
        use super::*;
        let mut rock=super::super::tests::source_with_tile(0x2c,0,0,0x11);rock.tileset_id="kanto".into();
        let mut bank=super::super::tests::source_with_tile(0x1f,1,0,0x33);bank.tileset_id="kanto".into();
        let mut water=super::super::tests::source_with_tile(0x43,1,1,0x14);water.tileset_id="kanto".into();
        let frame=super::super::tests::frame(3,2,vec![rock.clone(),bank,rock.clone(),rock.clone(),water,rock]);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:3,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut levels=vec![[1.;4];6];levels[1]=[22./crate::profile::JUMP_LEDGE_HEIGHT;4];levels[4]=levels[1];
        let mut datum=crystal_render_api::VisualTerraceDatum {origin:IVec2::ZERO,size:bevy::prelude::UVec2::new(3,2),
            levels:levels.into(),open_water:None,water_barriers:std::sync::Arc::from([])};
        let points=vec![[15.7,0.,5.],[16.2,0.2,5.2],[15.8,0.8,7.]];
        let original=SurfaceMeshData {uvs:points.iter().map(|p|[88000.+p[0]/8.,-70000.-p[2]/8.]).collect(),
            positions:points,normals:vec![[0.,1.,0.];3],colors:vec![[0.3,0.3,0.3,1.];3],indices:vec![0,1,2],..Default::default()};
        let mut shifted=original.clone();
        shape_surface(&mut shifted,&cells,&Field::new(&[],&g,[0,0]).with_datum(Some(&datum)));
        for (a,b) in shifted.positions.iter().zip(&original.positions) {
            assert!((a[1]-b[1]-22.).abs()<0.0001,"sculpted bank must not inherit the lower neighboring offset");
            assert_eq!([a[0],a[2]],[b[0],b[2]]);
        }
        assert_eq!(shifted.uvs,original.uvs);
        datum.levels=vec![[0.;4];6].into();
        let mut retained=original.clone();
        shape_surface(&mut retained,&cells,&Field::new(&[],&g,[0,0]).with_datum(Some(&datum)));
        assert_eq!(retained.positions,original.positions,"ordinary source banks keep their original grading");
    }

    #[test]
    fn sculpted_volcanic_cap_keeps_its_own_level_when_it_protrudes_into_a_raised_pond() {
        use super::*;
        let mut rock=super::super::tests::source_with_tile(0x2c,0,0,0x11);rock.tileset_id="kanto".into();
        let mut water=super::super::tests::source_with_tile(0x43,1,0,0x14);water.tileset_id="kanto".into();
        let frame=super::super::tests::frame(2,1,vec![rock,water]);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:2,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let datum=crystal_render_api::VisualTerraceDatum {origin:IVec2::ZERO,size:bevy::prelude::UVec2::new(2,1),
            levels:vec![[0.;4],[16./crate::profile::JUMP_LEDGE_HEIGHT;4]].into(),open_water:None,water_barriers:std::sync::Arc::from([])};
        let field=Field::new(&[],&g,[0,0]).with_datum(Some(&datum));
        let mut surface=SurfaceMeshData {positions:vec![[8.1,16.,1.],[8.1,16.,7.],[8.9,16.,7.]],
            normals:vec![[0.,1.,0.];3],uvs:vec![[-10007.,-60001.],[-10007.,-60007.],[-10008.,-60007.]],
            colors:vec![[0.2,0.2,0.2,1.];3],indices:vec![0,1,2],..Default::default()};
        let original=surface.clone();
        shape_surface(&mut surface,&cells,&field);
        assert_eq!(surface.positions,original.positions,"mineral cap cannot inherit the pond's lift");
        assert_eq!(surface.uvs,original.uvs);
        assert_eq!(surface.indices,original.indices);
        // A folded source wall has thickness beyond its exact native plane.
        // Its centroid can lie in the water even though the face belongs to rock.
        let mut wall=SurfaceMeshData {positions:vec![[8.05,0.,1.],[8.05,16.,1.],[8.05,16.,7.]],
            normals:vec![[1.,0.,0.];3],uvs:vec![[-10008.05,-60001.],[-10008.05,-60001.],[-10008.05,-60007.]],
            colors:vec![[0.2,0.2,0.2,1.];3],indices:vec![0,1,2],..Default::default()};
        let original_wall=wall.clone();
        shape_surface(&mut wall,&cells,&field);
        assert_eq!(wall.positions,original_wall.positions,"a folded mineral course keeps its original rock-side offset");
        let mut shore=SurfaceMeshData::default();
        append_low_water_shore(&mut shore,"CinnabarIsland",&cells,&field);
        assert!(!shore.indices.is_empty(),"the raised pond needs a continuous native mineral bank");
        assert!(shore.positions.iter().any(|p|(p[1]-(16.+crate::profile::WATER_HEIGHT)).abs()<0.001));
        assert!(shore.positions.iter().any(|p|(p[1]-16.).abs()<0.001));
        assert!(shore.positions.iter().all(|p|p[1]<18.),"the shore is a shallow lip, never a tall retaining wall");
    }

    use super::*;
    #[test]
    fn recessed_sea_rocks_close_the_native_flat_bank_side_without_raising_the_cap() {
        let g=GridGeometry {width:4,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let frame=super::super::tests::frame(4,2,(0..8).map(|i| {
            let (x,y)=(i%4,i/4);
            let mut s=super::super::tests::source_with_tile(if x<2 {0x6b}else{0x1f},
                x as u8,if x<2 {y as u8}else{0},if x<2 {[[0x2a,0x2b],[0x3a,0x3b]][y][x]}else{0x33});
            s.tileset_id="kanto".into();s
        }).collect());
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        assert!(super::super::kanto_shore::source_bank(&cells[2].source));
        assert!(!matches!(super::super::new_bark::ground_material(&cells[2].source),Some(
            super::super::new_bark::GroundMaterial::Lawn|super::super::new_bark::GroundMaterial::Path
            |super::super::new_bark::GroundMaterial::Pavers)));
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;8],
            authored_cells:(0..8).map(|i|Some(if i%4<2 {"world-exterior/kanto_boundary_shore"}
                else{"outdoor/kanto-shore"})).collect(),..Default::default()};
        append_top(&mut mesh.textured,[0.,16.,0.,16.],crate::profile::WATER_HEIGHT,(0.,1.,0.,1.));
        let before=mesh.clone();
        close_flat_rock_contacts(&mut mesh,"VermilionCity",&cells,&g,[0,0],None);
        assert!(!mesh.solid.positions.is_empty(),"the recessed rock bed exposes a real native bank side");
        assert!(mesh.solid.positions.iter().all(|p|p[0]>15. && p[0]<16.01 && p[1]>=-2. && p[1]<=0.),
            "only the hidden side between the existing bank and sea level is added");
        assert!(mesh.solid.positions.iter().any(|p|p[1]==0.));
        assert!(mesh.solid.positions.iter().any(|p|p[1]==crate::profile::WATER_HEIGHT));
        assert_eq!(mesh.textured,before.textured);
        assert_eq!(mesh.footing_heights,before.footing_heights);
        assert_eq!(mesh.authored_cells,before.authored_cells);
    }

    #[test]
    fn modeled_sea_rock_beds_close_level_sand_contacts_without_new_rims_or_water_walls() {
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let make=|dry:bool|super::super::tests::frame(4,4,(0..16).map(|i| {
            let (x,y)=(i%4,i/4);
            let mut s=super::super::tests::source_with_tile(0x43,(x%4)as u8,(y%4)as u8,0x14);
            s.tileset_id="kanto".into();
            if x<2 {
                s.metatile_id=0x6b;s.subtile_column=x as u8;s.subtile_row=(y%2)as u8;
                s.tile_index=[[0x2a,0x2b],[0x3a,0x3b]][y%2][x];
            } else if dry {s.metatile_id=0x31;s.tile_index=0x39;}
            s
        }).collect());
        let frame=make(true);let cells=frame.tiles.iter().collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;16],
            authored_cells:(0..16).map(|i|(i%4<2).then_some("world-exterior/kanto_boundary_shore")).collect(),
            ..Default::default()};
        append_top(&mut mesh.textured,[0.,16.,0.,32.],crate::profile::WATER_HEIGHT,(0.,1.,0.,1.));
        append_solid_quad(&mut mesh.solid,[[1.,crate::profile::WATER_HEIGHT,1.],[1.,3.,1.],[7.,3.,1.],[7.,crate::profile::WATER_HEIGHT,1.]],
            [0.,0.,1.],[0.5;4]);
        let original=mesh.clone();
        let neighboring_terrace=crystal_render_api::VisualTerraceDatum {
            origin:IVec2::ZERO,size:bevy::prelude::UVec2::splat(4),
            levels:vec![[2.;4];16].into(),open_water:None,water_barriers:Default::default(),
        };
        finish_with_datum(&mut mesh,"Route20",&cells,&g,[0,0],&[],Some(&neighboring_terrace));
        let added=&mesh.solid.positions[original.solid.positions.len()..];
        assert!(!added.is_empty(),"a recessed rock bed must close the neighboring flat sand underside");
        assert!(added.iter().all(|p|p[1]>=crate::profile::WATER_HEIGHT&&p[1]<=0.),
            "only the concealed face below the existing bank, never a new rim or cap");
        assert!(added.iter().all(|p|p[0]>14.&&p[0]<17.),"only the exact rock/land contact");
        assert!(mesh.solid.normals[original.solid.positions.len()..].iter().all(|n|n[1].abs()<0.9),
            "no horizontal top surface is introduced");
        assert_eq!(mesh.textured,original.textured,"water positions, UVs and source clock stay exact");
        assert_eq!(mesh.footing_heights,original.footing_heights);
        assert_eq!(mesh.authored_cells,original.authored_cells);
        assert_eq!(&mesh.solid.positions[..original.solid.positions.len()],&original.solid.positions);
        let sea=make(false);let refs=sea.tiles.iter().collect::<Vec<_>>();
        let mut offshore=original.clone();
        close_flat_rock_contacts(&mut offshore,"Route20",&refs,&g,[0,0],None);
        assert_eq!(offshore,original,"no walls between two water owners");
        let mut unmodeled=original.clone();unmodeled.authored_cells.fill(None);
        let before=unmodeled.clone();
        close_flat_rock_contacts(&mut unmodeled,"Route20",&cells,&g,[0,0],None);
        assert_eq!(unmodeled,before,"a source drawing alone cannot authorize a recessed bed");
    }
    #[test]
    fn water_backed_shore_rocks_close_only_the_raised_land_contact() {
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let make=|land:bool|super::super::tests::frame(4,4,(0..16).map(|i| {
            let (x,y)=(i%4,i/4);
            let mut s=super::super::tests::source_with_tile(1,(x%4)as u8,(y%4)as u8,0x14);
            s.tileset_id="kanto".into();
            if x<2 {
                s.metatile_id=0x6b;s.subtile_column=x as u8;s.subtile_row=(y%2+2)as u8;
                s.tile_index=if y%2==0 {if x==0 {0x2a}else{0x2b}} else if x==0 {0x3a}else{0x3b};
            } else if land {s.tile_index=0x39;}
            s
        }).collect());
        let mut frame=make(true);frame.map_id="Route18".into();
        frame.jump_ledges=vec![VisualJumpLedge {start:IVec2::new(0,3),end:IVec2::new(4,3),descent:IVec2::Y}].into();
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        assert!(crate::kanto_cliff::shoreline_round_barrier_local(&cells[0].source).is_some());
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;16],..Default::default()};
        let water=crate::profile::WATER_HEIGHT;
        for row in 0..4 {for col in 0..2 {
            append_top(&mut mesh.textured,[col as f32*8.,(col+1)as f32*8.,row as f32*8.,(row+1)as f32*8.],water,g.uv(col,row));
        }}
        append_solid_quad(&mut mesh.solid,[[4.,water,4.],[4.,water+1.,4.],[8.,water+1.,4.],[8.,water,4.]],
            [0.,0.,1.],[0.5;4]);
        mesh.solid.uvs.fill([super::super::shore_ecology::ROOTED_WATER_UV,0.]);
        let original=mesh.clone();finish_on_map(&mut mesh,"Route18",&cells,&g,[0,0],&frame.jump_ledges);
        assert_eq!(mesh.textured,original.textured,"all water-underlay vertices, normals and UVs stay exact");
        assert_eq!(&mesh.solid.positions[..original.solid.positions.len()],&original.solid.positions,
            "the original sea rock stays on the water datum");
        let field=Field::new(&frame.jump_ledges,&g,[0,0]);
        let mut backing=SurfaceMeshData::default();append_low_water_shore(&mut backing,"Route18",&cells,&field);
        assert!(!backing.positions.is_empty(),"close the land-side gap behind the protected rock row");
        assert!(backing.positions.iter().any(|p|p[1]>1.));
        // Existing fractured support may protrude slightly toward its water
        // contact. It cannot span the pool or add a new top above the land.
        assert!(backing.positions.iter().all(|p|p[0]>14. && p[0]<18.));
        for row in 0..4 {for col in 0..2 {assert_eq!(mesh.footing_heights[row*4+col],0.);}}
        let sea=make(false);let refs=sea.tiles.iter().collect::<Vec<_>>();
        let mut no_wall=SurfaceMeshData::default();
        append_low_water_shore(&mut no_wall,"Route18",&refs,&field);
        assert!(no_wall.positions.is_empty(),"no walls between offshore rocks and standing water");
    }
    #[test]
    fn continued_border_water_cannot_regrow_terrace_walls_but_real_shores_stay_closed() {
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let f=super::super::tests::frame(4,4,(0..16).map(|i| {
            let mut s=super::super::tests::source_with_tile(0x43,0,0,0x14);
            s.tileset_id="kanto".into();
            if i==5 {s.metatile_id=0x2d;s.subtile_column=3;s.tile_index=0x33;}
            if i==3 {s.metatile_id=0x31;s.tile_index=0x39;}
            s
        }).collect());
        let cells=f.tiles.iter().collect::<Vec<_>>();
        let edges=[VisualJumpLedge{start:IVec2::new(0,3),end:IVec2::new(4,3),descent:IVec2::Y}];
        let field=Field::with_source(&edges,&g,[0,0],&cells);
        let mut mask=vec![false;16];mask[5]=true;
        let mut old=SurfaceMeshData::default();append_low_water_shore(&mut old,"MountMoonSquare",&cells,&field);
        assert!(old.positions.iter().any(|p|p[0]<18.),"the former corner would grow a false island");
        let mut corrected=SurfaceMeshData::default();
        append_low_water_shore_with_continuation(&mut corrected,"MountMoonSquare",&cells,&field,&mask,None);
        assert!(!corrected.positions.is_empty(),"the genuine raised shore retains its backing");
        assert!(corrected.positions.iter().all(|p|p[0]>20.),"no wall around retired border corners");
        let mut mesh=TerrainMeshData{footing_heights:vec![0.;16],authored_cells:vec![None;16],..Default::default()};
        mesh.authored_cells[5]=Some("outdoor/border-water");
        append_top(&mut mesh.animated_textured,[8.,16.,8.,16.],crate::profile::WATER_HEIGHT,g.uv(0,1));
        let surface=mesh.animated_textured.clone();
        finish_on_map(&mut mesh,"MountMoonSquare",&cells,&g,[0,0],&edges);
        assert_eq!(mesh.animated_textured,surface,"continued water stays one level");
        assert_eq!(mesh.footing_heights[5],0.,"water support retains its original datum");
        assert!(mesh.footing_heights[3]>0.,"actual dry shore retains its terrace");
    }
    #[test]
    fn floating_buoy_courses_cannot_acquire_land_terrace_walls_or_height() {
        for (family,block,column) in [("johto",0x34,0),("johto_modern",0x34,0),
            ("johto",0x3a,2),("johto_modern",0x3a,2)] {
            let g=GridGeometry {width:4,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let mut frame=super::super::tests::frame(4,8,(0..32).map(|i| {
                let mut s=super::super::tests::source_with_tile(if i%4==1 {block}else{1},
                    if i%4==1 {column}else{(i%4) as u8},(i/4%4)as u8,
                    if i%4==1 {0x58}else if i%4<3 {0x14}else{6});
                s.tileset_id=family.into();s
            }).collect());
            frame.map_id="OlivineCity".into();
            frame.jump_ledges=vec![VisualJumpLedge {start:IVec2::new(0,4),
                end:IVec2::new(4,4),descent:IVec2::Y}].into();
            let cells=frame.tiles.iter().collect::<Vec<_>>();
            assert!(super::super::buoy_course::source(&cells[1].source));
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;32],..Default::default()};
            let water=crate::profile::WATER_HEIGHT;
            for row in 0..8 {for col in 0..3 {
                append_top(&mut mesh.textured,[col as f32*8.,(col+1)as f32*8.,
                    row as f32*8.,(row+1)as f32*8.],water,(0.,1.,0.,1.));
                if col==1 {append_solid_quad(&mut mesh.solid,
                    [[10.,water-0.5,row as f32*8.+4.],[10.,water+1.,row as f32*8.+4.],
                    [14.,water+1.,row as f32*8.+4.],[14.,water-0.5,row as f32*8.+4.]],
                    [0.,0.,1.],[1.;4]);}
            }}
            let original=mesh.clone();
            finish_on_map(&mut mesh,"OlivineCity",&cells,&g,[0,0],&frame.jump_ledges);
            assert_eq!(mesh.textured.positions,original.textured.positions,"level water bed");
            assert_eq!(&mesh.solid.positions[..original.solid.positions.len()],&original.solid.positions,
                "floating caps cannot inherit the island's height");
            for tri in mesh.solid.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|mesh.solid.positions[i as usize]);
                if p.iter().all(|p|p[0]<20.) {
                    assert!(p.iter().all(|p|p[1]<=water+1.001),"no new walls along floating courses: {p:?}");
                }
            }
            for row in 0..8 {assert_eq!(mesh.footing_heights[row*4+1],0.);}
            assert!(mesh.footing_heights.iter().enumerate().any(|(i,h)|i%4==3&&*h>0.),
                "the actual island still owns its terrace");
        }
    }

    #[test]
    fn island_terraces_never_slope_water_or_its_surf_support() {
        for (map, tileset) in [("Route42", "johto"), ("Route22", "kanto"),
            ("TohjoFalls", "cave")] {
            let g = GridGeometry { width: 4, height: 8, tile_width: 8., tile_height: 8.,
                origin_x: 0., origin_z: 0. };
            let mut frame = super::super::tests::frame(4, 8, (0..32).map(|i| {
                let mut source = super::super::tests::source_with_tile(if tileset=="cave" {1}else{0x43}, (i % 4) as u8,
                    (i / 4 % 4) as u8, if i % 4 < 2 { 0x14 } else { 0x2c });
                source.tileset_id = tileset.into(); source
            }).collect());
            frame.map_id = map.into();
            frame.jump_ledges = vec![VisualJumpLedge { start: IVec2::new(0,4),
                end: IVec2::new(4,4), descent: IVec2::Y }].into();
            let cells = frame.tiles.iter().collect::<Vec<_>>();
            assert!(level_water(&cells[0].source),"{map}: native water fixture");
            let mut mesh = TerrainMeshData { footing_heights: vec![0.;32], ..Default::default() };
            for surface in [&mut mesh.solid, &mut mesh.textured,
                &mut mesh.animated_solid, &mut mesh.animated_textured] {
                for row in 0..8 { for col in 0..2 {
                    append_top(surface, [col as f32*8.,(col+1) as f32*8.,row as f32*8.,(row+1) as f32*8.],
                        crate::profile::WATER_HEIGHT, (0.,1.,0.,1.));
                }}
            }
            let original = mesh.clone();
            finish_on_map(&mut mesh, map, &cells, &g, [0,0], &frame.jump_ledges);
            for (after, before) in [&mesh.solid,&mesh.textured,&mesh.animated_solid,&mesh.animated_textured]
                .into_iter().zip([&original.solid,&original.textured,&original.animated_solid,&original.animated_textured]) {
                assert_eq!(&after.positions[..before.positions.len()],before.positions.as_slice(),"{map}: water geometry");
            }
            for tri in mesh.solid.indices.chunks_exact(3) {
                let p = [tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.solid.positions[i as usize]));
                let center = (p[0]+p[1]+p[2])/3.;
                // Exclude the bank's fractured contact shoulder itself;
                // a wall through the open pool is the forbidden transition.
                if center.x>=0. && center.x<8. && center.z>=0. && center.z<64. {
                    assert!(p.iter().all(|p|(p.y-crate::profile::WATER_HEIGHT).abs()<0.0001),
                        "{map}: land ledge wall must not cut through the water: {p:?}");
                }
            }
            for row in 0..8 { for col in 0..2 {
                let i=row*4+col;
                assert_eq!(mesh.footing_heights[i],0.,"{map}: water support");
                assert_eq!(support_correction(&frame,bevy::prelude::UVec2::new(col as u32,row as u32),
                    Vec2::new(col as f32*8.-12.,28.-row as f32*8.)),0.);
            }}
            assert!(mesh.footing_heights.iter().enumerate().any(|(i,h)|i%4>=2&&*h>0.),
                "{map}: the island land still owns its ledge");
        }
    }

    #[test]
    fn level_city_and_tower_hops_leave_mesh_and_actor_support_unchanged() {
        let g=GridGeometry{width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let tiles=(0..16).map(|i|VisualTile{column:(i%4)as u32,row:(i/4)as u32,
            texture:Default::default(),priority:false,animation_frames:None,
            source:VisualTileSource{tileset_id:"tower".into(),metatile_id:9,
                subtile_column:(i%4)as u8,subtile_row:(i/4)as u8,tile_index:2}}).collect::<Vec<_>>();
        let refs=tiles.iter().collect::<Vec<_>>();
        let edges=vec![VisualJumpLedge{start:IVec2::new(0,2),end:IVec2::new(4,2),descent:IVec2::Y}];
        let mut original=TerrainMeshData{footing_heights:vec![0.;16],..Default::default()};
        for surface in [&mut original.solid,&mut original.textured,&mut original.animated_solid,&mut original.animated_textured] {
            append_top(surface,[0.,32.,0.,32.],0.,(0.,1.,0.,1.));
        }
        let maps=(1..=9).map(|level|format!("TinTower{level}F")).chain(["SaffronCity".to_owned(),"Route20".to_owned()]);
        for name in maps {
            let mut mesh=original.clone();
            finish_on_map(&mut mesh,&name,&refs,&g,[0,0],&edges);assert_eq!(mesh,original);
            let frame=VisualWorldFrame{map_id:name.into(),jump_ledges:edges.clone().into(),
                grid_size:bevy::prelude::UVec2::splat(4),tile_size:Vec2::splat(8.),..Default::default()};
            assert_eq!(support_correction(&frame,bevy::prelude::UVec2::ZERO,Vec2::new(3.,7.)),0.);
        }
        let mut outdoor=original.clone();finish_on_map(&mut outdoor,"RouteTest",&refs,&g,[0,0],&edges);
        assert!(outdoor.footing_heights.iter().any(|h|*h>0.),"outdoor terraces keep their cumulative elevation");
    }
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
    fn kanto_cap_detail_side_faces_keep_their_clearance_above_ground() {
        let g=GridGeometry{width:4,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let sources=(0..8).flat_map(|y|(0..4).map(move|x| {
            let mut s=super::super::tests::source_with_tile(if y<4 {0x07}else{0x0a},x,y%4,
                if y==3 {0x37}else{0x2c});s.tileset_id="kanto".into();s
        })).collect();
        let frame=super::super::tests::frame(4,8,sources);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let edges=[VisualJumpLedge{start:IVec2::new(0,4),end:IVec2::new(4,4),descent:IVec2::Y}];
        let mut field=Field::new(&edges,&g,[0,0]);
        field.levels=super::super::terrace_levels::get_for_family(&edges,true);
        let mut detail=SurfaceMeshData::default();
        append_solid_quad(&mut detail,[[12.,6.012,12.],[12.,6.04,12.],
            [12.,6.04,13.],[12.,6.012,13.]],[1.,0.,0.],[0.3,0.2,0.1,1.]);
        let original=detail.positions.clone();
        shape_surface_excluding(&mut detail,&cells,&field,None);
        assert_eq!(detail.positions,original,"pebble sides cannot collapse into coplanar floor cards");
        assert_eq!(detail.indices.len(),6);
    }

    #[test]
    fn kanto_return_above_stair_row_cannot_become_a_floating_floor_card() {
        let g=GridGeometry{width:4,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let sources=(0..8).flat_map(|y|(0..4).map(move|x| {
            let mut s=super::super::tests::source_with_tile(if y<4 {0x2f}else{0x0a},x,y%4,
                if y==3 {[0x37,0x34,0x04,0x04][x as usize]}else{0x2c});s.tileset_id="kanto".into();s
        })).collect();
        let frame=super::super::tests::frame(4,8,sources);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let edges=[VisualJumpLedge{start:IVec2::new(0,4),end:IVec2::new(2,4),descent:IVec2::Y}];
        let mut field=Field::new(&edges,&g,[0,0]);
        field.levels=super::super::terrace_levels::get_for_family(&edges,true);
        let mut wall=SurfaceMeshData::default();
        append_solid_quad(&mut wall,[[16.,1.,16.],[16.4,3.,16.],
            [16.3,3.,24.],[16.1,1.,24.]],[1.,0.,0.],[0.3,0.2,0.1,1.]);
        super::super::ledge_approach::mark_bank_surface(&mut wall,0,9);
        shape_surface_excluding(&mut wall,&cells,&field,None);
        assert!(wall.indices.is_empty(),"flattened relief must not remain on the walking cap");
        assert!(wall.positions.iter().all(|p|(p[1]-6.).abs()<0.001),
            "both triangles must retain their source owner until fitting finishes");
    }

    #[test]
    fn relieved_kanto_return_keeps_its_upper_bank_contact_above_the_stair_ramp() {
        let g=GridGeometry{width:4,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let sources=(0..8).flat_map(|y|(0..4).map(move|x| {
            let mut s=super::super::tests::source_with_tile(if y<4 {0x2f}else{0x0a},x,y%4,
                if y==3 {[0x37,0x34,0x04,0x04][x as usize]}else{0x2c});
            s.tileset_id="kanto".into();s
        })).collect();
        let frame=super::super::tests::frame(4,8,sources);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let edges=[VisualJumpLedge{start:IVec2::new(0,4),end:IVec2::new(2,4),descent:IVec2::Y}];
        let mut field=Field::new(&edges,&g,[0,0]);
        field.levels=super::super::terrace_levels::get_for_family(&edges,true);
        let mut wall=SurfaceMeshData::default();
        append_solid_quad(&mut wall,[[16.4,0.,24.],[16.4,6.,24.],
            [16.4,6.,32.],[16.4,0.,32.]],[1.,0.,0.],[0.3,0.2,0.1,1.]);
        super::super::ledge_approach::mark_bank_surface(&mut wall,0,13);
        shape_surface_excluding(&mut wall,&cells,&field,None);
        assert!(!wall.indices.is_empty());
        assert!((wall.positions[1][1]-6.).abs()<0.001);
        assert!((wall.positions[2][1]-6.).abs()<0.001,"the relieved corner cannot dive onto the bypass");
        assert!(wall.positions[3][1]<0.002);
        assert!(wall.uvs.iter().all(|uv|uv[0]!=super::super::ledge_approach::BANK_OWNER_UV));
    }

    #[test]
    fn closed_lips_retire_legacy_bank_faces_beyond_the_directed_cut_end() {
        let g=GridGeometry{width:4,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let sources=(0..8).flat_map(|y|(0..4).map(move |x| {
            let mut s=super::super::tests::source_with_tile(0x07,x,y%4,if y%4==3 {0x37}else{0x2c});
            s.tileset_id="kanto".into();s
        })).collect();
        let frame=super::super::tests::frame(4,8,sources);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let edges=[VisualJumpLedge{start:IVec2::new(0,4),end:IVec2::new(2,4),descent:IVec2::Y}];
        let field=Field::new(&edges,&g,[0,0]);
        let mut old=SurfaceMeshData::default();
        append_solid_quad(&mut old,[[32.,0.,32.],[32.,6.,32.],[24.,6.,32.],[24.,0.,32.]],
            [0.,0.,1.],[0.43,0.385,0.29,1.]);
        super::super::ledge_approach::mark_bank_surface(&mut old,0,15);
        let mut new=old.clone();
        // Unattributed legacy geometry uses centroid inference. A proven
        // Kanto owner now selects the closed replacement without a town switch.
        old.uvs.fill([0.;2]);
        shape_surface_excluding(&mut old,&cells,&field,None);
        shape_surface_excluding_with_lips(&mut new,&cells,&field,None,false);
        assert!(!old.indices.is_empty(),"the old face survived outside the cut span");
        assert!(new.indices.is_empty(),"replacement must not retain a flattened legacy wall");
    }

    #[test]
    fn blackthorn_closed_lips_preserve_authoritative_terrace_support() {
        let mut frame=super::super::tests::frame(8,8,
            vec![super::super::tests::source_with_tile(0,0,0,5);64]);
        frame.jump_ledges=vec![VisualJumpLedge{start:IVec2::new(2,4),end:IVec2::new(6,4),descent:IVec2::Y}].into();
        frame.map_id="Route37".into();
        let old=build_terrain_mesh(&frame).unwrap();
        frame.map_id="BlackthornCity".into();
        let new=build_terrain_mesh(&frame).unwrap();
        assert_eq!(new.footing_heights,old.footing_heights);
        assert_eq!(new.authored_cells,old.authored_cells);
        assert_ne!(new.solid,old.solid,"the town's closed lip must reach production meshing");
        for tri in new.solid.indices.chunks_exact(3) {
            let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(new.solid.positions[i as usize]));
            assert!((p[1]-p[0]).cross(p[2]-p[0]).length_squared()>1e-10);
        }
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
