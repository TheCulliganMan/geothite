//! Observe the actual actor and grass sprites on the Fen's northern beach.
use bevy::input::InputPlugin;
use bevy::prelude::*;
use crystal_core::systems::field_moves::CANOE_ITEM;
use crystal_core::world::map::TilePosition;

use super::super::super::{
    BevyRuntimeShell, GrassRustleMarker, PLAYFIELD_LEFT, PLAYFIELD_TOP, PlayerMarker,
    PlayerSpriteFrames,
};
use super::super::shore_watch_bank_walk::stride;
use super::super::{
    canoe, interior_life, menu_render_test_app, pokemon_overworld_context_canvas_scaled,
};

#[derive(Resource, Default)]
struct GrassFrames(Vec<(TilePosition, bool, Option<AssetId<Image>>)>);

/// Crystal's upper two 8×8 OAM pieces retain their own priority. Only the
/// lower pair inherit the grass flag. Check the final LCD against the pack's
/// walking/standing image at the actor's interpolated on-screen position.
fn check_visible_trainer(world: &mut World) -> (usize, usize) {
    let mut players =
        world.query_filtered::<(&PlayerSpriteFrames, &Sprite, &Transform), With<PlayerMarker>>();
    let (frames, sprite, transform) = players.single(world).expect("one native actor");
    let native = sprite.image.clone();
    assert!(
        native == frames.standing || frames.walking.as_ref() == Some(&native),
        "the underlying actor uses the original standing or action image"
    );
    let size = sprite.custom_size.expect("native actor dimensions");
    assert_eq!(size, Vec2::splat(64.0));
    let x0 = ((transform.translation.x - size.x / 2.0 - PLAYFIELD_LEFT) / 4.0).round() as u32;
    let y0 = ((PLAYFIELD_TOP - transform.translation.y - size.y / 2.0) / 4.0).round() as u32;
    let flip = sprite.flip_x;
    let source_id = frames.source_id.clone();
    let tile = world
        .resource::<BevyRuntimeShell>()
        .shell
        .session()
        .overworld()
        .player
        .tile;
    world.resource_scope(|world, images: Mut<Assets<Image>>| {
        let lcd = pokemon_overworld_context_canvas_scaled(world, &images, 4);
        let texture = images.get(&native).expect("original 16×16 actor frame");
        let bytes = texture.data.as_deref().expect("source actor pixels");
        assert_eq!(bytes.len(), 16 * 16 * 4);
        let mut hidden_lower = 0;
        let mut visible_lower = 0;
        let (pixels, remainder) = bytes.as_chunks::<4>();
        assert!(remainder.is_empty(), "native RGBA pixels");
        for (i, original) in pixels.iter().enumerate() {
            if original[3] == 0 {
                continue;
            }
            let column = (i % 16) as u32;
            let row = (i / 16) as u32;
            let displayed = lcd.get_pixel(x0 + if flip { 15 - column } else { column }, y0 + row);
            if row < 8 || source_id.starts_with("mn_canoe_") {
                assert_eq!(
                    displayed.0, *original,
                    "{source_id} pixel ({column},{row}) covered at {tile:?}, LCD origin ({x0},{y0})"
                );
            } else if displayed.0 == *original {
                visible_lower += 1;
            } else {
                hidden_lower += 1;
            }
        }
        (hidden_lower, visible_lower)
    })
}

fn check_grass_frame(world: &mut World) {
    check_visible_trainer(world);
}

