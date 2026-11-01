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

// BG callbacks own these objects; they are not anim_obj script events.
#[derive(Clone)]
struct VisibleBattleBattlerRowFrame {
    bg_event_index: usize,
    spawn_frame: u16,
    player_side: bool,
    row_count: u8,
    bytes: [u8; 24],
    frameset: &'static str,
    frame: usize,
    oam: VisibleBattleObjectOam,
}

#[derive(Clone, Copy)]
enum VisibleBattleObjectOwner {
    Event(usize),
    BattlerRow {
        bg_event_index: usize,
        player_side: bool,
        row_count: u8,
    },
}

struct VisibleBattleObjects {
    slots: [Option<VisibleBattleObjectFrame>; 10],
    battler_rows: [Option<VisibleBattleBattlerRowFrame>; 10],
    owners: [Option<VisibleBattleObjectOwner>; 10],
    next_tick: u32,
    next_event: usize,
    last_id: u8,
    obp0_write: Option<(u32, u8)>,
    source: Vec<VisibleMoveObjectEvent>,
    bg_source: Vec<VisibleMoveBgEvent>,
    move_id: String,
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

// These are cartridge indices throughout the VM and emitted OAM. Translate
// their meaning only when resolving colors for a host texture.
fn battle_object_palette_id(name: &str) -> Result<u8> {
    (0..8).find(|&id| battle_object_palette_name(id).ok() == Some(name))
        .with_context(|| format!("unknown battle object palette {name}"))
}

fn battle_object_palette_name(id: u8) -> Result<&'static str> {
    ["PAL_BATTLE_OB_ENEMY", "PAL_BATTLE_OB_PLAYER", "PAL_BATTLE_OB_GRAY",
     "PAL_BATTLE_OB_YELLOW", "PAL_BATTLE_OB_RED", "PAL_BATTLE_OB_GREEN",
     "PAL_BATTLE_OB_BLUE", "PAL_BATTLE_OB_BROWN"]
        .get(usize::from(id)).copied()
        .with_context(|| format!("invalid cartridge battle palette {id}"))
}

fn capture_source_item_byte(ball: &str) -> Result<u8> {
    // Verified constants/item_constants.asm at pret/pokecrystal
    // 5beda23ffa505f62e1dad7e3d7c214d1737b3358. The current pack's Item
    // records do not contain numeric source IDs. Do not infer an ID from
    // catalog order or wBattleAnimParam (blocked/Kurt branches change it).
    Ok(match ball {
        "MASTER_BALL" => 0x01,
        "ULTRA_BALL" => 0x02,
        "GREAT_BALL" => 0x04,
        "POKE_BALL" => 0x05,
        "HEAVY_BALL" => 0x9d,
        "LEVEL_BALL" => 0x9f,
        "LURE_BALL" => 0xa0,
        "FAST_BALL" => 0xa1,
        "FRIEND_BALL" => 0xa4,
        "MOON_BALL" => 0xa5,
        "LOVE_BALL" => 0xa6,
        "PARK_BALL" => 0xb1,
        other => anyhow::bail!("capture ball {other} has no verified source item byte"),
    })
}

fn visible_capture_oam_slots(
    playback: &VisibleBattleObjects,
    capture: Option<&VisibleCaptureAnimation>,
) -> [Option<VisibleBattleObjectFrame>; 10] {
    let mut slots = playback.slots.clone();
    if capture.is_some_and(|capture| capture.caught && capture.complete
        && capture.frame >= capture.total_frames()) {
        // BattleAnim_ClearOAM changes only final shadow-OAM attributes after
        // the anim_ret tick. The object structs and interpreter stay frozen.
        for live in slots.iter_mut().flatten() {
            for entry in &mut live.oam.entries {
                entry[3] &= 0xf0;
            }
        }
    }
    slots
}

