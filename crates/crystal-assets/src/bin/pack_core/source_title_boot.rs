//! Source-certified boot, save continuation and guarded save-management entries.
use super::{Cursor, Source};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn blocks(
    vendor: &Path,
    constants: &BTreeMap<String, i64>,
) -> Result<BTreeMap<String, Value>> {
    let menu = Source::read(vendor, "engine/menus/intro_menu.asm")?;
    let mut blocks = BTreeMap::new();
    let mut cursor = Cursor::new(&menu, "GameInit")?;
    let mut ops = Vec::new();
    macro_rules! take {
        ($seq:expr,$op:expr,$fields:expr) => {
            ops.push(cursor.take($seq, $op, $fields)?);
        };
    }
    take!(
        &["farcall TryLoadSaveData"],
        "try_load_save_data",
        json!({"routine":"TryLoadSaveData"})
    );
    take!(&["call ClearWindowData"], "clear_window_data", json!({}));
    take!(&["call ClearBGPalettes"], "clear_bg_palettes", json!({}));
    take!(&["call ClearTilemap"], "clear_tilemap", json!({}));
    take!(
        &[
            "ld a, HIGH(vBGMap0)",
            "ldh [hBGMapAddress + 1], a",
            "xor a",
            "ldh [hBGMapAddress], a"
        ],
        "set_bg_map_address",
        json!({"address":"vBGMap0"})
    );
    for register in ["hJoyDown", "hSCX", "hSCY"] {
        take!(
            &[&format!("ldh [{register}], a")],
            "write_memory_byte",
            json!({"target":register,"value":0})
        );
    }
    take!(
        &["ld a, $90", "ldh [hWY], a"],
        "write_memory_byte",
        json!({"target":"hWY","value":144})
    );
    take!(&["call WaitBGMap"], "wait_frames", json!({"frames":4}));
    take!(
        &["jp IntroSequence"],
        "jump",
        json!({"target":"IntroSequence"})
    );
    cursor.finish()?;
    blocks.insert(
        "GameInit".into(),
        json!({"source_span":menu.span("GameInit")?,"operations":ops}),
    );
    let mut cursor = Cursor::new(&menu, "NewGame")?;
    let mut ops = Vec::new();
    macro_rules! take {
        ($seq:expr,$op:expr,$fields:expr) => {
            ops.push(cursor.take($seq, $op, $fields)?);
        };
    }
    take!(
        &["xor a", "ld [wDebugFlags], a"],
        "write_memory_byte",
        json!({"target":"wDebugFlags","value":0})
    );
    for (routine, operation) in [
        ("ResetWRAM", "reset_world_memory"),
        ("NewGame_ClearTilemapEtc", "prepare_new_game_display"),
        ("PlayerProfileSetup", "setup_player_profile"),
    ] {
        take!(
            &[&format!("call {routine}")],
            operation,
            json!({"routine":routine})
        );
    }
    take!(
        &["call OakSpeech"],
        "call_subprogram",
        json!({"program":"oak_speech"})
    );
    take!(
        &["call InitializeWorld"],
        "initialize_world",
        json!({"routine":"InitializeWorld"})
    );
    for (token, target) in [
        ("LANDMARK_NEW_BARK_TOWN", "wPrevLandmark"),
        ("SPAWN_HOME", "wDefaultSpawnpoint"),
    ] {
        take!(
            &[&format!("ld a, {token}"), &format!("ld [{target}], a")],
            "write_memory_byte",
            json!({"target":target,"value":constants.get(token).with_context(||format!("source boot constant {token}"))?})
        );
    }
    take!(
        &["ld a, MAPSETUP_WARP", "ldh [hMapEntryMethod], a"],
        "write_memory_byte",
        json!({"target":"hMapEntryMethod","value":constants.get("MAPSETUP_WARP").context("source map setup")?})
    );
    take!(
        &["jp FinishContinueFunction"],
        "enter_overworld",
        json!({"routine":"FinishContinueFunction"})
    );
    cursor.finish()?;
    blocks.insert(
        "NewGame".into(),
        json!({"source_span":menu.span("NewGame")?,"operations":ops}),
    );
    let continuation=vec![
        menu.operation("Continue",&["farcall TryLoadSaveFile","jr c, .FailToLoad","farcall _LoadData"],"load_continue_save",json!({"save_routine":"TryLoadSaveFile","restore_routine":"_LoadData","failure_returns_to_menu":true}))?,
        menu.operation("Continue",&["call LoadStandardMenuHeader","call DisplaySaveInfoOnContinue","ld a, $1","ldh [hBGMapMode], a","ld c, 20","call DelayFrames"],"present_continue_summary",json!({"wait_frames":20,"bg_map_mode":1}))?,
        menu.operation("Continue",&["call ConfirmContinue","jr nc, .Check1Pass","call CloseWindow","jr .FailToLoad"],"confirm_continue",json!({"accept":"A","cancel":"B","cancel_returns_to_menu":true}))?,
        menu.operation("Continue",&["call Continue_CheckRTC_RestartClock","jr nc, .Check2Pass","call CloseWindow","jr .FailToLoad"],"check_continue_clock",json!({"routine":"Continue_CheckRTC_RestartClock","cancel_returns_to_menu":true}))?,
        menu.operation("Continue",&["ld a, $8","ld [wMusicFade], a","ld a, LOW(MUSIC_NONE)","ld [wMusicFadeID], a","ld a, HIGH(MUSIC_NONE)","ld [wMusicFadeID + 1], a"],"fade_music",json!({"audio":"MUSIC_NONE","rate":8}))?,
        menu.operation("Continue",&["call ClearBGPalettes","call Continue_MobileAdapterMenu","call CloseWindow","call ClearTilemap","ld c, 20","call DelayFrames"],"prepare_continue_display",json!({"wait_frames":20,"mobile_routine":"Continue_MobileAdapterMenu"}))?,
        menu.operation("Continue",&["farcall JumpRoamMons","farcall CopyMysteryGiftReceivedDecorationsToPC","farcall ClockContinue"],"restore_continue_world",json!({"roaming_routine":"JumpRoamMons","gift_routine":"CopyMysteryGiftReceivedDecorationsToPC","clock_routine":"ClockContinue"}))?,
        menu.operation("Continue",&["ld a, [wSpawnAfterChampion]","cp SPAWN_LANCE","jr z, .SpawnAfterE4","ld a, MAPSETUP_CONTINUE","ldh [hMapEntryMethod], a","jp FinishContinueFunction"],"enter_continued_overworld",json!({"champion_spawn":constants.get("SPAWN_LANCE").context("champion spawn")?,"champion_return_spawn":constants.get("SPAWN_NEW_BARK").context("post champion spawn")?,"map_entry_method":constants.get("MAPSETUP_CONTINUE").context("continue map setup")?,"routine":"FinishContinueFunction"}))?,
    ];
    blocks.insert(
        "Continue".into(),
        json!({"source_span":menu.span("Continue")?,"operations":continuation}),
    );
    // These exits are separate authored paths, not unconditional effects of
    // entering Continue. Keep them addressable for the production consumer.
    let failed = menu.operation("Continue", &[".FailToLoad:", "ret"], "return", json!({}))?;
    blocks.insert(
        "Continue.FailToLoad".into(),
        json!({"source_span":failed["source_span"],"operations":[failed]}),
    );
    let champion = menu.operation("Continue", &[".SpawnAfterE4:","ld a, SPAWN_NEW_BARK","ld [wDefaultSpawnpoint], a","call PostCreditsSpawn","jp FinishContinueFunction"], "enter_post_champion_overworld", json!({"spawn_identifier":constants.get("SPAWN_NEW_BARK").context("post champion spawn")?,"map_entry_method":constants.get("MAPSETUP_WARP").context("warp map setup")?,"clear_champion_marker":true}))?;
    blocks.insert(
        "Continue.SpawnAfterE4".into(),
        json!({"source_span":champion["source_span"],"operations":[champion]}),
    );
    for (file, entry, header, prompt, operation) in [
        (
            "engine/menus/delete_save.asm",
            "_DeleteSaveData",
            ".NoYesMenuHeader",
            ".ClearAllSaveDataText",
            "delete_save_data",
        ),
        (
            "engine/rtc/reset_password.asm",
            "_ResetClock",
            ".NoYes_MenuHeader",
            ".PasswordAskResetClockText",
            "reset_clock_with_password",
        ),
    ] {
        let source = Source::read(vendor, file)?;
        let body = source.body(entry)?;
        ensure!(
            body.iter().any(|(_, line)| line == "db \"NO@\"")
                && body.iter().any(|(_, line)| line == "db \"YES@\""),
            "source guarded menu choices"
        );
        source.operation(
            entry,
            &["ld a, [wMenuCursorY]", "cp 1", "ret z"],
            "source_no_choice_returns",
            json!({}),
        )?;
        if entry == "_DeleteSaveData" {
            source.operation(
                entry,
                &["farcall EmptyAllSRAMBanks", "ret"],
                "source_delete_save_effect",
                json!({}),
            )?;
        } else {
            source.operation(
                entry,
                &["call ClockResetPassword", "jr c, .wrongpassword"],
                "source_password_gate",
                json!({}),
            )?;
            source.operation(
                entry,
                &[
                    "ld a, RTC_RESET",
                    "ld [sRTCStatusFlags], a",
                    "call CloseSRAM",
                ],
                "source_reset_rtc_effect",
                json!({}),
            )?;
        }
        let operation=source.operation(entry,&["farcall BlankScreen","ld b, SCGB_DIPLOMA","call GetSGBLayout","call LoadStandardFont","call LoadFontsExtra","ld de, MUSIC_MAIN_MENU","call PlayMusic",&format!("ld hl, {prompt}"),"call PrintText",&format!("ld hl, {header}"),"call CopyMenuHeader","call VerticalMenu","ret c"],operation,json!({"music":"MUSIC_MAIN_MENU","text":prompt.trim_start_matches('.'),"menu":{"choices":["NO","YES"],"default_option":1,"coordinates":{"left":14,"top":7,"right":constants.get("SCREEN_WIDTH").context("screen width")?-1,"bottom":constants.get("TEXTBOX_Y").context("textbox y")?-1},"flags":["STATICMENU_CURSOR","STATICMENU_NO_TOP_SPACING"]},"cancel_returns":true}))?;
        blocks.insert(
            entry.into(),
            json!({"source_span":source.span(entry)?,"operations":[operation]}),
        );
    }
    let password_source = Source::read(vendor, "engine/rtc/reset_password.asm")?;
    let calculation = password_source.operation("ClockResetPassword", &["ld hl, sPlayerData + (wPlayerID - wPlayerData)","ld c, 2","call .ComponentFromNumber","ld hl, sPlayerData + (wPlayerName - wPlayerData)","ld c, NAME_LENGTH_JAPANESE - 1","call .ComponentFromString","ld hl, sPlayerData + (wMoney - wPlayerData)","ld c, 3","call .ComponentFromNumber"], "source_password_recipe", json!({}))?;
    let digits = password_source.operation("ClockResetPassword", &["ld hl, wStringBuffer2","ld bc, 5","xor a","call ByteFill","ld a, 4","ld [wStringBuffer2 + 5], a"], "source_password_digits", json!({}))?;
    let name_bytes = constants.get("NAME_LENGTH_JAPANESE").context("password source name length")? - 1;
    ensure!(name_bytes == 5, "authored password name byte count changed");
    blocks.get_mut("_ResetClock").context("reset clock block")?["operations"][0]["password"] = json!({"player_id_bytes":2,"player_name_bytes":name_bytes,"money_bytes":3,"digit_count":5,"initial_digits":[0,0,0,0,0],"initial_cursor":4,"rtc_reset_flag":constants.get("RTC_RESET").context("RTC_RESET")?,"recipe_source_span":calculation["source_span"],"digits_source_span":digits["source_span"]});
    Ok(blocks)
}

