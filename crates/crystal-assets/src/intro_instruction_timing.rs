use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;

/// The pinned farcall macro, rst vector and bank-preserving wrapper. The
/// callee's instructions (including its RET) run between these two sequences.
pub fn runtime_intro_farcall_instruction_machine_cycles() -> (Vec<u64>, Vec<u64>) {
    (
        vec![2, 3, 4, 4, 3, 3, 4, 3, 4, 3, 4, 4, 6, 1],
        vec![1, 4, 1, 4, 3, 1, 4, 3, 4, 4, 4, 1, 4, 1, 4],
    )
}

/// Ordered instruction costs for the source byte-copy helpers. Interrupts and
/// the caller's CALL are separate instructions owned by the calling program.
pub fn runtime_intro_byte_loop_instruction_machine_cycles(
    routine: &str,
    byte_count: u16,
) -> Result<Vec<u64>> {
    let body: &[u64] = match routine {
        "ByteFill" => &[2],
        "CopyBytes" => &[2, 2, 2],
        _ => bail!("unsupported intro byte-loop routine {routine}"),
    };
    let mut tape = vec![1, 1, 3];
    let mut b = ((byte_count >> 8) as u8).wrapping_add(1);
    let mut c = (byte_count as u8).wrapping_add(1);
    loop {
        c = c.wrapping_sub(1);
        tape.push(1);
        if c != 0 {
            tape.push(3);
        } else {
            tape.extend([2, 1]);
            b = b.wrapping_sub(1);
            if b == 0 {
                tape.extend([2, 4]);
                return Ok(tape);
            }
            tape.push(3);
        }
        tape.extend_from_slice(body);
    }
}

/// Ordered source register-counter clear loops, including their final RET.
pub fn runtime_intro_clear_loop_instruction_machine_cycles(
    routine: &str,
    byte_count: u16,
) -> Result<Vec<u64>> {
    ensure!(byte_count != 0, "intro clear loop has a zero counter");
    let (prefix, body): (&[u64], &[u64]) = match routine {
        "ClearSpriteAnims" => (&[3, 3], &[3, 2, 2, 1, 1]),
        "ClearSprites" => {
            ensure!(
                byte_count <= 256,
                "intro sprite clear exceeds its byte counter"
            );
            (&[3, 2, 1], &[2, 1])
        }
        _ => bail!("unsupported intro clear-loop routine {routine}"),
    };
    let mut tape = prefix.to_vec();
    for index in 0..byte_count {
        tape.extend_from_slice(body);
        tape.push(if index + 1 == byte_count { 2 } else { 3 });
    }
    tape.push(4);
    Ok(tape)
}

/// Work through the memory-fill effect of a split helper. Remaining authored
/// palette requests and blocking waits are separate semantic operations.
fn split_helper_memory_instruction_machine_cycles(
    routine: &str,
    byte_count: u16,
) -> Result<Vec<u64>> {
    let (prefix, suffix): (&[u64], &[u64]) = match routine {
        "Intro_ClearBGPals" | "Intro_ResetLYOverrides" => (&[3, 4, 2, 3, 3, 3, 1, 6], &[3, 3]),
        "ClearBGPalettes" => (&[6, 3, 1, 3, 3, 4, 2, 3, 3, 3, 2, 6], &[3, 3]),
        "ClearTilemap" => (&[3, 2, 3, 6], &[3, 2, 2, 4]),
        _ => bail!("unsupported split intro helper {routine}"),
    };
    match routine {
        "Intro_ClearBGPals" | "ClearBGPalettes" => ensure!(
            byte_count == 128,
            "intro palette clear differs from 16 source palettes"
        ),
        "Intro_ResetLYOverrides" => ensure!(
            byte_count == 144,
            "intro LY clear differs from source scanline count"
        ),
        "ClearTilemap" => ensure!(
            byte_count == 360,
            "intro tilemap clear differs from source screen area"
        ),
        _ => unreachable!(),
    }
    let mut tape = prefix.to_vec();
    tape.extend(runtime_intro_byte_loop_instruction_machine_cycles(
        "ByteFill", byte_count,
    )?);
    tape.extend_from_slice(suffix);
    Ok(tape)
}

