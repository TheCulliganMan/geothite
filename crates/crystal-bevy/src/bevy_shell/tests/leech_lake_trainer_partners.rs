//! Walk into the three revised trainers' sightlines, then inspect their actual send-outs.
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use bevy::prelude::{Assets, Image, Mut, Sprite, With};
use crystal_core::input::GameButton::{self, A, Down, Left, Right, Up};
use crystal_core::models::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::interior_life::fixture;
use super::super::leech_lake::battle_button;
use super::super::northstar_ice_lair::event;
use super::super::rochester_earned::try_battle;
use super::super::winona::{press, travel};
use super::super::{menu_render_test_app, pokemon_overworld_context_canvas_scaled};
use super::load;
use crate::bevy_shell::{
    BattleBattlerMarker, BevyRuntimeShell, PokemonSpriteSide, RenderedTilesetArt,
    VisibleShellController, mark_runtime_snapshot_dirty,
};

const ROUTE: &str = "MnR07C00R00";
const LEAD: &str = "BOREALYNX";
type Tile = (i16, i16);

struct Trainer {
    id: &'static str,
    victory: &'static str,
    start: Tile,
    walk: &'static [(GameButton, Tile)],
    seen: &'static str,
    party: [(&'static str, [&'static str; 4]); 2],
    featured: usize,
    frame: &'static str,
}

const NELL: Trainer = Trainer {
    id: "MN_R07_T05",
    victory: "EVENT_MN_R07_TRAINER_05",
    start: (49, 35),
    walk: &[(Up, (49, 31))],
    seen: "That branch is off the trail. I promised my team a battle. Yours!",
    party: [
        (
            "RICELING",
            ["SWEET_SCENT", "MEGA_DRAIN", "SYNTHESIS", "RAZOR_LEAF"],
        ),
        (
            "VELVAWN",
            ["STOMP", "RAZOR_LEAF", "LEECH_SEED", "TAKE_DOWN"],
        ),
    ],
    featured: 1,
    frame: "nell-velvawn.png",
};
const SILAS: Trainer = Trainer {
    id: "MN_R07_T07",
    victory: "EVENT_MN_R07_TRAINER_07",
    start: (36, 14),
    walk: &[(Right, (40, 14)), (Down, (40, 15))],
    seen: "Something dodges every cast I make. Let us see if your team can!",
    party: [
        ("JIGLET", ["WATER_PULSE", "METAL_CLAW", "SWIFT", "PURSUIT"]),
        (
            "CASTOROCK",
            ["ANCIENTPOWER", "BUBBLEBEAM", "SUPER_FANG", "SLAM"],
        ),
    ],
    featured: 0,
    frame: "silas-jiglet.png",
};
const WREN: Trainer = Trainer {
    id: "MN_R07_T08",
    victory: "EVENT_MN_R07_TRAINER_08",
    start: (17, 15),
    walk: &[(Left, (15, 15)), (Up, (15, 13))],
    seen: "The rink is shut. We have the puck. Practice goes on. You are up!",
    party: [
        ("DARTDUCK", ["QUICK_ATTACK", "LOW_KICK", "PURSUIT", "SLAM"]),
        ("FLOSSAIL", ["GUST", "ACID", "COTTON_SPORE", "GIGA_DRAIN"]),
    ],
    featured: 0,
    frame: "wren-dartduck.png",
};

fn show_partners(
    c: VisibleShellController,
    trainer: &Trainer,
    frames: Option<&Path>,
) -> VisibleShellController {
    let mut app = menu_render_test_app(c.shell);
    app.update();
    app.update();
    let world = app.world_mut();
    let displayed = world
        .query_filtered::<&Sprite, With<BattleBattlerMarker>>()
        .iter(world)
        .map(|sprite| sprite.image.id())
        .collect::<HashSet<_>>();
    let art = world.resource::<RenderedTilesetArt>();
    for (species, side) in [
        (trainer.party[trainer.featured].0, PokemonSpriteSide::Front),
        (LEAD, PokemonSpriteSide::Back),
    ] {
        assert!(
            art.pokemon_cache.iter().any(|(key, frame)| {
                key.species_id.eq_ignore_ascii_case(species)
                    && key.side == side
                    && displayed.contains(&frame.handle.id())
            }),
            "native battle must display {species} {side:?} at the command menu"
        );
    }
    if let Some(directory) = frames {
        let lcd = world.resource_scope(|world, images: Mut<Assets<Image>>| {
            pokemon_overworld_context_canvas_scaled(world, &images, 4)
        });
        assert_eq!(lcd.dimensions(), (160, 144));
        std::fs::create_dir_all(directory).expect("Route 7 native frames");
        lcd.save(directory.join(trainer.frame))
            .expect("battle command menu");
    }
    VisibleShellController {
        shell: world
            .remove_resource::<BevyRuntimeShell>()
            .expect("continue the same battle"),
    }
}

