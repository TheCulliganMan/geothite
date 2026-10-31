//! Explicit local party/badge/capacity setup; every discovery uses native input.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::LearnedMove;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::ITEM_POCKET_ITEM;
use crystal_core::world::map::Direction;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::super::{
    FieldPackPocket, PartyFieldMove, VisibleShellController, mark_runtime_snapshot_dirty,
};
use super::super::r10_gallery::continue_save;
use super::super::winona::{at, press};
use super::{event, finish, flash, teaching, use_move, walk};

const ROOM: &str = "MnDriftlessLampGallery";
const HOLLOW: &str = "MnR01Cave1F";
const OUTSIDE: &str = "MnR01C00R00";
const READ: &str = "EVENT_MN_LAMP_GALLERY_READ";
const CLAIMED: &str = "EVENT_MN_LAMP_GALLERY_CACHE";

#[derive(Default)]
struct Setup {
    badge: bool,
    learned_flash: bool,
    full_pocket: bool,
}

fn source() -> Value {
    let path = std::env::var_os("CRYSTAL_DRIFTLESS_GALLERY_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/caves/driftless-fossil-gallery-v01/gallery.json")
        });
    serde_json::from_str(&std::fs::read_to_string(path).expect("frozen published gallery source"))
        .expect("gallery source")
}
fn point(v: &Value) -> (i16, i16) {
    (
        v[0].as_i64().expect("x") as i16,
        v[1].as_i64().expect("y") as i16,
    )
}
fn fixture(
    map: &str,
    at: (i16, i16),
    setup: Setup,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = super::load();
    let mut c = super::fixture(&root, &r, map, at);
    let mut partner = r
        .data()
        .create_pokemon("BEETYNAMO", 40, Default::default())
        .expect("active Minnesota Flash learner");
    assert!(partner.species.tmhm_learnset.iter().any(|m| m == "FLASH"));
    partner.moves.retain(|m| m.name != "FLASH");
    partner.moves.truncate(3);
    if setup.learned_flash {
        partner.moves.push(LearnedMove {
            name: "FLASH".into(),
            current_pp: r.data().moves["FLASH"].pp,
            pp_ups: 0,
        });
    }
    partner.item = Some("SMOKE_BALL".into());
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.badges.johto[0] = setup.badge;
    state
        .flags
        .set_engine_flag("ENGINE_ZEPHYRBADGE", setup.badge)
        .expect("declared prior Care Badge fixture");
    if setup.full_pocket {
        for (_, item) in r
            .data()
            .items
            .iter()
            .filter(|(id, item)| item.pocket == ITEM_POCKET_ITEM && id.as_str() != "RARE_CANDY")
            .take(ITEM_POCKET_CAPACITY)
        {
            assert!(
                state
                    .bag
                    .add_item(item, 1)
                    .expect("declared capacity fixture")
            );
        }
    }
    state.money = 1000;
    // Initial setup ends here. No later party, position, bag, flag or Flash writes.
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!flash(&c) && !event(&c, READ) && !event(&c, CLAIMED));
    let s = source();
    let m = &r.data().maps[ROOM];
    assert_eq!([m.attributes.width, m.attributes.height], [12, 10]);
    assert_eq!(m.attributes.time_of_day.as_deref(), Some("dark"));
    assert_eq!(m.attributes.phone_service & 0x0f, 4);
    assert_eq!(
        r.data().runtime_map_metadata["MN_DRIFTLESS_LAMP_GALLERY"].phone_service & 0x0f,
        4
    );
    let blocks = s["rows"]
        .as_array()
        .expect("complete rows")
        .iter()
        .flat_map(|r| r.as_str().expect("row").split_whitespace())
        .map(|v| u16::from_str_radix(v, 16).expect("native cave block"))
        .collect::<Vec<_>>();
    assert_eq!(m.blocks, blocks);
    (root, r, c)
}
fn checkpoint(c: &mut VisibleShellController, name: &str) -> PathBuf {
    let out = PathBuf::from(
        std::env::var_os("CRYSTAL_DRIFTLESS_GALLERY_REVIEW_DIR")
            .expect("fresh gallery review output"),
    );
    std::fs::create_dir_all(&out).expect("checkpoint directory");
    let path = out.join(format!("{name}.crystalsave"));
    assert!(!path.exists(), "retain every gallery checkpoint");
    c.save(&path).expect("actual reached gallery save");
    path
}
fn view(c: &mut VisibleShellController, name: &str) {
    let s = c.snapshot().expect("actual camera state");
    let pos = (s.overworld.tile.x, s.overworld.tile.y);
    if s.overworld.facing != Direction::Down {
        press(c, GameButton::Down);
    }
    let now = c.snapshot().expect("camera turn");
    assert_eq!(
        (now.overworld.tile.x, now.overworld.tile.y),
        pos,
        "turning for review must not move the player"
    );
    checkpoint(c, name);
}
fn visible_cache(c: &mut VisibleShellController) -> bool {
    c.snapshot()
        .expect("real object visibility")
        .visible_objects
        .iter()
        .any(|o| o.script == "MN_LAMP_GALLERY_CACHE")
}
fn quantity(c: &VisibleShellController, r: &CrystalRuntime, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[item])
}
fn enter_hollow(c: &mut VisibleShellController, r: &CrystalRuntime) {
    at(c, OUTSIDE, (18, 18));
    super::super::winona::travel(c, GameButton::Up, HOLLOW, (9, 11));
    walk(c, r, (9, 10));
}
fn enter_gallery(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (11, 8));
    super::super::winona::travel(c, GameButton::Down, ROOM, (18, 17));
    c.wait_frames(60)
        .expect("objects callback and passive arrival");
    at(c, ROOM, (18, 17));
}
fn leave_gallery(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (19, 16));
    super::super::winona::travel(c, GameButton::Down, HOLLOW, (10, 9));
    c.wait_frames(60).expect("native passive hollow return");
    at(c, HOLLOW, (10, 9));
}
fn leave_world(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (9, 10));
    super::super::winona::travel(c, GameButton::Down, OUTSIDE, (18, 18));
    c.wait_frames(60).expect("original passive world return");
    at(c, OUTSIDE, (18, 18));
}
fn read(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (5, 4));
    press(c, GameButton::Right);
    press(c, GameButton::A);
}
fn cache(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (18, 9));
    press(c, GameButton::Right);
    press(c, GameButton::A);
    finish(c);
}
fn acknowledge_fossils(c: &mut VisibleShellController) {
    let mut pages = BTreeMap::new();
    for _ in 0..64 {
        c.wait_frames(240).expect("fully print each fossil page");
        if event(c, READ) {
            break;
        }
        if let Some((index, page)) = super::super::r10_gallery::completed_visible_page(c) {
            let snapshot = c.snapshot().expect("native fossil page");
            assert_eq!(
                snapshot.ui.text.as_ref().map(|t| t.label.as_str()),
                Some("MnLampFossilsText")
            );
            pages.insert(index, page);
        }
        press(c, GameButton::A);
    }
    let source = source();
    for (index, pair) in source["read_pages"]
        .as_array()
        .expect("authored fossil pages")
        .iter()
        .enumerate()
    {
        let expected = pair
            .as_array()
            .expect("two lines")
            .iter()
            .map(|line| line.as_str().expect("native text line"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            pages.get(&index),
            Some(&expected),
            "all fossil pages, including the east-shelf direction, must be visible before acknowledgment"
        );
    }
    assert!(event(c, READ));
    finish(c);
}

fn blocked(c: &mut VisibleShellController, b: GameButton, at_tile: (i16, i16)) {
    for _ in 0..4 {
        press(c, b);
        at(c, ROOM, at_tile);
    }
}

#[test]
#[ignore = "requires the external fossil-gallery pack and fresh review output; local badge/partner only"]
fn fossil_gallery_controller_hm_gift_teaching_held_fossils_reward_and_original_return() {
    let (root, r, mut c) = fixture(
        OUTSIDE,
        (18, 18),
        Setup {
            badge: true,
            ..Default::default()
        },
    );
    checkpoint(&mut c, "initial-care-badge-return");
    enter_hollow(&mut c, &r);
    walk(&mut c, &r, (5, 6));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(
        quantity(&c, &r, "HM_FLASH"),
        1,
        "original native guide gift"
    );
    teaching::teach(&mut c, "HM_FLASH", "FLASH");
    walk(&mut c, &r, (10, 5));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(quantity(&c, &r, "ANTIDOTE"), 1);
    assert!(event(&c, "EVENT_MN_R01_ITEM_03"));
    enter_gallery(&mut c, &r);
    assert!(!visible_cache(&mut c));
    view(&mut c, "dark-entry");
    read(&mut c, &r);
    finish(&mut c);
    assert!(!event(&c, READ) && !event(&c, CLAIMED));
    view(&mut c, "dark-fossil");
    let moves = c.snapshot().expect("before field Flash").party.slots[0]
        .pokemon
        .moves
        .clone();
    use_move(&mut c, PartyFieldMove::Flash);
    finish(&mut c);
    assert!(flash(&c));
    assert_eq!(
        c.snapshot()
            .expect("native field move preserves PP")
            .party
            .slots[0]
            .pokemon
            .moves,
        moves
    );
    assert_eq!(quantity(&c, &r, "HM_FLASH"), 1);
    view(&mut c, "lit-before-read");
    read(&mut c, &r);
    c.wait_frames(240).expect("print held fossil page");
    assert!(!event(&c, READ));
    press(&mut c, GameButton::Left);
    at(&mut c, ROOM, (5, 4));
    let held = checkpoint(&mut c, "held-fossil-page");
    c = VisibleShellController::load_save(root.clone(), r.clone(), held, None)
        .expect("Continue held native page");
    assert!(!event(&c, READ));
    acknowledge_fossils(&mut c);
    assert!(event(&c, READ) && !event(&c, CLAIMED));
    assert!(visible_cache(&mut c));
    for camera in source()["review_contract"]["viewpoints"]
        .as_array()
        .expect("whole-gallery cameras")
    {
        let name = camera["id"].as_str().expect("camera id");
        walk(&mut c, &r, point(&camera["tile"]));
        view(&mut c, &format!("lit-{name}"));
    }
    cache(&mut c, &r);
    assert!(event(&c, CLAIMED));
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 1);
    assert!(!visible_cache(&mut c));
    checkpoint(&mut c, "cache-claimed");
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(event(&c, READ) && event(&c, CLAIMED));
    walk(&mut c, &r, (19, 9));
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 1);
    leave_gallery(&mut c, &r);
    assert!(flash(&c));
    leave_world(&mut c, &r);
    assert!(!flash(&c));
    checkpoint(&mut c, "original-world-return");
    enter_hollow(&mut c, &r);
    enter_gallery(&mut c, &r);
    assert!(!flash(&c) && !visible_cache(&mut c));
    assert!(event(&c, READ) && event(&c, CLAIMED));
    assert_eq!(quantity(&c, &r, "ANTIDOTE"), 1);
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 1);
    checkpoint(&mut c, "claimed-dark-reentry");
}

