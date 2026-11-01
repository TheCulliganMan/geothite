use crystal_core::random::ReplayDivider;
use crystal_core::state::GameState;
use crystal_core::systems::time::{ClockTime, GameDate};

#[test]
#[ignore = "requires CRYSTAL_NORTHSTAR_TITLE_PACK"]
fn old_manual_clock_does_not_expire_same_day_events() {
    let path = std::env::var_os("CRYSTAL_NORTHSTAR_TITLE_PACK").expect("Minnesota review pack");
    let pack = crystal_assets::read_verified_compiled_game_pack(path).expect("verified pack");
    let data = pack.data();
    assert!(data.is_minnesota());
    let mut state = GameState::default();
    let date = GameDate::new(2026, 11, 1);
    state
        .time
        .set_manual_time(date, 12, 0, 0, ClockTime::new(3, 7, 30, 0));
    state
        .flags
        .set_engine_flag("ENGINE_GINA_HAS_LEAF_STONE", true)
        .expect("daily flag");
    let mut divider = ReplayDivider::new([0; 32]);
    data.update_clock_from_datetime(&mut state, date, 12, 0, 0, &mut divider)
        .expect("local clock");
    assert_eq!(state.time.start_time, ClockTime::default());
    assert_eq!(state.time.registers.hours, 12);
    assert!(
        state
            .flags
            .is_engine_flag_set("ENGINE_GINA_HAS_LEAF_STONE")
            .expect("daily flag")
    );
    data.update_clock_from_datetime(
        &mut state,
        GameDate::new(2026, 11, 2),
        0,
        0,
        0,
        &mut divider,
    )
    .expect("midnight");
    assert!(
        !state
            .flags
            .is_engine_flag_set("ENGINE_GINA_HAS_LEAF_STONE")
            .expect("daily flag")
    );
}
