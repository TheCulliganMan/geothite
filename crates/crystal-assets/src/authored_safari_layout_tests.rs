//! Check the proposed preserve against the actual Rust compiler and native art.
use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use super::Layout;
use crate::authored_route_modpack::collision;

#[test]
fn boundary_safari_bench_keeps_native_pixels_palettes_and_source_content() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../content-packs/text-tui.crystalpack")
        .canonicalize()
        .expect("external source pack");
    let base = crate::read_verified_compiled_game_pack(path).expect("source");
    let result = crate::build_boundary_safari_art_pack(&base).expect("native seating overlay");
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    for (name, map) in &base.data().maps {
        assert_eq!(result.data().maps.get(name), Some(map), "changed {name}");
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed {name}"
        );
    }
    let files = result.runtime_files();
    let blocks = &files["data/tilesets/mn_boundary_safari_metatiles.bin"];
    let original = &files["data/tilesets/mn_headwaters_metatiles.bin"];
    assert_eq!(&blocks[..131 * 16], original);
    assert_eq!(blocks.len(), 132 * 16);
    let glyph = |tileset: &str, id: u8| {
        let palette = result.data().tilesets[tileset].palette_map[usize::from(id)];
        let index = if palette & 8 == 0 {
            usize::from(id)
        } else {
            usize::from(id & 0x7f) + 96
        };
        let pixels = &files[&format!("gfx/tilesets/{tileset}.2bpp")];
        (&pixels[index * 16..(index + 1) * 16], palette & 7)
    };
    for &id in original {
        assert_eq!(glyph("mn_headwaters", id), glyph("mn_boundary_safari", id));
    }
    for (&source, &target) in files["data/tilesets/kanto_metatiles.bin"][8 * 16..9 * 16]
        .iter()
        .zip(&blocks[131 * 16..132 * 16])
    {
        assert_eq!(glyph("kanto", source), glyph("mn_boundary_safari", target));
    }
    assert_eq!(
        result.data().tilesets["mn_boundary_safari"].collision["83"],
        result.data().tilesets["kanto"].collision["08"]
    );
}

#[test]
fn boundary_safari_terrain_compiles_with_native_cut_and_closed_water_edges() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let text = fs::read_to_string(
        root.join("planning/minnesota/zones/boundary-waters-safari.layout.json"),
    )
    .expect("authored preserve terrain");
    let layout = Layout::parse(&text).expect("supported Rust authoring schema");
    let blocks = layout.compile().expect("all explicit strokes compile");
    assert_eq!(blocks.len(), 48 * 40);
    let base =
        crate::read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))
            .expect("external source pack");
    let pack =
        crate::build_boundary_safari_art_pack(&base).expect("native water/deck/campsite art");
    let kind =
        |tile| collision(&pack, &blocks, 48, tile, "mn_boundary_safari").expect("native collision");
    let design: Value = serde_json::from_str(&text).expect("terrain anchors");
    let point = |value: &Value| {
        [0, 1].map(|axis| {
            u16::try_from(value[axis].as_u64().expect("authored coordinate"))
                .expect("coordinate range")
        })
    };
    assert_eq!(
        kind(point(&design["entry_contract"]["arrival_tile"])),
        "FLOOR"
    );
    let mut native_trees = BTreeSet::new();
    for y in 0..80 {
        for x in 0..96 {
            let token = kind([x, y]);
            if x == 0 || x == 95 || y == 0 || y == 79 {
                assert_ne!(token, "WATER", "unconnected water edge at {x},{y}");
            }
            if token == "CUT_TREE" {
                native_trees.insert([x, y]);
            }
        }
    }
    let gates = design["cut_gates"].as_array().expect("Cut contracts");
    assert_eq!(
        native_trees,
        gates.iter().map(|gate| point(&gate["tile"])).collect()
    );
    for gate in gates {
        let [x, y] = point(&gate["tile"]);
        let block = blocks[usize::from(y / 2) * 48 + usize::from(x / 2)];
        let native = &pack.data().field_moves.cut.replacements["mn_boundary_safari"][&block];
        assert_eq!(native.variant, "tree");
        assert_eq!(
            native.replacement_block_id,
            u16::from_str_radix(gate["replacement_block"].as_str().expect("replacement"), 16)
                .expect("native block ID")
        );
    }

    let occupied = ["retire_anchors", "landmark_actors", "decorations"]
        .into_iter()
        .flat_map(|key| design[key].as_array().expect("authored actors"))
        .map(|actor| point(&actor["tile"]))
        .collect::<BTreeSet<_>>();
    let land = (0..80)
        .flat_map(|y| (0..96).map(move |x| [x, y]))
        .filter(|tile| matches!(kind(*tile), "FLOOR" | "TALL_GRASS"))
        .filter(|tile| !occupied.contains(tile))
        .collect::<BTreeSet<_>>();
    let gate = &design["entry_gate"];
    let gate_tiles = gate["gate_tiles"]
        .as_array()
        .expect("native gate throat")
        .iter()
        .map(point)
        .collect::<BTreeSet<_>>();
    assert!(gate_tiles.iter().all(|tile| kind(*tile) == "FLOOR"));
    let start = point(&gate["forecourt_tile"]);
    let target = point(&gate["launch_target"]);
    assert!(land_path_exists(start, target, &land));
    let closed = land.difference(&gate_tiles).copied().collect();
    assert!(
        !land_path_exists(start, target, &closed),
        "native arrival-to-launch path bypasses the entry gate"
    );
    let portages = design["portages"].as_array().expect("grassy portages");
    assert_eq!(portages.len(), 4);
    for portage in portages {
        let west = point(&portage["west_land"]);
        let east = point(&portage["east_land"]);
        assert_eq!(kind(west), "FLOOR");
        assert_eq!(kind(east), "FLOOR");
        assert_eq!(kind(point(&portage["west_water"])), "WATER");
        assert_eq!(kind(point(&portage["east_water"])), "WATER");
        assert!(land_path_exists(west, east, &land));
        let bounds = portage["grass_bounds_core"]
            .as_array()
            .expect("grass crossing bounds");
        let bounds = bounds
            .iter()
            .map(|value| u16::try_from(value.as_u64().expect("coordinate")).expect("range"))
            .collect::<Vec<_>>();
        let grass = (bounds[1]..bounds[3])
            .flat_map(|y| (bounds[0]..bounds[2]).map(move |x| [x, y]))
            .collect::<BTreeSet<_>>();
        assert!(grass.iter().all(|tile| kind(*tile) == "TALL_GRASS"));
        let without_grass = land.difference(&grass).copied().collect();
        assert!(
            !land_path_exists(west, east, &without_grass),
            "{} has a native dry grass bypass",
            portage["id"]
        );
    }
}

fn land_path_exists(start: [u16; 2], target: [u16; 2], land: &BTreeSet<[u16; 2]>) -> bool {
    let mut seen = BTreeSet::from([start]);
    let mut pending = VecDeque::from([start]);
    while let Some([x, y]) = pending.pop_front() {
        if [x, y] == target {
            return true;
        }
        for next in [
            x.checked_sub(1).map(|x| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if land.contains(&next) && seen.insert(next) {
                pending.push_back(next);
            }
        }
    }
    false
}
