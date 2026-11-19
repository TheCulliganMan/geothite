//! Make Castorock's three field observations recognizable in the River Pocket Cave.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::grand_marais_modpack::authoring::command;
use crate::minnesota_pokemon_pixel_art::bitplanes;
use crate::northstar_rival_arc::put;
use crate::{
    CompiledGamePack, MapModule, authored_dialogue, authored_town_publication, native_tile_png,
};

const ROOM: &str = "MnR03Cave1F";
const WALKER: &str = "MN_R03_CAVE_WALKER";
const RECIPES: &str = "data/tilesets/cave_metatiles.bin";
const SHEET: &str = "gfx/tilesets/cave.2bpp";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Marks {
    pool: [String; 16],
    pillar: [String; 16],
    bed: [String; 16],
}

fn mark_stones(pack: &mut CompiledGamePack, room: &mut MapModule) -> Result<()> {
    let marks: Marks = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/cave-returns-v01/caves-marks.json"
    ))?;
    let mut layout = pack
        .runtime_files
        .get(RECIPES)
        .context("cave blocks")?
        .clone();
    let graphics = pack.runtime_files.get_mut(SHEET).context("cave sheet")?;
    let bank = pack
        .data
        .tilesets
        .get_mut("cave")
        .context("cave palette and terrain")?;
    // This 96-tile single-bank sheet also serves Johto. Write only the original
    // blank physical slots, with no pre-existing block pointing to any of them.
    ensure!(
        graphics.len() == 96 * 16
            && layout.len().is_multiple_of(16)
            && bank.palette_map.len() >= 96
            && layout
                .iter()
                .all(|&g| g < 96 && bank.palette_map[usize::from(g)] & 8 == 0)
            && layout.iter().all(|g| !(0x50..0x5c).contains(g))
            && graphics[0x50 * 16..0x5c * 16].iter().all(|&v| v == 0),
        "occupied cave art or unexpected tile addressing"
    );
    for (label, x, y, before, first, rows) in [
        ("MnJawPool", 2_u16, 5_u16, 0x04_u16, 0x50_u8, marks.pool),
        ("MnJawPillar", 20, 4, 0x34, 0x54, marks.pillar),
        ("MnJawBed", 23, 13, 0x21, 0x58, marks.bed),
    ] {
        let place = usize::from(y / 2 * room.attributes.width + x / 2);
        ensure!(
            room.blocks.get(place) == Some(&before)
                && room.events.bg_events.iter().any(|e| e.x == x
                    && e.y == y
                    && e.script == label
                    && e.event_type == "BGEVENT_READ")
                && !room
                    .script_block_changes
                    .iter()
                    .any(|c| c.x / 2 == x / 2 && c.y / 2 == y / 2),
            "moved or scripted Castorock landmark {label}"
        );
        ensure!(
            rows.iter()
                .all(|row| row.len() == 16 && row.bytes().all(|b| (b'0'..=b'3').contains(&b))),
            "invalid Castorock artwork {label}"
        );
        let pixels: Vec<_> = rows
            .iter()
            .flat_map(|s| s.bytes().map(|b| b - b'0'))
            .collect();
        let start = usize::from(first) * 16;
        graphics[start..start + 64].copy_from_slice(&bitplanes(&pixels, 16, 16));
        let old = usize::from(before) * 16;
        let mut recipe: [u8; 16] = layout
            .get(old..old + 16)
            .context("original cave block")?
            .try_into()?;
        let offset = usize::from(8 * (y % 2) + 2 * (x % 2));
        for (slot, glyph) in [0, 1, 4, 5].into_iter().zip(first..first + 4) {
            let source = usize::from(recipe[offset + slot]);
            ensure!(bank.palette_map[source] == 5, "unexpected stone palette");
            bank.palette_map[usize::from(glyph)] = bank.palette_map[source];
            recipe[offset + slot] = glyph;
        }
        let next = u8::try_from(layout.len() / 16)?;
        let collision = bank
            .collision
            .get(&format!("{before:02x}"))
            .context("original stone terrain")?
            .clone();
        ensure!(
            bank.collision
                .insert(format!("{next:02x}"), collision)
                .is_none(),
            "occupied cave block"
        );
        layout.extend(recipe);
        room.blocks[place] = u16::from(next);
    }
    let png = native_tile_png::from_2bpp(graphics)?;
    pack.runtime_files
        .insert("gfx/tilesets/cave.png".into(), png);
    pack.runtime_files.insert(RECIPES.into(), layout);
    pack.runtime_files.insert(
        "data/tilesets/cave.json".into(),
        serde_json::to_vec(&bank.collision)?,
    );
    pack.runtime_files.insert(
        "data/tilesets/cave_palette_map.json".into(),
        serde_json::to_vec(&bank.palette_map)?,
    );
    Ok(())
}

