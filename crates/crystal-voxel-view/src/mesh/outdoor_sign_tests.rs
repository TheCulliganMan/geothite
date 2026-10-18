use super::*;
use std::sync::Arc;

fn fixture(b: &'static Binding) -> (Vec<VisualTile>, GridGeometry) {
    let mut cells = Vec::new();
    for y in 0..4 {
        for x in 0..4 {
            let i = (y * 4 + x) as usize;
            let source = if (1..=2).contains(&x) && (1..=2).contains(&y) {
                let local = ((y - 1) * 2 + x - 1) as usize;
                VisualTileSource {
                    tileset_id: Arc::from(b.tileset),
                    metatile_id: b.block,
                    subtile_column: b.origin[0] + (local % 2) as u8,
                    subtile_row: b.origin[1] + (local / 2) as u8,
                    tile_index: b.art[local],
                }
            } else {
                VisualTileSource {
                    tileset_id: Arc::from(b.tileset),
                    metatile_id: b.block,
                    subtile_column: 2,
                    subtile_row: 0,
                    tile_index: b.backing[i % 4],
                }
            };
            cells.push(VisualTile {
                animation_frames: None,
                column: x,
                row: y,
                source,
                texture: Handle::default(),
                priority: false,
            });
        }
    }
    (
        cells,
        GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -16.,
            origin_z: -16.,
        },
    )
}
fn placements(
    cells: &[VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
) -> Vec<Placement> {
    resolve(
        "UnlistedNativeMap",
        &cells.iter().collect::<Vec<_>>(),
        g,
        profiles,
        &vec![false; cells.len()],
    )
}
#[test]
fn complete_native_patterns_select_their_shared_frames() {
    assert_eq!(BINDINGS.len(), 13);
    for b in BINDINGS {
        let (cells, g) = fixture(b);
        let p = placements(&cells, &g, None);
        assert_eq!(p.len(), 1, "{b:?}");
        assert_eq!(p[0].binding.kind, b.kind);
        assert_eq!(p[0].indices(g.width).collect::<Vec<_>>(), [5, 6, 9, 10]);
    }
    assert_eq!(BINDINGS.iter().filter(|b| b.tileset == "johto" && b.kind==Kind::Modern).count(),3);
}
#[test]
fn every_changed_identity_and_single_owned_cell_rejects_the_whole_sign() {
    for b in BINDINGS {
        let (base, g) = fixture(b);
        for index in [5, 6, 9, 10] {
            for field in 0..5 {
                let mut cells = base.clone();
                let s = &mut cells[index].source;
                match field {
                    0 => s.tileset_id = Arc::from("different_atlas"),
                    1 => s.metatile_id ^= 1,
                    2 => s.subtile_column ^= 1,
                    3 => s.subtile_row ^= 1,
                    _ => s.tile_index ^= 1,
                }
                assert!(
                    placements(&cells, &g, None).is_empty(),
                    "{b:?}, {index}, {field}"
                );
            }
            let mut reserved = vec![false; base.len()];
            reserved[index] = true;
            assert!(resolve("", &base.iter().collect::<Vec<_>>(), &g, None, &reserved).is_empty());
        }
    }
}
#[test]
fn ground_is_same_atlas_same_palette_block_and_preserves_native_pattern() {
    for b in BINDINGS {
        let (cells, g) = fixture(b);
        let p = placements(&cells, &g, None).remove(0);
        for (i, sample) in p.ground.iter().enumerate() {
            assert_eq!(cells[*sample].source.tile_index, b.backing[i]);
        }
        for mutation in 0..3 {
            let mut changed = cells.clone();
            for (i, cell) in changed.iter_mut().enumerate() {
                if [5, 6, 9, 10].contains(&i) {
                    continue;
                }
                match mutation {
                    0 => cell.source.tileset_id = Arc::from("foreign"),
                    1 => cell.source.metatile_id = 0xffff,
                    _ => cell.source.tile_index = 0xffff,
                }
            }
            assert!(placements(&changed, &g, None).is_empty());
        }
    }
    assert_eq!(BINDINGS.iter().find(|b|b.tileset=="kanto" && b.block==0x79).unwrap().backing,
        [0x30, 0x39, 0x39, 0x30]);
    assert_eq!(BINDINGS.iter().find(|b|b.tileset=="park" && b.block==0x15).unwrap().backing,
        [0x00, 0x16, 0x06, 0x00]);
}
#[test]
fn custom_profiles_protect_single_cells_and_incomplete_cropped_drawings() {
    let b = BINDINGS.iter().find(|b|b.tileset=="kanto" && b.block==0x08).unwrap();
    let (cells, g) = fixture(b);
    for tiles in ["[[70]]", "[[70,999],[998,997]]"] {
        let json = format!(
            r#"{{"objects":[{{"name":"Custom sign artwork","tileset":"kanto","metatile":8,"origin":[2,2],"tiles":{tiles},"ground":999,"top_pixels":0,"depth_pixels":2}}]}}"#
        );
        let d: Document = serde_json::from_str(&json).unwrap();
        assert!(placements(&cells, &g, Some(&d)).is_empty());
    }
    let shipped: Document = serde_json::from_slice(include_bytes!(
        "../../../../modpacks/voxel-view/profiles.json"
    ))
    .unwrap();
    assert!(
        resolve(
            "CinnabarIsland",
            &cells.iter().collect::<Vec<_>>(),
            &g,
            Some(&shipped),
            &vec![false; cells.len()]
        )
        .is_empty()
    );
    assert_eq!(
        resolve(
            "CeladonCity",
            &cells.iter().collect::<Vec<_>>(),
            &g,
            Some(&shipped),
            &vec![false; cells.len()]
        )
        .len(),
        1
    );
}
#[test]
fn live_text_footprint_footing_and_atomic_append_are_preserved() {
    for b in BINDINGS {
        let (cells, mut g) = fixture(b);
        g.origin_x = -47.;
        g.origin_z = 81.;
        g.tile_width = 7.;
        g.tile_height = 11.;
        let p = placements(&cells, &g, None).remove(0);
        let refs = cells.iter().collect::<Vec<_>>();
        let before = cells.iter().map(|c| c.source.clone()).collect::<Vec<_>>();
        let mut mesh = TerrainMeshData {
            footing_heights: vec![3.25; cells.len()],
            authored_cells: vec![None; cells.len()],
            ..Default::default()
        };
        let mut claimed = vec![false; cells.len()];
        assert!(append(&mut mesh, &refs, &g, &p, &mut claimed));
        assert_eq!(mesh.footing_heights, vec![3.25; cells.len()]);
        assert_eq!(
            claimed
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| v.then_some(i))
                .collect::<Vec<_>>(),
            [5, 6, 9, 10]
        );
        assert_eq!(
            cells.iter().map(|c| c.source.clone()).collect::<Vec<_>>(),
            before
        );
        assert_eq!(mesh.textured.indices.len(), 30); // Four native floor cells and one live inscription.
        assert!(mesh.textured.positions[..16].iter().all(|v| v[1] == 0.));
        assert!(mesh.textured.uvs[..16].iter().all(|&uv|
            crate::scenery_finish::source_atlas_coordinates(uv).is_none()),
            "floor backing keeps its existing lighting; only the live inscription is tagged");
        let (w, _, n, _) = g.bounds(1, 1);
        let e = w + 2. * g.tile_width;
        let s = n + 2. * g.tile_height;
        assert!(!mesh.solid.positions.is_empty());
        assert!(
            mesh.solid
                .positions
                .iter()
                .chain(mesh.textured.positions[16..].iter())
                .all(|v| v[0] >= w
                    && v[0] <= e
                    && v[2] >= n
                    && v[2] <= s
                    && v[1] >= 0.
                    && v[1] <= 2. * g.tile_height)
        );
        let [px, py, pw, ph] = b.lettering.map(f32::from);
        assert_eq!(
            mesh.textured.uvs[16..].iter().map(|&uv|
                crate::scenery_finish::source_atlas_coordinates(uv).unwrap_or(uv))
                .collect::<Vec<_>>(),
            [
                [(1. + px / 8.) / 4., (1. + py / 8.) / 4.],
                [(1. + px / 8.) / 4., (1. + (py + ph) / 8.) / 4.],
                [(1. + (px + pw) / 8.) / 4., (1. + (py + ph) / 8.) / 4.],
                [(1. + (px + pw) / 8.) / 4., (1. + py / 8.) / 4.]
            ]
        );
        assert!(mesh.textured.uvs[16..].iter().all(|&uv|
            crate::scenery_finish::source_atlas_coordinates(uv).is_some()==(b.tileset!="forest")),
            "preserve the forest sign's existing treatment");
        let vertex_count = mesh.solid.positions.len();
        assert!(!append(&mut mesh, &refs, &g, &p, &mut claimed));
        assert_eq!(mesh.solid.positions.len(), vertex_count);
        assert!(
            p.indices(g.width)
                .all(|i| mesh.authored_cells[i] == Some(b.kind.label()))
        );
    }
}
#[test]
fn cropped_or_revised_frames_fall_back_without_partial_claims() {
    for b in BINDINGS {
        let (mut cells, mut g) = fixture(b);
        let p = placements(&cells, &g, None).remove(0);
        cells[10].source.tile_index = 0xffff;
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; 16];
        assert!(!append(
            &mut mesh,
            &cells.iter().collect::<Vec<_>>(),
            &g,
            &p,
            &mut claimed
        ));
        assert!(mesh.solid.positions.is_empty() && mesh.textured.positions.is_empty());
        assert!(claimed.iter().all(|v| !v));
        g.width = 1;
        g.height = 16;
        assert!(placements(&cells, &g, None).is_empty());
    }
}

