//! Native walking connections and visible checked passages at the free camp.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::{CoordEvent, MapConnection};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const CAMP: &str = "MnBoundaryWaters";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const MANIFEST: &str = "minnesota-boundary-basecamp-connections-v1";
const BADGES: [&str; 8] = [
    "ENGINE_ZEPHYRBADGE",
    "ENGINE_HIVEBADGE",
    "ENGINE_PLAINBADGE",
    "ENGINE_FOGBADGE",
    "ENGINE_STORMBADGE",
    "ENGINE_MINERALBADGE",
    "ENGINE_GLACIERBADGE",
    "ENGINE_RISINGBADGE",
];

struct Passage {
    route: &'static str,
    outward: &'static str,
    reverse: &'static str,
    offset: i32,
    camp_edge: [[u16; 2]; 2],
    route_edge: [[u16; 2]; 2],
    bodies: [[u16; 2]; 2],
    inside: [[u16; 2]; 2],
    outside: [[u16; 2]; 2],
    identifiers: [&'static str; 2],
    root: &'static str,
    incoming_facing: &'static str,
    needs_restored: bool,
}

const PASSAGES: [Passage; 3] = [
    Passage {
        route: "MnR12C00R01",
        outward: "east",
        reverse: "west",
        offset: 8,
        camp_edge: [[63, 24], [63, 25]],
        route_edge: [[0, 8], [0, 9]],
        bodies: [[61, 24], [61, 25]],
        inside: [[60, 24], [60, 25]],
        outside: [[62, 24], [62, 25]],
        identifiers: [
            "MN_BOUNDARY_GUNFLINT_GUARD_NORTH",
            "MN_BOUNDARY_GUNFLINT_GUARD_SOUTH",
        ],
        root: "MnBoundaryGunflintCheck",
        incoming_facing: "LEFT",
        needs_restored: false,
    },
    Passage {
        route: "MnR13C00R00",
        outward: "south",
        reverse: "north",
        offset: 1,
        camp_edge: [[32, 47], [33, 47]],
        route_edge: [[30, 0], [31, 0]],
        bodies: [[32, 43], [33, 43]],
        inside: [[32, 42], [33, 42]],
        outside: [[32, 44], [33, 44]],
        identifiers: [
            "MN_BOUNDARY_LEAGUE_GUARD_WEST",
            "MN_BOUNDARY_LEAGUE_GUARD_EAST",
        ],
        root: "MnBoundaryLeagueCheck",
        incoming_facing: "UP",
        needs_restored: true,
    },
    Passage {
        route: "MnR20C01R00",
        outward: "west",
        reverse: "east",
        offset: -6,
        camp_edge: [[0, 24], [0, 25]],
        route_edge: [[63, 36], [63, 37]],
        bodies: [[2, 24], [2, 25]],
        inside: [[3, 24], [3, 25]],
        outside: [[1, 24], [1, 25]],
        identifiers: [
            "MN_BOUNDARY_VOYAGEURS_GUARD_NORTH",
            "MN_BOUNDARY_VOYAGEURS_GUARD_SOUTH",
        ],
        root: "MnBoundaryVoyageursCheck",
        incoming_facing: "RIGHT",
        needs_restored: false,
    },
];

/// Adds three reciprocal walking seams and their visible, native checkpoints.
///
/// Requires the actual published free town and current R12/R13/R20 chunks.
/// Existing edge Floor already aligns; no metatiles, warps, original actors,
/// room/service scripts, events, badges, source files or audio are replaced.
/// Six additive Officers use temporary native visibility (`event_flag=-1`),
/// never a story flag as a visibility switch. Each inner plane checks all eight
/// badges and the lighthouse signal; the League also requires restoration.
/// Refusal pushes inward. Inward and transverse outer-plane arrivals are
/// escorted two native steps through the cleared pair to free public care;
/// only movement facing outward skips the care escort.
/// Officers reappear if requirements remain unmet. Fresh map modules begin
/// closed; saved visibility and map-entry resets follow the native controller,
/// without an invented permission event or admission state.
///
/// The operation is atomic: the borrowed input remains unchanged on error. The
/// result is not exported. Native controller proof remains necessary for input
/// ownership, forced movement, direction, temporary visibility and Continue.
///
/// # Errors
/// Returns an error for missing/misaligned native geometry, occupied anchors,
/// conflicting connections/scripts/objects, missing native flag/direction
/// constants, a walking/Surf guard bypass, parser/identity errors or invalid
/// native connection resolution. It never returns an unguarded connected pack.
pub fn connect_boundary_basecamp(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "Boundary basecamp walking connections already installed"
    );
    let mut pack = base.clone();
    let mut camp = pack
        .data
        .maps
        .get(CAMP)
        .context("actual published free camp")?
        .clone();
    ensure!(
        [camp.attributes.width, camp.attributes.height] == [32, 24]
            && camp.attributes.tileset_name == "mn_boundary_safari",
        "Boundary camp geometry/tileset changed"
    );
    for name in BADGES.into_iter().chain([
        SIGNAL,
        RESTORED,
        "VAR_FACING",
        "UP",
        "DOWN",
        "LEFT",
        "RIGHT",
    ]) {
        ensure!(
            pack.data
                .story_event_script_constants
                .global
                .contains_key(name),
            "missing native checkpoint constant {name}"
        );
    }
    validate_geometry(&pack, &camp)?;
    for passage in &PASSAGES {
        add_guards(&pack, &mut camp, passage)?;
        add_connection(&mut camp, passage.outward, passage.route, passage.offset)?;
        let mut route = pack
            .data
            .maps
            .get(passage.route)
            .context("actual published neighboring route")?
            .clone();
        ensure!(
            [route.attributes.width, route.attributes.height] == [32, 24],
            "changed neighboring clipped chunk extent"
        );
        add_connection(&mut route, passage.reverse, CAMP, -passage.offset)?;
        install_module(&mut pack, route)?;
    }
    authoring::finish(&pack, &mut camp)?;
    camp.script_object_commands = crate::parse_script_object_commands(CAMP, &camp.scripts)?;
    camp.script_movements =
        crate::parse_script_movements(CAMP, &camp.scripts, &camp.script_object_commands)?;
    camp.script_variable_commands = crate::parse_script_variable_commands(CAMP, &camp.scripts)?;
    install_module(&mut pack, camp)?;
    validate_connections(&pack)?;
    pack.report.manifests.push(MANIFEST.into());
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

fn add_connection(
    module: &mut MapModule,
    direction: &str,
    target: &str,
    offset: i32,
) -> Result<()> {
    ensure!(
        !module
            .attributes
            .connections
            .iter()
            .any(|c| c.direction == direction),
        "{} already has a {direction} connection",
        module.id
    );
    module.attributes.connections.push(MapConnection {
        direction: direction.into(),
        target_map: target.into(),
        offset,
    });
    module.attributes.connection_flags = Some(
        module
            .attributes
            .connections
            .iter()
            .map(|c| c.direction.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    Ok(())
}

fn install_module(pack: &mut CompiledGamePack, module: MapModule) -> Result<()> {
    let scripts = module
        .attributes
        .map_scripts_label
        .as_deref()
        .context("native script section")?;
    let events = module
        .attributes
        .map_events_label
        .as_deref()
        .context("native event section")?;
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &module.id,
        scripts,
        events,
        &module,
    )?;
    pack.data
        .map_attributes
        .insert(module.id.clone(), module.attributes.clone());
    pack.data.maps.insert(module.id.clone(), module);
    Ok(())
}

fn floor(pack: &CompiledGamePack, module: &MapModule, tile: [u16; 2]) -> Result<()> {
    ensure!(
        crate::authored_route_modpack::collision(
            pack,
            &module.blocks,
            module.attributes.width,
            tile,
            &module.attributes.tileset_name
        )? == "FLOOR",
        "{} checked passage {tile:?} is not ordinary Floor",
        module.id
    );
    Ok(())
}

fn validate_geometry(pack: &CompiledGamePack, camp: &MapModule) -> Result<()> {
    let mut occupied = BTreeSet::new();
    for passage in &PASSAGES {
        let route = pack
            .data
            .maps
            .get(passage.route)
            .context("native neighboring route")?;
        for tile in passage
            .camp_edge
            .into_iter()
            .chain(passage.bodies)
            .chain(passage.inside)
            .chain(passage.outside)
        {
            floor(pack, camp, tile)?;
            ensure!(
                !camp.objects.iter().any(|actor| [actor.x, actor.y] == tile),
                "checkpoint overlaps original camp actor"
            );
            ensure!(
                !camp
                    .events
                    .coord_events
                    .iter()
                    .any(|event| [event.x, event.y] == tile),
                "checkpoint overlaps original coordinate event"
            );
        }
        for tile in passage.route_edge {
            floor(pack, route, tile)?;
        }
        let [dx, dy] = match passage.incoming_facing {
            "LEFT" => [-1, 0],
            "RIGHT" => [1, 0],
            "UP" => [0, -1],
            _ => unreachable!("fixed native inward step"),
        };
        for ((outside, body), inside) in passage
            .outside
            .into_iter()
            .zip(passage.bodies)
            .zip(passage.inside)
        {
            let moved = |tile: [u16; 2]| -> Result<[u16; 2]> {
                Ok([
                    u16::try_from(i32::from(tile[0]) + dx)?,
                    u16::try_from(i32::from(tile[1]) + dy)?,
                ])
            };
            ensure!(
                moved(outside)? == body && moved(body)? == inside,
                "inbound escort must cross only its cleared guard body"
            );
            let refusal_landing = moved(inside)?;
            floor(pack, camp, refusal_landing)?;
            ensure!(
                !camp
                    .objects
                    .iter()
                    .any(|actor| [actor.x, actor.y] == refusal_landing),
                "checkpoint refusal hits original actor"
            );
        }
        // Coordinate planes are the logical barrier. Clear every ordinary NPC
        // for this conservative bypass proof, including the new guard bodies.
        let blocked = BTreeSet::from(passage.inside);
        let seen =
            crate::authored_route_surface::reachable_map_with_surf(pack, camp, [43, 20], &blocked)?;
        ensure!(
            passage.camp_edge.iter().all(|p| !seen.contains(p)),
            "{} checkpoint has a walking/Surf bypass",
            passage.route
        );
        occupied.extend(passage.inside);
    }
    let free =
        crate::authored_route_surface::reachable_map_with_surf(pack, camp, [43, 20], &occupied)?;
    for tile in [[56, 24], [32, 40], [6, 24], [13, 14], [43, 20]] {
        ensure!(
            free.contains(&tile),
            "closed checkpoints deny free camp care/dock/landing {tile:?}"
        );
    }
    Ok(())
}

fn checks(
    passage: &Passage,
    badge_denied: &str,
    signal_denied: &str,
    restored_denied: &str,
) -> Vec<Value> {
    let mut commands = Vec::new();
    for badge in BADGES {
        commands.extend([
            authoring::command("checkflag", &[badge]),
            authoring::command("iffalse", &[badge_denied]),
        ]);
    }
    commands.extend([
        authoring::command("checkevent", &[SIGNAL]),
        authoring::command("iffalse", &[signal_denied]),
    ]);
    if passage.needs_restored {
        commands.extend([
            authoring::command("checkevent", &[RESTORED]),
            authoring::command("iffalse", &[restored_denied]),
        ]);
    }
    commands
}

fn visibility(passage: &Passage, command: &str) -> Vec<Value> {
    passage
        .identifiers
        .iter()
        .map(|id| authoring::command(command, &[id]))
        .collect()
}

fn outward_facing(passage: &Passage) -> &'static str {
    match passage.outward {
        "east" => "RIGHT",
        "south" => "DOWN",
        "west" => "LEFT",
        _ => unreachable!("fixed native camp exit direction"),
    }
}

fn add_guards(pack: &CompiledGamePack, camp: &mut MapModule, passage: &Passage) -> Result<()> {
    let root = passage.root;
    let badge = format!("{root}Badges");
    let signal = format!("{root}Signal");
    let restored = format!("{root}Restored");
    let inward = format!("{root}Inward");
    let show = format!("{root}Show");
    let reconcile = format!("{root}Reconcile");
    let done = format!("{root}End");
    let two_steps = format!("{root}TwoSteps");
    let one_step = format!("{root}OneStep");
    let mut scripts = std::collections::BTreeMap::new();
    let mut checked = checks(passage, &badge, &signal, &restored);
    checked.extend(visibility(passage, "disappear"));
    checked.push(authoring::command("end", &[]));
    scripts.insert(root.to_string(), Value::Array(checked));
    for (label, lines) in [
        (&badge, vec!["Eight BADGES first", "Care stays free."]),
        (
            &signal,
            vec![
                "The harbor signal",
                "has not reached us",
                "Care stays free.",
            ],
        ),
        (
            &restored,
            vec![
                "Restore the survey",
                "at the east desk.",
                "Care stays free.",
            ],
        ),
    ] {
        let text = format!("{label}Text");
        ensure!(
            lines.iter().all(|line| line.chars().count() <= 18),
            "checkpoint text width"
        );
        ensure!(
            !camp.scripts.contains_key(&text) && !pack.data.map_scripts.contains_key(&text),
            "checkpoint text collision"
        );
        authoring::text(camp, &text, &lines);
        scripts.insert(
            label.clone(),
            json!([
                authoring::command("opentext", &[]),
                authoring::command("writetext", &[&text]),
                authoring::command("waitbutton", &[]),
                authoring::command("closetext", &[]),
                authoring::command("applymovement", &["PLAYER", &one_step]),
                authoring::command("end", &[]),
            ]),
        );
    }
    // This is the native VAR_FACING flow, as used by Script_ForcedMovement.
    // Inbound travelers are never charged or required to qualify for free care.
    let mut incoming = vec![
        authoring::command("readvar", &["VAR_FACING"]),
        authoring::command("ifequal", &[outward_facing(passage), &done]),
    ];
    incoming.extend(visibility(passage, "disappear"));
    incoming.extend([
        authoring::command("applymovement", &["PLAYER", &two_steps]),
        authoring::command("sjump", &[&reconcile]),
    ]);
    scripts.insert(inward.clone(), Value::Array(incoming));
    let mut qualify = checks(passage, &show, &show, &show);
    qualify.push(authoring::command("end", &[]));
    scripts.insert(reconcile, Value::Array(qualify));
    let mut close = visibility(passage, "appear");
    close.push(authoring::command("end", &[]));
    scripts.insert(show, Value::Array(close));
    scripts.insert(done, json!([authoring::command("end", &[])]));
    scripts.insert(
        one_step,
        json!([
            authoring::command("step", &[passage.incoming_facing]),
            authoring::command("step_end", &[])
        ]),
    );
    scripts.insert(
        two_steps,
        json!([
            authoring::command("step", &[passage.incoming_facing]),
            authoring::command("step", &[passage.incoming_facing]),
            authoring::command("step_end", &[])
        ]),
    );
    for (label, commands) in scripts {
        ensure!(
            !camp.scripts.contains_key(&label) && !pack.data.map_scripts.contains_key(&label),
            "checkpoint script collision {label}"
        );
        camp.scripts.insert(label, commands);
    }
    for ((id, [x, y]), (inner, outer)) in passage
        .identifiers
        .into_iter()
        .zip(passage.bodies)
        .zip(passage.inside.into_iter().zip(passage.outside))
    {
        ensure!(
            !camp
                .objects
                .iter()
                .any(|actor| actor.object_identifier.as_deref() == Some(id)),
            "checkpoint object collision {id}"
        );
        authoring::npc(
            pack,
            camp,
            root,
            "SPRITE_OFFICER",
            i16::try_from(x)?,
            i16::try_from(y)?,
        )?;
        let actor = camp.objects.last_mut().context("native Officer inserted")?;
        actor.object_identifier = Some(id.into());
        actor.spritemovedata = format!("SPRITEMOVEDATA_STANDING_{}", passage.incoming_facing);
        actor.object_type = "OBJECTTYPE_SCRIPT".into();
        actor.radius = 0;
        actor.event_flag = "-1".into();
        camp.events.coord_events.extend([
            CoordEvent {
                x: inner[0],
                y: inner[1],
                scene_id: String::new(),
                script_name: root.into(),
            },
            CoordEvent {
                x: outer[0],
                y: outer[1],
                scene_id: String::new(),
                script_name: inward.clone(),
            },
        ]);
    }
    Ok(())
}

fn validate_connections(pack: &CompiledGamePack) -> Result<()> {
    for passage in &PASSAGES {
        for (source, target, direction, edge, expected) in [
            (
                CAMP,
                passage.route,
                passage.outward,
                passage.camp_edge,
                passage.route_edge,
            ),
            (
                passage.route,
                CAMP,
                passage.reverse,
                passage.route_edge,
                passage.camp_edge,
            ),
        ] {
            let module = pack
                .data
                .maps
                .get(source)
                .context("connected native source")?;
            let map = OverworldMapData::from_attributes(
                source,
                &module.attributes,
                module.blocks.clone(),
            );
            for (from, to) in edge.into_iter().zip(expected) {
                let mut crossing = native_tile(from)?;
                match direction {
                    "east" => crossing.x += 1,
                    "west" => crossing.x -= 1,
                    "north" => crossing.y -= 1,
                    "south" => crossing.y += 1,
                    _ => unreachable!("fixed native direction"),
                }
                let trigger = connection_for_tile_checked(&map, crossing)?
                    .context("native boundary trigger")?;
                ensure!(
                    trigger.connection.target_map == target
                        && trigger.connection.direction == direction,
                    "wrong native seam target/direction"
                );
                let transition = pack.data.resolve_connection_transition(&trigger)?;
                ensure!(
                    transition.destination.map_name == target
                        && transition.destination.tile == native_tile(to)?,
                    "native offset does not align paired Floor seam"
                );
                floor(pack, &pack.data.maps[target], to)?;
            }
        }
    }
    Ok(())
}

fn native_tile([x, y]: [u16; 2]) -> Result<TilePosition> {
    Ok(TilePosition::new(i16::try_from(x)?, i16::try_from(y)?))
}

#[cfg(test)]
mod boundary_basecamp_connection_tests {
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crystal_core::state::{GameState, OverworldMemory};
    use crystal_core::systems::script_control::ScriptControlAction;
    use crystal_core::world::map::Direction;
    use crystal_core::world::movement::MovementMode;

