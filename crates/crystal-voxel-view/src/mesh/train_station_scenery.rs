//! Source-aware stationary body and separately posed plug/slide doors.
//! Native door drawings, rail backing and the gate/platform aisle survive.
use super::*;
#[path = "train_station_source.rs"]
mod source;
use crate::live_profiles::Document;
use source::Identity;
pub(super) struct Placement {
    resolved: source::Match,
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        self.resolved.indices(width)
    }
}
fn identity(s: &crystal_render_api::VisualTileSource) -> Identity<'_> {
    Identity {
        tileset: &s.tileset_id,
        metatile: s.metatile_id,
        column: s.subtile_column,
        row: s.subtile_row,
        tile: s.tile_index,
    }
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if !source::is_map(map)
        || !crate::train_station_models::train().has_native_train_doors()
        || g.width.checked_mul(g.height) != Some(cells.len())
        || cells.len() != reserved.len()
    {
        return Vec::new();
    }
    let mut blocked = reserved.to_vec();
    // No canonical override exists for this body. All custom objects,
    // including ones in either doorway or adjoining platform, win atomically.
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    source::resolve(map, &identities, g.width, g.height, origin, &blocked)
        .into_iter()
        .map(|resolved| Placement { resolved })
        .collect()
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placement: &Placement,
    claimed: &mut [bool],
) -> bool {
    let p = &placement.resolved;
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    if claimed.len() != cells.len()
        || !crate::train_station_models::train().has_native_train_materials()
        || !crate::train_station_models::train().has_native_train_doors()
        || !p.coherent(&identities, g.width, g.height)
        || claimed[p.ground]
        || p.guard_indices(g.width).any(|i| claimed[i])
    {
        return false;
    }
    let uv = g.uv(p.ground % g.width, p.ground / g.width);
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            uv,
        );
    }
    let (west, _, north, _) = g.bounds(p.column, p.row);
    let sample=|pixel:[usize;2]| {
        let uv=g.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
        [uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]
    };
    assert!(crate::train_station_models::train().append_native_train_body(
        &mut mesh.textured,
        [
            west,
            west + 20. * g.tile_width,
            north,
            north + 4. * g.tile_height,
        ],
        crate::train_station_models::HEIGHT_PIXELS * g.tile_height / SOURCE_TILE_HEIGHT,
        // Verified packed pigments: tile0a cream, tile1a gray/dark glass,
        // tile27 ink, and tile28's orange band at native pixel(80,22).
        [[28,1],[28,10],[28,10],[28,17],[80,22],[28,15]].map(sample),
    ));
    assert!(append_doors(&mut mesh.animated_textured,cells,g,p,1.));
    mesh.animated_train_doors=true;
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some("train:stationary-body");
        }
    }
    // The unclaimed door/rail/platform cells use the ordinary live source
    // pass. No footing height, collision, actor, warp or controller changes.
    true
}
// Source-authored officer path: (9,9) -> UP UP RIGHT -> (10,7).
// Arrival reverses it. Observe the published pose; never advance a timer or game state.
pub(super) fn door_openness(frame:&VisualWorldFrame)->f32 {
    if !source::is_map(&frame.map_id)||!frame.active||!frame.tile_size.is_finite()
        ||frame.tile_size.min_element()<=0. {return 1.;}
    let Some(actor)=frame.actors.iter().find(|a|
        a.id==crystal_render_api::VisualActorId::Object(0)&&a.source_id.as_ref()=="officer") else{return 1.;};
    let point=crate::footing::actor_ground_point(frame,actor);
    let northwest=frame.center+bevy::prelude::Vec2::new(
        -(frame.grid_size.x as f32)*frame.tile_size.x*0.5,
        frame.grid_size.y as f32*frame.tile_size.y*0.5);
    let x=(point.x-northwest.x)/frame.tile_size.x+frame.grid_origin.x as f32;
    let y=(northwest.y-point.y)/frame.tile_size.y+frame.grid_origin.y as f32;
    if !x.is_finite()||!y.is_finite() {return 1.;}
    let epsilon=0.025;
    let up=(x-19.).abs()<epsilon&&(15.-epsilon..=19.+epsilon).contains(&y);
    let right=(y-15.).abs()<epsilon&&(19.-epsilon..=21.+epsilon).contains(&x);
    // An unknown/custom driver keeps the reference open boarding pose.
    if !up&&!right {return 1.;}
    (((x-19.)+(19.-y))/6.).clamp(0.,1.)+0.
}
fn append_doors(out:&mut SurfaceMeshData,cells:&[&VisualTile],g:&GridGeometry,p:&source::Match,open:f32)->bool {
    let (west,_,north,_)=g.bounds(p.column,p.row);
    let swatches=[[28,1],[28,10],[28,10],[28,17],[80,22],[28,15]].map(|pixel| {
        let uv=g.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
        [uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
            uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]
    });
    let _=cells;
    crate::train_station_models::train().append_native_train_doors(out,
        [west,west+20.*g.tile_width,north,north+4.*g.tile_height],
        crate::train_station_models::HEIGHT_PIXELS*g.tile_height/SOURCE_TILE_HEIGHT,swatches,open)
}
pub(super) fn append_selected_doors(out:&mut SurfaceMeshData,frame:&VisualWorldFrame,
    cells:&[&VisualTile],g:&GridGeometry,open:f32)->bool {
    let ids=cells.iter().map(|t|identity(&t.source)).collect::<Vec<_>>();
    let matched=source::resolve(&frame.map_id,&ids,g.width,g.height,frame.grid_origin.to_array(),&vec![false;cells.len()]);
    matched.len()==1&&append_doors(out,cells,g,&matched[0],open)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn fixture() -> (Vec<VisualTile>, GridGeometry) {
        let mut tiles = (0..32 * 20)
            .map(|i| VisualTile {
                animation_frames: None,
                column: (i % 32) as u32,
                row: (i / 32) as u32,
                source: crystal_render_api::VisualTileSource {
                    tileset_id: Arc::from("train_station"),
                    metatile_id: 0x03,
                    subtile_column: (i % 4) as u8,
                    subtile_row: ((i / 32) % 4) as u8,
                    tile_index: 0x3d,
                },
                texture: Default::default(),
                priority: false,
            })
            .collect::<Vec<_>>();
        // Small source-identity regression fixture, never source pixels.
        let blocks = [0x10, 0x11, 0x15, 0x12, 0x13];
        let drawing = [
            [
                0x1f, 0x05, 0x06, 0x0a, 0x0a, 0x0a, 0x09, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x09,
                0x0a, 0x0a, 0x0a, 0x0b, 0x0c, 0x1f,
            ],
            [
                0x14, 0x15, 0x16, 0x1a, 0x1a, 0x1a, 0x19, 0x1a, 0x1a, 0x1a, 0x1a, 0x1a, 0x1a, 0x19,
                0x1a, 0x1a, 0x1a, 0x1b, 0x1c, 0x1d,
            ],
            [
                0x24, 0x25, 0x26, 0x27, 0x07, 0x2f, 0x29, 0x27, 0x28, 0x28, 0x28, 0x28, 0x2a, 0x29,
                0x07, 0x2f, 0x2a, 0x2b, 0x2c, 0x2d,
            ],
            [
                0x20, 0x1f, 0x2e, 0x1f, 0x17, 0x00, 0x2e, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x2e,
                0x17, 0x00, 0x1f, 0x2e, 0x1f, 0x0f,
            ],
        ];
        for y in 0..8 {
            for x in 0..20 {
                let source = &mut tiles[(y + 8) * 32 + x + 8].source;
                source.metatile_id = if y < 4 { blocks[x / 4] } else { 0x04 };
                source.tile_index = if y < 4 {
                    drawing[y][x]
                } else {
                    [0x32, 0x33, 0x3e, 0x3e][y - 4]
                };
            }
        }
        tiles[8 * 32].source.metatile_id = 0x14;
        tiles[8 * 32].source.tile_index = 0x1f;
        let g = GridGeometry {
            width: 32,
            height: 20,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        (tiles, g)
    }
    #[test]
    fn station_doors_observe_interpolated_officer_route_in_map_coordinates() {
        use bevy::prelude::{IVec2,UVec2,Vec2};
        use crystal_render_api::{VisualActor,VisualActorId};
        for map in ["GoldenrodMagnetTrainStation","SaffronMagnetTrainStation"] {
            for (origin,size,center,tile) in [
                (IVec2::ZERO,UVec2::new(40,36),Vec2::ZERO,Vec2::splat(8.)),
                (IVec2::new(8,10),UVec2::new(24,20),Vec2::new(120.,-70.),Vec2::new(12.,6.))] {
                let mut frame=VisualWorldFrame {active:true,map_id:map.into(),grid_origin:origin,
                    grid_size:size,center,tile_size:tile,..Default::default()};
                let northwest=center+Vec2::new(-(size.x as f32)*tile.x/2.,size.y as f32*tile.y/2.);
                for (x,y,expected) in [(19.,19.,0.),(19.,18.,1./6.),(19.,17.,1./3.),
                    (19.,15.,2./3.),(20.,15.,5./6.),(21.,15.,1.),(20.,15.,5./6.),
                    (19.,15.,2./3.),(19.,17.,1./3.),(19.,19.,0.)] {
                    let ground=northwest+Vec2::new((x-origin.x as f32)*tile.x,-(y-origin.y as f32)*tile.y);
                    frame.actors=vec![VisualActor {id:VisualActorId::Object(0),source_id:"officer".into(),
                        model_palette:None,texture:Default::default(),center:ground,
                        size:Vec2::new(16.,tile.y*2.),flip_x:false,above_priority:false,facing:Some(Vec2::X)}];
                    assert!((door_openness(&frame)-expected).abs()<0.0001,"{map}: {x},{y}");
                }
                frame.actors[0].id=VisualActorId::Player;
                assert_eq!(door_openness(&frame),1.);
                frame.actors[0].id=VisualActorId::Object(0);
                frame.actors[0].source_id="custom".into();
                assert_eq!(door_openness(&frame),1.);
                frame.actors[0].source_id="officer".into();
                frame.actors[0].center+=Vec2::X*tile.x*4.;
                assert_eq!(door_openness(&frame),1.);
                frame.actors.clear();assert_eq!(door_openness(&frame),1.);
            }
        }
    }
    #[test]
    fn selected_train_doors_update_without_tile_phases_or_static_terrain_rebuild() {
        use bevy::prelude::*;
        use crystal_render_api::{VisualActor,VisualActorId};
        use crate::{TerrainRevisionCache,TerrainCacheKey,VoxelViewStatus,VoxelMaterial};
        let (tiles,g)=fixture();
        let mut frame=VisualWorldFrame {active:true,map_id:"GoldenrodMagnetTrainStation".into(),
            grid_size:UVec2::new(g.width as u32,g.height as u32),tile_size:Vec2::splat(8.),
            tiles,..Default::default()};
        frame.actors=vec![VisualActor {id:VisualActorId::Object(0),source_id:"officer".into(),
            model_palette:None,texture:Default::default(),center:Vec2::new(24.,-72.),
            size:Vec2::splat(16.),flip_x:false,above_priority:false,facing:Some(Vec2::NEG_Y)}];
        let key=TerrainCacheKey::from_frame(&frame);
        let images=Assets::<Image>::default();
        let (open,solid)=super::super::build_animated_scene(&frame,&images,&[],Some(1.)).unwrap();
        let positions=|mesh:&Mesh|match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            bevy::mesh::VertexAttributeValues::Float32x3(values)=>values.clone(),_=>panic!("positions")};
        let before=positions(&open);
        let mut meshes=Assets::<Mesh>::default();
        let handle=meshes.add(open);let solid_handle=meshes.add(solid);
        let mut app=App::new();
        app.insert_resource(images).insert_resource(meshes)
            .insert_resource(Assets::<VoxelMaterial>::default()).insert_resource(frame.clone())
            .insert_resource(VoxelViewStatus {active:true,terrain_builds:9,..Default::default()})
            .insert_resource(TerrainRevisionCache {built_frame:Some(frame.clone()),
                built_train_doors:true,built_train_open:Some(1f32.to_bits()),
                animated_textured_mesh:Some(handle.clone()),animated_solid_mesh:Some(solid_handle.clone()),
                built_footing_heights:vec![2.5;g.width*g.height],key:Some(key.clone()),..Default::default()})
            .add_systems(Update,crate::terrain_animation::sync);
        app.update();
        let closed=positions(app.world().resource::<Assets<Mesh>>().get(&handle).unwrap());
        assert_ne!(before,closed);
        assert_eq!(app.world().resource::<TerrainRevisionCache>().built_train_open,Some(0f32.to_bits()));
        // Two north steps and one east step. No animation-phase resource is
        // installed: ordinary published object motion must drive the doors.
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].center+=Vec2::new(16.,32.);
        app.update();
        assert_eq!(positions(app.world().resource::<Assets<Mesh>>().get(&handle).unwrap()),before);
        let cache=app.world().resource::<TerrainRevisionCache>();
        assert_eq!(cache.animated_textured_mesh,Some(handle.clone()));
        assert_eq!(cache.animated_solid_mesh,Some(solid_handle));
        assert_eq!(cache.key,Some(key));assert_eq!(cache.built_frame.as_ref(),Some(&frame));
        assert!(cache.built_footing_heights.iter().all(|&h|h==2.5));
        assert_eq!(app.world().resource::<VoxelViewStatus>().terrain_builds,9);
        assert!(app.world().resource::<VoxelViewStatus>().active);
        // An unselected/custom train must not grow an independent door model.
        {
            let mut cache=app.world_mut().resource_mut::<TerrainRevisionCache>();
            cache.built_train_doors=false;cache.built_train_open=None;
            cache.animated_textured_mesh=None;
        }
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].center=Vec2::new(24.,-72.);
        app.update();
        assert!(app.world().resource::<TerrainRevisionCache>().animated_textured_mesh.is_none());
    }
    #[test]
    fn train_model_keeps_open_boarding_portals_below_continuous_roof() {
        let mut surface = SurfaceMeshData::default();
        crate::train_station_models::train().append_fitted(
            &mut surface,
            [0., 160., 0., 32.],
            0.,
            crate::train_station_models::HEIGHT_PIXELS,
        );
        assert!(!surface.indices.is_empty());
        let clearance=crate::train_station_models::BOARDING_CLEARANCE_PIXELS;
        for kind in [crate::new_bark_actors::character_meshes::CharacterKind::Trainer,
            crate::new_bark_actors::character_meshes::CharacterKind::TrainerFemale] {
            let player=crystal_render_api::VisualActor {id:crystal_render_api::VisualActorId::Player,
                source_id:"chris".into(),texture:Default::default(),center:bevy::prelude::Vec2::ZERO,size:bevy::prelude::Vec2::splat(16.),
                flip_x:false,above_priority:false,facing:None,model_palette:None};
            let player_height=crate::new_bark_actors::character_scale(&player,8.)*crate::npc_appearance::look(kind).height;
            assert!(clearance>=player_height+1.,"native boarding opening must clear the actual player plus margin");
        }
        for (left,right) in [(32.,48.),(112.,128.)] {
            assert!(surface.indices.chunks_exact(3).any(|tri| {
                let vs=tri.iter().map(|&i|surface.positions[i as usize]).collect::<Vec<_>>();
                vs.iter().all(|v|v[1]>=clearance-0.0001)
                    && vs.iter().any(|v|v[0]<=left)
                    && vs.iter().any(|v|v[0]>=right)
            }),"curved roof must continue across each boarding portal");
        }
        for tri in surface.indices.chunks_exact(3) {
            let vs = tri
                .iter()
                .map(|&i| surface.positions[i as usize])
                .collect::<Vec<_>>();
            for (left, right) in [(32., 48.), (112., 128.)] {
                assert!(
                    vs.iter().all(|v| v[0] <= left + 0.0001)
                        || vs.iter().all(|v| v[0] >= right - 0.0001)
                        || vs.iter().all(|v| v[2] <= 16.0001)
                        || vs.iter().all(|v| v[1] >= clearance-0.0001),
                    "solid covers a native train door"
                );
            }
        }
    }
    #[test]
    fn kanto_native_train_appends_body_once_and_preserves_live_doors_rail_and_footing() {
        let (tiles, g) = fixture();
        let cells = tiles.iter().collect::<Vec<_>>();
        let mut claimed = vec![false; cells.len()];
        let p = resolve(
            "GoldenrodMagnetTrainStation",
            &cells,
            &g,
            [0, 0],
            None,
            &claimed,
        );
        assert_eq!(p.len(), 1);
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; cells.len()];
        mesh.footing_heights = vec![2.5; cells.len()];
        assert!(append(&mut mesh, &cells, &g, &p[0], &mut claimed));
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 60);
        assert_eq!(
            mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
            60
        );
        assert!(mesh.animated_train_doors);
        assert!(!mesh.animated_textured.positions.is_empty());
        assert!(mesh.solid.positions.is_empty(),"native train uses its live source palette");
        assert!(mesh.textured.positions.len()>60*4);
        assert!(mesh.textured.positions[..60*4].iter().all(|v| v[1] == 0.));
        assert!(mesh.textured.positions[60*4..].iter().any(|v|v[1]>20.));
        let uv=g.uv(p[0].resolved.column+10,p[0].resolved.row+2);
        let orange=[uv.0+(uv.1-uv.0)*0.5/8.,uv.2+(uv.3-uv.2)*6.5/8.];
        assert!(mesh.textured.uvs[60*4..].contains(&orange),"native tile28 orange band, never a generic red stripe");
        assert!(mesh.footing_heights.iter().all(|&v| v == 2.5));
        assert!(mesh.solid.cutaway_ranges.is_empty());
        for y in 10..12 {
            for x in [12, 13, 22, 23] {
                assert!(!claimed[y * 32 + x]);
            }
        }
        assert!((12..16).all(|y| (8..28).all(|x| !claimed[y * 32 + x])));
        let before = mesh.solid.positions.len();
        assert!(!append(&mut mesh, &cells, &g, &p[0], &mut claimed));
        assert_eq!(mesh.solid.positions.len(), before);
    }
    #[test]
    fn every_custom_door_platform_or_body_override_rejects_whole_train() {
        let (tiles, g) = fixture();
        let cells = tiles.iter().collect::<Vec<_>>();
        let reserved = vec![false; cells.len()];
        for (block, origin, tile, ground) in [
            (0x10, [1, 0], 0x05, 0x1f),
            (0x11, [0, 2], 0x07, 0x1f),
            (0x04, [0, 2], 0x3e, 0x3d),
        ] {
            let custom:Document=serde_json::from_value(serde_json::json!({"objects":[{
                "name":"Custom station override","tileset":"train_station","map":"GoldenrodMagnetTrainStation",
                "metatile":block,"origin":origin,"tiles":[[tile]],"ground":ground,"top_pixels":1,"depth_pixels":3.0
            }],"atmosphere":null})).unwrap();
            assert!(
                !live::resolve(
                    &cells,
                    g.width,
                    g.height,
                    "GoldenrodMagnetTrainStation",
                    Some(&custom)
                )
                .is_empty()
            );
            assert!(
                resolve(
                    "GoldenrodMagnetTrainStation",
                    &cells,
                    &g,
                    [0, 0],
                    Some(&custom),
                    &reserved
                )
                .is_empty()
            );
        }
    }
    #[test]
    fn stale_or_custom_opening_rejects_before_any_mesh_mutation() {
        let (mut tiles, g) = fixture();
        let p = {
            let cells = tiles.iter().collect::<Vec<_>>();
            resolve(
                "SaffronMagnetTrainStation",
                &cells,
                &g,
                [0, 0],
                None,
                &vec![false; cells.len()],
            )
        };
        let door = 10 * 32 + 12;
        let mut claimed = vec![false; tiles.len()];
        claimed[door] = true;
        let mut mesh = TerrainMeshData::default();
        assert!(!append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            &p[0],
            &mut claimed
        ));
        claimed[door] = false;
        tiles[door].source.tile_index ^= 1;
        assert!(!append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            &p[0],
            &mut claimed
        ));
        assert!(mesh.textured.positions.is_empty() && mesh.solid.positions.is_empty());
    }
    #[test]
    fn unrelated_shared_atlas_or_incomplete_train_never_resolves() {
        let (mut tiles, g) = fixture();
        let reserved = vec![false; tiles.len()];
        assert!(
            resolve(
                "ViridianGym",
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                None,
                &reserved
            )
            .is_empty()
        );
        tiles[8 * 32 + 9].source.tile_index ^= 1;
        assert!(
            resolve(
                "GoldenrodMagnetTrainStation",
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                None,
                &reserved
            )
            .is_empty()
        );
    }
}
