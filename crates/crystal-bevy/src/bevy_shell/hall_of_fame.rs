// Hall of Fame presentation uses the existing pack artwork and source LCD
// coordinates. The runtime sequence owns timing; this module only composes art.
fn render_visible_hall_of_fame_member(
    pokemon: &crate::core::models::Pokemon,
    animation_frame: u16,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<SpriteFrame> {
    anyhow::ensure!(!pokemon.is_egg, "Hall of Fame cannot display an egg record");
    let asset_id = pokemon_asset_id_for_dvs(&pokemon.species.id, pokemon.dvs);
    let picture = pokemon_animation_frame_for_art(
        rendered_art, asset_root, &asset_id, PokemonSpriteSide::Front,
        visible_pokemon_is_shiny(pokemon), animation_frame, images,
    ).with_context(|| format!("Hall of Fame picture {asset_id} is unavailable"))?;
    let picture = battle_padded_frontpic(rendered_art, images, &picture)?;
    let border = crate::open_runtime_image(asset_root.runtime_assets().join("gfx/frames/1.png"))?
        .to_rgba8();
    let mut data = vec![255; 160 * 144 * 4];
    blit_sprite_frame_image(images.get(&picture.handle).context("Hall of Fame image missing")?,
        6 * 8, 5 * 8, 160, 144, &mut data);
    draw_time_set_window(&border, 0, 0, 20, 5, &mut data)?;
    draw_time_set_window(&border, 0, 12, 20, 6, &mut data)?;
    let mut draw_text = |text: &str, x: usize, y: usize, pixels: &mut [u8]| -> Result<()> {
        let frames = bitmap_text_frames(rendered_art, asset_root, images, text);
        if let Some(error) = rendered_art.font_error.as_deref() { anyhow::bail!("{error}"); }
        for (index, glyph) in frames.iter().enumerate() {
            blit_sprite_frame_image(images.get(&glyph.handle).context("Hall of Fame glyph missing")?,
                x + index * 8, y, 160, 144, pixels);
        }
        Ok(())
    };
    draw_text("New Hall of Famer!", 8, 16, &mut data)?;
    // DisplayHOFMon prints the national number, species, nickname, level and
    // original trainer ID at separate source tile positions.
    draw_text(&format!("№.{:03}", pokemon.species.int_id), 8, 13 * 8, &mut data)?;
    draw_text(&crate::core::models::pokemon_species_display_name(&pokemon.species.id),
        7 * 8, 13 * 8, &mut data)?;
    let gender = match crate::core::battle::turn::battle_pokemon_gender(pokemon) {
        Some(crate::core::battle::turn::BattlePokemonGender::Male) => "♂",
        Some(crate::core::battle::turn::BattlePokemonGender::Female) => "♀",
        None => " ",
    };
    draw_text(gender, 18 * 8, 13 * 8, &mut data)?;
    let nickname = pokemon.nickname.chars().take(10).collect::<String>();
    draw_text(&format!("/{nickname}"), 8 * 8, 14 * 8, &mut data)?;
    draw_text(&format!("<LV>{}", pokemon.level), 8, 16 * 8, &mut data)?;
    draw_text(&format!("<ID>№/{:05}", pokemon.original_trainer_id),
        7 * 8, 16 * 8, &mut data)?;
    let mut image = Image::new(Extent3d { width: 160, height: 144, depth_or_array_layers: 1 },
        TextureDimension::D2, data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    image.sampler = ImageSampler::nearest();
    Ok(SpriteFrame { handle: images.add(image), size: Vec2::new(160.0, 144.0) })
}
