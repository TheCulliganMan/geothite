//! Original source-complete facility tables and square-backed chairs.
use super::*;
use crate::interior_models::Model;
use crate::live_profiles::Document;
use std::sync::OnceLock;
#[path = "facility_tables_source.rs"]
mod source;
#[path = "facility_chair_facing.rs"]
mod chair_facing;
use source::{Identity, Kind};
pub(super) struct Placement {
    resolved: source::Match,
    chair_turn: u8,
    chair_target: Option<source::Match>,
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        self.resolved.indices(width)
    }
}
fn identity(s: &VisualTileSource) -> Identity<'_> {
    Identity {
        tileset: &s.tileset_id,
        metatile: s.metatile_id,
        column: s.subtile_column,
        row: s.subtile_row,
        tile: s.tile_index,
    }
}
fn model(kind: Kind) -> &'static Model {
    static MODELS: OnceLock<[Model; 4]> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!(
                "models/facility_tables/facility_square_document_table.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/facility_tables/facility_meeting_table.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/facility_tables/facility_document_side_desk.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/facility_tables/facility_square_back_chair.mesh.json"
            ),
        ]
        .map(|s| Model::parse(s).expect("validated original facility table mesh"))
    })[kind as usize]
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if g.width.checked_mul(g.height) != Some(cells.len()) || reserved.len() != cells.len() {
        return Vec::new();
    }
    let mut customized = vec![false; cells.len()];
    // All live profiles win, including renamed profiles with identical source.
    // Existing generic signature models are replaced only after this resolves.
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            customized[i] = true;
        }
    }
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    let resolved=source::resolve(
        map,
        &identities,
        g.width,
        g.height,
        origin,
        reserved,
        &customized,
    )
    .into_iter()
    .filter(|p| if p.native_silph_meeting_table() {native_meeting_model().is_some()}
        else {matches!(p.kind(),Kind::Sofa|Kind::ReceptionCounter)
            || model(p.kind()).has_native_facility_furniture_materials()})
    .collect::<Vec<_>>();
    let facings=resolved.iter().map(|p|chair_facing::choose(p,&resolved)).collect::<Vec<_>>();
    resolved.into_iter().zip(facings).map(|(resolved,(chair_turn,chair_target))|
        Placement {resolved,chair_turn,chair_target}).collect()
}
// Use the shared selected table and its strict material contract. Missing or
// unknown selected materials leave the entire original source drawing intact.
fn native_meeting_model() -> Option<&'static Model> {
    crate::interior_models::selected_model(crate::interior_models::ModelKind::DiningTable)
        .filter(|m| m.has_native_dining_table_materials())
}
/// Only the small original document ink face stays live on a physical page.
/// Pixel clipping splits at source-cell boundaries so UVs do not cross tiles.
fn live_panel(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &source::Match,
    pixels: [f32; 4],
    corners: [[f32; 3]; 4],
    normal: [f32; 3],
) {
    let [sx, sy, sw, sh] = pixels;
    let mix = |u: f32, v: f32| -> [f32; 3] {
        std::array::from_fn(|i| {
            corners[0][i] * (1. - u) * (1. - v)
                + corners[1][i] * u * (1. - v)
                + corners[2][i] * u * v
                + corners[3][i] * (1. - u) * v
        })
    };
    let mut y = sy;
    while y < sy + sh {
        let y1 = (sy + sh).min(((y / 8.).floor() + 1.) * 8.);
        let mut x = sx;
        while x < sx + sw {
            let x1 = (sx + sw).min(((x / 8.).floor() + 1.) * 8.);
            // Compute all UVs from this cell, including its far edge.
            let tx = (x / 8.).floor() as usize;
            let ty = (y / 8.).floor() as usize;
            let uv = g.uv(p.column + tx, p.row + ty);
            let u = |v: f32| uv.0 + (uv.1 - uv.0) * (v - tx as f32 * 8.) / 8.;
            let v = |v: f32| uv.2 + (uv.3 - uv.2) * (v - ty as f32 * 8.) / 8.;
            append_quad(
                &mut mesh.textured,
                [
                    mix((x - sx) / sw, (y1 - sy) / sh),
                    mix((x1 - sx) / sw, (y1 - sy) / sh),
                    mix((x1 - sx) / sw, (y - sy) / sh),
                    mix((x - sx) / sw, (y - sy) / sh),
                ],
                normal,
                [[u(x), v(y1)], [u(x1), v(y1)], [u(x1), v(y)], [u(x), v(y)]],
                TEXTURED_SHADE,
            );
            x = x1;
        }
        y = y1;
    }
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placement: &Placement,
    claimed: &mut [bool],
) -> bool {
    let p = &placement.resolved;
    if claimed.len() != cells.len() || p.indices(g.width).any(|i| i >= claimed.len() || claimed[i])
    {
        return false;
    }
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    if !p.coherent(&identities, g.width, g.height) {
        return false;
    }
    let native = if p.native_silph_meeting_table() {
        let Some(model) = native_meeting_model() else { return false; };
        Some(model)
    } else { None };
    if native.is_none() && !matches!(p.kind(),Kind::Sofa|Kind::ReceptionCounter)
        && !model(p.kind()).has_native_facility_furniture_materials() {return false;}
    let (west, _, north, _) = g.bounds(p.column, p.row);
    let sx = g.tile_width / 8.;
    let sy = g.tile_height / 8.;
    for i in p.indices(g.width) {
        let sample = p.ground_for(i % g.width, i / g.width);
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            g.uv(sample % g.width, sample / g.width),
        );
    }
    let ([w, e, n, s], rise) = p.kind().fitting();
    let bounds = [west + w * sx, west + e * sx, north + n * sy, north + s * sy];
    if p.kind() == Kind::ReceptionCounter {
        append_reception_counter(&mut mesh.textured,g,p);
    } else if p.kind() == Kind::Sofa {
        let first=mesh.textured.positions.len();
        append_sofa(&mut mesh.textured,g,p);
        // A foreground back must reveal the player as one complete object;
        // the already-appended native floor remains opaque and walkable.
        mesh.textured.cutaway_ranges.push(first..mesh.textured.positions.len());
    } else if let Some(model) = native {
        // Pixel(4,4) of native tile $51 is the plain gold field. The match's
        // complete 8x8 guard places that tile at local source cell(3,3).
        let uv = g.uv(p.column + 3, p.row + 3);
        let gold = [uv.0 + (uv.1-uv.0)*4.5/8., uv.2 + (uv.3-uv.2)*4.5/8.];
        assert!(model.append_native_dining_table(&mut mesh.textured, bounds, rise * sy, [gold;2]));
    } else {
        let first=mesh.textured.positions.len();
        assert!(model(p.kind()).append_native_facility_furniture(&mut mesh.textured,
            bounds,rise*sy,facility_swatches(g,p)));
        if p.kind()==Kind::Chair && placement.chair_target.as_ref()
            .is_some_and(|target|target.coherent(&identities,g.width,g.height)) {
            chair_facing::rotate(&mut mesh.textured,first,g,p,placement.chair_turn);
        }
    }
    if matches!(p.kind(), Kind::SquareTable | Kind::SideDesk) {
        let (source_y, paper_z) = if p.kind() == Kind::SquareTable {
            (17., 18.36)
        } else {
            (9., 14.36)
        };
        let point = |x: f32, z: f32| [west + x * sx, 9.835 * sy, north + z * sy];
        live_panel(
            mesh,
            g,
            p,
            [9., source_y, 6., 6.],
            [
                point(9.05, paper_z),
                point(15., paper_z),
                point(15., paper_z + 6.16),
                point(9.05, paper_z + 6.16),
            ],
            [0., 1., 0.],
        );
    }
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.kind().label());
        }
    }
    true
}

