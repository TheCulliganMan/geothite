//! Native movement QA: ordinary keyboard input, per-frame trace and moving captures.
use super::*;
pub(super) mod league;

#[derive(Resource)]
pub(super) struct RenderWalk {
    route: Vec<KeyCode>,
    league: Option<league::LeagueReview>,
    tick: usize,
    finished: bool,
    settle: usize,
    path: PathBuf,
    trace: String,
    times: Vec<f64>,
    first_builds: Option<u64>,
    origins: std::collections::HashSet<(i32, i32)>,
    maps: std::collections::HashSet<String>,
    player_bounds: Option<(Vec2, Vec2)>,
    player_moved: bool,
    capture_steps: bool,
    capture_tick: usize,
    arrival_reveal_captured: bool,
    synthetic_input: bool,
    pc_input_pulse: bool,
    pc_mail_review: bool,
    pc_item_review: bool,
    step_input: bool,
    segment_position: Option<(String, TilePosition)>,
    running_input: bool,
    hold_on_battle: bool,
    hold_on_move_menu: bool,
    hold_healing_balls:Option<u8>,
    held_healing:bool,
    hold_field_notice:Option<String>,
    held_field_notice:bool,
    hold_menu_id:Option<String>,
    held_menu:bool,
    keep_current_battler: bool,
    segment_key: Option<KeyCode>,
    first_position: Option<(String, TilePosition)>,
    authoritative_moved: bool,
}
impl RenderWalk {
    pub(super) fn started(&self) -> bool {
        self.first_builds.is_some() && self.tick > 0
    }

    pub(super) fn settled(&self) -> bool {
        self.finished && self.settle >= 30
    }

    pub(super) fn hold_for_source_capture(&mut self) -> Result<()> {
        anyhow::ensure!(self.started(), "source hold requires an ordinary QA input route");
        std::fs::write(self.path.with_extension("movement.csv"), &self.trace)?;
        self.tick = self.route.len() * 32;
        self.finished = true;
        self.settle = 30;
        Ok(())
    }
}

pub(super) fn move_menu_is_presented(shell: &BevyRuntimeShell, rendered: bool) -> bool {
    rendered && shell.battle_move_cursor.is_some()
        && shell.battle_messages.is_empty()
        && !visible_battle_command_animation_active(shell)
        && !shell.battle_level_stats.front().is_some_and(|stats| stats.active)
}

// A bounded native QA policy observes the actual PC owner and emits only
// ordinary keys. It never dispatches a game action, writes a cursor, seeds
// progression or bypasses a text page. The initial field walk stays authored
// by --walk; after PC entry this avoids timing-dependent blind A counts.
fn pc_mail_review_key(runtime:&BevyRuntimeShell)->Option<KeyCode> {
    if !runtime.pc_hub_session_open {return None;}
    if runtime.special_boundary.is_some() || runtime.field_notice.is_some() || runtime.pc_notice.is_some() {
        return Some(KeyCode::KeyZ);
    }
    if let Some(cursor)=runtime.pc_hub_cursor.as_ref() {
        return Some(if cursor.option_index==1 {KeyCode::KeyZ}
            else if cursor.option_index<1 {KeyCode::ArrowDown}else{KeyCode::ArrowUp});
    }
    if let Some(cursor)=runtime.player_pc_action_cursor.as_ref() {
        return Some(if cursor.option_index==3 {KeyCode::KeyZ}
            else if cursor.option_index<3 {KeyCode::ArrowDown}else{KeyCode::ArrowUp});
    }
    if let Some(cursor)=runtime.mailbox_action_cursor.as_ref() {
        return Some(if cursor.option_index==0 {KeyCode::KeyZ}else{KeyCode::ArrowUp});
    }
    if let Some(cursor)=runtime.mailbox_cursor.as_ref() {
        return Some(if cursor.option_index==1 {KeyCode::KeyZ}
            else if cursor.option_index<1 {KeyCode::ArrowDown}else{KeyCode::ArrowUp});
    }
    None
}

// Like the Mail policy, this observes real input owners and emits only ordinary
// keys. The named capture stops input when the actual item surface is ready.
fn pc_item_review_key(runtime:&BevyRuntimeShell,wanted:&str)->Option<KeyCode> {
    if !runtime.pc_hub_session_open {return None;}
    if wanted=="pc:oak-shutdown" && matches!(runtime.pc_confirmation,Some(VisiblePcConfirmation::OakRating)) {
        return Some(KeyCode::KeyX);
    }
    if runtime.special_boundary.is_some() || runtime.field_notice.is_some() || runtime.pc_notice.is_some() {
        return Some(KeyCode::KeyZ);
    }
    if let Some(cursor)=runtime.pc_hub_cursor.as_ref() {
        let selected=if wanted.starts_with("pc:oak-") {2}else{1};
        return Some(if cursor.option_index==selected {KeyCode::KeyZ}
            else if cursor.option_index<selected {KeyCode::ArrowDown}else{KeyCode::ArrowUp});
    }
    if let Some(cursor)=runtime.player_pc_action_cursor.as_ref() {
        let action=if matches!(wanted,"pc:toss-confirmation"|"pc:toss-closing") {2}else{0};
        return Some(if cursor.option_index==action {KeyCode::KeyZ}
            else if cursor.option_index<action {KeyCode::ArrowDown}else{KeyCode::ArrowUp});
    }
    runtime.pc_item_cursor.as_ref().map(|_|KeyCode::KeyZ)
}

// Distinct maps can share the same arrival coordinates (Red's two floors do).
// A committed warp must release a step-mode key even at identical coordinates.
fn position_changed(start: Option<&(String, TilePosition)>, map: &str, tile: TilePosition) -> bool {
    start.is_some_and(|(first_map, first_tile)| first_map != map || *first_tile != tile)
}

