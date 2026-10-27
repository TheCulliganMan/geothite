//! Additive native publication and collision checks for the authored gym rooms.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring;
use crate::gym_design::Gym;
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RuntimeMapMetadata,
    RuntimeSpawnPoint,
};

pub(crate) const GROUP_ID: u16 = 63;
pub(crate) const HALL: &str = "MnGymReviewHall";
pub(crate) const HALL_CONSTANT: &str = "MN_GYM_REVIEW_HALL";
const GROUP: &str = "MINNESOTA_GYMS";
const LANDMARK: &str = "LANDMARK_MN_GYM_REVIEW";

pub(crate) fn room(
    pack: &CompiledGamePack,
    gym: &Gym,
    symbols: &BTreeMap<char, u16>,
    index: usize,
) -> Result<MapModule> {
    let mut module = authoring::blank(pack, "VioletGym", &gym.id, &gym.constant)?;
    module.attributes.width = gym.dimensions_metatiles[0];
    module.attributes.height = gym.dimensions_metatiles[1];
    module.attributes.tileset_name = gym.tileset();
    module.attributes.border_block = u8::try_from(symbols[&'#'])?;
    module.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
    module.attributes.location = Some(LANDMARK.into());
    module.blocks = gym
        .rows
        .iter()
        .flat_map(|row| row.chars().map(|c| symbols[&c]))
        .collect();
    let hall_return = i16::try_from(index * 3 + 3)?;
    for [x, y] in gym.exit_tiles_core.into_iter().chain([gym.arrival_core]) {
        authoring::warp(&mut module, (x, y), HALL_CONSTANT, hall_return);
    }
    for lift in &gym.lifts {
        let first = i16::try_from(module.events.warps.len() + 1)?;
        let [a, b] = lift.ends_core;
        authoring::warp(&mut module, (a[0], a[1]), &gym.constant, first + 1);
        authoring::warp(&mut module, (b[0], b[1]), &gym.constant, first);
    }
    Ok(module)
}

pub(crate) fn hall(pack: &CompiledGamePack, gyms: &[Gym]) -> Result<MapModule> {
    let mut hall = authoring::blank(pack, "VioletGym", HALL, HALL_CONSTANT)?;
    // A clearly separate review facility; these are not city facades.
    if gyms.len() == 1 {
        authoring::blocks(
            &mut hall,
            5,
            5,
            "
            14 14 14 14 14
            14 02 3c 02 14
            14 02 02 02 14
            14 02 02 02 14
            14 14 14 14 14
        ",
        )?;
    } else {
        authoring::blocks(
            &mut hall,
            18,
            10,
            "
        14 14 14 14 14 14 14 14 14 14 14 14 14 14 14 14 14 14
        14 02 3c 02 02 02 3c 02 02 02 3c 02 02 02 3c 02 02 14
        14 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 14
        14 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 14
        14 02 02 02 07 02 02 02 02 02 02 02 07 02 02 02 02 14
        14 02 02 02 07 02 02 02 02 02 02 02 07 02 02 02 02 14
        14 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 14
        14 02 3c 02 02 02 3c 02 02 02 3c 02 02 02 3c 02 02 14
        14 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 02 14
        14 14 14 14 14 14 14 14 14 14 14 14 14 14 14 14 14 14
    ",
        )?;
    }
    hall.attributes.border_block = 0x14;
    hall.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
    hall.attributes.location = Some(LANDMARK.into());
    for (index, gym) in gyms.iter().enumerate() {
        let x = 4 + u16::try_from(index % 4)? * 8;
        let y = if index < 4 { 2 } else { 14 };
        authoring::warp(&mut hall, (x, y), &gym.constant, 3);
        authoring::warp(&mut hall, (x + 1, y), &gym.constant, 3);
        authoring::warp(&mut hall, (x, y + 3), &gym.constant, 3);
        let label = format!("MnGymReviewDoor{}", gym.slot);
        authoring::speech(
            pack,
            &mut hall,
            &label,
            "SPRITE_GYM_GUIDE",
            (i16::try_from(x + 2)?, i16::try_from(y + 2)?),
            &[
                &format!("{}: {}", gym.slot, gym.city),
                &format!("Leader {}", gym.leader),
                "Review room only.",
            ],
        )?;
    }
    authoring::speech(
        pack,
        &mut hall,
        "Welcome",
        "SPRITE_SCIENTIST",
        if gyms.len() == 1 { (3, 6) } else { (17, 10) },
        &[
            "GYM INTERIOR REVIEW",
            "Cities are not",
            "connected here.",
            "Use the marked door.",
        ],
    )?;
    crate::gym_scripts::finish(pack, &mut hall)?;
    Ok(hall)
}

