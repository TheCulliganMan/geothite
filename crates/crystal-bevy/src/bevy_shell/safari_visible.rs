//! Native capture/permit input and source-art presentation for every frontend.

use std::hash::{Hash, Hasher};

use anyhow::Result;
use bevy::prelude::{Assets, Color, Commands, Image, Sprite, Transform, Vec2, Vec3};
use crystal_assets::{AssetRoot, RuntimeSafariOutcome, SafariAdmissionOutcome};
use crystal_core::input::GameButton;
use crystal_core::systems::safari::{self, SafariError, SafariPhase};
use crystal_core::systems::safari_prompt::{self, SafariPromptKind};
use crystal_core::world::map::Direction;
use crystal_runtime::RuntimeShellSnapshot;

use super::{
    BattleCommandMarker, BattleHpSide, BevyRuntimeShell, PLAYER_GENDER_FEMALE, PLAYFIELD_HEIGHT,
    PLAYFIELD_LEFT, PLAYFIELD_TOP, PLAYFIELD_WIDTH, RenderedTilesetArt, SOURCE_TILE_SIZE,
    TILE_SIZE, load_pc_pokemon_picture, mark_runtime_snapshot_dirty, require_bitmap_font_art,
    spawn_battle_command_bitmap_text, spawn_battle_hud_borders, spawn_battle_hud_side,
    spawn_battle_trainer_marker, spawn_battle_window, visible_pc_pokemon_info,
};

/// Wildlife and its result own the complete battle LCD; field endings do not.
pub(super) fn battle_active(shell: &BevyRuntimeShell) -> bool {
    shell.visible_fishing_animation.is_none()
        && safari::session(shell.shell.session().state())
            .ok()
            .flatten()
            .is_some_and(|active| {
                matches!(
                    active.phase,
                    SafariPhase::Encounter(_) | SafariPhase::Presenting { .. }
                )
            })
}

fn trainer_back_asset_id(snapshot: &RuntimeShellSnapshot) -> &'static str {
    if snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE {
        "battle-player:kris_back"
    } else {
        "battle-player:chris_back"
    }
}

pub(super) fn render_key(snapshot: &RuntimeShellSnapshot, base: u64) -> u64 {
    let capture = snapshot
        .safari
        .as_ref()
        .filter(|active| active.phase != SafariPhase::Exploring);
    if capture.is_none() && snapshot.safari_prompt.is_none() {
        return base;
    }
    let mut key = std::collections::hash_map::DefaultHasher::new();
    base.hash(&mut key);
    if let Some(presentation) = &snapshot.safari_catch_presentation {
        presentation.hash(&mut key);
    }
    if let Some(prompt) = &snapshot.safari_prompt {
        std::mem::discriminant(&prompt.kind).hash(&mut key);
        prompt.selected_yes.hash(&mut key);
    }
    if let Some(active) = capture {
        snapshot.trainer.player_gender.hash(&mut key);
        active.command_cursor.hash(&mut key);
        active.balls_remaining.hash(&mut key);
        active.steps_remaining.hash(&mut key);
        std::mem::discriminant(&active.phase).hash(&mut key);
        let wild = match &active.phase {
            SafariPhase::Encounter(wild) => Some(wild),
            SafariPhase::Presenting { encounter, result } => {
                crystal_runtime::safari_result_lines(encounter, result).hash(&mut key);
                Some(encounter)
            }
            _ => None,
        };
        if let Some(wild) = wild {
            let info = visible_pc_pokemon_info(&wild.pokemon);
            info.frontpic_asset_id.hash(&mut key);
            info.is_shiny.hash(&mut key);
            wild.pokemon.nickname.hash(&mut key);
            wild.pokemon.level.hash(&mut key);
        }
    }
    key.finish()
}

pub(super) fn snapshot_owns(snapshot: &RuntimeShellSnapshot) -> bool {
    snapshot.safari_prompt.is_some()
        || snapshot
            .safari
            .as_ref()
            .is_some_and(|active| active.phase != SafariPhase::Exploring)
}

pub(super) fn owns(shell: &BevyRuntimeShell) -> bool {
    let state = shell.shell.session().state();
    if safari_prompt::prompt(state).map_or(true, |prompt| prompt.is_some()) {
        return true;
    }
    safari::session(state).map_or(true, |session| {
        session.is_some_and(|session| session.phase != SafariPhase::Exploring)
    })
}

