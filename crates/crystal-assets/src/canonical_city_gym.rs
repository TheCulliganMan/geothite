//! Compare complete canonical gyms before changing only their exterior warp wires.
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_town_design::Room;
use crate::gym_design::Gym;
use crate::{CompiledGamePack, MapModule};

pub(crate) struct Binding<'a> {
    pub slot: u8,
    pub town: &'a str,
    pub constant: &'a str,
    pub return_warp: i16,
}

pub(crate) fn bind(
    pack: &CompiledGamePack,
    room: &Room,
    reference: &Value,
    source: &str,
    binding: Binding<'_>,
) -> Result<MapModule> {
    let source: Value = serde_json::from_str(source).context("canonical gym collection")?;
    let candidates = source["gyms"]
        .as_array()
        .context("gym collection")?
        .iter()
        .filter(|gym| gym["id"] == room.id)
        .collect::<Vec<_>>();
    ensure!(candidates.len() == 1, "one canonical city gym required");
    let value = candidates[0];
    ensure!(
        reference["path"] == "planning/minnesota/gyms/interiors.json"
            && reference["gym_id"] == room.id
            && reference["room_sha256"]
                == format!("{:x}", Sha256::digest(serde_json::to_vec(value)?)),
        "city canonical gym reference is stale"
    );
    let gym: Gym = serde_json::from_value(value.clone())?;
    ensure!(
        gym.slot == binding.slot
            && room.id == gym.id
            && room.native_constant == gym.constant
            && room.dimensions_metatiles == gym.dimensions_metatiles
            && room.tileset == gym.tileset()
            && room.rows.is_empty()
            && room.residents.is_empty()
            && room.signs.is_empty()
            && room.passive_arrival.index == 3
            && room.passive_arrival.tile == gym.arrival_core
            && room.exits.len() == 2
            && room.exits.iter().enumerate().all(|(i, exit)| {
                usize::from(exit.index) == i + 1
                    && exit.tile == gym.exit_tiles_core[i]
                    && exit.target == binding.town
                    && exit.target_warp == binding.return_warp
            }),
        "city must reference the complete canonical gym"
    );
    let native = pack
        .data
        .maps
        .get(&gym.id)
        .context("installed canonical gym")?;
    ensure!(
        native.events.warps.len() == 3 + gym.lifts.len() * 2
            && native.events.warps.iter().take(3).all(|w| {
                w.target_map_constant == "MN_GYM_REVIEW_HALL"
                    && w.target_map == "MN_GYM_REVIEW_HALL"
                    && (w.target_warp_id == 3 || w.target_warp_id == i16::from(gym.slot) * 3)
            }),
        "gym is missing, stale or already city-bound"
    );
    let mut comparison = pack.clone();
    let tileset = gym.tileset();
    comparison.data.tilesets.remove(&tileset);
    comparison
        .report
        .manifests
        .retain(|m| m != &format!("minnesota-gym-{}-art-v1", gym.slot));
    let files = [
        format!("data/tilesets/{tileset}.json"),
        format!("data/tilesets/{tileset}_metatiles.bin"),
        format!("data/tilesets/{tileset}_palette_map.json"),
        format!("gfx/tilesets/{tileset}.png"),
        format!("gfx/tilesets/{tileset}.2bpp"),
    ];
    for path in &files {
        comparison.runtime_files.remove(path);
    }
    let symbols = crate::gym_art::install(&mut comparison, &gym)?;
    ensure!(
        serde_json::to_value(
            symbols
                .iter()
                .map(|(s, b)| (*s, format!("{b:02x}")))
                .collect::<std::collections::BTreeMap<_, _>>()
        )? == reference["symbol_blocks"]
            && comparison.data.tilesets.get(&tileset) == pack.data.tilesets.get(&tileset)
            && files
                .iter()
                .all(|p| comparison.runtime_files.get(p) == pack.runtime_files.get(p)),
        "canonical gym native art or glyph allocation differs"
    );
    let battles = gym
        .trainers
        .iter()
        .map(|t| t.id.clone())
        .chain([gym.battle_id()])
        .collect::<Vec<_>>();
    for id in &battles {
        comparison.data.trainers.trainers.remove(id);
        comparison
            .data
            .story_event_script_constants
            .global
            .remove(id);
    }
    let index = if native.events.warps[0].target_warp_id == 3 {
        0
    } else {
        usize::from(gym.slot - 1)
    };
    let mut expected = crate::gym_publication::room(&comparison, &gym, &symbols, index)?;
    crate::gym_scripts::install(&mut comparison, &mut expected, &gym, &symbols)?;
    ensure!(expected == *native, "complete canonical gym module differs");
    for id in battles {
        ensure!(
            comparison.data.trainers.trainers.get(&id) == pack.data.trainers.trainers.get(&id),
            "canonical gym trainer differs: {id}"
        );
    }
    for warp in expected.events.warps.iter_mut().take(3) {
        warp.target_map_constant = binding.constant.into();
        warp.target_map = binding.constant.into();
        warp.target_warp_id = binding.return_warp;
    }
    Ok(expected)
}
