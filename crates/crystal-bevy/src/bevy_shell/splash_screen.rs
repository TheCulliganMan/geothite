// The copyright/studio host consumes the authored SplashScreen subprogram.
// Display effects are explicit; unknown operations remain fatal.
#[derive(Debug, Clone)]
struct VisibleSplashScreen {
    interpreter: RuntimePresentationSubprogramInterpreter,
    parameters: serde_json::Value,
    wait: u16,
    copyright: bool,
    logo_visible: bool,
    studio_visible: bool,
    presents_visible: bool,
    scene: u8,
    timer: u8,
    sprite_scene: u8,
    jump_height: u8,
    sine_offset: u8,
    sprite_timer: u8,
    sprite_y_offset: i16,
    sprite_position: [i16; 2],
    animation_step: i16,
    animation_timer: u8,
    oam_set: Option<String>,
    cancelled: bool,
    finished: bool,
    sampled_input: u8,
    pending_input: u8,
    frame: u64,
}

#[cfg(test)]
mod splash_source_tests {
    use super::*;

    #[test]
    #[ignore = "requires separately exported canonical splash and sprite bundle"]
    fn authored_splash_runs_to_completion_and_cancellation_after_exact_cleanup() {
        let subprogram = serde_json::from_slice(
            &std::fs::read(
                std::env::var_os("CRYSTAL_SPLASH_REVIEW_JSON").expect("CRYSTAL_SPLASH_REVIEW_JSON"),
            )
            .unwrap(),
        )
        .unwrap();
        let mut program = RuntimePresentationProgram {
            schema_version: 1,
            ..Default::default()
        };
        program.subprograms.push(subprogram);
        let bundle = load_intro_sprite_anim_bundle(
            &std::fs::read_to_string(
                std::env::var_os("CRYSTAL_SPLASH_REVIEW_BUNDLE")
                    .expect("CRYSTAL_SPLASH_REVIEW_BUNDLE"),
            )
            .unwrap(),
        )
        .unwrap();
        let mut natural = VisibleSplashScreen::from_program(&program).unwrap();
        let asset_root = AssetRoot::new(
            std::env::var_os("CRYSTAL_EXPORT_WORKSPACE").expect("CRYSTAL_EXPORT_WORKSPACE"),
        );
        let mut captured = std::collections::BTreeSet::new();
        let mut sounds = Vec::new();
        for _ in 0..1024 {
            // A one-frame button press during copyright must not cancel the
            // later logo animation, whose input loop has not begun yet.
            if natural.frame == 20 {
                natural.pending_input = 1;
            }
            let input = natural.consume_pending_input();
            sounds.extend(natural.tick(&program, &bundle, input).unwrap());
            let rgba = compose_splash_screen_pixels(&asset_root, &natural, &bundle).unwrap();
            assert_eq!(rgba.len(), 160 * 144 * 4);
            assert!(rgba.chunks_exact(4).all(|pixel| pixel[3] == 255));
            let stage = if natural.copyright {
                Some("copyright")
            } else if natural.presents_visible {
                Some("studio-presents")
            } else if natural.sprite_scene == 2 {
                Some("ditto")
            } else {
                None
            };
            if let Some(stage) = stage {
                assert!(
                    rgba.chunks_exact(4).any(|pixel| pixel != &rgba[..4]),
                    "{stage} must draw authored graphics"
                );
                if captured.insert(stage) {
                    if let Some(directory) = std::env::var_os("CRYSTAL_SPLASH_REVIEW_IMAGES") {
                        image::RgbaImage::from_raw(160, 144, rgba)
                            .unwrap()
                            .save(
                                std::path::PathBuf::from(directory)
                                    .join(format!("splash-{stage}.png")),
                            )
                            .unwrap();
                    }
                }
            }
            if natural.finished {
                break;
            }
        }
        assert!(natural.finished && !natural.cancelled);
        assert_eq!(captured.len(), 3);
        assert_eq!(
            sounds
                .iter()
                .filter(|id| id.as_str() == "SFX_DITTO_BOUNCE")
                .count(),
            2
        );
        for id in [
            "SFX_DITTO_POP_UP",
            "SFX_DITTO_TRANSFORM",
            natural.parameters["studio_audio"].as_str().unwrap(),
        ] {
            assert_eq!(
                sounds.iter().filter(|value| value.as_str() == id).count(),
                1
            );
        }
        let mut cancelled = VisibleSplashScreen::from_program(&program).unwrap();
        while !cancelled.logo_visible {
            cancelled.tick(&program, &bundle, 0).unwrap();
        }
        cancelled.tick(&program, &bundle, 1).unwrap();
        assert!(!cancelled.logo_visible && !cancelled.finished);
        let cleanup = cancelled.parameter("cleanup_frames").unwrap();
        for _ in 1..cleanup {
            cancelled.tick(&program, &bundle, 0).unwrap();
            assert!(!cancelled.finished);
        }
        cancelled.tick(&program, &bundle, 0).unwrap();
        assert!(cancelled.finished && cancelled.cancelled);
        assert!(cancelled.oam_set.is_none());
    }
}

