//! Source-backed Crystal intro allocations and sprite callback programs.
//! Dispatch ticks below are derived from the authored postincrement comparisons;
//! elapsed ROM frame crossings are supplied separately by the timing exporter.
use super::super::source_syntax::{code_part, source_arguments};
use super::Source;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn span(source: &Source, first: u32, last: u32) -> Value {
    json!({"file":source.file,"start_line":first,"end_line":last})
}

fn coordinates(raw: &str) -> Result<(u8, u8)> {
    let args = source_arguments(raw)?;
    ensure!(
        matches!(args.len(), 2 | 4),
        "intro depixel requires two or four arguments"
    );
    let value = |index: usize| -> Result<u16> { Ok(args[index].parse()?) };
    // depixel loads D from the first argument (Y), E from the second (X).
    let y = value(0)? * 8 + if args.len() == 4 { value(2)? } else { 0 };
    let x = value(1)? * 8 + if args.len() == 4 { value(3)? } else { 0 };
    Ok((u8::try_from(x)?, u8::try_from(y)?))
}

fn callback(source: &Source, core: &Source, symbol: &str, target: &str) -> Result<Value> {
    let body = source.body(target)?;
    let mut instructions = Vec::<Value>::new();
    let mut labels = BTreeMap::new();
    let mut reads = Vec::new();
    let mut reinitializations = Vec::new();
    for (line_number, line) in body.iter().skip(1) {
        if line.starts_with('.') && !line.contains(' ') {
            ensure!(
                labels
                    .insert(line.trim_end_matches(':').to_string(), instructions.len())
                    .is_none(),
                "duplicate intro callback label"
            );
            continue;
        }
        let (opcode, args) = line.split_once(' ').unwrap_or((line, ""));
        ensure!(
            matches!(
                opcode,
                "ld" | "add"
                    | "cp"
                    | "and"
                    | "xor"
                    | "inc"
                    | "push"
                    | "pop"
                    | "call"
                    | "jr"
                    | "jp"
                    | "ret"
            ),
            "unsupported authored intro callback instruction {line}"
        );
        let args = if args.is_empty() {
            Vec::new()
        } else {
            source_arguments(args)?
        };
        if opcode == "ld" && args.first().map(String::as_str) == Some("a") {
            if let Some(read) = args
                .get(1)
                .and_then(|arg| arg.strip_prefix("[w"))
                .and_then(|arg| arg.strip_suffix(']'))
            {
                let source_symbol = format!("w{read}");
                let canonical = match source_symbol.as_str() {
                    "wIntroSceneTimer" => "wIntroSceneTimer",
                    "wSlotsDelay" => "wIntroSceneFrameCounter",
                    _ => anyhow::bail!("unsupported outer intro callback read {source_symbol}"),
                };
                reads.push(json!({"source_symbol":source_symbol,"symbol":canonical,"source_span":span(source,*line_number,*line_number)}));
            }
        }
        if opcode == "call" && args == ["_ReinitSpriteAnimFrame"] {
            let previous = instructions
                .last()
                .context("intro callback reinit lacks frameset load")?;
            let frameset = previous["args"][1]
                .as_str()
                .filter(|name| name.starts_with("SPRITE_ANIM_FRAMESET_"))
                .context("intro callback reinit lacks symbolic frameset")?;
            reinitializations.push(json!({"frameset":frameset,"guard":{"kind":"authored_control_flow"},"application":"reset_frame_and_duration","source_span":span(source,*line_number-1,*line_number),"implementation_source_span":core.span("_ReinitSpriteAnimFrame")?}));
        }
        let mut instruction = json!({"opcode":opcode,"args":args,"source_span":span(source,*line_number,*line_number)});
        if line == "call AnimSeqs_Sine" || line == "call AnimSeqs_Cosine" {
            instruction["source_machine_cycles"] = json!([6]);
            instruction["source_helper_timing"] = json!({"routine":line.strip_prefix("call ").unwrap(),
                "source_file":"engine/sprite_anims/core.asm","model":"sine_amplitude_shift_v1",
                "angle_register":"a","amplitude_register":"d","caller_call_included":false,"interrupts_included":false});
        } else if line == "call _ReinitSpriteAnimFrame" {
            let body = core.body("_ReinitSpriteAnimFrame")?;
            let mut tape = vec![6];
            for (_, line) in body.iter().skip(1) {
                tape.extend(
                    super::source_title_intro_measurements::plain_instruction_cycles(line, false)
                        .context("unmodeled source sprite reinit instruction")?,
                );
            }
            instruction["source_machine_cycles"] = json!(tape);
        } else {
            instruction["source_machine_cycles"] = json!(
                super::source_title_intro_measurements::plain_instruction_cycles(line, false)
                    .with_context(|| format!("unmodeled intro callback timing {line}"))?
            );
            let taken =
                super::source_title_intro_measurements::plain_instruction_cycles(line, true)
                    .context("intro callback taken timing")?;
            if instruction["source_machine_cycles"] != json!(taken) {
                instruction["source_machine_cycles_taken"] = json!(taken);
            }
        }
        instructions.push(instruction);
    }
    for instruction in &instructions {
        if matches!(instruction["opcode"].as_str(), Some("jr" | "jp")) {
            let target = instruction["args"]
                .as_array()
                .and_then(|args| args.last())
                .and_then(Value::as_str)
                .context("intro callback branch target")?;
            ensure!(
                labels.contains_key(target),
                "unresolved intro callback branch {target}"
            );
        }
    }
    Ok(
        json!({"kind":"direct","symbol":symbol,"target":target,"source_span":source.span(target)?,"instructions":instructions,"labels":labels,"outer_memory_reads":reads,"frameset_reinitializations":reinitializations,"per_tick_struct_deltas":[],"host_operations":[]}),
    )
}