fn visible_battle_object_battler_palettes(
    snapshot: &RuntimeShellSnapshot,
    asset_root: &AssetRoot,
    animation: &VisibleMoveAnimation,
    slots: &[Option<VisibleBattleObjectFrame>; 10],
) -> Result<[Option<Palette>; 2]> {
    let needed = [0_u8, 1].map(|id| slots.iter().flatten().any(|live|
        live.oam.entries.iter().any(|piece| piece[3] & 7 == id)));
    let mut palettes = [None, None];
    if !needed.into_iter().any(|value| value) { return Ok(palettes); }
    let battle = snapshot.battle.as_ref().context("battler object palette lost battle")?;
    let (player_bgp, enemy_bgp) = visible_move_battler_bgps(Some(animation));
    let global_bgp = visible_battle_dmg_palette_registers(Some(animation)).bgp;
    for index in 0..2 {
        if !needed[index] { continue; }
        let (species, dvs, side, bgp) = if index == 0 {
            (battle.enemy_transformed_species.as_deref().unwrap_or(&battle.enemy_pokemon.species.id),
             battle.enemy_transformed_dvs.unwrap_or(battle.enemy_pokemon.dvs),
             PokemonSpriteSide::Front, enemy_bgp.unwrap_or(global_bgp))
        } else {
            let player = battle.active_player_party_index.and_then(|active|
                snapshot.party.slots.iter().find(|slot| slot.index == active))
                .context("player object palette lost active Pokemon")?;
            (battle.player_transformed_species.as_deref().unwrap_or(&player.pokemon.species.id),
             battle.player_transformed_dvs.unwrap_or(player.pokemon.dvs),
             PokemonSpriteSide::Back, player_bgp.unwrap_or(global_bgp))
        };
        let palette = load_pokemon_palette(asset_root,
            &pokemon_asset_id_for_dvs(species, dvs), side, visible_dvs_are_shiny(dvs))?;
        palettes[index] = Some(std::array::from_fn(|color|
            palette[usize::from((bgp >> (color * 2)) & 3)]));
    }
    Ok(palettes)
}

fn load_battle_object_palette(
    asset_root: &AssetRoot,
    id: u8,
    battler_palettes: &[Option<Palette>; 2],
) -> Result<[[u8; 4]; 4]> {
    if id < 2 {
        let palette = battler_palettes[usize::from(id)]
            .with_context(|| format!("cartridge battle palette {id} needs its battler colors"))?;
        return Ok(std::array::from_fn(|color| [palette[color][0], palette[color][1],
            palette[color][2], if color == 0 { 0 } else { 255 }]));
    }
    let name = ["gray", "yellow", "red", "green", "blue", "brown"]
        .get(usize::from(id - 2))
        .context("unsupported cartridge battle palette")?;
    load_battle_anim_palette(asset_root, name)
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
    if animation.animation_label == "BattleAnim_ThrowPokeBall" {
        let ball = animation.move_id.strip_prefix("THROW_")
            .context("capture animation is missing its item identity")?;
        machine.write(battle_program::W_CUR_ITEM, capture_source_item_byte(ball)?);
    }
    // InitBattleAnimBuffer's three move-ID adjustments are source-owned.
    // move_id retains the canonical invoked move (including called moves),
    // while animation_label can join Substitute's lower/move/raise scripts.
    // Never infer the source move from that composite presentation label.
    let move_id = match animation.move_id.as_str() {
        "KINESIS" => 134_u16,
        "SOFTBOILED" => 135,
        "MILK_DRINK" => 208,
        _ => 0,
    };
    machine.write(battle_program::W_F_X_ANIM_I_D, move_id as u8);
    machine.write(battle_program::W_F_X_ANIM_I_D + 1, (move_id >> 8) as u8);
    Ok(VisibleBattleObjects {
        slots: std::array::from_fn(|_| None),
        battler_rows: std::array::from_fn(|_| None),
        machine,
        owners: [None; 10],
        next_tick: 0,
        next_event: 0,
        last_id: 0,
        obp0_write: None,
        source: animation.object_events.clone(),
        bg_source: animation.bg_events.clone(),
        move_id: animation.move_id.clone(),
        player: animation.player_move,
        label: animation.animation_label.clone(),
    })
}

/// A single BG-owned source allocation, shared by the classic sprite host and
/// native row pilot. Script objects still allocate first on the same tick;
/// neither the slot nor its creation ID is implied by this plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VisibleBattleRowPlan {
    bg_event_index: usize,
    spawn_frame: u16,
    player_side: bool,
    row_count: u8,
}

