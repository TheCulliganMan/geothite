//! The courier's witness gives a spoken quest; its old paper gives no result.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::northstar_ice_lair::{choice, event, page, save, talk};
use super::winona::{finish, load};

const QUEST: &str = "EVENT_MN_DEEP_BORE_MANIFEST_READ";

#[test]
#[ignore = "requires the spoken-bridge chapter and isolated saves"]
fn depot_worker_witnesses_courier_then_offers_retryable_acknowledged_quest() {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnStPaulUnionDepot".into(),
            tile_x: 19,
            tile_y: 16,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit prior-access fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let state = c.shell.shell.session_mut().state_mut();
    assert!(
        state.storage.party.add_pokemon(
            r.data()
                .create_pokemon("BOREALYNX", 36, Dv::default())
                .expect("regional geometry partner")
        )
    );
    state.sync_party_from_storage();
    state.badges.johto = [true, true, true, false, false, false, false, false];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..3] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("prior badge fixture");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_DISPATCH_NOTE", true)
        .expect("prior ELLIS conversation");
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("fixture ready");
    // No state writes after this boundary. The actual battle and acknowledged
    // human conversation must independently produce their own native results.
    talk(&mut c, GameButton::Left);
    page(&mut c, "MnNSTPMinaInterruptText");
    finish(&mut c);
    assert!(!event(&c, QUEST));
    talk(&mut c, GameButton::Right);
    page(&mut c, "MnNSTPMinaInterruptText");
    let scene = c.snapshot().expect("worker confronts courier");
    for id in ["MN_NSTP_MINA", "MN_ST_PAUL_DEPOT_COURIER"] {
        assert!(
            scene
                .visible_objects
                .iter()
                .any(|o| o.object_identifier.as_deref() == Some(id))
        );
    }
    assert!(super::rochester_earned::try_battle(
        &mut c,
        &r,
        "MN_LODE_COURIER",
        "EVENT_MN_COURIER_DEFEATED"
    ));
    finish(&mut c);
    assert!(
        !event(&c, QUEST),
        "victory cannot also accept MINA's request"
    );
    c = save(&mut c, &root, &r, "courier-witnessed");
    super::winona::at(&mut c, "MnStPaulUnionDepot", (19, 16));
    talk(&mut c, GameButton::Left);
    page(&mut c, "MnNSTPMinaReadyText");
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, QUEST));
    c = save(&mut c, &root, &r, "mina-quest-declined");
    talk(&mut c, GameButton::Left);
    choice(&mut c, true);
    page(&mut c, "MnNSTPMinaQuestText");
    assert!(!event(&c, QUEST));
    finish(&mut c);
    assert!(event(&c, QUEST));
    c = save(&mut c, &root, &r, "mina-quest-acknowledged");
    talk(&mut c, GameButton::Left);
    page(&mut c, "MnNSTPMinaAfterText");
    finish(&mut c);
    assert!(event(&c, QUEST));
}