/// Source Decompress instruction boundaries selected by the actual LZ3 stream.
/// Payload bytes affect output, but only command, length and offset sign affect
/// CPU work. No recorded elapsed totals participate in prediction.
pub fn runtime_intro_decompress_instruction_machine_cycles(compressed: &[u8]) -> Result<Vec<u64>> {
    fn counter(tape: &mut Vec<u64>, b: &mut u8, c: &mut u8, relative_exit: bool) -> bool {
        *c = c.wrapping_sub(1);
        tape.push(1);
        if *c != 0 {
            tape.push(3);
            return true;
        }
        tape.extend([2, 1]);
        *b = b.wrapping_sub(1);
        tape.push(if relative_exit {
            if *b == 0 { 3 } else { 2 }
        } else if *b == 0 {
            4
        } else {
            3
        });
        *b != 0
    }
    let mut tape = vec![1, 4, 1, 4];
    let mut cursor = 0usize;
    loop {
        let control = *compressed
            .get(cursor)
            .context("intro LZ3 timing has no terminator")?;
        cursor += 1;
        if control == 255 {
            tape.extend([2, 2, 5]);
            return Ok(tape);
        }
        tape.extend([2, 2, 2, 2, 2]);
        let (command, length) = if control >> 5 == 7 {
            tape.extend([2, 2, 1, 1, 1, 2, 4, 2, 2, 1, 2, 1, 2, 3]);
            let low = *compressed
                .get(cursor)
                .context("intro LZ3 long length is truncated")?;
            cursor += 1;
            (
                (control >> 2) & 7,
                (((control as usize & 3) << 8) | low as usize) + 1,
            )
        } else {
            tape.extend([3, 4, 2, 2, 1, 2, 1]);
            (control >> 5, (control as usize & 31) + 1)
        };
        ensure!(
            command <= 6,
            "intro LZ3 timing has an invalid nested long command"
        );
        tape.extend([1, 1, 3, 2]);
        let mut b = ((length >> 8) as u8).wrapping_add(1);
        let mut c = (length as u8).wrapping_add(1);
        if command < 4 {
            tape.push(2);
            for target in 1..=3 {
                tape.extend([2, if command == target { 3 } else { 2 }]);
                if command == target {
                    break;
                }
            }
            match command {
                0 => {
                    while counter(&mut tape, &mut b, &mut c, false) {
                        tape.extend([2, 2, 2, 3]);
                    }
                    cursor = cursor
                        .checked_add(length)
                        .context("intro LZ3 literal overflow")?;
                }
                1 | 3 => {
                    tape.push(if command == 1 { 2 } else { 1 });
                    while counter(&mut tape, &mut b, &mut c, false) {
                        tape.extend([2, 2, 3]);
                    }
                    if command == 1 {
                        cursor += 1;
                    }
                }
                2 => {
                    let mut first = true;
                    loop {
                        if !counter(&mut tape, &mut b, &mut c, false) {
                            if first {
                                tape.push(2);
                            }
                            tape.extend([2, 3]);
                            break;
                        }
                        tape.extend([2, 2, 2]);
                        if !first {
                            tape.push(3);
                        }
                        first = !first;
                    }
                    cursor += 2;
                }
                _ => unreachable!(),
            }
        } else {
            tape.extend([3, 4, 4, 2, 2]);
            let sign = *compressed
                .get(cursor)
                .context("intro LZ3 rewrite offset is truncated")?
                & 128
                != 0;
            if sign {
                tape.extend([2, 2, 1, 1, 1, 2, 1, 1, 3]);
                cursor += 1;
            } else {
                tape.extend([3, 2, 1, 4, 1, 1, 4, 1, 1]);
                cursor += 2;
            }
            tape.push(3);
            for target in 4..=6 {
                tape.extend([2, if command == target { 3 } else { 2 }]);
                if command == target {
                    break;
                }
            }
            while counter(&mut tape, &mut b, &mut c, command == 4) {
                if command == 5 {
                    tape.extend([2, 4, 3]);
                    for bit in 0..8 {
                        tape.extend([1, 2, 1, if bit == 7 { 2 } else { 3 }]);
                    }
                    tape.extend([1, 3, 2, 2, 3]);
                } else {
                    tape.extend([2, 2, 2, 3]);
                }
            }
            tape.extend([3, 3, if sign { 3 } else { 2 }]);
            if !sign {
                tape.push(2);
            }
            tape.extend([2, 4]);
        }
        ensure!(
            cursor < compressed.len(),
            "intro LZ3 command payload is truncated"
        );
    }
}

