//! Continue the room's existing rear architecture around its actual map bounds.
//! All geometry is presentation-only; source floors and support remain intact.
use super::*;
use bevy::prelude::IVec2;
#[path = "tin_room_bounds.rs"]
mod tin_room_bounds;
#[path = "tin_north_outline.rs"]
mod tin_north_outline;
#[path = "pokecom_room_bounds.rs"]
mod pokecom_room_bounds;
#[path = "hall_room.rs"]
mod hall_room;
#[path = "home_stair_recess.rs"]
mod home_stair_recess;
pub(super) use home_stair_recess::append as append_home_stair_recess;
#[path = "native_arcade_backing.rs"]
mod native_arcade_backing;
#[path = "native_facility_room_shell.rs"]
mod native_facility_room_shell;
#[path = "cerulean_room.rs"]
mod cerulean_room;
pub(in crate::mesh) use hall_room::finish as finish_hall_room;
#[cfg(test)]
use bevy::prelude::UVec2;

fn is_wall(kind: ModelKind) -> bool {
    matches!(
        kind,
        ModelKind::WallDomestic
            | ModelKind::WallTraditional
            | ModelKind::WallClinical
            | ModelKind::WallAcoustic
            | ModelKind::WallPanel
            | ModelKind::ArcadeWall
            | ModelKind::WindowWall
    )
}

fn plain_panel(p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    let source = &cells[p.row * g.width + p.column].source;
    p.indices(g.width)
        .all(|i| cells[i].source.tile_index == source.tile_index)
}

fn ungrounded_wall_donor(map: &str, cells: &[&VisualTile], g: &GridGeometry) -> Option<Placement> {
    let mut drawings = INTERIOR_DRAWINGS
        .iter()
        .filter(|d| is_wall(d.kind))
        .collect::<Vec<_>>();
    drawings.sort_by_key(|d| {
        (
            d.kind == ModelKind::WindowWall,
            !d.tiles.iter().all(|t| *t == d.tiles[0]),
        )
    });
    for drawing in drawings {
        // This wall carries a control unit; returns must copy plain field
        // paint rather than repeating the control around the room.
        if drawing.tileset == "battle_tower_inside" && matches!(drawing.block, 3 | 4 | 0x1b | 0x1d | 0x1e | 0x2a | 0x2b) {
            continue;
        }
        if drawing.tileset == "underground" && map != "VermilionPortPassage" {
            continue;
        }
        if map == "CeladonGameCornerPrizeRoom" && drawing.tileset == "game_corner" {
            continue;
        }
        if reviewed_native_room(map)
            && drawing.tileset == "players_house"
            && drawing.tiles.iter().all(|t| *t == 0x44)
        {
            continue;
        }
        let [w, h] = drawing.size;
        if w > g.width || h > g.height {
            continue;
        }
        for row in 0..=g.height - h {
            for column in 0..=g.width - w {
                if !(0..h).all(|dy| {
                    (0..w).all(|dx| {
                        let s = &cells[(row + dy) * g.width + column + dx].source;
                        s.tileset_id.as_ref() == drawing.tileset
                            && s.metatile_id == drawing.block
                            && s.subtile_column as usize == drawing.origin[0] as usize + dx
                            && s.subtile_row as usize == drawing.origin[1] as usize + dy
                            && s.tile_index == drawing.tiles[dy * w + dx]
                    })
                }) {
                    continue;
                }
                let (depth_pixels, height_pixels) = proportions(drawing.kind, w, h);
                return Some(Placement {
                    kind: drawing.kind,
                    column,
                    row,
                    width: w,
                    height: h,
                    // This temporary floor is discarded by above_floor. Only
                    // complete authenticated wall geometry reaches the room shell.
                    ground: row * g.width + column,
                    depth_pixels,
                    height_pixels,
                    base_pixels: 0.,
                    footing_pixels: None,
                });
            }
        }
    }
    None
}

fn cafe_rear_template(out:&mut SurfaceMeshData,map:&str,cells:&[&VisualTile],g:&GridGeometry,
    placements:&[Placement])->Option<usize> {
    if !matches!(map,"CeladonCafe"|"OlivineCafe") {return None;}
    let donor=placements.iter().find(|p|p.kind==ModelKind::ArcadeWall && p.width==4 && p.height==2
        &&p.base_pixels==0. &&p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()=="game_corner" && s.metatile_id==2 && s.tile_index==2
                &&s.subtile_column==(j%4)as u8&&s.subtile_row==(j/4)as u8
        }))?;
    // Two columns match the perimeter's two-cell sections at native texel
    // scale, while the complete four-column donor remains authenticated.
    let mut p=donor.clone();p.width=2;
    let w=g.origin_x+p.column as f32*g.tile_width;
    let front=g.origin_z+(p.row+2)as f32*g.tile_height;
    append_kanto_house_wall(out,g,&p,[w,w+2.*g.tile_width,front-3.*g.tile_height/8.,front]);
    Some(p.row)
}

// The downstairs side returns continue block $05's solid rear planks.
// The projecting divider and kitchen trim are separate source drawings.
fn player_house_rear_template(out:&mut SurfaceMeshData,map:&str,cells:&[&VisualTile],g:&GridGeometry)->Option<usize> {
    if map!="PlayersHouse1F" || g.width<4 || g.height<2 {return None;}
    for row in 0..=g.height-2 {for column in 0..=g.width-4 {
        if !(0..2).all(|y|(0..4).all(|x| {
            let s=&cells[(row+y)*g.width+column+x].source;
            s.tileset_id.as_ref()=="players_house" && s.metatile_id==5 && s.tile_index==0x11
                && s.subtile_column==x as u8 && s.subtile_row==y as u8
        })) {continue;}
        let w=g.origin_x+column as f32*g.tile_width;
        let front=g.origin_z+(row+2)as f32*g.tile_height;
        native_home_timber::append(out,g,[w,w+2.*g.tile_width,front-3.*g.tile_height/8.,front],
            0.,32.*g.tile_height/8.,[column,row]);
        return Some(row);
    }}
    None
}

fn above_floor(source: &SurfaceMeshData) -> SurfaceMeshData {
    let indices = source
        .indices
        .chunks_exact(3)
        .filter(|triangle| {
            triangle
                .iter()
                .any(|&i| source.positions[i as usize][1] > 0.01)
        })
        .flatten()
        .copied();
    select_surface_indices(
        source,
        &vec![false; source.positions.len()],
        &mut vec![u32::MAX; source.positions.len()],
        indices,
    )
}