fn tick_visible_splash_screen(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let (audio, finished, cancelled) = {
        let splash = runtime_shell
            .splash_screen
            .as_mut()
            .context("SplashScreen is not active")?;
        let bundle = runtime_shell
            .intro_sprite_bundle
            .as_ref()
            .context("SplashScreen requires packed sprite bundle")?;
        let pending_input = splash.consume_pending_input();
        let audio = splash.tick(
            runtime_shell.runtime.title_presentation_program(),
            bundle,
            pending_input,
        )?;
        (audio, splash.finished, splash.cancelled)
    };
    for id in audio {
        if id == "MUSIC_NONE" {
            stop_visible_silent_music(runtime_shell, &id, "splash:music:none")?;
        } else {
            queue_visible_sound_effect(
                runtime_shell.shell.runtime().audio(),
                &mut runtime_shell.pending_audio,
                &mut runtime_shell.last_audio_events,
                &id,
            )?;
        }
    }
    if finished {
        runtime_shell.splash_screen = None;
        if cancelled {
            runtime_shell.intro_screen = None;
            runtime_shell.intro_display_screen = None;
        }
        record_visible_runtime_action(
            runtime_shell,
            if cancelled {
                "splash:cancelled"
            } else {
                "splash:complete"
            },
        )?;
    }
    Ok(())
}

impl VisibleSplashScreen {
    fn from_program(program: &RuntimePresentationProgram) -> Result<Self> {
        let source = program
            .subprograms
            .iter()
            .find(|s| s.id == "splash_screen")
            .context("required SplashScreen subprogram is missing")?;
        let required = source.required_consumer.clone();
        for field in [
            "cleanup_frames",
            "ditto_wait_frames",
            "transform_frames",
            "background_palette_index",
        ] {
            required
                .get(field)
                .and_then(serde_json::Value::as_u64)
                .with_context(|| format!("SplashScreen requires authored {field}"))?;
        }
        anyhow::ensure!(
            source.sprite_operations.len() == 3,
            "SplashScreen has no exact sprite initialization"
        );
        let byte = |field: &str| -> Result<u8> {
            source
                .sprite_operations
                .iter()
                .find_map(|op| op.get(field))
                .and_then(serde_json::Value::as_u64)
                .and_then(|n| u8::try_from(n).ok())
                .with_context(|| format!("SplashScreen has no source byte {field}"))
        };
        Ok(Self {
            interpreter: RuntimePresentationSubprogramInterpreter::new(
                program,
                "splash_screen",
                "splash",
            )?,
            sprite_y_offset: i16::try_from(
                required
                    .get("initial_hidden_y_offset")
                    .and_then(serde_json::Value::as_u64)
                    .context("SplashScreen lacks authored hidden Y offset")?,
            )?,
            parameters: required,
            wait: 0,
            copyright: false,
            logo_visible: false,
            studio_visible: false,
            presents_visible: false,
            scene: 0,
            timer: 0,
            sprite_scene: 0,
            jump_height: byte("initial_jump_height")?,
            sine_offset: byte("initial_sine_offset")?,
            sprite_timer: 0,
            sprite_position: serde_json::from_value(
                source.sprite_operations[0]["pixel_position"].clone(),
            )?,
            animation_step: -1,
            animation_timer: 0,
            oam_set: None,
            cancelled: false,
            finished: false,
            sampled_input: 0,
            pending_input: 0,
            frame: 0,
        })
    }

    fn consume_pending_input(&mut self) -> u8 {
        std::mem::take(&mut self.pending_input)
    }