#[test]
fn elevated_pier_notice_and_its_floor_follow_the_deck_without_changing_ownership() {
    let binding=BINDINGS.iter().find(|b|b.tileset=="kanto"&&b.block==0x79).unwrap();
    let (cells,g)=fixture(binding);let refs=cells.iter().collect::<Vec<_>>();
    let p=placements(&cells,&g,None).remove(0);
    let build=|height| {
        let mut mesh=TerrainMeshData{footing_heights:vec![height;cells.len()],authored_cells:vec![None;cells.len()],..Default::default()};
        let mut claimed=vec![false;cells.len()];assert!(append(&mut mesh,&refs,&g,&p,&mut claimed));
        (mesh,claimed)
    };
    let (flat,flat_claims)=build(0.);let (raised,raised_claims)=build(crate::port::PIER_HEIGHT);
    assert_eq!(flat_claims,raised_claims);assert_eq!(flat.authored_cells,raised.authored_cells);
    assert_eq!(raised.footing_heights,vec![crate::port::PIER_HEIGHT;cells.len()]);
    for (old,new) in [(&flat.solid,&raised.solid),(&flat.textured,&raised.textured)] {
        assert_eq!(old.indices,new.indices);assert_eq!(old.normals,new.normals);assert_eq!(old.uvs,new.uvs);
        for (a,b) in old.positions.iter().zip(&new.positions) {
            assert_eq!([a[0],a[2]],[b[0],b[2]]);assert!((b[1]-a[1]-crate::port::PIER_HEIGHT).abs()<0.0001);
        }
    }
}

