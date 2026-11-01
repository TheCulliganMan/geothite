//! Safe encounter grass is a valid ledge approach/landing; water and warps are not.
use super::*;
use serde_json::json;

fn fixture() -> RouteDesign {
    let mut design = RouteDesign::parse(include_str!(
        "../../../planning/minnesota/revisions/northwoods-expansion/before/MN_R11.json"
    ))
    .expect("authored previous route");
    design.one_way_ledges = serde_json::from_value(json!([{
        "id":"GRASS_DROP","bounds_core":[4,4,6,6],"direction":"south","source_block":"57",
        "upper_approach":[4,3],"hop_from":[4,4],"landing":[4,6],"reverse_blocked":[4,5],
        "side_barriers":[[3,5],[6,5]],"return_passage_core":[8,2,10,8],"minimum_return_steps":9
    }]))
    .expect("explicit ledge contract");
    design
}

#[test]
fn authored_closed_high_cliff_requires_wall_and_safe_approaches() {
    let mut design = fixture();
    design.closed_high_cliffs = std::mem::take(&mut design.one_way_ledges);
    let collision = |tile: [u16; 2]| match tile {
        [4 | 5, 4] | [4, 3] | [4, 6] => "FLOOR",
        _ => "WALL",
    };
    validate_ledges(&design, |tile| Ok(collision(tile))).expect("closed retaining face");
    for (tile, replacement) in [
        ([4, 4], "HOP_DOWN"),
        ([4, 5], "FLOOR"),
        ([4, 3], "WATER"),
        ([4, 6], "DOOR"),
    ] {
        assert!(
            validate_ledges(&design, |at| Ok(if at == tile {
                replacement
            } else {
                collision(at)
            }))
            .is_err()
        );
    }
}

#[test]
fn authored_closed_high_cliff_still_requires_ordinary_return() {
    let mut design = fixture();
    design.one_way_ledges[0].return_passage_core = [8, 4, 9, 5];
    design.one_way_ledges[0].return_region_core = Some([4, 1, 10, 8]);
    design.closed_high_cliffs = std::mem::take(&mut design.one_way_ledges);
    let mut floor = walking_loops();
    validate_ledge_returns(&design, &floor, |_| Ok("FLOOR"))
        .expect("closed cliff retains the ordinary stair circuit");
    floor.remove(&[8, 4]);
    assert!(validate_ledge_returns(&design, &floor, |_| Ok("FLOOR")).is_err());
}

#[test]
fn authored_closed_high_cliff_dossier_cannot_remove_other_route_contracts() {
    let mut value: serde_json::Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/routes/MN_R10.json"
    ))
    .expect("current R10 authoring dossier");
    value["closed_high_cliffs"] = value["one_way_ledges"].take();
    value["one_way_ledges"] = json!([]);
    let encode = |value: &serde_json::Value| serde_json::to_string(value).expect("dossier JSON");
    RouteDesign::parse(&encode(&value)).expect("explicit closed carrier cliff");
    for (field, replacement) in [
        ("closed_high_cliffs", json!([])),
        ("required_flags", json!(["BADGE_7"])),
        ("required_grass_crossings", json!([])),
        ("id", json!("MN_R09")),
    ] {
        let mut invalid = value.clone();
        invalid[field] = replacement;
        assert!(RouteDesign::parse(&encode(&invalid)).is_err());
    }
    value["closed_high_cliffs"][0]["bounds_core"] = json!([22, 20, 28, 22]);
    assert!(RouteDesign::parse(&encode(&value)).is_err());
}
fn kind(tile: [u16; 2], approach: &'static str, landing: &'static str) -> &'static str {
    match tile {
        [4, 3] => approach,
        [4, 6] => landing,
        [4 | 5, 4] => "HOP_DOWN",
        _ => "WALL",
    }
}
#[test]
fn authored_ledges_accept_native_encounter_grass_on_both_sides() {
    let design = fixture();
    for approach in ["FLOOR", "TALL_GRASS"] {
        for landing in ["FLOOR", "TALL_GRASS"] {
            validate_ledges(&design, |tile| Ok(kind(tile, approach, landing)))
                .expect("safe native ground");
        }
    }
}
#[test]
fn authored_ledges_reject_water_walls_and_warp_triggers_on_either_side() {
    let design = fixture();
    for unsafe_kind in ["WATER", "WALL", "DOOR", "LADDER", "PIT", "WATERFALL"] {
        assert!(validate_ledges(&design, |tile| Ok(kind(tile, unsafe_kind, "FLOOR"))).is_err());
        assert!(validate_ledges(&design, |tile| Ok(kind(tile, "FLOOR", unsafe_kind))).is_err());
    }
}