fn other_owner(shell: &BevyRuntimeShell) -> bool {
    shell.field_notice.is_some()
        || shell.visible_fishing_animation.is_some()
        || shell.active_script_cursor.is_some()
        || shell.pending_scene_script.is_some()
        || shell.pending_name_input.is_some()
        || shell.pending_name_choice.is_some()
        || shell.start_menu_cursor.is_some()
        || shell.field_pack_pocket.is_some()
        || shell.party_menu_open
        || shell.pokedex_menu_open
        || shell.pokegear_menu_open
        || shell.options_menu_open
        || shell.save_menu_open
        || shell
            .shell
            .session()
            .state()
            .script_runtime
            .text_window_open
        || shell
            .shell
            .session()
            .state()
            .script_runtime
            .active_menu
            .is_some()
}

pub(super) fn press(shell: &mut BevyRuntimeShell, button: GameButton) -> Result<bool> {
    let owned = owns(shell);
    if super::safari_catch_visible::native_surface_owns(shell) {
        return Ok(false);
    }
    if other_owner(shell) {
        return Ok(owned && !matches!(button, GameButton::A | GameButton::B));
    }
    if shell.player_walk_frame_ticks > 0 {
        return Ok(owned);
    }
    let prompt = safari_prompt::prompt(shell.shell.session().state())?;
    if prompt.is_some() {
        match button {
            GameButton::Up => {
                shell.shell.navigate_safari_prompt(Direction::Up)?;
            }
            GameButton::Down => {
                shell.shell.navigate_safari_prompt(Direction::Down)?;
            }
            GameButton::A => {
                let result = shell.shell.confirm_safari_prompt()?;
                show_outcome(shell, &result);
            }
            GameButton::B => {
                shell.shell.cancel_safari_prompt()?;
            }
            _ => {}
        }
        mark_runtime_snapshot_dirty(shell);
        return Ok(true);
    }
    let active = safari::session(shell.shell.session().state())?;
    if let Some(active) = active
        && active.phase != SafariPhase::Exploring
    {
        match active.phase {
            SafariPhase::Encounter(_) => match button {
                GameButton::Up => {
                    shell.shell.navigate_safari(Direction::Up)?;
                }
                GameButton::Down => {
                    shell.shell.navigate_safari(Direction::Down)?;
                }
                GameButton::Left => {
                    shell.shell.navigate_safari(Direction::Left)?;
                }
                GameButton::Right => {
                    shell.shell.navigate_safari(Direction::Right)?;
                }
                GameButton::A => match shell.shell.confirm_safari() {
                    Ok(_) => {}
                    Err(error)
                        if error.downcast_ref::<SafariError>()
                            == Some(&SafariError::StorageFull) =>
                    {
                        shell.field_notice = Some("Party and current\nBOX are full!".into());
                    }
                    Err(error) => return Err(error),
                },
                _ => {}
            },
            SafariPhase::Presenting { .. } if matches!(button, GameButton::A | GameButton::B) => {
                shell.shell.acknowledge_safari_result()?;
                super::safari_catch_visible::restore(shell)?;
            }
            SafariPhase::Ending(_) if matches!(button, GameButton::A | GameButton::B) => {
                shell.shell.return_from_safari()?;
            }
            _ => {}
        }
        mark_runtime_snapshot_dirty(shell);
        return Ok(true);
    }
    Ok(false)
}

pub(super) fn open_ranger(shell: &mut BevyRuntimeShell, script: &str) -> Result<bool> {
    let data = shell.shell.runtime().data();
    let Some(zone) = &data.safari_zone else {
        return Ok(false);
    };
    if script != zone.admission_script && !zone.retire_scripts.iter().any(|role| role == script) {
        return Ok(false);
    }
    if other_owner(shell) || shell.player_walk_frame_ticks > 0 {
        return Ok(false);
    }
    if data
        .safari_ranger_at_player(
            shell.shell.session().state(),
            shell.shell.session().overworld(),
        )?
        .is_none()
    {
        return Ok(false);
    }
    shell.shell.open_safari_prompt()?;
    mark_runtime_snapshot_dirty(shell);
    Ok(true)
}

