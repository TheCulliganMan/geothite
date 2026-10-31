//! Visit real public services with the earned party and native input only.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use crate::bevy_shell::VisibleShellController;
use crate::bevy_shell::tests::interior_life::{talk, target_name, walk as room_walk};
use crate::bevy_shell::tests::native_city_ports::city_port;
use crate::bevy_shell::tests::northstar_ice_lair::path;
use crate::bevy_shell::tests::northstar_progression_rewards::register_and_ride;
use crate::bevy_shell::tests::rochester_earned::try_battle;
use crate::bevy_shell::tests::winona::{continue_in_place, finish, press, travel};

use super::northstar_campaign::learn_machine_on;

type Point = (i16, i16);

struct Gift {
    room: &'static str,
    stand: Point,
    actor: Point,
    item: &'static str,
    checkpoint: &'static str,
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[item])
}

pub(super) fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: Point) {
    let name = c
        .snapshot()
        .expect("earned service approach")
        .overworld
        .map_name;
    for _ in 0..1024 {
        let snapshot = c.snapshot().expect("live public approach");
        if let Some(battle) = snapshot.battle {
            match battle.kind {
                RuntimeBattleKind::Trainer {
                    trainer_id,
                    event_flag,
                    ..
                } => {
                    assert!(
                        try_battle(c, r, &trainer_id, &event_flag),
                        "actual service approach battle lost"
                    );
                }
                _ => panic!("public service approach entered an unexpected wild battle"),
            }
            continue;
        }
        if snapshot.ui.text_window_open || c.has_pending_script_work() {
            finish(c);
            continue;
        }
        assert_eq!(
            snapshot.overworld.map_name, name,
            "public approach used an unintended door"
        );
        if (snapshot.overworld.tile.x, snapshot.overworld.tile.y) == to {
            return;
        }
        if let Some(button) = path(c, r, to).and_then(|steps| steps.first().copied()) {
            press(c, button);
        } else {
            c.wait_frames(24).expect("wait for a passing resident");
        }
    }
    panic!("earned service walk did not reach {name} {to:?}");
}

fn converse(c: &mut VisibleShellController, source: &Gift) {
    if source.item.starts_with("HM_") {
        // The giver resumes its original service after both gifts and repeats.
        crate::bevy_shell::tests::minnesota_hm_progression::talk(c, source.actor);
    } else {
        talk(c, source.actor);
    }
}

fn collect(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot, source: &Gift) {
    walk(c, r, source.stand);
    converse(c, source);
    assert_eq!(
        quantity(c, r, source.item),
        1,
        "actual once-only service gift"
    );
    continue_in_place(c, root, r, source.checkpoint);
    converse(c, source);
    assert_eq!(
        quantity(c, r, source.item),
        1,
        "native repeat after Continue must not duplicate a gift"
    );
}

fn enter_and_collect(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &AssetRoot,
    city: &str,
    source: &Gift,
) {
    assert_eq!(c.snapshot().expect("service city").overworld.map_name, city);
    let (door, arrival) = city_port(r, city, source.room);
    walk(c, r, door);
    travel(c, GameButton::Up, source.room, arrival);
    collect(c, r, root, source);
}

fn leave(c: &mut VisibleShellController, r: &CrystalRuntime, city: &str, room: &str) {
    let exit = r.data().maps[room]
        .events
        .warps
        .iter()
        .find(|warp| target_name(r, &warp.target_map_constant) == city)
        .expect("real public room return");
    room_walk(c, r, (exit.x as i16, exit.y as i16));
    for _ in 0..8 {
        if c.snapshot()
            .expect("native service return")
            .overworld
            .map_name
            == city
        {
            break;
        }
        press(c, GameButton::Down);
    }
    assert_eq!(
        c.snapshot()
            .expect("earned service returned outdoors")
            .overworld
            .map_name,
        city
    );
}

pub(super) fn minneapolis(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    let city = "MnMinneapolis";
    let bicycle = Gift {
        room: "MnMinneapolisToolWorkshop",
        stand: (2, 5),
        actor: (2, 4),
        item: "BICYCLE",
        checkpoint: "campaign-earned-workshop-bicycle",
    };
    enter_and_collect(c, r, root, city, &bicycle);
    if c.shell
        .shell
        .session()
        .state()
        .registered_key_item
        .as_deref()
        != Some("BICYCLE")
    {
        register_and_ride(c, r);
    } else {
        leave(c, r, city, bicycle.room);
        if c.shell.shell.session().overworld().player.mode != MovementMode::Bike {
            press(c, GameButton::Select);
        }
    }
    assert_eq!(
        c.shell.shell.session().overworld().player.mode,
        MovementMode::Bike
    );
    continue_in_place(c, root, r, "campaign-earned-bicycle-registered-riding");
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .registered_key_item
            .as_deref(),
        Some("BICYCLE")
    );
    assert_eq!(
        c.shell.shell.session().overworld().player.mode,
        MovementMode::Bike
    );
    press(c, GameButton::Select);
    assert_eq!(
        c.shell.shell.session().overworld().player.mode,
        MovementMode::Normal
    );
    let strength = Gift {
        room: "MnMinneapolisMillForemanHome",
        stand: (2, 5),
        actor: (2, 4),
        item: "HM_STRENGTH",
        checkpoint: "campaign-earned-mill-strength",
    };
    enter_and_collect(c, r, root, city, &strength);
    leave(c, r, city, strength.room);
    coin_case(c, r, root);
}

