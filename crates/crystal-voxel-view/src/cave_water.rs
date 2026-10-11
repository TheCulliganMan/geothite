//! Presentation mask for proven live cave water. UVs, animation frames and
//! source pixels remain untouched; the surface shader colors only these slots.
use bevy::asset::RenderAssetUsages;
use bevy::{
    prelude::*,
    render::{
        
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use crystal_render_api::{VisualTile, VisualWorldFrame};

fn role(tile: &VisualTile) -> [u8; 2] { role_for_map(tile, "") }

fn role_for_map(tile: &VisualTile, map: &str) -> [u8; 2] {
    let s = &tile.source;
    let den = map == "DragonsDenB1F" && s.tileset_id.as_ref() == "johto";
    if !(matches!(s.tileset_id.as_ref(), "cave" | "dark_cave") || den)
        || s.subtile_column >= 4
        || s.subtile_row >= 4
        || !tile
            .animation_frames
            .as_ref()
            .is_some_and(|frames| frames.len() > 1 && frames.contains(&tile.texture))
    {
        return [0, 0];
    }
    match s.tile_index {
        0x14 => [255, 0],
        0x40 if !den && s.metatile_id == 0x2c => [0, 255],
        _ => [0, 0],
    }
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

pub(super) fn registration(frame: &VisualWorldFrame) -> Vec4 {
    Vec4::new(
        1.,
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
    for tile in &frame.tiles {
        if tile.column >= frame.grid_size.x || tile.row >= frame.grid_size.y {
            return None;
        }
        let [pool, fall] = role_for_map(tile, &frame.map_id);
        for y in 0..8 {
            for x in 0..8 {
                let i = (((tile.row * 8 + y) * width + tile.column * 8 + x) * 4) as usize;
                pixels[i..i + 4].copy_from_slice(&[
                    pool,
                    fall,
                    if pool > 0 || fall > 0 { phase(tile) } else { 0 },
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
    image.sampler = bevy::image::ImageSampler::nearest();
    Some(image)
}

/// Encode only phases accepted by the retained source atlas. Geometry, source
/// assets stay intact; role channels keep requiring the authenticated family.
/// Unchanged frames do not mark the image dirty or upload it again.
pub(super) fn refresh_mask_phases(
    built: &VisualWorldFrame,
    handle: &Handle<Image>,
    images: &mut Assets<Image>,
) -> bool {
    let Some(mask) = images.get(handle) else {
        return false;
    };
    if mask.width() != built.grid_size.x * 8 || mask.height() != built.grid_size.y * 8 {
        return false;
    }
    let width = mask.width() as usize;
    let mut updates = Vec::new();
    let mut roles_changed = false;
    for tile in &built.tiles {
        if tile.column >= built.grid_size.x || tile.row >= built.grid_size.y {
            continue;
        }
        let [pool, fall] = role_for_map(tile, &built.map_id);
        let next = [
            pool,
            fall,
            if pool > 0 || fall > 0 { phase(tile) } else { 0 },
        ];
        let first = (tile.row as usize * 8 * width + tile.column as usize * 8) * 4;
        if mask.data.as_ref().expect("CPU image pixels")[first..first + 3] != next {
            roles_changed |= mask.data.as_ref().expect("CPU image pixels")[first..first + 2] != next[..2];
            updates.push((tile.column as usize, tile.row as usize, next));
        }
    }
    if updates.is_empty() {
        return false;
    }
    if roles_changed {
        if let Some(rebuilt) = self::mask(built) {
            images.get_mut(handle).unwrap().data = rebuilt.data;
            return true;
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
    true
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
        assert_eq!(role_for_map(&t, "Route45"), [0,0]);
        t.source.tile_index = 0x40;
        assert_eq!(role_for_map(&t, "DragonsDenB1F"), [0,0]);
        t.source.tile_index = 0x14; t.animation_frames = None;
        assert_eq!(role_for_map(&t, "DragonsDenB1F"), [0,0]);
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
