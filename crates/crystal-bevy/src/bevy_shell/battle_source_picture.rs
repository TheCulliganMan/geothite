// Keep native picture-bank clipping local to the battler. The field, camera,
// source clock and authoritative battle remain unchanged.
fn immersive_picture_extractions(
    animation: Option<&VisibleMoveAnimation>,
    art: &RenderedTilesetArt,
) -> [Option<VisibleBattlerRowExtraction>; 2] {
    if let Some(animation) = animation.filter(|a| a.started && visible_battle_uses_implicit_rows(a))
        && let Some(playback) = &art.battle_object_runtime
    {
        let (player, enemy) = visible_live_battler_row_extractions(animation, playback);
        [player, enemy]
    } else {
        let (player, enemy) = visible_move_battler_row_extractions(animation);
        [player, enemy]
    }
}

fn immersive_picture_mask(
    size: Vec2,
    clip: Option<u8>,
    remove: Option<VisibleRemoveMonClip>,
    extraction: Option<VisibleBattlerRowExtraction>,
    extracted_only: bool,
) -> Vec<u8> {
    let width = size.x as usize;
    let height = size.y as usize;
    let extracted = extraction.filter(|e| e.render_extracted)
        .map(|e| visible_battler_extracted_row_rects(size.x, size.y, e))
        .unwrap_or_default();
    (0..width * height).map(|index| {
        let x = (index % width) as f32;
        let y = (index / width) as f32;
        let inside = if extracted_only {
            extracted.iter().any(|r| x >= r[0] && y >= r[1] && x < r[2] && y < r[3])
        } else {
            let edge = clip.map(|tiles| f32::from(tiles) * SOURCE_TILE_SIZE as f32);
            let visible = edge.is_none_or(|edge| {
                let half = (size - Vec2::splat(edge).min(size)) * 0.5;
                x >= half.x && y >= half.y && x < size.x - half.x && y < size.y - half.y
            });
            let half = edge.map_or(Vec2::ZERO,
                |edge| (size - Vec2::splat(edge).min(size)) * 0.5);
            let retained = remove.is_none_or(|r| {
                let cut = f32::from(r.source_pixels).min(size.x - half.x * 2.);
                if r.crop_left { x >= half.x + cut }
                else { x < size.x - half.x - cut }
            });
            let retained_rows = extraction.is_none_or(|e| {
                !e.bg_rows_cleared || if e.top {
                    y >= f32::from(e.rows) * SOURCE_TILE_SIZE as f32
                } else { y < size.y - f32::from(e.rows) * SOURCE_TILE_SIZE as f32 }
            });
            visible && retained && retained_rows
        };
        u8::from(inside)
    }).collect()
}

fn immersive_masked_picture(
    art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
    source: &SpriteFrame,
    mask: Vec<u8>,
) -> Result<SpriteFrame> {
    let key = (source.handle.id(), mask);
    if let Some(frame) = art.battle_source_picture_cache.get(&key) { return Ok(frame.clone()); }
    let mut image = images.get(&source.handle).context("source picture bank")?.clone();
    let pixels = image.data.as_mut().context("source picture pixels")?;
    anyhow::ensure!(pixels.len() == key.1.len() * 4, "source picture mask dimensions");
    for (pixel, visible) in pixels.chunks_exact_mut(4).zip(&key.1) {
        if *visible == 0 { pixel[3] = 0; }
    }
    let frame = SpriteFrame { handle: images.add(image), size: source.size };
    art.battle_source_picture_cache.insert(key, frame.clone());
    Ok(frame)
}

