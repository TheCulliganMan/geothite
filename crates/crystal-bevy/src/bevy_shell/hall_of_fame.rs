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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct VisibleHallOfFame {
    sequence: crystal_runtime::hall_of_fame::HallOfFameSequence,
    team_slots: Vec<usize>,
    animation: Option<VisibleFrontpicAnimation>,
    animation_stage: u8,
    animation_wait: u8,
    rating_pages: VecDeque<String>,
    rating_text: String,
    rating_visible: usize,
    rating_delay: u8,
    rating_sound: String,
    rating_sound_started: bool,
    rating_acknowledged: bool,
}

fn begin_visible_hall_of_fame(shell: &mut BevyRuntimeShell) -> Result<()> {
    let team_slots = shell.shell.session().state().storage.party.pokemon.iter().enumerate()
        .filter_map(|(slot, mon)| mon.as_ref().filter(|mon| !mon.is_egg).map(|_| slot))
        .collect::<Vec<_>>();
    let sequence = crystal_runtime::hall_of_fame::HallOfFameSequence::new(team_slots.len())
        .context("Hall of Fame party exceeds six members")?;
    stop_visible_music(shell, "hall_of_fame:opening")?;
    shell.credits_screen.as_mut().context("Hall of Fame requires presentation owner")?
        .hall_of_fame = Some(VisibleHallOfFame {
            sequence, team_slots, animation: None, animation_stage: 0, animation_wait: 0,
            rating_pages: VecDeque::new(), rating_text: String::new(), rating_visible: 0,
            rating_delay: 0, rating_sound: String::new(), rating_sound_started: false,
            rating_acknowledged: false,
        });
    set_shell_action_status(shell, "HALL OF FAME");
    mark_runtime_snapshot_dirty(shell);
    Ok(())
}

fn queue_visible_hall_of_fame_music(shell: &mut BevyRuntimeShell) -> Result<()> {
    let id = "MUSIC_HALL_OF_FAME";
    let playback = shell.shell.runtime().audio().require_playback_entry(AudioKind::Music, id)?;
    enqueue_bevy_audio_command(&mut shell.pending_audio, BevyAudioCommand {
        cry_parameters: None, audio_id: id.into(), kind: ModpackAudioKind::Music,
        mode: playback.mode,
        looped: matches!(playback.loop_policy, crate::assets::ModpackAudioLoopPolicy::Loop),
    });
    shell.pending_music_stop = true;
    shell.active_music = Some(id.into());
    shell.faded_music = None;
    Ok(())
}

fn prepare_visible_hall_of_fame_rating(shell: &BevyRuntimeShell, ceremony: &mut VisibleHallOfFame) -> Result<()> {
    let state = shell.shell.session().state();
    let seen = state.pokedex.seen_count();
    let caught = state.pokedex.caught_count();
    let rating = shell.runtime.data().oak_ratings.iter()
        .find(|rating| caught <= rating.caught_count_limit)
        .context("Hall of Fame has no matching Oak rating")?;
    let buffers = BTreeMap::from([("wStringBuffer3".into(), seen.to_string()),
        ("wStringBuffer4".into(), caught.to_string())]);
    // ProfOaksPCRating calls Rate directly, omitting the PC boot message.
    for label in ["_OakPCText3".to_string(), format!("_{}", rating.text_label)] {
        for boundary in visible_exported_special_text_boundaries_with_named_buffers(
            shell, "ProfOaksPcBoot", &label, &buffers)? {
            for page in boundary.details.iter().flat_map(|text| visible_field_notice_pages(text)) {
                ceremony.rating_pages.push_back(page);
            }
        }
    }
    ceremony.rating_text = ceremony.rating_pages.pop_front().context("Oak rating has no pages")?;
    ceremony.rating_sound = rating.fanfare.clone();
    Ok(())
}

