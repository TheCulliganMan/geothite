//! Sand terraces, visible stairs and wooden stair gates for the second gym.
use anyhow::{Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::gym_design::GymDesign;

const MANIFEST: &str = "minnesota-mankato-sand-terraces-v1";

/// Replaces Mankato's ambiguous wooden banks with a legible terrace course.
///
/// Original parties, trainer numbers, prior-badge permission, rewards, indexed
/// city doors, other maps and audio remain intact. No progress is granted.
/// Old in-room saves need a separate position review before migration.
///
/// # Errors
/// Returns an error for a different parent, changed battle/binding contract,
/// unsafe ledges, duplicate installation or invalid native payloads. Inputs
/// remain unchanged on failure.
pub fn revise_mankato_terraces(
    base: &CompiledGamePack,
    historical_json: &str,
    candidate_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash
            == "cd560f97f332b1f81070cbf82bb79d6d692c443ba49b609b4eb3ee6028ed6967"
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "current Rochester gate parent required"
    );
    let mut historical: Value = serde_json::from_str(historical_json)?;
    let candidate: Value = serde_json::from_str(candidate_json)?;
    ensure!(
        candidate["gyms"].as_array().is_some_and(|g| g.len() == 1),
        "one Mankato revision required"
    );
    let old = &historical["gyms"][1];
    let new = &candidate["gyms"][0];
    for field in [
        "slot",
        "id",
        "constant",
        "city",
        "leader",
        "trainer_class",
        "leader_sprite",
        "leader_at",
        "badge_flag",
        "requires",
        "reward",
        "party",
        "dimensions_metatiles",
        "arrival_core",
        "exit_tiles_core",
        "guide_at",
        "leader_court_locked",
        "walkthrough",
    ] {
        ensure!(
            old[field] == new[field],
            "second-gym contract changed: {field}"
        );
    }
    ensure!(
        new["gates"].as_array().is_some_and(|g| g.len() == 2),
        "two guarded stairs required"
    );
    for index in 0..2 {
        for field in ["at", "requires", "event"] {
            ensure!(
                old["gates"][index][field] == new["gates"][index][field],
                "stair qualification changed: {field}"
            );
        }
    }
    ensure!(
        new["ledges"].as_array().is_some_and(|l| l.len() == 3),
        "three native drops required"
    );
    for index in 0..3 {
        for field in [
            "id",
            "direction",
            "hop_from",
            "landing",
            "return_passage_core",
        ] {
            ensure!(
                old["ledges"][index][field] == new["ledges"][index][field],
                "native drop contract changed: {field}"
            );
        }
    }
    ensure!(
        new["trainers"].as_array().is_some_and(|t| t.len() == 2),
        "two original trainers required"
    );
    for index in 0..2 {
        for field in [
            "id",
            "name",
            "trainer_class",
            "sprite",
            "at",
            "party",
            "optional",
            "requires",
        ] {
            ensure!(
                old["trainers"][index][field] == new["trainers"][index][field],
                "practice battle contract changed: {field}"
            );
        }
    }
    historical["gyms"][1] = new.clone();
    let design: GymDesign = serde_json::from_value(historical)?;
    design.validate()?;
    let gym = &design.gyms[1];
    ensure!(
        gym.id == "MnMankatoGym"
            && gym.rows[6].starts_with("ltp")
            && gym.rows[2].chars().nth(6) == Some('t')
            && gym
                .art
                .get(&'t')
                .is_some_and(|a| a[0] == "champions_room" && a[1] == "21")
            && gym.gates[0].open_symbol == 'w'
            && gym.gates[1].open_symbol == '.',
        "both native stairs must be visible behind gates which open to floor"
    );
    let crate::gym_room_revision::RebuiltGym {
        mut pack,
        room,
        symbols,
    } = crate::gym_room_revision::rebuild(base, gym)?;
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
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
