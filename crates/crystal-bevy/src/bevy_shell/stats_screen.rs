struct SourceStatsLayout {
    tiles: [u8; 360],
    text: [char; 360],
}

fn stats_mon_uses_menu_animation(pokemon: &crate::core::models::Pokemon) -> bool {
    // StatsScreen_GetAnimationParam / CheckFaintedFrzSlp. EggStatsScreen
    // has its own animation and optional SFX_2_BOOPS, never a species cry.
    !pokemon.is_egg
        && pokemon.hp != 0
        && !matches!(pokemon.status.as_deref(), Some("SLEEP" | "FREEZE"))
}

impl Default for SourceStatsLayout {
    fn default() -> Self {
        Self {
            tiles: [0x7f; 360],
            text: [' '; 360],
        }
    }
}

fn queue_visible_stats_egg_sound(
    runtime_shell: &mut BevyRuntimeShell,
    pokemon: &crate::core::models::Pokemon,
) -> Result<()> {
    // EggStatsScreen uses the retained hatch-cycle byte, independently of
    // HP/status and the species cry predicate in StatsScreen_GetAnimationParam.
    if pokemon.is_egg && pokemon.happiness < 6 {
        queue_visible_shell_sound_effect(runtime_shell, "SFX_2_BOOPS")?;
    }
    Ok(())
}

// StatsScreen_InitUpperHalf and LoadPink/Green/BluePage use a 20x18 tilemap.
fn stats_write_text(map: &mut SourceStatsLayout, x: usize, y: usize, text: &str) -> Result<()> {
    let charmap = bitmap_font_char_map();
    let text = normalize_bitmap_font_text(text);
    for (index, ch) in text.chars().enumerate() {
        anyhow::ensure!(
            x + index < 20 && y < 18,
            "Stats text {text:?} exceeds its tile row"
        );
        map.text[y * 20 + x + index] = ch;
        map.tiles[y * 20 + x + index] = match ch {
            '№' => 0x74,
            '⁂' => 0x3f,
            _ => u8::try_from(
                *charmap
                    .get(&ch)
                    .with_context(|| format!("Stats glyph {ch:?} is unmapped"))?,
            )?,
        };
    }
    Ok(())
}

fn stats_palette_colors(asset_root: &AssetRoot, path: &str) -> Result<Vec<[u8; 3]>> {
    let text = crate::read_runtime_asset_to_string(asset_root.runtime_assets().join(path))?;
    text.lines()
        .filter_map(|line| {
            let line = line.split(';').next().unwrap_or("").trim();
            line.starts_with("RGB").then_some(line)
        })
        .map(|line| {
            let values = parse_rgb_values(line)?;
            anyhow::ensure!(
                values.len() == 3,
                "Stats palette line must contain one RGB triplet"
            );
            rgb_triplet_to_u8(&values)
        })
        .collect()
}

fn stats_exp_pixels(
    runtime_shell: &BevyRuntimeShell,
    pokemon: &crate::core::models::Pokemon,
) -> Result<u8> {
    let catalog = runtime_shell.shell.runtime().growth_rates();
    let base = crate::core::systems::experience::calculate_experience(
        catalog,
        &pokemon.species.growth_rate,
        pokemon.level,
    )? as u32;
    let next = crate::core::systems::experience::calculate_experience(
        catalog,
        &pokemon.species.growth_rate,
        pokemon.level.wrapping_add(1),
    )? as u32;
    // CalcExpBar reduces its 16-bit divisor and 32-bit product together
    // until the divisor fits the hardware's eight-bit Divide routine.
    let mut divisor = next.wrapping_sub(base) & 0xffff;
    let mut product = (next.wrapping_sub(pokemon.experience as u32) & 0xffffff) * 64;
    while divisor > 255 {
        divisor >>= 1;
        product >>= 1;
    }
    anyhow::ensure!(divisor != 0, "Stats EXP interval has a zero divisor");
    Ok(64u8.wrapping_sub((product / divisor) as u8))
}

