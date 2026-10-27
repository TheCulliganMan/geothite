//! Local position/party/inventory/RNG fixtures followed exclusively by the
//! production controller. These cases do not earn regional mine/story access.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::{ITEM_POCKET_ITEM, is_mail_item_id};
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, indoor_walk};

const ROOM: &str = "MnR10Cave1F";
const OUTSIDE: &str = "MnR10C00R00";
const EVENT: &str = "EVENT_MN_R10_ITEM_04";

fn load() -> (AssetRoot, CrystalRuntime) {
    // Only immutable native data is reused; controllers and sessions stay fresh.
    // The pack environment path is captured once for this test process.
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_R10_GALLERY_CONTROLLER_PACK")
                    .expect("actual revised native R10 pack"),
            );
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path)
                    .expect("verified revised gallery pack"),
            )
            .expect("native runtime");
            (root, runtime)
        })
        .clone()
}
fn fixture(
    map: &str,
    at: Point,
    full_bag: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let m = &runtime.data().maps[ROOM];
    assert_eq!([m.attributes.width, m.attributes.height], [18, 14]);
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: map.into(),
            tile_x: at.0,
            tile_y: at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local gallery fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    if full_bag {
        let mut pokemon = runtime
            .data()
            .create_pokemon("MEW", 30, Dv::default())
            .expect("legal injured medicine fixture partner");
        pokemon.hp = pokemon.max_hp - 40;
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
        state.bag.items.clear();
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["POTION"], 1)
                .expect("initial usable medicine")
        );
        for (id, item) in runtime
            .data()
            .items
            .iter()
            .filter(|(id, item)| {
                item.pocket == ITEM_POCKET_ITEM
                    && !["POTION", "REVIVE"].contains(&id.as_str())
                    && !is_mail_item_id(id)
            })
            .take(ITEM_POCKET_CAPACITY - 1)
        {
            assert!(
                state
                    .bag
                    .add_item(item, 1)
                    .unwrap_or_else(|e| panic!("initial filler{id}: {e}"))
            );
        }
        assert_eq!(state.bag.items.len(), ITEM_POCKET_CAPACITY);
    }
    let state = c.shell.shell.session_mut().state_mut();
    state.money = 1000;
    state.random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Initial placement, empty party or one injured Mew30, no earned badges,
    // native inventory and deterministic RNG end here. No later state writes.
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(tile(&mut c), at);
    assert!(!collected(&c));
    (root, runtime, c)
}
pub(super) fn tile(c: &mut VisibleShellController) -> Point {
    let t = c.snapshot().expect("native location").overworld.tile;
    (t.x, t.y)
}
pub(super) fn map(c: &mut VisibleShellController) -> String {
    c.snapshot().expect("native map").overworld.map_name
}
pub(super) fn flags(c: &VisibleShellController) -> (BTreeSet<String>, BTreeSet<String>) {
    let f = &c.shell.shell.session().state().flags;
    (
        f.active_event_flags().cloned().collect(),
        f.engine_flags
            .iter()
            .filter_map(|(k, v)| v.then_some(k.clone()))
            .collect(),
    )
}
fn collected(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(EVENT)
        .expect("original pickup event")
}
fn quantity(c: &VisibleShellController, runtime: &CrystalRuntime, id: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items[id])
}
pub(super) fn press(c: &mut VisibleShellController, b: GameButton) {
    c.press(b).expect("production gallery input");
    c.wait_frames(16).expect("native presentation frames");
    assert_eq!(c.shell.last_error, None);
}
pub(super) fn settle(c: &mut VisibleShellController, context: &str) {
    let mut diagnostics = VecDeque::new();
    for _ in 0..128 {
        remember_dialogue(c, &mut diagnostics);
        let s = c.snapshot().expect("native input ownership");
        assert!(s.battle.is_none(), "{context}: {diagnostics:#?}");
        if s.phase == RuntimeShellPhase::Overworld
            && s.ui.pending_text_wait.is_none()
            && !s.ui.text_window_open
            && super::super::visible_field_dialog_pages(&s, &c.shell).is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
        {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("gallery text/notice did not release input during {context}: {diagnostics:#?}");
}
fn step(c: &mut VisibleShellController, button: GameButton, to: Point) {
    for _ in 0..8 {
        if tile(c) == to {
            return;
        }
        press(c, button);
        assert!(c.snapshot().expect("local traversal").battle.is_none());
    }
    assert_eq!(tile(c), to);
}
fn blocked(c: &mut VisibleShellController, button: GameButton) {
    let before = tile(c);
    for _ in 0..4 {
        press(c, button);
        assert_eq!(tile(c), before);
    }
}
pub(super) fn continue_save(
    root: AssetRoot,
    runtime: CrystalRuntime,
    c: &mut VisibleShellController,
) -> VisibleShellController {
    settle(c, "before Save/Continue");
    let before = c.snapshot().expect("pre-save native state");
    let before_flags = flags(c);
    let money = c.shell.shell.session().state().money;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r10-gallery-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    c.save(&path).expect("production save");
    let mut loaded = VisibleShellController::load_save(root, runtime, path.clone(), None)
        .expect("native Continue");
    let after = loaded.snapshot().expect("continued gallery");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(flags(&loaded), before_flags);
    assert_eq!(loaded.shell.shell.session().state().money, money);
    std::fs::remove_file(path).expect("remove temporary save");
    loaded
}
fn outside_grid(runtime: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR10",
            columns: 2,
            rows: 1,
            tileset: match runtime.data().maps["MnR10C00R00"]
                .attributes
                .tileset_name
                .as_str()
            {
                "johto" => "johto",
                "mn_r10_rockstairs" => "mn_r10_rockstairs",
                other => panic!("unsupported native R10 fixture tileset {other}"),
            },
            start: (36, 18),
            doors: BTreeMap::from([((36, 17), ROOM)]),
        },
    )
}
fn exit(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(c, runtime, (3, 12));
    indoor_walk(c, runtime, (3, 13));
    for _ in 0..4 {
        if map(c) == OUTSIDE {
            break;
        }
        press(c, GameButton::Down);
    }
    assert_eq!(map(c), OUTSIDE);
    assert_eq!(tile(c), (36, 18));
}
fn enter(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    let g = outside_grid(runtime);
    g.walk(c, (36, 17));
    assert_eq!(map(c), ROOM);
    assert_eq!(tile(c), (3, 13));
    indoor_walk(c, runtime, (3, 12));
    assert_eq!(tile(c), (3, 12));
}

