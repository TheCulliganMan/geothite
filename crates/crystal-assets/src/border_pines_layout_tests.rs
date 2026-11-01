//! Existing inland winter terrain remains distinct from the warm coast.
use serde_json::{Value, json};

use super::Layout;

fn source(tileset: &str, fitting: &str) -> Value {
    json!({
        "schema_version": 2,
        "id": "MN_R17",
        "dimensions_metatiles": [32, 56],
        "compiled_tileset": tileset,
        "shore_fitting": fitting,
        "reject_cliff_cuts": true,
        "layers": []
    })
}

#[test]
fn border_pines_accepts_its_installed_winter_alias_and_original_source() {
    for value in [
        source("mn_border_pines", "forest_8_neighbors"),
        source("mn_winter_outdoor", "explicit_source_edges"),
    ] {
        Layout::parse(&value.to_string()).expect("existing Border Pines source contract");
    }
}

#[test]
fn border_pines_winter_support_retains_route_extent_and_edge_policy() {
    let mut extent = source("mn_winter_outdoor", "explicit_source_edges");
    extent["dimensions_metatiles"] = json!([64, 56]);
    assert!(Layout::parse(&extent.to_string()).is_err());
    let wrong_edges = source("mn_winter_outdoor", "johto_8_neighbors");
    assert!(Layout::parse(&wrong_edges.to_string()).is_err());
}

#[test]
fn border_pines_winter_alias_is_not_a_north_shore_exception() {
    let mut coast = source("mn_winter_outdoor", "explicit_source_edges");
    coast["id"] = json!("MN_R11");
    coast["dimensions_metatiles"] = json!([32, 72]);
    assert!(Layout::parse(&coast.to_string()).is_err());
    let wrong_alias = source("mn_north_shore", "explicit_source_edges");
    assert!(Layout::parse(&wrong_alias.to_string()).is_err());
}
