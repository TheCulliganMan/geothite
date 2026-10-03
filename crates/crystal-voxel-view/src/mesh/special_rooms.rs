//! Complete native drawings fitted with original, editable 3D room geometry.
//! Fingerprints bind every immutable source identity in an entire object plot.
//! Rendering never reads or changes collision, warps, actors, or footing.
use super::*;
use crate::dungeon_models::{RoomAsset as Asset, room_model};
use crate::interior_models::{ModelKind as Interior, model as interior_model};
#[derive(Clone, Copy, Debug)]
pub(super) enum Surface {
    Stone,
    Ceramic,
    Timber,
    Tatami,
    Facility,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Wall {
    Steel,
    Timber,
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
    FloorBand,
    ShutterTrack,
    Pit,
    Stairs,
    RoofInset,
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
}
impl Detail {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Asset(a) => a.label(),
            Self::Interior(Interior::VendingMachine) => "special-room:terrace-vending-machine",
            Self::Interior(Interior::StationBench) => "special-room:terrace-seat",
            Self::Interior(Interior::GiftShelf) => "special-room:prize-display",
            Self::Interior(Interior::LabWorkstation) => "special-room:facility-workstation",
            Self::Interior(_) => "special-room:verified-furniture",
            Self::Surface(Surface::Stone) => "special-room:surface-stone",
            Self::Surface(Surface::Ceramic) => "special-room:surface-ceramic",
            Self::Surface(Surface::Timber) => "special-room:surface-timber",
            Self::Surface(Surface::Tatami) => "special-room:surface-dojo-mat",
            Self::Surface(Surface::Facility) => "special-room:surface-facility",
            Self::Wall(_) | Self::WallSegment { .. } => "special-room:connected-enclosure",
            Self::RoofRail => "special-room:terrace-parapet",
            Self::LivePlaque => "special-room:live-sign-in-frame",
            Self::Threshold => "special-room:surface-threshold",
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
        form: Form::Room(detail),
        rise_pixels: rise,
        depth_pixels: depth,
        front_rows: h as f32,
    });
}
pub(super) fn resolve_into(r: &mut Resolver<'_>) {
    let (map, cells, g) = (r.map, r.cells, r.g);
    let Some(ground) = cells
        .iter()
        .position(|c| matches!(cell_detail(map, &c.source), Some(Detail::Surface(_))))
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
    let c = match kind {
        Surface::Stone => [0.55, 0.57, 0.50, 1.],
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
fn underlay(mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement, kind: Surface) {
    for i in p.indices(g.width) {
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
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    detail: Detail,
) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + p.width as f32 * g.tile_width;
    let s = n + p.height as f32 * g.tile_height;
    let k = g.tile_height / SOURCE_TILE_HEIGHT;
    let rise = p.rise_pixels * k;
    let depth = p.depth_pixels * k;
    let b = [w, e, n, s];
    let room_floor = match cells[p.ground].source.tileset_id.as_ref() {
        "lab" => Surface::Timber,
        "train_station" => Surface::Tatami,
        "facility" => Surface::Facility,
        "elite_four_room" => Surface::Stone,
        _ => Surface::Ceramic,
    };
    if !matches!(
        detail,
        Detail::Pit
            | Detail::Stairs
            | Detail::RoofInset
            | Detail::FacilityInstrumentTrack
            | Detail::FacilitySourceBarrier
            | Detail::Surface(_)
            | Detail::Threshold
            | Detail::FloorBand
            | Detail::ShutterTrack
    ) {
        underlay(mesh, g, p, room_floor);
    }
    match detail {
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
        Detail::Asset(a) => room_model(a).append(&mut mesh.solid, [w, e, s - depth, s], 0., rise),
        Detail::Interior(a) => {
            interior_model(a).append_fitted(&mut mesh.solid, [w, e, s - depth, s], 0., rise)
        }
        Detail::Guardian => {
            model(Kind::TowerGuardian).append(&mut mesh.solid, [w, e, s - depth, s], 0., rise)
        }
        Detail::Surface(kind) => underlay(mesh, g, p, kind),
        Detail::Wall(material) | Detail::WallSegment { material, .. } => {
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
            slab(&mut mesh.solid, b, 0., 7. * k, [0.40, 0.48, 0.49, 1.]);
            slab(&mut mesh.solid, b, 7. * k, 8. * k, [0.69, 0.73, 0.68, 1.]);
            for j in 0..3 {
                let x = w + (e - w) * (j as f32 + 0.5) / 3.;
                slab(
                    &mut mesh.solid,
                    [x - 0.18 * k, x + 0.18 * k, s - 0.35 * k, s + 0.01 * k],
                    1. * k,
                    6.5 * k,
                    [0.27, 0.35, 0.35, 1.],
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
                slab(
                    &mut mesh.solid,
                    [w + 0.4 * k, e - 0.4 * k, z0, z1],
                    -8. * k,
                    y,
                    [0.60, 0.63, 0.56, 1.],
                );
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
                Detail::Asset(_) | Detail::Interior(_) | Detail::LinkPlatform { .. }
            ));
        }
        let maps = OBJECTS
            .iter()
            .map(|o| o.map)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(maps.len(), 17);
        for o in OBJECTS {
            assert!(o.width * o.height > 1);
            assert!(o.depth > 0.);
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
            form: Form::Room(detail),
            rise_pixels: 24.,
            depth_pixels: h as f32 * 8.,
            front_rows: h as f32,
        };
        append(&mut m, &grid(w, h), &p, &refs, detail);
        assert_eq!(m.footing_heights, vec![0.; w * h]);
        m
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
            Surface::Stone,
            Surface::Ceramic,
            Surface::Timber,
            Surface::Tatami,
            Surface::Facility,
        ] {
            let m = rendered(Detail::Surface(s), 2, 2);
            assert!(m.solid.positions.iter().all(|p| p[1] <= 0.002));
        }
    }
    #[test]
    fn special_room_thresholds_and_tracks_have_one_complete_floor_surface() {
        for detail in [Detail::Threshold, Detail::FloorBand, Detail::ShutterTrack] {
            let mesh = rendered(detail, 2, 2);
            let mut floor_area = 0.;
            for tri in mesh.solid.indices.chunks_exact(3) {
                let [a, b, c] = [tri[0], tri[1], tri[2]]
                    .map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
                let cross = (b - a).cross(c - a);
                if [a, b, c].iter().all(|p| p.y.abs() < 0.0001) && cross.y > 0. {
                    floor_area += cross.y * 0.5;
                }
            }
            // An overlapping underlay adds almost another full plot of area;
            // missing floor geometry reduces it. Raised grip strips are above
            // this plane, and rendered() separately checks unchanged footing.
            assert!(
                (floor_area - 16. * 16.).abs() < 0.001,
                "{detail:?}: {floor_area}"
            );
            assert!(mesh.solid.positions.iter().any(|p| p[1] > 0.));
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
            form: Form::Room(Detail::LinkPlatform { trade: false }),
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