fn object_definitions(vendor: &Path) -> Result<BTreeMap<String, (String, String)>> {
    let constants = std::fs::read_to_string(vendor.join("constants/sprite_anim_constants.asm"))?;
    let names = |prefix: &str| {
        constants
            .lines()
            .map(code_part)
            .filter_map(|line| {
                line.strip_prefix("const ")
                    .filter(|name| name.starts_with(prefix))
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>()
    };
    let objects = std::fs::read_to_string(vendor.join("data/sprite_anims/objects.asm"))?
        .lines()
        .map(code_part)
        .filter_map(|line| line.strip_prefix("db ").map(source_arguments))
        .collect::<Result<Vec<_>>>()?;
    let object_names = names("SPRITE_ANIM_OBJ_");
    ensure!(
        objects.len() == object_names.len(),
        "intro object constant/table mismatch"
    );
    object_names
        .into_iter()
        .zip(objects)
        .filter(|(name, _)| name.starts_with("SPRITE_ANIM_OBJ_INTRO_"))
        .map(|(name, args)| {
            ensure!(
                args.len() == 3 && args[2] == "SPRITE_ANIM_DICT_DEFAULT",
                "intro object dictionary requires authored default binding"
            );
            Ok((name, (args[0].clone(), args[1].clone())))
        })
        .collect()
}

pub(super) fn export(vendor: &Path) -> Result<Value> {
    let movie = Source::read(vendor, "engine/movie/intro.asm")?;
    let functions = Source::read(vendor, "engine/sprite_anims/functions.asm")?;
    let core = Source::read(vendor, "engine/sprite_anims/core.asm")?;
    {
        use sha2::Digest;
        for (file, expected) in [
            (
                "home/sprite_anims.asm",
                "825ad2cf62c29504f900c222733c16c95ff7c404c8c6d682968c484e0eabf698",
            ),
            (
                "engine/movie/intro.asm",
                "4d55128b81014155450a8b1ba53f6972c04e0225cc9f05056cccf1e16cee8b90",
            ),
            (
                "engine/sprite_anims/core.asm",
                "1dc8847309147b1d9704d2e3c15dfcc6f4461623f93bad71e52d5c26399706a9",
            ),
            (
                "macros/code.asm",
                "560fd40cc7e1a2005cb4b9e81a02077833087d9698d35cff8dac6a47ffaafd57",
            ),
        ] {
            ensure!(
                format!(
                    "{:x}",
                    sha2::Sha256::digest(std::fs::read(vendor.join(file))?)
                ) == expected,
                "intro sprite instruction-model source changed: {file}"
            );
        }
    }
    let objects = object_definitions(vendor)?;
    let constants = std::fs::read_to_string(vendor.join("constants/sprite_anim_constants.asm"))?;
    ensure!(
        constants
            .lines()
            .map(code_part)
            .any(|line| line == "DEF NUM_SPRITE_ANIM_STRUCTS EQU 10"),
        "intro scheduler allocation capacity changed"
    );
    let function_names = constants
        .lines()
        .map(code_part)
        .filter_map(|line| {
            line.strip_prefix("const SPRITE_ANIM_FUNC_")
                .map(|suffix| format!("SPRITE_ANIM_FUNC_{suffix}"))
        })
        .collect::<Vec<_>>();
    let pointers = functions
        .body("DoSpriteAnimFrame")?
        .iter()
        .filter_map(|(_, line)| line.strip_prefix("dw ").map(str::to_owned))
        .collect::<Vec<_>>();
    ensure!(
        pointers.len() == function_names.len(),
        "intro callback pointer count mismatch"
    );
    let targets: BTreeMap<_, _> = function_names.into_iter().zip(pointers).collect();
    let ram = std::fs::read_to_string(vendor.join("ram/wram.asm"))?;
    let union = ram
        .split("; intro data")
        .nth(1)
        .context("source intro WRAM union")?
        .split("; title data")
        .next()
        .unwrap();
    ensure!(
        union.contains("wIntroSceneFrameCounter:: db\nwIntroSceneTimer:: db"),
        "intro WRAM byte ordering changed"
    );
    ensure!(
        ram.contains("; slot machine\nwSlotsDelay:: db"),
        "slot delay no longer the first union byte"
    );
    let mut scheduler = movie.operation("CrystalIntro", &["call IntroSceneJumper", "farcall PlaySpriteAnimations", "call DelayFrame"], "sprite_scheduler_step", json!({"call_order":["DoSpriteAnimFrame","UpdateAnimFrame"],"allocation_order":"first_free_slot","capacity":10,"implementation_source_span":core.span("DoNextFrameForAllSprites")?}))?;
    scheduler["source_scheduler_timing"] = json!({"routine":"PlaySpriteAnimations","source_file":"engine/sprite_anims/core.asm",
        "model":"slot_callback_update_oam_v1","capacity":10,"structure_byte_count":16,"shadow_oam_byte_count":160,
        "caller_call_included":false,"interrupts_included":false,"callback_costs":"source_machine_cycles",
        "update_frame_model":"frame_command_flip_and_oam_counter_v1"});
    let initializer = core.span("_InitSpriteAnimStruct")?;
    core.operation(
        "_InitSpriteAnimStruct",
        &[
            "xor a",
            "ld [hli], a",
            "ld [hli], a",
            "xor a",
            "ld [hli], a",
            "ld [hli], a",
            "dec a",
            "ld [hli], a",
            "xor a",
            "ld [hli], a",
            "ld [hli], a",
            "ld [hli], a",
            "ld [hli], a",
            "ld [hl], a",
        ],
        "initialize_sprite_memory",
        json!({}),
    )?;
    let mut programs = Vec::new();
    let mut operations = Vec::new();
    // Every guarded allocation must be certified against the actual comparison.
    let plans = [
        ("IntroScene2", 97, "depixel 11, 11", true),
        ("IntroScene6", 33, "depixel 7, 15", true),
        ("IntroScene6", 97, "depixel 14, 6", true),
        ("IntroScene7", 1, "depixel 13, 27, 4, 0", false),
        ("IntroScene10", 33, "depixel 22, 6", false),
        ("IntroScene10", 65, "depixel 21, 16, 1, 0", false),
        ("IntroScene13", 1, "depixel 13, 11, 4, 0", false),
        ("IntroScene15", 1, "depixel 8, 5", false),
        ("IntroScene15", 1, "depixel 12, 0", false),
        ("IntroScene19", 1, "depixel 12, 0", false),
    ];
    for (scene, tick, depixel, group) in plans {
        let entry: u8 = scene.strip_prefix("IntroScene").unwrap().parse::<u8>()? - 1;
        let scene_body = movie.body(scene)?;
        if tick > 1 {
            ensure!(
                scene_body.iter().any(|(_, line)| line == "inc [hl]"),
                "intro allocation clock is no longer postincremented"
            );
            let comparison = format!("cp ${:x}", tick - 1);
            ensure!(
                scene_body.iter().any(|(_, line)| line == &comparison),
                "intro allocation comparison changed"
            );
        }
        let start = scene_body
            .iter()
            .position(|(_, line)| line == depixel)
            .context("missing authored sprite coordinate load")?;
        let (x, y) = coordinates(depixel.strip_prefix("depixel ").unwrap())?;
        let allocation_line = scene_body[start].0;
        let allocations = if group {
            ensure!(
                scene_body[start + 1].1 == "call CrystalIntro_InitUnownAnim",
                "intro group call changed"
            );
            movie
                .body("CrystalIntro_InitUnownAnim")?
                .iter()
                .enumerate()
                .filter(|(_, (_, line))| line == "call InitSpriteAnimStruct")
                .map(|(index, _)| ("CrystalIntro_InitUnownAnim", index))
                .collect::<Vec<_>>()
        } else {
            ensure!(
                scene_body[start + 2].1 == "call InitSpriteAnimStruct",
                "intro direct allocation changed"
            );
            vec![(scene, start + 2)]
        };
        let mut instances = Vec::new();
        for (allocation_root, index) in allocations {
            let body = movie.body(allocation_root)?;
            let object = body[index - 1]
                .1
                .strip_prefix("ld a, ")
                .context("intro allocation object load")?;
            let (default_frameset, function) =
                objects.get(object).context("unknown intro sprite object")?;
            let target = targets
                .get(function)
                .context("unknown intro callback pointer")?;
            let callback = callback(&functions, &core, function, target)?;
            let line = body[index].0;
            let instance = format!("sprite:engine/movie/intro.asm:{allocation_line}:{tick}:{line}");
            let (frameset, var1) = if group {
                ensure!(
                    body[index + 1].1 == "ld hl, SPRITEANIMSTRUCT_VAR1"
                        && body[index + 2].1 == "add hl, bc",
                    "intro unown initialization changed"
                );
                let var1 = body[index + 3]
                    .1
                    .strip_prefix("ld [hl], $")
                    .context("intro unown radius")?;
                let var1 = u8::from_str_radix(var1, 16)?;
                let frameset = body[index + 4]
                    .1
                    .strip_prefix("ld a, ")
                    .context("intro unown frameset")?;
                ensure!(
                    body[index + 5].1 == "call ReinitSpriteAnimFrame",
                    "intro unown reinitialization changed"
                );
                (frameset.to_string(), var1)
            } else {
                (default_frameset.clone(), 0)
            };
            let gfx = match object {
                "SPRITE_ANIM_OBJ_INTRO_UNOWN" => "IntroPulseGFX",
                "SPRITE_ANIM_OBJ_INTRO_SUICUNE" => "IntroSuicuneRunGFX",
                "SPRITE_ANIM_OBJ_INTRO_PICHU" | "SPRITE_ANIM_OBJ_INTRO_WOOPER" => {
                    "IntroPichuWooperGFX"
                }
                "SPRITE_ANIM_OBJ_INTRO_UNOWN_F" => "IntroUnownBackGFX",
                "SPRITE_ANIM_OBJ_INTRO_SUICUNE_AWAY" => "IntroGrass4GFX",
                _ => anyhow::bail!("unknown intro graphic binding"),
            };
            let resource = movie
                .body(gfx)?
                .iter()
                .find_map(|(_, line)| {
                    line.strip_prefix("INCBIN \"")
                        .and_then(|raw| raw.strip_suffix('"'))
                })
                .context("intro sprite graphic source")?;
            let tile_base = if scene == "IntroScene19" { 127 } else { 0 };
            programs.push(json!({"instance":instance,"allocation_source_span":span(&movie,allocation_line,line.max(allocation_line)),"initializer_source_span":initializer,"object":{"symbol":object},"frameset":{"symbol":frameset},"callback":callback,"graphic_binding":{"resource":resource,"tile_base":tile_base,"source_span":movie.span(gfx)?},"initial_memory":{"xcoord":x,"ycoord":y,"xoffset":0,"yoffset":0,"duration":0,"duration_offset":0,"frame":255,"tile_id":tile_base,"jumptable_index":0,"var1":var1,"var2":0,"var3":0,"var4":0},"lifetime":{"allocation_dispatcher_entry":entry,"allocation_dispatch_tick":tick}}));
            instances.push(instance);
        }
        let certificate = if group {
            vec![depixel, "call CrystalIntro_InitUnownAnim"]
        } else {
            vec![
                depixel,
                scene_body[start + 1].1.as_str(),
                "call InitSpriteAnimStruct",
            ]
        };
        let fields = if group {
            json!({"dispatcher_entry":entry,"dispatch_tick":tick,"instances":instances})
        } else {
            json!({"dispatcher_entry":entry,"dispatch_tick":tick,"instance":instances[0],"lifetime":{"allocation_dispatcher_entry":entry,"allocation_dispatch_tick":tick}})
        };
        let mut fields = fields;
        fields["source_sprite_init_caller_instruction_machine_cycles"] = if group {
            json!([3, 6])
        } else {
            json!([3, 2, 6])
        };
        fields["source_sprite_init_timing"] = json!({"routine":if group {"CrystalIntro_InitUnownAnim"} else {"InitSpriteAnimStruct"},
            "source_file":"engine/sprite_anims/core.asm","model":"first_free_slot_and_count_v1","capacity":10,
            "dictionary_first_key":0,"caller_call_included":false,"interrupts_included":false});
        operations.push(movie.operation(
            scene,
            &certificate,
            if group {
                "sprite_init_group"
            } else {
                "sprite_activate"
            },
            fields,
        )?);
    }
    ensure!(
        programs.len() == 19,
        "source intro allocation count changed"
    );
    Ok(
        json!({"sprite_programs":programs,"activation_operations":operations,"scheduler":scheduler,"required_consumer":{"sprite_scheduler":"production_sprite_animation_bundle","callback_execution":"authored_opcode_program","allocation_policy":"first_free_slot","capacity":10,"scheduler_source_span":core.span("PlaySpriteAnimations")?,"initializer_source_span":initializer}}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn depixel_preserves_register_order_and_subtile_offsets() {
        assert_eq!(coordinates("13, 27, 4, 0").unwrap(), (216, 108));
        assert_eq!(coordinates("21, 16, 1, 0").unwrap(), (128, 169));
        assert!(coordinates("40, 2").is_err());
    }
    #[test]
    fn real_intro_allocations_have_exact_callbacks_and_source_ticks() {
        let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else {
            return;
        };
        let data = export(&Path::new(&root).join("vendor/pokecrystal")).unwrap();
        assert_eq!(data["sprite_programs"].as_array().unwrap().len(), 19);
        let groups = data["activation_operations"].as_array().unwrap();
        assert_eq!(
            groups
                .iter()
                .filter(|op| op["op"] == "sprite_init_group")
                .count(),
            3
        );
        let pichu = data["sprite_programs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["object"]["symbol"] == "SPRITE_ANIM_OBJ_INTRO_PICHU")
            .unwrap();
        assert_eq!(pichu["initial_memory"]["ycoord"], 169);
        assert_eq!(pichu["lifetime"]["allocation_dispatch_tick"], 65);
        let away = data["sprite_programs"].as_array().unwrap().last().unwrap();
        assert_eq!(away["initial_memory"]["tile_id"], 127);
        assert_eq!(away["graphic_binding"]["resource"], "gfx/intro/grass4.2bpp");
        if let Some(path) = std::env::var_os("CRYSTAL_INTRO_ORACLE_TRACE") {
            let trace: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            let records = trace["records"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|record| record["hook"] == "InitSpriteAnimStruct")
                .collect::<Vec<_>>();
            let constants = std::fs::read_to_string(
                Path::new(&root).join("vendor/pokecrystal/constants/sprite_anim_constants.asm"),
            )
            .unwrap();
            let object_names = constants
                .lines()
                .map(code_part)
                .filter_map(|line| {
                    line.strip_prefix("const SPRITE_ANIM_OBJ_")
                        .map(|name| format!("SPRITE_ANIM_OBJ_{name}"))
                })
                .collect::<Vec<_>>();
            let programs = data["sprite_programs"].as_array().unwrap();
            assert_eq!(records.len(), programs.len());
            for (record, program) in records.into_iter().zip(programs) {
                assert_eq!(
                    record["dispatcher_entry"],
                    program["lifetime"]["allocation_dispatcher_entry"]
                );
                assert_eq!(
                    record["dispatch_tick"],
                    program["lifetime"]["allocation_dispatch_tick"]
                );
                let de = record["de"].as_u64().unwrap();
                assert_eq!(
                    de & 255,
                    program["initial_memory"]["xcoord"].as_u64().unwrap()
                );
                assert_eq!(
                    de >> 8,
                    program["initial_memory"]["ycoord"].as_u64().unwrap()
                );
                let object_id = object_names
                    .iter()
                    .position(|name| Some(name.as_str()) == program["object"]["symbol"].as_str())
                    .unwrap();
                assert_eq!(record["a"].as_u64().unwrap(), object_id as u64);
            }
        }
    }
}
