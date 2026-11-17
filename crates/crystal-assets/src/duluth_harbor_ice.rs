//! A distinct eastward ice course and readable gates for Duluth's gym.
use anyhow::{Context, Result, ensure};
use crystal_core::models::LearnedMove;
use serde_json::json;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::gym_design::GymDesign;

const PARENT: &str = "c5100712c38c7cca51495a9d86b5ff1b8aae6c5a060503bb88e222afb79febbc";
const MANIFEST: &str = "minnesota-duluth-harbor-ice-review-v1";

/// Builds an unselected eight-gym review pack with Duluth's revised ice course.
///
/// Existing actors, city doors, story and gate conditions are retained. Two
/// retired species are replaced with their active Minnesota counterparts.
/// Existing in-room block overrides need migration before selecting this pack.
///
/// # Errors
/// Rejects a changed parent, course, party, gate script, collision or output.
pub fn build_duluth_harbor_ice(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == PARENT,
        "seven-gym review parent required"
    );
    let design: GymDesign = serde_json::from_str(source)?;
    ensure!(
        design.schema_version == 1 && design.gyms.len() == 1,
        "one ice course required"
    );
    let gym = &design.gyms[0];
    ensure!(
        gym.id == "MnDuluthGym"
            && gym.slot == 8
            && gym.dimensions_metatiles == [7, 10]
            && gym.arrival_core == [6, 17]
            && gym.exit_tiles_core == [[6, 19], [7, 19]]
            && gym.guide_at == [10, 17]
            && gym.leader_at == [8, 3]
            && gym.rows
                == [
                    "WWWWWWW", "l.#...r", "l..XWWW", "BB.V.rr", "lIaKlBr", "lIrKbIB", "lIrKlbr",
                    "BrYKBZB", "..L.R..", "...s..."
                ]
            && gym.trainers.len() == 3
            && gym.gates.len() == 4
            && gym.gates[0].at == [2, 7]
            && gym.gates[0].open_symbol == '.'
            && gym.gates[0].requires == ["EVENT_MN_GYM_8_TRAINER_1_DEFEATED"]
            && gym.gates[1].at == [5, 7]
            && gym.gates[1].open_symbol == 'l'
            && gym.gates[1].requires == ["EVENT_MN_GYM_8_TRAINER_1_DEFEATED"]
            && gym.gates[2].at == [3, 3]
            && gym.gates[2].open_symbol == '.'
            && gym.gates[2].requires == ["EVENT_MN_GYM_8_TRAINER_2_DEFEATED"]
            && gym.gates[3].at == [3, 2]
            && gym.gates[3].open_symbol == '.'
            && gym.gates[3].requires
                == [
                    "EVENT_MN_GYM_8_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_8_TRAINER_2_DEFEATED",
                    "EVENT_MN_GYM_8_TRAINER_3_DEFEATED"
                ]
            && gym.gates.iter().map(|g| g.event.as_str()).eq([
                "EVENT_MN_GYM_8_WEST_RETURN_OPEN",
                "EVENT_MN_GYM_8_EAST_ENTRY_OPEN",
                "EVENT_MN_GYM_8_EAST_RETURN_OPEN",
                "EVENT_MN_GYM_8_COURT_OPEN"
            ])
            && gym.cut_gates.is_empty()
            && gym.winds.is_empty()
            && gym.barriers.is_empty(),
        "incompatible harbor ice course"
    );
    let mut pack = base.clone();
    let mut room = crate::northstar_rival_arc::map(&pack, &gym.id)?;
    ensure!(
        room.attributes.width == 7
            && room.attributes.height == 10
            && room.attributes.tileset_name == "mn_gym_8"
            && room.events.warps.len() == 3
            && !pack
                .data
                .maps
                .values()
                .any(|m| m.id != gym.id && m.attributes.tileset_name == "mn_gym_8")
            && !pack
                .data
                .field_moves
                .cut
                .replacements
                .contains_key("mn_gym_8")
            && !pack
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key("mn_gym_8"),
        "unexpected ice room parent"
    );
    for (id, at) in [
        ("MnDuluthGymGuide", [10, 17]),
        ("MnDuluthGymLeader", [8, 3]),
        ("MN_GYM_8_TRAINER_1", [3, 4]),
        ("MN_GYM_8_TRAINER_2", [9, 6]),
        ("MN_GYM_8_TRAINER_3", [5, 6]),
    ] {
        let actor = room
            .objects
            .iter()
            .find(|o| o.script == id)
            .context("ice course actor")?;
        ensure!([actor.x, actor.y] == at, "ice course actor moved: {id}");
    }
    for (trainer_id, index, retired, replacement, level, expected_moves) in [
        (
            "MN_GYM_8_TRAINER_1",
            0,
            "WINKFIN",
            "THAWBONNET",
            39,
            ["ICY_WIND", "PSYBEAM", "AMNESIA", "ICE_BEAM"],
        ),
        (
            "MN_GYM_8",
            3,
            "TIPULORD",
            "VERNACAPE",
            45,
            ["ICE_BEAM", "PSYBEAM", "FUTURE_SIGHT", "AMNESIA"],
        ),
    ] {
        let member = if trainer_id == "MN_GYM_8" {
            &gym.party[index]
        } else {
            &gym.trainers[0].party[index]
        };
        ensure!(
            member.species == replacement
                && member.level == level
                && member.moves == expected_moves,
            "unexpected active replacement for {trainer_id}"
        );
        let learnset = pack
            .data
            .learnsets
            .get(replacement)
            .context("replacement learnset")?;
        let moves = member
            .moves
            .iter()
            .map(|name| {
                ensure!(
                    learnset
                        .iter()
                        .any(|entry| entry.0 <= level && &entry.1 == name),
                    "{replacement} cannot learn {name} at {level}"
                );
                Ok(LearnedMove {
                    name: name.clone(),
                    current_pp: pack.data.moves.get(name).context("replacement move")?.pp,
                    pp_ups: 0,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let trainer = pack
            .data
            .trainers
            .trainers
            .get_mut(trainer_id)
            .context("ice trainer")?;
        let old = trainer.party.get_mut(index).context("retired party slot")?;
        ensure!(
            old.species == retired && old.level == level,
            "unexpected retired party for {trainer_id}"
        );
        old.species = replacement.into();
        old.moves = moves;
    }
    for (id, party) in std::iter::once(("MN_GYM_8", &gym.party))
        .chain(gym.trainers.iter().map(|t| (t.id.as_str(), &t.party)))
    {
        let original = &pack
            .data
            .trainers
            .trainers
            .get(id)
            .context("ice trainer party")?
            .party;
        ensure!(
            original.len() == party.len()
                && original
                    .iter()
                    .zip(party)
                    .all(|(old, new)| old.species == new.species
                        && old.level == new.level
                        && old.moves.iter().map(|m| &m.name).eq(new.moves.iter())),
            "ice source must match installed party {id}"
        );
    }
    let old_collision = pack
        .data
        .tilesets
        .get("mn_gym_8")
        .context("ice collision")?
        .collision
        .clone();
    let old_blocks = room.blocks.clone();
    pack.data.tilesets.remove("mn_gym_8");
    for path in [
        "data/tilesets/mn_gym_8.json",
        "data/tilesets/mn_gym_8_metatiles.bin",
        "data/tilesets/mn_gym_8_palette_map.json",
        "gfx/tilesets/mn_gym_8.png",
        "gfx/tilesets/mn_gym_8.2bpp",
    ] {
        ensure!(
            pack.runtime_files.remove(path).is_some(),
            "missing ice art {path}"
        );
    }
    pack.report
        .manifests
        .retain(|m| m != "minnesota-gym-8-art-v1");
    let symbols = crate::gym_art::install(&mut pack, gym)?;
    room.attributes.border_block = u8::try_from(symbols[&'#'])?;
    room.blocks = gym
        .rows
        .iter()
        .flat_map(|row| row.chars().map(|s| symbols[&s]))
        .collect();
    let collision = &pack.data.tilesets["mn_gym_8"].collision;
    for (index, (old, new)) in old_blocks.iter().zip(&room.blocks).enumerate() {
        let before = old_collision
            .get(&format!("{old:02x}"))
            .context("original ice collision")?;
        let after = collision
            .get(&format!("{new:02x}"))
            .context("new ice collision")?;
        if index == 6 * 7 + 5 {
            ensure!(
                before == &["ICE", "ICE", "ICE", "ICE"] && after == &["ICE", "FLOOR", "ICE", "ICE"],
                "east turn must add exactly one rough stopping tile"
            );
        } else {
            ensure!(
                before == after,
                "unexpected ice collision change at {index}"
            );
        }
    }
    for (label, index, x, y, old, symbol) in [
        ("MnDuluthGymGate0", 2, "4", "14", "1", '.'),
        ("MnDuluthGymGate0Done", 0, "4", "14", "9", 'Y'),
        ("MnDuluthGymGate1", 2, "10", "14", "13", 'l'),
        ("MnDuluthGymGate1Done", 0, "10", "14", "10", 'Z'),
        ("MnDuluthGymGate2", 2, "6", "6", "1", '.'),
        ("MnDuluthGymGate2Done", 0, "6", "6", "6", 'V'),
        ("MnDuluthGymGate3", 6, "6", "4", "1", '.'),
        ("MnDuluthGymGate3Done", 0, "6", "4", "8", 'X'),
    ] {
        let commands = room
            .scripts
            .get_mut(label)
            .and_then(|v| v.as_array_mut())
            .context("ice gate script")?;
        let command = commands.get_mut(index).context("ice gate changeblock")?;
        ensure!(
            *command == json!({"command":"changeblock","args":[x,y,old]}),
            "unexpected gate program {label}"
        );
        *command = json!({"command":"changeblock","args":[x,y,symbols[&symbol].to_string()]});
    }
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    authoring::text(
        &mut room,
        "MnDuluthGymGuideText",
        &gym.guide.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    for trainer in &gym.trainers {
        authoring::text(
            &mut room,
            &format!("{}AfterText", trainer.id),
            &trainer
                .after_lines
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        );
    }
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