fn visible_battle_row_plan(animation: &VisibleMoveAnimation) -> Option<VisibleBattleRowPlan> {
    // These unwrapped roots have independently checked row rendering. Source
    // allocation is broader; visual support and the native pilot stay scoped.
    if !matches!(
        (
            animation.move_id.as_str(),
            animation.animation_label.as_str()
        ),
        ("TACKLE", "BattleAnim_Tackle")
            | ("WATER_GUN", "BattleAnim_WaterGun")
            | ("FLAIL", "BattleAnim_Flail")
            | ("HEADBUTT", "BattleAnim_Headbutt")
            | ("THIEF", "BattleAnim_Thief")
    ) {
        return None;
    }
    visible_battle_source_row_plan(animation)
}

fn visible_battle_source_row_plan(animation: &VisibleMoveAnimation) -> Option<VisibleBattleRowPlan> {
    // BATTLEROBJ owns a real NULL object even when its visual effect uses the
    // classic fallback. It shares slots, creation IDs and OAM limits with
    // script objects; omitting it can redirect a later anim_incobj.
    // Only one ordinary initialization is represented by this plan. Multiple
    // rows, direct state changes and nonstandard parameters need their own
    // source implementation before they can be allocated here.
    let mut rows = animation.bg_events.iter().enumerate().filter(|(_, event)| {
        matches!(
            event.effect_id.as_str(),
            "BATTLE_BG_EFFECT_BATTLEROBJ_1ROW" | "BATTLE_BG_EFFECT_BATTLEROBJ_2ROW"
        )
    });
    let (bg_event_index, event) = rows.next()?;
    if rows.next().is_some() || event.incremented || event.duration != 0 || event.param != 0 {
        return None;
    }
    let player_side = match event.target.as_str() {
        "BG_EFFECT_USER" => animation.player_move,
        "BG_EFFECT_TARGET" => !animation.player_move,
        _ => return None,
    };
    Some(VisibleBattleRowPlan {
        bg_event_index,
        spawn_frame: event.frame,
        player_side,
        row_count: if event.effect_id.ends_with("1ROW") {
            1
        } else {
            2
        },
    })
}

fn visible_battle_uses_implicit_rows(animation: &VisibleMoveAnimation) -> bool {
    visible_battle_row_plan(animation).is_some()
}

