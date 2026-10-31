//! Stable Minnesota outdoor weather, shared across connected map boundaries.
use crystal_core::systems::time::{DEFAULT_RTC_ANCHOR, TimeState, compute_day_count};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Weather {
    pub rain: f32,
    pub snow: f32,
}

fn wet_window(window: u32) -> bool {
    let mut seed = 1_664_525u32.wrapping_add(window.wrapping_mul(1_013_904_223));
    seed ^= seed >> 16;
    seed = seed.wrapping_mul(0x7feb_352d);
    seed ^= seed >> 15;
    seed.is_multiple_of(5)
}

fn rain(time: &TimeState) -> f32 {
    // Twenty-minute weather windows use the saved/synchronized game clock.
    // No map name, render-frame counter or gameplay RNG enters this schedule.
    let second = u32::from(time.registers.hours) * 3600
        + u32::from(time.registers.minutes) * 60
        + u32::from(time.registers.seconds);
    let days = u32::from(compute_day_count(DEFAULT_RTC_ANCHOR, time.current_date));
    let window = days * 72 + second / 1200;
    let current = f32::from(u8::from(wet_window(window)));
    let previous = f32::from(u8::from(wet_window(window.saturating_sub(1))));
    let blend = ((second % 1200) as f32 / 12.0).min(1.0);
    previous + (current - previous) * blend
}

fn snow(map: &str, tileset: &str, tile_x: i16) -> f32 {
    // The lake preserve stays green even beside snowy inland routes. Keep this
    // explicit so a shared winter art alias cannot enable snowfall here.
    if map.starts_with("MnBoundarySafari")
        || matches!(map, "MnBoundaryWaters" | "MnPreservePineEntry")
    {
        return 0.0;
    }
    // These names are historical art aliases, not permission to freeze Superior.
    if map.starts_with("MnR11C") || matches!(map, "MnGrandMarais" | "MnMaraisCedarApproach") {
        return 0.0;
    }
    if map == "MnElyShorePass" {
        // The authored pass has a real inland-to-coast snowline within one map.
        return ((8.0 - f32::from(tile_x)) / 4.0).clamp(0.0, 1.0);
    }
    if matches!(
        tileset,
        "mn_ely_snow"
            | "mn_international_falls"
            | "mn_winter_outdoor"
            | "mn_winter_gunflint"
            | "mn_outdoor_winter_stairs"
    ) || map.starts_with("MnR13C") && tileset == "mn_winter_north_shore"
    {
        1.0
    } else {
        0.0
    }
}

pub(super) fn sample(
    map: &str,
    tileset: &str,
    environment: &str,
    tile_x: i16,
    time: &TimeState,
) -> Weather {
    if !map.starts_with("Mn") || !matches!(environment, "TOWN" | "ROUTE") {
        return Weather::default();
    }
    let snow = snow(map, tileset, tile_x);
    Weather {
        rain: rain(time) * (1.0 - snow),
        snow,
    }
}

#[cfg(test)]
mod tests {
    use super::{rain, sample};
    use crystal_core::systems::time::{GameDate, TimeState};

    #[test]
    fn shower_changes_fade_and_midnight_keeps_the_previous_window() {
        let mut time = TimeState::default();
        time.current_date = GameDate::new(2026, 10, 31);
        time.registers.hours = 23;
        time.registers.minutes = 59;
        time.registers.seconds = 59;
        let before = rain(&time);
        time.current_date = GameDate::new(2026, 11, 1);
        time.registers.hours = 0;
        time.registers.minutes = 0;
        time.registers.seconds = 0;
        assert_eq!(rain(&time), before);
        time.registers.seconds = 6;
        let midway = rain(&time);
        time.registers.seconds = 12;
        assert!((midway - (before + rain(&time)) * 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn rain_is_occasional_and_identical_across_warm_map_seams() {
        let mut time = TimeState::default();
        let mut wet = 0;
        let mut dry = 0;
        for hour in 0..24 {
            time.registers.hours = hour;
            for minute in [1, 21, 41] {
                time.registers.minutes = minute;
                let city = sample("MnMinneapolis", "mn_minneapolis_city", "TOWN", 20, &time);
                let road = sample("MnR05C00R00", "mn_headwaters", "ROUTE", 1, &time);
                assert_eq!(city, road);
                assert_eq!(
                    city,
                    sample("MnGrandMarais", "mn_marais_shore", "TOWN", 4, &time)
                );
                wet += usize::from(city.rain == 1.0);
                dry += usize::from(city.rain == 0.0);
                assert_eq!(city.snow, 0.0);
            }
        }
        assert!(
            wet >= 5 && dry > wet,
            "weather needs both showers and longer clear intervals"
        );
    }

    #[test]
    fn indoors_original_regions_and_coastal_snow_are_excluded() {
        let time = TimeState::default();
        for (map, tileset, environment) in [
            ("MnElyPokecenter1F", "mn_ely_snow", "INDOOR"),
            ("MnR12Cave1F", "cave", "CAVE"),
            ("GoldenrodCity", "johto", "TOWN"),
        ] {
            assert_eq!(
                sample(map, tileset, environment, 0, &time),
                Default::default()
            );
        }
        for map in ["MnR11C00R00", "MnGrandMarais", "MnMaraisCedarApproach"] {
            assert_eq!(
                sample(map, "mn_winter_outdoor", "ROUTE", 0, &time).snow,
                0.0
            );
        }
        assert_eq!(sample("MnEly", "mn_ely_snow", "TOWN", 10, &time).snow, 1.0);
        assert_eq!(
            sample("MnElyShorePass", "mn_winter_north_shore", "ROUTE", 0, &time).snow,
            1.0
        );
        assert_eq!(
            sample(
                "MnElyShorePass",
                "mn_winter_north_shore",
                "ROUTE",
                12,
                &time
            )
            .snow,
            0.0
        );
    }

    #[test]
    fn boundary_waters_and_safari_keep_rain_without_snow() {
        let mut time = TimeState::default();
        let mut showers = 0;
        for hour in 0..24 {
            time.registers.hours = hour;
            time.registers.minutes = 1;
            let warm = sample("MnGrandMarais", "mn_marais_shore", "TOWN", 0, &time);
            showers += usize::from(warm.rain > 0.0);
            for map in [
                "MnBoundaryWaters",
                "MnBoundarySafariC00R00",
                "MnBoundarySafariC01R01",
                "MnPreservePineEntry",
            ] {
                for tileset in [
                    "mn_boundary_waters",
                    "mn_winter_outdoor",
                    "mn_winter_gunflint",
                ] {
                    assert_eq!(sample(map, tileset, "ROUTE", 0, &time), warm);
                }
            }
        }
        assert!(
            showers > 0,
            "the snow exclusion must retain occasional rain"
        );
    }
}
