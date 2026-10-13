//! Crystal intro scene and transfer lowering. ROM timing is attached separately.
use super::{Cursor, Source};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
#[path = "source_intro_scene_dynamic.rs"]
mod dynamic;
#[path = "source_intro_background_bindings.rs"]
mod backgrounds;
#[path = "source_intro_scene_helpers.rs"]
mod helpers;

pub(super) fn scene_dispatch(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let setup = setup_phases(vendor, constants)?;
    let animated = dynamic::phases(vendor, constants)?;
    let bindings = backgrounds::lower(vendor, constants, &setup)?;
    let mut labels = serde_json::Map::new();
    let mut operations = Vec::new();
    let mut spans = Vec::new();
    for number in 1..=28 {
        let label = format!("IntroScene{number}");
        let phase = setup
            .get(&label)
            .or_else(|| animated.get(&label))
            .with_context(|| format!("missing intro phase {label}"))?;
        let offset = operations.len();
        labels.insert(label, json!(offset));
        if let Some(locals) = phase["labels"].as_object() {
            for (name, position) in locals {
                labels.insert(
                    name.clone(),
                    json!(offset + position.as_u64().context("intro label offset")? as usize),
                );
            }
        }
        spans.push(phase["source_span"].clone());
        operations.extend(
            phase["operations"]
                .as_array()
                .context("intro phase operations")?
                .iter()
                .cloned(),
        );

    }
    // Rendering metadata follows all returning executable scene bodies.
    operations.extend((1..=28).map(|number| bindings[&format!("IntroScene{number}")].clone()));
    Ok(
        json!({"phase":{"entry_label":"IntroScene1","labels":labels,"operations":operations},"transfer_metadata":transfer_metadata(vendor)?,"implementation_spans":spans}),
    )
}

fn emit(
    cursor: &mut Cursor<'_>,
    count: usize,
    op: &str,
    fields: Value,
    operations: &mut Vec<Value>,
) -> Result<()> {
    let sequence = cursor
        .body
        .get(cursor.index..cursor.index + count)
        .context("intro lowering exceeds source routine")?
        .iter()
        .map(|(_, line)| line.as_str())
        .collect::<Vec<_>>();
    operations.push(cursor.take(&sequence, op, fields)?);
    Ok(())
}

fn resource(source: &Source, symbol: &str) -> Result<String> {
    source
        .body(symbol)?
        .iter()
        .find_map(|(_, line)| {
            line.strip_prefix("INCBIN \"")
                .or_else(|| line.strip_prefix("INCLUDE \""))
                .and_then(|name| name.strip_suffix('"'))
                .map(str::to_owned)
        })
        .with_context(|| format!("intro resource binding {symbol}"))
}

fn symbols(vendor: &Path) -> Result<(BTreeMap<String, i64>, BTreeMap<String, u8>)> {
    let mut addresses = BTreeMap::new();
    let mut banks = BTreeMap::new();
    for line in std::fs::read_to_string(vendor.join("pokecrystal.sym"))?.lines() {
        let Some((address, name)) = line.split_once(' ') else {
            continue;
        };
        let Some((bank, address)) = address.split_once(':') else {
            continue;
        };
        addresses.insert(name.to_string(), i64::from_str_radix(address, 16)?);
        banks.insert(name.to_string(), u8::from_str_radix(bank, 16)?);
    }
    Ok((addresses, banks))
}

fn evaluate(
    expression: &str,
    constants: &BTreeMap<String, i64>,
    banks: &BTreeMap<String, u8>,
) -> Result<i64> {
    if let Some(name) = expression
        .strip_prefix("BANK(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return Ok(i64::from(
            *banks.get(name).context("intro source bank binding")?,
        ));
    }
    for (function, mask, shift) in [("LOW(", 255, 0), ("HIGH(", 255, 8)] {
        if let Some(inner) = expression
            .strip_prefix(function)
            .and_then(|s| s.strip_suffix(')'))
        {
            return Ok((evaluate(inner, constants, banks)? >> shift) & mask);
        }
    }
    let expression = if let Some((base, tile)) = expression.split_once(" tile ") {
        format!("{base} + ({tile}) * 16")
    } else {
        expression.replace(" palettes", " * 8")
    };
    super::super::source_expressions::evaluate(&expression, constants)
}

