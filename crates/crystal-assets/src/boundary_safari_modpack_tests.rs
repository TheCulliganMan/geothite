//! Native publication contracts; active Safari and controller proof remain separate.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_core::world::map::TilePosition;
use crystal_core::world::session::OverworldSession;

use super::*;
use crate::ROOT_BEER_CABIN_CONSTANT;

fn inputs() -> &'static (CompiledGamePack, String, String, String) {
    static INPUTS: OnceLock<(CompiledGamePack, String, String, String)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let path = std::env::var_os("CRYSTAL_BOUNDARY_SAFARI_BASE_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("content-packs/text-tui.crystalpack"))
            .canonicalize()
            .expect("external source pack");
        let pack = crate::read_verified_compiled_game_pack(path).expect("verified source");
        let read =
            |path| std::fs::read_to_string(root.join(path)).expect("authoritative preserve input");
        (
            pack,
            read("planning/minnesota/zones/boundary-waters-safari.layout.json"),
            read("planning/minnesota/zones/boundary-waters-safari.json"),
            read("planning/minnesota/playable/root-beer-cabin.json"),
        )
    })
}
fn published() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let (base, layout, zone, cabin) = inputs();
        build_boundary_safari_modpack(base, layout, zone, cabin)
            .expect("native closed preserve and actual cabin")
    })
}

