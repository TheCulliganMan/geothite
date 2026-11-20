//! A three-win run is supplied. The conversation, medicine, save and battle use native input.
use std::path::{Path, PathBuf};

use bevy::prelude::{Assets, Image, Mut, Sprite, Transform, Vec2};
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton::{A, Down};
use crystal_core::models::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::authored_route_fixture::use_super_potion;
use super::super::brainerd_guide::capture;
use super::super::{menu_render_test_app, pokemon_overworld_context_canvas_scaled};
use super::{
    VisibleShellController, at, check_run, choice, continue_save, event, in_progress, interact,
    load, mark_runtime_snapshot_dirty, page, press, settle, travel,
};

const ROOM: &str = "MnLeagueRoom4";
const READY: &str = "MnHaldenReadyText";
const WAITED: &str = "EVENT_MN_HALDEN_GAVE_TIME";
const TRAINER: &str = "MN_HALDEN_FIRST";

fn challenger(root: &AssetRoot, runtime: &CrystalRuntime) -> VisibleShellController {
    let mut c = in_progress(root, runtime, 3, false);
    let state = c.shell.shell.session_mut().state_mut();
    let mut partner = runtime
        .data()
        .create_pokemon("CINDORE", 45, Dv::default())
        .expect("same regional partner with carried battle damage");
    partner.hp = partner
        .max_hp
        .checked_sub(35)
        .expect("injured living partner");
    state.storage.party.pokemon[0] = Some(partner);
    state.sync_party_from_storage();
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["SUPER_POTION"], 1)
            .expect("carried medicine")
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    c
}

fn first_question(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    interact(c, runtime, (5, 7));
    page(c, "MnLeagueRoom4BeforeText");
    page(c, READY);
    assert!(!event(c, WAITED));
    check_run(c, true, 3, false);
}

fn begin_fight(mut c: VisibleShellController, frames: Option<&Path>, name: &str) {
    let expected_hp = c.snapshot().expect("lead before accepting").party.slots[0]
        .pokemon
        .hp;
    choice(&mut c, true);
    for _ in 0..80 {
        c.wait_frames(48).expect("Halden battle introduction");
        let snapshot = c.snapshot().expect("actual match");
        if let Some(battle) = snapshot.battle {
            assert!(
                matches!(&battle.kind, RuntimeBattleKind::Trainer { trainer_id, .. } if trainer_id == TRAINER)
            );
            assert_eq!(battle.player_turns_taken, 0, "await the first real command");
            if c.shell.battle_action_cursor.is_some()
                && c.shell.battle_messages.is_empty()
                && c.shell.visible_battle_transition.is_none()
                && c.shell.visible_battle_sliding_intro.is_none()
                && c.shell.visible_send_out_animation.is_none()
            {
                let first = &c.shell.shell.runtime().data().trainers.trainers[TRAINER].party[0];
                assert_eq!(battle.enemy_pokemon.species.id, first.species);
                let lead = &snapshot.party.slots[0].pokemon;
                assert_eq!(lead.species.id, "CINDORE");
                assert_eq!(lead.hp, expected_hp);
                assert_eq!(battle.active_player_party_index, Some(0));
                let mut app = menu_render_test_app(c.shell);
                app.update();
                app.update();
                let world = app.world_mut();
                let mut sprites = world.query::<(&Sprite, &Transform)>();
                assert!(
                    sprites.iter(world).any(|(sprite, transform)| {
                        // The right-hand four-command window is absent from the trainer introduction.
                        sprite.custom_size == Some(Vec2::new(384.0, 192.0))
                            && transform.translation.truncate() == Vec2::new(128.0, -192.0)
                    }),
                    "the native screen must actually display the first command menu"
                );
                if let Some(directory) = frames {
                    let lcd = world.resource_scope(|world, images: Mut<Assets<Image>>| {
                        pokemon_overworld_context_canvas_scaled(world, &images, 4)
                    });
                    assert_eq!(lcd.dimensions(), (160, 144));
                    std::fs::create_dir_all(directory).expect("battle frame directory");
                    lcd.save(directory.join(name)).expect("command menu frame");
                }
                return;
            }
        }
        press(&mut c, A);
    }
    panic!("accepting Halden did not open the battle command menu");
}

#[test]
#[ignore = "requires the Ely Return pack in MN_BADGE_TRAILS_PACK"]
fn halden_readiness_first_accept_or_prepare_then_continue_and_accept() {
    let (root, runtime) = load();
    let frames = std::env::var_os("MN_ELY_RETURN_FRAMES").map(PathBuf::from);
    let mut direct = challenger(&root, &runtime);
    first_question(&mut direct, &runtime);
    direct = capture(direct, frames.as_deref(), "halden-ready.png");
    begin_fight(direct, frames.as_deref(), "halden-fresh-fight.png");

    let mut c = challenger(&root, &runtime);
    let supplies = c.snapshot().expect("party before Halden");
    first_question(&mut c, &runtime);
    choice(&mut c, false);
    page(&mut c, "MnHaldenWaitText");
    c = capture(c, frames.as_deref(), "halden-waits.png");
    settle(&mut c, "Halden waits for the trainer");
    at(&mut c, ROOM, (5, 8));
    check_run(&c, true, 3, false);
    assert!(event(&c, WAITED));
    let declined = c.snapshot().expect("room after declining");
    assert!(declined.battle.is_none() && declined.ui.menu.is_none());
    assert_eq!(declined.party, supplies.party);
    assert_eq!(declined.bag, supplies.bag);
    travel(&mut c, Down, ROOM, (5, 9));
    use_super_potion(&mut c);
    let healed = c.snapshot().expect("party healed from the Pack");
    let lead = &healed.party.slots[0].pokemon;
    assert_eq!(lead.hp, supplies.party.slots[0].pokemon.hp + 35);
    assert_eq!(lead.moves, supplies.party.slots[0].pokemon.moves);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["SUPER_POTION"]),
        0
    );
    c = continue_save(root, runtime.clone(), &mut c);
    check_run(&c, true, 3, false);
    at(&mut c, ROOM, (5, 9));
    let continued = c.snapshot().expect("recovered party after Continue");
    assert_eq!(continued.party, healed.party);
    assert_eq!(continued.bag, healed.bag);
    assert!(event(&c, WAITED));
    interact(&mut c, &runtime, (5, 7));
    c.wait_frames(64)
        .expect("print the first page of the repeat invitation");
    let prompt = c
        .snapshot()
        .expect("repeat question")
        .ui
        .text
        .expect("Halden spoke");
    assert_eq!(prompt.label, READY, "repeat visits skip the long greeting");
    assert_eq!(prompt.asm_text.as_deref(), Some("HALDEN: Ready to\nstart?"));
    c = capture(c, frames.as_deref(), "halden-ready-after-continue.png");
    begin_fight(c, frames.as_deref(), "halden-prepared-fight.png");
}
