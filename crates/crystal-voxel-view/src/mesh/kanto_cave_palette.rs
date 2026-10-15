//! Native cave pigment with distinct walking floors, shelf tops and rock faces.
//! This finish runs after shaping: it cannot alter collision or conceal stairs.
//! The live 2D swatches own hue and temperature; mineral detail is monochrome.
use super::*;

fn linear(rgb: [u8;3]) -> Vec3 {
    Vec3::from_array(rgb.map(|v| {
        let v=f32::from(v)/255.;
        if v<=0.04045 {v/12.92} else {((v+0.055)/1.055).powf(2.4)}
    }))
}

fn field(cells:&[&VisualTile],images:&TerrainImageSamples,index:u16)->Option<Vec3> {
    for tile in cells {
        let s=&tile.source;
        if !matches!(s.tileset_id.as_ref(),"cave"|"dark_cave")
            || s.tile_index!=index || s.subtile_column>=4 || s.subtile_row>=4 {continue;}
        let valid=match index {
            0x16=>crate::cave::cave_shape(s)==Some(CellShape::Flat),
            0x01=>crate::cave::cave_shape(s).is_some_and(|shape|
                shape.surface_height(SOURCE_TILE_HEIGHT)==crate::cave::CAVE_SHELF_HEIGHT),
            0x0c=>crate::cave::small_rock_local(s)==Some((0,0)),
            0x26=>crate::cave::cave_shape(s).is_some_and(|shape|
                shape.surface_height(SOURCE_TILE_HEIGHT)==crate::cave::rock_height(s)),
            _=>false,
        };
        if !valid {continue;}
        let Ok(rgba)=tile_rgba(images,tile) else {continue;};
        if rgba.len()!=256 || rgba.chunks_exact(4).any(|p|p[3]!=255) {continue;}
        let mut counts=std::collections::BTreeMap::<[u8;3],usize>::new();
        for p in rgba.chunks_exact(4) {*counts.entry([p[0],p[1],p[2]]).or_default()+=1;}
        // The repeated floor field owns the mineral hue. Black outlines are
        // source ink, not a substitute pigment for every walking surface.
        let visible=counts.iter().filter(|(rgb,_)|
            linear(**rgb).dot(Vec3::new(0.2126,0.7152,0.0722))>0.006);
        // Dense hatch ink can outnumber the pale rock in a ceiling drawing.
        // Walking dust uses its background field; shelf/ceiling caps use the
        // actual light mineral swatch, never a scaled or invented floor tint.
        if index==0x16 {
            // The classic dust drawing includes darker mineral pigment.
            // Preserve its actual linear average rather than lighting the
            // entire 3D floor with only the brightest/background swatch.
            let (sum,count)=visible.fold((Vec3::ZERO,0usize),|(sum,count),(rgb,n)|
                (sum+linear(*rgb)*(*n as f32),count+*n));
            if count>0 {return Some(sum/count as f32);}
            continue;
        }
        let sample=visible.max_by(|(a,_),(b,_)| {
            let luma=|rgb|linear(rgb).dot(Vec3::new(0.2126,0.7152,0.0722));
            luma(**a).total_cmp(&luma(**b))
        });
        let Some((rgb,_))=sample else {continue;};
        return Some(linear(*rgb));
    }
    None
}

