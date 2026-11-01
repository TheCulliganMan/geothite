//! Exact art ancestry and icy encounter blocks for the inland winter belt.

/// Return only an authored inland alias's original presentation source.
pub(super) fn source(tileset: &str) -> Option<&'static str> {
    match tileset {
        "mn_winter_bemidji" => Some("mn_bemidji_plateau_stairs"),
        "mn_winter_bemidji_northwoods" => Some("johto"),
        "mn_winter_grand_rapids" => Some("mn_grand_rapids_riverfront"),
        "mn_winter_iron_range" => Some("mn_iron_range"),
        "mn_winter_border_pines" => Some("mn_winter_outdoor"),
        "mn_winter_border_pines_south" => Some("mn_border_pines"),
        "mn_winter_tamarack" | "mn_tamarack_thaw" => Some("mn_headwaters"),
        "mn_winter_iron_descent" | "mn_iron_descent_thaw" => Some("mn_r10_rockstairs"),
        "mn_winter_headwaters" | "mn_headwaters_thaw" => Some("mn_headwaters"),
        "mn_winter_headwaters_stairs" => Some("mn_outdoor_headwaters_stairs"),
        "mn_winter_itasca_trail" | "mn_itasca_thaw" => Some("mn_headwaters"),
        _ => None,
    }
}

/// One complete icy metatile supplies the authored encounter effect's colors.
pub(super) fn grass_block(tileset: &str) -> Option<usize> {
    match tileset {
        "mn_winter_bemidji_northwoods" | "mn_winter_iron_range" => Some(0x03),
        "mn_winter_tamarack" => Some(0x03),
        "mn_tamarack_thaw" => Some(0x88),
        "mn_winter_iron_descent" => Some(0x03),
        "mn_iron_descent_thaw" => Some(0x88),
        "mn_winter_headwaters" | "mn_winter_headwaters_stairs" => Some(0x03),
        "mn_headwaters_thaw" => Some(0x88),
        "mn_winter_itasca_trail" => Some(0x03),
        "mn_itasca_thaw" => Some(0x88),
        "mn_winter_border_pines" => Some(0x3d),
        "mn_winter_border_pines_south" => Some(0x37),
        _ => None,
    }
}

/// Border Pines recolors both of its native encounter blocks, not just the new one.
pub(super) fn is_icy_grass(tileset: &str, block: Option<u16>) -> bool {
    if tileset == "mn_winter_border_pines" {
        matches!(block, Some(0x3d | 0x37))
    } else if matches!(tileset, "mn_tamarack_thaw" | "mn_itasca_thaw") {
        matches!(block, Some(0x88 | 0xb1))
    } else if tileset == "mn_headwaters_thaw" {
        matches!(block, Some(0x88 | 0x9f))
    } else if tileset == "mn_winter_iron_descent" {
        matches!(block, Some(0x03 | 0x81 | 0x83 | 0x84))
    } else if tileset == "mn_iron_descent_thaw" {
        matches!(
            block,
            Some(0x88 | 0xac | 0xa5 | 0xa7 | 0xa8 | 0xc9 | 0xcb | 0xcc)
        )
    } else {
        grass_block(tileset).is_some_and(|icy| u16::try_from(icy).ok() == block)
    }
}

/// Native Cut removes the grass on these authored south cliff crowns. Their
/// FLOOR/FLOOR/WALL/WALL shape is also used by building eaves, but the cleared
/// upper floor must not acquire an eave's extra player-occluding foreground.
pub(super) fn is_cleared_cliff_floor(tileset: &str, block: u16) -> bool {
    matches!(
        (tileset, block),
        ("mn_winter_iron_descent", 0x88) | ("mn_iron_descent_thaw", 0xd0 | 0xd3 | 0xd6)
    )
}

