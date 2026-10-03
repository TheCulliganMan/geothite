// Controller-owned provenance, independent of the live world frame/cache and
// of GameState, saves, replay payloads and deterministic gameplay checksums.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleOriginKind {
    Wild,
    StaticWild,
    Trainer,
}

impl BattleOriginKind {
    fn from_battle(battle: &crate::RuntimeBattleSnapshot) -> Self {
        match &battle.kind {
            crate::RuntimeBattleKind::Wild { .. } => Self::Wild,
            crate::RuntimeBattleKind::StaticWild { .. } => Self::StaticWild,
            crate::RuntimeBattleKind::Trainer { .. } => Self::Trainer,
        }
    }
}

/// A contact witnessed by an authoritative encounter result. Absence means
/// that this entry path has not supplied contact provenance; it does not mean
/// grass, dry ground, or the player's current tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleOriginContact {
    OverworldEncounter {
        map_id: String,
        tile: TilePosition,
        surface: crate::core::world::encounters::EncounterSurface,
    },
    FishingWaterTarget {
        map_id: String,
        tile: TilePosition,
    },
}

/// Visual interpolation at core commitment. This is not a world-space support
/// anchor: terrain height and the eventual landed pose belong to the renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleOriginVisualStep {
    pub walk_from: Option<TilePosition>,
    pub walk_ticks_remaining: u8,
    pub walk_total_ticks: u8,
    pub ledge_jump: Option<(TilePosition, TilePosition, u8)>,
}

/// One frozen source value, shared unchanged through entry, turns and retained
/// terminal narration. `source` carries the exact map ID, authoritative player
/// tile, facing, movement mode and source frame. `contact` is separate because
/// a fishing target is water while the player and core catch tile are on shore.
/// No field is an arena selection or proof that the source scene was rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattlePresentationOrigin {
    /// Presentation-only serial, scoped to this controller instance. Never use
    /// it as a gameplay, RNG, save or journal identity.
    pub generation: u64,
    pub source: crate::core::world::session::OverworldSnapshot,
    pub kind: BattleOriginKind,
    pub battle_type: String,
    pub contact: Option<BattleOriginContact>,
    pub authoritative_step: Option<StepOutcome>,
    pub visual_step: BattleOriginVisualStep,
}

/// Read-only publication for renderer consumers. Publishing clones only an Arc;
/// it does not read the live map, rebuild geometry, or select a cache key.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct BattlePresentationOriginFrame {
    origin: Option<Arc<BattlePresentationOrigin>>,
}

impl BattlePresentationOriginFrame {
    pub fn origin(&self) -> Option<&BattlePresentationOrigin> {
        self.origin.as_deref()
    }
}

#[derive(Debug, Default)]
struct VisibleBattleOriginState {
    generation: u64,
    pending: Option<Arc<BattlePresentationOrigin>>,
    active: Option<Arc<BattlePresentationOrigin>>,
}

impl VisibleBattleOriginState {
    fn published(&self) -> Option<&Arc<BattlePresentationOrigin>> {
        self.pending.as_ref().or(self.active.as_ref())
    }

    fn clear(&mut self) {
        self.pending = None;
        self.active = None;
        // Preserve the serial across reload so a renderer cannot alias a later
        // encounter with a scene it retained before the load.
    }

    fn stage(&mut self, mut origin: BattlePresentationOrigin) {
        if self.pending.is_some() {
            return;
        }
        self.generation = self.generation.wrapping_add(1);
        origin.generation = self.generation;
        self.pending = Some(Arc::new(origin));
    }
}

fn stage_visible_battle_origin(
    shell: &mut BevyRuntimeShell,
    source: &crate::core::world::session::OverworldSnapshot,
    kind: BattleOriginKind,
    battle_type: &str,
    contact: Option<BattleOriginContact>,
    authoritative_step: Option<StepOutcome>,
) {
    let origin = BattlePresentationOrigin {
        generation: 0, // assigned only by the presentation owner
        source: source.clone(),
        kind,
        battle_type: battle_type.to_string(),
        contact,
        authoritative_step,
        visual_step: BattleOriginVisualStep {
            walk_from: (shell.player_walk_frame_ticks > 0)
                .then_some(shell.player_walk_from)
                .flatten(),
            walk_ticks_remaining: shell.player_walk_frame_ticks,
            walk_total_ticks: shell.player_walk_total_ticks,
            ledge_jump: shell
                .visible_ledge_jump
                .map(|jump| (jump.from, jump.to, jump.frame)),
        },
    };
    shell.battle_origin.stage(origin);
}

fn capture_visible_wild_battle_origin(
    shell: &mut BevyRuntimeShell,
    frame: &crate::RuntimeOverworldFrame,
) {
    let Some(battle) = frame.wild_battle.as_ref() else {
        return;
    };
    // Use this exact committed result, not last_frame (which can be stale for
    // script/field entries) and not a map-name/environment water heuristic.
    stage_visible_battle_origin(
        shell,
        &frame.snapshot,
        BattleOriginKind::Wild,
        &battle.battle_type,
        Some(BattleOriginContact::OverworldEncounter {
            map_id: battle.encounter.map_name.clone(),
            tile: battle.encounter.tile,
            surface: battle.encounter.surface,
        }),
        frame.movement.clone(),
    );
}

fn take_visible_battle_origin_for_entry(
    shell: &mut BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    battle: &crate::RuntimeBattleSnapshot,
) -> Arc<BattlePresentationOrigin> {
    if let Some(origin) = shell.battle_origin.pending.take() {
        return origin;
    }
    if let Some(origin) = shell.battle_origin.active.take() {
        // Re-preparing the same battle must never follow a later live pose.
        return origin;
    }
    // Scripted/trainer/link/tower entries have exact source pose, but no checked
    // encounter contact supplied by this bounded patch. Keep that absence.
    stage_visible_battle_origin(
        shell,
        &snapshot.overworld,
        BattleOriginKind::from_battle(battle),
        &battle.battle_type,
        None,
        None,
    );
    shell
        .battle_origin
        .pending
        .take()
        .expect("origin just staged")
}

fn publish_visible_battle_origin(
    shell: Res<BevyRuntimeShell>,
    mut frame: ResMut<BattlePresentationOriginFrame>,
) {
    let next = shell.battle_origin.published().cloned();
    if frame.origin != next {
        frame.origin = next;
    }
}
