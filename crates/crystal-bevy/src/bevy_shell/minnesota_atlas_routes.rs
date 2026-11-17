//! Route cards resolve real regional maps even when a town and a road share
//! Crystal's seven-bit caught-location ID. Cursor indices are presentation-only.
use std::collections::{BTreeMap, BTreeSet};

use super::minnesota_town_map;
use crate::core::models::display_metadata::{PokegearLandmark, PokegearLandmarksPayload};

const TOWN_MAPS: [&str; minnesota_town_map::TOWNS.len()] = [
    "MnWinona",
    "MnRochester",
    "MnMankato",
    "MnMinneapolis",
    "MnStPaul",
    "MnStCloud",
    "MnBrainerd",
    "MnLeechLake",
    "MnBemidji",
    "MnGrandRapids",
    "MnDuluth",
    "MnGrandMarais",
    "MnBoundaryWaters",
    "MnEly",
    "MnLakeItasca",
    "MnDetroitLakes",
    "MnMoorhead",
    "MnInternationalFalls",
    "MnMilleLacs",
];

// Walking endpoints, in Route 1–20 order. The North Shore uses the coastal curve.
pub(super) const CONNECTIONS: [(usize, usize); 20] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (3, 5),
    (5, 6),
    (6, 7),
    (7, 8),
    (8, 9),
    (9, 10),
    (10, 11),
    (11, 12),
    (12, 13),
    (8, 14),
    (6, 15),
    (15, 16),
    (8, 17),
    (6, 18),
    (14, 15),
    (17, 12),
];

pub(super) struct Stop {
    pub(super) cursor: usize,
    pub(super) landmark: PokegearLandmark,
    pub(super) route: Option<usize>,
}

pub(super) fn endpoints(number: usize) -> (&'static str, &'static str) {
    let (a, b) = CONNECTIONS[number - 1];
    (
        minnesota_town_map::TOWNS[a].0,
        minnesota_town_map::TOWNS[b].0,
    )
}

pub(super) fn destination_footer(number: usize, origin: &str) -> String {
    let (a, b) = endpoints(number);
    let destination = if origin == b { a } else { b };
    let short = if destination == "INTERNATIONAL FALLS" {
        "INT'L FALLS"
    } else {
        destination
    };
    format!("TO {short}")
}

fn add(
    stops: &mut Vec<Stop>,
    seen: &mut BTreeSet<usize>,
    cursor: usize,
    base: &PokegearLandmark,
    name: &str,
    point: (i16, i16),
    route: Option<usize>,
) {
    if !seen.insert(cursor) {
        return;
    }
    let mut landmark = base.clone();
    landmark.name = name.to_owned();
    landmark.x = point.0 + 8;
    landmark.y = point.1 + 16;
    stops.push(Stop {
        cursor,
        landmark,
        route,
    });
}

pub(super) fn stops(catalog: &PokegearLandmarksPayload) -> Vec<Stop> {
    let catalog_indices: BTreeMap<_, _> = catalog
        .landmarks
        .iter()
        .enumerate()
        .map(|(i, l)| (l.constant.as_str(), i))
        .collect();
    let town_slots = TOWN_MAPS.map(|map| {
        catalog
            .map_to_landmark
            .get(map)
            .and_then(|id| catalog_indices.get(id.as_str()).copied())
    });
    let mut seen = BTreeSet::new();
    let mut stops = Vec::new();
    let town = |i: usize, stops: &mut Vec<Stop>, seen: &mut BTreeSet<usize>| {
        if let Some(index) = town_slots[i] {
            let name = minnesota_town_map::TOWNS[i].0;
            if let Some(point) = minnesota_town_map::town_position(name) {
                add(
                    stops,
                    seen,
                    index,
                    &catalog.landmarks[index],
                    name,
                    point,
                    None,
                );
            }
        }
    };
    for (i, &(a, b)) in CONNECTIONS.iter().enumerate() {
        town(a, &mut stops, &mut seen);
        let number = i + 1;
        let map = format!("MnR{number:02}C00R00");
        if let Some(&index) = catalog
            .map_to_landmark
            .get(&map)
            .and_then(|id| catalog_indices.get(id.as_str()))
        {
            let landmark = &catalog.landmarks[index];
            let cursor = if town_slots.contains(&Some(index)) {
                catalog.landmarks.len() + i
            } else {
                index
            };
            let point = minnesota_town_map::position(
                &landmark.constant,
                &landmark.name,
                landmark.x - 8,
                landmark.y - 16,
            );
            add(
                &mut stops,
                &mut seen,
                cursor,
                landmark,
                &landmark.name,
                point,
                Some(number),
            );
        }
        town(b, &mut stops, &mut seen);
    }
    for (map, id) in &catalog.map_to_landmark {
        if !map.starts_with("Mn") || map.contains("Review") || map.contains("Gallery") {
            continue;
        }
        if let Some(&index) = catalog_indices.get(id.as_str())
            && !seen.contains(&index)
        {
            let landmark = &catalog.landmarks[index];
            let point =
                minnesota_town_map::position(id, &landmark.name, landmark.x - 8, landmark.y - 16);
            add(
                &mut stops,
                &mut seen,
                index,
                landmark,
                &landmark.name,
                point,
                None,
            );
        }
    }
    stops
}