pub(super) fn texts(vendor: &Path) -> Result<Vec<Value>> {
    let source = Source::read(vendor, "data/text/common_3.asm")?;
    let mut texts = Vec::new();
    for id in [
        "ClearAllSaveDataText",
        "PasswordAskResetClockText",
        "PasswordAskResetText",
        "PasswordWrongText",
        "PasswordAskEnterText",
        "ClockTimeMayBeWrongText",
        "ClockSetWithControlPadText",
        "ClockIsThisOKText",
        "ClockHasResetText",
    ] {
        let body = source.body(&format!("_{id}"))?;
        let mut commands = Vec::new();
        for (line, code) in &body[1..] {
            let (command, operands) = code.split_once(' ').unwrap_or((code, ""));
            ensure!(
                matches!(
                    command,
                    "text" | "line" | "cont" | "para" | "prompt" | "done" | "text_end"
                ),
                "unsupported save menu text {code}"
            );
            commands.push(json!({"command":command,"args":super::super::source_syntax::source_arguments(operands)?,"source_span":{"file":source.file,"start_line":line,"end_line":line}}));
            if matches!(command, "done" | "prompt" | "text_end") {
                break;
            }
        }
        texts.push(
            json!({"id":id,"source_span":source.span(&format!("_{id}"))?,"commands":commands}),
        );
    }
    Ok(texts)
}
