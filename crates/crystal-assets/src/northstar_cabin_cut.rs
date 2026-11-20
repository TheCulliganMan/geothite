//! A Cut-through from the Route 15 cabin to the hidden Ether by the lake.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::authored_route_surface::walking_map_distances;
use crate::{CompiledGamePack, authored_town_publication};

pub(super) fn open_shore_path(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnR15C00R00")
        .context("Shiner Narrows")?
        .clone();
    ensure!(
        room.attributes.tileset_name == "mn_shiner_narrows",
        "expected the Shiner Narrows shore"
    );
    let width = usize::from(room.attributes.width);
    // Keep the north sapling. Carry its narrow woodland path through the
    // evergreen row. South of the cabin the isolated fence and second sapling
    // become an ordinary stand of trees beside the public pier.
    for (x, y, old, new) in [
        (24, 4, 0x0f, 0x6d),
        (24, 8, 0x77, 0x01),
        (24, 9, 0x34, 0x0f),
    ] {
        let block = room
            .blocks
            .get_mut(y * width + x)
            .context("cabin trail bounds")?;
        ensure!(*block == old, "unexpected cabin terrain at {x},{y}");
        *block = new;
    }
    let obstacles: BTreeSet<_> = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .chain(room.events.warps.iter().map(|w| [w.x, w.y]))
        .collect();
    let gate = 5 * width + 24;
    let sapling = room.blocks[gate];
    let replacement = pack
        .data
        .field_moves
        .cut
        .replacements
        .get(&room.attributes.tileset_name)
        .and_then(|rules| rules.get(&sapling))
        .context("native cabin sapling")?;
    ensure!(
        replacement.variant == "tree" && replacement.replacement_block_id == 0x6d,
        "unexpected cabin Cut replacement"
    );
    let doorstep = [53, 13];
    let ether_approach = [49, 5];
    for (block, length) in [(sapling, 24), (replacement.replacement_block_id, 12)] {
        room.blocks[gate] = block;
        for (from, to) in [(doorstep, ether_approach), (ether_approach, doorstep)] {
            let distance = walking_map_distances(pack, &room, from, &obstacles)?;
            ensure!(
                distance.get(&to) == Some(&length),
                "Route 15 shore path {from:?} to {to:?} must take {length} steps"
            );
        }
    }
    room.blocks[gate] = sapling;
    authored_town_publication::replace(pack, room)
}
