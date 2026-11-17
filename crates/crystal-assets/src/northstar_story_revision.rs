//! Final, authoritative scene and prose revision on an installed Northstar game.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule};

pub(crate) const MANIFEST: &str = "northstar-dialogue-overhaul-v1";

pub(crate) fn ensure_legacy_stage_allowed(base: &CompiledGamePack) -> Result<()> {
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|manifest| manifest == MANIFEST),
        "canonical staged dialogue is installed; rebuild from a reviewed parent with revise_northstar_story"
    );
    Ok(())
}

/// Protects canonical dialogue from later local-flavor overlays.
pub(crate) fn owns_text(map: &str, label: &str) -> bool {
    static LABELS: OnceLock<BTreeMap<String, BTreeSet<String>>> = OnceLock::new();
    LABELS
        .get_or_init(|| {
            let source: serde_json::Value =
                serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))
                    .expect("checked-in canonical story JSON must be valid");
            source["dialogue_overhaul"]["texts"]
                .as_object()
                .expect("canonical dialogue map table must exist")
                .iter()
                .map(|(map, texts)| {
                    (
                        map.clone(),
                        texts
                            .as_object()
                            .expect("canonical dialogue labels must be objects")
                            .keys()
                            .cloned()
                            .collect(),
                    )
                })
                .collect()
        })
        .get(map)
        .is_some_and(|labels| labels.contains(label))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    schema_version: u8,
    id: String,
    approved_parents: Vec<String>,
    texts: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>,
}

/// Applies canonical pages to plain text programs in an installed Northstar pack.
///
/// Returns the revised pack and the map and label of each conversation that
/// needs separate staging because its installed program is missing, shared
/// with differing text, or includes commands beyond ordinary text pagination.
///
/// # Errors
/// Rejects invalid source pages, invalid regional packs, or invalid output.
pub fn refresh_installed_northstar_story(
    base: &CompiledGamePack,
    source: &str,
) -> Result<(CompiledGamePack, Vec<String>)> {
    refresh_installed_story(base, source, InvestigationAccess::AcceptEvidenceEarly)
}

pub(crate) enum InvestigationAccess {
    PreserveBadgeChecks,
    AcceptEvidenceEarly,
}

