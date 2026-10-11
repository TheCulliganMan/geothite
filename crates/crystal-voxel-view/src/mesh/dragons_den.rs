//! Dragon's Den borrows Johto drawings without becoming an outdoor meadow.
//! Repaint proven terrain only; keep source contours, heights and object art.
use super::*;
use bevy::math::Vec2;

fn stone(source: &VisualTileSource) -> bool {
    if source.tileset_id.as_ref() != "johto"
        || source.subtile_column >= 4 || source.subtile_row >= 4 {
        return false;
    }
    if matches!(shape_for_source(source), CellShape::Flat)
        && matches!(source.tile_index, 0x05 | 0x06) {
        return true;
    }
    const ROCK: [[u16; 4]; 4] = [
        [0x2b,0x2c,0x2c,0x2d], [0x3b,0x3c,0x3c,0x3d],
        [0x3b,0x3c,0x3c,0x3d], [0x4b,0x4c,0x4c,0x4d],
    ];
    source.metatile_id == 0x0a
        && source.tile_index == ROCK[source.subtile_row as usize][source.subtile_column as usize]
        || matches!(shape_for_source(source), CellShape::ShoreBand)
}

pub(super) fn finish(mesh: &mut TerrainMeshData, map: &str, cells: &[&VisualTile],
    g: &GridGeometry, origin: [i32;2]) -> bool {
    if map != "DragonsDenB1F" || cells.is_empty()
        || cells.iter().any(|t| t.source.tileset_id.as_ref() != "johto") {
        return false;
    }
    let boundary = den_water_boundary::append(mesh,cells,g,origin);
    let source = &mut mesh.textured;
    let mut retained = Vec::with_capacity(source.indices.len());
    for ids in source.indices.chunks_exact(3) {
        let uv = ids.iter().fold(Vec2::ZERO, |v, &i| v + Vec2::from_array(source.uvs[i as usize])) / 3.;
        let x = (uv.x * g.width as f32).floor() as usize;
        let z = (uv.y * g.height as f32).floor() as usize;
        if x < g.width && z < g.height && boundary.contains(&(z*g.width+x)) {
            continue;
        }
        // Folded rock sides can borrow another atlas cell. Probe slightly
        // inside the actual face to retain the owning platform's material.
        let normal = Vec3::from_array(source.normals[ids[0] as usize]);
        let center = ids.iter().fold(Vec3::ZERO, |v,&i| v + Vec3::from_array(source.positions[i as usize])) / 3.
            - normal * 0.01;
        let wx = ((center.x-g.origin_x)/g.tile_width).floor() as i32;
        let wz = ((center.z-g.origin_z)/g.tile_height).floor() as i32;
        let owned = wx >= 0 && wz >= 0 && wx < g.width as i32 && wz < g.height as i32
            && { let s=&cells[wz as usize*g.width+wx as usize].source;
                s.metatile_id == 0x0a && stone(s) };
        let sampled = (0. ..1.).contains(&uv.x) && (0. ..1.).contains(&uv.y)
            && x < g.width && z < g.height && stone(&cells[z*g.width+x].source);
        if !owned && !sampled {
            retained.extend_from_slice(ids);
            continue;
        }
        let first = mesh.solid.positions.len() as u32;
        for &i in ids {
            let p = Vec3::from_array(source.positions[i as usize]);
            let normal = source.normals[i as usize];
            let top = normal[1] > 0.99;
            let base = if top && p.y.abs() < 0.01 { [0.43,0.405,0.32,1.] }
                else { [0.34,0.345,0.31,1.] };
            mesh.solid.positions.push(p.to_array());
            mesh.solid.normals.push(normal);
            mesh.solid.uvs.push(if top { cave_cliff::floor_uv(p,g,origin,false) }
                else { cave_cliff::stone_uv(p,g,origin,false) });
            mesh.solid.colors.push(cave_cliff::pigment(base,p,g,origin,false));
        }
        mesh.solid.indices.extend([first,first+1,first+2]);
    }
    // Retain the old vertex indexing and cutaway ownership for untouched art.
    // Only the selected terrain triangles move to the mineral material.
    source.indices = retained;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn den_rock_material_preserves_native_cap_and_water_and_ignores_outdoors() {
        let g = GridGeometry { width:2,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0. };
        let mut a=VisualTile {column:0,row:0,texture:Default::default(),
            animation_frames:None,priority:false,
            source:VisualTileSource {tileset_id:"johto".into(),metatile_id:0x0a,
                subtile_column:1,subtile_row:1,tile_index:0x3c}};
        let mut b=a.clone();b.source.metatile_id=0x35;b.source.tile_index=0x14;
        let cells=vec![&a,&b];
        let make=|| {
            let mut mesh=TerrainMeshData::default();
            append_top(&mut mesh.textured,[0.,8.,0.,8.],16.,g.uv(0,0));
            append_top(&mut mesh.textured,[8.,16.,0.,8.],-2.,g.uv(1,0));
            mesh
        };
        let mut outdoor=make();let original=outdoor.textured.clone();
        assert!(!finish(&mut outdoor,"Route45",&cells,&g,[0,0]));
        assert_eq!(outdoor.textured,original);
        let mut den=make();assert!(finish(&mut den,"DragonsDenB1F",&cells,&g,[0,0]));
        assert_eq!(den.textured.indices,original.indices[6..]);
        assert_eq!(den.textured.positions,original.positions);
        assert_eq!(den.textured.uvs,original.uvs);
        assert!(den.solid.positions.iter().all(|p|p[1]==16.));
        assert!(den.solid.uvs.iter().all(|p|(p[1]/100000.).round()==4.));
        let mut folded = TerrainMeshData::default();
        append_quad(&mut folded.textured,
            [[0.,-2.,0.],[0.,-2.,8.],[0.,16.,8.],[0.,16.,0.]],[-1.,0.,0.],
            [[0.75,0.5];4],[1.;4]);
        let face = folded.textured.positions.clone();
        assert!(finish(&mut folded,"DragonsDenB1F",&cells,&g,[0,0]));
        assert!(folded.textured.indices.is_empty(), "a folded platform face must follow its owner");
        assert_eq!(folded.solid.positions,vec![face[0],face[1],face[2],face[0],face[2],face[3]]);
        assert!(folded.solid.uvs.iter().all(|uv|(uv[1]/100000.).round()==6.));
        a.source.tile_index=0x14;assert!(!stone(&a.source));
        a.source.tileset_id="house".into();a.source.tile_index=0x06;assert!(!stone(&a.source));
    }
}
