use std::collections::BTreeSet;

use super::minimum_grass_cost_with_ice;

#[test]
fn ice_crossing_counts_required_grass_on_the_landing_route() {
    let walkable = BTreeSet::from([[0, 0], [2, 0]]);
    let cost = minimum_grass_cost_with_ice([0, 0], [2, 0], &walkable, [2, 0, 3, 1], [3, 1], |at| {
        Ok(match at {
            [1, 0] => "ICE",
            [2, 0] => "TALL_GRASS",
            _ => "FLOOR",
        })
    })
    .expect("ice crossing graph");
    assert_eq!(cost, Some(1));
}

#[test]
fn ice_crossing_does_not_allow_a_turn_on_transient_ice() {
    let walkable = BTreeSet::from([[0, 0], [2, 0], [1, 1]]);
    let cost = minimum_grass_cost_with_ice([0, 0], [1, 1], &walkable, [1, 1, 2, 2], [3, 2], |at| {
        Ok(match at {
            [1, 0] => "ICE",
            [0, 0] | [2, 0] => "FLOOR",
            [1, 1] => "TALL_GRASS",
            _ => "WALL",
        })
    })
    .expect("no transient ice turns");
    assert_eq!(cost, None);
}

#[test]
fn ice_crossing_still_detects_a_dry_grass_bypass() {
    let walkable = BTreeSet::from([[0, 0], [2, 0], [3, 0], [0, 1], [1, 1], [2, 1], [3, 1]]);
    let cost = minimum_grass_cost_with_ice([0, 0], [3, 0], &walkable, [2, 0, 3, 1], [4, 2], |at| {
        Ok(match at {
            [1, 0] => "ICE",
            [2, 0] => "TALL_GRASS",
            _ => "FLOOR",
        })
    })
    .expect("dry alternate path");
    assert_eq!(cost, Some(0));
}
