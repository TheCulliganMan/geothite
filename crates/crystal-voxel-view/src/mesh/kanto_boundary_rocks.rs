//! Complete Kanto boundary drawings replaced by original closed stone meshes.
//!
//! The existing classifiers own source identity and drawing size. This adapter
//! additionally requires a coherent native block and the correct live underlay.
//! It never reads collision and never changes the original footing heights.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct Placement {
    pub(super) rect: [usize; 4],
    sample: usize,
    kind: Kind,
    base: f32,
}

fn coherent(cells: &[&VisualTile], g: &GridGeometry, p: TreePlacement) -> bool {
    let first = &cells[p.row * g.width + p.column].source;
    // Every family occupies one quadrant of one native 4x4 block. Modulo-2
    // classification alone would also admit an unrelated allowed block or a
    // source quadrant moved by two cells into this drawing.
    (0..p.height).all(|y| {
        (0..p.width).all(|x| {
            let source = &cells[(p.row + y) * g.width + p.column + x].source;
            source.tileset_id == first.tileset_id
                && source.metatile_id == first.metatile_id
                && usize::from(source.subtile_column) == usize::from(first.subtile_column) + x
                && usize::from(source.subtile_row) == usize::from(first.subtile_row) + y
        })
    })
}

pub(super) fn resolve(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
) -> Vec<Placement> {
    resolve_with_datum(cells,shapes,g,[0,0],None)
}
fn resolve_with_datum(cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,
    origin:[i32;2],datum:Option<&crystal_render_api::VisualTerraceDatum>)->Vec<Placement> {
    // A copied border cell may have water art without live-phase evidence.
    // Prefer a proven water family so replacement beds retain the same clock
    // and finish as the surrounding ocean instead of becoming static squares.
    let live_water=cells.iter().zip(shapes).position(|(cell,shape)|
        cell.source.tileset_id.as_ref()=="kanto" && cell.source.tile_index==0x14
            && matches!(shape,CellShape::Water|CellShape::PlaneAt {..})
            && (shape.surface_height(g.tile_height)-crate::profile::WATER_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT).abs()<0.001
            && cell.animation_frames.as_ref().is_some_and(|f|f.len()>1 && f.contains(&cell.texture)));
    let mut placements = Vec::new();
    for (kind, groups) in [
        (
            Kind::KantoBoundaryLand,
            kanto_round_barrier_placements(cells, g),
        ),
        (
            Kind::KantoBoundaryShore,
            kanto_shoreline_round_barrier_placements(cells, g),
        ),
    ] {
        for p in groups {
            if !coherent(cells, g, p) {
                continue;
            }
            let wet=kind==Kind::KantoBoundaryLand && (0..p.height).all(|dy|(0..p.width).all(|dx| {
                let i=(p.row+dy)*g.width+p.column+dx;
                crate::kanto_cliff::water_backed_barrier_local(&cells[i].source,
                    bevy::prelude::IVec2::new(origin[0]+(p.column+dx)as i32,origin[1]+(p.row+dy)as i32),datum).is_some()
            }));
            let kind=if wet {Kind::KantoBoundaryShore}else{kind};
            let ground=if wet {0x14}else{p.ground_tile_index};
            let base = if wet {crate::profile::WATER_HEIGHT}else{p.base_height} * g.tile_height / SOURCE_TILE_HEIGHT;
            let Some(sample) = cells.iter().zip(shapes).position(|(cell, shape)| {
                cell.source.tileset_id.as_ref() == "kanto"
                    && cell.source.tile_index == ground
                    && matches!(
                        shape,
                        CellShape::Flat
                            | CellShape::Water
                            | CellShape::PlaneAt { .. }
                            | CellShape::RaisedTop {
                                solid: SolidKind::Bank,
                                ..
                            }
                    )
                    && (shape.surface_height(g.tile_height) - base).abs() < 0.001
            }) else {
                continue;
            };
            let sample=if kind==Kind::KantoBoundaryShore {live_water.unwrap_or(sample)}else{sample};
            placements.push(Placement {
                rect: [p.column, p.row, p.width, p.height],
                sample,
                kind,
                base,
            });
        }
    }
    placements
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
    map_origin: [i32; 2],
) {
    append_with_samples(mesh,cells,shapes,g,claimed,map_origin,None);
}

