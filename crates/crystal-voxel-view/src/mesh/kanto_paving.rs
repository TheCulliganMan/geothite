//! Walkable Kanto paving: complete source drawings, live paint, recessed joints.
//! The original collision and standing datum remain authoritative.
use super::*;

const LABEL: &str = "outdoor/kanto-paving";
const BRIDGE_LABEL: &str = "outdoor/kanto-brick-bridge";
const BRICK_PATH_LABEL: &str = "outdoor/kanto-brick-path";
const TOWER_PATH_LABEL: &str = "outdoor/tournament-brick-path";
fn source_drawing(source: &VisualTileSource) -> Option<[u16;16]> {
    match source.tileset_id.as_ref() {
        "kanto" => drawing(source.metatile_id),
        "battle_tower_outside" if source.metatile_id==0x06 => Some([0x2f;16]),
        _ => None,
    }
}
fn drawing(block: u16) -> Option<[u16; 16]> {
    Some(match block {
        // The authored brick deck retains its original LADDER permission.
        0x05 => [0x4b; 16],
        // City paths use a distinct two-course drawing, not the bridge glyph.
        0x55 => [0x5b; 16],
        0x22 => [
            0x21, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x21, 0x23,
            0x23, 0x10,
        ],
        0x23 => [
            0x20, 0x20, 0x20, 0x20, 0x23, 0x23, 0x23, 0x23, 0x23, 0x23, 0x23, 0x23, 0x20, 0x20,
            0x20, 0x20,
        ],
        0x26 => [
            0x10, 0x23, 0x23, 0x21, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23,
            0x23, 0x21,
        ],
        0x27 => [
            0x10, 0x23, 0x23, 0x10, 0x10, 0x23, 0x23, 0x10, 0x10, 0x23, 0x23, 0x10, 0x10, 0x23,
            0x23, 0x10,
        ],
        0x44 => [
            0x10, 0x23, 0x23, 0x21, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x21, 0x20,
            0x20, 0x20,
        ],
        0x45 => [
            0x21, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x20, 0x20,
            0x20, 0x21,
        ],
        0x48 => [
            0x20, 0x20, 0x20, 0x21, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x21, 0x23,
            0x23, 0x10,
        ],
        0x49 => [
            0x21, 0x20, 0x20, 0x20, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23,
            0x23, 0x21,
        ],
        0x5d => [
            0x21, 0x23, 0x23, 0x21, 0x10, 0x23, 0x23, 0x10, 0x10, 0x23, 0x23, 0x10, 0x10, 0x23,
            0x23, 0x10,
        ],
        0x5e => [
            0x10, 0x23, 0x23, 0x10, 0x10, 0x23, 0x23, 0x10, 0x10, 0x23, 0x23, 0x10, 0x21, 0x23,
            0x23, 0x21,
        ],
        _ => return None,
    })
}

// Authenticate the existing outline art by palette equality, so time-of-day
// colors can change without accepting a changed drawing or transparent tile.
fn ink(tile: u16, x: usize, y: usize) -> usize {
    match tile {
        0x4b | 0x2f => {
            [
                "00000000", "11111110", "11111110", "00000000", "11101111", "11101111", "00000000",
                "01111110",
            ][y]
                .as_bytes()[x] as usize
                - b'0' as usize
        }
        0x5b => {
            [
                "11111111", "00010000", "00010000", "00010000", "11111111", "00000001", "00000001",
                "00000001",
            ][y]
                .as_bytes()[x] as usize
                - b'0' as usize
        }
        0x10 => [1, 1, 3, 0, 2, 3, 2, 1][x],
        0x20 => match y {
            0 | 1 | 7 => 1,
            2 => 3,
            3 => 0,
            4 => 2,
            5 => {
                if x < 2 || x > 4 {
                    3
                } else {
                    2
                }
            }
            _ => {
                if (2..6).contains(&x) {
                    3
                } else {
                    2
                }
            }
        },
        0x21 => {
            [
                "11333311", "13200231", "32000023", "32000023", "31200213", "33122133", "13300332",
                "11333311",
            ][y]
                .as_bytes()[x] as usize
                - b'0' as usize
        }
        _ => 0,
    }
}
fn palette(
    cells: &[&VisualTile],
    indices: &[usize; 16],
    images: &TerrainImageSamples,
) -> Option<[[u8; 4]; 4]> {
    let mut colors = [None; 4];
    for &i in indices {
        let rgba = tile_rgba(images, cells[i]).ok()?;
        if rgba.len() != 256 {
            return None;
        }
        for (p, bytes) in rgba.chunks_exact(4).enumerate() {
            let color: [u8; 4] = bytes.try_into().ok()?;
            if color[3] != 255 {
                return None;
            }
            let slot = &mut colors[ink(cells[i].source.tile_index, p % 8, p / 8)];
            if slot.is_some_and(|old| old != color) {
                return None;
            }
            *slot = Some(color);
        }
    }
    if matches!(cells[indices[0]].source.metatile_id, 0x05 | 0x55)
        || cells[indices[0]].source.tileset_id.as_ref()=="battle_tower_outside" {
        let a = colors[0]?;
        let b = colors[1]?;
        return (a != b).then_some([a, b, a, b]);
    }
    let colors = colors.map(|c| c.unwrap_or([0; 4]));
    if colors.iter().any(|c| c[3] != 255)
        || (0..4).any(|a| (a + 1..4).any(|b| colors[a] == colors[b]))
    {
        return None;
    }
    Some(colors)
}
fn paint(rgba: [u8; 4], gain: f32) -> [f32; 4] {
    let linear = |v: u8| {
        let s = f32::from(v) / 255.;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    [
        linear(rgba[0]) * gain,
        linear(rgba[1]) * gain,
        linear(rgba[2]) * gain,
        1.,
    ]
}
fn land_brick_paint(rgba: [u8; 4], gain: f32) -> [f32; 4] {
    let mut color = paint(rgba, gain);
    let luminance = color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722;
    // Translate the pale source field into warm mineral paving. Keep its live
    // day/night illumination without turning the broad street into mint enamel.
    for (channel, warmth) in color[..3].iter_mut().zip([1., 0.97, 0.88]) {
        *channel = *channel * 0.25 + luminance * warmth * 0.75;
    }
    color
}
fn quad(out: &mut SurfaceMeshData, p: [[f32; 3]; 4], color: [f32; 4]) {
    let normal = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
        .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]))
        .normalize();
    // Terrain is palette-preserving/unlit. Match the existing exterior kit's
    // baked directional finish so chamfers read as volume in that material.
    let shade = 0.62 + 0.38 * normal.dot(Vec3::new(-0.35, 0.85, 0.4).normalize()).max(0.);
    let mut color = color;
    for c in &mut color[..3] {
        *c *= shade;
    }
    // These colors already contain the current source time-of-day palette.
    // Tag bevels and grout too: untagged faces receive a second moon tint.
    append_quad(
        out, p, normal.to_array(),
        [crate::scenery_finish::SOURCE_PAINT_UV; 4], color,
    );
}
fn slab(out: &mut SurfaceMeshData, b: [f32; 4], color: [f32; 4], scale: f32) {
    let [w, e, n, s] = b;
    let gap = scale * 0.009;
    let bevel = scale * 0.025;
    let low = -scale * 0.018;
    let outer = [
        [w + gap, low, n + gap],
        [w + gap, low, s - gap],
        [e - gap, low, s - gap],
        [e - gap, low, n + gap],
    ];
    let inner = [
        [w + gap + bevel, 0., n + gap + bevel],
        [w + gap + bevel, 0., s - gap - bevel],
        [e - gap - bevel, 0., s - gap - bevel],
        [e - gap - bevel, 0., n + gap + bevel],
    ];
    quad(out, inner, color);
    for j in 0..4 {
        let k = (j + 1) % 4;
        quad(out, [outer[j], outer[k], inner[k], inner[j]], color);
    }
}