fn show_outcome(shell: &mut BevyRuntimeShell, outcome: &RuntimeSafariOutcome) {
    let line = match outcome {
        RuntimeSafariOutcome::Admission(SafariAdmissionOutcome::NeedSurf) => Some(
            if shell.shell.runtime().data().items.contains_key("CANOE") {
                "Bring a CANOE or\na SURF partner\nwith its badge."
            } else {
                "Bring a SURF\npartner with\nits badge."
            },
        ),
        RuntimeSafariOutcome::Admission(SafariAdmissionOutcome::InsufficientMoney) => {
            Some("Entry costs 500.\nYou need more money.")
        }
        RuntimeSafariOutcome::Admission(SafariAdmissionOutcome::StorageFull) => {
            Some("Party and current\nBOX are full!")
        }
        RuntimeSafariOutcome::Admission(SafariAdmissionOutcome::Busy) => {
            Some("Finish the current\nactivity first.")
        }
        _ => None,
    };
    if let Some(line) = line {
        shell.field_notice = Some(line.into());
    }
}

fn text(
    commands: &mut Commands,
    art: &mut RenderedTilesetArt,
    root: &AssetRoot,
    images: &mut Assets<Image>,
    label: &str,
    cell: (f32, f32),
    z: f32,
) {
    let (col, row) = cell;
    spawn_battle_command_bitmap_text(
        commands,
        art,
        root,
        images,
        label,
        PLAYFIELD_LEFT + (col + 0.5) * TILE_SIZE,
        PLAYFIELD_TOP - (row + 0.5) * TILE_SIZE,
        z,
    );
}

