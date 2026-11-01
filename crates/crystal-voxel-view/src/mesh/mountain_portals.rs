//! Complete native mountain entrance drawings, with an open recessed mouth.
//! The source still owns footing and warps; this only replaces their artwork.
use super::*;
use bevy::math::Vec2;

#[derive(Clone,Debug,PartialEq)]
pub(super) struct Portal {
    pub vertices:std::ops::Range<usize>,
    pub height:f32,
}

pub(super) struct Site {
    x:usize,
    y:usize,
    height:f32,
    label:&'static str,
}

impl Site {
    pub(super) fn owns_cell(&self,x:usize,y:usize)->bool {
        (self.x..self.x+2).contains(&x) && (self.y..self.y+2).contains(&y)
    }

    pub(super) fn body_rise(&self,point:Vec2,tile:f32)->f32 {
        let distance=point.distance(point.clamp(Vec2::new(self.x as f32,self.y as f32),
            Vec2::new(self.x as f32+2.,self.y as f32+2.)));
        let weight=(1.-distance/4.).clamp(0.,1.);
        self.height.max(tile*3.7)*weight*weight*(3.-2.*weight)
    }

    pub(super) fn roof_rise(&self,point:Vec2,tile:f32)->f32 {
        let distance=point.distance(point.clamp(Vec2::new(self.x as f32,self.y as f32),
            Vec2::new(self.x as f32+2.,self.y as f32+2.)));
        let weight=(1.-distance/2.).clamp(0.,1.);
        // Only short cave roofs need extra rock. Blend it into the adjoining
        // mountain over two tiles, keeping the route-wide relief unchanged.
        (tile*3.7-self.height).max(0.)*weight*weight*(3.-2.*weight)
    }
}

