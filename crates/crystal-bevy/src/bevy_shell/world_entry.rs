// The production transition clock supplies effects for the retained 3D field.
// This is presentation extraction only; no scene or gameplay clock advances.
fn battle_transition_outro_step(transition: VisibleBattleTransition) -> Option<(usize, usize)> {
    let prefix = if transition.trainer_battle { 4 } else { 3 };
    let (between, frames) = match (transition.cave_environment, transition.stronger_enemy) {
        (true, false) => (2, 15), (true, true) => (1, 9),
        (false, false) => (2, 61), (false, true) => (2, 21),
    };
    Some((usize::from(transition.frame).checked_sub(prefix + 75 + between)? + 1, frames))
}

fn battle_transition_black_cells(transition: VisibleBattleTransition) -> [bool; 360] {
    let mut cells = [false; 360];
    let Some((step, frames)) = battle_transition_outro_step(transition) else { return cells; };
    if step > frames { return [true; 360]; }
    match (transition.cave_environment, transition.stronger_enemy) {
        (true, true) => {
            let boxes = step.min(9);
            let width = 2 + boxes * 2;
            let height = boxes * 2;
            for y in (18 - height) / 2..(18 + height) / 2 {
                for x in (20 - width) / 2..(20 + width) / 2 { cells[y * 20 + x] = true; }
            }
        }
        (false, true) => {
            let mut seed = 0_u32;
            for _ in 0..step.min(16) * 12 {
                for _ in 0..360 {
                    seed = (seed * 9_301 + 49_297) % 233_280;
                    let y = (seed * 18 / 233_280) as usize;
                    seed = (seed * 9_301 + 49_297) % 233_280;
                    let x = (seed * 20 / 233_280) as usize;
                    if !cells[y * 20 + x] { cells[y * 20 + x] = true; break; }
                }
            }
        }
        (false, false) => {
            for wedge in 0..((step + 2) / 3).min(20) {
                for (x, y) in battle_transition_wedge_cells(wedge) { cells[y * 20 + x] = true; }
            }
        }
        (true, false) => {}
    }
    cells
}

fn battle_transition_world_effect(
    transition: VisibleBattleTransition,
    dark: bool,
    strength: f32,
    root: &AssetRoot,
) -> Result<crystal_render_api::VisualWorldEntryEffect> {
    let bgp = battle_transition_bgp(transition, dark);
    let mut effect = crystal_render_api::VisualWorldEntryEffect {
        active: true,
        dark: f32::from(bgp & 3) / 3.0 * strength,
        light: (1.0 - f32::from((bgp >> 6) & 3) / 3.0) * strength,
        ..default()
    };
    for (index, filled) in battle_transition_black_cells(transition).into_iter().enumerate() {
        if filled { effect.black_cells[index / 128][index / 32 % 4] |= 1 << (index % 32); }
    }
    if transition.cave_environment && !transition.stronger_enemy {
        if let Some((step, frames)) = battle_transition_outro_step(transition).filter(|(step, frames)| step <= frames) {
            let mut counter = 0_u8;
            let mut offset = 0_u8;
            let mut amplitude = 0_u8;
            for _ in 0..step {
                amplitude = counter;
                let previous = offset;
                offset = offset.wrapping_add(1);
                counter = counter.wrapping_add(previous);
            }
            let _ = frames;
            effect.line_x_offsets = std::array::from_fn(|y|
                visible_battle_anim_sine((y as u8).wrapping_mul(2), amplitude) as i8);
        }
    }
    if transition.trainer_battle && transition.frame >= 2 {
        let (tile, palette) = battle_transition_trainer_art(root, dark)?;
        for (y, row) in BATTLE_TRANSITION_POKEBALL.iter().enumerate() {
            for (x, byte) in row.bytes().enumerate().filter(|(_, byte)| *byte == b'X') {
                let _ = byte;
                let index = (y + 1) * 20 + x + 2;
                effect.trainer_cells[index / 128][index / 32 % 4] |= 1 << (index % 32);
            }
        }
        for y in 0..8 {
            for x in 0..8 {
                let bit = 7 - x;
                let index = (tile[y * 2] >> bit & 1) | ((tile[y * 2 + 1] >> bit & 1) << 1);
                let pixel = y * 8 + x;
                effect.trainer_pixels[pixel / 16] |= u32::from(index) << (pixel % 16 * 2);
            }
        }
        effect.trainer_palette = std::array::from_fn(|index| {
            let color = |rgb: [u8; 3]| Color::srgb_u8(rgb[0], rgb[1], rgb[2]).to_linear().to_vec4();
            color(palette[index]).lerp(color(palette[((bgp >> (index * 2)) & 3) as usize]), strength)
        });
    }
    Ok(effect)
}

