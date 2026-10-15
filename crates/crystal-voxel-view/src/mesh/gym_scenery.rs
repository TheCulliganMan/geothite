//! Exact Gym drawings with original planter, broadleaf and connected-wall art.
use super::*;
#[path = "gym_scenery_source.rs"]
mod source;
use crate::live_profiles::Document;
use source::{Identity, Kind};
pub(super) struct Placement {
    resolved: source::Match,
}
impl Placement {
    /// The surface pass may finish only the actual successful model's native
    /// backing sample, after the same complete source drawing resolves again.
    pub(super) fn floor_sample_and_label(&self) -> (usize, &'static str) {
        (self.resolved.ground, self.resolved.kind.label())
    }
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
    if !matches!(
        map,
        "AzaleaGym" | "GoldenrodGym" | "CeladonGym" | "ViridianGym"
    ) || g.width.checked_mul(g.height) != Some(cells.len())
        || reserved.len() != cells.len()
    {
        return Vec::new();
    }
    let mut blocked = reserved.to_vec();
    // Gym scenery has no bundled profile override. Every resolving profile is
    // authoritative, including renamed/custom profiles with the same source art.
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let sources: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    source::resolve(map, &sources, g.width, g.height, origin, &blocked)
        .into_iter()
        .map(|resolved| Placement { resolved })
        .collect()
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
) -> bool {
    let p = &p.resolved;
    if claimed.len() != cells.len()
        || p.ground >= cells.len()
        || p.indices(g.width).any(|i| claimed[i])
    {
        return false;
    }
    let sources: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    if !p.coherent(&sources, g.width, g.height) {
        return false;
    }
    let ground_uv = g.uv(p.ground % g.width, p.ground / g.width);
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            ground_uv,
        );
    }
    let rise = p.kind.rise_pixels() * g.tile_height / SOURCE_TILE_HEIGHT;
    if p.kind == Kind::Maze {
        for y in 0..p.height {
            for x in 0..p.width {
                if !p.owns(x, y) {
                    continue;
                }
                let cutaway_start = mesh.solid.positions.len();
                let bounds = g.bounds(p.column + x, p.row + y);
                let triangle_start = mesh.solid.indices.len() / 3;
                let exposed = p.open_mask(x, y);
                crate::dungeon_models::gym_model(4 + usize::from(exposed)).append(
                    &mut mesh.solid,
                    bounds.into(),
                    0.,
                    rise,
                );
                finish_maze_cap(&mut mesh.solid,cutaway_start,bounds.into(),rise);
                let joins = crate::dungeon_models::gym_wall_join_triangles(
                    exposed, p.covered_join_mask(x, y),
                );
                if !joins.is_empty() {
                    mesh.reveal_join_batches.push(joins.iter()
                        .map(|&ordinal| triangle_start + ordinal).collect());
                }
                // Each owned wall cell is a coherent section. A single large
                // maze range would fade the entire maze for one blocked player.
                if mesh.solid.positions.len() > cutaway_start {
                    mesh.solid.cutaway_ranges.push(cutaway_start..mesh.solid.positions.len());
                }
            }
        }
    } else if p.kind == Kind::BlueFlowerBed {
        let (w, _, n, _) = g.bounds(p.column, p.row);
        let bounds = [w, w + p.width as f32 * g.tile_width,
            n, n + p.height as f32 * g.tile_height];
        blue_flowerbed::append(&mut mesh.textured, bounds,
            g.tile_height / SOURCE_TILE_HEIGHT, p.height, |[x,y]| {
                let uv = g.uv(p.column+x/8, p.row+y/8);
                [uv.0+(uv.1-uv.0)*((x%8) as f32+0.5)/8.,
                 uv.2+(uv.3-uv.2)*((y%8) as f32+0.5)/8.]
            });
    } else {
        let (w, _, n, _) = g.bounds(p.column, p.row);
        let e = w + p.width as f32 * g.tile_width;
        let s = n + p.height as f32 * g.tile_height;
        let model = if p.kind == Kind::PlanterRound {
            crate::dungeon_models::gym_flower_planter_base()
        } else {crate::dungeon_models::gym_model(p.kind.asset_index())};
        model.append(
            &mut mesh.solid,
            [w, e, n, s],
            0.,
            rise,
        );
        if p.kind == Kind::PlanterRound {
            append_planter_blossoms(&mut mesh.textured,[w,e,n,s],rise,
                g.uv(p.column,p.row));
        }
    }
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.kind.label());
        }
    }
    true
}
/// The authored coping already has beveled corners. Keep that closed geometry
/// and its shared joins, using a continuous stone finish instead of raised studs.
fn finish_maze_cap(mesh:&mut SurfaceMeshData,start:usize,b:[f32;4],rise:f32) {
    let scale=(b[1]-b[0]).min(b[3]-b[2])/8.;
    for i in start..mesh.positions.len() {
        let p=mesh.positions[i];
        if mesh.normals[i][1]<0.5||p[1]<rise-0.2*scale {continue;}
        let color=&mut mesh.colors[i];let mean=(color[0]+color[1]+color[2])/3.;
        let x=p[0]/scale;let z=p[2]/scale;
        let grain=0.96+0.025*((x*0.21+z*0.17).sin()+(x*0.071-z*0.113).sin());
        for c in &mut color[..3] {*c=(*c*0.86+mean*0.14)*grain;}
    }
}
#[path = "celadon_flowerbed.rs"]
mod blue_flowerbed;
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn kanto_maze_coping_keeps_authored_bevels_closed_joins_and_cached_models() {
        for mask in 0..16 {for scale in [0.5,1.,2.] {
            let model=crate::dungeon_models::gym_model(4+mask);
            let mut mesh=SurfaceMeshData::default();
            model.append(&mut mesh,[0.,8.*scale,0.,8.*scale],0.,12.*scale);
            let baseline=mesh.clone();
            finish_maze_cap(&mut mesh,0,[0.,8.*scale,0.,8.*scale],12.*scale);
            assert_eq!(mesh.indices,baseline.indices,"covered join ordinals stay authoritative");
            assert_eq!(mesh.positions,baseline.positions,"continuous coping adds no checkerboard studs or gaps");
            assert_eq!(mesh.normals,baseline.normals);assert_ne!(mesh.colors,baseline.colors);
            for tri in mesh.indices.chunks_exact(3) {
                let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let normal=Vec3::from_array(mesh.normals[tri[0]as usize]);
                assert!((b-a).cross(c-a).dot(normal)>0.,"authored faces retain outward winding");
            }
            let mut cached=SurfaceMeshData::default();model.append(&mut cached,[0.,8.*scale,0.,8.*scale],0.,12.*scale);
            assert_eq!(cached,baseline);
        }}
    }
    fn fixture() -> (Vec<VisualTile>, GridGeometry) {
        let tiles = (0..16)
            .map(|i| VisualTile {
                animation_frames: None,
                column: (i % 4) as u32,
                row: (i / 4) as u32,
                source: crystal_render_api::VisualTileSource {
                    tileset_id: Arc::from("elite_four_room"),
                    metatile_id: if i % 4 < 2 && i / 4 < 2 { 0x0f } else { 0x02 },
                    subtile_column: (i % 4) as u8,
                    subtile_row: (i / 4) as u8,
                    tile_index: if i % 4 < 2 && i / 4 < 2 {
                        [[0x07, 0x08], [0x17, 0x18]][i / 4][i % 4]
                    } else {
                        0x03
                    },
                },
                texture: Default::default(),
                priority: false,
            })
            .collect();
        let geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        (tiles, geometry)
    }
    #[test]
    fn complete_gym_planter_reserves_once_and_retains_native_ground() {
        let (tiles, g) = fixture();
        let cells = tiles.iter().collect::<Vec<_>>();
        let mut claimed = vec![false; 16];
        let placements = resolve("GoldenrodGym", &cells, &g, [0, 0], None, &claimed);
        assert_eq!(placements.len(), 1);
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; 16];
        assert!(append(&mut mesh, &cells, &g, &placements[0], &mut claimed));
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 4);
        assert_eq!(
            mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
            4
        );
        assert!(mesh.textured.positions.iter().all(|p|p[1]==0.));
        assert!(mesh.textured.cutaway_ranges.is_empty() && mesh.solid.cutaway_ranges.is_empty());
        assert!(mesh.solid.positions.iter().all(|p| p[0] >= 0.
            && p[0] <= 16.001
            && p[1] >= 0.
            && p[1] <= 14.001
            && p[2] >= 0.
            && p[2] <= 16.001));
        let length = mesh.solid.positions.len();
        assert!(!append(&mut mesh, &cells, &g, &placements[0], &mut claimed));
        assert_eq!(mesh.solid.positions.len(), length);
    }
    #[test]
    fn celadon_blue_flowerbed_requires_complete_source_and_custom_ownership() {
        let (original,mut g)=fixture();g.width=8;
        let art=[[0x50,0x51],[0x52,0x53],[0x52,0x53],[0x54,0x55]];
        let tiles:Vec<_>=(0..32).map(|i| {
            let x=i%8;let y=i/8;let mut t=original[0].clone();
            t.column=x as u32;t.row=y as u32;
            t.source=crystal_render_api::VisualTileSource {
                tileset_id:Arc::from("train_station"),metatile_id:if x<4 {0x20} else {0x19},
                subtile_column:(x%4) as u8,subtile_row:y as u8,
                tile_index:if x<4 {art[y][x%2]} else {0x57},
            };t
        }).collect();
        let refs=tiles.iter().collect::<Vec<_>>();let empty=vec![false;32];
        let beds=resolve("CeladonGym",&refs,&g,[0,0],None,&empty);
        assert_eq!(beds.len(),2);
        assert!(beds.iter().all(|p|p.resolved.kind==Kind::BlueFlowerBed));
        let mut mesh=TerrainMeshData {footing_heights:vec![3.;32],authored_cells:vec![None;32],..Default::default()};
        let mut claimed=empty.clone();
        for p in &beds {assert!(append(&mut mesh,&refs,&g,p,&mut claimed));}
        assert_eq!(claimed.iter().filter(|v|**v).count(),16);
        assert_eq!(mesh.footing_heights,vec![3.;32]);
        assert!(mesh.solid.positions.is_empty());
        assert!(mesh.textured.positions.iter().any(|p|p[1]>10.));
        assert!(mesh.authored_cells.iter().flatten().all(|s|*s=="gym:celadon-blue-flowerbed"));
        assert!(resolve("GoldenrodStation",&refs,&g,[0,0],None,&empty).is_empty());
        for index in beds[0].indices(g.width) {for mode in 0..6 {
            let mut bad=tiles.clone();let mut blocked=empty.clone();
            match mode {0=>bad[index].source.tile_index^=1,
                1=>bad[index].source.metatile_id^=1,
                2=>bad[index].source.subtile_column^=1,
                3=>bad[index].source.subtile_row^=1,
                4=>bad[index].source.tileset_id=Arc::from("custom"),
                _=>blocked[index]=true}
            let bad_refs=bad.iter().collect::<Vec<_>>();
            assert_eq!(resolve("CeladonGym",&bad_refs,&g,[0,0],None,&blocked).len(),1);
            if mode<5 {
                let mut target=TerrainMeshData::default();let mut claims=empty.clone();
                assert!(!append(&mut target,&bad_refs,&g,&beds[0],&mut claims));
                assert!(target.textured.positions.is_empty());
                assert_eq!(claims,empty);
            }
        }}
        let profiles:Document=serde_json::from_str(r#"{"objects":[{"name":"Custom blue bed","map":"CeladonGym","tileset":"train_station","metatile":32,"origin":[0,0],"tiles":[[80,81],[82,83],[82,83],[84,85]],"ground":87,"top_pixels":8,"depth_pixels":16}]}"#).unwrap();
        let custom=resolve("CeladonGym",&refs,&g,[0,0],Some(&profiles),&empty);
        assert_eq!(custom.len(),1);
        assert_eq!(custom[0].resolved.column,2);
    }
    #[test]
    fn custom_gym_profile_is_authoritative_and_changed_drawing_falls_back() {
        let (mut tiles, g) = fixture();
        let profiles:Document=serde_json::from_str(r#"{"objects":[{"name":"My custom plant","map":"GoldenrodGym","tileset":"elite_four_room","metatile":15,"origin":[0,0],"tiles":[[7,8],[23,24]],"ground":3,"top_pixels":8,"depth_pixels":9}]}"#).unwrap();
        assert!(
            resolve(
                "GoldenrodGym",
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                Some(&profiles),
                &[false; 16]
            )
            .is_empty()
        );
        tiles[5].source.tile_index = 0x99;
        assert!(
            resolve(
                "GoldenrodGym",
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                None,
                &[false; 16]
            )
            .is_empty()
        );
    }
    #[test]
    fn gym_planter_native_flower_drawing_receives_blossoms_and_preserves_support() {
        let (mut tiles,g)=fixture();
        for t in &mut tiles {t.source.metatile_id=2;t.source.tile_index=3;}
        for y in 0..2 {for x in 2..4 {
            let s=&mut tiles[y*4+x].source;s.metatile_id=5;
            s.tile_index=[[0x3e,0x3f],[0x17,0x18]][y][x-2];
        }}
        let refs=tiles.iter().collect::<Vec<_>>();let mut claimed=vec![false;16];
        let p=resolve("GoldenrodGym",&refs,&g,[0,0],None,&claimed);
        assert_eq!(p.len(),1);assert_eq!(p[0].resolved.kind,Kind::PlanterRound);
        let mut mesh=TerrainMeshData {footing_heights:vec![3.;16],..Default::default()};
        assert!(append(&mut mesh,&refs,&g,&p[0],&mut claimed));
        assert_eq!(mesh.footing_heights,vec![3.;16]);
        assert_eq!(claimed.iter().filter(|v|**v).count(),4);
        assert!(mesh.solid.positions.iter().all(|p|p[1]<9.6));
        assert!(mesh.textured.positions[16..].iter().all(|p|p[0]>16. && p[0]<32.
            && p[1]>0. && p[1]<=16. && p[2]>0. && p[2]<16.));
        assert_eq!(mesh.textured.indices.len()/3,8+288);
    }
    #[test]
    fn gym_planter_blossoms_are_closed_with_live_source_shades_and_bounded_height() {
        for scale in [0.5,1.,1.5] {
            let mut mesh = SurfaceMeshData::default();
            append_planter_blossoms(&mut mesh,[0.,16.*scale,0.,16.*scale],14.*scale,(0.,0.25,0.,0.25));
            assert_eq!(mesh.indices.len()/3,288);
            assert!(mesh.positions.iter().all(|p|p[0]>0. && p[0]<16.*scale
                && p[2]>0. && p[2]<16.*scale && p[1]>0. && p[1]<=14.*scale));
            assert!(mesh.normals.iter().all(|n|(Vec3::from_array(*n).length()-1.).abs()<0.0001));
            for [x,y] in [[3,4],[5,5],[4,4],[2,3]] {
                assert!(mesh.uvs.contains(&[0.25*(x as f32+0.5)/8.,0.25*(y as f32+0.5)/8.]));
            }
            for petal in mesh.positions.chunks_exact(24) {
                let mut edges = std::collections::BTreeMap::new();
                for tri in petal.chunks_exact(3) {
                    for i in 0..3 {
                        let mut edge = [tri[i].map(f32::to_bits),tri[(i+1)%3].map(f32::to_bits)];
                        edge.sort(); *edges.entry(edge).or_insert(0usize)+=1;
                    }
                }
                assert_eq!(edges.len(),12);
                assert!(edges.values().all(|n|*n==2),"petals must be closed volumes");
            }
        }
    }
    #[test]
    fn stale_gym_source_never_appends_partial_geometry() {
        let (mut tiles, g) = fixture();
        let placements = resolve(
            "GoldenrodGym",
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            None,
            &[false; 16],
        );
        assert_eq!(placements.len(), 1);
        tiles[5].source.tile_index = 0x99;
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; 16];
        assert!(!append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            &placements[0],
            &mut claimed
        ));
        assert!(mesh.solid.positions.is_empty() && mesh.textured.positions.is_empty());
        assert!(claimed.iter().all(|v| !*v));
    }
}

