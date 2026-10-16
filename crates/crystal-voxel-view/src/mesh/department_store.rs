//! Complete source-aware store architecture and closed fifth-floor islands.
//! Sparse claims preserve live stairs, lift doors, thresholds, shop surfaces,
//! and the small genuine floor inset above the closed lower display.
use super::*;
use crate::interior_models::Model;
use crate::live_profiles::Document;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Asset {
    Window1,
    Window2,
    Window4,
    Window8,
    Window10,
    LiftSide,
    Directory,
    ClosedU,
    StairUp,
    StairDown,
    LiftPortal,
    LiftThreshold,
    CounterPlain,
    CounterRegister,
    CounterNorth,
    CounterWest,
}
pub(super) fn append_lift_cabin_glazing(out: &mut SurfaceMeshData, bounds: [f32;4], height: f32) {
    let [w,e,n,s] = bounds;
    for pane in 0..3 {
        let left = w+(e-w)*pane as f32/3.;
        let right = w+(e-w)*(pane+1) as f32/3.;
        Asset::Window1.model().append_fitted(out, [left,right,n,s], 0., height);
    }
}
impl Asset {
    fn model(self) -> &'static Model {
        if matches!(
            self,
            Self::CounterPlain | Self::CounterRegister | Self::CounterNorth | Self::CounterWest
        ) {
            return crate::interior_models::model(
                crate::interior_models::ModelKind::ReceptionCounter,
            );
        }
        if self == Self::LiftPortal {
            return crate::interior_models::model(crate::interior_models::ModelKind::GateDoorFrame);
        }
        assert!(
            self != Self::LiftThreshold,
            "threshold uses original Rust geometry"
        );
        if matches!(self, Self::StairUp | Self::StairDown) {
            return crate::interior_models::model(crate::interior_models::ModelKind::StairFlight);
        }
        static MODELS: OnceLock<[Model; 8]> = OnceLock::new();
        &MODELS.get_or_init(|| {
            [
                crate::model_storage::include_model!(
                    "models/department_store/window_course_1.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/window_course_2.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/window_course_4.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/window_course_8.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/window_course_10.mesh.json"
                ),
                crate::model_storage::include_model!("models/department_store/lift_side.mesh.json"),
                crate::model_storage::include_model!("models/department_store/directory.mesh.json"),
                crate::model_storage::include_model!(
                    "models/department_store/closed_u_display.mesh.json"
                ),
            ]
            .map(|s| Model::parse(s).expect("validated department store kit"))
        })[self as usize]
    }
}
struct Part {
    x: usize,
    y: usize,
    width: usize,
    depth: usize,
    asset: Asset,
}
fn counter_part(asset:Asset)->bool {
    matches!(asset,Asset::CounterPlain|Asset::CounterRegister|Asset::CounterNorth|Asset::CounterWest)
}
fn counter_joins(part:&Part,parts:&[Part])->[bool;4] {
    let mut joined=[false;4];
    for other in parts.iter().filter(|other|counter_part(other.asset)) {
        let spans_z=other.y<=part.y&&other.y+other.depth>=part.y+part.depth;
        let spans_x=other.x<=part.x&&other.x+other.width>=part.x+part.width;
        joined[0]|=spans_z&&other.x+other.width==part.x;
        joined[1]|=spans_z&&other.x==part.x+part.width;
        joined[2]|=spans_x&&other.y+other.depth==part.y;
        joined[3]|=spans_x&&other.y==part.y+part.depth;
    }
    joined
}
fn omit_counter_join_faces(out:&mut SurfaceMeshData,vertex_first:usize,index_first:usize,b:[f32;4],joined:[bool;4]) {
    // The cabinet body is inset beneath its overhanging cap. Extend that
    // hidden body edge to the shared plane before omitting its end wall, so
    // adjacent bays form one shell rather than leaving a slit under the top.
    for ((axis,plane,normal),joined) in [(0,b[0],-1.),(0,b[1],1.),(2,b[2],-1.),(2,b[3],1.)]
        .into_iter().zip(joined) {
        if !joined {continue;}
        let body=out.indices[index_first..].chunks_exact(3).filter(|tri|tri.iter().all(|&i|
            out.normals[i as usize][axis]*normal>0.999)).map(|tri| {
                let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(out.positions[i as usize]));
                ((b-a).cross(c-a).length_squared(),a[axis])
            }).max_by(|a,b|a.0.total_cmp(&b.0)).map(|(_,body)|body);
        if let Some(body)=body {
            for v in &mut out.positions[vertex_first..] {
                if (v[axis]-body)*normal>=-0.0001&&(plane-v[axis])*normal>=-0.0001 {
                    v[axis]=plane;
                }
            }
        }
    }
    let faces=out.indices.split_off(index_first);
    out.indices.extend(faces.chunks_exact(3).filter(|tri| {
        ![(0,b[0],-1.),(0,b[1],1.),(2,b[2],-1.),(2,b[3],1.)]
            .into_iter().zip(joined).any(|((axis,plane,normal),joined)|joined&&tri.iter().all(|&i| {
                let i=i as usize;
                (out.positions[i][axis]-plane).abs()<0.0001&&out.normals[i][axis]*normal>0.999
            }))
    }).flatten().copied());
}
struct Network {
    map: Option<&'static str>,
    floor: u8,
    anchor: [i32; 2],
    width: usize,
    height: usize,
    fingerprint: u64,
    rows: &'static [u32],
    parts: &'static [Part],
    label: &'static str,
}
include!("department_store_bindings.rs");
pub(super) struct Placement {
    network: &'static Network,
    column: usize,
    row: usize,
    ground: usize,
    grid_origin: [i32; 2],
    native_store: bool,
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        self.network
            .rows
            .iter()
            .enumerate()
            .flat_map(move |(y, bits)| {
                (0..self.network.width).filter_map(move |x| {
                    (bits & (1u32 << x) != 0).then_some((self.row + y) * width + self.column + x)
                })
            })
    }
}
fn floor(map: &str) -> Option<u8> {
    let floor = map
        .strip_prefix("CeladonDeptStore")
        .or_else(|| map.strip_prefix("GoldenrodDeptStore"))?;
    match floor {
        "1F" => Some(1),
        "2F" => Some(2),
        "3F" => Some(3),
        "4F" => Some(4),
        "5F" => Some(5),
        "6F" => Some(6),
        _ => None,
    }
}
fn identity_hash<'a>(sources: impl IntoIterator<Item = &'a VisualTileSource>) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for s in sources {
        for byte in [
            (s.metatile_id & 255) as u8,
            (s.metatile_id >> 8) as u8,
            s.subtile_column,
            s.subtile_row,
            (s.tile_index & 255) as u8,
            (s.tile_index >> 8) as u8,
        ] {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    hash
}
fn complete(
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    x: usize,
    y: usize,
    n: &Network,
) -> bool {
    if cells.len() != g.width * g.height
        || x + n.width > g.width
        || y + n.height > g.height
        || i64::from(origin[0]) + x as i64 != i64::from(n.anchor[0])
        || i64::from(origin[1]) + y as i64 != i64::from(n.anchor[1])
    {
        return false;
    }
    let sources = || {
        (0..n.height).flat_map(move |dy| {
            (0..n.width).map(move |dx| &cells[(y + dy) * g.width + x + dx].source)
        })
    };
    sources().all(|s| s.tileset_id.as_ref() == "mart") && identity_hash(sources()) == n.fingerprint
}
fn valid_ground(cell: &VisualTile) -> bool {
    let s = &cell.source;
    s.tileset_id.as_ref() == "mart"
        && s.metatile_id == 0x04
        && s.tile_index == 0x01
        && s.subtile_column < 4
        && s.subtile_row < 4
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    let Some(floor) = floor(map) else {
        return Vec::new();
    };
    resolve_networks(NETWORKS, floor, map, cells, g, origin, profiles, reserved)
}
pub(super) fn perimeter_template(mesh:&mut TerrainMeshData,map:&str,cells:&[&VisualTile],
    g:&GridGeometry,origin:[i32;2],profiles:Option<&Document>)->Option<usize> {
    let placements=resolve(map,cells,g,origin,profiles,&vec![false;cells.len()]);
    let (p,part)=placements.iter().flat_map(|p|p.network.parts.iter().map(move|part|(p,part)))
        .filter(|(_,part)|matches!(part.asset,Asset::Window1|Asset::Window2|Asset::Window4
            |Asset::Window8|Asset::Window10))
        .min_by_key(|(p,part)|(p.row+part.y,p.column+part.x))?;
    let row=p.row+part.y;let (w,_,n,_)=g.bounds(p.column+part.x,row);
    let s=n+part.depth as f32*g.tile_height;
    part.asset.model().append_fitted(&mut mesh.solid,
        [w,w+part.width as f32*g.tile_width,s-3.*g.tile_height/8.,s],0.,16.*g.tile_height/8.);
    Some(row)
}
fn resolve_networks(
    networks: &'static [Network],
    floor: u8,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if cells.len() != g.width * g.height || reserved.len() != cells.len() {
        return Vec::new();
    }
    let Some(ground) = cells.iter().position(|c| valid_ground(c)) else {
        return Vec::new();
    };
    // No live profile is silently displaced. The bundled Mart furniture lies
    // outside this kit's sparse masks; added or changed profiles win overlaps.
    let mut blocked = reserved.to_vec();
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let mut out = Vec::new();
    for n in networks.iter().filter(|n| n.floor == floor && n.map.is_none_or(|name|name == map)) {
        let x = i64::from(n.anchor[0]) - i64::from(origin[0]);
        let y = i64::from(n.anchor[1]) - i64::from(origin[1]);
        if x < 0 || y < 0 {
            continue;
        }
        let (column, row) = (x as usize, y as usize);
        if !complete(cells, g, origin, column, row, n) {
            continue;
        }
        let p = Placement {
            network: n,
            column,
            row,
            ground,
            grid_origin: origin,
            native_store: map.starts_with("CeladonDeptStore") || map.starts_with("GoldenrodDeptStore"),
        };
        if p.indices(g.width).any(|i| blocked[i]) {
            continue;
        }
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
        out.push(p);
    }
    out
}
// Tile rectangles remain live atlas samples, folded into the original housing.
fn inscription(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    source: [usize; 2],
    size: [usize; 2],
    bounds: [f32; 4],
    south: f32,
) {
    let [left, right, bottom, top] = bounds;
    for y in 0..size[1] {
        for x in 0..size[0] {
            let w = left + (right - left) * x as f32 / size[0] as f32;
            let e = left + (right - left) * (x + 1) as f32 / size[0] as f32;
            let hi = top - (top - bottom) * y as f32 / size[1] as f32;
            let lo = top - (top - bottom) * (y + 1) as f32 / size[1] as f32;
            let (u0, u1, v0, v1) = g.uv(source[0] + x, source[1] + y);
            append_quad(
                &mut mesh.textured,
                [
                    [w, lo, south],
                    [e, lo, south],
                    [e, hi, south],
                    [w, hi, south],
                ],
                [0., 0., 1.],
                [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
                TEXTURED_SHADE,
            );
        }
    }
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
) -> bool {
    if claimed.len() != cells.len()
        || p.ground >= cells.len()
        || !valid_ground(cells[p.ground])
        || !complete(cells, g, p.grid_origin, p.column, p.row, p.network)
        || p.indices(g.width).any(|i| claimed[i])
    {
        return false;
    }
    let counter_swatches=if p.native_store {
        p.indices(g.width).find_map(|i|super::modeled_interiors::mart_counter_swatches(g,i,cells[i].source.tile_index))
    }else {None};
    if p.native_store && p.network.parts.iter().any(|part|matches!(part.asset,
        Asset::CounterPlain|Asset::CounterRegister|Asset::CounterNorth|Asset::CounterWest))
        && (counter_swatches.is_none() || !Asset::CounterPlain.model().has_native_store_counter_materials()) {
        // Material failure must not re-enter a legacy 3D counter extrusion.
        // Preserve the complete sparse drawing at its original 2D datum.
        for i in p.indices(g.width) {
            append_top(&mut mesh.textured,g.bounds(i%g.width,i/g.width).into(),0.,g.uv(i%g.width,i/g.width));
            claimed[i]=true;
            if mesh.authored_cells.len()==cells.len() {mesh.authored_cells[i]=None;}
        }
        return true;
    }
    let uv = g.uv(
        cells[p.ground].column as usize,
        cells[p.ground].row as usize,
    );
    for i in p.indices(g.width) {
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        // Descending flights need a well below the room datum. A room-height
        // backing quad would hide all eight treads.
        let down = p
            .network
            .parts
            .iter()
            .any(|part| part.asset == Asset::StairDown);
        append_top(
            &mut mesh.textured,
            [w, e, n, s],
            if down { -16. * g.tile_height / 8. } else { 0. },
            uv,
        );
    }
    let cutaway_start = mesh.solid.positions.len();
    let textured_start = mesh.textured.positions.len();
    let sx = g.tile_width / 8.;
    let sy = g.tile_height / 8.;
    for part in p.network.parts {
        let (w, _, n, _) = g.bounds(p.column + part.x, p.row + part.y);
        let e = w + part.width as f32 * g.tile_width;
        let s = n + part.depth as f32 * g.tile_height;
        let is_display = part.asset == Asset::ClosedU;
        if matches!(
            part.asset,
            Asset::CounterPlain | Asset::CounterRegister | Asset::CounterNorth | Asset::CounterWest
        ) {
            let solid_start=mesh.solid.positions.len();
            let painted_start=mesh.textured.positions.len();
            let solid_indices=mesh.solid.indices.len();
            let painted_indices=mesh.textured.indices.len();
            if let Some(swatches)=counter_swatches {
                append_native_service_counter(&mut mesh.textured,part.asset,[w,e,n,s],sy,swatches);
            }else {append_service_counter(&mut mesh.solid, part.asset, [w, e, n, s], sy);}
            let joined=counter_joins(part,p.network.parts);
            omit_counter_join_faces(&mut mesh.solid,solid_start,solid_indices,[w,e,n,s],joined);
            omit_counter_join_faces(&mut mesh.textured,painted_start,painted_indices,[w,e,n,s],joined);
            if part.asset == Asset::CounterRegister {
                append_cash_register(&mut mesh.solid, [w, e, n, s], sy);
            }
            // Reveal only the bay blocking the trainer. The floor backing was
            // emitted before these ranges; other bays and the clerk's return
            // remain independently opaque as the trainer walks along the desk.
            let solid=solid_start..mesh.solid.positions.len();
            let painted=painted_start..mesh.textured.positions.len();
            if !solid.is_empty() {
                mesh.solid.cutaway_ranges.push(solid.clone());
                mesh.depth_coverage_solid_ranges.push(solid.clone());
            }
            if !painted.is_empty() {
                mesh.textured.cutaway_ranges.push(painted.clone());
                mesh.depth_coverage_textured_ranges.push(painted.clone());
            }
            if !solid.is_empty()&&!painted.is_empty() {
                mesh.cutaway_links.push((solid.start,painted.start));
            }
            continue;
        }
        if part.asset == Asset::LiftThreshold {
            append_lift_threshold(&mut mesh.solid, [w, e, n, s], sx, sy);
            continue;
        }
        if matches!(part.asset, Asset::StairUp | Asset::StairDown) {
            let down = part.asset == Asset::StairDown;
            // The shared editable flight ascends along X. Fit in its rotated
            // coordinate system so unequal native tile axes keep valid normals.
            let bounds = if down { [n, s, -e, -w] } else { [-s, -n, w, e] };
            let mut stairs = SurfaceMeshData::default();
            part.asset.model().append_fitted(
                &mut stairs,
                bounds,
                if down { -16. * sy } else { 0. },
                22.4 * sy,
            );
            for point in &mut stairs.positions {
                *point = if down {
                    [-point[2], point[1], point[0]]
                } else {
                    [point[2], point[1], -point[0]]
                };
            }
            for normal in &mut stairs.normals {
                *normal = if down {
                    [-normal[2], normal[1], normal[0]]
                } else {
                    [normal[2], normal[1], -normal[0]]
                };
            }
            let offset = mesh.solid.positions.len() as u32;
            mesh.solid
                .indices
                .extend(stairs.indices.into_iter().map(|i| i + offset));
            mesh.solid.positions.extend(stairs.positions);
            mesh.solid.normals.extend(stairs.normals);
            mesh.solid.uvs.extend(stairs.uvs);
            mesh.solid.colors.extend(stairs.colors);
            continue;
        }
        if is_display {
            let display_start=mesh.textured.positions.len();
            // All ten named materials sample this complete authenticated
            // drawing, including its distinct gray case and blue glass palettes.
            assert!(part.asset.model().append_store_display(
                &mut mesh.textured, [w, e, n, s], 8. * sy, |material| {
                    let pixel = match material {
                        0 | 3 => [1, 1], // Gray case and drawer faces, tile 2a.
                        1 => [9, 2], // Pale enamel, tile 3b.
                        2 | 7 | 8 => [0, 0], // Dark folded trim, pulls and plinth.
                        4 => [3, 54], // Blue glazing, tile 0e.
                        5 | 6 => [3, 49], // Pale reflection and glint, tile 0e.
                        9 => [0, 48], // Dark recessed glazing rim.
                        _ => unreachable!("validated named store display material"),
                    };
                    let uv = g.uv(p.column + pixel[0] / 8, p.row + pixel[1] / 8);
                    [uv.0 + (uv.1 - uv.0) * ((pixel[0] % 8) as f32 + 0.5) / 8.,
                     uv.2 + (uv.3 - uv.2) * ((pixel[1] % 8) as f32 + 0.5) / 8.]
                },
            ), "canonical store display requires all named materials");
            let glass=part.asset.model().take_store_display_glass(&mut mesh.textured,display_start)
                .expect("validated store glazing roles");
            mesh.scenery.push(crate::scenery_surfaces::ScenerySurface {
                mesh:glass,texture:crate::scenery_surfaces::SceneryTexture::glass_panel(),
            });
            continue;
        }
        let north = s - 3. * sy;
        part.asset.model().append_fitted(
            &mut mesh.solid,
            [w, e, north, s],
            0.,
            16. * sy,
        );
        // Window glass, reflections, frames and shadows carry explicit authored
        // material paint. Never infer their role from RGB channel comparisons.
        if part.asset == Asset::Directory {
            inscription(
                mesh,
                g,
                [p.column + part.x, p.row + part.y],
                [2, 2],
                [w + 1.5 * sx, e - 1.5 * sx, 1.5 * sy, 14.5 * sy],
                s + 0.015 * sy,
            );
        } else if part.asset == Asset::LiftSide {
            inscription(
                mesh,
                g,
                [p.column + part.x, p.row + part.y],
                [1, 1],
                [w + 1.5 * sx, w + 7. * sx, 8. * sy, 13.5 * sy],
                s + 0.015 * sy,
            );
        }
    }
    // Counters own individual bay ranges above; the closed display owns only
    // its front. Architecture keeps its existing whole-object reveal.
    let reveal = !matches!(
        p.network.label,
        "mart/department_store_counter_end"
            | "mart/department_store_service_loop"
            | "mart/department_store_closed_u"
    );
    if reveal {
        mesh.solid
            .cutaway_ranges
            .push(cutaway_start..mesh.solid.positions.len());
        if mesh.textured.positions.len() > textured_start {
            mesh.textured
                .cutaway_ranges
                .push(textured_start..mesh.textured.positions.len());
            mesh.cutaway_links.push((cutaway_start, textured_start));
        }
    }
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.network.label);
        }
    }
    true
}
fn append_service_counter(out: &mut SurfaceMeshData, asset: Asset, b: [f32; 4], sy: f32) {
    let [w, e, n, s] = b;
    // Fit in the rotated axes first: long return runs must keep the same
    // countertop depth and panel orientation as the shorter store counters.
    let bounds = match asset {
        Asset::CounterNorth => [-e, -w, -s, -n],
        Asset::CounterWest => [n, s, -e, -w],
        _ => b,
    };
    let first = out.positions.len();
    assert!(asset.model().append_store_counter_body(out,bounds,7.*sy),
        "canonical store counter requires all five named material roles");
    let rotate = |[x, y, z]: [f32; 3]| match asset {
        Asset::CounterNorth => [-x, y, -z],
        Asset::CounterWest => [-z, y, x],
        _ => [x, y, z],
    };
    for point in &mut out.positions[first..] {
        *point = rotate(*point);
    }
    for (normal, color) in out.normals[first..]
        .iter_mut()
        .zip(&mut out.colors[first..])
    {
        let old =
            crate::interior_models::authored_face_color([1.; 4], Vec3::from_array(*normal))[0];
        *normal = rotate(*normal);
        let new =
            crate::interior_models::authored_face_color([1.; 4], Vec3::from_array(*normal))[0];
        for channel in &mut color[..3] {
            *channel *= new / old;
        }
    }
    // Every bay shares this exact worktop height and perimeter. No separate
    // privacy lip turns across an adjoining top, and no rounded end leaves
    // a trench between two touching cabinet plots.
    super::modeled_interiors::architecture_box(out,[w,e,7.*sy,8.*sy,n,s],
        [0.76,0.75,0.65]);
}
fn append_native_service_counter(out:&mut SurfaceMeshData,asset:Asset,b:[f32;4],sy:f32,swatches:[[f32;2];5]) {
    let [w,e,n,s]=b;let bounds=match asset {Asset::CounterNorth=>[-e,-w,-s,-n],
        Asset::CounterWest=>[n,s,-e,-w],_=>b};
    let first=out.positions.len();
    assert!(asset.model().append_native_store_counter(out,bounds,7.*sy,swatches,true));
    let rotate=|[x,y,z]:[f32;3]|match asset {Asset::CounterNorth=>[-x,y,-z],Asset::CounterWest=>[-z,y,x],_=>[x,y,z]};
    for v in &mut out.positions[first..] {*v=rotate(*v);}
    for v in &mut out.normals[first..] {*v=rotate(*v);}
    for (normal,color) in out.normals[first..].iter().zip(&mut out.colors[first..]) {
        *color=crate::interior_models::authored_face_color([1.;4],Vec3::from_array(*normal));
    }
    let first=out.positions.len();super::modeled_interiors::architecture_box(out,[w,e,7.*sy,8.*sy,n,s],[1.;3]);
    for i in first..out.positions.len() {out.uvs[i]=swatches[if out.normals[i][1]>0.5 {3}else{4}];}
}
fn append_lift_threshold(out: &mut SurfaceMeshData, b: [f32; 4], sx: f32, sy: f32) {
    let [w, e, n, s] = b;
    let top = 0.14 * sy;
    let metal = [0.48, 0.55, 0.56, 1.0];
    for (points, normal) in [
        (
            [[w, top, n], [w, top, s], [e, top, s], [e, top, n]],
            [0., 1., 0.],
        ),
        (
            [[w, 0., n], [e, 0., n], [e, 0., s], [w, 0., s]],
            [0., -1., 0.],
        ),
        (
            [[e, 0., n], [w, 0., n], [w, top, n], [e, top, n]],
            [0., 0., -1.],
        ),
        (
            [[w, 0., s], [e, 0., s], [e, top, s], [w, top, s]],
            [0., 0., 1.],
        ),
        (
            [[w, 0., n], [w, 0., s], [w, top, s], [w, top, n]],
            [-1., 0., 0.],
        ),
        (
            [[e, 0., s], [e, 0., n], [e, top, n], [e, top, s]],
            [1., 0., 0.],
        ),
    ] {
        let color = crate::interior_models::authored_face_color(metal, Vec3::from_array(normal));
        append_solid_quad(out, points, normal, color);
    }
    // Recessed rail marks are paint on the thin tread, never raised obstacles.
    for row in 1..4 {
        let z = n + (s - n) * row as f32 / 4.;
        append_solid_quad(
            out,
            [
                [w + 0.7 * sx, top + 0.002 * sy, z],
                [w + 0.7 * sx, top + 0.002 * sy, z + 0.2 * sy],
                [e - 0.7 * sx, top + 0.002 * sy, z + 0.2 * sy],
                [e - 0.7 * sx, top + 0.002 * sy, z],
            ],
            [0., 1., 0.],
            crate::interior_models::authored_face_color([0.25, 0.31, 0.32, 1.], Vec3::Y),
        );
    }
}