/// Source Request2bpp's normal-link path, split around each blocking DelayFrame.
/// Each tuple is (requested tiles, instructions before wait, instructions after
/// wait). DelayFrame's CALL is the final before-wait instruction; its body is
/// driven by the shared interrupt clock. Its RET precedes the after-wait tape.
/// Exact multiples retain the source's final zero-size DelayFrame request.
pub fn runtime_intro_request2bpp_instruction_segments(
    tile_count: u8,
    tiles_per_cycle: u8,
) -> Result<Vec<(u8, Vec<u64>, Vec<u64>)>> {
    ensure!(
        tiles_per_cycle != 0,
        "intro Request2bpp has zero transfer rate"
    );
    let mut prefix = vec![
        3, 4, 1, 3, 3, 4, 1, 4, 3, 4, 4, 3, 4, 2, 3, 4, 2, 3, 1, 4, 1, 4, 1, 4, 1, 4,
    ];
    let mut remaining = tile_count;
    let mut segments = Vec::new();
    loop {
        let full = remaining >= tiles_per_cycle;
        prefix.extend([1, 3, 2, if full { 3 } else { 2 }]);
        let requested = if full {
            prefix.extend([3, 4]);
            tiles_per_cycle
        } else {
            prefix.push(4);
            remaining
        };
        prefix.push(6);
        let mut after = vec![4, 1, 2];
        if full {
            after.extend([1, 3, 2, 1, 3]);
            remaining -= tiles_per_cycle;
        } else {
            after.extend([3, 3, 3, 4, 3, 4, 4, 3, 3, 4]);
        }
        segments.push((requested, std::mem::take(&mut prefix), after));
        if !full {
            return Ok(segments);
        }
    }
}

/// Colored Suicune's source tile loop branches on each actual tile byte.
/// Passing a frame runs the conditional scene16 wrapper; None enters the swap
/// helper directly. The caller's CALL and interrupt work remain separate.
pub fn runtime_intro_tilemap_xor_instruction_machine_cycles(
    tilemap: &[u8],
    frame: Option<u8>,
) -> Result<Vec<u64>> {
    ensure!(
        tilemap.len() == 360,
        "intro tilemap timing differs from source screen area"
    );
    let mut tape = Vec::new();
    if let Some(frame) = frame {
        let phase = frame & 3;
        tape.extend([4, 2, if phase == 0 { 3 } else { 2 }]);
        if phase != 0 {
            tape.extend([2, if phase == 3 { 3 } else { 2 }]);
            if phase == 3 {
                tape.extend([1, 3, 4]);
            } else {
                tape.push(4);
            }
            return Ok(tape);
        }
    }
    tape.extend([3, 3]);
    for (index, &tile) in tilemap.iter().enumerate() {
        tape.extend([2, 1, if tile == 0 { 3 } else { 2 }]);
        if tile != 0 {
            tape.extend([2, if tile >= 128 { 3 } else { 2 }]);
            if tile < 128 {
                tape.extend([2, 2]);
            }
        }
        tape.extend([2, 2, 1, 1, if index + 1 == tilemap.len() { 2 } else { 3 }]);
    }
    tape.extend([2, 3, 4]);
    Ok(tape)
}

/// The calc_sine_wave source macro multiplies by shifting the actual amplitude
/// until it becomes zero. Angle sign and amplitude bits select instruction
/// paths independently of the lookup table's numeric sine values.
pub fn runtime_intro_sprite_sine_instruction_machine_cycles(
    cosine: bool,
    angle: u8,
    amplitude: u8,
    animseq_wrapper: bool,
) -> Vec<u64> {
    let mut tape = Vec::new();
    if animseq_wrapper {
        tape.push(6);
    }
    let angle = if cosine {
        tape.push(2);
        angle.wrapping_add(16)
    } else {
        angle
    };
    let negative = angle & 63 >= 32;
    tape.extend([2, 2, if negative { 3 } else { 2 }]);
    if negative {
        tape.push(2);
    }
    tape.push(6);
    tape.extend([1, 1, 2, 3, 2, 2, 2, 2, 2, 3]);
    let mut remaining = amplitude;
    loop {
        let odd = remaining & 1 != 0;
        remaining >>= 1;
        tape.extend([2, if odd { 2 } else { 3 }]);
        if odd {
            tape.push(2);
        }
        tape.extend([2, 2, 1, if remaining == 0 { 2 } else { 3 }]);
        if remaining == 0 {
            break;
        }
    }
    tape.extend([4, 1]);
    if negative {
        tape.extend([2, 1]);
    }
    tape.push(4);
    if animseq_wrapper {
        tape.push(4);
    }
    tape
}

