// Presentation-only evidence for ordinary trainer-table sight. The trainer's
// final contact is observed after writeobjectxy; the sight event's target tile
// belongs to the pre-approach pose and never supplies the settled anchor.
#[derive(Debug, Clone)]
struct VisibleTrainerEncounterEvidence {
    object_script: String,
    provenance: crystal_render_api::VisualBattleTrainerProvenance,
    source: crate::core::world::session::OverworldSnapshot,
    request: crate::core::battle::start::TrainerBattleRequest,
    target_identifier: String,
    target_movement: String,
    target_index: usize,
    target_tile: TilePosition,
    target_facing: Direction,
    map_size: UVec2,
    placement: crystal_render_api::VisualBattleDerivedTrainerPlacement,
    minimum_snapshot_revision: u64,
    warm_checked: bool,
    capture_attempted: bool,
    witnessed_actor: Option<crystal_render_api::VisualActorId>,
    witnessed_foot: Option<Vec2>,
    anchors: Option<Arc<crystal_render_api::VisualBattleAnchorFrame>>,
}

// The ordinary table intro remains separate from scripted object lineage.
#[derive(Debug, Clone)]
struct VisibleTrainerEncounterCandidate {
    intro: PendingTrainerIntro,
    evidence: VisibleTrainerEncounterEvidence,
}

impl std::ops::Deref for VisibleTrainerEncounterCandidate {
    type Target = VisibleTrainerEncounterEvidence;
    fn deref(&self) -> &Self::Target {
        &self.evidence
    }
}

impl std::ops::DerefMut for VisibleTrainerEncounterCandidate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.evidence
    }
}

#[derive(Debug)]
struct VisibleBoundTrainerEncounter {
    origin: Arc<BattlePresentationOrigin>,
    candidate: VisibleTrainerEncounterEvidence,
    publication: Option<Arc<crystal_render_api::VisualBattleLocation>>,
}

fn trainer_candidate_matches_snapshot(
    candidate: &VisibleTrainerEncounterEvidence,
    snapshot: &RuntimeShellSnapshot,
) -> bool {
    same_static_encounter_pose(&candidate.source, &snapshot.overworld)
        && snapshot.script_events.last_talked_object.as_deref()
            == Some(candidate.target_identifier.as_str())
        && checked_static_target_present(
            snapshot,
            &candidate.target_identifier,
            candidate.target_tile,
            &candidate.target_movement,
            &candidate.object_script,
        )
        && snapshot
            .visible_objects
            .get(candidate.target_index)
            .is_some_and(|object| {
                object.object_identifier.as_deref() == Some(candidate.target_identifier.as_str())
                    && object.object_type
                        == match candidate.provenance {
                            crystal_render_api::VisualBattleTrainerProvenance::TrainerTable {
                                ..
                            } => "OBJECTTYPE_TRAINER",
                            crystal_render_api::VisualBattleTrainerProvenance::Scripted {
                                ..
                            } => "OBJECTTYPE_SCRIPT",
                        }
            })
        && snapshot
            .visible_object_facings
            .get(&candidate.target_identifier)
            == Some(&candidate.target_facing)
}

fn trainer_candidate_unbeaten(
    shell: &BevyRuntimeShell,
    candidate: &VisibleTrainerEncounterEvidence,
) -> bool {
    !candidate.request.event_flag.is_empty()
        && matches!(
            shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&candidate.request.event_flag),
            Ok(false)
        )
}

