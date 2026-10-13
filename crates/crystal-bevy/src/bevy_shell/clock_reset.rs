// Source ClockResetPassword uses SRAM bytes, not the current title/new-game state.
fn visible_clock_password(state: &crate::core::state::GameState) -> Result<u16> {
    let name = crate::core::systems::radio_text::encode_radio_string(&state.player_name)?;
    let mut password = state
        .player_id
        .to_be_bytes()
        .into_iter()
        .map(u16::from)
        .sum::<u16>();
    for byte in name.into_iter().take(5).take_while(|byte| *byte != 0x50) {
        password = password.wrapping_add(u16::from(byte));
    }
    for byte in state.money.to_be_bytes().into_iter().skip(1) {
        password = password.wrapping_add(u16::from(byte));
    }
    Ok(password)
}

fn set_visible_clock_prompt(runtime: &mut BevyRuntimeShell, id: &str) -> Result<()> {
    let pages = visible_presentation_text_pages(
        runtime.runtime.title_presentation_program(),
        id,
        "PLAYER",
    )?;
    if let Some(clock) = runtime.pending_clock_reset.as_mut() {
        clock.prompt_pages = pages;
        clock.prompt_page = 0;
    }
    mark_runtime_presentation_dirty(runtime);
    Ok(())
}

fn advance_visible_clock_prompt(runtime: &mut BevyRuntimeShell) -> bool {
    let Some(clock) = runtime.pending_clock_reset.as_mut() else {
        return false;
    };
    if clock.prompt_page + 1 >= clock.prompt_pages.len() {
        return false;
    }
    clock.prompt_page += 1;
    mark_runtime_presentation_dirty(runtime);
    true
}

fn begin_visible_clock_password(runtime: &mut BevyRuntimeShell) -> Result<()> {
    let program = runtime.runtime.title_presentation_program();
    let block = program
        .entrypoints
        .get("reset_clock")
        .and_then(|entry| program.blocks.get(entry))
        .context("clock reset missing authored entrypoint")?;
    let recipe = block
        .operations
        .iter()
        .find_map(|operation| operation.fields.get("password"))
        .context("clock reset missing authored password recipe")?;
    anyhow::ensure!(
        recipe["player_id_bytes"] == 2
            && recipe["player_name_bytes"] == 5
            && recipe["money_bytes"] == 3
            && recipe["digit_count"] == 5
            && recipe["initial_cursor"] == 4
            && recipe["initial_digits"] == serde_json::json!([0, 0, 0, 0, 0])
            && recipe["rtc_reset_flag"] == 128,
        "clock reset password recipe is unsupported"
    );
    let path = runtime
        .title_menu
        .as_ref()
        .and_then(|title| title.save_path.clone())
        .context("clock reset requires a saved trainer")?;
    let save = crate::core::save::read_save_game_for_modpack(
        &path,
        runtime.shell.runtime().modpack(),
        &runtime.shell.runtime().pack_identity().content_hash,
    )?;
    let password = visible_clock_password(save.state())?;
    let clock = runtime
        .pending_clock_reset
        .as_mut()
        .context("clock reset prompt missing")?;
    clock.phase = VisibleClockResetPhase::Password;
    clock.expected_password = Some(password);
    clock.saved_state_hash = Some(save.metadata().saved_state_hash());
    clock.password_digits = [0; 5];
    clock.password_cursor = 4;
    set_visible_clock_prompt(runtime, "PasswordAskEnterText")?;
    record_visible_runtime_action(runtime, "clock_reset:password:open")
}

fn confirm_visible_clock_password(runtime: &mut BevyRuntimeShell) -> Result<()> {
    let clock = runtime
        .pending_clock_reset
        .as_ref()
        .context("clock password prompt missing")?;
    let entered = clock.password_digits.iter().fold(0_u16, |value, digit| {
        value.wrapping_mul(10).wrapping_add(u16::from(*digit))
    });
    if Some(entered) != clock.expected_password {
        runtime.pending_clock_reset.as_mut().unwrap().phase = VisibleClockResetPhase::WrongPassword;
        set_visible_clock_prompt(runtime, "PasswordWrongText")?;
        return record_visible_runtime_action(runtime, "clock_reset:password:rejected");
    }
    let hash = clock
        .saved_state_hash
        .context("clock password lacks saved generation")?;
    let path = runtime
        .title_menu
        .as_ref()
        .and_then(|title| title.save_path.clone())
        .context("clock password lost save path")?;
    crate::core::save::request_saved_clock_reset_for_modpack(
        &path,
        runtime.shell.runtime().modpack(),
        &runtime.shell.runtime().pack_identity().content_hash,
        hash,
    )?;
    runtime.pending_clock_reset.as_mut().unwrap().phase = VisibleClockResetPhase::Requested;
    set_visible_clock_prompt(runtime, "PasswordAskResetText")?;
    record_visible_runtime_action(runtime, "clock_reset:password:accepted")
}

