//! Bind the existing credits consumer to the pinned source in external packs.
use super::Certificate;
use anyhow::{Context, Result, ensure};
use crystal_assets::RuntimePresentationSubprogram;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn lower(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let credits = Certificate::read(vendor, "engine/movie/credits.asm")?;
    let palettes = Certificate::read(vendor, "home/tilemap.asm")?;
    bind(&credits, &palettes, constants, |path| {
        std::fs::read_to_string(vendor.join(path))
            .with_context(|| format!("read credits source {path}"))
    })
}

fn bind(
    credits: &Certificate,
    palettes: &Certificate,
    constants: &BTreeMap<String, i64>,
    read: impl Fn(&str) -> Result<String>,
) -> Result<Value> {
    let entry = credits.sequence(&["Credits::", "bit STATUSFLAGS_HALL_OF_FAME_F, b"])?;
    let menu = credits.sequence(&["ld a, TRUE", "ldh [hInMenu], a", "xor a"])?;
    ensure!(
        constants.get("TRUE") == Some(&1),
        "credits TRUE must be one"
    );
    ensure!(
        credits
            .lines
            .iter()
            .filter(|(_, code)| code.contains("[hInMenu]"))
            .count()
            == 1,
        "credits unexpectedly reads or restores hInMenu"
    );
    let skip = credits.sequence(&[
        "ld hl, wCreditsPos",
        "ld a, [hli]",
        "cp $d",
        "jr nc, .okay",
        "ld a, [hli]",
        "and a",
        "ret z",
    ])?;
    let fade = credits.sequence(&[
        "ld a, 32",
        "ld [wMusicFade], a",
        "ld a, LOW(MUSIC_POST_CREDITS)",
        "ld [wMusicFadeID], a",
        "ld a, HIGH(MUSIC_POST_CREDITS)",
        "ld [wMusicFadeID + 1], a",
        "ret",
    ])?;
    let exit = credits.sequence(&[
        ".exit_credits",
        "call ClearBGPalettes",
        "xor a",
        "ldh [hLCDCPointer], a",
        "ldh [hBGMapAddress], a",
        "pop af",
        "ldh [hVBlank], a",
        "pop af",
        "ldh [rWBK], a",
        "ret",
    ])?;
    let clear = palettes.sequence(&[
        "ClearBGPalettes::",
        "call ClearPalettes",
        "WaitBGMap::",
        "ld a, 1",
        "ldh [hBGMapMode], a",
        "ld c, 4",
        "call DelayFrames",
        "ret",
    ])?;
    let fill = palettes.sequence(&[
        "ld hl, wBGPals2",
        "ld bc, 16 palettes",
        "ld a, $ff",
        "call ByteFill",
    ])?;
    let operation = credits.operation(
        &[
            "INCLUDE \"data/credits_script.asm\"",
            "INCLUDE \"data/credits_strings.asm\"",
        ],
        "credits_source_data",
        json!({
            "script_source":read("data/credits_script.asm")?,
            "constants_source":read("constants/credits_constants.asm")?,
            "strings_source":read("data/credits_strings.asm")?,
            "skip_position_threshold":13,
            "end_music_fade":{"target":"MUSIC_POST_CREDITS","rate":32},
            "exit_clear":{"frames":4,"cgb_palette_fill_byte":255},
            "menu_state":{"register":"hInMenu","value":1,"restored_on_return":false}
        }),
    )?;
    let mut subprogram = serde_json::to_value(RuntimePresentationSubprogram::default())?;
    subprogram["id"] = json!("credits");
    subprogram["source_entry"] = json!("Credits");
    subprogram["accepted_call_forms"] = json!(["farcall"]);
    subprogram["source_span"] = serde_json::to_value(&entry)?;
    subprogram["phases"] = json!([{"id":"credits","source_span":entry,"operations":[operation]}]);
    subprogram["implementation_source_spans"] = json!([menu, skip, fade, exit, clear, fill]);
    Ok(subprogram)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires pinned external source via CRYSTAL_EXPORT_WORKSPACE"]
    fn external_credits_lowering_preserves_sources_and_rejects_changed_control() {
        let root = std::path::PathBuf::from(
            std::env::var_os("CRYSTAL_EXPORT_WORKSPACE").expect("set CRYSTAL_EXPORT_WORKSPACE"),
        );
        let vendor = root.join("vendor/pokecrystal");
        let globals: Value = serde_json::from_slice(
            &std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json"))
                .unwrap(),
        )
        .unwrap();
        let constants = serde_json::from_value(globals["global"].clone()).unwrap();
        let value = lower(&vendor, &constants).unwrap();
        let subprogram: RuntimePresentationSubprogram = serde_json::from_value(value).unwrap();
        let fields = &subprogram.phases[0].operations[0].fields;
        for (field, path) in [
            ("script_source", "data/credits_script.asm"),
            ("strings_source", "data/credits_strings.asm"),
            ("constants_source", "constants/credits_constants.asm"),
        ] {
            assert_eq!(
                fields[field].as_str().unwrap(),
                std::fs::read_to_string(vendor.join(path)).unwrap()
            );
        }
        let palettes = Certificate::read(&vendor, "home/tilemap.asm").unwrap();
        for (from, to) in [
            ("cp $d", "cp $e"),
            ("ld a, 32", "ld a, 33"),
            ("ldh [hInMenu], a", "ldh [hOther], a"),
        ] {
            let mut credits = Certificate::read(&vendor, "engine/movie/credits.asm").unwrap();
            let line = credits
                .lines
                .iter_mut()
                .find(|(_, code)| code == from)
                .unwrap();
            line.1 = to.into();
            assert!(
                bind(&credits, &palettes, &constants, |_| Ok("unused".into())).is_err(),
                "changed credits instruction must reject stale metadata: {from}"
            );
        }
    }
}
