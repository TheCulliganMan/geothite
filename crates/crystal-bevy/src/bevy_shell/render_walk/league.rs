//! Adaptive, opt-in native QA. Decisions produce keys, never gameplay mutations.
//! Only `seed` supplies initial legal inventory/party; all later state is earned.
use super::*;
use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};

const ROOMS: [&str; 5] = [
    "WillsRoom",
    "KogasRoom",
    "BrunosRoom",
    "KarensRoom",
    "LancesRoom",
];

fn validate_room_checkpoints(checkpoints: &BTreeSet<String>) -> Result<()> {
    for room in ROOMS {
        for phase in ["arrival", "walking", "earned"] {
            anyhow::ensure!(
                checkpoints.contains(&format!("{room}-{phase}")),
                "continuous native review missed {room} {phase}"
            );
        }
        let prefix = format!("{room}-battle-");
        anyhow::ensure!(
            checkpoints.iter().any(|label| {
                let Some((species, slot)) = label.strip_prefix(&prefix)
                    .and_then(|suffix| suffix.rsplit_once("-slot")) else { return false; };
                let species = species.split('-').collect::<Vec<_>>();
                species.len() == 2 && species.iter().all(|id| !id.is_empty())
                    && slot.parse::<usize>().is_ok_and(|slot| (1..=6).contains(&slot))
            }),
            "continuous native review missed {room} species-and-party-slot battle capture"
        );
    }
    Ok(())
}

pub(in crate::bevy_shell) fn seed(shell: &mut RuntimeGameShell) -> Result<()> {
    anyhow::ensure!(
        shell.current_map_name() == "IndigoPlateauPokecenter1F",
        "League fixture must begin outside Will's room"
    );
    anyhow::ensure!(
        shell.snapshot()?.party.slots.is_empty(),
        "League fixture must be fresh"
    );
    shell.set_trainer_identity("CHRIS", 1)?;
    shell.add_party_pokemon(
        "MEW",
        100,
        None,
        None,
        "CHRIS",
        1,
        Dv::from_non_hp(15, 15, 15, 15),
    )?;
    let moves = ["SURF", "ICE_BEAM", "PSYCHIC_M", "RECOVER"]
        .into_iter()
        .map(|name| crate::core::models::LearnedMove {
            name: name.into(),
            current_pp: shell.runtime().data().moves[name].pp * 8 / 5,
            pp_ups: 3,
        })
        .collect();
    let state = shell.session_mut().state_mut();
    state.storage.party.pokemon[0].as_mut().unwrap().moves = moves;
    state.sync_party_from_storage();
    anyhow::ensure!(
        shell.add_bag_item("FULL_RESTORE", 5)?.added,
        "initial healing stock"
    );
    anyhow::ensure!(
        shell.session().state().hall_of_fame.count == 0,
        "fresh Hall record"
    );
    for map in ROOMS {
        anyhow::ensure!(
            !shell
                .session()
                .state()
                .flags
                .is_script_flag_set(&exit_flag(shell, map)?)?,
            "{map}: fixture must not supply victory"
        );
    }
    Ok(())
}

fn exit_flag(shell: &RuntimeGameShell, map: &str) -> Result<String> {
    shell.runtime().data().maps[map]
        .script_flag_commands
        .iter()
        .find(|command| {
            command.command == "setevent"
                && if map == "LancesRoom" {
                    command.flag_id == "EVENT_BEAT_CHAMPION_LANCE"
                } else {
                    command.flag_id.ends_with("_EXIT_OPEN")
                }
        })
        .map(|command| command.flag_id.clone())
        .context("compiled League exit flag")
}

fn pp(snapshot: &RuntimeShellSnapshot) -> Vec<u8> {
    snapshot
        .party
        .slots
        .first()
        .map(|slot| slot.pokemon.moves.iter().map(|m| m.current_pp).collect())
        .unwrap_or_default()
}
fn stock(snapshot: &RuntimeShellSnapshot) -> u16 {
    snapshot
        .bag
        .items
        .iter()
        .find(|item| item.item_id == "FULL_RESTORE")
        .map_or(0, |item| u16::from(item.quantity))
}
fn direction_key(direction: Direction) -> KeyCode {
    match direction {
        Direction::Up => KeyCode::ArrowUp,
        Direction::Down => KeyCode::ArrowDown,
        Direction::Left => KeyCode::ArrowLeft,
        Direction::Right => KeyCode::ArrowRight,
    }
}

