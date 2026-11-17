//! Let Minnesota's public guides react to evidence already earned on the route.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Guide {
    entry: String,
    original_text: String,
    replies: Vec<Reply>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    id: String,
    after: String,
    until: Option<String>,
    pages: Vec<AuthoredDialoguePage>,
}

/// Adds read-only replies to existing public actors and signs. Existing native
/// interactions retain their facing behavior and original fallback dialogue.
pub(crate) fn install(pack: &mut CompiledGamePack, story: &Value) -> Result<()> {
    let guides: BTreeMap<String, Guide> =
        serde_json::from_value(story["regional_guidance_revision"].clone())
            .context("regional guidance source")?;
    ensure!(!guides.is_empty(), "missing regional guides");
    let mut allocated = BTreeSet::new();
    for (map, guide) in guides {
        let mut room = pack
            .data
            .maps
            .get(&map)
            .with_context(|| format!("guide map {map}"))?
            .clone();
        ensure!(
            room.objects.iter().any(|actor| actor.script == guide.entry)
                || room
                    .events
                    .bg_events
                    .iter()
                    .any(|sign| sign.script == guide.entry && sign.event_type == "BGEVENT_READ"),
            "missing visible owner {map}/{}",
            guide.entry
        );
        let old = room
            .scripts
            .get(&guide.entry)
            .and_then(Value::as_array)
            .context("original guide program")?;
        let opcode = old
            .first()
            .and_then(|c| c["command"].as_str())
            .context("guide interaction")?
            .to_owned();
        ensure!(
            matches!(opcode.as_str(), "jumptextfaceplayer" | "jumptext")
                && old.as_slice() == [command(&opcode, &[&guide.original_text])]
                && room.script_text_bodies.contains_key(&guide.original_text),
            "expected read-only guide {map}/{}",
            guide.entry
        );
        ensure!(!guide.replies.is_empty(), "empty guide {map}");
        let fallback = format!("{}Original", guide.replies[0].id);
        add(
            pack,
            &mut room,
            &mut allocated,
            &fallback,
            json!([command(&opcode, &[&guide.original_text])]),
        )?;
        // A failed condition proceeds to the next local reply. Completing a
        // handoff can therefore retire an instruction even on a later return.
        for (index, reply) in guide.replies.iter().enumerate() {
            for flag in std::iter::once(&reply.after).chain(&reply.until) {
                ensure!(
                    flag.starts_with("EVENT_MN_")
                        && pack
                            .data
                            .story_event_script_constants
                            .global
                            .contains_key(flag),
                    "unknown guide event {map}/{flag}"
                );
            }
            let next = guide
                .replies
                .get(index + 1)
                .map_or(fallback.as_str(), |r| r.id.as_str());
            let text = format!("{}Text", reply.id);
            let pages = authored_dialogue::compile(&reply.pages)
                .with_context(|| format!("{map}/{text}"))?;
            add(pack, &mut room, &mut allocated, &text, pages)?;
            let mut body = vec![
                command("checkevent", &[&reply.after]),
                command("iffalse", &[next]),
            ];
            if let Some(until) = &reply.until {
                body.extend([command("checkevent", &[until]), command("iftrue", &[next])]);
            }
            body.push(command(&opcode, &[&text]));
            add(
                pack,
                &mut room,
                &mut allocated,
                &reply.id,
                Value::Array(body),
            )?;
        }
        room.scripts.insert(
            guide.entry,
            json!([command("sjump", &[&guide.replies[0].id])]),
        );
        crate::northstar_story_revision::publish(pack, room)?;
    }
    Ok(())
}

fn add(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    allocated: &mut BTreeSet<String>,
    label: &str,
    body: Value,
) -> Result<()> {
    ensure!(
        label.starts_with("MnWay")
            && allocated.insert(label.to_owned())
            && !pack.data.map_scripts.contains_key(label)
            && !room.scripts.contains_key(label),
        "overlapping regional guidance label {label}"
    );
    room.scripts.insert(label.to_owned(), body);
    Ok(())
}
