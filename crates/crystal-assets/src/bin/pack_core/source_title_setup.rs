//! Authored clock and player naming metadata for the production title client.
use super::{Cursor, Source};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn clock(vendor: &Path) -> Result<(Value, Vec<Value>)> {
    let source = Source::read(vendor, "engine/rtc/timeset.asm")?;
    let mut cursor = Cursor::new(&source, "InitClock")?;
    let mut startup = Vec::new();
    macro_rules! take {
        ($lines:expr,$op:expr,$fields:expr) => {
            startup.push(cursor.take($lines, $op, $fields)?);
        };
    }
    take!(
        &["ldh a, [hInMenu]", "push af"],
        "save_menu_state",
        json!({"register":"hInMenu"})
    );
    take!(
        &["ld a, $1", "ldh [hInMenu], a"],
        "set_menu_state",
        json!({"register":"hInMenu","value":1})
    );
    take!(
        &["ld a, FALSE", "ld [wSpriteUpdatesEnabled], a"],
        "disable_sprite_updates",
        json!({"register":"wSpriteUpdatesEnabled"})
    );
    take!(
        &[
            "ld a, $10",
            "ld [wMusicFade], a",
            "ld a, LOW(MUSIC_NONE)",
            "ld [wMusicFadeID], a",
            "ld a, HIGH(MUSIC_NONE)",
            "ld [wMusicFadeID + 1], a"
        ],
        "fade_music",
        json!({"audio":"MUSIC_NONE","rate":16})
    );
    take!(
        &["ld c, 8", "call DelayFrames"],
        "wait",
        json!({"frames":8})
    );
    take!(
        &["call RotateFourPalettesLeft"],
        "rotate_palettes_left",
        json!({"steps":4,"frames_per_step":8})
    );
    take!(&["call ClearTilemap"], "clear_tilemap", json!({}));
    take!(&["call ClearSprites"], "clear_sprites", json!({}));
    take!(
        &["ld b, SCGB_DIPLOMA", "call GetSGBLayout"],
        "load_sgb_layout",
        json!({"layout":"SCGB_DIPLOMA"})
    );
    take!(
        &["xor a", "ldh [hBGMapMode], a"],
        "disable_bg_map_updates",
        json!({"register":"hBGMapMode"})
    );
    take!(&["call LoadStandardFont"], "load_standard_font", json!({}));
    let mut tile_lines = Vec::new();
    for (gfx, tile) in [
        ("TimeSetBackgroundGFX", "$00"),
        ("TimeSetUpArrowGFX", "$01"),
        ("TimeSetDownArrowGFX", "$02"),
    ] {
        tile_lines.extend([
            format!("ld de, {gfx}"),
            format!("ld hl, vTiles2 tile {tile}"),
            format!("lb bc, BANK({gfx}), 1"),
            "call Request1bpp".to_owned(),
        ]);
    }
    take!(
        &tile_lines.iter().map(String::as_str).collect::<Vec<_>>(),
        "load_1bpp_tiles",
        json!({"entries":[{"gfx":"TimeSetBackgroundGFX","tile":0,"count":1},{"gfx":"TimeSetUpArrowGFX","tile":1,"count":1},{"gfx":"TimeSetDownArrowGFX","tile":2,"count":1}],"frames_per_entry":1})
    );
    take!(&["call .ClearScreen"], "clear_screen", json!({}));
    take!(&["call WaitBGMap"], "wait_bg_map", json!({"frames":4}));
    take!(
        &["call RotateFourPalettesRight"],
        "rotate_palettes_right",
        json!({"steps":4,"frames_per_step":8})
    );
    let joy = Source::read(vendor, "home/joypad.asm")?;
    joy.operation(
        "JoyTextDelay",
        &["ld a, 15", "ld [wTextDelayFrames], a"],
        "source_repeat",
        json!({}),
    )?;
    joy.operation(
        "JoyTextDelay",
        &["ld a, 5", "ld [wTextDelayFrames], a"],
        "source_repeat",
        json!({}),
    )?;
    let fade = Source::read(vendor, "home/fade.asm")?;
    for label in ["RotateFourPalettesLeft", "RotateFourPalettesRight"] {
        ensure!(
            fade.body(label)?.iter().any(|(_, line)| line == "ld b, 4"),
            "source clock palette count"
        );
    }
    for label in ["RotatePalettesLeft", "RotatePalettesRight"] {
        fade.operation(
            label,
            &["ld c, 8", "call DelayFrames"],
            "source_palette_wait",
            json!({}),
        )?;
    }
    Source::read(vendor, "home/tilemap.asm")?.operation(
        "WaitBGMap",
        &["ld c, 4", "call DelayFrames"],
        "source_redraw_wait",
        json!({}),
    )?;
    for (label, maximum) in [("SetHour", 23), ("SetMinutes", 59)] {
        source.operation(
            label,
            &[&format!("cp {maximum}")],
            "source_clock_range",
            json!({}),
        )?;
        source.operation(
            label,
            &["call DelayFrame", "and a", "ret"],
            "source_clock_idle",
            json!({}),
        )?;
    }
    source.operation(
        "InitClock",
        &["ld a, 10", "ld [wInitHourBuffer], a"],
        "source_default_hour",
        json!({}),
    )?;
    source.operation(
        "InitClock",
        &[
            "ld hl, wTimeSetBuffer",
            "ld bc, wTimeSetBufferEnd - wTimeSetBuffer",
            "xor a",
            "call ByteFill",
        ],
        "source_default_minute",
        json!({}),
    )?;
    source.operation(
        "InitClock",
        &[
            "ld c, 10",
            "call DelayFrames",
            ".SetHourLoop:",
            "call JoyTextDelay",
            "call SetHour",
        ],
        "source_hour_selection_delay",
        json!({}),
    )?;
    source.operation(
        "InitClock",
        &[
            "ld c, 10",
            "call DelayFrames",
            ".SetMinutesLoop:",
            "call JoyTextDelay",
            "call SetMinutes",
        ],
        "source_minute_selection_delay",
        json!({}),
    )?;
    source.operation(
        "OakText_ResponseToSetTime",
        &[
            "cp MORN_HOUR",
            "jr c, .nite",
            "cp DAY_HOUR + 1",
            "jr c, .morn",
            "cp NITE_HOUR",
            "jr c, .day",
        ],
        "source_clock_response_period",
        json!({}),
    )?;
    let ids = [
        "OakTimeWokeUpText",
        "OakTimeWhatTimeIsItText",
        "OakTimeWhatHoursText",
        "OakTimeHoursQuestionMarkText",
        "OakTimeHowManyMinutesText",
        "OakTimeWhoaMinutesText",
        "OakTimeMinutesQuestionMarkText",
        "OakTimeOversleptText",
        "OakTimeYikesText",
        "OakTimeSoDarkText",
    ];
    let texts = source_texts(vendor, &ids)?;
    Ok((
        json!({"default_hour":10,"default_minute":0,"hour_range":[0,23],"minute_range":[0,59],"selection_delay_frames":10,"startup_delay_frames":8,"startup_sequence":startup,"selection_directions":["up","down"],"held_direction_repeat":{"h_in_menu":1,"press_counter":15,"restart_counter":5,"redraw_wait_frames":4,"idle_wait_frames":1,"first_repeat_frames":10,"later_repeat_frames":5},"text_entries":ids,"response_text_by_period":{"morn":"OakTimeOversleptText","day":"OakTimeYikesText","nite":"OakTimeSoDarkText"}}),
        texts,
    ))
}

