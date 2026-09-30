fn visible_move_animation_definition(
    snapshot: &RuntimeShellSnapshot,
    move_id: &str,
    animation_param: i32,
) -> Option<(
    String,
    u16,
    Vec<(u16, String)>,
    Vec<(u16, u8)>,
    Vec<VisibleMoveObjectEvent>,
    Vec<VisibleMoveBgEvent>,
)> {
    let normalized = move_id.replace(' ', "_").to_ascii_uppercase();
    // The verified catalog carries the actual one-based source index. Internal
    // IDs can differ from display text (PSYCHIC_M is displayed as PSYCHIC).
    // Resolve those IDs without guessing a label or losing their animation.
    let table_index = snapshot
        .moves
        .iter()
        .find(|entry| entry.move_id == normalized)
        .map(|entry| usize::from(entry.source_index))
        .or_else(|| {
            snapshot.presentation.move_names.iter()
                .position(|name| name.replace(' ', "_").to_ascii_uppercase() == normalized)
                .map(|index| index.saturating_add(1))
        })?;
    // BattleAnimations begins with the status-animation entry; move one is
    // therefore table entry one, matching TypeScript's table.slice(1).
    let label = snapshot
        .presentation
        .battle_animation_table
        .get(table_index)?
        .clone();
    visible_battle_animation_definition(snapshot, label, animation_param)
}

fn visible_battle_animation_definition(
    snapshot: &RuntimeShellSnapshot,
    label: String,
    animation_param: i32,
) -> Option<(
    String,
    u16,
    Vec<(u16, String)>,
    Vec<(u16, u8)>,
    Vec<VisibleMoveObjectEvent>,
    Vec<VisibleMoveBgEvent>,
)> {
    let (timeline_frame, sound_events, cry_events, object_events, bg_events) =
        compile_visible_battle_animation_timeline(snapshot, &label, animation_param)?;
    // RunBattleAnimCommand stops the root script at anim_ret. Live objects
    // and BG effects do not extend it; RunBattleAnimScript clears OAM as soon
    // as that tick completes. Events are one-based, so include the return tick.
    Some((
        label,
        timeline_frame.saturating_add(1),
        sound_events,
        cry_events,
        object_events,
        bg_events,
    ))
}

fn visible_move_animation_definition_with_substitute(
    snapshot: &RuntimeShellSnapshot,
    move_id: &str,
    animation_param: i32,
    lower_substitute: bool,
    raise_substitute: bool,
) -> Option<(
    String,
    u16,
    Vec<(u16, String)>,
    Vec<(u16, u8)>,
    Vec<VisibleMoveObjectEvent>,
    Vec<VisibleMoveBgEvent>,
)> {
    if !lower_substitute && !raise_substitute {
        return visible_move_animation_definition(snapshot, move_id, animation_param);
    }
    let mut parts = Vec::new();
    if lower_substitute {
        parts.push(visible_move_animation_definition(
            snapshot,
            "SUBSTITUTE",
            1,
        )?);
    }
    parts.push(visible_move_animation_definition(
        snapshot,
        move_id,
        animation_param,
    )?);
    if raise_substitute {
        parts.push(visible_move_animation_definition(
            snapshot,
            "SUBSTITUTE",
            2,
        )?);
    }

    let mut labels = Vec::new();
    let mut total_frames = 0_u16;
    let mut sounds = Vec::new();
    let mut cries = Vec::new();
    let mut objects = Vec::new();
    let mut bg_effects = Vec::new();
    for (label, frames, part_sounds, part_cries, part_objects, part_bg_effects) in parts {
        labels.push(label);
        sounds.extend(
            part_sounds
                .into_iter()
                .map(|(frame, sound)| (frame.saturating_add(total_frames), sound)),
        );
        cries.extend(
            part_cries
                .into_iter()
                .map(|(frame, selector)| (frame.saturating_add(total_frames), selector)),
        );
        objects.extend(part_objects.into_iter().map(|mut event| {
            event.frame = event.frame.saturating_add(total_frames);
            event
        }));
        bg_effects.extend(part_bg_effects.into_iter().map(|mut event| {
            event.frame = event.frame.saturating_add(total_frames);
            event
        }));
        total_frames = total_frames.saturating_add(frames);
    }
    Some((
        labels.join(" → "),
        total_frames,
        sounds,
        cries,
        objects,
        bg_effects,
    ))
}

