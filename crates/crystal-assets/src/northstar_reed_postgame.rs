//! Reed's optional Champion challenge, retaining his existing native identity.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "northstar-reed-champion-challenge-v1";
const TRAINER: &str = "MN_REED_REFINEMENT";
const WON: &str = "EVENT_MN_REED_CHAMPION_DEFEATED";
const ROOT: &str = "MN_ARC_AFTERMATH_REED";

/// Adds a voluntary battle after the League; loss and refusal remain retryable.
///
/// The existing actor, restoration visibility, pre-Champion speech, and all
/// earlier rival battles remain intact. The victory awards ordinary battle
/// money only, with no item claim or repeat prize to duplicate.
pub(crate) fn install(
    pack: &mut CompiledGamePack,
    story_source: &str,
    pokemon_source: &str,
) -> Result<()> {
    ensure!(
        !pack.report.manifests.iter().any(|name| name == MANIFEST),
        "Reed Champion challenge already installed"
    );
    let source: Value = serde_json::from_str(story_source)?;
    let dialogue: BTreeMap<String, Vec<AuthoredDialoguePage>> =
        serde_json::from_value(source["refinement_reed_challenge"]["dialogue"].clone())?;
    let mut station = pack
        .data
        .maps
        .get("MnStCloudStation")
        .context("Reed's existing station")?
        .clone();
    ensure!(
        station.scripts.get(ROOT) == Some(&json!([c("jumptext", &["MnArcAftermathText"])])),
        "changed Reed aftermath program"
    );
    let actor = station
        .objects
        .iter()
        .find(|object| object.object_identifier.as_deref() == Some(ROOT))
        .context("existing Reed actor")?;
    ensure!(
        actor.sprite == "SPRITE_MN_RIVAL_REED" && actor.script == ROOT,
        "Reed must retain his accepted native actor"
    );
    let original_objects = station.objects.clone();
    let original_blocks = station.blocks.clone();
    let original_events = station.events.clone();
    let class = crate::northstar_refinement_battles::register_reed_challenge(pack, pokemon_source)?;
    ensure!(
        class == "MN_RIVAL_REED",
        "Reed's accepted native portrait required"
    );
    crate::northstar_lair_authoring::flag(pack, WON)?;
    for kind in ["Offer", "Won", "Lost", "After", "Declined"] {
        let label = format!("MnRefineReed{kind}Text");
        let pages = dialogue
            .get(&label)
            .context("canonical Reed challenge text")?;
        ensure!(
            !station.scripts.contains_key(&label),
            "occupied Reed text {label}"
        );
        station
            .scripts
            .insert(label, authored_dialogue::compile(pages)?);
    }
    put(
        &mut station,
        ROOT,
        vec![
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_MN_LEAGUE_CHAMPION"]),
            c("iffalse", &["MnRefineReedBeforeChampion"]),
            c("checkevent", &[WON]),
            c("iftrue", &["MnRefineReedAfter"]),
            c("opentext", &[]),
            c("writetext", &["MnRefineReedOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnRefineReedDeclined"]),
            c("closetext", &[]),
            c(
                "winlosstext",
                &["MnRefineReedWonText", "MnRefineReedLostText"],
            ),
            c("loadtrainer", &[&class, TRAINER]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("setevent", &[WON]),
            c("sjump", &["MnRefineReedAfter"]),
        ],
    );
    put(
        &mut station,
        "MnRefineReedBeforeChampion",
        vec![c("jumptext", &["MnArcAftermathText"])],
    );
    put(
        &mut station,
        "MnRefineReedAfter",
        vec![c("jumptext", &["MnRefineReedAfterText"])],
    );
    put(
        &mut station,
        "MnRefineReedDeclined",
        vec![
            c("writetext", &["MnRefineReedDeclinedText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    ensure!(
        station.objects == original_objects
            && station.blocks == original_blocks
            && station.events == original_events,
        "Reed challenge must preserve station geometry and actor identities"
    );
    crate::northstar_story_revision::publish(pack, station)?;
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}

fn put(map: &mut MapModule, label: &str, rows: Vec<Value>) {
    map.scripts.insert(label.into(), Value::Array(rows));
}