fn source_texts(vendor: &Path, ids: &[&str]) -> Result<Vec<Value>> {
    let source = Source::read(vendor, "data/text/common_1.asm")?;
    let mut texts = Vec::new();
    for id in ids {
        let body = source.body(&format!("_{id}"))?;
        let mut commands = Vec::new();
        for (line, code) in &body[1..] {
            let (command, operands) = code.split_once(' ').unwrap_or((code, ""));
            ensure!(
                matches!(
                    command,
                    "text"
                        | "line"
                        | "cont"
                        | "para"
                        | "prompt"
                        | "done"
                        | "text_end"
                        | "text_promptbutton"
                ),
                "unsupported clock text {code}"
            );
            let args = super::super::source_syntax::source_arguments(operands)?;
            commands.push(json!({"command":command,"args":args,"source_span":{"file":source.file,"start_line":line,"end_line":line}}));
            if matches!(command, "prompt" | "done" | "text_end") {
                break;
            }
        }
        ensure!(!commands.is_empty(), "empty clock source text");
        texts.push(
            json!({"id":id,"source_span":source.span(&format!("_{id}"))?,"commands":commands}),
        );
    }
    Ok(texts)
}

pub(super) fn naming(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let names = Source::read(vendor, "data/player_names.asm")?;
    let mut choices = BTreeMap::new();
    for (gender, header, array) in [
        ("male", "ChrisNameMenuHeader", "MalePlayerNameArray"),
        ("female", "KrisNameMenuHeader", "FemalePlayerNameArray"),
    ] {
        names.operation(
            header,
            &["menu_coords 0, 0, 10, TEXTBOX_Y - 1"],
            "source_naming_menu",
            json!({}),
        )?;
        names.operation(
            header,
            &[
                "db STATICMENU_CURSOR | STATICMENU_PLACE_TITLE | STATICMENU_DISABLE_B",
                "db 5",
                "db \"NEW NAME@\"",
            ],
            "source_naming_choices",
            json!({}),
        )?;
        names.operation(
            array,
            &["db 2", "db \" NAME @\""],
            "source_naming_title",
            json!({}),
        )?;
        let body = names.body(array)?;
        let mut values = vec!["NEW NAME".to_owned()];
        for (_, line) in &body[1..] {
            let Some(value) = line.strip_prefix("db \"") else {
                continue;
            };
            let value = value
                .strip_suffix("@\"")
                .context("source preset name string")?;
            if value == " NAME " {
                continue;
            }
            values.push(value.to_owned());
        }
        ensure!(values.len() == 5, "authored preset name count");
        choices.insert(gender, values);
    }
    let gfx = Source::read(vendor, "engine/gfx/player_gfx.asm")?;
    gfx.operation(
        "MovePlayerPic",
        &["ld c, $8"],
        "source_picture_steps",
        json!({}),
    )?;
    gfx.operation(
        "MovePlayerPic",
        &["call WaitBGMap", "call DelayFrame"],
        "source_picture_cadence",
        json!({}),
    )?;
    gfx.operation(
        "DrawIntroPlayerPic",
        &["hlcoord 6, 4", "lb bc, 7, 7", "predef PlaceGraphic"],
        "source_picture_geometry",
        json!({}),
    )?;
    let menu = Source::read(vendor, "engine/menus/intro_menu.asm")?;
    menu.operation(
        "NamePlayer",
        &[
            "call RotateThreePalettesRight",
            "call ClearTilemap",
            "call LoadFontsExtra",
            "call WaitBGMap",
        ],
        "source_custom_name_return",
        json!({}),
    )?;
    let fade = Source::read(vendor, "home/fade.asm")?;
    for label in ["RotateThreePalettesRight", "RotateThreePalettesLeft"] {
        ensure!(
            fade.body(label)?.iter().any(|(_, line)| line == "ld b, 3"),
            "source naming palette count"
        );
    }
    let bottom = constants
        .get("TEXTBOX_Y")
        .context("source textbox coordinate")?
        - 1;
    Ok(
        json!({"preset_choices":choices,"preset_menu":{"flags":["STATICMENU_CURSOR","STATICMENU_PLACE_TITLE","STATICMENU_DISABLE_B"],"coordinates":{"left":0,"top":0,"right":10,"bottom":bottom},"default_option":1,"title":" NAME ","title_indent":2},"picture_motion":{"right_tiles":7,"left_tiles":7,"steps":8,"bg_map_wait_frames":4,"delay_frames":1},"custom_return":{"clear_tilemap":true,"fade_out":{"steps":3,"frames_per_step":8},"fade_in":{"steps":3,"frames_per_step":8},"bg_map_wait_frames":4,"player_picture":{"species_value":0,"palette_layout":"SCGB_TRAINER_OR_MON_FRONTPIC_PALS","coordinate":[6,4],"dimensions":[7,7]}},"source_span":menu.span("NamePlayer")?}),
    )
}