// Clip a whole staggered brick, including its bevels, at the source-cell edge.
// Cutting a half-brick adds no false mortar line or bevel at that clipping seam.
fn clipped_slab(
    out: &mut SurfaceMeshData,
    b: [f32; 4],
    clip: [f32; 4],
    color: [f32; 4],
    scale: f32,
) {
    let mut whole = SurfaceMeshData::default();
    slab(&mut whole, b, color, scale);
    for (q, p) in whole.positions.chunks_exact(4).enumerate() {
        let mut points = p.iter().copied().map(Vec3::from_array).collect::<Vec<_>>();
        for (axis, bound, sign) in [
            (0, clip[0], 1.),
            (0, clip[1], -1.),
            (2, clip[2], 1.),
            (2, clip[3], -1.),
        ] {
            let mut clipped = Vec::new();
            for j in 0..points.len() {
                let a = points[j];
                let z = points[(j + 1) % points.len()];
                let da = (a[axis] - bound) * sign;
                let dz = (z[axis] - bound) * sign;
                if da >= 0. {
                    clipped.push(a);
                }
                if (da >= 0.) != (dz >= 0.) {
                    let mut p = a.lerp(z, da / (da - dz));
                    p[axis] = bound;
                    clipped.push(p);
                }
            }
            points = clipped;
        }
        if points.len() < 3 {
            continue;
        }
        // Directed ground interpolates separately across the source cell's
        // NW-SE diagonal. A brick triangle spanning that crease can sink
        // beneath its bedding on a graded bypass, exposing white triangles.
        // Preserve the same two planar domains before the shared deformation.
        for sign in [-1., 1.] {
            let distance = |p: Vec3| {
                sign * ((p.x - clip[0]) / (clip[1] - clip[0])
                    - (p.z - clip[2]) / (clip[3] - clip[2]))
            };
            let mut half = Vec::new();
            for j in 0..points.len() {
                let a = points[j];
                let b = points[(j + 1) % points.len()];
                let da = distance(a);
                let db = distance(b);
                if da >= 0. {
                    half.push(a);
                }
                if (da >= 0.) != (db >= 0.) {
                    let mut p = a.lerp(b, da / (da - db));
                    p.z = clip[2] + (p.x - clip[0]) * (clip[3] - clip[2]) / (clip[1] - clip[0]);
                    half.push(p);
                }
            }
            if half.len() < 3 {
                continue;
            }
            let vertices = half
                .into_iter()
                .map(|p| (p.to_array(), crate::scenery_finish::SOURCE_PAINT_UV))
                .collect::<Vec<_>>();
            append_polygon(out, &vertices, whole.normals[q * 4], whole.colors[q * 4]);
        }
    }
}

fn brick_deck(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    i: usize,
    source: &VisualTileSource,
    colors: [[u8; 4]; 4],
) {
    let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
    let scale = g.tile_width.min(g.tile_height);
    let land = source.tile_index == 0x5b;
    let bridge = source.tile_index == 0x4b
        && matches!(source.tileset_id.as_ref(), "kanto" | "johto");
    let colors = if land {
        [colors[1], colors[0], colors[1], colors[0]]
    } else {
        colors
    };
    quad(
        out,
        [
            [w, -scale * 0.02, n],
            [w, -scale * 0.02, s],
            [e, -scale * 0.02, s],
            [e, -scale * 0.02, n],
        ],
        if bridge {
            // Native gold and white describe the bond, not metallic bricks
            // with luminous mortar. Use the shared live mineral pigment and
            // derive recessed joints from the brick's own exposure.
            land_brick_paint(colors[1], 0.54)
        } else { paint(colors[0], 0.76) },
    );
    let rows = if land { 2 } else { 3 };
    let height = (s - n) / rows as f32;
    for row in 0..rows {
        let phase = i32::from(source.subtile_row) * rows + row;
        let shifted = if land { phase % 2 == 0 } else { phase % 2 != 0 };
        let offset = if shifted { (e - w) * 0.5 } else { 0. };
        for col in -1..1 {
            let x = w + col as f32 * (e - w) + offset;
            // Clipped halves of one brick share pigment, even at a native
            // metatile phase wrap. Viewport translations cannot repaint it.
            let brick = (i32::from(source.subtile_column) + col).rem_euclid(4);
            let gain = 0.91 + (brick + phase).rem_euclid(3) as f32 * 0.015;
            clipped_slab(
                out,
                [
                    x,
                    x + e - w,
                    n + row as f32 * height,
                    n + (row + 1) as f32 * height,
                ],
                [w, e, n, s],
                if land {
                    land_brick_paint(colors[1], gain * 0.86)
                } else if bridge {
                    land_brick_paint(colors[1], gain * 0.82)
                } else {
                    paint(colors[1], gain)
                },
                scale,
            );
        }
    }
}
fn raised_brick_colors(tile:&VisualTile,images:&TerrainImageSamples)->Option<[[u8;4];4]> {
    let rgba=tile_rgba(images,tile).ok()?;
    if rgba.len()!=256 {return None;}
    let mut colors=[None;2];
    for (p,bytes) in rgba.chunks_exact(4).enumerate() {
        let color:[u8;4]=bytes.try_into().ok()?;
        let slot=&mut colors[ink(0x4b,p%8,p/8)];
        if color[3]!=255 || slot.is_some_and(|old|old!=color) {return None;}
        *slot=Some(color);
    }
    let [Some(a),Some(b)]=colors else {return None;};
    (a!=b).then_some([a,b,a,b])
}

/// The suspended masonry deck continues around its shallow structural edge.
/// Live native paint owns both the deck and fascia; timber piers keep their frame.
pub(super) fn raised_brick_side(out:&mut SurfaceMeshData,tile:&VisualTile,
    images:&TerrainImageSamples,face:[[f32;3];4],normal:[f32;3],scale:f32)->bool {
    let Some(colors)=raised_brick_colors(tile,images) else {return false;};
    // Bridge side winding starts along its bottom edge; the shared beveled
    // masonry producer starts up its left edge. Rotate without reversing it.
    let p=[face[1],face[2],face[3],face[0]].map(Vec3::from_array);
    let first=out.positions.len();
    for j in 0..2 {
        let a=j as f32/2.;let b=(j+1)as f32/2.;
        let q=[p[0].lerp(p[3],a),p[1].lerp(p[2],a),
            p[1].lerp(p[2],b),p[0].lerp(p[3],b)].map(|p|p.to_array());
        beveled_masonry_face(out,q,normal,land_brick_paint(colors[1],0.54),
            land_brick_paint(colors[1],0.82*0.94),scale);
    }
    // The fascia is already fitted to its deck, including bevels projecting
    // across a shore contact. Land shaping must not lift it a second time.
    out.uvs[first..].fill(crate::scenery_finish::LEVEL_PAVING_UV);
    true
}

