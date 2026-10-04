// A bounded, presentation-only witness for direct scripted object battles.
// The object dispatch and actual interpreter results own all provenance;
// authored metadata identifies candidate commands but never proves execution.
#[derive(Debug, Clone)]
struct VisibleScriptedTrainerEncounterCandidate {
    evidence: VisibleTrainerEncounterEvidence,
    expected_step: RuntimeCompiledScriptCursor,
    faceplayer_witnessed: bool,
    dialogue_witnessed: bool,
    loadtrainer_witnessed: bool,
}

fn stage_visible_scripted_trainer_candidate(
    shell: &mut BevyRuntimeShell,
    interaction: &crate::core::world::session::OverworldInteraction,
    dispatch: &crate::RuntimeInteractionScriptDispatch,
) {
    shell.battle_origin.scripted_trainer_candidate = None;
    let crate::core::world::session::OverworldInteractionTarget::Object {
        object_identifier: Some(identifier),
        object_type,
        ..
    } = &interaction.target
    else {
        return;
    };
    if identifier.is_empty()
        || object_type != "OBJECTTYPE_SCRIPT"
        || dispatch.next_script != interaction.script
        || dispatch.last_talked_object.as_deref() != Some(identifier.as_str())
        || !shell
            .shell
            .session()
            .state()
            .script_runtime
            .next_script
            .as_ref()
            .is_some_and(|next| {
                next.origin_map_name == interaction.map_name && next.script == interaction.script
            })
        || !shell
            .shell
            .runtime()
            .compiled_script_command_name(&interaction.script, 0)
            .is_ok_and(|command| command == "faceplayer")
    {
        return;
    }
    let Ok(snapshot) = shell.shell.presentation_snapshot() else {
        return;
    };
    let source = &snapshot.overworld;
    if snapshot.battle.is_some()
        || source.map_name != interaction.map_name
        || source.tile != interaction.player_tile
        || source.facing != interaction.facing
        || source.mode != MovementMode::Normal
        || shell.player_walk_frame_ticks > 0
        || shell.object_walk_frame_ticks > 0
        || shell.visible_ledge_jump.is_some()
        || shell.pending_trainer_sight.is_some()
        || shell.pending_trainer_intro.is_some()
        || crate::core::world::movement::checked_move_by_stride(source.tile, source.facing, 1)
            != Some(interaction.target_tile)
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
    if target.object_type != "OBJECTTYPE_SCRIPT"
        || target.script != interaction.script
        || !checked_static_target_present(
            &snapshot,
            identifier,
            interaction.target_tile,
            &target.spritemovedata,
            &interaction.script,
        )
    {
        return;
    }
    let Some(target_facing) = snapshot.visible_object_facings.get(identifier).copied() else {
        return;
    };
    let Ok(module) = shell
        .shell
        .runtime()
        .data()
        .map_module(&interaction.map_name)
    else {
        return;
    };
    let mut matches = module
        .scripted_trainer_battles
        .iter()
        .filter(|battle| battle.source_script == interaction.script);
    let Some(authored) = matches.next() else {
        return;
    };
    if matches.next().is_some()
        || authored.loadtrainer_command_index == 0
        || authored.loadtrainer_command_index >= authored.startbattle_command_index
        || authored.request.source_script != interaction.script
        || authored.request.battle_type != "BATTLETYPE_TRAINER"
        || authored.request.trainer_class.is_empty()
        || authored.request.trainer_id.is_empty()
        || !shell
            .shell
            .runtime()
            .compiled_script_command_name(&interaction.script, authored.loadtrainer_command_index)
            .is_ok_and(|command| command == "loadtrainer")
        || !shell
            .shell
            .runtime()
            .compiled_script_command_name(&interaction.script, authored.startbattle_command_index)
            .is_ok_and(|command| command == "startbattle")
    {
        return;
    }
    let Some((map_size, placement)) =
        visible_trainer_floor_placement(shell, source, interaction.target_tile)
    else {
        return;
    };
    let evidence = VisibleTrainerEncounterEvidence {
        object_script: interaction.script.clone(),
        provenance: crystal_render_api::VisualBattleTrainerProvenance::Scripted {
            loadtrainer_command_index: authored.loadtrainer_command_index,
            startbattle_command_index: authored.startbattle_command_index,
        },
        source: source.clone(),
        request: authored.request.clone(),
        target_identifier: identifier.clone(),
        target_movement: target.spritemovedata.clone(),
        target_index,
        target_tile: interaction.target_tile,
        target_facing,
        map_size,
        placement,
        minimum_snapshot_revision: shell.snapshot_revision,
        warm_checked: false,
        capture_attempted: false,
        witnessed_actor: None,
        witnessed_foot: None,
        anchors: None,
    };
    if !trainer_candidate_matches_snapshot(&evidence, &snapshot) {
        return;
    }
    shell.battle_origin.scripted_trainer_candidate =
        Some(VisibleScriptedTrainerEncounterCandidate {
            evidence,
            expected_step: RuntimeCompiledScriptCursor {
                origin_map_name: interaction.map_name.clone(),
                source_script: interaction.script.clone(),
                command_index: 0,
            },
            faceplayer_witnessed: false,
            dialogue_witnessed: false,
            loadtrainer_witnessed: false,
        });
}

