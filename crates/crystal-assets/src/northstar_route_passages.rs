//! The farmhouse return bank and the Cut-gated Mille Lacs boulder grove.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapEventSectionCommand;
use crystal_core::state::GameState;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::session::{OverworldInteraction, OverworldInteractionTarget};

use crate::authored_route_surface::{reachable_map_with_surf, walking_map_distances};
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule, authored_town_publication, northstar_regional_design};

const FARM: &str = "MnR02C00R00";
const SHORE: &str = "MnR18C00R00";
const REWARD: &str = "MN_R18_ERRATIC_STONE";
const ITEM: &str = "HARD_STONE";
const FLAG: &str = "EVENT_MN_R18_ERRATIC_STONE";
type Tile = [u16; 2];

fn occupied(room: &MapModule) -> BTreeSet<Tile> {
    room.objects
        .iter()
        .map(|body| [body.x, body.y])
        .chain(room.events.warps.iter().map(|warp| [warp.x, warp.y]))
        .collect()
}

// The existing native cliff remains the bottom eight pixels. Retain its hop
// permissions and the side tree, finishing the ground above in the farm's own
// plowed earth and stubble, the same materials visible across the field gate.
fn install_farm_bank(pack: &mut CompiledGamePack) -> Result<()> {
    let tileset = "mn_prairie_kanto";
    let def = pack
        .data
        .tilesets
        .get_mut(tileset)
        .context("farm tileset")?;
    let path = format!("data/tilesets/{tileset}_metatiles.bin");
    let layout = pack.runtime_files.get_mut(&path).context("farm art")?;
    for (new, surface, original) in [(0x82, 0x2c, 0x07), (0x83, 0x80, 0x07), (0x84, 0x80, 0x42)] {
        ensure!(layout.len() == new * 16, "farm bank atlas preimage");
        let mut glyphs: [u8; 16] = layout[original * 16..(original + 1) * 16].try_into()?;
        for (slot, glyph) in glyphs[..12].iter_mut().enumerate() {
            if original != 0x42 || slot % 4 < 2 {
                *glyph = layout[surface * 16 + slot];
            }
        }
        let collision = def
            .collision
            .get(&format!("{original:02x}"))
            .context("native farm lip")?
            .clone();
        ensure!(
            collision.first().is_some_and(|c| c == "HOP_DOWN")
                && def
                    .collision
                    .insert(format!("{new:02x}"), collision)
                    .is_none(),
            "occupied farm bank"
        );
        layout.extend(glyphs);
    }
    pack.runtime_files.insert(
        format!("data/tilesets/{tileset}.json"),
        serde_json::to_vec(&def.collision)?,
    );
    Ok(())
}

fn check_farm(pack: &CompiledGamePack, before: &MapModule) -> Result<()> {
    let room = &pack.data.maps[FARM];
    let people = occupied(room);
    let distance = |map, start, end| -> Result<u32> {
        walking_map_distances(pack, map, start, &people)?
            .get(&end)
            .copied()
            .context("farm return disconnected")
    };
    let yard = [10, 22];
    let lower = [10, 28];
    let old = distance(before, yard, lower)?;
    ensure!(
        kind(pack, room, [10, 26])? == "HOP_DOWN"
            && kind(pack, room, [10, 27])? == "WALL"
            && distance(room, yard, lower)? == 6
            && old >= 28
            && distance(room, lower, yard)? == distance(before, lower, yard)?
            && distance(room, [11, 13], [2, 30])? + 12 < distance(before, [11, 13], [2, 30])?
            && distance(room, [2, 30], [11, 13])? == distance(before, [2, 30], [11, 13])?,
        "the crop bank must shorten the downhill return only"
    );
    Ok(())
}

