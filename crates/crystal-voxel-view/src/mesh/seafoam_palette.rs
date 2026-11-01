//! Seafoam Gym's live ochre palette on authenticated cave material domains.
//! Geometry, mineral coordinates and authoritative support remain unchanged.
use super::*;

fn linear(rgb: [u8; 3]) -> Vec3 {
    Vec3::from_array(rgb.map(|v| {
        let v = f32::from(v) / 255.;
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    }))
}

fn source_palette(cells: &[&VisualTile], images: &TerrainImageSamples) -> Option<[Vec3; 3]> {
    for tile in cells {
        let s = &tile.source;
        // Seafoam's reviewed plain shelf block has sixteen tile01 cells.
        // Its image carries all three native mineral shades. Read the live
        // image rather than assuming the map's day/night or custom colors.
        if s.tileset_id.as_ref() != "cave" || s.metatile_id != 0x24
            || s.tile_index != 0x01 || s.subtile_column >= 4 || s.subtile_row >= 4 {
            continue;
        }
        let Ok(rgba) = tile_rgba(images,tile) else { continue; };
        if rgba.len() != 8*8*4 { continue; }
        let mut palette = [Vec3::ZERO;3];
        let mut opaque = true;
        for (i,[x,y]) in [[1,1],[0,0],[3,0]].into_iter().enumerate() {
            let at = (y*8+x)*4;
            opaque &= rgba[at+3] == 255;
            palette[i] = linear([rgba[at],rgba[at+1],rgba[at+2]]);
        }
        if opaque { return Some(palette); }
    }
    None
}

fn apply(surface: &mut SurfaceMeshData, palette: [Vec3;3]) -> usize {
    let mut changed = 0;
    for i in 0..surface.positions.len() {
        let Some(uv) = surface.uvs.get(i) else { continue; };
        if uv[0] <= 90000. { continue; }
        let category = (uv[1]/100000.).round() as i32;
        // These tags originate only in the cave enclosure, mineral floor
        // and complete low-rock producers. Frozen/outdoor/water/actor art
        // occupies separate domains and is never recolored here.
        let reference = match category {2=>0.34,4=>0.402,6=>0.268,_=>continue};
        let c = &mut surface.colors[i];
        let luma = Vec3::new(c[0],c[1],c[2]).dot(Vec3::new(0.2126,0.7152,0.0722));
        let t = ((luma/reference-0.65)/0.55).clamp(0.,1.);
        let color = if t<0.5 {palette[0].lerp(palette[1],t*2.)}
            else {palette[1].lerp(palette[2],(t-0.5)*2.)};
        c[..3].copy_from_slice(&color.to_array());
        changed += 1;
    }
    changed
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData, map: &str, cells: &[&VisualTile], images: &TerrainImageSamples,
) -> usize {
    if map != "SeafoamGym" { return 0; }
    let Some(palette) = source_palette(cells,images) else { return 0; };
    apply(&mut mesh.solid,palette)
        + apply(&mut mesh.cave_enclosure,palette)
        + apply(&mut mesh.textured,palette)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seafoam_live_palette_image_roles_follow_current_colors_and_reject_missing_alpha() {
        let tile = VisualTile {column:0,row:0,texture:Default::default(),priority:false,
            animation_frames:None,source:VisualTileSource {tileset_id:"cave".into(),
                metatile_id:0x24,subtile_column:0,subtile_row:0,tile_index:1}};
        let mut images=TerrainImageSamples::default();
        assert!(source_palette(&[&tile],&images).is_none());
        let mut rgba=vec![255;256];
        for ([x,y],rgb) in [([1,1],[10u8,20,30]),([0,0],[40,50,60]),([3,0],[70,80,90])] {
            let at=(y*8+x)*4;rgba[at..at+3].copy_from_slice(&rgb);
        }
        images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(rgba.clone()));
        assert_eq!(source_palette(&[&tile],&images).unwrap(),
            [[10,20,30],[40,50,60],[70,80,90]].map(linear));
        rgba[(1*8+1)*4+3]=0;
        images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(rgba));
        assert!(source_palette(&[&tile],&images).is_none());
        images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(vec![255;4]));
        assert!(source_palette(&[&tile],&images).is_none());
    }
    #[test]
    fn seafoam_live_palette_keeps_geometry_support_and_other_material_domains() {
        let mut tile = VisualTile {column:0,row:0,texture:Default::default(),priority:false,
            animation_frames:None,source:VisualTileSource {tileset_id:"cave".into(),
                metatile_id:0x24,subtile_column:0,subtile_row:0,tile_index:1}};
        let mut rgba=vec![255;256];
        for ([x,y],rgb) in [([1,1],[132,107,24]),([0,0],[173,140,57]),([3,0],[214,198,140])] {
            rgba[(y*8+x)*4..(y*8+x)*4+3].copy_from_slice(&rgb.map(|v|v as u8));
        }
        let mut images=TerrainImageSamples::default();
        images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(rgba));
        let mut mesh=TerrainMeshData {footing_heights:vec![4.],..Default::default()};
        for category in [2,4,6,3,5,7,8,9] {
            mesh.solid.positions.push([1.,4.,2.]);mesh.solid.normals.push([0.,1.,0.]);
            mesh.solid.uvs.push([100001.,category as f32*100000.+2.]);
            mesh.solid.colors.push([0.3,0.3,0.3,0.75]);mesh.solid.indices.push(category as u32);
        }
        mesh.cave_enclosure=mesh.solid.clone();
        mesh.textured=mesh.solid.clone();
        // Real source-art UVs remain untouched even in the same surface.
        mesh.textured.uvs[0]=[0.25,0.25];
        let before=mesh.clone();
        assert_eq!(finish(&mut mesh,"SeafoamIslandsB1F",&[&tile],&images),0);
        assert_eq!(mesh.solid,before.solid);
        assert_eq!(finish(&mut mesh,"SeafoamGym",&[&tile],&images),8);
        assert_eq!(mesh.solid.positions,before.solid.positions);
        assert_eq!(mesh.solid.normals,before.solid.normals);
        assert_eq!(mesh.solid.uvs,before.solid.uvs);
        assert_eq!(mesh.solid.indices,before.solid.indices);
        assert_eq!(mesh.footing_heights,before.footing_heights);
        assert_eq!(mesh.cave_enclosure.positions,before.cave_enclosure.positions);
        assert_eq!(mesh.cave_enclosure.uvs,before.cave_enclosure.uvs);
        assert_eq!(mesh.cave_enclosure.indices,before.cave_enclosure.indices);
        assert_eq!(mesh.textured.colors[0],before.textured.colors[0]);
        assert_eq!(&mesh.cave_enclosure.colors[3..],&before.cave_enclosure.colors[3..]);
        assert_eq!(&mesh.textured.colors[3..],&before.textured.colors[3..]);
        assert_eq!(&mesh.solid.colors[3..],&before.solid.colors[3..]);
        assert!(mesh.solid.colors[..3].iter().all(|c|c[0]>c[1] && c[1]>c[2] && c[3]==0.75));
        tile.source.subtile_row=255;
        assert_eq!(finish(&mut mesh,"SeafoamGym",&[&tile],&images),0);
        assert_eq!(finish(&mut mesh,"SeafoamGym",&[],&images),0);
    }
}
