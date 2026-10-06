//! A presentation grade for exact native Violet floor texels in a private atlas.
//! Geometry, UVs, source images, alpha, and every other atlas region stay intact.
use crate::{
    live_profiles::Document,
    profile::{CellShape, shape_for_source_on_map},
};
use bevy::{
    prelude::Image,
    render::render_resource::{TextureDimension, TextureFormat},
};
use crystal_render_api::{VisualTileSource, VisualWorldFrame};
use std::collections::HashMap;

const TILE_PIXELS: u32 = 8;
const COMPRESSION: f32 = 0.6;

/// This is an exact floor-art allowlist, not a general flat-surface predicate.
/// The native recess proof also identifies $19's upper rows, $2d, and the
/// unpainted halves of the plaque/statue blocks as platform backing. $15 has
/// wall art in rows 0/1, trim tile $11 in row 2, and floor tile $01 in row 3.
fn native_floor(source: &VisualTileSource) -> bool {
    if source.tileset_id.as_ref() != "elite_four_room"
        || source.tile_index != 0x01
        || source.subtile_column >= 4
        || source.subtile_row >= 4
    {
        return false;
    }
    let phase_matches = match source.metatile_id {
        0x2d => true,
        0x19 => source.subtile_row < 2,
        0x15 => source.subtile_row == 3,
        0x33 => source.subtile_row >= 2,
        0x1d => source.subtile_column >= 2,
        0x1e => source.subtile_column < 2,
        _ => false,
    };
    phase_matches
        && matches!(
            shape_for_source_on_map("VioletGym", source),
            CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
        )
}

fn custom_floor_drawing(profiles: &Document) -> bool {
    profiles.objects.iter().any(|object| {
        object.tileset == "elite_four_room"
            && object.map.as_deref().is_none_or(|map| map == "VioletGym")
            && object
                .maps
                .as_ref()
                .is_none_or(|maps| maps.iter().any(|map| map == "VioletGym"))
            && object.tiles.iter().flatten().any(|&tile| tile == 0x01)
    })
}

