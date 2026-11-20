//! Independent partner offers at Winona and a recoverable cache in Lamp Gallery.

#[path = "northstar_lamp_cache_notice.rs"]
mod lamp;

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::grand_marais_modpack::authoring::command;
use crate::northstar_rival_arc::put;
use crate::{CompiledGamePack, MapModule, authored_dialogue, authored_town_publication};

const ROOM: &str = "MnWinonaFieldStation";
const ENTRY: &str = "MnEarlyPartners";
const DELIVERED: &str = "EVENT_MN_DELIVERED_ROCHESTER_PARCEL";
const ORIGINAL_OFFERS: &str = "MnEarlyPartnerPOUCHID";
const PROMPT: &str = "MnResearchChoiceText";
const PARENT_HASHES: [&str; 2] = [
    "3479802b5c0190f797db0d580d59537dabb703fb8c2057f142936d5df74ee02b",
    "5556a4bb8e73679ea21e9f156453f3ab554794d72f090957c2ae3c973e1e4c46",
];

struct PartnerMenu {
    original_choice: &'static str,
    label: &'static str,
    species: [&'static str; 2],
}

const MENUS: [PartnerMenu; 3] = [
    PartnerMenu {
        original_choice: "EVENT_MN_WINONA_CHIKORITA",
        label: "MnResearchChoiceAfterPouchid",
        species: ["KINDLIT", "BIRCHIT"],
    },
    PartnerMenu {
        original_choice: "EVENT_MN_WINONA_CYNDAQUIL",
        label: "MnResearchChoiceAfterKindlit",
        species: ["POUCHID", "BIRCHIT"],
    },
    PartnerMenu {
        original_choice: "EVENT_MN_WINONA_TOTODILE",
        label: "MnResearchChoiceAfterBirchit",
        species: ["POUCHID", "KINDLIT"],
    },
];

fn ensure_new_label(pack: &CompiledGamePack, room: &MapModule, label: &str) -> Result<()> {
    ensure!(
        !room.scripts.contains_key(label) && !pack.data.map_scripts.contains_key(label),
        "occupied research partner label {label}"
    );
    Ok(())
}

fn install_menu(pack: &CompiledGamePack, room: &mut MapModule, menu: &PartnerMenu) -> Result<()> {
    let header = format!("{}Header", menu.label);
    let data = format!("{}MenuData", menu.label);
    for label in [menu.label, &header, &data] {
        ensure_new_label(pack, room, label)?;
    }
    put(
        room,
        &header,
        vec![
            command("db", &["MENU_BACKUP_TILES"]),
            command("menu_coords", &["0", "0", "10", "7"]),
            command("dw", &[&data]),
            command("db", &["1"]),
        ],
    );
    let mut rows = vec![command("db", &["STATICMENU_CURSOR"]), command("db", &["3"])];
    for label in menu.species.into_iter().chain(["CANCEL"]) {
        rows.push(command("db", &[&format!("\"{label}@\"")]));
    }
    put(room, &data, rows);

    // Once either partner has been claimed, the original guarded offer scan
    // finds the one still waiting or gives the aide's original farewell.
    let mut program = Vec::new();
    for species in menu.species {
        program.push(command(
            "checkevent",
            &[&format!("EVENT_MN_{species}_RESEARCH_RECEIVED")],
        ));
        program.push(command("iftrue", &[ORIGINAL_OFFERS]));
    }
    program.extend([
        command("opentext", &[]),
        command("writetext", &[PROMPT]),
        command("loadmenu", &[&header]),
        command("verticalmenu", &[]),
        command("closewindow", &[]),
        command("closetext", &[]),
    ]);
    for (index, species) in menu.species.iter().enumerate() {
        program.push(command(
            "ifequal",
            &[
                &(index + 1).to_string(),
                &format!("MnEarlyPartner{species}"),
            ],
        ));
    }
    // Both B (zero) and the explicit third row return control without a gift.
    program.push(command("end", &[]));
    put(room, menu.label, program);
    Ok(())
}

fn install_winona_choices(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context(ROOM)?.clone();
    let mut entry = vec![
        command("checkevent", &[DELIVERED]),
        command("iffalse", &["MnEarlyPartnerWait"]),
    ];
    let legacy_scan = command("sjump", &[ORIGINAL_OFFERS]);
    ensure!(
        room.scripts.get(ENTRY) == Some(&json!([entry[0], entry[1], legacy_scan])),
        "unexpected Winona partner availability check"
    );
    ensure_new_label(pack, &room, PROMPT)?;
    authored_dialogue::install_canonical(&mut room, PROMPT)?;
    for menu in &MENUS {
        install_menu(pack, &mut room, menu)?;
        entry.push(command("checkevent", &[menu.original_choice]));
        entry.push(command("iftrue", &[menu.label]));
    }
    entry.push(legacy_scan);
    put(&mut room, ENTRY, entry);
    room.script_control_commands = crate::parse_script_control_commands(ROOM, &room.scripts)?;
    room.script_flag_commands = crate::parse_script_flag_commands(ROOM, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(ROOM, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(ROOM, &room.scripts)?;
    let runtime = crate::parse_script_runtime_commands(ROOM, &room.scripts)?;
    room.script_runtime_commands.extend(
        runtime
            .into_iter()
            .filter(|row| MENUS.iter().any(|menu| row.source_script == menu.label)),
    );
    room.script_menu_definitions = crate::parse_script_menu_definitions(ROOM, &room.scripts)?;
    room.script_vertical_menus =
        crate::parse_script_vertical_menus(ROOM, &room.scripts, &room.script_menu_definitions)?;
    authored_town_publication::replace(pack, room)
}

/// Lets the player choose either optional Winona partner and retry a full-pocket Lamp cache.
///
/// # Errors
/// Returns an error for a different parent archive, unexpected installed scripts,
/// a colliding label, or a revision that fails runtime verification.
pub fn revise_research_choices(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        pack.report.manifests.last().map(String::as_str) == Some("northstar-marais-supplies-v1")
            && PARENT_HASHES.contains(&pack.identity.content_hash.as_str()),
        "research choices require a matching Marais Supplies archive"
    );
    install_winona_choices(&mut pack)?;
    lamp::apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-research-choices-v1".into());
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
