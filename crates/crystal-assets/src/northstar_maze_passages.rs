//! The Route 10 spruce crown and the survey gallery's breakable eastern passage.
use anyhow::{Context, Result, ensure};
use crystal_core::state::GameState;
use crystal_core::systems::field_moves::apply_cut_field_move;
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::session::OverworldSession;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring;
use crate::northstar_cave_geometry::resting_tiles;
use crate::northstar_maze_tiles as tiles;
use crate::{CompiledGamePack, MapModule};

const ROAD: &str = "MnR10C00R00";
const CAVE: &str = "MnR10Cave1F";
const ROCK: &str = "MN_R10_SURVEY_FRACTURE";
const ORE: &str = "MN_R10_STAR_SEAM";
const SUPPLY: &str = "MN_R10_CROWN_SUPPLY";
const SMASH_MENU: &str = "RockSmashFromMenuScript";
const SMASH: &str = "RockSmashScript";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Terrain {
    road: Vec<tiles::Placement>,
    cave: Vec<tiles::Placement>,
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    retain_rock_smash_menu(pack)?;
    let terrain: Terrain = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/wayfinding-v01/route10-maze.json"
    ))?;
    let mut road = pack.data.maps.get(ROAD).context("Route 10")?.clone();
    tiles::install(pack, &mut road, &terrain.road)?;
    tiles::snow_sapling(pack, &road)?;
    let flag = format!("EVENT_{SUPPLY}");
    crate::northstar_lair_authoring::flag(pack, &flag)?;
    let mut bag = road
        .objects
        .iter()
        .find(|a| a.object_type == "OBJECTTYPE_ITEMBALL")
        .context("regional item ball")?
        .clone();
    bag.x = 12;
    bag.y = 4;
    bag.script = SUPPLY.into();
    bag.object_identifier = Some(SUPPLY.into());
    bag.event_flag = flag;
    road.objects.push(bag);
    ensure!(
        road.scripts
            .insert(
                SUPPLY.into(),
                json!([authoring::command("itemball", &["SHARP_BEAK"])])
            )
            .is_none(),
        "occupied survey supply script"
    );
    publish(pack, road)?;
    let mut cave = pack.data.maps.get(CAVE).context("survey gallery")?.clone();
    tiles::install(pack, &mut cave, &terrain.cave)?;
    add_breakable_rock(pack, &mut cave, ROCK, [40, 6])?;
    crate::northstar_cave_encounters::add_hidden_item(
        pack,
        &mut cave,
        ORE,
        "STAR_PIECE",
        [44, 10],
    )?;
    cave.script_control_commands = crate::parse_script_control_commands(CAVE, &cave.scripts)?;
    publish(pack, cave)?;
    check_road(pack)?;
    check_cave(pack)
}

fn retain_rock_smash_menu(pack: &mut CompiledGamePack) -> Result<()> {
    // The party menu calls this native OverworldEvents label directly.
    let roots = pack
        .data
        .story_events
        .iter_mut()
        .find_map(|catalog| catalog.get_mut("StandardScripts"))
        .and_then(|standard| standard.get_mut("GlobalScriptRoots"))
        .and_then(Value::as_array_mut)
        .context("native executable field script roots")?;
    if !roots.iter().any(|root| root.as_str() == Some(SMASH_MENU)) {
        roots.push(Value::String(SMASH_MENU.into()));
    }
    pack.data.materialize_global_scripts()
}

/// Appends a native, resettable Rock Smash event without disturbing existing object indices.
pub(super) fn add_breakable_rock(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    script: &str,
    at: [u16; 2],
) -> Result<()> {
    ensure!(
        !pack.data.map_scripts.contains_key(script)
            && !room
                .objects
                .iter()
                .any(|o| [o.x, o.y] == at || o.object_identifier.as_deref() == Some(script))
            && !room.events.warps.iter().any(|w| [w.x, w.y] == at)
            && crate::authored_town_geometry::kind(pack, room, at)? == "FLOOR",
        "occupied or impassable smashable rock position"
    );
    let mut rock = pack.data.maps["DarkCaveVioletEntrance"]
        .objects
        .iter()
        .find(|actor| actor.spritemovedata == "SPRITEMOVEDATA_SMASHABLE_ROCK")
        .context("native breakable rock")?
        .clone();
    ensure!(
        rock.sprite == "SPRITE_ROCK"
            && rock.object_type == "OBJECTTYPE_SCRIPT"
            && rock.event_flag == "-1",
        "native rock identity"
    );
    [rock.x, rock.y] = at;
    rock.object_identifier = Some(script.into());
    rock.script = script.into();
    room.objects.push(rock);
    ensure!(
        room.scripts
            .insert(
                script.into(),
                json!([authoring::command("jumpstd", &["SmashRockScript"])])
            )
            .is_none(),
        "occupied rock script"
    );
    Ok(())
}

fn publish(pack: &mut CompiledGamePack, mut room: MapModule) -> Result<()> {
    room.script_field_pickups =
        crate::parse_script_field_pickups(&room.id, &room.scripts, &room.objects)?;
    authoring::refresh_event_section(&mut room);
    crate::northstar_cave_overhaul::validate_native_art(pack, &room)?;
    authored_town_publication::replace(pack, room)
}

