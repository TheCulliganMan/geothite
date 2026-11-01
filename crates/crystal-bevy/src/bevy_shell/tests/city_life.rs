//! City/party placement is an explicit fixture. Every conversation, choice,
//! exchange and repeat attempt below goes through production controller input.
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
};
use super::{continue_save, press};

fn load() -> (AssetRoot, CrystalRuntime, Value) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_CITY_LIFE_PACK").expect("published city population"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified population"),
    )
    .expect("population runtime");
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/city-life.json"),
    )
    .expect("population source");
    let contract = serde_json::from_str(&source).expect("population contract");
    (root, runtime, contract)
}

fn fixture(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    map: &str,
    x: u16,
    y: u16,
) -> VisibleShellController {
    let mut c = VisibleShellController {
        shell: initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime
                    .title_new_game_spawn_identifier()
                    .expect("original HOME"),
                map_name: map.into(),
                tile_x: x as i16,
                tile_y: y as i16,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..Default::default()
            },
        )
        .expect("explicit local city fixture"),
    };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    // A deterministic divider is explicit fixture setup. NPC positions and
    // movement after setup are controlled solely by production input/clock.
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0..65536)
                .scan(0x9e3779b9_u32, |state, _| {
                    *state ^= *state << 13;
                    *state ^= *state >> 17;
                    *state ^= *state << 5;
                    Some((*state >> 16) as u8)
                })
                .collect::<Vec<_>>(),
        );
    c
}

fn talk(c: &mut VisibleShellController) {
    press(c, GameButton::A);
    for _ in 0..128 {
        if c.shell.pc_confirmation.is_some() {
            return;
        }
        press(c, GameButton::A);
    }
    let snapshot = c.snapshot().expect("stalled trade");
    panic!(
        "{} trader did not offer confirmation: text={:?} boundary={:?} notice={:?} pending={} position={:?}",
        snapshot.overworld.map_name,
        snapshot.ui.text.as_ref().map(|t| &t.label),
        c.shell.special_boundary.as_ref().map(|b| &b.label),
        c.shell.pc_notice,
        c.has_pending_script_work(),
        snapshot.overworld.tile
    );
}

fn idle(c: &mut VisibleShellController) {
    assert!(
        c.shell.pc_confirmation.is_none(),
        "a choice must receive explicit input"
    );
    for _ in 0..128 {
        let snapshot = c.snapshot().expect("trade result/dialogue");
        if !snapshot.ui.text_window_open
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.pc_notice.is_none()
            && c.shell.special_boundary.is_none()
            && c.shell.battle_messages.is_empty()
            && !c.shell.party_menu_open
        {
            assert_eq!(
                c.shell.shell.session().state().joypad.h_joy_down,
                0,
                "the closing A must be released before another conversation"
            );
            return;
        }
        press(c, GameButton::A);
    }
    panic!("local dialogue did not release its input owner");
}

#[test]
#[ignore = "requires CRYSTAL_CITY_LIFE_PACK published external population pack"]
fn city_life_trades_decline_reject_cancel_exchange_and_persist_in_every_city() {
    let (root, runtime, source) = load();
    let mut cities = 0;
    for city in source["cities"].as_array().expect("cities") {
        let name = city["map"].as_str().expect("city");
        let Some(map) = runtime.data().maps.get(name) else {
            continue;
        };
        cities += 1;
        let trader = map
            .objects
            .iter()
            .find(|o| o.script == format!("{name}Life01"))
            .expect("city trader");
        let x = trader.x;
        let y = trader.y;
        let mut c = fixture(&root, &runtime, name, x, y + 1);
        for species in [
            "MEW",
            city["trade"]["requested"].as_str().expect("request"),
            city["trade"]["requested"].as_str().expect("second request"),
        ] {
            c.shell
                .shell
                .add_party_pokemon(species, 20, None, None, "CHRIS", 1, Dv::default())
                .expect("explicit trade party");
        }
        super::super::super::mark_runtime_snapshot_dirty(&mut c.shell);
        press(&mut c, GameButton::Up);
        let pos = c.snapshot().expect("actor collision").overworld.tile;
        assert_eq!((pos.x, pos.y), (x as i16, (y + 1) as i16));
        let original_party = c.snapshot().expect("initial party").party;
        eprintln!("{name}: decline");
        talk(&mut c);
        press(&mut c, GameButton::B); // Decline the initial offer.
        idle(&mut c);
        assert_eq!(
            c.snapshot().expect("declined").party,
            original_party,
            "decline {name}"
        );
        eprintln!("{name}: party cancel");
        talk(&mut c);
        press(&mut c, GameButton::A);
        assert!(c.shell.party_menu_open);
        press(&mut c, GameButton::B); // Cancel the party selector.
        idle(&mut c);
        assert_eq!(
            c.snapshot().expect("cancelled").party,
            original_party,
            "cancel {name}"
        );
        talk(&mut c);
        press(&mut c, GameButton::A);
        assert_eq!(c.shell.party_cursor, 0);
        press(&mut c, GameButton::A); // MEW is deliberately the wrong species.
        idle(&mut c);
        assert_eq!(
            c.snapshot().expect("wrong species").party,
            original_party,
            "reject {name}"
        );
        let trade_id = format!("NPC_TRADE_{}_LOCAL", name.to_ascii_uppercase());
        assert!(
            !c.snapshot()
                .expect("not traded")
                .script_events
                .completed_trades
                .contains(&trade_id)
        );
        talk(&mut c);
        press(&mut c, GameButton::A);
        while c.shell.party_cursor != 1 {
            press(&mut c, GameButton::Down);
        }
        press(&mut c, GameButton::A);
        assert_eq!(c.snapshot().expect("before cable").party, original_party);
        assert_eq!(
            c.shell.special_boundary.as_ref().map(|b| b.label.as_str()),
            Some("NPCTradeCableText")
        );
        idle(&mut c);
        let traded = c.snapshot().expect("native exchange");
        assert!(traded.script_events.completed_trades.contains(&trade_id));
        assert_eq!(traded.party.slots.len(), original_party.slots.len());
        let offered = traded.party.slots.last().expect("appended traded partner");
        assert_eq!(
            offered.pokemon.species.id,
            city["trade"]["offered"].as_str().expect("offer")
        );
        assert_eq!(
            offered.pokemon.nickname,
            city["trade"]["nickname"].as_str().expect("nickname")
        );
        assert_eq!(
            offered.pokemon.original_trainer_name,
            city["trade"]["trainer"].as_str().expect("OT")
        );
        assert_eq!(offered.pokemon.item.as_deref(), Some("BERRY"));
        assert_eq!(
            traded
                .party
                .slots
                .iter()
                .filter(|p| p.pokemon.species.id
                    == city["trade"]["requested"].as_str().expect("request"))
                .count(),
            1
        );
        let party = traded.party.clone();
        let mut c = continue_save(&mut c, &root, &runtime);
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        idle(&mut c);
        assert_eq!(
            c.snapshot().expect("once-only after Continue").party,
            party,
            "repeat {name}"
        );
        assert!(
            c.snapshot()
                .expect("saved trade")
                .script_events
                .completed_trades
                .contains(&trade_id)
        );
    }
    assert!(cities >= 14);
}