fn immersive_source_picture_rows(
    source: &mut VisualBattleSourceFrame,
    animation: &VisibleMoveAnimation,
    pictures: &[Option<SpriteFrame>; 2],
    art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
) -> Result<()> {
    let extractions = immersive_picture_extractions(Some(animation), art);
    for (index, extraction) in extractions.into_iter().enumerate() {
        // A supported modeled strip already has its own exact compositor.
        if source.battler_rows[index].is_some() { continue; }
        let Some(extraction) = extraction.filter(|e| e.render_extracted) else { continue; };
        let Some(picture) = &pictures[index] else { continue; };
        let mask = immersive_picture_mask(picture.size, None, None, Some(extraction), true);
        if mask.iter().all(|v| *v == 0) { continue; }
        let neutral = immersive_masked_picture(art, images, picture, mask)?;
        let rendered = battle_battler_bgp_frame(art, images, &neutral, source.battler_bgps[index])?;
        let slot = extraction.oam_slot.map_or(index, usize::from);
        let anchor = if index == 0 { Vec2::new(16., 48.) } else { Vec2::new(96., 0.) };
        source.objects.push(VisualBattleSourceObject {
            slot,
            object_id: Arc::from("SOURCE_BATTLER_ROWS"),
            texture: rendered.handle,
            neutral_texture: neutral.handle,
            center: anchor + picture.size * 0.5,
            size: picture.size,
            uv_rect: Rect::new(0., 0., 1., 1.),
            placement: Some(crystal_render_api::VisualBattleSourcePlacement::BattlerLocal {
                side: if index == 0 { VisualBattleSide::Player } else { VisualBattleSide::Enemy },
            }),
        });
    }
    Ok(())
}

#[cfg(test)]
mod source_picture_tests {
    use super::*;

    #[test]
    fn source_picture_faint_and_withdraw_keep_exact_bank_masks() {
        let extraction = VisibleBattlerRowExtraction {
            rows: 2, top: false, bg_rows_cleared: true, render_extracted: false,
            oam_row_masks: None, oam_slot: None, oam_layer: VisibleBattleOamLayer::Legacy,
        };
        let mask = immersive_picture_mask(Vec2::splat(56.), None, None, Some(extraction), false);
        assert!(mask[..56 * 40].iter().all(|v| *v == 1));
        assert!(mask[56 * 40..].iter().all(|v| *v == 0));
        let mask = immersive_picture_mask(Vec2::splat(48.), Some(4),
            Some(VisibleRemoveMonClip { source_pixels: 16, crop_left: true }), None, false);
        for y in 0..48 {
            for x in 0..48 {
                assert_eq!(mask[y * 48 + x], u8::from((24..40).contains(&x) && (8..40).contains(&y)));
            }
        }
        let rows = immersive_picture_mask(Vec2::splat(56.), None, None,
            Some(VisibleBattlerRowExtraction { render_extracted: true, ..extraction }), true);
        assert!(rows[..56 * 40].iter().all(|v| *v == 0));
        assert!(rows[56 * 40..].iter().all(|v| *v == 1));
    }

    #[test]
    fn source_picture_masks_share_cached_images_and_preserve_source_pixels() {
        let mut images = Assets::<Image>::default();
        let source = SpriteFrame { handle: images.add(Image::new(
            Extent3d { width: 8, height: 8, depth_or_array_layers: 1 },
            TextureDimension::D2, [11, 22, 33, 255].repeat(64),
            TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default())), size: Vec2::splat(8.) };
        let original = images.get(&source.handle).unwrap().data.clone();
        let mut art = RenderedTilesetArt::default();
        let mut mask = vec![1; 64]; mask[32..].fill(0);
        let first = immersive_masked_picture(&mut art, &mut images, &source, mask.clone()).unwrap();
        let second = immersive_masked_picture(&mut art, &mut images, &source, mask).unwrap();
        assert_eq!(first.handle, second.handle);
        assert_eq!(images.get(&source.handle).unwrap().data, original);
        let pixels = images.get(&first.handle).unwrap().data.as_ref().unwrap();
        for (index, pixel) in pixels.chunks_exact(4).enumerate() {
            assert_eq!(pixel, &[11, 22, 33, if index < 32 {255} else {0}]);
        }
    }
}
