// These tests use the real pack, command compiler, object VM, tile raster and
// classic battle renderer. No separate approximation of an effect is drawn.

#[derive(Clone)]
struct SignatureOamSnapshot {
    event_index: usize,
    pieces: Vec<[u8; 4]>,
}

#[test]
fn northwoods_moves_render_native_frames_both_sides_and_cleanup() {
    use sha2::Digest as _;
    let selection =
        northwoods_animation_selection(std::env::var("NORTHWOODS_ANIMATION_MOVES").ok().as_deref())
            .expect("valid unique comma-separated extension move IDs");
    let review_only = std::env::var_os("NORTHWOODS_ANIMATION_REVIEW_ONLY").is_some();
    let output = std::env::var_os("NORTHWOODS_ANIMATION_RENDER_DIR").map(PathBuf::from);
    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        assert!(
            output.is_none(),
            "capture requires an explicit Northwoods move pack"
        );
        return;
    }
    let mut shell = route36_battle_shell_for_render_regression();
    shell.visible_move_audio_wait = None;
    shell.battle_fainted_hud = [false; 2];
    let base_snapshot = shell.shell.snapshot().expect("battle snapshot");
    if !base_snapshot
        .moves
        .iter()
        .any(|entry| entry.move_id == "MAGNETIC_RAIL")
    {
        assert!(
            output.is_none(),
            "capture pack does not include Northwoods signatures"
        );
        return; // Ordinary core packs intentionally do not contain this add-on.
    }
    if let Some(directory) = &output {
        if directory.exists() {
            assert!(
                std::fs::read_dir(directory)
                    .expect("inspect capture directory")
                    .next()
                    .is_none(),
                "capture directory contains retained renders; choose a fresh run directory"
            );
        } else {
            std::fs::create_dir_all(directory).expect("create retained capture root");
        }
    }
    let actors = selection
        .into_iter()
        .map(|id| {
            let level =
                if crystal_assets::minnesota_signature_animations::SIGNATURE_MOVES.contains(&id) {
                    60
                } else {
                    50
                };
            (
                id,
                northwoods_animation_actor(shell.runtime.data(), id, level),
                level,
            )
        })
        .collect::<Vec<_>>();
    let mut fingerprints = std::collections::BTreeSet::new();
    for (move_id, actor, actor_level) in actors {
        let (label, duration, sounds, cries, objects, effects) =
            visible_move_animation_definition(&base_snapshot, move_id, 0)
                .expect("signature must resolve through the production move table");
        let expected_label =
            if crystal_assets::minnesota_signature_animations::SIGNATURE_MOVES.contains(&move_id) {
                crystal_assets::minnesota_signature_animations::animation_label(move_id)
            } else {
                crystal_assets::northwoods_move_animations::animation_label(move_id)
            };
        assert_eq!(label, expected_label);
        assert!(cries.is_empty());
        assert!(
            effects.is_empty(),
            "signature must preserve background palettes"
        );
        assert!((45..=125).contains(&duration), "{move_id}: {duration}");
        for (_, sound) in &sounds {
            assert!(
                shell
                    .runtime
                    .data()
                    .audio
                    .iter()
                    .any(|asset| asset.id == sound.id),
                "{move_id} refers to missing sound {}",
                sound.id
            );
        }
        let mut player_normal = None;
        let mut player_oam = Vec::<Vec<SignatureOamSnapshot>>::new();
        for player_move in [true, false] {
            if review_only && !player_move {
                continue;
            }
            let mut normal_effect = None;
            for shiny in [false, true] {
                if review_only && shiny {
                    continue;
                }
                let mut snapshot = base_snapshot.clone();
                let ordinary_dvs = Dv::from_non_hp(9, 10, 10, 10);
                snapshot.party.slots[0].pokemon.dvs = ordinary_dvs;
                snapshot.battle.as_mut().expect("battle").enemy_pokemon.dvs = ordinary_dvs;
                let data = shell.runtime.data();
                let species = data.saved_species(&actor).expect("signature actor in pack");
                let dvs = if shiny {
                    Dv::from_non_hp(2, 10, 10, 10)
                } else {
                    ordinary_dvs
                };
                assert_eq!(visible_dvs_are_shiny(dvs), shiny);
                let actor_pokemon = crate::core::models::pokemon::create_pokemon_from_known_dvs(
                    &species,
                    actor_level,
                    dvs,
                    &data.learnsets,
                    &data.moves,
                    &data.growth_rates,
                )
                .expect("signature actor Pokemon");
                if player_move {
                    snapshot.party.slots[0].pokemon = actor_pokemon;
                } else {
                    snapshot.battle.as_mut().expect("battle").enemy_pokemon = actor_pokemon;
                }
                let mut animation = battle_anim_regression_timeline(objects.clone(), 0);
                animation.move_id = move_id.into();
                animation.animation_label = label.clone();
                animation.total_frames = duration;
                animation.player_move = player_move;
                animation.sound_events = sounds.clone();
                let mut art = RenderedTilesetArt::default();
                let bundle =
                    battle_anim_render_bundle(&mut art, &snapshot).expect("animation bundle");
                let mut playback =
                    new_visible_battle_objects(&bundle, &animation).expect("production OAM player");
                let mut images = Assets::<Image>::default();
                let mut rendered = Vec::new();
                let mut effect_fingerprint = sha2::Sha256::new();
                let mut unique_effect_frames = std::collections::BTreeSet::new();
                let mut encoded_frames = Vec::new();
                let mut contact_scores = Vec::new();
                let mut active_frames = Vec::new();
                let mut checked_object_frames = 0;
                let mut unique_frames = std::collections::BTreeSet::new();
                let variant = format!(
                    "{}-{}",
                    if player_move { "player" } else { "enemy" },
                    if shiny { "shiny" } else { "normal" }
                );
                let directory = output
                    .as_ref()
                    .map(|root| root.join(move_id.to_ascii_lowercase()).join(&variant));
                if let Some(directory) = &directory {
                    std::fs::create_dir_all(directory).expect("capture directory");
                }
                for frame in 0..duration {
                    animation.frame = frame;
                    advance_visible_battle_objects(&mut playback, &bundle, &animation)
                        .expect("production OAM playback");
                    let custom_oam = playback
                        .slots
                        .iter()
                        .flatten()
                        .filter_map(|live| {
                            match &animation.object_events[live.event_index].command {
                                VisibleMoveObjectCommand::Spawn { object_id, .. }
                                    if object_id.starts_with("BATTLE_ANIM_OBJ_MN_") =>
                                {
                                    Some(SignatureOamSnapshot {
                                        event_index: live.event_index,
                                        pieces: live.oam.entries.clone(),
                                    })
                                }
                                _ => None,
                            }
                        })
                        .collect::<Vec<_>>();
                    if !custom_oam.is_empty() {
                        checked_object_frames += 1;
                    }
                    if player_move && !shiny {
                        player_oam.push(custom_oam.clone());
                    }
                    if !player_move {
                        let reference = &player_oam[usize::from(frame)];
                        assert_eq!(custom_oam.len(), reference.len());
                        for (actual, expected) in custom_oam.iter().zip(reference) {
                            assert_eq!(actual.event_index, expected.event_index);
                            assert_eq!(actual.pieces.len(), expected.pieces.len());
                            let VisibleMoveObjectCommand::Spawn { object_id, y, .. } =
                                &animation.object_events[actual.event_index].command
                            else {
                                unreachable!()
                            };
                            let flags = bundle["objects"][object_id]["flags"]
                                .as_u64()
                                .expect("object flags")
                                as u8;
                            for (actual, expected) in actual.pieces.iter().zip(&expected.pieces) {
                                // Original HORN OAM basis: enemy coordinates mirror
                                // the source anchor, with flags controlling whether
                                // the authored picture itself turns upside down.
                                assert_eq!(
                                    actual[1],
                                    172_u8.wrapping_sub(expected[1]),
                                    "{move_id}: native enemy X"
                                );
                                let expected_y = if flags & 0x40 != 0 {
                                    136 - i32::from(expected[0])
                                } else {
                                    i32::from(expected[0]) + 144 - 2 * i32::from(*y)
                                };
                                assert_eq!(
                                    actual[0], expected_y as u8,
                                    "{move_id}: native enemy Y"
                                );
                                assert_eq!(
                                    actual[2], expected[2],
                                    "mirroring preserves tile identity"
                                );
                                assert_eq!(
                                    actual[3] ^ expected[3],
                                    flags & 0x60,
                                    "native OAM flips"
                                );
                            }
                        }
                    }
                    let pieces = playback
                        .slots
                        .iter()
                        .flatten()
                        .flat_map(|object| &object.oam.entries)
                        .collect::<Vec<_>>();
                    assert!(
                        pieces.len() <= 40,
                        "{move_id} frame{frame} exceeds hardware OAM count"
                    );
                    for y in 0_i32..144 {
                        let overlaps = pieces
                            .iter()
                            .filter(|piece| {
                                let top = i32::from(piece[0]) - 16;
                                (top..top + 8).contains(&y)
                            })
                            .count();
                        assert!(
                            overlaps <= 10,
                            "{move_id} frame{frame} scanline{y}: {overlaps} OAM pieces"
                        );
                    }
                    if frame + 1 == duration {
                        assert!(pieces.is_empty(), "{move_id} leaves residual OAM");
                        assert!(playback.slots.iter().all(Option::is_none));
                    }
                    shell.visible_move_animations.clear();
                    shell.visible_move_animations.push_back(animation.clone());
                    let mut world = World::new();
                    let mut queue = bevy::ecs::world::CommandQueue::default();
                    {
                        let mut commands = Commands::new(&mut queue, &world);
                        spawn_visible_move_animation_objects(
                            &mut commands,
                            &snapshot,
                            &shell,
                            &mut art,
                            &shell.asset_root,
                            &mut images,
                        )
                        .expect("production effect raster");
                    }
                    queue.apply(&mut world);
                    for sprite in world.query::<&Sprite>().iter(&world) {
                        let image = images.get(&sprite.image).expect("effect texture");
                        let mut visible_inks = std::collections::BTreeSet::new();
                        for pixel in image
                            .data
                            .as_ref()
                            .expect("native effect pixels")
                            .chunks_exact(4)
                        {
                            assert!(
                                pixel[3] == 0 || pixel[3] == 255,
                                "source effect inks cannot alpha-fade"
                            );
                            if pixel[3] != 0 {
                                visible_inks.insert(pixel.to_vec());
                            }
                        }
                        assert!(
                            visible_inks.len() <= 3,
                            "a GBC object has three visible inks"
                        );
                    }
                    let effect_only = render_pc_audit_canvas(&mut world, &images, move_id);
                    let effect_only = image::imageops::resize(
                        &effect_only,
                        160,
                        144,
                        image::imageops::FilterType::Nearest,
                    );
                    effect_fingerprint.update(effect_only.as_raw());
                    unique_effect_frames
                        .insert(sha2::Sha256::digest(effect_only.as_raw()).to_vec());
                    let mut activity = 0;
                    let mut contact = 0;
                    for (x, y, pixel) in effect_only.enumerate_pixels() {
                        if pixel[3] != 0 {
                            activity += 1;
                            let target = if player_move {
                                (132_i32, 55_i32)
                            } else {
                                (48_i32, 88_i32)
                            };
                            if (x as i32 - target.0).abs() <= 21
                                && (y as i32 - target.1).abs() <= 23
                            {
                                contact += 1;
                            }
                        }
                    }
                    if custom_oam.is_empty() {
                        contact = 0;
                    }
                    contact_scores.push((contact, activity));
                    if activity > 0 {
                        active_frames.push(frame);
                    }
                    {
                        let mut commands = Commands::new(&mut queue, &world);
                        northwoods_spawn_battle_layers(
                            &mut commands,
                            &snapshot,
                            &shell,
                            &animation,
                            &mut art,
                            &mut images,
                        );
                    }
                    queue.apply(&mut world);
                    let full = render_pc_audit_canvas(&mut world, &images, move_id);
                    let mut native = image::imageops::resize(
                        &full,
                        160,
                        144,
                        image::imageops::FilterType::Nearest,
                    );
                    for pixel in native.pixels_mut() {
                        assert!(
                            pixel[3] == 0 || pixel[3] == 255,
                            "native ink cannot alpha-fade"
                        );
                        if pixel[3] == 0 {
                            *pixel = image::Rgba([255; 4]);
                        }
                    }
                    unique_frames.insert(native.as_raw().clone());
                    if let Some(directory) = &directory {
                        use image::ImageEncoder as _;
                        let mut png = Vec::new();
                        image::codecs::png::PngEncoder::new(&mut png)
                            .write_image(native.as_raw(), 160, 144, image::ExtendedColorType::Rgba8)
                            .expect("encode retained native frame");
                        std::fs::write(directory.join(format!("frame-{frame:03}.png")), &png)
                            .expect("retain native frame");
                        encoded_frames.push(png);
                    }
                    rendered.push(native);
                }
                assert!(
                    checked_object_frames > 0,
                    "{move_id}: checked custom OAM must be nonempty"
                );
                let effect_fingerprint: [u8; 32] = effect_fingerprint.finalize().into();
                assert!(
                    unique_effect_frames.len() >= 6,
                    "{move_id}: effect-only raster must have at least six distinct frames"
                );
                assert!(
                    unique_frames.len() >= 6,
                    "{move_id} must visibly progress through authored poses"
                );
                assert!(
                    rendered.first() == rendered.last(),
                    "{move_id}: object cleanup must restore exact initial battle layers"
                );
                if player_move && !shiny {
                    assert!(
                        fingerprints.insert(effect_fingerprint),
                        "each signature needs distinct effect geometry/timing independent of its actor"
                    );
                    player_normal = rendered.first().cloned();
                }
                if shiny {
                    assert_eq!(
                        normal_effect.as_ref(),
                        Some(&effect_fingerprint),
                        "fixed effect palettes must not recolor with the shiny actor"
                    );
                } else {
                    normal_effect = Some(effect_fingerprint);
                }
                if player_move && shiny {
                    assert_ne!(
                        player_normal.as_ref(),
                        rendered.first(),
                        "shiny actor must retain its own palette"
                    );
                }
                if let Some(directory) = &directory {
                    let destination = directory
                        .parent()
                        .expect("move capture parent")
                        .join(format!("{variant}.png"));
                    write_northwoods_apng(&destination, &rendered, &encoded_frames);
                    let indices = [
                        0,
                        duration / 5,
                        duration * 2 / 5,
                        duration * 3 / 5,
                        duration * 4 / 5,
                        duration - 1,
                    ];
                    let mut strip = image::RgbaImage::new(160 * 6, 144);
                    for (column, index) in indices.into_iter().enumerate() {
                        image::imageops::overlay(
                            &mut strip,
                            &rendered[usize::from(index)],
                            column as i64 * 160,
                            0,
                        );
                    }
                    strip
                        .save(
                            directory
                                .parent()
                                .expect("move capture parent")
                                .join(format!("{variant}-filmstrip.png")),
                        )
                        .expect("retain review filmstrip");
                    let contact = contact_scores
                        .iter()
                        .enumerate()
                        .max_by_key(|(_, score)| score.0)
                        .map(|(index, _)| index)
                        .expect("contact timeline");
                    let peak = contact_scores
                        .iter()
                        .enumerate()
                        .max_by_key(|(_, score)| score.1)
                        .map(|(index, _)| index)
                        .expect("activity timeline");
                    let indices = [
                        usize::from(active_frames[0]),
                        usize::from(active_frames[active_frames.len() / 3]),
                        contact.saturating_sub(3),
                        contact,
                        peak,
                        usize::from(*active_frames.last().expect("active frame")),
                    ];
                    let mut contacts = image::RgbaImage::new(160 * 6, 144);
                    for (column, index) in indices.into_iter().enumerate() {
                        image::imageops::overlay(
                            &mut contacts,
                            &rendered[index],
                            column as i64 * 160,
                            0,
                        );
                    }
                    let parent = directory.parent().expect("move capture parent");
                    contacts
                        .save(parent.join(format!("{variant}-contact-strip.png")))
                        .expect("retain contact poses");
                    std::fs::write(
                        parent.join(format!("{variant}-keyframes.json")),
                        serde_json::to_vec_pretty(
                            &serde_json::json!({"move_id":move_id,"variant":variant,
                            "review_only":review_only,"contact_indices":indices,"frames":duration,
                            "effect_sha256":effect_fingerprint.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                            "unique_effect_frames":unique_effect_frames.len(),
                            "checked_object_frames":checked_object_frames}),
                        )
                        .expect("keyframe metadata"),
                    )
                    .expect("retain keyframe indices");
                }
            }
        }
    }
}

