//! Optional daily café challenge on the current lodge lineage.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as events;
use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePack, LoadedCompiledGamePack};

const TODAY: &str = "EVENT_MN_MOTHLIGHT_CAFE_WON_TODAY";
const PENDING: &str = "EVENT_MN_MOTHLIGHT_CAFE_PRIZE_PENDING";
const TRAINER: &str = "MN_MOTHLIGHT_CAFE";

/// Adds a voluntary daily battle, retaining the lodge's artwork and services.
///
/// # Errors
/// Rejects a different parent, retired species, occupied identities, invalid
/// native dialogue, or a pack that fails runtime verification.
pub fn build_mothlight_battle_cafe(base: &LoadedCompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.pack_content_hash()?
            == "3483684ad3daf84694b26639a82aed9131bb5fbcac3c3d35ea9086490ed2888a",
        "cafe requires the selected northern trails Mothlight parent"
    );
    let mut pack = base.pack().clone();
    let roster = RegionalRoster::current()?;
    let party = [("MOTHOSAIC", 36), ("MUFFIEND", 36)];
    for (species, _) in party {
        ensure!(
            roster.allowed.contains(species),
            "inactive cafe species {species}"
        );
    }
    events::trainer(&mut pack, TRAINER, "FISHER", "COOK", &party)?;
    for flag in [TODAY, PENDING] {
        events::flag(&mut pack, flag)?;
    }
    let source: serde_json::Value =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    let prose: BTreeMap<String, Vec<AuthoredDialoguePage>> =
        serde_json::from_value(source["mothlight_battle_cafe"].clone())?;
    let mut hall = pack
        .data
        .maps
        .get("MnMothlightLodge")
        .context("installed lodge")?
        .clone();
    for (label, pages) in &prose {
        hall.scripts
            .insert(label.clone(), authored_dialogue::compile(pages)?);
    }
    hall.scripts.insert(
        "MnMothlightCook".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &[PENDING]),
            c("iftrue", &["MnCafePrize"]),
            c("checkevent", &[TODAY]),
            c("iftrue", &["MnCafeToday"]),
            c("opentext", &[]),
            c("writetext", &["MnCafeOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnCafeDeclined"]),
            c("closetext", &[]),
            c("winlosstext", &["MnCafeWonText", "MnCafeLostText"]),
            c("loadtrainer", &["FISHER", TRAINER]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("setevent", &[TODAY]),
            c("setevent", &[PENDING]),
            c("sjump", &["MnCafePrize"])
        ]),
    );
    hall.scripts.insert(
        "MnCafePrize".into(),
        json!([
            c("giveitem", &["BERRY_JUICE", "1"]),
            c("iffalse", &["MnCafeFull"]),
            c("clearevent", &[PENDING]),
            c("jumptext", &["MnCafePrizeText"])
        ]),
    );
    for suffix in ["Declined", "Today", "Full"] {
        hall.scripts.insert(
            format!("MnCafe{suffix}"),
            json!([c("jumptext", &[&format!("MnCafe{suffix}Text")])]),
        );
    }
    crate::northstar_story_revision::publish(&mut pack, hall)?;
    let mut cedar = pack
        .data
        .maps
        .get("MnMaraisCedarApproach")
        .context("lodge approach")?
        .clone();
    cedar.scripts.insert(
        "MnMothlightTrailSignText".into(),
        authored_dialogue::compile(&prose["MnCafeSignText"])?,
    );
    crate::northstar_story_revision::publish(&mut pack, cedar)?;
    ensure!(
        pack.audio_manifest == base.pack().audio_manifest,
        "retain all audio"
    );
    ensure!(
        pack.data.audio == base.pack().data.audio,
        "retain audio programs and PCM metadata"
    );
    ensure!(
        pack.runtime_files == base.pack().runtime_files,
        "retain native artwork"
    );
    for (name, original) in &base.pack().data.maps {
        let revised = &pack.data.maps[name];
        if matches!(name.as_str(), "MnMothlightLodge" | "MnMaraisCedarApproach") {
            ensure!(
                revised.blocks == original.blocks
                    && revised.objects == original.objects
                    && revised.events == original.events,
                "retain lodge geometry and actors"
            );
        } else {
            ensure!(revised == original, "unrelated map changed: {name}");
        }
    }
    pack.report
        .manifests
        .push("mothlight-battle-cafe-v1".into());
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