#[test]
#[ignore = "requires CRYSTAL_CITY_LIFE_PACK published external population pack"]
fn city_life_local_dialogue_owns_input_and_walkers_move_within_authored_lanes() {
    let (root, runtime, source) = load();
    for city in source["cities"].as_array().expect("cities") {
        let name = city["map"].as_str().expect("city");
        let Some(map) = runtime.data().maps.get(name) else {
            continue;
        };
        eprintln!("{name}: local speech and walking");
        for actor in city["actors"].as_array().expect("cast").iter().skip(1) {
            let id = actor["id"].as_str().expect("resident");
            let object = map
                .objects
                .iter()
                .find(|o| o.script == id)
                .expect("installed resident");
            let positions = [
                (object.x, object.y + 1, GameButton::Up),
                (object.x + 1, object.y, GameButton::Left),
                (object.x - 1, object.y, GameButton::Right),
                (object.x, object.y - 1, GameButton::Down),
            ];
            let (x, y, face) = positions
                .into_iter()
                .find(|&(x, y, _)| {
                    let block = map.blocks[usize::from(y / 2) * usize::from(map.attributes.width)
                        + usize::from(x / 2)];
                    runtime.data().tilesets[&map.attributes.tileset_name].collision
                        [&format!("{block:02x}")][usize::from(y % 2 * 2 + x % 2)]
                        == "FLOOR"
                        && !map.objects.iter().any(|o| o.x == x && o.y == y)
                        && !map.events.warps.iter().any(|w| w.x == x && w.y == y)
                        && !map.events.coord_events.iter().any(|e| e.x == x && e.y == y)
                })
                .expect("safe neighbor for local fixture");
            let mut c = fixture(&root, &runtime, name, x, y);
            if actor["movement"] == "stand" {
                press(&mut c, face);
                press(&mut c, GameButton::A);
                let snapshot = c.snapshot().expect("local dialogue");
                assert_eq!(
                    snapshot.ui.text.as_ref().map(|t| t.label.as_str()),
                    Some(format!("{id}Text").as_str()),
                    "{id}"
                );
                let at = snapshot.overworld.tile;
                press(&mut c, GameButton::Left);
                assert_eq!(
                    c.snapshot().expect("text owns input").overworld.tile,
                    at,
                    "{id}"
                );
                idle(&mut c);
            } else {
                let mut seen =
                    std::collections::BTreeSet::from([(object.x as i16, object.y as i16)]);
                for frame in 0..256 {
                    c.wait_frames(1).expect("native NPC clock");
                    let tile = c
                        .shell
                        .shell
                        .session()
                        .overworld()
                        .object_runtime_tile_by_id(id)
                        .expect("live walker");
                    assert!(
                        tile.x.abs_diff(object.x as i16) <= object.move_range_x.saturating_sub(1)
                            && tile.y.abs_diff(object.y as i16)
                                <= object.move_range_y.saturating_sub(1),
                        "{id} left its lane"
                    );
                    seen.insert((tile.x, tile.y));
                    if frame >= 32 && seen.len() > 1 {
                        break;
                    }
                }
                assert!(seen.len() > 1, "{id} must actually walk");
            }
        }
    }
}
