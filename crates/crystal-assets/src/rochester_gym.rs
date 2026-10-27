//! Explicit canonical Gym1 refresh before binding its doors to Rochester.
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

use crate::gym_design::GymDesign;
use crate::{CompiledGamePack, MapModule};

pub(super) fn rebuild(pack: &mut CompiledGamePack, source: &str) -> Result<MapModule> {
    let design: GymDesign = serde_json::from_str(source).context("captured canonical Gym1")?;
    let raw: serde_json::Value = serde_json::from_str(source)?;
    ensure!(
        format!("{:x}", Sha256::digest(serde_json::to_vec(&raw["gyms"][0])?))
            == "160c6bb9b03aa6baabfb4717d270a6866b2b6b49ebf1627c205921274f076f87",
        "captured canonical Gym1 source differs"
    );
    ensure!(
        design.schema_version == 1 && !design.status.is_empty() && design.gyms.len() == 1,
        "one captured canonical Gym1 required"
    );
    let gym = &design.gyms[0];
    ensure!(
        gym.slot == 1
            && gym.id == "MnRochesterGym"
            && gym.constant == "MN_ROCHESTER_GYM"
            && gym.leader == "VAREL"
            && gym.dimensions_metatiles == [7, 10]
            && gym.requires == ["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"],
        "canonical first-gym identity and delivery prerequisite required"
    );
    ensure!(
        gym.rows.len() == 10
            && gym.rows.iter().all(|r| r.chars().count() == 7)
            && gym
                .rows
                .iter()
                .flat_map(|r| r.chars())
                .all(|s| gym.art.contains_key(&s)),
        "complete canonical Gym1 rows and native recipes required"
    );
    let old = pack.data.maps.get(&gym.id).context("installed old Gym1")?;
    ensure!(
        old.attributes.width == 8
            && old.attributes.height == 10
            && old.attributes.tileset_name == "mn_gym_1"
            && old.events.warps.len() == 3
            && old
                .events
                .warps
                .iter()
                .all(|w| w.target_map_constant == "MN_GYM_REVIEW_HALL" && w.target_warp_id == 3),
        "unsupported Gym1 generation or already city-bound gym"
    );
    let tileset = gym.tileset();
    ensure!(
        !pack
            .data
            .maps
            .values()
            .any(|m| m.id != gym.id && m.attributes.tileset_name == tileset)
            && !pack
                .data
                .field_moves
                .cut
                .replacements
                .contains_key(&tileset)
            && !pack
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key(&tileset),
        "Gym1 art must be private with no field replacement aliases"
    );
    pack.data.tilesets.remove(&tileset);
    for path in [
        format!("data/tilesets/{tileset}.json"),
        format!("data/tilesets/{tileset}_metatiles.bin"),
        format!("data/tilesets/{tileset}_palette_map.json"),
        format!("gfx/tilesets/{tileset}.png"),
        format!("gfx/tilesets/{tileset}.2bpp"),
    ] {
        ensure!(
            pack.runtime_files.remove(&path).is_some(),
            "missing owned Gym1 art {path}"
        );
    }
    pack.report
        .manifests
        .retain(|m| m != "minnesota-gym-1-art-v1");
    for id in std::iter::once(gym.battle_id()).chain(gym.trainers.iter().map(|t| t.id.clone())) {
        ensure!(
            pack.data.trainers.trainers.remove(&id).is_some(),
            "missing original trainer {id}"
        );
        ensure!(
            pack.data
                .story_event_script_constants
                .global
                .remove(&id)
                .is_some(),
            "missing trainer constant {id}"
        );
    }
    let symbols = crate::gym_art::install(pack, gym)?;
    let mut module = crate::gym_publication::room(pack, gym, &symbols, 0)?;
    crate::gym_scripts::install(pack, &mut module, gym, &symbols)?;
    crate::gym_publication::validate(pack, &module, gym, &symbols)?;
    // Actual city bindings replace only the three review-hall destinations.
    for warp in &mut module.events.warps {
        warp.target_map_constant = "MN_ROCHESTER".into();
        warp.target_map = "MN_ROCHESTER".into();
        warp.target_warp_id = 6;
    }
    Ok(module)
}
