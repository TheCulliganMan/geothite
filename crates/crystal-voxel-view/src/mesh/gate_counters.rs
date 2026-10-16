//! Source-complete civic counters. Each connected drawing, including its phone,
//! is reserved atomically; gaps and surrounding source floor are never owned.
use super::*;
use crate::live_profiles::Document;

pub(super) const JOHTO_GATE_ROOMS: &[&str] = &[
    "IlexForestAzaleaGate",
    "Route29Route46Gate",
    "Route31VioletGate",
    "Route32RuinsOfAlphGate",
    "Route34IlexForestGate",
    "Route35GoldenrodGate",
    "Route35NationalParkGate",
    "Route36NationalParkGate",
    "Route36RuinsOfAlphGate",
    "Route38EcruteakGate",
    "Route40BattleTowerGate",
    "Route42EcruteakGate",
    "Route43Gate",
    "Route43MahoganyGate",
];

pub(super) fn reviewed_gate_map(map: &str) -> bool {
    JOHTO_GATE_ROOMS.contains(&map) || matches!(map,
        "Route15FuchsiaGate"|"Route16Gate"|"Route17Route18Gate"
        |"Route19FuchsiaGate"|"Route2Gate"|"Route5SaffronGate"|"Route6SaffronGate"
        |"Route5UndergroundPathEntrance"|"Route6UndergroundPathEntrance"
        |"Route7SaffronGate"|"Route8SaffronGate"|"SafariZoneFuchsiaGateBeta"|"VictoryRoadGate")
}

struct Network {
    maps: &'static [&'static str],
    // Verified original-map dimensions in source 8px cells, never canvas size.
    map_size: [usize; 2],
    width: usize,
    height: usize,
    fingerprint: u64,
    rows: &'static [u16],
    phones: &'static [(usize, usize)],
    // An intentional source-map edge may omit that side's guard halo.
    boundary: [bool; 4], // west, north, east, south
}
include!("gate_counter_bindings.rs");

