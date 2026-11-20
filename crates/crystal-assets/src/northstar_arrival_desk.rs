//! Give southern arrivals a choice of walking directions from the desk.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_dialogue;
use crate::grand_marais_modpack::authoring::command;
use crate::northstar_rival_arc::put;
use crate::{CompiledGamePack, authored_town_publication};

const ROOM: &str = "MnMinneapolisArrivalInfo";
const HOST: &str = "MN_MP_ARRIVALINFO_HOST";
const PROMPT: &str = "MN_MP_ARRIVALINFO_HOSTText";
const HEADER: &str = "MnSouthDeskHeader";
const MENU: &str = "MnSouthDeskMenu";
const SOURCE: &str = include_str!(
    "../../../planning/minnesota/revisions/arcade-frontage-v01/arrival-directory.json"
);

#[derive(Deserialize)]
struct Destination {
    choice: String,
    label: String,
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context(ROOM)?.clone();
    ensure!(
        room.objects
            .iter()
            .any(|o| o.script == HOST && (o.x, o.y) == (2, 4))
            && room.scripts.get(HOST)
                == Some(&serde_json::json!([command(
                    "jumptextfaceplayer",
                    &[PROMPT]
                )])),
        "expected the original South Arrival Desk host"
    );
    let destinations: [Destination; 3] = serde_json::from_str(SOURCE)?;
    authored_dialogue::install_canonical(&mut room, PROMPT)?;
    put(
        &mut room,
        HEADER,
        vec![
            command("db", &["MENU_BACKUP_TILES"]),
            command("menu_coords", &["0", "0", "14", "9"]),
            command("dw", &[MENU]),
            command("db", &["1"]),
        ],
    );
    let mut options = vec![command("db", &["STATICMENU_CURSOR"]), command("db", &["4"])];
    let mut ask = vec![
        command("faceplayer", &[]),
        command("opentext", &[]),
        command("writetext", &[PROMPT]),
        command("loadmenu", &[HEADER]),
        command("verticalmenu", &[]),
        command("closewindow", &[]),
    ];
    for (index, destination) in destinations.iter().enumerate() {
        let text = format!("{}Text", destination.label);
        authored_dialogue::install_canonical(&mut room, &text)?;
        put(
            &mut room,
            &destination.label,
            vec![
                command("writetext", &[&text]),
                command("waitbutton", &[]),
                command("closetext", &[]),
                command("end", &[]),
            ],
        );
        ask.push(command(
            "ifequal",
            &[&(index + 1).to_string(), &destination.label],
        ));
        options.push(command("db", &[&format!("\"{}@\"", destination.choice)]));
    }
    options.push(command("db", &["\"CANCEL@\""]));
    put(&mut room, MENU, options);
    ask.extend([command("closetext", &[]), command("end", &[])]);
    put(&mut room, HOST, ask);

    room.script_object_commands = crate::parse_script_object_commands(ROOM, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(ROOM, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(ROOM, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(ROOM, &room.scripts)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(ROOM, &room.scripts)?;
    room.script_menu_definitions = crate::parse_script_menu_definitions(ROOM, &room.scripts)?;
    room.script_vertical_menus =
        crate::parse_script_vertical_menus(ROOM, &room.scripts, &room.script_menu_definitions)?;
    authored_town_publication::replace(pack, room)
}