/// Observe the renderer's actual current page, including ordinary ASM text.
/// Retained execution history or a different/unfinished printer cannot hold QA.
pub(super) fn revealed_dialogue_page(
    runtime: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    wanted: &str,
) -> Option<(usize, usize)> {
    let pages = visible_field_dialog_pages(snapshot, runtime)?;
    let reveal = runtime.field_text_reveal.as_ref()?;
    (reveal.text == pages.join("\u{1e}")
        && pages.get(reveal.page_index).is_some_and(|page| page == wanted)
        && reveal.visible_chars >= wanted.chars().count())
        .then_some((reveal.page_index, reveal.visible_chars))
}

/// These menus belong to the production controller while PCScript retains its
/// suspended core chooser. Observe that owner; never seed a review cursor.
pub(super) fn retained_pc_review_menu(runtime:&BevyRuntimeShell,wanted:&str)->Result<Option<Vec<String>>> {
    if runtime.field_notice.is_none() && runtime.pc_notice.is_none() {
        if wanted == "field:fly-destinations" && runtime.fly_cursor.is_some() {
            let snapshot = runtime.shell.presentation_snapshot()?;
            return Ok(Some(visible_fly_destination_entries(&snapshot, runtime)?));
        }
        if wanted == "field:party-actions" && runtime.party_menu_open
            && runtime.party_action_cursor.is_some() && runtime.fly_cursor.is_none()
        {
            let snapshot = runtime.shell.presentation_snapshot()?;
            return Ok(Some(visible_party_menu_entries(&snapshot, runtime)?));
        }
    }
    // A read-only hold after ordinary Name Rater script/party/YES inputs.
    // Naming owns the full LCD, so the dormant room is not its readiness gate.
    if wanted=="service:name-rater-entry" {
        if matches!(runtime.shell.current_map_name(),"LavenderNameRater"|"GoldenrodNameRater")
            &&runtime.special_boundary.is_none()&&runtime.pc_confirmation.is_none()
            &&runtime.pending_name_input.as_ref().is_some_and(|input|input.label=="POKéMON'S NAME?") {
            let mut entries=Vec::new();
            push_visible_name_input_entries(&mut entries,runtime)?;
            return Ok(Some(entries));
        }
        return Ok(None);
    }
    // Field Save is a shell-owned YesNoBox above the actual room. Observe
    // its production flow/cursor rather than the suspended core menu below it.
    let save_stage = if wanted == "field:save-overwrite" {
        VisibleSaveFlowStage::OverwritePrompt
    } else {
        VisibleSaveFlowStage::Prompt
    };
    if matches!(wanted, "field:save-confirmation" | "field:save-overwrite")
        && runtime.save_menu_open
        && runtime.special_boundary.is_none()
        && runtime.save_flow.as_ref().is_some_and(|flow| {
            flow.stage == save_stage && matches!(flow.origin, VisibleSaveFlowOrigin::StartMenu)
        })
    {
        let mut entries = Vec::new();
        let snapshot = runtime.shell.presentation_snapshot()?;
        scene_dialog_yes_no_cursor_index(&snapshot, runtime)?;
        push_visible_save_dialog_entries(&mut entries, &snapshot, runtime)?;
        return Ok(Some(entries));
    }
    if runtime.pc_hub_session_open && let Some(boundary)=runtime.special_boundary.as_ref() {
        let expected=match wanted {
            "pc:oak-shutdown"=>boundary.label=="OakPCShutdown",
            "pc:oak-counts"=>boundary.label=="ProfOaksPcBoot"&&boundary.details.iter().any(|text|text.contains("POKéMON seen")),
            _=>false,
        };
        if expected && let Some(text)=boundary.details.first()
            && revealed_dialogue_page(runtime,&runtime.shell.presentation_snapshot()?,text).is_some() {
            return Ok(Some(boundary.details.clone()));
        }
    }
    if !runtime.pc_hub_session_open || runtime.special_boundary.is_some()
        ||runtime.field_notice.is_some()
        ||runtime.pc_transfer_sequence.is_some() {return Ok(None);}
    let item_question=runtime.pc_notice.as_deref().is_some_and(|text|
        visible_field_text_reveal_is_complete_for_text(runtime,text));
    if runtime.pc_notice.is_some() && !(item_question&&(
        (wanted=="pc:quantity"&&runtime.pc_item_quantity.is_some())
        ||(wanted=="pc:toss-confirmation"&&matches!(runtime.pc_confirmation,Some(VisiblePcConfirmation::TossItem{..})))
        ||(wanted=="pc:oak-confirmation"&&matches!(runtime.pc_confirmation,Some(VisiblePcConfirmation::OakRating)))
        ||(wanted=="pc:oak-closing"&&runtime.mailbox_confirmation_response.is_some()&&matches!(runtime.pc_confirmation,Some(VisiblePcConfirmation::OakRating)))
        ||(wanted=="pc:toss-closing"&&runtime.mailbox_confirmation_response.is_some()&&matches!(runtime.pc_confirmation,Some(VisiblePcConfirmation::TossItem{..}))))) {return Ok(None);}
    match wanted {
        "pc:hub" if runtime.pc_hub_cursor.is_some()=> {
            let mut entries=Vec::new();
            push_visible_pc_hub_dialog_entries(&mut entries,&runtime.shell.presentation_snapshot()?,runtime)?;
            Ok(Some(entries))
        }
        "pc:bill-actions" if runtime.bill_pc_action_cursor.is_some()=>
            visible_bill_pc_action_entries(runtime).map(Some),
        "pc:items" if runtime.pc_item_cursor.is_some() && !visible_field_pack_is_open(runtime)
            &&runtime.pc_item_quantity.is_none()&&runtime.pc_confirmation.is_none()=> {
            let mut entries=Vec::new();
            push_visible_pc_item_dialog_entries(&mut entries,&runtime.shell.presentation_snapshot()?,runtime)?;
            Ok(Some(entries))
        }
        "pc:quantity" if let Some(quantity)=runtime.pc_item_quantity.as_ref()
            &&visible_pc_item_quantity_input_ready(runtime)=> {
            Ok(Some(vec![runtime.pc_notice.clone().expect("printed quantity question"),
                format!("{} x{}",quantity.item_id,quantity.quantity)]))
        }
        "pc:toss-confirmation" if matches!(runtime.pc_confirmation,Some(VisiblePcConfirmation::TossItem{..}))
            &&runtime.yes_no_cursor.is_some()&&item_question=> {
            Ok(Some(vec![runtime.pc_notice.clone().expect("printed toss question"),"YES / NO".into()]))
        }
        "pc:oak-closing"|"pc:toss-closing" if runtime.mailbox_confirmation_response.is_some()
            &&runtime.yes_no_cursor.is_some()&&item_question=> {
            Ok(Some(vec![runtime.pc_notice.clone().expect("printed closing question"),
                format!("answer={:?} remaining_vblanks={}",runtime.mailbox_confirmation_response,runtime.pc_menu_input_wait_frames)]))
        }
        "pc:oak-confirmation" if matches!(runtime.pc_confirmation,Some(VisiblePcConfirmation::OakRating))
            &&runtime.yes_no_cursor.is_some()&&item_question=> {
            Ok(Some(vec![runtime.pc_notice.clone().expect("printed Oak rating question"),"YES / NO".into()]))
        }
        "pc:mailbox" if runtime.mailbox_cursor.is_some() && runtime.mailbox_action_cursor.is_none()
            &&runtime.pending_mail_read.is_none()&&!runtime.party_menu_open=> {
            let mut entries=Vec::new();
            push_visible_mailbox_dialog_entries(&mut entries,&runtime.shell.presentation_snapshot()?,runtime)?;
            Ok(Some(entries))
        }
        "pc:mail-read" if runtime.mailbox_cursor.is_some() && !runtime.party_menu_open
            &&runtime.pending_mail_read.is_some()=> {
            let reader=runtime.pending_mail_read.as_ref().unwrap();
            Ok(Some(vec![reader.mail.author.clone(),reader.mail.message.clone()]))
        }
        "pc:move-carry" if pc_move_carry_review_ready(runtime)=> {
            let mut entries=Vec::new();
            push_visible_storage_dialog_entries(&mut entries,&runtime.shell.presentation_snapshot()?,runtime)?;
            Ok(Some(entries))
        }
        "pc:save-prompt" | "pc:saved" | "pc:save-error" if runtime.save_flow.as_ref().is_some_and(|flow|
            matches!(flow.origin,VisibleSaveFlowOrigin::BillsPcChangeBox{..}|VisibleSaveFlowOrigin::BillsPcMove)
                && match wanted {
                    "pc:save-prompt"=>flow.stage==VisibleSaveFlowStage::Prompt,
                    "pc:saved"=>flow.stage==VisibleSaveFlowStage::Saved,
                    _=>flow.stage==VisibleSaveFlowStage::Error,
                })=> {
            let mut entries=Vec::new();
            push_visible_save_dialog_entries(&mut entries,&runtime.shell.presentation_snapshot()?,runtime)?;
            Ok(Some(entries))
        }
        _=>Ok(None),
    }
}