pub(super) fn sites(cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry)->Vec<Site> {
    let mut sites=Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        let s = &cell.source;
        let (metatile, column, art, label) = match s.tileset_id.as_ref() {
            "johto" => (0x73, 0, [[0x46, 0x47], [0x56, 0x57]], "johto/mountain_cave"),
            "johto_modern" => (0x73, 2, [[0x45, 0x22], [0x27, 0x28]], "johto/mountain_cave"),
            // The tall northern cliff embeds the same complete mouth in
            // block 70; block 06 supplies the separate low mound entrance.
            "kanto" if matches!(s.metatile_id, 0x06 | 0x70) => (
                s.metatile_id,
                0,
                [[0x48, 0x49], [0x58, 0x59]],
                "outdoor/kanto_mountain_cave",
            ),
            _ => continue,
        };
        if s.metatile_id != metatile || s.subtile_column != column || s.subtile_row != 2 {
            continue;
        }
        let (x, y) = (i % g.width, i / g.width);
        if x + 2 > g.width || y + 2 > g.height {
            continue;
        }
        let complete = (0..2).all(|dy| {
            (0..2).all(|dx| {
                let at = (y + dy) * g.width + x + dx;
                let other = &cells[at].source;
                other.tileset_id == s.tileset_id
                    && other.metatile_id == s.metatile_id
                    && usize::from(other.subtile_column) == usize::from(column) + dx
                    && usize::from(other.subtile_row) == 2 + dy
                    && other.tile_index == art[dy][dx]
            })
        });
        if !complete {
            continue;
        }
        // Use the same resolved local mountain datum, including stacked tiers.
        let mut h = g.tile_height * crate::profile::MOUNTAIN_CLIFF_HEIGHT / SOURCE_TILE_HEIGHT;
        for row in y.saturating_sub(1)..=(y + 1).min(g.height - 1) {
            for col in x.saturating_sub(1)..=(x + 2).min(g.width - 1) {
                if matches!(
                    shapes[row * g.width + col],
                    CellShape::RaisedTop {
                        solid: SolidKind::Bank,
                        ..
                    } | CellShape::LedgeBand { .. }
                ) {
                    h = h.max(shapes[row * g.width + col].surface_height(g.tile_height));
                }
            }
        }
        sites.push(Site {x,y,height:h,label});
    }
    sites
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
    origin: [i32; 2],
) {
    for site in sites(cells,shapes,g) {
        let Site {x,y,height:h,label}=site;
        if (0..2).any(|dy|(0..2).any(|dx|claimed[(y+dy)*g.width+x+dx])) {continue;}
        let (w, _, n, _) = g.bounds(x, y);
        let (_, e, _, south) = g.bounds(x + 1, y + 1);
        let t = g.tile_height;
        let side = t * 0.15;
        let portal_start=mesh.solid.positions.len();
        rock_block(
            &mut mesh.solid,
            [w, w + side, 0., h, n, south],
            g,
            origin,
            1.,
        );
        rock_block(
            &mut mesh.solid,
            [e - side, e, 0., h, n, south],
            g,
            origin,
            -1.,
        );
        let span = e - w - side * 2.;
        let arch = [
            (0., 0.68),
            (0.22, 0.84),
            (0.5, 0.90),
            (0.78, 0.84),
            (1., 0.68),
        ];
        for segment in arch.windows(2) {
            let (x0, y0) = (w + side + span * segment[0].0, h * segment[0].1);
            let (x1, y1) = (w + side + span * segment[1].0, h * segment[1].1);
            for (face, points) in [
                [
                    [x0, y0, south],
                    [x1, y1, south],
                    [x1, h, south],
                    [x0, h, south],
                ],
                [[x1, y1, n], [x0, y0, n], [x0, h, n], [x1, h, n]],
                [[x0, h, n], [x0, h, south], [x1, h, south], [x1, h, n]],
                [[x0, y0, south], [x0, y0, n], [x1, y1, n], [x1, y1, south]],
            ]
            .into_iter()
            .enumerate()
            {
                let p = points.map(Vec3::from_array);
                let normal = (p[1] - p[0]).cross(p[3] - p[0]).normalize().to_array();
                let first = mesh.solid.colors.len();
                outdoor_stone::append_canyon(&mut mesh.solid, points, normal, g, origin);
                if face == 2 {
                    // Keep the whole source perimeter at the neighboring rock
                    // datum. A raised perimeter tears away from the mountain.
                    // Only the cap's interior receives this closed mineral crest.
                    for p in &mut mesh.solid.positions[first..] {
                        let u = ((p[0] - w - side) / span).clamp(0., 1.);
                        let v = ((p[2] - n) / (south - n)).clamp(0., 1.);
                        let crest = u.min(1. - u).min(v.min(1. - v)) * 2.;
                        p[1] += t * 0.16 * crest;
                    }
                }
                if face == 3 {
                    shade_tunnel(&mut mesh.solid, first, n, south);
                }
            }
        }
        // Only the recessed vertical wall is dark. A closed dark cuboid
        // exposes its top above the mineral-deformed arch as a black cap seam.
        // Extend the vertical backdrop into the closed roof. Stopping below
        // the intrados leaves a sky window when the entrance is viewed low.
        for segment in arch.windows(2) {
            let x0 = w + side + span * segment[0].0;
            let x1 = w + side + span * segment[1].0;
            append_quad(
                &mut mesh.solid,
                [
                    [x0, 0., n + t * 0.10],
                    [x1, 0., n + t * 0.10],
                    [x1, h, n + t * 0.10],
                    [x0, h, n + t * 0.10],
                ],
                [0., 0., 1.],
                [[0., 0.]; 4],
                [0.0015, 0.0012, 0.0008, 1.],
            );
        }
        // A visible threshold fades into the same dark recess. A uniformly
        // sunlit floor and lining made the opening read as a gray flat panel.
        let floor = t * 0.015;
        append_quad_colors(
            &mut mesh.solid,
            [
                [w + side, floor, n],
                [w + side, floor, south],
                [e - side, floor, south],
                [e - side, floor, n],
            ],
            [0., 1., 0.],
            [[0., 0.]; 4],
            [
                [0.008, 0.006, 0.004, 1.],
                [0.16, 0.115, 0.065, 1.],
                [0.16, 0.115, 0.065, 1.],
                [0.008, 0.006, 0.004, 1.],
            ],
        );
        // The threshold is slightly above its authored walking floor. Close
        // that thin slab instead of exposing sky through its paper-thin edge.
        let (left, right) = (w + side, e - side);
        for (points, normal) in [
            (
                [
                    [left, 0., south],
                    [right, 0., south],
                    [right, floor, south],
                    [left, floor, south],
                ],
                [0., 0., 1.],
            ),
            (
                [
                    [left, 0., n],
                    [left, 0., south],
                    [left, floor, south],
                    [left, floor, n],
                ],
                [-1., 0., 0.],
            ),
            (
                [
                    [right, 0., south],
                    [right, 0., n],
                    [right, floor, n],
                    [right, floor, south],
                ],
                [1., 0., 0.],
            ),
            (
                [
                    [right, 0., n],
                    [left, 0., n],
                    [left, floor, n],
                    [right, floor, n],
                ],
                [0., 0., -1.],
            ),
            (
                [
                    [left, 0., n],
                    [right, 0., n],
                    [right, 0., south],
                    [left, 0., south],
                ],
                [0., -1., 0.],
            ),
        ] {
            append_quad_colors(
                &mut mesh.solid,
                points,
                normal,
                [[0., 0.]; 4],
                points.map(|p| {
                    let t = ((p[2] - n) / (south - n)).clamp(0., 1.);
                    [0.008 + 0.152 * t, 0.006 + 0.109 * t, 0.004 + 0.061 * t, 1.]
                }),
            );
        }
        mesh.mountain_portals.push(Portal {vertices:portal_start..mesh.solid.positions.len(),height:h});
        for dy in 0..2 {
            for dx in 0..2 {
                let at = (y + dy) * g.width + x + dx;
                claimed[at] = true;
                mesh.authored_cells[at] = Some(label);
                if let Some(footing)=mesh.footing_heights.get_mut(at) {*footing=0.;}
            }
        }
    }
}