fn observe_visible_encounter_script_step(
    shell: &mut BevyRuntimeShell,
    step: &crate::RuntimeCompiledScriptStep,
) {
    observe_visible_static_encounter_step(shell, step);
    observe_visible_scripted_trainer_step(shell, step);
}

fn observe_visible_scripted_trainer_step(
    shell: &mut BevyRuntimeShell,
    step: &crate::RuntimeCompiledScriptStep,
) {
    let Some(mut candidate) = shell.battle_origin.scripted_trainer_candidate.take() else {
        return;
    };
    let observed = RuntimeCompiledScriptCursor {
        origin_map_name: step.origin_map_name.clone(),
        source_script: step.source_script.clone(),
        command_index: step.command_index,
    };
    if candidate.expected_step != observed {
        return;
    }
    // This first direct-object path accepts only an unchanged field scene.
    // Unknown commands (including changeblock, movement, object visibility or
    // warps) cannot carry old terrain/occupancy evidence into a later battle.
    if !matches!(
        step.command.as_str(),
        "faceplayer"
            | "opentext"
            | "checkevent"
            | "iftrue"
            | "iffalse"
            | "writetext"
            | "waitbutton"
            | "closetext"
            | "winlosstext"
            | "loadtrainer"
            | "startbattle"
    ) {
        return;
    }
    let Ok(snapshot) = shell.shell.presentation_snapshot() else {
        return;
    };
    if step.command == "faceplayer" {
        if candidate.faceplayer_witnessed
            || !matches!(&step.mutation.result,
            RuntimeMutationResult::ScriptObjectMutated(result)
                if result.command == "faceplayer" && result.object_id == candidate.evidence.target_identifier
                    && result.source_script == observed.source_script && result.command_index == observed.command_index)
        {
            return;
        }
        candidate.evidence.target_facing =
            visible_opposite_direction(candidate.evidence.source.facing);
        candidate.faceplayer_witnessed = true;
    }
    if !trainer_candidate_matches_snapshot(&candidate.evidence, &snapshot) {
        return;
    }
    let crystal_render_api::VisualBattleTrainerProvenance::Scripted {
        loadtrainer_command_index,
        startbattle_command_index,
    } = candidate.evidence.provenance
    else {
        return;
    };
    if matches!(
        &step.mutation.result,
        RuntimeMutationResult::ScriptTextApplied(
            crate::core::systems::script_text::ScriptTextAction::Write { .. }
        )
    ) && matches!(
        &step.boundary,
        Some(crate::RuntimeCompiledScriptBoundary::TextLabel(_))
    ) {
        if !candidate.faceplayer_witnessed || candidate.loadtrainer_witnessed {
            return;
        }
        if !candidate.dialogue_witnessed {
            candidate.dialogue_witnessed = true;
            candidate.evidence.source = snapshot.overworld.clone();
            // Integration of this actual text boundary increments the revision.
            // Never capture the preceding faceplayer/approach publication.
            candidate.evidence.minimum_snapshot_revision = shell.snapshot_revision.wrapping_add(1);
        }
    }
    if step.command_index == loadtrainer_command_index {
        if !candidate.dialogue_witnessed
            || candidate.loadtrainer_witnessed
            || !matches!(&step.mutation.result, RuntimeMutationResult::ScriptRuntimeApplied(command, _)
                if step.command == "loadtrainer" && command.command == "loadtrainer"
                    && command.source_script == observed.source_script && command.command_index == observed.command_index
                    && command.args == [candidate.evidence.request.trainer_class.clone(), candidate.evidence.request.trainer_id.clone()])
        {
            return;
        }
        candidate.loadtrainer_witnessed = true;
    }
    if step.command_index == startbattle_command_index {
        if candidate.loadtrainer_witnessed && step.command == "startbattle" {
            if let RuntimeMutationResult::ScriptedTrainerBattleStarted(started) =
                &step.mutation.result
            {
                bind_visible_scripted_trainer_encounter(shell, candidate, started, &snapshot);
            }
        }
        return;
    }
    // One strictly forward cursor, within this same authored script. Branches
    // that leave it (including the already-defeated branch) discard the witness.
    let Some(next) = &step.next_cursor else {
        return;
    };
    if next.origin_map_name != observed.origin_map_name
        || next.source_script != observed.source_script
        || next.command_index <= observed.command_index
        || next.command_index > startbattle_command_index
        || (!candidate.loadtrainer_witnessed && next.command_index > loadtrainer_command_index)
    {
        return;
    }
    candidate.expected_step = next.clone();
    shell.battle_origin.scripted_trainer_candidate = Some(candidate);
}