fn guide(pack: &CompiledGamePack, room: &mut MapModule) -> Result<()> {
    ensure!(
        room.scripts.get(WALKER)
            == Some(&json!([command(
                "jumptextfaceplayer",
                &["MN_R03_CAVE_WALKERText"]
            )]))
            && room
                .objects
                .iter()
                .any(|o| o.script == WALKER && (o.x, o.y) == (5, 5))
            && !pack
                .data
                .map_scripts
                .keys()
                .any(|k| k.starts_with("MnJawGuide")),
        "unexpected River Pocket guide"
    );
    put(
        room,
        WALKER,
        vec![
            command("faceplayer", &[]),
            command("opentext", &[]),
            command("checkevent", &["EVENT_MN_CASTOROCK_CAUGHT"]),
            command("iftrue", &["MnJawGuideCaught"]),
            command("checkevent", &["EVENT_MN_CASTOROCK_TRACKS_READ"]),
            command("iftrue", &["MnJawGuideFound"]),
            command("checkevent", &["EVENT_MN_CASTOROCK_POOL"]),
            command("iffalse", &["MnJawGuidePool"]),
            command("checkevent", &["EVENT_MN_CASTOROCK_PILLAR"]),
            command("iffalse", &["MnJawGuidePillar"]),
            command("writetext", &["MnJawGuideBedText"]),
            command("sjump", &["MnJawGuideTravel"]),
        ],
    );
    for (stage, end) in [
        ("Pool", "Travel"),
        ("Pillar", "Travel"),
        ("Found", "Finish"),
        ("Caught", "Finish"),
    ] {
        let label = format!("MnJawGuide{stage}");
        put(
            room,
            &label,
            vec![
                command("writetext", &[&format!("{label}Text")]),
                command("sjump", &[&format!("MnJawGuide{end}")]),
            ],
        );
    }
    put(
        room,
        "MnJawGuideTravel",
        vec![
            command("waitbutton", &[]),
            command("checkitem", &["HM_STRENGTH"]),
            command("iftrue", &["MnJawGuideClose"]),
            command("writetext", &["MnJawGuideStrengthText"]),
            command("sjump", &["MnJawGuideFinish"]),
        ],
    );
    put(
        room,
        "MnJawGuideFinish",
        vec![
            command("waitbutton", &[]),
            command("sjump", &["MnJawGuideClose"]),
        ],
    );
    put(
        room,
        "MnJawGuideClose",
        vec![command("closetext", &[]), command("end", &[])],
    );
    for stem in ["Pool", "Pillar", "Bed", "Found", "Caught", "Strength"] {
        authored_dialogue::install_canonical(room, &format!("MnJawGuide{stem}Text"))?;
    }
    room.script_flag_commands = crate::parse_script_flag_commands(ROOM, &room.scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(ROOM, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(ROOM, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(ROOM, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(ROOM, &room.scripts)?;
    (room.script_item_checks, room.script_item_takes) =
        crate::parse_script_item_accesses(ROOM, &room.scripts)?;
    Ok(())
}

/// Marks three original read squares in the existing cave and gives the walker follow-up directions.
/// Returns an error if the bank, read sites, or guide differ from the reviewed parents.
pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(ROOM)
        .context("River Pocket Cave")?
        .clone();
    ensure!(
        room.attributes.tileset_name == "cave"
            && (room.attributes.width, room.attributes.height) == (14, 8),
        "unexpected River Pocket Cave terrain"
    );
    guide(pack, &mut room)?;
    mark_stones(pack, &mut room)?;
    authored_town_publication::replace(pack, room)
}