pub(crate) fn refresh_installed_story(
    base: &CompiledGamePack,
    source: &str,
    access: InvestigationAccess,
) -> Result<(CompiledGamePack, Vec<String>)> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    crate::validate_compact_superior_geography(base)?;
    let root: Value = serde_json::from_str(source)?;
    let authored_texts = &root["dialogue_overhaul"]["texts"];
    let revision: Revision = serde_json::from_value(root["dialogue_overhaul"].clone())
        .context("canonical dialogue overhaul")?;
    ensure!(
        revision.schema_version == 1 && revision.id == MANIFEST,
        "unsupported dialogue revision"
    );

    let mut pack = base.clone();
    if let Some(pages) = revision
        .texts
        .get("MnStPaulUnionDepot")
        .and_then(|texts| texts.get("MnNSTPMinaDeclinedText"))
    {
        crate::northstar_story_bridges::install_mina_refusal(&mut pack, pages)?;
    }
    if let Some(texts) = revision.texts.get("MnGrandMaraisFieldStation") {
        crate::northstar_marais_part::install(&mut pack, texts)?;
    }
    if let Some(texts) = revision.texts.get("MnGrandMaraisLighthouse") {
        crate::northstar_lighthouse_reward::install(&mut pack, texts)?;
    }
    if let Some(texts) = revision.texts.get("MnWinonaHome2F")
        && pack.data.maps.contains_key("MnWinonaHome2F")
    {
        crate::northstar_opening_revision::install_cousin_dialogue(&mut pack, Some(texts))?;
    }
    if let Some(texts) = revision.texts.get("MnWinonaFieldStation")
        && pack.data.maps.contains_key("MnWinonaFieldStation")
    {
        crate::northstar_opening_revision::install_tamarack_guidance(&mut pack, texts)?;
    }
    if let Some(texts) = revision.texts.get("MnRochesterPokecenter1F")
        && pack.data.maps.contains_key("MnRochesterPokecenter1F")
    {
        crate::northstar_clinic_revision::install_ivy_reminders(&mut pack, texts)?;
    }
    if matches!(access, InvestigationAccess::AcceptEvidenceEarly) {
        crate::northstar_story_pacing::install_headwaters(&mut pack)?;
    }
    crate::northstar_story_pacing::install_returns(&mut pack, Some(&revision.texts))?;
    install_shop_greetings(&mut pack, &revision)?;
    if let Some(texts) = revision.texts.get("MnMinneapolisBattleClub") {
        crate::northstar_club_dialogue::install(&mut pack, texts)?;
    }
    if let Some(pages) = revision
        .texts
        .get("MnDuluthHarborWorkshop")
        .and_then(|texts| texts.get("MnOverhaulHarborMissingLogText"))
    {
        crate::northstar_story_pacing::install_harbor_missing_log(&mut pack, pages)?;
        if matches!(access, InvestigationAccess::AcceptEvidenceEarly) {
            crate::northstar_story_pacing::install_harbor(&mut pack)?;
        }
    }
    let mut deferred = Vec::new();
    for (name, texts) in revision.texts {
        let Some(map) = pack.data.maps.get_mut(&name) else {
            deferred.extend(texts.keys().map(|label| format!("{name}/{label}")));
            continue;
        };
        let mut changed = false;
        for (label, pages) in texts {
            let mut replacement = authored_dialogue::compile(&pages)
                .with_context(|| format!("dialogue {name}/{label}"))?;
            let Some(old) = map.scripts.get(&label).and_then(Value::as_array) else {
                deferred.push(format!("{name}/{label}"));
                continue;
            };
            // A shared label can change only when every installed owner
            // has the same old script and requests the same new pages.
            let conflicting_owner = base.data.maps.values().any(|other| {
                other.id != name
                    && other.scripts.get(&label).is_some_and(|body| {
                        body != &map.scripts[&label]
                            || authored_texts[&other.id][&label] != authored_texts[&name][&label]
                    })
            });
            let Some((last, lines)) = old.split_last() else {
                deferred.push(format!("{name}/{label}"));
                continue;
            };
            if conflicting_owner
                || !matches!(last["command"].as_str(), Some("done" | "prompt"))
                || !lines.iter().all(|line| {
                    matches!(
                        line["command"].as_str(),
                        Some("text" | "line" | "para" | "cont")
                    )
                })
            {
                deferred.push(format!("{name}/{label}"));
                continue;
            }
            let terminal = replacement
                .as_array_mut()
                .and_then(|commands| commands.last_mut())
                .expect("authored dialogue compiler always emits a terminator");
            *terminal = last.clone();
            if pack.data.map_scripts.get(&label).is_some_and(|body| {
                body != &map.scripts[&label]
                    && (body != &replacement
                        || base.data.map_scripts.get(&label) != Some(&map.scripts[&label]))
            }) {
                deferred.push(format!("{name}/{label}"));
                continue;
            }
            if let Some(mirror) = pack.data.map_scripts.get_mut(&label) {
                *mirror = replacement.clone();
            }
            map.scripts.insert(label, replacement);
            changed = true;
        }
        if changed {
            map.script_text_commands = crate::parse_script_text_commands(&name, &map.scripts)?;
            map.script_text_bodies = crate::parse_script_text_bodies(&name, &map.scripts)?;
        }
    }
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok((pack, deferred))
}