#[test]
fn sign_clearance_preserves_roots_coverage_models_and_unowned_or_forest_grass() {
    for binding in BINDINGS {
        let (cells,g)=fixture(binding);let signs=placements(&cells,&g,None);
        let refs=cells.iter().collect::<Vec<_>>();
        let mut mesh=TerrainMeshData{footing_heights:vec![3.25;16],
            authored_cells:vec![None;16],..Default::default()};
        append(&mut mesh,&refs,&g,&signs[0],&mut vec![false;16]);
        let first=mesh.solid.positions.len();
        crate::grass_blades::append(&mut mesh.solid,[-8.,0.,8.,16.],0.,24.,[1,3],true);
        crate::grass_blades::append(&mut mesh.solid,[100.,108.,100.,108.],0.,24.,[15,15],false);
        let original=mesh.clone();
        let mut unowned=mesh.clone();unowned.authored_cells.fill(None);
        let before=unowned.clone();
        clear_grass(&mut unowned,first,0.,[-8.,108.,8.,108.],&g,&signs);
        assert_eq!(unowned,before,"unappended/custom signs cannot trim grass");
        clear_grass(&mut mesh,first,0.,[-8.,108.,8.,108.],&g,&signs);
        if binding.kind==Kind::Forest {assert_eq!(mesh,original);continue;}
        assert_eq!(&mesh.solid.positions[..first],&original.solid.positions[..first],"sign geometry");
        assert_eq!(mesh.textured,original.textured,"live lettering and ground");
        assert_eq!(mesh.footing_heights,original.footing_heights);
        assert_eq!(mesh.authored_cells,original.authored_cells);
        assert_eq!(mesh.solid.indices,original.solid.indices,"same blade coverage");
        assert_eq!(mesh.solid.colors,original.solid.colors);
        assert!(mesh.solid.positions[first..].iter().zip(&original.solid.positions[first..])
            .any(|(a,b)|a[1]<b[1]-1.),"nearby blades expose the board");
        for i in first..mesh.solid.positions.len() {
            let a=mesh.solid.positions[i];let b=original.solid.positions[i];
            assert_eq!([a[0],a[2]],[b[0],b[2]],"roots and occupied footprint");
            assert!(a[1]>=0. && a[1]<=b[1]);
            assert!((mesh.solid.uvs[i][1]-a[1]).abs()<0.0001,"wind height follows the shorter blade");
            assert!((Vec3::from_array(mesh.solid.normals[i]).length()-1.).abs()<0.0001);
            if b[1]==0. || b[0]>90. {assert_eq!(a,b,"roots and distant grass stay exact");}
        }
    }
}
