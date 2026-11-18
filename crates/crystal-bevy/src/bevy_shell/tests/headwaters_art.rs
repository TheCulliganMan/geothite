//! Derived pier art must keep native Johto flower/water animation.
use std::path::PathBuf;

use bevy::asset::Assets;
use bevy::image::Image;
use crystal_assets::{AssetRoot, MapModule, read_loaded_verified_compiled_game_pack};
use crystal_core::world::collision::{sample_collision, spawns_shaking_grass_object};
use crystal_core::world::map::TilePosition;
use crystal_runtime::CrystalRuntime;
use image::RgbaImage;

use super::super::{
    RenderedTilesetArt, TilesetArt, grass_rustle_frames_for_art, load_tileset_art,
    winter_render_behavior::rustle_tileset_at,
};

#[test]
#[ignore = "requires the external Headwaters pack"]
fn headwaters_animations_keep_native_johto_flower_and_water_frames() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_HEADWATERS_TEST_PACK").expect("external Headwaters pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("art pack"),
    )
    .expect("art runtime");
    let root = runtime
        .materialize_runtime_files()
        .expect("temporary runtime mount");
    let mut images = Assets::<Image>::default();
    let mut load = |id: &str| {
        load_tileset_art(
            &root,
            id,
            "day",
            &runtime.data().tilesets[id].palette_map,
            &mut images,
        )
        .expect("native tileset")
    };
    let source = load("johto");
    let derived = load("mn_headwaters");
    assert_eq!(source.animated_tiles.len(), derived.animated_tiles.len());
    for tile in [0x03, 0x14] {
        let a = &derived.animated_tiles[&tile];
        let b = &source.animated_tiles[&tile];
        assert_eq!(a.frame_ticks, b.frame_ticks);
        assert_eq!(a.phase_offset, b.phase_offset);
        assert_eq!(a.advance_on_phase_offset, b.advance_on_phase_offset);
        assert_eq!(a.requires_forest_restless, b.requires_forest_restless);
        assert_eq!(a.cave_water_composite, b.cave_water_composite);
        assert_eq!(a.frames.len(), b.frames.len());
        for (actual, expected) in a.frames.iter().zip(b.frames.iter()) {
            assert_eq!(
                images.get(actual).expect("derived frame").data,
                images.get(expected).expect("source frame").data,
                "tile {tile:02x}"
            );
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_SHINER_WATER_TEST_PACK with the outdoor shoreline revision"]
fn shiner_water_cycles_are_local_and_leave_the_dock_still() {
    let (root, runtime) = shoreline_fixture();
    let bank = &runtime.data().maps["MnR15C00R00"].attributes.tileset_name;
    assert_eq!(bank, "mn_shiner_narrows");
    for time in ["day", "nite"] {
        let mut images = Assets::<Image>::default();
        let mut load = |name: &str| {
            load_tileset_art(
                &root,
                name,
                time,
                &runtime.data().tilesets[name].palette_map,
                &mut images,
            )
            .expect("native dock art")
        };
        let original = load("kanto");
        let lake = load(bank);
        assert!(
            lake.metatile_layout == original.metatile_layout,
            "Shiner native metatile art changed"
        );
        assert_eq!(lake.animated_tiles.len(), original.animated_tiles.len() + 6);
        for (id, expected) in &original.animated_tiles {
            let actual = &lake.animated_tiles[id];
            assert_eq!(actual.frame_ticks, expected.frame_ticks);
            for (a, b) in actual.frames.iter().zip(expected.frames.iter()) {
                assert!(
                    images.get(a).expect("dock frame").data
                        == images.get(b).expect("Kanto frame").data,
                    "altered inherited frame for native tile {id:02x}"
                );
            }
        }
        for id in [0x91, 0x92, 0x93, 0x95, 0x96, 0x97] {
            assert!(!original.animated_tiles.contains_key(&id));
            let clip = &lake.animated_tiles[&id];
            assert_eq!((clip.frame_ticks, clip.frames.len()), (22, 4));
            let states: std::collections::BTreeSet<_> = clip
                .frames
                .iter()
                .map(|frame| {
                    images
                        .get(frame)
                        .expect("native ripple")
                        .data
                        .as_deref()
                        .expect("RGBA pixels")
                })
                .collect();
            assert_eq!(states.len(), 4, "unchanging water at {id:02x} by {time}");
        }
        for id in (0x60..=0x90).chain([0x94]).chain(0x98..=0x9f) {
            assert!(
                !lake.animated_tiles.contains_key(&id),
                "moving board, post or bank {id:02x}"
            );
        }
        let room = &runtime.data().maps["MnR15C00R00"];
        let pier = |tick| native_region(room, &lake, &images, [33, 12], [11, 6], tick);
        let frames = [0, 22, 44, 66].map(pier);
        // The flowers on the approach have their own 60-tick clock. Only the
        // open water above the pier should loop after four 22-tick phases.
        let water = |tick| native_region(room, &lake, &images, [34, 12], [6, 2], tick);
        assert!(water(0) == water(88), "Shiner's deep water must loop");
        for phase in 0..4 {
            assert!(
                frames[phase] != frames[(phase + 1) % 4],
                "still Shiner camera at phase {phase}"
            );
            if time == "day"
                && let Some(folder) = std::env::var_os("CRYSTAL_DOCK_FRAME_DIR")
            {
                let path = PathBuf::from(folder);
                std::fs::create_dir_all(&path).expect("dock proof directory");
                frames[phase]
                    .save(path.join(format!("shiner-tick-{}.png", phase * 22)))
                    .expect("native Shiner water frame");
            }
        }
        for (name, deck, water) in [
            ("MnBrainerd", [47, 13], [47, 12]),
            ("MnR01C00R00", [12, 34], [12, 33]),
            ("MnR05C00R01", [61, 40], [62, 40]),
            ("MnR06C00R00", [30, 42], [29, 42]),
            ("MnR07C00R00", [41, 14], [42, 14]),
            ("MnR08C00R01", [58, 23], [57, 23]),
            ("MnR15C00R00", [34, 14], [34, 13]),
            ("MnR16C00R00", [19, 15], [20, 15]),
            ("MnR18C00R00", [31, 22], [32, 22]),
            ("MnR18C00R00", [21, 34], [22, 34]),
            ("MnR20C00R00", [19, 25], [20, 25]),
        ] {
            check_pier(&runtime, &root, time, name, deck, water);
        }
        let mountain = &runtime.data().maps["MnR10C00R00"];
        let summit = TilePosition::new(34, 8);
        let terrain = runtime
            .data()
            .overworld_session("MnR10C00R00", summit, 0)
            .expect("mountain terrain");
        let permission = sample_collision(&terrain.map, &terrain.tileset, summit)
            .expect("summit square")
            .permission;
        assert!(spawns_shaking_grass_object(permission));
        let style = rustle_tileset_at(
            &mountain.attributes.tileset_name,
            mountain.attributes.width,
            &mountain.blocks,
            summit,
        );
        assert_eq!(style, "mn_iron_descent_thaw");
        let frames = grass_rustle_frames_for_art(
            &mut RenderedTilesetArt::default(),
            &root,
            style,
            time,
            &mut images,
        )
        .expect("installed mountaintop rustle");
        let pixels = frames.map(|frame| images.get(&frame.handle).expect("native grass drawing"));
        assert!(pixels.iter().all(|p| p.width() == 18 && p.height() == 9));
        assert!(
            pixels[0].data != pixels[1].data,
            "identical mountain grass frames"
        );
    }
}

fn shoreline_fixture() -> (AssetRoot, CrystalRuntime) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_SHINER_WATER_TEST_PACK").expect("external shoreline pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified shoreline pack"),
    )
    .expect("shoreline runtime");
    let root = runtime
        .materialize_runtime_files()
        .expect("native art mount");
    (root, runtime)
}