fn install_shop_greetings(pack: &mut CompiledGamePack, revision: &Revision) -> Result<()> {
    for (name, clerk) in [
        ("MnMinneapolis", "MN_MP_MARKET_SNACK"),
        ("MnMinneapolis", "MN_MP_MARKET_BERRY"),
        ("MnMinneapolis", "MN_MP_MARKET_TRAIL"),
        ("MnMinneapolisGardenShop", "MN_MP_GARDENSHOP_CLERK"),
        ("MnMinneapolisSouthMarket", "MN_MP_SOUTHMARKET_CLERK"),
        ("MnMinneapolisSupplyCoop", "MN_MP_SUPPLYCOOP_CLERK"),
        ("MnStPaulFairFestivalMail", "MN_SPX_FAIRFESTIVALMAIL_HOST"),
        ("MnStPaulFairGardenStand", "MN_SPX_FAIRGARDENSTAND_HOST"),
        ("MnStPaulRiversideSupplies", "MN_SPX_RIVERSIDESUPPLIES_HOST"),
        ("MnStCloudMart", "MN_ST_CLOUD_MART_CLERK"),
        ("MnDetroitLakesMart", "MN_DETROIT_MART_CLERK"),
        ("MnBrainerdMart", "MN_BRAINERD_MART_CLERK"),
        ("MnBemidjiMart", "MN_BEMIDJI_CLERK"),
        ("MnGrandRapidsMart", "MN_GR_MART_CLERK"),
        ("MnDuluthMart", "MN_DULUTH_MART_CLERK"),
        (
            "MnDuluthCommunityKitchen",
            "MN_DULUTH_X_COMMUNITYKITCHEN_F1_HOST",
        ),
        ("MnDuluthDockSupplies", "MN_DULUTH_X_DOCKSUPPLIES_F1_HOST"),
        ("MnDuluthHillSupplies", "MN_DULUTH_X_HILLSUPPLIES_F1_HOST"),
        ("MnDuluthLakeOutfitter", "MN_DULUTH_X_LAKEOUTFITTER_F1_HOST"),
        ("MnDuluthMaritimeCoop1F", "MN_DULUTH_X_MARITIMECOOP_F1_HOST"),
        ("MnDuluthMaritimeCoop2F", "MN_DULUTH_X_MARITIMECOOP_F2_HOST"),
        (
            "MnDuluthPointOutfitter",
            "MN_DULUTH_X_POINTOUTFITTER_F1_HOST",
        ),
        ("MnDuluthToolExchange", "MN_DULUTH_X_TOOLEXCHANGE_F1_HOST"),
        ("MnMinneapolisDepartmentStore1F", "MN_MP_DEPT_1_CLERK"),
        ("MnMinneapolisDepartmentStore2F", "MN_MP_DEPT_2_CLERK"),
        ("MnMinneapolisDepartmentStore3F", "MN_MP_DEPT_3_CLERK"),
        ("MnMinneapolisDepartmentStore4F", "MN_MP_DEPT_4_CLERK"),
        ("MnMinneapolisDepartmentStore5F", "MN_MP_DEPT_5_CLERK"),
        ("MnMinneapolisMart", "MN_MP_MART_CLERK"),
        ("MnStPaulFairBerrySnack", "MN_SPX_FAIRBERRYSNACK_HOST"),
        ("MnStPaulFairFoodHall", "MN_SPX_FAIRFOODHALL_HOST"),
        ("MnStPaulLowertownCafe", "MN_SPX_LOWERTOWNCAFE_HOST"),
        ("MnStPaulLowertownCrafts", "MN_SPX_LOWERTOWNCRAFTS_HOST"),
        ("MnStPaulMart", "MN_ST_PAUL_MART_CLERK"),
        ("MnStPaulWestSeventhBakery", "MN_SPX_WESTSEVENTHBAKERY_HOST"),
    ] {
        let label = format!("{clerk}Text");
        let Some(pages) = revision.texts.get(name).and_then(|texts| texts.get(&label)) else {
            continue;
        };
        let Some(mut map) = pack.data.maps.get(name).cloned() else {
            continue;
        };
        let script = map
            .scripts
            .get_mut(clerk)
            .and_then(Value::as_array_mut)
            .with_context(|| format!("missing shop interaction {name}/{clerk}"))?;
        // A broad prose overlay may have installed the text program without
        // connecting it to the clerk. Presence alone is not a staged greeting.
        if script
            .iter()
            .any(|row| *row == command("writetext", &[&label]))
        {
            continue;
        }
        ensure!(
            matches!(script.as_slice(), [face, open, mart, close, end]
                if *face == command("faceplayer", &[])
                    && *open == command("opentext", &[])
                    && mart["command"] == "pokemart"
                    && mart["args"][0] == "MARTTYPE_STANDARD"
                    && *close == command("closetext", &[])
                    && *end == command("end", &[])),
            "changed shop interaction {name}/{clerk}"
        );
        script.splice(
            2..2,
            [command("writetext", &[&label]), command("waitbutton", &[])],
        );
        map.scripts
            .insert(label, authored_dialogue::compile(pages)?);
        publish(pack, map)?;
    }
    Ok(())
}

/// Revises the installed Minnesota story without rebuilding unrelated content.
///
/// The canonical `dialogue_overhaul` section owns every revised text label and
/// runs after chapter overlays. Existing event identities and save identities
/// are not migrated. The returned pack has a new content identity.
/// Public reference records and recovered evidence are accepted independently
/// of Gym credentials. Existing travel safeguards still apply, and return
/// conversations follow evidence custody, radio contact and completed shutdown.
///
/// # Errors
/// Rejects unreviewed parents, duplicate publication, missing scene anchors,
/// invalid native pages, retired geography, or invalid resulting runtime data.
/// The borrowed parent remains unchanged on error.
pub fn revise_northstar_story(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    crate::validate_compact_superior_geography(base)
        .context("dialogue parent outdoor placement")?;
    let revision = read_revision(base, source)?;
    ensure!(
        revision
            .approved_parents
            .contains(&base.identity()?.content_hash),
        "reviewed current regional dialogue parent required"
    );
    let pack = install_revision(base, revision)?;
    crate::validate_compact_superior_geography(&pack)
        .context("revised dialogue outdoor placement")?;
    Ok(pack)
}