fn shade_tunnel(out: &mut SurfaceMeshData, first: usize, back: f32, front: f32) {
    for (p, c) in out.positions[first..].iter().zip(&mut out.colors[first..]) {
        let daylight = 0.06 + 0.24 * ((p[2] - back) / (front - back)).clamp(0., 1.);
        for value in &mut c[..3] {
            *value *= daylight;
        }
    }
}

fn rock_block(
    out: &mut SurfaceMeshData,
    bounds: [f32; 6],
    g: &GridGeometry,
    origin: [i32; 2],
    inner_normal: f32,
) {
    let mut block = SurfaceMeshData::default();
    super::modeled_exteriors::portal_block(&mut block, bounds, [0.49, 0.36, 0.20]);
    for (p, n) in block
        .positions
        .chunks_exact(4)
        .zip(block.normals.chunks_exact(4))
    {
        let first = out.colors.len();
        outdoor_stone::append_canyon(out, p.try_into().unwrap(), n[0], g, origin);
        if n[0][0] == inner_normal {
            shade_tunnel(out, first, bounds[4], bounds[5]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raised_portal_stays_closed_above_its_open_dark_mouth() {
        use bevy::prelude::{IVec2,UVec2,Vec3Swizzles};
        for (raised_ground,native_height,ridge_height) in [
            (false,16.,16.),(true,16.,16.),(false,32.,16.),(true,32.,16.),
            (false,16.,64.),(true,16.,64.),(false,32.,64.),(true,32.,64.),
        ] {
            let mut flat=super::super::tests::source_with_tile(1,0,0,6);
            flat.tileset_id="johto_modern".into();
            let mut rock=super::super::tests::source_with_tile(0x0a,1,1,0x3c);
            rock.tileset_id="johto_modern".into();
            let mut sources=vec![flat;144];
            for z in 2..10 {for x in 2..10 {sources[z*12+x]=rock.clone();}}
            let art=[[0x45,0x22],[0x27,0x28]];
            for z in 0..2 {for x in 0..2 {
                let mut s=super::super::tests::source_with_tile(0x73,2+x as u8,2+z as u8,art[z][x]);
                s.tileset_id="johto_modern".into();sources[(6+z)*12+5+x]=s;
            }}
            let frame=super::super::tests::frame(12,12,sources);
            let cells=frame.tiles.iter().collect::<Vec<_>>();
            let mut shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
            shapes[2*12+2]=CellShape::RaisedTop {height:ridge_height,solid:SolidKind::Bank};
            shapes[5*12+5]=CellShape::RaisedTop {height:native_height,solid:SolidKind::Bank};
            let g=GridGeometry {width:12,height:12,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let mut mesh=TerrainMeshData {authored_cells:vec![None;144],
                footing_heights:vec![0.;144],..Default::default()};
            for z in 6..8 {for x in 5..7 {mesh.footing_heights[z*12+x]=native_height;}}
            let mut claimed=vec![false;144];
            append(&mut mesh,&cells,&shapes,&g,&mut claimed,[0,0]);
            for z in 6..8 {for x in 5..7 {
                assert_eq!(mesh.footing_heights[z*12+x],0.,
                    "a cave's old roof drawing is not its walking floor");
            }}
            assert_eq!(mesh.mountain_portals.len(),1);
            let portal=mesh.mountain_portals[0].clone();
            let ground_start=mesh.solid.positions.len();
            // The roof field reaches over this ordinary path. Its real floor
            // must remain at source ground rather than following the ceiling.
            append_solid_quad(&mut mesh.solid,
                [[40.,0.,64.],[40.,0.,72.],[56.,0.,72.],[56.,0.,64.]],
                [0.,1.,0.],[0.2,0.4,0.1,1.]);
            let ground=ground_start..mesh.solid.positions.len();
            super::super::modeled_dungeons::append_terrain(&mut mesh,"BlackthornCity",&cells,&shapes,&g,&mut claimed,[0,0]);
            let before=mesh.solid.positions.clone();
            let before_normals=mesh.solid.normals.clone();
            let datum=raised_ground.then(||crystal_render_api::VisualTerraceDatum {
                origin:IVec2::ZERO,size:UVec2::new(12,12),levels:vec![[2.;4];144].into(),
                walkable_mineral:Default::default(),water_barriers:Default::default(),open_water:None,
            });
            super::super::outdoor_stone::finish_rock_geometry(&mut mesh,"BlackthornCity",&cells,&shapes,&g,[0,0],datum.as_ref());
            if let Some(datum)=&datum {
                super::super::terraces::finish_with_datum(&mut mesh,"BlackthornCity",&cells,&g,[0,0],&[],Some(datum));
            }
            assert!(mesh.mountain_portals.is_empty(),"temporary portal ownership is consumed before upload");
            let (_,_,_,front)=g.bounds(6,7);
            let roof=portal.vertices.clone().filter(|&i|
                (before[i][1]-portal.height).abs()<0.001 && (before[i][2]-front).abs()<0.001)
                .map(|i|mesh.solid.positions[i][1]).fold(f32::INFINITY,f32::min);
            assert!(roof>=g.tile_height*3.7-0.001,
                "a short source roof must grow enough rock for the standing opening");
            let floor=if raised_ground {2.*crate::profile::JUMP_LEDGE_HEIGHT}else{0.};
            for z in 6..8 {for x in 5..7 {
                assert_eq!(mesh.footing_heights[z*12+x],floor,
                    "the complete entrance follows its real ground datum");
            }}
            for i in ground {
                assert_eq!(mesh.solid.positions[i],[before[i][0],before[i][1]+floor,before[i][2]],
                    "ordinary path geometry under the roof keeps its real source floor");
            }
            let lowest_roof=portal.vertices.clone().filter(|&i|
                (before[i][1]-portal.height).abs()<0.001)
                .map(|i|mesh.solid.positions[i][1]-floor).fold(f32::INFINITY,f32::min);
            let opening=portal.height*0.90;
            let clearance=opening.max((g.tile_height*3.3).min(lowest_roof*0.90));
            let hit=|x:f32,y:f32| {
                let origin=Vec3::new(x,y,front+g.tile_height);
                let direction=Vec3::NEG_Z;
                let mut nearest=None::<(f32,usize)>;
                for tri in mesh.solid.indices.chunks_exact(3) {
                    let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.solid.positions[i as usize]));
                    let (ab,ac)=(b-a,c-a);
                    let cross=direction.cross(ac);let determinant=ab.dot(cross);
                    if determinant<=0.000001 {continue;}
                    let t=origin-a;let u=t.dot(cross)/determinant;
                    let q=t.cross(ab);let v=direction.dot(q)/determinant;
                    let distance=ac.dot(q)/determinant;
                    if u>=-0.0001 && v>=-0.0001 && u+v<=1.0001 && distance>=0.
                        && nearest.is_none_or(|(d,_)|distance<d) {nearest=Some((distance,tri[0] as usize));}
                }
                nearest.expect("the source cave cannot expose sky through its back or roof")
            };
            for u in [0.02,0.10,0.25,0.50,0.75,0.90,0.98] {for v in [0.94,0.97] {
                let (distance,i)=hit((5.+u*2.)*g.tile_width,floor+(roof-floor)*v);
                assert!(distance<g.tile_height*1.2 && mesh.solid.uvs[i][0]< -9000.,
                    "the raised front must be continuous mineral above the mouth at {u},{v}: {distance}");
            }}
            let (_,i)=hit(6.*g.tile_width,floor+portal.height*0.4);
            assert!(mesh.solid.colors[i][0]<0.003 && mesh.solid.uvs[i]==[0.;2],
                "the actual doorway stays open to its dark recess");
            let (_,i)=hit(6.*g.tile_width,floor+g.tile_height*2.9);
            assert!(mesh.solid.colors[i][0]<0.003 && mesh.solid.uvs[i]==[0.;2],
                "the trainer's head must fit inside the mountain entrance");
            for i in portal.vertices.clone() {
                assert!(Vec3::from_array(before[i]).xz().distance(Vec3::from_array(mesh.solid.positions[i]).xz())<0.001,
                    "the complete entrance keeps its source footprint");
                if before_normals[i][1]< -0.3 && before[i][1]<=portal.height*0.90 {
                    let seat=g.tile_height*0.015;
                    let expected=if before[i][1]<=seat {before[i][1]}
                        else {seat+(before[i][1]-seat)*(clearance-seat)/(opening-seat)};
                    assert!((mesh.solid.positions[i][1]-expected-floor).abs()<0.001,
                        "the doorway's underside retains standing clearance without following the full roof rise");
                }
            }
            for z in 6..8 {for x in 5..7 {
                assert!((mesh.footing_heights[z*12+x]-floor).abs()<0.001,
                    "the source warp floor retains ordinary terrain support");
            }}
            for x in 5..7 {
                assert!((mesh.footing_heights[8*12+x]-floor).abs()<0.001,
                    "the ordinary approach path cannot inherit overhead roof height");
            }
        }
    }

    #[test]
    fn complete_portals_seat_only_the_four_source_cells_on_their_real_floor() {
        for (tileset, metatile, first_column, art) in [
            ("johto", 0x73, 0, [[0x46, 0x47], [0x56, 0x57]]),
            ("johto_modern", 0x73, 2, [[0x45, 0x22], [0x27, 0x28]]),
            ("kanto", 0x06, 0, [[0x48, 0x49], [0x58, 0x59]]),
            ("kanto", 0x70, 0, [[0x48, 0x49], [0x58, 0x59]]),
        ] {
            let mut sources = vec![super::super::tests::source_with_tile(1, 0, 0, 6); 16];
            for y in 0..2 {
                for x in 0..2 {
                    let mut source = super::super::tests::source_with_tile(
                        metatile,
                        first_column + x as u8,
                        2 + y as u8,
                        art[y][x],
                    );
                    source.tileset_id = tileset.into();
                    sources[(y + 1) * 4 + x + 1] = source;
                }
            }
            let mut frame = super::super::tests::frame(4, 4, sources);
            let g = GridGeometry {
                width: 4,
                height: 4,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: -16.,
                origin_z: -16.,
            };
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![7.; 16];
            mesh.authored_cells = vec![None; 16];
            let mut claimed = vec![false; 16];
            let cells = frame.tiles.iter().collect::<Vec<_>>();
            let shapes = cells
                .iter()
                .map(|cell| shape_for_source(&cell.source))
                .collect::<Vec<_>>();
            append(&mut mesh, &cells, &shapes, &g, &mut claimed, [0, 0]);
            let mut expected=vec![7.;16];
            for i in [5,6,9,10] {expected[i]=0.;}
            assert_eq!(mesh.footing_heights,expected,
                "only the complete mouth receives the source-ground floor; other support is untouched");
            assert_eq!(claimed.iter().filter(|v| **v).count(), 4);
            assert!(mesh.textured.positions.is_empty());
            assert!(!mesh.solid.indices.is_empty());
            let darkness = mesh
                .solid
                .colors
                .iter()
                .zip(&mesh.solid.normals)
                .filter(|(color, normal)| color[0] < 0.003 && color[1] < 0.003 && normal[2] > 0.99)
                .count();
            assert!(
                darkness >= 16,
                "the complete arched recess needs a dark backdrop"
            );
            for (color, normal) in mesh.solid.colors.iter().zip(&mesh.solid.normals) {
                if color[0] < 0.003 && color[1] < 0.003 {
                    assert_eq!(normal[1], 0., "the dark recess must not expose a black cap");
                }
            }
            let floor = mesh
                .solid
                .positions
                .iter()
                .zip(&mesh.solid.colors)
                .filter(|(p, _)| (p[1] - g.tile_height * 0.015).abs() < 0.001)
                .map(|(_, c)| c[0])
                .collect::<Vec<_>>();
            assert!(
                floor.iter().any(|c| *c < 0.01) && floor.iter().any(|c| *c > 0.1),
                "a lit threshold must lead into a shaded tunnel floor"
            );
            let (_, _, _, front) = g.bounds(2, 2);
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .zip(&mesh.solid.normals)
                    .zip(&mesh.solid.uvs)
                    .any(|((p, n), uv)| (p[2] - front).abs() < 0.0001
                        && (p[1] - g.tile_height * 0.015).abs() < 0.0001
                        && n[2] > 0.99
                        && *uv == [0., 0.]),
                "the threshold front needs a solid riser beneath its visible top"
            );
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .zip(&mesh.solid.normals)
                    .zip(&mesh.solid.uvs)
                    .any(|((p, n), uv)| p[1] == 0. && n[1] < -0.99 && *uv == [0., 0.]),
                "the source-ground underside must close the threshold slab"
            );
            let h = g.tile_height * crate::profile::MOUNTAIN_CLIFF_HEIGHT / SOURCE_TILE_HEIGHT;
            let (_, _, north, _) = g.bounds(1, 1);
            let (_, _, _, south) = g.bounds(2, 2);
            let mut joins = 0;
            for ((p, normal), uv) in mesh
                .solid
                .positions
                .iter()
                .zip(&mesh.solid.normals)
                .zip(&mesh.solid.uvs)
            {
                if uv[0] < -9000.
                    && normal[1] > 0.9
                    && ((p[2] - north).abs() < 0.001 || (p[2] - south).abs() < 0.001)
                {
                    assert!(
                        (p[1] - h).abs() < 0.001,
                        "the mineral crown must meet its neighboring rock exactly"
                    );
                    joins += 1;
                }
            }
            assert!(
                joins > 20,
                "the real tessellated roof perimeter must be exercised"
            );
            frame.tiles[10].source.tile_index ^= 1;
            let mut rejected = TerrainMeshData::default();
            append(
                &mut rejected,
                &frame.tiles.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &mut vec![false; 16],
                [0, 0],
            );
            assert!(
                rejected.solid.positions.is_empty(),
                "a changed or partial source drawing stays untouched"
            );
        }
    }
}
