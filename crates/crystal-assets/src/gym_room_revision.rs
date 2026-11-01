//! Rebuild one private gym room while retaining its city and battle identities.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::gym_design::Gym;
use crate::{CompiledGamePack, MapModule};

pub(crate) struct RebuiltGym {
    pub pack: CompiledGamePack,
    pub room: MapModule,
    pub symbols: BTreeMap<char, u16>,
}

/// Prepares an unpublished room; the caller validates its specific course.
pub(crate) fn rebuild(base: &CompiledGamePack, gym: &Gym) -> Result<RebuiltGym> {
    let installed = base.data.maps.get(&gym.id).context("installed gym")?;
    let tileset = gym.tileset();
    ensure!(
        !base
            .data
            .maps
            .values()
            .any(|m| m.id != gym.id && m.attributes.tileset_name == tileset)
            && base
                .data
                .field_moves
                .cut
                .replacements
                .contains_key(&tileset)
                == !gym.cut_gates.is_empty()
            && !base
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key(&tileset),
        "revision requires private art with matching owned Cut behavior"
    );
    ensure!(
        installed.events.warps.len() == 3 + gym.lifts.len() * 2,
        "revision must retain the indexed city doors and reciprocal pad count"
    );
    let mut pack = base.clone();
    if !gym.cut_gates.is_empty() {
        pack.data.field_moves.cut.replacements.remove(&tileset);
    }
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
            "missing private art {path}"
        );
    }
    let art_manifest = format!("minnesota-gym-{}-art-v1", gym.slot);
    pack.report.manifests.retain(|m| m != &art_manifest);
    let symbols = crate::gym_art::install(&mut pack, gym)?;
    let mut room = crate::gym_publication::room(&pack, gym, &symbols, usize::from(gym.slot - 1))?;
    room.attributes = installed.attributes.clone();
    room.attributes.width = gym.dimensions_metatiles[0];
    room.attributes.height = gym.dimensions_metatiles[1];
    room.attributes.border_block = u8::try_from(symbols[&'#'])?;
    for (new, old) in room.events.warps.iter_mut().zip(&installed.events.warps) {
        let (x, y) = (new.x, new.y);
        *new = old.clone();
        new.x = x;
        new.y = y;
    }
    let trainers = std::iter::once(gym.battle_id())
        .chain(gym.trainers.iter().map(|t| t.id.clone()))
        .map(|id| {
            let original = pack
                .data
                .trainers
                .trainers
                .remove(&id)
                .context("original battle")?;
            let number = pack
                .data
                .story_event_script_constants
                .global
                .remove(&id)
                .context("original trainer number")?;
            Ok((id, original, number))
        })
        .collect::<Result<Vec<_>>>()?;
    crate::gym_scripts::install(&mut pack, &mut room, gym, &symbols)?;
    for (id, original, number) in trainers {
        pack.data.trainers.trainers.insert(id.clone(), original);
        pack.data
            .story_event_script_constants
            .global
            .insert(id, number);
    }
    Ok(RebuiltGym {
        pack,
        room,
        symbols,
    })
}