pub(super) fn transfer_metadata(vendor: &Path) -> Result<Value> {
    let source = Source::read(vendor, "engine/movie/intro.asm")?;
    let home = Source::read(vendor, "home/gfx.asm")?;
    let dispatcher = source
        .body("IntroScenes")?
        .iter()
        .filter_map(|(_, line)| line.strip_prefix("dw ").map(str::to_owned))
        .collect::<Vec<_>>();
    ensure!(dispatcher.len() == 28, "authored intro scene count changed");
    let mut transfers = Vec::new();
    for (entry, label) in dispatcher.iter().enumerate() {
        let body = source.body(label)?;
        let mut source_symbol = None;
        let mut destination = None;
        let mut bank = 0;
        let mut accumulator = None;
        for (line_number, line) in body {
            if let Some(symbol) = line.strip_prefix("ld hl, ") {
                source_symbol = Some(symbol.to_string());
            }
            if let Some(target) = line.strip_prefix("ld de, ") {
                destination = Some(target.to_string());
            }
            if line.starts_with("debgcoord ") {
                destination = Some("vBGMap0".into());
            }
            if let Some(value) = line.strip_prefix("ld a, $") {
                accumulator = Some(u8::from_str_radix(value, 16)?);
            }
            if line == "ldh [rVBK], a" {
                bank = accumulator.context("intro VRAM bank requires explicit source byte")?;
            }
            if let Some(wrapper) = line.strip_prefix("call Intro_DecompressRequest2bpp_") {
                let tiles: u16 = wrapper
                    .strip_suffix("Tiles")
                    .context("intro transfer wrapper")?
                    .parse()?;
                let source_symbol = source_symbol
                    .as_ref()
                    .context("intro decompression lacks authored source")?;
                let resource = resource(&source, source_symbol)?;
                let wrapper_label = format!("Intro_DecompressRequest2bpp_{wrapper}");
                source.operation(
                    &wrapper_label,
                    &[
                        "push de",
                        "ld de, wDecompressScratch",
                        "call Decompress",
                        "pop hl",
                        "ld de, wDecompressScratch",
                        &format!("lb bc, $01, ${tiles:02x}"),
                        "call Request2bpp",
                    ],
                    "source_transfer_wrapper",
                    json!({}),
                )?;
                transfers.push(json!({"dispatcher_entry":entry,"dispatch_tick":1,"source_symbol":source_symbol,"resource":resource,"destination":destination.clone().context("intro transfer destination")?,"target_vram_bank":bank,"tile_count":tiles,"wrapper":wrapper_label,"source_span":{"file":source.file,"start_line":line_number,"end_line":line_number},"implementation_source_span":source.span(&wrapper_label)?}));
            }
        }
    }
    ensure!(
        transfers.len() == 37,
        "authored intro decompression count changed"
    );
    let defaults = home
        .lines
        .iter()
        .filter_map(|(_, line)| {
            line.strip_prefix("DEF ")
                .and_then(|raw| raw.split_once(" EQU "))
        })
        .filter(|(name, _)| matches!(*name, "TILES_PER_CYCLE" | "MOBILE_TILES_PER_CYCLE"))
        .map(|(name, value)| Ok((name.to_string(), value.parse::<u8>()?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let chunking = json!({"default_tiles_per_vblank":defaults.get("TILES_PER_CYCLE").context("default source transfer rate")?,"mobile_tiles_per_vblank":defaults.get("MOBILE_TILES_PER_CYCLE").context("mobile source transfer rate")?});
    Ok(
        json!({"scene_labels":dispatcher,"resource_transfers":transfers,"chunking":chunking,"completion":{"blocking":true,"wait":"DelayFrame","until":"wRequested2bppSize == 0"},"request_source_span":home.span("Request2bpp")?}),
    )
}

/// Lower setup routines without assigning unmeasured CPU/frame costs.
/// The timing exporter annotates these source-certified operations afterwards.
pub(super) fn setup_phases(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let source = Source::read(vendor, "engine/movie/intro.asm")?;
    let (addresses, banks) = symbols(vendor)?;
    let mut constants = constants.clone();
    constants.extend(addresses);
    let byte =
        |expression: &str| -> Result<u8> { Ok(evaluate(expression, &constants, &banks)? as u8) };
    let sprites = super::source_title_sprites::export(vendor)?;
    let metadata = transfer_metadata(vendor)?;
    let mut phases = BTreeMap::new();
    for number in [1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 26] {
        let label = format!("IntroScene{number}");
        let mut cursor = Cursor::new(&source, &label)?;
        let mut index = cursor.index;
        let mut operations = Vec::new();
        let mut accumulator = 0_u8;
        let mut hl = String::new();
        let mut de = String::new();
        let mut bc = 0_i64;
        let mut c = 0_u8;
        let mut attrmap_offset = 0_i64;
        while index < cursor.body.len() {
            let line = cursor.body[index].1.clone();
            let mut operation = None;
            if line == "xor a" {
                accumulator = 0;
            } else if let Some(value) = line.strip_prefix("ld a, ") {
                accumulator = byte(value)?;
            } else if let Some(value) = line.strip_prefix("ld hl, ") {
                hl = value.to_string();
            } else if let Some(value) = line.strip_prefix("ld de, ") {
                de = value.to_string();
            } else if let Some(value) = line.strip_prefix("ld bc, ") {
                bc = evaluate(value, &constants, &banks)?;
            } else if let Some(value) = line.strip_prefix("ld c, ") {
                c = byte(value)?;
            } else if line.starts_with("lb bc, BANK(") {
                c = line
                    .rsplit_once(", ")
                    .context("intro Request2bpp operand")?
                    .1
                    .parse()?;
            } else if line == "hlcoord 0, 0, wAttrmap" {
                hl = "wAttrmap".into();
                attrmap_offset = 0;
            } else if line == "debgcoord 0, 0" {
                de = "vBGMap0".into();
            } else if line == "ldh a, [rWBK]" {
                ensure!(
                    cursor
                        .body
                        .get(index + 1)
                        .is_some_and(|(_, s)| s == "push af"),
                    "intro bank read lacks save"
                );
                index += 1;
                operation = Some((
                    "save_memory_byte",
                    json!({"source":"rWBK","result":"intro_saved_wram_bank"}),
                ));
            } else if line == "pop af" {
                ensure!(
                    cursor
                        .body
                        .get(index + 1)
                        .is_some_and(|(_, s)| s == "ldh [rWBK], a"),
                    "intro bank restore changed"
                );
                index += 1;
                operation = Some((
                    "restore_memory_byte",
                    json!({"target":"rWBK","result":"intro_saved_wram_bank"}),
                ));
            } else if line == "ld [hli], a" && hl == "wSpriteAnimDict" {
                ensure!(
                    cursor
                        .body
                        .get(index + 1)
                        .is_some_and(|(_, s)| s == "ld [hl], $7f"),
                    "intro sprite dictionary override changed"
                );
                index += 1;
                operation = Some((
                    "write_memory_bytes",
                    json!({"target":"wSpriteAnimDict","values":[accumulator,127]}),
                ));
            } else if let Some(target) = line
                .strip_prefix("ldh [")
                .or_else(|| line.strip_prefix("ld ["))
                .and_then(|s| s.strip_suffix("], a"))
            {
                operation = Some((
                    "write_memory_byte",
                    json!({"target":target,"value":accumulator}),
                ));
            } else if line.starts_with("depixel ") {
                let call_index = index + 2;
                ensure!(
                    cursor
                        .body
                        .get(call_index)
                        .is_some_and(|(_, s)| s == "call InitSpriteAnimStruct"),
                    "intro setup sprite allocation changed"
                );
                let activation = sprites["activation_operations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|op| {
                        op["dispatcher_entry"] == number - 1
                            && op["source_span"]["start_line"] == cursor.body[index].0
                    })
                    .context("intro setup allocation certificate")?
                    .clone();
                let mut fields = activation.as_object().unwrap().clone();
                fields.remove("op");
                fields.remove("source_span");
                index = call_index;
                operation = Some(("sprite_activate", Value::Object(fields)));
            } else if line == "ret" {
                operation = Some(("return", json!({})));
            } else if let Some(call) = line
                .strip_prefix("call ")
                .or_else(|| line.strip_prefix("farcall "))
            {
                operation = Some(match call {
                    "NextIntroScene" => (
                        "increment_memory_byte",
                        json!({"target":"wJumptableIndex","delta":1,"implementation_source_span":source.span(call)?}),
                    ),
                    "DelayFrame" => ("wait_frames", json!({"frames":1})),
                    "DelayFrames" => ("wait_frames", json!({"frames":c})),
                    "Intro_SetCGBPalUpdate" => (
                        "palette_transfer_request",
                        json!({"target":"hCGBPalUpdate","value":1,"implementation_source_span":source.span(call)?}),
                    ),
                    "ClearSprites" => (
                        "fill_memory",
                        json!({"target":"wShadowOAM","value":0,"byte_count":constants["wShadowOAMEnd"]-constants["wShadowOAM"]}),
                    ),
                    "ClearSpriteAnims" => (
                        "fill_memory",
                        json!({"target":"wSpriteAnimData","value":0,"byte_count":constants["wSpriteAnimDataEnd"]-constants["wSpriteAnimData"]}),
                    ),
                    "ClearTilemap" => (
                        "fill_memory",
                        json!({"target":"wTilemap","value":127,"byte_count":constants["wTilemapEnd"]-constants["wTilemap"]}),
                    ),
                    "ByteFill" => {
                        let fields = json!({"target":hl,"value":accumulator,"byte_count":bc,"target_offset":if hl == "wAttrmap" {attrmap_offset} else {0}});
                        if hl == "wAttrmap" {
                            attrmap_offset += bc;
                        }
                        ("fill_memory", fields)
                    }
                    "CopyBytes" => (
                        "copy_memory",
                        json!({"source":resource(&source,&hl)?,"target":de,"byte_count":bc}),
                    ),
                    "PlayMusic" => (
                        "play_audio",
                        json!({"audio":de,"dispatcher_entry":number-1,"dispatch_tick":1}),
                    ),
                    "Intro_LoadTilemap" => (
                        "copy_strided_memory",
                        json!({"source":"wDecompressScratch","target":"wTilemap","rows":constants["SCREEN_HEIGHT"],"row_bytes":constants["SCREEN_WIDTH"],"source_stride":constants["TILEMAP_WIDTH"],"target_stride":constants["SCREEN_WIDTH"],"implementation_source_span":source.span(call)?}),
                    ),
                    "Intro_ColoredSuicuneFrameSwap" => (
                        "tilemap_xor",
                        json!({"target":"wTilemap","xor_mask":8,"condition":{"nonzero":true,"unsigned_less_than":128},"completion_write":{"target":"hBGMapMode","value":1},"implementation_source_span":source.span(call)?}),
                    ),
                    "Intro_ClearBGPals" | "ClearBGPalettes" | "Intro_ResetLYOverrides" => {
                        let (target, value, count) = match call {
                            "Intro_ResetLYOverrides" => (
                                "wLYOverrides",
                                0,
                                constants["wLYOverridesEnd"] - constants["wLYOverrides"],
                            ),
                            "ClearBGPalettes" => ("wBGPals2", 255, 128),
                            _ => ("wBGPals2", 0, 128),
                        };
                        (
                            "fill_memory",
                            json!({"target":target,"value":value,"byte_count":count,"helper":call}),
                        )
                    }
                    wrapper if wrapper.starts_with("Intro_DecompressRequest2bpp_") => {
                        let transfer = metadata["resource_transfers"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|transfer| {
                                transfer["source_span"]["start_line"] == cursor.body[index].0
                            })
                            .context("source setup transfer certificate")?
                            .clone();
                        let fields = json!({"resource":transfer["resource"],"target":"wDecompressScratch","wrapper":wrapper,"dispatcher_entry":number-1,"dispatch_tick":1,"implementation_source_span":transfer["implementation_source_span"]});
                        let count = index + 1 - cursor.index;
                        emit(
                            &mut cursor,
                            count,
                            "decompress_lz3_resource",
                            fields,
                            &mut operations,
                        )?;
                        let request = json!({"op":"request_2bpp_transfer","source":"wDecompressScratch","target":transfer["destination"],"target_vram_bank":transfer["target_vram_bank"],"tile_count":transfer["tile_count"],"chunking":metadata["chunking"],"completion":metadata["completion"],"dispatcher_entry":number-1,"dispatch_tick":1,"source_span":transfer["source_span"],"implementation_source_span":metadata["request_source_span"]});
                        operations.push(request);
                        index += 1;
                        continue;
                    }
                    "Request2bpp" => (
                        "request_2bpp_transfer",
                        json!({"source":resource(&source,&de)?,"target":hl,"tile_count":c,"chunking":metadata["chunking"],"completion":metadata["completion"],"dispatcher_entry":number-1,"dispatch_tick":1,"implementation_source_span":metadata["request_source_span"]}),
                    ),
                    _ => anyhow::bail!("unlowered setup helper {label}: {call}"),
                });
            } else {
                anyhow::bail!("unlowered setup instruction {label}: {line}");
            }
            if let Some((op, fields)) = operation {
                let count = index + 1 - cursor.index;
                emit(&mut cursor, count, op, fields, &mut operations)?;
                if line == "call Intro_ResetLYOverrides" {
                    operations.push(source.operation(
                        "Intro_ResetLYOverrides",
                        &["ld a, LOW(rSCX)", "ldh [hLCDCPointer], a", "ret"],
                        "write_memory_byte",
                        json!({"target":"hLCDCPointer","value":constants["rSCX"]&255}),
                    )?);
                }
                if line == "call Intro_ClearBGPals" || line == "call ClearBGPalettes" {
                    let home = Source::read(vendor, "home/tilemap.asm")?;
                    let helper = if line == "call Intro_ClearBGPals" {
                        &source
                    } else {
                        &home
                    };
                    let label = if line == "call Intro_ClearBGPals" {
                        "Intro_ClearBGPals"
                    } else {
                        "ClearPalettes"
                    };
                    operations.push(helper.operation(
                        label,
                        &["ld a, TRUE", "ldh [hCGBPalUpdate], a"],
                        "palette_transfer_request",
                        json!({"target":"hCGBPalUpdate","value":1}),
                    )?);
                }
                if line == "call ClearTilemap" || line == "call ClearBGPalettes" {
                    let home = Source::read(vendor, "home/tilemap.asm")?;
                    operations.push(home.operation(
                        "WaitBGMap",
                        &["ld a, 1", "ldh [hBGMapMode], a"],
                        "write_memory_byte",
                        json!({"target":"hBGMapMode","value":1}),
                    )?);
                    operations.push(home.operation(
                        "WaitBGMap",
                        &["ld c, 4", "call DelayFrames", "ret"],
                        "wait_frames",
                        json!({"frames":4,"lcd_enabled_source_precondition":true}),
                    )?);
                }
                if line == "call Intro_ClearBGPals" {
                    let wait = source.operation(
                        "Intro_ClearBGPals",
                        &["call DelayFrame", "call DelayFrame", "ret"],
                        "wait_frames",
                        json!({"frames":2}),
                    )?;
                    operations.push(wait);
                }
            }
            index += 1;
        }
        cursor.finish()?;
        phases.insert(
            label.clone(),
            json!({"source_span":source.span(&label)?,"operations":operations}),
        );
    }
    Ok(json!(phases))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_dispatch_consumes_all_twenty_eight_scenes() {
        let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else {
            return;
        };
        let root = Path::new(&root);
        let constants: Value = serde_json::from_slice(
            &std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json"))
                .unwrap(),
        )
        .unwrap();
        let constants = serde_json::from_value(constants["global"].clone()).unwrap();
        let dispatch = scene_dispatch(&root.join("vendor/pokecrystal"), &constants).unwrap();
        for number in 1..=28 {
            assert!(
                dispatch["phase"]["labels"]
                    .get(format!("IntroScene{number}"))
                    .is_some()
            );
        }
    }
    #[test]
    fn animated_dispatch_executes_every_source_frame_in_production_phase_machine() {
        use crystal_assets::{
            RuntimePresentationPhase, RuntimePresentationPhaseMachine, RuntimePresentationProgram,
            RuntimePresentationSubprogram,
        };
        let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else {
            return;
        };
        let root = Path::new(&root);
        let constants: Value = serde_json::from_slice(
            &std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json"))
                .unwrap(),
        )
        .unwrap();
        let constants = serde_json::from_value(constants["global"].clone()).unwrap();
        let data = scene_dispatch(&root.join("vendor/pokecrystal"), &constants).unwrap();
        let phase = RuntimePresentationPhase {
            id: "scene_dispatch".into(),
            labels: serde_json::from_value(data["phase"]["labels"].clone()).unwrap(),
            operations: serde_json::from_value(data["phase"]["operations"].clone()).unwrap(),
            ..Default::default()
        };
        let program = RuntimePresentationProgram {
            subprograms: vec![RuntimePresentationSubprogram {
                id: "intro".into(),
                phases: vec![phase],
                ..Default::default()
            }],
            ..Default::default()
        };
        for scene in [2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 25, 27, 28] {
            for frame in 0..=255 {
                let mut machine =
                    RuntimePresentationPhaseMachine::new(&program, "intro", "scene_dispatch")
                        .unwrap();
                for (symbol, value) in [
                    ("wIntroSceneFrameCounter", frame),
                    ("wIntroSceneTimer", frame),
                    ("wJumptableIndex", scene - 1),
                    ("wGlobalAnimXOffset", 240),
                    ("hSCX", 0),
                    ("hSCY", 216),
                    ("hCGB", 1),
                ] {
                    machine.memory.insert(symbol.into(), value);
                }
                machine
                    .run_from_label(&program, &format!("IntroScene{scene}"), 0)
                    .unwrap_or_else(|error| panic!("scene {scene}, frame {frame}: {error:#}"));
            }
        }
    }
    #[test]
    fn real_intro_transfers_retain_source_counts_and_vram_banks() {
        let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else {
            return;
        };
        let data = transfer_metadata(&Path::new(&root).join("vendor/pokecrystal")).unwrap();
        assert_eq!(data["resource_transfers"].as_array().unwrap().len(), 37);
        assert_eq!(data["chunking"]["default_tiles_per_vblank"], 8);
        assert_eq!(data["chunking"]["mobile_tiles_per_vblank"], 6);
        let transfers = data["resource_transfers"].as_array().unwrap();
        assert!(transfers.iter().any(
            |x| x["resource"] == "gfx/intro/pichu_wooper.2bpp.lz" && x["target_vram_bank"] == 1
        ));
        assert!(
            transfers
                .iter()
                .any(|x| x["resource"] == "gfx/intro/suicune_run.2bpp.lz"
                    && x["target_vram_bank"] == 0
                    && x["tile_count"] == 255)
        );
    }

    #[test]
    fn setup_lowering_consumes_every_authored_instruction() {
        let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else {
            return;
        };
        let root = Path::new(&root);
        let constants: Value = serde_json::from_slice(
            &std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json"))
                .unwrap(),
        )
        .unwrap();
        let constants = serde_json::from_value(constants["global"].clone()).unwrap();
        let phases = setup_phases(&root.join("vendor/pokecrystal"), &constants).unwrap();
        assert_eq!(phases.as_object().unwrap().len(), 13);
        assert_eq!(
            phases
                .as_object()
                .unwrap()
                .values()
                .flat_map(|phase| phase["operations"].as_array().unwrap())
                .filter(|op| op["op"] == "decompress_lz3_resource")
                .count(),
            37
        );
    }
}
