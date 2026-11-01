// Opt-in review of real converted overworld banks on the production map.
// Runtime data/art live in an owned temporary pack mount; no pack is rewritten.
#[test]
fn species_overworld_art_renders_at_native_scale_with_map_and_trainer() {
    let Some(input) = std::env::var_os("NORTHWOODS_OVERWORLD_RUNTIME_DIR").map(PathBuf::from)
    else {
        return;
    };
    assert!(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_some());
    let output = std::env::var_os("NORTHWOODS_OVERWORLD_RENDER_DIR").map(PathBuf::from);
    let token = std::env::var("NORTHWOODS_OVERWORLD_SPECIES")
        .expect("explicit species token for retained overworld context review")
        .to_ascii_lowercase();
    let mut shell = route36_overworld_shell_for_battle_render_regression();
    assert!(
        shell
            .runtime
            .data()
            .saved_species(&token.to_ascii_uppercase())
            .is_some()
    );
    assert!(
        shell.runtime.has_runtime_files(),
        "fixture requires an owned materialized pack mount"
    );
    assert!(
        shell
            .asset_root
            .repository_root
            .file_name()
            .expect("mount name")
            .to_string_lossy()
            .starts_with("crystal-pack-assets-")
    );
    let source = input.join("gfx/pokemon").join(&token);
    let destination = shell
        .asset_root
        .runtime_assets()
        .join("gfx/pokemon")
        .join(&token);
    std::fs::create_dir_all(&destination).expect("temporary fixture art folder");
    for name in [
        "overworld.2bpp",
        "overworld.gbcpal",
        "overworld-shiny.gbcpal",
    ] {
        std::fs::copy(source.join(name), destination.join(name))
            .expect("real converted species art");
    }
    let species = shell
        .runtime
        .data()
        .saved_species(&token.to_ascii_uppercase())
        .expect("actual species profile");
    let shiny = crate::core::models::pokemon::create_pokemon_from_known_dvs(
        &species,
        50,
        Dv::from_non_hp(2, 10, 10, 10),
        &shell.runtime.data().learnsets,
        &shell.runtime.data().moves,
        &shell.runtime.data().growth_rates,
    )
    .expect("real shiny resident fixture");
    assert!(visible_pokemon_is_shiny(&shiny));
    let resident = &mut shell.shell.session_mut().state_mut().day_care.man;
    resident.pokemon = Some(shiny);
    resident.active = true;
    let player = shell.shell.snapshot().expect("map fixture").overworld.tile;
    let object_tile = TilePosition::new(player.x + 2, player.y);
    let object_id = {
        let map = shell.shell.session_mut().overworld_mut();
        let object = map
            .objects
            .iter_mut()
            .find(|object| object.object_identifier.is_some())
            .expect("real authored map object");
        object.sprite = format!("SPRITE_{}", token.to_ascii_uppercase());
        object.spritemovedata = "SPRITEMOVEDATA_WALK_LEFT_RIGHT".into();
        let id = object.object_identifier.clone().expect("fixture object ID");
        map.set_object_runtime_tile(&id, object_tile)
            .expect("place review actor beside trainer");
        id
    };
    let mut app = battle_render_regression_app(shell);
    // The normal schedule also settles retained standing/action handles.
    // A render-only schedule would leave the previous idle handle in place
    // when the position-only redraw fast path retains the object entity.
    app.add_systems(Update, sync_visible_object_sprites.after(render_playfield));
    for shiny in [false, true] {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            let object = shell
                .shell
                .session_mut()
                .overworld_mut()
                .objects
                .iter_mut()
                .find(|object| object.object_identifier.as_deref() == Some(&object_id))
                .expect("same authored actor");
            object.sprite = if shiny {
                "SPRITE_DAY_CARE_MON_1".into()
            } else {
                format!("SPRITE_{}", token.to_ascii_uppercase())
            };
        }
        for (direction, label) in [
            (Direction::Down, "down"),
            (Direction::Up, "up"),
            (Direction::Left, "left"),
            (Direction::Right, "right"),
        ] {
            for walking in [false, true] {
                {
                    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                    shell
                        .shell
                        .session_mut()
                        .overworld_mut()
                        .set_object_runtime_facing(&object_id, direction)
                        .expect("review direction");
                    shell.object_walk_from.clear();
                    if walking {
                        shell
                            .object_walk_from
                            .insert(object_id.clone(), object_tile);
                        shell.object_walk_phases.insert(object_id.clone(), 1);
                        shell
                            .object_walk_frame_ticks_by_id
                            .insert(object_id.clone(), 1);
                        shell
                            .object_walk_total_ticks_by_id
                            .insert(object_id.clone(), 1);
                    }
                    mark_runtime_snapshot_dirty(&mut shell);
                }
                app.update();
                let world = app.world_mut();
                let (image_handle, size) = {
                    let mut actors = world.query::<(&VisibleObjectSprite, &Sprite)>();
                    let (actor, sprite) = actors
                        .iter(world)
                        .find(|(actor, _)| actor.object_identifier.as_deref() == Some(&object_id))
                        .expect("species reaches actual map object renderer");
                    assert!(
                        actor
                            .source_id
                            .ends_with(if shiny { ":shiny" } else { ":normal" }),
                        "actual named/resident appearance chooses its own palette namespace"
                    );
                    assert!(
                        actor.walking.is_some(),
                        "real six-frame actor retains its walk pose"
                    );
                    assert_eq!(
                        sprite.image,
                        if walking {
                            actor.walking.clone().expect("walk")
                        } else {
                            actor.standing.clone()
                        }
                    );
                    (
                        sprite.image.clone(),
                        sprite.custom_size.expect("native actor size"),
                    )
                };
                assert_eq!(
                    size,
                    Vec2::splat(16.0 * TILE_SIZE / SOURCE_TILE_SIZE as f32)
                );
                assert!(
                    world
                        .query_filtered::<&Sprite, With<PlayerMarker>>()
                        .iter(world)
                        .next()
                        .is_some(),
                    "production trainer remains in context"
                );
                assert!(
                    world
                        .query_filtered::<&Sprite, With<PlayfieldTile>>()
                        .iter(world)
                        .next()
                        .is_some(),
                    "production terrain remains in context"
                );
                let images = world
                    .remove_resource::<Assets<Image>>()
                    .expect("native textures");
                let texture = images.get(&image_handle).expect("species native texture");
                assert_eq!((texture.width(), texture.height()), (16, 16));
                if let Some(output) = &output {
                    let stem = format!(
                        "{token}-{}-{label}-{}",
                        if shiny { "shiny" } else { "normal" },
                        if walking { "step" } else { "stand" }
                    );
                    std::fs::create_dir_all(output).expect("retained review folder");
                    let context = pokemon_overworld_context_canvas(world, &images);
                    let native = image::imageops::resize(
                        &context,
                        160,
                        144,
                        image::imageops::FilterType::Nearest,
                    );
                    native
                        .save(output.join(format!("{stem}.png")))
                        .expect("native map context");
                    context
                        .save(output.join(format!("{stem}.4x.png")))
                        .expect("nearest review context");
                    image::RgbaImage::from_raw(
                        16,
                        16,
                        texture.data.as_ref().expect("native ink").clone(),
                    )
                    .expect("RGBA native actor")
                    .save(output.join(format!("{stem}.actor.png")))
                    .expect("exact native actor pixels");
                }
                world.insert_resource(images);
            }
        }
    }
}