// Probe clones for routing only. No probe session is ever installed in the game.
fn next_step(
    start: &crate::core::world::session::OverworldSession,
    target: TilePosition,
) -> Result<KeyCode> {
    let mut queue = VecDeque::from([(start.clone(), None)]);
    let mut visited = BTreeSet::from([(start.player.tile.x, start.player.tile.y)]);
    while let Some((session, first)) = queue.pop_front() {
        if session.player.tile == target {
            return first.context("already at routing target");
        }
        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let mut next = session.clone();
            let mut step = next.step_and_check_warp_checked(direction, Default::default())?;
            if matches!(step.outcome, StepOutcome::Turned { .. }) {
                step = next.step_and_check_warp_checked(direction, Default::default())?;
            }
            if matches!(step.outcome, StepOutcome::Moved { .. })
                && step.warp.is_none()
                && visited.insert((next.player.tile.x, next.player.tile.y))
            {
                queue.push_back((next, first.or(Some(direction_key(direction)))));
            }
        }
    }
    anyhow::bail!(
        "no native League path from {} {:?} to {target:?}",
        start.map.name,
        start.player.tile
    )
}

struct Healing {
    stock: u16,
    pp: Vec<u8>,
}
type Completion = Arc<Mutex<Option<Result<(), String>>>>;
struct Checkpoint {
    label: String,
    stage: u8,
    warmup: u32,
    started: std::time::Instant,
    baseline: Option<render_capture::PairedState>,
    ui: String,
    completion: Option<Completion>,
    native_evidence: serde_json::Value,
}

pub(super) struct LeagueReview {
    directory: PathBuf,
    started: std::time::Instant,
    checkpoints: BTreeSet<String>,
    checkpoint: Option<Checkpoint>,
    current_room: Option<usize>,
    entered_pp: Vec<u8>,
    saw_battle: bool,
    wins: BTreeSet<String>,
    healing: Option<Healing>,
    healed_room: bool,
    saw_continue: bool,
    history: Vec<serde_json::Value>,
    actions: usize,
    battle_wait: Option<std::time::Instant>,
    field_wait: Option<std::time::Instant>,
    movement_start: Option<(String, TilePosition)>,
}
impl LeagueReview {
    pub(super) fn new(path: &Path) -> Self {
        Self {
            directory: path.with_extension("league"),
            started: std::time::Instant::now(),
            checkpoints: default(),
            checkpoint: None,
            current_room: None,
            entered_pp: vec![],
            saw_battle: false,
            wins: default(),
            healing: None,
            healed_room: false,
            saw_continue: false,
            history: vec![],
            actions: 0,
            battle_wait: None,
            field_wait: None,
            movement_start: None,
        }
    }

    fn phase(runtime: &BevyRuntimeShell) -> String {
        if let Some(credits) = &runtime.credits_screen {
            if let Some(hall) = &credits.hall_of_fame {
                return format!(
                    "hall-{:?}-elapsed{}-animation{:?}-rating{}:{}",
                    hall.sequence.phase(),
                    hall.sequence.elapsed_frames(),
                    hall.animation.as_ref().map(|a| a.frame),
                    hall.rating_visible,
                    hall.rating_text
                );
            }
            return format!("credits-end-{}", credits.awaiting_exit);
        }
        if let Some(summary) = &runtime.visible_continue_screen {
            return format!("continue-{}", summary.remaining_delay_frames);
        }
        if let Some(title) = &runtime.title_menu {
            return format!("title-{:?}", title.source_phase());
        }
        if runtime.battle_action_cursor.is_some() || runtime.battle_move_cursor.is_some() {
            return format!(
                "battle-action-{:?}-move-{:?}-text-{:?}",
                runtime
                    .battle_action_cursor
                    .as_ref()
                    .map(|c| c.option_index),
                runtime.battle_move_cursor.as_ref().map(|c| c.option_index),
                runtime.battle_messages
            );
        }
        format!(
            "field-{:?}",
            runtime
                .field_text_reveal
                .as_ref()
                .map(|p| (&p.text, p.page_index, p.visible_chars))
        )
    }