#[test]
fn boundary_safari_preserves_source_and_publishes_four_clipped_chunks_and_actual_cabin() {
    let (base, layout, _, cabin) = inputs();
    let pack = published();
    assert_eq!(pack.data().maps.len(), base.data().maps.len() + 5);
    assert_eq!(
        pack.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().expect("published audio"),
        base.audio_manifest().expect("source audio")
    );
    assert_eq!(pack.data().trainers, base.data().trainers);
    assert_eq!(
        pack.data().wild_encounters,
        base.data().wild_encounters,
        "terrain is not ordinary wild Safari encounters"
    );
    for (name, module) in &base.data().maps {
        assert_eq!(pack.data().maps.get(name), Some(module), "changed {name}");
    }
    for (name, metadata) in &base.data().runtime_map_metadata {
        assert_eq!(
            pack.data().runtime_map_metadata.get(name),
            Some(metadata),
            "changed {name}"
        );
    }
    for (name, tileset) in &base.data().tilesets {
        assert_eq!(
            pack.data().tilesets.get(name),
            Some(tileset),
            "changed source art {name}"
        );
    }
    for (name, item) in &base.data().items {
        assert_eq!(
            pack.data().items.get(name),
            Some(item),
            "changed source item {name}"
        );
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            pack.runtime_files().get(name),
            Some(bytes),
            "changed source file {name}"
        );
    }
    for (name, landmark) in &base.data().pokegear_landmarks.map_to_landmark {
        assert_eq!(
            pack.data().pokegear_landmarks.map_to_landmark.get(name),
            Some(landmark)
        );
    }
    let layout = Layout::parse(layout).expect("native layout");
    let blocks = layout.compile().expect("native strokes");
    let mut objects = 0;
    let mut signs = 0;
    let mut connections = 0;
    for (row, height) in [(0, 24), (1, 16)] {
        for (column, width) in [(0, 32), (1, 16)] {
            let name = chunk([column * 64, row * 48]);
            let module = &pack.data().maps[&name];
            assert_eq!(
                (module.attributes.width, module.attributes.height),
                (width, height)
            );
            assert_eq!(module.attributes.tileset_name, "mn_boundary_safari");
            let expected = (row * 24..row * 24 + height)
                .flat_map(|y| {
                    let offset = usize::from(y) * 48 + usize::from(column) * 32;
                    blocks[offset..offset + usize::from(width)].iter().copied()
                })
                .collect::<Vec<_>>();
            assert_eq!(module.blocks, expected);
            assert_eq!(
                pack.data()
                    .assemble_map_module_from_compiled_payloads(&name)
                    .expect("actual native split payload"),
                *module
            );
            let metadata = &pack.data().runtime_map_metadata[&map_constant(column, row)];
            assert_eq!(metadata.group_id, 61);
            assert_eq!(metadata.map_id, row * 2 + column + 1);
            assert_eq!((metadata.width, metadata.height), (width, height));
            assert_eq!(metadata.name, name);
            assert_eq!(metadata.group_name, GROUP);
            assert_eq!(module.attributes.connections.len(), 2);
            for edge in &module.attributes.connections {
                let opposite = match edge.direction.as_str() {
                    "north" => "south",
                    "south" => "north",
                    "east" => "west",
                    "west" => "east",
                    _ => panic!("native cardinal seam"),
                };
                assert_eq!(edge.offset, 0);
                assert!(
                    pack.data().maps[&edge.target_map]
                        .attributes
                        .connections
                        .iter()
                        .any(|back| back.target_map == name
                            && back.direction == opposite
                            && back.offset == 0)
                );
            }
            assert!(!pack.data().wild_encounters.contains_key(&name));
            assert!(
                module.trainer_scripts.is_empty()
                    && module.scripted_trainer_battles.is_empty()
                    && module.scripted_wild_battles.is_empty()
            );
            assert!(
                module
                    .objects
                    .iter()
                    .all(|actor| actor.object_type == "OBJECTTYPE_SCRIPT")
            );
            objects += module.objects.len();
            signs += module.events.bg_events.len();
            connections += module.attributes.connections.len();
        }
    }
    assert_eq!((objects, signs, connections), (22, 13, 8));
    let module = &pack.data().maps[crate::ROOT_BEER_CABIN_MAP];
    let cabin: Value = serde_json::from_str(cabin).expect("actual cabin");
    let expected = cabin["rows"]
        .as_array()
        .expect("room rows")
        .iter()
        .flat_map(|row| row.as_str().expect("row").split_whitespace())
        .map(|value| u16::from_str_radix(value, 16).expect("native room block"))
        .collect::<Vec<_>>();
    assert_eq!(module.blocks, expected);
    assert_eq!((module.attributes.width, module.attributes.height), (5, 4));
    assert!(
        module.objects.is_empty(),
        "default keeps one outside Dorothy"
    );
    assert_eq!(
        pack.data().runtime_map_metadata[ROOT_BEER_CABIN_CONSTANT].map_id,
        5
    );
    assert_eq!(module.events.warps.len(), 3);
    for (warp, (index, tile, kind)) in module.events.warps.iter().zip([
        (1, [5, 7], "WARP_CARPET_DOWN"),
        (2, [6, 7], "WARP_CARPET_DOWN"),
        (3, [5, 6], "FLOOR"),
    ]) {
        assert_eq!(warp.index, index);
        assert_eq!([warp.x, warp.y], tile);
        assert_eq!(warp.target_warp_id, 2);
        assert_eq!(warp.target_map, "MN_BOUNDARY_SAFARI_C00R00");
        assert_eq!(
            collision(pack, &module.blocks, 5, tile, "traditional_house")
                .expect("native cabin warp collision"),
            kind
        );
    }
    let exterior = &pack.data().maps["MnBoundarySafariC00R00"];
    assert_eq!(exterior.events.warps.len(), 2);
    assert_eq!(exterior.events.warps[0].target_warp_id, 3);
    assert_eq!(exterior.events.warps[1].target_warp_id, 3);
    let dorothy = exterior
        .objects
        .iter()
        .find(|actor| actor.object_identifier.as_deref() == Some("BWS_ROOT_BEER_LADY"))
        .expect("single outside Dorothy");
    assert_eq!([dorothy.x, dorothy.y], [49, 43]);
    assert_eq!(dorothy.script, "BWS_ROOT_BEER_LADYRootBeerService");
    assert!(
        exterior
            .script_item_grants
            .iter()
            .any(|grant| grant.item_id == crate::ROOT_BEER_ITEM_ID && grant.quantity == 1)
    );
    assert!(
        exterior
            .script_shop_commands
            .iter()
            .any(|shop| shop.mart_id == crate::ROOT_BEER_MART_ID)
    );
    assert!(
        exterior
            .script_runtime_commands
            .iter()
            .any(|command| command.command == "special" && command.args == ["HealParty"])
    );
    assert_eq!(pack.data().items[crate::ROOT_BEER_ITEM_ID].price, 300);
    let (base, layout, zone, cabin) = inputs();
    assert!(build_boundary_safari_modpack(published(), layout, zone, cabin).is_err());
    assert!(
        !base
            .data()
            .runtime_map_metadata
            .values()
            .any(|metadata| metadata.group_id == 61)
    );
}