// Full-LCD PC captures require an explicitly named, actually reached owner.
// This observer neither opens a surface nor changes a cursor/session.
pub(super) fn pc_lcd_review_ready(runtime:&BevyRuntimeShell,wanted:&str)->bool {
    matches!(wanted,"pc:move-carry"|"pc:mail-read"|"pc:items"|"pc:quantity"|"pc:toss-confirmation"|"pc:oak-confirmation"|"pc:oak-shutdown"|"pc:oak-counts"|"pc:oak-closing"|"pc:toss-closing"|"service:name-rater-entry"|"field:party-actions"|"field:fly-destinations")
        &&retained_pc_review_menu(runtime,wanted).expect("observe current PC LCD owner").is_some()
}

pub(super) fn pc_move_carry_review_ready(runtime:&BevyRuntimeShell)->bool {
    runtime.pc_hub_session_open && runtime.bill_pc_move_open && !runtime.party_menu_open
        && runtime.bill_pc_move_source.is_some() && runtime.storage_cursor.is_some()
        && runtime.bill_pc_pokemon_action_cursor.is_none() && runtime.bill_pc_move_save.is_none()
        && runtime.special_boundary.is_none() && runtime.field_notice.is_none() && runtime.pc_notice.is_none()
        && runtime.pc_transfer_sequence.is_none()
}

#[cfg(test)]
#[test]
fn walk_steps_release_after_a_warp_to_identical_coordinates() {
    let start = ("RedsHouse1F".to_owned(), TilePosition::new(7, 1));
    assert!(!position_changed(None, "RedsHouse1F", start.1));
    assert!(!position_changed(Some(&start), "RedsHouse1F", start.1));
    assert!(position_changed(Some(&start), "RedsHouse1F", TilePosition::new(7, 0)));
    assert!(position_changed(Some(&start), "RedsHouse2F", start.1));
}