fn visible_trainer_floor_placement(
    shell: &BevyRuntimeShell,
    source: &crate::core::world::session::OverworldSnapshot,
    target_tile: TilePosition,
) -> Option<(
    UVec2,
    crystal_render_api::VisualBattleDerivedTrainerPlacement,
)> {
    let overworld = shell.shell.session().overworld();
    let Some((width, height)) = overworld.map.checked_tile_bounds() else {
        return None;
    };
    if width == 0 || height == 0 {
        return None;
    }
    let map_size = UVec2::new(u32::from(width), u32::from(height));
    use crate::core::world::collision::sample_collision;
    if ![source.tile, target_tile].into_iter().all(|tile| {
        sample_collision(&overworld.map, &overworld.tileset, tile)
            .is_some_and(|sample| walking_plain_support(sample.permission))
    }) {
        return None;
    }
    let Ok(occupied) = overworld.occupied_tiles_checked() else {
        return None;
    };
    let core = IVec2::new(i32::from(source.tile.x), i32::from(source.tile.y));
    let mut walkable = Vec::new();
    // Same fixed 7x7 source occupancy budget as ordinary walking encounters.
    // The settled trainer remains occupied and cannot be a Pokémon support.
    for y in -3..=3 {
        for x in -3..=3 {
            let p = core + IVec2::new(x, y);
            if p.cmplt(IVec2::ZERO).any() || p.cmpge(map_size.as_ivec2()).any() {
                continue;
            }
            let tile = TilePosition::new(p.x as i16, p.y as i16);
            if tile != target_tile
                && !occupied.iter().any(|entry| entry.tile == tile)
                && sample_collision(&overworld.map, &overworld.tileset, tile)
                    .is_some_and(|sample| walking_plain_support(sample.permission))
            {
                walkable.push(p);
            }
        }
    }
    let (dx, dy) = source.facing.delta();
    let Some(presentation_core_tile) =
        walking_presentation_tile(core, IVec2::new(i32::from(dx), i32::from(dy)), &walkable)
    else {
        return None;
    };
    Some((
        map_size,
        crystal_render_api::VisualBattleDerivedTrainerPlacement {
            presentation_core_tile,
            walkable_core_tiles: walkable.into(),
        },
    ))
}

fn stage_visible_trainer_candidate(
    shell: &mut BevyRuntimeShell,
    sight: &crate::core::world::session::OverworldInteraction,
) {
    shell.battle_origin.trainer_candidate = None;
    let Some(intro) = shell.pending_trainer_intro.clone() else {
        return;
    };
    let crate::core::world::session::OverworldInteractionTarget::Object {
        object_identifier: Some(identifier),
        object_type,
        ..
    } = &sight.target
    else {
        return;
    };
    if identifier.is_empty()
        || object_type != "OBJECTTYPE_TRAINER"
        || intro.origin_map_name != sight.map_name
        || intro.source_script != sight.script
        || intro.command_index != 0
        || !shell
            .shell
            .runtime()
            .compiled_script_command_name(&intro.source_script, 0)
            .is_ok_and(|command| command == "trainer")
    {
        return;
    }
    let Ok(snapshot) = shell.shell.presentation_snapshot() else {
        return;
    };
    let source = &snapshot.overworld;
    if snapshot.battle.is_some()
        || source.map_name != sight.map_name
        || source.tile != sight.player_tile
        || source.facing != sight.facing
        || source.mode != MovementMode::Normal
        || shell.player_walk_frame_ticks > 0
        || shell.object_walk_frame_ticks > 0
        || shell.visible_ledge_jump.is_some()
        || shell.pending_trainer_sight.is_some()
    {
        return;
    }
    let Some((target_index, target)) = snapshot
        .visible_objects
        .iter()
        .enumerate()
        .find(|(_, object)| object.object_identifier.as_deref() == Some(identifier.as_str()))
    else {
        return;
    };
    let Some(target_tile) = snapshot
        .visible_object_runtime_tiles
        .get(identifier)
        .copied()
        .or_else(|| object_tile_position_checked(target))
    else {
        return;
    };
    let Some(target_facing) = snapshot.visible_object_facings.get(identifier).copied() else {
        return;
    };
    if target.script != intro.source_script
        || target_facing != visible_opposite_direction(source.facing)
        || crate::core::world::movement::checked_move_by_stride(source.tile, source.facing, 1)
            != Some(target_tile)
    {
        return;
    }
    let Ok(request) = shell
        .shell
        .runtime()
        .data()
        .scripted_trainer_battle_request(
            &intro.origin_map_name,
            &intro.source_script,
            intro.command_index,
        )
    else {
        return;
    };
    if request.source_script != intro.source_script
        || request.battle_type != "BATTLETYPE_TRAINER"
        || request.trainer_id.is_empty()
        || request.trainer_class.is_empty()
    {
        return;
    }
    let Some((map_size, placement)) = visible_trainer_floor_placement(shell, source, target_tile)
    else {
        return;
    };
    let candidate = VisibleTrainerEncounterCandidate {
        evidence: VisibleTrainerEncounterEvidence {
            source: source.clone(),
            object_script: intro.source_script.clone(),
            provenance: crystal_render_api::VisualBattleTrainerProvenance::TrainerTable {
                command_index: intro.command_index,
            },
            request,
            target_identifier: identifier.clone(),
            target_movement: target.spritemovedata.clone(),
            target_index,
            target_tile,
            target_facing,
            map_size,
            placement,
            minimum_snapshot_revision: shell.snapshot_revision,
            warm_checked: false,
            capture_attempted: false,
            witnessed_actor: None,
            witnessed_foot: None,
            anchors: None,
        },
        intro,
    };
    if trainer_candidate_matches_snapshot(&candidate, &snapshot)
        && trainer_candidate_unbeaten(shell, &candidate)
    {
        shell.battle_origin.trainer_candidate = Some(candidate);
    }
}

