//! Synthetic authored acreage shared by progressive-residency regressions.
use bevy::prelude::{Handle, IVec2, UVec2, Vec2};
use crystal_render_api::{VisualSourceRegion, VisualTile, VisualTileSource, VisualWorldFrame};

pub(crate) fn grove_window() -> (
    VisualWorldFrame,
    std::sync::Arc<crate::mesh::StreamedGroves>,
) {
    let (mut frame, _) = connected_frames(IVec2::new(40, 0));
    frame.grid_size = UVec2::new(5, 4);
    frame.grid_origin = IVec2::ZERO;
    frame.tiles.retain(|tile| tile.column < 5 && tile.row < 4);
    for tile in &mut frame.tiles {
        if tile.column < 4 {
            tile.source.metatile_id = 0x05;
            tile.source.subtile_column = tile.column as u8;
            tile.source.subtile_row = tile.row as u8;
            tile.source.tile_index = (if tile.row == 0 {
                0x1e
            } else if tile.row == 3 {
                0x3e
            } else {
                0x2e
            }) + (tile.column % 2) as u16;
        } else {
            tile.source.tile_index = 0x05;
        }
    }
    let profiles = crate::live_profiles::LiveProfiles::default();
    let source = crate::mesh::begin_runtime_terrain_mesh_with_profiles(
        &frame,
        &crate::mesh::TerrainImageSamples::default(),
        &profiles.document,
    )
    .expect("authored tree terrain")
    .into_streamed()
    .streamed_groves
    .expect("deferred source groves");
    assert!(source.len() > 0);
    (frame, source)
}

pub(crate) fn connected_frames(offset: IVec2) -> (VisualWorldFrame, VisualWorldFrame) {
    let texture = Handle::Uuid(
        bevy::asset::uuid::Uuid::from_u128(1),
        std::marker::PhantomData,
    );
    let mut built = VisualWorldFrame {
        active: true,
        map_id: "RouteA".into(),
        map_texture: texture.clone(),
        grid_size: UVec2::splat(28),
        tile_size: Vec2::splat(8.0),
        viewport_size: Vec2::splat(8.0),
        grid_origin: offset / 2 + IVec2::splat(6),
        source_regions: vec![
            VisualSourceRegion {
                map_id: "RouteA".into(),
                origin: IVec2::ZERO,
                size: UVec2::splat(40),
            },
            VisualSourceRegion {
                map_id: "RouteB".into(),
                origin: offset,
                size: UVec2::splat(40),
            },
        ]
        .into(),
        ..Default::default()
    };
    for row in 0..28 {
        for column in 0..28 {
            built.tiles.push(VisualTile {
                row,
                column,
                texture: texture.clone(),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: "johto".into(),
                    metatile_id: 1,
                    tile_index: 1,
                    subtile_column: 0,
                    subtile_row: 0,
                },
            });
        }
    }
    let mut live = built.clone();
    live.map_id = "RouteB".into();
    live.grid_origin -= offset;
    live.source_regions = vec![
        VisualSourceRegion {
            map_id: "RouteB".into(),
            origin: IVec2::ZERO,
            size: UVec2::splat(40),
        },
        VisualSourceRegion {
            map_id: "RouteA".into(),
            origin: -offset,
            size: UVec2::splat(40),
        },
    ]
    .into();
    (built, live)
}
