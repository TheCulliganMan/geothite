//! The Field Station aide uses the regional caught count and the actual river search receipt.
use std::path::{Path, PathBuf};

use crystal_core::input::GameButton::{A, Down, Up};
use crystal_core::models::KEY_ITEM_POCKET_CAPACITY;
use crystal_core::models::item::ITEM_POCKET_KEY_ITEM;
use crystal_core::world::map::Direction;

use super::super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::super::badge_trails::load;
use super::super::brainerd_guide::capture;
use super::super::duluth_story::press;
use super::super::interior_life::fixture;
use super::super::minnesota_field_use::{close_itemfinder, scan_with_itemfinder};
use super::super::minnesota_gyms::walk_to;
use super::super::r10_gallery::{continue_save, settle};
use super::super::winona::travel;

const STATION: &str = "MnWinonaFieldStation";
const FINDER: &str = "ITEMFINDER";
const RECEIPT: &str = "EVENT_MN_DEX_REWARD_1";
const BANK: &str = "EVENT_MN_HIDDEN_WINONA";
const HINT: &str = "MnRewardFinderBankText";
const FULL: &str = "MnRewardAideFullText";
const OFFER: &str = "MnRewardDexClaim1Text";

fn has(c: &VisibleShellController, flag: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("saved receipt")
}

fn count(c: &VisibleShellController, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&c.shell.shell.runtime().data().items[item])
}

fn visit(
    mut c: VisibleShellController,
    directory: Option<&Path>,
) -> (VisibleShellController, Vec<String>) {
    press(&mut c, Down);
    assert_eq!(
        c.shell
            .shell
            .current_overworld_interaction_checked()
            .expect("face aide")
            .expect("aide")
            .script,
        "MnKindlitResearchAide"
    );
    press(&mut c, A);
    let mut seen = Vec::new();
    let mut previous = None;
    for _ in 0..160 {
        let snapshot = c.snapshot().expect("aide's native page");
        if let Some(text) = snapshot.ui.text {
            let current = (text.label.clone(), text.asm_text);
            if previous.as_ref() != Some(&current) {
                if [HINT, FULL, OFFER, "MnRewardDexClaim1ClaimReceipt3Text"]
                    .contains(&text.label.as_str())
                {
                    c = capture(
                        c,
                        directory,
                        &format!("{:02}-{}.png", seen.len(), text.label),
                    );
                }
                seen.push(text.label);
                previous = Some(current);
            }
        }
        if !snapshot.ui.text_window_open
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.special_boundary.is_none()
        {
            return (c, seen);
        }
        press(&mut c, A);
    }
    panic!("Field Station aide did not release input: {seen:?}");
}

