// Optional art-review batches amortize real map rendering across up to six
// species. Only this test's owned session/mount receives review actors/assets.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PokemonOverworldBatchInput {
    species: Vec<PokemonOverworldBatchSpecies>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PokemonOverworldBatchSpecies {
    token: String,
    runtime_dir: PathBuf,
}

#[test]
fn species_overworld_batch_renders_normal_poses_and_verifies_shiny_banks() {
    use crate::core::world::collision::{
        PlayerTraversalState, Terrain, can_enter_tile, describe_collision, sample_collision,
    };
    use sha2::{Digest, Sha256};

    let Some(manifest) = std::env::var_os("NORTHWOODS_OVERWORLD_BATCH_MANIFEST").map(PathBuf::from)
    else {
        return;
    };
    assert!(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_some());
    let input_bytes = std::fs::read(&manifest).expect("batch input manifest");
    let input: PokemonOverworldBatchInput =
        serde_json::from_slice(&input_bytes).expect("typed batch inputs");
    assert!(
        (1..=6).contains(&input.species.len()),
        "one to six species per native context batch"
    );
    let output = PathBuf::from(
        std::env::var_os("NORTHWOODS_OVERWORLD_BATCH_RENDER_DIR")
            .expect("fresh durable batch output folder"),
    );
    if output.exists() {
        assert!(
            std::fs::read_dir(&output)
                .expect("inspect review folder")
                .next()
                .is_none(),
            "preserve previous renders: use a fresh batch output folder"
        );
    }
    std::fs::create_dir_all(&output).expect("batch output folder");
    std::fs::write(output.join("input-manifest.json"), &input_bytes)
        .expect("exact batch source manifest");

    let mut shell = route36_overworld_shell_for_battle_render_regression();
    assert!(shell.runtime.has_runtime_files());
    assert!(
        shell
            .asset_root
            .repository_root
            .file_name()
            .expect("owned mount name")
            .to_string_lossy()
            .starts_with("crystal-pack-assets-")
    );
    let mut tokens = std::collections::BTreeSet::new();
    let mut sources = Vec::new();
    let mut banks = Vec::new();
    let mut palettes = Vec::new();
    for item in &input.species {
        assert!(
            tokens.insert(item.token.clone()),
            "each batch species appears once"
        );
        assert_eq!(item.token, item.token.to_ascii_lowercase());
        assert!(
            item.token
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        );
        let species = shell
            .runtime
            .data()
            .saved_species(&item.token.to_ascii_uppercase())
            .expect("actual Northwoods species");
        assert!(species.int_id > 251);
        let source = item.runtime_dir.join("gfx/pokemon").join(&item.token);
        let destination = shell
            .asset_root
            .runtime_assets()
            .join("gfx/pokemon")
            .join(&item.token);
        std::fs::create_dir_all(&destination).expect("temporary species mount folder");
        let mut hashes = std::collections::BTreeMap::new();
        for name in [
            "overworld.2bpp",
            "overworld.gbcpal",
            "overworld-shiny.gbcpal",
        ] {
            let bytes = std::fs::read(source.join(name)).expect("actual converted runtime asset");
            hashes.insert(name, format!("{:x}", Sha256::digest(&bytes)));
            std::fs::write(destination.join(name), bytes).expect("private fixture asset copy");
        }
        let bank = std::fs::read(source.join("overworld.2bpp")).expect("native bank");
        assert_eq!(bank.len(), 384);
        banks.push(bank);
        let normal = pokemon_overworld_batch_palette(
            &std::fs::read(source.join("overworld.gbcpal")).expect("normal palette"),
        );
        let shiny = pokemon_overworld_batch_palette(
            &std::fs::read(source.join("overworld-shiny.gbcpal")).expect("shiny palette"),
        );
        palettes.push((normal, shiny));
        sources.push(serde_json::json!({"token":item.token,"runtime_dir":item.runtime_dir,"assets_sha256":hashes}));
    }

    let player = shell
        .shell
        .snapshot()
        .expect("Route36 snapshot")
        .overworld
        .tile;
    let (positions, permissions) = {
        let map = shell.shell.session().overworld();
        let occupied = map
            .occupied_tiles_checked()
            .expect("authored occupied tiles");
        let mut candidates = Vec::new();
        for dy in -3i16..=3 {
            for dx in -4i16..=4 {
                let tile = TilePosition::new(player.x + dx, player.y + dy);
                if tile == player || occupied.iter().any(|occupied| occupied.tile == tile) {
                    continue;
                }
                let Some(collision) = sample_collision(&map.map, &map.tileset, tile) else {
                    continue;
                };
                if describe_collision(collision.permission).terrain != Terrain::Land {
                    continue;
                }
                if ![
                    Direction::Down,
                    Direction::Up,
                    Direction::Left,
                    Direction::Right,
                ]
                .into_iter()
                .all(|direction| {
                    can_enter_tile(
                        &map.map,
                        &map.tileset,
                        tile,
                        direction,
                        PlayerTraversalState::Walk,
                    )
                }) {
                    continue;
                }
                if map
                    .map_events
                    .warps
                    .iter()
                    .any(|warp| warp.x == tile.x as u16 && warp.y == tile.y as u16)
                {
                    continue;
                }
                candidates.push((
                    dx.abs() + dy.abs(),
                    dy.abs(),
                    dx,
                    tile,
                    collision.permission,
                ));
            }
        }
        candidates.sort_by_key(|candidate| (candidate.0, candidate.1, candidate.2));
        let mut selected: Vec<(TilePosition, u8)> = Vec::new();
        for spacing in [2, 1] {
            selected.clear();
            for (_, _, _, tile, permission) in &candidates {
                if selected.iter().all(|(other, _)| {
                    (other.x - tile.x).abs().max((other.y - tile.y).abs()) >= spacing
                }) {
                    selected.push((*tile, *permission));
                    if selected.len() == input.species.len() {
                        break;
                    }
                }
            }
            if selected.len() == input.species.len() {
                break;
            }
        }
        assert_eq!(
            selected.len(),
            input.species.len(),
            "batch needs enough visible walkable Route36 land beside trainer"
        );
        selected.into_iter().unzip::<_, _, Vec<_>, Vec<_>>()
    };
    let mut actor_ids = Vec::new();
    {
        let map = shell.shell.session_mut().overworld_mut();
        let template = map
            .objects
            .iter()
            .find(|object| object.object_identifier.is_some())
            .expect("authored object template")
            .clone();
        for (index, (item, tile)) in input.species.iter().zip(&positions).enumerate() {
            let id = format!("OVERWORLD_REVIEW_{index}");
            let mut actor = template.clone();
            actor.sprite = format!("SPRITE_{}", item.token.to_ascii_uppercase());
            actor.sprite_has_facings = true;
            actor.spritemovedata = "SPRITEMOVEDATA_WALK_LEFT_RIGHT".into();
            actor.x = u16::try_from(tile.x).expect("positive review x");
            actor.y = u16::try_from(tile.y).expect("positive review y");
            actor.event_flag.clear();
            actor.object_identifier = Some(id.clone());
            actor.pal = 0;
            map.objects.push(actor);
            map.set_object_runtime_tile(&id, *tile)
                .expect("private review actor placement");
            actor_ids.push(id);
        }
        // New cloned records enter through the normal map-object setup
        // boundary. A same-tile roster refresh intentionally loads no actors.
        map.begin_map_object_setup();
        map.finish_map_object_setup()
            .expect("normal visible object roster");
        for id in &actor_ids {
            assert!(
                map.object_struct_slot(id).is_some(),
                "review actor {id} receives a normal object slot"
            );
        }
    }
    for ((source, tile), permission) in sources.iter_mut().zip(&positions).zip(permissions) {
        source["runtime_tile"] = serde_json::json!([tile.x, tile.y]);
        source["collision_permission"] = serde_json::json!(permission);
    }

    let mut app = battle_render_regression_app(shell);
    app.add_systems(Update, sync_visible_object_sprites.after(render_playfield));
    let mut context_board = image::RgbaImage::new(640, 288);
    let mut normal_banks = input
        .species
        .iter()
        .map(|_| image::RgbaImage::new(16, 96))
        .collect::<Vec<_>>();
    let mut rendered_hashes = Vec::new();
    for (direction_index, (direction, label)) in [
        (Direction::Down, "down"),
        (Direction::Up, "up"),
        (Direction::Left, "left"),
        (Direction::Right, "right"),
    ]
    .into_iter()
    .enumerate()
    {
        for walking in [false, true] {
            {
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                shell.object_walk_from.clear();
                for (id, tile) in actor_ids.iter().zip(&positions) {
                    shell
                        .shell
                        .session_mut()
                        .overworld_mut()
                        .set_object_runtime_facing(id, direction)
                        .expect("batch pose facing");
                    if walking {
                        shell.object_walk_from.insert(id.clone(), *tile);
                        shell.object_walk_phases.insert(id.clone(), 1);
                        shell.object_walk_frame_ticks_by_id.insert(id.clone(), 1);
                        shell.object_walk_total_ticks_by_id.insert(id.clone(), 1);
                    }
                }
                mark_runtime_snapshot_dirty(&mut shell);
            }
            app.update();
            let world = app.world_mut();
            assert!(
                world
                    .query_filtered::<&Sprite, With<PlayerMarker>>()
                    .iter(world)
                    .next()
                    .is_some()
            );
            assert!(
                world
                    .query_filtered::<&Sprite, With<PlayfieldTile>>()
                    .iter(world)
                    .next()
                    .is_some()
            );
            let mut actors = world.query::<(&VisibleObjectSprite, &Sprite, &Transform)>();
            let selected = actor_ids
                .iter()
                .map(|id| {
                    let (actor, sprite, transform) = actors
                        .iter(world)
                        .find(|(actor, _, _)| actor.object_identifier.as_deref() == Some(id))
                        .expect("each review species reaches actual map renderer");
                    assert!(actor.source_id.ends_with(":normal"));
                    assert_eq!(
                        sprite.image,
                        if walking {
                            actor.walking.clone().expect("authored action frame")
                        } else {
                            actor.standing.clone()
                        }
                    );
                    assert_eq!(
                        sprite.custom_size,
                        Some(Vec2::splat(16.0 * TILE_SIZE / SOURCE_TILE_SIZE as f32))
                    );
                    (sprite.image.clone(), transform.translation)
                })
                .collect::<Vec<_>>();
            let images = world
                .remove_resource::<Assets<Image>>()
                .expect("production image bank");
            let frame_index = direction_index.min(2) + if walking { 3 } else { 0 };
            for (index, (handle, translation)) in selected.iter().enumerate() {
                let texture = images.get(handle).expect("actual selected map texture");
                assert_eq!((texture.width(), texture.height()), (16, 16));
                let rgba = texture.data.as_ref().expect("native actor pixels");
                assert_eq!(
                    *rgba,
                    pokemon_overworld_batch_expected(
                        &banks[index],
                        &palettes[index].0,
                        frame_index,
                        direction == Direction::Right
                    ),
                    "selected pixels must exactly equal authored bank and species palette"
                );
                let actor =
                    image::RgbaImage::from_raw(16, 16, rgba.clone()).expect("native actor crop");
                let pose = if walking { "step" } else { "stand" };
                let folder = output.join(&input.species[index].token);
                std::fs::create_dir_all(&folder).expect("species review folder");
                actor
                    .save(folder.join(format!("{label}-{pose}.actor.png")))
                    .expect("actual selected16px actor");
                if direction_index < 3 {
                    image::imageops::overlay(
                        &mut normal_banks[index],
                        &actor,
                        0,
                        (frame_index * 16) as i64,
                    );
                }
                rendered_hashes.push(serde_json::json!({"token":input.species[index].token,"direction":label,"pose":pose,"rgba_sha256":format!("{:x}",Sha256::digest(rgba)),"world_translation":[translation.x,translation.y,translation.z]}));
            }
            let context = pokemon_overworld_context_canvas(world, &images);
            let native =
                image::imageops::resize(&context, 160, 144, image::imageops::FilterType::Nearest);
            let pose = if walking { "step" } else { "stand" };
            native
                .save(output.join(format!("context-{label}-{pose}.png")))
                .expect("native batch context");
            context
                .save(output.join(format!("context-{label}-{pose}.4x.png")))
                .expect("nearest context review");
            let scene = direction_index * 2 + usize::from(walking);
            image::imageops::overlay(
                &mut context_board,
                &native,
                ((scene % 4) * 160) as i64,
                ((scene / 4) * 144) as i64,
            );
            world.insert_resource(images);
        }
    }
    let world = app.world_mut();
    let asset_root = world.resource::<BevyRuntimeShell>().asset_root.clone();
    let mut images = world
        .remove_resource::<Assets<Image>>()
        .expect("actual sprite image store");
    for (index, item) in input.species.iter().enumerate() {
        let shiny = pokemon_overworld_art::load(
            &asset_root,
            &format!("pokemon_overworld:{}:shiny", item.token),
            &mut images,
        )
        .expect("actual own shiny palette loader")
        .expect("custom shiny art");
        let frames = [
            &shiny.down.standing,
            &shiny.up.standing,
            &shiny.left.standing,
            shiny.down.walking.as_ref().expect("shiny down step"),
            shiny.up.walking.as_ref().expect("shiny up step"),
            shiny.left.walking.as_ref().expect("shiny left step"),
        ];
        let mut shiny_bank = image::RgbaImage::new(16, 96);
        for (frame_index, frame) in frames.into_iter().enumerate() {
            let texture = images.get(&frame.handle).expect("shiny native texture");
            let rgba = texture.data.as_ref().expect("shiny pixels");
            assert_eq!(
                *rgba,
                pokemon_overworld_batch_expected(
                    &banks[index],
                    &palettes[index].1,
                    frame_index,
                    false
                ),
                "all six shiny poses preserve exact indexed art and own palette"
            );
            let crop = image::RgbaImage::from_raw(16, 16, rgba.clone()).expect("shiny pose");
            image::imageops::overlay(&mut shiny_bank, &crop, 0, (frame_index * 16) as i64);
        }
        let folder = output.join(&item.token);
        normal_banks[index]
            .save(folder.join("normal-bank.png"))
            .expect("selected normal bank");
        shiny_bank
            .save(folder.join("shiny-bank.png"))
            .expect("verified shiny bank");
        let mut actor_board = image::RgbaImage::from_pixel(96, 32, image::Rgba([255; 4]));
        for (row, bank) in [&normal_banks[index], &shiny_bank].into_iter().enumerate() {
            for frame in 0..6 {
                let crop = image::imageops::crop_imm(bank, 0, frame * 16, 16, 16).to_image();
                image::imageops::overlay(
                    &mut actor_board,
                    &crop,
                    i64::from(frame * 16),
                    (row * 16) as i64,
                );
            }
        }
        actor_board
            .save(folder.join("actor-board.native.png"))
            .expect("native normal/shiny pose board");
        for scale in [4, 8] {
            image::imageops::resize(
                &actor_board,
                96 * scale,
                32 * scale,
                image::imageops::FilterType::Nearest,
            )
            .save(folder.join(format!("actor-board.{scale}x.png")))
            .expect("nearest actor review board");
        }
    }
    world.insert_resource(images);
    context_board
        .save(output.join("all8-context.native.png"))
        .expect("all native batch scenes");
    image::imageops::resize(
        &context_board,
        1280,
        576,
        image::imageops::FilterType::Nearest,
    )
    .save(output.join("all8-context.2x.png"))
    .expect("nearest batch scenes");
    image::imageops::resize(
        &context_board,
        2560,
        1152,
        image::imageops::FilterType::Nearest,
    )
    .save(output.join("all8-context.4x.png"))
    .expect("large nearest batch scene board");
    std::fs::write(output.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"map":"Route36","player_tile":[player.x,player.y],"scene_count":8,"species_count":input.species.len(),"species":sources,"rendered_actor_frames":rendered_hashes,"shiny_bank_frames_checked":input.species.len()*6,"test_fixture_only":true})).expect("serialize evidence")).expect("batch provenance");
}

fn pokemon_overworld_batch_palette(bytes: &[u8]) -> [[u8; 3]; 4] {
    assert_eq!(bytes.len(), 8);
    std::array::from_fn(|index| {
        let word = u16::from_le_bytes([bytes[index * 2], bytes[index * 2 + 1]]);
        assert!(word <= 0x7fff);
        gbc_color_to_rgb(word)
    })
}

fn pokemon_overworld_batch_expected(
    bank: &[u8],
    palette: &[[u8; 3]; 4],
    frame: usize,
    mirrored: bool,
) -> Vec<u8> {
    let mut pixels = vec![0; 16 * 16 * 4];
    for y in 0..16 {
        for x in 0..16 {
            let source_x = if mirrored { 15 - x } else { x };
            let offset = (frame * 4 + (y / 8) * 2 + source_x / 8) * 16 + (y % 8) * 2;
            let shift = 7 - source_x % 8;
            let index = usize::from(
                ((bank[offset] >> shift) & 1) | (((bank[offset + 1] >> shift) & 1) << 1),
            );
            let target = (y * 16 + x) * 4;
            if index != 0 {
                pixels[target..target + 3].copy_from_slice(&palette[index]);
                pixels[target + 3] = 255;
            }
        }
    }
    pixels
}
