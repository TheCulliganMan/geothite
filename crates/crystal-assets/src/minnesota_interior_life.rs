//! Room-specific conversations layered over existing Minnesota interiors.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::authored_city_expansion::merge_new_content;
use crate::authored_town_geometry::{kind, reachable};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InteriorLife {
    schema_version: u8,
    manifest: String,
    all_installed_interiors: bool,
    rooms: Vec<Room>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Room {
    map: String,
    purpose: String,
    #[serde(default)]
    texts: Vec<Text>,
    #[serde(default)]
    conversations: Vec<Conversation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Text {
    label: String,
    lines: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Conversation {
    script: String,
    actor: Option<Actor>,
    lines: Vec<String>,
    #[serde(default)]
    repeat: Vec<String>,
    remember: Option<String>,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    need: Vec<String>,
    reward: Option<String>,
    #[serde(default)]
    heal: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Actor {
    sprite: String,
    tile: [u16; 2],
    approach: [u16; 2],
}

/// Adds authored local stories, remembered visits, once-only rewards and care.
/// Existing service and quest programs cannot be replaced: only an ordinary
/// one-command conversation may be promoted. Text revisions retain their labels
/// and callers, and cannot overwrite reviewed Northstar chapter text. Gym maps,
/// terrain, warps, existing actors and audio are preserved.
/// The returned pack has a new identity and needs its own saves.
///
/// # Errors
/// Returns an error for malformed authoring, missing rooms/art/items, occupied
/// labels/flags, a nonordinary replacement, unsafe actor placement, incomplete
/// errands, duplicate publication or failed pack verification. Inputs are borrowed.
pub fn build_minnesota_interior_life_modpack(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan: InteriorLife = serde_json::from_str(source).context("interior life contract")?;
    ensure!(plan.schema_version == 1, "unsupported interior life schema");
    ensure!(
        plan.manifest.starts_with("minnesota-interior-life-")
            && !base.report.manifests.contains(&plan.manifest),
        "invalid or already installed interior life manifest"
    );
    ensure!(!plan.rooms.is_empty(), "interior contract is empty");
    if plan.all_installed_interiors {
        let expected = base
            .data
            .maps
            .values()
            .filter(|map| {
                map.id.starts_with("Mn")
                    && !map.id.contains("Gym")
                    && matches!(
                        map.attributes.environment.as_deref(),
                        Some("INDOOR" | "GATE" | "CAVE" | "DUNGEON")
                    )
            })
            .map(|map| map.id.as_str())
            .collect::<BTreeSet<_>>();
        let authored = plan
            .rooms
            .iter()
            .map(|room| room.map.as_str())
            .collect::<BTreeSet<_>>();
        ensure!(
            authored == expected,
            "interior contract must cover all installed non-gym interiors; missing {:?}, absent {:?}",
            expected.difference(&authored).collect::<Vec<_>>(),
            authored.difference(&expected).collect::<Vec<_>>()
        );
    }
    let mut pack = base.clone();
    let mut rooms = BTreeSet::new();
    let mut flags = BTreeSet::new();
    for room in &plan.rooms {
        ensure!(rooms.insert(&room.map), "duplicate room {}", room.map);
        ensure!(!room.purpose.trim().is_empty(), "room needs a purpose");
        let map = base
            .data
            .maps
            .get(&room.map)
            .context(format!("missing room {}", room.map))?;
        ensure!(
            room.map.starts_with("Mn")
                && !room.map.contains("Gym")
                && matches!(
                    map.attributes.environment.as_deref(),
                    Some("INDOOR" | "GATE" | "CAVE" | "DUNGEON")
                ),
            "{} is not a non-gym Minnesota interior",
            room.map
        );
        for conversation in &room.conversations {
            if let Some(flag) = &conversation.remember {
                ensure!(flags.insert(flag.clone()), "duplicate interior flag {flag}");
            }
            ensure!(
                conversation.reward.is_none() || conversation.remember.is_some(),
                "once-only reward needs a remembered completion"
            );
            ensure!(
                conversation.requires.is_empty() || !conversation.need.is_empty(),
                "errand needs an actionable hint"
            );
        }
    }
    crate::authored_city_expansion_events::flags(
        &mut pack,
        &flags.iter().cloned().collect::<Vec<_>>(),
    )?;
    let names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut labels = base
        .data
        .map_scripts
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    for room in &plan.rooms {
        let original = &base.data.maps[&room.map];
        let mut map = original.clone();
        let mut additions = authoring::blank(
            base,
            &room.map,
            &room.map,
            original
                .attributes
                .map_constant
                .as_deref()
                .context("room constant")?,
        )?;
        for text in &room.texts {
            ensure!(
                !crate::northstar_dialogue::is_reviewed_story_text(&room.map, &text.label),
                "local flavor cannot replace reviewed Northstar story text: {}",
                text.label
            );
            ensure!(
                map.script_text_bodies.contains_key(&text.label),
                "missing room text {}",
                text.label
            );
            checked_text(&mut additions, &text.label, &text.lines)?;
            // Text bodies have no executable control metadata. Their callers and
            // command indices stay unchanged; only the wrapped printed body moves.
            map.scripts.remove(&text.label);
            map.script_text_bodies.remove(&text.label);
        }
        for conversation in &room.conversations {
            for flag in &conversation.requires {
                ensure!(
                    flags.contains(flag),
                    "errand refers to unprovided visit {flag}"
                );
            }
            if let Some(item) = &conversation.reward {
                ensure!(
                    base.data.items.contains_key(item),
                    "unknown interior reward {item}"
                );
            }
            if let Some(actor) = &conversation.actor {
                ensure!(
                    !labels.contains(&conversation.script),
                    "occupied new actor script"
                );
                let sprite = actor
                    .sprite
                    .strip_prefix("SPRITE_")
                    .context("native sprite token")?
                    .to_ascii_lowercase();
                ensure!(
                    base.runtime_files
                        .contains_key(&format!("gfx/sprites/{sprite}.png")),
                    "missing resident art"
                );
                authoring::npc(
                    base,
                    &mut map,
                    &conversation.script,
                    &actor.sprite,
                    i16::try_from(actor.tile[0])?,
                    i16::try_from(actor.tile[1])?,
                )?;
                validate_actor(base, original, &map, actor)?;
            } else {
                ensure!(
                    original
                        .objects
                        .iter()
                        .any(|o| o.script == conversation.script),
                    "conversation has no resident"
                );
                let old = map
                    .scripts
                    .get(&conversation.script)
                    .context("ordinary resident script")?;
                ensure!(
                    old.as_array().is_some_and(
                        |body| body.len() == 1 && body[0]["command"] == "jumptextfaceplayer"
                    ),
                    "refusing to replace service/quest {}",
                    conversation.script
                );
                map.scripts.remove(&conversation.script);
                map.script_text_commands
                    .retain(|c| c.source_script != conversation.script);
            }
            conversation_scripts(&mut additions, conversation)?;
        }
        for label in additions.scripts.keys() {
            ensure!(
                !labels.contains(label)
                    || room.texts.iter().any(|t| t.label == *label)
                    || room
                        .conversations
                        .iter()
                        .any(|c| c.actor.is_none() && c.script == *label),
                "occupied interior label {label}"
            );
            labels.insert(label.clone());
        }
        crate::authored_city_expansion_events::finish(&pack, &mut additions, &names)?;
        merge_new_content(&mut map, additions)?;
        ensure!(
            map.objects
                .iter()
                .any(|o| o.sprite != "SPRITE_POKE_BALL" && o.object_type != "OBJECTTYPE_ITEMBALL"),
            "{} is still uninhabited",
            room.map
        );
        ensure!(
            map.objects.len() <= crystal_core::world::object_capacity::MAX_MAP_OBJECTS,
            "room object capacity"
        );
        authoring::refresh_event_section(&mut map);
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    pack.report.manifests.push(plan.manifest);
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

fn checked_text(map: &mut MapModule, label: &str, lines: &[String]) -> Result<()> {
    ensure!(
        !lines.is_empty() && lines.iter().all(|line| !line.trim().is_empty()),
        "empty dialogue {label}"
    );
    authoring::text(
        map,
        label,
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    Ok(())
}

fn conversation_scripts(map: &mut MapModule, conversation: &Conversation) -> Result<()> {
    let c = authoring::command;
    let id = &conversation.script;
    let text = format!("{id}InteriorText");
    checked_text(map, &text, &conversation.lines)?;
    let mut body = vec![c("faceplayer", &[]), c("opentext", &[])];
    if let Some(flag) = &conversation.remember {
        body.extend([
            c("checkevent", &[flag]),
            c("iftrue", &[&format!("{id}InteriorRepeat")]),
        ]);
        checked_text(
            map,
            &format!("{id}InteriorRepeatText"),
            &conversation.repeat,
        )?;
        map.scripts.insert(
            format!("{id}InteriorRepeat"),
            json!([
                c("writetext", &[&format!("{id}InteriorRepeatText")]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    for flag in &conversation.requires {
        body.extend([
            c("checkevent", &[flag]),
            c("iffalse", &[&format!("{id}InteriorNeed")]),
        ]);
    }
    if !conversation.requires.is_empty() {
        checked_text(map, &format!("{id}InteriorNeedText"), &conversation.need)?;
        map.scripts.insert(
            format!("{id}InteriorNeed"),
            json!([
                c("writetext", &[&format!("{id}InteriorNeedText")]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    body.extend([c("writetext", &[&text]), c("waitbutton", &[])]);
    if conversation.heal {
        body.push(c("special", &["HealParty"]));
    }
    if let Some(item) = &conversation.reward {
        body.extend([
            c("giveitem", &[item, "1"]),
            c("iffalse", &[&format!("{id}InteriorFull")]),
        ]);
        checked_text(
            map,
            &format!("{id}InteriorFullText"),
            &["Your PACK is full. Make room and visit me again; your gift will wait.".into()],
        )?;
        map.scripts.insert(
            format!("{id}InteriorFull"),
            json!([
                c("writetext", &[&format!("{id}InteriorFullText")]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    if let Some(flag) = &conversation.remember {
        body.push(c("setevent", &[flag]));
    }
    body.extend([c("closetext", &[]), c("end", &[])]);
    map.scripts.insert(id.clone(), json!(body));
    Ok(())
}

fn validate_actor(
    base: &CompiledGamePack,
    original: &MapModule,
    map: &MapModule,
    actor: &Actor,
) -> Result<()> {
    ensure!(
        kind(base, map, actor.tile)? == "FLOOR",
        "resident needs native floor"
    );
    ensure!(
        actor.tile[0].abs_diff(actor.approach[0]) + actor.tile[1].abs_diff(actor.approach[1]) == 1,
        "resident approach must be adjacent"
    );
    ensure!(
        map.objects
            .iter()
            .filter(|o| [o.x, o.y] == actor.tile)
            .count()
            == 1,
        "overlapping residents"
    );
    ensure!(
        !map.events.warps.iter().any(|w| [w.x, w.y] == actor.tile)
            && !map
                .events
                .coord_events
                .iter()
                .any(|e| [e.x, e.y] == actor.tile),
        "resident obstructs an event"
    );
    for warp in &original.events.warps {
        let start = [warp.x, warp.y];
        if original.objects.iter().any(|o| [o.x, o.y] == start) {
            continue;
        }
        let before = reachable(base, original, start, false)?;
        if !before.contains(&actor.approach) {
            continue;
        }
        let after = reachable(base, map, start, false)?;
        ensure!(
            after.contains(&actor.approach),
            "resident cannot be approached"
        );
        ensure!(
            before
                .difference(&after)
                .all(|tile| map.objects.iter().any(|o| [o.x, o.y] == *tile)),
            "resident blocks room traversal"
        );
        return Ok(());
    }
    anyhow::bail!("resident approach has no connected room arrival")
}
