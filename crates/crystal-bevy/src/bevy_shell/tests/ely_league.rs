//! Explicit prior-access/party fixtures; all League progress uses production input.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};

use super::super::super::{
    FieldPackAction, FieldPackPocket, FieldPackTargetMode, StartMenuOption,
    visible_selected_pack_item_actions, visible_start_menu_options,
};

use super::super::winona::{at, continued, travel};
use super::leech_shore::{fixture, play_with_selected_move};
use super::north_shore_story::choice;
use super::{
    VisibleShellController, event, finish, interact, mark_runtime_snapshot_dirty, press, walk_to,
};

const ACTIVE: &str = "EVENT_MN_LEAGUE_RUN_ACTIVE";
const CHAMPION: &str = "EVENT_MN_LEAGUE_CHAMPION";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const CLINIC: &str = "MnElyPokecenter1F";

pub(super) fn cursor(
    c: &mut VisibleShellController,
    target: usize,
    read: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..64 {
        let index = read(c);
        if index == target {
            return;
        }
        press(
            c,
            if index < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("native menu did not reach item {target}");
}

// Initial medicine is an explicit fixture. Consumption, healing/status changes
// and every Pack cursor below use production input between matches.
fn restore_partner(c: &mut VisibleShellController, r: &crystal_runtime::CrystalRuntime) {
    restore_partner_with(c, r, "FULL_RESTORE", 0);
}

pub(super) fn restore_partner_with(
    c: &mut VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
    id: &str,
    slot: usize,
) {
    let before = c.snapshot().expect("pre-match partner");
    let p = &before.party.slots[slot].pokemon;
    let needed = match id {
        "FULL_RESTORE" => p.hp < p.max_hp || p.status.is_some(),
        "MAX_POTION" | "HYPER_POTION" => p.hp < p.max_hp,
        "FULL_HEAL" => p.status.is_some(),
        "REVIVE" => p.hp == 0,
        _ => panic!("unsupported carried League medicine {id}"),
    };
    if !needed {
        return;
    }
    let item = &r.data().items[id];
    let quantity = c.shell.shell.session().state().bag.quantity(item);
    let count = run(c);
    assert!(quantity > 0, "carried medicine must remain available");
    press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("Start menu");
    let target = visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("PACK");
    cursor(c, target, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    press(c, GameButton::A);
    let target = c
        .snapshot()
        .expect("carried medicine")
        .bag
        .items
        .iter()
        .filter(|i| i.quantity > 0)
        .position(|i| i.item_id == id)
        .expect("actual medicine stack");
    cursor(c, target, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("Pack cursor")
            .option_index
    });
    press(c, GameButton::A);
    let snapshot = c.snapshot().expect("item action menu");
    let target =
        visible_selected_pack_item_actions(&snapshot, &c.shell, &FieldPackPocket::Items, false)
            .expect("medicine actions")
            .iter()
            .position(|a| *a == FieldPackAction::Use)
            .expect("USE");
    cursor(c, target, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index
    });
    press(c, GameButton::A);
    assert_eq!(
        c.shell.field_pack_target_mode,
        Some(FieldPackTargetMode::PartyPokemon)
    );
    cursor(c, slot, |c| c.shell.party_cursor);
    press(c, GameButton::A);
    let healed = c.snapshot().expect("native medicine effect");
    let partner = &healed.party.slots[slot].pokemon;
    match id {
        "REVIVE" => assert!(partner.hp > 0),
        "FULL_HEAL" => assert!(partner.status.is_none()),
        "MAX_POTION" => assert_eq!(partner.hp, partner.max_hp),
        "HYPER_POTION" => assert_eq!(partner.hp, p.hp.saturating_add(200).min(p.max_hp)),
        "FULL_RESTORE" => {
            assert_eq!(partner.hp, partner.max_hp);
            assert!(partner.status.is_none());
        }
        _ => unreachable!("medicine validated before input"),
    }
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        quantity - 1
    );
    assert_eq!(run(c), count, "medicine never advances League progress");
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("close medicine surface");
        if c.shell.field_notice.is_some() || snapshot.ui.pending_text_wait.is_some() {
            c.wait_frames(60).expect("medicine notice");
            press(c, GameButton::A);
        } else if snapshot.ui.menu.is_some()
            || c.shell.field_pack_target_mode.is_some()
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.bag_cursor.is_some()
            || c.shell.field_pack_pocket.is_some()
            || c.shell.start_menu_cursor.is_some()
        {
            press(c, GameButton::B);
        } else {
            return;
        }
    }
    panic!("native medicine did not release input");
}

