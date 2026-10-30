//! Preserve a frozen campaign while independently accounting for revised prose.
use std::path::PathBuf;

use crate::CompiledGamePack;

pub(super) fn reauthor_frozen(mut pack: CompiledGamePack, source: &str) -> CompiledGamePack {
    let Some(path) = std::env::var_os("CRYSTAL_STORY_FROZEN_SOURCE") else {
        return pack;
    };
    let old: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(PathBuf::from(path)).expect("explicit frozen story source"),
    )
    .expect("frozen canonical JSON");
    let new: serde_json::Value = serde_json::from_str(source).expect("current canonical JSON");
    let old: super::Revision =
        serde_json::from_value(old["dialogue_overhaul"].clone()).expect("frozen dialogue schema");
    let new: super::Revision =
        serde_json::from_value(new["dialogue_overhaul"].clone()).expect("current dialogue schema");
    let mut changes = 0;
    for (name, labels) in old.texts {
        let mut map = pack.data.maps[&name].clone();
        let additions = new.texts[&name]
            .keys()
            .filter(|label| !labels.contains_key(*label))
            .map(String::as_str)
            .collect::<Vec<_>>();
        let expected: &[&str] = match name.as_str() {
            "MnBoundaryWatersSurveyShelter" => {
                &["MnOverhaulChiefExposedText", "MnOverhaulChiefStoppedText"]
            }
            "MnBemidji" => &[
                "MnOverhaulOfficerDoneText",
                "MnOverhaulOfficerHarborText",
                "MnOverhaulOfficerIntakeText",
                "MnOverhaulOfficerMeterText",
                "MnOverhaulOfficerNorthText",
            ],
            "MnBrainerd" => &["MnOverhaulRangerComparedText"],
            "MnLakeItasca" => &["MnOverhaulItascaComparedText"],
            "MnGrandRapidsSurveyDepot" => &[
                "MnOverhaulDepotDoneText",
                "MnOverhaulDepotIntakeText",
                "MnOverhaulDepotNorthText",
            ],
            _ => &[],
        };
        assert_eq!(additions, expected, "only reviewed phase-specific replies");
        for (label, pages) in labels {
            let before = crate::authored_dialogue::compile(&pages).expect("frozen native pages");
            assert_eq!(
                map.scripts.get(&label),
                Some(&before),
                "authentic frozen source {name}/{label}"
            );
            let after = crate::authored_dialogue::compile(&new.texts[&name][&label])
                .expect("current reviewed pages");
            changes += usize::from(before != after);
            map.scripts.insert(label, after);
        }
        for label in additions {
            map.scripts.insert(
                label.to_owned(),
                crate::authored_dialogue::compile(&new.texts[&name][label])
                    .expect("reviewed completed-chief reply"),
            );
        }
        super::publish(&mut pack, map)
            .expect("only recompile the reviewed text on the frozen actors and programs");
    }
    assert_eq!(changes, 94, "reviewed full-story pacing changes");
    crate::northstar_dispatch_revision::install(&mut pack)
        .expect("reviewed physical desk-copy handoff");
    let mut officer = pack.data.maps["MnBemidji"].clone();
    crate::northstar_report_revision::install(&mut officer).expect("reviewed comparison staging");
    super::publish(&mut pack, officer).expect("native report staging mirrors");
    crate::northstar_story_pacing::install(&mut pack)
        .expect("reviewed evidence acceptance and completed-chief routing");
    pack
}
