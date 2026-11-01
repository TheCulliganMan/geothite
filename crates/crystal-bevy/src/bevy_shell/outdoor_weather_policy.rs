//! Stable Minnesota outdoor weather, shared across connected map boundaries.
use crystal_core::systems::time::{DEFAULT_RTC_ANCHOR, TimeState, compute_day_count};

use super::inland_winter_render;

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

fn snow(map: &str, tileset: &str, tile_x: i16, tile_y: i16) -> f32 {
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
    if map.starts_with("MnR20C01R") && tileset == "mn_tamarack_thaw" {
        // Both clipped eastern chunks share the same eastward melt zone.
        // The preserve approach is fully green before the actual map seam.
        return ((38.0 - f32::from(tile_x)) / 24.0).clamp(0.0, 1.0);
    }
    if tileset == "mn_iron_descent_thaw" {
        let route_x = match map {
            "MnR10C00R00" => Some(f32::from(tile_x)),
            "MnR10C01R00" => Some(64.0 + f32::from(tile_x)),
            _ => None,
        };
        if let Some(x) = route_x {
            return ((64.0 - x) / 34.0).clamp(0.0, 1.0);
        }
    }
    if tileset == "mn_headwaters_thaw" && matches!(map, "MnR08C00R01" | "MnR08C01R01") {
        // Northern chunks are fully snowy. Both southern portages thaw along
        // the same route Y, reaching ordinary green before the Leech seam.
        return ((80.0 - (48.0 + f32::from(tile_y))) / 26.0).clamp(0.0, 1.0);
    }
    if tileset == "mn_itasca_thaw" && matches!(map, "MnR14C00R00" | "MnR14C00R01") {
        // Both western chunks share the young river's broad westward thaw.
        // Itasca is green before its real gatehouse/connection approach.
        return ((f32::from(tile_x) - 34.0) / 24.0).clamp(0.0, 1.0);
    }
    if matches!(
        tileset,
        "mn_ely_snow"
            | "mn_international_falls"
            | "mn_winter_outdoor"
            | "mn_winter_gunflint"
            | "mn_outdoor_winter_stairs"
    ) || inland_winter_render::source(tileset).is_some()
        || map.starts_with("MnR13C") && tileset == "mn_winter_north_shore"
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
    tile_y: i16,
    time: &TimeState,
) -> Weather {
    if !map.starts_with("Mn") || !matches!(environment, "TOWN" | "ROUTE") {
        return Weather::default();
    }
    let snow = snow(map, tileset, tile_x, tile_y);
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
                let city = sample("MnMinneapolis", "mn_minneapolis_city", "TOWN", 20, 0, &time);
                let road = sample("MnR05C00R00", "mn_headwaters", "ROUTE", 1, 0, &time);
                assert_eq!(city, road);
                assert_eq!(
                    city,
                    sample("MnGrandMarais", "mn_marais_shore", "TOWN", 4, 0, &time)
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
                sample(map, tileset, environment, 0, 0, &time),
                Default::default()
            );
        }
        for map in ["MnR11C00R00", "MnGrandMarais", "MnMaraisCedarApproach"] {
            assert_eq!(
                sample(map, "mn_winter_outdoor", "ROUTE", 0, 0, &time).snow,
                0.0
            );
        }
        assert_eq!(
            sample("MnEly", "mn_ely_snow", "TOWN", 10, 0, &time).snow,
            1.0
        );
        assert_eq!(
            sample(
                "MnElyShorePass",
                "mn_winter_north_shore",
                "ROUTE",
                0,
                0,
                &time
            )
            .snow,
            1.0
        );
        assert_eq!(
            sample(
                "MnElyShorePass",
                "mn_winter_north_shore",
                "ROUTE",
                12,
                0,
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
            let warm = sample("MnGrandMarais", "mn_marais_shore", "TOWN", 0, 0, &time);
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
                    assert_eq!(sample(map, tileset, "ROUTE", 0, 0, &time), warm);
                }
            }
        }
        assert!(
            showers > 0,
            "the snow exclusion must retain occasional rain"
        );
    }

    #[test]
    fn inland_belt_snow_replaces_showers_without_freezing_preserve_or_coast() {
        let mut time = TimeState::default();
        for hour in 0..24 {
            time.registers.hours = hour;
            time.registers.minutes = 1;
            let coast = sample("MnGrandMarais", "mn_marais_shore", "TOWN", 0, 0, &time);
            for (map, tileset) in [
                ("MnBemidji", "mn_winter_bemidji"),
                ("MnBemidjiNorthwoods", "mn_winter_bemidji_northwoods"),
                ("MnGrandRapids", "mn_winter_grand_rapids"),
                ("MnR09C00R00", "mn_winter_iron_range"),
                ("MnR17C00R00", "mn_winter_border_pines"),
                ("MnR17C00R02", "mn_winter_border_pines_south"),
            ] {
                let inland = sample(map, tileset, "ROUTE", 4, 0, &time);
                assert_eq!(inland.snow, 1.0, "{map}");
                assert_eq!(inland.rain, 0.0, "{map}");
                assert_eq!(
                    sample(map, tileset, "INDOOR", 4, 0, &time),
                    Default::default()
                );
                for green in [
                    "MnBoundaryWaters",
                    "MnBoundarySafariC00R00",
                    "MnPreservePineEntry",
                    "MnR11C00R00",
                    "MnGrandMarais",
                    "MnMaraisCedarApproach",
                ] {
                    assert_eq!(sample(green, tileset, "ROUTE", 4, 0, &time), coast);
                }
            }
            assert_eq!(
                sample("MnBemidji", "mn_winter_bemidji_extra", "TOWN", 0, 0, &time),
                coast
            );
        }
    }

    #[test]
    fn tamarack_thaws_within_both_eastern_chunks_and_matches_the_real_seams() {
        let mut time = TimeState::default();
        for hour in 0..24 {
            time.registers.hours = hour;
            time.registers.minutes = 1;
            let preserve = sample(
                "MnBoundaryWaters",
                "mn_boundary_safari",
                "ROUTE",
                0,
                0,
                &time,
            );
            let west = sample("MnR20C00R00", "mn_winter_tamarack", "ROUTE", 63, 0, &time);
            assert_eq!(west.snow, 1.0);
            assert_eq!(west.rain, 0.0);
            for x in 0..64 {
                let north = sample("MnR20C01R00", "mn_tamarack_thaw", "ROUTE", x, 0, &time);
                let south = sample("MnR20C01R01", "mn_tamarack_thaw", "ROUTE", x, 0, &time);
                assert_eq!(north, south, "vertical seam at x{x}");
                if x <= 14 {
                    assert_eq!(north, west);
                }
                if x == 26 {
                    assert_eq!(north.snow, 0.5);
                }
                if x >= 38 {
                    assert_eq!(north, preserve);
                }
                assert!((north.rain - preserve.rain * (1.0 - north.snow)).abs() < f32::EPSILON);
            }
            for alias in ["mn_winter_tamarack", "mn_tamarack_thaw"] {
                assert_eq!(
                    sample("MnBoundaryWaters", alias, "ROUTE", 0, 0, &time),
                    preserve
                );
                assert_eq!(
                    sample("MnBoundarySafariC00R00", alias, "ROUTE", 0, 0, &time),
                    preserve
                );
            }
        }
    }

    #[test]
    fn iron_descent_thaws_before_its_green_superior_chunk() {
        let mut time = TimeState::default();
        for hour in 0..24 {
            time.registers.hours = hour;
            time.registers.minutes = 1;
            let cold = sample(
                "MnGrandRapids",
                "mn_winter_grand_rapids",
                "TOWN",
                0,
                0,
                &time,
            );
            let coast = sample("MnDuluth", "mn_duluth", "TOWN", 0, 0, &time);
            assert_eq!(
                sample("MnR10C00R00", "mn_iron_descent_thaw", "ROUTE", 0, 0, &time),
                cold
            );
            let mut previous = 1.0;
            for x in 0..96 {
                let (map, local_x) = if x < 64 {
                    ("MnR10C00R00", x)
                } else {
                    ("MnR10C01R00", x - 64)
                };
                let weather = sample(map, "mn_iron_descent_thaw", "ROUTE", local_x, 0, &time);
                assert!(weather.snow <= previous);
                previous = weather.snow;
                if x <= 30 {
                    assert_eq!(weather, cold);
                }
                if x == 47 {
                    assert_eq!(weather.snow, 0.5);
                }
                if x >= 64 {
                    assert_eq!(weather, coast);
                }
                assert!((weather.rain - coast.rain * (1.0 - weather.snow)).abs() < f32::EPSILON);
                assert_eq!(
                    sample(map, "mn_iron_descent_thaw", "CAVE", local_x, 0, &time),
                    Default::default()
                );
            }
            for map in [
                "MnBoundaryWaters",
                "MnBoundarySafariC00R00",
                "MnGrandMarais",
                "MnR11C00R00",
            ] {
                assert_eq!(
                    sample(map, "mn_iron_descent_thaw", "ROUTE", 0, 0, &time),
                    coast
                );
            }
        }
    }

    #[test]
    fn headwaters_thaws_southward_across_both_lake_portages() {
        let mut time = TimeState::default();
        for hour in 0..24 {
            time.registers.hours = hour;
            time.registers.minutes = 7;
            let warm = sample("MnLeechLake", "mn_leech_lake", "TOWN", 0, 0, &time);
            let bemidji = sample("MnBemidji", "mn_winter_bemidji", "TOWN", 0, 0, &time);
            for (map, alias) in [
                ("MnR08C00R00", "mn_winter_headwaters_stairs"),
                ("MnR08C01R00", "mn_winter_headwaters"),
            ] {
                assert_eq!(sample(map, alias, "ROUTE", 0, 47, &time), bemidji);
            }
            for local_y in [0, 6, 19, 32, 47] {
                let west = sample(
                    "MnR08C00R01",
                    "mn_headwaters_thaw",
                    "ROUTE",
                    63,
                    local_y,
                    &time,
                );
                let east = sample(
                    "MnR08C01R01",
                    "mn_headwaters_thaw",
                    "ROUTE",
                    0,
                    local_y,
                    &time,
                );
                assert_eq!(west, east, "same latitude on either bank");
                if local_y <= 6 {
                    assert_eq!(west, bemidji);
                }
                if local_y == 19 {
                    assert_eq!(west.snow, 0.5);
                }
                if local_y >= 32 {
                    assert_eq!(west, warm);
                }
                assert!((west.rain - warm.rain * (1.0 - west.snow)).abs() < f32::EPSILON);
                assert_eq!(
                    sample(
                        "MnR08C00R01",
                        "mn_headwaters_thaw",
                        "CAVE",
                        0,
                        local_y,
                        &time
                    ),
                    Default::default()
                );
            }
            for map in [
                "MnBoundaryWaters",
                "MnBoundarySafariC00R00",
                "MnGrandMarais",
                "MnR11C00R00",
            ] {
                assert_eq!(
                    sample(map, "mn_winter_headwaters", "ROUTE", 0, 0, &time),
                    warm
                );
            }
        }
    }

    #[test]
    fn itasca_thaws_westward_without_a_weather_jump_at_the_eastern_chunk_seam() {
        let mut time = TimeState::default();
        for hour in 0..24 {
            time.registers.hours = hour;
            time.registers.minutes = 7;
            let warm = sample("MnLakeItasca", "mn_headwaters", "TOWN", 0, 0, &time);
            let snowy = sample("MnBemidji", "mn_winter_bemidji", "TOWN", 0, 0, &time);
            for y in [0, 15, 47] {
                for x in [0, 34, 46, 58, 63] {
                    let north = sample("MnR14C00R00", "mn_itasca_thaw", "ROUTE", x, y, &time);
                    let south = sample("MnR14C00R01", "mn_itasca_thaw", "ROUTE", x, y, &time);
                    assert_eq!(north, south);
                    if x <= 34 {
                        assert_eq!(north, warm);
                    }
                    if x == 46 {
                        assert_eq!(north.snow, 0.5);
                    }
                    if x >= 58 {
                        assert_eq!(north, snowy);
                    }
                    assert!((north.rain - warm.rain * (1.0 - north.snow)).abs() < f32::EPSILON);
                }
                for map in ["MnR14C01R00", "MnR14C01R01"] {
                    assert_eq!(
                        sample(map, "mn_winter_itasca_trail", "ROUTE", 0, y, &time),
                        snowy
                    );
                }
            }
            assert_eq!(
                sample("MnItascaGatehouse", "mn_itasca_thaw", "GATE", 60, 0, &time),
                Default::default()
            );
            for map in [
                "MnBoundaryWaters",
                "MnBoundarySafariC00R00",
                "MnGrandMarais",
                "MnR11C00R00",
            ] {
                assert_eq!(
                    sample(map, "mn_winter_itasca_trail", "ROUTE", 63, 0, &time),
                    warm
                );
            }
        }
    }
}