fn tick_visible_hall_of_fame(shell: &mut BevyRuntimeShell) -> Result<()> {
    use crystal_runtime::hall_of_fame::HallOfFamePhase as Phase;
    let mut ceremony = shell.credits_screen.as_mut().and_then(|credits| credits.hall_of_fame.take())
        .context("Hall of Fame sequence missing")?;
    let phase = ceremony.sequence.phase();
    let outcome = (|| -> Result<()> {
        match phase {
            Phase::PokemonAnimation { member } => {
                // ANIM_MON_HOF: cry without waiting, main program, 18-frame
                // pause, idle program. The inner byte interpreter is shared.
                if ceremony.animation_wait > 0 {
                    ceremony.animation_wait -= 1;
                    if ceremony.animation_wait == 0 {
                        let mon = shell.shell.session().state().storage.party.pokemon[ceremony.team_slots[member]]
                            .as_ref().context("Hall of Fame member missing")?;
                        ceremony.animation = Some(VisibleFrontpicAnimation {
                            species_id: format!("{}_IDLE", pokemon_asset_id_for_dvs(&mon.species.id, mon.dvs).to_ascii_uppercase()),
                            speed: 0, pointer: 0, repeat: 0, wait: 0, frame: 0,
                        });
                    }
                } else if let Some(animation) = ceremony.animation.as_mut() {
                    let snapshot = cached_runtime_snapshot(shell)?;
                    let program = snapshot.presentation.pokemon_frontpic_anim.get(&animation.species_id)
                        .with_context(|| format!("Hall of Fame missing animation {}", animation.species_id))?;
                    if step_visible_frontpic_animation(animation, program)? {
                        ceremony.animation = None;
                        if ceremony.animation_stage == 0 {
                            ceremony.animation_stage = 1;
                            ceremony.animation_wait = 18;
                        } else { ceremony.sequence.complete_pokemon_animation(member); }
                    }
                }
            }
            Phase::OakRating => {
                let options = &shell.shell.session().state().options;
                let length = ceremony.rating_text.chars().count();
                if options.no_text_scroll { ceremony.rating_visible = length; }
                else if ceremony.rating_delay > 0 { ceremony.rating_delay -= 1; }
                else if ceremony.rating_visible < length {
                    ceremony.rating_visible += 1;
                    ceremony.rating_delay = visible_text_frames_per_char(options.text_speed).saturating_sub(1);
                }
                if ceremony.rating_visible == length && ceremony.rating_pages.is_empty() {
                    if !ceremony.rating_sound_started {
                        stop_visible_music(shell, "hall_of_fame:rating")?;
                        queue_visible_shell_sound_effect(shell, &ceremony.rating_sound)?;
                        ceremony.rating_sound_started = true;
                    }
                    let busy = shell.transient_audio_playing || shell.pending_audio.iter()
                        .any(|command| command.kind != ModpackAudioKind::Music);
                    if ceremony.rating_acknowledged && !busy { ceremony.sequence.complete_rating(); }
                }
            }
            _ => { ceremony.sequence.tick(); }
        }
        if ceremony.sequence.phase() != phase {
            match ceremony.sequence.phase() {
                Phase::PokemonBack { member: 0 } => queue_visible_hall_of_fame_music(shell)?,
                Phase::PokemonAnimation { member } => {
                    let mon = shell.shell.session().state().storage.party.pokemon[ceremony.team_slots[member]]
                        .as_ref().context("Hall of Fame member missing")?;
                    let species = mon.species.id.clone();
                    ceremony.animation = Some(VisibleFrontpicAnimation {
                        species_id: pokemon_asset_id_for_dvs(&species, mon.dvs).to_ascii_uppercase(),
                        speed: 0, pointer: 0, repeat: 0, wait: 0, frame: 0,
                    });
                    ceremony.animation_stage = 0;
                    queue_visible_pokemon_cry(shell, &species, "hall_of_fame")?;
                }
                Phase::OakRating => prepare_visible_hall_of_fame_rating(shell, &mut ceremony)?,
                _ => {}
            }
        }
        Ok(())
    })();
    let complete = ceremony.sequence.phase() == Phase::Complete;
    if !complete || outcome.is_err() {
        shell.credits_screen.as_mut().context("Hall of Fame owner disappeared")?.hall_of_fame = Some(ceremony);
    }
    mark_runtime_presentation_dirty(shell);
    outcome
}

fn acknowledge_visible_hall_of_fame(shell: &mut BevyRuntimeShell) -> Result<()> {
    use crystal_runtime::hall_of_fame::HallOfFamePhase;
    let ceremony = shell.credits_screen.as_mut().and_then(|credits| credits.hall_of_fame.as_mut())
        .context("Hall of Fame input has no owner")?;
    if ceremony.sequence.phase() != HallOfFamePhase::OakRating { return Ok(()); }
    if ceremony.rating_visible < ceremony.rating_text.chars().count() { return Ok(()); }
    if let Some(next) = ceremony.rating_pages.pop_front() {
        ceremony.rating_text = next;
        ceremony.rating_visible = 0;
        ceremony.rating_delay = 0;
    } else { ceremony.rating_acknowledged = true; }
    mark_runtime_presentation_dirty(shell);
    Ok(())
}