    fn parameter(&self, name: &str) -> Result<u16> {
        self.parameters
            .get(name)
            .and_then(serde_json::Value::as_u64)
            .and_then(|n| u16::try_from(n).ok())
            .with_context(|| format!("SplashScreen has invalid {name}"))
    }

    fn advance_scene(&mut self) -> Result<Option<String>> {
        if self.scene == 0 {
            return Ok(None);
        }
        let index = usize::from(self.scene - 1);
        let delay = self.parameters["scene_delays"][index]["frames"]
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .context("SplashScreen scene lacks authored delay")?;
        if self.timer < delay {
            self.timer += 1;
            return Ok(None);
        }
        self.timer = 0;
        match self.scene {
            1 => {
                self.studio_visible = true;
                self.scene = 2;
                Ok(Some(
                    self.parameters["studio_audio"]
                        .as_str()
                        .context("splash studio audio")?
                        .to_owned(),
                ))
            }
            2 => {
                self.presents_visible = true;
                self.scene = 3;
                Ok(None)
            }
            3 => {
                self.scene |= 0x80;
                Ok(None)
            }
            _ => anyhow::bail!("SplashScreen scene index is invalid"),
        }
    }

    fn advance_sprite(&mut self, bundle: &SpriteAnimRuntimeBundle) -> Result<Option<&'static str>> {
        let mut audio = None;
        match self.sprite_scene {
            0 => self.sprite_scene = 1,
            1 if self.jump_height != 0 => {
                let mut angle = self.sine_offset & 0x3f;
                if angle < 32 {
                    angle += 32;
                }
                self.sprite_y_offset = visible_intro_sine(angle, i16::from(self.jump_height));
                let previous = self.sine_offset;
                self.sine_offset = self.sine_offset.wrapping_sub(1);
                if previous & 0x1f == 0 {
                    self.jump_height = self
                        .jump_height
                        .wrapping_sub(u8::try_from(self.parameter("bounce_height_decrement")?)?);
                    audio = Some("SFX_DITTO_BOUNCE");
                }
            }
            1 => {
                self.sprite_scene = 2;
                self.sprite_timer = 0;
                audio = Some("SFX_DITTO_POP_UP");
            }
            2 => {
                if u16::from(self.sprite_timer) >= self.parameter("ditto_wait_frames")? {
                    self.sprite_scene = 3;
                    self.sprite_timer = 0;
                    audio = Some("SFX_DITTO_TRANSFORM");
                } else {
                    self.sprite_timer += 1;
                }
            }
            3 => {
                if u16::from(self.sprite_timer) == self.parameter("transform_frames")? {
                    self.sprite_scene = 4;
                    self.scene += 1;
                } else {
                    self.sprite_timer += 1;
                }
            }
            4 => {}
            _ => anyhow::bail!("SplashScreen sprite callback state is invalid"),
        }
        let steps = visible_intro_frameset_steps(
            bundle,
            self.parameters["sprite_frameset"]
                .as_str()
                .context("splash sprite frameset")?,
        )?;
        for _ in 0..steps.len() + 1 {
            if self.animation_timer != 0 {
                self.animation_timer -= 1;
                break;
            }
            self.animation_step += 1;
            let step = steps
                .get(usize::try_from(self.animation_step)?)
                .context("SplashScreen frameset exceeded source steps")?;
            match step.command.as_str() {
                "frame" => {
                    self.oam_set = step.oam_set.clone();
                    self.animation_timer = u8::try_from(step.duration)?.saturating_sub(1);
                    break;
                }
                "end" => {
                    self.animation_step -= 2;
                }
                "restart" => self.animation_step = -1,
                command => anyhow::bail!("SplashScreen has unsupported frameset command {command}"),
            }
        }
        Ok(audio)
    }

    fn tick(
        &mut self,
        program: &RuntimePresentationProgram,
        bundle: &SpriteAnimRuntimeBundle,
        input: u8,
    ) -> Result<Vec<String>> {
        self.frame += 1;
        let mut audio = Vec::new();
        if self.wait > 0 {
            self.wait -= 1;
            return Ok(audio);
        }
        if self.finished {
            return Ok(audio);
        }
        for _ in 0..64 {
            let operation = self
                .interpreter
                .step(program)?
                .context("SplashScreen ended without authored return")?;
            let number = |field: &str| -> Result<u16> {
                operation
                    .fields
                    .get(field)
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|n| u16::try_from(n).ok())
                    .with_context(|| format!("SplashScreen {} lacks {field}", operation.op))
            };
            match operation.op.as_str() {
                "stop_audio" => audio.push("MUSIC_NONE".into()),
                "initialize_splash_display"
                | "prepare_splash_palette"
                | "write_memory_byte"
                | "require_color_console" => {}
                "present_copyright" => self.copyright = true,
                "initialize_studio_splash" => {
                    self.copyright = false;
                    self.logo_visible = true;
                }
                "sample_input" => self.sampled_input = input,
                "input_chord_branch" => {
                    if u16::from(self.sampled_input) & number("mask")? != 0 {
                        self.interpreter.jump_to_label(program, ".pressed_button")?;
                    }
                }
                "input_bit_branch" => {
                    if self.scene & (1 << number("bit")?) != 0 {
                        self.interpreter.jump_to_label(program, ".finish")?;
                    }
                }
                "advance_studio_scene" => {
                    if let Some(id) = self.advance_scene()? {
                        audio.push(id.into());
                    }
                }
                "advance_sprite_animations" => {
                    if let Some(id) = self.advance_sprite(bundle)? {
                        audio.push(id.into());
                    }
                }
                "wait_frames" => {
                    self.wait = number("frames")?.saturating_sub(1);
                    return Ok(audio);
                }
                "jump" => self.interpreter.jump_to_label(
                    program,
                    operation.fields["target"]
                        .as_str()
                        .context("SplashScreen jump lacks target")?,
                )?,
                "finish_studio_splash" => {
                    self.logo_visible = false;
                    self.studio_visible = false;
                    self.presents_visible = false;
                    self.oam_set = None;
                    self.wait = self.parameter("cleanup_frames")?.saturating_sub(1);
                    return Ok(audio);
                }
                "return" => {
                    self.cancelled = operation.fields["carry"]
                        .as_bool()
                        .context("SplashScreen return lacks carry")?;
                    self.finished = true;
                    return Ok(audio);
                }
                op => anyhow::bail!("unsupported SplashScreen host operation {op}"),
            }
        }
        anyhow::bail!("SplashScreen exceeded its bounded operation budget")
    }
}