#[test]
#[ignore = "external Ely pack; explicit original credits-marker fixtures"]
fn ely_league_controller_original_credits_destinations_remain_vanilla() {
    for (marker, map) in [
        ("SPAWN_LANCE", "NewBarkTown"),
        ("SPAWN_RED", "SilverCaveOutside"),
    ] {
        let (root, r, _, mut c) = fixture("PlayersHouse2F", [3, 3], 0, true);
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .hall_of_fame
            .spawn_after_champion = Some(
            u16::try_from(r.data().story_event_script_constants.global[marker])
                .expect("native marker"),
        );
        let path = std::env::temp_dir().join(format!(
            "ely-original-{marker}-{}.crystalsave",
            std::process::id()
        ));
        c.save(&path).expect("save original marker");
        let mut loaded = VisibleShellController::load_save(root, r, path.clone(), None)
            .expect("direct native Continue");
        assert_eq!(
            loaded
                .snapshot()
                .expect("original destination")
                .overworld
                .map_name,
            map
        );
        assert!(
            loaded
                .shell
                .shell
                .session()
                .state()
                .hall_of_fame
                .spawn_after_champion
                .is_none()
        );
        std::fs::remove_file(path).expect("remove marker fixture");
    }
}

#[test]
#[ignore = "external Ely pack; native indoor/cave palette regression"]
fn ely_league_controller_hall_uses_indoor_bank_and_ice_path_retains_cave_bank() {
    let (_, _, _, c) = fixture("MnLeagueHallOfFame", [4, 12], 8, true);
    let root = &c.shell.asset_root;
    let common = std::fs::read_to_string(root.runtime_assets().join("gfx/tilesets/bg_tiles.pal"))
        .expect("native common palette source");
    let cave = std::fs::read_to_string(root.runtime_assets().join("gfx/tilesets/ice_path.pal"))
        .expect("native Ice Path palette source");
    let load = super::super::super::load_tileset_palette_bank;
    let parse = super::super::super::parse_palette_file;
    assert_eq!(
        load(root, "ice_path", "indoor")
            .expect("indoor bank")
            .expect("bank"),
        parse(&common, Some("indoor")).expect("native indoor RGB")
    );
    assert_eq!(
        load(root, "ice_path", "day")
            .expect("cave bank")
            .expect("bank"),
        parse(&cave, None).expect("native Ice Path RGB")
    );
}