    use super::*;

    fn base() -> &'static CompiledGamePack {
        static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
        BASE.get_or_init(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .expect("repository root");
            let path = std::env::var_os("CRYSTAL_BOUNDARY_BASECAMP_CONNECTIONS_BASE_PACK")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    root.join("content-packs/minnesota-boundary-basecamp.crystalpack")
                })
                .canonicalize()
                .expect("actual published bare471-map town/story pack");
            crate::read_verified_compiled_game_pack(path)
                .expect("actual verified town/story publication")
        })
    }

    fn connected() -> &'static CompiledGamePack {
        static CONNECTED: OnceLock<CompiledGamePack> = OnceLock::new();
        CONNECTED.get_or_init(|| {
            connect_boundary_basecamp(base()).expect("atomic native guarded walking connections")
        })
    }

    fn state(at: [u16; 2]) -> GameState {
        let mut state = GameState::reset_wram_for_new_game();
        state.money = 0;
        state.overworld = OverworldMemory::Active {
            map_name: CAMP.into(),
            tile: native_tile(at).expect("authored core tile"),
            facing: Direction::Up,
            mode: MovementMode::Normal,
        };
        state
    }

    fn branch(state: &mut GameState, root: &str, index: usize) -> ScriptControlAction {
        connected()
            .data
            .apply_script_control_command(state, CAMP, CAMP, root, index)
            .expect("native checkpoint branch")
    }

    fn check(state: &mut GameState, root: &str, index: usize) {
        connected()
            .data
            .check_script_flag(state, CAMP, CAMP, root, index)
            .expect("native checkpoint flag check");
    }

    #[test]
    fn boundary_basecamp_connections_preserve_content_and_survive_split_reload() {
        let base = base();
        let pack = connected();
        assert_eq!(base.data.maps.len(), pack.data.maps.len());
        assert_eq!(base.runtime_files, pack.runtime_files);
        assert_eq!(base.audio_manifest, pack.audio_manifest);
        assert_eq!(base.data.tilesets, pack.data.tilesets);
        assert_eq!(base.data.trainers, pack.data.trainers);
        assert_eq!(base.data.pokemon, pack.data.pokemon);
        assert_eq!(base.data.items, pack.data.items);
        assert_eq!(base.data.marts, pack.data.marts);
        assert_eq!(
            base.data.runtime_spawn_points,
            pack.data.runtime_spawn_points
        );
        assert_eq!(
            base.data.runtime_map_metadata,
            pack.data.runtime_map_metadata
        );
        assert_eq!(
            base.data.story_event_script_constants,
            pack.data.story_event_script_constants
        );
        assert_eq!(base.data.map_blocks, pack.data.map_blocks);
        for (name, original) in &base.data.maps {
            let authored = &pack.data.maps[name];
            assert_eq!(original.blocks, authored.blocks);
            assert_eq!(original.events.warps, authored.events.warps);
            if name == CAMP {
                assert_eq!(
                    &authored.objects[..original.objects.len()],
                    original.objects.as_slice()
                );
                assert_eq!(authored.objects.len(), original.objects.len() + 6);
                assert_eq!(
                    authored.events.coord_events.len(),
                    original.events.coord_events.len() + 12
                );
                for (script, body) in &original.scripts {
                    assert_eq!(authored.scripts.get(script), Some(body));
                }
            } else if PASSAGES.iter().any(|p| p.route == name) {
                let mut expected = original.clone();
                let p = PASSAGES.iter().find(|p| p.route == name).expect("neighbor");
                add_connection(&mut expected, p.reverse, CAMP, -p.offset)
                    .expect("one additive neighbor");
                assert_eq!(authored, &expected);
            } else {
                assert_eq!(authored, original, "unrelated native map changed: {name}");
            }
            assert_eq!(
                pack.data
                    .assemble_map_module_from_compiled_payloads(name)
                    .expect("split-payload native reload"),
                *authored
            );
        }
        let camp = &pack.data.maps[CAMP];
        let raw_coords: Vec<_> = camp
            .map_event_section_commands
            .iter()
            .filter(|command| command.command == "coord_event")
            .collect();
        assert_eq!(raw_coords.len(), 12);
        assert!(
            raw_coords
                .iter()
                .all(|command| command.args.get(2).map(String::as_str) == Some("-1")),
            "always-active typed scenes must use the native -1 wire token"
        );
        assert!(
            camp.events
                .coord_events
                .iter()
                .all(|event| event.scene_id.is_empty()),
            "wire normalization must not change typed always-active semantics"
        );
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("round-trip fixture clock")
            .as_nanos();
        let path = std::env::temp_dir()
            .canonicalize()
            .expect("native temporary directory")
            .join(format!(
                "geothite-basecamp-connected-roundtrip-{}-{nonce}.crystalpack",
                std::process::id()
            ));
        pack.write_preserving_storage(&path)
            .expect("write actual full connected CBOR pack");
        let reloaded = crate::read_verified_compiled_game_pack(&path)
            .expect("strict full CBOR decode and runtime verification");
        assert_eq!(
            &reloaded, pack,
            "full connected pack must survive its actual external wire format"
        );
        std::fs::remove_file(path).expect("remove unique temporary pack only, never a render");
        for command in &camp.script_flag_commands {
            if command.source_script.starts_with("MnBoundary") {
                assert!(
                    matches!(command.command.as_str(), "checkflag" | "checkevent"),
                    "checkpoint may check but never grant story/badge flags"
                );
            }
        }
        assert!(
            camp.script_movements
                .iter()
                .filter(|m| m.label.contains("Check"))
                .count()
                >= 6
        );
        assert!(
            connect_boundary_basecamp(pack)
                .expect_err("duplicate integration")
                .to_string()
                .contains("already installed")
        );
    }

    #[test]
    fn boundary_basecamp_connections_resolve_all_twelve_native_crossings_and_offsets() {
        let pack = connected();
        validate_connections(pack).expect("all native triggers and resolver destinations");
        for p in &PASSAGES {
            for (name, direction, target, offset) in [
                (CAMP, p.outward, p.route, p.offset),
                (p.route, p.reverse, CAMP, -p.offset),
            ] {
                let connection = pack.data.maps[name]
                    .attributes
                    .connections
                    .iter()
                    .find(|c| c.direction == direction)
                    .expect("actual reciprocal record");
                assert_eq!(connection.target_map, target);
                assert_eq!(connection.offset, offset);
            }
        }
        let mut wrong = pack.clone();
        let mut camp = wrong.data.maps[CAMP].clone();
        camp.attributes
            .connections
            .iter_mut()
            .find(|c| c.direction == "east")
            .expect("east")
            .offset = 9;
        install_module(&mut wrong, camp).expect("actual wrong offset fixture");
        assert!(
            validate_connections(&wrong)
                .expect_err("misaligned native seam")
                .to_string()
                .contains("offset")
        );
    }

    #[test]
    fn boundary_basecamp_connections_native_guards_check_every_badge_signal_and_restoration() {
        for p in &PASSAGES {
            for missing in 0..8 {
                let mut state = state(p.inside[0]);
                for (i, badge) in BADGES.into_iter().enumerate() {
                    state
                        .flags
                        .set_engine_flag(badge, i != missing)
                        .expect("explicit component badge setup");
                }
                state
                    .flags
                    .set_event_flag(SIGNAL, true)
                    .expect("component upstream setup");
                state
                    .flags
                    .set_event_flag(RESTORED, true)
                    .expect("component restoration setup");
                check(&mut state, p.root, missing * 2);
                assert!(
                    matches!(branch(&mut state,p.root,missing*2+1), ScriptControlAction::Jump{target_script,..} if target_script==format!("{}Badges",p.root))
                );
                assert_eq!(state.money, 0);
            }
            let mut state = state(p.inside[0]);
            for badge in BADGES {
                state
                    .flags
                    .set_engine_flag(badge, true)
                    .expect("component badges");
            }
            check(&mut state, p.root, 16);
            assert!(
                matches!(branch(&mut state,p.root,17),ScriptControlAction::Jump{target_script,..} if target_script==format!("{}Signal",p.root))
            );
            state
                .flags
                .set_event_flag(SIGNAL, true)
                .expect("component upstream setup");
            check(&mut state, p.root, 16);
            assert!(matches!(
                branch(&mut state, p.root, 17),
                ScriptControlAction::Continue { .. }
            ));
            if p.needs_restored {
                check(&mut state, p.root, 18);
                assert!(
                    matches!(branch(&mut state,p.root,19),ScriptControlAction::Jump{target_script,..} if target_script==format!("{}Restored",p.root))
                );
                state
                    .flags
                    .set_event_flag(RESTORED, true)
                    .expect("component restored setup");
                check(&mut state, p.root, 18);
                assert!(matches!(
                    branch(&mut state, p.root, 19),
                    ScriptControlAction::Continue { .. }
                ));
            }
        }
    }

    #[test]
    fn boundary_basecamp_connections_native_facing_and_six_two_step_inbound_passages() {
        let data = &connected().data;
        assert_eq!(data.story_event_script_constants.global["VAR_FACING"], 9);
        for (token, value) in [("DOWN", 0), ("UP", 1), ("LEFT", 2), ("RIGHT", 3)] {
            assert_eq!(data.story_event_script_constants.global[token], value);
        }
        for p in &PASSAGES {
            let root = format!("{}Inward", p.root);
            for (facing, token) in [
                (Direction::Down, "DOWN"),
                (Direction::Up, "UP"),
                (Direction::Left, "LEFT"),
                (Direction::Right, "RIGHT"),
            ] {
                let mut session = data
                    .overworld_session(CAMP, native_tile(p.outside[0]).expect("outside"), 0)
                    .expect("actual native camp session");
                session.player.facing = facing;
                let mut state = state(p.outside[0]);
                data.apply_script_variable_command_now_in_session(
                    &mut state, &session, CAMP, &root, 0,
                )
                .expect("actual native VAR_FACING");
                let action = branch(&mut state, &root, 1);
                if token == outward_facing(p) {
                    assert!(
                        matches!(action, ScriptControlAction::Jump{target_script,..} if target_script==format!("{}End",p.root))
                    );
                } else {
                    assert!(matches!(action, ScriptControlAction::Continue { .. }));
                }
            }
            for (from, to) in p.outside.into_iter().zip(p.inside) {
                let mut session = data
                    .overworld_session(CAMP, native_tile(from).expect("outside"), 0)
                    .expect("actual native camp");
                let mut state = state(from);
                let flags = state.flags.clone();
                for (index, id) in p.identifiers.into_iter().enumerate() {
                    let effect = data
                        .apply_script_object_mutation(
                            &mut state,
                            &mut session,
                            CAMP,
                            CAMP,
                            &root,
                            index + 2,
                        )
                        .expect("native temporary guard removal");
                    assert_eq!(effect.object_id, id);
                    assert_eq!(
                        effect.event_flag, None,
                        "temporary visibility must never set restoration"
                    );
                    assert!(session.hidden_object_identifiers.contains(id));
                }
                let outcome = data
                    .apply_script_movement(&mut state, &mut session, CAMP, CAMP, &root, 4)
                    .expect("actual native two-step inward movement");
                assert_eq!(outcome.steps_applied, 2);
                assert_eq!(outcome.previous_tile, native_tile(from).expect("outside"));
                assert_eq!(outcome.tile, native_tile(to).expect("inside"));
                assert_eq!(session.player.mode, MovementMode::Normal);
                for (index, id) in p.identifiers.into_iter().enumerate() {
                    let effect = data
                        .apply_script_object_mutation(
                            &mut state,
                            &mut session,
                            CAMP,
                            CAMP,
                            &format!("{}Show", p.root),
                            index,
                        )
                        .expect("native guard reappearance");
                    assert_eq!(effect.object_id, id);
                    assert_eq!(effect.event_flag, None);
                    assert!(!session.hidden_object_identifiers.contains(id));
                }
                assert_eq!(state.flags, flags);
                assert_eq!(state.money, 0);
            }
        }
    }

    #[test]
    fn boundary_basecamp_connections_native_denial_moves_inward_on_actual_coord_planes() {
        let data = &connected().data;
        for p in &PASSAGES {
            for from in p.inside {
                let mut session = data
                    .overworld_session(CAMP, native_tile(from).expect("inner"), 0)
                    .expect("actual checked plane");
                let trigger = session
                    .check_coord_event_checked(None)
                    .expect("native coordinate check")
                    .expect("always-active actual inner checkpoint");
                assert_eq!(trigger.script_name, p.root);
                assert!(trigger.scene_id.is_empty());
                let mut state = state(from);
                let flags = state.flags.clone();
                let outcome = data
                    .apply_script_movement(
                        &mut state,
                        &mut session,
                        CAMP,
                        CAMP,
                        &format!("{}Badges", p.root),
                        4,
                    )
                    .expect("native one-step refusal");
                assert_eq!(outcome.steps_applied, 1);
                let expected = match p.incoming_facing {
                    "LEFT" => [from[0] - 1, from[1]],
                    "RIGHT" => [from[0] + 1, from[1]],
                    "UP" => [from[0], from[1] - 1],
                    _ => unreachable!("fixed source direction"),
                };
                assert_eq!(outcome.tile, native_tile(expected).expect("inward Floor"));
                assert_eq!(session.map.name, CAMP);
                assert_eq!(state.flags, flags);
                assert_eq!(state.money, 0);
                assert!(
                    p.identifiers
                        .iter()
                        .all(|id| !session.hidden_object_identifiers.contains(*id))
                );
            }
        }
    }

    #[test]
    fn boundary_basecamp_connections_reject_floor_damage_and_real_guard_surf_bypass() {
        let pack = base();
        let camp = &pack.data.maps[CAMP];
        validate_geometry(pack, camp).expect("actual native source geometry");
        let mut blocked = camp.clone();
        blocked.blocks[12 * 32 + 31] = 0x05;
        assert!(
            validate_geometry(pack, &blocked)
                .expect_err("blocked seam")
                .to_string()
                .contains("Floor")
        );
        let mut bypass = camp.clone();
        for x in 29..32 {
            bypass.blocks[11 * 32 + x] = 0x01;
        }
        assert!(
            validate_geometry(pack, &bypass)
                .expect_err("opened north shoulder bypass")
                .to_string()
                .contains("bypass")
        );
        let mut occupied = camp.clone();
        occupied.objects[0].x = 61;
        occupied.objects[0].y = 24;
        assert!(
            validate_geometry(pack, &occupied)
                .expect_err("checkpoint actor overlap")
                .to_string()
                .contains("actor")
        );
        assert_eq!(
            pack.data.maps[CAMP], *camp,
            "negative checks must not mutate borrowed source"
        );
    }
}