fn check_road(pack: &CompiledGamePack) -> Result<()> {
    let road = &pack.data.maps[ROAD];
    let frost = &road.attributes.tileset_name;
    let mut native = pack.data.overworld_map(ROAD)?;
    let collision = pack.data.tileset_collision(frost)?;
    let mut state = GameState::reset_wram_for_new_game();
    let mut storage = state.storage.clone();
    let mut scout = pack
        .data
        .create_pokemon("BRACLAW", 13, Default::default())?;
    scout.moves.clear();
    storage.party.pokemon[0] = Some(scout.clone());
    state.badges.johto[1] = true;
    ensure!(
        apply_cut_field_move(
            &pack.data.field_moves,
            &mut state,
            &storage,
            &mut native,
            &collision,
            frost,
            0,
            10,
            5
        )
        .is_err(),
        "missing field move opened the crown"
    );
    scout
        .moves
        .push(crystal_core::models::pokemon::LearnedMove {
            name: "CUT".into(),
            current_pp: 30,
            pp_ups: 0,
        });
    storage.party.pokemon[0] = Some(scout);
    state.badges.johto[1] = false;
    ensure!(
        apply_cut_field_move(
            &pack.data.field_moves,
            &mut state,
            &storage,
            &mut native,
            &collision,
            frost,
            0,
            10,
            5
        )
        .is_err(),
        "Cut opened the crown before its badge"
    );
    state.badges.johto[1] = true;
    let cleared = apply_cut_field_move(
        &pack.data.field_moves,
        &mut state,
        &storage,
        &mut native,
        &collision,
        frost,
        0,
        10,
        5,
    )?;
    ensure!(
        cleared.variant == "tree"
            && state
                .map_block_overrides
                .get(ROAD)
                .and_then(|changes| changes.get(&(10, 5)))
                == Some(&cleared.replacement_block_id),
        "Cut must record the native tree replacement"
    );
    let mut opened = road.clone();
    opened.blocks[5 * usize::from(road.attributes.width) + 10] = cleared.replacement_block_id;
    let lower = resting_tiles(pack, road, [18, 10], true)?;
    let upper = resting_tiles(pack, &opened, [18, 10], false)?;
    let reset = resting_tiles(pack, road, [12, 5], false)?;
    ensure!(
        lower.contains(&[24, 7])
            && !lower.contains(&[12, 5])
            && !lower.contains(&[24, 5])
            && upper.contains(&[12, 5])
            && upper.contains(&[24, 5])
            && reset.contains(&[24, 7])
            && reset.contains(&[18, 10]),
        "spruce gate, both crown branches and eastward exit"
    );
    Ok(())
}

/// Checks the production party action against the visible actor, with and without Rock Smash.
pub(super) fn check_rock_smash(
    pack: &CompiledGamePack,
    room: &MapModule,
    approach: TilePosition,
    facing: Direction,
    rock: &str,
) -> Result<()> {
    let mut session = OverworldSession::with_events_and_objects(
        OverworldMapData::from_attributes(&room.id, &room.attributes, room.blocks.clone()),
        room.events.clone(),
        room.objects.clone(),
        pack.data.tileset_collision(&room.attributes.tileset_name)?,
        approach,
    );
    session.set_player_facing(facing);
    let mut state = GameState::reset_wram_for_new_game();
    let breaker = pack
        .data
        .create_pokemon("BRACLAW", 13, Default::default())?;
    let mut untrained = breaker.clone();
    untrained.moves.retain(|skill| skill.name != "ROCK_SMASH");
    state.storage.party.pokemon[0] = Some(untrained);
    ensure!(
        pack.data
            .queue_rock_smash_from_menu(&mut state, &mut session, 0)
            .is_err(),
        "rock broken without its move"
    );
    state.storage.party.pokemon[0] = Some(breaker);
    ensure!(
        pack.data
            .queue_rock_smash_from_menu(&mut state, &mut session, 0)?
            == rock
            && state
                .script_runtime
                .next_script
                .as_ref()
                .is_some_and(|queued| {
                    queued.origin_map_name == room.id && queued.script == SMASH_MENU
                })
            && state.script_runtime.last_talked_object.as_deref() == Some(rock),
        "native party menu must target this fracture"
    );
    let globals = pack
        .data
        .global_scripts
        .as_ref()
        .context("global field scripts")?;
    for label in [SMASH_MENU, SMASH] {
        let source = globals
            .definitions
            .get(label)
            .context("native Rock Smash definition")?;
        ensure!(
            globals.scripts.get(label) == Some(source),
            "missing native executable {label}"
        );
    }
    let jump = pack.data.script_control_command(&room.id, SMASH_MENU, 2)?;
    let removal = pack.data.script_object_command(&room.id, SMASH, 7)?;
    ensure!(
        jump.command == "sjump"
            && jump.resolved_target_script.as_deref() == Some(SMASH)
            && removal.command == "disappear"
            && removal.object_id.as_deref() == Some("LAST_TALKED"),
        "Rock Smash menu must execute the common script that removes the target"
    );
    Ok(())
}

fn check_cave(pack: &CompiledGamePack) -> Result<()> {
    let cave = &pack.data.maps[CAVE];
    check_rock_smash(pack, cave, TilePosition::new(40, 5), Direction::Down, ROCK)?;
    let mut broken = cave.clone();
    broken.objects.retain(|o| o.script != ROCK);
    let sealed = resting_tiles(pack, cave, [3, 13], true)?;
    let open = resting_tiles(pack, &broken, [3, 13], false)?;
    let reset = resting_tiles(pack, cave, [43, 10], false)?;
    ensure!(
        sealed.contains(&[9, 6])
            && sealed.contains(&[40, 5])
            && !sealed.contains(&[43, 10])
            && !sealed.contains(&[42, 16])
            && open.contains(&[43, 10])
            && open.contains(&[42, 16])
            && reset.contains(&[42, 18])
            && reset.contains(&[3, 13]),
        "gallery field gate, ore seam and downhill escape"
    );
    Ok(())
}