fn prior(c: &mut VisibleShellController, restored: bool) {
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag(RESTORED, restored)
        .expect("explicit prior survey fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
}
pub(super) fn run(c: &VisibleShellController) -> u8 {
    c.shell
        .shell
        .session()
        .state()
        .script_runtime
        .memory
        .get("wMnLeagueRun")
        .map_or(0, |s| s.parse().expect("native run count"))
}
pub(super) fn registrar(c: &mut VisibleShellController, r: &crystal_runtime::CrystalRuntime) {
    walk_to(c, r, (5, 4));
    press(c, GameButton::Up);
    press(c, GameButton::A);
}
pub(super) fn enter_lobby(c: &mut VisibleShellController, r: &crystal_runtime::CrystalRuntime) {
    walk_to(c, r, (25, 14));
    travel(c, GameButton::Up, "MnLeagueLobby", (1, 4));
}
pub(super) fn until_map(c: &mut VisibleShellController, button: GameButton, map: &str) {
    for _ in 0..16 {
        if c.snapshot().expect("transition").overworld.map_name == map {
            return;
        }
        press(c, button);
    }
    panic!("native transition did not reach {map}");
}

#[test]
#[ignore = "external Ely pack; prior badges/survey/partner are explicit fixtures"]
fn ely_league_controller_seam_services_and_independent_registration() {
    for (badges, restored, denial) in [(7, true, "eight badges"), (8, false, "survey link")] {
        let (_, r, _, mut c) = fixture("MnR13C00R02", [34, 30], badges, false);
        prior(&mut c, restored);
        travel(&mut c, GameButton::Down, "MnEly", (24, 0));
        enter_lobby(&mut c, &r);
        registrar(&mut c, &r);
        assert!(
            c.snapshot()
                .expect("registration refusal")
                .ui
                .text
                .and_then(|t| t.asm_text)
                .is_some_and(|t| t
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_ascii_lowercase()
                    .contains(denial))
        );
        finish(&mut c);
        assert!(!event(&c, ACTIVE) && run(&c) == 0);
        walk_to(&mut c, &r, (1, 4));
        until_map(&mut c, GameButton::Left, "MnEly");
        at(&mut c, "MnEly", (25, 14));
        walk_to(&mut c, &r, (24, 0));
        travel(&mut c, GameButton::Up, "MnR13C00R02", (34, 31));
    }
    let (root, r, _, mut c) = fixture("MnEly", [24, 1], 0, false);
    {
        let state = c.shell.shell.session_mut().state_mut();
        state.money = 5000;
        let p = state.storage.party.pokemon[0]
            .as_mut()
            .expect("care fixture");
        p.hp = 1;
        p.moves[0].current_pp = 0;
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk_to(&mut c, &r, (11, 22));
    travel(&mut c, GameButton::Up, CLINIC, (3, 5));
    walk_to(&mut c, &r, (3, 3));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    let s = c.snapshot().expect("free native healing");
    assert_eq!(s.party.slots[0].pokemon.hp, s.party.slots[0].pokemon.max_hp);
    assert!(
        s.party.slots[0]
            .pokemon
            .moves
            .iter()
            .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    );
    assert_eq!(c.shell.shell.session().state().money, 5000);
    walk_to(&mut c, &r, (8, 1));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    for _ in 0..64 {
        if c.shell.pc_hub_cursor.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(
        c.shell.pc_hub_cursor.is_some(),
        "native PC access outside registration"
    );
    let standing = c.snapshot().expect("PC input ownership").overworld.tile;
    press(&mut c, GameButton::Left);
    assert_eq!(
        c.snapshot().expect("PC owns input").overworld.tile,
        standing
    );
    for _ in 0..64 {
        if c.shell.pc_hub_cursor.is_none()
            && c.shell.special_boundary.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(&mut c, GameButton::B);
    }
    c = continued(c, &root, &r, "ely-free-care");
    walk_to(&mut c, &r, (3, 6));
    until_map(&mut c, GameButton::Down, "MnEly");
    walk_to(&mut c, &r, (27, 28));
    travel(&mut c, GameButton::Up, "MnElyMart", (2, 5));
    walk_to(&mut c, &r, (3, 3));
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
    for _ in 0..32 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(c.shell.shop_top_cursor.is_some());
    let shop = c
        .snapshot()
        .expect("native Ely stock")
        .pending_shop
        .expect("shop");
    assert_eq!(
        shop.inventory,
        [
            "ULTRA_BALL",
            "HYPER_POTION",
            "MAX_POTION",
            "FULL_HEAL",
            "REVIVE",
            "MAX_REPEL"
        ]
    );
    let item = &r.data().items["ULTRA_BALL"];
    let quantity = c.shell.shell.session().state().bag.quantity(item);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell
            .shop_quantity
            .as_ref()
            .expect("native quantity")
            .item_id,
        "ULTRA_BALL"
    );
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        5000,
        "confirmation has not charged yet"
    );
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        5000 - u32::from(item.price)
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        quantity + 1
    );
    for _ in 0..64 {
        if c.snapshot().expect("shop closes").pending_shop.is_none() && !c.has_pending_script_work()
        {
            break;
        }
        press(&mut c, GameButton::B);
    }
    assert!(!event(&c, ACTIVE) && !event(&c, RESTORED) && !event(&c, CHAMPION));
    assert_eq!(
        c.shell.shell.session().state().money,
        5000 - u32::from(item.price)
    );
}

