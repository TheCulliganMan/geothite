//! Winter travel, cave landmarks, and the opening prairie road.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;
use crate::authored_town_publication;

/// Builds the next native Minnesota edition from its reviewed campaign parent.
///
/// # Errors
/// Rejects an unexpected parent or inconsistent terrain, scripts or native art.
pub fn revise_northern_trails(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-campaign-quality-v1"),
        "expected the Minnesota campaign quality revision"
    );
    let mut pack = base.clone();
    crate::northstar_basin_controls::apply(&mut pack)?;
    crate::northstar_cave_journey::apply(&mut pack)?;
    crate::northstar_voyageurs_crossing::install(&mut pack)?;
    crate::northstar_voyageurs_watch::install(&mut pack)?;
    crate::northstar_tamarack_hint::apply(&mut pack)?;
    face_prairie_road(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-northern-trails-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn face_prairie_road(pack: &mut CompiledGamePack) -> Result<()> {
    const TESS: &str = "MN_R02_T01";
    let mut room = pack
        .data
        .maps
        .get("MnR02C00R00")
        .context("Rochester farm road")?
        .clone();
    for x in [52, 53] {
        ensure!(
            matches!(kind(pack, &room, [x, 21])?, "FLOOR" | "TALL_GRASS")
                && !room.objects.iter().any(|o| (o.x, o.y) == (x, 21)),
            "Tess needs two open passing places on the road"
        );
    }
    let actor = room
        .objects
        .iter_mut()
        .find(|o| o.script == TESS)
        .context("Tess")?;
    ensure!(
        (actor.x, actor.y, actor.radius) == (51, 21, 2)
            && actor.spritemovedata == "SPRITEMOVEDATA_STANDING_LEFT"
            && actor.sightline_direction_override.as_deref() == Some("LEFT")
            && actor.object_type == "OBJECTTYPE_TRAINER",
        "expected Tess at the west shoulder"
    );
    let event = room
        .map_event_section_commands
        .iter_mut()
        .find(|e| e.command == "object_event" && e.args.get(11).map(String::as_str) == Some(TESS))
        .context("Tess's object event")?;
    ensure!(
        event.args[3] == actor.spritemovedata && event.args[10] == "2",
        "Tess's native event disagrees with her placement"
    );
    actor.spritemovedata = "SPRITEMOVEDATA_STANDING_RIGHT".into();
    actor.sightline_direction_override = Some("RIGHT".into());
    actor.radius = 1;
    event.args[3].clone_from(&actor.spritemovedata);
    event.args[10] = "1".into();
    authored_town_publication::replace(pack, room)
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result, ensure};
    use crystal_core::world::map::TilePosition;

    #[test]
    #[ignore = "requires an external MN_NORTHERN_TRAILS_PACK"]
    fn northern_trails_native_sight_and_guide_records() -> Result<()> {
        let path =
            std::env::var_os("MN_NORTHERN_TRAILS_PACK").context("northern trails pack path")?;
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let road = "MnR02C00R00";
        for (x, sees_tess) in [(52, true), (53, false)] {
            let session = pack
                .data
                .overworld_session(road, TilePosition::new(x, 21), 0)?;
            let seen = session
                .check_trainer_sight_checked()?
                .is_some_and(|t| t.script == "MN_R02_T01");
            ensure!(seen == sees_tess, "incorrect prairie challenge at ({x},21)");
        }
        for (name, guide) in [
            ("MnInternationalFalls", "MnFallsSkiGuide"),
            ("MnRainyRiverIcefields", "MnRainySkiGuide"),
        ] {
            let stored = pack.data.maps.get(name).context("stored ski stop")?;
            let loaded = pack.data.assemble_map_module_from_compiled_payloads(name)?;
            ensure!(
                stored.objects == loaded.objects
                    && stored.events == loaded.events
                    && stored.map_event_section_commands == loaded.map_event_section_commands,
                "native trailhead changed after loading {name}"
            );
            ensure!(
                loaded.objects.iter().filter(|o| o.script == guide).count() == 1,
                "{name} lost its ski guide"
            );
        }
        Ok(())
    }
}
