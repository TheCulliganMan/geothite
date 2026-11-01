//! Production source-sprite images/poses, with native input after initial setup.
//! The independent CPU compositor checks the emitted RGBA/depth stack; this is
//! not a GPU readback or a claim of projected/3D bridge acceptance.
use super::*;

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::{ScriptFlagCommand, apply_script_flag_mutation};

fn bridge_runtime() -> (AssetRoot, CrystalRuntime) {
    let path = std::env::var_os("CRYSTAL_BRIDGE_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .expect("set CRYSTAL_BRIDGE_RENDER_TEST_PACK to the published MillBridge pack")
        .canonicalize()
        .expect("actual bridge pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        crystal_assets::read_loaded_verified_compiled_game_pack(path).expect("strict actual pack"),
    )
    .expect("native bridge runtime");
    (root, runtime)
}

fn bridge_position(controller: &mut VisibleShellController) -> TilePosition {
    controller
        .snapshot()
        .expect("native bridge position")
        .overworld
        .tile
}

fn bridge_fixture(root: &AssetRoot, runtime: &CrystalRuntime) -> BevyRuntimeShell {
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: "MnMinneapolis".into(),
            tile_x: 31,
            tile_y: 10,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..default()
        },
    )
    .expect("legal original Floor launch");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("explicit initial trainer");
    let mut partner = runtime
        .data()
        .create_pokemon("MEW", 80, Dv::default())
        .expect("native legal Surf partner");
    partner.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: runtime.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    apply_script_flag_mutation(
        state,
        ScriptFlagCommand {
            command: "setflag".into(),
            flag_id: "ENGINE_FOGBADGE".into(),
            source_script: "BridgeRendererExplicitInitialFixture".into(),
            command_index: 0,
        },
    )
    .expect("consistent native Surf badge");
    state.random_state = Default::default();
    mark_runtime_snapshot_dirty(&mut controller.shell);
    // Initial fixture ends. All gameplay changes below use production input.
    controller.press(GameButton::Right).expect("face river");
    controller.press(GameButton::A).expect("native Surf prompt");
    for _ in 0..128 {
        if controller.snapshot().expect("mount mode").overworld.mode == MovementMode::Surf {
            controller
                .wait_frames(60)
                .expect("native Surf mount presentation");
            break;
        }
        controller
            .press(GameButton::A)
            .expect("native Surf confirmation");
        controller
            .wait_frames(16)
            .expect("native Surf source continuation");
    }
    assert_eq!(bridge_position(&mut controller), TilePosition::new(32, 10));
    for (button, target) in [
        (GameButton::Right, TilePosition::new(33, 10)),
        (GameButton::Right, TilePosition::new(34, 10)),
        (GameButton::Down, TilePosition::new(34, 11)),
    ] {
        for _ in 0..4 {
            if bridge_position(&mut controller) == target {
                break;
            }
            controller.press(button).expect("native mouth approach");
            controller
                .wait_frames(16)
                .expect("native approach presentation");
        }
        assert_eq!(bridge_position(&mut controller), target);
    }
    assert_eq!(
        controller
            .snapshot()
            .expect("actual mounted approach")
            .overworld
            .mode,
        MovementMode::Surf
    );
    controller.shell
}

#[derive(Clone)]
struct BridgeLayer {
    transform: Transform,
    size: Vec2,
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    flip_x: bool,
    flip_y: bool,
    player: bool,
    bridge: bool,
}

impl BridgeLayer {
    fn pixel(&self, x: usize, y: usize) -> Option<&[u8]> {
        // Actual sprite custom-size and native nearest-neighbor coordinates.
        let left = PLAYFIELD_WIDTH * 0.5 + self.transform.translation.x - self.size.x * 0.5;
        let top = PLAYFIELD_HEIGHT * 0.5 - self.transform.translation.y - self.size.y * 0.5;
        let u = (x as f32 + 0.5 - left) / self.size.x;
        let v = (y as f32 + 0.5 - top) / self.size.y;
        if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
            return None;
        }
        let sx = ((if self.flip_x { 1.0 - u } else { u }) * self.width as f32)
            .floor()
            .min((self.width - 1) as f32) as usize;
        let sy = ((if self.flip_y { 1.0 - v } else { v }) * self.height as f32)
            .floor()
            .min((self.height - 1) as f32) as usize;
        Some(&self.pixels[(sy * self.width + sx) * 4..][..4])
    }
}