// Kanto 4b and Johto 07 share this exact two-ink brick drawing.
// Reuse its modeled joints at the deck datum instead of flattening the art.
pub(super) fn raised_brick_deck(out:&mut SurfaceMeshData,g:&GridGeometry,i:usize,
    tile:&VisualTile,images:&TerrainImageSamples,heights:[f32;4])->bool {
    let Some(colors)=raised_brick_colors(tile,images) else {return false;};
    let mut source=tile.source.clone();source.tile_index=0x4b;
    let first=out.positions.len();brick_deck(out,g,i,&source,colors);
    let (w,e,n,s)=g.bounds(i%g.width,i/g.width);
    for (point,normal) in out.positions[first..].iter_mut().zip(&mut out.normals[first..]) {
        let x=(point[0]-w)/(e-w);let z=(point[2]-n)/(s-n);
        let north=heights[0]*(1.-x)+heights[3]*x;
        let south=heights[1]*(1.-x)+heights[2]*x;
        point[1]+=north*(1.-z)+south*z;
        let dx=((heights[3]-heights[0])*(1.-z)+(heights[2]-heights[1])*z)/(e-w);
        let dz=(south-north)/(s-n);
        *normal=Vec3::new(normal[0]-normal[1]*dx,normal[1],normal[2]-normal[1]*dz).normalize().to_array();
    }
    out.uvs[first..].fill(crate::scenery_finish::LEVEL_PAVING_UV);
    true
}
fn brick_water_face(
    out: &mut SurfaceMeshData,
    p: [[f32; 3]; 4],
    normal: [f32; 3],
    colors: [[u8; 4]; 4],
    scale: f32,
) {
    beveled_masonry_face(out,p,normal,paint(colors[0],0.72),paint(colors[1],0.94),scale);
}
fn beveled_masonry_face(out:&mut SurfaceMeshData,p:[[f32;3];4],normal:[f32;3],
    grout:[f32;4],stone:[f32;4],scale:f32) {
    let outer = p.map(Vec3::from_array);
    let n = Vec3::from_array(normal);
    let tangent = (outer[3] - outer[0]).normalize();
    let gap = scale * 0.018;
    let inner = [
        outer[0] + tangent * gap + Vec3::Y * gap + n * gap,
        outer[1] + tangent * gap - Vec3::Y * gap + n * gap,
        outer[2] - tangent * gap - Vec3::Y * gap + n * gap,
        outer[3] - tangent * gap + Vec3::Y * gap + n * gap,
    ];
    quad(out, p, grout);
    quad(
        out,
        inner.map(|point| point.to_array()),
        stone,
    );
    for j in 0..4 {
        let k = (j + 1) % 4;
        quad(
            out,
            [
                outer[j].to_array(),
                outer[k].to_array(),
                inner[k].to_array(),
                inner[j].to_array(),
            ],
            stone,
        );
    }
}
fn append(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    i: usize,
    source: &VisualTileSource,
    colors: [[u8; 4]; 4],
) {
    let tile = source.tile_index;
    if source.tileset_id.as_ref()=="battle_tower_outside" && tile==0x2f {
        // Same native two-ink bond as harbor brick, using its shared solid
        // geometry. Live gold pigment becomes muted ochre stone and grout.
        let mineral=|c:[u8;4]| {
            let l=0.2126*c[0] as f32+0.7152*c[1] as f32+0.0722*c[2] as f32;
            [0,1,2,3].map(|j|if j==3 {255} else {
                (c[j] as f32*0.45+l*[1.,0.97,0.88][j]*0.55).round() as u8
            })
        };
        let stone=mineral(colors[1]);
        let grout=std::array::from_fn(|j|if j==3 {255} else {
            (stone[j] as f32*0.48+mineral(colors[0])[j] as f32*0.12).round() as u8
        });
        let mut shared=source.clone();shared.tile_index=0x4b;
        brick_deck(out,g,i,&shared,[grout,stone,grout,stone]);
        return;
    }
    if matches!(tile, 0x4b | 0x5b) {
        brick_deck(out, g, i, source, colors);
        return;
    }
    let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
    let scale = g.tile_width.min(g.tile_height);
    if tile == 0x23 {
        // This native field is blank, not a tile bond. Carry continuous mineral
        // paving across source cells; authored curbs and covers keep their trim.
        quad(out,[[w,0.,n],[w,0.,s],[e,0.,s],[e,0.,n]],
            land_brick_paint(colors[0],0.66));
        return;
    }
    quad(out,[[w,-scale*0.02,n],[w,-scale*0.02,s],
        [e,-scale*0.02,s],[e,-scale*0.02,n]],paint(colors[1],0.78));
    if tile == 0x21 {
        let center = Vec3::new((w + e) * 0.5, 0., (n + s) * 0.5);
        let ring: Vec<_> = (0..16)
            .map(|j| {
                let a = -(j as f32) * std::f32::consts::TAU / 16.;
                center + Vec3::new(a.cos() * (e - w) * 0.46, 0., a.sin() * (s - n) * 0.46)
            })
            .collect();
        let top: Vec<_> = ring.iter().map(|p| center + (*p - center) * 0.92).collect();
        let inner: Vec<_> = top.iter().map(|p| center + (*p - center) * 0.66).collect();
        let vertices: Vec<_> = inner.iter()
            .map(|p| (p.to_array(), crate::scenery_finish::SOURCE_PAINT_UV)).collect();
        append_polygon(out, &vertices, [0., 1., 0.], paint(colors[0], 0.90));
        for j in 0..16 {
            let k = (j + 1) % 16;
            quad(
                out,
                [
                    top[j].to_array(),
                    top[k].to_array(),
                    inner[k].to_array(),
                    inner[j].to_array(),
                ],
                paint(colors[1], 0.96),
            );
            quad(
                out,
                [
                    (ring[j] - Vec3::Y * scale * 0.018).to_array(),
                    (ring[k] - Vec3::Y * scale * 0.018).to_array(),
                    top[k].to_array(),
                    top[j].to_array(),
                ],
                paint(colors[1], 0.96),
            );
        }
    } else {
        slab(out, [w, e, n, s], paint(colors[1], 0.96), scale);
    }
}

/// Carry an adjacent, complete native brick street under a replacement Kanto
/// cottage or brick building. The exposed plot border uses the same paving producer as the
/// street. Cutaways can reveal the centre, so it receives finished paving too.
pub(super) fn building_underlay(out: &mut SurfaceMeshData, cells: &[&VisualTile],
    shapes: &[CellShape], g: &GridGeometry, rect: [usize;4],
    images: &TerrainImageSamples, origin: [i32;2]) -> bool {
    let [x,y,w,h]=rect;
    if x+w>g.width || y+h>g.height || cells.len()!=g.width*g.height
        || shapes.len()!=cells.len() {return false;}
    let mut colors=None;
    'search: for yy in y.saturating_sub(1)..(y+h+1).min(g.height) {
        for xx in x.saturating_sub(1)..(x+w+1).min(g.width) {
            if (x..x+w).contains(&xx) && (y..y+h).contains(&yy) {continue;}
            let s=&cells[yy*g.width+xx].source;
            if s.tileset_id.as_ref()!="kanto" || s.metatile_id!=0x55 || s.tile_index!=0x5b
                || s.subtile_column>3 || s.subtile_row>3 {continue;}
            let (Some(bx),Some(by))=(xx.checked_sub(s.subtile_column as usize),yy.checked_sub(s.subtile_row as usize)) else {continue;};
            if bx+4>g.width || by+4>g.height {continue;}
            let indices=std::array::from_fn(|j|(by+j/4)*g.width+bx+j%4);
            if indices.iter().enumerate().any(|(j,&i)| {
                let s=&cells[i].source;
                s.tileset_id.as_ref()!="kanto" || s.metatile_id!=0x55 || s.tile_index!=0x5b
                    || s.subtile_column as usize!=j%4 || s.subtile_row as usize!=j/4
                    || !matches!(shapes[i],CellShape::Flat|CellShape::PlaneAt{height:0.})
            }) {continue;}
            if let Some(paint)=palette(cells,&indices,images) {colors=Some(paint);break 'search;}
        }
    }
    let Some(colors)=colors else {return false;};
    for yy in y..y+h {for xx in x..x+w {
        let i=yy*g.width+xx;
        let source=VisualTileSource{tileset_id:"kanto".into(),metatile_id:0x55,tile_index:0x5b,
            subtile_column:(xx as i32+origin[0]).rem_euclid(4) as u8,
            subtile_row:(yy as i32+origin[1]).rem_euclid(4) as u8};
        brick_deck(out,g,i,&source,colors);
    }}
    true
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    images: &TerrainImageSamples,
) {
    finish_at_origin(mesh,cells,shapes,g,images,[0,0]);
}