fn kind<'a>(pack: &'a CompiledGamePack, module: &MapModule, at: [u16; 2]) -> Result<&'a str> {
    let [x, y] = at;
    ensure!(
        x < module.attributes.width * 2 && y < module.attributes.height * 2,
        "{}: coordinate bounds {at:?}",
        module.id
    );
    let block = module.blocks
        [usize::from(y / 2) * usize::from(module.attributes.width) + usize::from(x / 2)];
    let quadrants = pack.data.tilesets[&module.attributes.tileset_name]
        .collision
        .get(&format!("{block:02x}"))
        .context("gym collision")?;
    Ok(&quadrants[usize::from(y % 2 * 2 + x % 2)])
}

/// Rejects invisible bypasses, blocked actors and unsafe room exits before export.
pub(crate) fn validate(
    pack: &CompiledGamePack,
    module: &MapModule,
    gym: &Gym,
    symbols: &BTreeMap<char, u16>,
) -> Result<()> {
    ensure!(
        kind(pack, module, gym.arrival_core)? == "FLOOR",
        "{}: passive arrival",
        gym.id
    );
    for at in gym.exit_tiles_core {
        ensure!(
            kind(pack, module, at)? == "WARP_CARPET_DOWN",
            "{}: native carpet",
            gym.id
        );
    }
    let occupied: BTreeSet<_> = module.objects.iter().map(|o| [o.x, o.y]).collect();
    ensure!(occupied.len() == module.objects.len(), "gym actors overlap");
    crate::gym_lifts::validate(pack, module, gym, &occupied)?;
    crate::gym_cut::validate_geometry(pack, module, gym, &occupied)?;
    ensure!(
        module.attributes.connections.is_empty(),
        "gym rooms must be bounded without connections"
    );
    let flood = |room: &MapModule| -> Result<BTreeSet<[u16; 2]>> {
        let reached = crate::gym_lifts::reachable(pack, room, gym, &occupied)?;
        if gym.enclosed_pool {
            for y in 0..room.attributes.height * 2 {
                for x in 0..room.attributes.width * 2 {
                    if x == 0
                        || y == 0
                        || x + 1 == room.attributes.width * 2
                        || y + 1 == room.attributes.height * 2
                    {
                        ensure!(
                            matches!(
                                kind(pack, room, [x, y])?,
                                "WALL" | "FLOOR" | "WARP_CARPET_DOWN"
                            ),
                            "{}: enclosed pool must retain a dry bounded perimeter",
                            gym.id
                        );
                    }
                }
            }
        }
        // A drained pool can become water again. Check the reachable bank in
        // this phase, never old dry cells against a later flooded map.
        for [x, y] in reached.iter().filter(|_| !gym.enclosed_pool) {
            for at in [
                x.checked_sub(1).map(|x| [x, *y]),
                y.checked_sub(1).map(|y| [*x, y]),
                (*x + 1 < room.attributes.width * 2).then_some([*x + 1, *y]),
                (*y + 1 < room.attributes.height * 2).then_some([*x, *y + 1]),
            ]
            .into_iter()
            .flatten()
            {
                ensure!(
                    !matches!(kind(pack, room, at)?, "WATER" | "WATERFALL"),
                    "{}: unprotected water beside walking deck at {at:?}",
                    gym.id
                );
            }
        }
        Ok(reached)
    };
    let reached = flood(module)?;
    let adjacent = |set: &BTreeSet<[u16; 2]>, [x, y]: [u16; 2]| {
        set.iter().any(|p| p[0].abs_diff(x) + p[1].abs_diff(y) == 1)
    };
    ensure!(
        adjacent(&reached, gym.leader_at) != gym.leader_court_locked,
        "{}: leader court access differs from the authored room",
        gym.id
    );
    let mut opened = module.clone();
    let mut completed = BTreeSet::new();
    let mut ever_reached = reached;
    for step in &gym.walkthrough {
        let (at, event, stand_on) = if let Some(trainer) =
            gym.trainers.iter().find(|t| &t.id == step)
        {
            ensure!(
                trainer.requires.iter().all(|r| completed.contains(r)),
                "{}: {} has unearned prerequisites",
                gym.id,
                trainer.id
            );
            (trainer.at, format!("EVENT_{}_DEFEATED", trainer.id), false)
        } else if let Some(gate) = gym.cut_gates.iter().find(|gate| &gate.id == step) {
            let reachable = flood(&opened)?;
            ensure!(
                reachable.contains(&gate.approach_core),
                "{}: Cut approach is unreachable for {step}",
                gym.id
            );
            let index = usize::from(gate.tree_core[1] / 2) * usize::from(module.attributes.width)
                + usize::from(gate.tree_core[0] / 2);
            opened.blocks[index] = symbols[&gate.replacement_symbol];
            (gate.approach_core, gate.event.clone(), true)
        } else if let Some(pad) = gym.pads.iter().find(|p| &p.id == step) {
            ensure!(
                pad.requires.iter().all(|r| completed.contains(r)),
                "{}: {} has unearned prerequisites",
                gym.id,
                pad.id
            );
            (
                *pad.tiles_core.first().context("floor pad needs a tile")?,
                pad.event.clone(),
                true,
            )
        } else if let Some((router, mode)) = gym.routing_mode(step) {
            ensure!(
                mode.requires.iter().all(|r| completed.contains(r)),
                "{}: {} has unearned prerequisites",
                gym.id,
                mode.id
            );
            for other in &router.modes {
                if other.event != mode.event {
                    completed.remove(&other.event);
                }
            }
            (router.at, mode.event.clone(), false)
        } else if let Some((puzzle, control, index)) = gym.switch_step(step) {
            ensure!(
                puzzle.requires.iter().all(|r| completed.contains(r))
                    && (index == 0 || completed.contains(&puzzle.events[0])),
                "{}: switch {step} has unearned prerequisites",
                gym.id
            );
            (control.at, puzzle.events[index].clone(), false)
        } else {
            let station = gym
                .stations
                .iter()
                .find(|s| &s.id == step)
                .context("unknown walkthrough station")?;
            ensure!(
                station.requires.iter().all(|r| completed.contains(r)),
                "{}: {} has unearned prerequisites",
                gym.id,
                station.id
            );
            (station.at, station.event.clone(), false)
        };
        let reachable = flood(&opened)?;
        ensure!(
            if stand_on {
                reachable.contains(&at)
            } else {
                adjacent(&reachable, at)
            },
            "{}: walkthrough cannot approach {step} at {at:?}",
            gym.id
        );
        ever_reached.extend(reachable);
        completed.insert(event);
        for gate in &gym.gates {
            let index = usize::from(gate.at[1]) * usize::from(module.attributes.width)
                + usize::from(gate.at[0]);
            if gate.requires.iter().all(|r| completed.contains(r)) {
                opened.blocks[index] = symbols[&gate.open_symbol];
                completed.insert(gate.event.clone());
            } else {
                opened.blocks[index] = module.blocks[index];
                completed.remove(&gate.event);
            }
        }
    }
    let open_reached = flood(&opened)?;
    if let Some(puzzle) = &gym.switch_puzzle {
        for control in &puzzle.controls {
            ensure!(
                matches!(kind(pack, module, control.at)?, "WALL" | "COUNTER")
                    && adjacent(&open_reached, control.at),
                "{}: switch {} must be a reachable solid fixture",
                gym.id,
                control.id
            );
        }
    }
    ensure!(
        gym.court_requires.iter().all(|r| completed.contains(r)),
        "{}: walkthrough does not qualify for the leader",
        gym.id
    );
    ever_reached.extend(&open_reached);
    crate::gym_ledges::validate(pack, &opened, gym, &occupied)?;
    for pad in &gym.pads {
        for at in &pad.tiles_core {
            ensure!(
                kind(pack, module, *at)? == "FLOOR",
                "{}: floor pad {} must be walkable at {at:?}",
                gym.id,
                pad.id
            );
        }
    }
    for (id, at) in gym.stations.iter().map(|s| (&s.id, s.at)).chain(
        gym.routers
            .iter()
            .filter(|r| r.operator.is_none())
            .map(|r| (&r.id, r.at)),
    ) {
        ensure!(
            matches!(kind(pack, module, at)?, "WALL" | "COUNTER"),
            "{}: control {} must belong to a solid fixture",
            gym.id,
            id
        );
    }
    for router in &gym.routers {
        let mut recoverable = adjacent(&open_reached, router.at);
        // A lane switch may temporarily isolate another control. Accept that
        // only when an accessible, already-qualified toggle can restore it.
        // Ordinary routing desks retain their original direct-access contract.
        if router.toggle && !recoverable {
            for control in gym
                .routers
                .iter()
                .filter(|r| r.toggle && adjacent(&open_reached, r.at))
            {
                for mode in &control.modes {
                    if !mode.requires.iter().all(|r| completed.contains(r)) {
                        continue;
                    }
                    let mut flags = completed.clone();
                    for other in &control.modes {
                        flags.remove(&other.event);
                    }
                    flags.insert(mode.event.clone());
                    let mut restored = opened.clone();
                    for gate in &gym.gates {
                        let index = usize::from(gate.at[1]) * usize::from(module.attributes.width)
                            + usize::from(gate.at[0]);
                        restored.blocks[index] = if gate.requires.iter().all(|r| flags.contains(r))
                        {
                            symbols[&gate.open_symbol]
                        } else {
                            module.blocks[index]
                        };
                    }
                    recoverable |= adjacent(&flood(&restored)?, router.at);
                }
            }
        }
        ensure!(
            recoverable,
            "{}: control becomes unreachable and cannot be restored",
            gym.id
        );
    }
    for actor in &module.objects {
        let at = [actor.x, actor.y];
        ensure!(
            kind(pack, module, at)? == "FLOOR",
            "{}: actor {} is on {} at {at:?}",
            gym.id,
            actor.script,
            kind(pack, module, at)?
        );
        ensure!(
            adjacent(&ever_reached, at),
            "{}: inaccessible actor {} at {at:?}",
            gym.id,
            actor.script
        );
    }
    ensure!(
        gym.exit_tiles_core.iter().all(|[x, y]| y
            .checked_sub(1)
            .is_some_and(|y| open_reached.contains(&[*x, y]))),
        "{}: return blocked",
        gym.id
    );
    let width = module.attributes.width * 2;
    let height = module.attributes.height * 2;
    for y in 0..height {
        for x in 0..width {
            if (x == 0 || x == width - 1 || y == 0 || y == height - 1)
                && !gym.exit_tiles_core.contains(&[x, y])
            {
                ensure!(
                    matches!(kind(pack, module, [x, y])?, "WALL" | "FLOOR"),
                    "{}: open outer edge ({x},{y})",
                    gym.id
                );
            }
        }
    }
    Ok(())
}