pub(super) fn player_cursor(stops: &[Stop], map: &str, constant: &str) -> Option<usize> {
    // Route interiors and continuation chunks inherit their road's position.
    let number = map
        .strip_prefix("MnR")
        .and_then(|s| s.get(..2))
        .and_then(|s| s.parse::<usize>().ok());
    stops
        .iter()
        .find(|s| s.route.is_some() && s.route == number && s.landmark.constant == constant)
        .or_else(|| {
            stops
                .iter()
                .find(|s| s.route.is_none() && s.landmark.constant == constant)
        })
        .or_else(|| stops.iter().find(|s| s.landmark.constant == constant))
        .map(|s| s.cursor)
}

pub(super) fn selected_and_current(
    stops: &[Stop],
    map: &str,
    constant: &str,
    cursor: usize,
) -> Option<(PokegearLandmark, PokegearLandmark, Option<usize>)> {
    let selected = stops.iter().find(|s| s.cursor == cursor)?;
    let here = player_cursor(stops, map, constant)?;
    let current = stops.iter().find(|s| s.cursor == here)?;
    Some((
        selected.landmark.clone(),
        current.landmark.clone(),
        selected.route,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn town_and_route_sharing_a_caught_location_remain_separate_stops() {
        let landmark = PokegearLandmark {
            id: 100,
            constant: "LANDMARK_MN_TWIN_CITIES".into(),
            label: "TWIN_CITIES".into(),
            name: "TWIN CITIES PARK".into(),
            x: 48,
            y: 50,
            region: "JOHTO".into(),
        };
        let catalog = PokegearLandmarksPayload {
            landmarks: vec![landmark.clone()],
            map_to_landmark: BTreeMap::from([
                ("MnStPaul".into(), landmark.constant.clone()),
                ("MnR04C00R00".into(), landmark.constant.clone()),
            ]),
        };
        let stops = stops(&catalog);
        let road = player_cursor(&stops, "MnR04C00R00", &landmark.constant).expect("road");
        let town = player_cursor(&stops, "MnStPaulGym", &landmark.constant).expect("town");
        assert_ne!(road, town);
        assert_eq!(
            stops
                .iter()
                .find(|s| s.cursor == road)
                .expect("stop")
                .landmark
                .name,
            "TWIN CITIES PARK"
        );
        assert_eq!(
            stops
                .iter()
                .find(|s| s.cursor == town)
                .expect("stop")
                .landmark
                .name,
            "ST PAUL"
        );
        assert_eq!(destination_footer(4, "ST PAUL"), "TO MINNEAPOLIS");
        for number in 1..=CONNECTIONS.len() {
            let (a, b) = endpoints(number);
            assert!(
                destination_footer(number, a).len() <= 18
                    && destination_footer(number, b).len() <= 18
            );
        }
    }

    #[test]
    #[ignore = "requires CRYSTAL_IRON_TARGET; reads the compiled landmark catalog only"]
    fn compiled_atlas_has_every_town_and_route_in_campaign_order() {
        let path = std::env::var_os("CRYSTAL_IRON_TARGET").expect("compiled Minnesota pack");
        let pack = crate::assets::read_verified_compiled_game_pack(std::path::PathBuf::from(path))
            .expect("verified pack");
        let catalog = &pack.data().pokegear_landmarks;
        let stops = stops(catalog);
        let mut distinct = BTreeSet::new();
        for (map, &(town, _, _)) in TOWN_MAPS.into_iter().zip(minnesota_town_map::TOWNS) {
            let cursor =
                player_cursor(&stops, map, &catalog.map_to_landmark[map]).expect("town cursor");
            let stop = stops.iter().find(|s| s.cursor == cursor).expect("town");
            assert!(distinct.insert(cursor));
            assert_eq!(stop.landmark.name, town);
            assert_eq!(
                Some((stop.landmark.x - 8, stop.landmark.y - 16)),
                minnesota_town_map::town_position(town)
            );
        }
        let mut previous = None;
        for number in 1..=20 {
            let map = format!("MnR{number:02}C00R00");
            let cursor =
                player_cursor(&stops, &map, &catalog.map_to_landmark[&map]).expect("road cursor");
            let (index, stop) = stops
                .iter()
                .enumerate()
                .find(|(_, s)| s.cursor == cursor)
                .expect("road");
            assert!(distinct.insert(cursor));
            assert_eq!(stop.route, Some(number));
            assert!(previous.is_none_or(|old| old < index));
            previous = Some(index);
        }
        assert_eq!(distinct.len(), 39);
    }
}
