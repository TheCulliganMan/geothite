//! Production source preparation must retain native city architecture and banks.
use bevy::prelude::{Assets, Image, IntoScheduleConfigs, Update};
use crystal_render_api::VisualWorldFrame;

use super::super::{BevyRuntimeShell, publish_visual_world_frame, render_playfield};
use super::interior_life::{fixture, load};
use super::menu_render_test_app;

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK containing the Minnesota cities"]
fn city_service_facades_and_rocky_banks_use_their_real_source_without_state_changes() {
    let (root, runtime, _) = load();
    let mut services = 0;
    let mut banks = 0;
    for at in [(15, 46), (55, 32), (21, 34)] {
        let name = "MnStPaul";
        let controller = fixture(&root, &runtime, name, at);
        // Bridges require the actual live three-layer compositor. No preload
        // substitute or altered bridge snapshot is used in this regression.
        let mut app = menu_render_test_app(controller.shell);
        app.init_resource::<VisualWorldFrame>()
            .add_systems(Update, publish_visual_world_frame.after(render_playfield));
        app.update();
        app.update();
        let frame = app.world().resource::<VisualWorldFrame>();
        assert!(frame.active);
        let before = app
            .world()
            .resource::<BevyRuntimeShell>()
            .shell
            .snapshot()
            .expect("before mesh")
            .state_checksum;
        let mesh = crystal_voxel_view::audit_terrain_mesh_with_images(
            frame,
            app.world().resource::<Assets<Image>>(),
        )
        .expect("city models from live source layers");
        let width = frame.grid_size.x as usize;
        for tile in &frame.tiles {
            let source = &tile.source;
            let index = tile.row as usize * width + tile.column as usize;
            if source.tileset_id.as_ref() != "kanto" {
                continue;
            }
            if matches!(source.metatile_id, 0x72 | 0x73)
                && source.subtile_column == 0
                && source.subtile_row == 0
            {
                let origin = index - width * 4 - 4;
                if frame.tiles[origin].source.metatile_id == 0x20 {
                    services += 1;
                    assert_eq!(
                        mesh.authored_cells[index],
                        Some("outdoor/kanto_brick_office"),
                        "{name}: the native flat-roof service must retain its masonry grammar"
                    );
                }
            }
            if matches!(
                source.metatile_id,
                0x1d | 0x1e | 0x1f | 0x2d | 0x2e | 0x64 | 0x65
            ) && matches!(source.tile_index, 0x33 | 0x54)
            {
                banks += 1;
                assert_eq!(
                    mesh.authored_cells[index],
                    Some("outdoor/kanto-shore"),
                    "{name}: native bank cap must retain rock ownership"
                );
                assert_eq!(mesh.footing_heights[index], 0., "bank support stays fixed");
            }
        }
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .shell
                .snapshot()
                .expect("after mesh")
                .state_checksum,
            before
        );
    }
    assert!(
        services >= 3,
        "exercise real service facades across the city"
    );
    assert!(banks >= 20, "exercise river bends and complete bank strips");
}
