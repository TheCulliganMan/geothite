//! Repair passive arrivals in Minnesota's authored house interiors.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::StepOptions;
use crystal_core::world::session::OverworldSession;

use crate::CompiledGamePack;

const MANIFEST: &str = "minnesota-house-arrivals-v1";
fn affected_rooms(pack: &CompiledGamePack) -> Vec<(String, u16, u16)> {
    let data = &pack.data;
    data.maps
        .values()
        .filter_map(|room| {
            if !room.id.starts_with("Mn")
                || !matches!(
                    room.attributes.tileset_name.as_str(),
                    "house" | "players_house" | "traditional_house"
                )
            {
                return None;
            }
            let constant = room.attributes.map_constant.as_deref()?;
            let arrival = room.events.warps.get(2)?;
            if arrival.index != 3
                || !room.events.warps[..2].iter().any(|exit| {
                    exit.x == arrival.x
                        && exit.y == arrival.y + 2
                        && exit.target_map_constant != constant
                })
            {
                return None;
            }
            let has_entrance =
                data.maps.values().any(|outside| {
                    outside.events.warps.iter().any(|door| {
                        door.target_map_constant == constant && door.target_warp_id == 3
                    })
                });
            has_entrance.then(|| (room.id.clone(), arrival.x, arrival.y))
        })
        .collect()
}

/// Move affected passive house arrivals to the floor directly above the carpet.
///
/// The parent must have the reviewed room and reciprocal door geometry. All
/// other content is retained. The resulting pack has a new identity.
///
/// # Errors
/// Returns an error if a room or door differs from the expected geometry, its
/// arrival or exit is obstructed, or pack validation fails.
pub fn correct_minnesota_house_arrivals(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "house arrivals already repaired"
    );
    let rooms = affected_rooms(base);
    ensure!(!rooms.is_empty(), "no affected house arrivals found");
    let mut pack = base.clone();
    for (name, x, y) in &rooms {
        let (x, y) = (*x, *y);
        let mut room = pack.data.maps.get(name).context("missing house")?.clone();
        let arrival = &room.events.warps[2];
        ensure!(
            arrival.index == 3 && (arrival.x, arrival.y) == (x, y),
            "unexpected arrival in {name}"
        );
        ensure!(
            room.events.warps[..2]
                .iter()
                .any(|w| (w.x, w.y) == (x, y + 2)),
            "missing exit carpet in {name}"
        );
        ensure!(
            !room
                .objects
                .iter()
                .any(|object| (object.x, object.y) == (x, y + 1)),
            "occupied arrival in {name}"
        );
        ensure!(
            crate::authored_town_geometry::kind(base, &room, [x, y + 1])? == "FLOOR",
            "arrival is not floor in {name}"
        );
        room.events.warps[2].y = y + 1;
        let event = room
            .map_event_section_commands
            .iter_mut()
            .filter(|command| command.command == "warp_event")
            .nth(2)
            .context("missing serialized arrival")?;
        ensure!(
            event.args.first() == Some(&x.to_string())
                && event
                    .args
                    .get(1)
                    .is_some_and(|value| value == &y.to_string()),
            "unexpected serialized arrival in {name}"
        );
        event.args[1] = (y + 1).to_string();
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
    verify_doors(&pack, &rooms)?;
    Ok(pack)
}

fn verify_doors(pack: &CompiledGamePack, rooms: &[(String, u16, u16)]) -> Result<()> {
    let data = &pack.data;
    for (name, x, y) in rooms {
        let (x, y) = (*x, *y);
        let room = &data.maps[name];
        let constant = room
            .attributes
            .map_constant
            .as_deref()
            .context("house constant")?;
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
            entry.destination.map_name == *name
                && entry.destination.tile
                    == TilePosition::new(i16::try_from(x)?, i16::try_from(y + 1)?),
            "incorrect entry into {name}"
        );
        let mut inside = session(room, x, y + 1)?;
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
            inside.player.tile == TilePosition::new(i16::try_from(x)?, i16::try_from(y + 2)?),
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
                .context("cannot walk from return tile into house")?,
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
    use super::{affected_rooms, correct_minnesota_house_arrivals};

    #[test]
    #[ignore = "requires the selected external Minnesota review pack"]
    fn house_arrivals_round_trip_in_selected_pack() {
        let path = std::env::var_os("CRYSTAL_MINNESOTA_HOUSE_TEST_PACK")
            .expect("set CRYSTAL_MINNESOTA_HOUSE_TEST_PACK to the selected review pack");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified parent");
        let rooms = affected_rooms(&base);
        let corrected = correct_minnesota_house_arrivals(&base).expect("all house journeys pass");
        assert_eq!(corrected.compiled_audio(), base.compiled_audio());
        assert_eq!(corrected.runtime_files(), base.runtime_files());
        for (name, map) in &base.data().maps {
            if !rooms.iter().any(|(room, _, _)| room == name) {
                assert_eq!(
                    corrected.data().maps.get(name),
                    Some(map),
                    "unrelated map {name}"
                );
            }
        }
        assert!(correct_minnesota_house_arrivals(&corrected).is_err());
    }
}
