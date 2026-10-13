use super::*;

fn instruction_array(value: &Value, name: &str) -> Result<Vec<u64>> {
    value
        .as_array()
        .with_context(|| format!("intro {name} is not an instruction sequence"))?
        .iter()
        .map(|value| {
            value
                .as_u64()
                .filter(|cycles| (1..=6).contains(cycles))
                .with_context(|| format!("intro {name} contains an invalid SM83 instruction cost"))
        })
        .collect()
}

/// Expand source-certified work at the actual taken/untaken control path.
/// Missing timing is an error, never an aggregate guessed instruction.
pub fn runtime_intro_operation_instruction_machine_cycles(
    operation: &RuntimePresentationOperation,
    branch_taken: bool,
) -> Result<Vec<u64>> {
    runtime_intro_operation_instruction_machine_cycles_with_context(
        operation,
        branch_taken,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
        None,
    )
}

/// Resolve branch-sensitive source work from the state before the operation.
pub fn runtime_intro_operation_instruction_machine_cycles_with_context(
    operation: &RuntimePresentationOperation,
    branch_taken: bool,
    memory: &BTreeMap<String, u16>,
    values: &BTreeMap<String, u16>,
    regions: &BTreeMap<String, Vec<u8>>,
    compressed_resource: Option<&[u8]>,
) -> Result<Vec<u64>> {
    if let Some(descriptor) = operation.fields.get("source_sprite_init_timing") {
        anyhow::ensure!(
            descriptor["model"].as_str() == Some("first_free_slot_and_count_v1")
                && descriptor["capacity"].as_u64() == Some(10)
                && descriptor["dictionary_first_key"].as_u64() == Some(0)
                && descriptor["caller_call_included"].as_bool() == Some(false)
                && descriptor["interrupts_included"].as_bool() == Some(false),
            "unsupported intro sprite initializer timing model"
        );
        let slots = regions
            .get("wSpriteAnimationStructs")
            .context("intro sprite timing missing source slots")?;
        anyhow::ensure!(
            slots.len() == 160,
            "intro source sprite slots have invalid size"
        );
        let count = u8::try_from(
            *memory
                .get("wSpriteAnimCount")
                .context("intro sprite timing missing source count")?,
        )?;
        let mut occupied: Vec<bool> = slots.chunks_exact(16).map(|slot| slot[0] != 0).collect();
        let mut next_free = || {
            let free = occupied.iter().position(|used| !used).unwrap_or(10);
            if free < 10 {
                occupied[free] = true;
            }
            free as u8
        };
        let body = match descriptor["routine"].as_str() {
            Some("InitSpriteAnimStruct") => {
                runtime_intro_sprite_init_instruction_machine_cycles(next_free(), count)?
            }
            Some("CrystalIntro_InitUnownAnim") => {
                runtime_intro_sprite_group_instruction_machine_cycles(
                    [next_free(), next_free(), next_free(), next_free()],
                    count,
                )?
            }
            _ => anyhow::bail!("unsupported intro sprite initializer routine"),
        };
        let mut tape = instruction_array(
            &operation.fields["source_sprite_init_caller_instruction_machine_cycles"],
            "sprite initializer caller",
        )?;
        tape.extend(body);
        return Ok(tape);
    }
    if let Some(descriptor) = operation.fields.get("source_instruction_timing_variants") {
        let selector = &descriptor["selector"];
        let name = selector["source"]
            .as_str()
            .context("intro timing selector has no source")?;
        let state = match selector.get("scope").and_then(Value::as_str) {
            None | Some("memory") => memory,
            Some("values") => values,
            Some(scope) => anyhow::bail!("unsupported intro timing selector scope {scope}"),
        };
        let mask = selector
            .get("mask")
            .map(|v| v.as_u64().context("invalid intro selector mask"))
            .transpose()?
            .unwrap_or(255);
        let value = u64::from(
            *state
                .get(name)
                .with_context(|| format!("intro timing selector missing {name}"))?,
        ) & mask;
        let variants = descriptor["variants"]
            .as_array()
            .context("intro timing variants missing")?;
        let mut selected = None;
        for variant in variants {
            let matches = if let Some(equal) = variant.get("equals") {
                value == equal.as_u64().context("invalid intro timing equals")?
            } else {
                let operand = variant["operand"]
                    .as_u64()
                    .context("intro timing variant missing operand")?;
                match variant["predicate"]
                    .as_str()
                    .context("intro timing variant missing predicate")?
                {
                    "equal" => value == operand,
                    "not_equal" => value != operand,
                    "unsigned_less_than" => value < operand,
                    "unsigned_greater_than" => value > operand,
                    "unsigned_greater_or_equal" => value >= operand,
                    predicate => anyhow::bail!("unsupported intro timing predicate {predicate}"),
                }
            };
            if matches {
                anyhow::ensure!(selected.is_none(), "ambiguous intro source timing variants");
                selected = Some(instruction_array(
                    &variant["instruction_machine_cycles"],
                    "variant",
                )?);
            }
        }
        return selected.context("no intro source timing variant matches the current state");
    }
    if let Some(descriptor) = operation.fields.get("source_decompress_timing") {
        anyhow::ensure!(
            descriptor["model"].as_str() == Some("lz3_source_command_counter_v1")
                && descriptor["routine"].as_str() == Some("Decompress")
                && descriptor["caller_call_included"].as_bool() == Some(false)
                && descriptor["interrupts_included"].as_bool() == Some(false),
            "unsupported intro decompression timing model"
        );
        let compressed =
            compressed_resource.context("intro source timing requires its compressed resource")?;
        anyhow::ensure!(
            descriptor["compressed_byte_count"].as_u64() == Some(compressed.len() as u64),
            "intro compressed resource length mismatch"
        );
        let body = runtime_intro_decompress_instruction_machine_cycles(compressed)?;
        anyhow::ensure!(
            descriptor["machine_cycles"].as_u64() == Some(body.iter().sum()),
            "intro decompression source work mismatch"
        );
        let mut tape = instruction_array(
            &operation.fields["source_decompress_caller_instruction_machine_cycles"],
            "decompression caller",
        )?;
        tape.extend(body);
        return Ok(tape);
    }
    if let Some(descriptor) = operation.fields.get("source_tilemap_timing") {
        anyhow::ensure!(
            descriptor["model"].as_str() == Some("tile_byte_branch_counter_v1")
                && descriptor["target"].as_str() == Some("wTilemap")
                && descriptor["byte_count"].as_u64() == Some(360)
                && descriptor["caller_call_included"].as_bool() == Some(false)
                && descriptor["interrupts_included"].as_bool() == Some(false),
            "unsupported intro tilemap timing model"
        );
        let frame = match descriptor["routine"].as_str() {
            Some("Intro_ColoredSuicuneFrameSwap") => None,
            Some("Intro_Scene16_AnimateSuicune") => Some(u8::try_from(
                *memory
                    .get("wIntroSceneFrameCounter")
                    .context("intro tilemap timing missing frame counter")?,
            )?),
            _ => anyhow::bail!("unsupported intro tilemap timing routine"),
        };
        let tiles = regions
            .get("wTilemap")
            .context("intro tilemap timing missing source RAM bytes")?;
        let mut tape = instruction_array(
            &operation.fields["source_tilemap_caller_instruction_machine_cycles"],
            "tilemap caller",
        )?;
        tape.extend(runtime_intro_tilemap_xor_instruction_machine_cycles(
            tiles, frame,
        )?);
        return Ok(tape);
    }
    if let Some(helper) = operation.fields.get("source_helper_timing") {
        let caller = operation
            .fields
            .get("source_caller_instruction_machine_cycles")
            .context("intro helper has no caller instruction sequence")?;
        let mut tape = instruction_array(caller, "helper caller")?;
        let returns = match operation
            .fields
            .get("source_caller_model")
            .and_then(Value::as_str)
        {
            Some("farcall_hl_v1") => {
                let (prefix, suffix) = runtime_intro_farcall_instruction_machine_cycles();
                anyhow::ensure!(
                    tape == prefix,
                    "intro farcall prefix disagrees with its source model"
                );
                let returns = instruction_array(
                    operation
                        .fields
                        .get("source_return_instruction_machine_cycles")
                        .context("intro farcall has no return sequence")?,
                    "farcall return",
                )?;
                anyhow::ensure!(
                    returns == suffix,
                    "intro farcall return disagrees with its source model"
                );
                returns
            }
            None => {
                anyhow::ensure!(
                    tape.last() == Some(&6)
                        && !operation
                            .fields
                            .contains_key("source_return_instruction_machine_cycles"),
                    "intro helper caller does not end with its direct CALL"
                );
                Vec::new()
            }
            Some(model) => anyhow::bail!("unsupported intro caller timing model {model}"),
        };
        tape.extend(runtime_intro_helper_instruction_machine_cycles(helper)?);
        tape.extend(returns);
        return Ok(tape);
    }
    let field = if branch_taken
        && operation
            .fields
            .contains_key("source_instruction_machine_cycles_taken")
    {
        "source_instruction_machine_cycles_taken"
    } else {
        "source_instruction_machine_cycles"
    };
    instruction_array(
        operation
            .fields
            .get(field)
            .with_context(|| format!("intro {} has no source instruction timing", operation.op))?,
        field,
    )
}

