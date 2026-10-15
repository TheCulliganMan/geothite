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
    let foot = crate::actor_foot(actor);
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
    if frame.map_id.as_ref() == "CeruleanGym" {
        Vec4::new(1.35, 0.10, 0.12, 0.12)
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
                "OlivinePort" | "VermilionPort" | "CinnabarIsland"
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
    let mut distance = vec![REACH; width.checked_mul(height)?];
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
                distance[(i / columns * SAMPLES + y) * width + i % columns * SAMPLES + x] = 0.0;
            }
        }
    }
    let step = 1.0 / SAMPLES as f32;
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
    // Signed half-sample offsets put the zero crossing at the actual bank,
    // instead of outlining source-cell centers. Linear sampling joins cells.
    let pixels = distance
        .into_iter()
        .flat_map(|d| {
            let signed = (d - step * 0.5).clamp(-REACH, REACH);
            let encoded = ((signed / REACH * 0.5 + 0.5) * 255.0).round() as u8;
            [encoded, 0, 0, 255]
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

    fn depth(image: &Image, x: usize, y: usize) -> f32 {
        let byte = image.data.as_ref().unwrap()[(y * image.width() as usize + x) * 4];
        (byte as f32 / 255.0 * 2.0 - 1.0) * REACH
    }

    #[test]
    fn bank_distance_resolves_shallows_and_islands_without_frame_edge_foam() {
        let frame = frame(12, 8, |x, y| x == 0 || (x == 5 && y == 4));
        let original = frame.clone();
        let image = shoreline(&frame).unwrap();
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
        frame.map_id = "UnionCaveB1F".into();
        assert_eq!(style(&frame).w, 0.0, "caves cannot reflect an exposed sky");
        frame.map_id = "CeruleanGym".into();
        assert!(style(&frame).y < lake.y);
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
        assert_eq!(initial, Vec4::new(3.5, 2.5, 1.1, 1.0));
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
        frame.tiles[11].animation_frames = None;
        assert_eq!(
            contact(&frame),
            Vec4::ZERO,
            "unavailable source art cannot invent water contact"
        );
    }
}
