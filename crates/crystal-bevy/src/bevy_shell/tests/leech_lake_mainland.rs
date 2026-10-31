//! Declared no-Surf local setup; subsequent mainland actions use native input.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::ITEM_POCKET_ITEM;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::{
    FieldPackAction, FieldPackPocket, StartMenuOption, VisibleShellController, at, checkpoint,
    choice, event, finish, mark_runtime_snapshot_dirty, press, quantity, select, talk,
    visible_selected_pack_item_actions, visible_start_menu_options, walk,
};

const TRAINER: &str = "MN_EXTRA_LEECH_MAINLAND_FLY_TIER";
const WON: &str = "EVENT_MN_EXTRA_LEECH_MAINLAND_FLY_TIER";
const CLAIMED: &str = "EVENT_MN_LEECH_MAINLAND_REPEL";

fn setup(full: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime, mut c) = super::fixture((11, 14), true, false, false, false);
    let state = c.shell.shell.session_mut().state_mut();
    for mon in state.storage.party.pokemon.iter_mut().flatten() {
        mon.moves
            .retain(|m| m.name != "SURF" && m.name != "WHIRLPOOL");
    }
    state.sync_party_from_storage();
    if full {
        state.bag.items.clear();
        for (_, item) in runtime
            .data()
            .items
            .iter()
            .filter(|(id, item)| item.pocket == ITEM_POCKET_ITEM && id.as_str() != "SUPER_REPEL")
            .take(ITEM_POCKET_CAPACITY)
        {
            assert!(
                state
                    .bag
                    .add_item(item, 1)
                    .expect("declared full item pocket")
            );
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Setup ends before the first checkpoint/input. No later state writes.
    (root, runtime, c)
}

#[test]
#[ignore = "requires current mainland pack and CRYSTAL_LEECH_ISLAND_REVIEW_DIR"]
fn leech_mainland_controller_walks_no_surf_grass_link_and_real_wildlife() {
    let (_, r, mut c) = setup(false);
    checkpoint(&mut c, "mainland-no-surf-initial");
    assert!(
        c.snapshot()
            .expect("declared no-Surf party")
            .party
            .slots
            .iter()
            .all(|slot| slot.pokemon.moves.iter().all(|m| m.name != "SURF"))
    );
    walk(&mut c, &r, (16, 22));
    let mut encountered = false;
    for i in 0..128 {
        let (button, target) = if i % 2 == 0 {
            (GameButton::Right, (17, 22))
        } else {
            (GameButton::Left, (16, 22))
        };
        for _ in 0..8 {
            let s = c.snapshot().expect("native trail step");
            if s.battle.is_some() || (s.overworld.tile.x, s.overworld.tile.y) == target {
                break;
            }
            press(&mut c, button);
            c.wait_frames(16).expect("grass presentation");
        }
        if let Some(battle) = c.snapshot().expect("native trail wildlife").battle {
            assert!(
                [
                    "PATCHEBEE",
                    "SKEETEREX",
                    "HATCHRUSH",
                    "VELVAWN",
                    "SHEDSTAG",
                    "MOTHOSAIC"
                ]
                .contains(&battle.enemy_pokemon.species.id.as_str())
            );
            checkpoint(&mut c, "mainland-wild-battle");
            super::super::super::rochester::flee(&mut c);
            encountered = true;
            break;
        }
    }
    assert!(
        encountered,
        "new mainland grass must produce real regional wildlife"
    );
    walk(&mut c, &r, (13, 28));
    checkpoint(&mut c, "mainland-south-trail");
    walk(&mut c, &r, (9, 36));
    checkpoint(&mut c, "mainland-fisher-porch");
    walk(&mut c, &r, (11, 14));
    assert_eq!(
        c.snapshot().expect("walked cabin return").overworld.mode,
        MovementMode::Normal
    );
    checkpoint(&mut c, "mainland-walked-care-return");
}

fn use_reward(c: &mut VisibleShellController) {
    press(c, GameButton::Start);
    let s = c.snapshot().expect("Start");
    let pack = visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("PACK");
    select(c, pack, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    for _ in 0..4 {
        if c.shell.field_pack_pocket == Some(FieldPackPocket::Items) {
            break;
        }
        press(c, GameButton::Right);
    }
    let s = c.snapshot().expect("native item pocket");
    let reward = s
        .bag
        .items
        .iter()
        .position(|i| i.item_id == "SUPER_REPEL")
        .expect("earned reward");
    select(c, reward, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("item cursor")
            .option_index
    });
    let s = c.snapshot().expect("reward actions");
    let use_action =
        visible_selected_pack_item_actions(&s, &c.shell, &FieldPackPocket::Items, false)
            .expect("item actions")
            .iter()
            .position(|a| *a == FieldPackAction::Use)
            .expect("USE");
    select(c, use_action, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index
    });
    finish(c);
}

