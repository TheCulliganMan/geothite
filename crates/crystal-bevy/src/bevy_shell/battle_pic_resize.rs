// ReturnMon/EnterMon compact selected intact tiles in a fixed battler slot.
// The source grid is column-major in VRAM, but decoded images are row-major.
// This is neither a central crop nor resampling of the complete picture.
fn visible_battle_pic_resize_rgba(
    source: &[u8],
    source_tiles: usize,
    shown_tiles: u8,
) -> Option<Vec<u8>> {
    let axes: &[usize] = match (source_tiles, shown_tiles) {
        (7, 7) => &[0, 1, 2, 3, 4, 5, 6],
        (7, 5) => &[0, 1, 3, 5, 6],
        (7, 3) => &[0, 3, 6],
        (6, 6) => &[0, 1, 2, 3, 4, 5],
        (6, 4) => &[0, 2, 3, 5],
        (6, 2) => &[0, 5],
        _ => return None,
    };
    let width = source_tiles * 8;
    if source.len() != width * width * 4 {
        return None;
    }
    let mut output = vec![0; source.len()];
    let left = (source_tiles - axes.len()) / 2 * 8;
    let top = (source_tiles - axes.len()) * 8;
    for (dy, sy) in axes.iter().copied().enumerate() {
        for (dx, sx) in axes.iter().copied().enumerate() {
            for row in 0..8 {
                let from = ((sy * 8 + row) * width + sx * 8) * 4;
                let to = ((top + dy * 8 + row) * width + left + dx * 8) * 4;
                output[to..to + 8 * 4].copy_from_slice(&source[from..from + 8 * 4]);
            }
        }
    }
    Some(output)
}

fn battle_pic_resize_frame(
    art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
    source: &SpriteFrame,
    side: PokemonSpriteSide,
    shown_tiles: u8,
) -> Result<SpriteFrame> {
    let source_tiles = match side {
        PokemonSpriteSide::Front => 7,
        PokemonSpriteSide::Back => 6,
    };
    anyhow::ensure!(
        source.size == Vec2::splat((source_tiles * 8) as f32),
        "battle pic resize requires a normalized full battler slot"
    );
    if usize::from(shown_tiles) == source_tiles {
        return Ok(source.clone());
    }
    let key = IntroArtKey {
        asset_id: format!("battle-pic-resize:{:?}:{shown_tiles}", source.handle.id()),
    };
    if let Some(cached) = art.intro_cache.get(&key) {
        return Ok(cached.clone());
    }
    let mut image = images
        .get(&source.handle)
        .context("battle pic resize source image is unavailable")?
        .clone();
    anyhow::ensure!(
        matches!(image.texture_descriptor.format,
            TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm),
        "battle pic resize requires decoded RGBA pixels"
    );
    image.data = visible_battle_pic_resize_rgba(&image.data, source_tiles, shown_tiles)
        .context("unsupported battle pic resize grid or pixel length")?;
    image.sampler = ImageSampler::nearest();
    let frame = SpriteFrame {
        handle: images.add(image),
        size: source.size,
    };
    art.intro_cache.insert(key, frame.clone());
    Ok(frame)
}

#[cfg(test)]
mod battle_pic_resize_tests {
    use super::*;

    fn pattern(width: usize) -> Vec<u8> {
        (0..width * width).flat_map(|pixel| {
            let (x, y) = (pixel % width, pixel / width);
            [x as u8, y as u8, (x ^ (y * 3)) as u8,
                if (x + y * 2) % 11 == 0 { 0 } else { 255 }]
        }).collect()
    }