fn advance_visible_battle_objects(
    playback: &mut VisibleBattleObjects,
    bundle: &serde_json::Value,
    animation: &VisibleMoveAnimation,
) -> Result<()> {
    // Extraction may repeat a tick or seek backwards. Rebuild on a new
    // timeline or rewind so BG-owned allocations cannot accumulate.
    if u32::from(animation.frame) + 1 < playback.next_tick
        || playback.source != animation.object_events
        || playback.bg_source != animation.bg_events
        || playback.move_id != animation.move_id
        || playback.player != animation.player_move
        || playback.label != animation.animation_label
    {
        *playback = new_visible_battle_objects(bundle, animation)?;
    }
    let row_plan = visible_battle_source_row_plan(animation);
    let VisibleBattleObjects {
        machine,
        owners,
        last_id,
        obp0_write,
        next_event,
        next_tick,
        slots: output,
        battler_rows,
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
                            object["palette"].as_str().context("missing object palette")?,
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
                        owners[slot] = Some(VisibleBattleObjectOwner::Event(event_index));
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
        // RunBattleAnimScript executes script commands, then BG callbacks,
        // then DoBattleAnimFrame/OAM. BATTLEROBJ age zero queues an ordinary
        // NULL object at absolute screen coordinates. It consumes the same
        // slot and object ID as an explicit spawn, before later anim_incobj.
        // The shared plan validates one allocation; the source VM owns its
        // real slot, creation ID, callback retirement and final-tick OAM.
        if let Some(VisibleBattleRowPlan {
            bg_event_index,
            spawn_frame,
            player_side,
            row_count,
        }) = row_plan.filter(|plan| u32::from(plan.spawn_frame) == tick)
        {
            debug_assert_eq!(animation.bg_events[bg_event_index].frame, spawn_frame);
            let object_id = match (player_side, row_count) {
                (true, 1) => "BATTLE_ANIM_OBJ_PLAYERHEAD_1ROW",
                (true, _) => "BATTLE_ANIM_OBJ_PLAYERHEAD_2ROW",
                (false, 1) => "BATTLE_ANIM_OBJ_ENEMYFEET_1ROW",
                (false, _) => "BATTLE_ANIM_OBJ_ENEMYFEET_2ROW",
            };
            if let Some(slot) = (0..10).find(|&slot| machine.object(slot)[0] == 0) {
                let object = &bundle["objects"][object_id];
                let function = battle_anim_object_function(object_id, object)?;
                anyhow::ensure!(
                    function == "BATTLE_ANIM_FUNC_NULL",
                    "battler row must use NULL"
                );
                let frameset = battle_program::FRAMESETS
                    .iter()
                    .position(|name| Some(*name) == object["frameset"].as_str())
                    .context("missing battler row frameset")? as u8;
                let palette = battle_object_palette_id(
                    object["palette"]
                        .as_str()
                        .context("missing battler row palette")?,
                )?;
                *last_id = last_id.wrapping_add(1);
                machine.initialize(
                    slot,
                    *last_id,
                    [
                        battle_object_byte(object, "flags")?,
                        battle_object_byte(object, "fix_y")?,
                        frameset,
                        0,
                        palette,
                        0,
                    ],
                    if player_side { 48 } else { 132 },
                    64,
                    0,
                );
                owners[slot] = Some(VisibleBattleObjectOwner::BattlerRow {
                    bg_event_index,
                    player_side,
                    row_count,
                });
            }
        }
        machine.obp0_write = None;
        machine.begin_oam();
        *output = std::array::from_fn(|_| None);
        *battler_rows = std::array::from_fn(|_| None);
        let mut scanlines = [0_u8; 144];
        for slot in 0..10 {
            if machine.object(slot)[0] == 0 {
                continue;
            }
            let function_id = machine.object(slot)[4];
            let function = battle_program::FUNCTIONS
                .get(usize::from(function_id))
                .copied()
                .unwrap_or("unknown");
            let state = machine.object(slot)[14];
            machine.step_object(slot).map_err(|error| {
                anyhow::anyhow!(
                    "battle object callback slot{slot} function={function}({function_id}) state={state} source_tick={tick}: {error}"
                )
            })?;
            let before = usize::from(machine.read(battle_program::W_BATTLE_ANIM_O_A_M_POINTER_LO));
            let state = machine.object(slot)[14];
            let full = machine.oam_update(slot).map_err(|error| {
                anyhow::anyhow!(
                    "battle object OAM slot{slot} function={function}({function_id}) state={state} source_tick={tick}: {error}"
                )
            })?;
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
            let owner = owners[slot].context("active battle object has no owner")?;
            // A function may deinitialize while still emitting this tick's OAM.
            if bytes[0] != 0 || !entries.is_empty() {
                let frameset = *battle_program::FRAMESETS
                    .get(usize::from(bytes[3]))
                    .context("invalid live frameset ID")?;
                let frame = usize::from(bytes[13]);
                let oam = VisibleBattleObjectOam {
                    entries,
                    rows,
                    origin: (i32::from(x), i32::from(y)),
                };
                match owner {
                    VisibleBattleObjectOwner::Event(event_index) => {
                        output[slot] = Some(VisibleBattleObjectFrame {
                            event_index,
                            spawn_frame: animation.object_events[event_index].frame,
                            bytes,
                            frameset,
                            frame,
                            oam,
                        });
                    }
                    VisibleBattleObjectOwner::BattlerRow {
                        bg_event_index,
                        player_side,
                        row_count,
                    } => {
                        battler_rows[slot] = Some(VisibleBattleBattlerRowFrame {
                            bg_event_index,
                            spawn_frame: animation.bg_events[bg_event_index].frame,
                            player_side,
                            row_count,
                            bytes,
                            frameset,
                            frame,
                            oam,
                        });
                    }
                }
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

/// Shared source descriptor for classic OAM and the immersive adapter. Its
/// clock is copied from VisibleCaptureAnimation, never advanced by a renderer.
fn visible_capture_source_animation(capture: &VisibleCaptureAnimation) -> VisibleMoveAnimation {
    VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: format!("THROW_{}", capture.ball_id),
        animation_label: "BattleAnim_ThrowPokeBall".into(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: capture.object_frame(),
        total_frames: capture.total_frames(),
        sound_events: Vec::new(), next_sound_event: 0,
        cry_events: Vec::new(), next_cry_event: 0,
        object_events: capture.object_events(), bg_events: Vec::new(),
        actor_species_override: None, actor_shiny_override: None,
    }
}