fn pokemon_overworld_context_canvas(world: &mut World, images: &Assets<Image>) -> image::RgbaImage {
    let mut query = world.query::<(&Sprite, &Transform, Option<&Visibility>)>();
    let mut sprites = query
        .iter(world)
        .filter(|(_, _, visibility)| !matches!(visibility, Some(Visibility::Hidden)))
        .collect::<Vec<_>>();
    sprites.sort_by(|a, b| a.1.translation.z.total_cmp(&b.1.translation.z));
    let mut canvas = image::RgbaImage::new(PLAYFIELD_WIDTH as u32, PLAYFIELD_HEIGHT as u32);
    for (sprite, transform, _) in sprites {
        let Some(size) = sprite.custom_size else {
            continue;
        };
        let mut raster = if let Some(source) = images.get(&sprite.image) {
            image::RgbaImage::from_raw(
                source.width(),
                source.height(),
                source.data.as_ref().expect("CPU native pixels").clone(),
            )
            .expect("RGBA render texture")
        } else if sprite.image == Handle::<Image>::default() {
            // Bevy's untextured Sprite is a white quad tinted by its color.
            // Party backdrops use this path and must cover retained battlers.
            image::RgbaImage::from_pixel(1, 1, image::Rgba([255; 4]))
        } else {
            continue;
        };
        if let Some(rect) = sprite.rect {
            raster = image::imageops::crop_imm(
                &raster,
                rect.min.x as u32,
                rect.min.y as u32,
                rect.width() as u32,
                rect.height() as u32,
            )
            .to_image();
        }
        if sprite.flip_x {
            image::imageops::flip_horizontal_in_place(&mut raster);
        }
        if sprite.flip_y {
            image::imageops::flip_vertical_in_place(&mut raster);
        }
        let tint = sprite.color.to_srgba();
        for pixel in raster.pixels_mut() {
            for (channel, tint) in pixel
                .0
                .iter_mut()
                .zip([tint.red, tint.green, tint.blue, tint.alpha])
            {
                *channel = (f32::from(*channel) * tint).round() as u8;
            }
        }
        let scaled = image::imageops::resize(
            &raster,
            size.x as u32,
            size.y as u32,
            image::imageops::FilterType::Nearest,
        );
        let x = (transform.translation.x - size.x / 2.0 - PLAYFIELD_LEFT).round() as i64;
        let y = (PLAYFIELD_TOP - transform.translation.y - size.y / 2.0).round() as i64;
        image::imageops::overlay(&mut canvas, &scaled, x, y);
    }
    canvas
}