#[test]
#[ignore = "requires actual pack with revised R10 seam gallery"]
fn r10_gallery_controller_original_mouth_passive_return_four_stairs_and_short_native_drop() {
    let (_, runtime, mut c) = fixture(OUTSIDE, (36, 18), false);
    enter(&mut c, &runtime);
    for (lane, ys, side) in [
        (7, [7, 8, 9, 10], GameButton::Left),
        (20, [5, 6, 7, 8], GameButton::Right),
        (20, [7, 8, 9, 10], GameButton::Right),
        (7, [13, 14, 15, 16], GameButton::Left),
    ] {
        for y in ys {
            indoor_walk(&mut c, &runtime, (lane, y));
            assert_eq!(tile(&mut c), (lane, y));
            assert_eq!(map(&mut c), ROOM);
        }
        indoor_walk(&mut c, &runtime, (lane, ys[2]));
        blocked(&mut c, side);
        for y in ys.into_iter().rev() {
            indoor_walk(&mut c, &runtime, (lane, y));
            assert_eq!(map(&mut c), ROOM);
        }
    }
    for (lane, expected_return_steps) in [(12, 13), (13, 15)] {
        indoor_walk(&mut c, &runtime, (lane, 5));
        step(&mut c, GameButton::Down, (lane, 6));
        for _ in 0..8 {
            if tile(&mut c) == (lane, 8) {
                break;
            }
            press(&mut c, GameButton::Down);
            assert_ne!(tile(&mut c), (lane, 7));
        }
        assert_eq!(tile(&mut c), (lane, 8));
        blocked(&mut c, GameButton::Up);
        let mut return_path = Vec::new();
        if lane == 13 {
            return_path.push((12, 8));
        }
        return_path.extend([
            (11, 8),
            (10, 8),
            (9, 8),
            (8, 8),
            (7, 8),
            (7, 7),
            (7, 6),
            (8, 6),
            (9, 6),
            (10, 6),
            (11, 6),
            (11, 5),
            (12, 5),
        ]);
        if lane == 13 {
            return_path.push((13, 5));
        }
        assert_eq!(return_path.len(), expected_return_steps);
        for p in return_path {
            let before = tile(&mut c);
            indoor_walk(&mut c, &runtime, p);
            let after = tile(&mut c);
            assert_eq!((before.0 - after.0).abs() + (before.1 - after.1).abs(), 1);
        }
    }
    exit(&mut c, &runtime);
    let g = outside_grid(&runtime);
    g.walk(&mut c, (36, 19));
    g.walk(&mut c, (36, 18));
    assert_eq!(map(&mut c), OUTSIDE);
    assert_eq!(tile(&mut c), (36, 18), "passive return must not reenter");
    enter(&mut c, &runtime);
    assert!(!collected(&c));
}

