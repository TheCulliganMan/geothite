//! Repair passive arrivals in Minnesota's authored Mart interiors.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::StepOptions;
use crystal_core::world::session::OverworldSession;

use crate::CompiledGamePack;

const MANIFEST: &str = "minnesota-mart-arrivals-v1";
const ROOMS: [(&str, u16); 11] = [
    ("MnBemidjiMart", 6),
    ("MnBrainerdMart", 6),
    ("MnDuluthMart", 6),
    ("MnElyMart", 2),
    ("MnGrandRapidsMart", 6),
    ("MnMankatoMart", 6),
    ("MnMinneapolisMart", 6),
    ("MnRochesterMart", 6),
    ("MnStCloudMart", 6),
    ("MnStPaulMart", 3),
    ("MnWinonaMart", 6),
];

/// Move affected passive Mart arrivals to the floor directly above the carpet.
///
/// The parent must have the reviewed room and reciprocal door geometry. All
/// other content is retained. The resulting pack has a new identity.
///
/// # Errors
/// Returns an error if a room or door differs from the expected geometry, its
/// arrival or exit is obstructed, or pack validation fails.
pub fn correct_minnesota_mart_arrivals(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "Mart arrivals already repaired"
    );
    let mut pack = base.clone();
    for (name, x) in ROOMS {
        let mut room = pack.data.maps.get(name).context("missing Mart")?.clone();
        ensure!(room.events.warps.len() == 3, "unexpected warps in {name}");
        let arrival = &room.events.warps[2];
        ensure!(
            arrival.index == 3 && (arrival.x, arrival.y) == (x, 5),
            "unexpected arrival in {name}"
        );
        ensure!(
            room.events.warps[..2].iter().any(|w| (w.x, w.y) == (x, 7)),
            "missing exit carpet in {name}"
        );
        ensure!(
            !room
                .objects
                .iter()
                .any(|object| (object.x, object.y) == (x, 6)),
            "occupied arrival in {name}"
        );
        ensure!(
            crate::authored_town_geometry::kind(base, &room, [x, 6])? == "FLOOR",
            "arrival is not floor in {name}"
        );
        room.events.warps[2].y = 6;
        let event = room
            .map_event_section_commands
            .iter_mut()
            .filter(|command| command.command == "warp_event")
            .nth(2)
            .context("missing serialized arrival")?;
        ensure!(
            event.args.first() == Some(&x.to_string())
                && event.args.get(1).is_some_and(|y| y == "5"),
            "unexpected serialized arrival in {name}"
        );
        event.args[1] = "6".into();
        crate::authored_town_publication::replace(&mut pack, room)?;
    }
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    verify_doors(&pack)?;
    Ok(pack)
}

fn verify_doors(pack: &CompiledGamePack) -> Result<()> {
    let data = &pack.data;
    for (name, x) in ROOMS {
        let room = &data.maps[name];
        let constant = room
            .attributes
            .map_constant
            .as_deref()
            .context("Mart constant")?;
        let session = |map: &crate::MapModule, x: u16, y: u16| -> Result<OverworldSession> {
            Ok(OverworldSession::with_events_and_objects(
                data.overworld_map(&map.id)?,
                map.events.clone(),
                map.objects.clone(),
                data.tileset_collision(&map.attributes.tileset_name)?,
                TilePosition::new(i16::try_from(x)?, i16::try_from(y)?),
            ))
        };
        let mut doors = Vec::new();
        for map in data.maps.values() {
            for warp in &map.events.warps {
                if warp.target_map_constant == constant
                    && warp.target_warp_id == 3
                    && session(map, warp.x, warp.y)?
                        .check_warp_checked()?
                        .is_some()
                {
                    doors.push((map, warp));
                }
            }
        }
        ensure!(
            doors.len() == 1,
            "expected one entrance to {name}, found {}",
            doors.len()
        );
        let (outside, door) = doors[0];
        let trigger = session(outside, door.x, door.y)?
            .check_warp_checked()?
            .context("entrance is not an active door")?;
        let entry = data.resolve_warp_transition(&trigger)?;
        ensure!(
            entry.destination.map_name == name
                && entry.destination.tile == TilePosition::new(i16::try_from(x)?, 6),
            "incorrect entry into {name}"
        );
        let mut inside = session(room, x, 6)?;
        ensure!(
            inside.check_warp_checked()?.is_none(),
            "arrival activates warp in {name}"
        );
        inside.step_checked(
            Direction::Down,
            StepOptions {
                force_step_after_turn: true,
                ..StepOptions::default()
            },
        )?;
        ensure!(
            inside.player.tile == TilePosition::new(i16::try_from(x)?, 7),
            "cannot reach exit carpet in {name}"
        );
        inside.step_checked(Direction::Down, StepOptions::default())?;
        let exit = inside
            .check_warp_checked()?
            .context("cannot walk from arrival to exit")?;
        let destination = data.resolve_warp_transition(&exit)?;
        ensure!(
            destination.destination.map_name == outside.id,
            "incorrect return from {name}"
        );
        for carpet in &room.events.warps[..2] {
            let exit = session(room, carpet.x, carpet.y)?
                .check_warp_checked()?
                .context("inactive exit carpet")?;
            let destination = data.resolve_warp_transition(&exit)?;
            ensure!(
                destination.destination.map_name == outside.id
                    && destination.destination.tile.x == i16::try_from(door.x)?
                    && destination.destination.tile.y == i16::try_from(door.y)? + 1
                    && session(
                        outside,
                        destination.destination.warp.x,
                        destination.destination.warp.y
                    )?
                    .check_warp_checked()?
                    .is_none(),
                "incorrect exit from {name}"
            );
        }
        let return_warp = &destination.destination.warp;
        let mut returned = session(outside, return_warp.x, return_warp.y)?;
        let reentry = returned.step_and_check_warp_checked(
            Direction::Up,
            StepOptions {
                force_step_after_turn: true,
                ..StepOptions::default()
            },
        )?;
        let reentry = data.resolve_warp_transition(
            &reentry
                .warp
                .context("cannot walk from return tile into Mart")?,
        )?;
        ensure!(
            reentry.destination == entry.destination,
            "incorrect reentry into {name}"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ROOMS, correct_minnesota_mart_arrivals};

    #[test]
    #[ignore = "requires the selected external Minnesota review pack"]
    fn mart_arrivals_round_trip_in_selected_pack() {
        let path = std::env::var_os("CRYSTAL_MINNESOTA_MART_TEST_PACK")
            .expect("set CRYSTAL_MINNESOTA_MART_TEST_PACK to the selected review pack");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified parent");
        let corrected = correct_minnesota_mart_arrivals(&base).expect("all Mart journeys pass");
        assert_eq!(corrected.compiled_audio(), base.compiled_audio());
        assert_eq!(corrected.runtime_files(), base.runtime_files());
        for (name, map) in &base.data().maps {
            if !ROOMS.iter().any(|(room, _)| *room == name.as_str()) {
                assert_eq!(
                    corrected.data().maps.get(name),
                    Some(map),
                    "unrelated map {name}"
                );
            }
        }
        assert!(correct_minnesota_mart_arrivals(&corrected).is_err());
    }
}