#[test]
fn boundary_cabin_native_core_triggers_resolve_constants_and_passive_floor_arrivals() {
    let data = published().data();
    let session_at = |name: &str, tile: TilePosition| {
        let module = data.map_module(name).expect("actual published module");
        OverworldSession::with_events(
            data.overworld_map(name).expect("actual native map"),
            module.events.clone(),
            data.tileset_collision(&module.attributes.tileset_name)
                .expect("actual native collision"),
            tile,
        )
    };
    // This is an explicitly placed core-boundary fixture, not a player journey.
    // It uses the same native collision trigger and destination resolver as
    // production, rather than inferring a working warp from record equality.
    for (name, tile, source_index, destination, arrival, destination_index) in [
        (
            "MnBoundarySafariC00R00",
            TilePosition::new(51, 39),
            1,
            crate::ROOT_BEER_CABIN_MAP,
            TilePosition::new(5, 6),
            3,
        ),
        (
            crate::ROOT_BEER_CABIN_MAP,
            TilePosition::new(5, 7),
            1,
            "MnBoundarySafariC00R00",
            TilePosition::new(51, 40),
            2,
        ),
        (
            crate::ROOT_BEER_CABIN_MAP,
            TilePosition::new(6, 7),
            2,
            "MnBoundarySafariC00R00",
            TilePosition::new(51, 40),
            2,
        ),
    ] {
        let trigger = session_at(name, tile)
            .check_warp_checked()
            .expect("native trigger coordinate")
            .expect("actual DOOR or carpet triggers");
        assert_eq!(trigger.warp.index, source_index);
        assert_eq!(trigger.warp.target_map, trigger.warp.target_map_constant);
        let transition = data
            .resolve_warp_transition(&trigger)
            .expect("actual native constant and destination payload resolution");
        assert_eq!(transition.destination.map_name, destination);
        assert_eq!(transition.destination.tile, arrival);
        assert_eq!(transition.destination.warp.index, destination_index);
        assert!(
            session_at(destination, arrival)
                .check_warp_checked()
                .expect("actual passive FLOOR arrival")
                .is_none(),
            "safe native arrival must not bounce"
        );
        let mut wrong_name = trigger.clone();
        wrong_name.warp.target_map = destination.into();
        let error = data
            .resolve_warp_transition(&wrong_name)
            .expect_err("literal map names in native target fields must remain rejected");
        assert!(
            error
                .to_string()
                .contains("does not match target_map_constant"),
            "wrong rejection: {error}"
        );
    }
}

