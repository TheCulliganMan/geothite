//! Reliable native fishing-rod handoffs throughout the Minnesota campaign.

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

const MANIFEST: &str = "minnesota-fishing-rods-v2";
const RODS: [&str; 3] = ["OLD_ROD", "GOOD_ROD", "SUPER_ROD"];

/// Adds three native fishing-guru houses on Driftless Trail, River Valley and
/// Pine Country Road, with free Old, Good and Super Rod gifts. Possession checks
/// are independent, and failed grants remain retryable. Defeated fishermen and
/// bank anglers advertise the houses. Touched fishermen use the active regional
/// roster; battles, defeat flags, levels, services and audio remain intact.
///
/// # Errors
/// Rejects occupied houses, unsafe or overlapping facade/door placements,
/// missing native assets or roster bindings, malformed scripts, duplicate
/// installation or an invalid resulting pack. The parent is unchanged.
pub fn build_minnesota_fishing_rods(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "fishing rod handoffs already installed"
    );
    for rod in RODS {
        let item = base.data.items.get(rod).context("native fishing rod")?;
        ensure!(
            item.pocket == "KEY_ITEM" && item.field_usable && !item.consumable,
            "rod must retain native reusable key-item behavior: {rod}"
        );
    }
    let mut pack = base.clone();
    crate::minnesota_fishing_houses::install(&mut pack)?;
    crate::minnesota_fishing_hints::install(&mut pack)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

pub(crate) fn add_handoff(
    map: &mut crate::MapModule,
    programs: &mut std::collections::BTreeMap<String, Value>,
    label: &str,
    original: &str,
    tier: usize,
) -> Result<()> {
    ensure!(
        map.scripts.contains_key(original),
        "missing original service {original}"
    );
    ensure!(
        !map.scripts.contains_key(label),
        "occupied handoff label {label}"
    );
    let done = format!("{label}Continue");
    let full = format!("{label}Full");
    let text = format!("{label}Text");
    let full_text = format!("{label}FullText");
    let c = authoring::command;
    let mut commands = vec![
        c("faceplayer", &[]),
        c("checkitem", &[RODS[tier]]),
        c("iftrue", &[&done]),
    ];
    commands.extend([
        c("opentext", &[]),
        c("verbosegiveitem", &[RODS[tier], "1"]),
        c("iffalse", &[&full]),
        c("writetext", &[&text]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("sjump", &[&done]),
    ]);
    programs.insert(label.into(), Value::Array(commands));
    programs.insert(done, serde_json::json!([c("sjump", &[original])]));
    programs.insert(
        full,
        serde_json::json!([
            c("writetext", &[&full_text]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("sjump", &[original]),
        ]),
    );
    authoring::text(
        map,
        &text,
        &[
            match tier {
                0 => "Take this OLD ROD!",
                1 => "Take this GOOD ROD!",
                _ => "Take a SUPER ROD!",
            },
            "Our water is full",
            "of wild POKEMON.",
            "Face the water.",
            "Open PACK, then",
            "KEY ITEMS. Select",
            "the ROD and USE.",
            match tier {
                0 => "For a GOOD ROD,",
                1 => "For a SUPER ROD,",
                _ => "Different RODS",
            },
            match tier {
                0 => "visit the GURU on",
                1 => "visit the GURU on",
                _ => "catch different",
            },
            match tier {
                0 => "RIVER VALLEY.",
                1 => "PINE COUNTRY RD.",
                _ => "kinds of POKEMON.",
            },
        ],
    );
    authoring::text(
        map,
        &full_text,
        &[
            "Room for a ROD?",
            "Make space in your",
            "KEY ITEMS, then",
            "talk to me again.",
        ],
    );
    for text_label in [&text, &full_text] {
        let body = map.scripts.remove(text_label).context("new handoff text")?;
        programs.insert(text_label.clone(), body);
    }
    Ok(())
}
