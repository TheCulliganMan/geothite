// A map-owned arena for committed entries without a checked enemy contact.
// The floor placement is explicitly derived; it never becomes core state.
#[derive(Debug)]
struct VisibleBoundScriptedGround {
    origin: Arc<BattlePresentationOrigin>,
    map_size: UVec2,
    placement: crystal_render_api::VisualBattleDerivedTrainerPlacement,
    source_world_key: u64,
    capture_attempted: bool,
    publication: Option<Arc<crystal_render_api::VisualBattleLocation>>,
}

fn bind_visible_scripted_ground(
    shell: &mut BevyRuntimeShell,
    origin: &Arc<BattlePresentationOrigin>,
) {
    let Ok(snapshot) = shell.shell.presentation_snapshot() else { return; };
    if origin.contact.is_some()
        || origin.source.mode != MovementMode::Normal
        || origin.visual_step.ledge_jump.is_some()
        || shell.shell.session().snapshot() != origin.source
        || snapshot.battle.is_none()
    {
        return;
    }
    let Some((map_size, placement)) = visible_battle_floor_placement(shell, &origin.source, None)
    else { return; };
    // Compare with the same field-only presentation expansion used by
    // render_playfield. The committed battle snapshot omits distant actors
    // and follower overrides, which would otherwise reject an unchanged map.
    let mut field_snapshot = snapshot;
    field_snapshot.battle = None;
    #[cfg(any(feature = "fullscreen-scaling", feature = "voxel-view"))]
    if expand_fullscreen_object_presentation(&mut field_snapshot, shell).is_err() { return; }
    for (id, tile) in &shell.follower_visible_tile_overrides {
        field_snapshot.visible_object_runtime_tiles.insert(id.clone(), *tile);
    }
    shell.battle_origin.bound_scripted_ground = Some(VisibleBoundScriptedGround {
        origin: origin.clone(), map_size, placement,
        source_world_key: overworld_render_world_key(&field_snapshot),
        capture_attempted: false, publication: None,
    });
}

fn publish_visible_scripted_ground(
    shell: &mut BevyRuntimeShell,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
) -> Option<Arc<crystal_render_api::VisualBattleLocation>> {
    let valid = shell.battle_origin.bound_scripted_ground.as_ref().is_some_and(|bound|
        shell.battle_origin.published().is_some_and(|origin| Arc::ptr_eq(origin, &bound.origin)));
    if !valid {
        shell.battle_origin.bound_scripted_ground = None;
        return None;
    }
    let entry_window = shell.visible_battle_transition.is_some_and(|transition| transition.frame < 3);
    let current = shell.shell.session().snapshot();
    let bound = shell.battle_origin.bound_scripted_ground.as_mut()?;
    if let Some(location) = &bound.publication { return Some(location.clone()); }
    if bound.capture_attempted { return None; }
    // Never substitute a later map, pose, or returning field frame. A cold
    // entry remains unbound instead of acquiring unrelated scenery afterward.
    bound.capture_attempted = true;
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("CRYSTAL_ENCOUNTER_TRACE").is_some() {
        eprintln!("scripted floor capture: window={entry_window} pose={} world={:?}/{:?} motion={} frame_active={} rendered_map={:?} tile={:?} facing={:?}",
            same_static_encounter_pose(&bound.origin.source, &current), rendered.world_key,
            bound.source_world_key, rendered.object_motion_active,
            frame.is_some_and(|frame| frame.active), rendered.map_name, rendered.tile, rendered.player_sprite_facing);
    }
    if !entry_window || !same_static_encounter_pose(&bound.origin.source, &current)
        // Battle commitment changes the shell revision even when the field
        // pixels stay unchanged. Require its exact map blocks, occupancy,
        // player pose and object appearances instead of that UI revision.
        || rendered.world_key != Some(bound.source_world_key)
        || rendered.object_motion_active
    { return None; }
    let scene = capture_visible_fishing_source_frame(
        &bound.origin.source, Some(bound.map_size), rendered, frame)?;
    let target = bound.placement.presentation_core_tile;
    let target_foot = fishing_source_support(&scene.terrain,
        TilePosition::new(target.x as i16, target.y as i16))?;
    use crystal_render_api::{VisualActorId, VisualBattleAnchorFrame, VisualBattleLocation,
        VisualBattleSourceMovement, VisualBattleSourcePose, VisualBattleTarget};
    let source = &bound.origin.source;
    let (dx, dy) = source.facing.delta();
    let location = Arc::new(VisualBattleLocation {
        generation: bound.origin.generation,
        source: VisualBattleSourcePose {
            map_id: Arc::from(source.map_name.as_str()), source_frame: source.frame,
            core_tile: IVec2::new(i32::from(source.tile.x), i32::from(source.tile.y)),
            facing: IVec2::new(i32::from(dx), i32::from(dy)),
            movement: VisualBattleSourceMovement::Normal,
        },
        target: VisualBattleTarget::ScriptedGround { presentation: bound.placement.clone() },
        source_map_size_core_tiles: Some(bound.map_size),
        anchors: Some(Arc::new(VisualBattleAnchorFrame {
            terrain: scene.terrain, source_actor: VisualActorId::Player, target_actor: None,
            source_foot: scene.source_foot, target_foot,
        })),
    });
    bound.publication = Some(location.clone());
    Some(location)
}

/// Disposable source setup only. The original coordinate event, movements,
/// dialogue and battle request must still execute through ordinary input.
#[cfg(feature = "location-tester")]
fn prepare_rival_encounter_preview(mut shell: BevyRuntimeShell) -> Result<BevyRuntimeShell> {
    anyhow::ensure!(shell.quick_save_path.is_none(), "rival preview cannot write a user save");
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("CHRIS"))?;
    let initial = shell.shell.snapshot()?;
    anyhow::ensure!(initial.party.slots.is_empty() && initial.battle.is_none()
        && initial.overworld.map_name == "CherrygroveCity"
        && initial.overworld.tile == TilePosition::new(32, 7),
        "rival preview requires a fresh empty-party Cherrygrove session at (32,7)");
    let runtime = shell.shell.runtime().clone();
    shell.shell.add_party_pokemon("CYNDAQUIL", 20, None, None,
        &initial.trainer.player_name, initial.trainer.player_id, Dv::from_non_hp(9, 9, 9, 9))?;
    {
        let state = shell.shell.session_mut().state_mut();
        state.flags.set_event_flag("EVENT_GOT_CYNDAQUIL_FROM_ELM", true)?;
        state.scenes.set_current_scene("SCENE_CHERRYGROVECITY_MEET_RIVAL",
            runtime.data().map_scene_table("CherrygroveCity")?)?;
    }
    reset_visible_navigation_state(&mut shell);
    mark_runtime_snapshot_dirty(&mut shell);
    settle_visible_shell_smoke_until_idle(&mut shell)?;
    anyhow::ensure!(shell.shell.snapshot()?.battle.is_none(), "rival fixture must await ordinary Right");
    Ok(shell)
}
