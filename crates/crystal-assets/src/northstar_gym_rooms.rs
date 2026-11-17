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
            let trainer = gym.trainers
                .iter()
                .find(|trainer| trainer.id == id)
                .with_context(|| format!("unclassified saved gym actor {id}"))?;
            if !gym.wading_water.is_empty() {
                actor.sprite.clone_from(&trainer.sprite);
            }
            trainer.at
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
        validate_lake_shore(pack, gym, contract)?;
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

/// NPCs stay on native dry ground, including after walking toward a challenger.
fn validate_lake_shore(pack: &CompiledGamePack, gym: &Gym, contract: &RoomContract) -> Result<()> {
    let dry = |[x, y]: [u16; 2]| -> Result<()> {
        let symbol = gym.rows.get(usize::from(y / 2))
            .and_then(|row| row.chars().nth(usize::from(x / 2)))
            .context("Gym shore position outside the room")?;
        let [tileset, block] = &gym.art[&symbol];
        let quadrant = usize::from(2 * (y % 2) + x % 2);
        ensure!(
            pack.data.tilesets[tileset].collision[block][quadrant] == "FLOOR",
            "shore actor would stand in water or on a barrier at ({x},{y})"
        );
        Ok(())
    };
    dry(gym.leader_at)?;
    dry(gym.guide_at)?;
    for trainer in &gym.trainers {
        let sight = &contract.sightlines[&trainer.id];
        let (dx, dy): (i32, i32) = match sight.direction.as_str() {
            "up" => (0, -1),
            "right" => (1, 0),
            "down" => (0, 1),
            "left" => (-1, 0),
            _ => anyhow::bail!("unknown shore trainer direction"),
        };
        for distance in 0..i32::from(sight.radius) {
            let x = u16::try_from(i32::from(trainer.at[0]) + dx * distance)?;
            let y = u16::try_from(i32::from(trainer.at[1]) + dy * distance)?;
            dry([x, y]).with_context(|| format!("shore approach for {}", trainer.name))?;
        }
    }
    Ok(())
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