/// Some rear courses are entirely occupied by equipment or built-in shelving.
/// Their returns use the rear drawing's field paint, never repeated furniture.
fn rear_paint_template(
    out: &mut TerrainMeshData,
    existing: &TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    frame: &VisualWorldFrame,
    images: Option<&TerrainImageSamples>,
) -> Option<usize> {
    let size = frame.source_map_size_core_tiles?;
    let row = (0..g.height).find(|&row| frame.grid_origin.y + row as i32 >= 0)?;
    let mut paint = std::collections::BTreeMap::<[u8; 4], (usize, usize, usize)>::new();
    for (i, tile) in cells.iter().enumerate().filter(|(i, _)| i / g.width == row) {
        let sx = frame.grid_origin.x + (i % g.width) as i32;
        if sx < 0 || sx >= size.x as i32 * 2 || tile.source.metatile_id == u16::MAX {
            continue;
        }
        if let Some(TileImageSample::Rgba(pixels)) =
            images.and_then(|images| images.pixels.get(&tile.texture.id()))
        {
            for (pixel, rgba) in pixels
                .chunks_exact(4)
                .enumerate()
                .filter(|(_, p)| p[3] == 255)
            {
                let entry = paint
                    .entry(rgba.try_into().unwrap())
                    .or_insert((0, i, pixel));
                entry.0 += 1;
            }
        } else {
            // Geometry-only callers do not provide pixel samples.
            paint.entry([0, 0, 0, 255]).or_insert((1, i, 0));
        }
    }
    let (_, (_, donor, pixel)) = paint.into_iter().max_by_key(|(_, v)| v.0)?;
    let uv = g.uv(donor % g.width, donor / g.width);
    let sample = [
        uv.0 + (uv.1 - uv.0) * ((pixel % 8) as f32 + 0.5) / 8.,
        uv.2 + (uv.3 - uv.2) * ((pixel / 8) as f32 + 0.5) / 8.,
    ];
    let north = g.origin_z - frame.grid_origin.y as f32 * g.tile_height;
    let west = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let height = existing
        .solid
        .positions
        .iter()
        .chain(&existing.textured.positions)
        .filter(|p| {
            p[0] >= west
                && p[0] <= west + size.x as f32 * 2. * g.tile_width
                && p[2] >= north
                && p[2] <= north + 4. * g.tile_height
        })
        .map(|p| p[1])
        .fold(0., f32::max);
    let height = if height > g.tile_height {
        height
    } else {
        4. * g.tile_height
    };
    let front = g.origin_z + (row + 2) as f32 * g.tile_height;
    architecture_box(
        &mut out.textured,
        [
            west,
            west + 2. * g.tile_width,
            0.,
            height,
            front - 3. * g.tile_height / 8.,
            front,
        ],
        [1.; 3],
    );
    out.textured.uvs.fill(sample);
    Some(row)
}

