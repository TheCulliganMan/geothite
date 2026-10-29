use serde_json::json;

use super::{InstalledChunk, accepts_installed_tileset};
use crate::authored_route_layout::Layout;

fn layout() -> Layout {
    Layout::parse(
        &json!({
            "schema_version": 2,
            "id": "MN_R17",
            "dimensions_metatiles": [32, 56],
            "compiled_tileset": "mn_winter_outdoor",
            "shore_fitting": "explicit_source_edges",
            "reject_cliff_cuts": true,
            "layers": []
        })
        .to_string(),
    )
    .expect("registered Border Pines winter layout")
}

fn southern_chunk() -> InstalledChunk {
    InstalledChunk {
        name: "MnR17C00R02".into(),
        label: "NativeBlocks".into(),
        origin: [0, 48],
        size: [32, 8],
    }
}

#[test]
fn border_pines_installed_southern_transition_is_preserved() {
    let layout = layout();
    let chunk = southern_chunk();
    assert!(accepts_installed_tileset(
        &layout,
        &chunk,
        "mn_border_pines"
    ));
    assert!(accepts_installed_tileset(
        &layout,
        &chunk,
        "mn_winter_outdoor"
    ));
}

#[test]
fn border_pines_transition_requires_exact_chunk_identity_and_extent() {
    let layout = layout();
    let mut wrong_chunk = southern_chunk();
    wrong_chunk.name = "MnR17C00R00".into();
    wrong_chunk.origin = [0, 0];
    assert!(!accepts_installed_tileset(
        &layout,
        &wrong_chunk,
        "mn_border_pines"
    ));
    let mut wrong_extent = southern_chunk();
    wrong_extent.size = [16, 8];
    assert!(!accepts_installed_tileset(
        &layout,
        &wrong_extent,
        "mn_border_pines"
    ));
}

#[test]
fn border_pines_transition_does_not_authorize_other_routes_or_aliases() {
    let mut layout = layout();
    let chunk = southern_chunk();
    assert!(!accepts_installed_tileset(
        &layout,
        &chunk,
        "mn_north_shore"
    ));
    layout.id = "MN_R11".into();
    assert!(!accepts_installed_tileset(
        &layout,
        &chunk,
        "mn_border_pines"
    ));
}