pub(super) fn append_with_samples(
    mesh:&mut TerrainMeshData,cells:&[&VisualTile],shapes:&[CellShape],
    g:&GridGeometry,claimed:&mut [bool],map_origin:[i32;2],
    images:Option<&TerrainImageSamples>,
) {
    append_with_datum(mesh,cells,shapes,g,claimed,map_origin,images,None);
}
pub(super) fn append_with_datum(mesh:&mut TerrainMeshData,cells:&[&VisualTile],shapes:&[CellShape],
    g:&GridGeometry,claimed:&mut [bool],map_origin:[i32;2],images:Option<&TerrainImageSamples>,
    datum:Option<&crystal_render_api::VisualTerraceDatum>) {
    for p in resolve_with_datum(cells, shapes, g,map_origin,datum) {
        if !clear(claimed, g, p.rect) {
            continue;
        }
        // floor uses this exact sample's datum, including recessed animated
        // water. All four consumed source cells retain an actual native top.
        let floor_first=mesh.textured.uvs.len();
        floor(mesh, cells, shapes, g, p.rect, p.sample, claimed);
        if p.kind==Kind::KantoBoundaryShore && cells[p.sample].animation_frames.as_ref()
            .is_some_and(|f|f.len()>1 && f.contains(&cells[p.sample].texture)) {
            // The borrowed live water cell is paint/clock evidence. Sampling
            // its boundary blends a neighboring dry mask into every new quad,
            // stamping square seams around otherwise continuous water.
            let (u0,u1,v0,v1)=g.uv(p.sample%g.width,p.sample/g.width);
            let (du,dv)=((u1-u0)/16.,(v1-v0)/16.);
            for uv in &mut mesh.textured.uvs[floor_first..] {
                uv[0]=uv[0].clamp(u0+du,u1-du);uv[1]=uv[1].clamp(v0+dv,v1-dv);
            }
        }
        let [west, east, north, south] = bounds(g, p.rect);
        let [inset_x,inset_z] = crate::exterior_models::KANTO_BOUNDARY_INSET;
        let fitted = [
            west + g.tile_width * inset_x,
            east - g.tile_width * inset_x,
            north + g.tile_height * inset_z,
            south - g.tile_height * inset_z,
        ];
        let m = variant_model(p.kind, [map_origin[0] + p.rect[0] as i32, map_origin[1] + p.rect[1] as i32]);
        // Shared, cached material meshes append to the existing single solid
        // terrain batch. No per-rock render entities, image copies or textures.
        let rise = g.tile_height * 1.55;
        let source=(0..p.rect[3]).flat_map(|y|(0..p.rect[2]).map(move |x|
            cells[(p.rect[1]+y)*g.width+p.rect[0]+x])).collect::<Vec<_>>();
        let paint=images.and_then(|images| {
            if !source.iter().all(|tile|tile_rgba(images,tile).is_ok_and(|pixels|
                pixels.len()==256 && pixels.chunks_exact(4).all(|p|p[3]==255))) {return None;}
            fence_palette(images,&source)
        });
        let first = mesh.solid.positions.len();
        m.append_fitted(
            &mut mesh.solid,
            fitted,
            p.base,
            rise / (m.max[1] - m.min[1]),
            None,
        );
        if p.kind == Kind::KantoBoundaryShore {
            // Sea barriers sit in the same standing water as their native
            // underlay. An island's terrace must not lift this offshore row.
            mesh.solid.uvs[first..].fill([super::shore_ecology::ROOTED_WATER_UV, 0.]);
        }
        if let Some(paint)=paint {
            let canonical=model(p.kind);
            let mut crown_vertices=vec![false;m.surface.positions.len()];
            for tri in m.surface.indices.chunks_exact(3) {
                let role=m.vertex_material(tri[0] as usize);
                if role.ends_with("Pale crown") || (role.ends_with("Mineral facet") && tri.iter().all(|&i|
                    (canonical.surface.positions[i as usize][1]-canonical.min[1])/(canonical.max[1]-canonical.min[1])>0.9)) {
                    for &i in tri {crown_vertices[i as usize]=true;}
                }
            }
            for i in first..mesh.solid.positions.len() {
                let at=m.surface.positions[i-first];
                let shore=p.kind==Kind::KantoBoundaryShore;
                let height=((at[1]-m.min[1])/(m.max[1]-m.min[1])).clamp(0.,1.);
                // Weathered sea rocks have a continuous mineral shoulder,
                // rather than an opaque white cap painted on a dark shaft.
                let crown=if shore {0.08+0.28*height*height*(3.-2.*height)}
                    else if crown_vertices[i-first] {1.}else{0.};
                let shade=0.76+0.16*mesh.solid.normals[i][1].max(0.);
                let pigment=std::array::from_fn::<_,3,_>(|c|
                    paint[1][c]*(1.-crown)+paint[2][c]*crown);
                let luma=pigment[0]*0.2126+pigment[1]*0.7152+pigment[2]*0.0722;
                let grain=0.97+0.025*(at[0]*37.+at[1]*29.+at[2]*41.).sin();
                for c in 0..3 {
                    let wet=if shore {0.78+0.22*(height*3.).min(1.)}else{1.};
                    mesh.solid.colors[i][c]=(pigment[c]*0.45+luma*0.55)*shade*grain*wet;
                }
                mesh.solid.uvs[i]=if p.kind==Kind::KantoBoundaryShore {
                    [super::shore_ecology::ROOTED_WATER_UV,85000.]
                } else {crate::scenery_finish::SOURCE_PAINT_UV};
            }
        }
        mark_authored_rect(mesh, g, p.rect, p.kind.label());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn fixture(kind: Kind, tile_height: f32) -> (Vec<VisualTile>, Vec<CellShape>, GridGeometry) {
        let (block, column, ground_tile, ground_shape) = match kind {
            Kind::KantoBoundaryLand => (0x4d, 2, 0x2c, CellShape::Flat),
            Kind::KantoBoundaryShore => (0x19, 2, 0x14, CellShape::Water),
            Kind::KantoBoundaryPath => (0x29, 2, 0x11, CellShape::Flat),
            _ => unreachable!(),
        };
        let mut cells = Vec::new();
        let mut shapes = Vec::new();
        for row in 0..2 {
            for x in 0..3 {
                let ground = x == 2;
                cells.push(VisualTile {
                    animation_frames: None,
                    column: x as u32,
                    row: row as u32,
                    source: VisualTileSource {
                        tileset_id: Arc::from("kanto"),
                        metatile_id: if ground { 0x01 } else { block },
                        subtile_column: if ground { 0 } else { column + x as u8 },
                        subtile_row: row as u8,
                        tile_index: if ground {
                            ground_tile
                        } else if kind == Kind::KantoBoundaryPath {
                            0x24
                        } else {
                            [[0x2a, 0x2b], [0x3a, 0x3b]][row][x]
                        },
                    },
                    texture: Handle::default(),
                    priority: false,
                });
                shapes.push(if ground {
                    ground_shape
                } else {
                    CellShape::Flat
                });
            }
        }
        (
            cells,
            shapes,
            GridGeometry {
                width: 3,
                height: 2,
                tile_width: 8.0,
                tile_height,
                origin_x: -24.0,
                origin_z: 40.0,
            },
        )
    }

    fn kinds() -> [Kind; 2] {
        [
            Kind::KantoBoundaryLand,
            Kind::KantoBoundaryShore,
        ]
    }

    #[test]
    fn cliff_side_drawing_has_no_boundary_rock_or_replacement_bed() {
        let (tiles,shapes,g)=fixture(Kind::KantoBoundaryPath,8.);
        let cells=tiles.iter().collect::<Vec<_>>();
        assert!(resolve(&cells,&shapes,&g).is_empty());
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;tiles.len()],
            authored_cells:vec![None;tiles.len()],..Default::default()};
        let before=mesh.clone();
        let mut claimed=vec![false;tiles.len()];
        append(&mut mesh,&cells,&shapes,&g,&mut claimed,[0,0]);
        assert_eq!(mesh,before,"cliff ink cannot invent a boulder or borrowed underlay");
        assert!(claimed.iter().all(|claimed|!*claimed));
    }

    #[test]
    fn borrowed_water_beds_prefer_authenticated_clock_over_static_border_art() {
        let (tiles,shapes,mut g)=fixture(Kind::KantoBoundaryShore,8.);
        let mut extended=Vec::new();let mut extended_shapes=Vec::new();
        let mut assets=bevy::prelude::Assets::<bevy::prelude::Image>::default();
        let a=assets.add(bevy::prelude::Image::default());let b=assets.add(bevy::prelude::Image::default());
        for y in 0..2 {
            extended.extend_from_slice(&tiles[y*3..y*3+3]);extended_shapes.extend_from_slice(&shapes[y*3..y*3+3]);
            let mut water=tiles[y*3+2].clone();water.column=3;
            water.texture=a.clone();water.animation_frames=Some(vec![a.clone(),b.clone()].into());
            extended.push(water);extended_shapes.push(CellShape::Water);
        }
        g.width=4;
        let mut mesh=TerrainMeshData::default();
        append(&mut mesh,&extended.iter().collect::<Vec<_>>(),&extended_shapes,&g,&mut vec![false;8],[0,0]);
        let (u0,u1,v0,v1)=g.uv(3,0);
        assert!(!mesh.solid.indices.is_empty());
        assert_eq!(mesh.textured.positions.len(),16);
        assert!(mesh.textured.uvs.iter().all(|uv|uv[0]>u0 && uv[0]<u1 && uv[1]>v0 && uv[1]<v1));
    }

    #[test]
    fn harbor_barrier_quadrants_use_water_without_consuming_the_path_or_entry_mat() {
        for (block,c,r) in [(0x13,0,0),(0x13,2,0),(0x13,0,2),(0x13,2,2),(0x46,0,2),(0x4a,2,2)] {
            let (mut tiles,mut shapes,g)=fixture(Kind::KantoBoundaryShore,8.);
            let mut assets=bevy::prelude::Assets::<bevy::prelude::Image>::default();
            let a=assets.add(bevy::prelude::Image::default());let b=assets.add(bevy::prelude::Image::default());
            tiles[2].texture=a.clone();tiles[2].animation_frames=Some(vec![a,b].into());
            for y in 0..2 {for x in 0..2 {
                let source=&mut tiles[y*3+x].source;
                source.metatile_id=block;source.subtile_column=c+x as u8;source.subtile_row=r+y as u8;
            }}
            // The other half of a mat-end block remains independently owned.
            let dry=&mut tiles[5].source;
            dry.metatile_id=1;dry.subtile_column=0;dry.subtile_row=0;dry.tile_index=0x23;
            shapes[5]=CellShape::Flat;
            let mut mesh=TerrainMeshData {footing_heights:vec![3.25;6],authored_cells:vec![None;6],..Default::default()};
            let mut claimed=vec![false;6];
            let datum=crystal_render_api::VisualTerraceDatum {origin:bevy::prelude::IVec2::ZERO,
                size:bevy::prelude::UVec2::new(3,2),levels:vec![[0.;4];6].into(),
                open_water:None,water_barriers:vec![bevy::prelude::IVec2::new(0,0),bevy::prelude::IVec2::new(1,0),
                    bevy::prelude::IVec2::new(0,1),bevy::prelude::IVec2::new(1,1)].into()};
            append_with_datum(&mut mesh,&tiles.iter().collect::<Vec<_>>(),&shapes,&g,&mut claimed,[0,0],None,Some(&datum));
            assert_eq!(claimed,[true,true,false,true,true,false]);
            assert_eq!(mesh.footing_heights,[3.25;6]);
            assert!(!mesh.solid.indices.is_empty());
            assert_eq!(mesh.textured.positions.len(),16);
            assert!(mesh.textured.positions.iter().all(|p|(p[1]-crate::profile::WATER_HEIGHT).abs()<0.001));
            let (u0,u1,v0,v1)=g.uv(2,0);
            assert!(mesh.textured.uvs.iter().all(|uv|
                uv[0]>=u0+(u1-u0)/16. && uv[0]<=u1-(u1-u0)/16.
                    && uv[1]>=v0+(v1-v0)/16. && uv[1]<=v1-(v1-v0)/16.));
            for i in [0,1,3,4] {assert_eq!(mesh.authored_cells[i],Some(Kind::KantoBoundaryShore.label()));}
            assert_eq!(mesh.authored_cells[5],None);
            let mut shifted_tiles=Vec::new();let mut shifted_shapes=Vec::new();
            for y in 0..2 {
                let mut water=tiles[2].clone();water.column=0;water.row=y as u32;
                shifted_tiles.push(water);shifted_shapes.push(CellShape::Water);
                for x in 0..3 {
                    let mut tile=tiles[y*3+x].clone();tile.column+=1;
                    shifted_tiles.push(tile);shifted_shapes.push(shapes[y*3+x]);
                }
            }
            let mut shifted_g=g;shifted_g.width+=1;shifted_g.origin_x-=shifted_g.tile_width;
            let mut shifted=TerrainMeshData {footing_heights:vec![3.25;8],authored_cells:vec![None;8],..Default::default()};
            append_with_datum(&mut shifted,&shifted_tiles.iter().collect::<Vec<_>>(),&shifted_shapes,
                &shifted_g,&mut vec![false;8],[-1,0],None,Some(&datum));
            assert_eq!(mesh.solid,shifted.solid,"a shifted view must retain the same wet backing and sculpture");
        }
    }

    #[test]
    fn source_boundary_caps_follow_live_paint_without_moving_geometry_or_water_contacts() {
        for kind in kinds() {for origin in [[0,0],[1,2],[4,4],[10,20],[30,40]] {
            let (tiles,shapes,g)=fixture(kind,8.);let cells=tiles.iter().collect::<Vec<_>>();
            let render=|images| {
                let mut mesh=TerrainMeshData {footing_heights:vec![3.;cells.len()],authored_cells:vec![None;cells.len()],..Default::default()};
                let mut claimed=vec![false;cells.len()];
                append_with_samples(&mut mesh,&cells,&shapes,&g,&mut claimed,origin,images);
                (mesh,claimed)
            };
            let (original,claims)=render(None);
            let mut images=TerrainImageSamples::default();
            let colors=[[20,20,20,255],[70,70,70,255],[140,140,140,255],[220,250,220,255]];
            images.pixels.insert(tiles[0].texture.id(),TileImageSample::Rgba(colors.repeat(16).into_iter().flatten().collect()));
            let (day,new_claims)=render(Some(&images));
            assert_eq!(new_claims,claims);assert_eq!(day.solid.positions,original.solid.positions);
            assert_eq!(day.solid.normals,original.solid.normals);assert_eq!(day.solid.indices,original.solid.indices);
            assert_eq!(day.footing_heights,original.footing_heights);
            assert_ne!(day.solid.colors,original.solid.colors);
            assert!(day.solid.colors.iter().any(|c|c[1]>c[0]*1.05),"muted native mineral highlight");
            if kind==Kind::KantoBoundaryShore {
                let mut pigments=std::collections::BTreeMap::new();
                for (p,c) in day.solid.positions.iter().zip(&day.solid.colors) {
                    if let Some(previous)=pigments.insert(p.map(f32::to_bits),*c) {
                        assert_eq!(previous,*c,"offshore pigment cannot expose triangular material seams");
                    }
                }
                assert!(day.solid.colors.iter().all(|c|c[1]<0.5),
                    "native bright ink must blend into mineral, not form a white post cap");
            }
            let high=day.solid.positions.iter().map(|p|p[1]).fold(f32::NEG_INFINITY,f32::max);
            let low=day.solid.positions.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min);
            let mut crown_faces=0;
            for tri in day.solid.indices.chunks_exact(3) {
                if kind!=Kind::KantoBoundaryShore && tri.iter().all(|&i|(day.solid.positions[i as usize][1]-low)/(high-low)>0.92) {
                    assert!(tri.iter().all(|&i|day.solid.colors[i as usize][1]>0.5),
                        "complete crown triangles cannot contain dark mineral wedges");
                    crown_faces+=1;
                }
            }
            if kind!=Kind::KantoBoundaryShore {assert!(crown_faces>5);}
            let canonical=model(kind);
            for tri in canonical.surface.indices.chunks_exact(3) {
                if kind!=Kind::KantoBoundaryShore && tri.iter().all(|&i|canonical.surface.positions[i as usize][1]
                    >canonical.min[1]+0.9*(canonical.max[1]-canonical.min[1])) {
                    assert!(tri.iter().all(|&i|day.solid.colors[i as usize][1]>0.5),
                        "all cached variants must paint the same complete canonical crown");
                }
            }


            assert!(day.solid.uvs.iter().all(|&uv|uv==if kind==Kind::KantoBoundaryShore {
                [super::super::shore_ecology::ROOTED_WATER_UV,85000.]
            }else{crate::scenery_finish::SOURCE_PAINT_UV}));
            let mut night_images=images.clone();
            for sample in night_images.pixels.values_mut() {if let TileImageSample::Rgba(p)=sample {
                for px in p.chunks_exact_mut(4) {px[0]/=2;px[1]/=2;}
            }}
            let (night,night_claims)=render(Some(&night_images));
            assert_eq!(night_claims,claims);assert_eq!(night.solid.positions,day.solid.positions);
            assert_eq!(night.solid.uvs,day.solid.uvs);assert_ne!(night.solid.colors,day.solid.colors);
        }}
    }

    #[test]
    fn boundary_asset_selection_survives_a_shifted_camera_crop() {
        for kind in kinds() {
            let (tiles, shapes, mut g) = fixture(kind, 8.0);
            let mut first = TerrainMeshData::default();
            append(&mut first, &tiles.iter().collect::<Vec<_>>(), &shapes,
                &g, &mut vec![false; tiles.len()], [10, 20]);
            let mut shifted = Vec::new();
            let mut shifted_shapes = Vec::new();
            for row in 0..2 {
                let mut ground = tiles[row * 3 + 2].clone();
                ground.column = 0;
                shifted.push(ground);
                shifted_shapes.push(shapes[row * 3 + 2]);
                for col in 0..3 {
                    let mut tile = tiles[row * 3 + col].clone();
                    tile.column += 1;
                    shifted.push(tile);
                    shifted_shapes.push(shapes[row * 3 + col]);
                }
            }
            g.width += 1;
            g.origin_x -= g.tile_width;
            let mut second = TerrainMeshData::default();
            append(&mut second, &shifted.iter().collect::<Vec<_>>(), &shifted_shapes,
                &g, &mut vec![false; shifted.len()], [9, 20]);
            assert!(!first.solid.indices.is_empty());
            assert_eq!(first.solid, second.solid,
                "camera cropping must retain the selected rock asset and placement");
        }
    }

    #[test]
    fn complete_boundary_drawings_append_closed_geometry_and_preserve_footing() {
        for kind in kinds() {
            let (tiles, shapes, g) = fixture(kind, 8.0);
            let cells: Vec<_> = tiles.iter().collect();
            let before = tiles.iter().map(|t| t.source.clone()).collect::<Vec<_>>();
            let mut mesh = TerrainMeshData {
                footing_heights: vec![3.25; 6],
                authored_cells: vec![None; 6],
                ..Default::default()
            };
            let mut claimed = vec![false; 6];
            // Exercise the integrated exterior entry point, not only the resolver.
            append_props(&mut mesh, &cells, &shapes, &g, &mut claimed);
            assert_eq!(claimed, [true, true, false, true, true, false]);
            assert_eq!(mesh.footing_heights, [3.25; 6]);
            assert_eq!(mesh.textured.indices.len(), 4 * 6);
            assert_eq!(mesh.solid.indices.len(), model(kind).surface.indices.len());
            assert_eq!(
                mesh.authored_cells
                    .iter()
                    .filter(|v| **v == Some(kind.label()))
                    .count(),
                4
            );
            assert_eq!(
                before,
                tiles.iter().map(|t| t.source.clone()).collect::<Vec<_>>()
            );
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .all(|v| v[0] >= -24.0 && v[0] <= -8.0 && v[2] >= 40.0 && v[2] <= 56.0)
            );
            let y = if kind == Kind::KantoBoundaryShore {
                crate::profile::WATER_HEIGHT
            } else {
                0.0
            };
            assert!(
                mesh.textured
                    .positions
                    .iter()
                    .all(|v| (v[1] - y).abs() < 0.001)
            );
            assert!(mesh.solid.positions.iter().all(|v| v[1] >= y - 0.001));
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .any(|v| (v[1] - y).abs() < 0.001)
            );
            assert!(mesh.solid.positions.iter().any(|v| v[1] > y + 10.0));
            // A second append cannot double draw or double claim this rock.
            let count = mesh.solid.indices.len();
            append_props(&mut mesh, &cells, &shapes, &g, &mut claimed);
            assert_eq!(mesh.solid.indices.len(), count);
        }
    }

    #[test]
    fn cropped_changed_phase_and_mixed_drawings_keep_fallback() {
        for kind in kinds() {
            for change in 0..5 {
                let (mut tiles, shapes, mut g) = fixture(kind, 8.0);
                match change {
                    0 => tiles[4].source.tile_index ^= 1,
                    1 => tiles[4].source.subtile_column -= 2,
                    2 => tiles[4].source.subtile_row += 2,
                    3 => {
                        tiles[4].source.metatile_id = if kind == Kind::KantoBoundaryShore {
                            0x14
                        } else {
                            0x13
                        }
                    }
                    4 => {
                        g.height = 1;
                        tiles.truncate(3);
                    }
                    _ => unreachable!(),
                }
                let cells = tiles.iter().collect::<Vec<_>>();
                assert!(
                    resolve(&cells, &shapes[..tiles.len()], &g).is_empty(),
                    "{kind:?} change {change}"
                );
            }
        }
    }

    #[test]
    fn only_native_matching_datum_underlays_reserve_source_cells() {
        for kind in kinds() {
            let (mut tiles, mut shapes, g) = fixture(kind, 8.0);
            for i in [2, 5] {
                tiles[i].source.tileset_id = Arc::from("johto");
            }
            assert!(resolve(&tiles.iter().collect::<Vec<_>>(), &shapes, &g).is_empty());
            for i in [2, 5] {
                tiles[i].source.tileset_id = Arc::from("kanto");
                shapes[i] = CellShape::RaisedTop {
                    height: 16.0,
                    solid: SolidKind::Bank,
                };
            }
            assert!(resolve(&tiles.iter().collect::<Vec<_>>(), &shapes, &g).is_empty());
            for i in [2, 5] {
                tiles[i].source.tile_index = 0xffff;
                shapes[i] = CellShape::Flat;
            }
            let cells = tiles.iter().collect::<Vec<_>>();
            assert!(resolve(&cells, &shapes, &g).is_empty());
            assert!(!preferred_cells("Route17", &cells, &g).iter().any(|v| *v));
        }
    }

    #[test]
    fn shoreline_water_and_solid_use_the_same_scaled_datum() {
        let (tiles, shapes, g) = fixture(Kind::KantoBoundaryShore, 16.0);
        let cells = tiles.iter().collect::<Vec<_>>();
        let p = resolve(&cells, &shapes, &g)[0];
        assert_eq!(p.base, -4.0);
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; 6],
            ..Default::default()
        };
        append(&mut mesh, &cells, &shapes, &g, &mut [false; 6], [0; 2]);
        assert!(mesh.textured.positions.iter().all(|v| v[1] == -4.0));
        assert!(mesh.solid.positions.iter().all(|v| v[1] >= -4.0));
        assert!(mesh.solid.positions.iter().any(|v| v[1] == -4.0));
        let original = mesh.clone();
        let edges = [crystal_render_api::VisualJumpLedge {
            start: bevy::prelude::IVec2::new(0,1), end: bevy::prelude::IVec2::new(3,1),
            descent: bevy::prelude::IVec2::Y,
        }];
        super::super::terraces::finish_on_map(&mut mesh,"CinnabarIsland",&cells,&g,[0,0],&edges);
        assert_eq!(mesh.solid.positions,original.solid.positions,
            "an island contour cannot lift sea rocks or add an offshore ledge");
        assert_eq!(mesh.textured.positions,original.textured.positions);
    }

    #[test]
    fn preferred_cells_and_append_agree_and_existing_claims_win() {
        for kind in kinds() {
            let (tiles, shapes, g) = fixture(kind, 8.0);
            let cells = tiles.iter().collect::<Vec<_>>();
            assert_eq!(
                preferred_cells("Route17", &cells, &g),
                [true, true, false, true, true, false]
            );
            let mut mesh = TerrainMeshData {
                authored_cells: vec![None; 6],
                ..Default::default()
            };
            let mut claimed = [false; 6];
            claimed[4] = true;
            append(&mut mesh, &cells, &shapes, &g, &mut claimed, [0; 2]);
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.textured.indices.is_empty());
            assert!(!mesh.authored_cells.iter().any(Option::is_some));
            assert_eq!(claimed, [false, false, false, false, true, false]);
        }
    }
}
