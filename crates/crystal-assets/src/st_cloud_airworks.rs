//! Framed wind crossings for St. Cloud on the regional gym review lineage.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::gym_design::GymDesign;

const PARENT: &str = "0ea1c70ac3b17ae4818468f5b5ce5c1730abab2b369380b187c6b900c0c7b334";
const MANIFEST: &str = "minnesota-st-cloud-airworks-review-v1";

/// Builds a five-gym review pack with visible wind direction and tower framing.
///
/// The installed room's dimensions, collisions, actors, trainer teams, gate
/// qualifications, city destinations and story remain intact. Saved room block
/// overrides require migration before replacing a pack with different tile IDs.
///
/// # Errors
/// Rejects an incompatible parent, source, collision, gate program or output.
pub fn build_st_cloud_airworks(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == PARENT,
        "four-gym review parent required"
    );
    let design: GymDesign = serde_json::from_str(source)?;
    ensure!(
        design.schema_version == 1 && design.gyms.len() == 1,
        "one wind room required"
    );
    let gym = &design.gyms[0];
    ensure!(
        gym.id == "MnStCloudGym"
            && gym.slot == 5
            && gym.dimensions_metatiles == [9, 11]
            && gym.arrival_core == [4, 19]
            && gym.exit_tiles_core == [[4, 21], [5, 21]]
            && gym.guide_at == [7, 18]
            && gym.leader_at == [3, 3]
            && gym.trainers.len() == 2
            && gym.trainers[0].id == "MN_GYM_5_TRAINER_1"
            && gym.trainers[0].at == [4, 15]
            && gym.trainers[1].id == "MN_GYM_5_TRAINER_2"
            && gym.trainers[1].at == [14, 10]
            && gym.gates.len() == 2
            && gym.gates[0].at == [3, 6]
            && gym.gates[1].at == [6, 4]
            && gym.gates.iter().all(|gate| gate.open_symbol == '.')
            && gym.gates[0].event == "EVENT_MN_GYM_5_LOWER_CROSSING_OPEN"
            && gym.gates[0].requires == ["EVENT_MN_GYM_5_TRAINER_1_DEFEATED"]
            && gym.gates[1].event == "EVENT_MN_GYM_5_COURT_OPEN"
            && gym.gates[1].requires
                == [
                    "EVENT_MN_GYM_5_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_5_TRAINER_2_DEFEATED"
                ]
            && gym.wind_arrows
            && gym.winds.len() == 2
            && gym.winds.get(&'>').is_some_and(|v| v == "WALK_RIGHT")
            && gym.winds.get(&'<').is_some_and(|v| v == "WALK_LEFT")
            && gym.rows.len() == 11
            && gym.rows.iter().all(|r| r.chars().count() == 9)
            && gym.rows[6].chars().nth(3) == Some('B')
            && gym.rows[4].chars().nth(6) == Some('A')
            && gym.art.contains_key(&'#')
            && gym
                .rows
                .iter()
                .flat_map(|r| r.chars())
                .all(|s| gym.art.contains_key(&s))
            && gym.cut_gates.is_empty()
            && gym.barriers.is_empty(),
        "incompatible wind course source"
    );
    let mut pack = base.clone();
    let mut room = crate::northstar_rival_arc::map(&pack, &gym.id)?;
    ensure!(
        room.attributes.width == 9
            && room.attributes.height == 11
            && room.attributes.tileset_name == "mn_gym_5"
            && room.events.warps.len() == 3
            && !pack
                .data
                .maps
                .values()
                .any(|m| m.id != gym.id && m.attributes.tileset_name == "mn_gym_5")
            && !pack
                .data
                .field_moves
                .cut
                .replacements
                .contains_key("mn_gym_5")
            && !pack
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key("mn_gym_5"),
        "incompatible installed wind room"
    );
    for (id, at) in [
        ("MnStCloudGymGuide", gym.guide_at),
        ("MnStCloudGymLeader", gym.leader_at),
        ("MN_GYM_5_TRAINER_1", gym.trainers[0].at),
        ("MN_GYM_5_TRAINER_2", gym.trainers[1].at),
    ] {
        let actor = room
            .objects
            .iter()
            .find(|o| o.script == id)
            .context("wind room actor")?;
        ensure!([actor.x, actor.y] == at, "unexpected actor {id}");
    }
    let original_collision = pack
        .data
        .tilesets
        .get("mn_gym_5")
        .context("installed wind collision")?
        .collision
        .clone();
    let original_blocks = room.blocks.clone();
    pack.data.tilesets.remove("mn_gym_5");
    for path in [
        "data/tilesets/mn_gym_5.json",
        "data/tilesets/mn_gym_5_metatiles.bin",
        "data/tilesets/mn_gym_5_palette_map.json",
        "gfx/tilesets/mn_gym_5.png",
        "gfx/tilesets/mn_gym_5.2bpp",
    ] {
        ensure!(
            pack.runtime_files.remove(path).is_some(),
            "missing private art {path}"
        );
    }
    pack.report
        .manifests
        .retain(|m| m != "minnesota-gym-5-art-v1");
    let symbols = crate::gym_art::install(&mut pack, gym)?;
    room.attributes.border_block = u8::try_from(symbols[&'#'])?;
    room.blocks = gym
        .rows
        .iter()
        .flat_map(|r| r.chars().map(|s| symbols[&s]))
        .collect();
    let new_collision = &pack
        .data
        .tilesets
        .get("mn_gym_5")
        .context("new wind collision")?
        .collision;
    for (index, (old, new)) in original_blocks.iter().zip(&room.blocks).enumerate() {
        ensure!(
            original_collision.get(&format!("{old:02x}"))
                == new_collision.get(&format!("{new:02x}")),
            "collision changed at block {index}"
        );
    }
    for (label, index, x, y, old, symbol) in [
        ("MnStCloudGymGate0", 2, "6", "12", "1", '.'),
        ("MnStCloudGymGate0Done", 0, "6", "12", "5", 'B'),
        ("MnStCloudGymGate1", 4, "12", "8", "1", '.'),
        ("MnStCloudGymGate1Done", 0, "12", "8", "4", 'A'),
    ] {
        let commands = room
            .scripts
            .get_mut(label)
            .and_then(|v| v.as_array_mut())
            .context("gate script")?;
        let command = commands.get_mut(index).context("gate changeblock")?;
        ensure!(
            *command == json!({"command":"changeblock","args":[x,y,old]}),
            "unexpected gate program {label}"
        );
        let block = symbols.get(&symbol).context("gate art")?;
        *command = json!({"command":"changeblock","args":[x,y,block.to_string()]});
    }
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    for label in [
        "MnStCloudGymGuideText",
        "MN_GYM_5_TRAINER_1AfterText",
        "MN_GYM_5_TRAINER_2AfterText",
    ] {
        crate::authored_dialogue::install_canonical(&mut room, label)?;
    }
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