/// Legacy rooms retain source-band walls instead of a modeled panel. Continue
/// those exact bands, including their atlas paint, over a closed backing.
fn source_band_template(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: IVec2,
) -> Option<usize> {
    for row in 0..g.height {
        for column in 0..g.width.saturating_sub(1) {
            let sx = origin.x + column as i32;
            let sy = origin.y + row as i32;
            if sx < 0 || sy < 0 || sy > 3 {
                continue;
            }
            if crate::mart::department_store_wall_shape(map, &cells[row * g.width + column].source)
                .is_none()
            {
                continue;
            }
            let CellShape::FacadeBand {
                plane_subtile_row,
                band_from_top: 0,
                band_count,
                solid: SolidKind::FlatCard,
                ..
            } = shape_for_source_on_map(map, &cells[row * g.width + column].source)
            else {
                continue;
            };
            if band_count < 2 || row + band_count as usize > g.height {
                continue;
            }
            if !(0..band_count as usize).all(|dy|(0..2).all(|dx| matches!(
            shape_for_source_on_map(map,&cells[(row+dy)*g.width+column+dx].source),
            CellShape::FacadeBand{plane_subtile_row:p,band_from_top:b,band_count:c,solid:SolidKind::FlatCard,..}
                if p==plane_subtile_row&&b as usize==dy&&c==band_count))) {continue;}
            let west = g.origin_x + column as f32 * g.tile_width;
            let east = west + 2. * g.tile_width;
            let front = g.origin_z
                + (row as f32 - cells[row * g.width + column].source.subtile_row as f32
                    + plane_subtile_row as f32)
                    * g.tile_height;
            let height = band_count as f32 * g.tile_height;
            let uv = g.uv(column, row);
            let paint = [
                uv.0 + (uv.1 - uv.0) * 0.5 / 8.,
                uv.2 + (uv.3 - uv.2) * 0.5 / 8.,
            ];
            let first = mesh.textured.positions.len();
            architecture_box(
                &mut mesh.textured,
                [
                    west,
                    east,
                    0.,
                    height,
                    front - 3. * g.tile_height / 8.,
                    front,
                ],
                [1.; 3],
            );
            mesh.textured.uvs[first..].fill(paint);
            for dy in 0..band_count as usize {
                for dx in 0..2 {
                    let uv = g.uv(column + dx, row + dy);
                    let w = west + dx as f32 * g.tile_width;
                    let e = w + g.tile_width;
                    let top = height - dy as f32 * g.tile_height;
                    let bottom = top - g.tile_height;
                    append_quad(
                        &mut mesh.textured,
                        [
                            [e, bottom, front + 0.001],
                            [e, top, front + 0.001],
                            [w, top, front + 0.001],
                            [w, bottom, front + 0.001],
                        ],
                        Vec3::Z.to_array(),
                        [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                        TEXTURED_SHADE,
                    );
                }
            }
            return Some(row);
        }
    }
    None
}

#[derive(Clone, Copy, Debug)]
enum Edge {
    North,
    West,
    East,
    South,
}

fn copy_section(
    out: &mut SurfaceMeshData,
    source: &SurfaceMeshData,
    b: [f32; 4],
    edge: Edge,
    at: f32,
    start: f32,
    end: f32,
) {
    if source.indices.is_empty() {
        return;
    }
    let first = out.positions.len();
    let stretch = (end - start) / (b[1] - b[0]);
    let turn = |x: f32, y: f32, z: f32| match edge {
        Edge::North => [x, y, z],
        Edge::West => [z, y, -x],
        Edge::East => [-z, y, x],
        Edge::South => [-x, y, -z],
    };
    out.positions.extend(source.positions.iter().map(|p| {
        let mut v = turn((p[0] - b[0]) * stretch, p[1], p[2] - b[3]);
        match edge {
            Edge::North => {v[0]+=start;v[2]+=at;}
            Edge::West => {
                v[0] += at;
                v[2] += end;
            }
            Edge::East => {
                v[0] += at;
                v[2] += start;
            }
            Edge::South => {
                v[0] += end;
                v[2] += at;
            }
        }
        v
    }));
    out.normals.extend(source.normals.iter().map(|n| {
        Vec3::from_array(turn(n[0] / stretch, n[1], n[2]))
            .normalize()
            .to_array()
    }));
    out.uvs.extend_from_slice(&source.uvs);
    out.colors.extend_from_slice(&source.colors);
    out.indices
        .extend(source.indices.iter().map(|i| *i + first as u32));
    out.cutaway_ranges.push(first..out.positions.len());
}


fn append_section(mesh:&mut TerrainMeshData,solid:&SurfaceMeshData,textured:&SurfaceMeshData,
    b:[f32;4],edge:Edge,at:f32,start:f32,end:f32) {
    let first_solid=mesh.solid.positions.len();let first_textured=mesh.textured.positions.len();
    copy_section(&mut mesh.solid,solid,b,edge,at,start,end);
    copy_section(&mut mesh.textured,textured,b,edge,at,start,end);
    let plane=match edge {
        Edge::North=>[0.,1.,at],Edge::West=>[1.,0.,at],
        Edge::East=>[-1.,0.,-at],Edge::South=>[0.,-1.,-at],
    };
    if first_solid<mesh.solid.positions.len() {
        mesh.camera_wall_solid_ranges.push((first_solid..mesh.solid.positions.len(),plane));
    }
    if first_textured<mesh.textured.positions.len() {
        mesh.camera_wall_textured_ranges.push((first_textured..mesh.textured.positions.len(),plane));
    }
}

pub(in crate::mesh) fn append(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placements: &[Placement],
    dungeon_placements: &[super::super::modeled_dungeons::Placement],
    images: Option<&TerrainImageSamples>,
    profiles: Option<&Document>,
) {
    if super::super::lance_terraces::owns_perimeter(frame,cells,g,profiles) {
        return;
    }
    if native_facility_room_shell::append(mesh,frame,cells,g,profiles) {
        return;
    }
    if frame.map_id.as_ref()=="BattleTowerBattleRoom" {
        super::super::modeled_dungeons::append_tower_battle_perimeter(mesh,frame,cells,g);
        return;
    }
    if matches!(frame.map_id.as_ref(),"BattleTowerHallway"|"BattleTower1F") {
        native_tower_architecture::perimeter(mesh,frame,cells,g,placements);
        return;
    }
    if frame.map_id.as_ref()=="CeruleanGym" {
        cerulean_room::append(mesh,frame,cells,g);
        return;
    }
    if frame.map_id.as_ref() == "HallOfFame" {
        hall_room::append(mesh,frame,cells,g);
        return;
    }
    if frame.map_id.as_ref()==crate::pokecom::ADMIN_OFFICE {
        pokecom_room_bounds::append(mesh,frame,cells,g);
        return;
    }
    let Some(size) = frame.source_map_size_core_tiles else {
        return;
    };
    let Some(source) = cells.first() else {
        return;
    };
    if !crate::interior::has_room_perimeter(&frame.map_id, &source.source.tileset_id) {
        return;
    }
    // Resolve the source drawing through the same production wall builder.
    // Prefer plain panels; a glazed room can continue its window course.
    // Never repeat signs, cabinets or a whole furniture course.
    let donor = placements
        .iter()
        .filter(|p| is_wall(p.kind) && p.base_pixels == 0. && p.depth_pixels <= 16.
            && !(cells[p.row*g.width+p.column].source.tileset_id.as_ref()=="battle_tower_inside"
                && matches!(cells[p.row*g.width+p.column].source.metatile_id,3|4|0x1b|0x1d|0x1e|0x2a|0x2b)))
        .min_by_key(|p| {
            (
                p.kind == ModelKind::WindowWall,
                p.row,
                !plain_panel(p, cells, g),
                p.column,
            )
        });
    let mut template = TerrainMeshData::default();
    let donor_row = if let Some(row)=cafe_rear_template(&mut template.textured,&frame.map_id,cells,g,placements) {
        row
    } else if let Some(row)=player_house_rear_template(&mut template.textured,&frame.map_id,cells,g) {
        row
    } else if let Some(row) = native_classroom::perimeter_template(
        &mut template.textured,&frame.map_id,cells,g,placements) {
        row
    } else if let Some(row) = traditional_wood_wall::perimeter_template(
        &mut template.textured,
        &frame.map_id,
        cells,
        g,
        placements,
    ) {
        row
    } else if let Some(donor) = dungeon_placements.iter()
        .filter(|p| (super::super::modeled_dungeons::tin_tower_guardian_map(&frame.map_id)
            ||matches!(frame.map_id.as_ref(),"SproutTower1F"|"SproutTower2F"|"SproutTower3F"))
            &&p.native_tower_wall())
        .min_by_key(|p|p.source_row()) {
        // A side-facing post's width/depth axes are already rotated. Copying
        // it as a rear-facing donor creates thick repeated blocks and open
        // seams. Native rear courses retain their actual thin wall geometry.
        donor.append_room_wall(&mut template,g,cells,frame.grid_origin.to_array());
        donor.source_row()
    } else if let Some(donor) = donor {
        if !super::append(
            &mut template,
            &frame.map_id,
            cells,
            g,
            donor,
            &mut vec![false; cells.len()],
            frame.grid_origin.to_array(),
        ) {
            return;
        }
        donor.row
    } else if let Some(donor) = dungeon_placements
        .iter()
        .filter(|p| p.room_wall())
        .min_by_key(|p| p.source_row())
    {
        donor.append_room_wall(&mut template, g, cells, frame.grid_origin.to_array());
        donor.source_row()
    } else if let Some(row) = super::super::department_store::perimeter_template(
        &mut template,
        &frame.map_id,
        cells,
        g,
        frame.grid_origin.to_array(),
        None,
    ) {
        row
    } else if let Some(donor) = ungrounded_wall_donor(&frame.map_id, cells, g) {
        if !super::append(
            &mut template,
            &frame.map_id,
            cells,
            g,
            &donor,
            &mut vec![false; cells.len()],
            frame.grid_origin.to_array(),
        ) {
            return;
        }
        donor.row
    } else if let Some(row) =
        source_band_template(&mut template, &frame.map_id, cells, g, frame.grid_origin)
    {
        row
    } else if let Some(row) = rear_paint_template(&mut template, mesh, cells, g, frame, images) {
        row
    } else {
        return;
    };
    let mut solid = above_floor(&template.solid);
    let mut textured = above_floor(&template.textured);
    let mut b = [
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ];
    for p in solid.positions.iter().chain(&textured.positions) {
        b[0] = b[0].min(p[0]);
        b[1] = b[1].max(p[0]);
        b[2] = b[2].min(p[2]);
        b[3] = b[3].max(p[2]);
    }
    if b.iter().any(|v| !v.is_finite()) || b[1] <= b[0] || b[3] <= b[2] {
        return;
    }
    // Source-band rooms can fold a 16px rear wall while the shared panel kit
    // is 32px tall. Match the actual built rear course at this donor's seam.
    let rear_height = mesh
        .solid
        .positions
        .iter()
        .chain(&mesh.textured.positions)
        .filter(|p| p[0] >= b[0] && p[0] <= b[1] && (p[2] - b[3]).abs() <= g.tile_height * 0.25)
        .map(|p| p[1])
        .fold(0., f32::max);
    let template_height = solid
        .positions
        .iter()
        .chain(&textured.positions)
        .map(|p| p[1])
        .fold(0., f32::max);
    if rear_height >= g.tile_height && template_height > 0. {
        let scale = rear_height / template_height;
        for surface in [&mut solid, &mut textured] {
            for p in &mut surface.positions {
                p[1] *= scale;
            }
            for n in &mut surface.normals {
                *n = Vec3::new(n[0], n[1] / scale, n[2]).normalize().to_array();
            }
        }
    }
    let width = size.x as i32 * 2;
    let height = size.y as i32 * 2;
    let map_west = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let inset = tin_room_bounds::insets(frame, cells, g, width, height);
    let west = map_west + inset[0] as f32 * g.tile_width;
    let east = map_west + (width - inset[1]) as f32 * g.tile_width;
    let north = g.origin_z - frame.grid_origin.y as f32 * g.tile_height;
    let south = north + (height - inset[2]) as f32 * g.tile_height;
    let thickness = b[3] - b[2];
    let rear = north + b[2] - (g.origin_z + donor_row as f32 * g.tile_height);
    for (a, bound) in native_arcade_backing::gaps(frame,cells,g,profiles) {
        let mut x=map_west+a as f32*g.tile_width;
        let to=map_west+bound as f32*g.tile_width;
        while x<to-0.001 {
            let end=(x+2.*g.tile_width).min(to);
            append_section(mesh,&solid,&textured,b,Edge::North,rear+thickness,x,end);
            x=end;
        }
    }
    let north_gaps=tin_north_outline::gaps(frame,cells,g,profiles)
        .filter(|_|inset[0]==4&&inset[1]==4).filter(|gaps|!gaps.is_empty());
    if let Some(gaps)=&north_gaps {
        for &(a,bound) in gaps {
            let a=map_west+a as f32*g.tile_width;
            let bound=map_west+bound as f32*g.tile_width;
            let from=if a==west {a-thickness}else{a};
            let to=if bound==east {bound+thickness}else{bound};
            let mut x=from;
            while x<to-0.001 {
                let end=(x+g.tile_width*2.).min(to);
                append_section(mesh,&solid,&textured,b,Edge::North,north,x,end);
                x=end;
            }
            // Return into the already authored rear course. Each return's
            // backing sits on the WALL side, leaving the neighboring floor or
            // horizontal grating untouched. Internal walls are not extended.
            let front=north+2.*g.tile_height-thickness;
            if a>west {append_section(mesh,&solid,&textured,b,Edge::West,a,north,front);}
            if bound<east {append_section(mesh,&solid,&textured,b,Edge::East,bound,north,front);}
        }
    }
    let side_rear=if north_gaps.is_some(){north}else{rear.max(north)};
    // Actual room edges, clipped to this builder's terrain region. A cropped
    // viewport is never a room boundary. Corners overlap outside the floor.
    for (edge, at, from, to, step) in [
        (Edge::West, west, side_rear, south, g.tile_height * 2.),
        (Edge::East, east, side_rear, south, g.tile_height * 2.),
        (
            Edge::South,
            south,
            west - thickness,
            east + thickness,
            g.tile_width * 2.,
        ),
    ] {
        let (grid_from, grid_to, visible) = match edge {
            Edge::West => (
                g.origin_z,
                g.origin_z + g.height as f32 * g.tile_height,
                frame.grid_origin.x <= inset[0] && frame.grid_origin.x + g.width as i32 >= inset[0],
            ),
            Edge::East => (
                g.origin_z,
                g.origin_z + g.height as f32 * g.tile_height,
                frame.grid_origin.x <= width - inset[1] && frame.grid_origin.x + g.width as i32 >= width - inset[1],
            ),
            Edge::North => unreachable!("north joins use authenticated source gaps"),
            Edge::South => (
                g.origin_x,
                g.origin_x + g.width as f32 * g.tile_width,
                frame.grid_origin.y <= height - inset[2] && frame.grid_origin.y + g.height as i32 >= height - inset[2],
            ),
        };
        if !visible {
            continue;
        }
        let mut gaps = frame
            .boundary_warps
            .iter()
            .filter_map(|warp| {
                if !tin_room_bounds::opens_perimeter(&frame.map_id, *warp) {
                    return None;
                }
                if warp.x < 0 || warp.y < 0 || warp.x >= size.x as i32 || warp.y >= size.y as i32 {
                    return None;
                }
                let (on_edge, offset, unit) = match edge {
                    Edge::West => (warp.x == inset[0] / 2, warp.y, north),
                    Edge::East => (warp.x == (width - inset[1]) / 2 - 1, warp.y, north),
                    Edge::North => unreachable!("north joins do not open internal warps"),
                    Edge::South => (warp.y >= (height - inset[2]) / 2 - 2, warp.x, map_west),
                };
                let tile = match edge {
                    Edge::South => g.tile_width,
                    _ => g.tile_height,
                };
                on_edge.then_some((
                    unit + offset as f32 * tile * 2.,
                    unit + (offset + 1) as f32 * tile * 2.,
                ))
            })
            .collect::<Vec<_>>();
        if matches!(edge, Edge::East) {
            gaps.extend(native_radio_studio::perimeter_glass(
                &frame.map_id,
                cells,
                g,
                placements,
                &mesh.authored_cells,
                frame.grid_origin.to_array(),
                width,
            ));
        }
        gaps.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut start = from;
        while start < to - 0.001 {
            let end = (start + step).min(to);
            let mut runs = vec![(start, end)];
            for &(a, b) in &gaps {
                runs = runs
                    .into_iter()
                    .flat_map(|(s, e)| {
                        if b <= s || a >= e {
                            vec![(s, e)]
                        } else {
                            [(s, a.min(e)), (b.max(s), e)]
                                .into_iter()
                                .filter(|(l, r)| r - l > 0.001)
                                .collect()
                        }
                    })
                    .collect();
            }
            for (s, e) in runs {
                if e < grid_from || s > grid_to {
                    continue;
                }
                append_section(mesh,&solid,&textured,b,edge,at,s,e);
            }
            start = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_room_wall_donors() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let mut total = 0;
        let mut missing = std::collections::BTreeMap::<String, Vec<String>>::new();
        for (map, data) in &pack.data().maps {
            let tileset = pack.data().map_tileset_name(map).unwrap();
            if !crate::interior::has_room_perimeter(map, &tileset) {
                continue;
            }
            if matches!(
                map.as_str(),
                "GoldenrodDeptStoreRoof" | "CeladonMansionRoof" | "TinTowerRoof"
            ) {
                continue;
            }
            total += 1;
            let art = &pack.runtime_files()[&format!("data/tilesets/{tileset}_metatiles.bin")];
            let (w, h) = (
                data.attributes.width as usize * 4,
                data.attributes.height as usize * 4,
            );
            let g = GridGeometry {
                width: w,
                height: h,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let tiles = (0..w * h)
                .map(|i| {
                    let (x, y) = (i % w, i / w);
                    let b = data.blocks[y / 4 * (w / 4) + x / 4];
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        texture: Default::default(),
                        priority: false,
                        animation_frames: None,
                        source: VisualTileSource {
                            tileset_id: tileset.into(),
                            metatile_id: b,
                            subtile_column: (x % 4) as u8,
                            subtile_row: (y % 4) as u8,
                            tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                        },
                    }
                })
                .collect::<Vec<_>>();
            let refs = tiles.iter().collect::<Vec<_>>();
            let p = resolve(map, &refs, &g, None);
            if matches!(map.as_str(),"CeladonCafe"|"OlivineCafe") {
                let mut wall=SurfaceMeshData::default();
                assert_eq!(cafe_rear_template(&mut wall,map,&refs,&g,&p),Some(0));
                let min=wall.positions.iter().map(|v|v[0]).fold(f32::INFINITY,f32::min);
                let max=wall.positions.iter().map(|v|v[0]).fold(f32::NEG_INFINITY,f32::max);
                assert_eq!(max-min,16.,"cafe returns preserve source texel width");
                assert_eq!(wall.positions.iter().map(|v|v[1]).fold(0.,f32::max),32.);
            }
            if map=="CeladonCafe" {
                let display=p.iter().find(|p|p.kind==ModelKind::ArcadePrizeRack&&p.row==0).unwrap();
                let mut rear=SurfaceMeshData::default();
                append_cafe_display_rear_wall(&mut rear,map,&refs,&g,display,[0,0]);
                assert!(!rear.indices.is_empty(),"native timber closes behind the rear cabinet");
                assert_eq!(rear.positions.iter().map(|v|v[1]).fold(0.,f32::max),32.);
                let min=rear.positions.iter().map(|v|v[0]).fold(f32::INFINITY,f32::min);
                let max=rear.positions.iter().map(|v|v[0]).fold(f32::NEG_INFINITY,f32::max);
                assert_eq!((min,max),(display.column as f32*8.,(display.column+4)as f32*8.));
                let mut interior=SurfaceMeshData::default();
                append_cafe_display_rear_wall(&mut interior,map,&refs,&g,display,[0,4]);
                assert!(interior.indices.is_empty());
                let mut changed=tiles.clone();
                for t in &mut changed {if t.source.metatile_id==2 {t.source.tile_index=0xffff;}}
                let mut rejected=SurfaceMeshData::default();
                append_cafe_display_rear_wall(&mut rejected,map,&changed.iter().collect::<Vec<_>>(),&g,display,[0,0]);
                assert!(rejected.indices.is_empty(),"unproven rear timber cannot be copied");
            }
            if map == "PlayersHouse1F" {
                let mut rear=SurfaceMeshData::default();
                assert_eq!(player_house_rear_template(&mut rear,map,&refs,&g),Some(0));
                assert_eq!(rear.positions.iter().map(|v|v[1]).fold(0.,f32::max),32.);
                // Every front texel comes from the authenticated rear plank
                // drawing, never the interior divider or kitchen trim.
                let front_uvs=(12..16).flat_map(|x|[g.uv(x,0),g.uv(x,1)]).collect::<Vec<_>>();
                for i in rear.indices.iter().map(|&i|i as usize).filter(|&i|rear.normals[i][2]>0.5) {
                    let uv=rear.uvs[i];
                    assert!(front_uvs.iter().any(|b|uv[0]>=b.0&&uv[0]<=b.1&&uv[1]>=b.2&&uv[1]<=b.3));
                }
                for y in 0..2 {for x in 12..16 {
                    let mut changed=tiles.clone();changed[y*w+x].source.tile_index=0xffff;
                    assert!(player_house_rear_template(&mut SurfaceMeshData::default(),map,
                        &changed.iter().collect::<Vec<_>>(),&g).is_none());
                }}
            }
            if map == "KurtsHouse" {
                let mut timber = SurfaceMeshData::default();
                assert!(
                    traditional_wood_wall::perimeter_template(&mut timber, map, &refs, &g, &p)
                        .is_some()
                );
                let swatches = timber
                    .uvs
                    .iter()
                    .map(|uv| uv.map(f32::to_bits))
                    .collect::<std::collections::BTreeSet<_>>();
                assert!(
                    swatches.len() >= 3,
                    "timber returns keep their native grain and joints"
                );
            }
            if map == "VioletPokecenter1F" {
                let donor = p
                    .iter()
                    .filter(|p| is_wall(p.kind) && p.base_pixels == 0. && p.depth_pixels <= 16.)
                    .min_by_key(|p| {
                        (
                            p.kind == ModelKind::WindowWall,
                            p.row,
                            !plain_panel(p, &refs, &g),
                            p.column,
                        )
                    })
                    .unwrap();
                assert!(
                    plain_panel(donor, &refs, &g),
                    "Center returns must not repeat wall-mounted displays"
                );
                assert_eq!(refs[donor.row * w + donor.column].source.tile_index, 2);
            }
            let dungeon = super::super::super::modeled_dungeons::resolve(map, &refs, &g, None);
            let mut frame = super::super::super::tests::frame(
                w as u32,
                h as u32,
                tiles.iter().map(|t| t.source.clone()).collect(),
            );
            frame.map_id = map.as_str().into();
            frame.source_map_size_core_tiles = Some(UVec2::new(w as u32 / 2, h as u32 / 2));
            frame.boundary_warps = data
                .events
                .warps
                .iter()
                .map(|warp| IVec2::new(i32::from(warp.x), i32::from(warp.y)))
                .collect();
            if crate::mesh::modeled_dungeons::tin_tower_guardian_map(map) {
                let outline=tin_north_outline::gaps(&frame,&refs,&g,None).expect("complete source north row");
                let affected=matches!(map.as_str(),"TinTower2F"|"TinTower3F"|"TinTower4F"|"TinTower5F"|"TinTower7F"|"TinTower9F");
                assert_eq!(!outline.is_empty(),affected,"{map}: only missing boundary sections are added");
                if map=="TinTower9F" {assert_eq!(outline,vec![(4,8),(32,36)]);}
                if map=="TinTower9F" {
                    let padded_g=GridGeometry{width:w+8,height:h+8,origin_x:-32.,origin_z:-32.,..g};
                    let mut padded=vec![refs[0];padded_g.width*padded_g.height];
                    for y in 0..h {for x in 0..w {padded[(y+4)*padded_g.width+x+4]=refs[y*w+x];}}
                    let mut shifted=frame.clone();shifted.grid_origin=IVec2::new(-4,-4);
                    assert_eq!(tin_north_outline::gaps(&shifted,&padded,&padded_g,None),Some(outline.clone()),
                        "viewport padding cannot move the physical source outline");
                }
                if map=="TinTower5F" {assert_eq!(outline,vec![(4,36)],"north walkway stays in front of the boundary");}
                for y in 0..4 {for x in 0..w {
                    let mut changed=tiles.clone();changed[y*w+x].source.tile_index^=1;
                    assert!(tin_north_outline::gaps(&frame,&changed.iter().collect::<Vec<_>>(),&g,None).is_none(),
                        "every padding, grating and wall source cell authenticates the outline");
                }}
                let short_g=GridGeometry{height:3,..g};
                assert!(tin_north_outline::gaps(&frame,&refs[..w*3],&short_g,None).is_none());
                let mut wrong=frame.clone();wrong.map_id="TinTowerRoof".into();
                assert!(tin_north_outline::gaps(&wrong,&refs,&g,None).is_none());
                wrong=frame.clone();wrong.source_map_size_core_tiles=Some(UVec2::new(20,20));
                assert!(tin_north_outline::gaps(&wrong,&refs,&g,None).is_none());
                let mut custom=built_in_profiles().objects[0].clone();
                custom.name="Custom north source".into();custom.map=Some(map.clone());custom.maps=None;
                custom.tileset="tower".into();custom.metatile=refs[4].source.metatile_id;custom.metatiles=None;
                custom.origin=[0,0];custom.ground=2;custom.tiles=vec![vec![refs[4].source.tile_index]];
                custom.parts.clear();custom.footing_pixels=None;
                assert!(tin_north_outline::gaps(&frame,&refs,&g,Some(&Document{objects:vec![custom],atmosphere:None})).is_none(),
                    "a custom owner of the actual source band suppresses inferred joins");
                for warp in &data.events.warps {
                    assert_eq!(tin_room_bounds::opens_perimeter(map,
                        IVec2::new(i32::from(warp.x), i32::from(warp.y))),
                        warp.target_map == "ECRUTEAK_CITY",
                        "{map}: stairs and teleport pads keep wall backing");
                }
            }
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; w * h],
                authored_cells: vec![None; w * h],
                ..Default::default()
            };
            if crate::mesh::modeled_dungeons::tin_tower_guardian_map(map) {
                mesh.animated_textured.positions.push([17.,19.,23.]);
                mesh.animated_pillars.push([7,9]);
            }
            append(&mut mesh, &frame, &refs, &g, &p, &dungeon, None, None);
            if super::super::super::lance_terraces::owns_perimeter(&frame,&refs,&g,None) {
                assert!(mesh.solid.indices.is_empty() && mesh.textured.indices.is_empty(),
                    "authenticated champion enclosure must not receive a second inferred black shell");
                super::super::super::lance_terraces::finish(&mut mesh,&frame,&refs,&g,None);
                assert_eq!(mesh.footing_heights[0],18.75,"upper dais retains its authored16px rise");
                assert!(mesh.footing_heights[12*w..].iter().all(|&h|h==2.75),
                    "the entrance and statue hall keep every walking height");
                assert_eq!(mesh.authored_cells.iter().filter(|&&v|v==Some("lance:south-barrier")).count(),32,
                    "the base drawing keeps both four-source-cell entrance lanes open");
                for x in [0.,160.] {
                    assert!(mesh.textured.positions.iter().any(|p|p[0]==x && p[1]==48. && p[2]==384.),
                        "native south corners close at the same ceiling as both blue walls");
                }
                continue;
            }
            if crate::mesh::modeled_dungeons::tin_tower_guardian_map(map) {
                assert_eq!(mesh.animated_textured.positions,vec![[17.,19.,23.]]);
                assert_eq!(mesh.animated_pillars,vec![[7,9]],"enclosure cannot replace the moving pillar");
                let gaps=tin_north_outline::gaps(&frame,&refs,&g,None).unwrap();
                let ranges=mesh.camera_wall_textured_ranges.iter().filter(|(_,plane)|*plane==[0.,1.,0.]).collect::<Vec<_>>();
                assert_eq!(!ranges.is_empty(),!gaps.is_empty());
                for (a,bound) in gaps {
                    let mut spans=ranges.iter().filter_map(|(range,_)| {
                        let points=&mesh.textured.positions[range.clone()];
                        assert!(points.iter().all(|p|p[2]>=-3.001&&p[2]<=0.001&&p[1]<=32.001));
                        let lo=points.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min);
                        let hi=points.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max);
                        (hi>a as f32*8.&&lo<bound as f32*8.).then_some((lo,hi))
                    }).collect::<Vec<_>>();
                    spans.sort_by(|a,b|a.0.total_cmp(&b.0));let mut end=a as f32*8.;
                    for (lo,hi) in spans {assert!(lo<=end+0.001,"{map}: no north gap");end=end.max(hi);}
                    assert!(end>=bound as f32*8.);
                    for (x,side) in [(a,1.),(bound,-1.)] {
                        if x==4||x==36 {continue;}
                        let at=x as f32*8.;
                        assert!(mesh.camera_wall_textured_ranges.iter().any(|(range,plane)| {
                            if *plane!=[side,0.,side*at] {return false;}
                            let points=&mesh.textured.positions[range.clone()];
                            let lo=points.iter().map(|p|p[2]).fold(f32::INFINITY,f32::min);
                            let hi=points.iter().map(|p|p[2]).fold(f32::NEG_INFINITY,f32::max);
                            lo<=0.001&&hi>=12.999&&hi<=13.001
                        }),"{map}: short return must meet the rear body without coplanar overlap");
                    }
                }
                assert_eq!(mesh.authored_cells,vec![None;w*h],"inferred edges do not claim grating or floor");
            }
            if map=="DayCare" {
                assert_eq!(tin_room_bounds::insets(&frame,&refs,&g,w as i32,h as i32),[0,4,0]);
                assert!(mesh.camera_wall_textured_ranges.iter().any(|(_,plane)|*plane==[-1.,0.,-128.]));
                assert!(mesh.textured.positions.iter().all(|p|p[0]<=131.001),"Day Care returns meet its floor edge");
                let mut changed=tiles.clone();changed[w-1].source.tile_index=0xffff;
                assert_eq!(tin_room_bounds::insets(&frame,&changed.iter().collect::<Vec<_>>(),&g,w as i32,h as i32),[0;3]);
                let mut other=frame.clone();other.map_id="TrainerHouse1F".into();
                assert_eq!(tin_room_bounds::insets(&other,&refs,&g,w as i32,h as i32),[0;3]);
                assert_eq!(tin_room_bounds::insets(&frame,&refs,&g,w as i32+4,h as i32),[0;3]);
            }
            if map=="CopycatsHouse2F" {
                assert_eq!(tin_room_bounds::insets(&frame,&refs,&g,w as i32,h as i32),[0,4,0]);
                assert!(mesh.camera_wall_textured_ranges.iter().any(|(_,plane)|*plane==[-1.,0.,-128.]));
                assert!(mesh.textured.positions.iter().all(|p|p[0]<=131.001),
                    "the side and front returns must meet the actual floor edge, not blank padding");
                let mut changed=tiles.clone();
                changed[w-1].source.tile_index=0xffff;
                assert_eq!(tin_room_bounds::insets(&frame,&changed.iter().collect::<Vec<_>>(),&g,w as i32,h as i32),[0;3]);
                let mut other=frame.clone();other.map_id="RedsHouse2F".into();
                assert_eq!(tin_room_bounds::insets(&other,&refs,&g,w as i32,h as i32),[0;3]);
            }
            if matches!(map.as_str(),"SproutTower1F"|"SproutTower2F"|"SproutTower3F") {
                let inset=if map=="SproutTower3F" {8}else{4};
                assert_eq!(tin_room_bounds::insets(&frame,&refs,&g,w as i32,h as i32),[inset,inset,0]);
                for warp in &data.events.warps {
                    assert_eq!(tin_room_bounds::opens_perimeter(map,IVec2::new(warp.x.into(),warp.y.into())),
                        warp.target_map=="VIOLET_CITY","internal stairs retain continuous wall backing");
                }
                for side in [1.,-1.] {
                    let at=if side>0. {inset as f32*8.}else{(w as i32-inset)as f32*8.};
                    let mut spans=Vec::new();
                    for (surface,ranges) in [(&mesh.solid,&mesh.camera_wall_solid_ranges),
                        (&mesh.textured,&mesh.camera_wall_textured_ranges)] {
                        for (range,plane) in ranges {
                            if plane[0]!=side {continue;}
                            assert_eq!(plane[2],side*at,"returns meet the actual room's timber corners");
                            let points=&surface.positions[range.clone()];
                            spans.push((points.iter().map(|p|p[2]).fold(f32::INFINITY,f32::min),
                                points.iter().map(|p|p[2]).fold(f32::NEG_INFINITY,f32::max)));
                        }
                    }
                    spans.sort_by(|a,b|a.0.total_cmp(&b.0));
                    assert!(!spans.is_empty());
                    let mut end=spans[0].0;
                    for (start,stop) in spans {
                        assert!(start<=end+0.001,"{map}: continuous side backing behind ladders");
                        end=end.max(stop);
                    }
                    assert!(end>=h as f32*8.);
                }
                let mut changed=tiles.clone();changed[w*4].source.tile_index=2;
                assert_eq!(tin_room_bounds::insets(&frame,&changed.iter().collect::<Vec<_>>(),&g,w as i32,h as i32),
                    [0,inset,0],"edited black padding cannot move a wall");
                if map=="SproutTower3F" {
                    changed=tiles.clone();changed[w*4+4].source.tile_index=2;
                    assert_eq!(tin_room_bounds::insets(&frame,&changed.iter().collect::<Vec<_>>(),&g,w as i32,h as i32),
                        [0,8,0],"edited lattice cannot be inferred as exterior void");
                    assert!(mesh.textured.positions.iter().chain(&mesh.solid.positions)
                        .all(|p|p[0]>=61.-0.001&&p[0]<=259.+0.001),
                        "no detached side wall may remain over the outer lattice");
                }
            }
            if map=="CeruleanGym" {
                assert!(cerulean_room::source(&mesh,&frame,&refs,&g).is_some());
                assert!(mesh.solid.indices.is_empty());
                assert_eq!(mesh.textured.positions.iter().map(|p|p[1]).fold(0.,f32::max),32.);
                assert!(mesh.textured.positions.iter().all(|p|p[2]>=13.-0.002),"north padding stays outside the room");
                assert!(mesh.camera_wall_textured_ranges.iter().any(|(_,p)|*p==[0.,1.,16.]));
                for (range,plane) in &mesh.camera_wall_textured_ranges {
                    assert!(mesh.textured.cutaway_ranges.contains(range),"every camera wall supports cutaway");
                    if plane[1]==-1. {
                        assert!(mesh.textured.positions[range.clone()].iter().all(|p|p[0]<=64.||p[0]>=96.),
                            "both authoritative exit tiles remain open");
                    }
                }
                for triangle in mesh.textured.indices.chunks_exact(3) {
                    let [a,b,c]=[0,1,2].map(|j|Vec3::from_array(mesh.textured.positions[triangle[j]as usize]));
                    let normal=Vec3::from_array(mesh.textured.normals[triangle[0]as usize]);
                    assert!((b-a).cross(c-a).dot(normal)>0.,"outward wall faces remain visible");
                }
                for (i,tile) in tiles.iter().enumerate() {
                    let (x,y)=(i%w,i/w);
                    if !(x<2||x>=18||y<4||y>=28) {continue;}
                    let mut changed=tiles.clone();changed[i].source.tile_index=0xffff;
                    assert!(cerulean_room::source(&mesh,&frame,&changed.iter().collect::<Vec<_>>(),&g).is_none(),
                        "changed boundary cell {x},{y} cannot seed walls: {:?}",tile.source);
                }
                let mut other=frame.clone();other.map_id="VermilionPort".into();
                assert!(cerulean_room::source(&mesh,&other,&refs,&g).is_none());
                other=frame.clone();
                let mut warps=other.boundary_warps.to_vec();warps.pop();
                other.boundary_warps=warps.into();
                assert!(cerulean_room::source(&mesh,&other,&refs,&g).is_none());
            }
            if mesh.solid.indices.is_empty() && mesh.textured.indices.is_empty() {
                missing
                    .entry(tileset.to_string())
                    .or_default()
                    .push(map.clone());
            } else {
                let inset = tin_room_bounds::insets(&frame, &refs, &g, w as i32, h as i32);
                if crate::mesh::modeled_dungeons::tin_tower_guardian_map(map) {
                    assert_eq!(inset, [4;3], "all nine tower floors use the inner padding edge");
                    let mut altered = tiles.clone();
                    altered[w * 4].source.tile_index = 2;
                    let altered_refs = altered.iter().collect::<Vec<_>>();
                    assert_eq!(tin_room_bounds::insets(&frame, &altered_refs, &g, w as i32, h as i32),
                        [0,4,4], "an edited boundary is never inferred as blank padding");
                    let mut other = frame.clone();
                    other.map_id = "SproutTower1F".into();
                    assert_eq!(tin_room_bounds::insets(&other, &refs, &g, w as i32, h as i32),
                        [0;3], "other tower architecture retains its original boundaries");
                    // A wall can exist at both ends while still having a doorway-sized
                    // hole behind a stair. Check the complete inner side span.
                    for side in [1., -1.] {
                        let at = if side > 0. { 32. } else { (w as f32 - 4.) * 8. };
                        let mut spans = Vec::new();
                        for (surface, ranges) in [(&mesh.solid, &mesh.camera_wall_solid_ranges),
                            (&mesh.textured, &mesh.camera_wall_textured_ranges)] {
                            for (range, plane) in ranges {
                                // Internal north returns have the same orientation,
                                // but belong to their own source-column plane.
                                if *plane != [side, 0., side * at] { continue; }
                                let points = &surface.positions[range.clone()];
                                assert!(points.iter().any(|p| (p[0] - at).abs() < 0.001),
                                    "{map}: outer side wall meets the source padding edge");
                                spans.push((points.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min),
                                    points.iter().map(|p| p[2]).fold(f32::NEG_INFINITY, f32::max)));
                            }
                        }
                        assert!(!spans.is_empty(), "{map}: outer side {side} wall exists");
                        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
                        let mut covered_to = if tin_north_outline::gaps(&frame, &refs, &g, None)
                            .unwrap().is_empty() { 13. } else { 0. };
                        for (start, end) in spans {
                            assert!(start <= covered_to + 0.001,
                                "{map}: side {side} wall gap from {covered_to} to {start}");
                            covered_to = covered_to.max(end);
                        }
                        assert!(covered_to >= (h as f32 - 4.) * 8.,
                            "{map}: side {side} wall stops before the south corner");
                    }
                }
                let points = mesh
                    .solid
                    .positions
                    .iter()
                    .chain(&mesh.textured.positions)
                    .collect::<Vec<_>>();
                assert!(
                    points.iter().any(|p| p[0] <= inset[0] as f32 * 8. && p[1] > 8.),
                    "{map}: west wall"
                );
                assert!(
                    points.iter().any(|p| p[0] >= (w as i32 - inset[1]) as f32 * 8. && p[1] > 8.),
                    "{map}: east wall"
                );
                if map == "TrainerHouseB1F" {
                    // Its green upper returns join the source-owned gold arena.
                    // Verify the actual arena closes the front instead of
                    // requiring a second inferred green wall behind it.
                    assert!(points.iter().all(|p| p[2] <= 96.001));
                    let mut arena = TerrainMeshData::default();
                    let mut claims = vec![false; w*h];
                    crate::mesh::append_facility_divider_network(&mut arena,map,&refs,&g,&mut claims).unwrap();
                    for x in 0..w {
                        let sample = (x as f32 + 0.5)*8.;
                        // The corner drawings end in a gray footing cell;
                        // their solid wall ends one source cell before the
                        // middle front course. Preserve that native outline.
                        let front = if x<2 || x>=w-2 {h-1}else{h} as f32*8.;
                        assert!(arena.textured.positions.chunks_exact(4)
                            .zip(arena.textured.normals.chunks_exact(4)).any(|(p,n)| {
                                n[0][2] > 0.99 && p.iter().all(|p| (p[2]-front).abs()<0.001)
                                    && p.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min)<=sample
                                    && p.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max)>=sample
                                    && p.iter().any(|p|p[1]>=16.)
                            }),"{map}: source arena front remains closed at column{x}");
                    }
                } else {
                    assert!(
                        points.iter().any(|p| p[2] >= (h as i32 - inset[2]) as f32 * 8. && p[1] > 8.),
                        "{map}: front wall"
                    );
                }
                assert_eq!(
                    mesh.footing_heights,
                    vec![2.75; w * h],
                    "{map}: floors remain authoritative"
                );
                assert_eq!(
                    mesh.authored_cells,
                    vec![None; w * h],
                    "{map}: no source cells replaced"
                );
                for surface in [&mesh.solid, &mesh.textured] {
                    assert_eq!(surface.positions.len(), surface.normals.len());
                    assert_eq!(surface.positions.len(), surface.uvs.len());
                    assert!(
                        surface
                            .normals
                            .iter()
                            .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
                    );
                    assert_eq!(
                        surface
                            .cutaway_ranges
                            .iter()
                            .map(|r| r.len())
                            .sum::<usize>(),
                        surface.positions.len()
                    );
                }
                for (surface, sections) in [
                    (&mesh.solid, &mesh.camera_wall_solid_ranges),
                    (&mesh.textured, &mesh.camera_wall_textured_ranges),
                ] {
                    for (range, plane) in sections {
                        let points = &surface.positions[range.clone()];
                        // BattleRoom's source-art trim deliberately clears its
                        // structural backing by0.02px. Its dedicated regression
                        // pins that exact offset; keep the body boundary strict.
                        let trim=if map=="BattleTowerBattleRoom" {
                            surface.indices.chunks_exact(3).filter(|t|
                                t.iter().all(|&i|range.contains(&(i as usize)))
                                &&t.iter().any(|&i|surface.uvs[i as usize]!=surface.uvs[t[0]as usize]))
                                .flat_map(|t|t.iter().map(|&i|i as usize))
                                .collect::<std::collections::BTreeSet<_>>()
                        } else {Default::default()};
                        for (offset,p) in points.iter().enumerate() {
                            let relief=if trim.contains(&(range.start+offset)) {0.02*g.tile_height/8.}else{0.};
                            assert!(plane[0]*p[0]+plane[1]*p[2]<=plane[2]+relief+0.001,
                                "{map}: wall body or source trim exceeds its physical boundary");
                        }
                        for warp in frame.boundary_warps.iter() {
                            if !tin_room_bounds::opens_perimeter(map, *warp) {
                                continue;
                            }
                            let horizontal = plane[1] < 0.;
                            let exit = if horizontal {
                                warp.y >= (h as i32 - inset[2]) / 2 - 2
                            } else if plane[0] > 0. {
                                warp.x == inset[0] / 2
                            } else {
                                warp.x == (w as i32 - inset[1]) / 2 - 1
                            };
                            if !exit {
                                continue;
                            }
                            let axis = if horizontal { 0 } else { 2 };
                            let coordinate = if horizontal { warp.x } else { warp.y };
                            let start = coordinate as f32 * 16.;
                            let end = start + 16.;
                            let min = points.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min);
                            let max = points
                                .iter()
                                .map(|p| p[axis])
                                .fold(f32::NEG_INFINITY, f32::max);
                            assert!(
                                max <= start + 0.001 || min >= end - 0.001,
                                "{map}: boundary warp {warp:?} closed"
                            );
                        }
                    }
                }
            }
        }
        eprintln!("rooms={total} missing={missing:?}");
        assert!(total > 0);
        assert!(
            missing.is_empty(),
            "every enclosed map needs side and front walls: {missing:?}"
        );
    }
}