fn apply(out:&mut SurfaceMeshData,pigment:Vec3,shelf:Option<Vec3>,cap:Option<Vec3>,rock:Option<Vec3>,g:&GridGeometry,tag_shift:f32)->usize {
    let mut changed=0;
    for i in 0..out.positions.len() {
        let uv=&mut out.uvs[i];
        if uv[0]<=90000. {continue;}
        let category=(uv[1]/100000.).round() as i32;
        let reference=match category {2=>0.34,4=>0.402,6|32=>0.268,26|34=>0.30,28=>0.65,30=>0.402,_=>continue};
        let normal=out.normals[i];let y=out.positions[i][1]/g.tile_height*8.;
        let wall=normal[1].abs()<0.45;
        // Broad value changes communicate the playable surface before grain:
        // pale shelf tops, a quieter lower floor, dark vertical scarps/rock.
        // The pale walking shelf has its own source pigment. A scaled lower
        // floor color loses the authored distinction, especially in darkness.
        let raised=matches!(category,4|30) && !wall && y>=crate::cave::CAVE_SHELF_HEIGHT-0.01;
        let enclosing_cap=category==2 && !wall && y>crate::cave::CAVE_SHELF_HEIGHT;
        let pigment=if category==28 {let Some(shelf)=shelf else {continue;};shelf}
            else if raised {let Some(shelf)=shelf else {continue;};shelf}
            else if enclosing_cap {let Some(cap)=cap else {continue;};cap}
            else if matches!(category,6|32) {let Some(rock)=rock else {continue;};rock}
            else {pigment};
        let role=match (category,wall) {
            (26 | 28 | 30 | 34,_)=>1.0,
            (4,false) if raised=>1.0,
            (4,false)=>1.0,
            (_,true)=>0.52,
            (2,false) if enclosing_cap=>1.0,
            (2,false)=>0.78,
            (6|32,false)=>0.88,
            _=>0.70,
        };
        let c=&mut out.colors[i];
        let luma=Vec3::new(c[0],c[1],c[2]).dot(Vec3::new(0.2126,0.7152,0.0722));
        let variation=if matches!(category,26|30|34) {(luma/reference).clamp(0.,1.07)}
            else {(luma/reference).clamp(0.91,1.07)};
        c[..3].copy_from_slice(&(pigment*role*variation).to_array());
        // Native domains keep the source hue in the shader and use restrained
        // monochrome mineral grain. Retain the same map-space coordinates.
        if !matches!(category,26|28|30|32|34) {uv[1]+=tag_shift;}
        changed+=1;
    }
    changed
}

pub(super) fn finish(mesh:&mut TerrainMeshData,map:&str,cells:&[&VisualTile],
    images:&TerrainImageSamples,g:&GridGeometry)->usize {
    if map.starts_with("UnionCave") || map.ends_with("Gym")
        || !cells.first().is_some_and(|tile|
            matches!(tile.source.tileset_id.as_ref(),"cave"|"dark_cave")) {return 0;}
    let Some(pigment)=field(cells,images,0x16) else {return 0;};
    let shelf=field(cells,images,0x01);
    let cap=field(cells,images,0x26);
    let rock=field(cells,images,0x0c);
    let tag_shift=if map=="DiglettsCave" {1600000.} else {800000.};
    apply(&mut mesh.textured,pigment,shelf,cap,rock,g,tag_shift)+apply(&mut mesh.solid,pigment,shelf,cap,rock,g,tag_shift)
        +apply(&mut mesh.cave_enclosure,pigment,shelf,cap,rock,g,tag_shift)
}

