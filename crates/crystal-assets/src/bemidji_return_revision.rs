//! A visible victory gate shortens the return through the two native Cut hedges.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::authored_route_surface::walking_map_distances;
use crate::gym_design::GymDesign;

const MANIFEST: &str = "minnesota-bemidji-victory-return-v1";

/// Adds one gold return gate to Bemidji's otherwise native Cut garden.
///
/// Both original trees, replacements, trainer requirements, battles/numbers,
/// actors, badge permission, reward and indexed city doors remain intact. The
/// gate opens only after Orlis; one hedge block above it becomes ordinary floor.
/// Other content and source audio stay unchanged. Installation grants no progress;
/// existing saves require migration review before using the revised private art.
///
/// # Errors
/// Returns an error for another parent, changed course/battle contracts, an early
/// or ineffective return, bypassable Cut trees, duplicate installation or invalid
/// compiled payloads.
pub fn revise_bemidji_return(
    base: &CompiledGamePack,
    historical_json: &str,
    candidate_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash
            == "9e67016986aef09f2cae8fafd3d5c8e9970a3ca5653f6ab9466c4752f3e1e053"
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "current Brainerd dock-control parent required"
    );
    let mut historical: Value = serde_json::from_str(historical_json)?;
    let candidate: Value = serde_json::from_str(candidate_json)?;
    ensure!(
        candidate["gyms"].as_array().is_some_and(|g| g.len() == 1),
        "one Bemidji garden required"
    );
    let new = &candidate["gyms"][0];
    let mut expected = historical["gyms"][6].clone();
    expected["rows"][5] = "p.#..p".into();
    expected["rows"][6] = "#c#G##".into();
    expected["art"]["G"] = json!(["underground", "0b"]);
    expected["gates"] = json!([{"at":[3,6],"requires":["EVENT_MN_GYM_7_DEFEATED"],"open_symbol":"=","event":"EVENT_MN_GYM_7_RETURN_OPEN"}]);
    expected["guide"] = new["guide"].clone();
    expected["after_lines"] = new["after_lines"].clone();
    ensure!(
        expected == *new,
        "only the victory gate, cleared approach and return hints may change"
    );
    historical["gyms"][6] = new.clone();
    let design: GymDesign = serde_json::from_value(historical)?;
    design.validate()?;
    let gym = &design.gyms[6];
    let crate::gym_room_revision::RebuiltGym {
        mut pack,
        mut room,
        symbols,
    } = crate::gym_room_revision::rebuild(base, gym)?;
    // Native event parsing represents an unconditional coordinate scene as -1.
    // Keep the cached room identical to its uncached native reconstruction.
    for coordinate in &mut room.events.coord_events {
        if coordinate.scene_id.is_empty() {
            coordinate.scene_id = "-1".into();
        }
    }
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    let occupied = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut pruned = room.clone();
    for tree in &gym.cut_gates {
        pruned.blocks[usize::from(tree.tree_core[1] / 2) * usize::from(room.attributes.width)
            + usize::from(tree.tree_core[0] / 2)] = symbols[&tree.replacement_symbol];
    }
    let before = walking_map_distances(&pack, &pruned, [5, 4], &occupied)?;
    let gate = &gym.gates[0];
    pruned.blocks
        [usize::from(gate.at[1]) * usize::from(room.attributes.width) + usize::from(gate.at[0])] =
        symbols[&gate.open_symbol];
    let after = walking_map_distances(&pack, &pruned, [5, 4], &occupied)?;
    let old = before
        .get(&gym.arrival_core)
        .context("ordinary return route")?;
    let new = after
        .get(&gym.arrival_core)
        .context("victory return route")?;
    ensure!(
        *new + 4 <= *old,
        "gold gate must shorten the real return by at least four steps: {old} -> {new}"
    );
    crate::authored_town_publication::replace(&mut pack, room)?;
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