    fn milestone(
        &self,
        runtime: &BevyRuntimeShell,
        status: &crystal_voxel_view::VoxelViewStatus,
        frame: &crystal_render_api::VisualWorldFrame,
        battle: bool,
        native_battle: bool,
        modeled_species: &[String],
        enemy_party_index: Option<usize>,
    ) -> Option<String> {
        if let Some(credits) = &runtime.credits_screen {
            if let Some(hall) = &credits.hall_of_fame {
                use crystal_runtime::hall_of_fame::HallOfFamePhase as Phase;
                let elapsed = hall.sequence.elapsed_frames();
                let phase = hall.sequence.phase();
                // Entrance registers wrap through offscreen coordinates. Capture
                // actual visible poses, never a blank phase-entry frame.
                let visible = match phase {
                    Phase::PokemonBack { .. } | Phase::PlayerBack => elapsed >= 28,
                    Phase::PokemonFront { .. } => elapsed >= 40,
                    Phase::PlayerFront => elapsed >= 64,
                    Phase::PokemonHold { .. } => elapsed >= 8,
                    Phase::PokemonAnimation { .. } => hall.animation.is_some(),
                    Phase::OakRating => hall.rating_visible == hall.rating_text.chars().count(),
                    Phase::ClosingFade => elapsed >= 1,
                    _ => false,
                };
                return visible.then(|| match phase {
                    Phase::PokemonAnimation { .. } => format!(
                        "hall-{phase:?}-frame{}",
                        hall.animation.as_ref().unwrap().frame
                    ),
                    Phase::OakRating => format!("hall-{phase:?}-page{}", hall.rating_pages.len()),
                    _ => format!("hall-{phase:?}"),
                });
            }
            if credits.awaiting_exit {
                return Some("credits-the-end".into());
            }
        }
        if runtime
            .visible_continue_screen
            .as_ref()
            .is_some_and(|s| s.remaining_delay_frames == 0)
        {
            return Some("continue-summary".into());
        }
        if battle {
            // Each opposing species needs its own paired evidence. A room's
            // first opponent cannot certify later replacements or AI switches.
            return native_battle.then(|| format!("{}-battle-{}-slot{}",
                runtime.shell.current_map_name(), modeled_species.join("-"),
                enemy_party_index.map_or_else(||"unknown".into(),|index|(index+1).to_string())));
        }
        if runtime.credits_screen.is_none()
            && runtime.title_menu.is_none()
            && runtime.visible_continue_screen.is_none()
            && runtime.intro_screen.is_none()
            && status.active
            && frame.active
            && frame.map_id.as_ref() == runtime.shell.current_map_name()
            && status.terrain_map.as_deref() == Some(runtime.shell.current_map_name())
            && !status.terrain_pending
            && runtime.visible_walk_warp_phase.is_none()
            && runtime.screen_fade.is_none()
        {
            let map = runtime.shell.current_map_name();
            if ROOMS.contains(&map) {
                if runtime.player_walk_frame_ticks > 0
                    && self
                        .movement_start
                        .as_ref()
                        .is_some_and(|(from_map, from)| {
                            from_map == map
                                && *from != runtime.shell.session().overworld().player.tile
                        })
                    && runtime.visible_script_movement.is_none()
                    && !self.checkpoints.contains(&format!("{map}-walking"))
                {
                    return Some(format!("{map}-walking"));
                }
                if self.wins.contains(map) {
                    return Some(format!("{map}-earned"));
                }
                return Some(format!("{map}-arrival"));
            }
            if map == "HallOfFame" {
                return Some("HallOfFame-arrival".into());
            }
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn frame(
        &mut self,
        runtime: &BevyRuntimeShell,
        frame: &crystal_render_api::VisualWorldFrame,
        status: &crystal_voxel_view::VoxelViewStatus,
        battle_frame: Option<&crystal_render_api::VisualBattleFrame>,
        battle_status: &crystal_voxel_view::BattleViewStatus,
        settings: &mut crystal_voxel_view::VoxelViewSettings,
        keyboard: &mut ButtonInput<KeyCode>,
        commands: &mut Commands,
        capture: Option<&render_capture::Capture>,
        window: Option<Entity>,
        hold: &mut render_capture::PairedCaptureHold,
        clock: &mut Time<Virtual>,
        tick: usize,
    ) -> Result<bool> {
        anyhow::ensure!(
            runtime.last_error.is_none(),
            "native controller error: {:?}",
            runtime.last_error
        );
        anyhow::ensure!(
            self.started.elapsed().as_secs() < 7200 && self.actions < 10000,
            "native League review exceeded its bounded run; inspect league history"
        );
        let snapshot = runtime.shell.presentation_snapshot()?;
        let battle = snapshot.battle.is_some();
        let native_battle_ready = battle_status.active
                        && battle_status.map_scenery_fallback_reason.is_none()
                        && battle_status.map_scenery_id.as_deref()
                            == Some(snapshot.overworld.map_name.as_str())
                        && battle_status.source_art_species.is_empty()
                        // Row receipts apply only to an image-composite pose.
                        // Ordinary native scene draws do not populate that cache.
                        && (!battle_status.capture_image_active
                            || battle_status.row_capture_ready.iter().all(|ready| *ready))
                        && battle_frame.is_some_and(|b| {
                            b.active
                                && !b.use_source_scene
                                && b.cues.is_empty()
                                && battle_status.modeled_species.iter().map(String::as_str).eq(
                                    b.battlers.iter().flatten().map(|b| b.species_id.as_ref()))
                                && b.battlers
                                    .iter()
                                    .all(|slot| slot.as_ref().is_some_and(|b| b.visible))
                        });
        let native_command = battle
            && (runtime.battle_action_cursor.is_some() || runtime.battle_move_cursor.is_some())
            && runtime.battle_messages.is_empty()
            && !visible_battle_command_animation_active(runtime)
            && runtime.battle_switch_cursor.is_none()
            && !runtime.battle_party_summary_open
            && runtime.battle_pack_target_mode.is_none()
            && runtime.bag_cursor.is_none()
            && runtime.ball_cursor.is_none()
            && runtime.key_item_cursor.is_none()
            && runtime.tmhm_cursor.is_none()
            && !runtime.battle_level_stats.front().is_some_and(|stats| stats.active);
        self.saw_battle |= battle;
        if self.checkpoint.is_none() {
            if let Some(label) = self
                .milestone(
                    runtime,
                    status,
                    frame,
                    battle,
                    native_battle_ready && native_command,
                    &battle_status.modeled_species,
                    snapshot.battle.as_ref().and_then(|b|b.active_enemy_party_index),
                )
                .filter(|label| !self.checkpoints.contains(label))
            {
                std::fs::create_dir_all(&self.directory)?;
                self.checkpoint = Some(Checkpoint {
                    label,
                    stage: 0,
                    warmup: 0,
                    started: std::time::Instant::now(),
                    baseline: None,
                    ui: String::new(),
                    completion: None,
                    native_evidence: serde_json::json!({
                        "field_active":status.active,"field_map":status.terrain_map,
                        "field_pending":status.terrain_pending,"battle_active":battle_status.active,
                        "modeled_species":battle_status.modeled_species,
                        "presented_species":battle_frame.map(|frame|frame.battlers.iter().flatten().map(|b|b.species_id.as_ref()).collect::<Vec<_>>()),
                        "enemy_species":snapshot.battle.as_ref().map(|b|b.enemy_pokemon.species.id.as_str()),
                        "enemy_party_index":snapshot.battle.as_ref().and_then(|b|b.active_enemy_party_index),
                        "source_art_species":battle_status.source_art_species,
                        "scenery":battle_status.map_scenery_id,
                        "scenery_fallback":battle_status.map_scenery_fallback_reason,
                        "row_capture_ready":battle_status.row_capture_ready,
                    }),
                });
                hold.frozen = true;
                clock.pause();
            }
        }
        if let Some(checkpoint) = self.checkpoint.as_mut() {
            anyhow::ensure!(
                checkpoint.started.elapsed().as_secs() < 120,
                "checkpoint {} timed out",
                checkpoint.label
            );
            checkpoint.warmup += 1;
            if checkpoint.warmup < 30 {
                return Ok(false);
            }
            if checkpoint.baseline.is_none() {
                checkpoint.baseline = Some(render_capture::paired_state(runtime, frame)?);
                checkpoint.ui = Self::phase(runtime);
            } else {
                anyhow::ensure!(
                    checkpoint.baseline.as_ref()
                        == Some(&render_capture::paired_state(runtime, frame)?)
                        && checkpoint.ui == Self::phase(runtime),
                    "{}: paired capture changed state",
                    checkpoint.label
                );
            }
            if let Some(completion) = checkpoint.completion.as_ref() {
                let result = completion.lock().unwrap().take();
                let Some(result) = result else {
                    return Ok(false);
                };
                result.map_err(anyhow::Error::msg)?;
                checkpoint.completion = None;
                checkpoint.stage += 1;
                checkpoint.warmup = 0;
                settings.enabled = checkpoint.stage != 1;
                return Ok(false);
            }
            if checkpoint.stage == 2 {
                self.checkpoints.insert(checkpoint.label.clone());
                let state = runtime.shell.session().state();
                let observed_flags = runtime.runtime.data().maps[&snapshot.overworld.map_name]
                    .script_flag_commands
                    .iter()
                    .filter(|command| command.command == "setevent")
                    .map(|command| {
                        (
                            command.flag_id.clone(),
                            state.flags.is_script_flag_set(&command.flag_id).ok(),
                        )
                    })
                    .collect::<std::collections::BTreeMap<_, _>>();
                let blocks = state
                    .map_block_overrides
                    .get(&snapshot.overworld.map_name)
                    .into_iter()
                    .flat_map(|blocks| blocks.iter())
                    .map(|(&(x, y), &block)| serde_json::json!([x, y, block]))
                    .collect::<Vec<_>>();
                self.history.push(serde_json::json!({"checkpoint":checkpoint.label,
                    "state_preserved":true,"observed_flags":observed_flags,"map_block_overrides":blocks,"native_evidence":checkpoint.native_evidence,"native_battle":battle_frame.map(|b|b.active && !b.use_source_scene),"checksum":format!("{:?}",runtime.shell.state_checksum()?),
                    "map":snapshot.overworld.map_name,"tile":[snapshot.overworld.tile.x,snapshot.overworld.tile.y],
                    "pp":pp(&snapshot),"hp":snapshot.party.slots.first().map(|slot|slot.pokemon.hp),"full_restore":stock(&snapshot)}));
                self.checkpoint = None;
                hold.frozen = false;
                clock.unpause();
                self.write_history()?;
                return Ok(false);
            }
            let screenshot = render_capture::moving_screenshot(
                capture,
                window.context("headless capture entity")?,
            )
            .context("League capture requires prepared image target")?;
            let label = checkpoint.label.replace([' ', '{', '}', ':'], "");
            let path = self.directory.join(format!(
                "{label}-{}.png",
                if checkpoint.stage == 0 { "3d" } else { "2d" }
            ));
            let completion: Completion = Arc::new(Mutex::new(None));
            let result = completion.clone();
            commands
                .spawn(screenshot)
                .observe(move |event: On<ScreenshotCaptured>| {
                    let saved = event
                        .image
                        .clone()
                        .try_into_dynamic()
                        .map_err(|e| e.to_string())
                        .and_then(|image| image.to_rgb8().save(&path).map_err(|e| e.to_string()));
                    *result.lock().unwrap() = Some(saved);
                });
            checkpoint.completion = Some(completion);
            return Ok(false);
        }
        let battle_presentation = battle
            || !runtime.battle_messages.is_empty()
            || runtime.battle_action_cursor.is_some()
            || battle_frame.is_some_and(|frame| frame.active);
        let field_input = !battle_presentation
            && snapshot.phase == RuntimeShellPhase::Overworld
            && runtime.credits_screen.is_none()
            && runtime.title_menu.is_none()
            && runtime.intro_screen.is_none()
            && runtime.visible_continue_screen.is_none()
            && runtime.start_menu_cursor.is_none()
            && runtime.field_pack_pocket.is_none()
            && !runtime.party_menu_open
            && runtime.field_notice.is_none();
        if field_input
            && (status.terrain_map.as_deref() != Some(snapshot.overworld.map_name.as_str())
                || frame.map_id.as_ref() != snapshot.overworld.map_name
                || !frame.active
                || !status.active
                || status.terrain_pending
                || status.profiles_pending)
        {
            let waiting = self.field_wait.get_or_insert_with(std::time::Instant::now);
            anyhow::ensure!(
                waiting.elapsed().as_secs() < 120,
                "native field did not present {}: {status:?}; frame={} active={}",
                snapshot.overworld.map_name,
                frame.map_id,
                frame.active
            );
            return Ok(false);
        }
        self.field_wait = None;
        if tick % 32 != 0 {
            return Ok(false);
        }
        if status.terrain_pending
            && !battle
            && runtime.credits_screen.is_none()
            && runtime.title_menu.is_none()
            && runtime.intro_screen.is_none()
            && runtime.visible_continue_screen.is_none()
        {
            return Ok(false);
        }
        // Intro text must still advance through ordinary A. Once the actual
        // command surface owns input, wait for modeled GPU evidence before
        // issuing FIGHT; a source fallback cannot silently win the battle.
        if native_command && !native_battle_ready
        {
            let waiting = self.battle_wait.get_or_insert_with(std::time::Instant::now);
            anyhow::ensure!(
                waiting.elapsed().as_secs() < 120,
                "native battle did not present: {battle_status:?}"
            );
            return Ok(false);
        }
        self.battle_wait = None;
        let key = self.decide(runtime, &snapshot)?;
        if let Some(key) = key {
            self.actions += 1;
            self.history.push(serde_json::json!({"action":self.actions,"key":format!("{key:?}"),
                "map":snapshot.overworld.map_name,"tile":[snapshot.overworld.tile.x,snapshot.overworld.tile.y],
                "hp":snapshot.party.slots.first().map(|slot|slot.pokemon.hp),"pp":pp(&snapshot),"phase":Self::phase(runtime),
                "native_command":native_command,"native_ready":native_battle_ready,"native_species":battle_status.modeled_species,
                "presented_species":battle_frame.map(|frame|frame.battlers.iter().flatten().map(|b|b.species_id.as_ref()).collect::<Vec<_>>()),
                "enemy_species":snapshot.battle.as_ref().map(|b|b.enemy_pokemon.species.id.as_str()),
                "enemy_party_index":snapshot.battle.as_ref().and_then(|b|b.active_enemy_party_index)}));
            self.write_history()?;
            if matches!(
                key,
                KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::ArrowLeft | KeyCode::ArrowRight
            ) && field_input
            {
                self.movement_start =
                    Some((snapshot.overworld.map_name.clone(), snapshot.overworld.tile));
            }
            keyboard.press(key);
        }
        let done = self.saw_continue
            && runtime.visible_continue_screen.is_none()
            && runtime.title_menu.is_none()
            && runtime.intro_screen.is_none()
            && snapshot.overworld.map_name == "NewBarkTown"
            && self.wins.len() == 5;
        if done {
            std::fs::write(
                self.directory.join("completed.json"),
                serde_json::to_vec_pretty(&serde_json::json!({
                    "wins":self.wins,"checkpoints":self.checkpoints,"actions":self.actions,
                    "final_map":snapshot.overworld.map_name,"final_tile":[snapshot.overworld.tile.x,snapshot.overworld.tile.y],
                    "checksum":format!("{:?}",runtime.shell.state_checksum()?),
                    "hall_count":runtime.shell.session().state().hall_of_fame.count,
                    "beat_elite_four":runtime.shell.session().state().flags.is_script_flag_set("EVENT_BEAT_ELITE_FOUR")?,
                    "save_path":runtime.quick_save_path,"individual_images_reviewed":false,
                    "whole_room_acceptance":false
                }))?,
            )?;
        }
        Ok(done)
    }

    fn write_history(&self) -> Result<()> {
        std::fs::create_dir_all(&self.directory)?;
        std::fs::write(
            self.directory.join("history.json"),
            serde_json::to_vec_pretty(&self.history)?,
        )?;
        Ok(())
    }

    fn decide(
        &mut self,
        runtime: &BevyRuntimeShell,
        snapshot: &RuntimeShellSnapshot,
    ) -> Result<Option<KeyCode>> {
        if let Some(credits) = &runtime.credits_screen {
            if credits.awaiting_exit {
                return Ok(Some(KeyCode::KeyZ));
            }
            return Ok(credits
                .hall_of_fame
                .as_ref()
                .filter(|h| {
                    h.sequence.phase() == crystal_runtime::hall_of_fame::HallOfFamePhase::OakRating
                })
                .map(|_| KeyCode::KeyZ));
        }
        if let Some(summary) = &runtime.visible_continue_screen {
            self.saw_continue = true;
            return Ok((summary.remaining_delay_frames == 0).then_some(KeyCode::KeyZ));
        }
        if runtime.intro_screen.is_some() {
            return Ok(Some(KeyCode::Enter));
        }
        if let Some(title) = &runtime.title_menu {
            let entries = visible_title_menu_entries(runtime, title)?;
            return Ok(Some(
                if visible_title_main_menu_active(title)
                    && !entries.iter().any(|entry| entry == ">CONTINUE")
                {
                    KeyCode::ArrowDown
                } else {
                    KeyCode::KeyZ
                },
            ));
        }
        let map = snapshot.overworld.map_name.as_str();
        if self.saw_continue && map == "NewBarkTown" {
            let state = runtime.shell.session().state();
            anyhow::ensure!(
                state.hall_of_fame.count == 1
                    && state.hall_of_fame.spawn_after_champion.is_none()
                    && state.game_timer_counting,
                "Continue must finish the earned champion return"
            );
            validate_room_checkpoints(&self.checkpoints)?;
            for required in [
                "PokemonBack",
                "PokemonFront",
                "PokemonAnimation",
                "PokemonHold",
                "PlayerBack",
                "PlayerFront",
                "OakRating",
                "ClosingFade",
            ] {
                anyhow::ensure!(
                    self.checkpoints
                        .iter()
                        .any(|phase| phase.starts_with(&format!("hall-{required}"))),
                    "native ceremony missed {required}"
                );
            }
            anyhow::ensure!(
                self.checkpoints.contains("credits-the-end")
                    && self.checkpoints.contains("continue-summary"),
                "actual credits and Continue summary must be captured"
            );
            return Ok(None);
        }
        if map == "IndigoPlateauPokecenter1F" {
            let entrance = runtime.runtime.data().maps[map]
                .events
                .warps
                .iter()
                .find(|warp| warp.target_map_constant == "WILLS_ROOM")
                .and_then(warp_tile_position_checked)
                .context("compiled Indigo entrance")?;
            let delta = (
                entrance.x - snapshot.overworld.tile.x,
                entrance.y - snapshot.overworld.tile.y,
            );
            return match delta {
                (0, 0) => Ok(None), // the real warp animation owns the next frames
                (0, -1) => Ok(Some(KeyCode::ArrowUp)),
                (0, 1) => Ok(Some(KeyCode::ArrowDown)),
                (-1, 0) => Ok(Some(KeyCode::ArrowLeft)),
                (1, 0) => Ok(Some(KeyCode::ArrowRight)),
                _ => anyhow::bail!(
                    "League fixture must approach the actual entrance from one adjacent tile"
                ),
            };
        }
        if map == "HallOfFame" {
            return Ok(Some(KeyCode::KeyZ));
        }
        let index = ROOMS
            .iter()
            .position(|room| *room == map)
            .context("unexpected map during continuous League")?;
        if self.current_room != Some(index) {
            if let Some(previous) = self.current_room {
                anyhow::ensure!(
                    index == previous + 1 && self.wins.contains(ROOMS[previous]),
                    "room traversal must follow earned wins"
                );
            } else {
                anyhow::ensure!(index == 0, "League must begin with Will");
            }
            if !self.entered_pp.is_empty() {
                anyhow::ensure!(
                    pp(snapshot) == self.entered_pp,
                    "PP must carry through the doorway"
                );
            }
            self.current_room = Some(index);
            self.entered_pp = pp(snapshot);
            self.saw_battle = false;
            self.healed_room = false;
        }
        let flag = exit_flag(&runtime.shell, map)?;
        let won = runtime
            .shell
            .session()
            .state()
            .flags
            .is_script_flag_set(&flag)?;
        if battle_present(snapshot) {
            self.saw_battle = true;
            let pokemon = &snapshot.party.slots[0].pokemon;
            let enemy = &snapshot.battle.as_ref().unwrap().enemy_pokemon.species.id;
            let bond = enemy == "GENGAR"
                && snapshot.battle.as_ref().unwrap().enemy_last_move.as_deref()
                    == Some("DESTINY_BOND");
            let desired =
                if (pokemon.hp < pokemon.max_hp / 2 || bond) && pokemon.moves[3].current_pp > 0 {
                    3
                } else {
                    match (map, enemy.as_str()) {
                        ("WillsRoom", _) => 1,
                        ("KogasRoom", _) => 2,
                        ("KarensRoom", "VILEPLUME" | "GENGAR") => 2,
                        ("KarensRoom", "MURKROW") => 1,
                        ("LancesRoom", "GYARADOS") => 2,
                        ("LancesRoom", "CHARIZARD") => 0,
                        ("LancesRoom", _) => 1,
                        _ => 0,
                    }
                };
            let desired = if pokemon.moves[desired].current_pp > 0 {
                desired
            } else {
                pokemon
                    .moves
                    .iter()
                    .position(|m| m.current_pp > 0)
                    .context("carried PP exhausted")?
            };
            return Ok(Some(
                if let Some(cursor) = runtime
                    .battle_move_cursor
                    .as_ref()
                    .filter(|_| runtime.battle_messages.is_empty())
                {
                    if cursor.option_index == desired {
                        KeyCode::KeyZ
                    } else {
                        KeyCode::ArrowDown
                    }
                } else {
                    KeyCode::KeyZ
                },
            ));
        }
        // Core victory precedes the frontend's final EXP/result pages. A
        // real textbox still owns input after snapshot.battle becomes None.
        if !runtime.battle_messages.is_empty() {
            return Ok(Some(KeyCode::KeyZ));
        }
        if won && !self.wins.contains(map) {
            anyhow::ensure!(
                self.checkpoints.iter().any(|label|label.starts_with(&format!("{map}-battle-"))),
                "{map}: actual native battle presentation must be captured before victory"
            );
            anyhow::ensure!(
                self.saw_battle
                    && pp(snapshot).iter().map(|&p| u16::from(p)).sum::<u16>()
                        < self.entered_pp.iter().map(|&p| u16::from(p)).sum::<u16>(),
                "{map}: actual battle must spend PP"
            );
            anyhow::ensure!(
                snapshot.party.slots[0].pokemon.hp > 0,
                "{map}: carried party defeated"
            );
            self.wins.insert(map.into());
            self.entered_pp = pp(snapshot);
            return Ok(None); // capture the earned milestone before further input
        }
        if won && !self.checkpoints.contains(&format!("{map}-earned")) {
            // Keep the earned room visible before ordinary dialogue/exit input
            // can carry this same session into the next map.
            return Ok(None);
        }
        if snapshot.ui.window_open
            || snapshot.phase == RuntimeShellPhase::Text
            || runtime.field_notice.is_some()
            || !runtime.field_notice_queue.is_empty()
        {
            return Ok(Some(KeyCode::KeyZ));
        }
        if !won && !self.healed_room {
            if let Some(key) = self.heal(runtime, snapshot)? {
                return Ok(Some(key));
            }
            self.healed_room = true;
        }
        let module = &runtime.runtime.data().maps[map];
        if won {
            if map == "LancesRoom" {
                return Ok(Some(KeyCode::KeyZ));
            }
            let exit = module
                .events
                .warps
                .iter()
                .filter_map(warp_tile_position_checked)
                .min_by_key(|p| p.y)
                .context("north exit")?;
            let target = TilePosition::new(exit.x, exit.y + 1);
            return Ok(Some(if snapshot.overworld.tile == target {
                KeyCode::ArrowUp
            } else {
                next_step(runtime.shell.session().overworld(), target)?
            }));
        }
        let script = &module
            .script_flag_commands
            .iter()
            .find(|c| c.flag_id == flag && c.command == "setevent")
            .context("trainer victory script")?
            .source_script;
        let object = module
            .objects
            .iter()
            .find(|o| &o.script == script)
            .context("compiled League trainer")?;
        let trainer = crate::core::world::session::object_tile_position_checked(object)
            .context("trainer position")?;
        let target = TilePosition::new(trainer.x, trainer.y + 1);
        Ok(Some(if snapshot.overworld.tile != target {
            next_step(runtime.shell.session().overworld(), target)?
        } else if runtime.shell.session().overworld().player.facing != Direction::Up {
            KeyCode::ArrowUp
        } else {
            KeyCode::KeyZ
        }))
    }

    fn heal(
        &mut self,
        runtime: &BevyRuntimeShell,
        snapshot: &RuntimeShellSnapshot,
    ) -> Result<Option<KeyCode>> {
        let pokemon = &snapshot.party.slots[0].pokemon;
        if self.healing.is_none() {
            if pokemon.hp == pokemon.max_hp && pokemon.status.is_none() {
                return Ok(None);
            }
            anyhow::ensure!(
                pokemon.hp > 0 && stock(snapshot) > 0,
                "must heal through remaining legal inventory"
            );
            self.healing = Some(Healing {
                stock: stock(snapshot),
                pp: pp(snapshot),
            });
        }
        let before = self.healing.as_ref().unwrap();
        if pokemon.hp == pokemon.max_hp
            && pokemon.status.is_none()
            && stock(snapshot) == before.stock - 1
        {
            anyhow::ensure!(
                pp(snapshot) == before.pp,
                "Full Restore must preserve carried PP"
            );
            if runtime.field_pack_pocket.is_some()
                || runtime.start_menu_cursor.is_some()
                || runtime.party_menu_open
            {
                return Ok(Some(KeyCode::KeyX));
            }
            self.healing = None;
            return Ok(None);
        }
        if runtime.field_pack_pocket.is_some() {
            if runtime.field_pack_target_mode.is_some() {
                return Ok(Some(KeyCode::KeyZ));
            }
            let entries = visible_field_pack_entries(snapshot, runtime)?;
            let selected = entries.iter().any(|entry| {
                entry.starts_with('>')
                    && if runtime.field_pack_action_cursor.is_some() {
                        entry == ">USE"
                    } else {
                        entry.replace('_', " ").contains("FULL RESTORE")
                    }
            });
            return Ok(Some(if selected {
                KeyCode::KeyZ
            } else {
                KeyCode::ArrowDown
            }));
        }
        if runtime.start_menu_cursor.is_some() {
            return Ok(Some(
                if visible_start_menu_entries(runtime)?
                    .iter()
                    .any(|e| e == ">PACK")
                {
                    KeyCode::KeyZ
                } else {
                    KeyCode::ArrowDown
                },
            ));
        }
        Ok(Some(KeyCode::Enter))
    }
}
fn battle_present(snapshot: &RuntimeShellSnapshot) -> bool {
    snapshot.battle.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn completed_rooms() -> BTreeSet<String> {
        ROOMS.into_iter().flat_map(|room| [
            format!("{room}-arrival"), format!("{room}-walking"),
            format!("{room}-earned"), format!("{room}-battle-MEW-XATU-slot1"),
        ]).collect()
    }

    #[test]
    fn league_completion_accepts_species_and_distinct_party_slot_captures() {
        let mut checkpoints = completed_rooms();
        checkpoints.insert("WillsRoom-battle-MEW-XATU-slot5".into());
        assert!(validate_room_checkpoints(&checkpoints).is_ok());
        assert!(!checkpoints.contains("WillsRoom-battle"));
    }

    #[test]
    fn league_completion_requires_every_room_and_field_phase() {
        for room in ROOMS {
            for phase in ["arrival", "walking", "earned", "battle-MEW-XATU-slot1"] {
                let mut checkpoints = completed_rooms();
                checkpoints.remove(&format!("{room}-{phase}"));
                assert!(validate_room_checkpoints(&checkpoints).is_err(), "{room} {phase}");
            }
        }
    }

    #[test]
    fn league_completion_rejects_generic_unknown_and_malformed_battle_evidence() {
        for suffix in ["battle", "battle-MEW-XATU-slotunknown", "battle-MEW-XATU-slot0",
            "battle-MEW-XATU-slot7", "battle-MEW-slot1", "battle-MEW--slot1"] {
            let mut checkpoints = completed_rooms();
            checkpoints.remove("WillsRoom-battle-MEW-XATU-slot1");
            checkpoints.insert(format!("WillsRoom-{suffix}"));
            assert!(validate_room_checkpoints(&checkpoints).is_err(), "{suffix}");
        }
    }
}