pub(super) fn install(app: &mut App, route: &str, path: &Path, capture_steps: bool, league: bool) -> Result<()> {
    // Short keyboard steps can settle before the usual review frame. Keep
    // the default unchanged while allowing offscreen mid-stride inspection.
    let capture_tick = match std::env::var("CRYSTAL_RENDER_TEST_WALK_CAPTURE_TICK") {
        Err(std::env::VarError::NotPresent) => 12,
        Ok(value) => value.parse::<usize>()?,
        Err(error) => anyhow::bail!(error),
    };
    anyhow::ensure!(capture_tick < 32, "walk capture tick must be from 0 to 31");
    let synthetic_input = match std::env::var("CRYSTAL_RENDER_TEST_WALK_INPUT") {
        Err(std::env::VarError::NotPresent) => false,
        Ok(value) if value == "keyboard" => false,
        Ok(value) if value == "synthetic" => true,
        _ => anyhow::bail!("CRYSTAL_RENDER_TEST_WALK_INPUT must be keyboard or synthetic"),
    };
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let route = route
        .chars()
        .map(|c| match c {
            'R' => Ok(KeyCode::ArrowRight),
            'L' => Ok(KeyCode::ArrowLeft),
            'U' => Ok(KeyCode::ArrowUp),
            'D' => Ok(KeyCode::ArrowDown),
            'A' => Ok(KeyCode::KeyZ),
            'B' => Ok(KeyCode::KeyX),
            'S' => Ok(KeyCode::Enter),
            _ => Err(anyhow::anyhow!("--walk accepts only R,L,U,D,A,B,S")),
        })
        .collect::<Result<Vec<_>>>()?;
    anyhow::ensure!(!route.is_empty(), "--walk must contain directions");
    let pc_input_pulse=match std::env::var("CRYSTAL_RENDER_TEST_WALK_PC_PULSE").as_deref() {
        Ok("1")=>true,Ok("0")|Err(std::env::VarError::NotPresent)=>false,
        _=>anyhow::bail!("CRYSTAL_RENDER_TEST_WALK_PC_PULSE must be 0 or 1"),
    };
    let hold_healing_balls=std::env::var("CRYSTAL_RENDER_TEST_HOLD_HEALING_BALLS").ok()
        .map(|v|v.parse::<u8>().context("healing hold count must be an integer")) .transpose()?;
    anyhow::ensure!(hold_healing_balls.is_none_or(|n|(1..=6).contains(&n)),"healing hold count must be between 1 and 6");
    let hold_field_notice=std::env::var("CRYSTAL_RENDER_TEST_HOLD_FIELD_NOTICE").ok();
    anyhow::ensure!(hold_field_notice.as_ref().is_none_or(|text|!text.is_empty()&&text.len()<=256),
        "field notice capture target must contain 1..256 bytes");
    let hold_menu_id=std::env::var("CRYSTAL_RENDER_TEST_HOLD_MENU_ID").ok();
    anyhow::ensure!(hold_menu_id.as_ref().is_none_or(|id|!id.is_empty()&&id.len()<=256),
        "menu capture target must contain 1..256 bytes");
    anyhow::ensure!(!league || synthetic_input, "League review requires synthetic native keyboard input");
    let pc_mail_review=match std::env::var("CRYSTAL_RENDER_TEST_WALK_PC_MAIL").as_deref() {
        Ok("1")=>true,Ok("0")|Err(std::env::VarError::NotPresent)=>false,
        _=>anyhow::bail!("CRYSTAL_RENDER_TEST_WALK_PC_MAIL must be 0 or 1"),
    };
    anyhow::ensure!(!pc_mail_review || (synthetic_input && pc_input_pulse
        &&hold_menu_id.as_deref()==Some("pc:mail-read")),
        "PC Mail review requires synthetic pulse input and the actual pc:mail-read capture owner");
    let pc_item_review=match std::env::var("CRYSTAL_RENDER_TEST_WALK_PC_ITEMS").as_deref() {
        Ok("1")=>true,Ok("0")|Err(std::env::VarError::NotPresent)=>false,
        _=>anyhow::bail!("CRYSTAL_RENDER_TEST_WALK_PC_ITEMS must be 0 or 1"),
    };
    anyhow::ensure!(!pc_item_review || (synthetic_input&&pc_input_pulse&&!pc_mail_review
        &&matches!(hold_menu_id.as_deref(),Some("pc:items"|"pc:quantity"|"pc:toss-confirmation"|"pc:oak-confirmation"|"pc:oak-shutdown"|"pc:oak-counts"|"pc:oak-closing"|"pc:toss-closing"))),
        "PC service review requires synthetic pulse input and an actual item or Oak confirmation owner");
    app.insert_resource(RenderWalk {
        league: league.then(|| league::LeagueReview::new(path)),
        route,
        tick: 0,
        finished: false,
        settle: 0,
        path: path.to_owned(),
        trace: "frame,map,origin_x,origin_y,center_x,center_y,builds,frame_ms,focused,input,tile_x,tile_y,terrain_map,terrain_pending,terrain_active,fade_alpha,warp_phase,runtime_map,route_key\n".into(),
        times: Vec::new(),
        first_builds: None,
        origins: default(),
        maps: default(),
        player_bounds: None,
        player_moved: false,
        capture_steps,
        capture_tick,
        arrival_reveal_captured: false,
        synthetic_input,
        pc_input_pulse,
        pc_mail_review,
        pc_item_review,
        step_input: std::env::var("CRYSTAL_RENDER_TEST_WALK_STEPS").as_deref()==Ok("1"),
        segment_position: None,
        running_input: std::env::var("CRYSTAL_RENDER_TEST_RUN").as_deref() == Ok("1"),
        hold_on_battle: std::env::var("CRYSTAL_RENDER_TEST_HOLD_BATTLE").as_deref() == Ok("1"),
        hold_on_move_menu: std::env::var("CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU").as_deref() == Ok("1"),
        hold_healing_balls,
        held_healing:false,
        hold_field_notice,
        held_field_notice:false,
        hold_menu_id,
        held_menu:false,
        keep_current_battler: std::env::var("CRYSTAL_RENDER_TEST_KEEP_CURRENT_BATTLER").as_deref()==Ok("1"),
        segment_key: None,
        first_position: None,
        authoritative_moved: false,
    })
    .add_systems(PreUpdate, drive.after(bevy::input::InputSystems));
    #[cfg(target_arch = "wasm32")]
    anyhow::ensure!(!synthetic_input, "synthetic walk input is native-only");
    #[cfg(not(target_arch = "wasm32"))]
    if synthetic_input {
        app.add_systems(
            Update,
            inject_synthetic_keys
                .after(handheld_input::gather_input)
                .before(apply_keyboard_input),
        );
    }
    Ok(())
}

