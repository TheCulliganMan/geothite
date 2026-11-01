//! Retained optical depth from the authored shoreline. This is presentation
//! data: source geometry, source animation and swimming support stay intact.
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use crystal_render_api::VisualWorldFrame;

const SAMPLES: usize = 4;
const REACH: f32 = 8.0;

fn contact_boundary(triangles:&[[Vec2;3]])->Vec<[Vec2;2]> {
    let mut edges=std::collections::BTreeMap::new();
    for triangle in triangles {for [a,b] in [[triangle[0],triangle[1]],[triangle[1],triangle[2]],[triangle[2],triangle[0]]] {
        let (ka,kb)=(a.to_array().map(f32::to_bits),b.to_array().map(f32::to_bits));
        let key=if ka<kb {(ka,kb)}else{(kb,ka)};
        let entry=edges.entry(key).or_insert((0,[a,b]));entry.0+=1;
    }}
    edges.into_values().filter_map(|(count,edge)|(count==1).then_some(edge)).collect()
}

fn contact_distance(point:Vec2,triangles:&[[Vec2;3]],boundary:&[[Vec2;2]])->f32 {
    let inside=triangles.iter().any(|&[a,b,c]| {
        let signs=[(b-a).perp_dot(point-a),(c-b).perp_dot(point-b),(a-c).perp_dot(point-c)];
        signs.iter().all(|&v|v>=-0.000001) || signs.iter().all(|&v|v<=0.000001)
    });
    let distance=boundary.iter().map(|&[a,b]| {
        let along=b-a;
        (point-a-along*((point-a).dot(along)/along.length_squared()).clamp(0.,1.)).length()
    }).fold(REACH,f32::min);
    if inside {-distance}else{distance}
}

/// Owned, frame-specific CPU images. None is an authenticated dry result,
/// distinct from a caller that has not prepared these images yet.
pub(super) struct PreparedImages {
    pub mask: Option<Image>,
    pub shore: Option<Image>,
}
impl PreparedImages {
    pub fn for_frame(frame: &VisualWorldFrame) -> Self {
        let mask = {
            let _trace = crate::loading_trace::span("region.prepare_water_mask_image", ||
                serde_json::json!({"grid":frame.grid_size.to_array()}));
            crate::cave_water::mask(frame)
        };
        let shore = mask.as_ref().and_then(|_| {
            let _trace = crate::loading_trace::span("region.prepare_water_shore_image", ||
                serde_json::json!({"grid":frame.grid_size.to_array()}));
            shoreline(frame)
        });
        Self { mask, shore }
    }
}

/// Water contact follows the published player foot, never collision or a
/// camera-relative decal. A dry tile, cascade or unavailable source clears it.
pub(super) fn contact(frame: &VisualWorldFrame) -> Vec4 {
    let Some(actor) = frame
        .actors
        .iter()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
    else {
        return Vec4::ZERO;
    };
    if !frame.tile_size.is_finite() || frame.tile_size.min_element() <= 0.0 {
        return Vec4::ZERO;
    }
    let foot = crate::footing::actor_ground_point(frame, actor);
    if !foot.is_finite() || !actor.size.is_finite() {
        return Vec4::ZERO;
    }
    let Some(cell) = crate::footing::tile_at_visual_point(frame, foot + Vec2::Y * 0.01) else {
        return Vec4::ZERO;
    };
    let registration = crate::cave_water::registration(frame);
    let map = (Vec2::new(foot.x, -foot.y) + Vec2::new(registration.z, registration.w))
        / frame.tile_size.x;
    let index = cell.y as usize * frame.grid_size.x as usize + cell.x as usize;
    let tile = frame
        .tiles
        .get(index)
        .filter(|tile| tile.column == cell.x && tile.row == cell.y)
        .or_else(|| {
            frame
                .tiles
                .iter()
                .find(|tile| tile.column == cell.x && tile.row == cell.y)
        });
    let wet =
        tile.is_some_and(|tile| crate::cave_water::role_for_map(tile, &frame.map_id) == [255, 0]);
    if !wet {
        return Vec4::ZERO;
    }
    Vec4::new(
        map.x,
        map.y,
        (actor.size.x / frame.tile_size.x * 0.55).clamp(0.6, 1.5),
        1.0,
    )
}

/// Smooth toward a received source frame, then stop there. No phase is
/// extrapolated when the runtime pauses or stops publishing animation.
pub(super) fn blend_clock(previous: Vec4, time: f32, steps: Vec2) -> Vec4 {
    let interval = if previous.y > 0.0 {
        (time - previous.x).rem_euclid(3600.0).clamp(0.05, 0.25)
    } else {
        0.10
    };
    Vec4::new(time, interval, steps.x, steps.y)
}

/// Accumulate slow optical motion from proven source advances. Unlike using
/// the three-frame atlas phase directly, a source wrap never reverses a swell.
/// Invalid/skipped steps hold the last pose; no renderer clock predicts water.
pub(super) fn advance_motion(previous: Vec4, steps: Vec2) -> Vec4 {
    let start = previous.x;
    let end = start + steps.x.max(0.) * 0.03;
    // Subtract from both endpoints so interpolation stays continuous at wrap.
    let wrap = (end / std::f32::consts::TAU).floor() * std::f32::consts::TAU;
    Vec4::new(end - wrap, start - wrap, 0., 0.)
}