fn render_visible_hall_of_fame_screen(
    shell: &BevyRuntimeShell, ceremony: &VisibleHallOfFame,
    art: &mut RenderedTilesetArt, images: &mut Assets<Image>,
) -> Result<SpriteFrame> {
    use crystal_runtime::hall_of_fame::HallOfFamePhase as Phase;
    let phase = ceremony.sequence.phase();
    let state = shell.shell.session().state();
    if let Phase::PokemonAnimation { member } | Phase::PokemonHold { member } = phase {
        let mon = state.storage.party.pokemon[ceremony.team_slots[member]].as_ref()
            .context("Hall of Fame portrait member missing")?;
        return render_visible_hall_of_fame_member(mon,
            ceremony.animation.as_ref().map_or(0, |animation| animation.frame), art, &shell.asset_root, images);
    }
    let mut data = vec![255; 160 * 144 * 4];
    let (picture, x, y) = match phase {
        Phase::PokemonBack { member } | Phase::PokemonFront { member } => {
            let mon = state.storage.party.pokemon[ceremony.team_slots[member]].as_ref()
                .context("Hall of Fame entrance member missing")?;
            let back = matches!(phase, Phase::PokemonBack { .. });
            let asset_id = pokemon_asset_id_for_dvs(&mon.species.id, mon.dvs);
            let frame = pokemon_frame_for_art(art, &shell.asset_root, &asset_id,
                if back { PokemonSpriteSide::Back } else { PokemonSpriteSide::Front },
                visible_pokemon_is_shiny(mon), images).context("Hall of Fame entrance art missing")?;
            let frame = if back { frame } else { battle_padded_frontpic(art, images, &frame)? };
            (Some(frame), 48, if back {48} else {40})
        }
        Phase::PlayerBack | Phase::PlayerFront | Phase::OakRating | Phase::ClosingFade => {
            let female = state.player_gender == PLAYER_GENDER_FEMALE;
            let back = phase == Phase::PlayerBack;
            let id = match (female, back) {
                (true, true) => "battle-player:kris_back", (false, true) => "battle-player:chris_back",
                (true, false) => "player:kris", (false, false) => "player:chris",
            };
            let key = IntroArtKey { asset_id: id.into() };
            if !art.intro_cache.contains_key(&key) {
                let frame = load_oak_intro_frame(&shell.asset_root, id, images)?;
                art.intro_cache.insert(key.clone(), frame);
            }
            (art.intro_cache.get(&key).cloned(), if back {48} else {96}, if back {48} else {40})
        }
        _ => (None, 0, 0),
    };
    if let Some(frame) = picture {
        let source = images.get(&frame.handle).context("Hall of Fame entrance texture missing")?;
        let (scroll_x, scroll_y) = ceremony.sequence.scroll();
        let width = source.texture_descriptor.size.width as usize;
        for row in 0..source.texture_descriptor.size.height as usize {
            for col in 0..width {
                let dx = ((x + col) as u8).wrapping_sub(scroll_x) as usize;
                let dy = ((y + row) as u8).wrapping_sub(scroll_y) as usize;
                let offset = (row * width + col) * 4;
                if dx < 160 && dy < 144 && source.data[offset + 3] != 0 {
                    data[(dy * 160 + dx) * 4..(dy * 160 + dx) * 4 + 4]
                        .copy_from_slice(&source.data[offset..offset + 4]);
                }
            }
        }
    }
    if matches!(phase, Phase::OakRating | Phase::ClosingFade) {
        let border = crate::open_runtime_image(shell.asset_root.runtime_assets().join("gfx/frames/1.png"))?.to_rgba8();
        draw_time_set_window(&border, 0, 2, 11, 10, &mut data)?;
        draw_time_set_window(&border, 0, 12, 20, 6, &mut data)?;
        let lines = [
            (2, 4, state.player_name.clone()), (1, 6, format!("<ID>№/{:05}", state.player_id)),
            (1, 8, "PLAY TIME".into()), (3, 9, format!("{:3}:{:02}", state.time.game_time_hours, state.time.game_time_minutes)),
        ];
        for (x, y, line) in lines {
            blit_hall_of_fame_text(art, &shell.asset_root, images, &line, x * 8, y * 8, &mut data)?;
        }
        for (row, line) in ceremony.rating_text.chars().take(ceremony.rating_visible).collect::<String>().lines().enumerate() {
            blit_hall_of_fame_text(art, &shell.asset_root, images, line, 8, (14 + row * 2) * 8, &mut data)?;
        }
    }
    if phase == Phase::ClosingFade {
        // RotateThreePalettesRight advances three whiteward palette steps,
        // eight frames each, followed by an eight-frame white hold.
        let step = (ceremony.sequence.elapsed_frames() / 8 + 1).min(3) as u16;
        for pixel in data.chunks_exact_mut(4) {
            for channel in &mut pixel[..3] {
                *channel = (u16::from(*channel) + (255 - u16::from(*channel)) * step / 3) as u8;
            }
        }
    }
    let mut image = Image::new(Extent3d {width:160, height:144, depth_or_array_layers:1},
        TextureDimension::D2, data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    image.sampler = ImageSampler::nearest();
    Ok(SpriteFrame {handle:images.add(image), size:Vec2::new(160.0,144.0)})
}

fn blit_hall_of_fame_text(art: &mut RenderedTilesetArt, root: &AssetRoot,
    images: &mut Assets<Image>, text: &str, x: usize, y: usize, data: &mut [u8]) -> Result<()> {
    let frames = bitmap_text_frames(art, root, images, text);
    if let Some(error) = art.font_error.as_deref() { anyhow::bail!("{error}"); }
    for (index, glyph) in frames.iter().enumerate() {
        blit_sprite_frame_image(images.get(&glyph.handle).context("Hall of Fame glyph missing")?,
            x + index * 8, y, 160, 144, data);
    }
    Ok(())
}