#[test]
#[ignore = "external Ely pack; prior badges/restoration/Mew100/five medicines fixture; actual five battles and Hall of Fame"]
fn ely_league_controller_five_wins_native_hall_of_fame_and_postgame_continue() {
    let (root, r, _, mut c) = fixture("MnEly", [24, 1], 8, true);
    prior(&mut c, true);
    let mut partner = r
        .data()
        .create_pokemon("MEW", 100, Dv::default())
        .expect("explicit battle partner fixture");
    partner.hp = partner.max_hp - 10;
    partner.moves = ["PSYCHIC_M", "FLAMETHROWER", "SURF", "THUNDERBOLT"]
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: r.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    assert!(
        state
            .bag
            .add_item(&r.data().items["FULL_RESTORE"], 5)
            .expect("initial carried medicine fixture")
    );
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    enter_lobby(&mut c, &r);
    registrar(&mut c, &r);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, ACTIVE) && run(&c) == 0);
    registrar(&mut c, &r);
    choice(&mut c, true);
    finish(&mut c);
    at(&mut c, "MnLeagueRoom1", (4, 16));
    assert!(event(&c, ACTIVE));
    c = continued(c, &root, &r, "ely-registered");
    for index in 1..=5 {
        let map = if index == 5 {
            "MnLeagueChampion".into()
        } else {
            format!("MnLeagueRoom{index}")
        };
        assert_eq!(
            c.snapshot()
                .expect("sequential League room")
                .overworld
                .map_name,
            map
        );
        assert_eq!(run(&c), index - 1);
        restore_partner(&mut c, &r);
        walk_to(&mut c, &r, (4, 4));
        if index < 5 {
            press(&mut c, GameButton::Up);
            let here = c.snapshot().expect("closed exit").overworld.tile;
            press(&mut c, GameButton::Up);
            assert_eq!(
                c.snapshot()
                    .expect("exit blocks skipping battle")
                    .overworld
                    .tile,
                here
            );
        }
        interact(&mut c, &r, (5, 7));
        play_with_selected_move(&mut c, |b| match b.enemy_pokemon.species.id.as_str() {
            "ARIADOS" | "VENOMOTH" | "PINSIR" | "SCYTHER" | "HERACROSS" => "FLAMETHROWER",
            "GRAVELER" | "ONIX" | "SUDOWOODO" | "GOLEM" | "HOUNDOOM" => "SURF",
            "NOCTOWL" | "GYARADOS" | "SEADRA" | "KINGDRA" | "LAPRAS" => "THUNDERBOLT",
            _ => "PSYCHIC_M",
        });
        finish(&mut c);
        assert_eq!(run(&c), index, "only actual win advances the run");
        assert!(!event(&c, CHAMPION), "Hall of Fame recording is separate");
        let money = c.shell.shell.session().state().money;
        interact(&mut c, &r, (5, 7));
        finish(&mut c);
        assert_eq!(
            c.shell.shell.session().state().money,
            money,
            "repeat cannot award battle money"
        );
        c = continued(c, &root, &r, &format!("ely-win-{index}"));
        walk_to(&mut c, &r, if index == 5 { (4, 2) } else { (4, 3) });
        let next = if index == 5 {
            "MnLeagueHallOfFame".into()
        } else if index == 4 {
            "MnLeagueChampion".into()
        } else {
            format!("MnLeagueRoom{}", index + 1)
        };
        until_map(&mut c, GameButton::Up, &next);
    }
    assert_eq!(run(&c), 5);
    let path = std::env::var_os("CRYSTAL_ELY_HOF_SAVE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("ely-hof-{}.crystalsave", std::process::id()))
        });
    c.shell.quick_save_path = Some(path.clone());
    let team = c.shell.shell.session().state().storage.party.clone();
    interact(&mut c, &r, (5, 5));
    for _ in 0..128 {
        if c.shell.credits_screen.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(
        c.shell
            .credits_screen
            .as_ref()
            .is_some_and(|s| s.hall_of_fame.is_some())
    );
    assert!(event(&c, CHAMPION) && !event(&c, ACTIVE) && run(&c) == 0);
    assert_eq!(c.shell.shell.session().state().hall_of_fame.count, 1);
    assert!(
        path.is_file(),
        "native ceremony autosaves the winning party"
    );
    for _ in 0..2048 {
        if c.shell.credits_screen.is_none() {
            break;
        }
        c.wait_frames(64)
            .expect("native ceremony and credits clock");
        press(&mut c, GameButton::A);
    }
    assert!(c.shell.credits_screen.is_none(), "credits return to title");
    let mut c = VisibleShellController::load_save(root.clone(), r.clone(), path, None)
        .expect("postgame Continue");
    at(&mut c, CLINIC, (3, 5));
    let state = c.shell.shell.session().state();
    assert!(event(&c, CHAMPION) && event(&c, RESTORED) && !event(&c, ACTIVE));
    assert_eq!(state.storage.party, team);
    assert_eq!(state.hall_of_fame.count, 1);
    assert!(state.hall_of_fame.spawn_after_champion.is_none());
    walk_to(&mut c, &r, (3, 6));
    until_map(&mut c, GameButton::Down, "MnEly");
    enter_lobby(&mut c, &r);
    registrar(&mut c, &r);
    choice(&mut c, true);
    finish(&mut c);
    at(&mut c, "MnLeagueRoom1", (4, 16));
    assert!(
        event(&c, ACTIVE) && event(&c, CHAMPION) && run(&c) == 0,
        "rematch registration preserves past championship"
    );
    walk_to(&mut c, &r, (5, 16));
    until_map(&mut c, GameButton::Down, "MnLeagueLobby");
    assert!(
        !event(&c, ACTIVE) && run(&c) == 0,
        "leaving ends the rematch run"
    );
}

#[test]
#[ignore = "external Ely pack; weak partner/prior badges/survey fixture; actual native loss"]
fn ely_league_controller_loss_resets_run_and_preserves_story() {
    let (root, r, _, mut c) = fixture("MnEly", [24, 1], 8, true);
    prior(&mut c, true);
    let mut p = r
        .data()
        .create_pokemon("MAGIKARP", 5, Dv::default())
        .expect("weak legal loss fixture");
    p.hp = 1;
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(p));
    state.money = 5000;
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    enter_lobby(&mut c, &r);
    registrar(&mut c, &r);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, ACTIVE));
    interact(&mut c, &r, (5, 7));
    super::battle(&mut c, true);
    finish(&mut c);
    at(&mut c, CLINIC, (3, 5));
    assert!(!event(&c, ACTIVE) && run(&c) == 0 && !event(&c, CHAMPION) && event(&c, RESTORED));
    assert_eq!(c.shell.shell.session().state().badges.johto, [true; 8]);
    assert!(c.shell.shell.session().state().money < 5000);
    c = continued(c, &root, &r, "ely-blackout");
    assert!(!event(&c, ACTIVE) && run(&c) == 0 && event(&c, RESTORED));
}