#[test]
#[ignore = "requires MnLayeredFen in CRYSTAL_CANOE_TEST_PACK"]
fn fen_canoe_lands_in_grass_before_the_native_rustle_begins() {
    let (root, runtime) = canoe::load();
    let room = &runtime.data().maps["MnLayeredFen"];
    let first_grass = TilePosition::new(18, 15);
    let shore = TilePosition::new(19, 15);
    let lake = TilePosition::new(19, 16);
    for (tile, expected) in [
        (first_grass, "TALL_GRASS"),
        (shore, "TALL_GRASS"),
        (lake, "WATER"),
    ] {
        assert_eq!(
            interior_life::collision(&runtime, room, (tile.x, tile.y)),
            Some(expected)
        );
    }
    let mut fixture = interior_life::fixture(&root, &runtime, "MnLayeredFen", (17, 15));
    let state = fixture.shell.shell.session_mut().state_mut();
    assert!(
        state
            .bag
            .add_item(&runtime.data().items[CANOE_ITEM], 1)
            .expect("owned canoe")
    );
    state
        .flags
        .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
        .expect("isolate the visual effect");
    let mut app = menu_render_test_app(fixture.shell);
    app.add_plugins(InputPlugin);
    app.init_resource::<GrassFrames>();
    app.add_systems(
        Last,
        |shell: Res<BevyRuntimeShell>,
         players: Query<&PlayerSpriteFrames, With<PlayerMarker>>,
         grass: Query<&Sprite, With<GrassRustleMarker>>,
         mut trace: ResMut<GrassFrames>| {
            let Ok(player) = players.single() else { return };
            let rider = player.source_id.as_ref() == "mn_canoe_chris";
            let effect = grass.single().ok().map(|sprite| sprite.image.id());
            trace
                .0
                .push((shell.shell.session().overworld().player.tile, rider, effect));
        },
    );
    app.update();
    app.update();
    app.add_systems(Last, check_grass_frame);
    stride(&mut app, first_grass, "chris", "mn_canoe_chris", false);
    let ordinary = &app.world().resource::<GrassFrames>().0;
    assert!(
        ordinary
            .iter()
            .any(|&(tile, rider, effect)| tile == first_grass && !rider && effect.is_some()),
        "walking into grass stirs the blades"
    );
    stride(&mut app, shore, "chris", "mn_canoe_chris", false);
    stride(&mut app, lake, "chris", "mn_canoe_chris", true);
    if let Some(directory) = std::env::var_os("MN_SHORE_MOTION_DIR") {
        super::super::record_overworld_frames(
            &mut app,
            &std::path::Path::new(&directory).join("grass-landing"),
        );
    }
    app.world_mut().resource_mut::<GrassFrames>().0.clear();
    stride(&mut app, shore, "chris", "mn_canoe_chris", false);
    for _ in 0..9 {
        app.update();
    }
    let frames = &app.world().resource::<GrassFrames>().0;
    let water_ticks = frames
        .iter()
        .filter(|&&(tile, rider, _)| tile == shore && rider)
        .count();
    assert!(
        water_ticks > 1,
        "expected the boat to remain visible before the northward step finishes"
    );
    assert!(
        frames
            .iter()
            .all(|&(_, rider, effect)| !rider || effect.is_none()),
        "grass drew while the canoe was still on the water: {frames:?}"
    );
    let mut rustles = frames.iter().filter_map(|&(tile, rider, effect)| {
        if tile == shore && !rider {
            effect
        } else {
            None
        }
    });
    assert_eq!(
        rustles.clone().count(),
        6,
        "the six ticks of landing grass belong to the walking trainer"
    );
    let first = rustles.next().expect("landing should stir the grass");
    assert!(
        rustles.any(|frame| frame != first),
        "both native grass frames should appear on land"
    );
    assert!(
        frames.last().is_some_and(|(_, _, frame)| frame.is_none()),
        "grass must settle"
    );
    let (hidden, visible) = check_visible_trainer(app.world_mut());
    assert!(
        hidden == 0 && visible > 0,
        "the low pebble lip must leave the parked trainer's boots visible"
    );
}