fn publish_visual_world_entry_effect(
    shell: Res<BevyRuntimeShell>,
    settings: Res<crystal_voxel_view::VoxelViewSettings>,
    frame: Res<crystal_render_api::VisualWorldFrame>,
    lighting: Res<crystal_render_api::VisualWorldLighting>,
    flash_mode: Res<crystal_render_api::BattleFlashMode>,
    mut published: ResMut<crystal_render_api::VisualWorldEntryEffect>,
) {
    let active = settings.enabled && frame.active && shell.visible_battle_transition.is_some()
        && !matches!(shell.pending_overworld_step_boundary,
            Some(PendingOverworldStepBoundary::WildBattle));
    let next = if active {
        let transition = shell
            .visible_battle_transition
            .expect("active entry has a transition");
        let dark = *lighting == crystal_render_api::VisualWorldLighting::Dark;
        let strength = if *flash_mode == crystal_render_api::BattleFlashMode::Reduced {
            0.25
        } else {
            1.0
        };
        match battle_transition_world_effect(transition, dark, strength, &shell.asset_root) {
            Ok(effect) => effect,
            Err(error) => {
                bevy::log::warn!("3D battle transition artwork unavailable: {error:#}");
                crystal_render_api::VisualWorldEntryEffect::default()
            }
        }
    } else {
        crystal_render_api::VisualWorldEntryEffect::default()
    };
    if *published != next { *published = next; }
}

#[cfg(test)]
mod world_entry_tests {
    use super::*;

    #[test]
    fn battle_transition_world_masks_match_every_classic_wipe_frame() {
        for cave in [false, true] { for strong in [false, true] { for trainer in [false, true] {
            let mut transition = VisibleBattleTransition { frame: 0, cave_environment: cave,
                stronger_enemy: strong, trainer_battle: trainer };
            for tick in 0..visible_battle_transition_total_frames(&transition) {
                transition.frame = tick;
                let expected = battle_transition_black_cells(transition);
                let mut world = World::new();
                let mut queue = bevy::ecs::world::CommandQueue::default();
                spawn_visible_battle_transition(&mut Commands::new(&mut queue, &world), transition, None, None);
                queue.apply(&mut world);
                let actual: Vec<_> = world.query::<(&Sprite, &Transform)>().iter(&world)
                    .filter(|(sprite, _)| sprite.color == Color::BLACK)
                    .map(|(sprite, transform)| (sprite.custom_size.unwrap(), transform.translation.truncate())).collect();
                for y in 0..18 { for x in 0..20 {
                    let (px, py) = render_tile_playfield_position(x as i16, y as i16);
                    let drawn = actual.iter().any(|(size, center)| {
                        let delta = (Vec2::new(px, py) - *center).abs();
                        delta.x < size.x * 0.5 && delta.y < size.y * 0.5
                    });
                    assert_eq!(expected[y * 20 + x], drawn,
                        "cave={cave} strong={strong} trainer={trainer} frame={tick} cell=({x},{y})");
                }}
            }
        }}}
    }

