use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;

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
            ensure!(byte_count <= 256, "intro sprite clear exceeds its byte counter");
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

/// Expand a source-certified compact helper descriptor without accepting
/// precomputed frame positions or double-counted call/interrupt work.
pub fn runtime_intro_helper_instruction_machine_cycles(descriptor: &Value) -> Result<Vec<u64>> {
    let routine = descriptor["routine"].as_str().context("intro helper routine is missing")?;
    let byte_count = descriptor["byte_count"].as_u64()
        .and_then(|count| u16::try_from(count).ok())
        .context("intro helper byte count exceeds the source counter")?;
    ensure!(descriptor["caller_call_included"].as_bool() == Some(false)
        && descriptor["interrupts_included"].as_bool() == Some(false),
        "intro helper includes caller or interrupt work");
    let (source_file, model, tape) = match routine {
        "ByteFill" | "CopyBytes" => ("home/copy.asm", "byte_count_and_counter_rollover_v1",
            runtime_intro_byte_loop_instruction_machine_cycles(routine, byte_count)?),
        "ClearSpriteAnims" => ("engine/sprite_anims/core.asm", "clear_register_counter_loop_v1",
            runtime_intro_clear_loop_instruction_machine_cycles(routine, byte_count)?),
        "ClearSprites" => ("home/clear_sprites.asm", "clear_register_counter_loop_v1",
            runtime_intro_clear_loop_instruction_machine_cycles(routine, byte_count)?),
        _ => bail!("unsupported source intro helper {routine}"),
    };
    ensure!(descriptor["source_file"].as_str() == Some(source_file)
        && descriptor["model"].as_str() == Some(model),
        "intro helper source contract differs from its instruction model");
    let actual: u64 = tape.iter().sum();
    ensure!(descriptor["machine_cycles"].as_u64() == Some(actual),
        "intro helper declared work disagrees with its instruction sequence");
    Ok(tape)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_contract_rejects_unknown_or_double_counted_work() {
        let descriptor = serde_json::json!({"routine":"CopyBytes","byte_count":256,
            "machine_cycles":2578,"caller_call_included":false,"interrupts_included":false,
            "source_file":"home/copy.asm","model":"byte_count_and_counter_rollover_v1"});
        assert_eq!(runtime_intro_helper_instruction_machine_cycles(&descriptor).unwrap().iter().sum::<u64>(), 2578);
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
            assert!(runtime_intro_helper_instruction_machine_cycles(&malformed).is_err(), "{field}");
        }
        assert!(runtime_intro_clear_loop_instruction_machine_cycles("ClearSpriteAnims", 0).is_err());
        assert!(runtime_intro_clear_loop_instruction_machine_cycles("ClearSprites", 257).is_err());
        assert_eq!(runtime_intro_clear_loop_instruction_machine_cycles("ClearSpriteAnims", 193).unwrap().iter().sum::<u64>(), 2325);
        assert_eq!(runtime_intro_clear_loop_instruction_machine_cycles("ClearSprites", 160).unwrap().iter().sum::<u64>(), 969);
    }
}
