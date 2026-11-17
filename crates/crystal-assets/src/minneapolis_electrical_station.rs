//! Native control room for Minneapolis on the regional gym review lineage.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_dialogue;
use crate::gym_design::GymDesign;

const PARENT: &str = "d74cd46e8cbabd2c8f011b7782585ff092955cfd88c46664a48c28184518c988";
const MANIFEST: &str = "minnesota-minneapolis-electrical-station-review-v1";

/// Builds a three-gym review pack with a Minneapolis electrical control room.
///
/// The installed switch programs, trainer teams, story and city destinations
/// are retained. The room, actor and control positions, gate block references
/// and one obsolete trainer clue are updated. Saved room positions and block
/// overrides require migration before replacing an existing pack.
///
/// # Errors
/// Returns an error when the parent, authored room, original gates or controls
/// do not match the expected contract, or the resulting pack is invalid.
pub fn build_minneapolis_electrical_station(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == PARENT,
        "two-gym review parent required"
    );
    let design: GymDesign = serde_json::from_str(source)?;
    ensure!(
        design.schema_version == 1 && design.gyms.len() == 1,
        "one station room required"
    );
    let gym = &design.gyms[0];
    ensure!(
        gym.id == "MnMinneapolisGym"
            && gym.slot == 3
            && gym.dimensions_metatiles == [7, 11]
            && gym.arrival_core == [6, 19]
            && gym.exit_tiles_core == [[6, 21], [7, 21]]
            && gym.guide_at == [9, 19]
            && gym.leader_at == [6, 3]
            && gym.trainers.len() == 2
            && gym.trainers[0].id == "MN_GYM_3_TRAINER_1"
            && gym.trainers[0].at == [3, 17]
            && gym.trainers[1].id == "MN_GYM_3_TRAINER_2"
            && gym.trainers[1].at == [10, 14]
            && gym.gates.len() == 2
            && gym.gates[0].at == [3, 3]
            && gym.gates[1].at == [3, 2]
            && gym.gates.iter().all(|g| g.open_symbol == '.')
            && gym.gates[0].event == "EVENT_MN_GYM_3_FIRST_GATE_OPEN"
            && gym.gates[0].requires
                == [
                    "EVENT_MN_GYM_3_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_3_TRAINER_2_DEFEATED",
                    "EVENT_MN_GYM_3_FIRST_SWITCH_ON"
                ]
            && gym.gates[1].event == "EVENT_MN_GYM_3_COURT_OPEN"
            && gym.gates[1].requires
                == [
                    "EVENT_MN_GYM_3_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_3_TRAINER_2_DEFEATED",
                    "EVENT_MN_GYM_3_FIRST_SWITCH_ON",
                    "EVENT_MN_GYM_3_SECOND_SWITCH_ON"
                ]
            && gym.rows.len() == 11
            && gym.rows.iter().all(|r| r.chars().count() == 7)
            && gym.rows[3].chars().nth(3) == Some('A')
            && gym.rows[2].chars().nth(3) == Some('B')
            && gym
                .rows
                .iter()
                .flat_map(|r| r.chars())
                .all(|s| gym.art.contains_key(&s))
            && gym.art.contains_key(&'#')
            && gym.barriers.len() == 2
            && gym.barriers.contains(&'A')
            && gym.barriers.contains(&'B')
            && gym.cut_gates.is_empty()
            && gym.winds.is_empty(),
        "incompatible station source"
    );
    let puzzle = gym.switch_puzzle.as_ref().context("six control stations")?;
    let controls = [
        ("UPPER_LEFT", [5, 11]),
        ("UPPER_MIDDLE", [7, 11]),
        ("UPPER_RIGHT", [9, 11]),
        ("LOWER_LEFT", [5, 15]),
        ("LOWER_MIDDLE", [7, 15]),
        ("LOWER_RIGHT", [9, 15]),
    ];
    ensure!(
        puzzle.controls.len() == controls.len()
            && puzzle
                .controls
                .iter()
                .zip(controls)
                .all(|(control, (id, at))| control.id == id && control.at == at)
            && puzzle.sequence == ["UPPER_LEFT", "UPPER_MIDDLE"]
            && puzzle.events
                == [
                    "EVENT_MN_GYM_3_FIRST_SWITCH_ON",
                    "EVENT_MN_GYM_3_SECOND_SWITCH_ON"
                ]
            && puzzle.requires
                == [
                    "EVENT_MN_GYM_3_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_3_TRAINER_2_DEFEATED"
                ],
        "control contract changed"
    );
    let mut pack = base.clone();
    let mut room = crate::northstar_rival_arc::map(&pack, &gym.id)?;
    ensure!(
        room.attributes.width == 5
            && room.attributes.height == 9
            && room.attributes.tileset_name == "mn_gym_3"
            && room.events.warps.len() == 3
            && room.events.bg_events.len() == 6
            && !pack
                .data
                .maps
                .values()
                .any(|m| m.id != gym.id && m.attributes.tileset_name == "mn_gym_3")
            && !pack
                .data
                .field_moves
                .cut
                .replacements
                .contains_key("mn_gym_3")
            && !pack
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key("mn_gym_3"),
        "incompatible installed station"
    );
    pack.data.tilesets.remove("mn_gym_3");
    for path in [
        "data/tilesets/mn_gym_3.json",
        "data/tilesets/mn_gym_3_metatiles.bin",
        "data/tilesets/mn_gym_3_palette_map.json",
        "gfx/tilesets/mn_gym_3.png",
        "gfx/tilesets/mn_gym_3.2bpp",
    ] {
        ensure!(
            pack.runtime_files.remove(path).is_some(),
            "missing private art {path}"
        );
    }
    pack.report
        .manifests
        .retain(|m| m != "minnesota-gym-3-art-v1");
    let symbols = crate::gym_art::install(&mut pack, gym)?;
    room.attributes.width = 7;
    room.attributes.height = 11;
    room.attributes.border_block = u8::try_from(symbols[&'#'])?;
    room.blocks = gym
        .rows
        .iter()
        .flat_map(|r| r.chars().map(|s| symbols[&s]))
        .collect();
    for (id, old, at) in [
        ("MnMinneapolisGymGuide", [7, 15], gym.guide_at),
        ("MnMinneapolisGymLeader", [4, 3], gym.leader_at),
        ("MN_GYM_3_TRAINER_1", [1, 13], gym.trainers[0].at),
        ("MN_GYM_3_TRAINER_2", [8, 10], gym.trainers[1].at),
    ] {
        let actor = room
            .objects
            .iter_mut()
            .find(|o| o.script == id)
            .context("station actor")?;
        ensure!([actor.x, actor.y] == old, "unexpected actor {id}");
        (actor.x, actor.y) = (at[0], at[1]);
    }
    for (warp, at) in room
        .events
        .warps
        .iter_mut()
        .zip(gym.exit_tiles_core.into_iter().chain([gym.arrival_core]))
    {
        (warp.x, warp.y) = (at[0], at[1]);
    }
    for (control, (id, at)) in room.events.bg_events.iter_mut().zip(controls) {
        ensure!(
            control.script == format!("MnMinneapolisGymSwitch{id}"),
            "unexpected switch binding {id}"
        );
        (control.x, control.y) = (at[0], at[1]);
    }
    for (label, index, old_x, old_y, old_block, new_y, symbol) in [
        ("MnMinneapolisGymGate0", 6, "4", "6", "1", "6", '.'),
        ("MnMinneapolisGymGate0Done", 0, "4", "6", "2", "6", 'A'),
        ("MnMinneapolisGymGate1", 8, "4", "4", "1", "4", '.'),
        ("MnMinneapolisGymGate1Done", 0, "4", "4", "3", "4", 'B'),
    ] {
        let commands = room
            .scripts
            .get_mut(label)
            .and_then(|v| v.as_array_mut())
            .context("gate script")?;
        let command = commands.get_mut(index).context("gate changeblock")?;
        ensure!(
            *command == json!({"command":"changeblock", "args":[old_x,old_y,old_block]}),
            "unexpected gate program {label}"
        );
        let block = symbols.get(&symbol).context("gate art")?;
        *command = json!({"command":"changeblock", "args":["6",new_y,block.to_string()]});
    }
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    authored_dialogue::install_canonical(&mut room, "MN_GYM_3_TRAINER_1AfterText")?;
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
