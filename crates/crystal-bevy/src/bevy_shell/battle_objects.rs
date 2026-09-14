use crate::battle_anim_machine::{Machine as BattleObjectMachine, program as battle_program};

#[derive(Clone)]
struct VisibleBattleObjectOam {
    entries: Vec<[u8; 4]>,
    rows: Vec<[bool; 8]>,
    origin: (i32, i32),
}

#[derive(Clone)]
struct VisibleBattleObjectFrame {
    event_index: usize,
    spawn_frame: u16,
    bytes: [u8; 24],
    frameset: &'static str,
    frame: usize,
    oam: VisibleBattleObjectOam,
}

struct VisibleBattleObjects {
    slots: [Option<VisibleBattleObjectFrame>; 10],
    owners: [Option<usize>; 10],
    next_tick: u32,
    next_event: usize,
    last_id: u8,
    obp0_write: Option<(u32, u8)>,
    source: Vec<VisibleMoveObjectEvent>,
    player: bool,
    label: String,
    machine: BattleObjectMachine,
}

fn battle_object_byte(object: &serde_json::Value, field: &str) -> Result<u8> {
    let value = object
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .with_context(|| format!("battle object has no numeric {field}"))?;
    u8::try_from(value).with_context(|| format!("battle object {field} exceeds a byte"))
}

fn battle_object_palette_id(name: &str) -> Result<u8> {
    match name {
        "PAL_BATTLE_OB_GRAY" => Ok(0),
        "PAL_BATTLE_OB_YELLOW" => Ok(1),
        "PAL_BATTLE_OB_RED" => Ok(2),
        "PAL_BATTLE_OB_GREEN" => Ok(3),
        "PAL_BATTLE_OB_BLUE" => Ok(4),
        "PAL_BATTLE_OB_BROWN" => Ok(5),
        _ => anyhow::bail!("unknown battle object palette {name}"),
    }
}

fn install_battle_object_data(
    machine: &mut BattleObjectMachine,
    bundle: &serde_json::Value,
) -> Result<()> {
    let mut cursor = 0x8000_u16;
    // Keep source definition order: GetBattleAnimFrame reads the byte after
    // oamdelete into duration before the object is deinitialized.
    let mut framesets: Vec<_> = battle_program::FRAMESETS.iter().enumerate().collect();
    framesets.sort_by_key(|(id, _)| {
        let address = battle_program::BATTLE_ANIM_FRAME_DATA + *id as u16 * 2;
        u16::from_le_bytes([machine.read(address), machine.read(address + 1)])
    });
    let source_addresses: Vec<_> = battle_program::FRAMESETS
        .iter()
        .enumerate()
        .map(|(id, _)| {
            let address = battle_program::BATTLE_ANIM_FRAME_DATA + id as u16 * 2;
            u16::from_le_bytes([machine.read(address), machine.read(address + 1)])
        })
        .collect();
    for (id, name) in framesets {
        let frames = bundle["framesets"][*name]
            .as_array()
            .with_context(|| format!("missing battle frameset {name}"))?;
        let mut bytes = Vec::new();
        for frame in frames {
            let command = frame["command"]
                .as_str()
                .context("missing frameset command")?;
            let code = match command {
                "frame" => battle_program::OAMSETS
                    .iter()
                    .position(|name| Some(*name) == frame["oam_set"].as_str())
                    .context("frameset references unknown OAM set")?
                    as u8,
                "end" => 0xff,
                "restart" => 0xfe,
                "wait" => 0xfd,
                "delete" => 0xfc,
                _ => anyhow::bail!("unknown frameset command {command}"),
            };
            let mut flags = 0;
            if command == "frame" || command == "wait" {
                flags = battle_object_byte(frame, "duration")?;
                anyhow::ensure!(flags & 0xc0 == 0, "frameset duration overlaps flip bits");
                if command == "frame" {
                    flags |=
                        u8::from(frame["xflip"].as_bool().context("missing frame xflip")?) << 6;
                    flags |=
                        u8::from(frame["yflip"].as_bool().context("missing frame yflip")?) << 7;
                }
            }
            bytes.push(code);
            if command == "frame" || command == "wait" {
                bytes.push(flags);
            }
        }
        // Delete also fetches the following physical byte. Some successors
        // are unused framesets absent from the public pointer table.
        if bytes.last() == Some(&0xfc) {
            bytes.push(machine.read(source_addresses[id] + bytes.len() as u16));
        }
        anyhow::ensure!(
            usize::from(cursor) + bytes.len() <= 0xa000,
            "battle frameset program exceeds data segment"
        );
        machine.install_data(
            battle_program::BATTLE_ANIM_FRAME_DATA + id as u16 * 2,
            &cursor.to_le_bytes(),
        );
        machine.install_data(cursor, &bytes);
        cursor += bytes.len() as u16;
    }
    cursor = 0xa000;
    for (id, name) in battle_program::OAMSETS.iter().enumerate() {
        let oam = &bundle["oam_sets"][*name];
        let entries = oam["entries"]
            .as_array()
            .with_context(|| format!("missing OAM set {name}"))?;
        let count = u8::try_from(entries.len()).context("OAM set exceeds byte count")?;
        anyhow::ensure!(count != 0, "OAM set must contain pieces");
        let mut bytes = Vec::new();
        for entry in entries {
            let x = entry["x"].as_i64().context("missing OAM x")?;
            let y = entry["y"].as_i64().context("missing OAM y")?;
            anyhow::ensure!(
                (-128..=255).contains(&x) && (-128..=255).contains(&y),
                "OAM coordinate exceeds byte range"
            );
            let attributes = battle_object_byte(entry, "attributes")?;
            bytes.extend([
                y as u8,
                x as u8,
                battle_object_byte(entry, "tile_id")?,
                attributes,
            ]);
        }
        anyhow::ensure!(
            usize::from(cursor) + bytes.len() <= 0xc000,
            "battle OAM program exceeds data segment"
        );
        let [lo, hi] = cursor.to_le_bytes();
        machine.install_data(
            battle_program::BATTLE_ANIM_O_A_M_DATA + id as u16 * 4,
            &[battle_object_byte(oam, "tile_offset")?, count, lo, hi],
        );
        machine.install_data(cursor, &bytes);
        cursor += bytes.len() as u16;
    }
    Ok(())
}

