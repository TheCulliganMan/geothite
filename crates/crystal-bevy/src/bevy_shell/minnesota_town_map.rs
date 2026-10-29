//! A one-screen Minnesota composition made exclusively from native Johto tiles.

// Native LCD centers of each town tile. The Town Map and Fly selector project
// their OAM coordinates onto these same positions.
const TOWNS: &[(&str, i16, i16)] = &[
    ("WINONA", 124, 116),
    ("ROCHESTER", 100, 116),
    ("MANKATO", 60, 116),
    ("MINNEAPOLIS", 76, 108),
    ("ST PAUL", 100, 108),
    ("ST CLOUD", 60, 100),
    ("BRAINERD", 68, 84),
    ("LEECH LAKE", 68, 68),
    ("BEMIDJI", 52, 52),
    ("GRAND RAPIDS", 92, 68),
    ("DULUTH", 108, 76),
    ("GRAND MARAIS", 140, 44),
    ("BOUNDARY WATERS", 116, 36),
    ("ELY", 108, 52),
    ("LAKE ITASCA", 36, 68),
    ("DETROIT LAKES", 44, 84),
    ("MOORHEAD", 20, 84),
    ("INTERNATIONAL FALLS", 76, 36),
    ("MILLE LACS", 92, 92),
];

pub(super) fn town_position(name: &str) -> Option<(i16, i16)> {
    let name = name.replace('_', " ");
    TOWNS
        .iter()
        .find(|(town, _, _)| name == *town)
        .map(|(_, x, y)| (*x, *y))
}

/// Fly map constants may include terrain suffixes such as LEECH_LAKE_SHORE.
/// Keep the selector's displayed town name consistent with the atlas.
pub(super) fn fly_town_name(label: &str) -> Option<&'static str> {
    TOWNS.iter().find_map(|(town, _, _)| {
        label
            .strip_prefix(town)
            .filter(|suffix| suffix.is_empty() || suffix.starts_with(' '))
            .map(|_| *town)
    })
}

pub(super) fn position(constant: &str, name: &str, old_x: i16, old_y: i16) -> (i16, i16) {
    if let Some(town) = town_position(name) {
        return town;
    }
    let cell = match constant.strip_prefix("LANDMARK_MN_").unwrap_or(constant) {
        "DRIFTLESS" => (14, 14),
        "PRAIRIE_FARM" => (10, 14),
        "RIVER_VALLEY" => (7, 13),
        "TWIN_CITIES" => (11, 13),
        "RIVER_BLUFFS" => (9, 12),
        "PINE_COUNTRY" => (7, 11),
        "NORTHERN_LAKES" => (8, 7),
        "IRON_RANGE" => (9, 8),
        "IRON_RANGE_DESCENT" => (12, 9),
        "NORTH_SHORE" => (15, 7),
        "GUNFLINT" => (16, 4),
        "LEAGUE_APPROACH" => (14, 5),
        "HEADWATERS" => (6, 8),
        "CENTRAL_LAKES" => (6, 10),
        "RED_RIVER_PRAIRIE" => (3, 10),
        "BORDER_PINES" => (6, 4),
        "MILLE_LACS_SHORE" => (9, 11),
        "PRAIRIE_FOREST_LINK" => (4, 9),
        "VOYAGEURS_BORDER_LOOP" => (11, 4),
        "BOUNDARY_SAFARI" | "BOUNDARY_BASECAMP" => (14, 4),
        _ => {
            return (
                20 + old_x.clamp(0, 160) * 112 / 160,
                36 + old_y.clamp(0, 144) * 80 / 144,
            );
        }
    };
    (cell.0 * 8 + 4, cell.1 * 8 + 4)
}