/// InitSpriteAnimStruct's bank-preserving home wrapper and engine body.
/// Intro object definitions all use dictionary key0; that key occupies the
/// first dictionary entry after the authored clears/away-scene write.
pub fn runtime_intro_sprite_init_instruction_machine_cycles(
    occupied_prefix: u8,
    count_before: u8,
) -> Result<Vec<u64>> {
    ensure!(
        occupied_prefix <= 10,
        "intro sprite allocator exceeds its source capacity"
    );
    let mut tape = vec![4, 3, 4, 2, 4, 3, 4, 4, 4, 6, 4, 4, 3, 2];
    for slot in 0..occupied_prefix {
        tape.extend([2, 1, 2, 3, 2, 1, if slot == 9 { 2 } else { 3 }]);
    }
    if occupied_prefix == 10 {
        tape.extend([3, 3, 1, 4]);
    } else {
        tape.extend([
            2,
            1,
            3,
            1,
            1,
            3,
            3,
            2,
            1,
            if count_before == 255 { 2 } else { 3 },
        ]);
        if count_before == 255 {
            tape.push(3);
        }
        tape.extend([
            3, 1, 2, 3, 2, 2, 2, 1, 1, 3, 2, 4, 2, 2, 2, 2, 2, 2, 2, 2, 6,
        ]);
        // GetSpriteAnimVTile's first-entry dictionary match, including its RET.
        tape.extend([4, 4, 3, 1, 2, 2, 1, 3, 2, 3, 3, 4]);
        tape.extend([
            2, 3, 3, 2, 1, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 2, 2, 2, 1, 4, 1, 4, 4,
        ]);
    }
    tape.extend([3, 4, 3, 4, 4, 4]);
    Ok(tape)
}

/// The home wrapper used by source-authored Unown group frameset resets.
pub fn runtime_intro_sprite_reinit_instruction_machine_cycles() -> Vec<u64> {
    let mut tape = vec![4, 3, 4, 2, 4, 3, 4, 4, 4, 6];
    tape.extend([3, 2, 2, 3, 2, 3, 3, 2, 3, 4]);
    tape.extend([3, 4, 3, 4, 4, 4]);
    tape
}

/// CrystalIntro_InitUnownAnim's four initializers, with their authored radius
/// writes and home frameset resets. Caller coordinate setup/CALL is separate.
/// The actual first-free slot before each allocation is supplied by the shared
/// allocator; it cannot be inferred from a renderer's cursor or frame count.
pub fn runtime_intro_sprite_group_instruction_machine_cycles(
    occupied_prefixes: [u8; 4],
    count_before: u8,
) -> Result<Vec<u64>> {
    let mut tape = Vec::new();
    let mut count = count_before;
    for (index, occupied) in occupied_prefixes.into_iter().enumerate() {
        ensure!(
            occupied < 10,
            "intro Unown group has no free source sprite slot"
        );
        if index != 3 {
            tape.push(4);
        }
        tape.extend([2, 6]);
        tape.extend(runtime_intro_sprite_init_instruction_machine_cycles(
            occupied, count,
        )?);
        count = count.wrapping_add(1);
        if count == 0 {
            count = 1;
        }
        tape.extend([3, 2, 3, 2, 6]);
        tape.extend(runtime_intro_sprite_reinit_instruction_machine_cycles());
        tape.push(if index == 3 { 4 } else { 3 });
    }
    Ok(tape)
}