#[test]
#[ignore = "requires the Layered Fen in CRYSTAL_CANOE_TEST_PACK"]
fn fen_south_lake_ordinary_grass_shows_the_head_and_masks_the_lower_body() {
    let (root, runtime) = canoe::load();
    let room = &runtime.data().maps["MnLayeredFen"];
    // This inland patch uses the same unmodified four grass glyphs as Route 29.
    let mut c = interior_life::fixture(&root, &runtime, "MnLayeredFen", (21, 9));
    let state = c.shell.shell.session_mut().state_mut();
    state.player_gender = 1;
    state
        .flags
        .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
        .expect("isolate walking art");
    let mut app = menu_render_test_app(c.shell);
    app.add_plugins(InputPlugin);
    app.update();
    app.update();
    app.add_systems(Last, check_grass_frame);
    for (x, y) in [(21, 10), (20, 10), (20, 11), (21, 11), (21, 10)] {
        assert_eq!(
            interior_life::collision(&runtime, room, (x, y)),
            Some("TALL_GRASS")
        );
        stride(
            &mut app,
            TilePosition::new(x, y),
            "kris",
            "mn_canoe_kris",
            false,
        );
    }
    // The transient brush has settled. The lower OAM pieces still show
    // through palette-zero holes, with the native grass blades in front.
    for _ in 0..9 {
        app.update();
    }
    let (hidden, visible) = check_visible_trainer(app.world_mut());
    assert!(
        hidden > 0 && visible > 0,
        "stationary native grass must partly cover the lower body: {hidden} covered, {visible} visible"
    );
}

#[test]
#[ignore = "requires MnFormerShores in CRYSTAL_CANOE_TEST_PACK"]
fn former_shores_prairie_grass_leaves_boots_visible_over_low_gravel() {
    // Identical collision with different authored ink underfoot.
    check_regional_grass_walk(
        "MnFormerShores",
        (13, 12),
        [(14, 12, false), (14, 13, true)],
        "former-shores-grass",
    );
}

#[test]
#[ignore = "requires MnDetroitLakesApproach in CRYSTAL_CANOE_TEST_PACK"]
fn detroit_lakes_ordinary_johto_grass_keeps_both_trainers_visible() {
    check_regional_grass_walk(
        "MnDetroitLakesApproach",
        (11, 12),
        [(12, 12, false), (13, 12, false)],
        "detroit-lakes-grass",
    );
}

fn check_regional_grass_walk(
    map: &str,
    from: (i16, i16),
    steps: [(i16, i16, bool); 2],
    capture: &str,
) {
    let (root, runtime) = canoe::load();
    let room = &runtime.data().maps[map];
    for (x, y, _) in steps {
        assert_eq!(
            interior_life::collision(&runtime, room, (x, y)),
            Some("TALL_GRASS")
        );
    }
    for (gender, walker, rider) in [(0, "chris", "mn_canoe_chris"), (1, "kris", "mn_canoe_kris")] {
        let mut c = interior_life::fixture(&root, &runtime, map, from);
        let state = c.shell.shell.session_mut().state_mut();
        state.player_gender = gender;
        state
            .flags
            .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
            .expect("isolate walking art");
        let mut app = menu_render_test_app(c.shell);
        app.add_plugins(InputPlugin);
        app.update();
        app.update();
        app.add_systems(Last, check_grass_frame);
        if gender == 1
            && let Some(dir) = std::env::var_os("MN_SHORE_MOTION_DIR")
        {
            super::super::record_overworld_frames(
                &mut app,
                &std::path::PathBuf::from(dir).join(capture),
            );
        }
        for (x, y, gravel) in steps {
            stride(&mut app, TilePosition::new(x, y), walker, rider, false);
            for _ in 0..9 {
                app.update();
            }
            let (hidden, visible) = check_visible_trainer(app.world_mut());
            assert!(visible > 0, "lower body disappeared at {map} ({x},{y})");
            assert_eq!(
                hidden == 0,
                gravel,
                "incorrect {walker} foreground at {map} ({x},{y}): {hidden} covered pixels"
            );
        }
    }
}