fn trainer_candidate_matches_start(
    candidate: &VisibleTrainerEncounterCandidate,
    pending: &PendingTrainerIntro,
    start: &crate::core::battle::start::TrainerBattleStart,
    battle: &crate::RuntimeBattleSnapshot,
) -> bool {
    &candidate.intro == pending
        && candidate.source.map_name == pending.origin_map_name
        && trainer_evidence_matches_start(&candidate.evidence, start, battle)
}

fn trainer_evidence_matches_start(
    candidate: &VisibleTrainerEncounterEvidence,
    start: &crate::core::battle::start::TrainerBattleStart,
    battle: &crate::RuntimeBattleSnapshot,
) -> bool {
    let crate::RuntimeBattleKind::Trainer {
        trainer_class,
        trainer_id,
        event_flag,
        source_script,
        seen_text,
        win_text,
        loss_text,
        callback,
        ..
    } = &battle.kind
    else {
        return false;
    };
    let request = &candidate.request;
    start.battle_type == request.battle_type
        && battle.battle_type == request.battle_type
        && start.trainer_class == request.trainer_class
        && trainer_class == &request.trainer_class
        && start.trainer_id == request.trainer_id
        && trainer_id == &request.trainer_id
        && start.event_flag == request.event_flag
        && event_flag == &request.event_flag
        && start.source_script == request.source_script
        && source_script == &request.source_script
        && start.seen_text == request.seen_text
        && seen_text == &request.seen_text
        && start.win_text == request.win_text
        && win_text == &request.win_text
        && start.loss_text == request.loss_text
        && loss_text == &request.loss_text
        && start.callback == request.callback
        && callback == &request.callback
}

fn bind_visible_trainer_encounter(
    shell: &mut BevyRuntimeShell,
    pending: &PendingTrainerIntro,
    started: &crate::TrainerBattleStartStatus,
) {
    let Some(mut candidate) = shell.battle_origin.trainer_candidate.take() else {
        return;
    };
    let crate::TrainerBattleStartStatus::Started(start) = started else {
        return;
    };
    let Ok(snapshot) = shell.shell.presentation_snapshot() else {
        return;
    };
    let Some(battle) = snapshot.battle.as_ref() else {
        return;
    };
    if shell.battle_origin.published().is_some()
        || !trainer_candidate_matches_snapshot(&candidate, &snapshot)
        || !trainer_candidate_unbeaten(shell, &candidate)
        || !trainer_candidate_matches_start(&candidate, pending, start, battle)
    {
        return;
    }
    // The seen-text window has ended. Never capture an arbitrary later scene.
    candidate.capture_attempted = true;
    stage_visible_battle_origin(
        shell,
        &candidate.source,
        BattleOriginKind::Trainer,
        &start.battle_type,
        Some(BattleOriginContact::TrainerObjectTarget {
            map_id: candidate.source.map_name.clone(),
            tile: candidate.target_tile,
            object_identifier: candidate.target_identifier.clone(),
            trainer_id: candidate.request.trainer_id.clone(),
            source_script: candidate.intro.source_script.clone(),
            trainer_command_index: candidate.intro.command_index,
        }),
        None,
    );
    let origin = shell
        .battle_origin
        .pending
        .as_ref()
        .expect("trainer origin just staged")
        .clone();
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("CRYSTAL_ENCOUNTER_TRACE").is_some() {
        eprintln!(
            "trainer witness: generation={} map={} player={:?} facing={:?} trainer={} tile={:?} facing={:?} class={} id={} script={} trainer_command={} pokemon={:?} anchored={}",
            origin.generation,
            candidate.source.map_name,
            candidate.source.tile,
            candidate.source.facing,
            candidate.target_identifier,
            candidate.target_tile,
            candidate.target_facing,
            candidate.request.trainer_class,
            candidate.request.trainer_id,
            candidate.intro.source_script,
            candidate.intro.command_index,
            candidate.placement.presentation_core_tile,
            candidate.anchors.is_some(),
        );
    }
    shell.battle_origin.bound_trainer = Some(VisibleBoundTrainerEncounter {
        origin,
        candidate: candidate.evidence,
        publication: None,
    });
}