fn new_visible_battle_objects(
    bundle: &serde_json::Value,
    animation: &VisibleMoveAnimation,
) -> Result<VisibleBattleObjects> {
    let mut machine = BattleObjectMachine::new(animation.player_move);
    install_battle_object_data(&mut machine, bundle)?;
    // InitBattleAnimBuffer's three move-ID adjustments are source-owned.
    let move_id = match animation.animation_label.as_str() {
        "BattleAnim_Kinesis" => 134_u16,
        "BattleAnim_Softboiled" => 135,
        "BattleAnim_MilkDrink" => 208,
        _ => 0,
    };
    machine.write(battle_program::W_F_X_ANIM_I_D, move_id as u8);
    machine.write(battle_program::W_F_X_ANIM_I_D + 1, (move_id >> 8) as u8);
    Ok(VisibleBattleObjects {
        slots: std::array::from_fn(|_| None),
        machine,
        owners: [None; 10],
        next_tick: 0,
        next_event: 0,
        last_id: 0,
        obp0_write: None,
        source: animation.object_events.clone(),
        player: animation.player_move,
        label: animation.animation_label.clone(),
    })
}

fn advance_visible_battle_objects(
    playback: &mut VisibleBattleObjects,
    bundle: &serde_json::Value,
    animation: &VisibleMoveAnimation,
) -> Result<()> {
    let VisibleBattleObjects {
        machine,
        owners,
        last_id,
        obp0_write,
        next_event,
        next_tick,
        slots: output,
        ..
    } = playback;
    for tick in *next_tick..=u32::from(animation.frame) {
        while animation
            .object_events
            .get(*next_event)
            .is_some_and(|event| u32::from(event.frame) == tick)
        {
            let event_index = *next_event;
            let event = &animation.object_events[event_index];
            *next_event += 1;
            match &event.command {
                VisibleMoveObjectCommand::Spawn {
                    object_id,
                    x,
                    y,
                    param,
                } => {
                    if let Some(slot) = (0..10).find(|&slot| machine.object(slot)[0] == 0) {
                        let object = &bundle["objects"][object_id];
                        let function = battle_anim_object_function(object_id, object)?;
                        let function = battle_program::FUNCTIONS
                            .iter()
                            .position(|name| *name == function)
                            .with_context(|| {
                                format!("unsupported battle object function {function}")
                            })? as u8;
                        let frameset = object["frameset"]
                            .as_str()
                            .context("missing object frameset")?;
                        let frameset = battle_program::FRAMESETS
                            .iter()
                            .position(|name| *name == frameset)
                            .with_context(|| format!("unknown frameset {frameset}"))?
                            as u8;
                        let palette = battle_object_palette_id(
                            object["palette"]
                                .as_str()
                                .context("missing object palette")?,
                        )?;
                        *last_id = last_id.wrapping_add(1);
                        machine.initialize(
                            slot,
                            *last_id,
                            [
                                battle_object_byte(object, "flags")?,
                                battle_object_byte(object, "fix_y")?,
                                frameset,
                                function,
                                palette,
                                0,
                            ],
                            *x as u8,
                            *y as u8,
                            *param,
                        );
                        owners[slot] = Some(event_index);
                    }
                }
                VisibleMoveObjectCommand::Clear => machine.clear_objects(),
                VisibleMoveObjectCommand::Increment { index }
                | VisibleMoveObjectCommand::Set { index, .. } => {
                    if let Some(slot) = (0..10).find(|&slot| {
                        machine.object(slot)[0] != 0 && machine.object(slot)[0] == *index
                    }) {
                        let object = machine.object_mut(slot);
                        object[14] = match event.command {
                            VisibleMoveObjectCommand::Set { value, .. } => value,
                            _ => object[14].wrapping_add(1),
                        };
                    }
                }
            }
        }
        machine.obp0_write = None;
        machine.begin_oam();
        *output = std::array::from_fn(|_| None);
        let mut scanlines = [0_u8; 144];
        for slot in 0..10 {
            if machine.object(slot)[0] == 0 {
                continue;
            }
            machine.step_object(slot).map_err(anyhow::Error::msg)?;
            let before = usize::from(machine.read(battle_program::W_BATTLE_ANIM_O_A_M_POINTER_LO));
            let full = machine.oam_update(slot).map_err(anyhow::Error::msg)?;
            let after = usize::from(machine.read(battle_program::W_BATTLE_ANIM_O_A_M_POINTER_LO));
            let bytes: [u8; 24] = machine.object(slot).try_into().unwrap();
            let entries = machine.oam()[before..after]
                .chunks_exact(4)
                .map(|entry| entry.try_into().unwrap())
                .collect::<Vec<[u8; 4]>>();
            let rows = entries
                .iter()
                .map(|entry| {
                    std::array::from_fn(|row| {
                        let y = i32::from(entry[0]) - 16 + row as i32;
                        if !(0..144).contains(&y) {
                            return false;
                        }
                        let count = &mut scanlines[y as usize];
                        let visible = *count < 10;
                        *count = count.saturating_add(1);
                        visible
                    })
                })
                .collect();
            let x = machine
                .read(battle_program::W_BATTLE_ANIM_TEMP_X_COORD)
                .wrapping_add(machine.read(battle_program::W_BATTLE_ANIM_TEMP_X_OFFSET));
            let y = machine
                .read(battle_program::W_BATTLE_ANIM_TEMP_Y_COORD)
                .wrapping_add(machine.read(battle_program::W_BATTLE_ANIM_TEMP_Y_OFFSET));
            let event_index = owners[slot].context("active battle object has no creation event")?;
            let event = &animation.object_events[event_index];
            // A function may deinitialize while still emitting this tick's OAM.
            if bytes[0] != 0 || !entries.is_empty() {
                output[slot] = Some(VisibleBattleObjectFrame {
                    event_index,
                    spawn_frame: event.frame,
                    bytes,
                    frameset: *battle_program::FRAMESETS
                        .get(usize::from(bytes[3]))
                        .context("invalid live frameset ID")?,
                    frame: usize::from(bytes[13]),
                    oam: VisibleBattleObjectOam {
                        entries,
                        rows,
                        origin: (i32::from(x), i32::from(y)),
                    },
                });
            }
            if full {
                break;
            }
        }
        if let Some(value) = machine.obp0_write {
            *obp0_write = Some((tick, value));
        }
    }
    *next_tick = u32::from(animation.frame) + 1;
    Ok(())
}

#[cfg(test)]
fn visible_battle_objects(
    bundle: &serde_json::Value,
    animation: &VisibleMoveAnimation,
) -> Result<VisibleBattleObjects> {
    let mut playback = new_visible_battle_objects(bundle, animation)?;
    advance_visible_battle_objects(&mut playback, bundle, animation)?;
    Ok(playback)
}
