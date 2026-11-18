//! Paired, visible guard rotations on the cargo depot's two crate aisles.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;
use crate::authored_town_publication;

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnIceCargoBasement")
        .context("cargo basement")?
        .clone();
    // Both posts keep all four native sight checks. The western searcher turns
    // toward the crate aisles; the manifest lookout turns the opposite way.
    let posts = [
        (
            "MN_ICE_HQ_CARGO_SEARCH",
            [5, 9],
            "SPRITEMOVEDATA_STANDING_DOWN",
            "SPRITEMOVEDATA_SPINCLOCKWISE",
        ),
        (
            "MN_ICE_HQ_CARGO_MANIFEST",
            [24, 13],
            "SPRITEMOVEDATA_STANDING_LEFT",
            "SPRITEMOVEDATA_SPINCOUNTERCLOCKWISE",
        ),
    ];
    for (script, at, before, movement) in posts {
        ensure!(
            kind(pack, &room, at)? == "FLOOR" && room.trainer_scripts.contains_key(script),
            "cargo lookout must stand on the aisle and own a sight battle"
        );
        let actor = room
            .objects
            .iter_mut()
            .find(|actor| actor.script == script)
            .context("cargo lookout")?;
        let event = room
            .map_event_section_commands
            .iter_mut()
            .find(|row| {
                row.command == "object_event"
                    && row.args.get(11).map(String::as_str) == Some(script)
            })
            .context("cargo lookout map event")?;
        ensure!(
            [actor.x, actor.y] == at
                && actor.object_type == "OBJECTTYPE_TRAINER"
                && actor.sightline_direction_override.is_none()
                && actor.spritemovedata == before
                && event.args.get(3).map(String::as_str) == Some(before),
            "cargo lookout preimage changed"
        );
        actor.spritemovedata = movement.into();
        event.args[3] = movement.into();
    }
    authored_town_publication::replace(pack, room)
}