#[test]
#[ignore = "requires the external gallery and fresh output; explicit known Flash without its badge"]
fn fossil_gallery_controller_dark_badge_denial_both_stairs_and_passive_pair() {
    let (root, r, mut c) = fixture(
        ROOM,
        (18, 17),
        Setup {
            learned_flash: true,
            ..Default::default()
        },
    );
    checkpoint(&mut c, "initial-no-badge");
    use_move(&mut c, PartyFieldMove::Flash);
    finish(&mut c);
    assert!(!flash(&c));
    read(&mut c, &r);
    finish(&mut c);
    assert!(!event(&c, READ) && !visible_cache(&mut c));
    let source = source();
    for stair in source["review_contract"]["stairs"]
        .as_array()
        .expect("native fitted stairs")
    {
        let path = std::iter::once(point(&stair["before"]))
            .chain(
                stair["lane"]
                    .as_array()
                    .expect("stair lane")
                    .iter()
                    .map(point),
            )
            .chain(std::iter::once(point(&stair["after"])))
            .collect::<Vec<_>>();
        for p in path.iter().chain(path.iter().rev()) {
            walk(&mut c, &r, *p);
            at(&mut c, ROOM, *p);
        }
        let from = point(&stair["blocked_from"]);
        let to = point(&stair["blocked_target"]);
        walk(&mut c, &r, from);
        let b = super::super::authored_route_fixture::neighbors(from)
            .into_iter()
            .find_map(|(at, b)| (at == to).then_some(b))
            .expect("adjacent sidewall");
        blocked(&mut c, b, from);
    }
    for v in source["review_contract"]["viewpoints"]
        .as_array()
        .expect("all standable cameras")
    {
        walk(&mut c, &r, point(&v["tile"]));
    }
    c = continue_save(root, r.clone(), &mut c);
    assert!(!flash(&c) && !event(&c, READ));
    leave_gallery(&mut c, &r);
    leave_world(&mut c, &r);
    assert!(!event(&c, CLAIMED));
    checkpoint(&mut c, "no-badge-original-return");
}

