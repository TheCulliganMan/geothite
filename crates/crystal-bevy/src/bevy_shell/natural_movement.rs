//! Continuous field locomotion; all cell entries use the existing gameplay loop.
use super::*;
use crate::core::world::free_movement::FreeMovement;

#[derive(Default)]
pub(super) struct NaturalMovement {
    pub enabled: bool,
    position: FreeMovement,
    anchor: Option<(String, TilePosition)>,
    pending: Option<Direction>,
    facing: Option<Vec2>,
}

impl NaturalMovement {
    pub fn crossing(&self) -> bool {
        self.pending.is_some()
    }
    pub fn facing(&self, map: &str, tile: TilePosition) -> Option<Vec2> {
        (self.enabled
            && self
                .anchor
                .as_ref()
                .is_some_and(|(m, t)| m == map && *t == tile))
        .then_some(self.facing)
        .flatten()
    }
    pub fn displacement(&self, map: &str, tile: TilePosition) -> Vec2 {
        if !self.enabled
            || self
                .anchor
                .as_ref()
                .is_none_or(|(m, t)| m != map || *t != tile)
        {
            return Vec2::ZERO;
        }
        Vec2::new(self.position.offset[0], -self.position.offset[1])
            * (TILE_SIZE * f32::from(RENDER_TILES_PER_RUNTIME_TILE))
    }
}

pub(super) fn keyboard_vector(keys: &ButtonInput<KeyCode>) -> Vec2 {
    let axis =
        |positive, negative| i32::from(keys.pressed(positive)) - i32::from(keys.pressed(negative));
    Vec2::new(
        axis(KeyCode::ArrowRight, KeyCode::ArrowLeft) as f32,
        axis(KeyCode::ArrowDown, KeyCode::ArrowUp) as f32,
    )
    .clamp_length_max(1.0)
}

fn vector_direction(vector: Vec2) -> Option<Direction> {
    (vector.is_finite() && vector != Vec2::ZERO).then(|| {
        if vector.x.abs() > vector.y.abs() {
            if vector.x < 0.0 {
                Direction::Left
            } else {
                Direction::Right
            }
        } else if vector.y < 0.0 {
            Direction::Up
        } else {
            Direction::Down
        }
    })
}

fn button(direction: Direction) -> GameButton {
    match direction {
        Direction::Up => GameButton::Up,
        Direction::Down => GameButton::Down,
        Direction::Left => GameButton::Left,
        Direction::Right => GameButton::Right,
    }
}

