//! Native saved block controls and trainer encounters on retained Gym scripts.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapScriptSectionCommand;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::command as c;
use crate::gym_design::Gym;
use crate::northstar_gym_design::RoomContract;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    room: &mut MapModule,
    gym: &Gym,
    contract: &RoomContract,
    symbols: &BTreeMap<char, u16>,
) -> Result<()> {
    for event in contract
        .court
        .all
        .iter()
        .chain(&contract.court.any)
        .chain(
            gym.routers
                .iter()
                .flat_map(|router| router.modes.iter().map(|mode| &mode.event)),
        )
        .chain(gym.gates.iter().map(|gate| &gate.event))
    {
        crate::gym_scripts::event(pack, event)?;
    }
    // The redesigned rooms own these spatial hooks. Reward, rematch, guide and
    // native cast scripts remain in place and keep their original event IDs.
    room.events.bg_events.clear();
    room.events.coord_events.clear();
    // Old private metatile indices have different meanings in the new recipe.
    // Keep legacy labels resolvable, but retire their former terrain writes.
    // The new callback below owns every current block reconstruction.
    for program in room.scripts.values_mut() {
        if program.as_array().is_some_and(|commands| {
            commands
                .iter()
                .any(|command| command["command"] == "changeblock")
        }) {
            *program = json!([c("end", &[])]);
        }
    }
    let refresh = format!("{}RefreshGates", gym.id);
    let court = format!("{}V2Qualify", gym.id);
    let accepted = format!("{court}Accepted");
    let denied = format!("{court}Denied");
    let mut body = vec![
        c("checkevent", &[&gym.defeated()]),
        c("iftrue", &[&accepted]),
    ];
    for event in &contract.court.all {
        crate::gym_scripts::requirement(&mut body, event, &denied)?;
    }
    for event in &contract.court.any {
        body.extend([c("checkevent", &[event]), c("iftrue", &[&accepted])]);
    }
    body.push(c(
        "sjump",
        &[if contract.court.any.is_empty() {
            &accepted
        } else {
            &denied
        }],
    ));
    room.scripts.insert(court.clone(), Value::Array(body));
    room.scripts.insert(
        accepted,
        json!([c("setevent", &[&gym.court_event()]), c("end", &[])]),
    );
    room.scripts.insert(
        denied,
        json!([c("clearevent", &[&gym.court_event()]), c("end", &[])]),
    );
    let mut refresh_body = vec![c("scall", &[&court])];
    crate::gym_cut::install_scripts(room, gym, symbols, &refresh, &mut refresh_body);
    for (index, gate) in gym.gates.iter().enumerate() {
        ensure!(
            gate.event != gym.court_event() || gate.requires == [gym.court_event()],
            "court geometry must consume its qualification rather than redefine it"
        );
        let label = format!("{}V2Gate{index}", gym.id);
        let closed = format!("{label}Closed");
        let opened = format!("{label}Open");
        let closed_symbol = gym
            .rows
            .get(usize::from(gate.at[1]))
            .and_then(|row| row.chars().nth(usize::from(gate.at[0])))
            .context("gate in authored geometry")?;
        let mut body = Vec::new();
        if contract.victory_opens_gates {
            body.extend([c("checkevent", &[&gym.defeated()]), c("iftrue", &[&opened])]);
        }
        for requirement in &gate.requires {
            crate::gym_scripts::requirement(&mut body, requirement, &closed)?;
        }
        body.push(c("sjump", &[&opened]));
        room.scripts.insert(label.clone(), Value::Array(body));
        let mut body = vec![block(gate.at, symbols[&gate.open_symbol])];
        if gate.event != gym.court_event() {
            body.push(c("setevent", &[&gate.event]));
        }
        body.push(c("end", &[]));
        room.scripts.insert(opened, Value::Array(body));
        let mut reset = vec![block(gate.at, symbols[&closed_symbol])];
        if gate.event != gym.court_event() {
            reset.push(c("clearevent", &[&gate.event]));
        }
        reset.push(c("end", &[]));
        room.scripts.insert(closed, Value::Array(reset));
        refresh_body.push(c("scall", &[&label]));
    }
    refresh_body.push(c("end", &[]));
    room.scripts
        .insert(refresh.clone(), Value::Array(refresh_body));
    room.map_script_section_commands.retain(|row| {
        !(row.command == "callback"
            && row
                .args
                .first()
                .is_some_and(|arg| arg == "MAPCALLBACK_TILES"))
    });
    room.map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec!["MAPCALLBACK_TILES".into(), refresh.clone()],
            command_index: room.map_script_section_commands.len(),
        });
    for (index, row) in room.map_script_section_commands.iter_mut().enumerate() {
        row.command_index = index;
    }
    crate::gym_routing::install(room, gym, &refresh)?;
    if contract.victory_opens_gates {
        let done = format!("{}V2Complete", gym.id);
        room.scripts.insert(
            done.clone(),
            json!([c("jumptext", &[&format!("{done}Text")])]),
        );
        // Canonical prose is installed after mechanics. Keep the label present
        // so that the authoring pass can require its reviewed replacement.
        room.scripts.insert(format!("{done}Text"), json!([]));
        for router in &gym.routers {
            let root = format!("{}{}Routing", gym.id, router.id);
            let body = room
                .scripts
                .get_mut(&root)
                .and_then(Value::as_array_mut)
                .context("physical control program")?;
            body.splice(
                0..0,
                [c("checkevent", &[&gym.defeated()]), c("iftrue", &[&done])],
            );
        }
    }
    if gym.slot == 1 {
        let label = format!("{}Leader", gym.id);
        let body = room
            .scripts
            .get_mut(&label)
            .and_then(Value::as_array_mut)
            .context("Rochester leader program")?;
        if body.get(5) != Some(&c("checkevent", &[&gym.court_event()])) {
            ensure!(
                body.get(5..9)
                    == Some(&[
                        c("checkevent", &["EVENT_MN_GYM_1_TRAINER_1_DEFEATED"]),
                        c("iffalse", &["MnRochesterGymBeatTavel"]),
                        c("checkevent", &["EVENT_MN_GYM_1_TRAINER_2_DEFEATED"]),
                        c("iffalse", &["MnRochesterGymBeatElvik"]),
                    ]),
                "Rochester qualification preimage"
            );
            body.splice(
                5..9,
                [
                    c("checkevent", &[&gym.court_event()]),
                    c("iffalse", &["MnRochesterGymBeatTavel"]),
                ],
            );
        }
    }
    if !gym.court_denied_lines.is_empty() {
        let denied = format!("{}CourtRequirements", gym.id);
        let text = format!("{denied}Text");
        room.scripts
            .insert(denied.clone(), json!([c("jumptextfaceplayer", &[&text])]));
        room.scripts.entry(text).or_insert_with(|| json!([]));
        let leader = format!("{}Leader", gym.id);
        let body = room
            .scripts
            .get_mut(&leader)
            .and_then(Value::as_array_mut)
            .context("retained leader entry")?;
        let index = body
            .iter()
            .position(|command| *command == c("checkevent", &[&gym.court_event()]))
            .context("leader court qualification")?;
        ensure!(
            body.get(index + 1)
                .is_some_and(|command| command["command"] == "iffalse"),
            "leader qualification branch"
        );
        body[index + 1] = c("iffalse", &[&denied]);
    }
    install_trainers(pack, room, gym, contract, &refresh)
}

