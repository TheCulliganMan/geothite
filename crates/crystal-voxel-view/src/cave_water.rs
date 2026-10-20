//! Presentation mask for proven live water. UVs, animation frames and
//! source pixels remain untouched; the surface shader colors only these slots.
use bevy::asset::RenderAssetUsages;
use bevy::{
    prelude::*,
    render::{
        
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use crystal_render_api::{VisualTile, VisualTileSource, VisualWorldFrame};

/// Source water identity shared by optical presentation and terrain height
/// ownership. Cave pools deliberately use a flat cave-floor profile, so the
/// geometric `Water` shape alone cannot identify every standing surface.
pub(crate) fn standing_water_source(source: &VisualTileSource) -> bool {
    source.subtile_column < 4 && source.subtile_row < 4
        && (crate::profile::shape_for_source(source) == crate::profile::CellShape::Water
            || (matches!(source.tileset_id.as_ref(), "cave" | "dark_cave") && source.tile_index == 0x14))
}

pub(crate) const DEN_WHIRLPOOL_DRAWING: [u16;16] = [
    0x32,0x33,0x58,0x58,0x42,0x43,0x58,0x58,
    0x14,0x14,0x58,0x58,0x14,0x14,0x58,0x58,
];

/// Complete source drawings and a genuine local water animation prove both
/// the whirlpool footprint and the clock used to present its currents.
pub(crate) fn den_whirlpool_cells(cells: &[&VisualTile], width: usize, height: usize) -> Vec<(usize,usize)> {
    if cells.len()!=width.saturating_mul(height) {return Vec::new();}
    let mut result=Vec::new();
    for row in 0..height.saturating_sub(3) {for col in 0..width.saturating_sub(3) {
        if !(0..16).all(|j| {
            let s=&cells[(row+j/4)*width+col+j%4].source;
            s.tileset_id.as_ref()=="johto" && s.metatile_id==0x07
                && s.subtile_column as usize==j%4 && s.subtile_row as usize==j/4
                && s.tile_index==DEN_WHIRLPOOL_DRAWING[j]
        }) {continue;}
        let Some(water)=[8,9,12,13].into_iter().map(|j|(row+j/4)*width+col+j%4)
            .find(|&i|role_for_map(cells[i],"DragonsDenB1F")==[255,0]) else {continue;};
        result.extend([0,1,4,5].into_iter().map(|j|((row+j/4)*width+col+j%4,water)));
    }}
    result
}

fn den_whirlpool_water(frame: &VisualWorldFrame) -> std::collections::HashMap<(u32,u32),&VisualTile> {
    if frame.map_id.as_ref()!="DragonsDenB1F" && !outdoor(frame.map_id.as_ref()) {
        return Default::default();
    }
    let (width,height)=(frame.grid_size.x as usize,frame.grid_size.y as usize);
    let mut ordered=vec![None;width.saturating_mul(height)];
    for tile in &frame.tiles {
        if tile.column as usize>=width || tile.row as usize>=height {return Default::default();}
        if ordered[tile.row as usize*width+tile.column as usize].replace(tile).is_some() {return Default::default();}
    }
    let Some(cells)=ordered.into_iter().collect::<Option<Vec<_>>>() else {return Default::default();};
    den_whirlpool_cells(&cells,width,height).into_iter().map(|(i,w)|
        ((cells[i].column,cells[i].row),cells[w])).collect()
}

fn role(tile: &VisualTile) -> [u8; 2] { role_for_map(tile, "") }

/// Maps whose standing water shares the outdoor shoreline rules.
pub fn outdoor(map: &str) -> bool {
    map.starts_with("Route") || map.ends_with("City") || map.ends_with("Town")
        || matches!(map,"LakeOfRage"|"IlexForest"|"NationalPark"|"NationalParkBugContest"|"BattleTowerOutside"
            |"RuinsOfAlphOutside"|"SilverCaveOutside"|"OlivinePort"|"VermilionPort"
            |"CinnabarIsland"|"SafariZoneBeta"|"MountMoonSquare")
}

pub(super) fn role_for_map(tile: &VisualTile, map: &str) -> [u8; 2] {
    let s = &tile.source;
    let den = map == "DragonsDenB1F" && s.tileset_id.as_ref() == "johto";
    let open_water = (outdoor(map) && matches!(s.tileset_id.as_ref(),
        "johto"|"johto_modern"|"kanto"|"forest"|"park"|"battle_tower_outside"|"port"))
        || (map == "CeruleanGym" && s.tileset_id.as_ref() == "port");
    if !(matches!(s.tileset_id.as_ref(), "cave" | "dark_cave") || den || open_water)
        || s.subtile_column >= 4
        || s.subtile_row >= 4
        || !tile
            .animation_frames
            .as_ref()
            .is_some_and(|frames| frames.len() > 1 && frames.contains(&tile.texture))
    {
        return [0, 0];
    }
    if open_water {
        return if standing_water_source(s) {
            [255,0]
        } else { [0,0] };
    }
    match s.tile_index {
        0x14 => [255, 0],
        0x40 if !den && s.metatile_id == 0x2c => [0, 255],
        _ => [0, 0],
    }
}

pub(crate) fn live_outdoor_water(tile: &VisualTile, map: &str) -> bool {
    outdoor(map) && role_for_map(tile,map) == [255,0]
}

pub(crate) fn live_pool_water(tile: &VisualTile, map: &str) -> bool {
    live_outdoor_water(tile, map)
        || (map == "CeruleanGym" && role_for_map(tile,map) == [255,0])
}

fn phase(tile: &VisualTile) -> u8 {
    let Some(frames) = tile.animation_frames.as_ref() else {
        return 0;
    };
    let Some(index) = frames.iter().position(|frame| frame == &tile.texture) else {
        return 0;
    };
    ((index * 255) / frames.len().max(1)) as u8
}

/// Cave water uses its own live source paint rather than the generic teal
/// surface. Foreign outdoor frames keep their existing regional registration.
pub(super) fn native_cave_source(frame: &VisualWorldFrame) -> bool {
    !outdoor(&frame.map_id) && frame.tiles.first().is_some_and(|tile|
        matches!(tile.source.tileset_id.as_ref(), "cave" | "dark_cave"))
}

pub(super) fn registration(frame: &VisualWorldFrame) -> Vec4 {
    Vec4::new(
        if frame.map_id.as_ref()=="TohjoFalls" || native_cave_source(frame) { 3. }
            else if outdoor(&frame.map_id) { 2. } else { 1. },
        frame.tile_size.x,
        (frame.grid_origin.x as f32 + frame.grid_size.x as f32 * 0.5) * frame.tile_size.x
            - frame.center.x,
        (frame.grid_origin.y as f32 + frame.grid_size.y as f32 * 0.5) * frame.tile_size.y
            + frame.center.y,
    )
}

pub(super) fn retained_registration(built: &VisualWorldFrame, live: &VisualWorldFrame) -> Vec4 {
    let mut r = registration(built);
    let offset = crate::terrain_tracking::offset(built, live);
    r.z -= live.center.x - built.center.x + offset.x;
    r.w -= built.center.y - live.center.y + offset.y;
    r
}

/// Preserve original map-space ripple and shoreline lookups when the exact
/// source mesh is uniformly rebased into an encounter's smaller world units.
pub(super) fn encounter_registration(frame: &VisualWorldFrame, translation: Vec3, scale: f32) -> Vec4 {
    let source = registration(frame);
    let local_offset = Vec2::new(source.z + frame.center.x, source.w - frame.center.y);
    let offset = local_offset * scale - Vec2::new(translation.x, translation.z);
    Vec4::new(source.x, source.y * scale, offset.x, offset.y)
}

// Only complete, authenticated cascades supply a downstream foam footprint.
// The mask's alpha channel is presentation data, never surface transparency.
struct CascadeImpact {
    fall: crate::waterfall::WaterfallPlacement,
    pool: std::collections::HashSet<(usize,usize)>,
}

fn receiving_falls(frame: &VisualWorldFrame) -> Vec<CascadeImpact> {
    let width = frame.grid_size.x as usize;
    let height = frame.grid_size.y as usize;
    let mut ordered = vec![None; width.saturating_mul(height)];
    for tile in &frame.tiles {
        if tile.column >= frame.grid_size.x || tile.row >= frame.grid_size.y { return Vec::new(); }
        if ordered[tile.row as usize * width + tile.column as usize].replace(tile).is_some() { return Vec::new(); }
    }
    let Some(cells) = ordered.into_iter().collect::<Option<Vec<_>>>() else { return Vec::new(); };
    crate::waterfall::waterfall_placements(&cells, width, height).into_iter().filter(|fall|
        (fall.row..fall.row+fall.height).all(|row|
            (fall.column..fall.column+fall.width).all(|column|
                role(cells[row*width+column]) == [0,255]))).map(|fall| {
        let foot=fall.row+fall.height;
        let mut pool=std::collections::HashSet::new();
        let mut queue=std::collections::VecDeque::new();
        if foot<height {
            for column in fall.column..fall.column+fall.width {
                if role(cells[foot*width+column])==[255,0] {
                    pool.insert((column,foot)); queue.push_back((column,foot));
                }
            }
        }
        while let Some((column,row))=queue.pop_front() {
            for (x,z) in [(column.wrapping_sub(1),row),(column+1,row),
                (column,row.wrapping_sub(1)),(column,row+1)] {
                if x>=width || z>=height || z<foot || z>foot+2
                    || x+2<fall.column || x>fall.column+fall.width+2
                    || role(cells[z*width+x])!=[255,0] { continue; }
                if pool.insert((x,z)) { queue.push_back((x,z)); }
            }
        }
        CascadeImpact { fall,pool }
    }).collect()
}

fn impact(falls: &[CascadeImpact], x: f32, z: f32, pool: bool) -> u8 {
    let mut strongest = 0_f32;
    for cascade in falls {
        let fall=&cascade.fall;
        if pool {
            if !cascade.pool.contains(&(x.floor() as usize,z.floor() as usize)) { continue; }
        } else if x<fall.column as f32 || x>=(fall.column+fall.width) as f32
            || z<fall.row as f32 || z>=(fall.row+fall.height) as f32 { continue; }
        let downstream = z - (fall.row + fall.height) as f32;
        // The last quarter of the sheet supplies the interpolation sample at
        // the join. Foam never spreads into the upstream pool above the head.
        if downstream < -0.25 { continue; }
        let side = (fall.column as f32 - x).max(0.)
            .max(x - (fall.column + fall.width) as f32);
        let distance = side.hypot(downstream.max(0.));
        let t = (1. - distance / 1.5).clamp(0., 1.);
        strongest = strongest.max(t*t*(3.-2.*t));
    }
    ((1. - strongest) * 255.).round() as u8
}

pub(super) fn mask(frame: &VisualWorldFrame) -> Option<Image> {
    if !frame.tiles.iter().any(|t| role_for_map(t, &frame.map_id) != [0, 0]) {
        return None;
    }
    let width = frame.grid_size.x.checked_mul(8)?;
    let height = frame.grid_size.y.checked_mul(8)?;
    if width == 0 || height == 0 {
        return None;
    }
    let mut pixels = vec![0; width as usize * height as usize * 4];
    for pixel in pixels.chunks_exact_mut(4) { pixel[3] = 255; }
    let falls = receiving_falls(frame);
    let whirlpools = den_whirlpool_water(frame);
    for tile in &frame.tiles {
        if tile.column >= frame.grid_size.x || tile.row >= frame.grid_size.y {
            return None;
        }
        let whirlpool = whirlpools.get(&(tile.column,tile.row));
        let [pool, fall] = if whirlpool.is_some() {[255,128]} else {role_for_map(tile, &frame.map_id)};
        let live_phase = whirlpool.map_or_else(||phase(tile), |water|phase(water));
        for y in 0..8 {
            for x in 0..8 {
                let i = (((tile.row * 8 + y) * width + tile.column * 8 + x) * 4) as usize;
                pixels[i..i + 4].copy_from_slice(&[
                    pool,
                    fall,
                    if pool > 0 || fall > 0 { live_phase } else { 0 },
                    if pool > 0 || fall > 0 {
                        impact(&falls, tile.column as f32 + (x as f32 + 0.5)/8.,
                            tile.row as f32 + (y as f32 + 0.5)/8., pool>0)
                    } else { 255 },
                ]);
            }
        }
    }
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    // Phase updates change bytes, not the texture bound by every terrain group.
    crystal_render_api::stream_composed_image(&mut image);
    Some(image)
}

/// Encode only phases accepted by the retained source atlas. Geometry, source
/// assets stay intact; role channels keep requiring the authenticated family.
/// Unchanged frames do not mark the image dirty or upload it again.
#[cfg(test)]
fn refresh_mask_phases(
    built: &VisualWorldFrame,
    handle: &Handle<Image>,
    images: &mut Assets<Image>,
) -> bool {
    refresh_mask_with_steps(built, handle, images).0
}

/// Alongside the exact source pixels, report a blend only when every slot of
/// that role advances by the same single authenticated frame. Mixed local
/// clocks, skipped frames and changed source roles must settle immediately.
pub(super) fn refresh_mask_with_steps(
    built: &VisualWorldFrame,
    handle: &Handle<Image>,
    images: &mut Assets<Image>,
) -> (bool, Vec2) {
    let Some(mask) = images.get(handle) else {
        return (false, Vec2::ZERO);
    };
    if mask.width() != built.grid_size.x * 8 || mask.height() != built.grid_size.y * 8 {
        return (false, Vec2::ZERO);
    }
    let width = mask.width() as usize;
    let mut updates = Vec::new();
    let mut roles_changed = false;
    let mut steps: [Option<f32>; 2] = [None, None];
    let mut invalid_steps = [false; 2];
    let whirlpools = den_whirlpool_water(built);
    for tile in &built.tiles {
        if tile.column >= built.grid_size.x || tile.row >= built.grid_size.y {
            continue;
        }
        let whirlpool = whirlpools.get(&(tile.column,tile.row));
        let [pool, fall] = if whirlpool.is_some() {[255,128]} else {role_for_map(tile, &built.map_id)};
        let next = [
            pool,
            fall,
            if pool > 0 || fall > 0 { whirlpool.map_or_else(||phase(tile), |water|phase(water)) } else { 0 },
        ];
        let first = (tile.row as usize * 8 * width + tile.column as usize * 8) * 4;
        let previous = &mask.data.as_ref().expect("CPU image pixels")[first..first + 3];
        if pool > 0 || fall == 255 {
            let family = whirlpool.copied().unwrap_or(tile).animation_frames.as_ref();
            let count = family.map_or(0, |frames| frames.len());
            let delta = ((f32::from(next[2]) - f32::from(previous[2])) / 255.0).rem_euclid(1.0);
            let single_step = if count > 1 && previous[..2] == next[..2]
                && (delta - 1.0 / count as f32).abs() < 1.5 / 255.0 {
                delta * std::f32::consts::TAU
            } else { 0.0 };
            let role = usize::from(fall == 255);
            invalid_steps[role] |= single_step == 0.0
                || steps[role].is_some_and(|step| (step - single_step).abs() >= 0.04);
            steps[role] = Some(match steps[role] {
                None => single_step,
                Some(step) if (step - single_step).abs() < 0.04 => step,
                Some(_) => 0.0,
            });
        }
        if mask.data.as_ref().expect("CPU image pixels")[first..first + 3] != next {
            roles_changed |= mask.data.as_ref().expect("CPU image pixels")[first..first + 2] != next[..2];
            updates.push((tile.column as usize, tile.row as usize, next));
        }
    }
    if updates.is_empty() {
        return (false, Vec2::ZERO);
    }
    if roles_changed {
        if let Some(rebuilt) = self::mask(built) {
            images.get_mut(handle).unwrap().data = rebuilt.data;
            return (true, Vec2::ZERO);
        }
    }
    let mut mask = images.get_mut(handle).unwrap();
    if roles_changed {
        for pixel in mask.data.as_mut().expect("CPU image pixels").chunks_exact_mut(4) { pixel[3] = 255; }
    }
    for (column, row, next) in updates {
        for y in 0..8 {
            for x in 0..8 {
                let i = ((row * 8 + y) * width + column * 8 + x) * 4;
                mask.data.as_mut().expect("CPU image pixels")[i..i + 3].copy_from_slice(&next);
            }
        }
    }
    (true, if roles_changed { Vec2::ZERO } else {
        Vec2::new(if invalid_steps[0] { 0.0 } else { steps[0].unwrap_or(0.0) },
            if invalid_steps[1] { 0.0 } else { steps[1].unwrap_or(0.0) })
    })
}

/// Copy authenticated live pixels into their retained atlas slots. Never infer
/// a phase from elapsed renderer time or mutate authored texture assets.
pub(super) fn refresh_phases(
    built: &mut VisualWorldFrame,
    live: &crystal_render_api::VisualTileAnimationFrame,
    images: &mut Assets<Image>,
) {
    if built.map_id != live.map_id {
        return;
    }
    let delta = live.grid_origin - built.grid_origin;
    let width = built.grid_size.x as usize;
    let mut updates = Vec::new();
    for tile in &live.tiles {
        if role_for_map(tile, &live.map_id) == [0, 0] {
            continue;
        }
        let slot = IVec2::new(tile.column as i32, tile.row as i32) + delta;
        if slot.x < 0
            || slot.y < 0
            || slot.x >= built.grid_size.x as i32
            || slot.y >= built.grid_size.y as i32
        {
            continue;
        }
        let index = slot.y as usize * width + slot.x as usize;
        let Some(old) = built.tiles.get(index) else {
            continue;
        };
        if old.source != tile.source
            || old.animation_frames != tile.animation_frames
            || old.texture == tile.texture
        {
            continue;
        }
        let Some(source) = images.get(&tile.texture) else {
            continue;
        };
        if source.width() == 8 && source.height() == 8 && source.data.as_ref().expect("CPU image pixels").len() == 256 {
            updates.push((index, tile.texture.clone(), source.data.as_ref().expect("CPU image pixels").clone()));
        }
    }
    if updates.is_empty() {
        return;
    }
    let Some(mut atlas) = images.get_mut(&built.map_texture) else {
        return;
    };
    if atlas.width() != built.grid_size.x * 8 || atlas.height() != built.grid_size.y * 8 {
        return;
    }
    for (index, texture, pixels) in updates {
        let x = index % width * 8;
        let y = index / width * 8;
        for row in 0..8 {
            let start = ((y + row) * width * 8 + x) * 4;
            atlas.data.as_mut().expect("CPU image pixels")[start..start + 32].copy_from_slice(&pixels[row * 32..row * 32 + 32]);
        }
        built.tiles[index].texture = texture;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::VisualTileSource;
    fn tile(index: u16) -> VisualTile {
        VisualTile {
            column: 0,
            row: 0,
            source: VisualTileSource {
                tileset_id: "cave".into(),
                metatile_id: 0x2c,
                subtile_column: 0,
                subtile_row: 0,
                tile_index: index,
            },
            texture: Handle::default(),
            animation_frames: Some(vec![Handle::default(), Handle::default()].into()),
            priority: false,
        }
    }
    #[test]
    fn den_whirlpool_requires_complete_art_and_follows_its_local_water_phase() {
        let first=Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(301),std::marker::PhantomData);
        let second=Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(302),std::marker::PhantomData);
        let mut frame=VisualWorldFrame{map_id:"DragonsDenB1F".into(),grid_size:UVec2::new(4,4),..default()};
        frame.tiles=(0..16).map(|i| {
            let mut t=tile(DEN_WHIRLPOOL_DRAWING[i]);
            t.column=(i%4) as u32;t.row=(i/4) as u32;
            t.source.tileset_id="johto".into();t.source.metatile_id=0x07;
            t.source.subtile_column=(i%4) as u8;t.source.subtile_row=(i/4) as u8;
            t.texture=first.clone();t.animation_frames=Some(vec![first.clone(),second.clone()].into());t
        }).collect();
        assert_eq!(den_whirlpool_water(&frame).len(),4);
        {
            let mut exterior=frame.clone();
            exterior.map_id="Route41".into();
            assert_eq!(den_whirlpool_water(&exterior).len(),4,
                "the same complete Johto drawing animates on exterior ocean routes");
            let mut ocean_images=Assets::<Image>::default();
            let ocean_mask=ocean_images.add(mask(&exterior).unwrap());
            assert_eq!(&ocean_images.get(&ocean_mask).unwrap().data.as_ref().unwrap()[..3],&[255,128,0]);
            exterior.tiles[8].texture=second.clone();
            assert!(refresh_mask_phases(&exterior,&ocean_mask,&mut ocean_images));
            assert_eq!(&ocean_images.get(&ocean_mask).unwrap().data.as_ref().unwrap()[..3],&[255,128,127]);
            exterior.tiles[0].source.tile_index=0x31;
            assert!(den_whirlpool_water(&exterior).is_empty());
        }
        let mut images=Assets::<Image>::default();
        let h=images.add(mask(&frame).unwrap());
        assert_eq!(&images.get(&h).unwrap().data.as_ref().unwrap()[..3],&[255,128,0]);
        frame.tiles[8].texture=second.clone();
        assert!(refresh_mask_phases(&frame,&h,&mut images));
        for (x,z) in [(0,0),(1,0),(0,1),(1,1)] {
            let i=(z*8*32+x*8)*4;
            assert_eq!(&images.get(&h).unwrap().data.as_ref().unwrap()[i..i+3],&[255,128,127]);
        }
        frame.tiles[0].source.tile_index=0x31;
        assert!(den_whirlpool_water(&frame).is_empty());
        assert!(refresh_mask_phases(&frame,&h,&mut images));
        assert_eq!(&images.get(&h).unwrap().data.as_ref().unwrap()[..3],&[0,0,0]);
        frame.tiles[0].source.tile_index=DEN_WHIRLPOOL_DRAWING[0];
        for t in &mut frame.tiles {t.animation_frames=None;}
        assert!(den_whirlpool_water(&frame).is_empty());
        frame.map_id="Route45".into();
        assert!(mask(&frame).is_none());
    }
    #[test]
    fn only_proven_cave_pool_and_waterfall_slots_receive_a_skin() {
        assert_eq!(role(&tile(0x14)), [255, 0]);
        assert_eq!(role(&tile(0x40)), [0, 255]);
        for index in [0x16, 0x2a, 0x3a, 0x26] {
            assert_eq!(role(&tile(index)), [0, 0]);
        }
        let mut t = tile(0x14);
        t.animation_frames = None;
        assert_eq!(role(&t), [0, 0]);
        t = tile(0x40);
        t.source.metatile_id = 0;
        assert_eq!(role(&t), [0, 0]);
        t = tile(0x14);
        t.source.tileset_id = "ice_path".into();
        assert_eq!(role(&t), [0, 0]);
        t = tile(0x14);
        t.texture = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(42), std::marker::PhantomData);
        assert_eq!(role(&t), [0, 0]);
    }
    #[test]
    fn den_pool_skin_requires_its_map_and_authenticated_johto_water() {
        let mut t = tile(0x14);
        t.source.tileset_id = "johto".into();
        t.source.metatile_id = 0x35;
        assert_eq!(role_for_map(&t, "DragonsDenB1F"), [255,0]);
        assert_eq!(role_for_map(&t, "CeruleanGym"), [0,0]);
        t.source.tile_index = 0x40;
        assert_eq!(role_for_map(&t, "DragonsDenB1F"), [0,0]);
        t.source.tile_index = 0x14; t.animation_frames = None;
        assert_eq!(role_for_map(&t, "DragonsDenB1F"), [0,0]);
    }
    #[test]
    fn outdoor_water_requires_source_geometry_and_an_authenticated_animation() {
        let mut t=tile(0x14);t.source.tileset_id="johto".into();t.source.metatile_id=0x35;
        for map in ["LakeOfRage","Route44","CherrygroveCity","NewBarkTown","MountMoonSquare"] {
            assert!(live_outdoor_water(&t,map));
        }
        assert!(!live_outdoor_water(&t,"PlayersHouse1F"));
        let mut mountain=t.clone();mountain.source.tileset_id="kanto".into();mountain.source.metatile_id=0x43;
        assert!(live_outdoor_water(&mountain,"MountMoonSquare"));
        for map in ["MountMoon","MountMoonGiftShop"] {assert!(!live_outdoor_water(&mountain,map));}
        t.source.tile_index=0x05;assert!(!live_outdoor_water(&t,"LakeOfRage"));
        t.source.tile_index=0x14;t.animation_frames=None;
        assert!(!live_outdoor_water(&t,"LakeOfRage"));
    }

    #[test]
    fn cerulean_pool_uses_live_port_water_without_promoting_the_deck() {
        let mut t = tile(0x14);
        t.source.tileset_id = "port".into();
        t.source.metatile_id = 0x2b;
        assert_eq!(role_for_map(&t, "CeruleanGym"), [255, 0]);
        assert!(live_pool_water(&t, "CeruleanGym"));
        assert!(!live_outdoor_water(&t, "CeruleanGym"));
        assert_eq!(role_for_map(&t, "PlayersHouse1F"), [0, 0]);
        t.source.tile_index = 0x0a;
        assert_eq!(role_for_map(&t, "CeruleanGym"), [0, 0]);
        t.source.tile_index = 0x14;
        t.animation_frames = None;
        assert_eq!(role_for_map(&t, "CeruleanGym"), [0, 0]);
    }

    #[test]
    fn optical_blending_accepts_only_coherent_single_source_steps_including_wrap() {
        let handles: Vec<_> = (800..803).map(|id|
            Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(id),std::marker::PhantomData)).collect();
        let mut frame = VisualWorldFrame { map_id:"UnionCaveB1F".into(),
            grid_size:UVec2::new(2,1), ..default() };
        frame.tiles = (0..2).map(|column| {
            let mut t = tile(0x14);
            t.column=column; t.texture=handles[0].clone();
            t.animation_frames=Some(handles.clone().into()); t
        }).collect();
        let mut images=Assets::default();
        let mask_handle=images.add(mask(&frame).unwrap());
        for phase in [1,2,0] {
            for tile in &mut frame.tiles { tile.texture=handles[phase].clone(); }
            let (changed,steps)=refresh_mask_with_steps(&frame,&mask_handle,&mut images);
            assert!(changed);
            assert!((steps.x-std::f32::consts::TAU/3.0).abs()<0.03);
            assert_eq!(steps.y,0.0);
            assert_eq!(images.get(&mask_handle).unwrap().data.as_ref().unwrap()[2],
                (phase*255/3) as u8,"blending never substitutes source pixels");
        }
        assert_eq!(refresh_mask_with_steps(&frame,&mask_handle,&mut images),(false,Vec2::ZERO));
        frame.tiles[0].texture=handles[1].clone();
        assert_eq!(refresh_mask_with_steps(&frame,&mask_handle,&mut images),(true,Vec2::ZERO),
            "a frozen local family cannot be rewound by another cell's blend");
        for tile in &mut frame.tiles { tile.texture=handles[0].clone(); }
        refresh_mask_with_steps(&frame,&mask_handle,&mut images);
        for tile in &mut frame.tiles { tile.texture=handles[2].clone(); }
        assert_eq!(refresh_mask_with_steps(&frame,&mask_handle,&mut images),(true,Vec2::ZERO),
            "dropped frames settle at the authenticated current phase");
    }
    #[test]
    fn waterfall_foam_reaches_only_the_lower_pool_and_survives_phase_updates() {
        let mut frame = VisualWorldFrame { grid_size: UVec2::new(5,5), ..default() };
        let first=Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(51), std::marker::PhantomData);
        let second=Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(52), std::marker::PhantomData);
        for row in 0..5 {
            for column in 0..5 {
                let mut t = tile(if column == 1 {
                    if row == 1 || row == 2 { 0x40 } else { 0x14 }
                } else if column==3 && row==3 { 0x14 } else { 0x16 });
                t.column = column; t.row = row;
                t.texture=first.clone();
                t.animation_frames=Some(vec![first.clone(),second.clone()].into());
                frame.tiles.push(t);
            }
        }
        let original = frame.clone();
        let image = mask(&frame).unwrap();
        let alpha = |image: &Image, x: usize, y: usize|
            image.data.as_ref().unwrap()[(y*40+x)*4+3];
        assert_eq!(alpha(&image,12,4),255,"upstream water remains unchanged");
        assert!(alpha(&image,12,24)<4,"foam is continuous across the landing");
        assert!(alpha(&image,12,28)<alpha(&image,12,34),"foam feathers downstream");
        assert_eq!(alpha(&image,12,39),255,"distant pool remains unchanged");
        assert_eq!(alpha(&image,4,24),255,"dry floor receives no water paint");
        assert_eq!(alpha(&image,24,24),255,"foam cannot jump across dry land into an isolated pool");
        assert_eq!(frame,original,"mask generation cannot alter source geometry or art");
        let mut partial=frame.clone();
        partial.tiles.retain(|t|t.row==3 && t.column==1);
        let incomplete=mask(&partial).unwrap();
        assert!(incomplete.data.as_ref().unwrap().chunks_exact(4).all(|p|p[3]==255),
            "missing atlas cells cannot invent an impact footprint");
        let mut images=Assets::<Image>::default();
        let handle=images.add(image.clone());
        frame.tiles[16].texture=frame.tiles[16].animation_frames.as_ref().unwrap()[1].clone();
        assert!(refresh_mask_phases(&frame,&handle,&mut images));
        let updated=images.get(&handle).unwrap();
        for (a,b) in image.data.as_ref().unwrap().chunks_exact(4).zip(updated.data.as_ref().unwrap().chunks_exact(4)) {
            assert_eq!(a[3],b[3],"animation cannot move the impact footprint");
        }
        frame.tiles[6].animation_frames=None;
        refresh_mask_phases(&frame,&handle,&mut images);
        assert_eq!(alpha(images.get(&handle).unwrap(),12,24),255,
            "an unauthenticated cascade cannot retain foam in its neighboring pool");
    }
    #[test]
    fn live_water_phases_refresh_retained_slots_and_reject_other_families() {
        let mut images = Assets::<Image>::default();
        let image = |value| {
            Image::new(
                Extent3d {
                    width: 8,
                    height: 8,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![value; 256],
                TextureFormat::Rgba8Unorm,
                RenderAssetUsages::default(),
            )
        };
        let first = images.add(image(10));
        let second = images.add(image(20));
        let foreign = images.add(image(30));
        let atlas = images.add(Image::new(
            Extent3d {
                width: 16,
                height: 8,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![10; 512],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        ));
        let mut water = tile(0x14);
        water.column = 1;
        water.texture = first.clone();
        water.animation_frames = Some(vec![first.clone(), second.clone()].into());
        let mut built = VisualWorldFrame {
            map_id: "UnionCave1F".into(),
            grid_size: UVec2::new(2, 1),
            map_texture: atlas.clone(),
            tiles: vec![tile(0x16), water.clone()],
            ..default()
        };
        let phase_mask = images.add(mask(&built).unwrap());
        let image_count = images.len();
        water.column = 0;
        water.texture = second.clone();
        let mut live = crystal_render_api::VisualTileAnimationFrame {
            map_id: built.map_id.clone(),
            grid_origin: IVec2::X,
            tiles: vec![water.clone()],
        };
        refresh_phases(&mut built, &live, &mut images);
        assert!(refresh_mask_phases(&built, &phase_mask, &mut images));
        assert!(!refresh_mask_phases(&built, &phase_mask, &mut images));
        assert_eq!(images.len(), image_count);
        let mask_pixels = images.get(&phase_mask).unwrap().data.as_ref().expect("CPU image pixels");
        assert_eq!(&mask_pixels[0..4], &[0, 0, 0, 255]);
        assert_eq!(&mask_pixels[32..36], &[255, 0, 127, 255]);
        assert_eq!(built.tiles[1].texture, second);
        let pixels = images.get(&atlas).unwrap().data.as_ref().expect("CPU image pixels");
        for row in 0..8 {
            assert_eq!(&pixels[row * 64..row * 64 + 32], &[10; 32]);
            assert_eq!(&pixels[row * 64 + 32..row * 64 + 64], &[20; 32]);
        }
        assert_eq!(images.get(&first).unwrap().data.as_ref().expect("CPU image pixels").as_slice(), vec![10; 256].as_slice());
        live.tiles[0].texture = foreign.clone();
        live.tiles[0].animation_frames = Some(vec![second.clone(), foreign].into());
        refresh_phases(&mut built, &live, &mut images);
        refresh_mask_phases(&built, &phase_mask, &mut images);
        assert_eq!(
            &images.get(&phase_mask).unwrap().data.as_ref().expect("CPU image pixels")[32..36],
            &[255, 0, 127, 255]
        );
        assert_eq!(
            built.tiles[1].texture, second,
            "palette/family changes require fresh geometry"
        );
        built.tiles[1].texture = live.tiles[0].texture.clone();
        refresh_mask_phases(&built, &phase_mask, &mut images);
        assert_eq!(
            &images.get(&phase_mask).unwrap().data.as_ref().expect("CPU image pixels")[32..36],
            &[0, 0, 0, 255]
        );
        built.tiles[1].texture = second.clone();
        refresh_mask_phases(&built, &phase_mask, &mut images);
        assert_eq!(
            &images.get(&phase_mask).unwrap().data.as_ref().expect("CPU image pixels")[32..36],
            &[255, 0, 127, 255]
        );
        live.tiles[0] = water;
        live.map_id = "different-map".into();
        refresh_phases(&mut built, &live, &mut images);
        assert_eq!(built.tiles[1].texture, second);
    }
    #[test]
    fn outdoor_source_water_palette_tracks_live_frames_and_missing_art() {
        let mut images=Assets::<Image>::default();
        let make=|deep:[u8;4],foam:[u8;4]|Image::new(
            Extent3d{width:8,height:8,depth_or_array_layers:1},TextureDimension::D2,
            (0..64).flat_map(|i|if i%8==0 {foam}else {deep}).collect(),
            TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::MAIN_WORLD);
        let day=images.add(make([0,0,255,255],[255,255,255,255]));
        let night=images.add(make([0,0,99,255],[123,123,189,255]));
        let mut water=tile(0x14);water.source.tileset_id="johto".into();water.source.metatile_id=0x35;
        water.texture=day.clone();water.animation_frames=Some(vec![day.clone(),night.clone()].into());
        let mut frame=VisualWorldFrame{map_id:"OlivineCity".into(),tiles:vec![water],..default()};
        let before=frame.clone();
        let finish=native_palette(&frame,&images).unwrap();
        assert_eq!(finish[0].w,3.,"proven Johto coast shares the regional wind surface");
        assert!(finish[0].z>finish[0].y && finish[0].y>finish[0].x && finish[0].z<0.5);
        assert!(finish[1].truncate().max_element()<=1.);
        assert_eq!(frame,before,"palette extraction must not mutate the source frame");
        frame.tiles[0].texture=night.clone();
        let dark=native_palette(&frame,&images).unwrap();
        assert!(dark[0].z<0.2 && dark[0].z>0.05);
        assert!(dark[1].z>dark[1].x && dark[1].z<0.6);
        images.get_mut(&night).unwrap().data.as_mut().unwrap()[3]=0;
        assert_eq!(native_palette(&frame,&images),Some([Vec4::new(0.,0.,0.,-2.);2]),
            "missing opaque art must retain its source surface, never guessed teal");
        frame.tiles[0].animation_frames=None;
        assert!(native_palette(&frame,&images).is_none());
        frame.tiles[0].animation_frames=Some(vec![day,night.clone()].into());
        for family in ["forest","park"] {
            frame.tiles[0].source.tileset_id=family.into();
            assert!(!outdoor_source_palette(&frame));
            assert!(native_palette(&frame,&images).is_none());
        }
        images.get_mut(&night).unwrap().data.as_mut().unwrap()[3]=255;
        for map in ["NationalPark","NationalParkBugContest","SafariZoneBeta"] {
            frame.map_id=map.into();
            let before=frame.clone();
            let finish=native_palette(&frame,&images).unwrap();
            assert_eq!(finish[0].w,3.,"native park ponds share source-driven water optics");
            assert!(finish[0].z>finish[0].y && finish[0].y>finish[0].x);
            assert_eq!(frame,before,"pond optics cannot alter the source or water height");
        }
        frame.tiles[0].animation_frames=None;
        assert!(native_palette(&frame,&images).is_none(),"static fixtures do not own live pond optics");
    }

    #[test]
    fn johto_water_shares_live_pigment_with_calmer_inland_ripples() {
        let mut images=Assets::<Image>::default();
        let handles=[[0,0,255,255],[0,0,99,255]].into_iter().enumerate().map(|(i,deep)| {
            let foam=if i==0 {[255,255,255,255]}else{[123,123,189,255]};
            images.add(Image::new(Extent3d{width:8,height:8,depth_or_array_layers:1},
                TextureDimension::D2,(0..64).flat_map(|p|if p%8==0 {foam}else{deep}).collect(),
                TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::MAIN_WORLD))
        }).collect::<Vec<_>>();
        let mut source=tile(0x14);source.source.metatile_id=0x35;
        source.animation_frames=Some(handles.clone().into());
        let mut frame=VisualWorldFrame{tiles:vec![source],..default()};
        let mut reference=[None,None];
        for family in ["johto","johto_modern"] {
            frame.tiles[0].source.tileset_id=family.into();
            for map in ["NewBarkTown","CherrygroveCity","OlivineCity","CianwoodCity",
                "Route26","Route27","Route34","Route40","Route41"] {
                frame.map_id=map.into();
                for (phase,h) in handles.iter().enumerate() {
                    frame.tiles[0].texture=h.clone();let before=frame.clone();
                    let finish=native_palette(&frame,&images).unwrap();
                    assert_eq!(finish[0].w,3.,"{map}/{family} must share coastal optics");
                    assert_eq!(frame,before,"optical selection must not mutate source or support");
                    if let Some(old)=reference[phase] {assert_eq!(finish,old,"one coastal pigment transform");}
                    else {reference[phase]=Some(finish);}
                }
            }
            for map in ["VioletCity","BlackthornCity","EcruteakCity","GoldenrodCity","Route42","LakeOfRage"] {
                frame.map_id=map.into();
                let style=crate::water_surface::style(&frame);
                assert!(style.y<0.23 && style.z<0.48 && style.w<0.40,
                    "{map}: inland normals, wash and reflection stay calmer than the coast");
                for (phase,h) in handles.iter().enumerate() {
                    frame.tiles[0].texture=h.clone();let before=frame.clone();
                    assert_eq!(native_palette(&frame,&images),reference[phase],
                        "{map}/{family}: inland water shares regional pigment and the existing ripple pass");
                    assert_eq!(frame,before,"source and support remain unchanged");
                }
            }
            frame.map_id="IlexForest".into();
            assert!(native_palette(&frame,&images).is_none(),"forest policy stays untouched");
        }
        let [Some(day),Some(night)]=reference else {panic!("both live phases were exercised");};
        assert!(night[0].truncate().cmplt(day[0].truncate()).all());
        assert!(night[1].truncate().cmplt(day[1].truncate()).all());
    }

    #[test]
    fn regional_optical_finish_tracks_night_and_preserves_indoor_water_families() {
        let mut images=Assets::<Image>::default();
        let make=|blue:u8,light:u8|Image::new(
            Extent3d{width:8,height:8,depth_or_array_layers:1},TextureDimension::D2,
            (0..64).flat_map(|i|if i%8==0 {[light,light,light,255]}else {[0,0,blue,255]}).collect(),
            TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::MAIN_WORLD);
        let day=images.add(make(255,255));let night=images.add(make(99,123));
        let mut water=tile(0x14);water.source.tileset_id="kanto".into();water.source.metatile_id=0x43;
        water.texture=day.clone();water.animation_frames=Some(vec![day.clone(),night.clone()].into());
        let mut frame=VisualWorldFrame{map_id:"VermilionCity".into(),tiles:vec![water],..default()};
        let before=frame.clone();let finish=native_palette(&frame,&images).unwrap();
        assert_eq!(frame,before);
        assert_eq!(finish[0].w, 3., "Kanto opts into wind-driven surface optics");
        assert!(finish[0].z>finish[0].y && finish[0].y>finish[0].x && finish[0].z<0.5);
        assert!(finish[1].z > 0.8 && finish[1].truncate().max_element() <= 1.0,
            "pale blue foam and reflection retain a bounded daylight highlight");
        frame.map_id="CeladonCity".into();assert_eq!(native_palette(&frame,&images),Some(finish));
        frame.tiles[0].texture=night.clone();let dark=native_palette(&frame,&images).unwrap();
        assert!(dark[0].truncate().cmplt(finish[0].truncate()).all());
        assert!(dark[1].truncate().cmplt(finish[1].truncate()).all());
        frame.tiles[0].texture=day;frame.tiles[0].source.tileset_id="johto".into();
        frame.tiles[0].source.metatile_id=0x35;
        assert_eq!(native_palette(&frame,&images),Some(finish));
        frame.tiles[0].source.tileset_id="port".into();
        frame.tiles[0].source.metatile_id=0x18;
        frame.map_id="VermilionPort".into();
        assert_eq!(native_palette(&frame,&images),Some(finish), "Kanto port joins its coastal surface");
        frame.map_id="OlivinePort".into();
        assert_eq!(native_palette(&frame,&images),Some(finish),
            "both harbors share the same source-family wind surface");
        frame.tiles[0].texture=night.clone();
        assert_eq!(native_palette(&frame,&images),Some(dark),
            "harbor reflections retain the live night palette");
        frame.map_id="CeruleanGym".into();
        assert!(native_palette(&frame,&images).is_none(),
            "indoor port-family pools retain their quiet pool optics");
        frame.map_id="VermilionCity".into();
        frame.tiles[0].source.tileset_id="kanto".into();frame.tiles[0].source.metatile_id=0x43;
        frame.tiles[0].texture=night.clone();
        images.get_mut(&night).unwrap().data.as_mut().unwrap()[3]=0;
        assert_eq!(native_palette(&frame,&images),Some([Vec4::new(0.,0.,0.,-2.);2]));
    }

    #[test]
    fn kanto_native_water_patterns_take_palette_without_repeating_source_pixels() {
        let mut images=Assets::<Image>::default();
        let mut data=vec![];
        for i in 0..64 {data.extend(if i%8==0 {[120,160,240,255]} else {[32,40,128,255]});}
        let image=Image::new(Extent3d{width:8,height:8,depth_or_array_layers:1},
            TextureDimension::D2,data,TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::MAIN_WORLD);
        let first=images.add(image.clone());let second=images.add(image);
        let mut water=tile(0x14);water.texture=first.clone();
        water.animation_frames=Some(vec![first.clone(),second.clone()].into());
        let mut frame=VisualWorldFrame{map_id:"TohjoFalls".into(),tiles:vec![water],..default()};
        let palette=native_palette(&frame,&images).unwrap();
        assert!(palette[0].z>palette[0].x && palette[1].z>palette[1].x);
        assert!(palette[1].y>palette[0].y);assert_eq!(palette[0].w,1.);
        frame.tiles[0].texture=second.clone();assert_eq!(native_palette(&frame,&images),Some(palette));
        frame.map_id="UnionCave1F".into();assert_eq!(native_palette(&frame,&images),Some(palette));
        frame.map_id="TohjoFalls".into();frame.tiles[0].animation_frames=None;
        assert!(native_palette(&frame,&images).is_none());
        frame.tiles[0].animation_frames=Some(vec![first,second.clone()].into());
        images.get_mut(&second).unwrap().data.as_mut().unwrap()[3]=0;
        assert!(native_palette(&frame,&images).is_none());
    }

    #[test]
    fn johto_cave_water_uses_live_blue_and_rejects_dark_missing_or_foreign_paint() {
        let mut images=Assets::<Image>::default();
        let texture=images.add(Image::new(Extent3d{width:8,height:8,depth_or_array_layers:1},
            TextureDimension::D2,(0..64).flat_map(|i|if i%8==0 {[120,160,240,255]}
                else {[32,40,128,255]}).collect(),TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::MAIN_WORLD));
        for (map,family) in [("DarkCaveVioletEntrance","dark_cave"),
            ("WhirlIslandB2F","cave"),("SilverCaveRoom1","dark_cave"),("UnionCave1F","cave")] {
            let mut water=tile(0x14);water.source.tileset_id=family.into();
            water.texture=texture.clone();water.animation_frames=Some(vec![texture.clone(),texture.clone()].into());
            let frame=VisualWorldFrame {map_id:map.into(),tiles:vec![water],..default()};
            let before=frame.clone();
            let palette=native_palette(&frame,&images).unwrap();
            assert_eq!(palette[0].w,1.);assert_eq!(registration(&frame).x,3.);
            assert!(palette[0].z>palette[0].y && palette[1].z>palette[1].y);
            assert_eq!(frame,before,"paint extraction cannot change phases, pixels or source identity");
            let mut invalid=frame.clone();invalid.tiles[0].animation_frames=None;
            assert!(native_palette(&invalid,&images).is_none());
            invalid=frame.clone();invalid.tiles[0].source.tileset_id="house".into();
            assert!(!native_cave_source(&invalid));assert!(native_palette(&invalid,&images).is_none());
        }
        let mut water=tile(0x14);water.source.tileset_id="dark_cave".into();
        water.texture=texture.clone();water.animation_frames=Some(vec![texture.clone(),texture.clone()].into());
        let frame=VisualWorldFrame {map_id:"DarkCaveVioletEntrance".into(),tiles:vec![water],..default()};
        images.get_mut(&texture).unwrap().data.as_mut().unwrap()[3]=0;
        assert!(native_palette(&frame,&images).is_none());
        images.get_mut(&texture).unwrap().data=Some([0,0,0,255].repeat(64));
        assert!(native_palette(&frame,&images).is_none(),"unlit source supplies no invented water color");
        images.remove(texture.id());assert!(native_palette(&frame,&images).is_none());
    }

    #[test]
    fn kanto_native_tohjo_water_keeps_live_source_phase_and_map_registration() {
        let first=Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(411),std::marker::PhantomData);
        let second=Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(412),std::marker::PhantomData);
        let mut water=tile(0x14);water.texture=first.clone();
        water.animation_frames=Some(vec![first.clone(),second.clone()].into());
        let mut frame=VisualWorldFrame{map_id:"TohjoFalls".into(),grid_size:UVec2::ONE,
            grid_origin:IVec2::new(10,30),tile_size:Vec2::splat(8.),tiles:vec![water],..default()};
        let native=registration(&frame);assert_eq!(native.x,3.);
        let before=mask(&frame).unwrap();
        assert_eq!(&before.data.as_ref().unwrap()[..4],&[255,0,0,255]);
        frame.tiles[0].texture=second.clone();
        let current=mask(&frame).unwrap();
        assert_eq!(&current.data.as_ref().unwrap()[..3],&[255,0,127]);
        assert_eq!(registration(&frame),native,"source phase must not move procedural flow");
        frame.map_id="UnionCave1F".into();let legacy=registration(&frame);
        assert_eq!(legacy.x,3.);
        assert_eq!([legacy.y,legacy.z,legacy.w],[native.y,native.z,native.w]);
        frame.map_id="VermilionPort".into();assert_eq!(registration(&frame).x,2.);
        frame.map_id="TohjoFalls".into();
        frame.tiles[0].source.tile_index=0x40;
        let cascade=mask(&frame).unwrap();
        assert_eq!(&cascade.data.as_ref().unwrap()[..3],&[0,255,127]);
        frame.tiles[0].source.tile_index=0x14;
        frame.tiles[0].animation_frames=None;assert!(mask(&frame).is_none());
        frame.tiles[0].animation_frames=Some(vec![first,second].into());
        frame.tiles[0].source.tileset_id="house".into();assert!(mask(&frame).is_none());
    }

    #[test]
    fn encounter_water_keeps_the_original_ripple_and_shore_coordinates() {
        for center in [Vec2::ZERO, Vec2::new(144.0, -88.0)] {
            let frame = VisualWorldFrame {
                map_id: "OlivineCity".into(),
                grid_size: UVec2::new(40, 28),
                grid_origin: IVec2::new(-12, 9),
                tile_size: Vec2::splat(32.0),
                center,
                ..default()
            };
            let source = registration(&frame);
            let bounds = crate::water_surface::bounds(&frame);
            for translation in [Vec3::ZERO, Vec3::new(-19.0, 3.0, 7.0)] {
                let scale = 1.0 / 16.0;
                let retained = encounter_registration(&frame, translation, scale);
                for local in [Vec3::new(-320.0, -8.0, -224.0), Vec3::ZERO, Vec3::new(140.0, -8.0, 99.0)] {
                    let world = local + Vec3::new(center.x, 0.0, -center.y);
                    let rebased = local * scale + translation;
                    let expected = (Vec2::new(world.x, world.z) + Vec2::new(source.z, source.w)) / source.y;
                    let actual = (Vec2::new(rebased.x, rebased.z) + Vec2::new(retained.z, retained.w)) / retained.y;
                    assert!((actual - expected).length() < 0.00001);
                    let shore = |map: Vec2| (map - Vec2::new(bounds.x, bounds.y)) / Vec2::new(bounds.z, bounds.w);
                    assert!((shore(actual) - shore(expected)).length() < 0.00001);
                    assert_eq!(retained.x, source.x, "retain the authored water mode");
                }
            }
        }
    }

    #[test]
    fn ripple_registration_keeps_source_position_when_viewport_scrolls_and_resizes() {
        let mut frame = VisualWorldFrame {
            grid_size: UVec2::new(20, 12),
            grid_origin: IVec2::new(10, 30),
            tile_size: Vec2::splat(8.),
            ..default()
        };
        let a = registration(&frame);
        let old_point = (Vec2::new(-80., -48.) + Vec2::new(a.z, a.w)) / a.y;
        frame.grid_origin = IVec2::new(12, 29);
        frame.grid_size = UVec2::new(24, 10);
        let b = registration(&frame);
        let new_point = (Vec2::new(-112., -32.) + Vec2::new(b.z, b.w)) / b.y;
        assert_eq!(old_point, Vec2::new(10., 30.));
        assert_eq!(new_point, old_point);
        let built = frame.clone();
        frame.grid_origin += IVec2::new(2, -1);
        frame.center = Vec2::new(32., 12.);
        let retained = retained_registration(&built, &frame);
        let displacement = Vec2::new(16., -4.);
        let retained_point =
            (Vec2::new(-112., -32.) + displacement + Vec2::new(retained.z, retained.w))
                / retained.y;
        assert_eq!(retained_point, old_point);
    }
    #[test]
    fn mask_matches_built_atlas_slots_without_mutating_source_frames() {
        let mut a = tile(0x14);
        a.column = 1;
        let frame = VisualWorldFrame {
            grid_size: UVec2::new(2, 1),
            tiles: vec![tile(0x16), a],
            ..default()
        };
        let original = frame.clone();
        let image = mask(&frame).unwrap();
        assert!(image.asset_usage.contains(RenderAssetUsages::MAIN_WORLD));
        assert_eq!((image.width(), image.height()), (16, 8));
        for y in 0..8 {
            for x in 0..16 {
                let i = (y * 16 + x) * 4;
                assert_eq!(
                    &image.data.as_ref().expect("CPU image pixels")[i..i + 4],
                    if x < 8 {
                        &[0, 0, 0, 255]
                    } else {
                        &[255, 0, 0, 255]
                    }
                );
            }
        }
        assert_eq!(frame, original);
    }
}


