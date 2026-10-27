//! A fixed adjacent switch pair with native interactions and saved reset state.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde_json::{Value, json};

use crate::MapModule;
use crate::grand_marais_modpack::authoring::{command, text};
use crate::gym_design::Gym;
use crate::gym_scripts::requirement;

fn message(module: &mut MapModule, label: &str, lines: &[String]) {
    let body = format!("{label}Text");
    text(
        module,
        &body,
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    module.scripts.insert(
        label.into(),
        json!([
            command("writetext", &[&body]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[])
        ]),
    );
}

pub(crate) fn install(module: &mut MapModule, gym: &Gym, refresh: &str) -> Result<()> {
    let Some(puzzle) = &gym.switch_puzzle else {
        return Ok(());
    };
    let prefix = format!("{}Switch", gym.id);
    let denied = format!("{prefix}Denied");
    let first = format!("{prefix}First");
    let second = format!("{prefix}Second");
    let reset = format!("{prefix}Reset");
    let solved = format!("{prefix}Solved");
    message(module, &denied, &puzzle.denied_lines);
    message(module, &first, &puzzle.first_lines);
    message(module, &second, &puzzle.second_lines);
    message(module, &solved, &puzzle.solved_lines);
    let reset_text = format!("{reset}Text");
    text(
        module,
        &reset_text,
        &puzzle
            .reset_lines
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    // Reset only this puzzle's two switches. Trainer wins and rewards survive.
    module.scripts.insert(
        reset.clone(),
        json!([
            command("clearevent", &[&puzzle.events[0]]),
            command("clearevent", &[&puzzle.events[1]]),
            command("scall", &[refresh]),
            command("writetext", &[&reset_text]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[])
        ]),
    );
    for control in &puzzle.controls {
        let label = format!("{prefix}{}", control.id);
        let mut commands = vec![
            command("opentext", &[]),
            command("checkevent", &[&gym.court_event()]),
            command("iftrue", &[&solved]),
        ];
        for token in &puzzle.requires {
            requirement(&mut commands, token, &denied)?;
        }
        commands.push(command("checkevent", &[&puzzle.events[0]]));
        if control.id == puzzle.sequence[0] {
            commands.extend([
                command("iftrue", &[&reset]),
                command("setevent", &[&puzzle.events[0]]),
                command("scall", &[refresh]),
                command("sjump", &[&first]),
            ]);
        } else if control.id == puzzle.sequence[1] {
            commands.extend([
                command("iffalse", &[&reset]),
                command("setevent", &[&puzzle.events[1]]),
                command("scall", &[refresh]),
                command("sjump", &[&second]),
            ]);
        } else {
            commands.push(command("sjump", &[&reset]));
        }
        module.scripts.insert(label.clone(), Value::Array(commands));
        module.events.bg_events.push(BackgroundEvent {
            x: control.at[0],
            y: control.at[1],
            event_type: "BGEVENT_READ".into(),
            script: label,
        });
    }
    Ok(())
}

/// Checks fixture identities, the hinted adjacent pair and reset-safe event ownership.
pub(crate) fn validate_design(
    gym: &Gym,
    actors: &mut BTreeSet<[u16; 2]>,
    routing_events: &mut BTreeSet<String>,
) -> Result<()> {
    let [width, height] = gym.dimensions_metatiles;
    if let Some(puzzle) = &gym.switch_puzzle {
        let mut controls = BTreeMap::new();
        for control in &puzzle.controls {
            ensure!(
                !control.id.is_empty()
                    && controls.insert(&control.id, control.at).is_none()
                    && control.at[0] < width * 2
                    && control.at[1] < height * 2
                    && actors.insert(control.at),
                "{}: overlapping, unnamed or outside switch",
                gym.id
            );
        }
        let first = controls.get(&puzzle.sequence[0]);
        let second = controls.get(&puzzle.sequence[1]);
        ensure!(
            puzzle.controls.len() >= 3
                && puzzle.sequence[0] != puzzle.sequence[1]
                && first
                    .zip(second)
                    .is_some_and(|(a, b)| { a[1] == b[1] && a[0].abs_diff(b[0]) == 2 })
                && !puzzle.requires.is_empty()
                && puzzle.requires.iter().all(|token| gym
                    .trainers
                    .iter()
                    .any(|trainer| !trainer.optional
                        && token == &format!("EVENT_{}_DEFEATED", trainer.id)))
                && gym
                    .trainers
                    .iter()
                    .filter(|trainer| !trainer.optional)
                    .all(|trainer| {
                        puzzle
                            .requires
                            .contains(&format!("EVENT_{}_DEFEATED", trainer.id))
                    })
                && puzzle.events.iter().all(|event| {
                    event.starts_with(&format!("EVENT_MN_GYM_{}_", gym.slot))
                        && routing_events.insert(event.clone())
                })
                && [
                    &puzzle.denied_lines,
                    &puzzle.first_lines,
                    &puzzle.second_lines,
                    &puzzle.reset_lines,
                    &puzzle.solved_lines,
                ]
                .iter()
                .all(|lines| !lines.is_empty()),
            "{}: switches need an adjacent pair, trainer requirements and private events",
            gym.id
        );
    }
    Ok(())
}
