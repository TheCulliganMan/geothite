// Retain actual classic-battle frames for rear-portrait art review. The battle
// renderer supplies both battlers and the HUD; only the contact board is tiled.

#[test]
fn northwoods_starter_backs_render_in_production_classic_battles() {
    use std::collections::BTreeSet;

    let output = std::env::var_os("NORTHWOODS_STARTER_RENDER_DIR").map(PathBuf::from);
    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        assert!(
            output.is_none(),
            "starter capture requires an explicit pack"
        );
        return;
    }
    let shell = route36_battle_shell_for_render_regression();
    let data = shell.runtime.data();
    if data.saved_species("KINDLIT").is_none() {
        assert!(
            output.is_none(),
            "starter capture requires the current Northwoods starter pack"
        );
        return;
    }
    if let Some(directory) = &output {
        if directory.exists() {
            assert!(
                std::fs::read_dir(directory)
                    .expect("inspect retained starter capture root")
                    .next()
                    .is_none(),
                "starter capture directory contains retained renders; choose a fresh directory"
            );
        } else {
            std::fs::create_dir_all(directory).expect("create starter capture root");
        }
    }
    let base = shell.shell.snapshot().expect("production battle snapshot");
    let actors = [
        "POUCHID",
        "SLIPPERINA",
        "CYPRIELLE",
        "KINDLIT",
        "RIMEPELT",
        "BOREALYNX",
        "BIRCHIT",
        "PORTABEAU",
        "VOYAGEAR",
        "TOTODILE",
        "CROCONAW",
        "FERALIGATR",
    ];
    let mut back_fingerprints = BTreeSet::new();
    let mut board = image::RgbaImage::from_pixel(480, 576, image::Rgba([255; 4]));
    for (index, actor) in actors.into_iter().enumerate() {
        let mut snapshot = base.clone();
        let species = data.saved_species(actor).expect("current starter species");
        let dvs = Dv::from_non_hp(9, 10, 10, 10);
        assert!(!visible_dvs_are_shiny(dvs));
        let pokemon = crate::core::models::pokemon::create_pokemon_from_known_dvs(
            &species,
            50,
            dvs,
            &data.learnsets,
            &data.moves,
            &data.growth_rates,
        )
        .expect("starter review Pokemon");
        let active_index = snapshot
            .battle
            .as_ref()
            .expect("battle")
            .active_player_party_index
            .expect("active player slot");
        let slot = snapshot
            .party
            .slots
            .iter_mut()
            .find(|slot| slot.index == active_index)
            .expect("active player party slot");
        slot.pokemon = pokemon;
        assert_eq!(slot.pokemon.species.id, actor);

        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        let back = pokemon_animation_frame_for_art(
            &mut art,
            &shell.asset_root,
            &pokemon_asset_id_for_dvs(actor, dvs),
            PokemonSpriteSide::Back,
            false,
            0,
            &mut images,
        )
        .expect("actual pack rear portrait");
        let texture = images.get(&back.handle).expect("rear portrait texture");
        assert_eq!(texture.width(), 48, "{actor}: native rear width");
        assert_eq!(texture.height(), 48, "{actor}: native rear height");
        let mut expected =
            image::RgbaImage::from_raw(48, 48, texture.data.as_ref().expect("rear pixels").clone())
                .expect("RGBA rear portrait");
        let mut visible_inks = BTreeSet::new();
        for pixel in expected.pixels_mut() {
            assert!(pixel[3] == 0 || pixel[3] == 255, "{actor}: flat ink");
            if pixel[3] == 0 {
                *pixel = image::Rgba([255; 4]);
            } else {
                for channel in &pixel.0[..3] {
                    let value = channel >> 3;
                    assert_eq!(*channel, (value << 3) | (value >> 2));
                }
                visible_inks.insert(pixel.0);
            }
        }
        // Battler pictures occupy the GBC background bank, with four inks;
        // the three-visible-ink restriction applies to animation OBJ tiles.
        assert!(visible_inks.len() <= 4, "{actor}: native battler inks");
        assert!(
            back_fingerprints.insert(expected.as_raw().clone()),
            "{actor}: each species needs its own rear portrait"
        );

        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            let battle = snapshot.battle.as_ref().expect("battle");
            spawn_battle_battler_markers(
                &mut commands,
                &snapshot,
                battle,
                0,
                None,
                false,
                false,
                false,
                None,
                false,
                None,
                None,
                None,
                None,
                [false; 2],
                &mut art,
                &shell.asset_root,
                &mut images,
            )
            .expect("production classic battlers");
            spawn_battle_hud(
                &mut commands,
                &snapshot,
                battle,
                0,
                false,
                false,
                false,
                None,
                None,
                &data.growth_rates,
                &mut art,
                &shell.asset_root,
                &mut images,
            )
            .expect("production battle HUD");
        }
        queue.apply(&mut world);
        assert_eq!(
            world.query::<&BattleBattlerMarker>().iter(&world).count(),
            2,
            "{actor}: actual enemy front and player rear"
        );
        let matching_rears = world
            .query::<(&BattleBattlerMarker, &Sprite)>()
            .iter(&world)
            .filter(|(_, sprite)| sprite.image == back.handle)
            .count();
        assert_eq!(matching_rears, 1, "{actor}: selected rear reaches renderer");
        let full = render_pc_audit_canvas(&mut world, &images, actor);
        let mut native =
            image::imageops::resize(&full, 160, 144, image::imageops::FilterType::Nearest);
        for pixel in native.pixels_mut() {
            assert!(pixel[3] == 0 || pixel[3] == 255);
            if pixel[3] == 0 {
                *pixel = image::Rgba([255; 4]);
            }
        }
        let actual = image::imageops::crop_imm(&native, 16, 48, 48, 48).to_image();
        assert_eq!(
            actual, expected,
            "{actor}: real battle keeps the full native rear bank at its source anchor"
        );
        image::imageops::overlay(
            &mut board,
            &native,
            (index % 3 * 160) as i64,
            (index / 3 * 144) as i64,
        );
        if let Some(directory) = &output {
            let stem = actor.to_ascii_lowercase();
            native
                .save(directory.join(format!("{stem}.battle.png")))
                .expect("retain actual native battle");
            image::imageops::resize(&native, 640, 576, image::imageops::FilterType::Nearest)
                .save(directory.join(format!("{stem}.battle-4x.png")))
                .expect("retain nearest-neighbor battle review");
            actual
                .save(directory.join(format!("{stem}.battle-back.png")))
                .expect("retain battle-extracted rear portrait");
        }
    }
    if let Some(directory) = &output {
        board
            .save(directory.join("starters-battle.native.png"))
            .expect("retain native battle contact board");
        image::imageops::resize(&board, 1920, 2304, image::imageops::FilterType::Nearest)
            .save(directory.join("starters-battle.review.png"))
            .expect("retain enlarged battle contact board");
    }
}

