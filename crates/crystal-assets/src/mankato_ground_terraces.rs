//! Mankato's terrace course on the current regional review lineage.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_dialogue;
use crate::gym_design::GymDesign;

const PARENT: &str = "1cd76d358609d3eba0ad549db3cf2c9142c60f5ef1b55473a67974a3fb819963";
const MANIFEST: &str = "minnesota-mankato-ground-terraces-review-v1";

/// Builds a two-gym review pack, retaining the installed Mankato story and teams.
///
/// Mankato's art, terrain, four gate block references and guide text change;
/// its actor positions, indexed city warps, battle data and progression remain.
/// Saves inside this room need position and block-override migration before use.
///
/// # Errors
/// Rejects a different parent, incompatible geometry or gate programs, or an
/// invalid compiled result. The input pack is not modified on failure.
pub fn build_mankato_ground_terraces(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == PARENT,
        "Rochester progress review parent required"
    );
    let design: GymDesign = serde_json::from_str(source)?;
    ensure!(
        design.schema_version == 1 && design.gyms.len() == 1,
        "one terrace room required"
    );
    let gym = &design.gyms[0];
    ensure!(
        gym.id == "MnMankatoGym"
            && gym.slot == 2
            && gym.dimensions_metatiles == [8, 10]
            && gym.arrival_core == [6, 17]
            && gym.exit_tiles_core == [[6, 19], [7, 19]]
            && gym.guide_at == [9, 16]
            && gym.leader_at == [11, 3]
            && gym.trainers.len() == 2
            && gym.trainers[0].id == "MN_GYM_2_TRAINER_1"
            && gym.trainers[0].at == [4, 17]
            && gym.trainers[1].id == "MN_GYM_2_TRAINER_2"
            && gym.trainers[1].at == [11, 11]
            && gym.gates.len() == 2
            && gym.gates[0].at == [1, 7]
            && gym.gates[0].open_symbol == 'w'
            && gym.gates[0].event == "EVENT_MN_GYM_2_LOWER_STAIR_OPEN"
            && gym.gates[0].requires == ["EVENT_MN_GYM_2_TRAINER_1_DEFEATED"]
            && gym.gates[1].at == [6, 3]
            && gym.gates[1].open_symbol == '.'
            && gym.gates[1].event == "EVENT_MN_GYM_2_COURT_OPEN"
            && gym.gates[1].requires
                == [
                    "EVENT_MN_GYM_2_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_2_TRAINER_2_DEFEATED"
                ]
            && gym.rows.len() == 10
            && gym.rows.iter().all(|row| row.chars().count() == 8)
            && gym.rows[7].chars().nth(1) == Some('A')
            && gym.rows[3].chars().nth(6) == Some('B')
            && gym
                .rows
                .iter()
                .flat_map(|row| row.chars())
                .all(|symbol| gym.art.contains_key(&symbol))
            && gym.art.contains_key(&'#')
            && gym.cut_gates.is_empty()
            && gym.winds.is_empty()
            && gym.barriers.is_empty(),
        "incompatible terrace contract"
    );
    let mut pack = base.clone();
    let mut room = crate::northstar_rival_arc::map(&pack, &gym.id)?;
    ensure!(
        room.attributes.width == 8
            && room.attributes.height == 10
            && room.attributes.tileset_name == "mn_gym_2"
            && room.events.warps.len() == 3
            && !pack
                .data
                .maps
                .values()
                .any(|map| map.id != gym.id && map.attributes.tileset_name == "mn_gym_2")
            && !pack
                .data
                .field_moves
                .cut
                .replacements
                .contains_key("mn_gym_2")
            && !pack
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key("mn_gym_2"),
        "incompatible installed terrace"
    );
    for (id, at) in [
        ("MnMankatoGymGuide", gym.guide_at),
        ("MnMankatoGymLeader", gym.leader_at),
        ("MN_GYM_2_TRAINER_1", gym.trainers[0].at),
        ("MN_GYM_2_TRAINER_2", gym.trainers[1].at),
    ] {
        let actor = room
            .objects
            .iter()
            .find(|o| o.script == id)
            .context("terrace actor")?;
        ensure!(
            [actor.x, actor.y] == at,
            "unexpected actor position for {id}"
        );
    }
    pack.data.tilesets.remove("mn_gym_2");
    for path in [
        "data/tilesets/mn_gym_2.json",
        "data/tilesets/mn_gym_2_metatiles.bin",
        "data/tilesets/mn_gym_2_palette_map.json",
        "gfx/tilesets/mn_gym_2.png",
        "gfx/tilesets/mn_gym_2.2bpp",
    ] {
        ensure!(
            pack.runtime_files.remove(path).is_some(),
            "missing private art {path}"
        );
    }
    pack.report
        .manifests
        .retain(|m| m != "minnesota-gym-2-art-v1");
    let symbols = crate::gym_art::install(&mut pack, gym)?;
    room.attributes.border_block = u8::try_from(symbols[&'#'])?;
    room.blocks = gym
        .rows
        .iter()
        .flat_map(|row| row.chars().map(|s| symbols[&s]))
        .collect();
    for (label, index, x, y, old, replacement) in [
        ("MnMankatoGymGate0", 2, "2", "14", "14", 'w'),
        ("MnMankatoGymGate0Done", 0, "2", "14", "2", 'A'),
        ("MnMankatoGymGate1", 4, "12", "6", "14", '.'),
        ("MnMankatoGymGate1Done", 0, "12", "6", "3", 'B'),
    ] {
        let commands = room
            .scripts
            .get_mut(label)
            .and_then(|v| v.as_array_mut())
            .context("gate script")?;
        let command = commands.get_mut(index).context("gate changeblock")?;
        ensure!(
            *command == json!({"command":"changeblock", "args":[x,y,old]}),
            "unexpected gate program {label}"
        );
        let block = symbols.get(&replacement).context("missing gate art")?;
        command["args"][2] = json!(block.to_string());
    }
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    authored_dialogue::install_canonical(&mut room, "MnMankatoGymGuideText")?;
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