fn source_stats_layout(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    pokemon: &crate::core::models::Pokemon,
    page: u8,
) -> Result<SourceStatsLayout> {
    anyhow::ensure!((1..=3).contains(&page), "Stats page {page} is invalid");
    let mut map = SourceStatsLayout::default();
    map.tiles[7 * 20..8 * 20].fill(0x62);
    if pokemon.is_egg {
        stats_write_text(&mut map, 8, 1, "EGG")?;
        stats_write_text(&mut map, 8, 3, "<ID>№.")?;
        stats_write_text(&mut map, 8, 5, "OT/")?;
        stats_write_text(&mut map, 11, 3, "?????")?;
        stats_write_text(&mut map, 11, 5, "?????")?;
        let lines: &[&str] = match pokemon.happiness {
            0..=5 => &["It's making sounds", "inside. It's going", "to hatch soon!"],
            6..=10 => &[
                "It moves around",
                "inside sometimes.",
                "It must be close",
                "to hatching.",
            ],
            11..=40 => &["Wonder what's", "inside? It needs", "more time, though."],
            _ => &["This EGG needs a", "lot more time to", "hatch."],
        };
        for (row, text) in lines.iter().enumerate() {
            stats_write_text(&mut map, 1, 9 + row * 2, text)?;
        }
    } else {
        stats_write_text(&mut map, 8, 0, &format!("№.{:03}", pokemon.species.int_id))?;
        stats_write_text(&mut map, 14, 0, &visible_print_level_text(pokemon.level))?;
        stats_write_text(&mut map, 8, 2, &pokemon.nickname)?;
        stats_write_text(
            &mut map,
            9,
            4,
            &format!("/{}", canonical_species_display_name(&pokemon.species.id)),
        )?;
        if let Some(gender) = visible_pc_pokemon_info(pokemon).gender {
            stats_write_text(&mut map, 18, 0, &gender.to_string())?;
        }
        if visible_pokemon_is_shiny(pokemon) {
            map.tiles[19] = 0x3f;
            map.text[19] = '⁂';
        }
        map.tiles[6 * 20 + 12] = 0x71;
        map.tiles[6 * 20 + 19] = 0xed;
        for index in 0..3 {
            let tile = if index + 1 == usize::from(page) {
                0x3a
            } else {
                0x36
            };
            let origin = 5 * 20 + 13 + index * 2;
            map.tiles[origin] = tile;
            map.tiles[origin + 1] = tile + 1;
            map.tiles[origin + 20] = tile + 2;
            map.tiles[origin + 21] = tile + 3;
        }
        match page {
            1 => {
                map.tiles[9 * 20] = 0x60;
                map.tiles[9 * 20 + 1] = 0x61;
                map.text[9 * 20] = 'H';
                map.text[9 * 20 + 1] = 'P';
                let pixels = usize::from(battle_hud_hp_pixels(pokemon.hp, pokemon.max_hp));
                for index in 0..6 {
                    map.tiles[9 * 20 + 2 + index] =
                        0x62 + pixels.saturating_sub(index * 8).min(8) as u8;
                }
                map.tiles[9 * 20 + 8] = 0x41;
                stats_write_text(
                    &mut map,
                    1,
                    10,
                    &format!("{:>3}/{:>3}", pokemon.hp, pokemon.max_hp),
                )?;
                stats_write_text(&mut map, 0, 12, "STATUS/")?;
                stats_write_text(&mut map, 0, 14, "TYPE/")?;
                if pokemon.pokerus & 15 != 0 {
                    stats_write_text(&mut map, 1, 13, "POKéRUS")?;
                } else {
                    if pokemon.pokerus & 0xf0 != 0 {
                        map.tiles[8 * 20 + 8] = 0xe8;
                    }
                    let status = if pokemon.hp == 0 {
                        "FNT"
                    } else if pokemon.status.is_none() {
                        "OK"
                    } else {
                        battle_status_token(pokemon.status.as_deref())
                            .context("Stats status is invalid")?
                    };
                    stats_write_text(&mut map, 6, 13, status)?;
                }
                stats_write_text(&mut map, 1, 15, source_type_display_name(&pokemon.species.type1)?)?;
                if pokemon.species.type2 != pokemon.species.type1 {
                    stats_write_text(&mut map, 1, 16, source_type_display_name(&pokemon.species.type2)?)?;
                }
                for y in 8..18 {
                    map.tiles[y * 20 + 9] = 0x31;
                }
                stats_write_text(&mut map, 10, 9, "EXP POINTS")?;
                stats_write_text(
                    &mut map,
                    13,
                    10,
                    &format!("{:>7}", pokemon.experience & 0xffffff),
                )?;
                stats_write_text(&mut map, 10, 12, "LEVEL UP")?;
                let next_level = pokemon.level.saturating_add(1).min(100);
                let next_exp = crate::core::systems::experience::calculate_experience(
                    runtime_shell.shell.runtime().growth_rates(),
                    &pokemon.species.growth_rate,
                    next_level,
                )?;
                let remaining = if pokemon.level == 100 {
                    0
                } else {
                    (next_exp - pokemon.experience) & 0xffffff
                };
                stats_write_text(&mut map, 13, 13, &format!("{:>7}", remaining))?;
                stats_write_text(&mut map, 14, 14, "TO")?;
                stats_write_text(&mut map, 17, 14, &visible_print_level_text(next_level))?;
                let pixels = usize::from(stats_exp_pixels(runtime_shell, pokemon)?);
                map.tiles[16 * 20 + 10] = 0x40;
                map.tiles[16 * 20 + 19] = 0x41;
                for index in 0..8 {
                    let fill = pixels.saturating_sub(index * 8).min(8);
                    map.tiles[16 * 20 + 18 - index] = match fill {
                        0 => 0x62,
                        8 => 0x6a,
                        n => 0x54 + n as u8,
                    };
                }
            }
            2 => {
                stats_write_text(&mut map, 0, 8, "ITEM")?;
                let held = if let Some(id) = pokemon.item.as_deref() {
                    snapshot
                        .items
                        .iter()
                        .find(|item| item.item_id == id)
                        .with_context(|| format!("Stats item {id} is missing"))?
                        .name
                        .replace('_', " ")
                } else {
                    "---".into()
                };
                stats_write_text(&mut map, 8, 8, &held)?;
                stats_write_text(&mut map, 0, 10, "MOVE")?;
                for index in 0..4 {
                    if let Some(learned) = pokemon.moves.get(index) {
                        let data = snapshot
                            .moves
                            .iter()
                            .find(|entry| entry.move_id == learned.name)
                            .context("Stats move metadata is missing")?;
                        stats_write_text(
                            &mut map,
                            8,
                            10 + index * 2,
                            &data.name.replace('_', " "),
                        )?;
                        map.tiles[(11 + index * 2) * 20 + 12] = 0x3e;
                        map.tiles[(11 + index * 2) * 20 + 13] = 0x3e;
                        map.text[(11 + index * 2) * 20 + 12] = 'P';
                        map.text[(11 + index * 2) * 20 + 13] = 'P';
                        stats_write_text(
                            &mut map,
                            15,
                            11 + index * 2,
                            &format!(
                                "{:>2}/{:>2}",
                                learned.current_pp & 0x3f,
                                crate::core::models::max_move_pp(data.pp, learned.pp_ups)
                            ),
                        )?;
                    } else {
                        stats_write_text(&mut map, 8, 10 + index * 2, "-")?;
                        stats_write_text(&mut map, 12, 11 + index * 2, "--")?;
                    }
                }
            }
            3 => {
                for y in 8..18 {
                    map.tiles[y * 20 + 10] = 0x31;
                }
                stats_write_text(&mut map, 0, 9, "<ID>№.")?;
                stats_write_text(
                    &mut map,
                    2,
                    10,
                    &format!("{:05}", pokemon.original_trainer_id),
                )?;
                stats_write_text(&mut map, 0, 12, "OT/")?;
                stats_write_text(&mut map, 2, 13, &pokemon.original_trainer_name)?;
                if let Some(caught) = &pokemon.caught_data {
                    let caught_byte = caught.location | (caught.original_trainer_gender << 7);
                    if caught_byte != 0 && caught_byte != 0x7f {
                        stats_write_text(
                            &mut map,
                            9,
                            13,
                            if caught.original_trainer_gender == 0 {
                                "♂"
                            } else {
                                "♀"
                            },
                        )?;
                    }
                }
                for (index, (label, value)) in [
                    ("ATTACK", pokemon.attack),
                    ("DEFENSE", pokemon.defense),
                    ("SPCL.ATK", pokemon.special_attack),
                    ("SPCL.DEF", pokemon.special_defense),
                    ("SPEED", pokemon.speed),
                ]
                .into_iter()
                .enumerate()
                {
                    stats_write_text(&mut map, 11, 8 + index * 2, label)?;
                    stats_write_text(&mut map, 17, 9 + index * 2, &format!("{value:>3}"))?;
                }
            }
            _ => unreachable!(),
        }
    }
    Ok(map)
}

