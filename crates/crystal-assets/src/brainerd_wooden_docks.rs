//! Native wooden docks and a physical pier control for Brainerd Gym.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::gym_design::GymDesign;

const PARENT: &str = "1804f99330dcbc18b467156507acf2b29e34d86bbf2adf273ee0b3c79fd57175";
const MANIFEST: &str = "minnesota-brainerd-wooden-docks-review-v1";

/// Builds a six-gym review pack with native plank docks and a pier control.
///
/// Trainer parties, badge requirements, rewards, indexed city warps and the
/// existing route flags are retained. Existing in-room tile overrides need
/// migration before replacing an older pack.
///
/// # Errors
/// Rejects an incompatible parent, source, actor, gate or routing program,
/// unsafe pier states, or invalid compiled output.
pub fn build_brainerd_wooden_docks(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == PARENT,
        "five-gym review parent required"
    );
    let design: GymDesign = serde_json::from_str(source)?;
    ensure!(
        design.schema_version == 1 && design.gyms.len() == 1,
        "one dock room required"
    );
    let gym = &design.gyms[0];
    ensure!(
        gym.id == "MnBrainerdGym"
            && gym.slot == 6
            && gym.wooden_port_deck
            && gym.dimensions_metatiles == [5, 8]
            && gym.arrival_core == [4, 14]
            && gym.exit_tiles_core == [[4, 15], [5, 15]]
            && gym.guide_at == [7, 14]
            && gym.leader_at == [5, 3]
            && gym.trainers.len() == 2
            && gym.trainers[0].id == "MN_GYM_6_TRAINER_1"
            && gym.trainers[0].at == [2, 10]
            && gym.trainers[1].id == "MN_GYM_6_TRAINER_2"
            && gym.trainers[1].at == [2, 7]
            && gym.rows.len() == 8
            && gym.rows.iter().all(|r| r.chars().count() == 5)
            && gym.rows[6].chars().nth(3) == Some('S')
            && gym.art.contains_key(&'#')
            && gym
                .rows
                .iter()
                .flat_map(|r| r.chars())
                .all(|s| gym.art.contains_key(&s))
            && gym.routers.len() == 1
            && gym.routers[0].id == "FLOAT_WALK"
            && gym.routers[0].operator.is_none()
            && gym.routers[0].toggle
            && gym.routers[0].at == [6, 13]
            && gym.routers[0].modes.len() == 2
            && gym.routers[0].modes[0].id == "CENTER"
            && gym.routers[0].modes[0].event == "EVENT_MN_GYM_6_WALK_CENTER"
            && gym.routers[0].modes[1].id == "LEFT"
            && gym.routers[0].modes[1].event == "EVENT_MN_GYM_6_WALK_LEFT"
            && gym.routers[0]
                .modes
                .iter()
                .all(|m| m.requires == ["EVENT_MN_GYM_6_TRAINER_1_DEFEATED"])
            && gym.gates.len() == 3
            && gym.gates[0].at == [2, 4]
            && gym.gates[0].open_symbol == 'v'
            && gym.gates[1].at == [1, 4]
            && gym.gates[1].open_symbol == 'v'
            && gym.gates[2].at == [3, 2]
            && gym.gates[2].open_symbol == 'j'
            && gym.enclosed_pool
            && gym.cut_gates.is_empty()
            && gym.winds.is_empty()
            && gym.barriers.is_empty(),
        "incompatible dock source"
    );
    for (gate, event, requires) in [
        (
            &gym.gates[0],
            "EVENT_MN_GYM_6_CENTER_WALK_OPEN",
            vec![
                "EVENT_MN_GYM_6_TRAINER_1_DEFEATED",
                "EVENT_MN_GYM_6_WALK_CENTER",
            ],
        ),
        (
            &gym.gates[1],
            "EVENT_MN_GYM_6_LEFT_WALK_OPEN",
            vec![
                "EVENT_MN_GYM_6_TRAINER_1_DEFEATED",
                "EVENT_MN_GYM_6_TRAINER_2_DEFEATED",
                "EVENT_MN_GYM_6_WALK_LEFT",
            ],
        ),
        (
            &gym.gates[2],
            "EVENT_MN_GYM_6_COURT_OPEN",
            vec![
                "EVENT_MN_GYM_6_TRAINER_1_DEFEATED",
                "EVENT_MN_GYM_6_TRAINER_2_DEFEATED",
                "EVENT_MN_GYM_6_WALK_LEFT",
            ],
        ),
    ] {
        ensure!(
            gate.event == event && gate.requires == requires,
            "dock gate requirements changed"
        );
    }
    let mut pack = base.clone();
    let mut room = crate::northstar_rival_arc::map(&pack, &gym.id)?;
    ensure!(
        room.attributes.width == 5
            && room.attributes.height == 8
            && room.attributes.tileset_name == "mn_gym_6"
            && room.events.warps.len() == 3
            && room.events.bg_events.is_empty()
            && !pack
                .data
                .maps
                .values()
                .any(|m| m.id != gym.id && m.attributes.tileset_name == "mn_gym_6")
            && !pack
                .data
                .field_moves
                .cut
                .replacements
                .contains_key("mn_gym_6")
            && !pack
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key("mn_gym_6"),
        "incompatible installed dock room"
    );
    for (id, at) in [
        ("MnBrainerdGymGuide", gym.guide_at),
        ("MnBrainerdGymLeader", gym.leader_at),
        ("MN_GYM_6_TRAINER_1", gym.trainers[0].at),
        ("MN_GYM_6_TRAINER_2", gym.trainers[1].at),
    ] {
        let actor = room
            .objects
            .iter()
            .find(|o| o.script == id)
            .context("dock actor")?;
        ensure!([actor.x, actor.y] == at, "unexpected actor {id}");
    }
    let original_collision = pack
        .data
        .tilesets
        .get("mn_gym_6")
        .context("installed dock collision")?
        .collision
        .clone();
    let original_blocks = room.blocks.clone();
    pack.data.tilesets.remove("mn_gym_6");
    for path in [
        "data/tilesets/mn_gym_6.json",
        "data/tilesets/mn_gym_6_metatiles.bin",
        "data/tilesets/mn_gym_6_palette_map.json",
        "gfx/tilesets/mn_gym_6.png",
        "gfx/tilesets/mn_gym_6.2bpp",
    ] {
        ensure!(
            pack.runtime_files.remove(path).is_some(),
            "missing private art {path}"
        );
    }
    pack.report
        .manifests
        .retain(|m| m != "minnesota-gym-6-art-v1");
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
        .get("mn_gym_6")
        .context("new dock collision")?
        .collision;
    for (index, (old, new)) in original_blocks.iter().zip(&room.blocks).enumerate() {
        if index != 6 * 5 + 3 {
            let before = original_collision
                .get(&format!("{old:02x}"))
                .context("original collision")?;
            let after = new_collision
                .get(&format!("{new:02x}"))
                .context("dock collision")?;
            for (quadrant, (old, new)) in before.iter().zip(after).enumerate() {
                ensure!(
                    old == new || (old == "WATER" && new == "BUOY"),
                    "unexpected collision change at block {index}, quadrant {quadrant}"
                );
            }
        }
    }
    for (label, index, x, y, old, symbol) in [
        ("MnBrainerdGymGate0", 4, "4", "8", "19", 'v'),
        ("MnBrainerdGymGate0Done", 0, "4", "8", "4", 'C'),
        ("MnBrainerdGymGate1", 6, "2", "8", "19", 'v'),
        ("MnBrainerdGymGate1Done", 0, "2", "8", "3", 'B'),
        ("MnBrainerdGymGate2", 6, "6", "4", "13", 'j'),
        ("MnBrainerdGymGate2Done", 0, "6", "4", "2", 'A'),
    ] {
        let commands = room
            .scripts
            .get_mut(label)
            .and_then(|v| v.as_array_mut())
            .context("dock gate script")?;
        let command = commands.get_mut(index).context("dock gate changeblock")?;
        ensure!(
            *command == json!({"command":"changeblock","args":[x,y,old]}),
            "unexpected gate program {label}"
        );
        let block = symbols.get(&symbol).context("dock gate art")?;
        *command = json!({"command":"changeblock","args":[x,y,block.to_string()]});
    }
    let trainer = room
        .scripts
        .get_mut("MN_GYM_6_TRAINER_1")
        .and_then(|v| v.as_array_mut())
        .context("Lorla script")?;
    ensure!(
        trainer.get(2)
            == Some(&json!({"command":"iftrue","args":["MnBrainerdGymFLOAT_WALKRouting"]})),
        "unexpected Lorla repeat program"
    );
    trainer[2] = json!({"command":"iftrue","args":["MN_GYM_6_TRAINER_1After"]});
    room.scripts
        .retain(|label, _| !label.starts_with("MnBrainerdGymFLOAT_WALKRouting"));
    crate::gym_routing::install(&mut room, gym, "MnBrainerdGymRefreshGates")?;
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    let occupied = room.objects.iter().map(|o| [o.x, o.y]).collect();
    for (first, second, mode) in [
        (false, false, None),
        (true, false, None),
        (true, false, Some(0)),
        (true, false, Some(1)),
        (true, true, Some(0)),
        (true, true, Some(1)),
    ] {
        let mut flags = BTreeSet::new();
        if first {
            flags.insert("EVENT_MN_GYM_6_TRAINER_1_DEFEATED".to_owned());
        }
        if second {
            flags.insert("EVENT_MN_GYM_6_TRAINER_2_DEFEATED".to_owned());
        }
        if let Some(index) = mode {
            flags.insert(gym.routers[0].modes[index].event.clone());
        }
        let mut phase = room.clone();
        for gate in &gym.gates {
            if gate.requires.iter().all(|r| flags.contains(r)) {
                phase.blocks[usize::from(gate.at[1]) * usize::from(room.attributes.width)
                    + usize::from(gate.at[0])] = symbols[&gate.open_symbol];
            }
        }
        let reached = crate::gym_lifts::reachable(&pack, &phase, gym, &occupied)?;
        let adjacent = |at: [u16; 2]| {
            reached
                .iter()
                .any(|p| p[0].abs_diff(at[0]) + p[1].abs_diff(at[1]) == 1)
        };
        ensure!(
            adjacent(gym.routers[0].at) && adjacent(gym.trainers[0].at),
            "control and first trainer must remain reachable"
        );
        ensure!(
            adjacent(gym.trainers[1].at)
                == (first && (mode == Some(0) || second && mode == Some(1))),
            "pier must control second trainer approach"
        );
        ensure!(
            adjacent(gym.leader_at) == (first && second && mode == Some(1)),
            "both wins and left pier must gate leader"
        );
        let water_reached = crate::authored_route_surface::reachable_map_with_surf(
            &pack,
            &phase,
            gym.arrival_core,
            &occupied,
        )?;
        let water_adjacent = |at: [u16; 2]| {
            water_reached
                .iter()
                .any(|p| p[0].abs_diff(at[0]) + p[1].abs_diff(at[1]) == 1)
        };
        ensure!(
            water_adjacent(gym.trainers[1].at)
                == (first && (mode == Some(0) || second && mode == Some(1))),
            "water travel must not bypass the second trainer's pier: first={first}, second={second}, mode={mode:?}, reachable={water_reached:?}"
        );
        ensure!(
            water_adjacent(gym.leader_at) == (first && second && mode == Some(1)),
            "water travel must not bypass the leader's pier: first={first}, second={second}, mode={mode:?}, reachable={water_reached:?}"
        );
        ensure!(
            gym.exit_tiles_core
                .iter()
                .all(|[x, y]| y.checked_sub(1).is_some_and(|y| reached.contains(&[*x, y]))),
            "both exits must remain reachable"
        );
    }
    authoring::text(
        &mut room,
        "MnBrainerdGymGuideText",
        &[
            "Beat LORLA, then",
            "use the blue winch",
            "to line up with",
            "BRAVEL. Beat him",
            "and come back to",
            "the winch. One",
            "more press moves",
            "the walk left to",
            "SIVREN.",
        ],
    );
    authoring::text(
        &mut room,
        "MN_GYM_6_TRAINER_1AfterText",
        &[
            "LORLA: Use the",
            "blue winch on the",
            "south pier. The",
            "first press brings",
            "the walk to the",
            "middle, where",
            "BRAVEL's waiting.",
        ],
    );
    authoring::text(
        &mut room,
        "MN_GYM_6_TRAINER_2AfterText",
        &[
            "BRAVEL: Go back to",
            "the blue winch.",
            "Press it again to",
            "shift the walk",
            "left to SIVREN.",
        ],
    );
    crate::gym_scripts::finish(&pack, &mut room)?;
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
