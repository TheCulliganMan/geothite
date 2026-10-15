//! Local sky fill for complete outdoor cave exits. Inter-floor ladders and
//! pits never emit light. Coordinates follow the live terrain scroll.
use bevy::prelude::*;
use crystal_render_api::VisualWorldFrame;

pub(super) fn exits(frame: &VisualWorldFrame) -> [Vec4; 4] {
    let mut result = [Vec4::ZERO; 4];
    if !frame.active || frame.tile_size.min_element() <= 0. {
        return result;
    }
    let extent = frame.grid_size.as_vec2() * frame.tile_size;
    let mut count = 0;
    for anchor in &frame.tiles {
        if crate::cave::daylight_exit_local(&anchor.source) != Some((0, 0)) {
            continue;
        }
        let complete = (0..2).all(|y| {
            (0..2).all(|x| {
                frame.tiles.iter().any(|tile| {
                    tile.column == anchor.column + x
                        && tile.row == anchor.row + y
                        && tile.source.tileset_id == anchor.source.tileset_id
                        && crate::cave::daylight_exit_local(&tile.source)
                            == Some((x as u8, y as u8))
                })
            })
        });
        if !complete {
            continue;
        }
        result[count] = Vec4::new(
            frame.center.x - extent.x * 0.5 + (anchor.column as f32 + 1.) * frame.tile_size.x,
            crate::cave::CAVE_SHELF_HEIGHT * frame.tile_size.y / 8.,
            -frame.center.y - extent.y * 0.5 + anchor.row as f32 * frame.tile_size.y,
            frame.tile_size.y * 5.,
        );
        count += 1;
        if count == result.len() {
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::{VisualTile, VisualTileSource};

    #[test]
    fn sky_temperature_comes_from_live_classic_pixels_and_rejects_missing_art() {
        use bevy::asset::RenderAssetUsages;
        use bevy::render::render_resource::{Extent3d,TextureDimension,TextureFormat};
        let mut images=Assets::<Image>::default();
        let mut data=[28,16,36,255].repeat(32);
        data.extend([198,165,222,255].repeat(32));
        let texture=images.add(Image::new(Extent3d{width:8,height:8,depth_or_array_layers:1},
            TextureDimension::D2,data,TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::MAIN_WORLD));
        let frame=VisualWorldFrame{active:true,tiles:vec![VisualTile{
            column:0,row:0,texture:texture.clone(),animation_frames:None,priority:false,
            source:VisualTileSource{tileset_id:"dark_cave".into(),metatile_id:0x24,
                tile_index:0x01,subtile_column:2,subtile_row:2}}],..Default::default()};
        let tint=sky_tint(&frame,&images);
        assert_eq!(tint.w,1.);
        assert_eq!(tint.z,1.);
        assert!(tint.x>tint.y && tint.x<tint.z,"preserve the classic violet temperature");
        images.get_mut(&texture).unwrap().data.as_mut().unwrap()[3]=0;
        assert_eq!(sky_tint(&frame,&images),Vec4::ZERO);
        images.remove(texture.id());
        assert_eq!(sky_tint(&frame,&images),Vec4::ZERO);
    }

    #[test]
    fn only_complete_outdoor_exits_supply_scroll_stable_sky_fill() {
        let mut frame = VisualWorldFrame {
            active: true,
            tile_size: Vec2::splat(8.),
            grid_size: UVec2::splat(4),
            center: Vec2::new(30., 50.),
            ..Default::default()
        };
        for y in 0..2 {
            for x in 0..2 {
                frame.tiles.push(VisualTile {
                    column: x + 2,
                    row: y + 2,
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: "cave".into(),
                        metatile_id: 0x24,
                        tile_index: 0x01,
                        subtile_column: x as u8 + 2,
                        subtile_row: y as u8 + 2,
                    },
                });
            }
        }
        assert_eq!(exits(&frame)[0], Vec4::new(38., 6., -50., 40.));
        frame.center += Vec2::new(8., -8.);
        for tile in &mut frame.tiles {
            tile.column -= 1;
            tile.row -= 1;
        }
        assert_eq!(exits(&frame)[0], Vec4::new(38., 6., -50., 40.));
        frame.tiles[0].source.tile_index = 0xff;
        assert_eq!(exits(&frame), [Vec4::ZERO; 4]);
        for tile in &mut frame.tiles {
            tile.source.metatile_id = 0x0b;
            tile.source.tile_index = 0x22;
        }
        assert_eq!(exits(&frame), [Vec4::ZERO; 4]);
    }
}

/// Normalize the actual light mineral swatch from the same live tile image
/// used by 2D. Missing, transparent or non-source artwork supplies no fill.
pub(super) fn sky_tint(frame: &VisualWorldFrame, images: &Assets<Image>) -> Vec4 {
    use bevy::render::render_resource::TextureFormat;
    if !frame.active {
        return Vec4::ZERO;
    }
    for index in [0x01, 0x16] {
        for tile in &frame.tiles {
            if !matches!(tile.source.tileset_id.as_ref(), "cave" | "dark_cave")
                || tile.source.tile_index != index
            {
                continue;
            }
            let Some(image) = images.get(&tile.texture) else {
                continue;
            };
            if image.texture_descriptor.size.width != 8
                || image.texture_descriptor.size.height != 8
                || !matches!(
                    image.texture_descriptor.format,
                    TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb
                )
            {
                continue;
            }
            let Some(data) = image.data.as_ref() else {
                continue;
            };
            if data.len() != 256 || data.chunks_exact(4).any(|p| p[3] != 255) {
                continue;
            }
            let mut brightest = Vec3::ZERO;
            let weights = Vec3::new(0.2126, 0.7152, 0.0722);
            for pixel in data.chunks_exact(4) {
                let color = Vec3::from_array([pixel[0], pixel[1], pixel[2]].map(|v| {
                    let c = f32::from(v) / 255.;
                    if c <= 0.04045 {
                        c / 12.92
                    } else {
                        ((c + 0.055) / 1.055).powf(2.4)
                    }
                }));
                if color.dot(weights) > brightest.dot(weights) {
                    brightest = color;
                }
            }
            if brightest.dot(weights) > 0.006 {
                return (brightest / brightest.max_element()).extend(1.);
            }
        }
    }
    Vec4::ZERO
}