#[test]
#[ignore = "requires the revised Field Station aide in MN_BADGE_TRAILS_PACK"]
fn winona_itemfinder_aide_awards_once_retries_and_remembers_the_river_ball() {
    let (root, runtime) = load();
    let output = std::env::var_os("MN_ITEMFINDER_FRAMES").map(PathBuf::from);
    let catches = runtime
        .data()
        .story_event_script_constants
        .global
        .keys()
        .filter_map(|name| name.strip_prefix("MN_DEX_MEMBER_"))
        .take(25)
        .collect::<Vec<_>>();
    assert_eq!(catches.len(), 25);
    for found_ball in [false, true] {
        let mut c = if found_ball {
            fixture(&root, &runtime, "MnWinona", (24, 8))
        } else {
            fixture(&root, &runtime, STATION, (2, 8))
        };
        let state = c.shell.shell.session_mut().state_mut();
        for flag in [
            "EVENT_MN_STARTER_RECEIVED",
            "EVENT_MN_WINONA_BALLS_RECEIVED",
        ] {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("opening completed");
        }
        // The research count is made of real regional Dex entries. The 25th registration comes later.
        for id in &catches[..24] {
            c.shell
                .shell
                .record_pokedex_caught(id)
                .expect("regional caught entry");
        }
        if found_ball {
            let at = c.shell.shell.session_mut().overworld_mut();
            at.set_player_facing(Direction::Right);
            mark_runtime_snapshot_dirty(&mut c.shell);
            assert_eq!(
                c.shell
                    .shell
                    .current_overworld_interaction_checked()
                    .expect("bank")
                    .expect("ball")
                    .script,
                "MN_HIDDEN_WINONA"
            );
            c = capture(c, output.as_deref(), "winona-river-ball-approach.png");
            press(&mut c, A);
            settle(&mut c, "found the river ball");
            assert!(has(&c, BANK));
            assert_eq!(count(&c, "POKE_BALL"), 1);
            walk_to(&mut c, &runtime, (18, 20));
            press(&mut c, Up);
            if c.shell.shell.session().overworld().map.name != STATION {
                press(&mut c, Up);
            }
            assert_eq!(c.shell.shell.session().overworld().map.name, STATION);
            walk_to(&mut c, &runtime, (2, 8));
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        let (c, first) = visit(c, None);
        assert!(first.iter().any(|label| label == "MnRewardDexClaim0Text"));
        assert_eq!(count(&c, "GREAT_BALL"), 10);
        let (mut c, too_soon) = visit(c, None);
        assert!(too_soon.iter().any(|label| label == "MnRewardDexNeedText"));
        assert_eq!(count(&c, FINDER), 0);
        c.shell
            .shell
            .record_pokedex_caught(catches[24])
            .expect("25th regional catch");
        assert!(!has(&c, RECEIPT));
        assert_eq!(has(&c, BANK), found_ball);
        if !found_ball {
            // There are only 24 other named keys. The inventory supports additional one-item
            // stacks of the same key. Use one duplicate for this capacity stress case.
            let state = c.shell.shell.session_mut().state_mut();
            for item in
                runtime.data().items.values().filter(|item| {
                    item.pocket == ITEM_POCKET_KEY_ITEM && item.script_name != FINDER
                })
            {
                assert!(state.bag.add_item(item, 1).expect("fill legal key stack"));
            }
            let key = &runtime.data().items["SQUIRTBOTTLE"];
            assert!(state.bag.add_item(key, 1).expect("one duplicate key stack"));
            assert_eq!(state.bag.key_items.len(), KEY_ITEM_POCKET_CAPACITY);
            mark_runtime_snapshot_dirty(&mut c.shell);
            let dir = output.as_ref().map(|base| base.join("refusal"));
            let (next, refused) = visit(c, dir.as_deref());
            c = next;
            assert!(refused.iter().any(|label| label == FULL));
            assert!(!refused.iter().any(|label| label == HINT));
            assert!(!has(&c, RECEIPT));
            assert_eq!(count(&c, FINDER), 0);
            c = continue_save(root.clone(), runtime.clone(), &mut c);
            assert!(
                c.shell
                    .shell
                    .session_mut()
                    .state_mut()
                    .bag
                    .remove_item(key, 1)
                    .expect("free key slot")
            );
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        let dir = output.as_ref().map(|base| {
            base.join(if found_ball {
                "already-found"
            } else {
                "unclaimed"
            })
        });
        let (mut c, granted) = visit(c, dir.as_deref());
        assert!(granted.iter().any(|label| label == OFFER));
        assert_eq!(
            granted.iter().any(|label| label == HINT),
            !found_ball,
            "{granted:?}"
        );
        assert!(has(&c, RECEIPT));
        assert_eq!(count(&c, FINDER), 1);
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        let (mut c, repeat) = visit(c, None);
        assert!(!repeat.iter().any(|l| l == HINT || l == OFFER || l == FULL));
        assert!(has(&c, RECEIPT));
        assert_eq!(count(&c, FINDER), 1);
        if !found_ball {
            walk_to(&mut c, &runtime, (5, 10));
            travel(&mut c, Down, "MnWinona", (18, 20));
            walk_to(&mut c, &runtime, (23, 8));
            travel(
                &mut c,
                crystal_core::input::GameButton::Right,
                "MnWinona",
                (24, 8),
            );
            c = capture(c, output.as_deref(), "winona-finder-earned-at-river.png");
            let before = count(&c, "POKE_BALL");
            scan_with_itemfinder(&mut c, true);
            c = capture(c, output.as_deref(), "winona-finder-river-nearby.png");
            close_itemfinder(&mut c);
            assert!(!has(&c, BANK));
            assert_eq!(count(&c, "POKE_BALL"), before);
            assert_eq!(
                c.shell
                    .shell
                    .current_overworld_interaction_checked()
                    .expect("search the clue")
                    .expect("river spot")
                    .script,
                "MN_HIDDEN_WINONA"
            );
            press(&mut c, A);
            settle(&mut c, "collect the ball marked by the detector");
            assert!(has(&c, BANK));
            assert_eq!(count(&c, "POKE_BALL"), before + 1);
            c = continue_save(root.clone(), runtime.clone(), &mut c);
            assert!(has(&c, BANK));
            scan_with_itemfinder(&mut c, false);
            c = capture(c, output.as_deref(), "winona-finder-river-empty.png");
            close_itemfinder(&mut c);
            press(&mut c, A);
            settle(&mut c, "repeat a spent river search");
            assert_eq!(count(&c, "POKE_BALL"), before + 1);
            assert_eq!(count(&c, FINDER), 1);
        }
    }
}
