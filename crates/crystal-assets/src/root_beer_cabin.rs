//! Prepare the actual Isle of Pines cabin and its existing exterior hospitality.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_route_design::{Door, Interior};
use crate::authored_route_layout::Layout;
use crate::authored_route_modpack::{collision, validate_room};
use crate::grand_marais_modpack::authoring;
use crate::{
    CompiledGamePack, MapModule, ROOT_BEER_ITEM_ID, RootBeerRestPolicy,
    build_root_beer_lady_modpack, prepare_root_beer_lady_service,
    verify_compiled_game_pack_for_runtime,
};

/// Stable authored room identity, independent from an existing source house.
pub const ROOT_BEER_CABIN_MAP: &str = "MnBoundaryRootBeerCabin";
/// Native constant for the authored cabin; the caller owns map registration.
pub const ROOT_BEER_CABIN_CONSTANT: &str = "MN_BOUNDARY_ROOT_BEER_CABIN";

/// Prepared modules for the caller's atomic native preserve publication.
///
/// The default cabin is empty while the single Dorothy remains outside. No
/// conditional indoor presence scene or Safari session behavior is installed.
/// Neither module is inserted into the returned pack by this preparation step.
pub struct PreparedRootBeerCabin {
    /// Verified additive drink, mart and gift-event metadata; old maps unchanged.
    pub pack: CompiledGamePack,
    /// The supplied actual exterior with paired door records and Dorothy service.
    pub exterior: MapModule,
    /// The exact hand-authored room, two carpet exits and passive FLOOR arrival3.
    pub cabin: MapModule,
    /// Unoccupied nontriggering indoor arrival in16px gameplay coordinates.
    pub arrival: [u16; 2],
    /// Actual exterior safe FLOOR return from the current preserve source.
    pub exterior_return: [u16; 2],
    /// Native gift/shop/rest script attached to the existing outside Dorothy.
    pub service_script: String,
}

#[derive(Deserialize)]
struct PreserveBinding {
    root_beer_lady: CabinBinding,
    landmark_actors: Vec<LandmarkActor>,
}

#[derive(Deserialize)]
struct CabinBinding {
    door: Door,
}

#[derive(Deserialize)]
struct LandmarkActor {
    id: String,
    tile: [u16; 2],
    sprite: String,
}