/// Called only after the production input owner releases field movement.
pub(super) fn prepare(
    shell: &mut BevyRuntimeShell,
    vector: Vec2,
    buttons: &mut Vec<GameButton>,
) -> Result<bool> {
    if !shell.natural_movement.enabled
        || shell.player_walk_frame_ticks > 0
        || shell.visible_ledge_jump.is_some()
        || shell
            .shell
            .session()
            .overworld()
            .forced_movement_direction()
            .is_some()
        || (matches!(
            shell.shell.session().overworld().player.mode,
            MovementMode::Bike | MovementMode::Skate
        ) && shell
            .shell
            .session()
            .state()
            .flags
            .is_engine_flag_set("ENGINE_DOWNHILL")
            .unwrap_or(false))
    {
        shell.natural_movement = NaturalMovement {
            enabled: shell.natural_movement.enabled,
            ..Default::default()
        };
        return Ok(false);
    }
    let world = shell.shell.session().overworld();
    let carpet_direction = crate::core::world::collision::sample_collision(
        &world.map,
        &world.tileset,
        world.player.tile,
    )
    .and_then(|sample| crate::core::world::collision::directional_warp_facing(sample.permission));
    if let Some(direction) =
        carpet_direction.filter(|required| vector_direction(vector) == Some(*required))
    {
        // Edge carpets warp after an attempted, often blocked, outbound step.
        // Their authoritative input must not wait for a traversable neighbor.
        buttons.retain(|button| !is_direction_button(*button));
        buttons.insert(0, button(direction));
        shell.natural_movement = NaturalMovement {
            enabled: true,
            ..Default::default()
        };
        return Ok(false);
    }
    let anchor = (world.map.name.clone(), world.player.tile);
    if shell.natural_movement.anchor.as_ref() != Some(&anchor) {
        shell.natural_movement.position = FreeMovement::default();
        shell.natural_movement.anchor = Some(anchor);
    }
    let mut occupied = world.occupied_tiles_checked()?;
    // The controller retains an NPC's visible origin until its stride lands.
    // Include it even after the lower-level cell state commits its destination.
    occupied.extend(shell.object_walk_from.values().map(|tile| {
        crate::core::world::movement::OccupiedTile {
            tile: *tile,
            object_identifier: None,
        }
    }));
    let mut open = [false; 4];
    for (index, direction) in [
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
    ]
    .into_iter()
    .enumerate()
    {
        let mut player = world.player;
        let result = crate::core::world::movement::attempt_step_with_occupied_tiles(
            &mut player,
            direction,
            &world.map,
            &world.tileset,
            crate::core::world::movement::StepOptions {
                force_step_after_turn: true,
                ..Default::default()
            },
            &occupied,
        );
        open[index] = matches!(result, StepOutcome::Moved { .. });
        if let StepOutcome::BlockedByObject {
            object_identifier: Some(ref id),
            ..
        } = result
        {
            let strength = shell
                .shell
                .session()
                .state()
                .flags
                .is_engine_flag_set("ENGINE_STRENGTH_ACTIVE")
                .unwrap_or(false);
            open[index] = strength
                && world.objects.iter().any(|object| {
                    object.object_identifier.as_ref() == Some(id)
                        && object_movement_spawns_strength_dust(&object.spritemovedata)
                });
        }

        // Ledges retain their authored jump and landing controller.
        if !open[index] {
            let mut player = world.player;
            open[index] = matches!(
                crate::core::world::movement::attempt_ledge_jump_with_occupied_tiles(
                    &mut player,
                    direction,
                    &world.map,
                    &world.tileset,
                    crate::core::world::movement::StepOptions {
                        force_step_after_turn: true,
                        ..Default::default()
                    },
                    &occupied
                ),
                LedgeJumpOutcome::Jumped { .. }
            );
        }
    }
    let speed = if buttons.contains(&GameButton::B)
        && shell.shell.runtime().data().running_shoes != Some(false)
        && world.player.mode == MovementMode::Normal
    {
        0.25
    } else {
        0.125
    } * f32::from(world.player.mode.speed_multiplier());
    // Project against solid faces first: a blocked axis must not claim the
    // neighboring row and incorrectly block sliding along the open axis.
    let projected: [f32; 2] = std::array::from_fn(|axis| {
        (shell.natural_movement.position.offset[axis] + vector[axis] * speed).clamp(
            if open[axis * 2] { -1.0 } else { -0.22 },
            if open[axis * 2 + 1] { 1.0 } else { 0.22 },
        )
    });
    // The player footprint near a corner also occupies the adjacent row/column. Do
    // not cut diagonally through its solid destination or an NPC's cell.
    let spans = shell
        .natural_movement
        .position
        .offset
        .map(|offset| offset.abs() > 0.22);
    let first = match (spans[0], spans[1]) {
        (true, false) => 0,
        (false, true) => 1,
        _ => usize::from(vector.y.abs() > vector.x.abs()),
    };
    let mut resolved: [f32; 2] = std::array::from_fn(|axis| {
        shell.natural_movement.position.offset[axis].clamp(
            if open[axis * 2] { -1.0 } else { -0.22 },
            if open[axis * 2 + 1] { 1.0 } else { 0.22 },
        )
    });
    // Resolve the leading axis first. Testing both final projections at once
    // pins the player against an outside corner instead of letting them slide.
    for axis in [first, 1 - first] {
        let other = 1 - axis;
        let offset = resolved[other];
        if offset.abs() <= 0.22 {
            resolved[axis] = projected[axis];
            continue;
        }
        let side = if other == 0 {
            if offset < 0.0 {
                Direction::Left
            } else {
                Direction::Right
            }
        } else if offset < 0.0 {
            Direction::Up
        } else {
            Direction::Down
        };
        for sign in 0..2 {
            let direction = [
                Direction::Left,
                Direction::Right,
                Direction::Up,
                Direction::Down,
            ][axis * 2 + sign];
            let mut player = world.player;
            if let Some(tile) =
                crate::core::world::movement::checked_move_by_stride(player.tile, side, 1)
            {
                player.tile = tile;
                let outcome = crate::core::world::movement::attempt_step_with_occupied_tiles(
                    &mut player,
                    direction,
                    &world.map,
                    &world.tileset,
                    crate::core::world::movement::StepOptions {
                        force_step_after_turn: true,
                        ..Default::default()
                    },
                    &occupied,
                );
                open[axis * 2 + sign] &= matches!(outcome, StepOutcome::Moved { .. });
            } else {
                open[axis * 2 + sign] = false;
            }
        }
        resolved[axis] = projected[axis].clamp(
            if open[axis * 2] { -1.0 } else { -0.22 },
            if open[axis * 2 + 1] { 1.0 } else { 0.22 },
        );
    }
    if vector.length_squared() > 0.0 {
        shell.natural_movement.facing = Some(Vec2::new(vector.x, -vector.y).normalize());
    }
    let next = shell
        .natural_movement
        .position
        .advance(vector.to_array(), speed, open);
    shell.natural_movement.pending = next;
    buttons.retain(|button| !is_direction_button(*button));
    let turn = vector_direction(vector).filter(|direction| *direction != world.player.facing);
    // A turn-only production frame keeps A interactions aligned with the
    // direction shown by the body, including movement within a single cell.
    if let Some(direction) = next.or(turn) {
        buttons.insert(0, button(direction));
    }
    Ok(true)
}