/// Closed chamfered upholstery pieces, sampled from the complete native sofa.
/// The sofa's source floor rows are guards only, never claimed or raised.
fn append_sofa(out:&mut SurfaceMeshData,g:&GridGeometry,p:&source::Match) {
    let (west,_,north,_)=g.bounds(p.column,p.row);
    let sx=g.tile_width/8.;let sy=g.tile_height/8.;
    let sample=|x:usize,y:usize| {
        let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8)as f32+0.5)/8.,
            uv.2+(uv.3-uv.2)*((y%8)as f32+0.5)/8.]
    };
    let blue=sample(12,12);let light=sample(11,3);let ink=sample(0,8);
    let mut piece=|b:[f32;4],bottom:f32,top:f32,r:f32,uv:[f32;2]| {
        let [w,e,n,s]=b;
        let ring=[[w+r,n],[w,n+r],[w,s-r],[w+r,s],
            [e-r,s],[e,s-r],[e,n+r],[e-r,n]];
        let pos=|a:[f32;2],y:f32|[west+a[0]*sx,y*sy,north+a[1]*sy];
        // Three rectangular strips partition the chamfered top and bottom;
        // the two end strips taper into the eight-sided perimeter.
        for (points,normal) in [
            ([ring[0],ring[1],ring[2],ring[3]], [0.,1.,0.]),
            ([ring[0],ring[3],ring[4],ring[7]], [0.,1.,0.]),
            ([ring[7],ring[4],ring[5],ring[6]], [0.,1.,0.]),
        ] {
            append_quad(out,points.map(|a|pos(a,top)),normal,[uv;4],TEXTURED_SHADE);
            append_quad(out,[points[3],points[2],points[1],points[0]].map(|a|pos(a,bottom)),
                [0.,-1.,0.],[uv;4],TEXTURED_SHADE);
        }
        for i in 0..8 {
            let a=ring[i];let b=ring[(i+1)%8];
            let normal=Vec3::new((a[1]-b[1])*sy,0.,(b[0]-a[0])*sx).normalize().to_array();
            append_quad(out,[pos(a,bottom),pos(b,bottom),pos(b,top),pos(a,top)],normal,[uv;4],TEXTURED_SHADE);
        }
    };
    for x in [3.,26.] {for z in [3.,12.] {
        piece([x,x+2.,z,z+2.],0.,3.,0.2,ink);
    }}
    piece([1.,31.,1.,15.],2.,5.5,0.8,blue);
    piece([2.,30.,1.,4.],5.,10.7,0.6,blue);
    piece([2.,30.,1.,4.],10.7,11.,0.6,light);
    piece([1.,4.,3.,15.],5.,8.5,0.6,blue);
    piece([28.,31.,3.,15.],5.,8.5,0.6,blue);
    for [w,e] in [[4.3,15.8],[16.2,27.7]] {
        piece([w,e,4.3,14.7],5.5,7.,0.5,blue);
    }
}

