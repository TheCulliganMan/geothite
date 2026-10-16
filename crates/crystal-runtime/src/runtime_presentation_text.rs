//! Canonical presentation lookup for authored pure TX_FAR text dependencies.
//! Keep the catalog's original labels/bodies and retain only compact aliases.
use std::collections::{BTreeMap, BTreeSet};
use anyhow::{Context, Result, ensure};
use crystal_assets::GameDataSet;
use crate::RuntimePresentationCatalogSnapshot;

pub(super) fn asm_text_aliases(data: &GameDataSet) -> BTreeMap<String, String> {
    data.global_scripts.iter().flat_map(|module| &module.definitions)
        .filter_map(|(label, body)| {
            let commands = body.as_array()?;
            if commands.len()!=2
                || commands[0].get("command")?.as_str()? != "text_far"
                || commands[1].get("command")?.as_str()? != "text_end"
                || !commands[1].get("args")?.as_array()?.is_empty() { return None; }
            let args = commands[0].get("args")?.as_array()?;
            if args.len()!=1 {return None;}
            let target = args[0].as_str()?;
            if target.is_empty() || target.trim()!=target {return None;}
            Some((label.clone(),target.to_owned()))
        }).collect()
}

impl RuntimePresentationCatalogSnapshot {
    /// Resolve an exported literal or a bounded chain of pure authored aliases.
    /// This reads no game state and never executes event/script instructions.
    pub fn asm_text_for_label(&self, label: &str) -> Result<&str> {
        let mut current = label;
        let mut visited = BTreeSet::new();
        for _ in 0..8 {
            ensure!(visited.insert(current), "presentation text TX_FAR cycle at {current}");
            if let Some(text) = self.asm_text.get(current) { return Ok(text); }
            current = self.asm_text_aliases.get(current).map(String::as_str)
                .with_context(|| format!("ASM text label {current} is missing from the presentation catalog (requested {label})"))?;
        }
        anyhow::bail!("presentation text TX_FAR chain exceeds eight bodies for {label}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn data() -> GameDataSet {
        let mut data=GameDataSet::default();
        data.asm_text.insert("_CanCutText".into(),"This tree can be\nCUT!".into());
        data.global_scripts=Some(Default::default());
        data
    }
    fn alias(data:&mut GameDataSet,label:&str,target:&str) {
        data.global_scripts.as_mut().unwrap().definitions.insert(label.into(),json!([
            {"command":"text_far","args":[target]}, {"command":"text_end","args":[]}
        ]));
    }
    #[test]
    fn presentation_resolves_exact_alias_closure_without_flattening_or_mutation() {
        let mut data=data();
        alias(&mut data,"CanCutText","_CanCutText");
        alias(&mut data,"NestedCutText","CanCutText");
        let snapshot=RuntimePresentationCatalogSnapshot::from_data(&data);
        assert!(!snapshot.asm_text.contains_key("CanCutText"));
        assert_eq!(snapshot.asm_text_aliases["CanCutText"],"_CanCutText");
        let before=snapshot.clone();
        for label in ["NestedCutText","CanCutText","_CanCutText"] {
            assert_eq!(snapshot.asm_text_for_label(label).unwrap(),"This tree can be\nCUT!");
        }
        assert_eq!(snapshot,before);
        // A literal authored under the same label always keeps precedence.
        data.asm_text.insert("CanCutText".into(),"Literal".into());
        assert_eq!(RuntimePresentationCatalogSnapshot::from_data(&data)
            .asm_text_for_label("CanCutText").unwrap(),"Literal");
    }
    #[test]
    fn presentation_rejects_cycles_missing_targets_and_nonpure_programs() {
        let mut data=data();
        alias(&mut data,"CycleA","CycleB");alias(&mut data,"CycleB","CycleA");
        alias(&mut data,"Missing","Absent");
        for (label,body) in [
            ("Executable",json!([{"command":"writetext","args":["_CanCutText"]},{"command":"end","args":[]} ])),
            ("Extra",json!([{"command":"text_far","args":["_CanCutText"]},{"command":"text_end","args":[]},{"command":"end","args":[]} ])),
            ("Malformed",json!([{"command":"text_far","args":[" _CanCutText"]},{"command":"text_end","args":[]} ])),
        ] {data.global_scripts.as_mut().unwrap().definitions.insert(label.into(),body);}
        let snapshot=RuntimePresentationCatalogSnapshot::from_data(&data);
        assert!(snapshot.asm_text_for_label("CycleA").unwrap_err().to_string().contains("cycle"));
        for label in ["Missing","Executable","Extra","Malformed"] {
            assert!(snapshot.asm_text_for_label(label).is_err(),"{label}");
        }
        for i in 0..9 {alias(&mut data,&format!("Long{i}"),&format!("Long{}",i+1));}
        assert!(RuntimePresentationCatalogSnapshot::from_data(&data).asm_text_for_label("Long0")
            .unwrap_err().to_string().contains("exceeds eight"));
    }
    #[test]
    #[ignore="requires completed external crystalpack via CRYSTAL_RENDER_TEST_PACK"]
    fn packed_presentation_resolves_cut_denial_and_field_move_wrappers() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack path");
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(std::path::Path::new(&path)).unwrap();
        let data=loaded.pack().data();
        let snapshot=RuntimePresentationCatalogSnapshot::from_data(data);
        for (label,target) in [("CanCutText","_CanCutText"),("AskCutText","_AskCutText"),
            ("AskSurfText","_AskSurfText"),("AskWhirlpoolText","_AskWhirlpoolText"),
            (".AskWaterfallText@Script_AskWaterfall","_AskWaterfallText"),
            (".HugeWaterfallText@Script_CantDoWaterfall","_HugeWaterfallText"),
            (".MayPassWhirlpoolText@Script_MightyWhirlpool","_MayPassWhirlpoolText"),
            (".CantGetOffBikeText@Script_CantGetOffBike","_CantGetOffBikeText"),
            (".UseEscapeRopeText@EscapeRopeOrDig","_UseEscapeRopeText"),
            (".TeleportReturnText@TeleportFunction","_TeleportReturnText")] {
            assert!(!snapshot.asm_text.contains_key(label),"authored aliases stay unflattened");
            assert_eq!(snapshot.asm_text_for_label(label).unwrap(),data.asm_text[target]);
        }
        assert_eq!(snapshot.asm_text_for_label("CanCutText").unwrap(),"This tree can be\nCUT!");
        assert_eq!(snapshot.asm_text_for_label("_MayRegisterItemText").unwrap(),data.asm_text["_MayRegisterItemText"]);
        // Canonical item text exists independently of engine event wrappers.
        for label in ["_ItemUsedText","_RepelUsedEarlierIsStillInEffectText","_SentTrophyHomeText"] {
            assert!(snapshot.asm_text.contains_key(label));
        }
    }
}