fn visible_substitute_move_delay_definition(
    snapshot: &RuntimeShellSnapshot,
) -> Option<(
    String,
    u16,
    Vec<(u16, String)>,
    Vec<(u16, u8)>,
    Vec<VisibleMoveObjectEvent>,
    Vec<VisibleMoveBgEvent>,
)> {
    let (_, lower_frames, mut sounds, mut cries, mut objects, mut bg_effects) =
        visible_move_animation_definition(snapshot, "SUBSTITUTE", 1)?;
    let (_, raise_frames, raise_sounds, raise_cries, raise_objects, raise_bg_effects) =
        visible_move_animation_definition(snapshot, "SUBSTITUTE", 2)?;
    let raise_offset = lower_frames.saturating_add(40);
    sounds.extend(
        raise_sounds
            .into_iter()
            .map(|(frame, sound)| (frame.saturating_add(raise_offset), sound)),
    );
    cries.extend(
        raise_cries
            .into_iter()
            .map(|(frame, selector)| (frame.saturating_add(raise_offset), selector)),
    );
    objects.extend(raise_objects.into_iter().map(|mut event| {
        event.frame = event.frame.saturating_add(raise_offset);
        event
    }));
    bg_effects.extend(raise_bg_effects.into_iter().map(|mut event| {
        event.frame = event.frame.saturating_add(raise_offset);
        event
    }));
    Some((
        "BattleCommand_LowerSub_MoveDelay_RaiseSub".to_string(),
        raise_offset.saturating_add(raise_frames),
        sounds,
        cries,
        objects,
        bg_effects,
    ))
}

fn visible_substitute_raise_after_delay_definition(
    snapshot: &RuntimeShellSnapshot,
) -> Option<(
    String,
    u16,
    Vec<(u16, String)>,
    Vec<(u16, u8)>,
    Vec<VisibleMoveObjectEvent>,
    Vec<VisibleMoveBgEvent>,
)> {
    let (_, raise_frames, mut sounds, mut cries, mut objects, mut bg_effects) =
        visible_move_animation_definition(snapshot, "SUBSTITUTE", 2)?;
    for (frame, _) in &mut sounds {
        *frame = frame.saturating_add(40);
    }
    for (frame, _) in &mut cries {
        *frame = frame.saturating_add(40);
    }
    for event in &mut objects {
        event.frame = event.frame.saturating_add(40);
    }
    for event in &mut bg_effects {
        event.frame = event.frame.saturating_add(40);
    }
    Some((
        "BattleCommand_MoveDelay_RaiseSub".to_string(),
        40_u16.saturating_add(raise_frames),
        sounds,
        cries,
        objects,
        bg_effects,
    ))
}

#[derive(Default)]
struct VisibleBattleAnimationTimeline {
    frame: u16,
    sounds: Vec<(u16, String)>,
    cries: Vec<(u16, u8)>,
    objects: Vec<VisibleMoveObjectEvent>,
    bg_effects: Vec<VisibleMoveBgEvent>,
    loops: std::collections::BTreeMap<(String, usize), i32>,
    anim_var: i32,
    anim_param: i32,
    commands_executed: usize,
}

fn compile_visible_battle_animation_timeline(
    snapshot: &RuntimeShellSnapshot,
    root_label: &str,
    animation_param: i32,
) -> Option<(
    u16,
    Vec<(u16, String)>,
    Vec<(u16, u8)>,
    Vec<VisibleMoveObjectEvent>,
    Vec<VisibleMoveBgEvent>,
)> {
    let mut timeline = VisibleBattleAnimationTimeline {
        anim_param: animation_param,
        ..Default::default()
    };
    execute_visible_battle_animation_script(snapshot, root_label, &mut timeline, 0)?;
    Some((
        timeline.frame,
        timeline.sounds,
        timeline.cries,
        timeline.objects,
        timeline.bg_effects,
    ))
}