pub(super) fn bounds(frame: &VisualWorldFrame) -> Vec4 {
    Vec4::new(
        frame.grid_origin.x as f32,
        frame.grid_origin.y as f32,
        frame.grid_size.x as f32,
        frame.grid_size.y as f32,
    )
}

/// Wavelength, normal strength, bank foam and exposed sky contribution.
pub(super) fn style(frame: &VisualWorldFrame) -> Vec4 {
    if frame.map_id.as_ref() == "TohjoFalls" {
        // The connected basins carry small source-driven ripples and very
        // little reflected fill under the cave roof.
        Vec4::new(0.95, 0.14, 0.10, 0.10)
    } else if frame.map_id.as_ref() == "CeruleanGym" {
        Vec4::new(1.35, 0.055, 0.0, 0.12)
    } else if matches!(frame.map_id.as_ref(), "NationalPark" | "NationalParkBugContest" | "SafariZoneBeta")
        && frame.tiles.iter().any(|tile| tile.source.tileset_id.as_ref() == "park")
    {
        // Sheltered ornamental ponds keep the shared source-driven crossing
        // ripples, with little breaking wash around the fountain's footing.
        Vec4::new(1.35, 0.075, 0.025, 0.32)
    } else if frame.map_id.as_ref() == "IlexForest"
        && frame.tiles.first().is_some_and(|tile| tile.source.tileset_id.as_ref() == "forest")
    {
        // A sheltered woodland pool has long low swells, almost no breaking
        // wash and little open-sky reflection. Reuse the existing optical pass.
        Vec4::new(0.55, 0.055, 0.035, 0.20)
    } else if crate::cave_water::outdoor(&frame.map_id) {
        if crate::environment_fx::coastal(frame)
            || matches!(
                frame.map_id.as_ref(),
                "OlivinePort" | "VermilionPort" | "VermilionCity" | "CinnabarIsland"
                    | "PalletTown" | "Route12" | "Route13"
            )
        {
            Vec4::new(0.72, 0.23, 0.48, 0.40)
        } else {
            Vec4::new(1.0, 0.15, 0.23, 0.32)
        }
    } else {
        Vec4::new(1.25, 0.09, 0.09, 0.0)
    }
}