/// Prepares the real authored cabin and binds its existing exterior Dorothy.
///
/// `cabin_json` is the current `playable/root-beer-cabin.json`; `layout_json` is
/// the current preserve layout. `exterior` must be the actual native clipped
/// preserve chunk containing their door, with its original authored Granny.
/// Its blocks must equal that layout's Rust compilation. This function clears
/// the source template's actors/scripts/warps and replaces every room block with
/// the supplied cabin rows; no existing house is presented as the new cabin.
///
/// Both carpet exits target the exterior's separate safe return record. Door1
/// enters passive cabin record3 on FLOOR, which cannot activate a warp merely
/// by walking over it. The existing outside actor receives the tested native
/// first-gift, standard mart at300 per bottle and optional HealParty service. The cabin
/// remains empty, preserving one Dorothy without claiming an indoor relocation.
///
/// The caller must publish both prepared modules atomically, register their
/// actual native map metadata/landmark/constant, and run controller entrance,
/// exit, counter/presence and persistence checks. This preparation publishes no
/// maps, recovery spawns, Safari admission, session counters or story flags.
///
/// # Errors
/// Returns an error for malformed cabin/arrival/source bindings, a mismatched
/// exterior module, furniture/door/return collision, duplicate or missing
/// Dorothy, conflicting warp records, incompatible service metadata, native
/// parsing or pack verification. Inputs remain unchanged on failure.
pub fn prepare_root_beer_cabin(
    base: &CompiledGamePack,
    cabin_json: &str,
    layout_json: &str,
    exterior: &MapModule,
    rest: RootBeerRestPolicy,
) -> Result<PreparedRootBeerCabin> {
    verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.data().maps.contains_key(ROOT_BEER_CABIN_MAP),
        "Root Beer cabin already exists"
    );
    let mut value: Value = serde_json::from_str(cabin_json).context("authored cabin JSON")?;
    let passive = value["warp_contract"]["interior_passive_arrival_warp"].clone();
    ensure!(
        passive["collision"] == "FLOOR" && passive["active_trigger"] == false,
        "cabin arrival must be explicitly passive FLOOR"
    );
    if let Some(explicit) = value.get("passive_arrival") {
        ensure!(
            explicit["index"] == passive["index"] && explicit["tile"] == passive["tile"],
            "conflicting cabin passive arrival declarations"
        );
    }
    value["passive_arrival"] = passive;
    let room: Interior = serde_json::from_value(value.clone()).context("typed cabin interior")?;
    let arrival = room
        .passive_arrival
        .as_ref()
        .context("cabin passive arrival")?;
    ensure!(
        room.id == ROOT_BEER_CABIN_MAP
            && room.template == "VioletNicknameSpeechHouse"
            && room.dimensions_metatiles == [5, 4]
            && value["tileset"] == "traditional_house"
            && room.border_block == Some(0)
            && room.exits == [[5, 7], [6, 7]]
            && room.exit_collision == "WARP_CARPET_DOWN"
            && room.clear_arrival == Some([5, 6])
            && arrival.index == 3
            && arrival.tile == [5, 6]
            && value["warp_contract"]["interior_exit_warp_indices"] == serde_json::json!([1, 2])
            && value["warp_contract"]["exterior_door_target_warp"] == 3,
        "retain the actual authored cabin, two carpets and passive arrival3"
    );
    ensure!(
        room.residents.is_empty()
            && room.items.is_empty()
            && room.signs.is_empty()
            && room.keeper_tile.is_none()
            && value["dorothy_presence"]["default_review_state"] == "outside_present_inside_empty",
        "default cabin must not create a second Dorothy or invented event"
    );
    let layout = Layout::parse(layout_json).context("native preserve layout")?;
    let binding: PreserveBinding =
        serde_json::from_str(layout_json).context("cabin exterior binding")?;
    let door = &binding.root_beer_lady.door;
    ensure!(
        door.target == ROOT_BEER_CABIN_MAP
            && door.target_warp == 3
            && door.warp_index == 1
            && door.return_warp_index == Some(2)
            && door.collision == "DOOR"
            && door.return_tile == door.arrival_tile
            && door.tile[0] == door.return_tile[0]
            && door.tile[1].checked_add(1) == Some(door.return_tile[1]),
        "cabin door must target passive arrival3 and separate exterior return2"
    );
    validate_exterior_source(&layout, exterior, door)?;
    let exterior_constant = exterior
        .attributes
        .map_constant
        .as_deref()
        .context("exterior constant")?;
    let local = |[x, y]: [u16; 2]| [x % 64, y % 48];
    ensure!(
        collision(
            base,
            &exterior.blocks,
            exterior.attributes.width,
            local(door.tile),
            &exterior.attributes.tileset_name
        )? == "DOOR"
            && collision(
                base,
                &exterior.blocks,
                exterior.attributes.width,
                local(door.return_tile),
                &exterior.attributes.tileset_name
            )? == "FLOOR"
            && !exterior
                .objects
                .iter()
                .any(|actor| [actor.x, actor.y] == local(door.return_tile)),
        "cabin exterior door/return must be actual DOOR and unoccupied FLOOR"
    );
    let dorothys = binding
        .landmark_actors
        .iter()
        .filter(|actor| actor.id == "BWS_ROOT_BEER_LADY")
        .collect::<Vec<_>>();
    ensure!(
        dorothys.len() == 1,
        "one actual exterior Dorothy source anchor required"
    );
    let dorothy = dorothys[0];
    ensure!(dorothy.sprite == "SPRITE_GRANNY", "native Dorothy sprite");
    let actors = exterior
        .objects
        .iter()
        .filter(|actor| actor.object_identifier.as_deref() == Some(dorothy.id.as_str()))
        .collect::<Vec<_>>();
    ensure!(
        actors.len() == 1
            && [actors[0].x, actors[0].y] == local(dorothy.tile)
            && actors[0].sprite == dorothy.sprite,
        "actual exterior Dorothy identity/anchor differs from current source"
    );
    let mut cabin = authoring::blank(base, &room.template, &room.id, ROOT_BEER_CABIN_CONSTANT)?;
    ensure!(
        cabin.attributes.tileset_name == "traditional_house",
        "native cabin template art"
    );
    authoring::blocks(&mut cabin, 5, 4, &room.rows.join(" "))?;
    cabin.attributes.border_block = 0;
    cabin.attributes.location = exterior.attributes.location.clone();
    cabin.attributes.map_group_constant = exterior.attributes.map_group_constant.clone();
    validate_room(base, &cabin, &room)?;
    validate_counter(base, &cabin, &value)?;
    for &exit in &room.exits {
        authoring::warp(&mut cabin, (exit[0], exit[1]), exterior_constant, 2);
    }
    authoring::warp(
        &mut cabin,
        (arrival.tile[0], arrival.tile[1]),
        exterior_constant,
        2,
    );
    authoring::finish(base, &mut cabin)?;
    let mut bound_exterior = exterior.clone();
    bind_warp(
        &mut bound_exterior,
        1,
        local(door.tile),
        ROOT_BEER_CABIN_CONSTANT,
        3,
    )?;
    bind_warp(
        &mut bound_exterior,
        2,
        local(door.return_tile),
        ROOT_BEER_CABIN_CONSTANT,
        3,
    )?;
    let item_pack = if base.data().items.contains_key(ROOT_BEER_ITEM_ID) {
        base.clone()
    } else {
        build_root_beer_lady_modpack(base)?
    };
    let service = prepare_root_beer_lady_service(&item_pack, &bound_exterior, &dorothy.id, rest)?;
    Ok(PreparedRootBeerCabin {
        pack: service.pack,
        exterior: service.module,
        cabin,
        arrival: arrival.tile,
        exterior_return: local(door.return_tile),
        service_script: service.script_name,
    })
}

