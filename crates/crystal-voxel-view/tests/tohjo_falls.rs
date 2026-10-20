//! Production geometry and moving Surf contact, read from the external pack.
use bevy::prelude::{Handle, UVec2, Vec2};
use crystal_render_api::{VisualTile, VisualTileSource, VisualWorldFrame};
use crystal_voxel_view::{build_terrain_mesh, resolved_footing_height};

#[test]
fn external_tohjo_basins_join_both_falls_and_support_a_continuous_ride() {
    let Some(path) = std::env::var_os("CRYSTAL_RENDER_TEST_PACK") else {
        eprintln!("external pack test skipped: CRYSTAL_RENDER_TEST_PACK unset");
        return;
    };
    let pack =
        crystal_assets::read_verified_compiled_game_pack(path).expect("verified external pack");
    let map = pack.data().map_module("TohjoFalls").expect("Tohjo Falls");
    let layout = &pack.runtime_files()["data/tilesets/cave_metatiles.bin"];
    let width = usize::from(map.attributes.width) * 4;
    let height = usize::from(map.attributes.height) * 4;
    let mut tiles = Vec::with_capacity(width * height);
    for row in 0..height {
        for column in 0..width {
            let block = map.blocks[(row / 4) * usize::from(map.attributes.width) + column / 4];
            tiles.push(VisualTile {
                column: column as u32,
                row: row as u32,
                texture: Handle::Uuid(
                    bevy::asset::uuid::Uuid::from_u128((row * width + column + 10) as u128),
                    std::marker::PhantomData,
                ),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: "cave".into(),
                    metatile_id: block,
                    subtile_column: (column % 4) as u8,
                    subtile_row: (row % 4) as u8,
                    tile_index: u16::from(
                        layout[usize::from(block) * 16 + (row % 4) * 4 + column % 4],
                    ),
                },
            });
        }
    }
    let native = VisualWorldFrame {
        active: true,
        map_id: "TohjoFalls".into(),
        map_texture: Handle::Uuid(
            bevy::asset::uuid::Uuid::from_u128(1),
            std::marker::PhantomData,
        ),
        grid_size: UVec2::new(width as u32, height as u32),
        tile_size: Vec2::splat(8.),
        viewport_size: Vec2::new(width as f32 * 8., height as f32 * 8.),
        tiles,
        ..Default::default()
    };
    let mesh = build_terrain_mesh(&native).expect("native cascade terrain");
    // Both complete source waterfalls, on opposite sides of the central island.
    for (column, row, span) in [(36, 12, 12), (16, 16, 8)] {
        for y in row..row + 8 {
            for x in column..column + span {
                let source = &native.tiles[y * width + x].source;
                assert_eq!((source.metatile_id, source.tile_index), (0x2c, 0x40));
            }
        }
        for x in column..column + span {
            assert_eq!(mesh.footing_heights[(row - 1) * width + x], 32.);
            assert_eq!(mesh.footing_heights[(row + 8) * width + x], 0.);
        }
        let x = (column as f32 + span as f32 * 0.5 - width as f32 * 0.5) * 8.;
        let top = height as f32 * 4. - row as f32 * 8.;
        let samples = (0..=128)
            .map(|step| {
                let foot = Vec2::new(x, top - step as f32 * 0.5);
                resolved_footing_height(&native, foot, &mesh.footing_heights).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(samples[0], 32.);
        assert_eq!(*samples.last().unwrap(), 0.);
        assert!(
            samples
                .windows(2)
                .all(|v| v[0] >= v[1] && v[0] - v[1] < 0.6),
            "Surf must follow the actual sheet without jumping at source-cell boundaries"
        );
    }
    let mut ordinary = native.clone();
    ordinary.map_id = "UnionCave1F".into();
    let baseline = build_terrain_mesh(&ordinary).unwrap();
    for (i, tile) in native.tiles.iter().enumerate() {
        if (24..36).contains(&(i % width)) && (12..16).contains(&(i / width)) {
            assert_eq!(
                mesh.footing_heights[i], 40.,
                "the entire island approach joins its rock crown"
            );
            continue;
        }
        if !matches!(tile.source.tile_index, 0x14 | 0x40) {
            assert_eq!(
                mesh.footing_heights[i], baseline.footing_heights[i],
                "the island, paths, exits and rock retain their original support"
            );
        }
    }
}