fn bind_visible_scripted_trainer_encounter(
    shell: &mut BevyRuntimeShell,
    candidate: VisibleScriptedTrainerEncounterCandidate,
    started: &crate::TrainerBattleStartStatus,
    snapshot: &RuntimeShellSnapshot,
) {
    let crate::TrainerBattleStartStatus::Started(start) = started else {
        return;
    };
    let Some(battle) = snapshot.battle.as_ref() else {
        return;
    };
    let mut evidence = candidate.evidence;
    let crystal_render_api::VisualBattleTrainerProvenance::Scripted {
        loadtrainer_command_index,
        startbattle_command_index,
    } = evidence.provenance
    else {
        return;
    };
    if !candidate.faceplayer_witnessed
        || !candidate.dialogue_witnessed
        || !candidate.loadtrainer_witnessed
        || candidate.expected_step.command_index != startbattle_command_index
        || shell.battle_origin.published().is_some()
        || !trainer_candidate_matches_snapshot(&evidence, snapshot)
        || !trainer_evidence_matches_start(&evidence, start, battle)
    {
        return;
    }
    evidence.capture_attempted = true;
    stage_visible_battle_origin(
        shell,
        &evidence.source,
        BattleOriginKind::Trainer,
        &start.battle_type,
        Some(BattleOriginContact::ScriptedTrainerObjectTarget {
            map_id: evidence.source.map_name.clone(),
            tile: evidence.target_tile,
            object_identifier: evidence.target_identifier.clone(),
            trainer_id: evidence.request.trainer_id.clone(),
            source_script: evidence.object_script.clone(),
            loadtrainer_command_index,
            startbattle_command_index,
        }),
        None,
    );
    let origin = shell
        .battle_origin
        .pending
        .as_ref()
        .expect("scripted trainer origin just staged")
        .clone();
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("CRYSTAL_ENCOUNTER_TRACE").is_some() {
        eprintln!(
            "scripted trainer witness: generation={} map={} player={:?} facing={:?} trainer={} tile={:?} facing={:?} class={} id={} script={} loadtrainer={} startbattle={} pokemon={:?} anchored={}",
            origin.generation,
            evidence.source.map_name,
            evidence.source.tile,
            evidence.source.facing,
            evidence.target_identifier,
            evidence.target_tile,
            evidence.target_facing,
            evidence.request.trainer_class,
            evidence.request.trainer_id,
            evidence.object_script,
            loadtrainer_command_index,
            startbattle_command_index,
            evidence.placement.presentation_core_tile,
            evidence.anchors.is_some()
        );
    }
    shell.battle_origin.bound_trainer = Some(VisibleBoundTrainerEncounter {
        origin,
        candidate: evidence,
        publication: None,
    });
}