pub(crate) fn publish(
    mut pack: CompiledGamePack,
    modules: Vec<(MapModule, [u16; 2])>,
) -> Result<CompiledGamePack> {
    ensure!(
        !pack
            .data
            .runtime_map_metadata
            .values()
            .any(|m| m.group_id == GROUP_ID),
        "gym group63 occupied"
    );
    pack.data
        .story_event_script_constants
        .global
        .insert(format!("MAPGROUP_{GROUP}"), i64::from(GROUP_ID));
    let id = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|l| l.id)
        .max()
        .context("landmark catalog")?
        .checked_add(1)
        .context("landmark overflow")?;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id,
            constant: LANDMARK.into(),
            label: "MN_GYM_REVIEW".into(),
            name: "GYM REVIEW".into(),
            x: 0,
            y: 0,
            region: "JOHTO".into(),
        });
    let temporary = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("recovery table")?
        .checked_add(1)
        .context("temporary spawn overflow")?;
    for (index, (module, [x, y])) in modules.into_iter().enumerate() {
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("gym constant")?;
        let map_id = u16::try_from(index + 1)?;
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), LANDMARK.into());
        crate::insert_compiled_map_extension(
            &mut pack,
            CompiledMapExtension {
                manifest_id: format!("minnesota-gym-interiors-v1-{index}"),
                map_name: name.clone(),
                map_constant: constant.clone(),
                metadata: RuntimeMapMetadata {
                    name: name.clone(),
                    constant: constant.clone(),
                    group_name: GROUP.into(),
                    group_id: GROUP_ID,
                    map_id,
                    width: module.attributes.width,
                    height: module.attributes.height,
                    environment: module
                        .attributes
                        .environment
                        .clone()
                        .context("gym environment")?,
                    phone_service: module.attributes.phone_service,
                },
                spawn_key: temporary.to_string(),
                spawn: RuntimeSpawnPoint {
                    identifier: temporary,
                    map_constant: constant,
                    map_name: name,
                    group_name: GROUP.into(),
                    group_id: i16::try_from(GROUP_ID)?,
                    map_id: i16::try_from(map_id)?,
                    tile_x: i16::try_from(x)?,
                    tile_y: i16::try_from(y)?,
                    metatile_x: i16::try_from(x / 2)?,
                    metatile_y: i16::try_from(y / 2)?,
                    subtile_x: i16::try_from(x % 2)?,
                    subtile_y: i16::try_from(y % 2)?,
                },
                module,
                wild_encounters: None,
                start_new_game_here: false,
            },
        )?;
        pack.data
            .runtime_spawn_points
            .remove(&temporary.to_string());
    }
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