pub(super) fn finish(shell: &mut BevyRuntimeShell, frame: &crystal_runtime::RuntimeOverworldFrame) {
    if overworld_frame_reaches_presentation_boundary(frame) || frame.ledge_jump.is_some() {
        // Talking in place freezes the continuous position. Authored arrivals
        // and jumps retain their existing centered landing presentation.
        if matches!(frame.movement, Some(StepOutcome::Moved { .. }))
            || frame.warp.is_some()
            || frame.connection.is_some()
        {
            shell.natural_movement.position = FreeMovement::default();
            shell.natural_movement.anchor = None;
        }
        shell.natural_movement.facing = None;
    } else if matches!(frame.movement, Some(StepOutcome::Moved { .. })) {
        if let Some(direction) = shell.natural_movement.pending.take() {
            shell.natural_movement.position.crossed(direction);
            let world = shell.shell.session().overworld();
            shell.natural_movement.anchor = Some((world.map.name.clone(), world.player.tile));
        }
    }
}

#[cfg(all(test, feature = "voxel-view"))]
mod tests {
    use super::*;
    fn app() -> App {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("content-packs/text-tui.crystalpack"));
        let asset_root = AssetRoot::new(path.parent().unwrap());
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path)
            .expect("external game pack");
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&asset_root, loaded).unwrap();
        let controller =
            VisibleShellController::new_game(asset_root, runtime, "CHRIS", None).unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(controller.shell)
            .insert_resource(NativeRtcSource::fixed(RuntimeRtcSample {
                date: GameDate::new(2000, 1, 2),
                hour: 12,
                minute: 0,
                second: 0,
            }))
            .insert_resource(RuntimeTickTimer::new(0.0))
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(crystal_voxel_view::VoxelViewSettings {
                enabled: true,
                ..Default::default()
            })
            .add_systems(
                Update,
                (apply_keyboard_input, apply_runtime_hotkeys).chain(),
            )
            .add_systems(PostUpdate, |mut keys: ResMut<ButtonInput<KeyCode>>| {
                keys.clear()
            });
        app
    }
    fn key(app: &mut App, key: KeyCode, down: bool) {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        if down {
            keys.press(key);
        } else {
            keys.release(key);
        }
    }
    #[test]
    fn natural_movement_stops_mid_cell_and_crosses_through_production_input() {
        let mut app = app();
        app.world_mut()
            .resource_mut::<BevyRuntimeShell>()
            .shell
            .set_runtime_journal_enabled(true);
        let start = app
            .world()
            .resource::<BevyRuntimeShell>()
            .shell
            .session()
            .overworld()
            .player
            .tile;
        key(&mut app, KeyCode::ArrowRight, true);
        app.update();
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        assert_eq!(shell.shell.session().overworld().player.tile, start);
        assert!(shell.natural_movement.position.offset[0] > 0.0);
        key(&mut app, KeyCode::ArrowRight, false);
        let offset = app
            .world()
            .resource::<BevyRuntimeShell>()
            .natural_movement
            .position
            .offset;
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .natural_movement
                .position
                .offset,
            offset
        );
        key(&mut app, KeyCode::ArrowRight, true);
        for _ in 0..10 {
            app.update();
        }
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        assert!(shell.shell.session().overworld().player.tile.x > start.x);
        assert_eq!(
            shell.player_walk_frame_ticks, 0,
            "free movement must not queue tile interpolation"
        );
        assert!(shell.natural_movement.position.offset[0].abs() < 0.5);
    }
    #[test]
    fn natural_movement_preserves_a_tap_released_before_the_tick() {
        let mut app = app();
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::ArrowRight);
            keys.release(KeyCode::ArrowRight);
        }
        app.update();
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        assert!(shell.natural_movement.position.offset[0] > 0.0);
        let stopped = shell.natural_movement.position.offset;
        app.update();
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .natural_movement
                .position
                .offset,
            stopped
        );
    }
    #[test]
    fn natural_movement_diagonal_crossings_do_not_insert_grid_turn_frames() {
        let mut app = app();
        let start = app
            .world()
            .resource::<BevyRuntimeShell>()
            .shell
            .session()
            .overworld()
            .player
            .tile;
        key(&mut app, KeyCode::ArrowRight, true);
        key(&mut app, KeyCode::ArrowUp, true);
        for _ in 0..7 {
            app.update();
        }
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        let tile = shell.shell.session().overworld().player.tile;
        assert_eq!(
            tile,
            TilePosition {
                x: start.x + 1,
                y: start.y - 1
            }
        );
        assert_eq!(shell.player_walk_frame_ticks, 0);
    }
    #[test]
    fn natural_movement_slides_past_an_outside_table_corner() {
        let mut app = app();
        key(&mut app, KeyCode::ArrowRight, true);
        key(&mut app, KeyCode::ArrowDown, true);
        for _ in 0..7 {
            app.update();
        }
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        assert_eq!(
            shell.shell.session().overworld().player.tile,
            TilePosition::new(4, 3)
        );
        assert_eq!(shell.natural_movement.position.offset[1], 0.22);
    }
    #[test]
    fn natural_movement_freezes_while_start_menu_owns_input() {
        let mut app = app();
        key(&mut app, KeyCode::ArrowDown, true);
        app.update();
        key(&mut app, KeyCode::ArrowDown, false);
        key(&mut app, KeyCode::Enter, true);
        app.update();
        key(&mut app, KeyCode::Enter, false);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.start_menu_cursor.is_some());
        let tile = shell.shell.session().overworld().player.tile;
        let offset = shell.natural_movement.position.offset;
        key(&mut app, KeyCode::ArrowRight, true);
        for _ in 0..20 {
            app.update();
        }
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        assert_eq!(shell.shell.session().overworld().player.tile, tile);
        assert_eq!(shell.natural_movement.position.offset, offset);
    }
    #[test]
    fn natural_movement_retains_authored_downhill_bike_control() {
        let mut app = app();
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.natural_movement.enabled = true;
        shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_engine_flag("ENGINE_DOWNHILL", true)
            .unwrap();
        shell.shell.session_mut().overworld_mut().player.mode = MovementMode::Bike;
        let mut buttons = vec![GameButton::Right];
        assert!(!prepare(&mut shell, Vec2::X, &mut buttons).unwrap());
        assert_eq!(buttons, vec![GameButton::Right]);
        assert_eq!(shell.natural_movement.position.offset, [0.0; 2]);
        shell.shell.session_mut().overworld_mut().player.mode = MovementMode::Normal;
        assert!(prepare(&mut shell, Vec2::X, &mut buttons).unwrap());
        assert!(shell.natural_movement.position.offset[0] > 0.0);
    }
    #[test]
    fn natural_movement_respects_an_npcs_retained_stride_origin() {
        let mut app = app();
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.natural_movement.enabled = true;
        shell
            .object_walk_from
            .insert("walking-npc".into(), TilePosition::new(4, 3));
        let mut buttons = Vec::new();
        for _ in 0..10 {
            prepare(&mut shell, Vec2::X, &mut buttons).unwrap();
        }
        assert_eq!(shell.natural_movement.position.offset[0], 0.22);
        assert!(!shell.natural_movement.crossing());
    }
    #[test]
    fn natural_movement_slides_along_a_solid_furniture_edge() {
        let mut app = app();
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.natural_movement.enabled = true;
        // This collision fixture sits north of the bedroom table. The open
        // east cell must remain reachable while the south face blocks input.
        shell.shell.session_mut().overworld_mut().player.tile = TilePosition::new(4, 3);
        let mut buttons = Vec::new();
        for _ in 0..7 {
            prepare(&mut shell, Vec2::ONE.normalize(), &mut buttons).unwrap();
        }
        assert!(shell.natural_movement.position.offset[0] > 0.5);
        assert_eq!(shell.natural_movement.position.offset[1], 0.22);
        assert_eq!(buttons, vec![GameButton::Right]);
    }
    #[test]
    fn natural_movement_passes_gentle_outbound_carpet_input_to_the_controller() {
        let mut app = app();
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.natural_movement.enabled = true;
        // An isolated collision fixture exercises an outbound carpet even
        // though the analog tilt has not generated any cardinal keyboard key.
        for metatile in &mut shell.shell.session_mut().overworld_mut().tileset.metatiles {
            metatile.collision = [crate::core::world::collision::permissions::WARP_CARPET_DOWN; 4];
        }
        let mut buttons = Vec::new();
        assert!(!prepare(&mut shell, Vec2::new(0.0, 0.2), &mut buttons).unwrap());
        assert_eq!(buttons, vec![GameButton::Down]);
        assert!(!shell.natural_movement.crossing());
    }
    #[test]
    fn natural_movement_is_disabled_in_2d() {
        let mut app = app();
        app.world_mut()
            .resource_mut::<crystal_voxel_view::VoxelViewSettings>()
            .enabled = false;
        key(&mut app, KeyCode::ArrowDown, true);
        app.update();
        assert!(
            !app.world()
                .resource::<BevyRuntimeShell>()
                .natural_movement
                .enabled
        );
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static BROWSER_STICK: std::cell::Cell<(f32,f32,bool)> = const { std::cell::Cell::new((0.0,0.0,false)) };
}

