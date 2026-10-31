//! Lower the bounded five-stop contract into existing native script tables.
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapScriptSectionCommand;
use serde_json::Value;

use crate::glacier_line_service::{SERVICE, Station};
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_rival_arc::put;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    stations: &[Station],
    origin: usize,
) -> Result<()> {
    let stop = &stations[origin];
    let prefix = stop.prefix();
    let original = room
        .scripts
        .get(&stop.conductor_script)
        .context("conductor speech")?
        .as_array()
        .context("native conductor program")?
        .clone();
    put(room, &format!("{prefix}Original"), original);
    let choices: Vec<_> = stations
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != origin)
        .collect();
    let denied = format!("{prefix}Denied");
    let unknown = format!("{prefix}Unknown");
    let cancel = format!("{prefix}Cancel");
    let tree = format!("{prefix}Choose0_0");
    put(
        room,
        &stop.conductor_script,
        vec![
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("checkevent", &[SERVICE]),
            c("iffalse", &[&denied]),
            c("writetext", &[&format!("{prefix}WelcomeText")]),
            c("waitbutton", &[]),
            c("sjump", &[&tree]),
        ],
    );
    authoring::text(
        room,
        &format!("{prefix}WelcomeText"),
        &[
            "GLACIER LINE.",
            "Return trips free.",
            "X marks a stop you",
            "haven't visited.",
            "Visit its hall on",
            "foot to unlock it.",
            "Choose a stop.",
            "Or CANCEL to stay.",
        ],
    );
    for (label, text) in [
        (
            &denied,
            "The courier's survey case has stopped passenger service. Help settle it at ST PAUL UNION DEPOT.",
        ),
        (
            &unknown,
            "We can take you back after you've walked to that station. No ticket issued today.",
        ),
        (&cancel, "Another day, then. Safe travels on the trail!"),
        (
            &format!("{prefix}NoStops"),
            "You haven't visited another station yet. Follow the line on foot, then ride back whenever you like.",
        ),
    ] {
        let text_label = format!("{label}Text");
        authoring::text(room, &text_label, &[text]);
        put(
            room,
            label,
            vec![
                c("writetext", &[&text_label]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[]),
            ],
        );
    }
    for (depth, (_, choice)) in choices.iter().enumerate() {
        for bits in 0..(1 << depth) {
            let label = format!("{prefix}Choose{depth}_{bits}");
            let child = |next| {
                if depth == 3 {
                    format!("{prefix}Menu{next}")
                } else {
                    format!("{prefix}Choose{}_{}", depth + 1, next)
                }
            };
            put(
                room,
                &label,
                vec![
                    c("checkevent", &[&choice.visited_flag]),
                    c("iftrue", &[&child(bits | (1 << depth))]),
                    c("sjump", &[&child(bits)]),
                ],
            );
        }
    }
    for bits in 0..16 {
        let label = format!("{prefix}Menu{bits}");
        let header = format!("{label}Header");
        let data = format!("{label}Data");
        let mut rows = if bits == 0 {
            vec![c("sjump", &[&format!("{prefix}NoStops")])]
        } else {
            vec![
                c("loadmenu", &[&header]),
                c("verticalmenu", &[]),
                c("closewindow", &[]),
            ]
        };
        for (row, (destination, _)) in choices.iter().enumerate() {
            rows.push(c(
                "ifequal",
                &[
                    &(row + 1).to_string(),
                    &format!("{prefix}Ride{destination}"),
                ],
            ));
        }
        rows.push(c("sjump", &[&cancel]));
        put(room, &label, rows);
        put(
            room,
            &header,
            vec![
                c("db", &["MENU_BACKUP_TILES"]),
                c("menu_coords", &["0", "0", "19", "12"]),
                c("dw", &[&data]),
                c("db", &["1"]),
            ],
        );
        let mut entries = vec![c("db", &["STATICMENU_CURSOR"]), c("db", &["5"])];
        for (row, (_, destination)) in choices.iter().enumerate() {
            let suffix = if bits & (1 << row) == 0 { " X" } else { "" };
            entries.push(c(
                "db",
                &[&format!("\"{}{suffix}@\"", destination.menu_label)],
            ));
        }
        entries.push(c("db", &["\"CANCEL@\""]));
        put(room, &data, entries);
    }
    let conductor = room
        .objects
        .iter()
        .find(|o| o.script == stop.conductor_script)
        .and_then(|o| o.object_identifier.as_deref())
        .context("named native conductor")?
        .to_owned();
    for (suffix, steps) in [
        ("ConductorOpen", &stop.conductor_open_steps),
        ("PlayerBoard", &stop.player_board_steps),
        ("ConductorReset", &stop.conductor_restore_steps),
    ] {
        let mut rows: Vec<_> = steps
            .iter()
            .map(|direction| c("step", &[direction]))
            .collect();
        rows.push(c("step_end", &[]));
        put(room, &format!("{prefix}{suffix}"), rows);
    }
    put(
        room,
        &format!("{prefix}Arrive"),
        vec![c("turn_head", &["DOWN"]), c("step_end", &[])],
    );
    for (index, destination) in choices {
        let text = format!("{prefix}Board{index}Text");
        authoring::text(
            room,
            &text,
            &[&format!(
                "Next stop, {}. All aboard!",
                destination.menu_label
            )],
        );
        let [x, y] = destination.ride_arrival_core;
        put(
            room,
            &format!("{prefix}Ride{index}"),
            vec![
                c("checkevent", &[SERVICE]),
                c("iffalse", &[&denied]),
                c("checkevent", &[&destination.visited_flag]),
                c("iffalse", &[&unknown]),
                c("writetext", &[&text]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c(
                    "applymovement",
                    &[&conductor, &format!("{prefix}ConductorOpen")],
                ),
                c(
                    "applymovement",
                    &["PLAYER", &format!("{prefix}PlayerBoard")],
                ),
                c(
                    "applymovement",
                    &[&conductor, &format!("{prefix}ConductorReset")],
                ),
                c("setval", &[if index < origin { "TRUE" } else { "FALSE" }]),
                c("special", &["MagnetTrain"]),
                c(
                    "warpfacing",
                    &[
                        &destination.constant,
                        &x.to_string(),
                        &y.to_string(),
                        "DOWN",
                    ],
                ),
                c("newloadmap", &["MAPSETUP_TRAIN"]),
                c("applymovement", &["PLAYER", &format!("{prefix}Arrive")]),
                c("wait", &["20"]),
                c("end", &[]),
            ],
        );
    }
    let callbacks: Vec<_> = room
        .map_script_section_commands
        .iter()
        .filter(|c| {
            c.command == "callback" && c.args.first().is_some_and(|a| a == "MAPCALLBACK_NEWMAP")
        })
        .collect();
    if [1, 4].contains(&origin) {
        ensure!(
            callbacks.len() == 1,
            "existing first-visit callback required"
        );
        let label = callbacks[0].args.get(1).context("visit callback body")?;
        ensure!(
            room.scripts.get(label)
                == Some(&Value::Array(vec![
                    c("setevent", &[&stop.visited_flag]),
                    c("endcallback", &[])
                ])),
            "existing callback must set only its own station visit"
        );
    } else {
        ensure!(callbacks.is_empty(), "unexpected station NEWMAP callback");
        let label = format!("{prefix}FirstVisit");
        put(
            room,
            &label,
            vec![c("setevent", &[&stop.visited_flag]), c("endcallback", &[])],
        );
        room.map_script_section_commands
            .push(MapScriptSectionCommand {
                command: "callback".into(),
                args: vec!["MAPCALLBACK_NEWMAP".into(), label],
                command_index: room.map_script_section_commands.len(),
            });
    }
    crate::ely_league_rooms::finish(pack, room)?;
    room.script_audio_commands = crate::parse_script_audio_commands(&room.id, &room.scripts)?;
    room.script_menu_definitions = crate::parse_script_menu_definitions(&room.id, &room.scripts)?;
    room.script_vertical_menus =
        crate::parse_script_vertical_menus(&room.id, &room.scripts, &room.script_menu_definitions)?;
    ensure!(
        room.script_vertical_menus.len() == 15,
        "all usable visit masks need native menus"
    );
    Ok(())
}
