//! Complete native drawings fitted with original, editable 3D room geometry.
//! Fingerprints bind every immutable source identity in an entire object plot.
//! Rendering never reads or changes collision, warps, actors, or footing.
use super::*;
use crate::dungeon_models::{RoomAsset as Asset, room_model};
use crate::interior_models::{ModelKind as Interior, model as interior_model};
#[derive(Clone, Copy, Debug)]
pub(super) enum Surface {
    Stone,
    BlueBrick,
    Ceramic,
    RoofTile,
    StoreTile,
    ArcadeCeramic,
    Timber,
    Tatami,
    Facility,
    DojoNative,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Wall {
    Steel,
    Timber,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FloorStyle {
    Default,
    Roof,
    StoreLift,
    Dojo,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Detail {
    Asset(Asset),
    Interior(Interior),
    Surface(Surface),
    Wall(Wall),
    WallSegment { material: Wall, open: [bool; 4] },
    RoofRail,
    LivePlaque,
    Threshold,
    ArcadeEntryMat,
    FloorBand,
    ShutterTrack,
    Pit,
    Stairs,
    RoofInset,
    RoofSeat,
    RoofSaleBoxes { stacked: bool },
    StoreLiftGlazing,
    FacilityInstrumentTrack,
    FacilitySourceBarrier,
    RoofExit,
    ServiceCounter,
    ShrineAltar,
    Guardian,
    LiveFloorInlay,
    Arena,
    LinkPlatform { trade: bool },
    BikeBoundary,
    TowerLadder { down: bool },
}
impl Detail {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Asset(a) => a.label(),
            Self::Interior(Interior::VendingMachine) => "special-room:terrace-vending-machine",
            Self::Interior(Interior::StationBench) => "special-room:terrace-seat",
            Self::RoofSeat => "special-room:terrace-seat",
            Self::RoofSaleBoxes { .. } => "special-room:roof-sale-cartons",
            Self::StoreLiftGlazing => "special-room:store-lift-glazing",
            Self::Interior(Interior::GiftShelf) => "special-room:prize-display",
            Self::Interior(Interior::LabWorkstation) => "special-room:facility-workstation",
            Self::Interior(_) => "special-room:verified-furniture",
            Self::Surface(Surface::Stone) => "special-room:surface-stone",
            Self::Surface(Surface::RoofTile) => "special-room:roof-paving",
            Self::Surface(Surface::StoreTile) => "special-room:store-lift-paving",
            Self::Surface(Surface::BlueBrick) => "special-room:surface-blue-brick",
            Self::Surface(Surface::Ceramic) => "special-room:surface-ceramic",
            Self::Surface(Surface::ArcadeCeramic) => "special-room:arcade-ceramic",
            Self::Surface(Surface::Timber) => "special-room:surface-timber",
            Self::Surface(Surface::Tatami) => "special-room:surface-dojo-mat",
            Self::Surface(Surface::Facility) => "special-room:surface-facility",
            Self::Surface(Surface::DojoNative) => "special-room:surface-dojo-mat",
            Self::Wall(_) | Self::WallSegment { .. } => "special-room:connected-enclosure",
            Self::RoofRail => "special-room:terrace-parapet",
            Self::LivePlaque => "special-room:live-sign-in-frame",
            Self::Threshold => "special-room:surface-threshold",
            Self::ArcadeEntryMat => "special-room:arcade-entry-mat",
            Self::FloorBand => "special-room:surface-corridor-band",
            Self::ShutterTrack => "special-room:facility-shutter-track",
            Self::Pit => "special-room:open-puzzle-shaft",
            Self::Stairs => "special-room:stair-flight",
            Self::RoofInset => "special-room:roof-enclosed-inset",
            Self::FacilityInstrumentTrack => "special-room:facility-instrument-track",
            Self::FacilitySourceBarrier => "special-room:facility-source-barrier",
            Self::RoofExit => "special-room:roof-access-door",
            Self::ServiceCounter => "special-room:bike-service-counter",
            Self::ShrineAltar => "special-room:shrine-altar-screen",
            Self::Guardian => "special-room:dojo-guardian",
            Self::LiveFloorInlay => "special-room:surface-live-inlay",
            Self::Arena => "special-room:surface-tower-arena",
            Self::LinkPlatform { trade: false } => "special-room:colosseum-platform-apparatus",
            Self::LinkPlatform { trade: true } => "special-room:trade-platform-apparatus",
            Self::BikeBoundary => "special-room:source-clipped-bicycle-wheels",
            Self::TowerLadder { down: false } => "tower:timber-ladder-up",
            Self::TowerLadder { down: true } => "tower:timber-ladder-down",
        }
    }
}
struct Object {
    map: &'static str,
    tileset: &'static str,
    width: usize,
    height: usize,
    fingerprint: u64,
    detail: Detail,
    rise: f32,
    depth: f32,
}
struct CellRule {
    map: &'static str,
    tileset: &'static str,
    detail: Detail,
    fingerprints: &'static [u64],
}
include!("special_room_bindings.rs");
fn identity_hash<'a>(cells: impl IntoIterator<Item = &'a VisualTileSource>) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for s in cells {
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
fn matches_object(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    x: usize,
    y: usize,
    o: &Object,
) -> bool {
    if map != o.map || x + o.width > g.width || y + o.height > g.height {
        return false;
    }
    if matches!(o.detail, Detail::BikeBoundary) && y != 0 {
        return false;
    }
    let mut sources = Vec::with_capacity(o.width * o.height);
    for dy in 0..o.height {
        for dx in 0..o.width {
            let s = &cells[(y + dy) * g.width + x + dx].source;
            if s.tileset_id.as_ref() != o.tileset {
                return false;
            }
            sources.push(s);
        }
    }
    identity_hash(sources) == o.fingerprint
}
fn cell_detail(map: &str, s: &VisualTileSource) -> Option<Detail> {
    let h = identity_hash([s]);
    CELLS
        .iter()
        .find(|r| {
            r.map == map
                && r.tileset == s.tileset_id.as_ref()
                && r.fingerprints.binary_search(&h).is_ok()
        })
        .map(|r| r.detail)
}
fn add(
    r: &mut Resolver<'_>,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    ground: usize,
    detail: Detail,
    rise: f32,
    depth: f32,
) {
    r.add(Placement {
        column: x,
        row: y,
        width: w,
        height: h,
        ground,
        form: Form::Room(detail, match r.map {
            "GoldenrodDeptStoreRoof" => FloorStyle::Roof,
            "GoldenrodDeptStoreElevator" | "CeladonDeptStoreElevator" => FloorStyle::StoreLift,
            "FightingDojo" => FloorStyle::Dojo,
            _ => FloorStyle::Default,
        }),
        rise_pixels: rise,
        depth_pixels: depth,
        front_rows: h as f32,
    });
}
pub(super) fn resolve_into(r: &mut Resolver<'_>) {
    let (map, cells, g) = (r.map, r.cells, r.g);
    let Some(ground) = cells
        .iter()
        .position(|c| {
            matches!(cell_detail(map, &c.source), Some(Detail::Surface(_)))
                || (crate::casino::is_game_corner_map(map)
                    && crate::casino::is_floor_cell(&c.source))
        })
    else {
        return;
    };
    for o in OBJECTS.iter().filter(|o| o.map == map) {
        for y in 0..g.height {
            for x in 0..g.width {
                if matches_object(map, cells, g, x, y, o) {
                    add(
                        r, x, y, o.width, o.height, ground, o.detail, o.rise, o.depth,
                    );
                }
            }
        }
    }
    for y in 0..g.height {
        for x in 0..g.width {
            let i = y * g.width + x;
            if r.claimed[i] {
                continue;
            }
            let Some(mut detail) = cell_detail(map, &cells[i].source) else {
                continue;
            };
            if let Detail::Wall(material) = detail {
                let directions = [(0isize, -1isize), (1, 0), (0, 1), (-1, 0)];
                let open = directions.map(|(dx, dy)| {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    nx < 0
                        || ny < 0
                        || nx >= g.width as isize
                        || ny >= g.height as isize
                        || !matches!(
                            cell_detail(map, &cells[ny as usize * g.width + nx as usize].source),
                            Some(Detail::Wall(_))
                        )
                });
                detail = Detail::WallSegment { material, open };
            }
            add(r, x, y, 1, 1, ground, detail, 16., 8.);
        }
    }
}
fn slab(out: &mut SurfaceMeshData, b: [f32; 4], low: f32, high: f32, c: [f32; 4]) {
    let [w, e, n, s] = b;
    for (p, normal) in [
        (
            [[w, high, n], [w, high, s], [e, high, s], [e, high, n]],
            [0., 1., 0.],
        ),
        (
            [[w, low, n], [e, low, n], [e, low, s], [w, low, s]],
            [0., -1., 0.],
        ),
        (
            [[e, low, n], [w, low, n], [w, high, n], [e, high, n]],
            [0., 0., -1.],
        ),
        (
            [[w, low, s], [e, low, s], [e, high, s], [w, high, s]],
            [0., 0., 1.],
        ),
        (
            [[w, low, n], [w, low, s], [w, high, s], [w, high, n]],
            [-1., 0., 0.],
        ),
        (
            [[e, low, s], [e, low, n], [e, high, n], [e, high, s]],
            [1., 0., 0.],
        ),
    ] {
        faceted_face(out, p, normal, c, 0.);
    }
}
fn surface(out: &mut SurfaceMeshData, b: [f32; 4], kind: Surface, x: usize, y: usize, k: f32) {
    let [w, e, n, s] = b;
    if matches!(kind,Surface::BlueBrick) {
        // The fingerprinted Blackthorn floor keeps its blue brick pigment.
        // Mortar and brick faces partition the top at zero; the substrate
        // stays below the unchanged actor datum.
        let paint=|out:&mut SurfaceMeshData,b:[f32;4],color:[f32;3]| {
            let [w,e,n,s]=b;
            if e<=w || s<=n {return;}
            append_solid_quad(out,[[w,0.,n],[w,0.,s],[e,0.,s],[e,0.,n]],[0.,1.,0.],[color[0],color[1],color[2],1.]);
        };
        slab(out,b,-1.5*k,-0.20*k,[0.16,0.20,0.30,1.]);
        for course in 0..2 {
            let top=n+(s-n)*course as f32/2.;
            let bottom=n+(s-n)*(course+1) as f32/2.;
            let joint=if (y*2+course)%2==0 {w} else {(w+e)*0.5};
            let seam=0.22*k;
            let mortar=[0.64,0.73,0.88];
            paint(out,[w,e,top,top+seam],mortar);
            paint(out,[w,e,bottom-seam,bottom],mortar);
            let pigment=((x*3+y*5+course)%5) as f32*0.006;
            let brick=[0.17+pigment,0.28+pigment,0.66+pigment];
            paint(out,[w,joint,top+seam,bottom-seam],brick);
            paint(out,[joint+seam,e,top+seam,bottom-seam],brick);
            paint(out,[joint,joint+seam,top+seam,bottom-seam],mortar);
        }
        return;
    }
    let c = match kind {
        Surface::BlueBrick | Surface::ArcadeCeramic => unreachable!("handled before generic paving"),
        Surface::Stone => [0.55, 0.57, 0.50, 1.],
        Surface::RoofTile | Surface::StoreTile | Surface::DojoNative => unreachable!("owned paving uses its verified source atlas"),
        Surface::Ceramic => {
            if (x + y) % 2 == 0 {
                [0.72, 0.74, 0.67, 1.]
            } else {
                [0.63, 0.67, 0.64, 1.]
            }
        }
        Surface::Timber => [0.40, 0.25, 0.13, 1.],
        Surface::Tatami => [0.62, 0.58, 0.36, 1.],
        Surface::Facility => {
            if (x + y) % 2 == 0 {
                [0.70, 0.73, 0.71, 1.]
            } else {
                [0.47, 0.55, 0.55, 1.]
            }
        }
    };
    slab(out, b, -1.5 * k, -0.20 * k, [0.22, 0.27, 0.27, 1.]);
    let gap = 0.12 * k;
    slab(out, [w + gap, e - gap, n + gap, s - gap], -0.20 * k, 0., c);
    if matches!(kind, Surface::Timber | Surface::Tatami) {
        let color = if matches!(kind, Surface::Timber) {
            [0.25, 0.17, 0.10, 1.]
        } else {
            [0.38, 0.42, 0.24, 1.]
        };
        for j in 1..4 {
            let z = n + (s - n) * j as f32 / 4.;
            slab(
                out,
                [w + gap, e - gap, z - 0.035 * k, z + 0.035 * k],
                -0.03 * k,
                0.001 * k,
                color,
            );
        }
    }
}
fn ring(
    out: &mut SurfaceMeshData,
    c: [f32; 2],
    r: [f32; 2],
    inner: f32,
    low: f32,
    high: f32,
    color: [f32; 4],
    sides: usize,
) {
    for i in 0..sides {
        let a = i as f32 * std::f32::consts::TAU / sides as f32;
        let b = (i + 1) as f32 * std::f32::consts::TAU / sides as f32;
        let p = |a: f32, t: f32, h: f32| [c[0] + a.cos() * r[0] * t, h, c[1] + a.sin() * r[1] * t];
        faceted_face(
            out,
            [
                p(a, inner, high),
                p(b, inner, high),
                p(b, 1., high),
                p(a, 1., high),
            ],
            [0., 1., 0.],
            color,
            0.,
        );
        let normal = [((a + b) * 0.5).cos(), 0., ((a + b) * 0.5).sin()];
        faceted_face(
            out,
            [p(a, 1., low), p(b, 1., low), p(b, 1., high), p(a, 1., high)],
            normal,
            color,
            0.,
        );
    }
}
fn octagon(out: &mut SurfaceMeshData, b: [f32; 4], low: f32, high: f32, color: [f32; 4]) {
    let [w, e, n, s] = b;
    let dx = (e - w) * 0.23;
    let dz = (s - n) * 0.23;
    let ps = [
        [w + dx, n],
        [e - dx, n],
        [e, n + dz],
        [e, s - dz],
        [e - dx, s],
        [w + dx, s],
        [w, s - dz],
        [w, n + dz],
    ];
    let c = [(w + e) * 0.5, high, (n + s) * 0.5];
    for i in 0..8 {
        let p = ps[i];
        let q = ps[(i + 1) % 8];
        // The top triangle is non-degenerate; duplicate its center only in the quad input.
        faceted_face(
            out,
            [[p[0], high, p[1]], c, c, [q[0], high, q[1]]],
            [0., 1., 0.],
            color,
            0.,
        );
        let n = Vec3::new(q[1] - p[1], 0., p[0] - q[0])
            .normalize()
            .to_array();
        faceted_face(
            out,
            [
                [p[0], low, p[1]],
                [q[0], low, q[1]],
                [q[0], high, q[1]],
                [p[0], high, p[1]],
            ],
            n,
            color,
            0.,
        );
    }
}
fn underlay(mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement, kind: Surface, cells: &[&VisualTile]) {
    for i in p.indices(g.width) {
        if matches!(kind,Surface::DojoNative) {
            let sample=if matches!(p.form,Form::Room(Detail::Surface(_),FloorStyle::Dojo)) {i} else {p.ground};
            let source=&cells[sample].source;
            assert!(source.tileset_id.as_ref()=="train_station" && source.tile_index==0x56,
                "Dojo floor requires its authenticated timber drawing");
            let bounds=g.bounds(i%g.width,i/g.width);
            super::modeled_interiors::native_beveled_floor_cell(&mut mesh.textured,bounds.into(),0.,
                [i32::from(cells[i].source.subtile_column),i32::from(cells[i].source.subtile_row)],
                g.uv(sample%g.width,sample/g.width));
            continue;
        }
        if matches!(kind, Surface::RoofTile | Surface::StoreTile) {
            let bounds = g.bounds(i % g.width, i / g.width);
            let first = mesh.solid.indices.len();
            slab(&mut mesh.solid, bounds.into(), -1.5 * g.tile_height / SOURCE_TILE_HEIGHT,
                0., [0.25, 0.25, 0.25, 1.]);
            let rest = mesh.solid.indices.split_off(first);
            mesh.solid.indices.extend(rest.chunks_exact(3)
                .filter(|tri| mesh.solid.normals[tri[0] as usize][1] < 0.5)
                .flatten().copied());
            let sample = if matches!(p.form, Form::Room(Detail::Surface(_), FloorStyle::Roof | FloorStyle::StoreLift)) { i } else { p.ground };
            append_top(&mut mesh.textured, bounds.into(), 0.,
                g.uv(sample % g.width, sample / g.width));
            continue;
        }
        if matches!(kind, Surface::ArcadeCeramic) {
            let bounds = g.bounds(i % g.width, i / g.width);
            let first = mesh.solid.indices.len();
            slab(&mut mesh.solid, bounds.into(), -1.5 * g.tile_height / SOURCE_TILE_HEIGHT, 0., [0.28,0.29,0.27,1.]);
            // Replace the body's faceted lid with one coplanar partition of
            // native tile paint. Keep every side and the sealed underside.
            let rest = mesh.solid.indices.split_off(first);
            mesh.solid.indices.extend(rest.chunks_exact(3).filter(|tri| mesh.solid.normals[tri[0] as usize][1] < 0.5).flatten().copied());
            let source = &cells[i].source;
            super::modeled_interiors::arcade_floor_cell(&mut mesh.textured, bounds.into(),
                [i32::from(source.subtile_column),i32::from(source.subtile_row)],
                g.uv(i%g.width,i/g.width));
            continue;
        }
        surface(
            &mut mesh.solid,
            g.bounds(i % g.width, i / g.width).into(),
            kind,
            i % g.width,
            i / g.width,
            g.tile_height / SOURCE_TILE_HEIGHT,
        );
    }
}
fn live_face(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    b: [f32; 4],
    base: f32,
    height: f32,
) {
    let [w, e, _, s] = b;
    let tw = (e - w) / p.width as f32;
    let th = height / p.height as f32;
    for y in 0..p.height {
        for x in 0..p.width {
            let uv = g.uv(p.column + x, p.row + y);
            let x0 = w + x as f32 * tw;
            let x1 = x0 + tw;
            let y1 = base + height - y as f32 * th;
            let y0 = y1 - th;
            append_quad(
                &mut mesh.textured,
                [[x1, y0, s], [x1, y1, s], [x0, y1, s], [x0, y0, s]],
                [0., 0., 1.],
                [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                TEXTURED_SHADE,
            );
        }
    }
}
// Keep unresolved architectural drawings on shallow geometry with their live
// source UVs. Clip by source cell so the pattern is neither stretched nor copied
// into a model asset. The solid top sits below its textured face, avoiding Z-fighting.
fn source_texel_uv(g: &GridGeometry, p: &Placement, pixel: [usize; 2]) -> [f32; 2] {
    let uv = g.uv(p.column + pixel[0] / 8, p.row + pixel[1] / 8);
    [
        uv.0 + (uv.1 - uv.0) * ((pixel[0] % 8) as f32 + 0.5) / 8.,
        uv.2 + (uv.3 - uv.2) * ((pixel[1] % 8) as f32 + 0.5) / 8.,
    ]
}
fn sampled_room_asset(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    kind: Asset,
    source_offset: [usize; 2],
    bounds: [f32; 4],
    base: f32,
    height: f32,
) {
    room_model(kind).append_source_sampled(&mut mesh.textured, bounds, base, height, |pixel| {
        source_texel_uv(
            g,
            p,
            [
                source_offset[0] + pixel[0] as usize,
                source_offset[1] + pixel[1] as usize,
            ],
        )
    });
}
// Cartons are authored geometry, painted with texels from the complete sale
// drawing. There is no generic crate material or unverified room fallback.
fn roof_sale_carton(
    mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement,
    bounds: [f32; 4], base: f32, height: f32, stacked: bool,
) {
    let k = g.tile_height / SOURCE_TILE_HEIGHT;
    let body = if stacked { [2, 10] } else { [2, 9] };
    let lid = [1, 2];
    let seam = [0, 0];
    let label = [3, 2];
    let mut paint = |b: [f32; 4], low: f32, high: f32, pixel| {
        let mut part = SurfaceMeshData::default();
        slab(&mut part, b, low, high, [1.; 4]);
        let uv = source_texel_uv(g, p, pixel);
        let start = mesh.textured.positions.len() as u32;
        for (position, normal) in part.positions.into_iter().zip(part.normals) {
            mesh.textured.positions.push(position);
            mesh.textured.normals.push(normal);
            mesh.textured.uvs.push(uv);
            mesh.textured.colors.push(crate::interior_models::authored_face_color([1.;4], Vec3::from_array(normal)));
        }
        mesh.textured.indices.extend(part.indices.into_iter().map(|i| i + start));
    };
    let [w,e,n,s] = bounds;
    paint(bounds, base, base + height, body);
    // Raised folded lid and its center seam, plus a small front price label.
    paint([w,e,n,s], base + height, base + height + 0.35*k, lid);
    let mid = (w+e)*0.5;
    paint([mid-0.18*k,mid+0.18*k,n,s], base+height+0.35*k,base+height+0.40*k,seam);
    paint([w+1.8*k,e-1.8*k,s-0.15*k,s],base+height*0.55,base+height*0.82,label);
    paint([w+3.2*k,e-3.2*k,s-0.10*k,s+0.02*k],base+height*0.65,base+height*0.70,seam);
}
fn live_source_slab(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    b: [f32; 4],
    low: f32,
    high: f32,
) {
    let k = g.tile_height / SOURCE_TILE_HEIGHT;
    let first = mesh.textured.uvs.len();
    slab(&mut mesh.textured, b, low, high - 0.015 * k, TEXTURED_SHADE);
    // The shallow side/underside uses an actual source border texel too.
    mesh.textured.uvs[first..].fill(source_texel_uv(g, p, [0, 0]));
    for i in p.indices(g.width) {
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        let clipped = [w.max(b[0]), e.min(b[1]), n.max(b[2]), s.min(b[3])];
        if clipped[0] >= clipped[1] || clipped[2] >= clipped[3] {
            continue;
        }
        let uv = g.uv(i % g.width, i / g.width);
        let u = |x: f32| uv.0 + (uv.1 - uv.0) * (x - w) / (e - w);
        let v = |z: f32| uv.2 + (uv.3 - uv.2) * (z - n) / (s - n);
        append_top(
            &mut mesh.textured,
            clipped,
            high,
            (u(clipped[0]), u(clipped[1]), v(clipped[2]), v(clipped[3])),
        );
    }
}

fn dojo_source_wall(mesh: &mut TerrainMeshData,g: &GridGeometry,p: &Placement,b:[f32;4]) {
    let [w,e,n,s]=b;let k=g.tile_height/SOURCE_TILE_HEIGHT;let height=16.*k;
    let out=&mut mesh.textured;let first=out.positions.len();let index=out.indices.len();
    slab(out,b,0.,height,TEXTURED_SHADE);
    out.uvs[first..].fill(source_texel_uv(g,p,[0,0]));
    // Keep the sealed cap/base; each vertical side carries two native brick
    // courses, with no generic timber rails or coplanar painted faces.
    let faces=out.indices.split_off(index);
    out.indices.extend(faces.chunks_exact(3).filter(|tri|
        out.normals[tri[0]as usize][1].abs()>0.5).flatten().copied());
    let uv=g.uv(p.column,p.row);
    for (a,b,normal) in [([w,s],[e,s],Vec3::Z),([e,n],[w,n],-Vec3::Z),
        ([w,n],[w,s],-Vec3::X),([e,s],[e,n],Vec3::X)] {
        for row in 0..2 {
            let bottom=row as f32*8.*k;let top=bottom+8.*k;
            append_quad(out,[[a[0],bottom,a[1]],[b[0],bottom,b[1]],
                [b[0],top,b[1]],[a[0],top,a[1]]],normal.to_array(),
                [[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],
                crate::interior_models::authored_face_color([1.;4],normal));
        }
    }
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    detail: Detail,
) {
    append_with_floor(mesh, g, p, cells, detail, FloorStyle::Default);
}
pub(super) fn append_roof(
    mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement,
    cells: &[&VisualTile], detail: Detail,
) {
    append_with_floor(mesh, g, p, cells, detail, FloorStyle::Roof);
}
pub(super) fn append_with_floor(
    mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement,
    cells: &[&VisualTile], detail: Detail, floor_style: FloorStyle,
) {
    let roof_tiles = floor_style == FloorStyle::Roof;
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + p.width as f32 * g.tile_width;
    let s = n + p.height as f32 * g.tile_height;
    let k = g.tile_height / SOURCE_TILE_HEIGHT;
    let rise = p.rise_pixels * k;
    let depth = p.depth_pixels * k;
    let b = [w, e, n, s];
    let room_floor = if floor_style==FloorStyle::Dojo {Surface::DojoNative}
    else if roof_tiles { Surface::RoofTile } else if floor_style == FloorStyle::StoreLift {
        Surface::StoreTile
    } else { match cells[p.ground].source.tileset_id.as_ref() {
        "lab" => Surface::Timber,
        "tower" => Surface::Timber,
        "train_station" => Surface::Tatami,
        "facility" => Surface::Facility,
        "elite_four_room" => Surface::Stone,
        _ => Surface::Ceramic,
    }};
    if !matches!(
        detail,
        Detail::Pit
            | Detail::Stairs
            | Detail::RoofInset
            | Detail::FacilityInstrumentTrack
            | Detail::FacilitySourceBarrier
            | Detail::Surface(_)
            | Detail::Threshold
            | Detail::ArcadeEntryMat
            | Detail::FloorBand
            | Detail::ShutterTrack
            | Detail::TowerLadder { .. }
    ) {
        underlay(mesh, g, p, room_floor, cells);
    }
    match detail {
        Detail::RoofSaleBoxes { stacked } => {
            let cartons: &[[f32; 6]] = if stacked {
                &[
                    [1.,12.,1.,14.,0.,18.],
                    [13.,23.,1.,14.,0.,26.],
                    [1.,11.,15.,30.,0.,10.],
                    [12.,23.,16.,31.,0.,9.],
                    [2.,10.,16.,28.,10.4,8.],
                ]
            } else {
                &[[1.,12.,1.,15.,0.,11.], [13.,23.,1.,15.,0.,15.]]
            };
            for &[x0,x1,z0,z1,y,h] in cartons {
                roof_sale_carton(mesh,g,p,[w+x0*k,w+x1*k,n+z0*k,n+z1*k],y*k,h*k,stacked);
            }
        }
        Detail::TowerLadder { down } => {
            let base = if down { -16. * k } else { 0. };
            let top = if down { 0. } else { 24. * k };
            let timber = [0.43, 0.29, 0.16, 1.];
            let edge = [0.30, 0.19, 0.10, 1.];
            if down {
                // Open well: no room-height backing can conceal the rungs.
                slab(&mut mesh.solid, b, base - k, base, [0.13, 0.10, 0.07, 1.]);
                for frame in [
                    [w, w + k, n, s],
                    [e - k, e, n, s],
                    [w + k, e - k, n, n + k],
                    [w + k, e - k, s - k, s],
                ] {
                    slab(&mut mesh.solid, frame, base, 0., edge);
                }
            } else {
                // Reuse the same native backing sample as the room. The late
                // floor pass gives it the continuous map-space plank finish.
                let uv = g.uv(
                    cells[p.ground].column as usize,
                    cells[p.ground].row as usize,
                );
                for i in p.indices(g.width) {
                    append_top(
                        &mut mesh.textured,
                        g.bounds(i % g.width, i / g.width).into(),
                        0.,
                        uv,
                    );
                }
            }
            let z = n + 3. * k;
            for x in [w + 2. * k, e - 3. * k] {
                slab(&mut mesh.solid, [x, x + k, z, z + k], base, top + k, edge);
            }
            for i in 0..=6 {
                let y = base + (top - base) * i as f32 / 6.;
                slab(
                    &mut mesh.solid,
                    [w + 2. * k, e - 2. * k, z - 0.35 * k, z + 1.35 * k],
                    y,
                    y + 0.75 * k,
                    timber,
                );
            }
        }
        Detail::Asset(Asset::RoofPlanter) => {
            sampled_room_asset(mesh, g, p, Asset::RoofPlanter, [0, 0],
                [w, e, s - depth, s], 0., rise);
        }
        Detail::Asset(Asset::RoofBinoculars) => {
            // The complete 2x2 source drawing also owns the east parapet strip.
            // Preserve it and fit the west-facing viewer in its observed span.
            sampled_room_asset(
                mesh,
                g,
                p,
                Asset::RoofBinoculars,
                [0, 0],
                [w + k, e - 4. * k, s - 12. * k, s - k],
                0.,
                rise,
            );
            live_source_slab(mesh, g, p, [e - 3. * k, e, n, s], 0., 8. * k);
        }
        Detail::Asset(Asset::FacilityInstrumentBank) => {
            sampled_room_asset(
                mesh,
                g,
                p,
                Asset::FacilityInstrumentBank,
                [0, 0],
                [w, e, s - depth, s],
                0.,
                rise,
            );
        }
        Detail::Asset(Asset::GymStonePartition) => {
            sampled_room_asset(mesh,g,p,Asset::GymStonePartition,[0,0],
                [w,e,s-depth,s],0.,rise);
        }
        Detail::Asset(a) => room_model(a).append(&mut mesh.solid, [w, e, s - depth, s], 0., rise),
        Detail::RoofSeat => {
            let first = mesh.solid.positions.len();
            let model = interior_model(Interior::StationBench);
            model.append_fitted(&mut mesh.solid, [w, e, s - depth, s], 0., rise);
            model.apply_roof_seat_palette(&mut mesh.solid, first);
        }
        Detail::StoreLiftGlazing => {
            super::super::department_store::append_lift_cabin_glazing(&mut mesh.solid,
                [w,e,s-3.*k,s], rise);
        }
        Detail::Interior(a) => {
            interior_model(a).append_fitted(&mut mesh.solid, [w, e, s - depth, s], 0., rise)
        }
        Detail::Guardian => {
            if floor_style==FloorStyle::Dojo {
                model(Kind::TowerGuardian).append_pewter_guardian(&mut mesh.textured,
                    [w,e,s-depth,s],rise,|pixel|source_texel_uv(g,p,pixel));
            } else {
                model(Kind::TowerGuardian).append(&mut mesh.solid, [w, e, s - depth, s], 0., rise)
            }
        }
        Detail::Surface(kind) => underlay(mesh, g, p,
            if matches!(kind,Surface::Tatami) && floor_style==FloorStyle::Dojo {Surface::DojoNative}
            else if matches!(kind, Surface::Ceramic) && roof_tiles { Surface::RoofTile }
            else if matches!(kind, Surface::Ceramic) && floor_style == FloorStyle::StoreLift {
                Surface::StoreTile
            } else { kind }, cells),
        Detail::Wall(material) | Detail::WallSegment { material, .. } => {
            if floor_style==FloorStyle::Dojo {
                dojo_source_wall(mesh,g,p,b);
                return;
            }
            let open = if let Detail::WallSegment { open, .. } = detail {
                open
            } else {
                [true; 4]
            };
            let c = match material {
                Wall::Steel => [0.52, 0.60, 0.60, 1.],
                Wall::Timber => [0.40, 0.26, 0.14, 1.],
            };
            let trim = match material {
                Wall::Steel => [0.27, 0.36, 0.39, 1.],
                Wall::Timber => [0.25, 0.15, 0.09, 1.],
            };
            slab(&mut mesh.solid, b, 0., 14. * k, c);
            slab(&mut mesh.solid, b, 14. * k, 16. * k, [0.72, 0.72, 0.63, 1.]);
            for (j, face) in [
                [w, e, n, n + 0.28 * k],
                [e - 0.28 * k, e, n, s],
                [w, e, s - 0.28 * k, s],
                [w, w + 0.28 * k, n, s],
            ]
            .into_iter()
            .enumerate()
            {
                if open[j] {
                    slab(&mut mesh.solid, face, 0., 1.7 * k, trim);
                    slab(&mut mesh.solid, face, 10. * k, 11. * k, trim);
                }
            }
        }
        Detail::RoofRail => {
            // Only the verified Goldenrod terrace owns the native gray metal.
            let (body, cap, rib) = if roof_tiles {
                ([0.147027,0.147027,0.147027,1.],
                 [0.332452,0.332452,0.332452,1.],
                 [0.040915,0.040915,0.040915,1.])
            } else {
                ([0.40,0.48,0.49,1.], [0.69,0.73,0.68,1.], [0.27,0.35,0.35,1.])
            };
            slab(&mut mesh.solid, b, 0., 7. * k, body);
            slab(&mut mesh.solid, b, 7. * k, 8. * k, cap);
            for j in 0..3 {
                let x = w + (e - w) * (j as f32 + 0.5) / 3.;
                slab(
                    &mut mesh.solid,
                    [x - 0.18 * k, x + 0.18 * k, s - 0.35 * k, s + 0.01 * k],
                    1. * k,
                    6.5 * k,
                    rib,
                );
            }
        }
        Detail::LivePlaque => {
            interior_model(Interior::PictureFrame).append_fitted(
                &mut mesh.solid,
                [w, e, s - 3. * k, s],
                3. * k,
                rise - 3. * k,
            );
            live_face(
                mesh,
                g,
                p,
                [w + 0.6 * k, e - 0.6 * k, n, s + 0.015 * k],
                4. * k,
                rise - 5. * k,
            );
        }
        Detail::ArcadeEntryMat => {
            // Complete 32x16 native cloth drawing, with its pink selvedge at
            // both ends. Each closed strip owns one partition of the floor;
            // the mat stays at the original actor/warp datum.
            let red = [0.90, 0.06, 0.02, 1.];
            let pink = [0.95, 0.20, 0.16, 1.];
            let light = [0.98, 0.36, 0.34, 1.];
            for row in 0..16 {
                let color = match row { 0 | 15 => light, 2 | 13 => pink, _ => red };
                let za = n + (s-n) * row as f32 / 16.;
                let zb = n + (s-n) * (row+1) as f32 / 16.;
                let first = mesh.solid.positions.len();
                slab(&mut mesh.solid, [w,e,za,zb], -0.4*k, 0., color);
                for i in first..mesh.solid.positions.len() {
                    let point = mesh.solid.positions[i];
                    mesh.solid.uvs[i] = [82000. + (point[0]-w)/(e-w)*16.,
                        2500. + (point[2]-n)/(s-n)*16.];
                }
            }
        }
        Detail::Threshold | Detail::FloorBand if floor_style==FloorStyle::Dojo => {
            live_source_slab(mesh,g,p,b,-k,0.);
        }
        Detail::Threshold | Detail::FloorBand if floor_style == FloorStyle::StoreLift => {
            let mut source_floor = p.clone();
            source_floor.form = Form::Room(Detail::Surface(Surface::StoreTile), FloorStyle::StoreLift);
            underlay(mesh, g, &source_floor, Surface::StoreTile, cells);
        }
        Detail::Threshold | Detail::FloorBand => {
            // This full-plot slab owns the floor; a ceramic underlay would
            // place another upward-facing surface at exactly the same height.
            slab(&mut mesh.solid, b, -1. * k, 0., [0.25, 0.29, 0.30, 1.]);
            for j in 0..4 {
                let z = n + (s - n) * (j as f32 + 0.5) / 4.;
                slab(
                    &mut mesh.solid,
                    [w + 0.2 * k, e - 0.2 * k, z - 0.15 * k, z + 0.15 * k],
                    -0.05 * k,
                    0.006 * k,
                    [0.62, 0.68, 0.65, 1.],
                );
            }
        }
        Detail::ShutterTrack => {
            slab(&mut mesh.solid, b, -0.8 * k, 0., [0.35, 0.42, 0.42, 1.]);
            for j in 0..4 {
                let z = n + (s - n) * (j as f32 + 0.5) / 4.;
                slab(
                    &mut mesh.solid,
                    [w, e, z - 0.15 * k, z + 0.15 * k],
                    -0.03 * k,
                    0.01 * k,
                    [0.73, 0.77, 0.72, 1.],
                );
            }
        }
        Detail::Pit => {
            // Hollow shaft: no surface spans the aperture and actor datum is untouched.
            let t = 0.6 * k;
            for edge in [
                [w, w + t, n, s],
                [e - t, e, n, s],
                [w + t, e - t, n, n + t],
                [w + t, e - t, s - t, s],
            ] {
                slab(&mut mesh.solid, edge, -10. * k, 0., [0.36, 0.38, 0.33, 1.]);
            }
            slab(
                &mut mesh.solid,
                [w + t, e - t, n + t, s - t],
                -10.1 * k,
                -10. * k,
                [0.055, 0.06, 0.06, 1.],
            );
        }
        Detail::Stairs => {
            for j in 0..5 {
                let z0 = n + (s - n) * j as f32 / 5.;
                let z1 = n + (s - n) * (j + 1) as f32 / 5.;
                let y = -(j as f32) * 1.4 * k;
                if roof_tiles {
                    // The two fingerprinted roof flights keep their gray/pale
                    // native drawing across the complete closed step footprint.
                    live_source_slab(mesh,g,p,[w,e,z0,z1],-8.*k,y);
                } else {
                    slab(
                        &mut mesh.solid,
                        [w + 0.4 * k, e - 0.4 * k, z0, z1],
                        -8. * k,
                        y,
                        [0.60, 0.63, 0.56, 1.],
                    );
                }
            }
        }
        Detail::RoofInset => {
            // The source establishes an enclosed patterned inset, not stairs.
            // Keep the original complete drawing and only its observed border
            // relief. No stair flight, shaft, doorway or function is inferred.
            let t = 2. * k;
            live_source_slab(mesh, g, p, [w + t, e - t, n + t, s - t], -k, 0.015 * k);
            for edge in [
                [w, w + t, n, s],
                [e - t, e, n, s],
                [w + t, e - t, n, n + t],
                [w + t, e - t, s - t, s],
            ] {
                live_source_slab(mesh, g, p, edge, -k, 3. * k);
            }
        }
        Detail::FacilityInstrumentTrack => {
            // The first two source rows are the continuous track; only the
            // southern two rows show the single pair of circular instruments.
            live_source_slab(mesh, g, p, [w, e, n, s - 16. * k], -k, 0.015 * k);
            for row in p.row + p.height - 2..p.row + p.height {
                for column in p.column..p.column + p.width {
                    surface(
                        &mut mesh.solid,
                        g.bounds(column, row).into(),
                        room_floor,
                        column,
                        row,
                        k,
                    );
                }
            }
            sampled_room_asset(
                mesh,
                g,
                p,
                Asset::FacilityInstrumentPair,
                [0, 16],
                [w, e, s - depth, s],
                0.,
                12. * k,
            );
        }
        Detail::FacilitySourceBarrier => {
            // Twelve continuous source cells are not a one-cell upright door.
            // Its structural identity is unresolved, so retain the entire live
            // strip at low relief without inventing a shutter or controls.
            live_source_slab(mesh, g, p, b, -k, 1. * k);
        }
        Detail::RoofExit => {
            interior_model(Interior::GateDoorFrame).append_fitted(
                &mut mesh.solid,
                [w, e, s - 7. * k, s],
                0.,
                18. * k,
            );
            slab(
                &mut mesh.solid,
                [w, e, n, s - 8. * k],
                16. * k,
                19. * k,
                [0.48, 0.57, 0.56, 1.],
            );
        }
        Detail::ServiceCounter => {
            // Complete L-shaped service desk, including its inset cash register.
            let elbow = w + 16. * k;
            for part in [[w, elbow, n, s], [elbow, e, s - 16. * k, s]] {
                slab(&mut mesh.solid, part, 0., 10.8 * k, [0.38, 0.24, 0.15, 1.]);
                slab(
                    &mut mesh.solid,
                    part,
                    10.8 * k,
                    12. * k,
                    [0.66, 0.50, 0.31, 1.],
                );
            }
            for j in 0..p.height {
                let z = n + (j as f32 + 0.5) * g.tile_height;
                slab(
                    &mut mesh.solid,
                    [w - 0.02 * k, w + 0.15 * k, z - 1.5 * k, z + 1.5 * k],
                    3. * k,
                    9. * k,
                    [0.26, 0.17, 0.11, 1.],
                );
            }
            slab(
                &mut mesh.solid,
                [w + 2. * k, w + 14. * k, n + 8. * k, n + 19. * k],
                12. * k,
                17. * k,
                [0.61, 0.66, 0.59, 1.],
            );
            slab(
                &mut mesh.solid,
                [w + 4. * k, w + 12. * k, n + 9. * k, n + 13. * k],
                17. * k,
                20. * k,
                [0.28, 0.38, 0.39, 1.],
            );
            for y in 0..3 {
                for x in 0..3 {
                    let xx = w + (4. + x as f32 * 2.5) * k;
                    let zz = n + (14. + y as f32 * 1.4) * k;
                    slab(
                        &mut mesh.solid,
                        [xx, xx + 1.5 * k, zz, zz + 0.9 * k],
                        17. * k,
                        17.4 * k,
                        [0.21, 0.26, 0.23, 1.],
                    );
                }
            }
        }
        Detail::ShrineAltar => {
            slab(
                &mut mesh.solid,
                [w, e, s - 4. * k, s],
                0.,
                19. * k,
                [0.29, 0.16, 0.10, 1.],
            );
            for j in 0..5 {
                let x = w + (e - w) * (j as f32 + 0.5) / 5.;
                slab(
                    &mut mesh.solid,
                    [x - 0.45 * k, x + 0.45 * k, s - 4.4 * k, s + 0.15 * k],
                    0.,
                    19. * k,
                    [0.60, 0.38, 0.18, 1.],
                );
            }
            slab(
                &mut mesh.solid,
                [w, e, s - 5. * k, s + 1. * k],
                16. * k,
                18. * k,
                [0.70, 0.54, 0.28, 1.],
            );
        }
        Detail::LiveFloorInlay => {
            for i in p.indices(g.width) {
                append_top(
                    &mut mesh.textured,
                    g.bounds(i % g.width, i / g.width).into(),
                    0.015 * k,
                    g.uv(i % g.width, i / g.width),
                );
            }
        }
        Detail::Arena => {
            let center = [(w + e) * 0.5, (n + s) * 0.5];
            let radius = [(e - w) * 0.48, (s - n) * 0.48];
            ring(
                &mut mesh.solid,
                center,
                radius,
                0.82,
                -0.3 * k,
                0.005 * k,
                [0.22, 0.28, 0.29, 1.],
                32,
            );
            // Horizontal semicircle inlays remain at actor floor level.
            for half in 0..2 {
                for j in 0..16 {
                    let a = (j + half * 16) as f32 * std::f32::consts::TAU / 32.;
                    let b = (j + half * 16 + 1) as f32 * std::f32::consts::TAU / 32.;
                    let c = [center[0], 0.006 * k, center[1]];
                    let pa = [
                        center[0] + radius[0] * 0.81 * a.cos(),
                        0.006 * k,
                        center[1] + radius[1] * 0.81 * a.sin(),
                    ];
                    let pb = [
                        center[0] + radius[0] * 0.81 * b.cos(),
                        0.006 * k,
                        center[1] + radius[1] * 0.81 * b.sin(),
                    ];
                    faceted_face(
                        &mut mesh.solid,
                        [c, pa, pb, c],
                        [0., 1., 0.],
                        if half == 0 {
                            [0.80, 0.80, 0.67, 1.]
                        } else {
                            [0.36, 0.42, 0.42, 1.]
                        },
                        0.,
                    );
                }
            }
            slab(
                &mut mesh.solid,
                [
                    w + 5. * k,
                    e - 5. * k,
                    center[1] - 1.1 * k,
                    center[1] + 1.1 * k,
                ],
                -0.2 * k,
                0.01 * k,
                [0.23, 0.28, 0.28, 1.],
            );
            ring(
                &mut mesh.solid,
                center,
                [5. * k, 5. * k],
                0.63,
                -0.2 * k,
                0.018 * k,
                [0.21, 0.26, 0.27, 1.],
                16,
            );
        }
        Detail::LinkPlatform { trade } => {
            // Exact source plot includes the octagon and all associated equipment.
            let pb = [w + 1. * k, e - 1. * k, n + 1. * k, s - 1. * k];
            octagon(
                &mut mesh.solid,
                pb,
                -1. * k,
                0.005 * k,
                [0.24, 0.34, 0.39, 1.],
            );
            octagon(
                &mut mesh.solid,
                [w + 3. * k, e - 3. * k, n + 3. * k, s - 3. * k],
                -0.4 * k,
                0.008 * k,
                [0.73, 0.78, 0.73, 1.],
            );
            octagon(
                &mut mesh.solid,
                [w + 7. * k, e - 7. * k, n + 7. * k, s - 7. * k],
                -0.2 * k,
                0.012 * k,
                [0.46, 0.59, 0.61, 1.],
            );
            octagon(
                &mut mesh.solid,
                [w + 9. * k, e - 9. * k, n + 9. * k, s - 9. * k],
                -0.15 * k,
                0.016 * k,
                [0.78, 0.82, 0.75, 1.],
            );
            let cx = (w + e) * 0.5;
            let cz = n + 5. * g.tile_height;
            room_model(if trade {
                Asset::LinkTradeConsole
            } else {
                Asset::LinkBattleConsole
            })
            .append(
                &mut mesh.solid,
                [cx - 16. * k, cx + 16. * k, cz - 12. * k, cz + 4. * k],
                0.02 * k,
                if trade { 13. * k } else { 20. * k },
            );
            for (seat_x, source_x) in [(cx - 24. * k, 16), (cx + 24. * k, 64)] {
                sampled_room_asset(
                    mesh,
                    g,
                    p,
                    Asset::LinkRoundStool,
                    [source_x, 32],
                    [seat_x - 6. * k, seat_x + 6. * k, cz - 6. * k, cz + 6. * k],
                    0.02 * k,
                    6. * k,
                );
            }
            if trade {
                room_model(Asset::LinkRoomReceiver).append(
                    &mut mesh.solid,
                    [cx - 8. * k, cx + 8. * k, n + 8. * k, n + 18. * k],
                    0.02 * k,
                    17. * k,
                );
            }
        }
        Detail::BikeBoundary => {
            // Only the visible lower wheel halves are authored at this boundary;
            // no bicycle frame is inferred from unavailable off-map source art.
            let z = (n + s) * 0.5;
            for cx in [w + (e - w) * 0.25, w + (e - w) * 0.75] {
                let r = (e - w) * 0.23;
                for j in 0..12 {
                    let a = std::f32::consts::PI + j as f32 * std::f32::consts::PI / 12.;
                    let b = std::f32::consts::PI + (j + 1) as f32 * std::f32::consts::PI / 12.;
                    let point = |angle: f32, rad: f32, zz: f32| {
                        [cx + angle.cos() * rad, r + angle.sin() * rad, zz]
                    };
                    for (zz, normal) in
                        [(z - 0.65 * k, [0., 0., -1.]), (z + 0.65 * k, [0., 0., 1.])]
                    {
                        faceted_face(
                            &mut mesh.solid,
                            [
                                point(a, r, zz),
                                point(b, r, zz),
                                point(b, r * 0.72, zz),
                                point(a, r * 0.72, zz),
                            ],
                            normal,
                            [0.10, 0.14, 0.15, 1.],
                            0.,
                        );
                        faceted_face(
                            &mut mesh.solid,
                            [
                                point(a, r * 0.77, zz + 0.01 * k),
                                point(b, r * 0.77, zz + 0.01 * k),
                                point(b, r * 0.72, zz + 0.01 * k),
                                point(a, r * 0.72, zz + 0.01 * k),
                            ],
                            normal,
                            [0.62, 0.68, 0.65, 1.],
                            0.,
                        );
                    }
                    let normal = [((a + b) * 0.5).cos(), ((a + b) * 0.5).sin(), 0.];
                    faceted_face(
                        &mut mesh.solid,
                        [
                            point(a, r, z - 0.65 * k),
                            point(b, r, z - 0.65 * k),
                            point(b, r, z + 0.65 * k),
                            point(a, r, z + 0.65 * k),
                        ],
                        normal,
                        [0.09, 0.13, 0.14, 1.],
                        0.,
                    );
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn grid(w: usize, h: usize) -> GridGeometry {
        GridGeometry {
            width: w,
            height: h,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        }
    }
    fn cells(w: usize, h: usize) -> Vec<VisualTile> {
        (0..w * h)
            .map(|i| VisualTile {
                animation_frames: None,
                column: (i % w) as u32,
                row: (i / w) as u32,
                texture: Default::default(),
                priority: false,
                source: VisualTileSource {
                    tileset_id: Arc::from("test-room"),
                    metatile_id: 0x32 + (i / 16) as u16,
                    subtile_column: (i % w % 4) as u8,
                    subtile_row: (i / w % 4) as u8,
                    tile_index: (i + 1) as u16,
                },
            })
            .collect()
    }
    #[test]
    fn kanto_native_dojo_floor_wall_and_guardian_keep_source_palette_and_support() {
        let g=grid(4,4);let mut tiles=cells(4,4);
        for tile in &mut tiles {tile.source.tileset_id="train_station".into();
            tile.source.metatile_id=0x1c;tile.source.tile_index=0x56;}
        let floor=Placement{column:0,row:0,width:4,height:4,ground:2,
            form:Form::Room(Detail::Surface(Surface::Tatami),FloorStyle::Dojo),
            rise_pixels:0.,depth_pixels:8.,front_rows:4.};
        let mut mesh=TerrainMeshData{footing_heights:vec![5.;16],..Default::default()};
        append_with_floor(&mut mesh,&g,&floor,&tiles.iter().collect::<Vec<_>>(),
            Detail::Surface(Surface::Tatami),FloorStyle::Dojo);
        assert!(mesh.solid.positions.is_empty());
        assert_eq!(mesh.textured.indices.len(),16*48);
        assert!(mesh.textured.positions.iter().all(|p|p[1]<=0. && p[1]>=-0.0481));
        assert_eq!(mesh.footing_heights,vec![5.;16]);
        for (position,uv) in mesh.textured.positions.iter().zip(&mesh.textured.uvs) {
            assert!((uv[0]-position[0]/32.).abs()<0.00001);
            assert!((uv[1]-position[2]/32.).abs()<0.00001);
        }
        tiles[0].source.metatile_id=0x24;tiles[0].source.tile_index=0x30;
        assert!(matches!(cell_detail("FightingDojo",&tiles[0].source),Some(Detail::Wall(_))));
        let wall=Placement{width:1,height:1,form:Form::Room(Detail::Wall(Wall::Timber),FloorStyle::Dojo),
            ..floor.clone()};
        let mut mesh=TerrainMeshData{footing_heights:vec![5.;16],..Default::default()};
        append_with_floor(&mut mesh,&g,&wall,&tiles.iter().collect::<Vec<_>>(),Detail::Wall(Wall::Timber),FloorStyle::Dojo);
        assert!(mesh.solid.indices.is_empty());
        assert_eq!(mesh.textured.indices.len(),48+2*12+8*6,
            "beveled floor, two four-triangle caps and eight native side quads");
        for tri in mesh.textured.indices[48..].chunks_exact(3) {
            let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
            assert!((b-a).cross(c-a).dot(Vec3::from_array(mesh.textured.normals[tri[0]as usize]))>0.);
        }
        let drawing=[[0x48,0x49],[0x58,0x59],[0x10,0x01],[0x5a,0x5b]];
        for y in 0..4 {for x in 0..2 {let s=&mut tiles[y*4+x].source;
            s.metatile_id=0x26;s.tile_index=drawing[y][x];}}
        let guardian=OBJECTS.iter().find(|o|matches!(o.detail,Detail::Guardian)
            && matches_object("FightingDojo",&tiles.iter().collect::<Vec<_>>(),&g,0,0,o)).unwrap();
        let p=Placement{width:2,height:4,form:Form::Room(Detail::Guardian,FloorStyle::Dojo),
            rise_pixels:guardian.rise,depth_pixels:guardian.depth,..floor};
        let mut mesh=TerrainMeshData{footing_heights:vec![5.;16],..Default::default()};
        append_with_floor(&mut mesh,&g,&p,&tiles.iter().collect::<Vec<_>>(),Detail::Guardian,FloorStyle::Dojo);
        assert!(mesh.solid.indices.is_empty());assert_eq!(mesh.footing_heights,vec![5.;16]);
        assert!(mesh.textured.uvs.contains(&source_texel_uv(&g,&p,[2,2])));
        assert!(mesh.textured.uvs.contains(&source_texel_uv(&g,&p,[8,24])));
        tiles[0].source.tile_index^=1;
        assert!(!matches_object("FightingDojo",&tiles.iter().collect::<Vec<_>>(),&g,0,0,guardian));
    }
    #[test]
    fn special_room_complete_objects_reject_every_identity_mutation_phase_crop_and_unknown_scope() {
        let t = cells(4, 4);
        let refs = t.iter().collect::<Vec<_>>();
        let o = Object {
            map: "TestRoom",
            tileset: "test-room",
            width: 4,
            height: 4,
            fingerprint: identity_hash(t.iter().map(|t| &t.source)),
            detail: Detail::Asset(Asset::MobileBattleTerminal),
            rise: 24.,
            depth: 20.,
        };
        assert!(matches_object("TestRoom", &refs, &grid(4, 4), 0, 0, &o));
        assert!(!matches_object("UnknownRoom", &refs, &grid(4, 4), 0, 0, &o));
        for index in 0..16 {
            for mutation in 0..5 {
                let mut changed = t.clone();
                let s = &mut changed[index].source;
                match mutation {
                    0 => s.tile_index ^= 1,
                    1 => s.metatile_id ^= 1,
                    2 => s.subtile_column = (s.subtile_column + 1) % 4,
                    3 => s.subtile_row = (s.subtile_row + 1) % 4,
                    _ => s.tileset_id = Arc::from("other-atlas"),
                }
                assert!(
                    !matches_object(
                        "TestRoom",
                        &changed.iter().collect::<Vec<_>>(),
                        &grid(4, 4),
                        0,
                        0,
                        &o
                    ),
                    "cell {index}, mutation {mutation}"
                );
            }
        }
        for (w, h) in [(3, 4), (4, 3), (1, 4), (4, 1)] {
            assert!(!matches_object("TestRoom", &refs, &grid(w, h), 0, 0, &o));
        }
        let mut phase = t.clone();
        for t in &mut phase {
            t.source.subtile_row = (t.source.subtile_row + 1) % 4;
        }
        assert!(!matches_object(
            "TestRoom",
            &phase.iter().collect::<Vec<_>>(),
            &grid(4, 4),
            0,
            0,
            &o
        ));
    }
    #[test]
    fn special_room_matchers_are_location_independent_but_keep_native_block_phase() {
        let mut t = cells(8, 5);
        let mut plot = cells(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                t[(y + 1) * 8 + x + 3].source = plot[y * 4 + x].source.clone();
            }
        }
        let o = Object {
            map: "TestRoom",
            tileset: "test-room",
            width: 4,
            height: 4,
            fingerprint: identity_hash(plot.iter().map(|t| &t.source)),
            detail: Detail::Pit,
            rise: 0.,
            depth: 16.,
        };
        assert!(matches_object(
            "TestRoom",
            &t.iter().collect::<Vec<_>>(),
            &grid(8, 5),
            3,
            1,
            &o
        ));
        plot[0].source.tile_index += 1;
        assert!(!matches_object(
            "TestRoom",
            &plot.iter().collect::<Vec<_>>(),
            &grid(4, 4),
            0,
            0,
            &o
        ));
    }
    #[test]
    fn special_room_surface_signatures_are_sorted_and_do_not_impersonate_objects() {
        for rule in CELLS {
            assert!(rule.fingerprints.windows(2).all(|w| w[0] < w[1]));
            assert!(!matches!(
                rule.detail,
                Detail::Asset(_) | Detail::Interior(_) | Detail::RoofSeat | Detail::StoreLiftGlazing | Detail::LinkPlatform { .. }
            ));
        }
        let maps = OBJECTS
            .iter()
            .map(|o| o.map)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(maps.len(), 19);
        for o in OBJECTS {
            assert!(o.width * o.height > 1);
            assert!(o.depth > 0.);
            if matches!(o.detail, Detail::RoofSeat) {
                assert_eq!(o.map, "GoldenrodDeptStoreRoof");
                assert_eq!((o.width, o.height, o.fingerprint), (2, 4, 0x8dc9d24b7ddbc743));
            }
            if matches!(o.detail, Detail::StoreLiftGlazing) {
                assert!(matches!(o.map, "GoldenrodDeptStoreElevator" | "CeladonDeptStoreElevator"));
                assert_eq!((o.width,o.height,o.fingerprint), (6,2,0x904830824255ead1));
            }
        }
        assert!(
            OBJECTS
                .iter()
                .any(|o| matches!(o.detail, Detail::BikeBoundary))
        );
        assert!(
            OBJECTS
                .iter()
                .filter(|o| matches!(o.detail, Detail::LinkPlatform { .. }))
                .count()
                == 3
        );
    }
    fn rendered(detail: Detail, w: usize, h: usize) -> TerrainMeshData {
        let t = cells(w, h);
        let refs = t.iter().collect::<Vec<_>>();
        let mut m = TerrainMeshData::default();
        m.footing_heights = vec![0.; w * h];
        let p = Placement {
            column: 0,
            row: 0,
            width: w,
            height: h,
            ground: 0,
            form: Form::Room(detail, FloorStyle::Default),
            rise_pixels: 24.,
            depth_pixels: h as f32 * 8.,
            front_rows: h as f32,
        };
        append(&mut m, &grid(w, h), &p, &refs, detail);
        assert_eq!(m.footing_heights, vec![0.; w * h]);
        m
    }
    #[test]
    fn roof_sale_cartons_have_native_materials_closed_depth_and_unchanged_footing() {
        for stacked in [false,true] { for scale in [0.5,1.,1.5] {
            let height = if stacked {4} else {2};
            let tiles=cells(3,height); let refs=tiles.iter().collect::<Vec<_>>();
            let mut g=grid(3,height);g.tile_width*=scale;g.tile_height*=scale;
            let detail=Detail::RoofSaleBoxes{stacked};
            let p=Placement{column:0,row:0,width:3,height,ground:0,
                form:Form::Room(detail,FloorStyle::Roof),rise_pixels:26.,
                depth_pixels:height as f32*8.,front_rows:height as f32};
            let mut mesh=TerrainMeshData{footing_heights:vec![7.;3*height],..Default::default()};
            append_roof(&mut mesh,&g,&p,&refs,detail);
            assert_eq!(mesh.footing_heights,vec![7.;3*height]);
            let anchors=[if stacked {[2,10]} else {[2,9]},[1,2],[0,0],[3,2]];
            let uv=anchors.map(|pixel|source_texel_uv(&g,&p,pixel));
            let mut faces=[false;6];let mut role_seen=[false;4];
            for ((position,normal),sample) in mesh.textured.positions.iter()
                .zip(&mesh.textured.normals).zip(&mesh.textured.uvs) {
                // Paving uses varying UVs; cartons use four verified texels.
                if let Some(role)=uv.iter().position(|expected|expected==sample) {
                    role_seen[role]=true;
                    assert!(position[0]>=0. && position[0]<=24.*scale);
                    assert!(position[2]>=0. && position[2]<=height as f32*8.*scale);
                    assert!(position[1]>=0. && position[1]<=26.5*scale);
                    for (face,(axis,sign)) in [(0,-1.),(0,1.),(1,-1.),(1,1.),(2,-1.),(2,1.)].into_iter().enumerate() {
                        if normal[axis]==sign {faces[face]=true;}
                    }
                }
            }
            assert!(faces.into_iter().all(|seen|seen),"cartons require closed six-sided volumes");
            assert!(role_seen.into_iter().all(|seen|seen),"native body, lid, seam and label must all be used");
        }}
    }
    #[test]
    fn roof_stairs_keep_complete_live_gray_drawing_without_painted_treads() {
        for height in [2,4] {for scale in [0.5,1.,1.5] {
            let tiles=cells(2,height);let refs=tiles.iter().collect::<Vec<_>>();
            let mut g=grid(2,height);g.tile_width*=scale;g.tile_height*=scale;
            let p=Placement {column:0,row:0,width:2,height,ground:0,
                form:Form::Room(Detail::Stairs,FloorStyle::Roof),rise_pixels:8.,
                depth_pixels:height as f32*8.,front_rows:height as f32};
            let mut mesh=TerrainMeshData {footing_heights:vec![3.;2*height],..Default::default()};
            append_roof(&mut mesh,&g,&p,&refs,Detail::Stairs);
            assert!(mesh.solid.indices.is_empty(),"roof stairs must retain live source paint");
            assert_eq!(mesh.footing_heights,vec![3.;2*height]);
            let mut area=0.;let mut levels=std::collections::BTreeSet::new();
            for tri in mesh.textured.indices.chunks_exact(3) {
                let uv=[tri[0],tri[1],tri[2]].map(|i|mesh.textured.uvs[i as usize]);
                if uv[0]==uv[1] && uv[1]==uv[2] {continue;}
                let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                area+=(points[1]-points[0]).cross(points[2]-points[0]).y*0.5;
                levels.insert(points[0].y.to_bits());
                for (point,uv) in points.iter().zip(uv) {
                    assert!((uv[0]-point.x/(16.*scale)).abs()<0.0001);
                    assert!((uv[1]-point.z/(height as f32*8.*scale)).abs()<0.0001);
                }
            }
            assert!((area-(2*height*64) as f32*scale*scale).abs()<0.001);
            assert_eq!(levels.len(),5);
            assert!(mesh.textured.positions.iter().all(|p|p[1]<=0. && p[1]>=-8.*scale));
        }}
    }
    #[test]
    fn roof_paving_preserves_live_tile_uvs_and_uses_floor_donor_under_seats() {
        let tiles = cells(2, 4);
        let refs = tiles.iter().collect::<Vec<_>>();
        let g = grid(2, 4);
        for detail in [Detail::Surface(Surface::Ceramic), Detail::RoofSeat] {
            let p = Placement {
                column: 0, row: 0, width: 2, height: 4, ground: 7,
                form: Form::Room(detail, FloorStyle::Roof), rise_pixels: 8.,
                depth_pixels: 22., front_rows: 4.,
            };
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![0.; 8];
            underlay(&mut mesh, &g, &p, Surface::RoofTile, &refs);
            let mut expected = SurfaceMeshData::default();
            for i in 0..8 {
                let donor = if matches!(detail, Detail::Surface(_)) { i } else { 7 };
                append_top(&mut expected, g.bounds(i % 2, i / 2).into(), 0.,
                    g.uv(donor % 2, donor / 2));
            }
            assert_eq!(mesh.textured, expected);
            assert_eq!(mesh.footing_heights, vec![0.; 8]);
            assert!(mesh.solid.indices.chunks_exact(3)
                .all(|tri| mesh.solid.normals[tri[0] as usize][1] < 0.5));
            assert!(mesh.solid.normals.iter().any(|n| n[1] < -0.5));
        }
    }
    #[test]
    fn blackthorn_partition_live_gray_palette_keeps_authored_geometry_and_support() {
        let mut tiles=cells(4,4);
        for t in &mut tiles {let s=&mut t.source;s.tileset_id="elite_four_room".into();
            s.metatile_id=0x10;s.tile_index=if s.subtile_column<2 {
                [[0x2a,0x2b],[0x3a,0x3b]][s.subtile_row as usize%2][s.subtile_column as usize]
            } else {1};}
        let refs=tiles.iter().collect::<Vec<_>>();let g=grid(4,4);
        let p=Placement {column:0,row:0,width:2,height:2,ground:2,
            form:Form::Room(Detail::Asset(Asset::GymStonePartition),FloorStyle::Default),
            rise_pixels:13.,depth_pixels:16.,front_rows:2.};
        let mut original=SurfaceMeshData::default();
        room_model(Asset::GymStonePartition).append(&mut original,[0.,16.,0.,16.],0.,13.);
        let mut mesh=TerrainMeshData {footing_heights:vec![3.;16],..Default::default()};
        let mut floor=TerrainMeshData::default();
        underlay(&mut floor,&g,&p,Surface::Stone,&refs);
        append(&mut mesh,&g,&p,&refs,Detail::Asset(Asset::GymStonePartition));
        assert_eq!(mesh.solid,floor.solid,"only the existing floor remains solid-colored");
        assert_eq!(mesh.textured.positions,original.positions);
        assert_eq!(mesh.textured.normals,original.normals);
        assert_eq!(mesh.footing_heights,vec![3.;16]);
        for pixel in [[4,4],[3,3],[4,10]] {
            assert!(mesh.textured.uvs.contains(&source_texel_uv(&g,&p,pixel)));
        }
        assert!(mesh.textured.colors.iter().all(|color|*color==[1.;4]));
        assert_eq!(original.indices.len()/3,220);
    }
    #[test]
    fn roof_planter_keeps_leaf_volume_live_source_palette_and_ground_support() {
        let tiles = cells(4, 2);
        let refs = tiles.iter().collect::<Vec<_>>();
        let g = grid(4, 2);
        let p = Placement {
            column: 0, row: 0, width: 4, height: 2, ground: 7,
            form: Form::Room(Detail::Asset(Asset::RoofPlanter), FloorStyle::Roof),
            rise_pixels: 13., depth_pixels: 12., front_rows: 2.,
        };
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![0.; 8];
        append_roof(&mut mesh, &g, &p, &refs, Detail::Asset(Asset::RoofPlanter));
        for pixel in [[2,12],[2,9],[4,13],[0,0],[4,2],[2,1],[9,2]] {
            assert!(mesh.textured.uvs.contains(&source_texel_uv(&g, &p, pixel)));
        }
        assert_eq!(mesh.footing_heights, vec![0.; 8]);
        assert!(mesh.textured.positions.iter().any(|v| v[1] > 12.9));
        assert!(mesh.textured.normals.iter().any(|n| n[1] < -0.5));
        assert!(mesh.textured.positions.iter().all(|v| v.iter().all(|c| c.is_finite())));
        assert!(mesh.textured.colors.iter().all(|c| *c == [1.;4]));
    }
    #[test]
    fn store_lift_cabin_preserves_live_floor_carpet_and_three_closed_blue_windows() {
        let tiles = cells(6,2);
        let refs = tiles.iter().collect::<Vec<_>>();
        let g = grid(6,2);
        for detail in [Detail::Surface(Surface::Ceramic), Detail::Threshold, Detail::StoreLiftGlazing] {
            let p = Placement {
                column:0,row:0,width:6,height:2,ground:11,
                form:Form::Room(detail,FloorStyle::StoreLift),
                rise_pixels:24.,depth_pixels:3.,front_rows:2.,
            };
            let mut mesh = TerrainMeshData { footing_heights:vec![0.;12], ..Default::default() };
            append_with_floor(&mut mesh,&g,&p,&refs,detail,FloorStyle::StoreLift);
            let mut expected = SurfaceMeshData::default();
            for i in 0..12 {
                let donor = if matches!(detail,Detail::StoreLiftGlazing) {11} else {i};
                append_top(&mut expected,g.bounds(i%6,i/6).into(),0.,g.uv(donor%6,donor/6));
            }
            assert_eq!(mesh.textured,expected);
            assert_eq!(mesh.footing_heights,vec![0.;12]);
            if matches!(detail,Detail::StoreLiftGlazing) {
                assert!(mesh.solid.positions.iter().any(|v|v[1]>23.9));
                let mut three = SurfaceMeshData::default();
                super::super::super::department_store::append_lift_cabin_glazing(
                    &mut three,[0.,48.,13.,16.],24.);
                assert!(three.colors.iter().any(|c|c[2]>c[0] && c[2]>c[1]));
                assert!(three.normals.iter().any(|n|n[2] < -0.9));
            }
        }
        assert!(!CELLS.iter().any(|r| matches!(r.map,
            "GoldenrodDeptStoreElevator" | "CeladonDeptStoreElevator")
            && matches!(r.detail,Detail::Wall(_))));
    }
    #[test]
    fn arcade_entry_mat_resolves_in_all_three_maps_without_partial_claims() {
        let mut tiles=cells(4,3);
        for (i,tile) in tiles.iter_mut().enumerate() {
            tile.source=if i<4 {
                VisualTileSource { tileset_id: Arc::from("game_corner"), metatile_id:1,
                    subtile_column:i as u8, subtile_row:0, tile_index:if i%2==0 {1} else {0x10} }
            } else {
                VisualTileSource { tileset_id: Arc::from("game_corner"), metatile_id:0x0c,
                    subtile_column:(i%4) as u8, subtile_row:(1+i/4) as u8,
                    tile_index:if i<8 {0x03} else {0x13} }
            };
        }
        for map in ["GoldenrodGameCorner","CeladonGameCorner","CeladonGameCornerPrizeRoom"] {
            let refs=tiles.iter().collect::<Vec<_>>(); let g=grid(4,3);
            let mut resolver=Resolver { map,cells:&refs,g:&g,claimed:vec![false;12],out:vec![] };
            resolve_into(&mut resolver);
            assert_eq!(resolver.out.iter().filter(|p| matches!(p.form, Form::Room(Detail::ArcadeEntryMat, _))).count(),1);
            assert!(resolver.claimed[4..].iter().all(|c| *c));
            let mut resolver=Resolver { map,cells:&refs,g:&g,claimed:vec![false;12],out:vec![] };
            resolver.claimed[10]=true;
            resolve_into(&mut resolver);
            assert!(!resolver.out.iter().any(|p| matches!(p.form, Form::Room(Detail::ArcadeEntryMat, _))));
            for i in 4..12 { assert_eq!(resolver.claimed[i],i==10); }
            let mut changed=tiles.clone();changed[6].source.tile_index^=1;
            let refs=changed.iter().collect::<Vec<_>>();
            let mut resolver=Resolver { map,cells:&refs,g:&g,claimed:vec![false;12],out:vec![] };
            resolve_into(&mut resolver);
            assert!(resolver.claimed[4..].iter().all(|c| !*c));
        }
    }
    #[test]
    fn arcade_entry_mat_requires_the_whole_drawing_and_keeps_warp_footing() {
        let object = OBJECTS.iter().find(|o| o.map == "CeladonGameCornerPrizeRoom" && matches!(o.detail, Detail::ArcadeEntryMat)).unwrap();
        let mut tiles = cells(4,2);
        for (i,tile) in tiles.iter_mut().enumerate() {
            tile.source = VisualTileSource { tileset_id: Arc::from("game_corner"),
                metatile_id: 0x0c, subtile_column: (i%4) as u8,
                subtile_row: (2+i/4) as u8, tile_index: if i<4 {0x03} else {0x13} };
        }
        for binding in OBJECTS.iter().filter(|o| matches!(o.detail, Detail::ArcadeEntryMat)) {
            assert!(matches_object(binding.map, &tiles.iter().collect::<Vec<_>>(), &grid(4,2),0,0,binding));
        }
        assert!(matches_object("CeladonGameCornerPrizeRoom", &tiles.iter().collect::<Vec<_>>(), &grid(4,2),0,0,object));
        assert!(!matches_object("GoldenrodGameCorner", &tiles.iter().collect::<Vec<_>>(), &grid(4,2),0,0,object));
        for i in 0..8 { for mutation in 0..5 {
            let mut changed=tiles.clone(); let source=&mut changed[i].source;
            match mutation {
                0 => source.tile_index ^=1,
                1 => source.metatile_id ^=1,
                2 => source.subtile_column ^=1,
                3 => source.subtile_row ^=1,
                _ => source.tileset_id=Arc::from("other"),
            }
            assert!(!matches_object("CeladonGameCornerPrizeRoom", &changed.iter().collect::<Vec<_>>(), &grid(4,2),0,0,object));
        }}
        assert!(!matches_object("CeladonGameCornerPrizeRoom", &tiles.iter().collect::<Vec<_>>(), &grid(3,2),0,0,object));
        let mesh=rendered(Detail::ArcadeEntryMat,4,2);
        assert!(mesh.solid.positions.iter().all(|p| p[1]<=0. && p[1]>=-0.401));
        let mut floor_area=0.;
        for triangle in mesh.solid.indices.chunks_exact(3) {
            let [a,b,c]=[triangle[0],triangle[1],triangle[2]].map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
            let cross=(b-a).cross(c-a);
            if [a,b,c].iter().all(|p| p.y==0.) && cross.y>0. { floor_area+=cross.y*0.5; }
        }
        assert!((floor_area-32.*16.).abs()<0.001);
        assert!(mesh.solid.colors.iter().any(|c| c[0]>c[1]*3.));
        assert!(mesh.textured.positions.is_empty());
    }
    #[test]
    fn special_room_holes_stay_open_stairs_descend_and_surfaces_keep_actor_datum() {
        let hole = rendered(Detail::Pit, 2, 2);
        assert!(hole.solid.positions.iter().all(|p| p[1] <= 0.));
        for tri in hole.solid.indices.chunks_exact(3) {
            let ps = tri
                .iter()
                .map(|&i| hole.solid.positions[i as usize])
                .collect::<Vec<_>>();
            if ps.iter().all(|p| p[1].abs() < 0.0001) {
                let cx = ps.iter().map(|p| p[0]).sum::<f32>() / 3.;
                let cz = ps.iter().map(|p| p[2]).sum::<f32>() / 3.;
                assert!(cx <= 0.61 || cx >= 15.39 || cz <= 0.61 || cz >= 15.39);
            }
        }
        let stairs = rendered(Detail::Stairs, 2, 2);
        assert!(stairs.solid.positions.iter().any(|p| p[1] < -4.));
        assert!(stairs.solid.positions.iter().all(|p| p[1] <= 0.));
        for s in [
            Surface::BlueBrick,
            Surface::Stone,
            Surface::Ceramic,
            Surface::ArcadeCeramic,
            Surface::Timber,
            Surface::Tatami,
            Surface::Facility,
        ] {
            let m = rendered(Detail::Surface(s), 2, 2);
            assert!(m.solid.positions.iter().all(|p| p[1] <= 0.002));
        }
    }
    #[test]
    fn arcade_ceramic_keeps_native_boxed_paint_on_beveled_floor() {
        let m = rendered(Detail::Surface(Surface::ArcadeCeramic), 2, 2);
        assert_eq!(m.textured.indices.len(),4*48);
        assert!(m.textured.positions.iter().any(|p|p[1]<0.));
        for (point,uv) in m.textured.positions.iter().zip(&m.textured.uvs) {
            assert!((uv[0]-point[0]/16.).abs()<0.0001);
            assert!((uv[1]-point[2]/16.).abs()<0.0001);
            assert!(point[1]<=0. && point[1]>=-0.0481);
        }
        assert!(m.solid.indices.chunks_exact(3).all(|t|m.solid.normals[t[0]as usize][1]<0.5));
        let s=VisualTileSource {tileset_id:Arc::from("game_corner"),metatile_id:1,
            subtile_column:0,subtile_row:0,tile_index:1};
        assert!(matches!(cell_detail("CeladonGameCornerPrizeRoom",&s),Some(Detail::Surface(Surface::ArcadeCeramic))));
        let mut changed=s.clone();changed.tile_index^=1;
        assert!(!matches!(cell_detail("CeladonGameCornerPrizeRoom",&changed),Some(Detail::Surface(Surface::ArcadeCeramic))));
        assert!(!matches!(cell_detail("UnknownGameCorner",&s),Some(Detail::Surface(Surface::ArcadeCeramic))));
    }
    #[test]
    fn special_room_thresholds_and_tracks_have_one_complete_floor_surface() {
        for detail in [Detail::Threshold, Detail::FloorBand, Detail::ShutterTrack, Detail::Surface(Surface::BlueBrick), Detail::Surface(Surface::ArcadeCeramic)] {
            let mesh = rendered(detail, 2, 2);
            let mut floor_area = 0.;
            let native_arcade=matches!(detail,Detail::Surface(Surface::ArcadeCeramic));
            let surfaces=[&mesh.solid,&mesh.textured];
            for surface in surfaces.iter().take(if native_arcade {2} else {1}) {for tri in surface.indices.chunks_exact(3) {
                let [a, b, c] = [tri[0], tri[1], tri[2]]
                    .map(|i| Vec3::from_array(surface.positions[i as usize]));
                let cross = (b - a).cross(c - a);
                if [a, b, c].iter().all(|p|if native_arcade {p.y<=0.0001 && p.y>=-0.0481}
                    else {p.y.abs()<0.0001}) && cross.y > 0. {
                    floor_area += cross.y * 0.5;
                }
            }}
            // An overlapping underlay adds almost another full plot of area;
            // missing floor geometry reduces it. Raised grip strips are above
            // this plane, and rendered() separately checks unchanged footing.
            assert!(
                (floor_area - 16. * 16.).abs() < 0.001,
                "{detail:?}: {floor_area}"
            );
            if matches!(detail, Detail::Surface(Surface::BlueBrick | Surface::ArcadeCeramic)) {
                assert!(mesh.solid.positions.iter().all(|p| p[1] <= 0.));
                assert!(mesh.solid.positions.iter().any(|p| p[1] < 0.));
            } else {
                assert!(mesh.solid.positions.iter().any(|p| p[1] > 0.));
            }
        }
    }
    #[test]
    fn special_room_unknown_inset_and_barrier_keep_complete_live_source_footprints() {
        for (detail, width, height) in [
            (Detail::RoofInset, 8, 5),
            (Detail::FacilitySourceBarrier, 1, 12),
        ] {
            let mesh = rendered(detail, width, height);
            let mut source_area = 0.;
            for triangle in mesh.textured.indices.chunks_exact(3) {
                if triangle
                    .windows(2)
                    .all(|w| mesh.textured.uvs[w[0] as usize] == mesh.textured.uvs[w[1] as usize])
                {
                    continue; // Constant-texel shallow side/underside material.
                }
                let [a, b, c] = [triangle[0], triangle[1], triangle[2]]
                    .map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
                source_area += (b - a).cross(c - a).y * 0.5;
            }
            assert!((source_area - (width * height * 64) as f32).abs() < 0.001);
            // Unknown source plots must not reintroduce a deep invented stairwell
            // or turn the twelve-cell strip back into a short upright door.
            assert!(
                mesh.textured
                    .positions
                    .iter()
                    .all(|p| p[1] >= -1.001 && p[1] <= 3.)
            );
            let depth = mesh
                .textured
                .positions
                .iter()
                .map(|p| p[2])
                .fold(f32::NEG_INFINITY, f32::max)
                - mesh
                    .textured
                    .positions
                    .iter()
                    .map(|p| p[2])
                    .fold(f32::INFINITY, f32::min);
            assert!((depth - height as f32 * 8.).abs() < 0.001);
        }
    }
    #[test]
    fn special_room_semantic_corrections_retain_original_complete_identity_plots() {
        let binoculars = OBJECTS
            .iter()
            .filter(|o| matches!(o.detail, Detail::Asset(Asset::RoofBinoculars)))
            .collect::<Vec<_>>();
        assert_eq!(binoculars.len(), 2); // Three fixtures, two native block phases.
        assert!(
            binoculars
                .iter()
                .all(|o| o.map == "GoldenrodDeptStoreRoof" && o.width == 2 && o.height == 2)
        );
        assert!(OBJECTS.iter().all(|o| o.map != "GoldenrodDeptStoreRoof"
            || !matches!(o.detail, Detail::Interior(Interior::PottedPlant))));
        let barrier = OBJECTS
            .iter()
            .find(|o| matches!(o.detail, Detail::FacilitySourceBarrier))
            .unwrap();
        assert_eq!((barrier.width, barrier.height), (1, 12));
        assert_eq!(barrier.depth, 96.);
        let banks = OBJECTS
            .iter()
            .filter(|o| {
                matches!(
                    o.detail,
                    Detail::Asset(Asset::FacilityInstrumentBank) | Detail::FacilityInstrumentTrack
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(banks.len(), 5);
        assert!(
            banks
                .iter()
                .all(|o| o.map == "TeamRocketBaseB2F" && o.width == 2 && o.height == 4)
        );
    }
    #[test]
    fn special_room_source_material_anchors_follow_local_subplots_and_live_grid_uvs() {
        let geometry = grid(24, 20);
        let p = Placement {
            column: 4,
            row: 4,
            width: 12,
            height: 10,
            ground: 0,
            form: Form::Room(Detail::LinkPlatform { trade: false }, FloorStyle::Default),
            rise_pixels: 0.,
            depth_pixels: 80.,
            front_rows: 10.,
        };
        // The same material on each stool must sample that stool's own source
        // cell, not a cached palette from the first instance or another map.
        for (offset, cell) in [([16, 32], [6, 8]), ([64, 32], [12, 8])] {
            let actual = source_texel_uv(&geometry, &p, [offset[0] + 7, offset[1] + 7]);
            let uv = geometry.uv(cell[0], cell[1]);
            let expected = [
                uv.0 + (uv.1 - uv.0) * 7.5 / 8.,
                uv.2 + (uv.3 - uv.2) * 7.5 / 8.,
            ];
            assert_eq!(actual, expected);
        }
    }
    #[test]
    fn special_room_geometry_is_finite_with_consistent_normals_and_no_footing_writes() {
        let details = [
            Detail::Arena,
            Detail::LinkPlatform { trade: false },
            Detail::LinkPlatform { trade: true },
            Detail::RoofInset,
            Detail::FacilityInstrumentTrack,
            Detail::FacilitySourceBarrier,
            Detail::RoofExit,
            Detail::ServiceCounter,
            Detail::ShrineAltar,
            Detail::LivePlaque,
            Detail::Pit,
            Detail::Stairs,
            Detail::WallSegment {
                material: Wall::Steel,
                open: [true, false, true, false],
            },
            Detail::RoofRail,
            Detail::BikeBoundary,
        ];
        for d in details {
            let m = rendered(d, 12, 10);
            assert!(
                !m.solid.indices.is_empty() || !m.textured.indices.is_empty(),
                "{d:?}"
            );
            for (p, n) in m
                .solid
                .positions
                .iter()
                .zip(&m.solid.normals)
                .chain(m.textured.positions.iter().zip(&m.textured.normals))
            {
                assert!(p.iter().all(|x| x.is_finite()));
                assert!((Vec3::from_array(*n).length() - 1.).abs() < 0.001);
            }
            for surface in [&m.solid, &m.textured] {
                for tri in surface.indices.chunks_exact(3) {
                    let a = Vec3::from_array(surface.positions[tri[0] as usize]);
                    let b = Vec3::from_array(surface.positions[tri[1] as usize]);
                    let c = Vec3::from_array(surface.positions[tri[2] as usize]);
                    let actual = (b - a).cross(c - a);
                    assert!(actual.length_squared() > 1e-12, "{d:?}");
                    let expected = Vec3::from_array(surface.normals[tri[0] as usize]);
                    assert!(actual.normalize().dot(expected) > 0.99, "{d:?}");
                }
            }
        }
    }
}
