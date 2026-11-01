//! Shared native guard scripts; each caller validates its authored map geometry.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::CoordEvent;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

/// Borrowed authored checkpoint. No permission or reward state is invented.
pub(crate) struct Checkpoint<'a> {
    pub identifiers: &'a [String; 2],
    pub bodies: [[u16; 2]; 2],
    pub inside: [[u16; 2]; 2],
    pub outside: [[u16; 2]; 2],
    pub root: &'a str,
    pub incoming_facing: &'a str,
    pub outward: &'a str,
    pub required_event: &'a str,
    pub badge_flags: &'a [&'a str],
    pub badge_lines: &'a [String],
    pub evidence_lines: &'a [String],
}

fn checks(p: &Checkpoint<'_>, badge: &str, evidence: &str) -> Vec<Value> {
    let mut commands = Vec::new();
    for flag in p.badge_flags {
        commands.extend([
            authoring::command("checkflag", &[flag]),
            authoring::command("iffalse", &[badge]),
        ]);
    }
    if !p.required_event.is_empty() {
        commands.extend([
            authoring::command("checkevent", &[p.required_event]),
            authoring::command("iffalse", &[evidence]),
        ]);
    }
    commands
}
fn visibility(p: &Checkpoint<'_>, command: &str) -> Vec<Value> {
    p.identifiers
        .iter()
        .map(|id| authoring::command(command, &[id]))
        .collect()
}
fn outward_facing(p: &Checkpoint<'_>) -> &'static str {
    match p.outward {
        "west" => "LEFT",
        "east" => "RIGHT",
        "north" => "UP",
        "south" => "DOWN",
        _ => unreachable!("validated checkpoint direction"),
    }
}

pub(crate) fn install(
    pack: &CompiledGamePack,
    town: &mut MapModule,
    passage: &Checkpoint<'_>,
) -> Result<()> {
    let expected_inward = match passage.outward {
        "west" => "RIGHT",
        "east" => "LEFT",
        "north" => "DOWN",
        "south" => "UP",
        _ => anyhow::bail!("unknown checkpoint direction"),
    };
    ensure!(
        passage.incoming_facing == expected_inward
            && (!passage.badge_flags.is_empty() || !passage.required_event.is_empty()),
        "checkpoint direction/prerequisite contract"
    );
    let root = passage.root;
    let badge = format!("{root}Badges");
    let evidence = format!("{root}Evidence");
    let inward = format!("{root}Inward");
    let show = format!("{root}Show");
    let reconcile = format!("{root}Reconcile");
    let done = format!("{root}End");
    let two_steps = format!("{root}TwoSteps");
    let one_step = format!("{root}OneStep");
    let mut scripts = BTreeMap::new();
    let mut checked = checks(passage, &badge, &evidence);
    checked.extend(visibility(passage, "disappear"));
    checked.push(authoring::command("end", &[]));
    scripts.insert(root.to_string(), Value::Array(checked));
    for (label, lines) in [
        (
            &badge,
            passage
                .badge_lines
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ),
        (
            &evidence,
            passage
                .evidence_lines
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ),
    ] {
        if label == &badge && passage.badge_flags.is_empty() {
            continue;
        }
        if label == &evidence && passage.required_event.is_empty() {
            continue;
        }
        let text = format!("{label}Text");
        ensure!(
            lines.iter().all(|line| line.chars().count() <= 18),
            "checkpoint text width"
        );
        ensure!(
            !town.scripts.contains_key(&text) && !pack.data.map_scripts.contains_key(&text),
            "checkpoint text collision"
        );
        authoring::text(town, &text, &lines);
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
    let mut qualify = checks(passage, &show, &show);
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
            !town.scripts.contains_key(&label) && !pack.data.map_scripts.contains_key(&label),
            "checkpoint script collision {label}"
        );
        town.scripts.insert(label, commands);
    }
    for ((id, [x, y]), (inner, outer)) in passage
        .identifiers
        .iter()
        .map(String::as_str)
        .zip(passage.bodies)
        .zip(passage.inside.into_iter().zip(passage.outside))
    {
        ensure!(
            !town
                .objects
                .iter()
                .any(|actor| actor.object_identifier.as_deref() == Some(id)),
            "checkpoint object collision {id}"
        );
        authoring::npc(
            pack,
            town,
            root,
            "SPRITE_OFFICER",
            i16::try_from(x)?,
            i16::try_from(y)?,
        )?;
        let actor = town.objects.last_mut().context("native Officer inserted")?;
        actor.object_identifier = Some(id.into());
        actor.spritemovedata = format!("SPRITEMOVEDATA_STANDING_{}", passage.incoming_facing);
        actor.object_type = "OBJECTTYPE_SCRIPT".into();
        actor.radius = 0;
        actor.event_flag = "-1".into();
        town.events.coord_events.extend([
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