/// Source-palette optics apply to proven regional water and native park ponds.
/// The animation/drawing role remains the authority for occupancy.
pub(super) fn outdoor_source_palette(frame:&VisualWorldFrame)->bool {
    if !outdoor(&frame.map_id) || frame.map_id.as_ref()=="IlexForest" {return false;}
    let mut found=false;
    for tile in &frame.tiles {
        if role_for_map(tile,&frame.map_id)==[0,0] {continue;}
        let native_park = tile.source.tileset_id.as_ref()=="park"
            && matches!(frame.map_id.as_ref(),"NationalPark"|"NationalParkBugContest"|"SafariZoneBeta");
        if !native_park && !matches!(tile.source.tileset_id.as_ref(),"johto"|"johto_modern"|"kanto"|"port") {return false;}
        found=true;
    }
    found
}

/// Palette samples supply currents without repeating the source drawing.
/// w=1 selects native cave paint; w=2 selects outdoor source paint;
/// w=3 enables the shared regional/harbor wind surface. w=-2
/// keeps the original source surface if an eligible outdoor palette is missing.
pub(super) fn native_palette(frame:&VisualWorldFrame,images:&Assets<Image>)->Option<[Vec4;2]> {
    let open=outdoor_source_palette(frame);
    if frame.map_id.as_ref()!="TohjoFalls" && !native_cave_source(frame) && !open {return None;}
    let palette=(|| {
        let mut colors=std::collections::BTreeSet::new();
        let mut sampled=std::collections::HashSet::new();
        for tile in &frame.tiles {
            if role_for_map(tile,&frame.map_id)==[0,0] || !sampled.insert(tile.texture.id()) {continue;}
            let image=images.get(&tile.texture)?;
            if image.texture_descriptor.size.width!=8 || image.texture_descriptor.size.height!=8
                || !matches!(image.texture_descriptor.format,TextureFormat::Rgba8Unorm|TextureFormat::Rgba8UnormSrgb) {return None;}
            let data=image.data.as_ref()?;
            if data.len()!=256 || data.chunks_exact(4).any(|p|p[3]!=255) {return None;}
            for pixel in data.chunks_exact(4) {
                let c=Vec3::from_array([pixel[0],pixel[1],pixel[2]].map(crate::source_color::linear));
                if c.dot(Vec3::new(0.2126,0.7152,0.0722))>0.006 {colors.insert([pixel[0],pixel[1],pixel[2]]);}
            }
        }
        let mut colors=colors.into_iter().map(|rgb|Vec3::from_array(rgb.map(crate::source_color::linear))).collect::<Vec<_>>();
        colors.sort_by(|a,b|a.dot(Vec3::new(0.2126,0.7152,0.0722)).total_cmp(&b.dot(Vec3::new(0.2126,0.7152,0.0722))));
        if colors.len()<2 {return None;}
        let mut deep=colors[0];
        let mut foam=colors[colors.len()-1];
        // Regional blue/white supplies absorption and pale reflection, rather
        // than a flat indexed-blue plane. The existing water style keeps
        // inland ponds calmer than exposed coasts; geometry stays untouched.
        if open {
            foam=Vec3::new(foam.x*0.62+deep.x*0.06,foam.y*0.82+deep.y*0.04,foam.z*0.90+deep.z*0.04);
            deep=Vec3::new(deep.x*0.45+deep.z*0.016,deep.y*0.55+deep.z*0.16,deep.z*0.48+deep.y*0.10);
        }
        let flag=if open {3.} else {1.};
        Some([deep.extend(flag),foam.extend(flag)])
    })();
    palette.or_else(||open.then_some([Vec4::new(0.,0.,0.,-2.);2]))
}