/// Union Cave's reviewed floors retain their authored mineral values without
/// changing their shelf topology, ladders, support, or enclosing rock.
pub(super) fn finish_union(mesh:&mut TerrainMeshData,map:&str,cells:&[&VisualTile],
    images:&TerrainImageSamples,g:&GridGeometry)->usize {
    if !matches!(map,"UnionCave1F"|"UnionCaveB1F"|"UnionCaveB2F") {return 0;}
    let Some(pigment)=field(cells,images,0x16) else {return 0;};
    let shelf=field(cells,images,0x01);
    let cap=field(cells,images,0x26);
    let rock=field(cells,images,0x0c);
    apply(&mut mesh.textured,pigment,shelf,cap,rock,g,1600000.)
        +apply(&mut mesh.solid,pigment,shelf,cap,rock,g,1600000.)
        +apply(&mut mesh.cave_enclosure,pigment,shelf,cap,rock,g,1600000.)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn floor_keeps_the_classic_dark_pigment_in_its_measured_color() {
        let tile=VisualTile{column:0,row:0,texture:Handle::default(),priority:false,
            animation_frames:None,source:VisualTileSource{tileset_id:"cave".into(),
                metatile_id:2,subtile_column:0,subtile_row:0,tile_index:0x16}};
        let mut pixels=[99,74,123,255].repeat(48);
        pixels.extend([66,33,41,255].repeat(16));
        let mut samples=TerrainImageSamples::default();
        samples.pixels.insert(tile.texture.id(),TileImageSample::Rgba(pixels));
        let actual=field(&[&tile],&samples,0x16).unwrap();
        let expected=linear([99,74,123])*0.75+linear([66,33,41])*0.25;
        assert!(actual.abs_diff_eq(expected,0.000001));
        assert!(!actual.abs_diff_eq(linear([99,74,123]),0.000001));
    }

    #[test]
    fn kanto_native_cave_roles_make_walkable_levels_distinct_without_changing_geometry() {
        let mut assets=Assets::<Image>::default();
        let tile=VisualTile{column:0,row:0,texture:assets.add(Image::default()),priority:false,
            animation_frames:None,source:VisualTileSource{tileset_id:"dark_cave".into(),
                metatile_id:0,subtile_column:0,subtile_row:0,tile_index:0x16}};
        let mut images=TerrainImageSamples::default();
        let rgba=[148,107,173,255].repeat(64);
        images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(rgba));
        let mut upper=tile.clone();
        upper.texture=assets.add(Image::default());
        upper.source.tile_index=0x01;
        images.pixels.insert(upper.texture.id(),TileImageSample::Rgba([198,165,222,255].repeat(64)));
        let mut ceiling=tile.clone();
        ceiling.texture=assets.add(Image::default());
        ceiling.source.metatile_id=0x2d;
        ceiling.source.tile_index=0x26;
        let mut hatched_cap=[66,33,41,255].repeat(56);
        hatched_cap.extend([123,115,198,255].repeat(8));
        images.pixels.insert(ceiling.texture.id(),TileImageSample::Rgba(hatched_cap));
        let mut stone=tile.clone();
        stone.texture=assets.add(Image::default());stone.source.tile_index=0x0c;
        images.pixels.insert(stone.texture.id(),TileImageSample::Rgba([99,74,123,255].repeat(64)));
        let g=GridGeometry{width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut mesh=TerrainMeshData{footing_heights:vec![6.],..Default::default()};
        for (y,n,category) in [(0.,[0.,1.,0.],4),(6.,[0.,1.,0.],4),
            (3.,[0.,0.,1.],6),(40.,[0.,1.,0.],2),(0.,[0.,1.,0.],5)] {
            mesh.solid.positions.push([2.,y,3.]);mesh.solid.normals.push(n);
            mesh.solid.uvs.push([100002.,category as f32*100000.+3.]);
            mesh.solid.colors.push([0.4,0.4,0.4,0.8]);
        }
        let before=mesh.clone();
        assert_eq!(finish(&mut mesh,"UnionCave1F",&[&tile],&images,&g),0);
        assert_eq!(mesh.solid,before.solid);
        let mut diglett=before.clone();
        assert_eq!(finish(&mut diglett,"DiglettsCave",&[&tile,&upper,&ceiling,&stone],&images,&g),4);
        assert_eq!(diglett.solid.positions,before.solid.positions);
        assert_eq!(diglett.footing_heights,before.footing_heights);
        assert_eq!(diglett.solid.uvs[..4].iter().map(|uv|(uv[1]/100000.).round() as u8)
            .collect::<Vec<_>>(),[20,20,22,18]);
        let expected=linear([148,107,173])*(0.4/0.402);
        for (actual,expected) in diglett.solid.colors[0][..3].iter().zip(expected.to_array()) {
            assert!((*actual-expected).abs()<0.00001);
        }
        for map in ["UnionCave1F","UnionCaveB1F","UnionCaveB2F"] {
            let mut union=before.clone();
            assert_eq!(finish_union(&mut union,map,&[&tile,&upper,&ceiling,&stone],&images,&g),4);
            assert_eq!(union.solid.positions,before.solid.positions);
            assert_eq!(union.solid.normals,before.solid.normals);
            assert_eq!(union.solid.indices,before.solid.indices);
            assert_eq!(union.footing_heights,before.footing_heights);
            assert!(union.solid.colors[1][0]>union.solid.colors[0][0]*1.8);
            assert_eq!(union.solid.colors[4],before.solid.colors[4]);
            assert_eq!(union.solid.uvs[4],before.solid.uvs[4]);
            assert_eq!(union.solid.uvs[..4].iter().map(|uv|(uv[1]/100000.).round() as u8)
                .collect::<Vec<_>>(),[20,20,22,18]);
        }
        assert_eq!(finish_union(&mut mesh,"SilverCaveRoom2",&[&tile],&images,&g),0);
        assert_eq!(mesh.solid,before.solid);
        assert_eq!(finish(&mut mesh,"VictoryRoad",&[&tile,&upper,&ceiling,&stone],&images,&g),4);
        assert_eq!(mesh.solid.positions,before.solid.positions);
        assert_eq!(mesh.solid.normals,before.solid.normals);
        assert_eq!(mesh.solid.indices,before.solid.indices);
        assert_eq!(mesh.footing_heights,before.footing_heights);
        let light=|c:[f32;4]|Vec3::new(c[0],c[1],c[2]).length();
        let colors=&mesh.solid.colors;
        assert!(light(colors[1])>light(colors[0])*1.8);
        let expected=linear([198,165,222])*(0.4/0.402);
        for (actual,expected) in colors[1][..3].iter().zip(expected.to_array()) {
            assert!((*actual-expected).abs()<0.00001);
        }
        let mut missing=before.clone();
        assert_eq!(finish(&mut missing,"VictoryRoad",&[&tile],&images,&g),1);
        assert_eq!(missing.solid.colors[1],before.solid.colors[1]);
        assert_eq!(missing.solid.uvs[1],before.solid.uvs[1]);
        assert_eq!(missing.solid.colors[2],before.solid.colors[2]);
        assert_eq!(missing.solid.uvs[2],before.solid.uvs[2]);
        let stone_expected=linear([99,74,123])*0.52*1.07;
        for (actual,expected) in colors[2][..3].iter().zip(stone_expected.to_array()) {
            assert!((*actual-expected).abs()<0.00001);
        }
        assert_eq!(missing.solid.colors[3],before.solid.colors[3]);
        assert_eq!(missing.solid.uvs[3],before.solid.uvs[3]);
        let cap_expected=linear([123,115,198])*1.07;
        for (actual,expected) in colors[3][..3].iter().zip(cap_expected.to_array()) {
            assert!((*actual-expected).abs()<0.00001);
        }
        assert!(light(colors[2])<light(colors[0])*0.8);
        assert!(colors[..4].iter().all(|c|c[3]==0.8));
        assert_eq!(colors[4],before.solid.colors[4]);
        assert_eq!(mesh.solid.uvs[4],before.solid.uvs[4]);
        assert_eq!(mesh.solid.uvs[..4].iter().map(|uv|(uv[1]/100000.).round() as u8)
            .collect::<Vec<_>>(),[12,12,14,10]);
        let mut source_colored=before.clone();
        assert_eq!(finish(&mut source_colored,"TohjoFalls",&[&tile,&upper,&ceiling,&stone],&images,&g),4);
        let expected=linear([198,165,222])*(0.4/0.402);
        for (actual,expected) in source_colored.solid.colors[1][..3].iter().zip(expected.to_array()) {
            assert!((*actual-expected).abs()<0.00001);
        }
        images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(vec![0;256]));
        let mut invalid=before.clone();
        assert_eq!(finish(&mut invalid,"VictoryRoad",&[&tile],&images,&g),0);
        assert_eq!(invalid.solid,before.solid);
    }
}
