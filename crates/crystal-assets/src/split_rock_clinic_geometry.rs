//! Clinic residents, original route boundaries and the public southern approach.
use anyhow::{Result, ensure};

use crate::authored_town_design::Room;
use crate::authored_town_geometry::{kind, reachable};
use crate::{CompiledGamePack, MapModule};

pub(crate) fn residents(room: &Room) -> Result<()> {
    for (actor, (id, sprite, tile, approach, service)) in room.residents.iter().zip([
        (
            "MnSplitRockNurse",
            "SPRITE_NURSE",
            [3, 1],
            [3, 3],
            Some("heal_party"),
        ),
        (
            "MN_SPLIT_ROCK_SHORE_GRANDPA",
            "SPRITE_FISHER",
            [1, 5],
            [2, 5],
            None,
        ),
        (
            "MN_SPLIT_ROCK_DECKHAND",
            "SPRITE_SAILOR",
            [6, 4],
            [6, 5],
            None,
        ),
    ]) {
        ensure!(
            actor.id == id
                && actor.sprite == sprite
                && actor.tile == tile
                && actor.approach == Some(approach)
                && actor.service.as_deref() == service
                && !actor.lines.is_empty()
                && actor
                    .lines
                    .iter()
                    .all(|line| !line.is_empty() && line.len() <= 18),
            "clinic needs its nurse and both readable local conversations"
        );
    }
    ensure!(
        room.residents[0].counter_tile == Some([3, 2]),
        "native nurse counter required"
    );
    Ok(())
}

pub(crate) fn shore(pack: &CompiledGamePack, old: &MapModule, new: &MapModule) -> Result<()> {
    ensure!(
        old.attributes == new.attributes && old.objects == new.objects,
        "original shore connections, location and actors must stay exact"
    );
    for y in 0..48 {
        for x in 0..64 {
            if x == 0 || x == 63 || y == 0 || y == 47 {
                ensure!(
                    kind(pack, old, [x, y])? == kind(pack, new, [x, y])?,
                    "clinic cannot open or change a regional walking/Surf edge"
                );
            }
        }
    }
    let seen = reachable(pack, new, [33, 34], false)?;
    ensure!(
        kind(pack, new, [33, 33])? == "DOOR"
            && kind(pack, new, [33, 34])? == "FLOOR"
            && kind(pack, new, [30, 34])? == "WALL"
            && kind(pack, new, [31, 34])? == "FLOOR"
            && [[33, 33], [31, 34], [34, 47], [35, 47], [36, 47], [37, 47]]
                .iter()
                .all(|point| seen.contains(point)),
        "the safe clinic porch, sign and all four original southern lanes must connect on foot"
    );
    Ok(())
}