/// UpdateAnimFrame's instruction paths selected by source animation state.
/// Command bytes use the source oam control values: restart254/end255 may
/// precede a terminal frame(<252), wait253 or delete252. Caller CALL excluded.
pub fn runtime_intro_sprite_update_instruction_machine_cycles(
    duration_before: u8,
    frame_commands: &[u8],
    frame_flags: u8,
    object_count: u8,
    oam_addr_before: u8,
) -> Result<Vec<u64>> {
    ensure!(
        !frame_commands.is_empty() && frame_commands.len() <= 64,
        "intro sprite frame control path is empty or unbounded"
    );
    let pointer = [3, 2, 2, 2, 3, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 4];
    // CALL InitSpriteAnimBuffer, full body, CALL GetSpriteAnimFrame.
    let mut tape = vec![6, 1, 4, 3, 2];
    for _ in 0..5 {
        tape.extend([2, 4]);
    }
    tape.extend([4, 6]);
    let mut terminal = None;
    for (index, &command) in frame_commands.iter().enumerate() {
        let holding = index == 0 && duration_before != 0;
        tape.extend([3, 2, 2, 1, if holding { 2 } else { 3 }]);
        if holding {
            ensure!(
                frame_commands.len() == 1 && command < 254,
                "holding sprite has a source control command"
            );
            tape.extend([3, 6]);
            tape.extend(pointer);
            tape.extend([2, 4, 3]);
        } else {
            tape.extend([3, 2, 3, 6]);
            tape.extend(pointer);
            tape.extend([2, 2, if command == 254 { 3 } else { 2 }]);
            if command == 254 {
                tape.extend([1, 3, 2, 2, 1, 3, 2, 2, 3]);
                continue;
            }
            tape.extend([2, if command == 255 { 3 } else { 2 }]);
            if command == 255 {
                tape.extend([1, 3, 2, 2, 3, 2, 3, 3, 3]);
                continue;
            }
            tape.extend([4, 2, 4, 2, 3, 2, 2, 3, 2, 2, 3]);
        }
        tape.extend([2, 2, 2, 4, 3, 4]);
        ensure!(
            index + 1 == frame_commands.len(),
            "intro sprite path continues after its terminal frame"
        );
        terminal = Some(command);
        break;
    }
    let command = terminal.context("intro sprite source control path has no terminal frame")?;
    tape.extend([2, if command == 253 { 3 } else { 2 }]);
    if command == 253 {
        tape.extend([1, 4]);
        return Ok(tape);
    }
    tape.extend([2, if command == 252 { 3 } else { 2 }]);
    if command == 252 {
        tape.extend([6, 3, 2, 3, 4, 1, 4]);
        return Ok(tape);
    }
    tape.extend([6, 1, 2, 3, 2, 2, 2, 4]); // GetFrameOAMPointer including its RET.
    tape.extend([4, 2, 4, 2, 2, 2, 1, 4, 4, 1, 2, 2, 1]);
    let mut remaining = object_count;
    let mut address = oam_addr_before;
    for _ in 0..64 {
        for flip in [64u8, 32] {
            tape.extend([4, 1, 4, 1, 1, 4, 1, 1, 6]);
            let flipped = frame_flags & flip != 0;
            tape.extend([4, 2, 3, 3, if flipped { 2 } else { 3 }]);
            if flipped {
                tape.extend([2, 2, 1]);
            }
            tape.extend([3, 4, 1, 2, 2, 2]);
        }
        tape.extend([4, 2, 2, 2, 2]);
        tape.extend([6, 4, 1, 2, 1, 2, 1, 2, 2, 1, 4]);
        address = address.wrapping_add(4);
        tape.extend([2, 2, 2, 1, 4, 2, if address >= 160 { 3 } else { 2 }]);
        if address >= 160 {
            tape.extend([3, 1, 4]);
            return Ok(tape);
        }
        remaining = remaining.wrapping_sub(1);
        tape.extend([1, if remaining == 0 { 2 } else { 3 }]);
        if remaining == 0 {
            tape.extend([3, 3, 1, 4]);
            return Ok(tape);
        }
    }
    anyhow::bail!("intro sprite OAM loop exceeds source capacity")
}