fn load_source_stats_frame(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    pokemon: &crate::core::models::Pokemon,
    page: u8,
    images: &mut Assets<Image>,
) -> Result<SpriteFrame> {
    let map = source_stats_layout(snapshot, runtime_shell, pokemon, page)?.tiles;
    let root = runtime_shell.asset_root.runtime_assets();
    let font = crate::open_runtime_image(root.join("gfx/font/font.png"))?.to_rgba8();
    let extra = crate::open_runtime_image(root.join("gfx/font/font_battle_extra.png"))?.to_rgba8();
    let stats = crate::open_runtime_image(root.join("gfx/stats/stats_tiles.png"))?.to_rgba8();
    let hp_end =
        crate::open_runtime_image(root.join("gfx/battle/enemy_hp_bar_border.png"))?.to_rgba8();
    let border =
        crate::open_runtime_image(root.join("gfx/battle/hp_exp_bar_border.png"))?.to_rgba8();
    let exp = crate::open_runtime_image(root.join("gfx/battle/expbar.png"))?.to_rgba8();
    let page_colors = stats_palette_colors(&runtime_shell.asset_root, "gfx/stats/stats.pal")?;
    let pages = parse_palette_file(
        &crate::read_runtime_asset_to_string(root.join("gfx/stats/pages.pal"))?,
        None,
    )?;
    let hp_colors = stats_palette_colors(&runtime_shell.asset_root, "gfx/battle/hp_bar.pal")?;
    let exp_colors = stats_palette_colors(&runtime_shell.asset_root, "gfx/battle/exp_bar.pal")?;
    anyhow::ensure!(
        page_colors.len() == 3 && pages.len() == 3 && hp_colors.len() == 6 && exp_colors.len() == 2,
        "Stats palettes have invalid lengths"
    );
    let white = [255, 255, 255];
    let black = [0, 0, 0];
    let bg = if pokemon.is_egg {
        white
    } else {
        page_colors[usize::from(page - 1)]
    };
    let zone = usize::from(2 - visible_hp_zone(battle_hud_hp_pixels(pokemon.hp, pokemon.max_hp)));
    let hp_palette = [bg, hp_colors[zone * 2], hp_colors[zone * 2 + 1], black];
    let exp_palette = [bg, exp_colors[0], exp_colors[1], black];
    let species = normalize_pokemon_asset_id(if pokemon.is_egg {
        "EGG"
    } else {
        &pokemon.species.id
    });
    let top_palette = load_pokemon_palette(
        &runtime_shell.asset_root,
        &species,
        PokemonSpriteSide::Front,
        visible_pokemon_is_shiny(pokemon),
    )?;
    let mut pixels = vec![255; 160 * 144 * 4];
    for (index, tile) in map.into_iter().enumerate() {
        let x = index % 20;
        let y = index / 20;
        let palette = if (5..7).contains(&y) && (13..19).contains(&x) && !pokemon.is_egg {
            &pages[(x - 13) / 2]
        } else if y < 8 {
            &top_palette
        } else if y == 16 && x >= 10 {
            &exp_palette
        } else {
            &hp_palette
        };
        if tile == 0x7f {
            pokegear_fill_rect(&mut pixels, 160, x * 8, y * 8, 8, 8, palette[0]);
            continue;
        }
        let (source, tile_index) = match tile {
            0x31..=0x41 => (&stats, usize::from(tile - 0x31)),
            0x55..=0x5c => (&exp, usize::from(tile - 0x55)),
            0x6c..=0x6f => (&hp_end, usize::from(tile - 0x6c)),
            0x76..=0x77 => (&border, usize::from(tile - 0x76 + 3)),
            0x78 => (&border, 0),
            0x60..=0x75 => (&extra, usize::from(tile - 0x60)),
            0x80..=0xff => (&font, usize::from(tile - 0x80)),
            _ => anyhow::bail!("Stats tile ${tile:02x} has no source art"),
        };
        pokegear_blit_paletted_tile(source, tile_index, palette, x * 8, y * 8, 160, &mut pixels)?;
    }
    let mut image = Image::new(
        Extent3d {
            width: 160,
            height: 144,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    Ok(SpriteFrame {
        handle: images.add(image),
        size: Vec2::new(160.0, 144.0),
    })
}

fn spawn_source_stats_screen(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    pokemon: &crate::core::models::Pokemon,
    page: u8,
    rendered_art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
    z: f32,
) -> Result<()> {
    let frame = load_source_stats_frame(snapshot, runtime_shell, pokemon, page, images)?;
    commit_presented_fullscreen_frame(
        commands,
        rendered_art,
        &frame,
        PresentedFullscreenFrameSource::Transient,
        z,
        images,
    )?;
    let picture = load_pc_pokemon_picture(
        &runtime_shell.asset_root,
        Some(&visible_pc_pokemon_info(pokemon)),
        true,
        images,
    )?;
    let (x, y) = battle_hud_tile_origin(3.0, 3.0);
    commands.spawn((
        SpriteBundle {
            texture: picture.handle,
            sprite: Sprite {
                custom_size: Some(Vec2::splat(7.0 * TILE_SIZE)),
                flip_x: pokemon.is_egg || pokemon.species.id != "UNOWN",
                ..default()
            },
            transform: Transform::from_xyz(x, y, z + 0.2),
            ..default()
        },
        FieldCommandMarker,
    ));
    Ok(())
}
fn visible_print_level_text(level: u8) -> String {
    // home/pokemon.asm::PrintLevel: three digits overwrite the LV tile.
    if level >= 100 {
        format!("{level}")
    } else {
        format!("\u{e10a}{level:<2}")
    }
}

fn visible_stats_screen_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    pokemon: &crate::core::models::Pokemon,
    page: u8,
) -> Result<Vec<String>> {
    let layout = source_stats_layout(snapshot, runtime_shell, pokemon, page)?;
    let mut entries = Vec::new();
    for row in layout.text.chunks_exact(20) {
        let text: String = row.iter().collect();
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let text = [
            ("\u{e10a}", "LV"),
            ("\u{e10b}", "ID"),
            ("\u{e105}", "PK"),
            ("\u{e106}", "MN"),
            ("\u{e120}", "'d"),
            ("\u{e121}", "'l"),
            ("\u{e122}", "'m"),
            ("\u{e123}", "'r"),
            ("\u{e124}", "'s"),
            ("\u{e125}", "'t"),
            ("\u{e126}", "'v"),
        ]
        .into_iter()
        .fold(text.to_owned(), |text, (glyph, readable)| {
            text.replace(glyph, readable)
        });
        entries.push(text);
    }
    Ok(entries)
}
