//! Compare reviewed dialogue with actual production pages and input ownership.
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{VisibleShellController, visible_field_dialog_pages};
use super::duluth_story::press;

pub(super) fn timely_evidence(runtime: &CrystalRuntime) -> bool {
    runtime.data().maps["MnLakeItasca"]
        .scripts
        .contains_key("MnOverhaulItascaCompared")
}

pub(super) fn canonical_pages(id: &str, label: &str) -> Vec<String> {
    let path = std::env::var_os("CRYSTAL_NORTHSTAR_CANONICAL_STORY_SOURCE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/story.json")
        });
    let story: Value = serde_json::from_str(&std::fs::read_to_string(path).expect("story source"))
        .expect("canonical JSON");
    if let Some(pages) = story["dialogue_overhaul"]["texts"]
        .as_object()
        .and_then(|maps| maps.values().find_map(|texts| texts[label].as_array()))
    {
        return pages
            .iter()
            .map(|page| {
                page.as_array()
                    .expect("authored page")
                    .iter()
                    .map(|line| line.as_str().expect("native line"))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect();
    }
    let milestone = story["milestones"]
        .as_array()
        .expect("milestones")
        .iter()
        .find(|m| m["id"] == id)
        .expect("canonical scene");
    let lines = &milestone["native_texts"][label];
    if lines.is_null() {
        return milestone["dialogue_pages"]
            .as_array()
            .expect("canonical pages")
            .iter()
            .map(|p| {
                p.as_array()
                    .expect("native page")
                    .iter()
                    .map(|l| l.as_str().expect("line"))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect();
    }
    lines
        .as_array()
        .expect("canonical native lines")
        .chunks(2)
        .map(|page| {
            page.iter()
                .map(|l| l.as_str().expect("native line"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect()
}

pub(super) fn acknowledge(c: &mut VisibleShellController, id: &str, label: &str, event: &str) {
    let expected = canonical_pages(id, label);
    let raw = c
        .shell
        .shell
        .presentation_snapshot()
        .expect("native stream");
    assert_eq!(
        visible_field_dialog_pages(&raw, &c.shell).expect("native pages"),
        expected
    );
    for page in expected {
        assert!(
            !c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(event)
                .expect("unacknowledged event")
        );
        let s = c.snapshot().expect("visible page");
        let text = s.ui.text.expect("canonical native text");
        assert_eq!(text.label, label);
        assert_eq!(text.asm_text, Some(page));
        let at = (s.overworld.map_name, s.overworld.tile);
        press(c, GameButton::Down);
        let blocked = c.snapshot().expect("dialogue owns input");
        assert_eq!((blocked.overworld.map_name, blocked.overworld.tile), at);
        assert!(
            !c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(event)
                .expect("movement grants no progress")
        );
        press(c, GameButton::A);
    }
}
