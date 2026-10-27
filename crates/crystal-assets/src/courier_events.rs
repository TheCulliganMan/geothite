//! Battle-only defeat and a separate shipping-record interaction.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_town_geometry::{kind, reachable};
use crate::courier_story::{Chapter, DEPOT, READ, TRAINER, WIN};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

fn milestone<'a>(story: &'a Value, id: &str) -> Result<&'a Value> {
    let found = story["milestones"]
        .as_array()
        .context("story milestones")?
        .iter()
        .filter(|m| m["id"] == id)
        .collect::<Vec<_>>();
    ensure!(found.len() == 1, "one canonical {id} required");
    Ok(found[0])
}

pub(crate) fn install(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    chapter: &Chapter,
    story: &Value,
) -> Result<()> {
    let actor = &chapter.courier;
    let record = &chapter.record;
    ensure!(
        map.id == DEPOT
            && [map.attributes.width, map.attributes.height] == [16, 12]
            && map.attributes.tileset_name == "train_station"
            && actor.map == DEPOT
            && actor.id == "MN_ST_PAUL_DEPOT_COURIER"
            && actor.tile == [20, 16]
            && actor.approach == [19, 16]
            && record.id == "MN_SP_DEEP_BORE_MANIFEST"
            && record.tile == [18, 16]
            && record.approach == [19, 16],
        "actual depot courier and separate record anchors required"
    );
    let fight = milestone(story, "MS04")?;
    let read = milestone(story, "MS04_MANIFEST")?;
    ensure!(
        fight["location"] == DEPOT
            && fight["requires"] == json!(["BADGE_3", "EVENT_MN_DISPATCH_NOTE"])
            && fight["sets"] == json!([WIN])
            && read["location"] == DEPOT
            && read["requires"] == json!([WIN])
            && read["sets"] == json!([READ]),
        "courier victory and acknowledged manifest must remain separate"
    );
    let matches = map
        .objects
        .iter()
        .filter(|o| o.object_identifier.as_deref() == Some(actor.id.as_str()))
        .collect::<Vec<_>>();
    ensure!(
        matches.len() == 1
            && [matches[0].x, matches[0].y] == actor.tile
            && matches[0].sprite == "SPRITE_ROCKET"
            && matches[0].script == actor.id,
        "existing depot courier differs"
    );
    ensure!(
        map.scripts.get(&actor.id)
            == Some(&json!([authoring::command(
                "jumptextfaceplayer",
                &["MN_ST_PAUL_DEPOT_COURIERText"]
            )])),
        "depot courier must still be the original dialogue-only placeholder"
    );
    ensure!(
        kind(pack, map, record.tile)? == "FLOOR"
            && kind(pack, map, record.approach)? == "FLOOR"
            && !map.objects.iter().any(|o| [o.x, o.y] == record.tile),
        "record anchor occupied or blocked"
    );
    for ms in [fight, read] {
        for (label, value) in ms["native_texts"]
            .as_object()
            .context("native courier text")?
        {
            let lines: Vec<String> = serde_json::from_value(value.clone())?;
            ensure!(
                !map.scripts.contains_key(label)
                    && !pack.data.map_scripts.contains_key(label)
                    && !lines.is_empty()
                    && lines.iter().all(|s| !s.is_empty()
                        && s.len() <= 18
                        && s.is_ascii()
                        && !s.contains(['\n', '\r', '"'])),
                "courier native text width/identity {label}"
            );
            authoring::text(
                map,
                label,
                &lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
        }
    }
    for (ms, label) in [(fight, "MnSPCourierSeenText"), (read, "MnSPManifestText")] {
        let pages: Vec<Vec<String>> = serde_json::from_value(ms["dialogue_pages"].clone())?;
        ensure!(
            pages.len() == 3
                && pages.iter().all(|p| !p.is_empty()
                    && p.len() <= 2
                    && p.iter().all(|s| !s.is_empty()
                        && s.len() <= 18
                        && s.is_ascii()
                        && !s.contains(['\n', '\r', '"']))),
            "three native one/two-line courier/record pages required"
        );
        ensure!(
            !map.scripts.contains_key(label) && !pack.data.map_scripts.contains_key(label),
            "courier page label already occupied"
        );
        let mut commands = Vec::new();
        for (page_index, page) in pages.iter().enumerate() {
            for (line_index, line) in page.iter().enumerate() {
                let command = if line_index > 0 {
                    "line"
                } else if page_index == 0 {
                    "text"
                } else {
                    "para"
                };
                commands.push(authoring::command(command, &[&format!("\"{line}\"")]));
            }
        }
        commands.push(authoring::command("done", &[]));
        map.scripts.insert(label.into(), Value::Array(commands));
    }
    let c = authoring::command;
    map.scripts.insert(
        actor.id.clone(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &[WIN]),
            c("iftrue", &["MnSPCourierAfter"]),
            c("checkflag", &["ENGINE_PLAINBADGE"]),
            c("iffalse", &["MnSPCourierBadge"]),
            c("checkevent", &["EVENT_MN_DISPATCH_NOTE"]),
            c("iffalse", &["MnSPCourierNote"]),
            c("opentext", &[]),
            c("writetext", &["MnSPCourierSeenText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c(
                "winlosstext",
                &["MnSPCourierWonText", "MnSPCourierLostText"]
            ),
            c("loadtrainer", &["GRUNTM", TRAINER]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("setevent", &[WIN]),
            c("sjump", &["MnSPCourierAfter"])
        ]),
    );
    map.scripts.insert(
        record.id.clone(),
        json!([
            c("checkevent", &[READ]),
            c("iftrue", &["MnSPManifestAfter"]),
            c("checkevent", &[WIN]),
            c("iffalse", &["MnSPManifestLocked"]),
            c("opentext", &[]),
            c("writetext", &["MnSPManifestText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("setevent", &[READ]),
            c("end", &[])
        ]),
    );
    // Retain the original property claim as a reachable part of the hostile
    // post-win response, so cached and uncached depot scripts agree.
    map.scripts.insert(
        "MnSPCourierAfter".into(),
        json!([
            c("opentext", &[]),
            c("writetext", &["MN_ST_PAUL_DEPOT_COURIERText"]),
            c("waitbutton", &[]),
            c("writetext", &["MnSPCourierAfterText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    for (label, text) in [
        ("MnSPCourierBadge", "MnSPCourierBadgeText"),
        ("MnSPCourierNote", "MnSPCourierNoteText"),
        ("MnSPManifestAfter", "MnSPManifestAfterText"),
        ("MnSPManifestLocked", "MnSPManifestLockedText"),
    ] {
        ensure!(
            !map.scripts.contains_key(label) && !pack.data.map_scripts.contains_key(label),
            "courier script collision"
        );
        map.scripts
            .insert(label.into(), json!([c("jumptext", &[text])]));
    }
    authoring::npc(pack, map, &record.id, "SPRITE_PAPER", 18, 16)?;
    let paper = map.objects.last_mut().context("manifest object")?;
    paper.object_identifier = Some(record.id.clone());
    paper.spritemovedata = "SPRITEMOVEDATA_STILL".into();
    ensure!(
        reachable(pack, map, [15, 21], false)?.contains(&actor.approach),
        "courier/manifest approach blocked"
    );
    Ok(())
}