#[test]
#[ignore = "requires CRYSTAL_SHINER_WATER_TEST_PACK with the Bemidji and Detroit Lakes piers"]
fn city_fishing_piers_have_animated_water_on_all_three_open_sides() {
    let (root, runtime) = shoreline_fixture();
    for time in ["day", "nite"] {
        for (name, deck, water) in [
            ("MnBemidji", [36, 35], [36, 34]),
            ("MnBemidji", [36, 36], [36, 37]),
            ("MnBemidji", [39, 35], [40, 35]),
            ("MnDetroitLakes", [22, 28], [22, 27]),
            ("MnDetroitLakes", [22, 29], [22, 30]),
            ("MnDetroitLakes", [17, 28], [16, 28]),
        ] {
            check_pier(&runtime, &root, time, name, deck, water);
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_SHINER_WATER_TEST_PACK with the open Boundary Waters and Duluth docks"]
fn boundary_waters_and_duluth_docks_keep_native_water_on_three_open_sides() {
    let (root, runtime) = shoreline_fixture();
    for time in ["day", "nite"] {
        for (name, deck, water) in [
            ("MnBoundaryWaters", [12, 16], [11, 16]),
            ("MnBoundaryWaters", [16, 16], [16, 15]),
            ("MnBoundaryWaters", [16, 17], [16, 18]),
            ("MnDuluth", [47, 64], [47, 65]),
            ("MnDuluth", [48, 63], [49, 63]),
            ("MnDuluth", [47, 63], [46, 63]),
        ] {
            check_pier(&runtime, &root, time, name, deck, water);
        }
        let bank = &runtime.data().maps["MnBoundaryWaters"]
            .attributes
            .tileset_name;
        let mut images = Assets::<Image>::default();
        let art = load_tileset_art(
            &root,
            bank,
            time,
            &runtime.data().tilesets[bank].palette_map,
            &mut images,
        )
        .expect("Boundary Waters terrain");
        let original = load_tileset_art(
            &root,
            "johto",
            time,
            &runtime.data().tilesets["johto"].palette_map,
            &mut images,
        )
        .expect("native water family");
        assert_eq!(art.animated_tiles.len(), 1, "the reserve only cycles water");
        let ripple = &art.animated_tiles[&0x14];
        assert_eq!((ripple.frame_ticks, ripple.frames.len()), (22, 4));
        for (frame, source) in ripple
            .frames
            .iter()
            .zip(original.animated_tiles[&0x14].frames.iter())
        {
            assert!(
                images.get(frame).expect("reserve frame").data
                    == images.get(source).expect("source ripple").data,
                "expected the native water drawing and clock palette at {time}"
            );
        }
    }
}

fn check_pier(
    runtime: &CrystalRuntime,
    root: &AssetRoot,
    time: &str,
    name: &str,
    deck: [usize; 2],
    water: [usize; 2],
) {
    let mut images = Assets::<Image>::default();
    let room = &runtime.data().maps[name];
    let bank = &room.attributes.tileset_name;
    let collision = |[x, y]: [usize; 2]| {
        let block = room.blocks[y / 2 * usize::from(room.attributes.width) + x / 2];
        &runtime.data().tilesets[bank].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
    };
    assert_eq!(
        collision(deck),
        "FLOOR",
        "fishing platform at {name} {deck:?}"
    );
    assert_eq!(
        collision(water),
        "WATER",
        "fishing water at {name} {water:?}"
    );
    let art = load_tileset_art(
        root,
        bank,
        time,
        &runtime.data().tilesets[bank].palette_map,
        &mut images,
    )
    .expect("installed dock bank");
    let at = |point, tick| native_region(room, &art, &images, point, [1, 1], tick);
    let mut water_states = std::collections::BTreeSet::new();
    for tick in [0, 22, 44, 66] {
        water_states.insert(at(water, tick).into_raw());
        assert!(
            at(deck, tick) == at(deck, 0),
            "moving timber at {name} {deck:?} by {time}"
        );
    }
    assert_eq!(
        water_states.len(),
        if name == "MnR15C00R00" { 4 } else { 3 },
        "still fishing water at {name} {water:?} by {time}"
    );
    // The original four-phase lake returns through the middle drawing.
    if name != "MnR15C00R00" {
        assert!(
            at(water, 22) == at(water, 66),
            "native return ripple at {name}"
        );
    }
}

// Read a camera window through the game's exact frame selector. Each walkable
// square contains four 8px tiles, and each map block contains four squares.
fn native_region(
    room: &MapModule,
    art: &TilesetArt,
    images: &Assets<Image>,
    origin: [usize; 2],
    size: [usize; 2],
    tick: u64,
) -> RgbaImage {
    let mut image = RgbaImage::new((size[0] * 16) as u32, (size[1] * 16) as u32);
    for y in 0..size[1] * 2 {
        for x in 0..size[0] * 2 {
            let gx = origin[0] * 2 + x;
            let gy = origin[1] * 2 + y;
            let block = room.blocks[gy / 4 * usize::from(room.attributes.width) + gx / 4];
            let handle = art
                .tile_handle_at_frame(block, gx % 4, gy % 4, tick, false)
                .expect("native tile");
            let bytes = images
                .get(&handle)
                .expect("native tile art")
                .data
                .as_ref()
                .expect("RGBA");
            let glyph = RgbaImage::from_raw(8, 8, bytes.clone()).expect("native 8px tile");
            image::imageops::replace(&mut image, &glyph, (x * 8) as i64, (y * 8) as i64);
        }
    }
    image
}
