//! External pack preservation and native counter geometry for the fair overlay.
use std::path::PathBuf;

use crystal_assets::{build_state_fair_modpack, read_verified_compiled_game_pack};

#[test]
#[ignore = "requires the external fair parent and published native fair pack"]
fn state_fair_preserves_other_maps_assets_items_home_and_audio() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let base = read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_FAIR_BASE")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("content-packs/minnesota-fair-grass-base-v1.crystalpack")),
    )
    .expect("verified fair parent");
    let fair = read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_FAIR_TEST_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root.join("content-packs/minnesota-state-fair-maze-trainers-v7.crystalpack")
            }),
    )
    .expect("verified fair publication");
    let mut source: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("planning/minnesota/zones/state-fairgrounds.json"))
            .expect("fair dossier"),
    )
    .expect("source JSON");
    if let Some(path) = std::env::var_os("CRYSTAL_FAIR_CITY_CONNECTION") {
        source["native_city_connection"] =
            serde_json::from_str(&std::fs::read_to_string(path).expect("city connection source"))
                .expect("city connection JSON");
    }
    let a = base.data();
    let b = fair.data();
    assert_eq!(b.maps.len(), a.maps.len() + 4);
    for (name, m) in &a.maps {
        if name != "MnStPaul" {
            assert_eq!(m, &b.maps[name], "unrelated map {name}");
        }
    }
    let old = &a.maps["MnStPaul"];
    let new = &b.maps["MnStPaul"];
    let edits = old
        .blocks
        .iter()
        .zip(&new.blocks)
        .enumerate()
        .filter_map(|(n, (a, b))| (a != b).then_some(n))
        .collect::<Vec<_>>();
    let expected_edits = source["native_city_connection"]["edits"]
        .as_array()
        .expect("declared city edits")
        .iter()
        .map(|edit| {
            edit["position"][1].as_u64().expect("edit y") as usize
                * usize::from(old.attributes.width)
                + edit["position"][0].as_u64().expect("edit x") as usize
        })
        .collect::<Vec<_>>();
    assert_eq!(edits, expected_edits);
    assert_eq!(old.objects, new.objects);
    assert_eq!(old.events, new.events);
    assert_eq!(old.scripts, new.scripts);
    assert_eq!(old.script_text_bodies, new.script_text_bodies);
    assert_eq!(
        new.attributes.connections.len(),
        old.attributes.connections.len() + 1
    );
    for (name, art) in &a.tilesets {
        assert_eq!(art, &b.tilesets[name], "unrelated native art {name}");
    }
    for (path, bytes) in base.runtime_files() {
        assert_eq!(
            Some(bytes),
            fair.runtime_files().get(path),
            "source file {path}"
        );
    }
    assert_eq!(base.compiled_audio(), fair.compiled_audio());
    assert_eq!(a.pokemon, b.pokemon);
    assert_eq!(a.moves, b.moves);
    for (map, encounters) in &a.wild_encounters {
        assert_eq!(Some(encounters), b.wild_encounters.get(map));
    }
    assert_eq!(b.wild_encounters.len(), a.wild_encounters.len() + 1);
    let habitats = &b.wild_encounters["MnStateFairgrounds"].zones;
    assert_eq!(habitats.len(), 14);
    for index in 0..3 {
        let species = habitats
            .iter()
            .flat_map(|zone| [&zone.grass.morning, &zone.grass.day, &zone.grass.night][index])
            .map(|slot| slot.species.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(species.len(), 29, "every evolution at every time");
        assert!(species.contains("PAILBARON") && species.contains("BUTTERMUSE"));
    }
    for (id, item) in &a.items {
        assert_eq!(item, &b.items[id], "existing item {id}");
    }
    for (id, trainer) in &a.trainers.trainers {
        assert_eq!(trainer, &b.trainers.trainers[id], "existing trainer {id}");
    }
    assert_eq!(a.runtime_title_screen, b.runtime_title_screen);
    let m = &b.maps["MnStateFairgrounds"];
    let grass = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-state-fair-grass-v2.crystalpack"),
    )
    .expect("previous verified grass fair");
    assert_eq!(
        m.blocks,
        grass.data().maps["MnStateFairgrounds"].blocks,
        "crowd changes preserve every terrain and grass tile"
    );
    assert_eq!(
        b.tilesets["mn_state_fair_native"],
        grass.data().tilesets["mn_state_fair_native"]
    );
    assert_eq!(
        b.wild_encounters["MnStateFairgrounds"],
        grass.data().wild_encounters["MnStateFairgrounds"]
    );
    for room in ["MnFairGrandstand", "MnFairBarn", "MnFairDairy"] {
        assert_eq!(b.maps[room], grass.data().maps[room]);
    }
    let collision = &b.tilesets[&m.attributes.tileset_name].collision;
    let kind = |x: usize, y: usize| {
        &collision[&format!(
            "{:02x}",
            m.blocks[y / 2 * usize::from(m.attributes.width) + x / 2]
        )][y % 2 * 2 + x % 2]
    };
    assert_eq!([m.attributes.width, m.attributes.height], [48, 36]);
    assert_eq!(
        m.objects.len(),
        255,
        "all staff, guests and trainers share one exterior"
    );
    let old_crowd = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-state-fair-crowd-v2.crystalpack"),
    )
    .expect("verified crowd predecessor");
    let previous = &old_crowd.data().maps["MnStateFairgrounds"];
    assert_eq!(m.blocks, previous.blocks, "food changes preserve terrain");
    for original in &previous.objects {
        let retained = m
            .objects
            .iter()
            .find(|actor| actor.object_identifier == original.object_identifier)
            .expect("original fair actor remains present");
        assert_eq!(
            retained, original,
            "original NPCs and patrols remain unchanged"
        );
    }
    assert_eq!(
        m.events, previous.events,
        "food changes preserve entrances and signs"
    );
    assert_eq!(fair.compiled_audio(), old_crowd.compiled_audio());
    for (id, item) in &old_crowd.data().items {
        if id != "FAIR_CORN_STICK" {
            assert_eq!(item, &b.items[id], "existing fair item {id}");
        } else {
            let mut renamed = item.clone();
            renamed.name = "SWEET CORN".into();
            renamed.description = "Roast sweet corn.\nRestores 20 HP.".into();
            assert_eq!(renamed, b.items[id]);
        }
    }
    let foods = source["food_contract"]["purchasable_items"]
        .as_array()
        .expect("food contract");
    assert_eq!(foods.len(), 16);
    for food in foods {
        let id = food.as_str().expect("food ID");
        assert!(b.items[id].field_usable && b.items[id].battle_usable);
        assert!(b.items[id].name.chars().count() <= 12);
        assert!(b.items[id].description.lines().count() <= 2);
        assert!(
            b.items[id]
                .description
                .lines()
                .all(|line| line.chars().count() <= 18)
        );
        assert!(
            source["native_map"]["residents"]
                .as_array()
                .expect("shops")
                .iter()
                .any(|a| a["stock"]
                    .as_array()
                    .is_some_and(|stock| stock.contains(food))),
            "{id} must actually be sold"
        );
    }
    let beer = &b.items["FAIR_BEER"];
    assert_eq!(
        (beer.price, beer.parameter, beer.confusion_inflict_turns),
        (500, 100, Some(4))
    );
    assert_eq!(b.items.len(), old_crowd.data().items.len() + 7);
    let counters = source["native_map"]["residents"]
        .as_array()
        .expect("fair residents")
        .iter()
        .filter(|a| a["counter_tile"].is_array())
        .collect::<Vec<_>>();
    assert_eq!(counters.len(), 39);
    assert_eq!(
        m.objects
            .iter()
            .filter(|a| a.object_type == "OBJECTTYPE_TRAINER")
            .count(),
        35
    );
    assert_eq!(
        m.objects
            .iter()
            .filter(|a| a.object_type == "OBJECTTYPE_TRAINER" && a.radius > 0)
            .count(),
        20
    );
    assert_eq!(
        m.objects
            .iter()
            .filter(|a| a.spritemovedata == "SPRITEMOVEDATA_PATROL_CLOCKWISE")
            .count(),
        16
    );
    assert_eq!(
        m.objects
            .iter()
            .filter(|a| a.spritemovedata == "SPRITEMOVEDATA_PATROL_CLOCKWISE_SLOW")
            .count(),
        12
    );
    assert_eq!(
        m.objects
            .iter()
            .filter(|a| a.spritemovedata == "SPRITEMOVEDATA_SPINCLOCKWISE")
            .count(),
        54
    );
    for actor in counters {
        let x = actor["tile"][0].as_u64().expect("staff x") as usize;
        let y = actor["tile"][1].as_u64().expect("staff y") as usize;
        assert_eq!(kind(x, y), "FLOOR");
        assert_eq!(kind(x, y + 1), "COUNTER");
        assert_eq!(kind(x, y + 2), "FLOOR");
        assert!(
            m.objects
                .iter()
                .any(|a| usize::from(a.x) == x && usize::from(a.y) == y),
            "separate staff NPC {x},{y}"
        );
    }
    // The wider alias bank must retain every original Johto block's actual
    // bitplanes and colors, including animated source glyph addresses.
    let original = &a.tilesets["johto"];
    let expanded = &b.tilesets["mn_state_fair_native"];
    let original_gfx = &base.runtime_files()["gfx/tilesets/johto.2bpp"];
    let expanded_gfx = &fair.runtime_files()["gfx/tilesets/mn_state_fair_native.2bpp"];
    let original_blocks = &base.runtime_files()["data/tilesets/johto_metatiles.bin"];
    let expanded_blocks = &fair.runtime_files()["data/tilesets/mn_state_fair_native_metatiles.bin"];
    assert_eq!(
        original_blocks.as_slice(),
        &expanded_blocks[..original_blocks.len()]
    );
    for &glyph in original_blocks {
        let glyph = usize::from(glyph);
        let old_pal = original.palette_map[glyph];
        let new_pal = expanded.palette_map[glyph];
        assert_eq!(old_pal & 7, new_pal & 7);
        let old_slot = if old_pal & 8 == 0 {
            glyph
        } else {
            (glyph & 127) + original_gfx.len() / 32
        };
        let new_slot = if new_pal & 8 == 0 {
            glyph
        } else {
            (glyph & 127) + expanded_gfx.len() / 32
        };
        assert_eq!(
            &original_gfx[old_slot * 16..old_slot * 16 + 16],
            &expanded_gfx[new_slot * 16..new_slot * 16 + 16],
            "original glyph {glyph}"
        );
    }
    assert!(
        !m.blocks
            .iter()
            .any(|block| collision[&format!("{block:02x}")]
                .iter()
                .any(|k| k == "CUT_TREE")),
        "fair canopy must not introduce incidental Cut gates"
    );
    let layout = std::fs::read_to_string(
        root.join("planning/minnesota/zones/state-fairgrounds.layout.json"),
    )
    .expect("fair layout");
    let source = source.to_string();
    let rebuilt =
        build_state_fair_modpack(&base, &layout, &source).expect("repeatable native build");
    assert_eq!(
        fair.identity().expect("published identity"),
        rebuilt.identity().expect("rebuilt identity")
    );
    let mut invalid: serde_json::Value = serde_json::from_str(&source).expect("source JSON");
    invalid["native_map"]["residents"][0]["tile"] = serde_json::json!([55, 11]);
    assert!(
        build_state_fair_modpack(&base, &layout, &invalid.to_string()).is_err(),
        "staff placed on a counter must be rejected"
    );
    assert!(
        build_state_fair_modpack(&fair, &layout, &source).is_err(),
        "duplicate publication must be rejected"
    );
    let mut invalid: serde_json::Value = serde_json::from_str(&source).expect("source JSON");
    invalid["native_encounters"]["zones"][0]["minY"] = serde_json::json!(1);
    assert!(
        build_state_fair_modpack(&base, &layout, &invalid.to_string()).is_err(),
        "habitats must not create encounters on ordinary roads"
    );
    let missing = source.replace("PAILBARON", "DOUGHLIN");
    assert!(
        build_state_fair_modpack(&base, &layout, &missing).is_err(),
        "an omitted evolution or duplicate family member must fail"
    );
    let mut invalid: serde_json::Value = serde_json::from_str(&source).expect("source JSON");
    invalid["native_city_connection"]["fair_to_city_offset"] = serde_json::json!(99);
    assert!(
        build_state_fair_modpack(&base, &layout, &invalid.to_string()).is_err(),
        "a road which misses the reciprocal fair gate must fail"
    );
    let mut invalid: serde_json::Value = serde_json::from_str(&source).expect("source JSON");
    invalid["native_city_connection"]["edits"][0]["before"] = serde_json::json!("00");
    assert!(
        build_state_fair_modpack(&base, &layout, &invalid.to_string()).is_err(),
        "a stale city street must fail"
    );
}