#[test]
fn authored_ledge_side_guards_accept_permanent_trees_and_reject_cut_openings() {
    let design = fixture();
    for side in ["WALL", "WATER", "HEADBUTT_TREE", "HEADBUTT_TREE_1D"] {
        validate_ledges(&design, |tile| {
            Ok(if tile == [3, 5] {
                side
            } else {
                kind(tile, "FLOOR", "FLOOR")
            })
        })
        .expect("permanent native side barrier");
    }
    for side in [
        "FLOOR",
        "TALL_GRASS",
        "CUT_TREE",
        "DOOR",
        "LADDER",
        "HOP_DOWN",
    ] {
        assert!(
            validate_ledges(&design, |tile| {
                Ok(if tile == [3, 5] {
                    side
                } else {
                    kind(tile, "FLOOR", "FLOOR")
                })
            })
            .is_err(),
            "openable or passable {side} is not a permanent guard"
        );
    }
}

fn walking_loops() -> BTreeSet<[u16; 2]> {
    let mut floor = BTreeSet::new();
    // Local right-hand return, thirteen ordinary steps around the drop.
    floor.extend((4..=8).map(|x| [x, 6]));
    floor.extend((2..=6).map(|y| [8, y]));
    floor.extend((4..=8).map(|x| [x, 2]));
    floor.insert([4, 3]);
    // A distinct nine-step circuit outside the explicitly bounded local loop.
    floor.extend((1..=4).map(|x| [x, 6]));
    floor.extend((3..=6).map(|y| [1, y]));
    floor.extend((1..=4).map(|x| [x, 3]));
    floor
}

#[test]
fn authored_ledge_local_return_allows_a_nontrivial_regional_alternative() {
    let mut design = fixture();
    design.one_way_ledges[0].return_passage_core = [8, 4, 9, 5];
    let floor = walking_loops();
    assert!(validate_ledge_returns(&design, &floor, |_| Ok("FLOOR")).is_err());
    design.one_way_ledges[0].return_region_core = Some([4, 1, 10, 8]);
    validate_ledge_returns(&design, &floor, |_| Ok("FLOOR"))
        .expect("bounded local return and nontrivial whole-map return");
}

#[test]
fn authored_ledge_local_return_rejects_invalid_regions_and_real_shortcuts() {
    let mut design = fixture();
    design.one_way_ledges[0].return_passage_core = [8, 4, 9, 5];
    let floor = walking_loops();
    for bounds in [[5, 1, 10, 8], [4, 1, 8, 8], [4, 6, 10, 3], [4, 1, 255, 8]] {
        design.one_way_ledges[0].return_region_core = Some(bounds);
        assert!(validate_ledge_returns(&design, &floor, |_| Ok("FLOOR")).is_err());
    }
    design.one_way_ledges[0].return_region_core = Some([4, 1, 10, 8]);
    let mut disconnected = floor.clone();
    disconnected.remove(&[8, 4]);
    assert!(validate_ledge_returns(&design, &disconnected, |_| Ok("FLOOR")).is_err());
    let mut local_bypass = floor.clone();
    local_bypass.extend([[9, 3], [9, 4], [9, 5], [8, 3], [8, 5]]);
    assert!(validate_ledge_returns(&design, &local_bypass, |_| Ok("FLOOR")).is_err());
    let mut cheap_global_bypass = floor;
    cheap_global_bypass.extend([[3, 4], [3, 5]]);
    assert!(validate_ledge_returns(&design, &cheap_global_bypass, |_| Ok("FLOOR")).is_err());
}