fn move_visible_clock_reset_horizontal(runtime: &mut BevyRuntimeShell, delta: i8) -> Result<()> {
    let Some(clock) = runtime.pending_clock_reset.as_mut() else {
        return Ok(());
    };
    match clock.phase {
        VisibleClockResetPhase::Password => {
            clock.password_cursor =
                (clock.password_cursor as i16 + i16::from(delta)).clamp(0, 4) as usize;
        }
        VisibleClockResetPhase::SetDay
        | VisibleClockResetPhase::SetHour
        | VisibleClockResetPhase::SetMinute => {
            let division = match clock.phase {
                VisibleClockResetPhase::SetDay => 0,
                VisibleClockResetPhase::SetHour => 1,
                _ => 2,
            };
            clock.phase = match (division + i16::from(delta)).clamp(0, 2) {
                0 => VisibleClockResetPhase::SetDay,
                1 => VisibleClockResetPhase::SetHour,
                _ => VisibleClockResetPhase::SetMinute,
            };
            clock.time_division = match clock.phase {
                VisibleClockResetPhase::SetDay => 0,
                VisibleClockResetPhase::SetHour => 1,
                _ => 2,
            };
        }
        _ => return Ok(()),
    }
    mark_runtime_presentation_dirty(runtime);
    record_visible_runtime_action(runtime, "clock_reset:horizontal")
}

fn open_visible_restart_clock(runtime: &mut BevyRuntimeShell, path: PathBuf) -> Result<()> {
    let save = crate::core::save::read_save_game_for_modpack(
        &path,
        runtime.shell.runtime().modpack(),
        &runtime.shell.runtime().pack_identity().content_hash,
    )?;
    let time = &save.state().time;
    runtime.pending_clock_reset = Some(VisibleClockResetScreen {
        time_division: 0,
        phase: VisibleClockResetPhase::TimeWarning,
        confirm_selection: 0,
        day: time.day_of_week % 7,
        hour: time.registers.hours,
        minute: time.registers.minutes,
        password_digits: [0; 5],
        password_cursor: 4,
        expected_password: None,
        saved_state_hash: Some(save.metadata().saved_state_hash()),
        continue_save_path: Some(path),
        prompt_pages: Vec::new(),
        prompt_page: 0,
    });
    set_visible_clock_prompt(runtime, "ClockTimeMayBeWrongText")?;
    record_visible_runtime_action(runtime, "continue:restart_clock:open")
}

fn confirm_visible_restart_clock(runtime: &mut BevyRuntimeShell) -> Result<()> {
    let clock = runtime
        .pending_clock_reset
        .clone()
        .context("restart clock prompt missing")?;
    // YesNoBox has YES first; declining cancels Continue, retaining the reset flag.
    if clock.confirm_selection != 0 {
        return close_visible_clock_reset_screen(runtime, "restart_cancel");
    }
    let path = clock
        .continue_save_path
        .context("restart clock lost Continue save")?;
    let rtc = required_native_rtc_sample(runtime)?;
    crate::core::save::restart_saved_clock_for_modpack(
        &path,
        runtime.shell.runtime().modpack(),
        &runtime.shell.runtime().pack_identity().content_hash,
        clock
            .saved_state_hash
            .context("restart clock lost saved generation")?,
        rtc.date,
        ClockTime::new(0, rtc.hour, rtc.minute, rtc.second),
        ClockTime::new(clock.day, clock.hour, clock.minute, 0),
    )?;
    runtime.pending_clock_reset.as_mut().unwrap().phase = VisibleClockResetPhase::TimeResetDone;
    set_visible_clock_prompt(runtime, "ClockHasResetText")?;
    record_visible_runtime_action(runtime, "continue:restart_clock:confirmed")
}

fn finish_visible_restart_clock(runtime: &mut BevyRuntimeShell) -> Result<()> {
    if advance_visible_clock_prompt(runtime) {
        return Ok(());
    }
    let path = runtime
        .pending_clock_reset
        .take()
        .and_then(|clock| clock.continue_save_path)
        .context("completed clock restart lost Continue save")?;
    runtime.title_menu = None;
    load_visible_runtime_save(runtime, &path, "title_continue")
}

fn draw_visible_restart_clock_values(
    font: &image::RgbaImage,
    frame: &image::RgbaImage,
    up_arrow: &image::RgbaImage,
    clock: &VisibleClockResetScreen,
    data: &mut [u8],
) -> Result<()> {
    // RestartClock.PrintTime: Textbox at (0,5), five inner rows and
    // eighteen inner columns, followed by the authored day/time values.
    draw_time_set_window(frame, 0, 5, 20, 7, data)?;
    draw_time_set_textbox(
        font,
        frame,
        clock
            .prompt_pages
            .get(clock.prompt_page)
            .context("restart clock missing source text")?,
        0,
        12,
        20,
        6,
        data,
    )?;
    for (x, value) in [
        (
            1,
            format!("{}DAY", CLOCK_RESET_DAY_NAMES[usize::from(clock.day % 7)]),
        ),
        (12, format!("{:02}", clock.hour)),
        (15, format!("{:02}", clock.minute)),
    ] {
        draw_time_set_text(
            font,
            &value,
            x * SOURCE_TILE_SIZE,
            8 * SOURCE_TILE_SIZE,
            data,
        )?;
    }
    draw_time_set_text(font, ":", 14 * SOURCE_TILE_SIZE, 8 * SOURCE_TILE_SIZE, data)?;
    let x = match clock.time_division {
        0 => 4,
        1 => 12,
        _ => 15,
    };
    if matches!(
        clock.phase,
        VisibleClockResetPhase::SetDay
            | VisibleClockResetPhase::SetHour
            | VisibleClockResetPhase::SetMinute
            | VisibleClockResetPhase::TimeConfirm
            | VisibleClockResetPhase::TimeResetDone
    ) {
        blit_time_set_tile_image_clipped(up_arrow, 0, 0, x * SOURCE_TILE_SIZE,
            7 * SOURCE_TILE_SIZE, false, false, true, None, data);
        draw_time_set_text(font, "▼", x * SOURCE_TILE_SIZE, 9 * SOURCE_TILE_SIZE, data)?;
    }
    Ok(())
}