fn to_linear(value: u8) -> f32 {
    let value = f32::from(value) / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn to_srgb(value: f32) -> u8 {
    let value = if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn grade(pixel: &mut [u8], linear: &[f32; 256]) {
    let rgb = [pixel[0], pixel[1], pixel[2]].map(|value| linear[value as usize]);
    let luminance = rgb[0] * 0.2126 + rgb[1] * 0.7152 + rgb[2] * 0.0722;
    // A shared scalar preserves linear RGB ratios. Bright cream is softened
    // more than blue; no gray overlay, black lift, or new hue is introduced.
    let gain = 1.0 / (1.0 + COMPRESSION * luminance);
    for channel in 0..3 {
        pixel[channel] = to_srgb(rgb[channel] * gain);
    }
}

/// Grade once, before uploading the encounter-owned copy of `frame.map_texture`.
/// The caller must establish that exact handle relationship; another image with
/// the same dimensions is not sufficient provenance. Use the actual built frame
/// and its built profile document, never a newer viewport or desired revision.
///
/// Returns the graded cell count. Unsupported or incomplete evidence leaves the
/// whole image untouched. Custom drawings that use this tile as artwork opt out
/// conservatively even when no complete placement is visible. A ground-only
/// reference is permitted: existing underlays intentionally sample that floor,
/// while their mask/geometry were already built from unchanged source images.
pub(super) fn grade_violet_floor(
    frame: &VisualWorldFrame,
    profiles: Option<&Document>,
    atlas: &mut Image,
) -> usize {
    let Some(profiles) = profiles else {
        return 0;
    };
    if !frame.active
        || frame.map_id.as_ref() != "VioletGym"
        || frame.validate().is_err()
        || custom_floor_drawing(profiles)
    {
        return 0;
    }
    let Some(width) = frame.grid_size.x.checked_mul(TILE_PIXELS) else {
        return 0;
    };
    let Some(height) = frame.grid_size.y.checked_mul(TILE_PIXELS) else {
        return 0;
    };
    let descriptor = &atlas.texture_descriptor;
    let Some(bytes) = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
    else {
        return 0;
    };
    if descriptor.dimension != TextureDimension::D2
        || descriptor.format != TextureFormat::Rgba8UnormSrgb
        || descriptor.size.width != width
        || descriptor.size.height != height
        || descriptor.size.depth_or_array_layers != 1
        || descriptor.mip_level_count != 1
        || descriptor.sample_count != 1
        || atlas.data.len() != bytes
    {
        return 0;
    }
    let linear = std::array::from_fn(|value| to_linear(value as u8));
    // A native floor repeats only a few palette colors. Keep this cache bounded
    // even if a caller supplies unusually varied source art.
    let mut palette = HashMap::<[u8; 3], [u8; 3]>::with_capacity(8);
    let mut count = 0;
    for tile in &frame.tiles {
        if !native_floor(&tile.source) {
            continue;
        }
        let x = tile.column as usize * TILE_PIXELS as usize;
        let y = tile.row as usize * TILE_PIXELS as usize;
        for row in y..y + TILE_PIXELS as usize {
            let start = (row * width as usize + x) * 4;
            let end = start + TILE_PIXELS as usize * 4;
            for pixel in atlas.data[start..end].chunks_exact_mut(4) {
                let before = [pixel[0], pixel[1], pixel[2]];
                if let Some(after) = palette.get(&before) {
                    pixel[..3].copy_from_slice(after);
                } else {
                    grade(pixel, &linear);
                    if palette.len() < 32 {
                        palette.insert(before, [pixel[0], pixel[1], pixel[2]]);
                    }
                }
            }
        }
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        prelude::{Handle, UVec2, Vec2},
        render::{render_asset::RenderAssetUsages, render_resource::Extent3d},
    };
    use crystal_render_api::VisualTile;
    use std::sync::Arc;

    fn source(block: u16, column: u8, row: u8, tile: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("elite_four_room"),
            metatile_id: block,
            subtile_column: column,
            subtile_row: row,
            tile_index: tile,
        }
    }

    fn fixture() -> (VisualWorldFrame, Image) {
        let sources = [
            source(0x2d, 0, 0, 0x01), // native floor
            source(0x1d, 0, 0, 0x20), // statue artwork
            source(0x19, 0, 2, 0x13), // pit lip
            source(0x7f, 0, 0, 0x01), // same tile, unrecognized drawing
        ];
        let frame = VisualWorldFrame {
            active: true,
            map_id: Arc::from("VioletGym"),
            map_texture: Handle::weak_from_u128(11),
            grid_size: UVec2::new(2, 2),
            tile_size: Vec2::splat(32.0),
            viewport_size: Vec2::splat(64.0),
            tiles: sources
                .into_iter()
                .enumerate()
                .map(|(i, source)| VisualTile {
                    column: i as u32 % 2,
                    row: i as u32 / 2,
                    source,
                    texture: Handle::weak_from_u128(20 + i as u128),
                    animation_frames: None,
                    priority: false,
                })
                .collect(),
            ..Default::default()
        };
        let mut atlas = Image::new_fill(
            Extent3d {
                width: 16,
                height: 16,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[200, 180, 160, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        for (i, pixel) in atlas.data.chunks_exact_mut(4).enumerate() {
            pixel[3] = i as u8;
        }
        (frame, atlas)
    }

    #[test]
    fn grade_only_exact_native_floor_rectangles_and_preserve_source_and_alpha() {
        let (mut frame, original) = fixture();
        // Coordinates own the atlas rectangles, not vector order or map origin.
        frame.tiles.reverse();
        frame.grid_origin = bevy::prelude::IVec2::new(-12, 7);
        let original_bytes = original.data.clone();
        let frame_before = frame.clone();
        let mut owned = original.clone();
        assert_eq!(
            grade_violet_floor(&frame, Some(&Document::default()), &mut owned),
            1
        );
        for (i, (before, after)) in original
            .data
            .chunks_exact(4)
            .zip(owned.data.chunks_exact(4))
            .enumerate()
        {
            if i % 16 < 8 && i / 16 < 8 {
                assert!(after[0] < before[0] && after[1] < before[1] && after[2] < before[2]);
                assert_eq!(after[3], before[3]);
            } else {
                assert_eq!(after, before, "non-floor texel {i}");
            }
        }
        assert_eq!(original.data, original_bytes);
        assert_eq!(frame, frame_before);
        assert_eq!(
            owned.texture_descriptor.size,
            original.texture_descriptor.size
        );
        assert_eq!(
            owned.texture_descriptor.format,
            original.texture_descriptor.format
        );
    }

    #[test]
    fn exact_floor_identity_rejects_wrong_phase_tileset_and_art() {
        for block in [0x15, 0x19, 0x1d, 0x1e, 0x2d, 0x33] {
            for y in 0..4 {
                for x in 0..4 {
                    let expected = match block {
                        0x2d => true,
                        0x19 => y < 2,
                        0x15 => y == 3,
                        0x33 => y >= 2,
                        0x1d => x >= 2,
                        0x1e => x < 2,
                        _ => unreachable!(),
                    };
                    let native = source(block, x, y, 0x01);
                    assert_eq!(native_floor(&native), expected, "{block:x} ({x},{y})");
                    let mut wrong = native.clone();
                    wrong.tile_index = 0x13;
                    assert!(!native_floor(&wrong));
                    wrong = native;
                    wrong.tileset_id = Arc::from("other");
                    assert!(!native_floor(&wrong));
                }
            }
        }
        assert!(!native_floor(&source(0x17, 0, 0, 0x01)));
        assert!(!native_floor(&source(0x2d, 4, 0, 0x01)));
    }

    #[test]
    fn more_than_32_floor_colors_are_fully_and_deterministically_graded() {
        let (frame, mut original) = fixture();
        for i in 0..64 {
            let offset = ((i / 8) * 16 + i % 8) * 4;
            original.data[offset..offset + 3].copy_from_slice(&[
                64 + i as u8,
                200 - i as u8,
                96 + i as u8,
            ]);
        }
        let linear = std::array::from_fn(|value| to_linear(value as u8));
        let mut expected = original.clone();
        // The uncached pixel transform is the reference for both sides of the
        // 32-color boundary. Non-floor bytes remain in this exact reference.
        for i in 0..64 {
            let offset = ((i / 8) * 16 + i % 8) * 4;
            grade(&mut expected.data[offset..offset + 4], &linear);
            assert_ne!(
                expected.data[offset..offset + 3],
                original.data[offset..offset + 3],
                "color {i} must be graded even after the cache fills"
            );
        }
        for _ in 0..2 {
            let mut owned = original.clone();
            assert_eq!(
                grade_violet_floor(&frame, Some(&Document::default()), &mut owned),
                1
            );
            assert_eq!(owned.data, expected.data);
        }
    }

    #[test]
    fn invalid_evidence_is_an_atomic_noop() {
        let (frame, atlas) = fixture();
        let profiles = Document::default();
        for case in 0..18 {
            let mut frame = frame.clone();
            let mut atlas = atlas.clone();
            let mut profiles = Some(&profiles);
            match case {
                0 => frame.active = false,
                1 => frame.map_id = Arc::from("OtherGym"),
                2 => frame.tiles[3].column = frame.tiles[2].column,
                3 => {
                    frame.tiles.pop();
                }
                4 => atlas.texture_descriptor.format = TextureFormat::Rgba8Unorm,
                5 => atlas.texture_descriptor.size.width += 1,
                6 => {
                    atlas.data.pop();
                }
                7 => atlas.texture_descriptor.size.depth_or_array_layers = 2,
                8 => atlas.texture_descriptor.mip_level_count = 2,
                9 => atlas.texture_descriptor.sample_count = 4,
                10 => profiles = None,
                11 => frame.map_texture = Handle::default(),
                12 => frame.tiles[0].texture = Handle::default(),
                13 => frame.tiles[3].column = frame.grid_size.x,
                14 => frame.tiles[3].row = frame.grid_size.y,
                15 => atlas.texture_descriptor.dimension = TextureDimension::D3,
                16 => frame.tiles[3].source.subtile_column = 4,
                17 => frame.grid_size.x = 0,
                _ => unreachable!(),
            }
            let before = atlas.data.clone();
            assert_eq!(
                grade_violet_floor(&frame, profiles, &mut atlas),
                0,
                "case {case}"
            );
            assert_eq!(atlas.data, before, "case {case}");
        }
    }

    #[test]
    fn custom_drawings_opt_out_but_native_floor_underlay_references_remain_eligible() {
        let (frame, atlas) = fixture();
        for (map, tiles, ground, expected) in [
            ("VioletGym", "[[1]]", 3, 0),
            ("VioletGym", "[[32]]", 1, 1),
            ("OtherGym", "[[1]]", 3, 1),
        ] {
            let document: Document = serde_json::from_str(&format!(
                r#"{{"objects":[{{"name":"Override","map":"{map}","tileset":"elite_four_room","metatile":45,"origin":[0,0],"tiles":{tiles},"ground":{ground},"top_pixels":0,"depth_pixels":1}}]}}"#,
            )).unwrap();
            let mut owned = atlas.clone();
            assert_eq!(
                grade_violet_floor(&frame, Some(&document), &mut owned),
                expected
            );
            if expected == 0 {
                assert_eq!(owned.data, atlas.data);
            }
        }
        let document: Document = serde_json::from_str(
            r#"{"objects":[{"name":"Shared override","tileset":"elite_four_room","metatile":45,"origin":[0,0],"tiles":[[1]],"ground":3,"top_pixels":0,"depth_pixels":1}]}"#,
        ).unwrap();
        let mut owned = atlas.clone();
        assert_eq!(grade_violet_floor(&frame, Some(&document), &mut owned), 0);
        assert_eq!(owned.data, atlas.data);
        for (maps, expected) in [("[\"VioletGym\",\"OtherGym\"]", 0), ("[\"OtherGym\"]", 1)] {
            let document: Document = serde_json::from_str(&format!(
                r#"{{"objects":[{{"name":"Scoped override","maps":{maps},"tileset":"elite_four_room","metatile":45,"origin":[0,0],"tiles":[[1]],"ground":3,"top_pixels":0,"depth_pixels":1}}]}}"#,
            )).unwrap();
            let mut owned = atlas.clone();
            assert_eq!(
                grade_violet_floor(&frame, Some(&document), &mut owned),
                expected
            );
            if expected == 0 {
                assert_eq!(owned.data, atlas.data);
            }
        }
    }

    #[test]
    fn grade_compresses_luminance_without_a_hue_shift_or_alpha_change() {
        let linear = std::array::from_fn(|value| to_linear(value as u8));
        let mut cream = [202, 194, 185, 137];
        let mut blue = [97, 110, 204, 255];
        let luminance = |rgb: &[u8]| {
            to_linear(rgb[0]) * 0.2126 + to_linear(rgb[1]) * 0.7152 + to_linear(rgb[2]) * 0.0722
        };
        let before_difference = luminance(&cream) - luminance(&blue);
        let before_blue = blue;
        grade(&mut cream, &linear);
        grade(&mut blue, &linear);
        assert!(luminance(&cream) - luminance(&blue) < before_difference);
        assert_eq!(cream[3], 137);
        assert_eq!(blue[3], 255);
        let gains =
            std::array::from_fn::<_, 3, _>(|i| to_linear(blue[i]) / to_linear(before_blue[i]));
        assert!(gains.iter().all(|gain| *gain < 1.0));
        assert!(
            gains.iter().copied().fold(f32::NEG_INFINITY, f32::max)
                - gains.iter().copied().fold(f32::INFINITY, f32::min)
                < 0.02
        );
        let mut black = [0, 0, 0, 91];
        grade(&mut black, &linear);
        assert_eq!(black, [0, 0, 0, 91]);
    }
}
