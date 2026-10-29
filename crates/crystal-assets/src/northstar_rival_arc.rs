//! A recurring rival, witnessed consequences and a physical legendary release.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as a;
use crate::{CompiledGamePack, MapModule};

pub(crate) const MANIFEST: &str = "northstar-rival-arc-v3";
pub(crate) const EARLY: &str = "EVENT_MN_REED_ROCHESTER_MET";
pub(crate) const EARLY_WON: &str = "EVENT_MN_REED_ROCHESTER_DEFEATED";
pub(crate) const DEPOT: &str = "EVENT_MN_REED_STPAUL_MET";
pub(crate) const DEPOT_WON: &str = "EVENT_MN_REED_STPAUL_DEFEATED";
pub(crate) const WORKER: &str = "EVENT_MN_REED_WORKER_RELEASED";
pub(crate) const LINK: &str = "EVENT_MN_ARC_LINK_SENT";
pub(crate) const FINAL_WON: &str = "EVENT_MN_REED_BOUNDARY_DEFEATED";
pub(crate) const RELEASED: &str = "EVENT_MN_KETTLURK_RELEASED";
pub(crate) const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";

#[derive(Deserialize)]
pub(crate) struct ArcSource {
    schema_version: u8,
    id: String,
    pub texts: BTreeMap<String, Vec<String>>,
    pub grunts: Vec<crate::northstar_rival_grunts::Guard>,
}

/// Completes Reed's regional arc and stages Kettlurk's physical intake release.
///
/// Requires the unbound interleaved-mine parent with installed native Kettlurk
/// overworld art. Adds three rival battles, an acted worker intervention and a
/// separately operated intake. Existing lairs, badges and quest results remain
/// independent. Apply the Minnesota cast binder after this overlay.
/// The certified current-region composer instead retains the existing binding
/// and binds only the story delta after all chapters have been installed.
///
/// # Errors
/// Returns an error for incompatible content, occupied identities, missing
/// native art, malformed source or layouts, or failed runtime validation.
pub fn build_northstar_rival_arc(
    base: &CompiledGamePack,
    source: &str,
    intake_layout: &str,
) -> Result<CompiledGamePack> {
    crate::northstar_story_revision::ensure_legacy_stage_allowed(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.report
            .manifests
            .iter()
            .any(|m| m == "northstar-iron-workings-v1")
            && !base.report.manifests.iter().any(|m| m == MANIFEST)
            && (!base
                .report
                .manifests
                .iter()
                .any(|m| m == "northstar-minnesota-only-v1")
                || crate::northstar_current_story::is_composing(base)),
        "unbound interleaved-mine parent required"
    );
    ensure!(
        base.runtime_files
            .contains_key("gfx/pokemon/kettlurk/overworld.2bpp")
            && base
                .runtime_files
                .contains_key("gfx/pokemon/gillpup/overworld.2bpp"),
        "install the reviewed native Kettlurk overworld bank first"
    );
    let source: ArcSource = serde_json::from_str(source)?;
    ensure!(
        source.schema_version == 1 && source.id == MANIFEST,
        "supported rival arc source"
    );
    let mut pack = base.clone();
    for flag in [
        EARLY, EARLY_WON, DEPOT, DEPOT_WON, WORKER, LINK, FINAL_WON, RELEASED,
    ] {
        a::flag(&mut pack, flag)?;
    }
    for (id, party) in [
        ("MN_REED_ROCHESTER", vec![("CROOKIT", 8), ("MIDGEBIT", 7)]),
        (
            "MN_REED_STPAUL",
            vec![("CROOKIT", 26), ("GOOSNIDE", 25), ("WRIGGLET", 26)],
        ),
        (
            "MN_REED_BOUNDARY",
            vec![
                ("CROOKVEIL", 44),
                ("GOOSNIDE", 43),
                ("CARTICK", 43),
                ("PUPUMP", 45),
            ],
        ),
    ] {
        a::trainer(&mut pack, id, "RIVAL1", "REED", &party)?;
    }
    crate::northstar_rival_early::install(&mut pack, &source)?;
    crate::northstar_rival_middle::install(&mut pack, &source)?;
    crate::northstar_rival_finale::install(&mut pack, &source, intake_layout)?;
    crate::northstar_rival_returns::install(&mut pack, &source)?;
    crate::northstar_rival_grunts::install(&mut pack, &source.grunts)?;
    pack.report.maps = pack.data.maps.len();
    pack.report.manifests.push(MANIFEST.into());
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

pub(crate) fn texts(room: &mut MapModule, source: &ArcSource, prefix: &str) {
    for (label, lines) in source.texts.iter().filter(|(k, _)| k.starts_with(prefix)) {
        authoring::text(
            room,
            label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
}
pub(crate) fn replace_texts(room: &mut MapModule, source: &ArcSource, prefix: &str) {
    for (label, lines) in source
        .texts
        .iter()
        .filter(|(key, _)| key.starts_with(prefix) && room.scripts.contains_key(*key))
        .collect::<Vec<_>>()
    {
        authoring::text(
            room,
            label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
}
pub(crate) fn put(room: &mut MapModule, label: &str, commands: Vec<Value>) {
    room.scripts.insert(label.into(), json!(commands));
}
pub(crate) fn jump(room: &mut MapModule, label: &str, text: &str) {
    put(room, label, vec![c("jumptext", &[text])]);
}
pub(crate) fn publish(pack: &mut CompiledGamePack, mut room: MapModule) -> Result<()> {
    crate::ely_league_rooms::finish(pack, &mut room)?;
    room.script_audio_commands = crate::parse_script_audio_commands(&room.id, &room.scripts)?;
    crate::authored_town_publication::replace(pack, room)
}
pub(crate) fn actor(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    id: &str,
    sprite: &str,
    at: [u16; 2],
) -> Result<()> {
    a::actor(pack, room, id, sprite, at)?;
    Ok(())
}
pub(crate) fn movement(room: &mut MapModule, label: &str, steps: &[(&str, usize)]) {
    let mut rows = Vec::new();
    for &(direction, count) in steps {
        rows.extend((0..count).map(|_| c("step", &[direction])));
    }
    rows.push(c("step_end", &[]));
    put(room, label, rows);
}
pub(crate) fn battle(
    room: &mut MapModule,
    label: &str,
    trainer: &str,
    won: &str,
    before: &str,
    after: &str,
) {
    let lost = format!("{label}LostText");
    let mut rows = a::say(before);
    rows.extend([
        c("winlosstext", &[after, &lost]),
        c("loadtrainer", &["RIVAL1", trainer]),
        c("startbattle", &[]),
        c("reloadmapafterbattle", &[]),
        c("setevent", &[won]),
    ]);
    rows.extend(a::say(after));
    rows.push(c("end", &[]));
    put(room, label, rows);
    authoring::text(room, &lost, &["REED: Move. I have work to do."]);
}

pub(crate) fn map(pack: &CompiledGamePack, name: &str) -> Result<MapModule> {
    pack.data
        .maps
        .get(name)
        .cloned()
        .with_context(|| format!("story location {name}"))
}
