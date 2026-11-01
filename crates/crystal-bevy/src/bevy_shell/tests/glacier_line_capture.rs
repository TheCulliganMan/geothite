//! Immutable ticket captures and explicitly labeled train presentation fixtures.
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Assets, Commands, Image, World};

use crate::bevy_shell::tests::render_pc_audit_canvas;
use crate::bevy_shell::{
    RenderedTilesetArt, VisibleShellController, commit_presented_fullscreen_solid,
    source_map_text_palette, spawn_scene_dialog, spawn_visible_magnet_train,
};

pub(super) fn capture(c: &mut VisibleShellController, label: &str, train: bool) {
    let Some(directory) = std::env::var_os("CRYSTAL_GLACIER_RENDER_DIR") else {
        return;
    };
    let directory = PathBuf::from(directory);
    std::fs::create_dir_all(&directory).expect("retained native LCD evidence directory");
    let snapshot = c.snapshot().expect("actual controller-owned surface");
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut images = Assets::<Image>::default();
    let mut art = RenderedTilesetArt::default();
    let mut commands = Commands::new(&mut queue, &world);
    let palette =
        source_map_text_palette(&snapshot, &c.shell.asset_root).expect("native text palette");
    commit_presented_fullscreen_solid(
        &mut commands,
        &mut art,
        [palette[0][0], palette[0][1], palette[0][2], 255],
        3.0,
        &mut images,
    )
    .expect("isolated native LCD backdrop");
    if train {
        spawn_visible_magnet_train(
            &mut commands,
            &c.shell,
            &mut art,
            &c.shell.asset_root,
            &mut images,
        )
        .expect("actual native train frame");
    } else {
        spawn_scene_dialog(
            &mut commands,
            &snapshot,
            &c.shell,
            &mut art,
            &c.shell.asset_root,
            &mut images,
        )
        .expect("actual native ticket menu");
    }
    queue.apply(&mut world);
    assert_eq!(art.font_error, None);
    let canvas = render_pc_audit_canvas(&mut world, &images, label);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let stem = format!("{label}-{}-{nonce}", std::process::id());
    let original = directory.join(format!("{stem}-original.png"));
    let lcd = directory.join(format!("{stem}-160x144.png"));
    assert!(
        !original.exists() && !lcd.exists(),
        "all original and derivative renders are immutable"
    );
    canvas
        .save(&original)
        .expect("retain full-resolution production sprites");
    image::imageops::resize(&canvas, 160, 144, image::imageops::FilterType::Nearest)
        .save(&lcd)
        .expect("retain native-size LCD derivative");
    let record = serde_json::json!({
        "scope":if train {
            "Explicit train presentation fixture advanced by the native animation clock; CPU composition of production Bevy sprites, no live-journey or GPU capture claim"
        } else {
            "CPU composition of production Bevy ticket sprites from the actual controller state; isolated LCD backdrop, no terrain/GPU capture claim"
        },
        "map":snapshot.overworld.map_name,"train":train,
        "source_dimensions":[canvas.width(),canvas.height()],"native_dimensions":[160,144],
        "original":original.file_name().and_then(|s| s.to_str()).expect("ASCII capture name"),
        "native_lcd":lcd.file_name().and_then(|s| s.to_str()).expect("ASCII capture name")
    });
    std::fs::write(
        directory.join(format!("{stem}.json")),
        serde_json::to_vec_pretty(&record).expect("capture metadata"),
    )
    .expect("retain capture provenance");
}

#[test]
#[ignore = "requires the published service in CRYSTAL_INTERIOR_PACK; optional CRYSTAL_GLACIER_RENDER_DIR"]
fn glacier_line_native_train_art_fixture_both_directions() {
    use crate::bevy_shell::{VisibleMagnetTrain, advance_visible_magnet_train};

    let (root, runtime, stops) = super::load();
    for direction in [-1, 1] {
        let mut c = super::setup(&runtime, &root, &stops, 3, true, 31);
        // Presentation setup only. This does not prove a trip or mutate story.
        c.shell.visible_magnet_train = Some(VisibleMagnetTrain {
            direction,
            hold_position: 64 * direction,
            final_position: -96 * direction,
            position: 96 * direction,
            offset: 96 * direction,
            player_x: if direction == 1 { -4 } else { 180 },
            player_sprite_visible: false,
            player_sprite_frame: 0,
            player_sprite_duration: 0,
            wait_counter: 0,
            phase: 0,
            arrival_sfx_played: false,
        });
        for _ in 0..512 {
            if c.shell
                .visible_magnet_train
                .as_ref()
                .is_some_and(|a| a.phase == 3)
            {
                break;
            }
            advance_visible_magnet_train(&mut c.shell).expect("native visual-fixture clock");
        }
        assert_eq!(
            c.shell
                .visible_magnet_train
                .as_ref()
                .expect("retained frame")
                .phase,
            3
        );
        capture(
            &mut c,
            &format!("train-visual-fixture-direction-{direction}"),
            true,
        );
    }
}
