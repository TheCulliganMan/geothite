//! Rebuild private native art while retaining installed room identities and scenes.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::gym_design::Gym;
use crate::northstar_gym_design::RoomContract;

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    gym: &Gym,
    contract: &RoomContract,
    story: &str,
) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(&gym.id)
        .context("installed gym")?
        .clone();
    let symbols = replace_art(pack, gym)?;
    room.attributes.width = gym.dimensions_metatiles[0];
    room.attributes.height = gym.dimensions_metatiles[1];
    room.attributes.border_block = u8::try_from(symbols[&'#'])?;
    room.blocks = gym
        .rows
        .iter()
        .flat_map(|row| row.chars().map(|glyph| symbols[&glyph]))
        .collect();
    let positions: Vec<_> = gym
        .exit_tiles_core
        .into_iter()
        .chain([gym.arrival_core])
        .chain(gym.lifts.iter().flat_map(|lift| lift.ends_core))
        .collect();
    ensure!(
        positions.len() == room.events.warps.len(),
        "retain indexed gym destinations"
    );
    for (warp, [x, y]) in room.events.warps.iter_mut().zip(positions) {
        warp.x = x;
        warp.y = y;
    }
    let mut placed = 0;
    for actor in &mut room.objects {
        let id = actor
            .object_identifier
            .as_deref()
            .context("saved gym actor identity")?;
        let at = if id == format!("{}Leader", gym.id) {
            gym.leader_at
        } else if id == format!("{}Guide", gym.id) {
            gym.guide_at
        } else {
            gym.trainers
                .iter()
                .find(|trainer| trainer.id == id)
                .with_context(|| format!("unclassified saved gym actor {id}"))?
                .at
        };
        actor.x = at[0];
        actor.y = at[1];
        placed += 1;
    }
    ensure!(
        placed == gym.trainers.len() + 2,
        "preserve complete gym cast"
    );
    crate::northstar_gym_mechanics::install(pack, &mut room, gym, contract, &symbols)?;
    crate::northstar_gym_dialogue::install(&mut room, gym, story)?;
    if room.id == "MnBrainerdGym" && !gym.wading_water.is_empty() {
        ensure!(gym.routers.is_empty() && gym.gates.is_empty(), "open lake has no dock controls");
        room.scripts.retain(|label, _| {
            ![
                "MnBrainerdGymFLOAT_WALKRouting",
                "MnBrainerdGymSOUTH_WINCHRouting",
                "MnBrainerdGymNORTH_WINCHRouting",
            ].iter().any(|prefix| label.starts_with(prefix))
        });
    }
    refresh_control_menus(&mut room)?;
    crate::northstar_shore_roster_repair::validate_visible_species(pack, &room)?;
    crate::northstar_gym_validation::validate(pack, &room, gym, contract, &symbols)?;
    crate::northstar_story_revision::publish(pack, room)
}

/// Rebuild the menu records consumed by the runtime from the installed scripts.
pub(super) fn refresh_control_menus(room: &mut crate::MapModule) -> Result<()> {
    room.script_menu_definitions = crate::parse_script_menu_definitions(&room.id, &room.scripts)?;
    room.script_vertical_menus = crate::parse_script_vertical_menus(
        &room.id, &room.scripts, &room.script_menu_definitions,
    )?;
    Ok(())
}

fn replace_art(pack: &mut CompiledGamePack, gym: &Gym) -> Result<BTreeMap<char, u16>> {
    let tileset = gym.tileset();
    ensure!(
        pack.data
            .maps
            .values()
            .all(|map| map.id == gym.id || map.attributes.tileset_name != tileset),
        "gym art must remain private"
    );
    ensure!(
        pack.data.maps[&gym.id].attributes.tileset_name == tileset,
        "installed gym tileset identity"
    );
    pack.data.field_moves.cut.replacements.remove(&tileset);
    ensure!(
        !pack
            .data
            .field_moves
            .whirlpool
            .replacements
            .contains_key(&tileset),
        "unexpected gym field behavior"
    );
    pack.data
        .tilesets
        .remove(&tileset)
        .context("installed private gym tileset")?;
    for path in [
        format!("data/tilesets/{tileset}.json"),
        format!("data/tilesets/{tileset}_metatiles.bin"),
        format!("data/tilesets/{tileset}_palette_map.json"),
        format!("gfx/tilesets/{tileset}.png"),
        format!("gfx/tilesets/{tileset}.2bpp"),
    ] {
        ensure!(
            pack.runtime_files.remove(&path).is_some(),
            "missing private art {path}"
        );
    }
    pack.report
        .manifests
        .retain(|name| *name != format!("minnesota-gym-{}-art-v1", gym.slot));
    crate::gym_art::install(pack, gym)
}