    #[test]
    fn battle_transition_world_trainer_pixels_match_the_compiled_source_texture() {
        let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap());
        let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(PathBuf::from)
            .unwrap_or_else(|| root.repository_root.join("content-packs/text-tui.crystalpack"));
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
        let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
        let shell = initialize_bevy_runtime_shell(root, runtime,
            BevyShellStart::NewGameAtRuntimeTile { spawn_identifier, map_name: "Route29".into(),
                tile_x: 50, tile_y: 9 }, BevyShellConfig::default()).unwrap();
        let tiles = vec![BattleTransitionTile { priority_from_row: None,
            indices: [0; 64], palette: [[255; 3], [170; 3], [85; 3], [0; 3]] };
            (CLASSIC_SCROLL_TILES_X * CLASSIC_SCROLL_TILES_Y) as usize];
        let (x, y) = BATTLE_TRANSITION_POKEBALL.iter().enumerate().find_map(|(y, row)|
            row.bytes().position(|byte| byte == b'X').map(|x| (x + 2, y + 1))).unwrap();
        for dark in [false, true] { for tick in [2, 4, 8, 14, 20, 29] {
            let transition = VisibleBattleTransition { frame: tick, cave_environment: false,
                stronger_enemy: false, trainer_battle: true };
            let effect = battle_transition_world_effect(transition, dark, 1., &shell.asset_root).unwrap();
            let cell = y * 20 + x;
            assert_ne!(effect.trainer_cells[cell / 128][cell / 32 % 4] & (1 << (cell % 32)), 0);
            let mut images = Assets::default();
            let image = prepare_battle_transition_texture(&shell.asset_root, transition,
                &tiles, Vec2::ZERO, dark, false, None, &mut images).unwrap();
            let data = images.get(&image).unwrap().data.as_ref().unwrap();
            for py in 0..8 { for px in 0..8 {
                let offset = ((y * 8 + py) * 160 + x * 8 + px) * 4;
                let expected = Color::srgb_u8(data[offset], data[offset + 1], data[offset + 2]).to_linear().to_vec4();
                let pixel = py * 8 + px;
                let index = effect.trainer_pixels[pixel / 16] >> (pixel % 16 * 2) & 3;
                assert!(effect.trainer_palette[index as usize].distance(expected) < 0.0001);
            }}
        }}
    }
}

#[cfg(test)]
#[test]
fn battle_transition_world_wave_matches_every_classic_scanline() {
    let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    for trainer in [false, true] {
        let mut transition = VisibleBattleTransition { frame: 0, cave_environment: true,
            stronger_enemy: false, trainer_battle: trainer };
        for tick in 0..visible_battle_transition_total_frames(&transition) {
            transition.frame = tick;
            let Some((step, frames)) = battle_transition_outro_step(transition) else { continue; };
            if step > frames { continue; }
            // Read only the wave computation, without requesting trainer source art.
            let mut wave_transition = transition;
            wave_transition.trainer_battle = false;
            wave_transition.frame -= if trainer { 1 } else { 0 };
            let effect = battle_transition_world_effect(wave_transition, false, 1., &root).unwrap();
            let mut world = World::new();
            let mut queue = bevy::ecs::world::CommandQueue::default();
            spawn_visible_battle_transition(&mut Commands::new(&mut queue, &world), transition,
                Some(Handle::default()), None);
            queue.apply(&mut world);
            let strips: Vec<_> = world.query::<(&Sprite, &Transform)>().iter(&world)
                .filter_map(|(sprite, transform)| sprite.rect.map(|rect| (rect.min.y as usize, transform.translation.x)))
                .collect();
            for y in 0..144 {
                let shift = f32::from(effect.line_x_offsets[y]) * (TILE_SIZE / SOURCE_TILE_SIZE as f32);
                assert!(strips.iter().any(|(row, x)| *row == y && (*x - shift).abs() < 0.0001),
                    "trainer={trainer} frame={tick} row={y}");
            }
        }
    }
}