pub(super) fn render(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    shell: &BevyRuntimeShell,
    art: &mut RenderedTilesetArt,
    root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let prompt = snapshot.safari_prompt.as_ref();
    let capture = snapshot
        .safari
        .as_ref()
        .filter(|session| session.phase != SafariPhase::Exploring);
    if prompt.is_none() && capture.is_none() {
        return Ok(());
    }
    if shell.visible_fishing_animation.is_some()
        || shell.pokedex_menu_open
        || shell.active_script_cursor.is_some()
        || shell.pending_scene_script.is_some()
        || shell.pending_name_input.is_some()
        || snapshot.ui.text_window_open
    {
        return Ok(());
    }
    require_bitmap_font_art(art, root, images)?;
    const Z: f32 = 3.6;
    if let Some(prompt) = prompt {
        spawn_battle_window(commands, art, root, images, 0.0, 12.0, 20.0, 6.0, Z);
        let lines = match prompt.kind {
            SafariPromptKind::Admission => ["Enter preserve?", "30 BALLS 500 STEPS", "Fee: 500"],
            SafariPromptKind::Retire => {
                ["End expedition?", "Your catches stay.", "Back to basecamp?"]
            }
        };
        for (i, line) in lines.iter().enumerate() {
            text(
                commands,
                art,
                root,
                images,
                line,
                (1.0, 13.0 + i as f32 * 1.5),
                Z + 0.2,
            );
        }
        spawn_battle_window(commands, art, root, images, 14.0, 7.0, 6.0, 5.0, Z);
        text(
            commands,
            art,
            root,
            images,
            if prompt.selected_yes { ">YES" } else { " YES" },
            (15.0, 8.0),
            Z + 0.2,
        );
        text(
            commands,
            art,
            root,
            images,
            if prompt.selected_yes { " NO" } else { ">NO" },
            (15.0, 10.0),
            Z + 0.2,
        );
        return Ok(());
    }
    let Some(capture) = capture else {
        return Ok(());
    };
    if matches!(capture.phase, SafariPhase::Ending(_)) {
        // Budget exhaustion and ranger retirement are field dialogue. There
        // is no wildlife left to justify a blank trainer-backed battle LCD.
        // Use the field presenter so the window and glyphs dock together.
        let background = super::spawn_scene_dialog_window(
            commands,
            art,
            root,
            images,
            super::FIELD_TEXT_BOX_LEFT_TILE,
            super::FIELD_TEXT_BOX_TOP_TILE,
            super::FIELD_TEXT_BOX_WIDTH_TILES,
            super::FIELD_TEXT_BOX_HEIGHT_TILES,
            Z,
        );
        commands.entity(background).insert(BattleCommandMarker);
        for (i, line) in ["Expedition over!", "Back to basecamp."].iter().enumerate() {
            let (x, y) = super::battle_hud_tile_origin(
                super::FIELD_TEXT_BOX_TEXT_LEFT_TILE,
                super::FIELD_TEXT_BOX_TEXT_TOP_TILE
                    + i as f32 * super::FIELD_TEXT_BOX_ROW_SPACING_TILES,
            );
            super::spawn_scene_dialog_bitmap_text(commands, art, root, images, line, x, y, Z + 0.3);
        }
        return Ok(());
    }
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 2.7),
        BattleCommandMarker,
    ));
    // InitBattleDisplay's native 6x6 trainer slot remains occupied throughout
    // a Safari encounter: the trainer throws, rather than sending out a partner.
    spawn_battle_trainer_marker(
        commands,
        art,
        root,
        images,
        trainer_back_asset_id(snapshot),
        Vec3::new(
            PLAYFIELD_LEFT + 2.0 * TILE_SIZE,
            PLAYFIELD_TOP - 6.0 * TILE_SIZE,
            Z + 0.1,
        ),
    )?;
    let wild = match &capture.phase {
        SafariPhase::Encounter(wild)
        | SafariPhase::Presenting {
            encounter: wild, ..
        } => Some(wild),
        _ => None,
    };
    if let Some(wild) = wild {
        spawn_battle_hud_side(
            commands,
            art,
            root,
            images,
            &wild.pokemon,
            BattleHpSide::Enemy,
            None,
            None,
        )?;
        spawn_battle_hud_borders(commands, art, root, images, true, false, false)?;
        let frame = load_pc_pokemon_picture(
            root,
            Some(&visible_pc_pokemon_info(&wild.pokemon)),
            true,
            images,
        )?;
        commands.spawn((
            Sprite {
                image: frame.handle,
                custom_size: Some(frame.size * (TILE_SIZE / SOURCE_TILE_SIZE as f32)),
                ..Default::default()
            },
            Transform::from_xyz(
                PLAYFIELD_LEFT + 15.5 * TILE_SIZE,
                PLAYFIELD_TOP - 3.5 * TILE_SIZE,
                Z + 0.1,
            ),
            BattleCommandMarker,
        ));
    }
    // The native YES/NO box owns the lower-right HUD area during naming.
    if shell.pending_name_choice.is_none() {
        text(
            commands,
            art,
            root,
            images,
            &format!("BALLS {:02}", capture.balls_remaining),
            (10.0, 8.0),
            Z + 0.2,
        );
        text(
            commands,
            art,
            root,
            images,
            &format!("STEPS {:03}", capture.steps_remaining),
            (10.0, 10.0),
            Z + 0.2,
        );
    }
    if shell.pending_name_choice.is_some() {
        return Ok(());
    }
    spawn_battle_window(commands, art, root, images, 0.0, 12.0, 20.0, 6.0, Z + 0.1);
    if let Some(notice) = &shell.field_notice {
        for (i, line) in notice.lines().enumerate() {
            text(
                commands,
                art,
                root,
                images,
                line,
                (1.0, 13.0 + i as f32 * 1.5),
                Z + 0.3,
            );
        }
    } else {
        match &capture.phase {
            SafariPhase::Encounter(_) => {
                for (index, label) in ["BALL", "BAIT", "ROCK", "RUN"].iter().enumerate() {
                    text(
                        commands,
                        art,
                        root,
                        images,
                        &format!(
                            "{}{label}",
                            if capture.command_cursor as usize == index {
                                ">"
                            } else {
                                " "
                            }
                        ),
                        (
                            1.0 + (index % 2) as f32 * 10.0,
                            13.0 + (index / 2) as f32 * 2.0,
                        ),
                        Z + 0.3,
                    );
                }
            }
            SafariPhase::Presenting { encounter, result } => {
                let lines = crystal_runtime::safari_catch_presentation_lines(snapshot)
                    .unwrap_or_else(|| crystal_runtime::safari_result_lines(encounter, result));
                for (i, line) in lines.iter().enumerate() {
                    text(
                        commands,
                        art,
                        root,
                        images,
                        line,
                        (1.0, 13.0 + i as f32 * 1.5),
                        Z + 0.3,
                    );
                }
            }
            _ => {}
        }
    }
    Ok(())
}
