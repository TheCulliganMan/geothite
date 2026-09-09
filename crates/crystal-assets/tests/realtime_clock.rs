#[cfg(test)]
mod tests {
    use crystal_assets::GameDataSet;
    use crystal_core::{
        random::ReplayDivider,
        state::GameState,
        systems::time::{ClockTime, GameDate},
    };

    #[test]
    fn server_clock_manual_settings_cannot_change_time_or_repeat_daily_reset() {
        let data = GameDataSet {
            server_clock: true,
            ..Default::default()
        };
        let mut state = GameState::default();
        let date = GameDate::new(2026, 9, 6);
        let mut divider = ReplayDivider::new([0; 32]);
        data.update_clock_from_datetime(&mut state, date, 14, 25, 30, &mut divider)
            .unwrap();
        let before = state.clone();
        data.set_manual_clock_time(
            &mut state,
            date,
            14,
            25,
            30,
            ClockTime::new(5, 2, 3, 4),
            &mut divider,
        )
        .unwrap();
        assert_eq!(state, before);
    }

    #[test]
    fn daily_quest_restrictions_survive_same_day_and_expire_at_midnight() {
        let data = GameDataSet { server_clock: true, ..Default::default() };
        let mut state = GameState::default();
        let mut divider = ReplayDivider::new([0; 32]);
        let today = GameDate::new(2026, 9, 6);
        data.update_clock_from_datetime(&mut state, today, 12, 0, 0, &mut divider).unwrap();
        let daily = ["ENGINE_KURT_MAKING_BALLS", "ENGINE_GOLDENROD_UNDERGROUND_GOT_HAIRCUT",
            "ENGINE_GINA_HAS_LEAF_STONE", "ENGINE_JOEY_READY_FOR_REMATCH"];
        for flag in daily { state.flags.set_engine_flag(flag, true).unwrap(); }
        state.flags.set_engine_flag("ENGINE_PLAINBADGE", true).unwrap();
        state.flags.set_event_flag("EVENT_GOT_HM01_CUT", true).unwrap();
        data.update_clock_from_datetime(&mut state, today, 23, 59, 59, &mut divider).unwrap();
        for flag in daily { assert!(state.flags.is_engine_flag_set(flag).unwrap(), "{flag}"); }
        data.update_clock_from_datetime(&mut state, GameDate::new(2026, 9, 7),
            0, 0, 0, &mut divider).unwrap();
        for flag in daily { assert!(!state.flags.is_engine_flag_set(flag).unwrap(), "{flag}"); }
        assert!(state.flags.is_engine_flag_set("ENGINE_PLAINBADGE").unwrap());
        assert!(state.flags.is_event_flag_set("EVENT_GOT_HM01_CUT").unwrap());
        // A new day's claim cannot be released again by another clock update.
        state.flags.set_engine_flag(daily[1], true).unwrap();
        data.update_clock_from_datetime(&mut state, GameDate::new(2026, 9, 7),
            0, 0, 1, &mut divider).unwrap();
        assert!(state.flags.is_engine_flag_set(daily[1]).unwrap());
    }

    #[test]
    fn verified_browser_pack_gets_a_distinct_repeatable_clock_identity() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../content-packs/core-modular.browser.crystalpack")
            .canonicalize()
            .unwrap();
        let base = crystal_assets::read_verified_compiled_game_pack(path).unwrap();
        let pack = crystal_assets::build_realtime_clock_modpack(&base).unwrap();
        assert!(pack.data().server_clock);
        assert!(!base.data().server_clock);
        assert!(
            pack.runtime_modpack_id()
                .unwrap()
                .ends_with(crystal_assets::REALTIME_CLOCK_MANIFEST_ID)
        );
        assert_ne!(base.identity().unwrap(), pack.identity().unwrap());
        assert_eq!(
            pack.identity().unwrap(),
            crystal_assets::build_realtime_clock_modpack(&base)
                .unwrap()
                .identity()
                .unwrap()
        );
        assert!(crystal_assets::build_realtime_clock_modpack(&pack).is_err());
        let mut unchanged = pack.data().clone();
        unchanged.server_clock = false;
        assert_eq!(&unchanged, base.data());
        assert_eq!(
            pack.audio_manifest().unwrap(),
            base.audio_manifest().unwrap()
        );
        assert_eq!(pack.runtime_files(), base.runtime_files());
    }

    #[test]
    fn disabled_clock_preserves_base_pack_serialization() {
        let data = GameDataSet::default();
        assert!(
            serde_json::to_value(&data)
                .unwrap()
                .get("server_clock")
                .is_none()
        );
        let data = GameDataSet {
            server_clock: true,
            ..data
        };
        assert_eq!(serde_json::to_value(&data).unwrap()["server_clock"], true);
    }
}
