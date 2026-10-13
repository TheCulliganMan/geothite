//! Entry and exit contracts of the authored Crystal intro, including helpers.
use super::Source;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn entry_cleanup(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let intro = Source::read(vendor, "engine/movie/intro.asm")?;
    let tilemap = Source::read(vendor, "home/tilemap.asm")?;
    let sprites = Source::read(vendor, "home/clear_sprites.asm")?;
    let text = Source::read(vendor, "home/text.asm")?;
    let mut entry = Vec::new();
    for (register, slot) in [("rWBK", 0), ("hInMenu", 1), ("hVBlank", 2)] {
        entry.push(intro.operation("CrystalIntro",&[&format!("ldh a, [{register}]"),"push af"],"save_memory_byte",json!({"source":register,"storage":{"kind":"cpu_stack","register_pair":"af","stack_slot":slot},"restore_required":true}))?);
        if slot == 0 {
            entry.push(intro.operation(
                "CrystalIntro",
                &["ld a, BANK(wGBCPalettes)", "ldh [rWBK], a"],
                "write_memory_byte",
                json!({"target":"rWBK","value":"BANK(wGBCPalettes)"}),
            )?);
        }
    }
    for (sequence, target, value) in [
        (
            vec!["assert VBLANK_NORMAL == 0", "xor a", "ldh [hVBlank], a"],
            "hVBlank",
            0,
        ),
        (vec!["ld a, TRUE", "ldh [hInMenu], a"], "hInMenu", 1),
        (vec!["xor a", "ldh [hMapAnims], a"], "hMapAnims", 0),
        (
            vec!["ldh [hMapAnims], a", "ld [wJumptableIndex], a"],
            "wJumptableIndex",
            0,
        ),
    ] {
        entry.push(intro.operation(
            "CrystalIntro",
            &sequence,
            "write_memory_byte",
            json!({"target":target,"value":value}),
        )?);
    }
    intro.operation(
        "CrystalIntro",
        &["push af", "call .InitRAMAddrs"],
        "source_entry_initialization",
        json!({}),
    )?;
    intro.operation(
        "CrystalIntro",
        &[
            ".done",
            "call ClearBGPalettes",
            "call ClearSprites",
            "call ClearTilemap",
        ],
        "source_cleanup_helpers",
        json!({}),
    )?;
    tilemap.operation(
        "ClearBGPalettes",
        &["call ClearPalettes"],
        "source_palette_clear",
        json!({}),
    )?;
    tilemap.operation(
        "WaitBGMap",
        &[
            "ld a, 1",
            "ldh [hBGMapMode], a",
            "ld c, 4",
            "call DelayFrames",
            "ret",
        ],
        "source_bg_map_wait",
        json!({}),
    )?;
    let mut cleanup = Vec::new();
    for register in ["rBGP", "rOBP0", "rOBP1"] {
        cleanup.push(tilemap.operation(
            "ClearPalettes",
            &[&format!("ldh [{register}], a")],
            "write_memory_byte",
            json!({"target":register,"value":0,"condition":{"source":"hCGB","predicate":"zero"}}),
        )?);
    }
    cleanup.push(tilemap.operation("ClearPalettes",&["ld hl, wBGPals2","ld bc, 16 palettes","ld a, $ff","call ByteFill"],"fill_memory",json!({"target":"wBGPals2","byte_count":128,"value":255,"condition":{"source":"hCGB","predicate":"nonzero"}}))?);
    cleanup.push(tilemap.operation("ClearPalettes",&["ld a, TRUE","ldh [hCGBPalUpdate], a"],"palette_transfer_request",json!({"target":"hCGBPalUpdate","value":1,"condition":{"source":"hCGB","predicate":"nonzero"}}))?);
    let mut wait = || -> Result<()> {
        cleanup.push(tilemap.operation(
            "WaitBGMap",
            &["ld a, 1", "ldh [hBGMapMode], a"],
            "write_memory_byte",
            json!({"target":"hBGMapMode","value":1}),
        )?);
        cleanup.push(tilemap.operation(
            "WaitBGMap",
            &["ld c, 4", "call DelayFrames"],
            "wait_frames",
            json!({"frames":4}),
        )?);
        Ok(())
    };
    wait()?;
    cleanup.push(sprites.operation(
        "ClearSprites",
        &[
            "ld hl, wShadowOAM",
            "ld b, wShadowOAMEnd - wShadowOAM",
            "xor a",
        ],
        "fill_memory",
        json!({"target":"wShadowOAM","byte_count":160,"value":0}),
    )?);
    sprites.operation(
        "ClearSprites",
        &[".loop", "ld [hli], a", "dec b", "jr nz, .loop", "ret"],
        "source_oam_clear_loop",
        json!({}),
    )?;
    let charmap = std::fs::read_to_string(vendor.join("constants/charmap.asm"))?;
    ensure!(
        charmap
            .lines()
            .map(super::super::source_syntax::code_part)
            .any(|line| super::normalize_spacing(line) == "charmap \" \", $7f"),
        "source blank tile mapping changed"
    );
    let screen_area = constants
        .get("SCREEN_AREA")
        .copied()
        .context("screen area")?;
    cleanup.push(text.operation(
        "ClearTilemap",
        &[
            "hlcoord 0, 0",
            "ld a, ' '",
            "ld bc, wTilemapEnd - wTilemap",
            "call ByteFill",
        ],
        "fill_memory",
        json!({"target":"wTilemap","byte_count":screen_area,"value":127}),
    )?);
    cleanup.push(tilemap.operation(
        "WaitBGMap",
        &["ld a, 1", "ldh [hBGMapMode], a"],
        "write_memory_byte",
        json!({"target":"hBGMapMode","value":1}),
    )?);
    cleanup.push(tilemap.operation(
        "WaitBGMap",
        &["ld c, 4", "call DelayFrames"],
        "wait_frames",
        json!({"frames":4}),
    )?);
    for (sequence, target, value) in [
        (vec!["xor a", "ldh [hSCX], a"], "hSCX", 0),
        (vec!["ldh [hSCX], a", "ldh [hSCY], a"], "hSCY", 0),
        (vec!["ld a, 7", "ldh [hWX], a"], "hWX", 7),
        (
            vec!["ld a, SCREEN_HEIGHT_PX", "ldh [hWY], a"],
            "hWY",
            *constants
                .get("SCREEN_HEIGHT_PX")
                .context("screen height pixels")?,
        ),
    ] {
        cleanup.push(intro.operation(
            "CrystalIntro",
            &sequence,
            "write_memory_byte",
            json!({"target":target,"value":value}),
        )?);
    }
    for (register, slot) in [("hVBlank", 2), ("hInMenu", 1), ("rWBK", 0)] {
        cleanup.push(intro.operation("CrystalIntro",&["pop af",&format!("ldh [{register}], a")],"restore_memory_byte",json!({"target":register,"storage":{"kind":"cpu_stack","register_pair":"af","stack_slot":slot}}))?);
    }
    cleanup.push(intro.operation(
        "CrystalIntro",
        &["ldh [rWBK], a", "ret"],
        "return",
        json!({}),
    )?);
    let cancel = intro.operation(
        "CrystalIntro",
        &["ld de, MUSIC_NONE", "call PlayMusic"],
        "stop_audio",
        json!({"audio":"MUSIC_NONE"}),
    )?;
    Ok(
        json!({"entry_init":{"id":"entry_init","source_span":intro.span("CrystalIntro")?,"operations":entry},"cleanup":{"id":"cleanup","source_span":intro.span("CrystalIntro")?,"operations":cleanup},"button_cancel":{"id":"button_cancel","source_span":cancel["source_span"],"operations":[cancel]},"implementation_source_spans":[tilemap.span("ClearBGPalettes")?,tilemap.span("ClearPalettes")?,tilemap.span("WaitBGMap")?,sprites.span("ClearSprites")?,text.span("ClearTilemap")?]}),
    )
}