#[test]
#[ignore = "requires external gallery, full native item-pocket fixture and fresh output"]
fn fossil_gallery_controller_full_pocket_claim_survives_native_toss_retry_and_continue() {
    let (root, r, mut c) = fixture(
        ROOM,
        (18, 17),
        Setup {
            badge: true,
            learned_flash: true,
            full_pocket: true,
        },
    );
    checkpoint(&mut c, "initial-full-pocket");
    let before = c.snapshot().expect("full item pocket").bag;
    assert_eq!(before.items.len(), ITEM_POCKET_CAPACITY);
    use_move(&mut c, PartyFieldMove::Flash);
    finish(&mut c);
    read(&mut c, &r);
    finish(&mut c);
    assert!(event(&c, READ) && visible_cache(&mut c));
    cache(&mut c, &r);
    assert!(!event(&c, CLAIMED) && visible_cache(&mut c));
    assert_eq!(
        c.snapshot().expect("refused award preserves bag").bag,
        before
    );
    checkpoint(&mut c, "full-pocket-refused");
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(!event(&c, CLAIMED));
    super::super::northstar_progression_rewards::native_toss_first_pocket_item(
        &mut c,
        &FieldPackPocket::Items,
    );
    for _ in 0..8 {
        press(&mut c, GameButton::B);
    }
    assert_eq!(
        c.snapshot()
            .expect("native Toss frees one slot")
            .bag
            .items
            .len(),
        ITEM_POCKET_CAPACITY - 1
    );
    cache(&mut c, &r);
    assert!(event(&c, CLAIMED));
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 1);
    c = continue_save(root, r.clone(), &mut c);
    assert!(event(&c, CLAIMED));
    assert!(!visible_cache(&mut c));
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 1);
    checkpoint(&mut c, "full-pocket-retry-continued");
    leave_gallery(&mut c, &r);
    leave_world(&mut c, &r);
}
