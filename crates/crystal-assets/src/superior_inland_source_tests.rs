//! Compile the promoted canonical source through the actual Rust rasterizer.
use std::path::PathBuf;

use super::Layout;

#[test]
#[ignore = "requires composed final pack and promoted canonical sources"]
fn superior_inland_canonical_rust_source_matches_every_published_terrain_cell() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_SUPERIOR_SOURCE_TEST_PACK").expect("explicit composed pack"),
    );
    let pack = crate::read_verified_compiled_game_pack(path).expect("strict final pack");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota");
    let compact = !pack.data.maps.contains_key("MnMaraisInlandNeck");
    let mut sources = vec![
        ("routes/MN_R11.layout.json", "MnR11"),
        ("routes/MN_R12.layout.json", "MnR12"),
        ("routes/MN_R13.layout.json", "MnR13"),
        ("towns/MnGrandMarais.layout.json", "MnGrandMarais"),
        (
            "towns/MnMaraisCedarApproach.layout.json",
            "MnMaraisCedarApproach",
        ),
    ];
    if compact {
        let exterior_names: Vec<_> = pack
            .data
            .maps
            .keys()
            .filter(|name| name.starts_with("MnR12C0"))
            .map(String::as_str)
            .collect();
        assert_eq!(exterior_names, ["MnR12C00R00", "MnR12C00R01"]);
    } else {
        sources.push(("towns/MnMaraisInlandNeck.layout.json", "MnMaraisInlandNeck"));
    }
    for (source, prefix) in sources {
        let text = std::fs::read_to_string(root.join(source)).expect("promoted source");
        let layout = Layout::parse(&text).expect("native Rust source schema");
        let cells = layout.compile().expect("native Rust rasterizer");
        let [width, height] = layout.dimensions_metatiles;
        let chunked = prefix.starts_with("MnR");
        let columns = if chunked { width.div_ceil(32) } else { 1 };
        let rows = if chunked { height.div_ceil(24) } else { 1 };
        let mut published_cells = 0;
        for row in 0..rows {
            for column in 0..columns {
                let name = if chunked {
                    format!("{prefix}C{column:02}R{row:02}")
                } else {
                    prefix.into()
                };
                let module = &pack.data.maps[&name];
                assert_eq!(
                    module.attributes.tileset_name, layout.compiled_tileset,
                    "{source}"
                );
                let a = &module.attributes;
                let expected_width = if compact && name == "MnR11C01R00" {
                    4
                } else if chunked {
                    (width - column * 32).min(32)
                } else {
                    width
                };
                let expected_height = if chunked {
                    (height - row * 24).min(24)
                } else {
                    height
                };
                assert_eq!([a.width, a.height], [expected_width, expected_height]);
                published_cells += usize::from(a.width) * usize::from(a.height);
                let x0 = if chunked { column * 32 } else { 0 };
                let y0 = if chunked { row * 24 } else { 0 };
                let mut expected = Vec::with_capacity(usize::from(a.width) * usize::from(a.height));
                for y in 0..a.height {
                    for x in 0..a.width {
                        expected.push(
                            cells[usize::from(y0 + y) * usize::from(width) + usize::from(x0 + x)],
                        );
                    }
                }
                assert_eq!(
                    module.blocks, expected,
                    "actual Rust source/native parity: {name}"
                );
            }
        }
        let retired_canvas_cells = if compact && prefix == "MnR11" {
            20 * 24
        } else {
            0
        };
        assert_eq!(
            published_cells + retired_canvas_cells,
            usize::from(width) * usize::from(height),
            "only the explicitly retired coastal canvas may be unpublished"
        );
    }
}

#[test]
fn north_shore_grass_source_rejects_winter_tileset_regression() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/routes/MN_R11.layout.json");
    let mut source: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("authored warm North Shore"))
            .expect("native layout JSON");
    assert_eq!(source["compiled_tileset"], "mn_north_shore");
    Layout::parse(&source.to_string()).expect("warm authored route");
    source["compiled_tileset"] = "mn_winter_north_shore".into();
    assert!(
        Layout::parse(&source.to_string()).is_err(),
        "North Shore winter regression accepted"
    );
}