/// Browser integration transports raw axes; Rust owns movement and deadzones.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_set_movement_axes(x: f32, y: f32, dpad: bool) -> bool {
    BROWSER_STICK.with(|s| s.set((x, y, dpad)));
    analog_vector(Vec2::new(x, y)) != Vec2::ZERO
}

pub(super) fn analog_vector(raw: Vec2) -> Vec2 {
    let length = raw.length();
    if !length.is_finite() || length <= 0.18 {
        Vec2::ZERO
    } else {
        raw / length * ((length - 0.18) / 0.82).min(1.0)
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) fn browser_vector(yaw: f32) -> Option<Vec2> {
    BROWSER_STICK.with(|s| {
        let (x, y, dpad) = s.get();
        let raw = Vec2::new(x, -y);
        let vector = if dpad {
            raw.clamp_length_max(1.0)
        } else {
            analog_vector(raw)
        };
        (vector != Vec2::ZERO).then(|| {
            let rotated = if dpad {
                vector
            } else {
                Vec2::from_angle(yaw).rotate(vector)
            };
            Vec2::new(rotated.x, -rotated.y)
        })
    })
}

#[cfg(test)]
mod axis_tests {
    use super::*;
    #[test]
    fn natural_analog_deadzone_preserves_direction_and_speed() {
        assert_eq!(analog_vector(Vec2::new(0.1, 0.1)), Vec2::ZERO);
        let vector = analog_vector(Vec2::new(0.3, 0.4));
        assert!((vector.length() - (0.5 - 0.18) / 0.82).abs() < 1e-6);
        assert!((vector.normalize() - Vec2::new(0.6, 0.8)).length() < 1e-6);
        assert!((analog_vector(Vec2::ONE).length() - 1.0).abs() < 1e-6);
        assert_eq!(analog_vector(Vec2::new(f32::NAN, 0.0)), Vec2::ZERO);
    }
}