/// PlaySpriteAnimations's source slot scan, callback/update ordering and OAM
/// clear loop. None denotes an inactive authoritative slot; each active slot
/// carries its separately source-derived callback and UpdateAnimFrame tapes.
/// The outer farcall wrapper remains separate from this callee's instructions.
pub fn runtime_intro_sprite_scheduler_instruction_machine_cycles(
    slots: &[Option<(Vec<u64>, Vec<u64>)>],
    oam_limit_slot: Option<usize>,
    final_oam_addr: u8,
) -> Result<Vec<u64>> {
    ensure!(
        slots.len() == 10,
        "intro scheduler slot count differs from source capacity"
    );
    ensure!(
        oam_limit_slot.is_none_or(|slot| slot < 10 && slots[slot].is_some()),
        "intro scheduler OAM limit is not an active source slot"
    );
    let mut tape = vec![4, 4, 4, 4, 2, 4, 6, 3, 2];
    for (slot, active) in slots.iter().enumerate() {
        tape.extend([2, 1, if active.is_some() { 2 } else { 3 }]);
        if let Some((callback, update)) = active {
            ensure!(
                callback
                    .iter()
                    .chain(update)
                    .all(|cost| (1..=6).contains(cost)),
                "intro sprite nested tape has an invalid instruction"
            );
            tape.extend([1, 1, 4, 4, 6, 3, 2, 2, 2, 3, 2, 2, 2, 2, 1, 1]);
            tape.extend_from_slice(callback);
            tape.push(6);
            tape.extend_from_slice(update);
            tape.extend([3, 3, if oam_limit_slot == Some(slot) { 3 } else { 2 }]);
            if oam_limit_slot == Some(slot) {
                tape.extend([4, 3, 3, 3, 3, 4]);
                return Ok(tape);
            }
        }
        tape.extend([3, 2, 1, if slot == 9 { 2 } else { 3 }]);
    }
    tape.extend([4, 1, 2]);
    let mut address = final_oam_addr;
    loop {
        tape.extend([1, 2, if address >= 160 { 3 } else { 2 }]);
        if address >= 160 {
            break;
        }
        tape.extend([1, 2, 3]);
        address = address.wrapping_add(1);
    }
    tape.extend([4, 3, 3, 3, 3, 4]);
    Ok(tape)
}

/// Scene12's source sound-table search, split around the nested PlaySFX call.
/// The prefix includes CALL PlaySFX but not the outer CALL .PlayUnownSound.
/// The no-match prefix returns directly after the table sentinel.
pub fn runtime_intro_scheduled_audio_instruction_machine_cycles(
    matched_entry: Option<usize>,
    entry_count: usize,
) -> Result<(Vec<u64>, Vec<u64>)> {
    ensure!(
        entry_count == 8 && matched_entry.is_none_or(|entry| entry < entry_count),
        "intro Unown sound table differs from source"
    );
    let mut prefix = vec![4, 1, 3];
    for _ in 0..matched_entry.unwrap_or(entry_count) {
        prefix.extend([2, 2, 2, 1, 2, 2, 2, 3]);
    }
    if matched_entry.is_none() {
        prefix.extend([2, 2, 5]);
        return Ok((prefix, Vec::new()));
    }
    prefix.extend([2, 2, 2, 1, 3, 2, 2, 1, 4, 6]);
    prefix.extend([1, 4, 4, 4, 4, 4, 4]); // SFXChannelsOff including its RET.
    prefix.extend([3, 6]);
    Ok((prefix, vec![4]))
}