/// Native 20x18 tile IDs, consumed by the existing paletted Johto renderer.
/// No source pixels or palettes are copied into the repository.
pub(super) fn tilemap() -> Vec<u8> {
    let mut tiles = vec![0x1d; 20 * 18];
    // Hand-shaped state silhouette: Northwest Angle, northern border,
    // arrowhead, Lake Superior shore, and the southeastern river bluff.
    for (row, left, right) in [
        (3, 4, 5),
        (4, 2, 15),
        (5, 2, 18),
        (6, 2, 17),
        (7, 2, 16),
        (8, 2, 15),
        (9, 2, 15),
        (10, 2, 14),
        (11, 2, 14),
        (12, 2, 14),
        (13, 2, 15),
        (14, 2, 16),
        (15, 2, 17),
    ] {
        for col in left..right {
            tiles[row * 20 + col] = 0x11;
        }
    }
    // Johto's mountain banks, lake tiles and shoreline corners.
    for (col, row, tile) in [
        (11, 5, 0x03),
        (12, 5, 0x04),
        (13, 5, 0x04),
        (14, 5, 0x05),
        (11, 6, 0x13),
        (12, 6, 0x14),
        (13, 6, 0x14),
        (14, 6, 0x15),
        (11, 7, 0x23),
        (12, 7, 0x24),
        (13, 7, 0x24),
        (14, 7, 0x25),
        (17, 6, 0x08),
        (18, 6, 0x18),
        (16, 7, 0x08),
        (17, 7, 0x08),
        (18, 7, 0x18),
        (15, 8, 0x08),
        (16, 8, 0x08),
        (17, 8, 0x08),
        (18, 8, 0x18),
        (14, 9, 0x08),
        (15, 9, 0x08),
        (16, 9, 0x08),
        (17, 9, 0x08),
        (18, 9, 0x18),
        (14, 10, 0x08),
        (15, 10, 0x08),
        (16, 10, 0x08),
        (17, 10, 0x18),
        (6, 7, 0x08),
        (7, 7, 0x18),
        (9, 5, 0x08),
        (10, 5, 0x18),
        (10, 10, 0x08),
        (11, 10, 0x18),
    ] {
        tiles[row * 20 + col] = tile;
    }
    // Pale native land corridors connect town squares, as on the Johto map.
    let mut roads = [[false; 20]; 18];
    for (a, b) in [
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
    ] {
        if (a, b) == (10, 11) {
            continue;
        }
        let (_, ax, ay) = TOWNS[a];
        let (_, bx, by) = TOWNS[b];
        let (ax, ay, bx, by) = (
            ax as usize / 8,
            ay as usize / 8,
            bx as usize / 8,
            by as usize / 8,
        );
        for row in &mut roads[ay.min(by)..=ay.max(by)] {
            row[ax] = true;
        }
        roads[by][ax.min(bx)..=ax.max(bx)].fill(true);
    }
    // The North Shore climbs diagonally along Superior instead of drawing a
    // misleading inland right-angle shortcut through the Iron Range.
    for (col, row) in [
        (13, 9),
        (13, 8),
        (14, 8),
        (14, 7),
        (15, 7),
        (15, 6),
        (16, 6),
        (16, 5),
        (17, 5),
    ] {
        roads[row][col] = true;
    }
    for (row, road_row) in roads.iter().enumerate().take(16).skip(3) {
        for (col, road) in road_row.iter().enumerate().take(19).skip(1) {
            if !road {
                continue;
            }
            tiles[row * 20 + col] = 0x1d;
        }
    }
    for &(_, x, y) in TOWNS {
        tiles[y as usize / 8 * 20 + x as usize / 8] = 0x0a;
    }
    for (col, row) in [(11, 4), (15, 7), (11, 13)] {
        tiles[row * 20 + col] = 0x0c;
    }
    // The original frame and header are retained; the UI writes the live name.
    tiles[0] = 0x06;
    tiles[19] = 0x17;
    tiles[1..19].fill(0x07);
    tiles[17 * 20 + 1..17 * 20 + 19].fill(0x07);
    for row in 1..17 {
        tiles[row * 20] = 0x16;
        tiles[row * 20 + 19] = 0x16;
    }
    tiles[17 * 20] = 0x26;
    tiles[17 * 20 + 19] = 0x27;
    tiles
}

#[cfg(test)]
mod tests {
    use super::{TOWNS, tilemap, town_position};
    #[test]
    fn fly_labels_resolve_map_constant_suffixes() {
        assert_eq!(super::fly_town_name("LEECH LAKE SHORE"), Some("LEECH LAKE"));
        assert_eq!(super::fly_town_name("MILLE LACS PARK"), Some("MILLE LACS"));
        assert_eq!(super::fly_town_name("GRAND MARAIS"), Some("GRAND MARAIS"));
        assert_eq!(super::fly_town_name("NORTH SHORE"), None);
    }

    #[test]
    fn route_cursors_stay_on_native_travel_corridors() {
        let tiles = tilemap();
        for route in [
            "DRIFTLESS",
            "PRAIRIE_FARM",
            "RIVER_VALLEY",
            "TWIN_CITIES",
            "RIVER_BLUFFS",
            "PINE_COUNTRY",
            "NORTHERN_LAKES",
            "IRON_RANGE",
            "IRON_RANGE_DESCENT",
            "NORTH_SHORE",
            "GUNFLINT",
            "LEAGUE_APPROACH",
            "HEADWATERS",
            "CENTRAL_LAKES",
            "RED_RIVER_PRAIRIE",
            "BORDER_PINES",
            "MILLE_LACS_SHORE",
            "PRAIRIE_FOREST_LINK",
            "VOYAGEURS_BORDER_LOOP",
        ] {
            let (x, y) = super::position(&format!("LANDMARK_MN_{route}"), "", 0, 0);
            let tile = tiles[y as usize / 8 * 20 + x as usize / 8];
            assert!(
                matches!(tile, 0x1d | 0x0c | 0x0a),
                "{route}: tile {tile:#x}"
            );
        }
    }

    #[test]
    fn all_towns_and_selection_brackets_fit_one_lcd() {
        let tiles = tilemap();
        assert_eq!(tiles.len(), 20 * 18);
        assert!(tiles.iter().all(|tile| *tile < 0x30));
        let mut positions = std::collections::BTreeSet::new();
        for &(name, x, y) in TOWNS {
            assert!((16..=143).contains(&x) && (32..=119).contains(&y), "{name}");
            assert!(positions.insert((x, y)), "overlapping town {name}");
            assert_eq!(tiles[y as usize / 8 * 20 + x as usize / 8], 0x0a);
            assert_eq!(town_position(name), Some((x, y)));
        }
        if let Ok(directory) = std::env::var("CRYSTAL_TOWN_MAP_RENDER_DIR") {
            std::fs::create_dir_all(&directory).expect("render directory");
            std::fs::write(
                std::path::Path::new(&directory).join("minnesota-native.tilemap"),
                tiles,
            )
            .expect("save native tilemap verification input");
        }
    }
}
