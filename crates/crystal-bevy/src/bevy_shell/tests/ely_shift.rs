//! Check the player-facing promise at Casey's replacement boundary.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton::{A, B, Down};
use crystal_core::models::Dv;
use crystal_core::state::BattleMemory;
use crystal_runtime::CrystalRuntime;

use super::super::{
    VisibleShellController, claim_visible_battle_rewards, mark_runtime_snapshot_dirty,
};
use super::badge_trails::load;
use super::interior_life::fixture;
use super::winona::press;

const ROOM: &str = "MnLeagueChampion";

fn first_knockout(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    lead: &str,
    reserve: &str,
) -> VisibleShellController {
    let mut c = fixture(root, runtime, ROOM, (5, 8));
    let state = c.shell.shell.session_mut().state_mut();
    for species in [lead, reserve] {
        let member = runtime
            .data()
            .create_pokemon(species, 48, Dv::default())
            .expect("regional League partner");
        assert!(state.storage.party.add_pokemon(member));
    }
    state.sync_party_from_storage();
    let entry = runtime.data().maps[ROOM]
        .scripted_trainer_battles
        .iter()
        .find(|b| b.request.trainer_id == "MN_CHAMPION")
        .expect("Casey's real party");
    c.shell
        .shell
        .start_scripted_trainer_battle(ROOM, &entry.source_script, entry.startbattle_command_index)
        .expect("begin Casey's battle");
    // Start at the first faint. The reward, message, decision and both
    // replacements then run through the production controller.
    let state = c.shell.shell.session_mut().state_mut();
    let BattleMemory::Trainer {
        enemy_party,
        enemy_pokemon,
        ..
    } = &mut state.battle
    else {
        panic!("Casey must be active");
    };
    assert_eq!(enemy_pokemon.species.id, "BEETYNAMO");
    enemy_party[0].hp = 0;
    enemy_pokemon.hp = 0;
    if let Some(combat) = &mut state.script_runtime.active_battle_combat {
        combat.enemy.hp = 0;
        combat.enemy_party[0].hp = 0;
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    claim_visible_battle_rewards(&mut c.shell).expect("award XP and offer the native Shift prompt");
    c
}

fn offered(c: &mut VisibleShellController) -> String {
    let mut named = None;
    for _ in 0..64 {
        if let Some(message) = c.shell.battle_messages.front() {
            if let Some((_, name)) = message.split_once("is about to use\n") {
                named = Some(name.trim_end_matches('.').to_owned());
            }
            press(c, A);
        } else {
            let menu = c
                .snapshot()
                .expect("visible prompt")
                .ui
                .menu
                .expect("Shift menu");
            assert!(
                menu.layout
                    .vertical_menus
                    .iter()
                    .flat_map(|m| &m.options)
                    .any(|line| line.contains("change <PKMN>"))
            );
            return named.expect("Casey must name the opponent before the decision");
        }
    }
    panic!("did not reach Casey's Shift decision")
}

fn decide(mut c: VisibleShellController, change: bool) -> String {
    let named = offered(&mut c);
    if change {
        press(&mut c, A);
        let selected = c
            .shell
            .battle_switch_cursor
            .as_ref()
            .expect("choose a partner")
            .option_index;
        if selected == 0 {
            press(&mut c, Down);
        }
        assert_eq!(
            c.shell
                .battle_switch_cursor
                .as_ref()
                .expect("reserve selected")
                .option_index,
            1
        );
        press(&mut c, A);
    } else {
        press(&mut c, B);
    }
    for _ in 0..48 {
        let s = c.snapshot().expect("trainer battle after Shift");
        let b = s.battle.expect("Casey's remaining team");
        if b.enemy_pokemon.hp > 0
            && c.shell.battle_messages.is_empty()
            && c.shell.battle_action_cursor.is_some()
        {
            assert_eq!(
                b.enemy_pokemon.nickname, named,
                "the announced opponent must actually appear"
            );
            assert_eq!(b.active_player_party_index, Some(usize::from(change)));
            return named;
        }
        assert!(
            !c.shell.battle_messages.is_empty(),
            "missing replacement message or battle menu"
        );
        press(&mut c, A);
    }
    panic!("send-out never returned to the battle menu")
}

#[test]
#[ignore = "requires the installed Ely League in MN_BADGE_TRAILS_PACK"]
fn ely_shift_names_the_actual_opponent_on_both_answers() {
    let (root, runtime) = load();
    let challenger = || first_knockout(&root, &runtime, "CYPRIELLE", "PYROPINE");
    let mut c = challenger();
    let battle = c
        .snapshot()
        .expect("rewarded knockout")
        .battle
        .expect("Casey");
    let first = &battle.enemy_party[1];
    let slot = battle
        .pending_trainer_replacement
        .expect("Casey's choice before Shift");
    let selected = &battle.enemy_party[slot];
    assert_eq!(first.species.id, "CHOIRBOG");
    assert_eq!(selected.species.id, "MUSKETUNGE");
    assert_eq!(decide(c, false), selected.nickname);
    assert_eq!(decide(challenger(), true), selected.nickname);
}

#[test]
#[ignore = "requires the installed Ely League in MN_BADGE_TRAILS_PACK"]
fn ely_shift_continue_keeps_the_offer_without_paying_experience_twice() {
    let (root, runtime) = load();
    let mut original = first_knockout(&root, &runtime, "CYPRIELLE", "PYROPINE");
    let promised = offered(&mut original);
    let before = original
        .snapshot()
        .expect("awarded knockout at the Yes/No question");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = super::authored_route_fixture::reached_save_path(&format!(
        "ely-shift-{}-{stamp}.crystalsave",
        std::process::id()
    ));
    original.save(&path).expect("save at the Shift decision");
    let mut resumed = VisibleShellController::load_save(root, runtime, path.clone(), None)
        .expect("Continue at Shift");
    super::authored_route_fixture::cleanup_reached_save(&path);
    let after = resumed.snapshot().expect("continued Casey battle");
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    assert_eq!(
        after
            .battle
            .as_ref()
            .expect("continued battle")
            .rewarded_enemy_party_indices,
        vec![0]
    );
    assert_eq!(decide(resumed, true), promised);
}