// Each sample belongs to this complete source drawing, never a neighboring
// floor or another object's palette. Coordinates denote pixel centers.
fn facility_swatches(g:&GridGeometry,p:&source::Match)->[[f32;2];4] {
    let pixels=match p.kind() {
        Kind::Chair=>[[5,4],[6,10],[4,3],[2,3]],
        Kind::SquareTable=>[[4,4],[4,25],[4,1],[4,24]],
        Kind::SideDesk=>[[4,4],[4,16],[4,1],[4,17]],
        Kind::MeetingTable=>[[20,20],[20,41],[20,17],[20,40]],
        _=>unreachable!("source-authenticated wood furniture"),
    };
    pixels.map(|[x,y]| {
        let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8)as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((y%8)as f32+0.5)/8.]
    })
}

/// Two joined closed courses form the native L without filling the clerk aisle.
fn append_reception_counter(out:&mut SurfaceMeshData,g:&GridGeometry,p:&source::Match) {
    let (west,_,north,_)=g.bounds(p.column,p.row);
    let sx=g.tile_width/8.;let sy=g.tile_height/8.;
    let uv=g.uv(p.column,p.row+6);
    let gold=[uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*4.5/8.];
    // Native tile77 supplies the dark lower fascia; tile65 supplies its light rim.
    // Sample the live atlas so palette changes remain authoritative.
    let lower=g.uv(p.column,p.row+7);
    let dark=[lower.0+(lower.1-lower.0)*4.5/8.,lower.2+(lower.3-lower.2)*1.5/8.];
    let ink=[lower.0+(lower.1-lower.0)*4.5/8.,lower.2+(lower.3-lower.2)*4.5/8.];
    let light=[uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*1.5/8.];
    for &[w,e,n,s] in chair_facing::RECEPTION_COURSES {
        let [w,e,n,s]=[west+w*sx,west+e*sx,north+n*sy,north+s*sy];
        let height=8.*sy;
        append_quad(out,[[w,height,n],[w,height,s],[e,height,s],[e,height,n]],
            [0.,1.,0.],[gold;4],TEXTURED_SHADE);
        append_quad(out,[[w,0.,n],[e,0.,n],[e,0.,s],[w,0.,s]],
            [0.,-1.,0.],[dark;4],TEXTURED_SHADE);
        for (bottom,top,paint) in [(0.,1.,ink),(1.,3.,dark),(3.,7.,gold),(7.,8.,light)] {
            let bottom=bottom*sy;let top=top*sy;
            for (points,normal) in [
                ([[w,bottom,s],[e,bottom,s],[e,top,s],[w,top,s]],[0.,0.,1.]),
                ([[e,bottom,n],[w,bottom,n],[w,top,n],[e,top,n]],[0.,0.,-1.]),
                ([[w,bottom,n],[w,bottom,s],[w,top,s],[w,top,n]],[-1.,0.,0.]),
                ([[e,bottom,s],[e,bottom,n],[e,top,n],[e,top,s]],[1.,0.,0.]),
            ] {
                append_quad(out,points,normal,[paint;4],TEXTURED_SHADE);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    include!("facility_tables_tests.rs");
    include!("facility_wall_sofa_tests.rs");
    include!("facility_chair_facing_tests.rs");
    include!("facility_native_finish_tests.rs");
}
