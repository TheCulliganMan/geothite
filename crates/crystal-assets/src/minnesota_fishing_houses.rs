//! Three explicitly placed route houses for Crystal-style fishing gurus.
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
    rod_tier: usize,
}

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    crate::minnesota_interior_lineage::require_compact_coast(pack)?;
    let houses: Vec<House> = serde_json::from_str(include_str!(
        "../../../planning/minnesota/fishing-gurus.json"
    ))?;
    ensure!(
        houses.len() == 3,
        "three staged fishing guru houses required"
    );
    for (expected_tier, house) in houses.into_iter().enumerate() {
        ensure!(
            house.rod_tier == expected_tier,
            "ordered Old/Good/Super guru houses required"
        );
        let outside_id = house.exterior.as_str();
        let room_id = house.room.as_str();
        let constant = house.constant.as_str();
        let [x, y] = house.facade_metatile;
        let tier = house.rod_tier;
        ensure!(
            !pack.data.maps.contains_key(room_id),
            "guru house already exists"
        );
        let mut outside = pack
            .data
            .maps
            .get(outside_id)
            .context("guru route")?
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
            // Never turn an existing river, ledge or cave into house frontage.
            let old = outside.blocks[index];
            let collision = &pack.data.tilesets[&outside.attributes.tileset_name].collision
                [&format!("{old:02x}")];
            ensure!(
                !collision.iter().any(|kind| kind == "WATER"
                    || kind == "WATERFALL"
                    || kind.starts_with("HOP")
                    || kind == "CAVE"),
                "house would remove protected traversal"
            );
            outside.blocks[index] = block;
        }
        ensure!(
            crate::authored_town_geometry::kind(pack, &outside, door)? == "DOOR",
            "native guru door"
        );
        ensure!(
            crate::authored_town_geometry::kind(pack, &outside, landing)? == "FLOOR",
            "dry guru return"
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
                "FISHING GURU",
                ["OLD ROD HOUSE", "GOOD ROD HOUSE", "SUPER ROD HOUSE"][tier],
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
        for tile in [[2, 7], [3, 7], [2, 6]] {
            authoring::warp(&mut room, (tile[0], tile[1]), &outside_constant, exit);
        }
        let guru = format!("{room_id}Guru");
        let after = format!("{guru}After");
        let after_text = format!("{after}Text");
        authoring::npc(pack, &mut room, &guru, "SPRITE_FISHING_GURU", 5, 3)?;
        authoring::text(
            &mut room,
            &after_text,
            &[
                "Try every shore!",
                "Different RODS",
                "catch different",
                "kinds of POKEMON.",
            ],
        );
        room.scripts.insert(
            after.clone(),
            serde_json::json!([authoring::command("jumptextfaceplayer", &[&after_text])]),
        );
        let mut programs = std::collections::BTreeMap::new();
        crate::minnesota_fishing_rods::add_handoff(&mut room, &mut programs, &guru, &after, tier)?;
        room.scripts.extend(programs);
        authoring::finish(pack, &mut room)?;
        room.script_object_commands = crate::parse_script_object_commands(room_id, &room.scripts)?;
        for tile in [[5, 3], [5, 4], [2, 6]] {
            ensure!(
                crate::authored_town_geometry::kind(pack, &room, tile)? == "FLOOR",
                "guru room floor {tile:?}"
            );
        }
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
            .context("guru map ID overflow")?;
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
            .context("guru spawn overflow")?;
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
            room.attributes.location.clone().context("guru landmark")?,
        );
        crate::insert_compiled_map_extension(
            pack,
            CompiledMapExtension {
                manifest_id: format!("fishing-guru-house-{tier}"),
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
