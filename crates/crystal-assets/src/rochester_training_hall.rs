//! Candidate Rochester training hall over the selected regional pack.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::gym_design::GymDesign;

const PARENT: &str = "834d4be07c45079fdc81f1777856a37058d064b4188f548e31616684436da5e3";
const MANIFEST: &str = "minnesota-rochester-training-hall-review-v1";

/// Builds a review candidate while retaining the selected pack's scripts and teams.
///
/// This changes Rochester's private art, blocks, four actor positions and three
/// room-specific text bodies. It retains room dimensions, doors, trainer data,
/// progression and all other maps. Existing in-room saves and block overrides
/// require migration before this candidate can replace the selected pack.
///
/// # Errors
/// Rejects an incompatible parent, layout or private tileset, and invalid output.
pub fn build_rochester_training_hall(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == PARENT,
        "selected R06 parent required"
    );
    let design: GymDesign = serde_json::from_str(source)?;
    ensure!(
        design.schema_version == 1 && design.gyms.len() == 1,
        "one hall required"
    );
    let gym = &design.gyms[0];
    ensure!(
        gym.id == "MnRochesterGym"
            && gym.slot == 1
            && gym.dimensions_metatiles == [7, 10]
            && gym.rows.len() == 10
            && gym.rows.iter().all(|row| row.chars().count() == 7)
            && gym
                .rows
                .iter()
                .flat_map(|row| row.chars())
                .all(|symbol| gym.art.contains_key(&symbol))
            && gym.art.contains_key(&'#')
            && gym.gates.is_empty()
            && gym.cut_gates.is_empty()
            && gym.winds.is_empty()
            && gym.barriers.is_empty(),
        "incompatible hall source"
    );
    let mut pack = base.clone();
    let mut room = crate::northstar_rival_arc::map(&pack, &gym.id)?;
    ensure!(
        room.attributes.width == 7
            && room.attributes.height == 10
            && room.attributes.tileset_name == "mn_gym_1"
            && room.script_block_changes.is_empty()
            && !pack
                .data
                .maps
                .values()
                .any(|map| map.id != gym.id && map.attributes.tileset_name == "mn_gym_1")
            && !pack
                .data
                .field_moves
                .cut
                .replacements
                .contains_key("mn_gym_1")
            && !pack
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key("mn_gym_1"),
        "incompatible installed hall"
    );
    pack.data.tilesets.remove("mn_gym_1");
    for path in [
        "data/tilesets/mn_gym_1.json",
        "data/tilesets/mn_gym_1_metatiles.bin",
        "data/tilesets/mn_gym_1_palette_map.json",
        "gfx/tilesets/mn_gym_1.png",
        "gfx/tilesets/mn_gym_1.2bpp",
    ] {
        ensure!(
            pack.runtime_files.remove(path).is_some(),
            "missing private art {path}"
        );
    }
    pack.report
        .manifests
        .retain(|m| m != "minnesota-gym-1-art-v1");
    let symbols = crate::gym_art::install(&mut pack, gym)?;
    room.attributes.border_block = u8::try_from(symbols[&'#'])?;
    room.blocks = gym
        .rows
        .iter()
        .flat_map(|row| row.chars().map(|s| symbols[&s]))
        .collect();
    for (id, at) in [
        ("MnRochesterGymGuide", gym.guide_at),
        ("MnRochesterGymLeader", gym.leader_at),
        (
            "MN_GYM_1_TRAINER_1",
            gym.trainers.first().context("Tavel")?.at,
        ),
        (
            "MN_GYM_1_TRAINER_2",
            gym.trainers.get(1).context("Elvik")?.at,
        ),
    ] {
        let actor = room
            .objects
            .iter_mut()
            .find(|object| object.script == id)
            .context("hall actor")?;
        actor.x = at[0];
        actor.y = at[1];
    }
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    for label in [
        "MN_GYM_1_TRAINER_1AfterText",
        "MN_GYM_1_TRAINER_2AfterText",
        "MnRochesterGymGuideText",
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