// Capture the complete regional roster through the same production battler/HUD
// path as the starter review. Canonical files are checked against the external
// pack before any retained frame is written.
#[test]
fn northwoods_full_roster_renders_front_and_back_in_production_battles() {
    northwoods_capture_battle_species(&[], false, "NORTHWOODS_ROSTER_RENDER_DIR");
}

#[test]
fn northwoods_sticky_family_renders_normal_and_shiny_in_production_battles() {
    northwoods_capture_battle_species(
        &["JOTLET", "MEMOFOLD", "RECALLISK"],
        true,
        "NORTHWOODS_STICKY_RENDER_DIR",
    );
}

// Both regressions share the production renderer and exact asset-bank checks.
fn northwoods_capture_battle_species(selected: &[&str], include_shiny: bool, output_env: &str) {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    let output = std::env::var_os(output_env).map(PathBuf::from);
    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        assert!(output.is_none(), "roster capture requires an explicit pack");
        return;
    }
    let shell = route36_battle_shell_for_render_regression();
    assert!(
        shell.visible_move_animations.is_empty(),
        "source-bank portrait review starts with no move OBJ state"
    );
    let data = shell.runtime.data();
    if data.saved_species("KENSIGARD").is_none() {
        assert!(
            output.is_none(),
            "roster capture requires the current complete Northwoods pack"
        );
        return;
    }
    if let Some(directory) = &output {
        if directory.exists() {
            assert!(
                std::fs::read_dir(directory)
                    .expect("inspect retained roster capture root")
                    .next()
                    .is_none(),
                "roster capture directory contains retained renders; choose a fresh directory"
            );
        } else {
            std::fs::create_dir_all(directory).expect("create roster capture root");
        }
    }
    let source_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/pokemon");
    let native_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/minnesota-pokemon/native");
    let roster: serde_json::Value = serde_json::from_slice(
        &std::fs::read(source_root.join("roster.json")).expect("canonical regional identities"),
    )
    .expect("regional roster JSON");
    let curation: serde_json::Value = serde_json::from_slice(
        &std::fs::read(source_root.join("roster-curation.json")).expect("active roster curation"),
    )
    .expect("roster curation JSON");
    let profiles: serde_json::Value = serde_json::from_slice(
        &std::fs::read(source_root.join("species-profiles.json")).expect("authored shiny palettes"),
    )
    .expect("species profiles JSON");
    let retired = curation["retired_families"]
        .as_array()
        .expect("retired families")
        .iter()
        .map(|family| family["id"].as_str().expect("retired family ID"))
        .collect::<BTreeSet<_>>();
    let removed = curation["removed_evolutions"]
        .as_array()
        .expect("removed evolutions")
        .iter()
        .map(|member| member["id"].as_str().expect("removed species ID"))
        .collect::<BTreeSet<_>>();
    let mut expected_actors = BTreeSet::new();
    for family in roster["families"].as_array().expect("canonical families") {
        if retired.contains(family["id"].as_str().expect("family ID")) {
            continue;
        }
        for member in family["members"].as_array().expect("family members") {
            if !removed.contains(member["id"].as_str().expect("stable species ID")) {
                assert!(
                    expected_actors.insert(
                        member["name"]
                            .as_str()
                            .expect("species name")
                            .to_ascii_uppercase()
                    )
                );
            }
        }
    }
    assert_eq!(
        expected_actors.len(),
        curation["active_species_count"]
            .as_u64()
            .expect("active species count") as usize
    );
    let registry: serde_json::Value = serde_json::from_slice(
        &std::fs::read(source_root.join("sprite-generation.json"))
            .expect("canonical sprite registry"),
    )
    .expect("sprite registry JSON");
    let mut canonical = BTreeMap::new();
    for family in roster["families"].as_array().expect("canonical families") {
        let family_id = family["id"].as_str().expect("canonical family ID");
        let Some(entry) = registry["families"]
            .as_array()
            .expect("registered families")
            .iter()
            .find(|entry| entry["id"].as_str() == Some(family_id))
        else {
            continue;
        };
        let manifest_path = source_root.join(
            entry["manifest"]
                .as_str()
                .expect("registered manifest path"),
        );
        // Retired authoring records can lack reviewed sprites. Only actual pack
        // species below request an entry from this map.
        if !manifest_path.exists() {
            continue;
        }
        let manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(manifest_path).expect("canonical sprite manifest"),
        )
        .expect("sprite manifest JSON");
        for member in family["members"]
            .as_array()
            .expect("canonical family members")
        {
            let actor = member["name"]
                .as_str()
                .expect("canonical name")
                .to_ascii_uppercase();
            canonical.insert(
                actor,
                (
                    family_id.to_owned(),
                    member["id"].as_str().expect("stable species ID").to_owned(),
                    manifest.clone(),
                ),
            );
        }
    }
    let mut actors = data
        .pokemon
        .values()
        .filter(|species| species.int_id > 251)
        .collect::<Vec<_>>();
    actors.sort_by_key(|species| species.int_id);
    assert_eq!(
        actors
            .iter()
            .map(|species| species.id.clone())
            .collect::<BTreeSet<_>>(),
        expected_actors,
        "external pack must contain exactly the curated active regional identities"
    );
    if !selected.is_empty() {
        actors.retain(|species| selected.contains(&species.id.as_str()));
        assert_eq!(
            actors.len(),
            selected.len(),
            "all selected species exist in the completed pack"
        );
    }
    let variants: &[bool] = if include_shiny {
        &[false, true]
    } else {
        &[false]
    };
    let actors = actors
        .into_iter()
        .flat_map(|species| variants.iter().map(move |&shiny| (species, shiny)))
        .collect::<Vec<_>>();
    let base = shell.shell.snapshot().expect("production battle snapshot");
    let mut fingerprints = [BTreeSet::new(), BTreeSet::new()];
    let mut board = image::RgbaImage::from_pixel(640, 576, image::Rgba([255; 4]));
    let mut records = Vec::new();
    for (index, (species, shiny)) in actors.iter().enumerate() {
        let actor = species.id.as_str();
        let (family_id, stable_id, manifest) = canonical
            .get(actor)
            .expect("pack actor has canonical identity");
        let dvs = if *shiny {
            Dv::from_non_hp(2, 10, 10, 10)
        } else {
            Dv::from_non_hp(9, 10, 10, 10)
        };
        assert_eq!(visible_dvs_are_shiny(dvs), *shiny);
        let pokemon = crate::core::models::pokemon::create_pokemon_from_known_dvs(
            species,
            50,
            dvs,
            &data.learnsets,
            &data.moves,
            &data.growth_rates,
        )
        .expect("regional review Pokemon");
        let mut snapshot = base.clone();
        let active_index = snapshot
            .battle
            .as_ref()
            .expect("battle")
            .active_player_party_index
            .expect("active player slot");
        snapshot
            .party
            .slots
            .iter_mut()
            .find(|slot| slot.index == active_index)
            .expect("active party slot")
            .pokemon = pokemon.clone();
        let battle = snapshot.battle.as_mut().expect("battle");
        battle.enemy_pokemon = pokemon;
        battle.enemy_party = vec![battle.enemy_pokemon.clone()];
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        let mut expected_views = Vec::new();
        let mut frame_handles = Vec::new();
        for (view_index, (view, side, size)) in [
            ("front", PokemonSpriteSide::Front, 56),
            ("back", PokemonSpriteSide::Back, 48),
        ]
        .into_iter()
        .enumerate()
        {
            let import = manifest["sprites"]
                .as_array()
                .expect("canonical selected views")
                .iter()
                .find(|sprite| {
                    sprite["species_id"].as_str() == Some(stable_id)
                        && sprite["view"].as_str() == Some(view)
                })
                .expect("selected canonical view");
            let stem = format!(
                "{}.{view}",
                import["id"].as_str().expect("selected native stem")
            );
            let selected = native_root.join(family_id).join(stem);
            let pack_path =
                pokemon_asset_path(&shell.asset_root, &actor.to_ascii_lowercase(), side, "2bpp");
            assert_eq!(
                crate::read_runtime_asset(&pack_path).expect("pack native bitplanes"),
                std::fs::read(selected.with_extension(format!("{view}.2bpp")))
                    .expect("canonical native bitplanes"),
                "{actor}/{view}: exact selected 2bpp reaches pack"
            );
            let png = std::fs::read(selected.with_extension(format!("{view}.png")))
                .expect("canonical native PNG");
            let pack_png =
                pokemon_asset_path(&shell.asset_root, &actor.to_ascii_lowercase(), side, "png");
            assert_eq!(
                crate::read_runtime_asset(&pack_png).expect("pack native PNG"),
                png,
                "{actor}/{view}: exact selected PNG reaches pack"
            );
            let expected_palette = import["normal_palette_rgb555"]
                .as_array()
                .expect("independent species palette")
                .iter()
                .flat_map(|word| {
                    u16::try_from(word.as_u64().expect("RGB555 word"))
                        .expect("RGB555 range")
                        .to_le_bytes()
                })
                .collect::<Vec<_>>();
            let side_palette = pokemon_asset_path(
                &shell.asset_root,
                &actor.to_ascii_lowercase(),
                side,
                "gbcpal",
            );
            let pack_palette = if crate::runtime_asset_exists(&side_palette) {
                side_palette
            } else {
                shell
                    .asset_root
                    .runtime_assets()
                    .join("gfx/pokemon")
                    .join(actor.to_ascii_lowercase())
                    .join("normal.gbcpal")
            };
            assert_eq!(
                crate::read_runtime_asset(&pack_palette).expect("pack GBC palette"),
                expected_palette,
                "{actor}/{view}: exact independent RGB555 palette"
            );
            let mut expected = image::load_from_memory(&png)
                .expect("canonical PNG decode")
                .to_rgba8();
            if *shiny {
                let profile = profiles["species"]
                    .as_array()
                    .expect("profile batch")
                    .iter()
                    .find(|profile| profile["id"].as_str() == Some(stable_id))
                    .expect("selected shiny profile");
                let chromatic = profile["shiny_rgb5"]
                    .as_array()
                    .expect("two shiny chromatic inks");
                assert_eq!(chromatic.len(), 2);
                let mut shiny_words = vec![0x7fff_u16];
                let mut shiny_text = String::new();
                for color in chromatic {
                    let channels = color.as_array().expect("shiny RGB5 channels");
                    assert_eq!(channels.len(), 3);
                    let rgb = channels
                        .iter()
                        .map(|channel| {
                            u16::try_from(channel.as_u64().expect("RGB5 channel"))
                                .expect("RGB5 range")
                        })
                        .collect::<Vec<_>>();
                    assert!(rgb.iter().all(|&channel| channel <= 31));
                    shiny_words.push(rgb[0] | (rgb[1] << 5) | (rgb[2] << 10));
                    shiny_text.push_str(&format!("RGB {}, {}, {}\n", rgb[0], rgb[1], rgb[2]));
                }
                shiny_words.push(0);
                let shiny_path = shell
                    .asset_root
                    .runtime_assets()
                    .join("gfx/pokemon")
                    .join(actor.to_ascii_lowercase())
                    .join("shiny.pal");
                assert_eq!(
                    crate::read_runtime_asset(&shiny_path).expect("actual shiny palette source"),
                    shiny_text.as_bytes(),
                    "{actor}: exact authored shiny bank"
                );
                let rgba = |word: u16| {
                    let channel = |shift: u32| {
                        let value = ((word >> shift) & 31_u16) as u8;
                        (value << 3) | (value >> 2)
                    };
                    [channel(0), channel(5), channel(10), 255]
                };
                let normal_inks = import["normal_palette_rgb555"]
                    .as_array()
                    .expect("normal palette")
                    .iter()
                    .map(|word| {
                        rgba(
                            u16::try_from(word.as_u64().expect("RGB555 word"))
                                .expect("RGB555 range"),
                        )
                    })
                    .collect::<Vec<_>>();
                for pixel in expected.pixels_mut() {
                    let ink = normal_inks
                        .iter()
                        .position(|color| *color == pixel.0)
                        .expect("canonical palette index");
                    pixel.0 = rgba(shiny_words[ink]);
                }
            }
            assert_eq!(
                expected.dimensions(),
                (size, size),
                "{actor}/{view}: native bank dimensions"
            );
            let frame = pokemon_animation_frame_for_art(
                &mut art,
                &shell.asset_root,
                &pokemon_asset_id_for_dvs(actor, dvs),
                side,
                *shiny,
                0,
                &mut images,
            )
            .expect("production pack battler texture");
            let texture = images.get(&frame.handle).expect("production CPU texture");
            assert_eq!((texture.width(), texture.height()), (size, size));
            let mut actual = image::RgbaImage::from_raw(
                size,
                size,
                texture.data.as_ref().expect("texture pixels").clone(),
            )
            .expect("native RGBA texture");
            let mut inks = BTreeSet::new();
            for pixel in actual.pixels_mut() {
                assert!(
                    pixel[3] == 0 || pixel[3] == 255,
                    "{actor}/{view}: no alpha blending"
                );
                if pixel[3] == 0 {
                    *pixel = image::Rgba([255; 4]);
                }
                for channel in &pixel.0[..3] {
                    let value = channel >> 3;
                    assert_eq!(
                        *channel,
                        (value << 3) | (value >> 2),
                        "{actor}/{view}: GBC RGB555 channel"
                    );
                }
                inks.insert(pixel.0);
            }
            assert!(inks.len() <= 4, "{actor}/{view}: four flat battler inks");
            if actual != expected {
                if let Some(directory) = &output {
                    let stem = format!(
                        "failed-{}-{view}-{}",
                        actor.to_ascii_lowercase(),
                        if *shiny { "shiny" } else { "normal" }
                    );
                    actual
                        .save(directory.join(format!("{stem}.actual.png")))
                        .expect("retain mismatched production texture");
                    expected
                        .save(directory.join(format!("{stem}.expected.png")))
                        .expect("retain exact expected texture");
                }
                let difference = actual.enumerate_pixels().zip(expected.pixels()).find_map(
                    |((x, y, actual), expected)| {
                        (actual != expected).then_some((x, y, actual.0, expected.0))
                    },
                );
                panic!("{actor}/{view} shiny={shiny}: source bank pixels differ at {difference:?}");
            }
            assert!(
                fingerprints[view_index].insert(actual.as_raw().clone()),
                "{actor}/{view}: unique species art"
            );
            expected_views.push(expected);
            frame_handles.push(frame.handle);
        }
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            let battle = snapshot.battle.as_ref().expect("battle");
            spawn_battle_battler_markers(
                &mut commands,
                &snapshot,
                battle,
                0,
                None,
                false,
                false,
                false,
                None,
                false,
                None,
                None,
                None,
                None,
                [false; 2],
                &mut art,
                &shell.asset_root,
                &mut images,
            )
            .expect("production classic battlers");
            spawn_battle_hud(
                &mut commands,
                &snapshot,
                battle,
                0,
                false,
                false,
                false,
                None,
                None,
                &data.growth_rates,
                &mut art,
                &shell.asset_root,
                &mut images,
            )
            .expect("production battle HUD");
        }
        queue.apply(&mut world);
        assert_eq!(
            world.query::<&BattleBattlerMarker>().iter(&world).count(),
            2,
            "{actor}: both actual battlers"
        );
        for handle in &frame_handles {
            assert_eq!(
                world
                    .query::<(&BattleBattlerMarker, &Sprite)>()
                    .iter(&world)
                    .filter(|(_, sprite)| sprite.image == *handle)
                    .count(),
                1,
                "{actor}: selected texture reaches production renderer"
            );
        }
        let full = render_pc_audit_canvas(&mut world, &images, actor);
        let mut native =
            image::imageops::resize(&full, 160, 144, image::imageops::FilterType::Nearest);
        for pixel in native.pixels_mut() {
            if pixel[3] == 0 {
                *pixel = image::Rgba([255; 4]);
            }
        }
        for ((x, y, size), expected) in [(96, 0, 56), (16, 48, 48)].into_iter().zip(&expected_views)
        {
            assert_eq!(
                image::imageops::crop_imm(&native, x, y, size, size).to_image(),
                *expected,
                "{actor}: native front/back raster at authentic production anchor"
            );
        }
        if index % 16 == 0 {
            board = image::RgbaImage::from_pixel(640, 576, image::Rgba([255; 4]));
        }
        image::imageops::overlay(
            &mut board,
            &native,
            (index % 4 * 160) as i64,
            (index % 16 / 4 * 144) as i64,
        );
        if let Some(directory) = &output {
            let stem = format!(
                "{:03}-{}{}",
                index + 1,
                actor.to_ascii_lowercase(),
                if *shiny { "-shiny" } else { "" }
            );
            native
                .save(directory.join(format!("{stem}.battle.png")))
                .expect("retain native production battle");
            image::imageops::resize(&native, 640, 576, image::imageops::FilterType::Nearest)
                .save(directory.join(format!("{stem}.battle-4x.png")))
                .expect("retain enlarged battle review");
            for (view, expected) in ["front", "back"].into_iter().zip(&expected_views) {
                expected
                    .save(directory.join(format!("{stem}.{view}.png")))
                    .expect("retain exact battle bank");
            }
            if index % 16 == 15 || index + 1 == actors.len() {
                let group = format!("roster-battles-{:02}", index / 16 + 1);
                board
                    .save(directory.join(format!("{group}.native.png")))
                    .expect("retain native grouped battle board");
                image::imageops::resize(&board, 1280, 1152, image::imageops::FilterType::Nearest)
                    .save(directory.join(format!("{group}.review.png")))
                    .expect("retain readable grouped battle board");
            }
        }
        records.push(serde_json::json!({"review_index":index+1,"actor":actor,"runtime_id":species.int_id,"family_id":family_id,"stable_id":stable_id,"front_anchor":[96,0],"back_anchor":[16,48],"native_and_pack_bytes_verified":true,"shiny":shiny}));
    }
    if let Some(directory) = &output {
        std::fs::write(
            directory.join("species-order.json"),
            serde_json::to_vec_pretty(&records).expect("review order JSON"),
        )
        .expect("retain battle coverage order");
    }
}