pub(super) struct Placement {
    network: &'static Network,
    column: usize,
    row: usize,
    ground: usize,
    grid_origin: [i32; 2],
    native_palette_sources: Option<[usize;2]>,
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        self.network
            .rows
            .iter()
            .enumerate()
            .flat_map(move |(dy, bits)| {
                (0..self.network.width).filter_map(move |dx| {
                    (*bits & (1u16 << dx) != 0)
                        .then_some((self.row + dy) * width + self.column + dx)
                })
            })
    }
}
fn owns(n: &Network, x: isize, y: isize) -> bool {
    x >= 0
        && y >= 0
        && (x as usize) < n.width
        && (y as usize) < n.height
        && n.rows[y as usize] & (1u16 << x as usize) != 0
}
fn exposed(n: &Network, x: usize, y: usize) -> u8 {
    [(0, -1), (1, 0), (0, 1), (-1, 0)]
        .iter()
        .enumerate()
        .fold(0, |mask, (bit, &(dx, dy))| {
            mask | (u8::from(!owns(n, x as isize + dx, y as isize + dy)) << bit)
        })
}
fn identity_hash<'a>(sources: impl IntoIterator<Item = &'a VisualTileSource>) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for s in sources {
        for b in [
            (s.metatile_id & 255) as u8,
            (s.metatile_id >> 8) as u8,
            s.subtile_column,
            s.subtile_row,
            (s.tile_index & 255) as u8,
            (s.tile_index >> 8) as u8,
        ] {
            hash = (hash ^ u64::from(b)).wrapping_mul(0x100000001b3);
        }
    }
    hash
}
fn complete(
    cells: &[&VisualTile],
    g: &GridGeometry,
    grid_origin: [i32; 2],
    x: usize,
    y: usize,
    n: &Network,
) -> bool {
    if cells.len() != g.width * g.height || x + n.width > g.width || y + n.height > g.height {
        return false;
    }
    // Native canvases include padding and move as the viewport scrolls. The
    // missing halo of a source-edge drawing is justified only at that original
    // map edge, never at a viewport/canvas edge or a copied interior location.
    let west = i64::from(grid_origin[0]) + x as i64;
    let north = i64::from(grid_origin[1]) + y as i64;
    let east = west + n.width as i64;
    let south = north + n.height as i64;
    let [map_width, map_height] = n.map_size.map(|value| value as i64);
    if west < 0
        || north < 0
        || east > map_width
        || south > map_height
        || (n.boundary[0] && west != 0)
        || (n.boundary[1] && north != 0)
        || (n.boundary[2] && east != map_width)
        || (n.boundary[3] && south != map_height)
    {
        return false;
    }
    let sources = || {
        (0..n.height).flat_map(move |dy| {
            (0..n.width).map(move |dx| &cells[(y + dy) * g.width + x + dx].source)
        })
    };
    sources().all(|s| s.tileset_id.as_ref() == if n.maps.contains(&"CeladonCafe") {"game_corner"} else {"gate"}) && identity_hash(sources()) == n.fingerprint
}
fn valid_ground(map: &str, cell: &VisualTile) -> bool {
    cell.source.tileset_id.as_ref() == if map=="CeladonCafe" {"game_corner"} else {"gate"}
        && cell.source.tile_index == 0x01
        && matches!(
            shape_for_source_on_map(map, &cell.source),
            CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
        )
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    grid_origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    resolve_networks(NETWORKS, map, cells, g, grid_origin, profiles, reserved)
}
fn resolve_networks(
    networks: &'static [Network],
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    grid_origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if cells.len() != g.width * g.height
        || reserved.len() != cells.len()
        || !networks.iter().any(|n| n.maps.contains(&map))
    {
        return Vec::new();
    }
    let Some(ground) = cells.iter().position(|c| valid_ground(map, c)) else {
        return Vec::new();
    };
    let mut blocked = reserved.to_vec();
    // Respect every matching live profile, including new custom profile names.
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let mut out = Vec::new();
    for network in networks.iter().filter(|n| n.maps.contains(&map)) {
        if network.width > g.width || network.height > g.height {
            continue;
        }
        for row in 0..=g.height - network.height {
            for column in 0..=g.width - network.width {
                if !complete(cells, g, grid_origin, column, row, network) {
                    continue;
                }
                let p = Placement {
                    network,
                    column,
                    row,
                    ground,
                    grid_origin,
                    native_palette_sources: None,
                };
                let mut p=p;
                if reviewed_gate_map(map) {
                    let top=p.indices(g.width).find(|&i|cells[i].source.tile_index==0x07);
                    let side=p.indices(g.width).find(|&i|matches!(cells[i].source.tile_index,0x24|0x25));
                    let (Some(top),Some(side))=(top,side) else {continue;};
                    p.native_palette_sources=Some([top,side]);
                }
                if map=="CeladonCafe" {
                    let top=p.indices(g.width).find(|&i|cells[i].source.tile_index==0x0e);
                    let side=p.indices(g.width).find(|&i|cells[i].source.tile_index==0x1e);
                    let (Some(top),Some(side))=(top,side) else {continue;};
                    p.native_palette_sources=Some([top,side]);
                }
                if p.indices(g.width).any(|i| blocked[i]) {
                    continue;
                }
                for i in p.indices(g.width) {
                    blocked[i] = true;
                }
                out.push(p);
            }
        }
    }
    out
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
) -> bool {
    let cafe=p.network.maps.contains(&"CeladonCafe");
    if claimed.len() != cells.len()
        || p.ground >= cells.len()
        || !valid_ground(if cafe {"CeladonCafe"}else{""}, cells[p.ground])
        || !complete(cells, g, p.grid_origin, p.column, p.row, p.network)
        || p.indices(g.width).any(|i| claimed[i])
        || p.native_palette_sources.is_some_and(|[top,side]| {
            !p.indices(g.width).any(|i|i==top) || !p.indices(g.width).any(|i|i==side)
                || if cafe {cells[top].source.tile_index!=0x0e || cells[side].source.tile_index!=0x1e}
                   else {cells[top].source.tile_index!=0x07 || !matches!(cells[side].source.tile_index,0x24|0x25)}
        })
    {
        return false;
    }
    // Validate all palette roles before mutating ground or object surfaces.
    let native=p.native_palette_sources.map(|[top,side]| {
        let sample=|i:usize,x:usize,y:usize| {
            let uv=g.uv(i%g.width,i/g.width);
            [uv.0+(uv.1-uv.0)*(x as f32+0.5)/8.,
             uv.2+(uv.3-uv.2)*(y as f32+0.5)/8.]
        };
        if cafe {
            // Wood body and green top are independent live source palettes.
            return [sample(side,3,0),sample(side,3,2),sample(side,3,3),sample(side,3,2)];
        }
        let shadow_x=if cells[side].source.tile_index==0x24{1}else{6};
        [sample(side,shadow_x,3),sample(top,4,3),sample(top,4,1),sample(top,4,3)]
    });
    if native.is_some() && p.indices(g.width).any(|i| {
        !crate::gate_counter_models::segment(exposed(p.network,i%g.width-p.column,i/g.width-p.row))
            .has_native_gate_counter_materials()
    }) {return false;}
    let native_phone=native.is_some() && !cafe && !p.network.phones.is_empty();
    if native_phone && (!crate::gate_counter_models::phone().has_native_gate_phone_materials()
        || p.network.phones.iter().any(|&(dx,dy)| {
            [0x0e,0x0f,0x20,0x21].iter().enumerate().any(|(offset,&tile)| {
                let i=(p.row+dy+offset/2)*g.width+p.column+dx+offset%2;
                cells[i].source.tile_index!=tile
            })
        })) {return false;}
    let ground = cells[p.ground];
    let uv = g.uv(ground.column as usize, ground.row as usize);
    // The native civic drawing has one eight-pixel front course. A 12px
    // body hides the guard's torso; source plot depth is not extra height.
    // Only verified civic palette bindings use this fit. Legacy rooms keep
    // their existing dimensions and the cafe retains its own counter height.
    let height = if cafe {crate::cafe::SERVICE_COUNTER_HEIGHT}
        else if native.is_some() {8.0}else{12.0} * g.tile_height / 8.0;
    for i in p.indices(g.width) {
        let x = i % g.width;
        let y = i / g.width;
        let (west, east, north, south) = g.bounds(x, y);
        append_top(&mut mesh.textured, [west, east, north, south], 0.0, uv);
        let model=crate::gate_counter_models::segment(exposed(p.network, x-p.column, y-p.row));
        if let Some(swatches)=native {
            if cafe {
                let top=p.native_palette_sources.unwrap()[0];let uv=g.uv(top%g.width,top/g.width);
                let top_uv=[uv.0+(uv.1-uv.0)*3.5/8.,uv.2+(uv.3-uv.2)*3.5/8.];
                assert!(model.append_native_service_counter(&mut mesh.textured,[west,east,north,south],height,swatches,top_uv),
                    "native service worktop keeps green off vertical trim");
            } else {
                assert!(model.append_native_gate_counter(&mut mesh.textured,
                    [west,east,north,south],height,swatches),"native connected service counter requires complete material roles");
            }
        } else {
            model.append_fitted(&mut mesh.solid,[west,east,north,south],0.,height);
        }
    }
    for &(dx, dy) in p.network.phones {
        let (west, _, north, _) = g.bounds(p.column + dx, p.row + dy);
        // The receiver remains on the two-by-two source phone plot, on top of
        // the joined counter. Its handset, keys and coiled cord are solid art.
        let bounds=[west+g.tile_width*0.25,west+g.tile_width*1.75,
            north+g.tile_height*0.5,north+g.tile_height*1.5];
        let phone=crate::gate_counter_models::phone();
        let phone_height=12.0*g.tile_height/8.0;
        if native_phone {
            let uv=g.uv(p.column+dx,p.row+dy);
            let sample=|x:f32,y:f32|[uv.0+(uv.1-uv.0)*(x+0.5)/8.,uv.2+(uv.3-uv.2)*(y+0.5)/8.];
            assert!(phone.append_native_gate_phone(&mut mesh.textured,bounds,height,phone_height,
                [sample(2.,4.),sample(3.,0.),sample(0.,0.),sample(1.,0.)]));
        } else {
            phone.append_fitted(&mut mesh.solid,bounds,height,phone_height);
        }
    }
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            let local = (i % g.width - p.column, i / g.width - p.row);
            let phone =
                p.network.phones.iter().any(|&(x, y)| {
                    local.0 >= x && local.0 < x + 2 && local.1 >= y && local.1 < y + 2
                });
            mesh.authored_cells[i] = Some(if cafe {"cafe/joined_service_counter"} else if phone {
                "gate/phone_console"
            } else {
                "gate/joined_counter"
            });
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn fixture() -> (Vec<VisualTile>, GridGeometry, Network) {
        let mut cells = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                cells.push(VisualTile {
                    animation_frames: None,
                    column: x,
                    row: y,
                    source: VisualTileSource {
                        tileset_id: Arc::from("gate"),
                        metatile_id: 0x0b,
                        subtile_column: x as u8,
                        subtile_row: y as u8,
                        tile_index: if y == 2 {
                            7
                        } else if y == 3 {
                            if x % 2 == 0 { 0x24 } else { 0x25 }
                        } else {
                            1
                        },
                    },
                    texture: Default::default(),
                    priority: false,
                });
            }
        }
        let n = Network {
            maps: &["TestGate"],
            map_size: [4, 4],
            width: 4,
            height: 4,
            fingerprint: identity_hash(cells.iter().map(|t| &t.source)),
            rows: &[0, 0, 15, 15],
            phones: &[],
            boundary: [false; 4],
        };
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        (cells, g, n)
    }
    #[test]
    fn native_gate_phone_palette_uses_only_its_complete_plot_across_reviewed_rooms() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file(),"explicit external pack must exist");
        if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let layout=&pack.pack().runtime_files()["data/tilesets/gate_metatiles.bin"];
        let mut phones=0;
        for (map,module) in &pack.pack().data().maps {
            if !reviewed_gate_map(map) {continue;}
            let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
            let mut tiles=Vec::new();
            for y in 0..h {for x in 0..w {
                let block=module.blocks[y/4*(w/4)+x/4];
                tiles.push(VisualTile{animation_frames:None,column:x as u32,row:y as u32,texture:Default::default(),priority:false,
                    source:VisualTileSource{tileset_id:Arc::from("gate"),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+y%4*4+x%4]as u16}});
            }}
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let refs=tiles.iter().collect::<Vec<_>>();let blank=vec![false;tiles.len()];
            let expected=tiles.iter().filter(|t|t.source.tile_index==0x0e).count();
            let mut room_phones=0;
            for p in resolve(map,&refs,&g,[0,0],None,&blank) {
                if p.network.phones.is_empty(){continue;}
                let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
                let mut claims=blank.clone();assert!(append(&mut mesh,&refs,&g,&p,&mut claims),"{map}");
                assert!(mesh.solid.positions.is_empty(),"{map}: native phone avoids cyan solid paint");
                assert_eq!(claims.iter().filter(|&&v|v).count(),p.indices(w).count());
                assert!(mesh.footing_heights.iter().all(|&v|v==2.75));
                for &(dx,dy) in p.network.phones {
                    phones+=1;
                    room_phones+=1;
                    let uv=g.uv(p.column+dx,p.row+dy);
                    for (x,y) in [(2.,4.),(3.,0.),(0.,0.),(1.,0.)] {
                        let sample=[uv.0+(uv.1-uv.0)*(x+0.5)/8.,uv.2+(uv.3-uv.2)*(y+0.5)/8.];
                        assert!(mesh.textured.uvs.contains(&sample),"{map}: own phone swatch");
                    }
                }
            }
            assert_eq!(room_phones,expected,"{map}: every native phone drawing is bound");
        }
        assert!(phones>=20,"must exercise shared Kanto and Johto phone plots, got {phones}");
    }

    #[test]
    fn kanto_native_cafe_counter_reuses_connected_modules_and_native_palette() {
        let path=std::env::var("CRYSTAL_RENDER_TEST_PACK").unwrap_or_else(|_|"content-packs/realtime-clock.browser.crystalpack".into());
        if !std::path::Path::new(&path).exists(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let module=&pack.pack().data().maps["CeladonCafe"];
        let layout=pack.pack().runtime_files().get("data/tilesets/game_corner_metatiles.bin").unwrap();
        let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
        let mut tiles=Vec::new();
        for y in 0..h {for x in 0..w {
            let block=module.blocks[y/4*(w/4)+x/4];
            tiles.push(VisualTile{animation_frames:None,column:x as u32,row:y as u32,texture:Default::default(),priority:false,
                source:VisualTileSource{tileset_id:Arc::from("game_corner"),metatile_id:block,subtile_column:(x%4)as u8,
                    subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+y%4*4+x%4]as u16}});
        }}
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let refs=tiles.iter().collect::<Vec<_>>();let reserved=vec![false;tiles.len()];
        let ps=resolve("CeladonCafe",&refs,&g,[0,0],None,&reserved);
        assert_eq!(ps.len(),1);let p=&ps[0];assert_eq!((p.column,p.row),(16,0));
        assert_eq!(p.indices(w).count(),34);
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
        let mut claims=reserved.clone();assert!(append(&mut mesh,&refs,&g,p,&mut claims));
        assert!(mesh.solid.positions.is_empty());assert!(mesh.textured.positions.iter().all(|v|v[1]<=8.+1e-5));
        assert!(mesh.textured.positions.iter().any(|v|v[1]>7.));
        assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
        for y in 2..10 {for x in 18..24 {assert!(!claims[y*w+x],"service opening stays clear");}}
        assert!(p.indices(w).all(|i|mesh.authored_cells[i]==Some("cafe/joined_service_counter")));
        let mut blocked=reserved.clone();blocked[2*w+16]=true;
        assert!(resolve("CeladonCafe",&refs,&g,[0,0],None,&blocked).is_empty());
        assert!(resolve("OlivineCafe",&refs,&g,[0,0],None,&reserved).is_empty());
        drop(refs);tiles[7*w+16].source.tile_index=0xffff;
        assert!(resolve("CeladonCafe",&tiles.iter().collect::<Vec<_>>(),&g,[0,0],None,&reserved).is_empty());
    }

    #[test]
    fn kanto_native_gate_counter_palette_keeps_connected_geometry_and_rejects_bad_donors() {
        for mask in 0..16 {
            let model=crate::gate_counter_models::segment(mask);
            assert!(model.has_native_gate_counter_materials());
            let mut original=SurfaceMeshData::default();
            model.append_fitted(&mut original,[0.,8.,0.,8.],0.,12.);
            let mut native=SurfaceMeshData::default();
            assert!(model.append_native_gate_counter(&mut native,[0.,8.,0.,8.],12.,
                [[0.1,0.2],[0.3,0.4],[0.5,0.6],[0.7,0.8]]));
            assert_eq!(original.positions,native.positions);
            assert_eq!(original.normals,native.normals);
            assert_eq!(original.indices,native.indices);
            let mut repeat=SurfaceMeshData::default();
            model.append_fitted(&mut repeat,[0.,8.,0.,8.],0.,12.);
            assert_eq!(repeat.colors,original.colors,"cached model palette stays immutable");
        }
        let mut unknown=SurfaceMeshData::default();
        assert!(!crate::interior_models::model(crate::interior_models::ModelKind::Chair)
            .append_native_gate_counter(&mut unknown,[0.,8.,0.,8.],12.,[[0.1,0.2];4]));
        assert!(unknown.positions.is_empty());
        let (cells,g,mut n)=fixture();n.maps=Box::leak(JOHTO_GATE_ROOMS.iter().copied().chain([
            "Route5SaffronGate","Route5UndergroundPathEntrance","Route6UndergroundPathEntrance"
        ]).collect::<Vec<_>>().into_boxed_slice());
        let networks=Box::leak(vec![n].into_boxed_slice());
        let refs=cells.iter().collect::<Vec<_>>();
        for map in networks[0].maps {
            let native=resolve_networks(networks,map,&refs,&g,[0,0],None,
                &vec![false;cells.len()]).pop().unwrap();
            assert_eq!(native.native_palette_sources,Some([8,12]),
                "{map} must use its authenticated green enclosure, not generic wood");
        }
        let p=resolve_networks(networks,"Route5SaffronGate",&refs,&g,[0,0],None,
            &vec![false;cells.len()]).pop().unwrap();
        assert_eq!(p.native_palette_sources,Some([8,12]));
        let mut mesh=TerrainMeshData{footing_heights:vec![2.25;cells.len()],..Default::default()};
        let mut claims=vec![false;cells.len()];
        assert!(append(&mut mesh,&refs,&g,&p,&mut claims));
        assert!(mesh.solid.indices.is_empty());
        assert!(mesh.footing_heights.iter().all(|h|*h==2.25));
        assert_eq!(claims.iter().filter(|&&c|c).count(),8);
        assert_eq!(mesh.textured.positions.iter().map(|p|p[1]).fold(0.,f32::max),8.);
        let texel=|x:usize,y:usize,px:f32,py:f32| {
            let uv=g.uv(x,y);[uv.0+(uv.1-uv.0)*px/8.,uv.2+(uv.3-uv.2)*py/8.]
        };
        for uv in [texel(0,2,4.5,3.5),texel(0,2,4.5,1.5),texel(0,3,1.5,3.5)] {
            assert!(mesh.textured.uvs.contains(&uv));
        }
        let mut original=TerrainMeshData::default();
        let legacy=Placement{native_palette_sources:None,..p};
        assert!(append(&mut original,&refs,&g,&legacy,&mut vec![false;cells.len()]));
        assert_eq!(original.solid.positions.iter().map(|p|p[1]).fold(0.,f32::max),12.);
        // Floor precedes the palette model; conserve every original model triangle
        // and vertex count. Kanto bodies fit the native8px course, while the
        // legacy12px fit above remains available to unrelated rooms.
        assert_eq!(mesh.textured.indices.len(),original.textured.indices.len()+original.solid.indices.len());
        assert_eq!(mesh.textured.positions.len(),original.textured.positions.len()+original.solid.positions.len());
        let forged=Placement{native_palette_sources:Some([0,12]),..legacy};
        let mut empty=TerrainMeshData::default();let mut unclaimed=vec![false;cells.len()];
        assert!(!append(&mut empty,&refs,&g,&forged,&mut unclaimed));
        assert!(empty.textured.positions.is_empty() && empty.solid.positions.is_empty());
        assert!(unclaimed.iter().all(|c|!*c));
    }

    #[test]
    fn complete_network_rejects_each_identity_change_and_clipping() {
        let (cells, g, n) = fixture();
        assert!(complete(
            &cells.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            0,
            0,
            &n
        ));
        for i in 0..cells.len() {
            for field in 0..5 {
                let mut changed = cells.clone();
                let s = &mut changed[i].source;
                match field {
                    0 => s.tileset_id = Arc::from("mart"),
                    1 => s.metatile_id ^= 1,
                    2 => s.subtile_column ^= 1,
                    3 => s.subtile_row ^= 1,
                    _ => s.tile_index ^= 1,
                }
                assert!(!complete(
                    &changed.iter().collect::<Vec<_>>(),
                    &g,
                    [0, 0],
                    0,
                    0,
                    &n
                ));
            }
        }
        assert!(!complete(
            &cells.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            1,
            0,
            &n
        ));
        assert!(!complete(
            &cells.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            0,
            1,
            &n
        ));
    }
    #[test]
    fn sparse_network_keeps_every_opening_and_connected_edge() {
        for n in NETWORKS {
            assert_eq!(n.rows.len(), n.height);
            assert!(n.width <= 16);
            for y in 0..n.height {
                for x in 0..n.width {
                    if !owns(n, x as isize, y as isize) {
                        continue;
                    }
                    let mask = exposed(n, x, y);
                    if owns(n, x as isize + 1, y as isize) {
                        assert_eq!(mask & 2, 0);
                        assert_eq!(exposed(n, x + 1, y) & 8, 0);
                    }
                    if owns(n, x as isize, y as isize + 1) {
                        assert_eq!(mask & 4, 0);
                        assert_eq!(exposed(n, x, y + 1) & 1, 0);
                    }
                }
            }
            for &(x, y) in n.phones {
                for dy in 0..2 {
                    for dx in 0..2 {
                        assert!(owns(n, (x + dx) as isize, (y + dy) as isize));
                    }
                }
            }
        }
    }
    #[test]
    fn source_bound_gate_layouts_retain_operator_bays_and_through_openings() {
        // Exact inspected base-map anchors. These checks bind the sparse masks
        // to real native openings, not merely to generic rectangular fixtures.
        let claims = |map: &str, width: usize, height: usize, anchors: &[(u64, usize, usize)]| {
            let mut result = vec![false; width * height];
            for &(hash, column, row) in anchors {
                let network = NETWORKS
                    .iter()
                    .find(|n| n.maps.contains(&map) && n.fingerprint == hash)
                    .unwrap();
                let p = Placement {
                    network,
                    column,
                    row,
                    ground: 0,
                    grid_origin: [0, 0],
                    native_palette_sources: None,
                };
                for i in p.indices(width) {
                    assert!(!result[i]);
                    result[i] = true;
                }
            }
            result
        };
        let vertical = claims(
            "Route29Route46Gate",
            20,
            16,
            &[(0x714f11acacae1eaf, 0, 3), (0xb957fb59790da59b, 15, 3)],
        );
        for y in 0..16 {
            for x in 4..16 {
                assert!(!vertical[y * 20 + x]);
            }
        }
        // The source operator stands in the open left desk bay.
        for y in 6..10 {
            for x in 0..2 {
                assert!(!vertical[y * 20 + x]);
            }
        }
        let horizontal = claims(
            "IlexForestAzaleaGate",
            20,
            16,
            &[(0xfdfa57adbd7d4b12, 3, 0), (0x0614362cd79c13f5, 3, 11)],
        );
        for y in 8..12 {
            for x in 0..20 {
                assert!(!horizontal[y * 20 + x]);
            }
        }
        for y in 1..6 {
            for x in 6..14 {
                assert!(!horizontal[y * 20 + x]);
            }
        }
        let victory = claims(
            "VictoryRoadGate",
            40,
            36,
            &[(0x73d18a5502d34a65, 15, 19), (0x8a0aec8840735e8a, 21, 21)],
        );
        // Distinct central and east counters must not bridge this service gap.
        for y in 22..24 {
            assert!(!victory[y * 40 + 20]);
            assert!(!victory[y * 40 + 21]);
        }
    }
    #[test]
    fn append_overlap_is_atomic_and_preserves_footing() {
        let (cells, g, n) = fixture();
        let n = Box::leak(Box::new(n));
        let p = Placement {
            network: n,
            column: 0,
            row: 0,
            ground: 0,
            grid_origin: [0, 0],
            native_palette_sources: None,
        };
        let refs = cells.iter().collect::<Vec<_>>();
        let mut claims = vec![false; cells.len()];
        claims[8] = true;
        let original = claims.clone();
        let mut mesh = TerrainMeshData::default();
        assert!(!append(&mut mesh, &refs, &g, &p, &mut claims));
        assert_eq!(claims, original);
        assert!(mesh.solid.positions.is_empty());
        assert!(mesh.textured.positions.is_empty());
        claims[8] = false;
        mesh.authored_cells = vec![None; cells.len()];
        mesh.footing_heights = vec![3.25; cells.len()];
        assert!(append(&mut mesh, &refs, &g, &p, &mut claims));
        assert_eq!(claims.iter().filter(|&&b| b).count(), 8);
        assert_eq!(mesh.footing_heights, vec![3.25; cells.len()]);
        assert!(mesh.authored_cells[..8].iter().all(Option::is_none));
        assert_eq!(mesh.textured.positions.len(), 8 * 4);
        let mut expected = SurfaceMeshData::default();
        append_top(&mut expected, [0.0, 8.0, 0.0, 8.0], 0.0, g.uv(0, 0));
        for quad in mesh.textured.uvs.chunks_exact(4) {
            assert_eq!(quad, expected.uvs.as_slice());
        }
        assert!(
            mesh.solid
                .normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length_squared() - 1.0).abs() < 0.0001)
        );
    }
    #[test]
    fn reservation_and_custom_live_profile_reject_the_entire_network() {
        let (cells, g, n) = fixture();
        let networks = Box::leak(vec![n].into_boxed_slice());
        let refs = cells.iter().collect::<Vec<_>>();
        let mut reserved = vec![false; cells.len()];
        assert_eq!(
            resolve_networks(networks, "TestGate", &refs, &g, [0, 0], None, &reserved).len(),
            1
        );
        for index in 8..16 {
            reserved[index] = true;
            assert!(
                resolve_networks(networks, "TestGate", &refs, &g, [0, 0], None, &reserved)
                    .is_empty()
            );
            reserved[index] = false;
        }
        let document: Document=serde_json::from_value(serde_json::json!({
            "objects":[{"name":"User's independent counter design","tileset":"gate","map":"TestGate",
                "metatile":11,"origin":[0,2],"tiles":[[7]],"ground":1,"top_pixels":1,"depth_pixels":3.0}],
            "atmosphere":null
        })).unwrap();
        assert!(
            resolve_networks(
                networks,
                "TestGate",
                &refs,
                &g,
                [0, 0],
                Some(&document),
                &reserved
            )
            .is_empty()
        );
        // A custom object owning only the guard floor must remain independent.
        let mut floor_profile = document.clone();
        floor_profile.objects[0].origin = [0, 0];
        floor_profile.objects[0].tiles = vec![vec![1]];
        assert_eq!(
            resolve_networks(
                networks,
                "TestGate",
                &refs,
                &g,
                [0, 0],
                Some(&floor_profile),
                &reserved
            )
            .len(),
            1
        );
        let mut missing = cells.clone();
        for t in &mut missing {
            if t.source.tile_index == 1 {
                t.source.tile_index = 0;
            }
        }
        assert!(
            resolve_networks(
                networks,
                "TestGate",
                &missing.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                None,
                &reserved
            )
            .is_empty()
        );
    }
    #[test]
    fn missing_or_foreign_floor_and_wrong_map_are_rejected() {
        let (mut cells, g, _) = fixture();
        let reserved = vec![false; cells.len()];
        assert!(
            resolve(
                "TestGate",
                &cells.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                None,
                &reserved
            )
            .is_empty()
        );
        for c in &mut cells {
            if c.source.tile_index == 1 {
                c.source.tileset_id = Arc::from("mart");
            }
        }
        assert!(!cells.iter().any(|c| valid_ground("TestGate", c)));
    }

    include!("gate_counter_frame_tests.rs");
}