fn execute_visible_battle_animation_script(
    snapshot: &RuntimeShellSnapshot,
    script_label: &str,
    timeline: &mut VisibleBattleAnimationTimeline,
    depth: usize,
) -> Option<()> {
    if depth > 32 {
        return None;
    }
    let source = snapshot.presentation.battle_animations.get(script_label)?;
    let mut labels = std::collections::BTreeMap::<String, usize>::new();
    let mut commands = Vec::<String>::new();
    for line in source {
        let trimmed = line.trim();
        if trimmed.starts_with('.') && !trimmed.contains(char::is_whitespace) {
            labels.insert(trimmed.to_string(), commands.len());
        } else {
            commands.push(trimmed.to_string());
        }
    }
    let mut pointer = 0_usize;
    while pointer < commands.len() {
        timeline.commands_executed = timeline.commands_executed.saturating_add(1);
        if timeline.commands_executed > 65_535 {
            return None;
        }
        let command_index = pointer;
        let command = &commands[pointer];
        pointer += 1;
        let (opcode, raw_arguments) = command
            .split_once(char::is_whitespace)
            .map_or((command.as_str(), ""), |(opcode, arguments)| {
                (opcode, arguments)
            });
        let arguments = raw_arguments
            .split(',')
            .map(str::trim)
            .filter(|argument| !argument.is_empty())
            .collect::<Vec<_>>();
        match opcode {
            "anim_wait" => {
                let frames = arguments
                    .first()
                    .and_then(|argument| parse_visible_battle_animation_int(argument))
                    .and_then(|frames| u16::try_from(frames).ok())?;
                timeline.frame = timeline.frame.saturating_add(frames);
            }
            "anim_sound" => {
                if let Some(sound) = arguments.get(2) {
                    timeline
                        .sounds
                        .push((timeline.frame.saturating_add(1), (*sound).to_string()));
                }
            }
            "anim_cry" => {
                let selector = arguments
                    .first()
                    .and_then(|argument| parse_visible_battle_animation_int(argument))
                    .unwrap_or(0);
                timeline.cries.push((
                    timeline.frame.saturating_add(1),
                    u8::try_from(selector & 0x03).ok()?,
                ));
            }
            "anim_obj" if arguments.len() == 4 => {
                let (x, y, param) = (
                    parse_visible_battle_animation_int(arguments[1])?,
                    parse_visible_battle_animation_int(arguments[2])?,
                    parse_visible_battle_animation_int(arguments[3])?,
                );
                timeline.objects.push(VisibleMoveObjectEvent {
                    frame: timeline.frame.saturating_add(1),
                    command: VisibleMoveObjectCommand::Spawn {
                        object_id: arguments[0].to_string(),
                        x: i16::try_from(x).ok()?,
                        y: i16::try_from(y).ok()?,
                        param: u8::try_from(param & 0xff).ok()?,
                    },
                });
            }
            "anim_clearobjs" => timeline.objects.push(VisibleMoveObjectEvent {
                frame: timeline.frame.saturating_add(1),
                command: VisibleMoveObjectCommand::Clear,
            }),
            "anim_incobj" => {
                let slot = parse_visible_battle_animation_int(arguments.first()?)?;
                timeline.objects.push(VisibleMoveObjectEvent {
                    frame: timeline.frame.saturating_add(1),
                    command: VisibleMoveObjectCommand::Increment {
                        index: u8::try_from(slot).ok()?,
                    },
                });
            }
            "anim_setobj" => {
                let slot = parse_visible_battle_animation_int(arguments.first()?)?;
                let value = parse_visible_battle_animation_int(arguments.get(1)?)?;
                timeline.objects.push(VisibleMoveObjectEvent {
                    frame: timeline.frame.saturating_add(1),
                    command: VisibleMoveObjectCommand::Set {
                        index: u8::try_from(slot).ok()?,
                        value: u8::try_from(value & 0xff).ok()?,
                    },
                });
            }
            "anim_transform"
            | "anim_raisesub"
            | "anim_dropsub"
            | "anim_minimize"
            | "anim_updateactorpic" => {
                timeline.bg_effects.push(VisibleMoveBgEvent {
                    frame: timeline.frame.saturating_add(1),
                    effect_id: format!(
                        "BATTLE_ACTOR_{}",
                        opcode.trim_start_matches("anim_").to_ascii_uppercase()
                    ),
                    duration: 0,
                    target: "BG_EFFECT_USER".to_string(),
                    param: 0,
                    incremented: false,
                });
            }
            "anim_bgp" | "anim_obp0" | "anim_obp1" => {
                let value = parse_visible_battle_animation_int(arguments.first()?)?;
                timeline.bg_effects.push(VisibleMoveBgEvent {
                    frame: timeline.frame.saturating_add(1),
                    effect_id: format!(
                        "BATTLE_PALETTE_{}",
                        opcode.trim_start_matches("anim_").to_ascii_uppercase()
                    ),
                    duration: 0,
                    target: String::new(),
                    param: u8::try_from(value & 0xff).ok()?,
                    incremented: false,
                });
            }
            "anim_resetobp0" => timeline.bg_effects.push(VisibleMoveBgEvent {
                frame: timeline.frame.saturating_add(1),
                effect_id: "BATTLE_PALETTE_OBP0".to_string(),
                duration: 0,
                target: String::new(),
                // BattleAnimCmd_ResetObp0 writes $e0 unless hSGB is set. The
                // Rust runtime has no Super Game Boy execution mode.
                param: battle_anim_reset_obp0_value(false),
                incremented: false,
            }),
            "anim_beatup" => timeline.bg_effects.push(VisibleMoveBgEvent {
                frame: timeline.frame.saturating_add(1),
                effect_id: "BATTLE_ACTOR_BEATUP".to_string(),
                duration: 0,
                target: "BG_EFFECT_USER".to_string(),
                param: u8::try_from(timeline.anim_param & 0xff).ok()?,
                incremented: false,
            }),
            "anim_1gfx"
            | "anim_2gfx"
            | "anim_3gfx"
            | "anim_battlergfx_1row"
            | "anim_battlergfx_2row"
            | "anim_checkpokeball"
            | "anim_keepsprites" => {}
            "anim_bgeffect" if arguments.len() >= 4 => {
                let duration = parse_visible_battle_animation_int(arguments[1])?;
                // This byte is BG_EFFECT_STRUCT_BATTLE_TURN. Most effects use
                // the user/target constants, while palette effects use it as
                // their reload counter.
                parse_visible_battle_animation_int(arguments[2])?;
                let param = parse_visible_battle_animation_int(arguments[3])?;
                timeline.bg_effects.push(VisibleMoveBgEvent {
                    frame: timeline.frame.saturating_add(1),
                    effect_id: arguments[0].to_string(),
                    duration: u16::try_from(duration & 0xffff).ok()?,
                    target: arguments[2].to_string(),
                    param: u8::try_from(param & 0xff).ok()?,
                    incremented: false,
                });
            }
            "anim_incbgeffect" => {
                timeline.bg_effects.push(VisibleMoveBgEvent {
                    frame: timeline.frame.saturating_add(1),
                    effect_id: arguments.first()?.to_string(),
                    duration: 0,
                    target: String::new(),
                    param: 0,
                    incremented: true,
                });
            }
            "anim_call" => {
                let target = *arguments.first()?;
                if target.starts_with('.') {
                    return None;
                }
                execute_visible_battle_animation_script(snapshot, target, timeline, depth + 1)?;
            }
            "anim_jump" => {
                let target = *arguments.first()?;
                if target.starts_with('.') {
                    pointer = *labels.get(target)?;
                } else {
                    execute_visible_battle_animation_script(snapshot, target, timeline, depth + 1)?;
                    return Some(());
                }
            }
            "anim_loop" => {
                let count = parse_visible_battle_animation_int(arguments.first()?)?;
                let target = *arguments.get(1)?;
                let key = (script_label.to_string(), command_index);
                if advance_visible_battle_animation_loop(&mut timeline.loops, key, count) {
                    pointer = *labels.get(target)?;
                }
            }
            "anim_setvar" => {
                timeline.anim_var = parse_visible_battle_animation_int(arguments.first()?)?;
            }
            "anim_incvar" => timeline.anim_var = (timeline.anim_var + 1) & 0xff,
            "anim_if_var_equal" => {
                let value = parse_visible_battle_animation_int(arguments.first()?)?;
                if timeline.anim_var == value {
                    let target = *arguments.get(1)?;
                    if target.starts_with('.') {
                        pointer = *labels.get(target)?;
                    } else {
                        execute_visible_battle_animation_script(
                            snapshot,
                            target,
                            timeline,
                            depth + 1,
                        )?;
                        return Some(());
                    }
                }
            }
            "anim_if_param_equal" => {
                let value = parse_visible_battle_animation_int(arguments.first()?)?;
                if timeline.anim_param == value {
                    let target = *arguments.get(1)?;
                    if target.starts_with('.') {
                        pointer = *labels.get(target)?;
                    } else {
                        execute_visible_battle_animation_script(
                            snapshot,
                            target,
                            timeline,
                            depth + 1,
                        )?;
                        return Some(());
                    }
                }
            }
            "anim_if_param_and" => {
                let mask = parse_visible_battle_animation_int(arguments.first()?)?;
                if timeline.anim_param & mask != 0 {
                    pointer = *labels.get(*arguments.get(1)?)?;
                }
            }
            "anim_jumpuntil" => {
                if timeline.anim_param > 0 {
                    timeline.anim_param -= 1;
                    pointer = *labels.get(*arguments.first()?)?;
                }
            }
            "anim_ret" => return Some(()),
            _ => return None,
        }
    }
    Some(())
}