#[test]
fn boundary_safari_native_source_anchors_camps_and_closed_arrival_are_honest() {
    let (_, layout, _, _) = inputs();
    let pack = published();
    let source: Value = serde_json::from_str(layout).expect("source anchors");
    for actor in ["retire_anchors", "landmark_actors", "decorations"]
        .into_iter()
        .flat_map(|key| source[key].as_array().expect("actors"))
    {
        let at = point(&actor["tile"]).expect("actual tile");
        let module = &pack.data().maps[&chunk(at)];
        let native = module
            .objects
            .iter()
            .find(|native| native.object_identifier.as_deref() == actor["id"].as_str())
            .expect("original native object identity");
        assert_eq!([native.x, native.y], [at[0] % 64, at[1] % 48]);
        assert_eq!(Some(native.sprite.as_str()), actor["sprite"].as_str());
    }
    let mut blocked = BTreeSet::new();
    for row in 0..2 {
        for column in 0..2 {
            for actor in &pack.data().maps[&chunk([column * 64, row * 48])].objects {
                blocked.insert([actor.x + column * 64, actor.y + row * 48]);
            }
        }
    }
    let native_kind = |at: [u16; 2]| {
        let module = &pack.data().maps[&chunk(at)];
        collision(
            pack,
            &module.blocks,
            module.attributes.width,
            [at[0] % 64, at[1] % 48],
            "mn_boundary_safari",
        )
        .expect("published native collision")
    };
    let mut allowed = BTreeSet::new();
    let mut cut = BTreeSet::new();
    let mut vwater = 0;
    let mut hwater = 0;
    for y in 0..80 {
        for x in 0..96 {
            let at = [x, y];
            let terrain = native_kind(at);
            if terrain == "CUT_TREE" {
                cut.insert(at);
            }
            if x == 0 || x == 95 || y == 0 || y == 79 {
                assert!(!matches!(
                    terrain,
                    "WATER" | "WHIRLPOOL" | "WATERFALL" | "FLOOR" | "TALL_GRASS"
                ));
            }
            if matches!(terrain, "FLOOR" | "TALL_GRASS" | "WATER" | "DOOR")
                && !blocked.contains(&at)
            {
                allowed.insert(at);
            }
            if x == 63 && terrain == "WATER" && native_kind([64, y]) == "WATER" {
                vwater += 1;
            }
            if y == 47 && terrain == "WATER" && native_kind([x, 48]) == "WATER" {
                hwater += 1;
            }
        }
    }
    assert_eq!(cut, BTreeSet::from([[10, 31]]));
    assert!(
        vwater > 0 && hwater > 0,
        "real internal Surf seams must remain open"
    );
    let forecourt = reached([15, 73], &allowed);
    assert_eq!(forecourt.len(), 42);
    assert!(
        !forecourt.contains(&[3, 58]),
        "pending admission has a walking/Surf bypass"
    );
    let entry = &pack.data().maps["MnBoundarySafariC00R01"];
    for (id, at) in BOUNDARY_SAFARI_PENDING_GUARDS {
        let actor = entry
            .objects
            .iter()
            .find(|actor| actor.object_identifier.as_deref() == Some(id))
            .expect("blocking native pending guard");
        assert_eq!([actor.x, actor.y], [at[0], at[1] - 48]);
        let body = entry
            .script_text_bodies
            .get(&format!("{id}Text"))
            .expect("truthful guard text");
        assert!(format!("{body:?}").contains("EXPEDITIONS CLOSED"));
    }
    assert!(
        entry.script_economy_commands.is_empty(),
        "no admission fee transaction exists"
    );
    assert!(
        pack.data()
            .maps
            .values()
            .filter(|module| module.id.starts_with("MnBoundarySafari"))
            .all(|module| module
                .scripts
                .values()
                .all(|script| !script.to_string().contains("wSafariBalls")
                    && !script.to_string().contains("wSafariSteps")))
    );
}

fn with_layer(layout: &str, layer: Value) -> String {
    let mut value: Value = serde_json::from_str(layout).expect("layout");
    value["layers"].as_array_mut().expect("strokes").push(layer);
    value.to_string()
}
#[test]
fn boundary_safari_rejects_outer_water_cut_portage_and_closed_gate_bypasses() {
    let (base, layout, zone, cabin) = inputs();
    for layer in [
        json!({"name":"Unconnected outer water","kind":"block_rect","bounds":[22,0,25,2],"block":"35"}),
        json!({"name":"Lost purposeful Cut tree","kind":"point","position":[5,15],"block":"3f"}),
        json!({"name":"Lost portage grass","kind":"rect","bounds":[4,30,6,32],"material":"floor"}),
        json!({"name":"Closed checkpoint bypass","kind":"path","points":[[7,36],[7,33],[6,33]],"width_metatiles":1,"material":"floor"}),
    ] {
        assert!(
            build_boundary_safari_modpack(base, &with_layer(layout, layer), zone, cabin).is_err(),
            "unsafe native source change accepted"
        );
    }
}
#[test]
fn boundary_safari_rejects_paid_story_access_bad_cabin_and_missing_source_actor() {
    let (base, layout, zone, cabin) = inputs();
    let mut bad: Value = serde_json::from_str(zone).expect("zone");
    bad["story_contract"]["paid_expedition_required_for_story"] = json!(true);
    assert!(build_boundary_safari_modpack(base, layout, &bad.to_string(), cabin).is_err());
    let mut bad: Value = serde_json::from_str(cabin).expect("cabin");
    bad["warp_contract"]["interior_passive_arrival_warp"]["index"] = json!(1);
    assert!(build_boundary_safari_modpack(base, layout, zone, &bad.to_string()).is_err());
    let mut bad: Value = serde_json::from_str(layout).expect("layout");
    bad["landmark_actors"] = json!([]);
    assert!(build_boundary_safari_modpack(base, &bad.to_string(), zone, cabin).is_err());
}
