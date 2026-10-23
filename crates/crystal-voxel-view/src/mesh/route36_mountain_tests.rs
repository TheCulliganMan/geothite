//! Exercise the real narrow ridge, including its native tree border.

use std::collections::HashMap;

use bevy::prelude::{IVec2, UVec2, Vec2, Vec3};
use crystal_render_api::{VisualSourceRegion, VisualTileSource};

use super::Relief;
use crate::mesh::{GridGeometry, TerrainMeshData, modeled_dungeons, outdoor_stone};
use crate::profile::{self, shape_for_source};

#[test]
#[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
fn external_route36_ridge_rises_above_the_trainer_and_keeps_native_contacts_and_paint() {
    let path = std::env::var("CRYSTAL_RENDER_TEST_PACK").expect("set the completed external pack");
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path)
        .expect("load the verified external pack");
    let pack = loaded.pack();
    let map = &pack.data().maps["Route36"];
    let atlas = map.attributes.tileset_name.as_str();
    let layout = &pack.runtime_files()[&format!("data/tilesets/{atlas}_metatiles.bin")];
    let native_size = UVec2::new(
        u32::from(map.attributes.width) * 4,
        u32::from(map.attributes.height) * 4,
    );
    let padding = 8;
    let width = native_size.x as usize + padding * 2;
    let height = native_size.y as usize + padding * 2;
    let mut sources = Vec::with_capacity(width * height);
    for row in 0..height {
        for column in 0..width {
            let x = column as i32 - padding as i32;
            let z = row as i32 - padding as i32;
            let block = if x >= 0 && z >= 0 && x < native_size.x as i32 && z < native_size.y as i32
            {
                map.blocks[z as usize / 4 * usize::from(map.attributes.width) + x as usize / 4]
            } else {
                u16::from(map.attributes.border_block)
            };
            let local_x = x.rem_euclid(4) as u8;
            let local_z = z.rem_euclid(4) as u8;
            sources.push(VisualTileSource {
                tileset_id: atlas.into(),
                metatile_id: block,
                subtile_column: local_x,
                subtile_row: local_z,
                tile_index: layout
                    [usize::from(block) * 16 + usize::from(local_z) * 4 + usize::from(local_x)]
                    as u16,
            });
        }
    }
    let frame = crate::mesh::tests::frame(width as u32, height as u32, sources);
    let cells: Vec<_> = frame.tiles.iter().collect();
    let mut shapes: Vec<_> = cells
        .iter()
        .map(|tile| shape_for_source(&tile.source))
        .collect();
    crate::rock_platform::resolve_rock_platform_tiers(&cells, &mut shapes, width);
    crate::elevation::resolve_authored_mountain_tiers(&mut shapes, width, height);
    let origin = [-(padding as i32); 2];
    let regions = [VisualSourceRegion {
        map_id: "Route36".into(),
        origin: IVec2::ZERO,
        size: native_size,
    }];
    let control_regions = [VisualSourceRegion {
        map_id: "Route44".into(),
        ..regions[0].clone()
    }];
    for tile_size in [8., 32.] {
        let grid = GridGeometry {
            width,
            height,
            tile_width: tile_size,
            tile_height: tile_size,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; cells.len()],
            footing_heights: shapes
                .iter()
                .map(|shape| shape.surface_height(tile_size))
                .collect(),
            ..Default::default()
        };
        let field = Relief::with_ground(
            &cells,
            &shapes,
            &grid,
            origin,
            None,
            &regions,
            Some(&mesh.footing_heights),
        );
        let point = |x: f32, z: f32| Vec2::new(x + padding as f32, z + padding as f32);
        let crest = point(104., 32.);
        let margin = point(104., 28.);
        assert!(
            field.geometry_height(crest.x, crest.y) > tile_size * 4.,
            "the actual narrow ridge must stand above the three-cell trainer"
        );
        assert!(
            field.geometry_height(crest.x, crest.y)
                > field.geometry_height(margin.x, margin.y) + tile_size * 2.,
            "the center must rise above the outer edge"
        );
        let mut claimed = vec![false; cells.len()];
        modeled_dungeons::append_terrain(
            &mut mesh,
            "Route36",
            &cells,
            &shapes,
            &grid,
            &mut claimed,
            origin,
        );
        outdoor_stone::sculpt_rock_rims_on_map(
            &mut mesh, "Route36", &cells, &shapes, &grid, origin,
        );
        outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
        let before = mesh.clone();
        let mut control = mesh.clone();
        super::finish_with_source_regions(
            &mut mesh, "Route36", &cells, &shapes, &grid, origin, None, &regions,
        );
        super::finish_with_source_regions(
            &mut control,
            "Route36",
            &cells,
            &shapes,
            &grid,
            origin,
            None,
            &control_regions,
        );
        assert_eq!(mesh.solid.indices, control.solid.indices);
        assert_eq!(mesh.solid.uvs, control.solid.uvs);
        assert_eq!(
            mesh.solid.colors, control.solid.colors,
            "retain the approved mineral colors"
        );
        assert_eq!(mesh.footing_heights, before.footing_heights);
        assert_eq!(mesh.authored_cells, before.authored_cells);
        let mut raised = 0;
        let mut shared = HashMap::new();
        for ((after, previous), native) in mesh
            .solid
            .positions
            .iter()
            .zip(&control.solid.positions)
            .zip(&before.solid.positions)
        {
            assert!(after.iter().all(|coordinate| coordinate.is_finite()));
            assert_eq!([after[0], after[2]], [previous[0], previous[2]]);
            assert!(after[1] >= previous[1] - 0.001);
            if native[1] <= profile::GROUND_HEIGHT {
                assert_eq!(after, previous, "native ground contacts must remain seated");
            }
            if let Some(previous) = shared.insert(native.map(f32::to_bits), *after) {
                assert!(
                    Vec3::from_array(previous).distance(Vec3::from_array(*after)) < 0.001,
                    "shared native cap and wall vertices must stay joined"
                );
            }
            raised += usize::from(after[1] > previous[1] + tile_size * 0.5);
        }
        assert!(
            raised > 100,
            "exercise the real native cap and wall geometry"
        );
        println!(
            "Route36 at scale {tile_size}: {raised} raised vertices; native paint, footprint and footing preserved"
        );
    }
}