fn block([x, y]: [u16; 2], symbol: u16) -> Value {
    c(
        "changeblock",
        &[
            &(x * 2).to_string(),
            &(y * 2).to_string(),
            &symbol.to_string(),
        ],
    )
}

fn install_trainers(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    gym: &Gym,
    contract: &RoomContract,
    refresh: &str,
) -> Result<()> {
    for trainer in &gym.trainers {
        let id = &trainer.id;
        let class = &pack
            .data
            .trainers
            .get(id)
            .context("retained gym trainer")?
            .trainer_class;
        let after = format!("{id}After");
        let after_text = format!("{after}Text");
        let seen = format!("{id}Seen");
        let won = format!("{id}WonText");
        let lost = format!("{id}LostText");
        ensure!(
            [&seen, &won, &lost, &after_text]
                .iter()
                .all(|label| room.scripts.contains_key(*label)),
            "retain native trainer dialogue labels: {id}"
        );
        room.scripts.insert(
            id.clone(),
            json!([c(
                "trainer",
                &[
                    class,
                    id,
                    &format!("EVENT_{id}_DEFEATED"),
                    &seen,
                    &won,
                    &lost,
                    &after,
                ]
            )]),
        );
        let next = gym
            .routers
            .iter()
            .find(|router| router.operator.as_deref() == Some(id))
            .map(|router| format!("{}{}Routing", gym.id, router.id));
        let mut body = vec![c("scall", &[refresh])];
        body.push(if let Some(next) = next {
            c("sjump", &[&next])
        } else {
            c("jumptext", &[&after_text])
        });
        room.scripts.insert(after, Value::Array(body));
        let sight = &contract.sightlines[id];
        let actor = room
            .objects
            .iter_mut()
            .find(|actor| actor.object_identifier.as_deref() == Some(id))
            .context("retained trainer actor")?;
        actor.object_type = "OBJECTTYPE_TRAINER".into();
        actor.radius = sight.radius;
        let direction = sight.direction.to_ascii_uppercase();
        actor.spritemovedata = format!("SPRITEMOVEDATA_STANDING_{direction}");
        actor.sightline_direction_override = Some(direction);
        actor.move_range_x = 0;
        actor.move_range_y = 0;
    }
    Ok(())
}