fn freeze_pending_visible_scripted_trainer(
    shell: &mut BevyRuntimeShell,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
    objects: &Query<&VisibleObjectSprite>,
) {
    let Some(mut candidate) = shell.battle_origin.scripted_trainer_candidate.take() else {
        return;
    };
    if !same_static_encounter_pose(
        &candidate.evidence.source,
        &shell.shell.session().snapshot(),
    ) || !shell.active_script_cursor.as_ref().is_some_and(|cursor| {
        cursor.origin_map_name == candidate.expected_step.origin_map_name
            && cursor.source_script == candidate.expected_step.source_script
            && cursor.next_command_index == candidate.expected_step.command_index
    }) {
        return;
    }
    if candidate.dialogue_witnessed && !candidate.evidence.capture_attempted {
        let Ok(snapshot) = shell.shell.presentation_snapshot() else {
            return;
        };
        if snapshot.battle.is_some()
            || !trainer_candidate_matches_snapshot(&candidate.evidence, &snapshot)
        {
            return;
        }
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
        freeze_visible_trainer_anchors(&mut candidate.evidence, rendered, frame, &roster);
    }
    shell.battle_origin.scripted_trainer_candidate = Some(candidate);
}

/// Disposable native QA setup. Ordinary A dispatches Falkner's full authored
/// dialogue and loadtrainer/startbattle; neither trainer nor script is moved.
#[cfg(any(test, feature = "location-tester"))]
fn prepare_falkner_encounter_preview(mut shell: BevyRuntimeShell) -> Result<BevyRuntimeShell> {
    anyhow::ensure!(
        shell.quick_save_path.is_none(),
        "Falkner preview cannot write a user save"
    );
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("CHRIS"))?;
    let initial = shell.shell.snapshot()?;
    anyhow::ensure!(
        initial.party.slots.is_empty()
            && initial.battle.is_none()
            && initial.overworld.map_name == "VioletGym"
            && initial.overworld.tile == TilePosition::new(4, 1),
        "Falkner preview requires a fresh empty-party Violet Gym session at (4,1)"
    );
    let runtime = shell.shell.runtime().clone();
    let data = runtime.data();
    let falkner = data
        .maps
        .get("VioletGym")
        .context("compiled Violet Gym")?
        .objects
        .iter()
        .find(|object| object.object_identifier.as_deref() == Some("VIOLETGYM_FALKNER"))
        .context("compiled Falkner object")?;
    let authored = data.scripted_trainer_battle("VioletGym", &falkner.script, 9)?;
    let trainer = data
        .trainers
        .get(&authored.request.trainer_id)
        .context("compiled Falkner party")?;
    anyhow::ensure!(
        falkner.object_type == "OBJECTTYPE_SCRIPT"
            && falkner.script == "VioletGymFalknerScript"
            && object_tile_position_checked(falkner) == Some(TilePosition::new(5, 1))
            && runtime.compiled_script_command_name(&falkner.script, 0)? == "faceplayer"
            && authored.loadtrainer_command_index == 8
            && authored.startbattle_command_index == 9
            && authored.request.trainer_class == "FALKNER"
            && authored.request.trainer_id == "FALKNER1"
            && authored.request.event_flag.is_empty()
            && trainer.party.len() == 2
            && trainer.party[0].species == "PIDGEY"
            && trainer.party[0].level == 7
            && trainer.party[1].species == "PIDGEOTTO"
            && trainer.party[1].level == 9,
        "Falkner preview requires the original scripted trainer and both original party members"
    );
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
        overworld.set_player_facing(Direction::Right);
        state.overworld = crate::core::state::OverworldMemory::from_snapshot(&overworld.snapshot());
    }
    reset_visible_navigation_state(&mut shell);
    mark_runtime_snapshot_dirty(&mut shell);
    settle_visible_shell_smoke_until_idle(&mut shell)?;
    let ready = shell.shell.snapshot()?;
    anyhow::ensure!(
        ready.battle.is_none()
            && ready.overworld.map_name == "VioletGym"
            && ready.overworld.tile == TilePosition::new(4, 1)
            && ready.overworld.facing == Direction::Right
            && ready.overworld.mode == MovementMode::Normal
            && shell.pending_trainer_intro.is_none()
            && shell.pending_trainer_sight.is_none()
            && !shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set("EVENT_BEAT_FALKNER")
                .map_err(|error| anyhow::anyhow!("check Falkner event: {error}"))?,
        "Falkner preview must await ordinary A before the unbeaten scripted object"
    );
    mark_runtime_snapshot_dirty(&mut shell);
    Ok(shell)
}