#[cfg(any(test, feature = "voxel-view"))]
fn freeze_visible_trainer_anchors(
    candidate: &mut VisibleTrainerEncounterEvidence,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
    objects: &[(usize, Option<&str>, &str)],
) {
    if candidate.capture_attempted {
        return;
    }
    if !candidate.warm_checked {
        candidate.warm_checked = true;
        if !frame.is_some_and(|frame| {
            frame.active
                && frame.validate().is_ok()
                && frame.map_id.as_ref() == candidate.source.map_name
                && frame.source_map_size_core_tiles == Some(candidate.map_size)
        }) || rendered.map_name.as_deref() != Some(candidate.source.map_name.as_str())
            || rendered.tile != Some(candidate.source.tile)
        {
            candidate.capture_attempted = true;
            return;
        }
    }
    // The immediately preceding approach frame may still contain the old
    // trainer pose. Only the completed intro snapshot is eligible to freeze.
    if !walking_snapshot_is_current(
        rendered.snapshot_revision,
        candidate.minimum_snapshot_revision,
    ) {
        return;
    }
    candidate.capture_attempted = true;
    let Some(scene) = capture_visible_fishing_source_frame(
        &candidate.source,
        Some(candidate.map_size),
        rendered,
        frame,
    ) else {
        return;
    };
    let Some(frame) = frame else {
        return;
    };
    let mut targets = objects
        .iter()
        .filter(|(_, identifier, _)| *identifier == Some(candidate.target_identifier.as_str()));
    let Some((index, _, source_id)) = targets.next() else {
        return;
    };
    if *index != candidate.target_index
        || targets.next().is_some()
        || objects.iter().filter(|(slot, _, _)| slot == index).count() != 1
    {
        return;
    }
    let Ok(index) = u32::try_from(*index) else {
        return;
    };
    let actor_id = crystal_render_api::VisualActorId::Object(index);
    let Some(actor) = frame.actors.iter().find(|actor| actor.id == actor_id) else {
        return;
    };
    let Some(contact_foot) = fishing_source_support(&scene.terrain, candidate.target_tile) else {
        return;
    };
    let actual_foot = actor.center - Vec2::Y * actor.size.y * 0.5;
    if actor.source_id.as_ref() != *source_id
        || !actual_foot.is_finite()
        || (actual_foot - contact_foot).length_squared() > 0.0001
        || actor.facing != Some(visual_facing(candidate.target_facing))
    {
        return;
    }
    let target = candidate.placement.presentation_core_tile;
    let Some(target_foot) = fishing_source_support(
        &scene.terrain,
        TilePosition::new(target.x as i16, target.y as i16),
    ) else {
        return;
    };
    candidate.witnessed_actor = Some(actor_id);
    candidate.witnessed_foot = Some(actual_foot);
    candidate.anchors = Some(Arc::new(crystal_render_api::VisualBattleAnchorFrame {
        terrain: scene.terrain,
        source_actor: crystal_render_api::VisualActorId::Player,
        target_actor: None,
        source_foot: scene.source_foot,
        target_foot,
    }));
}

