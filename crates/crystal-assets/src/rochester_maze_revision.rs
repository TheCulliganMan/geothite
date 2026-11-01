//! A readable three-turn planter course in the existing Rochester gym.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::gym_design::{Gym, GymDesign};

const MAP: &str = "MnRochesterGym";
const MANIFEST: &str = "minnesota-rochester-readable-maze-v1";

/// Replaces misleading planter stubs with three alternating, visible openings.
///
/// Only Rochester's terrain and two hints change. Existing actors, battles,
/// required trainer wins, delivery check, rewards, city doors, other maps,
/// source pixels, audio and recovery remain intact. Installation grants no
/// progress. Existing in-gym saves need a separate position review.
///
/// # Errors
/// Returns an error for a changed original generation, unsupported candidate
/// changes, inconsistent private art, repeated installation or unsafe native
/// geometry. Inputs remain unchanged on failure.
pub fn revise_rochester_maze(
    base: &CompiledGamePack,
    original_json: &str,
    candidate_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "maze revision already installed"
    );
    let original: Value = serde_json::from_str(original_json)?;
    let candidate: Value = serde_json::from_str(candidate_json)?;
    let old = &original["gyms"][0];
    ensure!(
        format!("{:x}", Sha256::digest(serde_json::to_vec(old)?))
            == "160c6bb9b03aa6baabfb4717d270a6866b2b6b49ebf1627c205921274f076f87",
        "original Rochester source generation changed"
    );
    ensure!(
        candidate["gyms"].as_array().is_some_and(|g| g.len() == 1),
        "one Rochester candidate required"
    );
    let new = &candidate["gyms"][0];
    let mut combined = original.clone();
    combined["gyms"][0] = new.clone();
    let design: GymDesign = serde_json::from_value(combined)?;
    design.validate()?;
    ensure!(design.gyms[0].id == MAP, "Rochester candidate required");
    let mut permitted = old.clone();
    permitted["rows"] = new["rows"].clone();
    permitted["guide"] = new["guide"].clone();
    permitted["trainers"][0]["after_lines"] = new["trainers"][0]["after_lines"].clone();
    ensure!(
        permitted == *new,
        "only planter rows and two hints may change"
    );
    let gym: &Gym = &design.gyms[0];
    ensure!(
        gym.rows
            == [
                "dtttttf", "l.....r", "l.....r", "guuuu.r", "l.....r", "l.uuuuh", "l.....r",
                "guuuu.r", "l.....r", ".L.s.R.",
            ],
        "three alternating planter rows required"
    );
    let installed = base.data.maps.get(MAP).context("installed Rochester gym")?;
    ensure!(
        installed.attributes.width == 7
            && installed.attributes.height == 10
            && installed.attributes.tileset_name == gym.tileset(),
        "installed gym generation changed"
    );
    let old_rows = old["rows"].as_array().context("original rows")?;
    let mut symbols = BTreeMap::new();
    let mut offset = 0;
    for row in old_rows {
        for symbol in row.as_str().context("original row")?.chars() {
            let block = *installed
                .blocks
                .get(offset)
                .context("original room extent")?;
            if let Some(previous) = symbols.insert(symbol, block) {
                ensure!(previous == block, "original {symbol} art is inconsistent");
            }
            offset += 1;
        }
    }
    ensure!(
        offset == installed.blocks.len(),
        "original room extent changed"
    );
    let mut room = installed.clone();
    room.blocks = gym
        .rows
        .iter()
        .flat_map(|row| row.chars())
        .map(|symbol| {
            symbols
                .get(&symbol)
                .copied()
                .context("candidate needs new art")
        })
        .collect::<Result<_>>()?;
    for (label, lines) in [
        ("MnRochesterGymGuideText", &gym.guide),
        ("MN_GYM_1_TRAINER_1AfterText", &gym.trainers[0].after_lines),
    ] {
        ensure!(
            room.scripts.contains_key(label),
            "missing existing hint {label}"
        );
        authoring::text(
            &mut room,
            label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    crate::gym_scripts::finish(base, &mut room)?;
    crate::gym_publication::validate(base, &room, gym, &symbols)?;
    let mut pack = base.clone();
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
