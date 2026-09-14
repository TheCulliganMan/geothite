#[cfg(test)]
mod server_clock_tests {
    use crystal_core::systems::time::*;
    use crystal_core::world::encounters::TimeOfDay;

    #[test]
    fn server_clock_overrides_offsets_and_tracks_real_weekdays() {
        let mut first = TimeState::new(GameDate::new(2000, 1, 1));
        let mut second = TimeState::new(GameDate::new(2025, 4, 15));
        second.start_time = ClockTime::new(3, 12, 42, 7);
        for state in [&mut first, &mut second] {
            state.update_server_datetime(GameDate::new(2026, 9, 6), 23, 59, 59);
            assert_eq!(state.day_of_week, 0);
            assert_eq!(state.registers.hours, 23);
            assert_eq!(state.registers.minutes, 59);
            assert_eq!(state.registers.seconds, 59);
            state.update_server_datetime(GameDate::new(2026, 9, 7), 4, 0, 0);
            assert_eq!(state.day_of_week, 1);
            assert_eq!(state.time_of_day, TimeOfDay::Morning);
            state.update_server_datetime(GameDate::new(2026, 9, 7), 10, 0, 0);
            assert_eq!(state.time_of_day, TimeOfDay::Day);
            state.update_server_datetime(GameDate::new(2026, 9, 7), 18, 0, 0);
            assert_eq!(state.time_of_day, TimeOfDay::Night);
        }
        assert_eq!(first, second);
    }

    #[test]
    fn server_clock_wraps_crystal_day_cycle_without_losing_weekday() {
        let mut state = TimeState::new(DEFAULT_RTC_ANCHOR);
        state.update_server_datetime(GameDate::new(2000, 5, 20), 23, 59, 59);
        assert_eq!(state.current_day, 139);
        state.update_server_datetime(GameDate::new(2000, 5, 21), 0, 0, 0);
        assert_eq!(state.current_day, 0);
        assert_eq!(state.day_of_week, 0);
        assert_eq!(state.rtc_status_flags, 0);
    }
}