fn challenge(
    mut c: VisibleShellController,
    runtime: &CrystalRuntime,
    trainer: &Trainer,
    frames: Option<&Path>,
) -> VisibleShellController {
    assert!(!event(&c, trainer.victory));
    for &(direction, destination) in trainer.walk {
        travel(&mut c, direction, ROUTE, destination);
    }
    c.wait_frames(160)
        .expect("trainer spots and approaches the walker");
    let notice = c
        .shell
        .field_notice
        .as_deref()
        .expect("walking alone triggers trainer challenge");
    assert_eq!(
        notice.split_whitespace().collect::<Vec<_>>().join(" "),
        trainer.seen
    );
    let mut saw_opener = false;
    for _ in 0..512 {
        c.wait_frames(24).expect("send-out presentation");
        let snapshot = c.snapshot().expect("active route battle");
        let Some(battle) = &snapshot.battle else {
            press(&mut c, A);
            continue;
        };
        assert!(
            matches!(&battle.kind, RuntimeBattleKind::Trainer { trainer_id, event_flag, .. } if trainer_id == trainer.id && event_flag == trainer.victory)
        );
        let command_menu = c.shell.battle_action_cursor.is_some()
            && c.shell.battle_messages.is_empty()
            && c.shell.visible_battle_transition.is_none()
            && c.shell.visible_battle_sliding_intro.is_none()
            && c.shell.visible_send_out_animation.is_none();
        if command_menu {
            let index = battle.active_enemy_party_index.expect("deployed enemy");
            assert_eq!(battle.enemy_pokemon.species.id, trainer.party[index].0);
            if !saw_opener {
                assert_eq!(index, 0);
                assert_eq!(battle.enemy_party.len(), trainer.party.len());
                for (pokemon, (species, moves)) in battle.enemy_party.iter().zip(trainer.party) {
                    assert_eq!((&*pokemon.species.id, pokemon.level), (species, 30));
                    assert_eq!(
                        pokemon
                            .moves
                            .iter()
                            .map(|m| m.name.as_str())
                            .collect::<Vec<_>>(),
                        moves
                    );
                    assert!(
                        pokemon
                            .moves
                            .iter()
                            .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
                    );
                }
                saw_opener = true;
            }
            if index == trainer.featured {
                if index == 1 {
                    assert_eq!(
                        battle.enemy_party[0].hp, 0,
                        "Nell replaces her fainted opener"
                    );
                    assert!(battle.rewarded_enemy_party_indices.contains(&0));
                }
                return show_partners(c, trainer, frames);
            }
        }
        let button = battle_button(&c, runtime, &snapshot);
        press(&mut c, button);
    }
    panic!("{} never sent out the expected partner", trainer.id);
}

fn revisit_wren(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    let before = c.shell.shell.session().state().money;
    assert!(try_battle(c, runtime, WREN.id, WREN.victory));
    assert!(event(c, WREN.victory));
    let paid = before + 600; // Picnicker's five-unit rate, level 30, four wallet shares.
    assert_eq!(c.shell.shell.session().state().money, paid);
    let actor = c
        .snapshot()
        .expect("Wren after victory")
        .visible_object_runtime_tiles[WREN.id];
    travel(c, Left, ROUTE, (actor.x + 1, actor.y));
    press(c, Left);
    press(c, A);
    let mut heard_return = false;
    for _ in 0..64 {
        let s = c.snapshot().expect("repeat interaction");
        assert!(s.battle.is_none(), "Wren already awarded the win");
        if let Some(text) = s.ui.text {
            heard_return |= text.label == "MN_R07_T08ReturnText";
        }
        if let Some(text) = c.shell.field_notice.as_deref() {
            heard_return |= text.contains("Come here, bud.");
        }
        if heard_return
            && !s.ui.text_window_open
            && c.shell.field_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(c, A);
    }
    assert!(heard_return, "Wren talks about setting the puck down");
    travel(c, Down, ROUTE, (actor.x + 1, 15));
    assert!(event(c, WREN.victory));
    assert!(c.snapshot().expect("free to walk on").battle.is_none());
    assert_eq!(c.shell.shell.session().state().money, paid);
}

#[test]
#[ignore = "requires Leech Lake Trainers in MN_BADGE_TRAILS_PACK"]
fn leech_lake_trainer_partners_see_the_walker_and_send_their_teams() {
    let (root, runtime) = load();
    let frames = std::env::var_os("MN_LEECH_PARTNER_FRAMES").map(PathBuf::from);
    for trainer in [&NELL, &SILAS, &WREN] {
        let mut c = fixture(&root, &runtime, ROUTE, trainer.start);
        // A supplied local partner isolates sight, send-out and return behavior from training time.
        let lead = runtime
            .data()
            .create_pokemon(LEAD, 48, Dv::default())
            .expect("regional partner");
        let state = c.shell.shell.session_mut().state_mut();
        state.badges.johto[..6].fill(true);
        assert!(state.storage.party.add_pokemon(lead));
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut c.shell);
        let mut c = challenge(c, &runtime, trainer, frames.as_deref());
        if trainer.id == WREN.id {
            revisit_wren(&mut c, &runtime);
        }
    }
}
