//! Native Brainerd exterior and services; the installed canonical Gym6 stays authoritative.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_route_design::OneWayLedge;
use crate::authored_route_layout::Layout;
use crate::grand_marais_modpack::authoring;

pub(crate) const TOWN: &str = "MnBrainerd";
pub(crate) const CONSTANT: &str = "MN_BRAINERD";
pub(crate) const GROUP: &str = "MINNESOTA_BRAINERD_CITY";
pub(crate) const GROUP_ID: u16 = 69;
pub(crate) const LANDMARK: &str = "LANDMARK_MN_BRAINERD";
pub(crate) const MANIFEST: &str = "minnesota-brainerd-city-v1";

#[derive(Deserialize)]
pub(crate) struct Town {
    pub id: String,
    pub native_constant: String,
    pub tileset: String,
    pub dimensions_metatiles: [u16; 2],
    pub ports: Vec<Port>,
    pub residents: Vec<Resident>,
    pub signs: Vec<Sign>,
    pub doors: Vec<Door>,
    pub interiors: Vec<Room>,
    pub one_way_ledges: Vec<OneWayLedge>,
}
#[derive(Deserialize)]
pub(crate) struct Port {
    pub route: String,
    pub side: String,
    pub gate_metatile: [u16; 2],
    pub landing_metatile: [u16; 2],
}
#[derive(Deserialize)]
pub(crate) struct Door {
    pub id: String,
    pub tile: [u16; 2],
    pub return_tile: [u16; 2],
    pub arrival_tile: [u16; 2],
    pub target: String,
    pub warp_index: u16,
    pub return_warp_index: u16,
    pub target_warp: i16,
}
#[derive(Deserialize)]
pub(crate) struct Room {
    pub id: String,
    pub native_constant: String,
    pub kind: String,
    pub dimensions_metatiles: [u16; 2],
    pub template: Option<String>,
    pub tileset: Option<String>,
    pub border_block: Option<u8>,
    #[serde(default)]
    pub rows: Vec<String>,
    #[serde(default)]
    pub residents: Vec<Resident>,
    #[serde(default)]
    pub signs: Vec<Sign>,
    pub passive_arrival: Arrival,
    pub exits: Vec<Exit>,
    #[serde(default)]
    pub inventory_contract: Vec<String>,
    pub pc: Option<Pc>,
    pub canonical_source: Option<String>,
    pub canonical_id: Option<String>,
    pub canonical_gym_sha256: Option<String>,
}
#[derive(Deserialize)]
pub(crate) struct Arrival {
    pub index: u16,
    pub tile: [u16; 2],
}
#[derive(Deserialize)]
pub(crate) struct Exit {
    pub index: u16,
    pub tile: [u16; 2],
    pub target: String,
    pub target_warp: i16,
}
#[derive(Deserialize)]
pub(crate) struct Resident {
    pub id: String,
    pub sprite: String,
    pub tile: [u16; 2],
    pub facing: String,
    pub lines: Vec<String>,
    pub service: Option<String>,
    pub reward: Option<Reward>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub before_lines: Vec<String>,
    #[serde(default)]
    pub repeat_lines: Vec<String>,
    pub counter_tile: Option<[u16; 2]>,
    pub approach: Option<[u16; 2]>,
}
#[derive(Deserialize)]
pub(crate) struct Reward {
    pub item: String,
    pub quantity: u8,
    pub once_flag: String,
    #[serde(default)]
    pub sets: Vec<String>,
}
#[derive(Deserialize)]
pub(crate) struct Sign {
    pub id: String,
    pub tile: [u16; 2],
    pub lines: Vec<String>,
    pub approach: Option<[u16; 2]>,
}
#[derive(Deserialize)]
pub(crate) struct Pc {
    pub tile: [u16; 2],
    pub approach: [u16; 2],
}

