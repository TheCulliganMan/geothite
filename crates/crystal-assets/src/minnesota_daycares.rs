//! Two ordinary native houses with independent Crystal daycare services.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, CompiledMapExtension, RuntimeSpawnPoint};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct House {
    exterior: String,
    room: String,
    constant: String,
    facade_metatile: [u16; 2],
    id: String,
    settlement: String,
}

const MANIFEST: &str = "minnesota-independent-daycares-v1";

/// Adds Mankato and Bemidji houses, using the engine's independent custody keys.
/// Preserves existing actors, story programs, entrances, encounters and audio.
/// Returns a separate pack identity; the source pack is unchanged.
///
/// # Errors
/// Rejects duplicate installation, missing assets, occupied plots, disconnected
/// doors or caretaker approaches, and an invalid resulting runtime pack.
pub fn build_minnesota_daycares(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "daycares already installed"
    );
    let mut result = base.clone();
    install(&mut result)?;
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

fn install(pack: &mut CompiledGamePack) -> Result<()> {
    crate::minnesota_interior_lineage::require_compact_coast(pack)?;
    let houses: Vec<House> =
        serde_json::from_str(include_str!("../../../planning/minnesota/daycares.json"))?;
    ensure!(houses.len() == 2, "two independent daycare houses required");
    for house in houses {
        ensure!(
            crystal_core::day_care::location_id(&house.room) == Some(house.id.as_str()),
            "house custody identity mismatch"
        );
        let outside_id = house.exterior.as_str();
        let room_id = house.room.as_str();
        let constant = house.constant.as_str();
        let [x, y] = house.facade_metatile;
        ensure!(
            !pack.data.maps.contains_key(room_id),
            "daycare house already exists"
        );
        let mut outside = pack
            .data
            .maps
            .get(outside_id)
            .context("daycare route")?
            .clone();
        let door = [x * 2 + 1, y * 2 + 3];
        let landing = [door[0], door[1] + 1];
        let sign = [door[0] + 1, landing[1]];
        // Native complete small-house facade, a dry porch and a readable sign.
        for (tile, block) in [
            ([x, y], 0x18),
            ([x + 1, y], 0x19),
            ([x, y + 1], 0x16),
            ([x + 1, y + 1], 0x1e),
            ([x, y + 2], 0x01),
            ([x + 1, y + 2], 0x45),
        ] {
            ensure!(
                tile[0] < outside.attributes.width && tile[1] < outside.attributes.height,
                "house extent"
            );
            for o in &outside.objects {
                ensure!(
                    [o.x / 2, o.y / 2] != tile,
                    "house overlaps actor {}",
                    o.script
                );
            }
            ensure!(
                !outside
                    .events
                    .warps
                    .iter()
                    .any(|w| [w.x / 2, w.y / 2] == tile)
                    && !outside
                        .events
                        .coord_events
                        .iter()
                        .any(|e| [e.x / 2, e.y / 2] == tile)
                    && !outside
                        .events
                        .bg_events
                        .iter()
                        .any(|e| [e.x / 2, e.y / 2] == tile),
                "house overlaps existing route event"
            );
            let index =
                usize::from(tile[1]) * usize::from(outside.attributes.width) + usize::from(tile[0]);
            // An occupied or non-walkable plot must be redesigned, never bulldozed.
            let old = outside.blocks[index];
            let collision = &pack.data.tilesets[&outside.attributes.tileset_name].collision
                [&format!("{old:02x}")];
            ensure!(
                collision.iter().all(|kind| kind == "FLOOR"),
                "daycare plot is not empty floor"
            );
            outside.blocks[index] = block;
        }
        ensure!(
            crate::authored_town_geometry::kind(pack, &outside, door)? == "DOOR",
            "native daycare door"
        );
        ensure!(
            crate::authored_town_geometry::kind(pack, &outside, landing)? == "FLOOR",
            "dry daycare return"
        );
        let entry = i16::try_from(outside.events.warps.len() + 1)?;
        let exit = entry + 1;
        authoring::warp(&mut outside, (door[0], door[1]), constant, 3);
        authoring::warp(&mut outside, (landing[0], landing[1]), constant, 3);
        let sign_label = format!("{room_id}Sign");
        let sign_text = format!("{sign_label}Text");
        authoring::text(
            &mut outside,
            &sign_text,
            &[
                &house.settlement,
                "POKEMON DAY-CARE",
                "Pick up partners",
                "at this house.",
                "100 plus 100 per",
                "level gained.",
            ],
        );
        outside.scripts.insert(
            sign_label.clone(),
            serde_json::json!([authoring::command("jumptext", &[&sign_text])]),
        );
        outside.events.bg_events.push(BackgroundEvent {
            x: sign[0],
            y: sign[1],
            event_type: "BGEVENT_READ".into(),
            script: sign_label,
        });
        // Only text was added to the exterior: existing gameplay indexes survive.
        outside.script_text_commands =
            crate::parse_script_text_commands(outside_id, &outside.scripts)?;
        outside.script_text_bodies = crate::parse_script_text_bodies(outside_id, &outside.scripts)?;
        authoring::refresh_event_section(&mut outside);
        let outside_constant = outside
            .attributes
            .map_constant
            .clone()
            .context("route constant")?;
        let mut room = authoring::blank(pack, "Route12SuperRodHouse", room_id, constant)?;
        room.attributes.location = outside.attributes.location.clone();
        room.attributes.music = outside.attributes.music.clone();
        room.attributes.map_group_constant = outside.attributes.map_group_constant.clone();
        // The dry room must not inherit the template house's foreign rod table.
        room.attributes.fishing_group = Some("FISHGROUP_NONE".into());
        for tile in [[2, 7], [3, 7], [2, 6]] {
            authoring::warp(&mut room, (tile[0], tile[1]), &outside_constant, exit);
        }
        for (suffix, sprite, at, routine) in [
            ("Man", "SPRITE_GRAMPS", [2, 3], "DayCareMan"),
            ("Lady", "SPRITE_GRANNY", [5, 3], "DayCareLady"),
            ("Eggs", "SPRITE_POKEFAN_F", [6, 5], "DayCareManOutside"),
        ] {
            let label = format!("{room_id}{suffix}");
            authoring::npc(pack, &mut room, &label, sprite, at[0], at[1])?;
            room.scripts.insert(
                label,
                serde_json::json!([
                    authoring::command("faceplayer", &[]),
                    authoring::command("opentext", &[]),
                    authoring::command("special", &[routine]),
                    authoring::command("waitbutton", &[]),
                    authoring::command("closetext", &[]),
                    authoring::command("end", &[]),
                ]),
            );
        }
        let rules = format!("{room_id}Rules");
        let rules_text = format!("{rules}Text");
        authoring::text(
            &mut room,
            &rules_text,
            &[
                &house.settlement,
                "POKEMON DAY-CARE",
                "Two partners can",
                "stay here. They",
                "grow as you walk.",
                "We never evolve",
                "your partners.",
                "Pick them up here;",
                "other houses keep",
                "their own guests.",
                "100 plus 100 per",
                "level gained.",
                "Our assistant",
                "checks for EGGS.",
            ],
        );
        room.scripts.insert(
            rules.clone(),
            serde_json::json!([authoring::command("jumptext", &[&rules_text])]),
        );
        room.events.bg_events.push(BackgroundEvent {
            x: 1,
            y: 1,
            event_type: "BGEVENT_READ".into(),
            script: rules,
        });
        authoring::finish(pack, &mut room)?;
        room.script_object_commands = crate::parse_script_object_commands(room_id, &room.scripts)?;
        room.script_runtime_commands =
            crate::parse_script_runtime_commands(room_id, &room.scripts)?;
        for tile in [[2, 3], [2, 4], [5, 3], [5, 4], [6, 5], [5, 5], [2, 6]] {
            ensure!(
                crate::authored_town_geometry::kind(pack, &room, tile)? == "FLOOR",
                "daycare room floor {tile:?}"
            );
        }
        let inside_reachable =
            crate::authored_town_geometry::reachable(pack, &room, [2, 6], false)?;
        for approach in [[2, 4], [5, 4], [5, 5]] {
            ensure!(
                inside_reachable.contains(&approach),
                "unreachable daycare caretaker"
            );
        }
        let before_reachable = crate::authored_town_geometry::reachable(
            pack,
            &pack.data.maps[outside_id],
            landing,
            false,
        )?;
        let after_reachable =
            crate::authored_town_geometry::reachable(pack, &outside, landing, false)?;
        ensure!(after_reachable.contains(&door), "unreachable daycare door");
        for warp in &pack.data.maps[outside_id].events.warps {
            let tile = [warp.x, warp.y];
            ensure!(
                !before_reachable.contains(&tile) || after_reachable.contains(&tile),
                "daycare blocked an existing entrance"
            );
        }
        ensure!(
            after_reachable.len() > 100,
            "daycare is disconnected from the settlement"
        );
        crate::authored_town_publication::replace(pack, outside)?;
        let mut metadata = pack.data.runtime_map_metadata[&outside_constant].clone();
        let map_id = pack
            .data
            .runtime_map_metadata
            .values()
            .filter(|m| m.group_id == metadata.group_id)
            .map(|m| m.map_id)
            .max()
            .context("existing route group")?
            .checked_add(1)
            .context("daycare map ID overflow")?;
        metadata.constant = constant.into();
        metadata.name = room_id.into();
        metadata.map_id = map_id;
        metadata.width = room.attributes.width;
        metadata.height = room.attributes.height;
        metadata.environment = "INDOOR".into();
        let spawn_id = pack
            .data
            .runtime_spawn_points
            .values()
            .map(|s| s.identifier)
            .max()
            .context("existing spawn")?
            .checked_add(1)
            .context("daycare spawn overflow")?;
        let spawn = RuntimeSpawnPoint {
            identifier: spawn_id,
            map_constant: constant.into(),
            map_name: room_id.into(),
            group_name: metadata.group_name.clone(),
            group_id: i16::try_from(metadata.group_id)?,
            map_id: i16::try_from(map_id)?,
            tile_x: 2,
            tile_y: 6,
            metatile_x: 1,
            metatile_y: 3,
            subtile_x: 0,
            subtile_y: 0,
        };
        pack.data.pokegear_landmarks.map_to_landmark.insert(
            room_id.into(),
            room.attributes
                .location
                .clone()
                .context("daycare landmark")?,
        );
        crate::insert_compiled_map_extension(
            pack,
            CompiledMapExtension {
                manifest_id: format!("daycare-house-{}", house.id),
                map_name: room_id.into(),
                map_constant: constant.into(),
                metadata,
                spawn_key: spawn_id.to_string(),
                spawn,
                module: room,
                wild_encounters: None,
                start_new_game_here: false,
            },
        )?;
        pack.data.runtime_spawn_points.remove(&spawn_id.to_string());
    }
    pack.report.maps = pack.data.maps.len();
    Ok(())
}
