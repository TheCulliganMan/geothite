//! Rochester's real doors, care, shopping and acknowledged survey delivery.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde_json::{Value, json};

use crate::authored_route_layout::Layout;
use crate::authored_town_design::Town;
use crate::authored_town_scripts::{finish, ordinary};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, rochester_gym, rochester_publication, rochester_story};

/// Adds Rochester and binds its refreshed canonical first gym to the city.
///
/// Delivery requires the actual starter and original-readings events and is
/// recorded only after both pages are acknowledged. Ordinary care/PC/shopping
/// use native services. Existing HOME/recovery, unrelated maps, art and audio
/// stay intact; Gym1's private art, roster and room are explicitly refreshed.
/// Route connections are a separate checked overlay; this grants no progress.
///
/// # Errors
/// Returns an error for stale identities, invalid authoring/story, unsafe
/// geometry, shared gym assets, occupied trainers, parsing or pack verification.
pub fn build_rochester_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    story_json: &str,
    gym_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let town: Town = serde_json::from_str(town_json).context("Rochester source")?;
    let layout: Layout = serde_json::from_str(layout_json).context("Rochester strokes")?;
    let story: Value = serde_json::from_str(story_json)?;
    let raw: Value = serde_json::from_str(town_json)?;
    ensure!(
        town.id == "MnRochester"
            && town.native_constant == "MN_ROCHESTER"
            && town.compiled_tileset == "johto_modern"
            && town.dimensions_metatiles == [20, 16]
            && layout.id == town.id
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && town.interiors.len() == 5
            && town.doors.len() == 5
            && !base.data.maps.contains_key(&town.id),
        "Rochester inventory/identity or duplicate installation"
    );
    for (room, (id, size)) in town.interiors.iter().zip([
        ("MnRochesterPokecenter1F", [10, 8]),
        ("MnRochesterMart", [6, 4]),
        ("MnRochesterGym", [7, 10]),
        ("MnRochesterClerkHouse", [5, 4]),
        ("MnRochesterPatientHouse", [5, 4]),
    ]) {
        ensure!(
            room.id == id && room.dimensions_metatiles == size,
            "complete Rochester room inventory and extents required"
        );
    }
    let mut recipients = town.interiors[0]
        .residents
        .iter()
        .filter(|a| a.service.as_deref() == Some("recipient"));
    let recipient = recipients.next().context("clinic recipient required")?;
    ensure!(
        recipients.next().is_none()
            && recipient.id == "MN_ROCHESTER_RECIPIENT"
            && recipient.tile == [11, 8],
        "one reachable clinic recipient must own the delivery"
    );
    let mut pack = base.clone();
    for routine in ["HealParty", "PokemonCenterPC"] {
        pack.data.require_special_routine(routine)?;
    }
    let mut gym = rochester_gym::rebuild(&mut pack, gym_json)?;
    let entry_lines: Vec<String> = serde_json::from_value(raw["gym_entry_lines"].clone())?;
    ensure!(
        entry_lines == ["Visit the clinic.", "Take your parcel."],
        "explicit first-gym delivery direction required"
    );
    authoring::text(
        &mut gym,
        "MnRochesterGymRequirementsText",
        &entry_lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    let clerk = town
        .interiors
        .iter()
        .flat_map(|r| &r.residents)
        .find(|a| a.service.as_deref() == Some("pokemart"))
        .context("clerk")?;
    ensure!(
        clerk.stock == ["POKE_BALL", "POTION", "ANTIDOTE", "PARLYZ_HEAL"]
            && !pack.data.marts.0.contains_key("MART_MN_ROCHESTER"),
        "canonical opening shop stock"
    );
    pack.data
        .marts
        .0
        .insert("MART_MN_ROCHESTER".into(), clerk.stock.clone());
    let mut exterior = authoring::blank(&pack, "GoldenrodCity", &town.id, &town.native_constant)?;
    exterior.attributes.width = 20;
    exterior.attributes.height = 16;
    exterior.attributes.tileset_name = town.compiled_tileset.clone();
    exterior.attributes.border_block = 5;
    exterior.blocks = layout.compile()?;
    ordinary(
        &pack,
        &mut exterior,
        &town.residents,
        &town.signs,
        "MN_ROCHESTER_POKECENTER_1F",
        "MART_MN_ROCHESTER",
    )?;
    for d in &town.doors {
        let r = town
            .interiors
            .iter()
            .find(|r| r.id == d.target)
            .context("paired room")?;
        ensure!(
            usize::from(d.warp_index) == exterior.events.warps.len() + 1
                && d.return_warp_index == Some(d.warp_index + 1)
                && d.target_warp == 3
                && d.arrival_tile == r.passive_arrival.tile,
            "passive door pairing"
        );
        authoring::warp(&mut exterior, (d.tile[0], d.tile[1]), &r.native_constant, 3);
        authoring::warp(
            &mut exterior,
            (d.return_tile[0], d.return_tile[1]),
            &town.native_constant,
            i16::try_from(d.return_warp_index.context("return")?)?,
        );
    }
    let mut modules = vec![(exterior, [32, 18])];
    for r in &town.interiors {
        if r.id == "MnRochesterGym" {
            ensure!(
                r.rows.is_empty()
                    && r.residents.is_empty()
                    && r.dimensions_metatiles == [7, 10]
                    && r.passive_arrival.tile == [6, 17],
                "canonical Gym1 reference required"
            );
            finish(&pack, &mut gym, &town)?;
            modules.push((gym.clone(), r.passive_arrival.tile));
            continue;
        }
        let mut m = authoring::blank(&pack, &r.template, &r.id, &r.native_constant)?;
        authoring::blocks(
            &mut m,
            r.dimensions_metatiles[0],
            r.dimensions_metatiles[1],
            &r.rows.join(" "),
        )?;
        m.attributes.tileset_name = r.tileset.clone();
        m.attributes.border_block = r.border_block;
        ordinary(
            &pack,
            &mut m,
            &r.residents,
            &r.signs,
            "MN_ROCHESTER_POKECENTER_1F",
            "MART_MN_ROCHESTER",
        )?;
        let door = town
            .doors
            .iter()
            .find(|d| d.target == r.id)
            .context("facade")?;
        for e in &r.exits {
            ensure!(
                usize::from(e.index) == m.events.warps.len() + 1
                    && e.target == town.id
                    && Some(u16::try_from(e.target_warp)?) == door.return_warp_index,
                "real carpet pairing"
            );
            authoring::warp(
                &mut m,
                (e.tile[0], e.tile[1]),
                &town.native_constant,
                e.target_warp,
            );
        }
        ensure!(r.passive_arrival.index == 3, "passive arrival3");
        authoring::warp(
            &mut m,
            (r.passive_arrival.tile[0], r.passive_arrival.tile[1]),
            &town.native_constant,
            i16::try_from(door.return_warp_index.context("return")?)?,
        );
        if let Some(pc) = &r.pc {
            let label = format!("{}PC", r.id);
            let c = authoring::command;
            m.scripts.insert(
                label.clone(),
                json!([
                    c("opentext", &[]),
                    c("special", &["PokemonCenterPC"]),
                    c("closetext", &[]),
                    c("end", &[])
                ]),
            );
            m.events.bg_events.push(BackgroundEvent {
                x: pc.tile[0],
                y: pc.tile[1],
                event_type: "BGEVENT_READ".into(),
                script: label,
            });
        }
        if r.id == "MnRochesterPokecenter1F" {
            rochester_story::install(&mut m, &story)?;
        }
        finish(&pack, &mut m, &town)?;
        modules.push((m, r.passive_arrival.tile));
    }
    for (m, _) in &mut modules {
        finish(&pack, m, &town)?;
    }
    crate::authored_town_geometry::validate(&pack, &town, &modules)?;
    rochester_publication::publish(&mut pack, modules)?;
    pack.report
        .manifests
        .push("minnesota-rochester-story-v1".into());
    pack.report.maps = pack.data.maps.len();
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