/// Adds the authored Brainerd town and four service rooms around an installed Gym6.
///
/// The canonical gym must match the supplied canonical source in native art,
/// geometry, scripts, actors and trainers. Its three existing review-hall warp
/// destinations are rebound to the town's passive return6; every other gym
/// field and all existing parties/rewards/puzzle state remain unchanged.
/// Native healing, PC, ordinary shop and success-only report/Surf/local gifts
/// use production scripts. No badge, story flag or item is granted on install.
/// The four authored regional ports stay unbound reservations; HOME and the
/// original recovery table are retained. This does not prove earned Q04 travel.
///
/// # Errors
/// Returns an error for malformed/stale source, a noncanonical installed gym,
/// an occupied group69/map/constant/catalog, unsupported services, unsafe
/// geometry/returns or failed native parsing/runtime verification. The borrowed
/// pack and all source strings remain unchanged on success and failure.
pub fn build_brainerd_city_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    canonical_gyms_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let town: Town = serde_json::from_str(town_json).context("Brainerd town source")?;
    let raw: Value = serde_json::from_str(layout_json)?;
    // Use the existing typed stroke compiler without expanding its route-ID
    // dispatcher: this city validates its own fixed identity and native art.
    let layout: Layout = serde_json::from_str(layout_json).context("Brainerd explicit strokes")?;
    ensure!(
        town.id == TOWN
            && town.native_constant == CONSTANT
            && town.dimensions_metatiles == [32, 24]
            && town.tileset == "johto"
            && layout.id == TOWN
            && layout.dimensions_metatiles == [32, 24]
            && layout.compiled_tileset == "johto"
            && layout.schema_version == 2
            && raw["tileset"] == "johto"
            && raw["shore_fitting"] == "johto_8_neighbors"
            && raw["reject_cliff_cuts"] == true
            && raw["source_pixels_modified"] == false
            && raw.get("derived_blocks").is_none(),
        "retain complete Brainerd identity, explicit source art and cliff fitting"
    );
    crate::brainerd_city_publication::available(base, &town)?;
    let gym = crate::brainerd_city_publication::canonical_gym(
        base,
        &town,
        town_json,
        canonical_gyms_json,
    )?;
    let mut pack = base.clone();
    pack.data.materialize_global_scripts()?;
    crate::brainerd_city_scripts::prepare_catalogs(&mut pack, &town)?;
    let mut exterior = authoring::blank(&pack, "CherrygroveCity", TOWN, CONSTANT)?;
    exterior.attributes.width = 32;
    exterior.attributes.height = 24;
    exterior.attributes.tileset_name = "johto".into();
    exterior.attributes.border_block = 5;
    exterior.blocks = layout.compile()?;
    crate::brainerd_city_scripts::people(&pack, &mut exterior, &town.residents, &town.signs)?;
    let mut modules = vec![(exterior, [32, 44])];
    for room in &town.interiors {
        if room.id == gym.id {
            continue;
        }
        let mut module = authoring::blank(
            &pack,
            room.template.as_deref().context("Brainerd room template")?,
            &room.id,
            &room.native_constant,
        )?;
        authoring::blocks(
            &mut module,
            room.dimensions_metatiles[0],
            room.dimensions_metatiles[1],
            &room.rows.join(" "),
        )?;
        module.attributes.tileset_name = room.tileset.clone().context("Brainerd room tileset")?;
        module.attributes.border_block = room.border_block.context("Brainerd room border")?;
        crate::brainerd_city_scripts::people(&pack, &mut module, &room.residents, &room.signs)?;
        crate::brainerd_city_scripts::pc(&pack, &mut module, room)?;
        let door = town
            .doors
            .iter()
            .find(|d| d.target == room.id)
            .context("Brainerd room door")?;
        for exit in &room.exits {
            ensure!(
                exit.index == module.events.warps.len() as u16 + 1
                    && exit.target == TOWN
                    && exit.target_warp == i16::try_from(door.return_warp_index)?,
                "Brainerd reciprocal room exit"
            );
            authoring::warp(
                &mut module,
                (exit.tile[0], exit.tile[1]),
                CONSTANT,
                exit.target_warp,
            );
        }
        ensure!(
            room.passive_arrival.index == 3,
            "Brainerd passive room arrival3"
        );
        authoring::warp(
            &mut module,
            (room.passive_arrival.tile[0], room.passive_arrival.tile[1]),
            CONSTANT,
            i16::try_from(door.return_warp_index)?,
        );
        modules.push((module, room.passive_arrival.tile));
    }
    for door in &town.doors {
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("Brainerd door target")?;
        let exterior = &mut modules[0].0;
        ensure!(
            door.warp_index == exterior.events.warps.len() as u16 + 1
                && door.return_warp_index == door.warp_index + 1
                && door.target_warp == 3,
            "Brainerd active/passive exterior index order"
        );
        for at in [door.tile, door.return_tile] {
            authoring::warp(exterior, (at[0], at[1]), &room.native_constant, 3);
        }
    }
    for (module, _) in &mut modules {
        module.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
        module.attributes.location = Some(LANDMARK.into());
        crate::brainerd_city_scripts::finish(&pack, module)?;
    }
    crate::brainerd_city_geometry::validate(&pack, &town, &modules)?;
    crate::brainerd_city_publication::publish(pack, modules, gym)
}