// A mechanical register, with a cash drawer, sloping keypad and receipt hood.
// The west-facing controls face the clerk inside the U-shaped service counter.
fn append_cash_register(out: &mut SurfaceMeshData, b: [f32; 4], sy: f32) {
    let [w, e, n, s] = b;
    // A register occupies part of a counter bay, rather than nearly its full
    // width/depth. Keep the drawer seated on the same countertop datum.
    let point = |x: f32, y: f32, z: f32| [
        w + (e - w) * (8. + (x - 8.) * 0.65) / 16.,
        (8. + (y - 10.) * 0.70) * sy,
        n + (s - n) * (8. + (z - 8.) * 0.65) / 16.,
    ];
    let mut prism =
        |x0: f32, x1: f32, z0: f32, z1: f32, base: f32, lo: f32, hi: f32, color: [f32; 4]| {
            let v = [
                point(x0, base, z0),
                point(x1, base, z0),
                point(x1, base, z1),
                point(x0, base, z1),
                point(x0, lo, z0),
                point(x1, hi, z0),
                point(x1, hi, z1),
                point(x0, lo, z1),
            ];
            for ids in [
                [4, 7, 6, 5],
                [0, 1, 2, 3],
                [1, 0, 4, 5],
                [3, 2, 6, 7],
                [0, 3, 7, 4],
                [2, 1, 5, 6],
            ] {
                let q = ids.map(|i| v[i]);
                let normal = (Vec3::from_array(q[1]) - Vec3::from_array(q[0]))
                    .cross(Vec3::from_array(q[2]) - Vec3::from_array(q[0]))
                    .normalize();
                append_solid_quad(
                    out,
                    q,
                    normal.to_array(),
                    crate::interior_models::authored_face_color(color, normal),
                );
            }
        };
    prism(2., 14., 2., 14., 10., 10.9, 10.9, [0.25, 0.28, 0.29, 1.]);
    prism(
        2.6,
        13.4,
        2.6,
        13.4,
        10.9,
        11.25,
        13.3,
        [0.56, 0.60, 0.61, 1.],
    );
    // Actual raised keys sit on the slope; the drawer handle faces the clerk.
    for x in 0..3 {
        for z in 0..4 {
            let a = 4. + x as f32 * 1.9;
            let c = 3.4 + z as f32 * 2.25;
            let height = 11.25 + (a - 2.6) * 2.05 / 10.8;
            prism(
                a,
                a + 1.25,
                c,
                c + 1.5,
                height,
                height + 0.30,
                height + 0.30 + 1.25 * 2.05 / 10.8,
                if z == 3 {
                    [0.72, 0.27, 0.21, 1.]
                } else {
                    [0.16, 0.20, 0.22, 1.]
                },
            );
        }
    }
    prism(1.8, 2.3, 6., 10., 10.3, 10.6, 10.6, [0.77, 0.69, 0.44, 1.]);
    prism(
        10.5,
        13.2,
        4.,
        11.5,
        12.7,
        14.1,
        14.1,
        [0.28, 0.32, 0.33, 1.],
    );
    prism(
        10.15,
        10.6,
        4.8,
        10.7,
        13.9,
        14.45,
        14.45,
        [0.96, 0.87, 0.66, 1.],
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    include!("department_store_tests.rs");
}

#[cfg(test)]
mod native_counter_tests {
    use super::*;
    #[test]
    fn kanto_native_service_counter_rotations_keep_identical_geometry() {
        let swatches=std::array::from_fn(|i|[i as f32/8.,0.5]);
        for asset in [Asset::CounterPlain,Asset::CounterNorth,Asset::CounterWest] {
            let mut old=SurfaceMeshData::default();let mut painted=SurfaceMeshData::default();
            append_service_counter(&mut old,asset,[4.,20.,8.,24.],1.);
            append_native_service_counter(&mut painted,asset,[4.,20.,8.,24.],1.,swatches);
            assert_eq!(old.positions,painted.positions);assert_eq!(old.normals,painted.normals);
            assert_eq!(old.indices,painted.indices);assert!(painted.uvs.iter().all(|uv|swatches.contains(uv)));
        }
    }
}