pub(super) fn finish_at_origin(mesh:&mut TerrainMeshData,cells:&[&VisualTile],
    shapes:&[CellShape],g:&GridGeometry,images:&TerrainImageSamples,origin:[i32;2]) {
    let old = &mesh.textured;
    if g.width == 0
        || cells.len() != g.width * g.height
        || shapes.len() != cells.len()
        || old.positions.len() % 4 != 0
        || old.indices.len() != old.positions.len() / 4 * 6
        || old.indices.chunks_exact(6).enumerate().any(|(q, indices)| {
            let b = (q * 4) as u32;
            indices != [b, b + 1, b + 2, b, b + 2, b + 3]
        })
    {
        return;
    }
    let mut faces = vec![None; cells.len()];
    let mut face_counts = vec![0; cells.len()];
    let mut bridge_walls = vec![Vec::new(); cells.len()];
    for (q, (p, uv)) in old
        .positions
        .chunks_exact(4)
        .zip(old.uvs.chunks_exact(4))
        .enumerate()
    {
        let Some(i) = new_bark::sampled_cell(uv, g) else {
            continue;
        };
        if cells[i].source.tileset_id.as_ref() == "kanto"
            && cells[i].source.metatile_id == 0x05
            && cells[i].source.tile_index == 0x4b
            && old.normals[q * 4][1] == 0.
            && old.normals[q * 4..q * 4 + 4]
                .iter()
                .all(|n| *n == old.normals[q * 4])
            && kanto_shore::native_face(p, old.normals[q * 4], uv, i, shapes, g)
        {
            bridge_walls[i].push(q);
        }
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        let expected = [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]];
        if p.iter().all(|p| expected.contains(p))
            && expected.iter().all(|v| p.contains(v))
            && old.normals[q * 4..q * 4 + 4].iter().all(|n| n[1] > 0.999)
        {
            faces[i] = Some(q);
            face_counts[i] += 1;
        }
    }
    let mut removed = std::collections::HashSet::new();
    for start in 0..cells.len() {
        let first = &cells[start].source;
        if first.subtile_column != 0
            || first.subtile_row != 0
        {
            continue;
        }
        let Some(drawing) = source_drawing(first) else {
            continue;
        };
        let x = start % g.width;
        let y = start / g.width;
        if x + 4 > g.width || y + 4 > g.height {
            continue;
        }
        let indices = std::array::from_fn(|j| (y + j / 4) * g.width + x + j % 4);
        if indices.iter().enumerate().any(|(j, &i)| {
            let s = &cells[i].source;
            let mut wall_normals = std::collections::HashSet::new();
            s.tileset_id != first.tileset_id
                || s.metatile_id != first.metatile_id
                || s.subtile_column as usize != j % 4
                || s.subtile_row as usize != j / 4
                || s.tile_index != drawing[j]
                || !matches!(
                    shapes[i],
                    CellShape::Flat | CellShape::PlaneAt { height: 0. }
                )
                || faces[i].is_none()
                || face_counts[i] != 1
                || faces[i].is_some_and(|q| {
                    mesh.cutaway_links
                        .iter()
                        .any(|&(_, anchor)| anchor / 4 == q)
                })
                || mesh.authored_cells.get(i).is_none_or(|c| c.is_some())
                || bridge_walls[i]
                    .iter()
                    .any(|&q| mesh.cutaway_links.iter().any(|&(_, a)| a / 4 == q))
                || bridge_walls[i]
                    .iter()
                    .any(|&q| !wall_normals.insert(old.normals[q * 4].map(f32::to_bits)))
        }) {
            continue;
        }
        let Some(colors) = palette(cells, &indices, images) else {
            continue;
        };
        for &i in &indices {
            let first_vertex=mesh.solid.positions.len();
            append(&mut mesh.solid, g, i, &cells[i].source, colors);
            if first.metatile_id==0x55 || first.tileset_id.as_ref()=="battle_tower_outside"
                || cells[i].source.tile_index==0x23 {
                let base=if cells[i].source.tile_index==0x23 {
                    crate::scenery_finish::ROAD_GRAIN_BASE
                } else {crate::scenery_finish::PAVING_GRAIN_BASE};
                for v in first_vertex..mesh.solid.positions.len() {
                    let p=mesh.solid.positions[v];
                    mesh.solid.uvs[v]=[
                        base[0]+origin[0] as f32
                            +(p[0]-g.origin_x)/g.tile_width,
                        base[1]-origin[1] as f32
                            -(p[2]-g.origin_z)/g.tile_height,
                    ];
                }
            }
            mesh.authored_cells[i] = Some(if first.tileset_id.as_ref()=="battle_tower_outside" {
                TOWER_PATH_LABEL
            } else {match first.metatile_id {
                0x05 => BRIDGE_LABEL,
                0x55 => BRICK_PATH_LABEL,
                _ => LABEL,
            }});
            removed.insert(faces[i].unwrap());
            for &q in &bridge_walls[i] {
                brick_water_face(
                    &mut mesh.solid,
                    old.positions[q * 4..q * 4 + 4].try_into().unwrap(),
                    old.normals[q * 4],
                    colors,
                    g.tile_width.min(g.tile_height),
                );
                removed.insert(q);
            }
        }
    }
    if removed.is_empty() {
        return;
    }
    let old = std::mem::take(&mut mesh.textured);
    for (_, anchor) in &mut mesh.cutaway_links {
        *anchor -= removed.iter().filter(|&&q| q < *anchor / 4).count() * 4;
    }
    let mut marked = vec![false; old.positions.len()];
    for r in &old.cutaway_ranges {
        marked[r.clone()].fill(true);
    }
    let mut remap = vec![u32::MAX; old.positions.len()];
    mesh.textured = select_surface_indices(
        &old,
        &marked,
        &mut remap,
        old.indices
            .chunks_exact(6)
            .enumerate()
            .filter(|(q, _)| !removed.contains(q))
            .flat_map(|(_, indices)| indices.iter().copied()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    const COLORS: [[u8; 4]; 4] = [
        [238, 246, 220, 255],
        [142, 156, 150, 255],
        [74, 94, 83, 255],
        [22, 32, 28, 255],
    ];
    fn fixture(
        block: u16,
    ) -> (
        Vec<VisualTile>,
        Vec<CellShape>,
        GridGeometry,
        TerrainImageSamples,
        TerrainMeshData,
    ) {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -16.,
            origin_z: 24.,
        };
        let mut assets = Assets::<Image>::default();
        let mut samples = TerrainImageSamples::default();
        let mut cells = Vec::new();
        let mut mesh = TerrainMeshData::default();
        for (i, tile) in drawing(block).unwrap().into_iter().enumerate() {
            let texture = assets.add(Image::default());
            let rgba = (0..64)
                .flat_map(|p| COLORS[ink(tile, p % 8, p / 8)])
                .collect();
            samples
                .pixels
                .insert(texture.id(), TileImageSample::Rgba(rgba));
            cells.push(VisualTile {
                column: (i % 4) as u32,
                row: (i / 4) as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from("kanto"),
                    metatile_id: block,
                    subtile_column: (i % 4) as u8,
                    subtile_row: (i / 4) as u8,
                    tile_index: tile,
                },
                texture,
                animation_frames: None,
                priority: false,
            });
            let (w, e, n, s) = g.bounds(i % 4, i / 4);
            append_top_shaded(
                &mut mesh.textured,
                [w, e, n, s],
                0.,
                g.uv(i % 4, i / 4),
                [1.; 4],
            );
        }
        mesh.footing_heights = vec![0.; 16];
        mesh.authored_cells = vec![None; 16];
        let shapes = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        (cells, shapes, g, samples, mesh)
    }
    #[test]
    fn tournament_walkway_reuses_brick_geometry_without_changing_support_or_unknown_art() {
        let (mut cells,_,g,images,original)=fixture(5);
        for cell in &mut cells {
            cell.source.tileset_id="battle_tower_outside".into();
            cell.source.metatile_id=6;cell.source.tile_index=0x2f;
        }
        let shapes=vec![CellShape::Flat;16];
        let mut mesh=original.clone();
        finish_at_origin(&mut mesh,&cells.iter().collect::<Vec<_>>(),&shapes,&g,&images,[16,20]);
        assert!(mesh.textured.indices.is_empty());
        assert!(mesh.authored_cells.iter().all(|v|*v==Some(TOWER_PATH_LABEL)));
        assert_eq!(mesh.footing_heights,original.footing_heights);
        assert!(mesh.solid.positions.iter().all(|p|p[1]<=0. && p[1]>=-0.16));
        let refs=cells.iter().collect::<Vec<_>>();
        let colors=palette(&refs,&std::array::from_fn(|i|i),&images).unwrap();
        let mut shared=SurfaceMeshData::default();
        for i in 0..16 {
            let mut source=cells[i].source.clone();source.tile_index=0x4b;
            brick_deck(&mut shared,&g,i,&source,colors);
        }
        assert_eq!(mesh.solid.positions,shared.positions);
        assert_eq!(mesh.solid.indices,shared.indices);
        assert!(mesh.solid.uvs.iter().all(|uv|uv[0]>=83000. && uv[1]< -3000.));
        for altered in 0..3 {
            let mut rejected_cells=cells.clone();let mut rejected_images=images.clone();
            match altered {
                0 => rejected_cells[0].source.tile_index=0x30,
                1 => for c in &mut rejected_cells {c.source.tileset_id="johto".into();},
                _ => if let TileImageSample::Rgba(p)=rejected_images.pixels.get_mut(&cells[0].texture.id()).unwrap(){p[0]^=1;},
            }
            let mut rejected=original.clone();
            finish(&mut rejected,&rejected_cells.iter().collect::<Vec<_>>(),&shapes,&g,&rejected_images);
            assert_eq!(rejected,original,"unproven drawing must retain its native art");
        }
    }
    #[test]
    fn elevated_pier_bricks_preserve_native_joints_and_follow_the_shore_ramp() {
        let (cells,_,g,mut images,_)=fixture(5);
        let mut baseline=SurfaceMeshData::default();
        let refs=cells.iter().collect::<Vec<_>>();
        let indices=std::array::from_fn(|i|i);
        let colors=palette(&refs,&indices,&images).unwrap();
        brick_deck(&mut baseline,&g,0,&cells[0].source,colors);
        for heights in [[8.;4],[0.,4.,4.,0.]] {
            let mut raised=SurfaceMeshData::default();
            let mut johto=cells[0].clone();johto.source.tileset_id="johto".into();johto.source.tile_index=7;
            assert!(raised_brick_deck(&mut raised,&g,0,&johto,&images,heights));
            assert_eq!(raised.indices,baseline.indices);assert_eq!(raised.colors,baseline.colors);
            let (_,_,n,s)=g.bounds(0,0);
            for (old,new) in baseline.positions.iter().zip(&raised.positions) {
                let z=(old[2]-n)/(s-n);let offset=heights[0]*(1.-z)+heights[1]*z;
                assert!((new[1]-old[1]-offset).abs()<0.0001);
                assert_eq!([new[0],new[2]],[old[0],old[2]]);
            }
            for tri in raised.indices.chunks_exact(3) {
                let p=tri.iter().map(|i|Vec3::from_array(raised.positions[*i as usize])).collect::<Vec<_>>();
                let winding=(p[1]-p[0]).cross(p[2]-p[0]);
                if winding.length()>0.00001 {assert!(winding.dot(Vec3::from_array(raised.normals[tri[0]as usize]))>0.);}
            }
        }
        if let TileImageSample::Rgba(rgba)=images.pixels.get_mut(&cells[0].texture.id()).unwrap() {rgba[0]^=1;}
        let mut rejected=SurfaceMeshData::default();
        assert!(!raised_brick_deck(&mut rejected,&g,0,&cells[0],&images,[8.;4]));
        assert!(rejected.positions.is_empty());
    }
    #[test]
    fn brick_building_underlay_uses_only_complete_adjacent_native_paving() {
        let (base,_,g,images,_)=fixture(0x55);
        let g=GridGeometry{width:8,..g};let mut cells=Vec::new();
        for y in 0..4 {for x in 0..8 {
            let mut cell=base[y*4+x%4].clone();cell.column=x as u32;
            if x>=4 {cell.source.metatile_id=0x20;cell.source.tile_index=0x4c;}
            cells.push(cell);
        }}
        let shapes=vec![CellShape::Flat;cells.len()];let original=cells.clone();
        let mut out=SurfaceMeshData::default();
        assert!(building_underlay(&mut out,&cells.iter().collect::<Vec<_>>(),&shapes,&g,[4,0,4,4],&images,[17,9]));
        assert!(!out.indices.is_empty());assert_eq!(cells,original);
        let (w,_,n,_)=g.bounds(4,0);let (_,e,_,s)=g.bounds(7,3);
        assert!(out.positions.iter().all(|p|p[0]>=w-0.001 && p[0]<=e+0.001 && p[2]>=n-0.001 && p[2]<=s+0.001));
        for yy in 1..3 {for xx in 5..7 {
            let (w,e,n,s)=g.bounds(xx,yy);
            assert!(out.positions.iter().any(|p|p[0]>w && p[0]<e && p[2]>n && p[2]<s),
                "cutaways must reveal finished bricks inside the plot too");
        }}
        let mut rejected=SurfaceMeshData::default();
        assert!(!building_underlay(&mut rejected,&cells.iter().collect::<Vec<_>>(),&shapes,&g,[5,0,3,4],&images,[17,9]));
        assert!(rejected.indices.is_empty());
        cells[0].source.tile_index=0x5c;
        assert!(!building_underlay(&mut rejected,&cells.iter().collect::<Vec<_>>(),&shapes,&g,[4,0,4,4],&images,[17,9]));
        assert!(rejected.positions.is_empty());
    }

    fn bridge_fixture() -> (
        Vec<VisualTile>,
        Vec<CellShape>,
        GridGeometry,
        TerrainImageSamples,
        TerrainMeshData,
    ) {
        let g = GridGeometry {
            width: 6,
            height: 6,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -24.,
            origin_z: 32.,
        };
        let mut assets = Assets::<Image>::default();
        let mut images = TerrainImageSamples::default();
        let mut cells = Vec::new();
        let mut shapes = Vec::new();
        let mut mesh = TerrainMeshData::default();
        for i in 0..36 {
            let x = i % 6;
            let y = i / 6;
            let bridge = (1..5).contains(&x) && (1..5).contains(&y);
            let tile = if bridge { 0x4b } else { 0x14 };
            let texture = assets.add(Image::default());
            let rgba = (0..64)
                .flat_map(|p| {
                    if bridge {
                        COLORS[ink(tile, p % 8, p / 8)]
                    } else {
                        [30, 70, 100, 255]
                    }
                })
                .collect();
            images
                .pixels
                .insert(texture.id(), TileImageSample::Rgba(rgba));
            cells.push(VisualTile {
                column: x as u32,
                row: y as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from("kanto"),
                    metatile_id: if bridge { 5 } else { 0x43 },
                    subtile_column: if bridge { (x - 1) as u8 } else { 0 },
                    subtile_row: if bridge { (y - 1) as u8 } else { 0 },
                    tile_index: tile,
                },
                texture,
                animation_frames: None,
                priority: false,
            });
            shapes.push(if bridge {
                CellShape::Flat
            } else {
                CellShape::Water
            });
            mesh.footing_heights.push(shapes[i].surface_height(8.));
            mesh.authored_cells.push(None);
            let (w, e, n, s) = g.bounds(x, y);
            append_top_shaded(
                &mut mesh.textured,
                [w, e, n, s],
                shapes[i].surface_height(8.),
                g.uv(x, y),
                [1.; 4],
            );
        }
        for i in 0..36 {
            if cells[i].source.metatile_id != 5 {
                continue;
            }
            let x = i % 6;
            let y = i / 6;
            let (w, e, n, s) = g.bounds(x, y);
            for d in Direction::ALL {
                let (dx, dy) = d.offset();
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0
                    && ny >= 0
                    && nx < 6
                    && ny < 6
                    && shapes[ny as usize * 6 + nx as usize] == CellShape::Water
                {
                    append_textured_shoreline(
                        &mut mesh.textured,
                        &g,
                        x,
                        y,
                        d,
                        [w, e, n, s],
                        -2.,
                        0.,
                    );
                }
            }
        }
        (cells, shapes, g, images, mesh)
    }
    #[test]
    fn city_brick_path_replaces_complete_art_with_two_courses_and_preserves_support() {
        let (cells, shapes, g, images, mut mesh) = fixture(0x55);
        let support = mesh.footing_heights.clone();
        finish(
            &mut mesh,
            &cells.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
            &images,
        );
        assert!(mesh.textured.indices.is_empty());
        assert!(
            mesh.authored_cells
                .iter()
                .all(|&v| v == Some(BRICK_PATH_LABEL))
        );
        assert_eq!(mesh.footing_heights, support);
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| p[1] <= 0. && p[1] >= -0.16)
        );
        // Source55 has two broad pale courses; bridge05 has three smaller
        // courses with the opposite pigment roles. They share construction,
        // but never acquire one another's source silhouette or permissions.
        let source = &cells[0].source;
        let mut one = SurfaceMeshData::default();
        brick_deck(&mut one, &g, 0, source, COLORS);
        let tops = one
            .positions
            .iter()
            .zip(&one.colors)
            .filter(|(p, _)| p[1] == 0.)
            .collect::<Vec<_>>();
        assert!(!tops.is_empty());
        assert!(
            tops.iter().all(|(_, c)| c[0] > paint(COLORS[1], 1.)[0]),
            "pale source brick field"
        );
        let mut bridge = source.clone();
        bridge.metatile_id = 5;
        bridge.tile_index = 0x4b;
        let mut old = SurfaceMeshData::default();
        brick_deck(&mut old, &g, 0, &bridge, COLORS);
        assert_ne!(one.positions, old.positions);
    }

    #[test]
    fn city_mineral_grain_keeps_the_same_map_coordinates_across_streamed_crops() {
        let (cells,shapes,g,images,mut crop)=fixture(0x55);
        let wide_g=GridGeometry {width:8,..g};
        let mut wide=TerrainMeshData {footing_heights:vec![0.;32],
            authored_cells:vec![None;32],..Default::default()};
        let mut wide_cells=Vec::new();let mut wide_shapes=Vec::new();
        for row in 0..4 {for col in 0..8 {
            let mut tile=cells[row*4+col%4].clone();
            tile.column=col as u32;tile.row=row as u32;
            wide_cells.push(tile);wide_shapes.push(shapes[row*4+col%4]);
            let (w,e,n,s)=wide_g.bounds(col,row);
            append_top_shaded(&mut wide.textured,[w,e,n,s],0.,wide_g.uv(col,row),[1.;4]);
        }}
        finish_at_origin(&mut wide,&wide_cells.iter().collect::<Vec<_>>(),
            &wide_shapes,&wide_g,&images,[100,80]);
        let crop_g=GridGeometry {origin_x:g.origin_x+32.,..g};
        for p in &mut crop.textured.positions {p[0]+=32.;}
        finish_at_origin(&mut crop,&cells.iter().collect::<Vec<_>>(),
            &shapes,&crop_g,&images,[104,80]);
        // Ignore the shared bedding edge itself: both adjacent source blocks
        // legitimately own vertices exactly on that boundary.
        let selected=|surface:&SurfaceMeshData|surface.uvs.iter().enumerate().filter_map(|(i,&uv)| {
            let p=crate::scenery_finish::paving_coordinates(uv).unwrap();
            (p[0]>104.001).then_some(i)
        }).collect::<Vec<_>>();
        let right=selected(&wide.solid);let cropped=selected(&crop.solid);
        assert_eq!(right.len(),cropped.len());assert!(right.len()>100);
        for (&i,&j) in right.iter().zip(&cropped) {
            assert_eq!(wide.solid.uvs[i],crop.solid.uvs[j]);
            assert_eq!(wide.solid.colors[i],crop.solid.colors[j]);
            assert_eq!(wide.solid.positions[i],crop.solid.positions[j]);
        }
        assert_eq!(crop.footing_heights,vec![0.;16]);
        assert!(crop.solid.uvs.windows(2).any(|uv|uv[0]!=uv[1]),
            "grain needs material coordinates, not a constant style flag");
        for block in [5,0x22] {
            let (tiles,shapes,g,images,mut control)=fixture(block);
            finish_at_origin(&mut control,&tiles.iter().collect::<Vec<_>>(),
                &shapes,&g,&images,[104,80]);
            for triangle in control.solid.indices.chunks_exact(3) {
                let p=[triangle[0],triangle[1],triangle[2]].map(|i|Vec3::from_array(control.solid.positions[i as usize]));
                if (p[1]-p[0]).cross(p[2]-p[0]).length_squared()<0.00000001 {continue;}
                let center=(p[0]+p[1]+p[2])/3.;
                let i=((center.z-g.origin_z)/g.tile_height).floor() as usize*g.width
                    +((center.x-g.origin_x)/g.tile_width).floor() as usize;
                let plain_road=tiles[i].source.tile_index==0x23;
                assert!(triangle.iter().all(|&i|
                    crate::scenery_finish::paving_coordinates(control.solid.uvs[i as usize]).is_some()==plain_road),
                    "only plain road fields gain grain; bridges and decorative borders retain their material");
            }
        }
    }

    #[test]
    fn city_path_rejects_changed_drawing_paint_claims_or_support() {
        for fault in 0..6 {
            let (mut cells, mut shapes, g, mut images, mut mesh) = fixture(0x55);
            match fault {
                0 => cells[6].source.tile_index = 0x4b,
                1 => cells[6].source.subtile_column = 0,
                2 => mesh.authored_cells[6] = Some("another/asset"),
                3 => shapes[6] = CellShape::PlaneAt { height: 2. },
                4 => {
                    if let TileImageSample::Rgba(p) =
                        images.pixels.get_mut(&cells[6].texture.id()).unwrap()
                    {
                        p[0] ^= 32;
                    }
                }
                _ => mesh.textured.positions[24][1] = 1.,
            }
            let before = mesh.textured.clone();
            finish(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &images,
            );
            assert_eq!(mesh.textured, before, "fault {fault}");
            assert!(mesh.solid.indices.is_empty());
        }
    }

    #[test]
    fn suspended_brick_fascia_follows_sloped_corners_and_live_source_paint() {
        let (cells,_,g,images,_)=bridge_fixture();
        let tile=&cells[7];
        for scale in [0.5,1.,2.] {
            for normal in [[1.,0.,0.],[-1.,0.,0.],[0.,0.,1.],[0.,0.,-1.]] {
                let n=Vec3::from_array(normal);
                let tangent=Vec3::Y.cross(n);
                let bottom=Vec3::new(17.,3.,29.);
                let end=bottom+tangent*8.*scale+Vec3::Y*0.75*scale;
                let face=[bottom,end,end+Vec3::Y*2.*scale,bottom+Vec3::Y*2.*scale].map(|p|p.to_array());
                let mut out=SurfaceMeshData::default();
                assert!(raised_brick_side(&mut out,tile,&images,face,normal,8.*scale));
                assert!(face.iter().all(|p|out.positions.contains(p)),"keep all four structural contacts");
                for tri in out.indices.chunks_exact(3) {
                    let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(out.positions[i as usize]));
                    let cross=(b-a).cross(c-a);
                    if cross.length_squared()>1e-10 {
                        assert!(cross.dot(Vec3::from_array(out.normals[tri[0]as usize]))>0.);
                    }
                }
                for p in &out.positions {
                    let p=Vec3::from_array(*p)-bottom;
                    let u=p.dot(tangent)/(8.*scale);
                    let height=p.y-u*0.75*scale;
                    assert!(u>=-0.00001 && u<=1.00001);
                    assert!(height>=-0.00001 && height<=2.*scale+0.00001,
                        "fascia stays below the walking surface and above the original soffit");
                }
                let mut night=images.clone();
                for sample in night.pixels.values_mut() {
                    if let TileImageSample::Rgba(bytes)=sample {
                        for rgba in bytes.chunks_exact_mut(4) {rgba[0]/=2;rgba[1]/=2;}
                    }
                }
                let mut other=SurfaceMeshData::default();
                assert!(raised_brick_side(&mut other,tile,&night,face,normal,8.*scale));
                assert_eq!(out.positions,other.positions);assert_eq!(out.normals,other.normals);
                assert_eq!(out.indices,other.indices);assert_ne!(out.colors,other.colors);
                let mut gold_grout=images.clone();
                if let Some(TileImageSample::Rgba(bytes))=gold_grout.pixels.get_mut(&tile.texture.id()) {
                    for (pixel,rgba) in bytes.chunks_exact_mut(4).enumerate() {
                        if ink(0x4b,pixel%8,pixel/8)==0 {rgba.copy_from_slice(&[255,190,0,255]);}
                    }
                }
                let mut grounded=SurfaceMeshData::default();
                assert!(raised_brick_side(&mut grounded,tile,&gold_grout,face,normal,8.*scale));
                assert_eq!(out.colors,grounded.colors,
                    "source bond ink is a joint, not luminous metallic mortar");
            }
        }
        let mut altered=images.clone();
        if let Some(TileImageSample::Rgba(bytes))=altered.pixels.get_mut(&tile.texture.id()) {bytes[3]=0;}
        let mut out=SurfaceMeshData::default();
        assert!(!raised_brick_side(&mut out,tile,&altered,[[0.;3];4],[1.,0.,0.],g.tile_width));
        assert!(out.positions.is_empty(),"transparent or altered source ink keeps the existing frame");
    }

    #[test]
    fn brick_bridge_keeps_original_support_and_finishes_all_water_edges() {
        let (cells, shapes, g, images, mut mesh) = bridge_fixture();
        let feet = mesh.footing_heights.clone();
        finish(
            &mut mesh,
            &cells.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
            &images,
        );
        assert_eq!(mesh.footing_heights, feet);
        assert!(mesh.textured.uvs.chunks_exact(4).all(|uv| {
            new_bark::sampled_cell(uv, &g).is_none_or(|i| cells[i].source.metatile_id != 5)
        }));
        for (i, c) in cells.iter().enumerate() {
            assert_eq!(
                mesh.authored_cells[i],
                (c.source.metatile_id == 5).then_some(BRIDGE_LABEL)
            );
        }
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| p[1] <= 0. && p[1] >= -2.)
        );
        for tri in mesh.solid.indices.chunks_exact(3) {
            let [a, b, c] =
                std::array::from_fn(|j| Vec3::from_array(mesh.solid.positions[tri[j] as usize]));
            let cross = (b - a).cross(c - a);
            if cross.length_squared() > 0.000001 {
                assert!(cross.dot(Vec3::from_array(mesh.solid.normals[tri[0] as usize])) > 0.);
            }
        }
        for normal in [[1., 0., 0.], [-1., 0., 0.], [0., 0., 1.], [0., 0., -1.]] {
            assert!(
                mesh.solid.normals.contains(&normal),
                "water edge {normal:?}"
            );
        }
    }
    #[test]
    fn graded_source_cell_cannot_push_white_bedding_through_its_brick_caps() {
        let g = GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let source = VisualTileSource {
            tileset_id: Arc::from("kanto"),
            metatile_id: 5,
            subtile_column: 0,
            subtile_row: 0,
            tile_index: 0x4b,
        };
        let mut deck = SurfaceMeshData::default();
        brick_deck(&mut deck, &g, 0, &source, COLORS);
        let grade = |p: Vec3| 6. * (p.x / 8.).min(p.z / 8.);
        for tri in deck.indices.chunks_exact(3) {
            if tri.iter().any(|&i| i < 4) {
                continue;
            }
            let p: [Vec3; 3] =
                std::array::from_fn(|j| Vec3::from_array(deck.positions[tri[j] as usize]));
            if (p[1] - p[0]).cross(p[2] - p[0]).length_squared() < 0.000001 {
                continue;
            }
            let center = (p[0] + p[1] + p[2]) / 3.;
            let top = p.iter().map(|p| p.y + grade(*p)).sum::<f32>() / 3.;
            assert!(
                top >= grade(center) - 0.16 - 0.00001,
                "bedding breaks through at {center:?}"
            );
        }
    }
    #[test]
    fn clipped_staggered_bricks_join_without_a_false_source_cell_seam() {
        let g = GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut pair = Vec::new();
        for column in 0..2 {
            let source = VisualTileSource {
                tileset_id: Arc::from("kanto"),
                metatile_id: 5,
                subtile_column: column,
                subtile_row: 0,
                tile_index: 0x4b,
            };
            let mut surface = SurfaceMeshData::default();
            brick_deck(&mut surface, &g, column as usize, &source, COLORS);
            assert!(surface.positions.iter().all(|p| p[0] >= column as f32 * 8.
                && p[0] <= (column + 1) as f32 * 8.
                && p[2] >= 0.
                && p[2] <= 8.));
            let edge = surface
                .positions
                .iter()
                .zip(&surface.colors)
                .zip(&surface.normals)
                .filter(|((p, _), _)| {
                    p[0] == 8. && p[2] > g.tile_height / 3. && p[2] < g.tile_height * 2. / 3.
                })
                .map(|((p, c), n)| {
                    (
                        [p[1].to_bits(), p[2].to_bits()],
                        c.map(f32::to_bits),
                        n.map(f32::to_bits),
                    )
                })
                .collect::<std::collections::BTreeSet<_>>();
            assert!(!edge.is_empty());
            pair.push(edge);
        }
        assert_eq!(pair[0], pair[1]);
    }
    #[test]
    fn unknown_brick_art_or_source_phase_does_not_consume_the_bridge() {
        for defect in 0..3 {
            let (mut cells, shapes, g, mut images, mut mesh) = bridge_fixture();
            if defect == 0 {
                cells[8].source.subtile_column = 0;
            } else if defect == 1 {
                mesh.authored_cells[8] = Some("other");
            } else if let TileImageSample::Rgba(p) =
                images.pixels.get_mut(&cells[8].texture.id()).unwrap()
            {
                p[0] ^= 32;
            }
            let old = mesh.textured.clone();
            finish(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &images,
            );
            assert_eq!(mesh.textured, old);
            assert!(mesh.solid.positions.is_empty());
        }
    }
    #[test]
    fn all_native_floor_drawings_get_bevels_without_raising_standing_datum() {
        for block in [0x22, 0x23, 0x26, 0x27, 0x44, 0x45, 0x48, 0x49, 0x5d, 0x5e] {
            let (cells, shapes, g, images, mut mesh) = fixture(block);
            let feet = mesh.footing_heights.clone();
            finish(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &images,
            );
            assert!(mesh.textured.positions.is_empty(), "block {block:x}");
            assert!(mesh.authored_cells.iter().all(|c| *c == Some(LABEL)));
            assert_eq!(mesh.footing_heights, feet);
            assert!(mesh.solid.positions.iter().all(|p| p[1] <= 0.
                && p[1] >= -0.16
                && (-16. ..=16.).contains(&p[0])
                && (24. ..=56.).contains(&p[2])));
            assert!(mesh.solid.positions.iter().any(|p| p[1] < 0.));
            assert!(mesh.solid.normals.iter().any(|n| n[1] > 0. && n[1] < 0.999));
            for tri in mesh.solid.indices.chunks_exact(3) {
                let p = tri
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.solid.positions[i as usize]))
                    .collect::<Vec<_>>();
                let cross = (p[1] - p[0]).cross(p[2] - p[0]);
                if cross.length_squared() > 0.000001 {
                    assert!(cross.y > 0., "upward floor winding");
                }
            }
        }
    }
    #[test]
    fn blank_road_fields_are_unbroken_at_the_native_datum_and_keep_world_grain() {
        let (cells,shapes,g,images,original)=fixture(0x27);
        let refs=cells.iter().collect::<Vec<_>>();
        let mut road=original.clone();
        finish_at_origin(&mut road,&refs,&shapes,&g,&images,[17,29]);
        assert_eq!(road.footing_heights,original.footing_heights);
        assert!(road.authored_cells.iter().all(|c|*c==Some(LABEL)));
        for (cell,tile) in cells.iter().enumerate().filter(|(_,t)|t.source.tile_index==0x23) {
            let (w,e,n,s)=g.bounds(cell%g.width,cell/g.width);
            let mut area=0.;let mut triangles=0;
            for ids in road.solid.indices.chunks_exact(3) {
                let p=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(road.solid.positions[i as usize]));
                let center=(p[0]+p[1]+p[2])/3.;
                if center.x<=w || center.x>=e || center.z<=n || center.z>=s {continue;}
                triangles+=1;area+=(p[1]-p[0]).cross(p[2]-p[0]).y*0.5;
                assert!(p.iter().all(|p|p.y==0. && (p.x==w||p.x==e) && (p.z==n||p.z==s)),
                    "the blank source field must have no inset tile seams: {:?}",tile.source);
                assert!(ids.iter().all(|&i|crate::scenery_finish::road_coordinates(road.solid.uvs[i as usize]).is_some()),
                    "blank pavement must have its own aggregate finish, not the brick bond material");
            }
            assert_eq!(triangles,2);assert!((area-g.tile_width*g.tile_height).abs()<0.00001);
        }
        let shifted_g=GridGeometry{origin_x:g.origin_x+64.,origin_z:g.origin_z+32.,..g};
        let mut shifted=original.clone();
        for p in &mut shifted.textured.positions {p[0]+=64.;p[2]+=32.;}
        finish_at_origin(&mut shifted,&refs,&shapes,&shifted_g,&images,[17,29]);
        assert_eq!(road.solid.indices,shifted.solid.indices);
        let mut grain_vertices=0;
        for (i,uv) in road.solid.uvs.iter().enumerate() {
            if let Some(at)=crate::scenery_finish::paving_coordinates(*uv) {
                grain_vertices+=1;
                let p=road.solid.positions[i];
                assert_eq!(at,[17.+(p[0]-g.origin_x)/g.tile_width,29.+(p[2]-g.origin_z)/g.tile_height]);
                assert_eq!(*uv,shifted.solid.uvs[i]);
                assert_eq!(road.solid.colors[i],shifted.solid.colors[i]);
            }
        }
        assert_eq!(grain_vertices,32,"eight blank source cells share one continuous finish");
    }

    #[test]
    fn changed_art_phase_claim_height_and_borrowed_sample_reject_the_complete_block() {
        for defect in 0..8 {
            let (mut cells, mut shapes, g, mut images, mut mesh) = fixture(0x44);
            match defect {
                0 => cells[6].source.subtile_column = 1,
                1 => cells[6].source.metatile_id = 0x45,
                2 => cells[6].source.tile_index = 0x39,
                3 => mesh.authored_cells[6] = Some("other"),
                4 => shapes[6] = CellShape::PlaneAt { height: 2. },
                5 => mesh.textured.positions[6 * 4][1] = 2.,
                6 => {
                    if let TileImageSample::Rgba(rgba) =
                        images.pixels.get_mut(&cells[6].texture.id()).unwrap()
                    {
                        rgba[3] = 0;
                    }
                }
                _ => mesh.textured.uvs[6 * 4..6 * 4 + 4].copy_from_slice(&[[0., 0.]; 4]),
            }
            let old = mesh.textured.clone();
            finish(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &images,
            );
            assert_eq!(mesh.textured, old, "defect {defect}");
            assert!(mesh.solid.positions.is_empty());
        }
        let (mut cells, shapes, g, images, mut mesh) = fixture(0x44);
        cells.truncate(12);
        let old = mesh.textured.clone();
        finish(
            &mut mesh,
            &cells.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
            &images,
        );
        assert_eq!(mesh.textured, old);
    }
    #[test]
    fn live_palette_changes_paint_but_not_geometry() {
        for block in [0x05, 0x55, 0x27, 0x44, 0x5e] {
            let (cells, shapes, g, images, mut day) = fixture(block);
            let mut night_images = images.clone();
            for sample in night_images.pixels.values_mut() {
                if let TileImageSample::Rgba(rgba) = sample {
                    for p in rgba.chunks_exact_mut(4) {
                        p[0] /= 2;
                        p[1] /= 2;
                        p[2] = (u16::from(p[2]) * 3 / 4) as u8;
                    }
                }
            }
            let mut night = day.clone();
            let cells = cells.iter().collect::<Vec<_>>();
            finish(&mut day, &cells, &shapes, &g, &images);
            finish(&mut night, &cells, &shapes, &g, &night_images);
            assert_eq!(day.solid.positions, night.solid.positions);
            assert_eq!(day.footing_heights, night.footing_heights);
            assert_ne!(day.solid.colors, night.solid.colors);
            assert!(!day.solid.indices.is_empty());
            for mesh in [&day.solid, &night.solid] {
                assert!(
                    mesh.uvs
                        .iter()
                        .all(|&uv| uv == crate::scenery_finish::SOURCE_PAINT_UV
                            || crate::scenery_finish::paving_coordinates(uv).is_some()),
                    "every source-colored face, including clipped brick fans and circular trim, must avoid a second night tint"
                );
            }
        }
    }
}