// Original closed floral assemblies accompany only complete3e/3f flower
// drawings. Leafy07/08 planters retain their original vegetation.
fn append_planter_blossoms(
    mesh: &mut SurfaceMeshData, bounds: [f32;4], rise: f32,
    uv: (f32,f32,f32,f32),
) {
    let [w,e,n,s] = bounds;
    let scale = (e-w).min(s-n);
    let (u0,u1,v0,v1) = uv;
    let sample = |[x,y]:[usize;2]| [u0+(u1-u0)*(x as f32+0.5)/8.,
        v0+(v1-v0)*(y as f32+0.5)/8.];
    let mut part = |points: [Vec3;6], stem: bool, pink: bool| {
        let center=points.into_iter().sum::<Vec3>()/6.;
        for ids in [[0,1,4],[1,2,4],[2,3,4],[3,0,4],
            [1,0,5],[2,1,5],[3,2,5],[0,3,5]] {
            let mut p = ids.map(|i|points[i]);
            let mut normal = (p[1]-p[0]).cross(p[2]-p[0]).normalize();
            if normal.dot((p[0]+p[1]+p[2])/3.-center)<0. {p.swap(1,2);normal=-normal;}
            let first = mesh.positions.len() as u32;
            mesh.positions.extend(p.map(|p|p.to_array()));
            mesh.normals.extend([normal.to_array();3]);
            let pixel = if stem {[2,3]} else if normal.y<0. {[3,4]}
                else if pink {[5,5]} else {[4,4]};
            mesh.uvs.extend([sample(pixel);3]);
            mesh.colors.extend([crate::interior_models::authored_face_color([1.;4],normal);3]);
            mesh.indices.extend([first,first+1,first+2]);
        }
    };
    for (plant,[x,z]) in [[0.29,0.28],[0.71,0.3],[0.27,0.51],
        [0.7,0.52],[0.3,0.74],[0.72,0.75]].into_iter().enumerate() {
        let head = Vec3::new(w+(e-w)*x,rise*(0.79+0.035*(plant%3) as f32),n+(s-n)*z);
        let root = Vec3::new(head.x,rise*0.4,head.z);
        let mid=(head+root)*0.5;
        part([mid+Vec3::X*scale*0.006,mid+Vec3::Z*scale*0.006,
            mid-Vec3::X*scale*0.006,mid-Vec3::Z*scale*0.006,head,root],true,false);
        for petal in 0..5 {
            let angle = petal as f32 * std::f32::consts::TAU / 5. + plant as f32 * 0.7;
            let along = Vec3::new(angle.cos(),0.,angle.sin());
            let across = Vec3::new(-angle.sin(),0.,angle.cos());
            let center = head + along * scale * 0.07;
            part([center+along*scale*0.085,center+across*scale*0.064,
                center-along*scale*0.085,center-across*scale*0.064,
                center+Vec3::Y*scale*0.045,center-Vec3::Y*scale*0.026],false,(plant+petal)%3==0);
        }
    }
}
