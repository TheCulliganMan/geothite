//! One ordered campaign recipe on the selected Glacier world.

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

/// Combines reviewed scenes and gyms with canonical writing and roster policy.
///
/// Historical donors contribute bounded deltas, never their older world. Scene
/// installation precedes the final writing/roster pass so a staged gift, cry or
/// conversation cannot reintroduce a retired species or obsolete dialogue.
/// Public roads are opened by the scene stage; action and League gates remain.
/// Returns any dialogue requiring further staging, which the publisher must
/// resolve before accepting the candidate. No saves or selection are changed.
///
/// # Errors
/// Rejects input drift, conflicting donor changes, invalid authored content,
/// or changes to existing audio. Inputs remain unchanged on failure.
pub fn integrate_northstar_campaign(
    current: &CompiledGamePack,
    gym_common: &CompiledGamePack,
    gym_review: &CompiledGamePack,
    story_common: &CompiledGamePack,
    story_review: &CompiledGamePack,
    consolidation_writing: &str,
    story: &str,
) -> Result<(CompiledGamePack, Vec<String>)> {
    ensure!(
        current.identity()?.content_hash
            == "38ef157b041a1809cc0aa89cdf4fa3080f46994b56c0dcc1e15365055e6eb693",
        "integrated campaign requires the selected Glacier world"
    );
    let staged = crate::consolidate_northstar_story(
        current,
        story_common,
        story_review,
        consolidation_writing,
    )?;
    let gyms =
        crate::northstar_gym_integration::install_reviewed_gyms(&staged, gym_common, gym_review)?;
    let scenes = crate::apply_minnesota_story_and_scene_fixes(&gyms, story)
        .context("integrated story scenes and public roads")?;
    let arrivals =
        crate::correct_minnesota_mart_arrivals(&scenes).context("integrated Mart door arrivals")?;
    let arrivals = crate::correct_minnesota_house_arrivals(&arrivals)
        .context("integrated house door arrivals")?;
    let trainers = crate::revise_minnesota_ordinary_challenges(&arrivals)
        .context("ordinary trainers challenge on sight")?;
    let passage = crate::add_northstar_driftless_passage(
        &trainers,
        include_str!(
            "../../../planning/minnesota/caves/geographic-audit-v01/r01-passage-draft.json"
        ),
    )
    .context("lit east-west Driftless passage")?;
    // Resize after installing the clinic scene, so its approaches are authored
    // for the final native room. A later clinic scene install would put the
    // triggers back at coordinates outside that room.
    let centers = crate::restore_minnesota_center_nurses(&passage)
        .context("native Centers and relocated Rochester scene")?;
    let (mut candidate, deferred) = crate::northstar_campaign_polish::polish_with_access(
        &centers,
        story,
        crate::northstar_story_revision::InvestigationAccess::AcceptEvidenceEarly,
    )?;
    crate::northstar_reward_receipts::install(&mut candidate)?;
    candidate.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        candidate.format_version,
        &candidate.data,
        &candidate.audio_manifest,
        &candidate.runtime_files,
        &candidate.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&candidate)?;
    ensure!(
        candidate.audio_manifest == current.audio_manifest
            && candidate.compiled_audio == current.compiled_audio
            && candidate.audio_compression == current.audio_compression,
        "campaign integration changed existing audio"
    );
    Ok((candidate, deferred))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::CompiledGamePack;

    fn load(root: &Path, name: &str) -> CompiledGamePack {
        crate::read_verified_compiled_game_pack(root.join("content-packs").join(name))
            .expect("exact external campaign input")
    }

    #[test]
    #[ignore = "requires five exact external packs; run in the final integration phase"]
    fn current_campaign_combines_scenes_gyms_services_and_roster() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let current = load(&root, "minnesota-glacier-rear-rail-v01.crystalpack");
        let gym_common = load(&root, "minnesota-reviewed-maps-v11.crystalpack");
        let gym_review = load(
            &root,
            "minnesota-eight-gyms-active-roster-review-v01.crystalpack",
        );
        let story_common = load(
            &root,
            "minnesota-minneapolis-citywide-cycling-v02.crystalpack",
        );
        let story_review = load(&root, "minnesota-dialogue-overhaul-v27.crystalpack");
        let writing = include_str!(
            "../../../planning/minnesota/revisions/story-consolidation-v01/writing.json"
        );
        let story = include_str!("../../../planning/minnesota/story.json");
        let (candidate, deferred) = super::integrate_northstar_campaign(
            &current,
            &gym_common,
            &gym_review,
            &story_common,
            &story_review,
            writing,
            story,
        )
        .expect("complete ordered campaign recipe");
        assert!(
            deferred.is_empty(),
            "unstaged canonical dialogue: {deferred:?}"
        );
        for name in [
            "MnRochesterGym",
            "MnMankatoGym",
            "MnMinneapolisGym",
            "MnStPaulGym",
            "MnStCloudGym",
            "MnBrainerdGym",
            "MnBemidjiGym",
            "MnDuluthGym",
        ] {
            assert_eq!(
                candidate.data.maps[name].blocks,
                gym_review.data.maps[name].blocks
            );
            assert_eq!(
                candidate.data.maps[name].events.warps,
                gym_review.data.maps[name].events.warps
            );
        }
        for (name, original) in &current.data.maps {
            if !name.starts_with("Mn") {
                assert_eq!(
                    &candidate.data.maps[name], original,
                    "unrelated region: {name}"
                );
            }
        }
        assert_eq!(candidate.audio_manifest, current.audio_manifest);
        assert_eq!(candidate.compiled_audio, current.compiled_audio);
        let station = &candidate.data.maps["MnWinonaFieldStation"];
        for label in [
            "MnOverhaulLabEntry",
            "MnWFollowup",
            "MnWStationSyncCallback",
            "MnRewardDex",
            "MnEarlyPartners",
        ] {
            assert!(
                station.scripts.contains_key(label),
                "opening/service: {label}"
            );
        }
        let clinic = &candidate.data.maps["MnRochesterPokecenter1F"];
        assert_eq!((clinic.attributes.width, clinic.attributes.height), (5, 4));
        assert!(clinic.events.coord_events.iter().any(|event| {
            event.script_name == "MnNativeCenterReedEntry3" && (event.x, event.y) == (3, 4)
        }));
        assert!(clinic.events.coord_events.iter().all(|event| event.y < 8));
        let hollow = &candidate.data.maps["MnR01Cave1F"];
        assert_eq!((hollow.attributes.width, hollow.attributes.height), (20, 8));
        assert_eq!(hollow.events.warps.len(), 4);
        assert!(candidate.data.items.contains_key("CANOE"));
        assert!(
            candidate
                .report
                .manifests
                .iter()
                .any(|marker| marker == "northstar-open-public-roads-v1")
        );
    }
}
