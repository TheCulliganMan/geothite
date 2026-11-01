//! Continue a finite mountain clearing's water border without repeating its
//! decorative corner as islands. The native map and connected routes stay put.
use bevy::prelude::UVec2;
use crystal_render_api::VisualWorldFrame;

pub(crate) fn replacements(frame: &VisualWorldFrame) -> Vec<(UVec2, UVec2)> {
    // This isolated outdoor map has native $2d border fill and no map
    // connections. The same drawing inside its real bounds is actual shoreline.
    if frame.map_id.as_ref() != "MountMoonSquare" {
        return Vec::new();
    }
    let Some(size) = frame.source_map_size_core_tiles else {
        return Vec::new();
    };
    let (w, h) = (frame.grid_size.x as usize, frame.grid_size.y as usize);
    if w == 0 || h == 0 || size.x == 0 || size.y == 0 {
        return Vec::new();
    }
    let mut cells = vec![None; w * h];
    for t in &frame.tiles {
        let (x, y) = (t.column as usize, t.row as usize);
        if x >= w || y >= h || cells[y * w + x].replace(t).is_some() {
            return Vec::new();
        }
    }
    let mut result = Vec::new();
    for y in 0..h.saturating_sub(3) {
        for x in 0..w.saturating_sub(3) {
            let complete = (0..16).all(|j| {
                let Some(t) = cells[(y + j / 4) * w + x + j % 4] else {
                    return false;
                };
                let s = &t.source;
                let px = x as i64 + (j % 4) as i64 + i64::from(frame.grid_origin.x);
                let py = y as i64 + (j / 4) as i64 + i64::from(frame.grid_origin.y);
                (px < 0 || py < 0 || px >= i64::from(size.x) * 2 || py >= i64::from(size.y) * 2)
                    && s.tileset_id.as_ref() == "kanto"
                    && s.metatile_id == 0x2d
                    && s.subtile_column as usize == j % 4
                    && s.subtile_row as usize == j / 4
                    && s.tile_index == if j == 3 { 0x33 } else { 0x14 }
            });
            if !complete {
                continue;
            }
            let donor = cells[(y + 1) * w + x].unwrap();
            if !crate::cave_water::live_outdoor_water(donor, &frame.map_id) {
                continue;
            }
            result.push((
                UVec2::new((x + 3) as u32, y as u32),
                UVec2::new(x as u32, (y + 1) as u32),
            ));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crystal_render_api::{VisualTile, VisualTileSource};
    fn frame() -> VisualWorldFrame {
        let texture = Handle::<Image>::default();
        let other = Handle::<Image>::Uuid(
            bevy::asset::uuid::Uuid::from_u128(91),
            std::marker::PhantomData,
        );
        VisualWorldFrame {
            map_id: "MountMoonSquare".into(),
            grid_size: UVec2::splat(12),
            source_map_size_core_tiles: Some(UVec2::splat(2)),
            grid_origin: IVec2::splat(-4),
            tile_size: Vec2::splat(8.),
            tiles: (0..144)
                .map(|i| {
                    let (x, y) = (i % 12, i / 12);
                    let wet = !(x % 4 == 3 && y % 4 == 0);
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        source: VisualTileSource {
                            tileset_id: "kanto".into(),
                            metatile_id: 0x2d,
                            subtile_column: (x % 4) as u8,
                            subtile_row: (y % 4) as u8,
                            tile_index: if wet { 0x14 } else { 0x33 },
                        },
                        texture: texture.clone(),
                        animation_frames: wet.then(|| vec![texture.clone(), other.clone()].into()),
                        priority: false,
                    }
                })
                .collect(),
            ..Default::default()
        }
    }
    #[test]
    fn only_complete_animated_border_fill_borrows_water_and_native_map_stays_intact() {
        let f = frame();
        let before = f.clone();
        let selected = replacements(&f);
        assert_eq!(selected.len(), 8);
        for (bank, donor) in selected {
            let p = bank.as_ivec2() + f.grid_origin;
            assert!(p.x < 0 || p.y < 0 || p.x >= 4 || p.y >= 4);
            assert_eq!(bank.x / 4, donor.x / 4);
            assert_eq!(bank.y / 4, donor.y / 4);
        }
        assert_eq!(f, before);
        for fault in 0..5 {
            let mut f = frame();
            match fault {
                0 => f.map_id = "MountMoon".into(),
                1 => f.source_map_size_core_tiles = None,
                2 => {
                    for t in &mut f.tiles {
                        t.source.tileset_id = "johto".into()
                    }
                }
                3 => {
                    for t in &mut f.tiles {
                        t.animation_frames = None
                    }
                }
                _ => {
                    for t in &mut f.tiles {
                        if t.source.tile_index == 0x33 {
                            t.source.tile_index = 0x39;
                        }
                    }
                }
            }
            assert!(replacements(&f).is_empty(), "unproven border: {fault}");
        }
        let mut cropped = frame();
        cropped.tiles.remove(0);
        assert_eq!(
            replacements(&cropped).len(),
            7,
            "a clipped block cannot authorize a corner"
        );
    }
    #[test]
    fn borrowed_border_corners_do_not_seed_square_shore_foam() {
        let f = frame();
        let mut expected = f.clone();
        for (bank, donor) in replacements(&f) {
            let d = expected.tiles[(donor.y * 12 + donor.x) as usize].clone();
            let t = &mut expected.tiles[(bank.y * 12 + bank.x) as usize];
            t.source.tile_index = 0x14;
            t.texture = d.texture;
            t.animation_frames = d.animation_frames;
        }
        assert_eq!(
            crate::water_surface::shoreline(&f).unwrap().data,
            crate::water_surface::shoreline(&expected).unwrap().data
        );
    }
}