/// Collect the public greeter's real Coin Case and retain its native return.
pub(super) fn coin_case(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    let source = Gift {
        room: "MnMinneapolisCornerHome",
        stand: (2, 10),
        actor: (2, 9),
        item: "COIN_CASE",
        checkpoint: "campaign-earned-game-corner-coin-case",
    };
    enter_and_collect(c, r, root, "MnMinneapolis", &source);
    leave(c, r, "MnMinneapolis", source.room);
}

pub(super) fn st_paul(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    let source = Gift {
        room: "MnStPaulAnglerHouse",
        stand: (5, 4),
        actor: (5, 3),
        item: "HM_SURF",
        checkpoint: "campaign-earned-angler-surf",
    };
    enter_and_collect(c, r, root, "MnStPaul", &source);
    leave(c, r, "MnStPaul", source.room);
}

pub(super) fn st_cloud(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    assert_eq!(
        c.snapshot().expect("earned Wind gift").overworld.map_name,
        "MnStCloud"
    );
    let source = Gift {
        room: "MnStCloud",
        stand: (37, 14),
        actor: (37, 13),
        item: "HM_FLY",
        checkpoint: "campaign-earned-wind-fly",
    };
    collect(c, r, root, &source);
    let snapshot = c.snapshot().expect("actual carried Fly learners");
    if let Some(slot) = snapshot
        .party
        .slots
        .iter()
        .find(|slot| {
            slot.pokemon
                .species
                .tmhm_learnset
                .iter()
                .any(|name| name == "FLY")
        })
        .map(|slot| slot.index)
    {
        learn_machine_on(c, r, "HM_FLY", "FLY", "GROWL", slot);
        continue_in_place(c, root, r, "campaign-earned-carried-fly-learning");
        assert_eq!(quantity(c, r, "HM_FLY"), 1, "native HMs remain reusable");
    }
}

/// Collect the current-clearing machine after earning the Pine Badge.
pub(super) fn bemidji(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    assert_eq!(
        c.snapshot().expect("earned Pine city").overworld.map_name,
        "MnBemidji"
    );
    collect(
        c,
        r,
        root,
        &Gift {
            room: "MnBemidji",
            stand: (34, 36),
            actor: (34, 35),
            item: "HM_WHIRLPOOL",
            checkpoint: "campaign-earned-pine-whirlpool",
        },
    );
}

/// Visit the lake outfitter before leaving Duluth for the coast.
pub(super) fn duluth(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    let gift = Gift {
        room: "MnDuluthLakeOutfitter",
        stand: (5, 5),
        actor: (4, 5),
        item: "HM_WATERFALL",
        checkpoint: "campaign-earned-outfitter-waterfall",
    };
    enter_and_collect(c, r, root, "MnDuluth", &gift);
    leave(c, r, "MnDuluth", gift.room);
}

#[test]
#[ignore = "requires current Minnesota pack; declared room fixture, then native gift, public return and Continue"]
fn earned_coin_case_native_greeter_gift_public_return_repeat_and_continue() {
    use crate::bevy_shell::tests::winona::load;
    use crate::bevy_shell::{
        BevyShellConfig, BevyShellStart, complete_visible_smoke_player_name_if_needed,
        initialize_bevy_runtime_shell,
    };

    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnMinneapolisCornerHome".into(),
            tile_x: 3,
            tile_y: 12,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("declared public greeter fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    controller.set_runtime_journal_enabled(false);
    // Only the starting room/name are fixture setup. No item, money, flag,
    // progression, party or RNG writes occur; gift/reentry use native input.
    assert_eq!(quantity(&controller, &runtime, "COIN_CASE"), 0);
    let money = controller.shell.shell.session().state().money;
    let source = Gift {
        room: "MnMinneapolisCornerHome",
        stand: (2, 10),
        actor: (2, 9),
        item: "COIN_CASE",
        checkpoint: "coin-case-local-greeter",
    };
    collect(&mut controller, &runtime, &root, &source);
    let coins = controller.shell.shell.session().state().coins;
    assert!(
        coins > 0,
        "acknowledged welcome supplies native starter coins"
    );
    leave(&mut controller, &runtime, "MnMinneapolis", source.room);
    coin_case(&mut controller, &runtime, &root);
    assert_eq!(quantity(&controller, &runtime, "COIN_CASE"), 1);
    assert_eq!(controller.shell.shell.session().state().money, money);
    assert_eq!(controller.shell.shell.session().state().coins, coins);
    continue_in_place(
        &mut controller,
        &root,
        &runtime,
        "coin-case-local-public-return",
    );
    assert_eq!(quantity(&controller, &runtime, "COIN_CASE"), 1);
    assert_eq!(controller.shell.shell.session().state().coins, coins);
}