#[cfg(not(any(test, feature = "voxel-view")))]
fn freeze_visible_trainer_anchors(
    candidate: &mut VisibleTrainerEncounterEvidence,
    _rendered: &RenderedViewport,
    _frame: Option<&crystal_render_api::VisualWorldFrame>,
    _objects: &[(usize, Option<&str>, &str)],
) {
    candidate.capture_attempted = true;
}

fn publish_visible_trainer_location(
    shell: &mut BevyRuntimeShell,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
    objects: &Query<&VisibleObjectSprite>,
) -> Option<Arc<crystal_render_api::VisualBattleLocation>> {
    freeze_pending_visible_scripted_trainer(shell, rendered, frame, objects);
    if let Some(mut candidate) = shell.battle_origin.trainer_candidate.take() {
        if shell.pending_trainer_intro.as_ref() == Some(&candidate.intro)
            && trainer_candidate_unbeaten(shell, &candidate)
            && same_static_encounter_pose(&candidate.source, &shell.shell.session().snapshot())
        {
            if !candidate.capture_attempted {
                let roster: Vec<_> = objects
                    .iter()
                    .map(|object| {
                        (
                            object.object_index,
                            object.object_identifier.as_deref(),
                            object.source_id.as_ref(),
                        )
                    })
                    .collect();
                freeze_visible_trainer_anchors(&mut candidate, rendered, frame, &roster);
            }
            shell.battle_origin.trainer_candidate = Some(candidate);
        }
    }
    if shell
        .battle_origin
        .bound_trainer
        .as_ref()
        .is_some_and(|bound| {
            !shell
                .battle_origin
                .published()
                .is_some_and(|origin| Arc::ptr_eq(origin, &bound.origin))
        })
    {
        shell.battle_origin.bound_trainer = None;
    }
    let bound = shell.battle_origin.bound_trainer.as_mut()?;
    Some(
        bound
            .publication
            .get_or_insert_with(|| {
                let candidate = &bound.candidate;
                let source = &candidate.source;
                let (dx, dy) = source.facing.delta();
                let (tx, ty) = candidate.target_facing.delta();
                Arc::new(crystal_render_api::VisualBattleLocation {
                    generation: bound.origin.generation,
                    source: crystal_render_api::VisualBattleSourcePose {
                        map_id: Arc::from(source.map_name.as_str()),
                        source_frame: source.frame,
                        core_tile: IVec2::new(i32::from(source.tile.x), i32::from(source.tile.y)),
                        facing: IVec2::new(i32::from(dx), i32::from(dy)),
                        movement: crystal_render_api::VisualBattleSourceMovement::Normal,
                    },
                    target: crystal_render_api::VisualBattleTarget::Trainer {
                        contact: crystal_render_api::VisualBattleTrainerTarget {
                            object_identifier: Arc::from(candidate.target_identifier.as_str()),
                            object_script: Arc::from(candidate.object_script.as_str()),
                            core_tile: IVec2::new(
                                i32::from(candidate.target_tile.x),
                                i32::from(candidate.target_tile.y),
                            ),
                            facing: IVec2::new(i32::from(tx), i32::from(ty)),
                            trainer_class: Arc::from(candidate.request.trainer_class.as_str()),
                            trainer_id: Arc::from(candidate.request.trainer_id.as_str()),
                            event_flag: Arc::from(candidate.request.event_flag.as_str()),
                            battle_source_script: Arc::from(candidate.object_script.as_str()),
                            provenance: candidate.provenance,
                            witnessed_actor: candidate.witnessed_actor,
                            witnessed_foot: candidate.witnessed_foot,
                        },
                        presentation: candidate.placement.clone(),
                    },
                    source_map_size_core_tiles: Some(candidate.map_size),
                    anchors: candidate.anchors.clone(),
                })
            })
            .clone(),
    )
}