// APNG contains exact complete native frames. Its u16 delay fields use
// 400/23891 seconds, within 0.16 ns of the production Game Boy frame period
// (70224/4194304 seconds). Use image's existing PNG backend for compression.
fn write_northwoods_apng(path: &Path, frames: &[image::RgbaImage], encoded_frames: &[Vec<u8>]) {
    use image::AnimationDecoder;
    assert_eq!(frames.len(), encoded_frames.len());
    let mut output = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut sequence = 0_u32;
    for (index, frame) in frames.iter().enumerate() {
        let png = &encoded_frames[index];
        let mut cursor = 8;
        let mut compressed = Vec::new();
        while cursor < png.len() {
            let length = u32::from_be_bytes(
                png[cursor..cursor + 4]
                    .try_into()
                    .expect("PNG chunk length"),
            ) as usize;
            let kind = &png[cursor + 4..cursor + 8];
            let data = &png[cursor + 8..cursor + 8 + length];
            if index == 0 && kind == b"IHDR" {
                northwoods_png_chunk(&mut output, b"IHDR", data);
            }
            if kind == b"IDAT" {
                compressed.extend_from_slice(data);
            }
            cursor += length + 12;
        }
        if index == 0 {
            let mut control = (frames.len() as u32).to_be_bytes().to_vec();
            control.extend_from_slice(&0_u32.to_be_bytes());
            northwoods_png_chunk(&mut output, b"acTL", &control);
        }
        let mut control = sequence.to_be_bytes().to_vec();
        sequence += 1;
        control.extend_from_slice(&frame.width().to_be_bytes());
        control.extend_from_slice(&frame.height().to_be_bytes());
        control.extend_from_slice(&[0; 8]); // full frame, origin(0,0)
        control.extend_from_slice(&400_u16.to_be_bytes());
        control.extend_from_slice(&23_891_u16.to_be_bytes());
        control.extend_from_slice(&[0, 0]); // disposal:none, blend:source
        northwoods_png_chunk(&mut output, b"fcTL", &control);
        if index == 0 {
            northwoods_png_chunk(&mut output, b"IDAT", &compressed);
        } else {
            let mut data = sequence.to_be_bytes().to_vec();
            sequence += 1;
            data.extend_from_slice(&compressed);
            northwoods_png_chunk(&mut output, b"fdAT", &data);
        }
    }
    northwoods_png_chunk(&mut output, b"IEND", &[]);
    std::fs::write(path, output).expect("retain native APNG");
    let decoder = image::codecs::png::PngDecoder::new(std::io::BufReader::new(
        std::fs::File::open(path).expect("read retained APNG"),
    ))
    .expect("decode retained APNG");
    assert!(decoder.is_apng().expect("APNG control chunk"));
    let decoded = decoder
        .apng()
        .expect("APNG decoder")
        .into_frames()
        .collect_frames()
        .expect("decode every retained APNG frame");
    assert_eq!(decoded.len(), frames.len());
    for (decoded, original) in decoded.iter().zip(frames) {
        assert_eq!(
            decoded.buffer(),
            original,
            "APNG must preserve exact production pixels"
        );
        assert_eq!(
            decoded.delay().numer_denom_ms(),
            (400_000, 23_891),
            "native Game Boy frame timing"
        );
        let (numerator, denominator) = decoded.delay().numer_denom_ms();
        let milliseconds = f64::from(numerator) / f64::from(denominator);
        assert!(
            (milliseconds - crate::core::timing::GB_FRAME_DURATION_MS).abs() < 0.000_001,
            "APNG frame cadence must match production hardware timing within 1 ns"
        );
    }
}

