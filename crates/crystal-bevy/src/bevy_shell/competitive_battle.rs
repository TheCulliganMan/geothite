const COMPETITIVE_MOVE_BOARD_HEIGHT_TILES: f32 = 11.0;

#[cfg(feature = "location-tester")]
#[derive(Component)]
struct CompetitiveMoveMenuMarker;

fn competitive_summary_source_marker() -> impl Bundle + Clone {
    #[cfg(feature = "voxel-view")]
    { ImmersiveBattleReplaced }
    #[cfg(not(feature = "voxel-view"))]
    { () }
}

/// A denser move board inside the same LCD bounds in 2D and 3D. Production
/// cursors and commands remain the sole owners of selection and execution.
fn spawn_competitive_move_menu(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    battle: &crate::RuntimeBattleSnapshot,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let (active, selected, _, _) = battle_move_menu_state(snapshot, runtime_shell, battle)?;
    let view = snapshot
        .competitive_battle
        .as_ref()
        .context("competitive battle view missing")?;
    spawn_battle_window(
        commands,
        rendered_art,
        asset_root,
        images,
        0.0,
        7.0,
        20.0,
        COMPETITIVE_MOVE_BOARD_HEIGHT_TILES,
        6.0,
    );
    // The expanded board covers the source player HUD. Keep its important
    // information visible above the board, beside the opponent's sprite.
    spawn_battle_window_tagged(
        commands,
        rendered_art,
        asset_root,
        images,
        0.0,
        4.0,
        12.0,
        4.0,
        6.0,
        competitive_summary_source_marker(),
    );
    spawn_competitive_move_info_text_tagged(
        commands,
        rendered_art,
        asset_root,
        images,
        &competitive_board_label(&active.pokemon.nickname, 10),
        1.0,
        5.0,
        competitive_summary_source_marker(),
    );
    let status = match active.pokemon.status.as_deref() {
        Some("BURN") => "BRN",
        Some("PARALYSIS") => "PAR",
        Some("SLEEP") => "SLP",
        Some("FREEZE") => "FRZ",
        Some("POISON" | "BAD_POISON" | "TOXIC") => "PSN",
        _ => "HP",
    };
    spawn_competitive_move_info_text_tagged(
        commands,
        rendered_art,
        asset_root,
        images,
        &format!("{status}{}/{}", active.pokemon.hp, active.pokemon.max_hp),
        1.0,
        6.0,
        competitive_summary_source_marker(),
    );
    let selected_name = battle.player_moves.get(selected).map_or_else(
        || "CANCEL".into(),
        |learned| battle_move_display_name(snapshot, &learned.name),
    );
    let damage_on_hit = view
        .moves
        .get(selected)
        .is_some_and(|m| m.damage_on_hit.is_some());
    spawn_competitive_move_info_text(
        commands,
        rendered_art,
        asset_root,
        images,
        &competitive_board_label(&selected_name, if damage_on_hit { 13 } else { 18 }),
        1.0,
        8.0,
    );
    if damage_on_hit {
        spawn_competitive_move_info_text(
            commands,
            rendered_art,
            asset_root,
            images,
            "/HIT",
            15.0,
            8.0,
        );
    }
    let total = battle.player_moves.len() + 1;
    let start = selected.saturating_sub(3).min(total.saturating_sub(4));
    for row in 0..4 {
        let index = start + row;
        if index >= total {
            break;
        }
        let tile_y = 9.0 + row as f32 * 2.0;
        let marker = if runtime_shell.battle_move_swap_origin == Some(index) {
            "▷"
        } else if index == selected {
            battle_cursor_glyph(runtime_shell)
        } else {
            " "
        };
        let label = battle.player_moves.get(index).map_or_else(
            || "CANCEL".into(),
            |learned| battle_move_display_name(snapshot, &learned.name),
        );
        spawn_competitive_move_info_text(
            commands,
            rendered_art,
            asset_root,
            images,
            // Column 10 stays empty between names/PP and the metadata at 11.
            &competitive_board_label(&format!("{marker}{label}"), 9),
            1.0,
            tile_y,
        );
        if let Some(learned) = battle.player_moves.get(index) {
            let disabled = battle.player_disabled_move.as_deref() == Some(&learned.name);
            let pp = if disabled {
                "DISABLED".into()
            } else {
                visible_move_pp_text(snapshot, learned)
            };
            spawn_competitive_move_info_text(
                commands,
                rendered_art,
                asset_root,
                images,
                &pp,
                2.0,
                tile_y + 1.0,
            );
        }
    }
    if let Some(move_view) = view.moves.get(selected) {
        use crate::core::battle::damage::MoveCategory;
        let category = match move_view.category {
            MoveCategory::Physical => "PHYS",
            MoveCategory::Special => "SPCL",
            MoveCategory::Status => "STATUS",
        };
        let type_name = source_type_display_name(&move_view.move_type)?;
        let multiplier =
            crate::RuntimeCompetitiveBattleSnapshot::effectiveness_text(move_view.effectiveness);
        let damage = move_view.damage_on_hit;
        let support = move_view.category == MoveCategory::Status;
        let rows = [
            compact_scene_label(&type_name, 8),
            category.into(),
            if support {
                "PWR --".into()
            } else {
                move_view
                    .power
                    .map_or_else(|| "PWR ?".into(), |power| format!("PWR{power:>3}"))
            },
            format!("ACC{:>3}", move_view.accuracy),
            format!(
                "PRI{:>3}",
                i16::from(move_view.priority) - i16::from(view.base_priority)
            ),
            if support {
                String::new()
            } else {
                format!("{}{multiplier}", if move_view.stab { "STAB" } else { "" })
                    .trim()
                    .to_string()
            },
            damage.map_or_else(
                || if support { "".into() } else { "COND.".into() },
                |[min, _]| format!("LO{min:>4}"),
            ),
            damage.map_or_else(|| "".into(), |[_, max]| format!("HI{max:>4}")),
        ];
        for (row, text) in rows.iter().enumerate() {
            spawn_competitive_move_info_text(
                commands,
                rendered_art,
                asset_root,
                images,
                text,
                11.0,
                9.0 + row as f32,
            );
        }
    } else {
        spawn_competitive_move_info_text(
            commands,
            rendered_art,
            asset_root,
            images,
            "BACK",
            11.0,
            9.0,
        );
    }
    #[cfg(feature = "location-tester")]
    commands.spawn((BattleCommandMarker, CompetitiveMoveMenuMarker));
    Ok(())
}