/// Continue an authored thaw forest beyond the camera's finite map boundary.
///
/// Source coordinates use 8px glyphs, four per metatile. Only pure forest
/// variants qualify; gameplay collision and nonforest borders keep their
/// existing selection. Missing or malformed map data has no override.
pub(super) fn forest_border_block(
    tileset: &str,
    width: u16,
    height: u16,
    blocks: &[u16],
    x: i32,
    y: i32,
) -> Option<u16> {
    let forest = match tileset {
        "mn_tamarack_thaw" | "mn_itasca_thaw" => [0x05, 0x8a, 0xb3],
        "mn_iron_descent_thaw" => [0x05, 0x8a, 0xae],
        "mn_headwaters_thaw" => [0x05, 0x8a, 0xa1],
        _ => return None,
    };
    if width == 0 || height == 0 {
        return None;
    }
    let column = x.div_euclid(4);
    let row = y.div_euclid(4);
    if (0..i32::from(width)).contains(&column) && (0..i32::from(height)).contains(&row) {
        return None;
    }
    let column = usize::try_from(column.clamp(0, i32::from(width) - 1)).ok()?;
    let row = usize::try_from(row.clamp(0, i32::from(height) - 1)).ok()?;
    let index = row.checked_mul(usize::from(width))?.checked_add(column)?;
    let block = *blocks.get(index)?;
    forest.contains(&block).then_some(block)
}

#[cfg(test)]
mod tests {
    use super::{forest_border_block, is_icy_grass};

    #[test]
    fn headwaters_forest_and_grass_follow_the_southward_thaw() {
        for block in [0x88, 0x9f] {
            assert!(is_icy_grass("mn_headwaters_thaw", Some(block)));
        }
        for block in [0x03, 0x87, 0x9e] {
            assert!(!is_icy_grass("mn_headwaters_thaw", Some(block)));
        }
        let edge = [0x8a, 0xa1, 0x05];
        for (y, expected) in [(0, 0x8a), (4, 0xa1), (8, 0x05)] {
            assert_eq!(
                forest_border_block("mn_headwaters_thaw", 1, 3, &edge, -1, y),
                Some(expected)
            );
            assert_eq!(
                forest_border_block("mn_headwaters_thaw", 1, 3, &edge, 4, y),
                Some(expected)
            );
        }
    }

    #[test]
    fn descent_corner_grass_and_forest_keep_their_actual_climate() {
        for block in [0x88, 0xac, 0xa5, 0xa7, 0xa8, 0xc9, 0xcb, 0xcc] {
            assert!(is_icy_grass("mn_iron_descent_thaw", Some(block)));
        }
        for block in [0x03, 0x81, 0x83, 0x84] {
            assert!(!is_icy_grass("mn_iron_descent_thaw", Some(block)));
        }
        for block in [0x8a, 0xae, 0x05] {
            assert_eq!(
                forest_border_block("mn_iron_descent_thaw", 1, 1, &[block], 0, -1),
                Some(block)
            );
        }
        assert_eq!(
            forest_border_block("mn_iron_descent_thaw", 1, 1, &[0xb3], 0, -1),
            None
        );
    }

    #[test]
    fn off_map_forest_follows_cold_melt_and_green_edges() {
        let forest = [0x8a, 0xb3, 0x05, 0x8a, 0xb3, 0x05];
        for y in [-4, -1, 8, 11] {
            assert_eq!(
                forest_border_block("mn_tamarack_thaw", 3, 2, &forest, 0, y),
                Some(0x8a)
            );
            assert_eq!(
                forest_border_block("mn_tamarack_thaw", 3, 2, &forest, 7, y),
                Some(0xb3)
            );
            assert_eq!(
                forest_border_block("mn_tamarack_thaw", 3, 2, &forest, 11, y),
                Some(0x05)
            );
        }
        assert_eq!(
            forest_border_block("mn_tamarack_thaw", 3, 2, &forest, -1, 4),
            Some(0x8a)
        );
        assert_eq!(
            forest_border_block("mn_tamarack_thaw", 3, 2, &forest, 12, 4),
            Some(0x05)
        );
        assert_eq!(
            forest_border_block("mn_tamarack_thaw", 3, 2, &forest, 4, 4),
            None
        );
        assert_eq!(
            forest_border_block("mn_headwaters", 3, 2, &forest, 4, -1),
            None
        );
        assert_eq!(
            forest_border_block("mn_tamarack_thaw", 0, 2, &forest, 4, -1),
            None
        );
        assert_eq!(
            forest_border_block("mn_tamarack_thaw", 3, 0, &forest, 4, -1),
            None
        );
        assert_eq!(
            forest_border_block("mn_tamarack_thaw", 3, 2, &[], 4, -1),
            None
        );
        assert_eq!(
            forest_border_block("mn_tamarack_thaw", 1, 1, &[0x00], 0, -1),
            None
        );
    }
}