fn northwoods_png_chunk(output: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(kind);
    output.extend_from_slice(data);
    let mut crc = !0_u32;
    for byte in kind.iter().chain(data) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    output.extend_from_slice(&(!crc).to_be_bytes());
}

fn northwoods_animation_selection(value: Option<&str>) -> Result<Vec<&'static str>, String> {
    let all = crystal_assets::northwoods_move_animations::IMPORTED_MOVES
        .into_iter()
        .chain(crystal_assets::minnesota_signature_animations::SIGNATURE_MOVES)
        .collect::<Vec<_>>();
    let Some(value) = value else {
        return Ok(all);
    };
    let mut selected = Vec::new();
    for token in value.split(',') {
        let token = token.trim().to_ascii_uppercase();
        let Some(&id) = all.iter().find(|&&id| id == token) else {
            return Err(format!("unknown extension move {token:?}"));
        };
        if selected.contains(&id) {
            return Err(format!("duplicate extension move {id}"));
        }
        selected.push(id);
    }
    Ok(selected)
}

#[test]
fn northwoods_animation_filter_rejects_unknown_empty_and_duplicate_ids() {
    assert_eq!(
        northwoods_animation_selection(None)
            .expect("all moves")
            .len(),
        19
    );
    assert_eq!(
        northwoods_animation_selection(Some("fairy_wind, MOONBLAST")).expect("selected moves"),
        ["FAIRY_WIND", "MOONBLAST"]
    );
    for invalid in ["", "SOUND", "FAIRY_WIND,", "FAIRY_WIND,FAIRY_WIND"] {
        assert!(northwoods_animation_selection(Some(invalid)).is_err());
    }
}

