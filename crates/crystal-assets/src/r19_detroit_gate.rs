//! Native gatehouse passage for the authored Prairie Forest Link endpoint.
use anyhow::{Context, Result, ensure};
use crystal_core::map::CoordEvent;
use serde_json::json;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, CompiledMapExtension, RuntimeSpawnPoint};

const ROOM: &str = "MnR19DetroitGate";
const CONSTANT: &str = "MN_R19_DETROIT_GATE";
const ROUTE: &str = "MnR19C00R01";
const TOWN: &str = "MnDetroitLakes";

/// Installs visible native gate entrances, paired room exits and the northbound badge check.
///
/// The eight reviewed exterior cells preserve all habitat and existing doors.
/// Southbound returns remain free; no badge, quest or travel flag is granted.
///
/// # Errors
/// Rejects a stale parent, source/preimage drift, occupied identities, incompatible
/// native warp anchors, or invalid runtime/geography output.
pub fn build_r19_detroit_gate(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/route-grass-v01/current-r19-successor-review-v01/gatehouse-room-source-v02.json"
    ))?;
    let exterior: serde_json::Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/route-grass-v01/current-r19-successor-review-v01/gatehouse-exterior-source-v02.json"
    ))?;
    ensure!(
        base.identity()?.content_hash
            == source["parent_content_identity"]
                .as_str()
                .context("parent identity")?,
        "exact gate parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let group = u16::try_from(source["group_id"].as_u64().context("group")?)?;
    let map_id = u16::try_from(source["map_id"].as_u64().context("map id")?)?;
    let identifier = u16::try_from(source["spawn_identifier"].as_u64().context("spawn")?)?;
    ensure!(
        !base.data.maps.contains_key(ROOM)
            && !base.data.runtime_map_metadata.contains_key(CONSTANT)
            && !base
                .data
                .runtime_map_metadata
                .values()
                .any(|m| m.group_id == group && m.map_id == map_id)
            && !base
                .data
                .runtime_spawn_points
                .values()
                .any(|s| s.identifier == identifier),
        "reserved gate identity occupied"
    );
    let mut pack = base.clone();
    let mut route = base.data.maps[ROUTE].clone();
    let mut town = base.data.maps[TOWN].clone();
    ensure!(
        route.attributes.width == 32
            && route.attributes.height == 6
            && town.attributes.width == 24
            && town.attributes.height == 18,
        "gate exterior extents"
    );
    for patch in exterior["patches"].as_array().context("exterior patches")? {
        let map = match patch["map"].as_str() {
            Some(ROUTE) => &mut route,
            Some(TOWN) => &mut town,
            _ => anyhow::bail!("unexpected gate exterior"),
        };
        let x = usize::try_from(patch["metatile"][0].as_u64().context("x")?)?;
        let y = usize::try_from(patch["metatile"][1].as_u64().context("y")?)?;
        let index = y * usize::from(map.attributes.width) + x;
        let before = u16::from_str_radix(patch["before"].as_str().context("preimage")?, 16)?;
        let after = u16::from_str_radix(patch["after"].as_str().context("new block")?, 16)?;
        ensure!(
            map.blocks.get(index) == Some(&before),
            "gate exterior preimage differs"
        );
        map.blocks[index] = after;
    }
    let route_return = i16::try_from(route.events.warps.len() + 3)?;
    let town_return = i16::try_from(town.events.warps.len() + 2)?;
    let town_constant = town
        .attributes
        .map_constant
        .clone()
        .context("Detroit native constant")?;
    for at in [(25, 9), (26, 9)] {
        authoring::warp(&mut route, at, CONSTANT, 5);
    }
    for at in [(25, 8), (26, 8)] {
        authoring::warp(&mut route, at, CONSTANT, 5);
    }
    authoring::warp(&mut town, (25, 3), CONSTANT, 6);
    authoring::warp(&mut town, (25, 4), CONSTANT, 6);
    let mut room = authoring::blank(base, "Route35GoldenrodGate", ROOM, CONSTANT)?;
    let rows = source["rows"]
        .as_array()
        .context("gate rows")?
        .iter()
        .map(|r| r.as_str().context("row"))
        .collect::<Result<Vec<_>>>()?;
    authoring::blocks(&mut room, 5, 4, &rows.join(" "))?;
    room.attributes.fishing_group = Some("FISHGROUP_MN_SHORE".into());
    room.attributes.location = route.attributes.location.clone();
    room.attributes.map_group_constant = route.attributes.map_group_constant.clone();
    for at in [(4, 0), (5, 0)] {
        authoring::warp(&mut room, at, ROUTE, route_return);
    }
    for at in [(4, 7), (5, 7)] {
        authoring::warp(&mut room, at, &town_constant, town_return);
    }
    authoring::warp(&mut room, (4, 1), ROUTE, route_return);
    authoring::warp(&mut room, (4, 6), &town_constant, town_return);
    let c = authoring::command;
    room.scripts.insert(
        "MnR19GateCheck".into(),
        json!([
            c("readvar", &["VAR_FACING"]),
            c("ifequal", &["UP", "MnR19GateQualify"]),
            c("end", &[])
        ]),
    );
    room.scripts.insert(
        "MnR19GateQualify".into(),
        json!([
            c("checkflag", &["ENGINE_MINERALBADGE"]),
            c("iftrue", &["MnR19GateDone"]),
            c("applymovement", &["PLAYER", "MnR19GateStepSouth"]),
            c("jumptext", &["MnR19GateDeniedText"])
        ]),
    );
    room.scripts
        .insert("MnR19GateDone".into(), json!([c("end", &[])]));
    room.scripts.insert(
        "MnR19GateStepSouth".into(),
        json!([c("step", &["DOWN"]), c("step_end", &[])]),
    );
    authoring::text(
        &mut room,
        "MnR19GateDeniedText",
        &[
            "The prairie trail",
            "needs six BADGES.",
            "Please return",
            "when qualified.",
        ],
    );
    for x in [4, 5] {
        room.events.coord_events.push(CoordEvent {
            x,
            y: 2,
            scene_id: String::new(),
            script_name: "MnR19GateCheck".into(),
        });
    }
    authoring::finish(base, &mut room)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(ROOM, &room.scripts)?;
    room.script_variable_commands = crate::parse_script_variable_commands(ROOM, &room.scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(ROOM, &room.scripts)?;
    room.script_movements =
        crate::parse_script_movements(ROOM, &room.scripts, &room.script_object_commands)?;
    let mut metadata = base.data.runtime_map_metadata[ROUTE].clone();
    metadata.name = ROOM.into();
    metadata.constant = CONSTANT.into();
    metadata.group_id = group;
    metadata.map_id = map_id;
    metadata.width = 5;
    metadata.height = 4;
    metadata.environment = room
        .attributes
        .environment
        .clone()
        .context("gate environment")?;
    let extension = CompiledMapExtension {
        manifest_id: "minnesota-r19-detroit-gate-v1".into(),
        map_name: ROOM.into(),
        map_constant: CONSTANT.into(),
        metadata,
        spawn_key: identifier.to_string(),
        spawn: RuntimeSpawnPoint {
            identifier,
            map_name: ROOM.into(),
            map_constant: CONSTANT.into(),
            group_name: source["group_name"].as_str().context("group name")?.into(),
            group_id: i16::try_from(group)?,
            map_id: i16::try_from(map_id)?,
            tile_x: 4,
            tile_y: 6,
            metatile_x: 2,
            metatile_y: 3,
            subtile_x: 0,
            subtile_y: 0,
        },
        module: room,
        wild_encounters: None,
        start_new_game_here: false,
    };
    crate::validate_compiled_map_extension(&extension)?;
    crate::insert_compiled_map_extension(&mut pack, extension)?;
    // Ordinary passage rooms do not register a new recovery/Fly destination.
    pack.data
        .runtime_spawn_points
        .remove(&identifier.to_string());
    for mut map in [route, town] {
        authoring::refresh_event_section(&mut map);
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    pack.data.pokegear_landmarks.map_to_landmark.insert(
        ROOM.into(),
        base.data.maps[ROUTE]
            .attributes
            .location
            .clone()
            .context("route landmark")?,
    );
    pack.report.maps = pack.data.maps.len();
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