    #[test]
    fn pic_resize_preserves_selected_tile_pixels_and_bottom_center() {
        // Behavioral reference coordinates, including skipped interior tiles.
        for (base, shown, left, top, axes) in [
            (7, 7, 0, 0, vec![0, 1, 2, 3, 4, 5, 6]),
            (7, 5, 8, 16, vec![0, 1, 3, 5, 6]),
            (7, 3, 16, 32, vec![0, 3, 6]),
            (6, 6, 0, 0, vec![0, 1, 2, 3, 4, 5]),
            (6, 4, 8, 16, vec![0, 2, 3, 5]),
            (6, 2, 16, 32, vec![0, 5]),
        ] {
            let width = base * 8;
            let source = pattern(width);
            let output = visible_battle_pic_resize_rgba(&source, base, shown).unwrap();
            assert_eq!(left * 2 + axes.len() * 8, width);
            assert_eq!(top + axes.len() * 8, width);
            for y in 0..width { for x in 0..width {
                let pixel = &output[(y * width + x) * 4..(y * width + x + 1) * 4];
                if x < left || x >= left + axes.len() * 8 || y < top {
                    assert_eq!(pixel, [0, 0, 0, 0]);
                } else {
                    let sx = axes[(x - left) / 8] * 8 + (x - left) % 8;
                    let sy = axes[(y - top) / 8] * 8 + (y - top) % 8;
                    assert_eq!(pixel, &source[(sy * width + sx) * 4..(sy * width + sx + 1) * 4]);
                }
            }}
            // Every phase retains the source bottom-right tile and its texels.
            let last = ((width - 1) * width + left + axes.len() * 8 - 1) * 4;
            assert_eq!(&output[last..last + 4], &source[source.len() - 4..]);
        }
        assert!(visible_battle_pic_resize_rgba(&pattern(16), 7, 3).is_none());
        assert!(visible_battle_pic_resize_rgba(&pattern(56), 7, 4).is_none());
        assert!(visible_battle_pic_resize_rgba(&pattern(48), 6, 3).is_none());
    }

    #[test]
    fn pic_resize_texture_cache_and_scanlines_keep_the_full_slot() {
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        let source = SpriteFrame {
            handle: images.add(Image::new(
                Extent3d { width: 56, height: 56, depth_or_array_layers: 1 },
                TextureDimension::D2, pattern(56), TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            )),
            size: Vec2::splat(56.0),
        };
        let remapped = battle_pic_resize_frame(&mut art, &mut images, &source,
            PokemonSpriteSide::Front, 5).unwrap();
        let repeat = battle_pic_resize_frame(&mut art, &mut images, &source,
            PokemonSpriteSide::Front, 5).unwrap();
        assert_eq!(remapped.handle, repeat.handle);
        assert_eq!(remapped.size, source.size);
        let identity = battle_pic_resize_frame(&mut art, &mut images, &source,
            PokemonSpriteSide::Front, 7).unwrap();
        assert_eq!(identity.handle, source.handle);
        // Destination slot (24,32) is source (24,24), not central-crop (24,32).
        let offset = (32 * 56 + 24) * 4;
        assert_eq!(&images.get(&remapped.handle).unwrap().data[offset..offset + 2], [24,24]);

        let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
        let center = Vec3::new(PLAYFIELD_LEFT + 124.0 * scale,
            PLAYFIELD_TOP - 28.0 * scale, 3.0);
        for scanlines in [false, true] {
            let mut world = World::new();
            let mut queue = bevy::ecs::world::CommandQueue::default();
            let offsets = VisibleBattleLineOffsets { x: [3; 0x5f], y: [2; 0x5f], bgp: None };
            spawn_battle_battler_texture(&mut Commands::new(&mut queue, &world),
                &remapped, remapped.size * scale, center, Color::WHITE,
                None, None, None, scanlines.then_some(&offsets), None);
            queue.apply(&mut world);
            let mut query = world.query_filtered::<(&Sprite, &Transform, &Handle<Image>), With<BattleBattlerMarker>>();
            if scanlines {
                let (sprite, pose, texture) = query.iter(&world).find(|(_, pose, _)|
                    (pose.translation.y - (PLAYFIELD_TOP - 30.5 * scale)).abs() < 0.001).unwrap();
                assert_eq!(texture, &remapped.handle);
                assert_eq!(sprite.rect, Some(Rect::new(0.0, 32.0, 56.0, 33.0)));
                assert_eq!(pose.translation.x, center.x - 3.0 * scale);
            } else {
                let (sprite, pose, texture) = query.single(&world);
                assert_eq!(texture, &remapped.handle);
                assert_eq!(sprite.rect, None);
                assert_eq!(sprite.custom_size, Some(Vec2::splat(56.0 * scale)));
                assert_eq!(pose.translation, center);
            }
        }
    }
}