pub(super) fn completed_visible_page(c: &mut VisibleShellController) -> Option<(usize, String)> {
    // The public controller projects only the current page. The production
    // printer resolves its complete stream from the underlying presentation.
    let raw = c
        .shell
        .shell
        .presentation_snapshot()
        .expect("native printer source");
    let pages = super::super::visible_field_dialog_pages(&raw, &c.shell)?;
    let reveal = c.shell.field_text_reveal.as_ref()?;
    if reveal.text != pages.join("\u{1e}") {
        return None;
    }
    let page = pages.get(reveal.page_index)?;
    if !super::super::visible_field_text_reveal_is_complete(reveal, page) {
        return None;
    }
    assert_eq!(
        super::super::visible_revealed_field_dialog_text(&c.shell, page),
        *page
    );
    let index = reveal.page_index;
    let projected = c.snapshot().expect("public currently visible page");
    assert_eq!(
        projected
            .ui
            .text
            .as_ref()
            .and_then(|t| t.asm_text.as_deref()),
        Some(page.as_str())
    );
    Some((index, page.clone()))
}
pub(super) fn remember_dialogue(c: &mut VisibleShellController, history: &mut VecDeque<String>) {
    let s = c.snapshot().expect("bounded dialogue diagnostic");
    let raw = c
        .shell
        .shell
        .presentation_snapshot()
        .expect("raw printer diagnostic");
    let projected_pages = super::super::visible_field_dialog_pages(&s, &c.shell);
    let raw_pages = super::super::visible_field_dialog_pages(&raw, &c.shell);
    let page_branch = |pages: &Option<Vec<String>>| {
        let reveal = c.shell.field_text_reveal.as_ref();
        match (pages.as_ref(), reveal) {
            (Some(pages), Some(reveal)) => format!(
                "identity={} current={:?} page_len={:?} complete={:?}",
                reveal.text == pages.join("\u{1e}"),
                pages.get(reveal.page_index),
                pages.get(reveal.page_index).map(|p| p.chars().count()),
                pages
                    .get(reveal.page_index)
                    .map(|p| super::super::visible_field_text_reveal_is_complete(reveal, p)),
            ),
            _ => "pages or printer absent".to_string(),
        }
    };
    if history.len() == 12 {
        history.pop_front();
    }
    history.push_back(format!(
        "phase={:?} window={} wait={:?} label={:?} reveal={:?} notice={:?} item_phase={:?} pending={} menu={:?} pack_target={:?} field_action={:?} projected_pages={:?} projected_branch={} raw_pages={:?} raw_branch={}",
        s.phase,
        s.ui.text_window_open,
        s.ui.pending_text_wait,
        s.ui.text.as_ref().map(|t| &t.label),
        c.shell.field_text_reveal,
        c.shell.field_notice,
        c.shell.visible_field_item_notice.as_ref().map(|n| &n.phase),
        c.has_pending_script_work(),
        s.ui.menu,
        c.shell.field_pack_target_mode,
        c.shell.field_pack_action_cursor,
        projected_pages,
        page_branch(&projected_pages),
        raw_pages,
        page_branch(&raw_pages),
    ));
}
fn reader(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(c, runtime, (10, 5));
    press(c, GameButton::Left);
    assert_eq!(tile(c), (10, 5));
    press(c, GameButton::A);
    let before = tile(c);
    press(c, GameButton::Down);
    assert_eq!(tile(c), before, "original reader text owns movement");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        c.wait_frames(240).expect("print native reader page");
        remember_dialogue(c, &mut diagnostics);
        let snapshot = c.snapshot().expect("actual printed reader text");
        if let Some((index, page)) = completed_visible_page(c) {
            assert_eq!(
                snapshot.ui.text.as_ref().map(|t| t.label.as_str()),
                Some("MN_R10_GALLERY_READERText"),
                "{diagnostics:#?}"
            );
            pages.insert(index, page);
        }
        // jumptextfaceplayer can finish its script before its textbox closes.
        // The production printer/window, rather than sparse script work, owns A.
        if !snapshot.ui.text_window_open
            && super::super::visible_field_dialog_pages(&snapshot, &c.shell).is_none()
            && snapshot.ui.pending_text_wait.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(c, GameButton::A);
    }
    assert!(
        pages.len() >= 2,
        "A must advance at least two actually printed original pages: pages={pages:#?}; last observations={diagnostics:#?}"
    );
    settle(c, "after original reader closes");
    let actual = pages.into_values().collect::<Vec<_>>().join("\n");
    for line in [
        "Both passages",
        "circle the pillar.",
        "The foreman was",
        "at the west mine.",
    ] {
        assert!(
            actual.contains(line),
            "missing original page text {line:?}: {actual}"
        );
    }
}
fn cache(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(c, runtime, (28, 23));
    press(c, GameButton::Right);
    assert_eq!(
        tile(c),
        (28, 23),
        "uncollected native item occupies its real relocated tile"
    );
    press(c, GameButton::A);
}
fn finish_successful_cache(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    assert!(collected(c));
    assert_eq!(quantity(c, runtime, "REVIVE"), 1);
    let party = c
        .snapshot()
        .expect("party before pickup presentation")
        .party;
    let before = tile(c);
    press(c, GameButton::Down);
    assert_eq!(tile(c), before, "native item notice must own movement");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        c.wait_frames(240)
            .expect("print successful native pocket page");
        remember_dialogue(c, &mut diagnostics);
        if let Some((index, page)) = completed_visible_page(c) {
            let notice = c
                .shell
                .visible_field_item_notice
                .as_ref()
                .expect("successful native item-ball presentation");
            assert_eq!(
                notice.presentation,
                super::super::VisibleFieldItemPresentation::ItemBall,
                "{diagnostics:#?}"
            );
            assert_eq!(
                notice.phase,
                super::super::VisibleFieldItemPhase::PocketText,
                "native found text and fanfare must finish autonomously: {diagnostics:#?}"
            );
            pages.insert(index, page);
        }
        if c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(c, GameButton::A);
    }
    let item_name = runtime.data().items["REVIVE"].name.replace('_', " ");
    let expected = [
        format!("CHRIS put the\n{item_name} in"),
        format!("{item_name} in\nthe ITEM POCKET."),
    ];
    for (index, page) in expected.iter().enumerate() {
        assert_eq!(
            pages.get(&index),
            Some(page),
            "successful item-ball page {index}; observed={pages:#?}; last observations={diagnostics:#?}"
        );
    }
    settle(
        c,
        "after successful native pocket pages and acknowledgement",
    );
    assert!(c.shell.visible_field_item_notice.is_none());
    assert_eq!(
        c.snapshot()
            .expect("item notice preserves party and PP")
            .party,
        party
    );
    assert_eq!(
        tile(c),
        before,
        "acknowledging item text cannot move the player"
    );
    assert!(collected(c));
    assert_eq!(quantity(c, runtime, "REVIVE"), 1);
}
#[test]
#[ignore = "requires actual pack with revised R10 seam gallery"]
fn r10_gallery_controller_original_reader_pages_revive_once_and_semantic_continue() {
    let (root, runtime, mut c) = fixture(ROOM, (3, 12), false);
    let original_flags = flags(&c);
    reader(&mut c, &runtime);
    assert_eq!(
        flags(&c),
        original_flags,
        "reader must not grant mine/story progress"
    );
    assert_eq!(quantity(&c, &runtime, "REVIVE"), 0);
    cache(&mut c, &runtime);
    finish_successful_cache(&mut c, &runtime);
    assert!(collected(&c));
    assert_eq!(quantity(&c, &runtime, "REVIVE"), 1);
    exit(&mut c, &runtime);
    enter(&mut c, &runtime);
    indoor_walk(&mut c, &runtime, (28, 23));
    step(&mut c, GameButton::Right, (29, 23));
    press(&mut c, GameButton::A);
    settle(&mut c, "after no second pickup interaction");
    assert_eq!(quantity(&c, &runtime, "REVIVE"), 1);
    c = continue_save(root, runtime.clone(), &mut c);
    assert!(collected(&c));
    assert_eq!(quantity(&c, &runtime, "REVIVE"), 1);
    reader(&mut c, &runtime);
    assert_eq!(quantity(&c, &runtime, "REVIVE"), 1);
    assert_eq!(c.shell.shell.session().state().money, 1000);
}