fn bridge_frame(world: &mut World) -> (Vec<u8>, usize, usize) {
    let mut query = world.query_filtered::<(
        &Sprite,
        &Transform,
        Has<PlayerMarker>,
        Has<PlayfieldBridgeTile>,
    ), Or<(
        With<PlayfieldTile>,
        With<PlayerMarker>,
        With<VisibleObjectSprite>,
    )>>();
    let images = world.resource::<Assets<Image>>();
    let mut layers = query
        .iter(world)
        .map(|(sprite, transform, player, bridge)| {
            assert!(
                sprite.rect.is_none() && sprite.texture_atlas.is_none(),
                "source sprite fixture"
            );
            let image = images.get(&sprite.image).expect("actual production image");
            BridgeLayer {
                transform: *transform,
                size: sprite.custom_size.expect("actual source display size"),
                width: image.width() as usize,
                height: image.height() as usize,
                pixels: image.data.clone().expect("production CPU source RGBA"),
                flip_x: sprite.flip_x,
                flip_y: sprite.flip_y,
                player,
                bridge,
            }
        })
        .collect::<Vec<_>>();
    layers.sort_by(|a, b| {
        a.transform
            .translation
            .z
            .total_cmp(&b.transform.translation.z)
    });
    let player = layers
        .iter()
        .find(|layer| layer.player)
        .expect("real player OAM");
    let bridge = layers
        .iter()
        .find(|layer| layer.bridge)
        .expect("real bridge surface");
    assert!(player.transform.translation.z < bridge.transform.translation.z);
    // Compare the actual actor/bridge decision crop rather than spending a
    // second renderer's worth of CPU on every offscreen/irrelevant LCD pixel.
    let left = ((PLAYFIELD_WIDTH * 0.5 + player.transform.translation.x - player.size.x * 0.5)
        .floor() as i32)
        .clamp(0, PLAYFIELD_WIDTH as i32) as usize;
    let top = ((PLAYFIELD_HEIGHT * 0.5 - player.transform.translation.y - player.size.y * 0.5)
        .floor() as i32)
        .clamp(0, PLAYFIELD_HEIGHT as i32) as usize;
    let right = (left + player.size.x.ceil() as usize + 1).min(PLAYFIELD_WIDTH as usize);
    let bottom = (top + player.size.y.ceil() as usize + 1).min(PLAYFIELD_HEIGHT as usize);
    let width = right - left;
    let mut rgba = vec![0; width * (bottom - top) * 4];
    let (mut covered, mut total) = (0, 0);
    for y in top..bottom {
        for x in left..right {
            let destination = &mut rgba[((y - top) * width + x - left) * 4..][..4];
            for layer in &layers {
                if let Some(pixel) = layer.pixel(x, y) {
                    assert!(
                        pixel[3] == 0 || pixel[3] == 255,
                        "native source binary alpha"
                    );
                    if pixel[3] == 255 {
                        destination.copy_from_slice(pixel);
                    }
                }
            }
            if player.pixel(x, y).is_some_and(|pixel| pixel[3] == 255) {
                total += 1;
                if let Some(pixel) = bridge.pixel(x, y).filter(|pixel| pixel[3] == 255) {
                    covered += 1;
                    assert_eq!(
                        destination, pixel,
                        "opaque native bridge pixels cover Surf OAM"
                    );
                }
            }
        }
    }
    (rgba, covered, total)
}

#[test]
#[ignore = "requires published native MillBridge pack; actual source-image app regression"]
fn bridge_native_source_partial_occlusion_matches_full_and_retained_draws() {
    let (root, runtime) = bridge_runtime();
    let mut retained = integrated_shell_test_app(bridge_fixture(&root, &runtime));
    let mut full = integrated_shell_test_app(bridge_fixture(&root, &runtime));
    retained.update();
    full.update();
    let mut partial = 0;
    for frame in 0..144 {
        for app in [&mut retained, &mut full] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::ArrowDown);
        }
        // Presentation-only invalidation; never edit gameplay after fixture.
        full.world_mut()
            .resource_mut::<RenderedViewport>()
            .shell_render_key = None;
        retained.update();
        full.update();
        let a = bridge_frame(retained.world_mut());
        let b = bridge_frame(full.world_mut());
        assert_eq!(
            a.0, b.0,
            "actual source RGBA differs retained/full at frame {frame}"
        );
        assert_eq!((a.1, a.2), (b.1, b.2));
        if a.1 > 0 && a.1 < a.2 {
            partial += 1;
        }
        let shell = retained.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none());
        let snapshot = shell.shell.snapshot().expect("native bridge journey");
        assert_eq!(snapshot.overworld.mode, MovementMode::Surf);
        if snapshot.overworld.tile.y >= 18 && shell.player_walk_frame_ticks == 0 {
            break;
        }
    }
    assert!(
        partial > 0,
        "must observe actual Surf OAM partially below bridge at a mouth"
    );
    let shell = retained.world().resource::<BevyRuntimeShell>();
    assert!(
        shell
            .shell
            .snapshot()
            .expect("native south mouth")
            .overworld
            .tile
            .y
            >= 18,
        "native inputs must traverse the entire underpass"
    );
}