// Native QA changes only the accepted key snapshot. The production input-owner
// dispatcher still owns movement, scripts, dialogue and menus. Real window focus
// remains observable, and the default physical-input path is unchanged.
#[cfg(not(target_arch = "wasm32"))]
fn inject_synthetic_keys(
    walk: Res<RenderWalk>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input: ResMut<handheld_input::HandheldInput>,
    shell: Res<BevyRuntimeShell>,
) {
    if walk.synthetic_input && walk.started() && !walk.finished {
        input.keys = keyboard.clone();
        if std::env::var_os("CRYSTAL_ENCOUNTER_TRACE").is_some() && keyboard.just_pressed(KeyCode::KeyZ) {
            eprintln!("review A: pose={:?} walk_ticks={} moving_objects={} interaction={:?}",
                shell.shell.session().snapshot(), shell.player_walk_frame_ticks,
                shell.object_walk_from.len(), shell.shell.current_overworld_interaction_checked());
        }
    } else if walk.synthetic_input && walk.finished {
        input.keys.reset_all();
    }
}

pub(super) fn drive(
    mut walk: ResMut<RenderWalk>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    status: Res<crystal_voxel_view::VoxelViewStatus>,
    frame: Res<crystal_render_api::VisualWorldFrame>,
    time: Res<Time<Real>>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut commands: Commands,
    capture: Option<Res<render_capture::Capture>>,
    runtime: Res<BevyRuntimeShell>,
    battle: Option<Res<crystal_render_api::VisualBattleFrame>>,
    mut settings: ResMut<crystal_voxel_view::VoxelViewSettings>,
    rendered: Res<RenderedViewport>,
    mut classic_warmup: Local<u32>,
    mut paired_hold:Option<ResMut<render_capture::PairedCaptureHold>>,
    mut virtual_time:ResMut<Time<Virtual>>,
    presentation: (Query<(), With<CompetitiveMoveMenuMarker>>,Query<&HealMachineWorldSprite>, Res<crystal_voxel_view::BattleViewStatus>),
) {
    if walk.finished {
        walk.settle += 1;
        return;
    }
    // Stop the QA input sequence at a real, visible field battle. This only
    // releases keys; it never creates an encounter or changes the controller.
    let visible_battle = battle.as_ref().is_some_and(|battle| battle.active
        && battle.battlers.iter().all(|b| b.as_ref().is_some_and(|b| b.visible))
        && battle.cues.is_empty());
    let visible_move_menu = visible_battle && move_menu_is_presented(&runtime, !presentation.0.is_empty());
    // Tactical UI review follows the real field encounter and pauses only
    // after ordinary A input opens the production FIGHT menu.
    if walk.started() && ((walk.hold_on_battle && visible_battle)
        || (walk.hold_on_move_menu && visible_move_menu)) {
        if walk.running_input {
            let origin = runtime.battle_origin.published()
                .expect("running battle capture requires a production origin");
            if origin.kind == BattleOriginKind::Wild
                && matches!(&origin.contact, Some(BattleOriginContact::OverworldEncounter {
                    surface: crate::core::world::encounters::EncounterSurface::Grass, ..
                })) {
                assert!(matches!(&origin.authoritative_step,
                    Some(StepOutcome::Moved { speed_multiplier: 2, .. })),
                    "running capture must witness the actual faster movement step");
                assert!(runtime.battle_origin.bound_walking.is_some(),
                    "running capture must retain the original route encounter evidence");
            }
        }
        walk.tick = walk.route.len() * 32;
        keyboard.reset_all();
    }
    if walk.started()&&!walk.held_healing&&status.active&&frame.active
        &&walk.hold_healing_balls.is_some_and(|n| {
            runtime.visible_heal_machine.as_ref().is_some_and(|a|a.party_count>=n)
            &&(frame.actors.iter().any(|a|a.id==crystal_render_api::VisualActorId::Effect(
                crystal_render_api::VisualEffectId::HealMachineBall(n-1)))
                // Hall uses its native source sprite overlay; clinic effects
                // are published as world actors. Require the actual drawn ball.
                ||(runtime.visible_heal_machine.as_ref().is_some_and(|a|a.kind==2)
                    &&presentation.1.iter().any(|marker|marker.0==
                        crystal_render_api::VisualEffectId::HealMachineBall(n-1))))
        }) {
        walk.held_healing=true;walk.tick=walk.route.len()*32;walk.settle=30;
        keyboard.reset_all();
        if let Some(hold)=paired_hold.as_mut() {hold.frozen=true;virtual_time.pause();}
    }
    if walk.started()&&!walk.held_menu&&let Some(id)=walk.hold_menu_id.as_deref()
        &&((status.active&&frame.active)||pc_lcd_review_ready(&runtime,id)
            ||(matches!(id,"field:save-confirmation"|"field:save-overwrite")&&runtime.save_menu_open))
        &&let Some(entries)=retained_pc_review_menu(&runtime,id).expect("read current PC review menu")
    {
        if walk.pc_mail_review {
            assert_eq!(runtime.mailbox_cursor.as_ref().map(|c|c.option_index),Some(1),
                "Mail review must reach the second letter through actual list navigation");
            let expected=&runtime.shell.session().state().mailbox.get(1)
                .expect("Mail review fixture has a second letter").mail;
            assert_eq!(&runtime.pending_mail_read.as_ref().expect("actual Mail reader").mail,expected,
                "Mail review must display the selected letter's complete data");
        }
        std::fs::write(walk.path.with_extension("menu-hold.txt"),format!(
            "menu_id={id}\nmap={}\ntile={:?}\noptions={entries:?}\n",
            runtime.shell.current_map_name(),runtime.shell.session().overworld().player.tile))
            .expect("write observed controller menu capture state");
        walk.held_menu=true;walk.tick=walk.route.len()*32;walk.settle=30;
        keyboard.reset_all();
        if let Some(hold)=paired_hold.as_mut() {hold.frozen=true;virtual_time.pause();}
    }
    // Observe a source menu reached by ordinary route input. Holding its
    // existing presentation releases keys before a later A selects a row;
    // it never opens a menu, sets a cursor or changes the script outcome.
    if walk.started()&&!walk.held_menu&&walk.hold_menu_id.is_some()
        &&runtime.field_notice.is_none()&&runtime.pc_notice.is_none()
        &&runtime.field_notice_queue.is_empty()&&runtime.pending_item_notification.is_none()
        &&runtime.visible_field_item_notice.is_none()
        &&let Ok(snapshot)=runtime.shell.presentation_snapshot()
        &&let Some(menu)=snapshot.ui.menu.as_ref()
        &&walk.hold_menu_id.as_deref()==Some(menu.menu_id.as_str())
        &&matches!(menu.source,crystal_runtime::RuntimeMenuSource::ScriptDefinition {..})
        &&menu.layout.vertical_menus.iter().any(|vertical|!vertical.options.is_empty())
    {
        let options=menu.layout.vertical_menus.iter().flat_map(|vertical|vertical.options.iter())
            .cloned().collect::<Vec<_>>();
        std::fs::write(walk.path.with_extension("menu-hold.txt"),format!(
            "menu_id={}\nmap={}\ntile={:?}\noptions={:?}\n",
            menu.menu_id,snapshot.overworld.map_name,snapshot.overworld.tile,options))
            .expect("write observed source menu capture state");
        walk.held_menu=true;walk.tick=walk.route.len()*32;walk.settle=30;
        keyboard.reset_all();
        if let Some(hold)=paired_hold.as_mut() {hold.frozen=true;virtual_time.pause();}
    }
    // Freeze only an actually revealed page from the live field text surface.
    // Releasing QA keys prevents the next A from skipping the review target;
    // the production dispatcher remains authoritative for every prior input.
    if walk.started()&&!walk.held_field_notice
        &&let Some((snapshot,page_index,visible_chars))=walk.hold_field_notice.as_ref().and_then(|wanted| {
            let snapshot=runtime.shell.presentation_snapshot().ok()?;
            let (page_index,visible_chars)=revealed_dialogue_page(&runtime,&snapshot,wanted)?;
            Some((snapshot,page_index,visible_chars))
        }) {
        // Presentation snapshots deliberately omit integrity hashes. Capture
        // this boundary once with the same authoritative hash as paired QA.
        let checksum=runtime.shell.state_checksum().expect("checksum observed dialogue state");
        let state=runtime.shell.session().state();
        std::fs::write(walk.path.with_extension("notice-hold.txt"),format!(
            "page={}\nvisible_chars={}\nmoney={}\ntext={}\n",
            page_index,visible_chars,state.money,
            walk.hold_field_notice.as_ref().unwrap())).expect("write observed notice capture state");
        let overrides=state.map_block_overrides.get(&snapshot.overworld.map_name)
            .into_iter().flat_map(|blocks|blocks.iter())
            .map(|(&(x,y),&block)|serde_json::json!([x,y,block])).collect::<Vec<_>>();
        let moves=snapshot.party.slots.iter().enumerate().map(|(slot,party)|serde_json::json!({
            "slot":slot,"moves":party.pokemon.moves.iter().map(|m|serde_json::json!({
                "name":m.name,"current_pp":m.current_pp})).collect::<Vec<_>>()
        })).collect::<Vec<_>>();
        let flags=["EVENT_LANCES_ROOM_ENTRANCE_CLOSED","EVENT_LANCES_ROOM_EXIT_OPEN","EVENT_BEAT_CHAMPION_LANCE"]
            .into_iter().map(|name|(name,state.flags.is_script_flag_set(name).ok()))
            .collect::<std::collections::BTreeMap<_,_>>();
        let witness=serde_json::json!({
            "map":snapshot.overworld.map_name,"tile":[snapshot.overworld.tile.x,snapshot.overworld.tile.y],
            "checksum":format!("{:?}",checksum),
            "source_label":snapshot.ui.text.as_ref().map(|text|text.label.as_str()),
            "special_notice":runtime.field_notice.is_some(),"text_window_open":snapshot.ui.text_window_open,
            "page":page_index,"visible_chars":visible_chars,"text":walk.hold_field_notice.as_ref().unwrap(),
            "party_moves":moves,"map_block_overrides":overrides,"observed_flags":flags,
        });
        std::fs::write(walk.path.with_extension("dialogue-hold.json"),
            serde_json::to_vec_pretty(&witness).expect("serialize observed dialogue state"))
            .expect("write observed dialogue state");
        walk.held_field_notice=true;walk.tick=walk.route.len()*32;walk.settle=30;
        keyboard.reset_all();
        if let Some(hold)=paired_hold.as_mut() {hold.frozen=true;virtual_time.pause();}
    }
    // Production keyboard handling requires real window focus. Pause the
    // route while focus is elsewhere, instead of consuming an unplayed step.
    if !walk.synthetic_input && !windows.single().is_ok_and(|(_, window)| window.focused) {
        for key in [
            KeyCode::ArrowRight,
            KeyCode::ArrowLeft,
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::KeyZ,
            KeyCode::KeyX,
            KeyCode::ShiftRight,
        ] {
            keyboard.release(key);
        }
        return;
    }
    if walk.first_builds.is_none() {
        if settings.enabled {
            if status.active_frames < 90 { return; }
        } else {
            // A matched classic review has no active voxel renderer to warm.
            // Wait for the real source field instead, using the same passive
            // warmup before sending ordinary inputs through the controller.
            // Optional world extraction is deliberately disabled in classic
            // mode. Its inactive frame cannot gate the actual LCD renderer.
            if rendered.title_active || rendered.map_name.is_none() { return; }
            *classic_warmup = classic_warmup.saturating_add(1);
            if *classic_warmup < 90 { return; }
        }
    }
    walk.first_builds.get_or_insert(status.terrain_builds);
    let tile = runtime.shell.session().overworld().player.tile;
    let map = runtime.shell.current_map_name();
    walk.first_position.get_or_insert_with(|| (map.to_owned(), tile));
    walk.authoritative_moved |= position_changed(walk.first_position.as_ref(), map, tile);
    if let Some(mut league) = walk.league.take() {
        keyboard.reset_all();
        let done = league.frame(&runtime, &frame, &status, battle.as_deref(), &presentation.2, &mut settings,
            &mut keyboard, &mut commands, capture.as_deref(),
            windows.single().ok().map(|(entity, _)| entity),
            paired_hold.as_deref_mut().expect("League paired capture hold"),
            &mut virtual_time, walk.tick).expect("continuous native League review");
        walk.tick += 1;
        walk.finished = done;
        walk.league = Some(league);
        return;
    }
    if walk.step_input && walk.started() && status.terrain_pending {
        // A debug terrain build can take longer than a step. Release input
        // while it fits the accepted position; don't audit empty/stale frames.
        for key in [KeyCode::ArrowRight,KeyCode::ArrowLeft,KeyCode::ArrowUp,
            KeyCode::ArrowDown,KeyCode::KeyZ,KeyCode::KeyX,KeyCode::ShiftRight] {
            keyboard.release(key);
        }
        return;
    }
    for key in [
        KeyCode::ArrowRight,
        KeyCode::ArrowLeft,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::KeyZ,
        KeyCode::KeyX,
        KeyCode::ShiftRight,
    ] {
        if walk.tick % 32 == 0 {
            keyboard.release(key);
        }
    }
    if walk.tick % 32 == 0 {walk.segment_position=Some((map.to_owned(),tile));}
    let segment = walk.tick / 32;
    if segment == walk.route.len() {
        assert!(!walk.hold_on_battle || visible_battle,
            "field battle review exhausted its ordinary inputs without a visible battle");
        assert!(!walk.hold_on_move_menu || visible_move_menu,
            "move menu review exhausted ordinary inputs before FIGHT opened");
        assert!(walk.hold_healing_balls.is_none()||walk.held_healing,
            "healing review exhausted ordinary inputs before the requested live ball count");
        assert!(walk.hold_field_notice.is_none()||walk.held_field_notice,
            "notice review exhausted ordinary inputs before the requested fully visible page");
        assert!(walk.hold_menu_id.is_none()||walk.held_menu,
            "menu review exhausted ordinary inputs before the requested source menu");
        walk.finished = true;
        std::fs::write(walk.path.with_extension("movement.csv"), &walk.trace)
            .expect("write movement trace");
        let mut times = walk.times.clone();
        times.sort_by(f64::total_cmp);
        println!(
            "movement QA: {} frames, {} grid origins, {} additional terrain builds, median {:.2} ms, p95 {:.2} ms",
            times.len(),
            walk.origins.len(),
            status.terrain_builds - walk.first_builds.unwrap(),
            times[times.len() / 2],
            times[times.len() * 95 / 100]
        );
        if walk.route.iter().any(|key| matches!(key, KeyCode::ArrowRight | KeyCode::ArrowLeft | KeyCode::ArrowUp | KeyCode::ArrowDown)) {
        if settings.enabled {
            assert!(
                walk.origins.len() > 1 || walk.maps.len() > 1 || walk.player_moved,
                "movement QA requires player translation, a tile-origin change, or a map boundary"
            );
        }
        // Classic captures have no active voxel frame. They still require
        // movement of the same authoritative production map/tile identity.
        assert!(
            walk.authoritative_moved,
            "movement QA requires authoritative production tile coordinates to change"
        );
        }
        return;
    }
    // Small rooms keep the grid fixed while the player walks across it.
    // Require a full source-tile span, so facing/stride changes alone do not
    // satisfy movement. Existing grid/map evidence covers scrolling scenes.
    if let Some(player) = frame
        .actors
        .iter()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
    {
        let point = player.center - frame.center;
        let (low, high) = walk.player_bounds.unwrap_or((point, point));
        let low = low.min(point);
        let high = high.max(point);
        let span = high - low;
        walk.player_moved |= span.x >= frame.tile_size.x || span.y >= frame.tile_size.y;
        walk.player_bounds = Some((low, high));
    }
    // Once the real encounter starts, route arrows can select PKMN while
    // differently timed introductions finish. This opt-in review advances
    // source pages and opens FIGHT with ordinary A inputs instead.
    let review_battle_menu = walk.hold_on_move_menu
        && battle.as_ref().is_some_and(|battle| battle.active);
    let requested_key = if review_battle_menu {
        KeyCode::KeyZ
    } else if walk.pc_mail_review {
        pc_mail_review_key(&runtime).unwrap_or(walk.route[segment])
    } else if walk.pc_item_review {
        pc_item_review_key(&runtime,walk.hold_menu_id.as_deref().expect("validated item review target")).unwrap_or(walk.route[segment])
    } else {walk.route[segment]};
    let replace_battle_arrow = review_battle_menu && walk.segment_key.is_some_and(|key|
        matches!(key, KeyCode::ArrowUp | KeyCode::ArrowRight | KeyCode::ArrowDown | KeyCode::ArrowLeft));
    if walk.tick % 32 == 0 || walk.segment_key.is_none() || replace_battle_arrow {
        if replace_battle_arrow { keyboard.release(walk.segment_key.unwrap()); }
        // Opt-in QA policy: keep the current battler by pressing ordinary B
        // at the actual Shift choice. Latch for this whole input segment so
        // closing the prompt cannot leave both A and B held simultaneously.
        walk.segment_key = Some(if walk.keep_current_battler && requested_key == KeyCode::KeyZ
            && runtime.battle_shift_prompt_cursor.is_some() && runtime.battle_messages.is_empty()
        {KeyCode::KeyX} else {requested_key});
    }
    let key = walk.segment_key.expect("current route segment has a key");
    if walk.running_input && matches!(key,
        KeyCode::ArrowUp | KeyCode::ArrowRight | KeyCode::ArrowDown | KeyCode::ArrowLeft) {
        keyboard.press(KeyCode::KeyX);
    }
    // Hold the ordinary key until one authoritative step commits, then let
    // its production interpolation settle. Turns can consume an initial press.
    // Hold through the source menu's BG upload, then leave an actual release
    // between presses, as required by its JoyTextDelay history.
    let pulse_pc=walk.pc_input_pulse && (runtime.pc_hub_session_open
        || runtime.party_menu_open || runtime.start_menu_cursor.is_some() || runtime.fly_cursor.is_some()
        || walk.hold_menu_id.as_deref()==Some("service:name-rater-entry")
        || (matches!(walk.hold_menu_id.as_deref(),Some("field:save-confirmation"|"field:save-overwrite"))
            &&(runtime.start_menu_cursor.is_some()||runtime.save_menu_open)));
    // Reaching the source move owner is enough to stop choosing actions while
    // the native menu finishes rendering; the capture still waits for it.
    if (review_battle_menu && move_menu_is_presented(&runtime, true))
        || ((pulse_pc || review_battle_menu) && walk.tick%32>=8)
        || (walk.step_input && position_changed(walk.segment_position.as_ref(), map, tile)) {
        keyboard.release(key);
    } else {
        keyboard.press(key);
    }
    let dt = time.delta_secs_f64() * 1000.0;
    walk.times.push(dt);
    walk.maps.insert(frame.map_id.to_string());
    walk.origins
        .insert((frame.grid_origin.x, frame.grid_origin.y));
    let line = format!(
        "{},{},{},{},{:.3},{:.3},{},{:.3},{},{},{},{},{},{},{},{},{:?},{},{:?}\n",
        walk.tick,
        frame.map_id,
        frame.grid_origin.x,
        frame.grid_origin.y,
        frame.center.x,
        frame.center.y,
        status.terrain_builds,
        dt,
        windows.single().is_ok_and(|(_, window)| window.focused),
        if walk.synthetic_input {
            "synthetic"
        } else {
            "keyboard"
        },
        tile.x,
        tile.y,
        status.terrain_map.as_deref().unwrap_or(""),
        status.terrain_pending,
        status.active,
        runtime.screen_fade.map_or(0, |fade| fade.alpha),
        runtime.visible_walk_warp_phase,
        runtime.shell.current_map_name(),
        key,
    );
    walk.trace.push_str(&line);
    if runtime.visible_walk_warp_phase == Some(VisibleWalkWarpPhase::FadeOut) {
        walk.arrival_reveal_captured = false;
    }
    if walk.capture_steps && !walk.arrival_reveal_captured
        && runtime.visible_walk_warp_phase == Some(VisibleWalkWarpPhase::FadeIn)
        && runtime.screen_fade.is_some_and(|fade| fade.alpha < 255)
    {
        if let Ok((window, _)) = windows.single() {
            if let Some(screenshot) = render_capture::moving_screenshot(capture.as_deref(), window) {
                let stem = walk.path.file_stem().unwrap().to_string_lossy();
                let path = walk.path.with_file_name(format!("{stem}-arrival-reveal-{}.png",walk.tick));
                commands.spawn(screenshot).observe(bevy::render::view::screenshot::save_to_disk(path));
                walk.arrival_reveal_captured = true;
            }
        }
    }
    if walk.capture_steps && walk.tick % 32 == walk.capture_tick {
        if let Ok((window, _)) = windows.single() {
            let stem = walk.path.file_stem().unwrap().to_string_lossy();
            let path = walk
                .path
                .with_file_name(format!("{stem}-moving-{segment:02}.png"));
            if let Some(screenshot) = render_capture::moving_screenshot(capture.as_deref(), window)
            {
                commands
                    .spawn(screenshot)
                    .observe(bevy::render::view::screenshot::save_to_disk(path));
            }
        }
    }
    if walk.step_input && walk.tick % 32 == 31 {
        std::fs::write(walk.path.with_extension("movement.csv"), &walk.trace)
            .expect("write exact route progress");
    }
    walk.tick += 1;
}