/// Decode a local source LZ3 resource into bounded scratch RAM.
pub fn runtime_intro_decode_lz3(compressed: &[u8], limit: usize) -> Result<Vec<u8>> {
    fn byte(input: &[u8], cursor: &mut usize) -> Result<u8> {
        let value = *input.get(*cursor).context("truncated intro LZ3 resource")?;
        *cursor += 1;
        Ok(value)
    }
    let mut cursor = 0;
    let mut output = Vec::new();
    loop {
        let control = byte(compressed, &mut cursor)?;
        if control == 255 {
            return Ok(output);
        }
        let (command, count) = if control >> 5 == 7 {
            (
                (control >> 2) & 7,
                (((control as usize & 3) << 8) | byte(compressed, &mut cursor)? as usize) + 1,
            )
        } else {
            (control >> 5, (control as usize & 31) + 1)
        };
        anyhow::ensure!(command <= 6, "invalid intro LZ3 command");
        anyhow::ensure!(
            output
                .len()
                .checked_add(count)
                .is_some_and(|size| size <= limit),
            "intro LZ3 output exceeds scratch RAM"
        );
        match command {
            0 => {
                for _ in 0..count {
                    output.push(byte(compressed, &mut cursor)?);
                }
            }
            1 => {
                let value = byte(compressed, &mut cursor)?;
                output.extend(std::iter::repeat_n(value, count));
            }
            2 => {
                let first = byte(compressed, &mut cursor)?;
                let second = byte(compressed, &mut cursor)?;
                output.extend((0..count).map(|index| if index % 2 == 0 { first } else { second }));
            }
            3 => output.resize(output.len() + count, 0),
            4..=6 => {
                let high = byte(compressed, &mut cursor)?;
                let mut source = if high & 128 != 0 {
                    output
                        .len()
                        .checked_sub((high as usize & 127) + 1)
                        .context("intro LZ3 relative offset precedes output")?
                } else {
                    ((high as usize) << 8) | byte(compressed, &mut cursor)? as usize
                };
                for index in 0..count {
                    let value = *output
                        .get(source)
                        .context("intro LZ3 rewrite offset outside output")?;
                    output.push(if command == 5 {
                        value.reverse_bits()
                    } else {
                        value
                    });
                    if index + 1 != count {
                        source = if command == 6 {
                            source
                                .checked_sub(1)
                                .context("intro LZ3 reverse offset precedes output")?
                        } else {
                            source + 1
                        };
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the pinned external source export workspace"]
    fn lz3_decodes_declared_intro_resources_to_the_assembled_source_bytes() {
        let root = PathBuf::from(
            std::env::var_os("CRYSTAL_EXPORT_WORKSPACE").expect("external source workspace"),
        );
        let title: RuntimeTitleScreen = serde_json::from_slice(
            &std::fs::read(root.join("apps/web/assets/data/runtime_title_screen.unvalidated.json"))
                .unwrap(),
        )
        .unwrap();
        let mut compared = 0;
        for resource in title
            .program
            .resources
            .iter()
            .filter(|resource| resource.kind == "graphics" && resource.path.ends_with(".lz"))
        {
            let path = root.join("vendor/pokecrystal").join(&resource.path);
            let compressed = std::fs::read(&path).unwrap();
            let expected = std::fs::read(path.with_extension("")).unwrap();
            assert_eq!(
                runtime_intro_decode_lz3(&compressed, 64 * 1024).unwrap(),
                expected,
                "{}",
                resource.path
            );
            compared += 1;
        }
        assert!(
            compared >= 30,
            "source intro graphics must actually be compared"
        );
    }
    #[test]
    fn lz3_rewrite_overlap_flip_and_reverse_use_live_output() {
        assert_eq!(
            runtime_intro_decode_lz3(&[2, 1, 2, 128, 0x82, 0x82, 0xa0, 0x80, 0xc2, 0x82, 255], 32)
                .unwrap(),
            vec![1, 2, 128, 1, 2, 128, 1, 2, 1, 128]
        );
        assert!(runtime_intro_decode_lz3(&[0x80, 0x80, 255], 32).is_err());
        assert!(runtime_intro_decode_lz3(&[0x63, 255], 3).is_err());
    }
    #[test]
    fn context_selects_the_actual_unsigned_branch_and_rejects_missing_state() {
        let op: RuntimePresentationOperation = serde_json::from_value(serde_json::json!({"op":"copy_memory", "source_span":{"file":"engine/movie/intro.asm","start_line":1,"end_line":1}, "source_instruction_timing_variants": {
            "selector":{"source":"counter", "mask":255}, "variants":[
                {"predicate":"unsigned_less_than","operand":36,"instruction_machine_cycles":[2,2]},
                {"predicate":"unsigned_greater_or_equal","operand":36,"instruction_machine_cycles":[2,5]}]}})).unwrap();
        let mut memory = BTreeMap::new();
        let empty = BTreeMap::new();
        assert!(runtime_intro_operation_instruction_machine_cycles(&op, false).is_err());
        for (value, expected) in [(35, vec![2, 2]), (36, vec![2, 5]), (255, vec![2, 5])] {
            memory.insert("counter".to_string(), value);
            assert_eq!(
                runtime_intro_operation_instruction_machine_cycles_with_context(
                    &op,
                    false,
                    &memory,
                    &empty,
                    &BTreeMap::new(),
                    None
                )
                .unwrap(),
                expected
            );
        }
    }
}
