//! Copyright/studio splash control flow and timing certified against source.
use super::Certificate;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn lower(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let source = Certificate::read(vendor, "engine/movie/splash.asm")?;
    let value = |name: &str| {
        constants
            .get(name)
            .copied()
            .with_context(|| format!("splash constant {name}"))
    };

    // Bind symbols through authored instruction operands and dispatch tables.
    // Exact sequences below still certify the pinned source behavior.
    let before = |instruction: &str, prefix: &str| -> Result<String> {
        let matches = source
            .lines
            .windows(2)
            .filter(|pair| pair[1].1 == instruction)
            .filter_map(|pair| pair[0].1.strip_prefix(prefix).map(str::to_owned))
            .collect::<Vec<_>>();
        anyhow::ensure!(
            matches.len() == 1,
            "ambiguous splash operand before {instruction}"
        );
        Ok(matches[0].clone())
    };
    let init_routine = before(".joy_loop", "call ")?;
    let scene_routine = before("farcall PlaySpriteAnimations", "call ")?;
    let cleanup_routine = source
        .lines
        .windows(2)
        .find(|pair| pair[0].1 == ".pressed_button")
        .and_then(|pair| pair[1].1.strip_prefix("call "))
        .context("splash cleanup call")?
        .to_owned();
    let layout_symbol = before("call GetSGBLayout", "ld b, ")?;
    let sprite_symbol = before("call InitSpriteAnimStruct", "ld a, ")?;
    let init_index = source
        .lines
        .iter()
        .position(|(_, code)| code == &format!("{init_routine}:"))
        .context("splash initializer definition")?;
    let logo_graphics = source.lines[init_index + 1]
        .1
        .strip_prefix("ld de, ")
        .context("splash lettering graphics operand")?
        .to_owned();
    let scene_index = source
        .lines
        .iter()
        .position(|(_, code)| code == &format!("{scene_routine}:"))
        .context("splash scene definition")?;
    let scene_table = source.lines[scene_index + 1]
        .1
        .strip_prefix("jumptable ")
        .and_then(|args| args.split(',').next())
        .context("splash scene dispatch")?;
    let table_index = source
        .lines
        .iter()
        .enumerate()
        .skip(scene_index + 2)
        .find(|(_, (_, code))| code == scene_table)
        .map(|(i, _)| i)
        .context("splash scene table")?;
    let scenes = source
        .lines
        .iter()
        .skip(table_index + 1)
        .take_while(|(_, code)| code.starts_with("dw "))
        .map(|(_, code)| code[3..].to_owned())
        .collect::<Vec<_>>();
    anyhow::ensure!(
        scenes.len() == 4,
        "splash scene table must have four authored entries"
    );
    let text_label = |routine: &str, position: &str| -> Result<String> {
        let start = source
            .lines
            .iter()
            .position(|(_, code)| code == &format!("{routine}:"))
            .context("splash text routine")?;
        let index = source
            .lines
            .iter()
            .enumerate()
            .skip(start)
            .find(|(_, (_, code))| code == position)
            .map(|(i, _)| i)
            .context("splash text destination")?;
        Ok(source.lines[index - 1]
            .1
            .strip_prefix("ld hl, ")
            .context("splash text operand")?
            .to_owned())
    };
    let studio_label = text_label(&scenes[1], "decoord 5, 10")?;
    let presents_label = text_label(&scenes[2], "decoord 7, 11")?;
    let studio_index = source
        .lines
        .iter()
        .position(|(_, code)| code == &format!("{}:", scenes[1]))
        .context("studio scene")?;
    let studio_audio = source
        .lines
        .iter()
        .skip(studio_index)
        .find_map(|(_, code)| {
            code.strip_prefix("ld de, SFX_")
                .map(|id| format!("SFX_{id}"))
        })
        .context("studio scene audio")?;
    let graphics_index = source
        .lines
        .iter()
        .position(|(_, code)| code == &format!("{logo_graphics}:"))
        .context("splash graphics definition")?;
    let lettering_resources = source
        .lines
        .iter()
        .skip(graphics_index + 1)
        .take_while(|(_, code)| code.starts_with("INCBIN "))
        .map(|(_, code)| {
            code.trim_start_matches("INCBIN ")
                .trim_matches('"')
                .to_owned()
        })
        .collect::<Vec<_>>();
    anyhow::ensure!(
        lettering_resources.len() == 2,
        "splash lettering requires two authored resources"
    );
    let objects = Certificate::read(vendor, "data/sprite_anims/objects.asm")?;
    let object_index = usize::try_from(value(&sprite_symbol)?)?;
    let object = objects
        .lines
        .iter()
        .filter_map(|(_, code)| code.strip_prefix("db "))
        .nth(object_index)
        .context("splash sprite object dispatch")?;
    let sprite_frameset = object
        .split(',')
        .next()
        .context("splash sprite frameset")?
        .trim();
    let layouts = Certificate::read(vendor, "engine/gfx/cgb_layouts.asm")?;
    let palette_routine = layouts
        .lines
        .iter()
        .filter_map(|(_, code)| code.strip_prefix("dw "))
        .nth(usize::try_from(value(&layout_symbol)?)?)
        .context("splash palette dispatch")?;
    let palette_start = layouts
        .lines
        .iter()
        .position(|(_, code)| code == &format!("{palette_routine}:"))
        .context("splash palette definition")?;
    let palette_symbol = layouts
        .lines
        .iter()
        .skip(palette_start)
        .find_map(|(_, code)| {
            code.strip_prefix("ld a, PREDEFPAL_")
                .map(|id| format!("PREDEFPAL_{id}"))
        })
        .context("splash background palette operand")?;
    let mut operations = Vec::new();
    macro_rules! op {
        ($sequence:expr,$op:expr,$fields:expr) => {
            operations.push(source.operation($sequence, $op, $fields)?);
        };
    }
    op!(
        &["ld de, MUSIC_NONE", "call PlayMusic"],
        "stop_audio",
        json!({"audio":"MUSIC_NONE"})
    );
    op!(
        &[
            "call ClearBGPalettes",
            "call ClearTilemap",
            "ld a, HIGH(vBGMap0)",
            "ldh [hBGMapAddress + 1], a",
            "xor a",
            "ldh [hBGMapAddress], a",
            "ldh [hJoyDown], a",
            "ldh [hSCX], a",
            "ldh [hSCY], a"
        ],
        "initialize_splash_display",
        json!({"bg_map":"vBGMap0","scroll":[0,0],"joy_down":0})
    );
    op!(
        &["ld a, SCREEN_HEIGHT_PX", "ldh [hWY], a"],
        "write_memory_byte",
        json!({"target":"hWY","value":value("SCREEN_HEIGHT_PX")?})
    );
    op!(
        &[
            "call WaitBGMap",
            &format!("ld b, {layout_symbol}"),
            "call GetSGBLayout",
            "call SetDefaultBGPAndOBP"
        ],
        "prepare_splash_palette",
        json!({"layout":value(&layout_symbol)?,"routine":"SetDefaultBGPAndOBP"})
    );
    op!(
        &["ld c, 10", "call DelayFrames"],
        "wait_frames",
        json!({"frames":10})
    );
    op!(
        &["callfar Copyright", "call WaitBGMap"],
        "present_copyright",
        json!({"routine":"Copyright"})
    );
    op!(
        &["ld c, 100", "call DelayFrames"],
        "wait_frames",
        json!({"frames":100})
    );
    op!(
        &["call ClearTilemap", "farcall GBCOnlyScreen"],
        "require_color_console",
        json!({"routine":"GBCOnlyScreen"})
    );
    op!(
        &[&format!("call {init_routine}")],
        "initialize_studio_splash",
        json!({"routine":init_routine,"sprite":sprite_symbol})
    );
    let loop_index = operations.len();
    op!(
        &[".joy_loop", "call JoyTextDelay", "ldh a, [hJoyLast]"],
        "sample_input",
        json!({"result":"hJoyLast","routine":"JoyTextDelay"})
    );
    op!(
        &["and PAD_BUTTONS", "jr nz, .pressed_button"],
        "input_chord_branch",
        json!({"sample":"hJoyLast","mask":value("PAD_BUTTONS")?,"predicate":"masked_nonzero","target":".pressed_button"})
    );
    op!(
        &[
            "ld a, [wJumptableIndex]",
            "bit JUMPTABLE_EXIT_F, a",
            "jr nz, .finish"
        ],
        "input_bit_branch",
        json!({"sample":"wJumptableIndex","bit":value("JUMPTABLE_EXIT_F")?,"predicate":"set","target":".finish"})
    );
    op!(
        &[&format!("call {scene_routine}")],
        "advance_studio_scene",
        json!({"routine":scene_routine})
    );
    op!(
        &["farcall PlaySpriteAnimations"],
        "advance_sprite_animations",
        json!({"routine":"PlaySpriteAnimations"})
    );
    op!(
        &["call DelayFrame", "jr .joy_loop"],
        "wait_frames",
        json!({"frames":1})
    );
    op!(
        &["call DelayFrame", "jr .joy_loop"],
        "jump",
        json!({"target":".joy_loop"})
    );
    let cancel_index = operations.len();
    op!(
        &[".pressed_button", &format!("call {cleanup_routine}")],
        "finish_studio_splash",
        json!({"routine":cleanup_routine})
    );
    op!(
        &["scf", "ret"],
        "return",
        json!({"carry":true,"result":"cancelled"})
    );
    let finish_index = operations.len();
    op!(
        &[".finish", &format!("call {cleanup_routine}")],
        "finish_studio_splash",
        json!({"routine":cleanup_routine})
    );
    op!(
        &["and a", "ret", &format!("{init_routine}:")],
        "return",
        json!({"carry":false,"result":"complete"})
    );
    let init = source.sequence(&[
        &format!("{init_routine}:"),
        &format!("ld de, {logo_graphics}"),
        "ld hl, vTiles2",
        &format!("lb bc, BANK({logo_graphics}), 28"),
        "call Get1bpp",
    ])?;
    let bounce = source.sequence(&[
        "sub 48",
        "ld [hl], a",
        "ld de, SFX_DITTO_BOUNCE",
        "call PlaySFX",
        "ret",
    ])?;
    let placement = source.sequence(&[
        "depixel 10, 11, 4, 0",
        &format!("ld a, {sprite_symbol}"),
        "call InitSpriteAnimStruct",
    ])?;
    let initial = source.sequence(&[
        "ld [hl], 96",
        "ld hl, SPRITEANIMSTRUCT_VAR2",
        "add hl, bc",
        "ld [hl], 48",
    ])?;
    let hidden = source.sequence(&[
        "ld hl, SPRITEANIMSTRUCT_YOFFSET",
        "add hl, bc",
        "ld [hl], OAM_YCOORD_HIDDEN",
    ])?;
    let delays=[(scenes[1].as_str(),"cp 32",32),(scenes[2].as_str(),"cp 64",64),(scenes[3].as_str(),"cp 128",128)].into_iter().map(|(id,comparison,frames)|Ok(json!({"routine":id,"frames":frames,"source_span":source.sequence(&[&format!("{id}:"),"ld hl, wIntroSceneTimer","ld a, [hl]",comparison])?}))).collect::<Result<Vec<_>>>()?;
    let end = source.sequence(&[
        &format!("{cleanup_routine}:"),
        "farcall ClearSpriteAnims",
        "call ClearTilemap",
        "call ClearSprites",
        "ld c, 16",
        "call DelayFrames",
        "ret",
    ])?;
    let ditto_wait = source.sequence(&[
        "ld hl, SPRITEANIMSTRUCT_VAR2",
        "add hl, bc",
        "ld a, [hl]",
        "cp 32",
        "jr nc, .start_transform",
        "inc [hl]",
        "ret",
    ])?;
    let transform_wait = source.sequence(&[
        "ld hl, SPRITEANIMSTRUCT_VAR2",
        "add hl, bc",
        "ld a, [hl]",
        "cp 64",
        "jr z, .done",
        "inc [hl]",
    ])?;
    let copyright = Certificate::read(vendor, "data/copyright.asm")?;
    let copyright_routine = Certificate::read(vendor, "engine/menus/intro_menu.asm")?;
    let copyright_position = copyright_routine.sequence(&[
        "hlcoord 2, 7",
        "ld de, CopyrightString",
        "jp PlaceString",
    ])?;
    let text_tiles = |global: &str, local: &str| -> Result<Vec<u8>> {
        let first = source
            .lines
            .iter()
            .position(|(_, code)| code == global)
            .context("missing splash text routine")?;
        let label = source
            .lines
            .iter()
            .skip(first)
            .position(|(_, code)| code == local)
            .context("missing splash text label")?
            + first;
        let code = &source
            .lines
            .get(label + 1)
            .context("missing splash text bytes")?
            .1;
        let args = code
            .strip_prefix("db ")
            .context("splash text is not authored byte data")?;
        args.split(',')
            .map(|arg| {
                Ok(u8::try_from(super::super::source_expressions::evaluate(
                    arg, constants,
                )?)?)
            })
            .collect()
    };
    let studio_tiles = text_tiles(&format!("{}:", scenes[1]), &studio_label)?;
    let presents_tiles = text_tiles(&format!("{}:", scenes[2]), &presents_label)?;
    let studio_position = source.sequence(&[
        &format!("ld hl, {studio_label}"),
        "decoord 5, 10",
        &format!("ld bc, .end - {studio_label}"),
        "call CopyBytes",
    ])?;
    let presents_position = source.sequence(&[
        &format!("ld hl, {presents_label}"),
        "decoord 7, 11",
        &format!("ld bc, .end - {presents_label}"),
        "call CopyBytes",
    ])?;
    let mut copyright_rows = vec![Vec::<u8>::new()];
    for (_, code) in &copyright.lines {
        if code == "db \"@\"" {
            break;
        }
        let args = if let Some(args) = code.strip_prefix("next ") {
            copyright_rows.push(Vec::new());
            args
        } else if let Some(args) = code.strip_prefix("db ") {
            args
        } else {
            anyhow::bail!("unsupported splash copyright source {code}");
        };
        for arg in args.split(',') {
            copyright_rows.last_mut().unwrap().push(u8::try_from(
                super::super::source_expressions::evaluate(arg, constants)?,
            )?);
        }
    }
    Ok(
        json!({"id":"splash_screen","source_entry":"SplashScreen","accepted_call_forms":["callfar"],
        "result":{"carry_set":"cancelled","carry_clear":"complete"},
        "phases":[{"id":"splash","source_span":source.sequence(&["SplashScreen:","ld de, MUSIC_NONE","call PlayMusic"])? ,
            "labels":{".joy_loop":loop_index,".pressed_button":cancel_index,".finish":finish_index},"operations":operations}],
        "loop":{"input_mask":value("PAD_BUTTONS")?,"scene_exit_bit":value("JUMPTABLE_EXIT_F")?,"frame_wait":1},
        "resources":[],"audio":[],"resource_transfers":[{"routine":init_routine,"tiles":28,"destination":"vTiles2","source_span":init}],
        "tilemap_writes":[],"sprite_operations":[{"sprite":sprite_symbol,"pixel_position":[88,84],"source_span":placement},{"initial_jump_height":96,"initial_sine_offset":48,"source_span":initial},{"jump_height_decrement":48,"audio":"SFX_DITTO_BOUNCE","source_span":bounce}],
        "sprite_programs":[],"required_consumer":{"scene_delays":delays,"cleanup_frames":16,"cleanup_source_span":end,
            "ditto_wait_frames":32,"ditto_wait_source_span":ditto_wait,"transform_frames":64,"transform_source_span":transform_wait,
            "copyright_rows":copyright_rows,"copyright_position":[2,7],"copyright_tile_base":96,
            "copyright_position_source_span":copyright_position,
            "studio_tiles":studio_tiles,"studio_position":[5,10],
            "presents_tiles":presents_tiles,"presents_position":[7,11],
            "studio_position_source_span":studio_position,"presents_position_source_span":presents_position,
            "bounce_height_decrement":48,
            "initial_hidden_y_offset":value("OAM_YCOORD_HIDDEN")?,"initial_hidden_y_offset_source_span":hidden,
            "studio_audio":studio_audio,"sprite_frameset":sprite_frameset,"lettering_resources":lettering_resources,
            "background_palette_index":value(&palette_symbol)?},
        "source_span":source.sequence(&["SplashScreen:","ld de, MUSIC_NONE","call PlayMusic"])? ,"implementation_source_spans":[]}),
    )
}