fn splash_screen_frame(
    runtime_shell: &BevyRuntimeShell,
    splash: &VisibleSplashScreen,
    images: &mut Assets<Image>,
) -> Result<SpriteFrame> {
    let bundle = runtime_shell
        .intro_sprite_bundle
        .as_ref()
        .context("SplashScreen sprite bundle is absent")?;
    let data = compose_splash_screen_pixels(&runtime_shell.asset_root, splash, bundle)?;
    let mut image = Image::new(
        Extent3d {
            width: 160,
            height: 144,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    Ok(SpriteFrame {
        handle: images.add(image),
        size: visible_intro_display_size(),
    })
}

fn compose_splash_screen_pixels(
    asset_root: &AssetRoot,
    splash: &VisibleSplashScreen,
    bundle: &SpriteAnimRuntimeBundle,
) -> Result<Vec<u8>> {
    let root = asset_root.runtime_assets();
    let predef = parse_palette_file(
        &crate::read_runtime_asset_to_string(&root.join("gfx/sgb/predef.pal"))?,
        None,
    )?;
    let background = predef
        .get(usize::from(splash.parameter("background_palette_index")?))
        .context("SplashScreen background palette is not packed")?;
    let mut data = background[0]
        .into_iter()
        .chain([255])
        .cycle()
        .take(160 * 144 * 4)
        .collect::<Vec<_>>();
    let draw_tile = |target: &mut [u8],
                     bytes: &[u8],
                     tile: usize,
                     bits: usize,
                     x: i16,
                     y: i16,
                     palette: &Palette,
                     transparent: bool,
                     flags: u8|
     -> Result<()> {
        let tile_bytes = bytes
            .get(tile * 8 * bits..(tile + 1) * 8 * bits)
            .context("SplashScreen graphic tile is missing")?;
        for row in 0..8usize {
            for column in 0..8usize {
                let sx = if flags & 0x20 != 0 {
                    7 - column
                } else {
                    column
                };
                let sy = if flags & 0x40 != 0 { 7 - row } else { row };
                let low = (tile_bytes[sy * bits] >> (7 - sx)) & 1;
                let level = if bits == 1 {
                    low * 3
                } else {
                    low | (((tile_bytes[sy * bits + 1] >> (7 - sx)) & 1) << 1)
                };
                if transparent && level == 0 {
                    continue;
                }
                let dx = x + column as i16;
                let dy = y + row as i16;
                if !(0..160).contains(&dx) || !(0..144).contains(&dy) {
                    continue;
                }
                let offset = (dy as usize * 160 + dx as usize) * 4;
                target[offset..offset + 3].copy_from_slice(&palette[usize::from(level)]);
                target[offset + 3] = 255;
            }
        }
        Ok(())
    };
    if splash.copyright {
        let bytes = crate::read_runtime_asset(&root.join("gfx/splash/copyright.2bpp"))?;
        let position: [i16; 2] =
            serde_json::from_value(splash.parameters["copyright_position"].clone())?;
        let rows = splash.parameters["copyright_rows"]
            .as_array()
            .context("SplashScreen copyright rows are missing")?;
        for (row, tiles) in rows.iter().enumerate() {
            for (column, tile) in tiles
                .as_array()
                .context("SplashScreen copyright row is invalid")?
                .iter()
                .enumerate()
            {
                let id = tile
                    .as_u64()
                    .context("SplashScreen copyright tile is invalid")?
                    as usize;
                let base = usize::from(splash.parameter("copyright_tile_base")?);
                draw_tile(
                    &mut data,
                    &bytes,
                    id.checked_sub(base)
                        .context("SplashScreen copyright tile precedes transfer")?,
                    2,
                    (position[0] + column as i16) * 8,
                    (position[1] + row as i16 * 2) * 8,
                    background,
                    false,
                    0,
                )?;
            }
        }
    }
    if splash.logo_visible {
        let resources = splash.parameters["lettering_resources"]
            .as_array()
            .context("splash lettering resources")?;
        anyhow::ensure!(
            resources.len() == 2,
            "splash lettering requires two resources"
        );
        let mut lettering = Vec::new();
        for resource in resources {
            lettering.extend(crate::read_runtime_asset(
                &root.join(resource.as_str().context("splash lettering path")?),
            )?);
        }
        for (visible, tiles_name, position_name) in [
            (splash.studio_visible, "studio_tiles", "studio_position"),
            (
                splash.presents_visible,
                "presents_tiles",
                "presents_position",
            ),
        ] {
            if !visible {
                continue;
            }
            let position: [i16; 2] =
                serde_json::from_value(splash.parameters[position_name].clone())?;
            for (column, tile) in splash.parameters[tiles_name]
                .as_array()
                .context("SplashScreen text tile data is missing")?
                .iter()
                .enumerate()
            {
                draw_tile(
                    &mut data,
                    &lettering,
                    tile.as_u64().context("SplashScreen text tile is invalid")? as usize,
                    1,
                    (position[0] + column as i16) * 8,
                    position[1] * 8,
                    background,
                    false,
                    0,
                )?;
            }
        }
        if let Some(name) = &splash.oam_set {
            let oam = bundle
                .oam_sets
                .get(name)
                .context("SplashScreen OAM set is missing")?;
            let ditto = crate::read_runtime_asset(&root.join("gfx/splash/ditto.2bpp"))?;
            let palettes = parse_palette_file(
                &crate::read_runtime_asset_to_string(&root.join("gfx/splash/ditto.pal"))?,
                None,
            )?;
            let mut palette = *palettes
                .first()
                .context("SplashScreen Ditto palette is missing")?;
            if splash.sprite_scene >= 3 {
                let fade = parse_palette_file(
                    &crate::read_runtime_asset_to_string(&root.join("gfx/splash/ditto_fade.pal"))?,
                    None,
                )?;
                let colors = fade.into_iter().flatten().collect::<Vec<_>>();
                palette[2] = *colors
                    .get(usize::from(splash.sprite_timer.saturating_sub(1) / 4))
                    .context("SplashScreen transformation palette step is missing")?;
            }
            for piece in &oam.pieces {
                let tile = (piece.tile + oam.tile_offset).rem_euclid(256) as usize;
                let x = (splash.sprite_position[0] + piece.x).rem_euclid(256) - 8;
                let y = (splash.sprite_position[1] + splash.sprite_y_offset + piece.y)
                    .rem_euclid(256)
                    - 16;
                draw_tile(
                    &mut data,
                    &ditto,
                    tile,
                    2,
                    x,
                    y,
                    &palette,
                    true,
                    piece.attributes,
                )?;
            }
        }
    }
    Ok(data)
}
