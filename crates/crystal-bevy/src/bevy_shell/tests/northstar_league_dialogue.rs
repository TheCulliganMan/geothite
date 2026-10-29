//! Declared Minnesota party/prior access, followed exclusively by native input.
use super::super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::super::northstar_ice_lair::{event, page};
use super::super::rochester_earned::try_league_battle;
use super::super::winona::{at, continued, load};
use super::ely_league::{enter_lobby, registrar, restore_partner_with, run, until_map};
use super::north_shore_story::choice;
use super::{finish, interact, press, walk_to};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
const ACTIVE: &str = "EVENT_MN_LEAGUE_RUN_ACTIVE";
const CHAMPION: &str = "EVENT_MN_LEAGUE_CHAMPION";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const CLINIC: &str = "MnElyPokecenter1F";

fn setup(
    map: &str,
    tile: [u16; 2],
    champion: bool,
    completed: u8,
) -> (
    crystal_assets::AssetRoot,
    crystal_runtime::CrystalRuntime,
    VisibleShellController,
) {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: i16::try_from(tile[0]).expect("fixture x fits native coordinates"),
            tile_y: i16::try_from(tile[1]).expect("fixture y fits native coordinates"),
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("declared League scene fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto = [true; 8];
    for flag in &BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("prior badge");
    }
    state
        .flags
        .set_event_flag(RESTORED, true)
        .expect("prior restoration");
    state
        .flags
        .set_event_flag(CHAMPION, champion)
        .expect("prior championship");
    if map.starts_with("MnLeague") {
        state
            .flags
            .set_event_flag(ACTIVE, true)
            .expect("declared registered run");
        state
            .script_runtime
            .memory
            .insert("wMnLeagueRun".into(), completed.to_string());
    }
    for species in [
        "BOREALYNX",
        "VOYAGEAR",
        "FORTIGARD",
        "FLOURGEIST",
        "GOUGECREST",
        "BEETYNAMO",
    ] {
        let mut p = r
            .data()
            .create_pokemon(species, 100, Dv::default())
            .expect("active Minnesota review partner");
        let legal = [
            "FLAMETHROWER",
            "ICE_BEAM",
            "THUNDERBOLT",
            "EARTHQUAKE",
            "SURF",
            "PSYCHIC_M",
            "SHADOW_BALL",
            "HEADBUTT",
            "IRON_TAIL",
        ]
        .into_iter()
        .filter(|m| p.species.tmhm_learnset.iter().any(|v| v == m))
        .take(4)
        .collect::<Vec<_>>();
        if !legal.is_empty() {
            p.moves = legal
                .into_iter()
                .map(|m| LearnedMove {
                    name: m.into(),
                    current_pp: r.data().moves[m].pp,
                    pp_ups: 0,
                })
                .collect();
        }
        assert!(state.storage.party.add_pokemon(p));
    }
    state.sync_party_from_storage();
    for item in ["FULL_RESTORE", "REVIVE"] {
        state
            .bag
            .add_item(&r.data().items[item], 99)
            .expect("declared carried medicine");
    }
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("presentation");
    // No direct state, position, result or RNG changes below the setup boundary.
    (root, r, c)
}

#[test]
#[ignore = "requires written League pack; declared prior access and strong native party"]
fn league_finale_five_native_wins_hall_of_fame_and_rematch_continue() {
    let (root, r, mut c) = setup("MnEly", [24, 1], false, 0);
    // Six level-100 Minnesota partners and medicine are declared review inputs.
    // This proves native dialogue/progression, not earned campaign balance.
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
    c = continued(c, &root, &r, "league-written-registered");
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
        for slot in 0..c.snapshot().expect("party").party.slots.len() {
            if c.snapshot().expect("party").party.slots[slot].pokemon.hp == 0 {
                restore_partner_with(&mut c, &r, "REVIVE", slot);
            }
            restore_partner_with(&mut c, &r, "FULL_RESTORE", slot);
        }
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
        walk_to(&mut c, &r, (5, 8));
        c = continued(c, &root, &r, &format!("league-written-before-{index}"));
        interact(&mut c, &r, (5, 7));
        page(&mut c, &format!("{map}BeforeText"));
        assert!(try_league_battle(
            &mut c,
            &r,
            if index == 5 {
                "MN_CHAMPION"
            } else {
                [
                    "",
                    "MN_LEAGUE_1",
                    "MN_LEAGUE_2",
                    "MN_LEAGUE_3",
                    "MN_LEAGUE_4",
                ][index as usize]
            },
            index
        ));
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
        c = continued(c, &root, &r, &format!("league-written-win-{index}"));
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
#[ignore = "requires written League pack; declared prior title/run for rematch scenes"]
fn league_finale_all_five_rematch_intros_require_recorded_championship() {
    for (index, map) in [
        "MnLeagueRoom1",
        "MnLeagueRoom2",
        "MnLeagueRoom3",
        "MnLeagueRoom4",
        "MnLeagueChampion",
    ]
    .into_iter()
    .enumerate()
    {
        for champion in [false, true] {
            let (_, r, mut c) = setup(map, [5, 8], champion, index as u8);
            interact(&mut c, &r, (5, 7));
            page(
                &mut c,
                &format!("{map}{}Text", if champion { "Rematch" } else { "Before" }),
            );
            assert_eq!(run(&c), index as u8, "reading an intro cannot award a win");
            assert!(c.snapshot().expect("intro").battle.is_none());
        }
    }
}