fn validate_exterior_source(layout: &Layout, exterior: &MapModule, door: &Door) -> Result<()> {
    ensure!(
        layout.id == "MN_BOUNDARY_WATERS_SAFARI" && layout.compiled_tileset == "mn_boundary_safari",
        "actual Boundary Waters layout required"
    );
    let name = format!(
        "MnBoundarySafariC{:02}R{:02}",
        door.tile[0] / 64,
        door.tile[1] / 48
    );
    let [column, row] = [door.tile[0] / 64, door.tile[1] / 48];
    let [width, height] = layout.dimensions_metatiles;
    let origin = [column * 32, row * 24];
    ensure!(
        origin[0] < width && origin[1] < height,
        "cabin door outside preserve"
    );
    ensure!(
        door.return_tile[0] / 64 == column && door.return_tile[1] / 48 == row,
        "cabin return must remain in its actual exterior chunk"
    );
    let clipped = [(width - origin[0]).min(32), (height - origin[1]).min(24)];
    ensure!(
        exterior.id == name
            && [exterior.attributes.width, exterior.attributes.height] == clipped
            && exterior.attributes.tileset_name == layout.compiled_tileset,
        "supplied cabin exterior must be its actual clipped preserve chunk"
    );
    let blocks = layout.compile()?;
    let mut expected = Vec::new();
    for y in origin[1]..origin[1] + clipped[1] {
        let offset = usize::from(y) * usize::from(width) + usize::from(origin[0]);
        expected.extend_from_slice(&blocks[offset..offset + usize::from(clipped[0])]);
    }
    ensure!(
        exterior.blocks == expected,
        "cabin exterior blocks differ from authored preserve"
    );
    Ok(())
}

fn bind_warp(
    module: &mut MapModule,
    index: u16,
    tile: [u16; 2],
    constant: &str,
    target_id: i16,
) -> Result<()> {
    let warp = crystal_core::map::WarpEvent {
        index,
        x: tile[0],
        y: tile[1],
        target_map_constant: constant.into(),
        // Native WarpEvent stores the exact constant in both target fields;
        // GameData::resolve_warp_transition resolves that constant to a map name.
        target_map: constant.into(),
        target_warp_id: target_id,
    };
    if let Some(existing) = module
        .events
        .warps
        .iter()
        .find(|existing| existing.index == index)
    {
        ensure!(
            existing == &warp,
            "conflicting existing cabin door/return record"
        );
    } else {
        ensure!(
            usize::from(index) == module.events.warps.len() + 1
                && !module
                    .events
                    .warps
                    .iter()
                    .any(|existing| [existing.x, existing.y] == tile),
            "cabin exterior warp order or position conflict"
        );
        module.events.warps.push(warp);
    }
    Ok(())
}

fn validate_counter(pack: &CompiledGamePack, cabin: &MapModule, source: &Value) -> Result<()> {
    let kind = |tile| collision(pack, &cabin.blocks, 5, tile, "traditional_house");
    let point = |value: &Value| -> Result<[u16; 2]> {
        Ok([
            u16::try_from(value[0].as_u64().context("counter x")?)?,
            u16::try_from(value[1].as_u64().context("counter y")?)?,
        ])
    };
    let counter = &source["service_counter"];
    ensure!(
        point(&counter["dorothy_anchor"])? == [3, 3]
            && point(&counter["customer_approach"])? == [5, 3]
            && point(&counter["interaction_counter_tile"])? == [4, 3]
            && counter["customer_facing"] == "left"
            && kind([3, 3])? == "FLOOR"
            && kind([5, 3])? == "FLOOR"
            && kind([4, 3])? == "COUNTER",
        "actual native two-tile cabin counter contract"
    );
    let counter_tiles = counter["native_counter_tiles"]
        .as_array()
        .context("native counter tiles")?;
    ensure!(
        counter_tiles.len() == 4,
        "retain all four authored counter quadrants"
    );
    for tile in counter_tiles {
        ensure!(
            kind(point(tile)?)? == "COUNTER",
            "cabin counter is not native COUNTER collision"
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "root_beer_cabin_tests.rs"]
mod tests;