// The original exact-parent composer precedes the later climate/grass overlays.
// Share its final scenes without making that intermediate a current-world
// publication parent. The composer separately preserves every outdoor cell
// except the guarded League turnout and verifies its original compact ports.
pub(crate) fn finish_composition(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        crate::northstar_current_story::is_composing(base),
        "active guarded story composition required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let revision = read_revision(base, source)?;
    let pack = install_revision(base, revision)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    Ok(pack)
}

fn read_revision(base: &CompiledGamePack, source: &str) -> Result<Revision> {
    let root: serde_json::Value = serde_json::from_str(source)?;
    let revision: Revision = serde_json::from_value(root["dialogue_overhaul"].clone())
        .context("canonical dialogue overhaul")?;
    ensure!(
        revision.schema_version == 1 && revision.id == MANIFEST,
        "unsupported dialogue revision"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "dialogue overhaul already installed"
    );
    Ok(revision)
}

fn install_revision(base: &CompiledGamePack, revision: Revision) -> Result<CompiledGamePack> {
    let mut pack = base.clone();
    crate::northstar_opening_revision::install(&mut pack)?;
    crate::northstar_courier_revision::install(&mut pack)?;
    let mut holding = pack
        .data
        .maps
        .get("MnMinneapolisDispatchOffice")
        .context("installed holding room")?
        .clone();
    crate::northstar_holding_fence::install(&mut pack, &mut holding)?;
    publish(&mut pack, holding)?;
    install_shop_greetings(&mut pack, &revision)?;
    if let Some(texts) = revision.texts.get("MnMinneapolisBattleClub") {
        crate::northstar_club_dialogue::install(&mut pack, texts)?;
    }
    if let Some(texts) = revision.texts.get("MnGrandMaraisFieldStation") {
        crate::northstar_marais_part::install(&mut pack, texts)?;
    }
    if let Some(texts) = revision.texts.get("MnGrandMaraisLighthouse") {
        crate::northstar_lighthouse_reward::install(&mut pack, texts)?;
    }
    for (name, texts) in revision.texts {
        ensure!(
            name.starts_with("Mn"),
            "dialogue revision is Minnesota-only"
        );
        let labels = texts.keys().cloned().collect::<BTreeSet<_>>();
        let mut map = pack
            .data
            .maps
            .get(&name)
            .context("revised story map")?
            .clone();
        for (label, pages) in texts {
            ensure!(
                map.script_text_bodies.contains_key(&label) || label.starts_with("MnOverhaul"),
                "missing revised dialogue anchor: {name}/{label}"
            );
            map.scripts.insert(
                label.clone(),
                authored_dialogue::compile(&pages)
                    .with_context(|| format!("dialogue {name}/{label}"))?,
            );
        }
        // These story waits are authored acknowledgements: expose the native
        // cursor and confirmation cue on the final page as well as paragraphs.
        for body in map
            .scripts
            .values_mut()
            .filter_map(serde_json::Value::as_array_mut)
        {
            for index in 1..body.len() {
                if body[index]["command"] == "waitbutton"
                    && body[index - 1]["command"] == "writetext"
                    && body[index - 1]["args"][0]
                        .as_str()
                        .is_some_and(|label| labels.contains(label))
                {
                    body[index]["command"] = "promptbutton".into();
                }
            }
        }
        publish(&mut pack, map)?;
    }
    crate::northstar_clinic_revision::install(&mut pack)?;
    crate::northstar_dispatch_revision::install(&mut pack)?;
    let mut officer = pack.data.maps["MnBemidji"].clone();
    crate::northstar_report_revision::install(&mut officer)?;
    publish(&mut pack, officer)?;
    crate::northstar_story_connections::install(&mut pack)?;
    stage_harbor(&mut pack)?;
    crate::northstar_story_pacing::install(&mut pack)?;
    crate::northstar_story_staff::install(&mut pack)?;
    // The cast binder verifies its complete input before changing any art.
    // Finalize the staged scripts' identity before crossing that boundary.
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    let mut pack = crate::northstar_story_cast::install(&pack)?;
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

fn stage_harbor(pack: &mut CompiledGamePack) -> Result<()> {
    use crate::grand_marais_modpack::authoring::command as c;

    let mut map = pack
        .data
        .maps
        .get("MnDuluthHarborWorkshop")
        .context("harbor workshop")?
        .clone();
    let body = map
        .scripts
        .get_mut("MN_HARBOR_RECEIVER")
        .and_then(serde_json::Value::as_array_mut)
        .context("harbor conversation")?;
    let depot = body
        .iter()
        .position(|row| *row == c("checkevent", &["EVENT_MN_DEPOT_CLEARED"]))
        .context("harbor custody prerequisite")?;
    ensure!(
        body.get(depot + 1) == Some(&c("iffalse", &["MN_HARBOR_RECEIVERDenied"])),
        "harbor custody branch changed"
    );
    body[depot + 1] = c("iffalse", &["MnOverhaulHarborMissingLog"]);
    let at = body
        .iter()
        .position(|row| *row == c("writetext", &["MN_HARBOR_RECEIVERText"]))
        .context("harbor evidence page")?;
    ensure!(
        body.get(at + 1) == Some(&c("promptbutton", &[])),
        "harbor acknowledgement changed"
    );
    let mut sequence = vec![
        c("writetext", &["MN_HARBOR_RECEIVERText"]),
        c("promptbutton", &[]),
    ];
    for label in [
        "MnOverhaulHarborEvidenceText",
        "MnOverhaulHarborReactionText",
        "MnOverhaulHarborDirectionsText",
    ] {
        sequence.extend([
            c("closetext", &[]),
            c("pause", &["10"]),
            c("opentext", &[]),
            c("writetext", &[label]),
            c("promptbutton", &[]),
        ]);
    }
    body.splice(at..at + 2, sequence);
    map.scripts.insert(
        "MnOverhaulHarborMissingLog".into(),
        serde_json::json!([c("jumptextfaceplayer", &["MnOverhaulHarborMissingLogText"])]),
    );
    publish(pack, map)
}

pub(crate) fn publish(pack: &mut CompiledGamePack, mut map: MapModule) -> Result<()> {
    crate::ely_league_rooms::finish(pack, &mut map)?;
    map.script_audio_commands = crate::parse_script_audio_commands(&map.id, &map.scripts)?;
    crate::authored_town_publication::replace(pack, map)
}

#[cfg(test)]
#[path = "northstar_story_wording_tests.rs"]
mod wording_tests;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    #[test]
    #[ignore = "requires the frozen shifted-seam CRYSTAL_STORY_SHIFTED_REVIEW_PACK"]
    fn dialogue_rejects_the_historical_shifted_league_placement_cycle() {
        let pack = crate::read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_STORY_SHIFTED_REVIEW_PACK")
                .expect("explicit historical shifted-seam review pack"),
        ))
        .expect("readable historical review pack");
        let identity = pack.identity().expect("unchanged historical identity");
        let error = crate::validate_compact_superior_geography(&pack)
            .expect_err("local walking offsets cannot contradict the regional cycle");
        assert!(error.to_string().contains("inconsistent outdoor placement"));
        let source = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/story.json"),
        )
        .expect("canonical source");
        let error = super::revise_northstar_story(&pack, &source)
            .expect_err("publication must reject the inconsistent parent before mutation");
        assert!(
            error
                .to_string()
                .contains("dialogue parent outdoor placement")
        );
        assert_eq!(pack.identity().expect("borrowed input retained"), identity);
    }

    #[test]
    fn dialogue_canonical_pages_cover_the_campaign_and_are_protected() {
        let source: serde_json::Value =
            serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))
                .expect("canonical JSON");
        let revision: super::Revision = serde_json::from_value(source["dialogue_overhaul"].clone())
            .expect("canonical dialogue schema");
        for (map, texts) in revision.texts {
            for (label, pages) in texts {
                assert!(
                    crate::northstar_dialogue::is_reviewed_story_text(&map, &label),
                    "canonical label must survive local overlays: {map}/{label}"
                );
                crate::authored_dialogue::compile(&pages).expect("authored native pages");
            }
        }
        assert!(super::owns_text("MnLeagueHallOfFame", "MnElyRecordText"));
        assert!(super::owns_text(
            "MnDuluthHarborWorkshop",
            "MnHarborSurveyReturnText"
        ));
        assert!(!super::owns_text("MnWinona", "UnrelatedFlavorText"));
    }

    #[test]
    #[ignore = "requires external CRYSTAL_STORY_PRIOR_PARENT and CRYSTAL_STORY_REVISION_PARENT"]
    fn dialogue_parent_keeps_reviewed_story_anchors() {
        let read = |name| {
            crate::read_verified_compiled_game_pack(PathBuf::from(
                std::env::var_os(name).expect("external dialogue parent"),
            ))
            .expect("verified dialogue parent")
        };
        let before = read("CRYSTAL_STORY_PRIOR_PARENT");
        let after = read("CRYSTAL_STORY_REVISION_PARENT");
        let source: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../planning/minnesota/story.json"),
            )
            .expect("canonical story"),
        )
        .expect("canonical JSON");
        assert_eq!(before.data.maps.len(), after.data.maps.len());
        assert_eq!(before.compiled_audio(), after.compiled_audio());
        assert_eq!(before.data.pokemon, after.data.pokemon);
        assert_eq!(before.data.trainers, after.data.trainers);
        assert_eq!(before.data.items, after.data.items);
        assert_eq!(before.data.moves, after.data.moves);
        for (name, value) in &before.data.story_event_script_constants.global {
            assert_eq!(
                after.data.story_event_script_constants.global.get(name),
                Some(value),
                "preserved global identity {name}"
            );
        }
        for (map, constants) in &before.data.story_event_script_constants.maps {
            if map != "MnR03Cave1F" {
                assert!(
                    after.data.story_event_script_constants.maps.get(map) == Some(constants),
                    "preserved local constants {map}"
                );
            }
        }
        for name in source["dialogue_overhaul"]["texts"]
            .as_object()
            .expect("story maps")
            .keys()
        {
            if matches!(
                name.as_str(),
                "MnLakeItasca"
                    | "MnRainyRiverIcefields"
                    | "MnDuluth"
                    | "MnGrandRapids"
                    | "MnGrandMarais"
                    | "MnRochester"
            ) {
                let mut expected = before.data.maps[name].clone();
                expected.blocks.clone_from(&after.data.maps[name].blocks);
                super::wording_tests::retain_reviewed_facade_events(
                    name,
                    &mut expected,
                    &after.data.maps[name],
                );
                assert_eq!(
                    expected, after.data.maps[name],
                    "reviewed native geography only {name}"
                );
                continue;
            }
            if name == "MnWinonaFieldStation"
                && after
                    .report
                    .manifests
                    .iter()
                    .any(|m| m == "minnesota-kindlit-research-v1")
            {
                let before = &before.data.maps[name];
                let after = &after.data.maps[name];
                assert_eq!(before.blocks, after.blocks);
                assert_eq!(before.attributes, after.attributes);
                assert_eq!(before.events, after.events);
                assert_eq!(before.objects.len(), after.objects.len());
                for (original, revised) in before.objects.iter().zip(&after.objects) {
                    let mut restored = revised.clone();
                    if original.object_identifier.as_deref() == Some("MN_W_AIDE") {
                        assert_eq!(revised.script, "MnKindlitResearchAide");
                        restored.script.clone_from(&original.script);
                    }
                    assert_eq!(*original, restored);
                }
                for (label, body) in &before.scripts {
                    assert_eq!(
                        after.scripts.get(label),
                        Some(body),
                        "existing aide/scene program {label}"
                    );
                }
                assert!(
                    after
                        .scripts
                        .keys()
                        .filter(|label| !before.scripts.contains_key(*label))
                        .all(|label| label.starts_with("MnKindlit"))
                );
                continue;
            }
            assert!(
                before.data.maps[name] == after.data.maps[name],
                "story anchor map {name}"
            );
        }
        for (name, map) in &before.data.maps {
            if map != &after.data.maps[name] {
                if matches!(
                    name.as_str(),
                    "MnR09C00R00"
                        | "MnR09C00R01"
                        | "MnR09C01R00"
                        | "MnR09C01R01"
                        | "MnR10C00R00"
                        | "MnR10C01R00"
                        | "MnMississippiUpstreamWoodsC01R00"
                        | "MnMississippiConfluenceC01R00"
                        | "MnMississippiConfluenceC02R00"
                        | "MnRainyRiverIcefields"
                        | "MnDuluth"
                        | "MnStateFairgrounds"
                        | "MnGrandRapids"
                        | "MnGrandMarais"
                        | "MnRochester"
                ) {
                    let mut expected = map.clone();
                    expected.blocks.clone_from(&after.data.maps[name].blocks);
                    super::wording_tests::retain_reviewed_facade_events(
                        name,
                        &mut expected,
                        &after.data.maps[name],
                    );
                    assert_eq!(
                        expected, after.data.maps[name],
                        "reviewed geography only {name}"
                    );
                    continue;
                }
                assert!(
                    matches!(
                        name.as_str(),
                        "MnR20C00R00"
                            | "MnR20C00R01"
                            | "MnMaraisCedarApproach"
                            | "MnR03Cave1F"
                            | "MnBemidjiNorthwoods"
                            | "MnDetroitLakesApproach"
                            | "MnElyShorePass"
                            | "MnWinonaFieldStation"
                            | "MnMilleLacs"
                            | "MnGrandMaraisFieldStation"
                            | "MnLakeItasca"
                            | "MnPreservePineEntry"
                            | "MnMississippiLowerWoodsC01R01"
                    ),
                    "unreviewed parent change {name}"
                );
                eprintln!("reviewed terrain descendant: {name}");
            }
        }
    }

    #[test]
    #[ignore = "requires CRYSTAL_STORY_REVISION_PARENT external current-region pack"]
    fn dialogue_revision_preserves_geography_catalogs_and_unrelated_rooms() {
        let base = crate::read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_STORY_REVISION_PARENT").expect("reviewed current pack"),
        ))
        .expect("verified parent");
        let source = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/story.json"),
        )
        .expect("canonical story");
        let child = super::revise_northstar_story(&base, &source).expect("staged revision");
        let cast = crate::northstar_story_cast::install(&base).expect("accepted cast bindings");
        let mut expected_trainers = base.data.trainers.clone();
        for id in [
            "MN_REED_WINONA",
            "MN_REED_ROCHESTER",
            "MN_REED_STPAUL",
            "MN_REED_RELAY",
            "MN_REED_MINE",
            "MN_REED_BOUNDARY",
        ] {
            expected_trainers
                .trainers
                .get_mut(id)
                .expect("existing rival variant")
                .trainer_class = "MN_RIVAL_REED".into();
        }
        assert_eq!(
            expected_trainers, cast.data.trainers,
            "only Reed's private art class changes; all parties, identities, AI and rewards remain exact"
        );
        crate::northstar_holding_fence::assert_enclosure(&child);
        if let Some(path) = std::env::var_os("CRYSTAL_STORY_FROZEN_CAMPAIGN") {
            let frozen = crate::read_verified_compiled_game_pack(PathBuf::from(path))
                .expect("frozen campaign pack");
            let frozen = super::wording_tests::reauthor_frozen(frozen, &source);
            let frozen = crate::northstar_story_cast::install(&frozen)
                .expect("same exact cast bindings on the frozen story");
            assert_eq!(frozen.data.maps.len(), child.data.maps.len());
            for (name, map) in &frozen.data.maps {
                let revised = &child.data.maps[name];
                assert_eq!(
                    map.scripts, revised.scripts,
                    "frozen campaign programs {name}"
                );
                assert_eq!(
                    map.script_text_bodies, revised.script_text_bodies,
                    "frozen campaign pages {name}"
                );
                assert_eq!(
                    map.objects, revised.objects,
                    "frozen campaign participants {name}"
                );
                let mut expected_events = map.events.clone();
                super::wording_tests::retain_reviewed_sign_positions(
                    name,
                    &mut expected_events,
                    &base.data.maps[name].events,
                );
                assert_eq!(
                    expected_events, revised.events,
                    "frozen campaign events {name}"
                );
            }
        }
        if let Some(path) = std::env::var_os("CRYSTAL_STORY_REVISION_PACK") {
            let published = crate::read_verified_compiled_game_pack(PathBuf::from(path))
                .expect("published dialogue candidate");
            assert_eq!(
                published.identity().expect("published identity"),
                child.identity().expect("source identity"),
                "published candidate must match the current canonical builder"
            );
        }
        assert_eq!(base.data.maps.len(), child.data.maps.len());
        assert_eq!(base.compiled_audio(), child.compiled_audio());
        for (name, bytes) in &base.runtime_files {
            assert_eq!(
                child.runtime_files.get(name),
                Some(bytes),
                "original art {name}"
            );
        }
        let added_files = if base
            .data
            .tilesets
            .contains_key(crate::northstar_holding_fence::TILESET)
        {
            0
        } else {
            5
        };
        assert_eq!(
            child.runtime_files.len(),
            cast.runtime_files.len() + added_files
        );
        assert!(
            child
                .runtime_files
                .keys()
                .filter(|name| !cast.runtime_files.contains_key(*name))
                .all(|name| name.contains("mn_northstar_holding"))
        );
        for (name, tileset) in &base.data.tilesets {
            assert_eq!(
                child.data.tilesets.get(name),
                Some(tileset),
                "original tileset {name}"
            );
        }
        assert_eq!(base.data.pokemon, child.data.pokemon);
        assert_eq!(base.data.moves, child.data.moves);
        for (path, bytes) in &cast.runtime_files {
            assert_eq!(
                child.runtime_files.get(path),
                Some(bytes),
                "accepted cast art {path}"
            );
        }
        assert_eq!(cast.data.trainers, child.data.trainers);
        let mut expected_staff = cast.data.maps["MnIceTransfer"].objects.clone();
        expected_staff
            .iter_mut()
            .find(|actor| actor.script == "MN_ICE_HQ_TESS")
            .expect("existing captive")
            .event_flag = "EVENT_MN_ICE_CAPTIVE_RESCUED".into();
        assert_eq!(
            expected_staff, child.data.maps["MnIceTransfer"].objects,
            "only Tess's existing rescue-event gate changes; all staff identities, positions and art remain exact"
        );
        assert_eq!(
            cast.data.trainer_class_names,
            child.data.trainer_class_names
        );
        assert_eq!(
            cast.data.sprite_palette_defaults,
            child.data.sprite_palette_defaults
        );
        assert_eq!(base.data.items, child.data.items);
        assert_eq!(base.data.wild_encounters, child.data.wild_encounters);
        assert_eq!(base.data.field_encounters, child.data.field_encounters);
        assert_eq!(
            base.data.runtime_title_screen,
            child.data.runtime_title_screen
        );
        assert_eq!(
            base.data.runtime_map_metadata,
            child.data.runtime_map_metadata
        );
        let source: serde_json::Value = serde_json::from_str(&source).expect("story JSON");
        let mut changed = source["dialogue_overhaul"]["texts"]
            .as_object()
            .expect("dialogue maps")
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        changed.extend([
            "MnWinonaHome1F",
            "MnWinonaFieldStation",
            "MnStPaulUnionDepot",
            "MnR13C00R00",
        ]);
        for (name, before) in &cast.data.maps {
            let after = &child.data.maps[name];
            let mut expected_blocks = before.blocks.clone();
            if name == "MnMinneapolisDispatchOffice" {
                for (x, y, id) in crate::northstar_holding_fence::CELLS {
                    expected_blocks[y * 10 + x] = id;
                }
            }
            if name == "MnR13C00R00" {
                expected_blocks[16] = expected_blocks[15];
            }
            assert_eq!(expected_blocks, after.blocks, "terrain {name}");
            let mut expected_attributes = before.attributes.clone();
            if name == "MnMinneapolisDispatchOffice" {
                expected_attributes.tileset_name = crate::northstar_holding_fence::TILESET.into();
            }
            for (map, direction, target, offset) in [
                ("MnBoundaryWaters", "south", "MnR13C00R00", 0),
                ("MnR13C00R00", "north", "MnBoundaryWaters", 0),
            ] {
                if name == map {
                    expected_attributes
                        .connections
                        .iter_mut()
                        .find(|link| link.direction == direction && link.target_map == target)
                        .expect("existing League connection")
                        .offset = offset;
                }
            }
            assert_eq!(
                expected_attributes, after.attributes,
                "connections and geometry {name}"
            );
            if name == "MnR13C00R00" {
                let mut expected = before.clone();
                expected.attributes = expected_attributes;
                expected.blocks = expected_blocks;
                assert_eq!(
                    &expected, after,
                    "League approach changes only its one native floor turnout and canonical offset"
                );
            }
            assert_eq!(before.events.warps, after.events.warps, "warps {name}");
            assert_eq!(
                before.events.bg_events, after.events.bg_events,
                "parent background events {name}"
            );
            if !changed.contains(name.as_str()) {
                assert!(before == after, "unrelated map {name}");
            }
            assert_eq!(
                child
                    .data
                    .assemble_map_module_from_compiled_payloads(name)
                    .expect("uncached native map"),
                *after,
                "native mirrors {name}"
            );
        }
        for (name, value) in &base.data.story_event_script_constants.global {
            assert_eq!(
                child.data.story_event_script_constants.global.get(name),
                Some(value),
                "event identity {name}"
            );
        }
        assert!(
            super::revise_northstar_story(&child, &serde_json::to_string(&source).expect("source"))
                .is_err()
        );
        for result in [
            crate::refresh_northstar_dialogue(&child, ""),
            crate::refresh_northstar_league_dialogue(&child, "", "", ""),
            crate::build_northstar_returns(&child, ""),
            crate::build_northstar_ice_lair(&child, "", "", ""),
            crate::build_northstar_drill_lair(&child, "", "", "", ""),
            crate::build_northstar_ice_headquarters(&child, "", &[]),
            crate::build_northstar_iron_workings(&child, "", &[]),
            crate::build_northstar_rival_arc(&child, "", ""),
        ] {
            match result {
                Err(error) => assert!(error.to_string().contains("canonical staged dialogue")),
                Ok(_) => panic!("legacy overlay must not replace canonical dialogue"),
            }
        }
        if child.data.maps["MnRochester"]
            .scripts
            .contains_key("MnFossilBagFull")
        {
            let mut rejected = child.clone();
            let mut map = rejected.data.maps["MnRochester"].clone();
            map.scripts
                .insert("MnFossilBagFull".into(), serde_json::json!([]));
            let result = crate::authored_town_publication::replace(&mut rejected, map);
            assert!(
                result.is_err_and(|error| error.to_string().contains("MnFossilBagFull")),
                "a shared fossil helper cannot be changed by a story publication"
            );
        }
        let mut invalid = source;
        invalid["dialogue_overhaul"]["texts"]["MnWinonaHome1F"]["MnWHomeIntroText"][0][0] =
            "A line that cannot fit the native text box".into();
        assert!(
            super::revise_northstar_story(
                &base,
                &serde_json::to_string(&invalid).expect("invalid source")
            )
            .is_err()
        );
    }
}
