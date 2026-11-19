//! Observable tracks, timber and the water's edge on Bemidji's Tall Tale Trail.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::{
    CompiledGamePack, MapModule, TilesetDefinition, authored_town_publication, native_tile_png,
};

const MAP: &str = "MnTallTaleTrail";
const SHEET: &str = "gfx/tilesets/johto.2bpp";
const ATLAS: &str = "data/tilesets/johto_metatiles.bin";
const BACKGROUNDS: &str = "data/tilesets/johto_native_backgrounds.json";
const KEEPER: &str = "MN_TALE_BUNJANT";
const ART: &str =
    include_str!("../../../planning/minnesota/revisions/trail-discoveries-v01/tall-tale-art.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Drawing {
    palette: u8,
    background: u8,
    rows: [String; 16],
}

/// The last 32 physical tiles are blank in the reviewed Johto sheet. The allocator
/// uses their native second-bank addresses and keeps the compact sheet the same size.
fn draw(
    drawing: &Drawing,
    width: usize,
    graphics: &mut [u8],
    bank: &mut TilesetDefinition,
    backgrounds: &mut BTreeMap<u8, u8>,
    next: &mut u8,
) -> Result<Vec<u8>> {
    ensure!(
        drawing
            .rows
            .iter()
            .all(|r| r.len() == width && r.bytes().all(|v| matches!(v, b'0'..=b'3'))),
        "invalid four-shade trail drawing"
    );
    ensure!(
        matches!(drawing.palette, 0 | 5) && matches!(drawing.background, 0 | 2),
        "unexpected trail palette"
    );
    let mut glyphs = Vec::new();
    for y in (0..16).step_by(8) {
        for x in (0..width).step_by(8) {
            let id = *next;
            ensure!(
                (0xc0..0xe0).contains(&id),
                "trail drawing exceeds blank Johto tiles"
            );
            let offset = (96 + usize::from(id & 127)) * 16;
            let dest = &mut graphics[offset..offset + 16];
            ensure!(dest.iter().all(|&b| b == 0), "occupied trail glyph {id:#x}");
            for (row, line) in drawing.rows[y..y + 8].iter().enumerate() {
                for (col, &px) in line.as_bytes()[x..x + 8].iter().enumerate() {
                    let shade = px - b'0';
                    dest[2 * row] |= (shade & 1) << (7 - col);
                    dest[2 * row + 1] |= (shade >> 1) << (7 - col);
                }
            }
            bank.palette_map[usize::from(id)] = 8 | drawing.palette;
            if drawing.background != 0 {
                backgrounds.insert(id, drawing.background);
            }
            glyphs.push(id);
            *next += 1;
        }
    }
    Ok(glyphs)
}

fn add_block(
    atlas: &mut Vec<u8>,
    bank: &mut TilesetDefinition,
    block: [u8; 16],
    collision: &[String],
) -> Result<u16> {
    if let Some((i, _)) = atlas.as_chunks::<16>().0.iter().enumerate().find(|(i, b)| {
        **b == block
            && bank
                .collision
                .get(&format!("{i:02x}"))
                .is_some_and(|c| c == collision)
    }) {
        return Ok(u16::try_from(i)?);
    }
    let id = u8::try_from(atlas.len() / 16).context("Johto metatile capacity")?;
    ensure!(
        bank.collision
            .insert(format!("{id:02x}"), collision.to_vec())
            .is_none(),
        "occupied trail block"
    );
    atlas.extend(block);
    Ok(u16::from(id))
}

// All shore edits are confined to the twenty-two water blocks of this pool.
// The west and east profiles and upper bends come straight from native Johto.
fn pond(room: &mut MapModule, atlas: &mut Vec<u8>, bank: &mut TilesetDefinition) -> Result<()> {
    let snapshot = room.blocks.clone();
    let water = |x: usize, y: usize| snapshot[y * 20 + x] == 0x35;
    let permission = bank.collision.get("35").context("pond water")?.clone();
    let mut count = 0;
    for y in 3..=7 {
        for x in 3..=7 {
            if !water(x, y) {
                continue;
            }
            count += 1;
            let (north, south, west, east) = (
                !water(x, y - 1),
                !water(x, y + 1),
                !water(x - 1, y),
                !water(x + 1, y),
            );
            let mut b = [0x14; 16];
            if north {
                b[..4].fill(0x4c);
            }
            if south {
                b[12..].fill(0x4c);
            }
            for row in b.as_chunks_mut::<4>().0 {
                if west {
                    row[0] = 0x3d;
                }
                if east {
                    row[3] = 0x3b;
                }
            }
            for (i, vertical, horizontal, diagonal, convex, concave) in [
                (0, north, west, water(x - 1, y - 1), 0x5c, 0x4d),
                (3, north, east, water(x + 1, y - 1), 0x5d, 0x4b),
                (12, south, west, water(x - 1, y + 1), 0x4d, 0x5c),
                (15, south, east, water(x + 1, y + 1), 0x4b, 0x5d),
            ] {
                if vertical && horizontal {
                    b[i] = convex;
                } else if !vertical && !horizontal && !diagonal {
                    b[i] = concave;
                }
            }
            room.blocks[y * 20 + x] = add_block(atlas, bank, b, &permission)?;
        }
    }
    ensure!(
        count == 22 && snapshot.iter().filter(|&&b| b == 0x35).count() == 22,
        "unexpected footprint of tall tale pool"
    );
    Ok(())
}