fn advance_visible_battle_animation_loop(
    loops: &mut std::collections::BTreeMap<(String, usize), i32>,
    key: (String, usize),
    count: i32,
) -> bool {
    if let Some(remaining) = loops.get_mut(&key) {
        if *remaining < 0 {
            return true;
        }
        if *remaining > 0 {
            *remaining -= 1;
            return true;
        }
        loops.remove(&key);
        return false;
    }
    if count <= 0 {
        loops.insert(key, -1);
        return true;
    }
    if count > 1 {
        loops.insert(key, count - 2);
        return true;
    }
    false
}

fn battle_anim_reset_obp0_value(super_game_boy: bool) -> u8 {
    if super_game_boy { 0xf0 } else { 0xe0 }
}

fn visible_battle_animation_frameset_lifetime(
    bundle: &serde_json::Value,
    frameset_name: &str,
) -> Option<u16> {
    let frames = bundle.get("framesets")?.get(frameset_name)?.as_array()?;
    let mut duration = 0_u16;
    for frame in frames {
        match frame.get("command")?.as_str()? {
            "frame" | "wait" => {
                let frames = frame.get("duration")?.as_u64()?.saturating_add(1);
                duration = duration.saturating_add(u16::try_from(frames).ok()?);
            }
            "delete" => return Some(duration.max(1)),
            "restart" | "end" => return None,
            _ => return None,
        }
    }
    None
}

fn parse_visible_battle_animation_int(token: &str) -> Option<i32> {
    let token = token.trim().replace('_', "");
    if let Some(hex) = token.strip_prefix('$') {
        i32::from_str_radix(hex, 16).ok()
    } else if let Some(binary) = token.strip_prefix('%') {
        i32::from_str_radix(binary, 2).ok()
    } else if let Some(hex) = token.strip_prefix("0x") {
        i32::from_str_radix(hex, 16).ok()
    } else if let Some(binary) = token.strip_prefix("0b") {
        i32::from_str_radix(binary, 2).ok()
    } else {
        token.parse::<i32>().ok().or_else(|| match token.as_str() {
            // constants/item_constants.asm. These are the only symbolic byte
            // values used in numeric battle-animation command positions.
            "NOITEM" => Some(0x00),
            "MASTERBALL" => Some(0x01),
            "ULTRABALL" => Some(0x02),
            "GREATBALL" => Some(0x04),
            "BGEFFECTTARGET" => Some(0),
            "BGEFFECTUSER" => Some(1),
            _ => None,
        })
    }
}