fn consume_potion(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    settle(c, "before opening native Potion menu");
    press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("production Start");
    let target = super::super::visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|o| *o == super::super::StartMenuOption::Pack)
        .expect("native PACK option");
    for _ in 0..16 {
        let selected = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    let bag = c.snapshot().expect("full carried pocket").bag;
    let target = bag
        .items
        .iter()
        .filter(|s| s.quantity > 0)
        .position(|s| s.item_id == "POTION")
        .expect("fixture medicine still present");
    for _ in 0..32 {
        let selected = c
            .shell
            .bag_cursor
            .as_ref()
            .expect("production bag cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    let snapshot = c.snapshot().expect("medicine action menu");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &c.shell,
        &super::super::FieldPackPocket::Items,
        false,
    )
    .expect("native item actions");
    let target = actions
        .iter()
        .position(|a| *a == super::super::FieldPackAction::Use)
        .expect("USE Potion");
    for _ in 0..16 {
        let selected = c
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("native action cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    assert_eq!(
        c.shell.field_pack_target_mode,
        Some(super::super::FieldPackTargetMode::PartyPokemon)
    );
    for _ in 0..6 {
        if c.shell.party_cursor == 0 {
            break;
        }
        press(c, GameButton::Up);
    }
    let before = c.snapshot().expect("injured Mew30").party.slots[0]
        .pokemon
        .hp;
    press(c, GameButton::A);
    let after = c.snapshot().expect("actual Potion effect");
    assert_eq!(after.party.slots[0].pokemon.hp, before + 20);
    assert_eq!(quantity(c, runtime, "POTION"), 0);
    for _ in 0..32 {
        let s = c.snapshot().expect("close native Pack");
        if c.shell.field_notice.is_some() || s.ui.pending_text_wait.is_some() {
            c.wait_frames(60).expect("medicine notice clock");
            press(c, GameButton::A);
        } else if s.ui.menu.is_some() {
            press(c, GameButton::B);
        } else {
            settle(c, "after closing native Potion menus");
            return;
        }
    }
    panic!("native Potion menus did not return walking");
}
#[test]
#[ignore = "requires actual pack with revised R10 seam gallery"]
fn r10_gallery_controller_full_bag_refusal_native_potion_use_and_original_pickup_retry() {
    // Native pickup_field_item returns BagFull before setting its collection
    // event. Free the slot with a real medicine action; never rewrite capacity.
    let (root, runtime, mut c) = fixture(ROOM, (28, 23), true);
    let full = c.shell.shell.session().state().bag.clone();
    let before_party = c.snapshot().expect("injured full-pocket setup").party;
    cache(&mut c, &runtime);
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..64 {
        c.wait_frames(240)
            .expect("print original native item-ball notice");
        remember_dialogue(&mut c, &mut diagnostics);
        if let Some((index, page)) = completed_visible_page(&mut c) {
            let notice = c
                .shell
                .visible_field_item_notice
                .as_ref()
                .expect("printed native item-ball phase");
            assert_eq!(
                notice.presentation,
                super::super::VisibleFieldItemPresentation::ItemBall
            );
            let phase = match notice.phase {
                super::super::VisibleFieldItemPhase::BagFullFoundText => "found",
                super::super::VisibleFieldItemPhase::BagFullText => "refusal",
                _ => panic!("unexpected refused item-ball phase: {diagnostics:#?}"),
            };
            pages.insert((phase, index), page);
        }
        if c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    // Item balls say "can't carry any more items", unlike verbose grants'
    // "pocket is full". Require all actually printed native pages/phases.
    let found = format!(
        "CHRIS found\n{}!",
        runtime.data().items["REVIVE"].name.replace('_', " ")
    );
    for (key, expected) in [
        (("found", 0), found.as_str()),
        (("refusal", 0), "But CHRIS can't\ncarry any more"),
        (("refusal", 1), "carry any more\nitems."),
    ] {
        assert_eq!(
            pages.get(&key).map(String::as_str),
            Some(expected),
            "native refused item-ball page {key:?}; observed={pages:#?}; last observations={diagnostics:#?}"
        );
    }
    settle(&mut c, "after all printed full-bag refusal pages");
    assert_eq!(c.shell.shell.session().state().bag, full);
    assert_eq!(
        c.snapshot().expect("atomic refused pickup").party,
        before_party
    );
    assert!(!collected(&c));
    consume_potion(&mut c, &runtime);
    assert_eq!(
        c.shell.shell.session().state().bag.items.len(),
        ITEM_POCKET_CAPACITY - 1
    );
    assert!(!collected(&c));
    cache(&mut c, &runtime);
    finish_successful_cache(&mut c, &runtime);
    assert!(collected(&c));
    assert_eq!(quantity(&c, &runtime, "REVIVE"), 1);
    assert_eq!(quantity(&c, &runtime, "POTION"), 0);
    assert_eq!(
        c.shell.shell.session().state().bag.items.len(),
        ITEM_POCKET_CAPACITY
    );
    let mut loaded = continue_save(root, runtime.clone(), &mut c);
    assert!(collected(&loaded));
    assert_eq!(quantity(&loaded, &runtime, "REVIVE"), 1);
    step(&mut loaded, GameButton::Right, (29, 23));
    press(&mut loaded, GameButton::A);
    settle(
        &mut loaded,
        "after continued already-collected cache interaction",
    );
    assert_eq!(quantity(&loaded, &runtime, "REVIVE"), 1);
    assert_eq!(loaded.shell.shell.session().state().money, 1000);
}
