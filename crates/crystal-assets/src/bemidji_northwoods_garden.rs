//! Native northwoods planting and an earned return through Bemidji's gym.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::models::LearnedMove;
use serde_json::json;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::gym_design::GymDesign;

const PARENT: &str = "c9ba8cf78477a844a12056a08bc8974f38a949da67c59b49b17b282d4a59a567";
const MANIFEST: &str = "minnesota-bemidji-northwoods-gym-review-v1";

/// Builds the unselected seven-gym review pack with Bemidji's native garden.
///
/// Existing actors, story, city warps and Cut events remain owned by the parent.
/// Brindel's retired species is replaced by an active Minnesota species. The
/// victory gate opens using the existing leader defeat flag.
/// Saved in-room block overrides need migration before selecting this pack.
///
/// # Errors
/// Rejects a changed parent, course, native Cut recipe, scripts, geometry or
/// invalid compiled output.
pub fn build_bemidji_northwoods_garden(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == PARENT,
        "six-gym canoe review parent required"
    );
    let design: GymDesign = serde_json::from_str(source)?;
    ensure!(
        design.schema_version == 1 && design.gyms.len() == 1,
        "one garden required"
    );
    let gym = &design.gyms[0];
    ensure!(
        gym.id == "MnBemidjiGym"
            && gym.slot == 7
            && gym.dimensions_metatiles == [6, 10]
            && gym.arrival_core == [4, 17]
            && gym.exit_tiles_core == [[4, 19], [5, 19]]
            && gym.guide_at == [9, 17]
            && gym.leader_at == [5, 3]
            && gym.rows
                == [
                    "WWWWWW", "pffffp", "p.ddFp", "T##C#T", "pFdd.p", "p.#d.p", "Tc#G#T", "a====b",
                    "=L==R=", "==s==="
                ]
            && gym.cut_gates.len() == 2
            && gym.cut_gates[0].tree_core == [3, 12]
            && gym.cut_gates[1].tree_core == [7, 6]
            && gym.gates.len() == 1
            && gym.gates[0].at == [3, 6]
            && gym.gates[0].requires == ["EVENT_MN_GYM_7_DEFEATED"]
            && gym.gates[0].event == "EVENT_MN_GYM_7_RETURN_OPEN"
            && gym.gates[0].open_symbol == '='
            && gym.trainers.len() == 3
            && gym.court_requires
                == [
                    "EVENT_MN_GYM_7_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_7_TRAINER_2_DEFEATED"
                ],
        "incompatible garden course"
    );
    let mut pack = base.clone();
    let mut room = crate::northstar_rival_arc::map(&pack, &gym.id)?;
    ensure!(
        room.attributes.width == 6
            && room.attributes.height == 10
            && room.attributes.tileset_name == "mn_gym_7"
            && room.events.warps.len() == 3
            && !pack
                .data
                .maps
                .values()
                .any(|m| m.id != gym.id && m.attributes.tileset_name == "mn_gym_7")
            && !pack
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key("mn_gym_7"),
        "unexpected garden parent"
    );
    for (id, at) in [
        ("MnBemidjiGymGuide", [9, 17]),
        ("MnBemidjiGymLeader", [5, 3]),
        ("MN_GYM_7_TRAINER_1", [7, 14]),
        ("MN_GYM_7_TRAINER_2", [8, 9]),
        ("MN_GYM_7_TRAINER_3", [3, 9]),
    ] {
        let actor = room
            .objects
            .iter()
            .find(|o| o.script == id)
            .context("garden actor")?;
        ensure!([actor.x, actor.y] == at, "garden actor moved: {id}");
    }
    let replacement = &gym.trainers[2].party;
    ensure!(
        gym.trainers[2].id == "MN_GYM_7_TRAINER_3"
            && replacement.len() == 1
            && replacement[0].species == "VERGLOOM"
            && replacement[0].level == 36
            && replacement[0].moves == ["VINE_WHIP", "LEECH_SEED", "FAINT_ATTACK", "RAZOR_LEAF"],
        "expected the active northwoods replacement for Brindel"
    );
    let learnset = pack
        .data
        .learnsets
        .get("VERGLOOM")
        .context("Vergloom learnset")?;
    let moves = replacement[0]
        .moves
        .iter()
        .map(|name| {
            ensure!(
                learnset
                    .iter()
                    .any(|entry| entry.0 <= 36 && &entry.1 == name),
                "Vergloom cannot learn {name} at level 36"
            );
            Ok(LearnedMove {
                name: name.clone(),
                current_pp: pack.data.moves.get(name).context("Vergloom move")?.pp,
                pp_ups: 0,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let trainer = pack
        .data
        .trainers
        .trainers
        .get_mut("MN_GYM_7_TRAINER_3")
        .context("Brindel")?;
    ensure!(
        trainer.party.len() == 1
            && trainer.party[0].species == "GLISNARE"
            && trainer.party[0].level == 36,
        "unexpected retired Brindel party"
    );
    trainer.party[0].species = "VERGLOOM".into();
    trainer.party[0].moves = moves;
    for (id, party) in std::iter::once(("MN_GYM_7", &gym.party))
        .chain(gym.trainers.iter().map(|t| (t.id.as_str(), &t.party)))
    {
        let original = &pack
            .data
            .trainers
            .trainers
            .get(id)
            .context("garden trainer")?
            .party;
        ensure!(
            original.len() == party.len()
                && original
                    .iter()
                    .zip(party)
                    .all(|(old, new)| old.species == new.species
                        && old.level == new.level
                        && old.moves.iter().map(|m| &m.name).eq(new.moves.iter())),
            "garden source must retain current party {id}"
        );
    }
    let old_collision = pack
        .data
        .tilesets
        .get("mn_gym_7")
        .context("garden collision")?
        .collision
        .clone();
    let old_blocks = room.blocks.clone();
    ensure!(
        pack.data
            .field_moves
            .cut
            .replacements
            .remove("mn_gym_7")
            .is_some(),
        "garden Cut rules missing"
    );
    pack.data.tilesets.remove("mn_gym_7");
    for path in [
        "data/tilesets/mn_gym_7.json",
        "data/tilesets/mn_gym_7_metatiles.bin",
        "data/tilesets/mn_gym_7_palette_map.json",
        "gfx/tilesets/mn_gym_7.png",
        "gfx/tilesets/mn_gym_7.2bpp",
    ] {
        ensure!(
            pack.runtime_files.remove(path).is_some(),
            "missing garden art {path}"
        );
    }
    pack.report
        .manifests
        .retain(|m| m != "minnesota-gym-7-art-v1");
    let symbols = crate::gym_art::install(&mut pack, gym)?;
    room.attributes.border_block = u8::try_from(symbols[&'#'])?;
    room.blocks = gym
        .rows
        .iter()
        .flat_map(|row| row.chars().map(|s| symbols[&s]))
        .collect();
    let collision = &pack.data.tilesets["mn_gym_7"].collision;
    for (index, (old, new)) in old_blocks.iter().zip(&room.blocks).enumerate() {
        if index == 5 * 6 + 3 {
            ensure!(
                old_collision[&format!("{old:02x}")]
                    .iter()
                    .all(|c| c == "WALL")
                    && collision[&format!("{new:02x}")]
                        .iter()
                        .all(|c| c == "FLOOR"),
                "return approach must clear one hedge"
            );
        } else {
            ensure!(
                old_collision.get(&format!("{old:02x}")) == collision.get(&format!("{new:02x}")),
                "garden collision changed at {index}"
            );
        }
    }
    for (label, x, y) in [
        ("MnBemidjiGymSOUTH_HEDGEPrunedUnchanged", "2", "12"),
        ("MnBemidjiGymNORTH_HEDGEPrunedUnchanged", "6", "6"),
    ] {
        let commands = room
            .scripts
            .get_mut(label)
            .and_then(|v| v.as_array_mut())
            .context("pruned hedge script")?;
        ensure!(
            commands.get(2) == Some(&json!({"command":"changeblock","args":[x,y,"11"]})),
            "unexpected Cut refresh {label}"
        );
        commands[2] = json!({"command":"changeblock","args":[x,y,symbols[&'o'].to_string()]});
    }
    crate::gym_scripts::event(&mut pack, "EVENT_MN_GYM_7_RETURN_OPEN")?;
    let c = authoring::command;
    ensure!(
        !room.scripts.contains_key("MnBemidjiGymReturnGate"),
        "return script occupied"
    );
    room.scripts.insert(
        "MnBemidjiGymReturnGate".into(),
        json!([
            c("checkevent", &["EVENT_MN_GYM_7_DEFEATED"]),
            c("iffalse", &["MnBemidjiGymReturnGateClosed"]),
            c("changeblock", &["6", "12", &symbols[&'='].to_string()]),
            c("setevent", &["EVENT_MN_GYM_7_RETURN_OPEN"]),
            c("end", &[]),
        ]),
    );
    room.scripts.insert(
        "MnBemidjiGymReturnGateClosed".into(),
        json!([
            c("changeblock", &["6", "12", &symbols[&'G'].to_string()]),
            c("clearevent", &["EVENT_MN_GYM_7_RETURN_OPEN"]),
            c("end", &[]),
        ]),
    );
    let refresh = room
        .scripts
        .get_mut("MnBemidjiGymRefreshGates")
        .and_then(|v| v.as_array_mut())
        .context("garden callback")?;
    ensure!(
        refresh.first()
            == Some(&json!({"command":"scall","args":["MnBemidjiGymSOUTH_HEDGEPrunedUnchanged"]})),
        "unexpected garden refresh"
    );
    refresh.insert(0, c("scall", &["MnBemidjiGymReturnGate"]));
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    let occupied = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut pruned = room.clone();
    for tree in &gym.cut_gates {
        pruned.blocks
            [usize::from(tree.tree_core[1] / 2) * 6 + usize::from(tree.tree_core[0] / 2)] =
            symbols[&tree.replacement_symbol];
    }
    let before =
        crate::authored_route_surface::walking_map_distances(&pack, &pruned, [5, 4], &occupied)?;
    pruned.blocks[6 * 6 + 3] = symbols[&'='];
    let after =
        crate::authored_route_surface::walking_map_distances(&pack, &pruned, [5, 4], &occupied)?;
    ensure!(
        after.get(&gym.arrival_core).context("open return")? + 4
            <= *before.get(&gym.arrival_core).context("closed return")?,
        "victory gate must shorten the return"
    );
    authoring::text(
        &mut room,
        "MnBemidjiGymGuideText",
        &gym.guide.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    authoring::text(
        &mut room,
        "MnBemidjiGymLeaderAfterText",
        &gym.after_lines
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
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