fn competitive_board_label(label: &str, width: usize) -> String {
    if label.chars().count() <= width {
        return label.to_string();
    }
    // The source bitmap font has a period but no tilde. Introducing an
    // unsupported truncation glyph makes the entire label fail to render.
    label
        .chars()
        .take(width.saturating_sub(1))
        .collect::<String>()
        + "."
}

/// Keep each dense row in one draw, using the pack's existing bitmap glyphs.
/// Opaque font cells must not interleave with adjacent rows or source HUD art.
fn spawn_competitive_move_info_text(
    commands: &mut Commands, art: &mut RenderedTilesetArt, root: &AssetRoot,
    images: &mut Assets<Image>, text: &str, tile_x: f32, tile_y: f32,
) {
    spawn_competitive_move_info_text_tagged(commands, art, root, images, text, tile_x, tile_y, ());
}

fn spawn_competitive_move_info_text_tagged<M: Bundle>(
    commands: &mut Commands,
    art: &mut RenderedTilesetArt,
    root: &AssetRoot,
    images: &mut Assets<Image>,
    text: &str,
    tile_x: f32,
    tile_y: f32,
    marker: M,
) {
    let frames = bitmap_text_frames(art, root, images, text);
    if frames.is_empty() {
        return;
    }
    let width = frames.len() * BITMAP_FONT_TILE_SIZE;
    let mut pixels = vec![0; width * BITMAP_FONT_TILE_SIZE * 4];
    for (index, glyph) in frames.iter().enumerate() {
        let Some(source) = images
            .get(&glyph.handle)
            .and_then(|image| image.data.as_ref())
        else {
            return;
        };
        if source.len() != BITMAP_FONT_TILE_SIZE * BITMAP_FONT_TILE_SIZE * 4 {
            return;
        }
        for row in 0..BITMAP_FONT_TILE_SIZE {
            let from = row * BITMAP_FONT_TILE_SIZE * 4;
            let to = (row * width + index * BITMAP_FONT_TILE_SIZE) * 4;
            pixels[to..to + BITMAP_FONT_TILE_SIZE * 4]
                .copy_from_slice(&source[from..from + BITMAP_FONT_TILE_SIZE * 4]);
        }
    }
    let mut bitmap = Image::new(
        Extent3d {
            width: width as u32,
            height: BITMAP_FONT_TILE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    bitmap.sampler = ImageSampler::nearest();
    let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
    commands.spawn((
        Sprite {
            image: images.add(bitmap),
            custom_size: Some(Vec2::new(frames.len() as f32 * TILE_SIZE, TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(x + (frames.len() - 1) as f32 * TILE_SIZE * 0.5, y, 6.2),
        BattleCommandMarker,
        marker,
    ));
}
