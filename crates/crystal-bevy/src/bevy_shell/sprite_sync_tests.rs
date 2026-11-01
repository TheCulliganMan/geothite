//! Verify that presentation changes still publish while idle frames stay clean.

use std::path::PathBuf;
use std::sync::Arc;

use bevy::prelude::{
    App, Assets, Changed, Color, Entity, Image, IntoScheduleConfigs, Last, Query, ResMut, Resource,
    Sprite, Update, Visibility, With,
};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_runtime::CrystalRuntime;

use super::{
    BevyRuntimeShell, BevyShellConfig, BevyShellStart, Direction, PlayerMarker, PlayerSpriteFrames,
    PlayfieldPriorityTile, PoisonFlashOverlay, RenderedViewport, ScreenFadeOverlay,
    ScriptFadeColor, ScriptFadeDirection, TilePosition, VisibleObjectSprite, VisibleScreenFade,
};

#[derive(Resource, Default)]
struct Changes(Vec<(usize, usize)>);

fn collect_changes(
    sprites: Query<Entity, Changed<Sprite>>,
    visibility: Query<Entity, (With<PlayfieldPriorityTile>, Changed<Visibility>)>,
    mut changes: ResMut<Changes>,
) {
    changes
        .0
        .push((sprites.iter().count(), visibility.iter().count()));
}

#[test]
fn idle_sprite_sync_preserves_change_detection_and_effect_transitions() {
    let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            root.repository_root
                .join("content-packs/text-tui.crystalpack")
        });
    let loaded = read_loaded_verified_compiled_game_pack(path).expect("load external pack");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)
        .expect("construct production runtime");
    let mut shell = super::initialize_bevy_runtime_shell(
        root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: 14,
            map_name: "PlayersHouse2F".to_string(),
            tile_x: 3,
            tile_y: 3,
        },
        BevyShellConfig::default(),
    )
    .expect("initialize sprite presentation fixture");
    shell.player_walk_frame_ticks = 0;
    shell.screen_fade = None;
    shell.poison_flash_frames_remaining = 0;
    let mut images = Assets::<Image>::default();
    let standing = images.add(Image::default());
    let walking = images.add(Image::default());
    let mut app = App::new();
    app.insert_resource(images)
        .insert_resource(shell)
        .init_resource::<RenderedViewport>()
        .init_resource::<Changes>()
        .add_systems(
            Update,
            (
                super::sync_visible_player_sprite,
                super::sync_visible_object_sprites,
                super::render_screen_fade_overlay,
                super::render_poison_flash_overlay,
            )
                .chain(),
        )
        .add_systems(Last, collect_changes);
    let player = app
        .world_mut()
        .spawn((
            Sprite::from_image(standing.clone()),
            PlayerMarker,
            PlayerSpriteFrames {
                source_id: Arc::from("player"),
                #[cfg(feature = "voxel-view")]
                directional_frames: Vec::new(),
                standing: standing.clone(),
                walking: Some(walking.clone()),
                mirror_walking: true,
            },
        ))
        .id();
    let object = app
        .world_mut()
        .spawn((
            Sprite::from_image(standing.clone()),
            VisibleObjectSprite {
                #[cfg(feature = "voxel-view")]
                directional_frames: Vec::new(),
                world_facing: Direction::Down,
                object_index: 0,
                object_identifier: Some("SPRITE_SYNC_TEST".to_string()),
                source_id: Arc::from("npc"),
                model_palette: None,
                above_priority: false,
                standing: standing.clone(),
                walking: Some(walking.clone()),
                mirror_walking: true,
                animated: true,
            },
        ))
        .id();
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            ..Default::default()
        },
        ScreenFadeOverlay,
    ));
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(230.0 / 255.0, 173.0 / 255.0, 1.0, 0.0),
            ..Default::default()
        },
        PoisonFlashOverlay,
    ));
    let priority = app
        .world_mut()
        .spawn((Visibility::Inherited, PlayfieldPriorityTile))
        .id();
    app.update();
    app.update();
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.player_walk_frame_ticks = 1;
        shell.player_walk_stride = true;
        shell.player_walk_mirror_stride = true;
        shell
            .object_walk_from
            .insert("SPRITE_SYNC_TEST".to_string(), TilePosition::new(3, 3));
        shell
            .object_walk_phases
            .insert("SPRITE_SYNC_TEST".to_string(), 3);
        let mut fade = VisibleScreenFade::new(ScriptFadeColor::Black, ScriptFadeDirection::Out, 8);
        fade.alpha = 64;
        shell.screen_fade = Some(fade);
        shell.poison_flash_frames_remaining = 2;
    }
    app.update();
    app.update();
    for entity in [player, object] {
        let sprite = app
            .world()
            .entity(entity)
            .get::<Sprite>()
            .expect("source sprite");
        assert!(sprite.flip_x);
        assert_eq!(sprite.image, walking);
    }
    assert_eq!(
        app.world().entity(priority).get::<Visibility>(),
        Some(&Visibility::Hidden)
    );
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.player_walk_frame_ticks = 0;
        shell.object_walk_from.clear();
        shell.object_walk_phases.clear();
        shell.screen_fade = None;
        shell.poison_flash_frames_remaining = 0;
    }
    app.update();
    app.update();
    for entity in [player, object] {
        let sprite = app
            .world()
            .entity(entity)
            .get::<Sprite>()
            .expect("source sprite");
        assert!(!sprite.flip_x);
        assert_eq!(sprite.image, standing);
    }
    assert_eq!(
        app.world().entity(priority).get::<Visibility>(),
        Some(&Visibility::Inherited)
    );
    assert_eq!(
        app.world().resource::<Changes>().0,
        vec![(4, 1), (0, 0), (4, 1), (0, 0), (4, 1), (0, 0)]
    );
}