fn prove_grove(pack: &CompiledGamePack, room: &MapModule) -> Result<()> {
    let people = occupied(room);
    let entrance = [5, 12];
    let grove = [4, 20];
    let gate = [4, 15];
    let landing = [6, 26];
    let closed = reachable_map_with_surf(pack, room, entrance, &people)?;
    ensure!(
        kind(pack, room, gate)? == "CUT_TREE"
            && kind(pack, room, [3, 20])? == "WALL"
            && closed.contains(&[4, 14])
            && closed.contains(&landing)
            && closed.contains(&[63, 10])
            && [[2, 20], [3, 19], [3, 21], grove]
                .iter()
                .all(|near| !closed.contains(near))
            && !closed.contains(&[6, 24]),
        "the grove can be entered with the tree standing or the public road is broken"
    );
    let mut open = room.clone();
    let index = 7 * usize::from(room.attributes.width) + 2;
    let block = room.blocks[index];
    let replacement = pack
        .data
        .field_moves
        .cut
        .replacements
        .get(&room.attributes.tileset_name)
        .and_then(|atlas| atlas.get(&block))
        .context("native grove Cut replacement")?;
    ensure!(
        replacement.variant == "tree",
        "grove sapling must use the native tree reset"
    );
    open.blocks[index] = replacement.replacement_block_id;
    let approach = walking_map_distances(pack, &open, entrance, &people)?;
    let goal = walking_map_distances(pack, &open, grove, &people)?;
    let reverse = walking_map_distances(pack, &open, landing, &people)?;
    ensure!(
        approach.contains_key(&grove)
            && kind(pack, &open, gate)? == "FLOOR"
            && kind(pack, room, [6, 24])? == "HOP_DOWN"
            && goal.get(&landing) == Some(&8)
            && reverse.get(&grove).is_some_and(|distance| *distance >= 25),
        "the grove must have a distinct one-way return to the marina"
    );
    // If the tree grows back while a save is inside, the downhill exit still works.
    ensure!(
        walking_map_distances(pack, room, grove, &people)?.contains_key(&landing)
            && !walking_map_distances(pack, room, landing, &people)?.contains_key(&grove),
        "Cut reset strands a saved visitor or permits reverse access"
    );
    Ok(())
}

fn add_grove_reward(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(SHORE)
        .context("Mille Lacs shore")?
        .clone();
    prove_grove(pack, &room)?;
    crate::northstar_cave_encounters::add_hidden_item(pack, &mut room, REWARD, ITEM, [3, 20])?;
    room.script_field_pickups =
        crate::parse_script_field_pickups(&room.id, &room.scripts, &room.objects)?;
    let at = room
        .map_event_section_commands
        .iter()
        .position(|s| s.command == "def_object_events")
        .context("shore object section")?;
    room.map_event_section_commands.insert(
        at,
        MapEventSectionCommand {
            command: "bg_event".into(),
            args: ["3", "20", "BGEVENT_ITEM", REWARD]
                .map(String::from)
                .to_vec(),
            command_index: at,
        },
    );
    for (i, row) in room
        .map_event_section_commands
        .iter_mut()
        .enumerate()
        .skip(at + 1)
    {
        row.command_index = i;
    }
    ensure!(
        room.script_field_pickups
            .iter()
            .any(|event| event.source_script == REWARD
                && event.item_id.as_deref() == Some(ITEM)
                && event.event_flag.as_deref() == Some(FLAG)
                && event.quantity == 1),
        "published boulder reward"
    );
    authored_town_publication::replace(pack, room)?;
    let interaction = OverworldInteraction {
        map_name: SHORE.into(),
        player_tile: TilePosition::new(4, 20),
        target_tile: TilePosition::new(3, 20),
        facing: Direction::Left,
        script: REWARD.into(),
        target: OverworldInteractionTarget::Background {
            event_type: "BGEVENT_ITEM".into(),
        },
    };
    let mut state = GameState::reset_wram_for_new_game();
    ensure!(
        pack.data
            .resolve_overworld_interaction(&state, &interaction)?
            .is_some(),
        "unclaimed erratic reward"
    );
    state.flags.set_event_flag(FLAG, true)?;
    ensure!(
        pack.data
            .resolve_overworld_interaction(&state, &interaction)?
            .is_none(),
        "collected erratic reward repeated"
    );
    Ok(())
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let before = pack
        .data
        .maps
        .get(FARM)
        .context("original farm road")?
        .clone();
    install_farm_bank(pack)?;
    northstar_regional_design::apply_rooms(
        pack,
        include_bytes!("../../../planning/minnesota/revisions/wayfinding-v01/route-passages.json"),
    )?;
    check_farm(pack, &before)?;
    add_grove_reward(pack)
}