fn stage_keeper(room: &mut MapModule, log: u16) -> Result<()> {
    let terrain = room
        .scripts
        .get_mut("MnTaleTerrain")
        .and_then(|s| s.as_array_mut())
        .context("closed trail callback")?;
    let closed = terrain
        .iter_mut()
        .find(|r| **r == json!({"command":"changeblock","args":["28","16","93"]}))
        .context("old trail saplings")?;
    closed["args"][2] = json!(log.to_string());
    let scene = room
        .scripts
        .get_mut("MnTalePath")
        .and_then(|s| s.as_array_mut())
        .context("moving the loose log")?;
    for (op, label) in [
        ("appear", "MnTaleKeeperAtLog"),
        ("disappear", "MnTaleKeeperReturnsToCamp"),
    ] {
        let slot = scene
            .iter_mut()
            .find(|r| **r == json!({"command":op,"args":[KEEPER]}))
            .context("Bunjant's original entrance and exit")?;
        *slot = json!({"command":"scall","args":[label]});
    }
    for (label, rows) in [
        (
            "MnTaleKeeperAtLog",
            json!([{"command":"moveobject","args":[KEEPER,"29","15"]},{"command":"appear","args":[KEEPER]},{"command":"turnobject","args":[KEEPER,"DOWN"]},{"command":"end","args":[]}]),
        ),
        (
            "MnTaleKeeperReturnsToCamp",
            json!([{"command":"disappear","args":[KEEPER]},{"command":"moveobject","args":[KEEPER,"30","8"]},{"command":"end","args":[]}]),
        ),
    ] {
        ensure!(
            room.scripts.insert(label.into(), rows).is_none(),
            "occupied encounter label {label}"
        );
    }
    room.script_control_commands = crate::parse_script_control_commands(MAP, &room.scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(MAP, &room.scripts)?;
    room.script_block_changes = crate::parse_script_block_changes(MAP, &room.scripts)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(MAP, &room.scripts)?;
    Ok(())
}

fn worker_reminders(room: &mut MapModule) -> Result<()> {
    use crate::authored_dialogue::install_canonical;
    let command = |op: &str, arg: &str| json!({"command":op,"args":[arg]});
    let worker = room
        .scripts
        .get_mut("MnTaleWorker")
        .and_then(|s| s.as_array_mut())
        .context("trail worker")?;
    let mut missing = worker
        .iter_mut()
        .filter(|r| **r == command("iffalse", "MnTaleIntroduction"));
    for label in ["MnTaleMissingPipe", "MnTaleFindPrint", "MnTaleFindStack"] {
        *missing.next().context("missing observation check")? = command("iffalse", label);
    }
    ensure!(missing.next().is_none(), "unexpected worker observation");
    let first = worker
        .iter()
        .position(|r| *r == command("checkevent", "EVENT_MN_TALL_TALE_CULVERT_READ"))
        .context("worker's first field observation")?;
    worker.splice(
        first..first,
        [
            command("checkevent", "EVENT_MN_TALL_TALE_CAMP_PATH_ACKNOWLEDGED"),
            command("iftrue", "MnTaleFindMark"),
            command("checkevent", "EVENT_MN_TALL_TALE_CAMP_DIRECTIONS"),
            command("iftrue", "MnTaleFindPath"),
        ],
    );
    let branch = json!([
        command("checkevent", "EVENT_MN_TALL_TALE_PRINTS_READ"),
        command("iftrue", "MnTaleFindPipe"),
        command("checkevent", "EVENT_MN_TALL_TALE_STACK_READ"),
        command("iftrue", "MnTaleFindPipe"),
        command("sjump", "MnTaleIntroduction")
    ]);
    ensure!(
        room.scripts
            .insert("MnTaleMissingPipe".into(), branch)
            .is_none(),
        "occupied missing-pipe branch"
    );
    for (label, page) in [
        ("MnTaleFindPipe", "MnTaleText_find_pipe"),
        ("MnTaleFindPrint", "MnTaleText_find_print"),
        ("MnTaleFindStack", "MnTaleText_find_stack"),
        ("MnTaleFindPath", "MnTaleText_find_path"),
        ("MnTaleFindMark", "MnTaleText_find_mark"),
    ] {
        ensure!(
            !room.scripts.contains_key(page)
                && room
                    .scripts
                    .insert(label.into(), json!([command("jumptext", page)]))
                    .is_none(),
            "occupied worker reminder {label}"
        );
        install_canonical(room, page)?;
    }
    for ask in ["MnTaleText_ox_ask", "MnTaleText_keeper_ask"] {
        ensure!(
            room.scripts.contains_key(ask),
            "missing voluntary battle {ask}"
        );
        install_canonical(room, ask)?;
    }
    room.script_flag_commands = crate::parse_script_flag_commands(MAP, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(MAP, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(MAP, &room.scripts)?;
    Ok(())
}

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(MAP).context(MAP)?.clone();
    let bank = pack.data.tilesets.get_mut("johto").context("Johto")?;
    let mut atlas = pack.runtime_files.get(ATLAS).context(ATLAS)?.clone();
    let mut graphics = pack.runtime_files.get(SHEET).context(SHEET)?.clone();
    ensure!(
        room.attributes.tileset_name == "johto"
            && room.attributes.width == 20
            && room.attributes.height == 20
            && atlas.len() == 128 * 16
            && graphics.len() == 192 * 16
            && bank.palette_map.len() == 224,
        "unexpected story trail or native tileset"
    );
    // Neither a compact physical slot nor its logical address may serve an old block.
    for &g in &atlas {
        let palette = bank.palette_map[usize::from(g)];
        let pos = if palette & 8 == 0 {
            usize::from(g)
        } else {
            96 + usize::from(g & 127)
        };
        ensure!(
            pos < 160 && g < 0xc0,
            "original Johto tile aliases a trail drawing"
        );
    }
    let mut backgrounds = BTreeMap::new();
    ensure!(
        !pack.runtime_files.contains_key(BACKGROUNDS),
        "unexpected Johto background overrides"
    );
    let drawings: BTreeMap<String, Drawing> = serde_json::from_str(ART)?;
    let mut next = 0xc0;
    let mut closed = None;
    for (name, x, y, old, first, width, base) in [
        ("culvert", 11, 14, 0x47, 10, 16, 0x05),
        ("hoofprint", 8, 7, 0x47, 10, 16, 0x05),
        ("windthrow", 14, 10, 0x5d, 0, 32, 0x05),
        ("marked_log", 14, 4, 0x5d, 0, 16, 0x05),
        ("loose_log", 14, 8, 0x5d, 0, 32, 0x06),
        ("record_post", 15, 6, 0x45, 0, 16, 0x06),
    ] {
        let at = y * 20 + x;
        ensure!(room.blocks[at] == old, "trail landmark {name} moved");
        let mut b = atlas.as_chunks::<16>().0[usize::from(old)];
        let mut collision = bank.collision[&format!("{old:02x}")].clone();
        if name == "loose_log" {
            b.fill(base);
        }
        let g = draw(
            drawings.get(name).context("original trail drawing")?,
            width,
            &mut graphics,
            bank,
            &mut backgrounds,
            &mut next,
        )?;
        for (i, tile) in g.into_iter().enumerate() {
            b[first + (i / (width / 8)) * 4 + i % (width / 8)] = tile;
        }
        if matches!(name, "loose_log" | "windthrow") {
            collision[..2].fill("WALL".into());
        }
        if name == "marked_log" {
            collision[0] = "WALL".into();
        }
        room.blocks[at] = add_block(&mut atlas, bank, b, &collision)?;
        if name == "loose_log" {
            closed = Some(room.blocks[at]);
        }
    }
    ensure!(next == 0xe0, "incomplete trail drawings");
    pond(&mut room, &mut atlas, bank)?;
    // A faces one movement square. Either half of the same trunk should answer.
    for (label, old_x, new_x, y) in [("MnTalePath", 28, 29, 16), ("MnTaleStack", 29, 28, 20)] {
        ensure!(
            room.events
                .bg_events
                .iter()
                .all(|e| (e.x, e.y) != (new_x, y)),
            "occupied second half of {label}"
        );
        let mut event = room
            .events
            .bg_events
            .iter()
            .find(|e| {
                (e.x, e.y) == (old_x, y) && e.script == label && e.event_type == "BGEVENT_READ"
            })
            .context("existing timber interaction")?
            .clone();
        event.x = new_x;
        room.events.bg_events.push(event);
    }
    crate::grand_marais_modpack::authoring::refresh_event_section(&mut room);
    worker_reminders(&mut room)?;
    stage_keeper(&mut room, closed.context("closed trail log")?)?;
    pack.runtime_files.insert(
        "gfx/tilesets/johto.png".into(),
        native_tile_png::from_2bpp(&graphics)?,
    );
    pack.runtime_files.insert(SHEET.into(), graphics);
    pack.runtime_files.insert(ATLAS.into(), atlas);
    pack.runtime_files
        .insert(BACKGROUNDS.into(), serde_json::to_vec(&backgrounds)?);
    authored_town_publication::replace(pack, room)
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result, ensure};
    use crystal_core::state::GameState;
    use crystal_core::world::map::{Direction, TilePosition};
    use crystal_core::world::movement::{StepOptions, StepOutcome};
    use std::path::PathBuf;

    use super::{KEEPER, MAP};

    #[test]
    #[ignore = "requires the locally built archive in MN_TRAIL_DISCOVERIES_PACK"]
    fn tall_tale_trunk_survives_real_map_entry_and_both_ends_are_readable() -> Result<()> {
        let path = PathBuf::from(
            std::env::var_os("MN_TRAIL_DISCOVERIES_PACK").context("discovery archive")?,
        );
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        let room = data.maps.get(MAP).context(MAP)?;
        let trunk = room.blocks[8 * 20 + 14];
        ensure!(trunk >= 0x80, "missing authored log");
        for cleared in [false, true] {
            for x in [28, 29] {
                let mut state = GameState::reset_wram_for_new_game();
                state
                    .flags
                    .set_event_flag("EVENT_MN_TALL_TALE_CAMP_PATH_ACKNOWLEDGED", cleared)
                    .map_err(|e| anyhow::anyhow!("camp access: {e:?}"))?;
                let mut session = data.overworld_session(MAP, TilePosition::new(x, 17), 0)?;
                data.apply_map_setup_callbacks(&mut state, &mut session, MAP, "MAPSETUP_WARP")?;
                ensure!(
                    session.map.metatile_at(14, 8) == Some(if cleared { 1 } else { trunk }),
                    "entry restored the wrong map tile"
                );
                session.player.facing = Direction::Up;
                let hit = session
                    .check_interaction_checked(1)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .context("face either end of the log")?;
                let active = data
                    .resolve_overworld_interaction(&state, &hit)?
                    .context("native timber interaction")?;
                ensure!(
                    active.script == "MnTalePath" && active.target_tile == TilePosition::new(x, 16),
                    "wrong native A target on the path"
                );
                let step = session
                    .step_checked(Direction::Up, StepOptions::default())
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                ensure!(
                    matches!(step, StepOutcome::Moved { .. }) == cleared,
                    "wrong walk permission at {x},16"
                );
            }
        }
        for x in [28, 29] {
            let state = GameState::reset_wram_for_new_game();
            let mut session = data.overworld_session(MAP, TilePosition::new(x, 21), 0)?;
            session.player.facing = Direction::Up;
            let hit = session
                .check_interaction_checked(1)
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .context("face the fallen timber")?;
            ensure!(
                data.resolve_overworld_interaction(&state, &hit)?
                    .context("native observation")?
                    .script
                    == "MnTaleStack",
                "unreadable timber evidence"
            );
        }
        let mut state = GameState::reset_wram_for_new_game();
        let mut session = data.overworld_session(MAP, TilePosition::new(28, 17), 0)?;
        data.apply_map_setup_callbacks(&mut state, &mut session, MAP, "MAPSETUP_WARP")?;
        for index in 0..3 {
            data.apply_script_object_mutation_in_session(
                &mut state,
                &mut session,
                MAP,
                "MnTaleKeeperAtLog",
                index,
            )?;
        }
        ensure!(
            session
                .object_runtime_tile_by_id(KEEPER)
                .map_err(|e| anyhow::anyhow!("{e}"))?
                == TilePosition::new(29, 15)
                && session
                    .visible_object_at(TilePosition::new(29, 15))
                    .is_some_and(|(_, o)| o.script == KEEPER),
            "keeper must be visible beside the trunk while its page is read"
        );
        for index in 0..2 {
            data.apply_script_object_mutation_in_session(
                &mut state,
                &mut session,
                MAP,
                "MnTaleKeeperReturnsToCamp",
                index,
            )?;
        }
        ensure!(
            session
                .object_runtime_tile_by_id(KEEPER)
                .map_err(|e| anyhow::anyhow!("{e}"))?
                == TilePosition::new(30, 8)
                && session
                    .visible_object_at(TilePosition::new(29, 15))
                    .is_none(),
            "keeper must vacate the path and return to its encounter square"
        );
        Ok(())
    }
}
