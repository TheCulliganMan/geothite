//! Authored Grand Rapids and the acknowledged Q06 evidence handover.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;
use crate::authored_town_design::{Actor, Town};
use crate::grand_marais_modpack::authoring;

#[path = "grand_rapids_publication.rs"]
mod publication;
#[path = "grand_rapids_scripts.rs"]
mod scripts;

const TOWN: &str = "MnGrandRapids";
const DEPOT: &str = "MnGrandRapidsSurveyDepot";
const CONSTANT: &str = "MN_GRAND_RAPIDS";
const GROUP: &str = "MINNESOTA_GRAND_RAPIDS";
const GROUP_ID: u16 = 71;
const LANDMARK: &str = "LANDMARK_MN_GRAND_RAPIDS";
const MANIFEST: &str = "minnesota-grand-rapids-v1";

/// Adds the authored town, clinic, shop and Q06 survey depot to a verified pack.
///
/// The handover checks actual foreman defeat and separate meter recovery. Its
/// native yes/no can be declined and retried; clearance is recorded only after
/// the technician's complete reading is acknowledged. Repeat visits give the
/// Duluth direction without repeating the handover. No bag item is fabricated.
/// Clinic healing/PC and ordinary paid shopping use native runtime services.
/// Added clinic/exterior recovery entries preserve every original destination.
///
/// The two regional ports remain unbound reservations. This local publication
/// alone does not establish earned travel from the mine or onward to Duluth.
/// Existing maps, source art, catalogs, HOME, audio and inputs are preserved.
///
/// # Errors
/// Returns an error for conflicting native identities, malformed or mismatched
/// authoring/story contracts, unsafe doors/returns, inaccessible interactions,
/// unknown services/assets, or failed script parsing/runtime pack validation.
pub fn build_grand_rapids_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    story_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let town: Town = serde_json::from_str(town_json).context("Grand Rapids town source")?;
    let layout: Layout =
        serde_json::from_str(layout_json).context("Grand Rapids explicit strokes")?;
    let raw: Value = serde_json::from_str(layout_json)?;
    ensure!(
        town.id == TOWN
            && town.native_constant == CONSTANT
            && town.compiled_tileset == "johto_modern"
            && town.dimensions_metatiles == [24, 24]
            && layout.id == TOWN
            && layout.schema_version == 2
            && layout.compiled_tileset == "johto_modern"
            && layout.dimensions_metatiles == [24, 24]
            && raw["tileset"] == "johto_modern"
            && raw["shore_fitting"] == "johto_8_neighbors"
            && raw["source_pixels_modified"] == false
            && raw["reject_cliff_cuts"] == true
            && raw.get("derived_blocks").is_none(),
        "retain complete Grand Rapids source identity and original art"
    );
    ensure!(
        town.ports.len() == 2,
        "Grand Rapids requires two reserved route ports"
    );
    for (port, (route, side, gate, landing)) in town.ports.iter().zip([
        ("MN_R09", "W", [1, 12], [3, 12]),
        ("MN_R10", "E", [22, 12], [20, 12]),
    ]) {
        ensure!(
            port.route == route
                && port.side == side
                && port.gate_metatile == gate
                && port.landing_metatile == landing,
            "Grand Rapids regional reservation changed"
        );
    }
    let expected = BTreeMap::from([
        (
            "MnGrandRapidsPokecenter1F",
            ([10, 8], "pokecenter", "GoldenrodPokecenter1F"),
        ),
        ("MnGrandRapidsMart", ([6, 4], "mart", "OlivineMart")),
        (DEPOT, ([12, 10], "lab", "ElmsLab")),
    ]);
    ensure!(
        town.interiors.len() == 3 && town.doors.len() == 3,
        "Grand Rapids needs exactly three paired rooms"
    );
    let mut names = BTreeSet::from([TOWN.to_owned()]);
    for room in &town.interiors {
        let (dims, tileset, template) = expected
            .get(room.id.as_str())
            .context("unknown Grand Rapids room")?;
        ensure!(
            names.insert(room.id.clone())
                && room.dimensions_metatiles == *dims
                && room.tileset == *tileset
                && room.template == *template
                && room.border_block == 0
                && matches!(
                    (room.id.as_str(), room.native_constant.as_str()),
                    ("MnGrandRapidsPokecenter1F", "MN_GRAND_RAPIDS_POKECENTER_1F")
                        | ("MnGrandRapidsMart", "MN_GRAND_RAPIDS_MART")
                        | (DEPOT, "MN_GRAND_RAPIDS_SURVEY_DEPOT")
                ),
            "Grand Rapids room identity changed"
        );
    }
    publication::available(base, &names)?;
    for constant in
        std::iter::once(CONSTANT).chain(town.interiors.iter().map(|r| r.native_constant.as_str()))
    {
        ensure!(
            !base
                .data
                .story_event_script_constants
                .global
                .contains_key(constant),
            "Grand Rapids native constant already occupied: {constant}"
        );
    }
    let mut pack = base.clone();
    scripts::prepare(&mut pack, &town)?;
    let texts = scripts::story(story_json)?;
    let mut exterior = authoring::blank(&pack, "OlivineCity", TOWN, CONSTANT)?;
    exterior.attributes.width = 24;
    exterior.attributes.height = 24;
    exterior.attributes.tileset_name = "johto_modern".into();
    exterior.attributes.border_block = 5;
    exterior.blocks = layout.compile()?;
    scripts::install(&pack, &mut exterior, &town.residents, &town.signs, &texts)?;
    let mut modules = vec![(exterior, [6, 24])];
    for room in &town.interiors {
        ensure!(
            room.rows.len() == usize::from(room.dimensions_metatiles[1])
                && room
                    .rows
                    .iter()
                    .all(|r| r.split_whitespace().count()
                        == usize::from(room.dimensions_metatiles[0])),
            "Grand Rapids rows do not match room extent"
        );
        let mut module = authoring::blank(&pack, &room.template, &room.id, &room.native_constant)?;
        authoring::blocks(
            &mut module,
            room.dimensions_metatiles[0],
            room.dimensions_metatiles[1],
            &room.rows.join(" "),
        )?;
        module.attributes.tileset_name = room.tileset.clone();
        module.attributes.border_block = room.border_block;
        scripts::install(&pack, &mut module, &room.residents, &room.signs, &texts)?;
        if let Some(pc) = &room.pc {
            ensure!(
                room.id == "MnGrandRapidsPokecenter1F",
                "PC belongs in Grand Rapids clinic"
            );
            let label = "MnGrandRapidsClinicPC";
            let c = authoring::command;
            module.scripts.insert(
                label.into(),
                serde_json::json!([
                    c("opentext", &[]),
                    c("special", &["PokemonCenterPC"]),
                    c("closetext", &[]),
                    c("end", &[])
                ]),
            );
            module.events.bg_events.push(BackgroundEvent {
                x: pc.tile[0],
                y: pc.tile[1],
                event_type: "BGEVENT_READ".into(),
                script: label.into(),
            });
        }
        let doors = town
            .doors
            .iter()
            .filter(|d| d.target == room.id)
            .collect::<Vec<_>>();
        ensure!(doors.len() == 1, "room needs one unique exterior pairing");
        let door = doors[0];
        ensure!(
            room.exits.len() == 2
                && room.passive_arrival.index == 3
                && door.target_warp == 3
                && door.arrival_tile == door.return_tile,
            "Grand Rapids requires two carpets and safe passive arrivals"
        );
        for (index, exit) in room.exits.iter().enumerate() {
            ensure!(
                exit.index == u16::try_from(index + 1)?
                    && exit.target == TOWN
                    && Some(u16::try_from(exit.target_warp)?) == door.return_warp_index,
                "Grand Rapids carpet must return to its own exterior"
            );
            authoring::warp(
                &mut module,
                (exit.tile[0], exit.tile[1]),
                CONSTANT,
                exit.target_warp,
            );
        }
        authoring::warp(
            &mut module,
            (room.passive_arrival.tile[0], room.passive_arrival.tile[1]),
            CONSTANT,
            i16::try_from(
                door.return_warp_index
                    .context("Grand Rapids exterior return")?,
            )?,
        );
        modules.push((module, room.passive_arrival.tile));
    }
    for door in &town.doors {
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("Grand Rapids door target")?;
        let exterior = &mut modules[0].0;
        ensure!(
            door.warp_index == u16::try_from(exterior.events.warps.len() + 1)?
                && door.return_warp_index == Some(door.warp_index + 1),
            "Grand Rapids exterior warp order"
        );
        authoring::warp(
            exterior,
            (door.tile[0], door.tile[1]),
            &room.native_constant,
            3,
        );
        authoring::warp(
            exterior,
            (door.return_tile[0], door.return_tile[1]),
            CONSTANT,
            i16::try_from(door.return_warp_index.context("Grand Rapids safe return")?)?,
        );
    }
    for (module, _) in &mut modules {
        module.attributes.location = Some(LANDMARK.into());
        module.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
        scripts::finish(&pack, module)?;
    }
    crate::authored_town_geometry::validate(&pack, &town, &modules)?;
    publication::publish(pack, modules)
}