#[test]
#[ignore = "requires current mainland pack and CRYSTAL_LEECH_ISLAND_REVIEW_DIR"]
fn leech_mainland_controller_declines_then_wins_fly_tier_claims_uses_and_continues() {
    let (root, r, mut c) = setup(false);
    walk(&mut c, &r, (18, 24));
    checkpoint(&mut c, "mainland-fly-tier-approach");
    let before = c.snapshot().expect("before optional battle").party;
    talk(&mut c, GameButton::Right);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, WON));
    assert_eq!(c.snapshot().expect("declined party").party, before);
    talk(&mut c, GameButton::Right);
    choice(&mut c, true);
    assert!(super::super::super::rochester_earned::try_battle(
        &mut c, &r, TRAINER, WON
    ));
    finish(&mut c);
    assert!(event(&c, WON));
    assert!(c.shell.shell.session().state().money > 5000);
    let after = c.snapshot().expect("real native victory").party;
    assert!(
        after
            .slots
            .iter()
            .any(|p| before.slots.iter().any(|old| old.index == p.index
                && p.pokemon.moves.iter().any(|m| old
                    .pokemon
                    .moves
                    .iter()
                    .any(|prior| prior.name == m.name && m.current_pp < prior.current_pp))))
    );
    walk(&mut c, &r, (18, 26));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(event(&c, CLAIMED));
    assert_eq!(quantity(&c, &r, "SUPER_REPEL"), 1);
    let save = checkpoint(&mut c, "mainland-reward-and-victory");
    c = VisibleShellController::load_save(root, r.clone(), save, None).expect("mainland Continue");
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, WON) && event(&c, CLAIMED));
    assert_eq!(quantity(&c, &r, "SUPER_REPEL"), 1);
    use_reward(&mut c);
    assert_eq!(quantity(&c, &r, "SUPER_REPEL"), 0);
    assert_eq!(c.shell.shell.session().state().repel_steps_remaining, 200);
    assert_eq!(
        c.shell.shell.session().state().active_repel_item.as_deref(),
        Some("SUPER_REPEL")
    );
    checkpoint(&mut c, "mainland-native-repel-used");
}

#[test]
#[ignore = "requires current mainland pack and CRYSTAL_LEECH_ISLAND_REVIEW_DIR"]
fn leech_mainland_controller_full_pocket_refuses_then_native_toss_retries_once() {
    let (root, r, mut c) = setup(true);
    walk(&mut c, &r, (18, 26));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(!event(&c, CLAIMED));
    assert_eq!(quantity(&c, &r, "SUPER_REPEL"), 0);
    let save = checkpoint(&mut c, "mainland-full-pocket-refused");
    c = VisibleShellController::load_save(root, r.clone(), save, None).expect("capacity Continue");
    super::super::super::northstar_progression_rewards::native_toss_first_pocket_item(
        &mut c,
        &FieldPackPocket::Items,
    );
    for _ in 0..8 {
        press(&mut c, GameButton::B);
    }
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(event(&c, CLAIMED));
    assert_eq!(quantity(&c, &r, "SUPER_REPEL"), 1);
    checkpoint(&mut c, "mainland-capacity-retry");
    at(&mut c, "MnLeechLake", (18, 26));
}