/// A fresh disposable setup only. Up must enter Abe's ordinary sight line;
/// no forced battle, trainer movement, flag, encounter roll or journal edit.
#[cfg(any(test, feature = "location-tester"))]
fn prepare_gym_encounter_preview(mut shell: BevyRuntimeShell) -> Result<BevyRuntimeShell> {
    anyhow::ensure!(
        shell.quick_save_path.is_none(),
        "gym preview cannot write a user save"
    );
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("CHRIS"))?;
    let initial = shell.shell.snapshot()?;
    anyhow::ensure!(
        initial.party.slots.is_empty()
            && initial.battle.is_none()
            && initial.overworld.map_name == "VioletGym"
            && initial.overworld.tile == TilePosition::new(5, 11),
        "gym preview requires a fresh empty-party Violet Gym session at (5,11)"
    );
    let runtime = shell.shell.runtime().clone();
    let data = runtime.data();
    let map = data.maps.get("VioletGym").context("compiled Violet Gym")?;
    let abe = map
        .objects
        .iter()
        .find(|object| object.object_identifier.as_deref() == Some("VIOLETGYM_YOUNGSTER2"))
        .context("compiled Abe object")?;
    anyhow::ensure!(
        abe.object_type == "OBJECTTYPE_TRAINER"
            && abe.script == "TrainerBirdKeeperAbe"
            && object_tile_position_checked(abe) == Some(TilePosition::new(2, 10))
            && abe.spritemovedata == "SPRITEMOVEDATA_STANDING_RIGHT"
            && abe.radius == 3
            && runtime.compiled_script_command_name(&abe.script, 0)? == "trainer",
        "gym preview requires the compiled ordinary Abe sight path"
    );
    let request = data.scripted_trainer_battle_request("VioletGym", &abe.script, 0)?;
    let trainer = data
        .trainers
        .get(&request.trainer_id)
        .context("compiled Abe party")?;
    anyhow::ensure!(
        request.trainer_class == "BIRD_KEEPER"
            && request.event_flag == "EVENT_BEAT_BIRD_KEEPER_ABE"
            && trainer.party.len() == 1
            && trainer.party[0].species == "SPEAROW"
            && trainer.party[0].level == 9,
        "gym preview requires Abe's original Spearow level 9"
    );
    let overworld = shell.shell.session().overworld();
    for tile in [
        TilePosition::new(5, 11),
        TilePosition::new(5, 10),
        TilePosition::new(3, 10),
        TilePosition::new(4, 10),
        TilePosition::new(5, 12),
    ] {
        anyhow::ensure!(
            crate::core::world::collision::sample_collision(
                &overworld.map,
                &overworld.tileset,
                tile,
            )
            .is_some_and(|sample| walking_plain_support(sample.permission)),
            "gym preview requires source floor at {tile:?}"
        );
    }
    shell.shell.add_party_pokemon(
        "CYNDAQUIL",
        20,
        None,
        None,
        &initial.trainer.player_name,
        initial.trainer.player_id,
        Dv::from_non_hp(9, 9, 9, 9),
    )?;
    {
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        overworld.set_player_facing(Direction::Up);
        state.overworld = crate::core::state::OverworldMemory::from_snapshot(&overworld.snapshot());
    }
    reset_visible_navigation_state(&mut shell);
    mark_runtime_snapshot_dirty(&mut shell);
    settle_visible_shell_smoke_until_idle(&mut shell)?;
    let ready = shell.shell.snapshot()?;
    anyhow::ensure!(
        ready.battle.is_none()
            && ready.overworld.map_name == "VioletGym"
            && ready.overworld.tile == TilePosition::new(5, 11)
            && ready.overworld.facing == Direction::Up
            && ready.overworld.mode == MovementMode::Normal
            && shell.pending_trainer_intro.is_none()
            && shell.pending_trainer_sight.is_none()
            && !shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&request.event_flag)
                .map_err(|error| anyhow::anyhow!("check Abe event: {error}"))?,
        "gym preview must await ordinary Up before an unbeaten trainer"
    );
    mark_runtime_snapshot_dirty(&mut shell);
    Ok(shell)
}