fn northwoods_animation_actor(
    data: &crystal_assets::GameDataSet,
    move_id: &str,
    level: u8,
) -> String {
    let preferred = match move_id {
        "MAGNETIC_RAIL" => "TACONARCH",
        "BOG_SIPHON" => "MOSQUITAN",
        "TIP_UP_STRIKE" => "TIPULORD",
        "SLIPPER_WALTZ" | "ENERGY_BALL" | "FAIRY_WIND" | "DISARMING_VOICE" | "DAZZLING_GLEAM"
        | "MOONBLAST" => "CYPRIELLE",
        "PIKE_DRILL" => "MUSKETUNGE",
        "WATER_PULSE" | "AQUA_JET" => "VOYAGEAR",
        "ICE_SHARD" => "TIPULORD",
        "IRON_HEAD" => "MESABLAST",
        "PLAY_ROUGH" => "BUTTERMUSE",
        "POISON_JAB" => "BARRICANT",
        "SIGNAL_BEAM" => "PADDARRAY",
        _ => "MOSQUITAN",
    };
    let learns = |id: &str| {
        data.learnsets.get(id).is_some_and(|learnset| {
            learnset
                .iter()
                .any(|entry| entry.0 <= level && entry.1 == move_id)
        })
    };
    if learns(preferred) {
        return preferred.to_string();
    }
    data.pokemon
        .iter()
        .find(|(id, species)| species.int_id > 251 && learns(id))
        .map(|(id, _)| id.clone())
        .expect("legal regional animation actor")
}

// Spawn fresh production layers on every frame: actor/HUD state can depend
// on the animation clock, including frames without visible effect objects.
fn northwoods_spawn_battle_layers(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    shell: &BevyRuntimeShell,
    animation: &VisibleMoveAnimation,
    art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
) {
    let battle = snapshot.battle.as_ref().expect("battle");
    spawn_battle_battler_markers(
        commands,
        snapshot,
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
        Some(animation),
        [false; 2],
        art,
        &shell.asset_root,
        images,
    )
    .expect("production battler raster");
    spawn_battle_hud(
        commands,
        snapshot,
        battle,
        0,
        false,
        false,
        false,
        None,
        None,
        &shell.runtime.data().growth_rates,
        art,
        &shell.asset_root,
        images,
    )
    .expect("production battle HUD");
    spawn_visible_battle_hud_clear(commands, shell);
}
