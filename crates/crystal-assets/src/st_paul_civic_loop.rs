//! Civic chambers and reciprocal pad loop for St. Paul's Gym.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::gym_design::GymDesign;

const PARENT: &str = "94f8e130be14bf18d626f3340b15aa4a586abf7337def60101592e14a80bdca5";
const MANIFEST: &str = "minnesota-st-paul-civic-loop-review-v1";

/// Builds a four-gym review pack with St. Paul's native reciprocal pad loop.
///
/// Retains city destinations, trainer parties and numbers, story flags and the
/// existing gate qualifications. In-room save positions and terrain overrides
/// need migration before this candidate can replace an existing pack.
///
/// # Errors
/// Rejects an incompatible parent, room, pad wiring, gate program or output.
pub fn build_st_paul_civic_loop(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == PARENT,
        "three-gym review parent required"
    );
    let design: GymDesign = serde_json::from_str(source)?;
    ensure!(
        design.schema_version == 1 && design.gyms.len() == 1,
        "one civic room required"
    );
    let gym = &design.gyms[0];
    ensure!(
        gym.id == "MnStPaulGym"
            && gym.slot == 4
            && gym.dimensions_metatiles == [9, 12]
            && gym.arrival_core == [2, 21]
            && gym.exit_tiles_core == [[2, 23], [3, 23]]
            && gym.guide_at == [6, 20]
            && gym.leader_at == [13, 3]
            && gym.trainers.len() == 2
            && gym.trainers[0].id == "MN_GYM_4_TRAINER_1"
            && gym.trainers[0].at == [3, 3]
            && gym.trainers[1].id == "MN_GYM_4_TRAINER_2"
            && gym.trainers[1].at == [13, 14]
            && gym.gates.len() == 1
            && gym.gates[0].at == [7, 2]
            && gym.gates[0].open_symbol == '.'
            && gym.gates[0].event == "EVENT_MN_GYM_4_COURT_OPEN"
            && gym.gates[0].requires
                == [
                    "EVENT_MN_GYM_4_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_4_TRAINER_2_DEFEATED"
                ]
            && gym.rows.len() == 12
            && gym.rows.iter().all(|r| r.chars().count() == 9)
            && gym.rows[2].chars().nth(7) == Some('X')
            && gym.art.contains_key(&'#')
            && gym
                .rows
                .iter()
                .flat_map(|r| r.chars())
                .all(|s| gym.art.contains_key(&s))
            && gym.cut_gates.is_empty()
            && gym.winds.is_empty()
            && gym.barriers.is_empty(),
        "incompatible civic source"
    );
    let expected_lifts = [
        ("WEST_ROOM", [[1, 13], [1, 3]]),
        ("CROSS_ROOM", [[7, 9], [11, 13]]),
        ("EAST_RETURN", [[15, 21], [11, 9]]),
        ("MAIN_LIFT", [[7, 19], [15, 9]]),
    ];
    ensure!(
        gym.lifts.len() == expected_lifts.len()
            && gym
                .lifts
                .iter()
                .zip(expected_lifts)
                .all(|(lift, (id, ends))| lift.id == id && lift.ends_core == ends),
        "unexpected pad layout"
    );
    let mut pack = base.clone();
    let mut room = crate::northstar_rival_arc::map(&pack, &gym.id)?;
    ensure!(
        room.attributes.width == 7
            && room.attributes.height == 10
            && room.attributes.tileset_name == "mn_gym_4"
            && room.events.warps.len() == 11
            && !pack
                .data
                .maps
                .values()
                .any(|m| m.id != gym.id && m.attributes.tileset_name == "mn_gym_4")
            && !pack
                .data
                .field_moves
                .cut
                .replacements
                .contains_key("mn_gym_4")
            && !pack
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key("mn_gym_4"),
        "incompatible installed civic room"
    );
    pack.data.tilesets.remove("mn_gym_4");
    for path in [
        "data/tilesets/mn_gym_4.json",
        "data/tilesets/mn_gym_4_metatiles.bin",
        "data/tilesets/mn_gym_4_palette_map.json",
        "gfx/tilesets/mn_gym_4.png",
        "gfx/tilesets/mn_gym_4.2bpp",
    ] {
        ensure!(
            pack.runtime_files.remove(path).is_some(),
            "missing private art {path}"
        );
    }
    pack.report
        .manifests
        .retain(|m| m != "minnesota-gym-4-art-v1");
    let symbols = crate::gym_art::install(&mut pack, gym)?;
    room.attributes.width = 9;
    room.attributes.height = 12;
    room.attributes.border_block = u8::try_from(symbols[&'#'])?;
    room.blocks = gym
        .rows
        .iter()
        .flat_map(|r| r.chars().map(|s| symbols[&s]))
        .collect();
    for (id, old, at) in [
        ("MnStPaulGymGuide", [4, 16], gym.guide_at),
        ("MnStPaulGymLeader", [11, 2], gym.leader_at),
        ("MN_GYM_4_TRAINER_1", [3, 3], gym.trainers[0].at),
        ("MN_GYM_4_TRAINER_2", [11, 12], gym.trainers[1].at),
    ] {
        let actor = room
            .objects
            .iter_mut()
            .find(|o| o.script == id)
            .context("civic actor")?;
        ensure!([actor.x, actor.y] == old, "unexpected actor {id}");
        (actor.x, actor.y) = (at[0], at[1]);
    }
    let coordinates = gym
        .exit_tiles_core
        .into_iter()
        .chain([gym.arrival_core])
        .chain(gym.lifts.iter().flat_map(|lift| lift.ends_core));
    for (warp, at) in room.events.warps.iter_mut().zip(coordinates) {
        (warp.x, warp.y) = (at[0], at[1]);
    }
    for (label, index, old_block, symbol) in [
        ("MnStPaulGymGate0", 4, "1", '.'),
        ("MnStPaulGymGate0Done", 0, "6", 'X'),
    ] {
        let commands = room
            .scripts
            .get_mut(label)
            .and_then(|v| v.as_array_mut())
            .context("gate script")?;
        let command = commands.get_mut(index).context("gate changeblock")?;
        ensure!(
            *command == json!({"command":"changeblock","args":["10","4",old_block]}),
            "unexpected gate program {label}"
        );
        let block = symbols.get(&symbol).context("gate art")?;
        *command = json!({"command":"changeblock","args":["14","4",block.to_string()]});
    }
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    authoring::text(
        &mut room,
        "MnStPaulGymGuideText",
        &[
            "Cross each room to",
            "the far pad. Beat",
            "BELVA and ORVEN to",
            "open BRANNIC's",
            "grille. To ride",
            "back, step off a",
            "pad and then step",
            "onto it again. Ask",
            "at the CANOE SHOP",
            "once you've got",
            "the badge.",
        ],
    );
    authoring::text(
        &mut room,
        "MN_GYM_4_TRAINER_2AfterText",
        &[
            "ORVEN: Take the",
            "far pad to",
            "BRANNIC. When",
            "you're done, the",
            "other pad in his",
            "room brings you",
            "back to the",
            "entrance.",
        ],
    );
    crate::northstar_rival_arc::publish(&mut pack, room)?;
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