/// Expand a source-certified compact helper descriptor without accepting
/// precomputed frame positions or double-counted call/interrupt work.
pub fn runtime_intro_helper_instruction_machine_cycles(descriptor: &Value) -> Result<Vec<u64>> {
    let routine = descriptor["routine"]
        .as_str()
        .context("intro helper routine is missing")?;
    let byte_count = descriptor["byte_count"]
        .as_u64()
        .and_then(|count| u16::try_from(count).ok())
        .context("intro helper byte count exceeds the source counter")?;
    ensure!(
        descriptor["caller_call_included"].as_bool() == Some(false)
            && descriptor["interrupts_included"].as_bool() == Some(false),
        "intro helper includes caller or interrupt work"
    );
    let (source_file, model, tape) = match routine {
        "ByteFill" | "CopyBytes" => (
            "home/copy.asm",
            "byte_count_and_counter_rollover_v1",
            runtime_intro_byte_loop_instruction_machine_cycles(routine, byte_count)?,
        ),
        "ClearSpriteAnims" => (
            "engine/sprite_anims/core.asm",
            "clear_register_counter_loop_v1",
            runtime_intro_clear_loop_instruction_machine_cycles(routine, byte_count)?,
        ),
        "ClearSprites" => (
            "home/clear_sprites.asm",
            "clear_register_counter_loop_v1",
            runtime_intro_clear_loop_instruction_machine_cycles(routine, byte_count)?,
        ),
        "Intro_ClearBGPals" | "Intro_ResetLYOverrides" => (
            "engine/movie/intro.asm",
            "split_memory_effect_v1",
            split_helper_memory_instruction_machine_cycles(routine, byte_count)?,
        ),
        "ClearBGPalettes" => (
            "home/tilemap.asm",
            "split_memory_effect_v1",
            split_helper_memory_instruction_machine_cycles(routine, byte_count)?,
        ),
        "ClearTilemap" => (
            "home/text.asm",
            "split_memory_effect_v1",
            split_helper_memory_instruction_machine_cycles(routine, byte_count)?,
        ),
        _ => bail!("unsupported source intro helper {routine}"),
    };
    ensure!(
        descriptor["source_file"].as_str() == Some(source_file)
            && descriptor["model"].as_str() == Some(model),
        "intro helper source contract differs from its instruction model"
    );
    let actual: u64 = tape.iter().sum();
    ensure!(
        descriptor["machine_cycles"].as_u64() == Some(actual),
        "intro helper declared work disagrees with its instruction sequence"
    );
    Ok(tape)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lz3_timing_rejects_truncated_streams_and_unknown_commands() {
        assert!(runtime_intro_decompress_instruction_machine_cycles(&[]).is_err());
        assert!(runtime_intro_decompress_instruction_machine_cycles(&[0]).is_err());
        assert!(runtime_intro_decompress_instruction_machine_cycles(&[0xe0]).is_err());
        assert!(runtime_intro_decompress_instruction_machine_cycles(&[0xfc, 0, 0, 255]).is_err());
        assert_eq!(
            runtime_intro_decompress_instruction_machine_cycles(&[255]).unwrap(),
            vec![1, 4, 1, 4, 2, 2, 5]
        );
        let requests = runtime_intro_request2bpp_instruction_segments(16, 8).unwrap();
        assert_eq!(
            requests.iter().map(|entry| entry.0).collect::<Vec<_>>(),
            vec![8, 8, 0]
        );
        assert!(runtime_intro_request2bpp_instruction_segments(16, 0).is_err());
    }

    #[test]
    fn helper_contract_rejects_unknown_or_double_counted_work() {
        let descriptor = serde_json::json!({"routine":"CopyBytes","byte_count":256,
            "machine_cycles":2578,"caller_call_included":false,"interrupts_included":false,
            "source_file":"home/copy.asm","model":"byte_count_and_counter_rollover_v1"});
        assert_eq!(
            runtime_intro_helper_instruction_machine_cycles(&descriptor)
                .unwrap()
                .iter()
                .sum::<u64>(),
            2578
        );
        for (field, invalid) in [
            ("routine", serde_json::json!("Unknown")),
            ("byte_count", serde_json::json!(65536)),
            ("machine_cycles", serde_json::json!(2579)),
            ("caller_call_included", serde_json::json!(true)),
            ("interrupts_included", serde_json::json!(true)),
            ("source_file", serde_json::json!("other.asm")),
            ("model", serde_json::json!("unknown")),
        ] {
            let mut malformed = descriptor.clone();
            malformed[field] = invalid;
            assert!(
                runtime_intro_helper_instruction_machine_cycles(&malformed).is_err(),
                "{field}"
            );
        }
        assert!(
            runtime_intro_clear_loop_instruction_machine_cycles("ClearSpriteAnims", 0).is_err()
        );
        assert!(runtime_intro_clear_loop_instruction_machine_cycles("ClearSprites", 257).is_err());
        assert_eq!(
            runtime_intro_clear_loop_instruction_machine_cycles("ClearSpriteAnims", 193)
                .unwrap()
                .iter()
                .sum::<u64>(),
            2325
        );
        assert_eq!(
            runtime_intro_clear_loop_instruction_machine_cycles("ClearSprites", 160)
                .unwrap()
                .iter()
                .sum::<u64>(),
            969
        );
    }
}