/// Four samples per source cell resolve narrow banks without another atlas-
/// sized texture. Only known dry cells seed the distance transform; a cropped
/// grid edge and unavailable cells never invent a shoreline in open water.
/// The bounded two-pass chamfer transform is linear in the retained grid size.
pub(super) fn shoreline(frame: &VisualWorldFrame) -> Option<Image> {
    let columns = frame.grid_size.x as usize;
    let rows = frame.grid_size.y as usize;
    let width = columns.checked_mul(SAMPLES)?;
    let height = rows.checked_mul(SAMPLES)?;
    if width == 0 || height == 0 {
        return None;
    }
    let mut cells = vec![None; columns.checked_mul(rows)?];
    for tile in &frame.tiles {
        if tile.column as usize >= columns || tile.row as usize >= rows {
            return None;
        }
        if cells[tile.row as usize * columns + tile.column as usize]
            .replace(tile)
            .is_some()
        {
            return None;
        }
    }
    // Complete native sea rocks use the waterline of their actual shared
    // model. Their transparent-looking atlas corners are water, not four
    // square dry cells. Keep incomplete/custom drawings on the source baseline.
    let mut rock_cells = vec![None; cells.len()];
    let mut rocks = Vec::new();
    for (i,tile) in cells.iter().enumerate() {
        let Some(tile) = tile else {continue;};
        if crate::kanto_cliff::water_backed_barrier_local(&tile.source,frame.grid_origin+IVec2::new(tile.column as i32,tile.row as i32),frame.terrace_datum.as_deref())!=Some((0,0)) {continue;}
        let (x,y)=(i%columns,i/columns);
        if x+1>=columns || y+1>=rows {continue;}
        let complete=(0..2).all(|dy|(0..2).all(|dx| {
            cells[(y+dy)*columns+x+dx].is_some_and(|other|
                other.source.metatile_id==tile.source.metatile_id
                && other.source.subtile_column as usize==tile.source.subtile_column as usize+dx
                && other.source.subtile_row as usize==tile.source.subtile_row as usize+dy
                && crate::kanto_cliff::water_backed_barrier_local(&other.source,frame.grid_origin+IVec2::new(other.column as i32,other.row as i32),frame.terrace_datum.as_deref())==Some((dx as u8,dy as u8)))
        }));
        if !complete {continue;}
        let contact=crate::exterior_models::kanto_shore_contact(
            [frame.grid_origin.x+x as i32,frame.grid_origin.y+y as i32]);
        if contact.is_empty() {continue;}
        for dy in 0..2 {for dx in 0..2 {rock_cells[(y+dy)*columns+x+dx]=Some(rocks.len());}}
        let boundary=contact_boundary(&contact);
        rocks.push((Vec2::new(x as f32,y as f32),contact,boundary));
    }
    // The harbor posts replace complete native water-backed drawings. Their
    // shadows ground the solids; their four-cell plots are not dry islands.
    let mut harbor_water=vec![false;cells.len()];
    if crate::port::harbor_map(&frame.map_id) {
        for (i,tile) in cells.iter().enumerate() {
            let Some(tile)=tile else {continue;};
            if crate::port::bollard_local(&tile.source)!=Some((0,0)) {continue;}
            let (x,y)=(i%columns,i/columns);
            if x+1>=columns || y+1>=rows {continue;}
            let complete=(0..2).all(|dy|(0..2).all(|dx|
                cells[(y+dy)*columns+x+dx].is_some_and(|tile|
                    crate::port::bollard_local(&tile.source)==Some((dx as u8,dy as u8)))));
            if complete {for dy in 0..2 {for dx in 0..2 {harbor_water[(y+dy)*columns+x+dx]=true;}}}
        }
        // A complete modeled ferry floats above continuous sea. Its native
        // sprite rectangle is not a dry island and must not seed square beach
        // foam. Incomplete/custom ship art keeps the original shoreline rule.
        for y in 0..rows.saturating_sub(7) { for x in 0..columns.saturating_sub(15) {
            if !cells[y*columns+x].is_some_and(|tile|crate::port::ferry_source(&tile.source,0,0)) {
                continue;
            }
            if (0..8).all(|dy|(0..16).all(|dx|cells[(y+dy)*columns+x+dx]
                .is_some_and(|tile|crate::port::ferry_source(&tile.source,dx,dy)))) {
                for dy in 0..8 {for dx in 0..16 {harbor_water[(y+dy)*columns+x+dx]=true;}}
            }
        }}
    }
    let mut border_water=vec![false;cells.len()];
    for (bank,_) in crate::border_water::replacements(frame) {
        border_water[bank.y as usize*columns+bank.x as usize]=true;
    }
    let mut distance = vec![REACH; width.checked_mul(height)?];
    // Small sea-rock feet interrupt waves and carry a narrow wash, but do
    // not imply a broad shallow seabed. Keep real land depth separately in
    // the same RG texture; no extra image or larger sampling grid is needed.
    let mut land_distance = distance.clone();
    let step = 1.0 / SAMPLES as f32;
    for (i, tile) in cells.iter().enumerate() {
        let Some(tile) = tile else {
            continue;
        };
        // The source buoy sprite floats at the water datum. Its retained
        // horizontal plane is scenery, not an island that creates bank foam.
        let buoy = matches!(tile.source.tileset_id.as_ref(), "johto" | "johto_modern")
            && (0x30..=0x39).contains(&tile.source.metatile_id)
            && tile.source.tile_index == 0x58
            && tile.source.subtile_column < 4
            && tile.source.subtile_row < 4;
        let wet = crate::cave_water::role_for_map(tile, &frame.map_id) != [0, 0]
            || harbor_water[i] || border_water[i]
            // Elevated decks have continuous water beneath them. Their
            // shadows describe the structure; a solid rectangular bank mask
            // made floating piers appear ringed by a pale, square beach.
            || crate::mesh::pier_over_water(&frame.map_id, &tile.source,
                i.checked_sub(columns).and_then(|north|cells[north]).map(|t|&t.source))
            || (buoy && matches!(
                crate::profile::shape_for_source_on_map(&frame.map_id, &tile.source),
                crate::profile::CellShape::PlaneAt { height }
                    if height == crate::profile::WATER_HEIGHT
            ))
            || matches!(
                crate::profile::shape_for_source_on_map(&frame.map_id, &tile.source),
                crate::profile::CellShape::Water | crate::profile::CellShape::Waterfall
            );
        if wet {
            continue;
        }
        for y in 0..SAMPLES {
            for x in 0..SAMPLES {
                if let Some(rock)=rock_cells[i] {
                    let (origin,triangles,boundary)=&rocks[rock];
                    let point=Vec2::new((i%columns) as f32+(x as f32+0.5)/SAMPLES as f32,
                        (i/columns) as f32+(y as f32+0.5)/SAMPLES as f32)-*origin;
                    // Keep the actual sub-sample rock contour. A binary
                    // four-sample raster expands each little foot into a
                    // square island and stamps a grid into dense shallows.
                    distance[(i / columns * SAMPLES + y) * width + i % columns * SAMPLES + x]
                        =contact_distance(point,triangles,boundary)+step*0.5;
                    continue;
                }
                let at=(i / columns * SAMPLES + y) * width + i % columns * SAMPLES + x;
                distance[at] = 0.0;
                land_distance[at] = 0.0;
            }
        }
    }
    for distance in [&mut distance,&mut land_distance] {
        let diagonal = step * std::f32::consts::SQRT_2;
        for y in 0..height {
            for x in 0..width {
                let i = y * width + x;
                if x > 0 {
                    distance[i] = distance[i].min(distance[i - 1] + step);
                }
                if y > 0 {
                    distance[i] = distance[i].min(distance[i - width] + step);
                    if x > 0 {
                        distance[i] = distance[i].min(distance[i - width - 1] + diagonal);
                    }
                    if x + 1 < width {
                        distance[i] = distance[i].min(distance[i - width + 1] + diagonal);
                    }
                }
            }
        }
        for y in (0..height).rev() {
            for x in (0..width).rev() {
                let i = y * width + x;
                if x + 1 < width {
                    distance[i] = distance[i].min(distance[i + 1] + step);
                }
                if y + 1 < height {
                    distance[i] = distance[i].min(distance[i + width] + step);
                    if x > 0 {
                        distance[i] = distance[i].min(distance[i + width - 1] + diagonal);
                    }
                    if x + 1 < width {
                        distance[i] = distance[i].min(distance[i + width + 1] + diagonal);
                    }
                }
            }
        }
    }
    // Signed half-sample offsets put the zero crossing at the actual bank,
    // instead of outlining source-cell centers. Linear sampling joins cells.
    let pixels = distance
        .into_iter().zip(land_distance)
        .flat_map(|(contact,land)| {
            let encode=|d:f32| {
                let signed = (d - step * 0.5).clamp(-REACH, REACH);
                ((signed / REACH * 0.5 + 0.5) * 255.0).round() as u8
            };
            [encode(contact), encode(land), 0, 255]
        })
        .collect();
    let mut image = Image::new(
        Extent3d {
            width: width.try_into().ok()?,
            height: height.try_into().ok()?,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::linear();
    Some(image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::{VisualTile, VisualTileSource};

    fn frame(columns: u32, rows: u32, dry: impl Fn(u32, u32) -> bool) -> VisualWorldFrame {
        let mut frame = VisualWorldFrame {
            map_id: "LakeOfRage".into(),
            grid_size: UVec2::new(columns, rows),
            tile_size: Vec2::splat(8.0),
            ..default()
        };
        frame.tiles = (0..rows)
            .flat_map(|row| (0..columns).map(move |column| (column, row)))
            .map(|(column, row)| VisualTile {
                column,
                row,
                source: VisualTileSource {
                    tileset_id: "johto".into(),
                    metatile_id: 0x35,
                    subtile_column: 0,
                    subtile_row: 0,
                    tile_index: if dry(column, row) { 0x05 } else { 0x14 },
                },
                texture: Handle::default(),
                animation_frames: None,
                priority: false,
            })
            .collect();
        frame
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn background_water_images_match_legacy_bytes_for_route_cave_and_phases() {
        fn same(a: Option<Image>, b: Option<Image>) {
            match (a,b) {
                (None,None) => {},
                (Some(a),Some(b)) => {
                    assert_eq!(a.data,b.data);
                    assert_eq!(format!("{:?}",a.texture_descriptor),format!("{:?}",b.texture_descriptor));
                    assert_eq!(format!("{:?}",a.sampler),format!("{:?}",b.sampler));
                    assert_eq!(a.asset_usage,b.asset_usage);
                },
                _ => panic!("prepared image presence differs"),
            }
        }
        let first = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(701),std::marker::PhantomData);
        let second = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(702),std::marker::PhantomData);
        let pool = bevy::tasks::TaskPoolBuilder::new().num_threads(2).build();
        for (map,tileset,columns,rows) in [("Route29","johto",116,114),
            ("LakeOfRage","johto",12,8),("UnionCave1F","cave",12,8),
            ("TohjoFalls","cave",12,8),("DragonsDenB1F","johto",4,4)] {
            for phase in 0..2 {
                let mut f = frame(columns,rows,|x,y| x==0 || y==0);
                f.map_id = map.into();
                for tile in &mut f.tiles {
                    tile.source.tileset_id = tileset.into();
                    tile.texture = if phase==0 {first.clone()} else {second.clone()};
                    tile.animation_frames = Some(vec![first.clone(),second.clone()].into());
                    if tileset=="cave" {tile.source.metatile_id=0x2c;}
                }
                if map=="DragonsDenB1F" {
                    for (i,tile) in f.tiles.iter_mut().enumerate() {
                        tile.source.metatile_id=0x07;
                        tile.source.subtile_column=(i%4) as u8;
                        tile.source.subtile_row=(i/4) as u8;
                        tile.source.tile_index=crate::cave_water::DEN_WHIRLPOOL_DRAWING[i];
                    }
                }
                let original = f.clone();
                let worker_frame = f.clone();
                let prepared = bevy::tasks::futures_lite::future::block_on(pool.spawn(async move {
                    PreparedImages::for_frame(&worker_frame)
                }));
                let mask = crate::cave_water::mask(&f);
                assert!(mask.is_some(),"water fixture {map}");
                let shore = mask.as_ref().and_then(|_|shoreline(&f));
                same(prepared.mask,mask); same(prepared.shore,shore);
                assert_eq!(f,original,"CPU preparation must preserve captured source frame");
            }
        }
        let dry = frame(4,4,|_,_|true);
        let prepared = PreparedImages::for_frame(&dry);
        assert!(prepared.mask.is_none() && prepared.shore.is_none(),"dry None must not synthesize shoreline");
    }

    fn depth(image: &Image, x: usize, y: usize) -> f32 {
        let byte = image.data.as_ref().unwrap()[(y * image.width() as usize + x) * 4];
        (byte as f32 / 255.0 * 2.0 - 1.0) * REACH
    }

    #[test]
    fn bank_distance_resolves_shallows_and_islands_without_frame_edge_foam() {
        let frame = frame(12, 8, |x, y| x == 0 || (x == 5 && y == 4));
        let original = frame.clone();
        let image = shoreline(&frame).unwrap();
        assert!(image.data.as_ref().unwrap().chunks_exact(4).all(|p|p[0]==p[1]),
            "ordinary banks retain their original contact and depth field");
        assert!(depth(&image, 3, 10) < 0.0);
        assert!((depth(&image, 4, 10) - 0.125).abs() < 0.04);
        assert!(depth(&image, 12, 10) > 1.0);
        assert!(
            depth(&image, 19, 18) < 0.2,
            "an island contributes its own bank"
        );
        assert!(depth(&image, 47, 0) > 6.0, "the cropped open edge is deep");
        assert_eq!(frame, original, "optics cannot change authored data");
    }

    #[test]
    fn model_contact_distance_uses_the_outer_outline_without_internal_triangle_seams() {
        let triangles=[[Vec2::ZERO,Vec2::X,Vec2::ONE],[Vec2::ZERO,Vec2::ONE,Vec2::Y]];
        let boundary=contact_boundary(&triangles);
        assert_eq!(boundary.len(),4,"the triangulation diagonal is not a shoreline");
        for (point,expected) in [(Vec2::splat(0.5),-0.5),(Vec2::new(0.25,0.75),-0.25),
            (Vec2::new(1.,0.5),0.),(Vec2::new(1.5,0.5),0.5),
            (Vec2::splat(1.5),std::f32::consts::FRAC_1_SQRT_2)] {
            assert!((contact_distance(point,&triangles,&boundary)-expected).abs()<0.000001);
        }
    }

    #[test]
    fn complete_kanto_sea_rocks_use_round_model_contacts_instead_of_square_banks() {
        let mut f=frame(8,6,|_,_|false);
        f.map_id="CinnabarIsland".into();
        f.grid_origin=IVec2::new(12,24);
        for tile in &mut f.tiles {
            tile.source.tileset_id="kanto".into();
            tile.source.metatile_id=0x43;
        }
        for y in 0..2 {for x in 0..2 {
            let s=&mut f.tiles[(y+2)*8+x+2].source;
            s.metatile_id=0x14;s.subtile_column=x as u8;s.subtile_row=y as u8;
            s.tile_index=[[0x2a,0x2b],[0x3a,0x3b]][y][x];
        }}
        let original=f.clone();
        let rounded=shoreline(&f).unwrap();
        assert!(rounded.data.as_ref().unwrap().chunks_exact(4).all(|p|p[1]>250),
            "isolated sea rocks cannot turn open sea into shallow land plots");
        let triangles=crate::exterior_models::kanto_shore_contact([14,26]);
        let boundary=contact_boundary(&triangles);
        assert!(!boundary.is_empty());
        for y in 0..8 {for x in 0..8 {
            let point=Vec2::new((x as f32+0.5)/SAMPLES as f32,(y as f32+0.5)/SAMPLES as f32);
            let expected=contact_distance(point,&triangles,&boundary);
            assert!((depth(&rounded,8+x,8+y)-expected).abs()<0.033,
                "model contour distance survives the retained texture's quantization at {x},{y}");
        }}
        assert!(depth(&rounded,8,8)>0.1,"the source drawing's square corner remains water");
        assert!(depth(&rounded,15,8)>0.1,"the other corner must not create a straight foam shelf");
        assert!(depth(&rounded,12,12)<0.,"the actual seated rock still contributes a bank");
        assert_eq!(f,original,"contact preparation cannot change scenery or footing");
        let mut coast=f.clone();
        for tile in &mut coast.tiles {if tile.column>=6 {
            tile.source.metatile_id=1;tile.source.tile_index=0x23;
        }}
        let coastal=shoreline(&coast).unwrap();
        let pixels=coastal.data.as_ref().unwrap();
        let rock=(12*coastal.width() as usize+12)*4;
        let quay=(12*coastal.width() as usize+6*SAMPLES)*4;
        assert!(pixels[rock]<128 && pixels[rock+1]>155,
            "rock contact remains grounded while sea depth follows the real coast");
        assert!(pixels[quay]<128 && pixels[quay+1]<128,"actual land still owns the shallow margin");
        let mut crop=f.clone();
        crop.grid_origin+=IVec2::ONE;crop.grid_size=UVec2::new(6,4);
        crop.tiles.retain(|t|(1..7).contains(&t.column)&&(1..5).contains(&t.row));
        for tile in &mut crop.tiles {tile.column-=1;tile.row-=1;}
        let cropped=shoreline(&crop).unwrap();
        for y in 0..cropped.height() as usize {for x in 0..cropped.width() as usize {
            assert_eq!(depth(&rounded,x+SAMPLES,y+SAMPLES),depth(&cropped,x,y),
                "the same world-seeded rock contact must survive a streamed crop");
            let original=((y+SAMPLES)*rounded.width() as usize+x+SAMPLES)*4;
            let current=(y*cropped.width() as usize+x)*4;
            assert_eq!(rounded.data.as_ref().unwrap()[original+1],cropped.data.as_ref().unwrap()[current+1],
                "streaming also preserves the real-land depth field");
        }}
        f.tiles[3*8+3].source.tile_index^=1;
        assert!(depth(&shoreline(&f).unwrap(),8,8)<0.,"altered drawings keep their source baseline");
        for y in 0..2 {for x in 0..2 {
            let s=&mut f.tiles[(y+2)*8+x+2].source;
            s.metatile_id=0x4d;s.subtile_column=2+x as u8;
            s.tile_index=[[0x2a,0x2b],[0x3a,0x3b]][y][x];
        }}
        assert!(depth(&shoreline(&f).unwrap(),8,8)<0.,"land rocks do not acquire a water backing");
    }

    #[test]
    fn complete_harbor_posts_keep_open_water_optics_and_real_quay_contacts() {
        for map in ["OlivinePort","VermilionPort"] {
            let mut f=frame(8,8,|x,_|x>=6);f.map_id=map.into();
            for t in &mut f.tiles {
                t.source.tileset_id="port".into();
                t.source.metatile_id=if t.column>=6 {0x11}else{0x0a};
                t.source.tile_index=if t.column>=6 {0x31}else{0x14};
            }
            let open=shoreline(&f).unwrap().data;
            for y in 0..2 {for x in 0..2 {
                let s=&mut f.tiles[(y+2)*8+x+2].source;
                s.metatile_id=0x01;s.subtile_column=x as u8;s.subtile_row=y as u8;
                s.tile_index=[[0x01,0x02],[0x11,0x12]][y][x];
            }}
            let original=f.clone();let image=shoreline(&f).unwrap();
            assert_eq!(image.data,open,"{map}: posts cannot add a rectangular beach");
            assert_eq!(f,original,"optics cannot alter geometry, source art or actor support");
            assert!(depth(&image,6*SAMPLES+1,3*SAMPLES)<0.,"the actual quay remains dry");
            let mut crop=f.clone();crop.grid_origin+=IVec2::ONE;crop.grid_size=UVec2::splat(6);
            crop.tiles.retain(|t|(1..7).contains(&t.column)&&(1..7).contains(&t.row));
            for t in &mut crop.tiles {t.column-=1;t.row-=1;}
            let cropped=shoreline(&crop).unwrap();
            for y in 0..24 {for x in 0..24 {
                assert_eq!(depth(&image,x+SAMPLES,y+SAMPLES),depth(&cropped,x,y),"streaming keeps the same shoreline");
            }}
            f.tiles[2*8+2].source.tile_index=3;
            assert!(depth(&shoreline(&f).unwrap(),2*SAMPLES+1,2*SAMPLES+1)<0.,"incomplete artwork keeps the native baseline");
            let mut ship=original.clone();for y in 2..4 {for x in 2..4 {ship.tiles[y*8+x].source.metatile_id=0x18;}}
            assert!(depth(&shoreline(&ship).unwrap(),2*SAMPLES+1,2*SAMPLES+1)<0.,"ship geometry is not an open-water barrier");
            let mut pool=original;pool.map_id="CeruleanGym".into();
            assert!(depth(&shoreline(&pool).unwrap(),2*SAMPLES+1,2*SAMPLES+1)<0.,"harbor treatment does not change the gym pool");
        }
    }

    #[test]
    fn complete_ferry_keeps_continuous_sea_without_consuming_the_real_quay() {
        for map in ["OlivinePort", "VermilionPort"] {
            let mut f = frame(20, 12, |_, y| y == 0); f.map_id = map.into();
            for tile in &mut f.tiles {
                tile.source.tileset_id = "port".into();
                tile.source.metatile_id = if tile.row == 0 { 0x11 } else { 0x0a };
                tile.source.tile_index = if tile.row == 0 { 0x31 } else { 0x14 };
            }
            let sea = shoreline(&f).unwrap();
            for y in 0..8 { for x in 0..16 {
                let source = &mut f.tiles[(y + 2) * 20 + x + 2].source;
                source.metatile_id = 0x18 + (y / 4 * 4 + x / 4) as u16;
                source.subtile_column = (x % 4) as u8; source.subtile_row = (y % 4) as u8;
                source.tile_index = crate::port::ferry_anchor(x, y).unwrap_or(0x14);
            }}
            let original = f.clone();
            let fitted = shoreline(&f).unwrap();
            assert_eq!(fitted.data, sea.data, "{map}: a ship cannot create a rectangular island");
            assert_eq!(f, original, "shore preparation cannot alter art, water height or support");
            assert!(depth(&fitted, 10 * SAMPLES, 1) < 0., "the real quay remains dry");
            for (x, y) in [(6, 0), (12, 1), (0, 2), (7, 3), (8, 4), (12, 6)] {
                let mut bad = original.clone(); bad.tiles[(y + 2) * 20 + x + 2].source.tile_index ^= 0x80;
                assert_ne!(shoreline(&bad).unwrap().data, sea.data, "changed anchor keeps the native baseline");
            }
            for fault in 0..4 {
                let mut bad = original.clone(); let source = &mut bad.tiles[3 * 20 + 3].source;
                match fault {
                    0 => source.tileset_id = "johto".into(),
                    1 => source.metatile_id = 0x1f,
                    2 => source.subtile_column = 3,
                    _ => source.subtile_row = 3,
                }
                assert_ne!(shoreline(&bad).unwrap().data, sea.data);
            }
            let mut room = original.clone(); room.map_id = "CeruleanGym".into();
            assert_ne!(shoreline(&room).unwrap().data, sea.data, "the gym cannot inherit harbor treatment");
            let mut clipped = original; clipped.tiles.retain(|t| t.column != 2 || t.row != 2);
            assert_ne!(shoreline(&clipped).unwrap().data, sea.data, "missing source retains fallback");
        }
    }

    #[test]
    fn floating_buoys_do_not_create_square_shorelines() {
        for tileset in ["johto", "johto_modern"] {
            let mut f = frame(4, 4, |_, _| false);
            let open_water = shoreline(&f).unwrap().data;
            f.tiles[5].source.tileset_id = tileset.into();
            f.tiles[5].source.metatile_id = 0x34;
            f.tiles[5].source.tile_index = 0x58;
            let original = f.clone();
            assert_eq!(shoreline(&f).unwrap().data, open_water,
                "a floating source buoy is surrounded by deep water");
            assert_eq!(f, original, "optics retain source scenery and footing");
            f.tiles[5].source.metatile_id = 0x01;
            assert!(depth(&shoreline(&f).unwrap(), 5, 5) < 0.,
                "the same byte outside the course still contributes a bank");
        }
    }

    #[test]
    fn raised_piers_keep_water_continuous_but_soil_islands_stay_dry() {
        for (map, tileset, block, tile) in [("Route20","kanto",5,0x4b),
            ("Route32","johto",6,7),("CeruleanGym","port",0x26,5)] {
            let mut f=frame(8,8,|_,_|false);
            f.map_id=map.into();
            let open=shoreline(&f).unwrap().data;
            for y in 0..4 {for x in 0..4 {
                let s=&mut f.tiles[(y+2)*8+x+2].source;
                s.tileset_id=tileset.into();s.metatile_id=block;s.tile_index=tile;
                s.subtile_column=x as u8;s.subtile_row=y as u8;
            }}
            let (face_block,face_tile,face_row)=match tileset {
                "kanto"=>(0x78,0x31,0),"johto"=>(0x43,0x49,0),_=>(0x08,0x22,0),
            };
            // Port08 has a full-width timber face; Port14 owns only its
            // left half, so repeating it across four columns is altered art.
            for x in 0..4 {
                let s=&mut f.tiles[6*8+x+2].source;
                s.tileset_id=tileset.into();s.metatile_id=face_block;s.tile_index=face_tile;
                s.subtile_column=x as u8;s.subtile_row=face_row;
            }
            let original=f.clone();
            assert_eq!(shoreline(&f).unwrap().data,open,"{map}: raised deck is not a shoreline");
            assert_eq!(f,original,"{map}: preserve source art and geometry inputs");
            f.tiles[3*8+3].source.tile_index=0xff;
            assert!(depth(&shoreline(&f).unwrap(),3*SAMPLES+1,3*SAMPLES+1)<0.,
                "an altered drawing retains its dry source baseline");
            f.tiles[5*8+4].source.tile_index=0x14;
            assert!(depth(&shoreline(&f).unwrap(),4*SAMPLES+1,6*SAMPLES+1)<0.,
                "a face without the matching deck above keeps its source baseline");
        }
        let mut garden=frame(8,8,|_,_|false);garden.map_id="Route12".into();
        let source=&mut garden.tiles[3*8+3].source;
        source.tileset_id="kanto".into();source.metatile_id=0x74;
        source.subtile_column=0;source.subtile_row=0;source.tile_index=0x2c;
        assert!(depth(&shoreline(&garden).unwrap(),3*SAMPLES+1,3*SAMPLES+1)<0.,
            "a supported soil island still has a real shoreline");
    }

    #[test]
    fn shore_registration_survives_scrolling_and_animation_updates() {
        let mut large = frame(16, 12, |x, _| x == 2);
        large.grid_origin = IVec2::new(20, 30);
        let original = shoreline(&large).unwrap().data;
        large.center = Vec2::new(27.0, -19.0);
        large.tiles[3].animation_frames = Some(vec![Handle::default(), Handle::default()].into());
        assert_eq!(shoreline(&large).unwrap().data, original);
        let mut crop = frame(12, 8, |x, _| x == 0);
        crop.grid_origin = IVec2::new(22, 32);
        let a = shoreline(&large).unwrap();
        let b = shoreline(&crop).unwrap();
        for y in 0..b.height() as usize {
            for x in 0..b.width() as usize {
                assert_eq!(depth(&a, x + 8, y + 8), depth(&b, x, y));
            }
        }
        assert_eq!(bounds(&crop), Vec4::new(22.0, 32.0, 12.0, 8.0));
    }

    #[test]
    fn missing_cells_do_not_invent_banks_and_bad_grids_are_rejected() {
        let mut frame = frame(4, 4, |_, _| false);
        frame.tiles.pop();
        let image = shoreline(&frame).unwrap();
        assert!(
            image
                .data
                .as_ref()
                .unwrap()
                .chunks_exact(4)
                .all(|p| p[0] >= 252)
        );
        frame.tiles.push(frame.tiles[0].clone());
        assert!(shoreline(&frame).is_none());
        frame.tiles.pop();
        frame.tiles[0].column = 4;
        assert!(shoreline(&frame).is_none());
        assert!(shoreline(&VisualWorldFrame::default()).is_none());
    }

    #[test]
    fn sheltered_forest_optics_require_both_map_and_source_family() {
        let mut frame = frame(1, 1, |_, _| false);
        frame.map_id = "IlexForest".into();
        let ordinary = style(&frame);
        frame.tiles[0].source.tileset_id = "forest".into();
        let original = frame.clone();
        let sheltered = style(&frame);
        assert!(sheltered.x < ordinary.x && sheltered.y < ordinary.y);
        assert!(sheltered.z < ordinary.z && sheltered.w < ordinary.w);
        assert_eq!(frame, original);
        frame.map_id = "UnionCave1F".into();
        assert_eq!(style(&frame).w, 0.);
    }

    #[test]
    fn enclosed_and_coastal_water_have_distinct_optics() {
        let mut frame = frame(1, 1, |_, _| false);
        let lake = style(&frame);
        frame.map_id = "Route40".into();
        let sea = style(&frame);
        assert!(sea.x < lake.x && sea.y > lake.y && sea.z > lake.z);
        for map in ["VermilionCity", "VermilionPort", "CinnabarIsland", "PalletTown",
            "Route12", "Route13", "Route19", "Route20", "Route21"] {
            frame.map_id = map.into();
            assert_eq!(style(&frame), sea, "exposed Kanto coast: {map}");
        }
        for map in ["CeruleanCity", "CeladonCity", "Route6", "MountMoonSquare"] {
            frame.map_id = map.into();
            assert_eq!(style(&frame), lake, "sheltered Kanto water: {map}");
        }
        frame.map_id = "UnionCaveB1F".into();
        assert_eq!(style(&frame).w, 0.0, "caves cannot reflect an exposed sky");
        frame.map_id = "CeruleanGym".into();
        assert!(style(&frame).y < lake.y);
        assert_eq!(style(&frame).z, 0.0, "indoor pools have no coastal foam");
    }

    #[test]
    fn presentation_blend_is_bounded_and_handles_the_renderer_clock_wrap() {
        let previous = Vec4::new(3599.95, 0.10, 2.0, 0.0);
        let wrapped = blend_clock(previous, 0.05, Vec2::new(2.0, 0.0));
        assert!((wrapped.y - 0.10).abs() < 0.002);
        let paused = blend_clock(wrapped, 10.0, Vec2::ZERO);
        assert_eq!(paused.y, 0.25);
        assert_eq!((paused.z, paused.w), (0.0, 0.0));
    }

    #[test]
    fn slow_motion_crosses_source_wraps_without_reset_and_holds_when_paused() {
        let step = Vec2::new(std::f32::consts::TAU / 3., 0.);
        let mut motion = Vec4::ZERO;
        for _ in 0..150 {
            let next = advance_motion(motion, step);
            assert!((next.x-next.y-step.x*0.03).abs() < 0.00001);
            assert!((next.y-motion.x).rem_euclid(std::f32::consts::TAU).min(
                (motion.x-next.y).rem_euclid(std::f32::consts::TAU)) < 0.00001);
            motion=next;
        }
        let paused=advance_motion(motion,Vec2::ZERO);
        assert_eq!(paused.x,paused.y);
        assert_eq!(paused.x,motion.x);
    }

    #[test]
    fn swimmer_contact_tracks_the_source_foot_and_clears_on_land() {
        let mut frame = frame(4, 4, |x, _| x < 2);
        let second = Handle::Uuid(
            bevy::asset::uuid::Uuid::from_u128(8500),
            std::marker::PhantomData,
        );
        for tile in &mut frame.tiles {
            tile.animation_frames = Some(vec![tile.texture.clone(), second.clone()].into());
        }
        frame.actors.push(crystal_render_api::VisualActor {
            id: crystal_render_api::VisualActorId::Player,
            source_id: "player:surf".into(),
            model_palette: None,
            texture: Handle::default(),
            center: Vec2::new(12.0, 4.0),
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
            facing: None,
        });
        let initial = contact(&frame);
        assert_eq!(initial, Vec4::new(3.5, 1.5, 1.1, 1.0));
        frame.center += Vec2::new(16.0, -8.0);
        frame.actors[0].center += Vec2::new(16.0, -8.0);
        assert_eq!(
            contact(&frame),
            initial,
            "camera motion must not drag the ripple"
        );
        frame.actors[0].center.x -= 24.0;
        assert_eq!(
            contact(&frame),
            Vec4::ZERO,
            "dry ground has no swimmer ring"
        );
        frame.actors[0].center.x += 24.0;
        frame.tiles[7].animation_frames = None;
        assert_eq!(
            contact(&frame),
            Vec4::ZERO,
            "unavailable source art cannot invent water contact"
        );
    }
}
