//! Shared game runtime for Geothite frontends.
//!
//! Owns pack loading, game commands, snapshots, saves, replay, and audio cue
//! resolution. No window, graphics engine, or audio device is required.

pub mod frontpic_animation;
pub mod hall_of_fame;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result};
#[cfg(test)]
use crystal_assets::RuntimeScriptedWildBattleTerminal;
use crystal_assets::modpack::{
    CompiledGamePack, CompiledGamePackIdentity, GameDataSet, LoadedCompiledGamePack,
    ModpackAudioKind, ModpackAudioManifest, ModpackAudioManifestEntry, ModpackAudioPlaybackEntry,
    ModpackAudioPlaybackPlan, ModpackAudioSource,
};
use crystal_assets::{
    ActiveBattleCommandOutcome, AssetRoot, BlackoutRecoveryOutcome, DecorationActionOutcome,
    DecorationCategory, DecorationDefinition, DecorationSide, OverworldInputFrame,
    PACK_AUDIO_COMPRESSION_GZIP, PACK_AUDIO_COMPRESSION_MIDI, PartyRecoveryOutcome,
    PokemonCryMetadata, RuntimeBadgeCommand, RuntimeBadgeRegion, RuntimeBagItemDeltaCommand,
    RuntimeBattleEnemyActionCommand, RuntimeBattleEscapeCommand, RuntimeBattleItemCommand,
    RuntimeBattleTowerActionCommand, RuntimeBattleTowerBattleCommand,
    RuntimeBattleTowerChallengeMenuCommand, RuntimeBattleTowerMobileFlag,
    RuntimeBattleTowerOpponentCommand, RuntimeBattleTowerRoomMenuCommand, RuntimeBattleTurnCommand,
    RuntimeBillsGrandfatherCommand, RuntimeBuenaPasswordCommand, RuntimeBuenaPrizeCommand,
    RuntimeBugContestAction, RuntimeBugContestCommand, RuntimeCableClubGenderCommand,
    RuntimeCableClubRequest, RuntimeCaptureCompletionCommand, RuntimeClockUpdateCommand,
    RuntimeComposeMailCommand, RuntimeCurrencyAccount, RuntimeCurrencyDeltaCommand,
    RuntimeDayCareAction, RuntimeDayCareCaretaker, RuntimeDayCareCommand,
    RuntimeDecorationPutAwayCommand, RuntimeDecorationSetupCommand, RuntimeDividerTrace,
    RuntimeElevatorFloorSelectionCommand, RuntimeFieldBlockMoveCommand, RuntimeFieldPartyCommand,
    RuntimeFishingCommand, RuntimeFishingItemCommand, RuntimeFishingSwarmCommand,
    RuntimeFlyCommand, RuntimeGameCornerCommand, RuntimeGameCornerService,
    RuntimeGameLogicPauseCommand, RuntimeGameTimerAdvanceCommand, RuntimeGameTimerCountingCommand,
    RuntimeGameTimerOutcome, RuntimeGiftPokemonCommand, RuntimeGiveDratiniCommand,
    RuntimeGraphicsSpecial, RuntimeHappinessServiceCommand, RuntimeHappinessServiceRoutine,
    RuntimeHeadbuttScriptCommand, RuntimeHeldItemCommand, RuntimeItemCommand,
    RuntimeKurtApricornCommand, RuntimeLinkBattleRecordCommand, RuntimeLinkBattleResult,
    RuntimeLinkFriendReadyCommand, RuntimeLinkRoomSelectionCommand, RuntimeLinkRoomSpecial,
    RuntimeLinkTimeoutCommand, RuntimeMagikarpLengthCommand, RuntimeMailboxPartyCommand,
    RuntimeMailboxSlotCommand, RuntimeManualClockCommand, RuntimeMapRadioCommand,
    RuntimeMobileHandshakeCommand, RuntimeMobileSelectThreeMonsCommand, RuntimeMoveDeletionCommand,
    RuntimeMoveLearnReplacementCommand, RuntimeMoveTutorCommand, RuntimeMutationCommand,
    RuntimeMutationOutcome, RuntimeMutationResult, RuntimeMysteryGiftAction,
    RuntimeNameRivalCommand, RuntimeOddEggCommand, RuntimeOptionsCommand,
    RuntimeOverworldInputCommand, RuntimePartyCheckCommand, RuntimePartyCheckSpecial,
    RuntimePartyHpTransferCommand, RuntimePartyHpTransferOutcome, RuntimePartyItemCommand,
    RuntimePartyMoveItemCommand, RuntimePartyMoveSwapCommand, RuntimePartyNicknameCommand,
    RuntimePartyPokemonCommand, RuntimePartyRecoverySetupCommand, RuntimePartyRecoverySetupOutcome,
    RuntimePartySlotCommand, RuntimePartySwapCommand, RuntimePcBagItemCheckCommand,
    RuntimePcBoxCommand, RuntimePcBoxNameCommand, RuntimePcDepositCommand, RuntimePcItemCommand,
    RuntimePcMoveCommand, RuntimePcReleaseCommand, RuntimePcWithdrawCommand,
    RuntimePendingScriptRequest, RuntimePendingScriptRequestCommand,
    RuntimePendingScriptRequestKind, RuntimePendingYesNoResolutionCommand,
    RuntimePhoneCallerCommand, RuntimePhoneRandomSpecial, RuntimePlayerGenderCommand,
    RuntimePlayerPaletteCommand, RuntimePokedexCommand, RuntimePokegearPhoneCallCommand,
    RuntimePokegearPhoneCallOutcome, RuntimePresentationInterpreter, RuntimePresentationProgram,
    RuntimePresentationSubprogramInterpreter, RuntimeRandomScriptCommand,
    RuntimeRandomScriptMapCommand, RuntimeRandomSpecialRoutineCommand,
    RuntimeRegisteredKeyItemCommand, RuntimeRegisteredKeyItemOutcome,
    RuntimeRememberPasswordCommand, RuntimeRockMonEncounterCommand, RuntimeScriptCommandRef,
    RuntimeScriptEventDrainCommand, RuntimeScriptEventDrainResult, RuntimeScriptEventQueue,
    RuntimeScriptRuntimeFlag, RuntimeScriptRuntimeFlagCommand, RuntimeScriptRuntimeFlagValue,
    RuntimeScriptRuntimeMemoryEntry, RuntimeScriptRuntimeMemoryEntryCommand,
    RuntimeScriptRuntimeMemoryEntryRemoved, RuntimeScriptRuntimeMemoryValue,
    RuntimeScriptRuntimeMemoryValueCommand, RuntimeScriptRuntimeMemoryValueTaken,
    RuntimeScriptRuntimeQueue, RuntimeScriptRuntimeQueueDrainCommand,
    RuntimeScriptRuntimeQueueDrainResult, RuntimeScriptedWildBattleCompletionCommand,
    RuntimeScriptedWildBattleStartCommand, RuntimeShopTransactionCommand, RuntimeShuckieAction,
    RuntimeShuckieCommand, RuntimeSpawnPoint, RuntimeSpecialCryCommand,
    RuntimeStaticWildBattleOrigin, RuntimeStoryGateSpecial,
    RuntimeSweetScentEncounterCommand, RuntimeTitleScreen, RuntimeTmHmCommand,
    RuntimeTrainerBattleCompletionCommand, RuntimeTrainerIdentityCommand,
    RuntimeTreeMonEncounterCommand, RuntimeVerticalMenuOpenCommand,
    RuntimeVerticalMenuSelectionCommand, TilesetDefinition, decode_runtime_mutation_command_frame,
    decode_runtime_mutation_command_payload, runtime_mutation_command_frame,
    runtime_mutation_result_frame as assets_runtime_mutation_result_frame,
    runtime_special_routine_requires_divider_trace, validate_compiled_audio_payload,
    validate_compiled_runtime_files,
};
use crystal_audio::{AudioKind, AudioPcmFormat, AudioProgram, AudioProgramSource};
use crystal_core::battle::capture::{CaptureCompletion, CaptureOutcome, StoredCapture};
use crystal_core::battle::capture::{CaptureRules, CaptureWobbleProbability};
use crystal_core::battle::damage::{TypeCategories, TypeEffectivenessTable, WeatherModifiers};
use crystal_core::battle::start::{
    LinkBattleStart, StaticWildBattleStart, TrainerBattleStartStatus, WildBattleStart,
    activate_link_battle_start, deactivate_battle_after_draw, deactivate_battle_after_loss,
    deactivate_battle_after_win, switch_active_battle_enemy_party_index,
};
use crystal_core::battle::stats::BattleStatMultiplierTables;
use crystal_core::battle::turn::{
    BattleAction, BattleEvent, BattleSide, BattleTurnOutcome, MovePriorityTable,
};
#[cfg(any(test, feature = "test-fixtures"))]
use crystal_core::input::{B_PAD_DOWN, B_PAD_RIGHT};
use crystal_core::input::{GameButton, JoypadState};
use crystal_core::models::{
    Dv, FrontpicAnimProgram, ITEM_POCKET_TM_HM, Item, LearnedMove, Move, PcBox, PokegearLandmark,
    PokegearLandmarksPayload, Pokemon, PokemonSpecies, RuntimePokedexEntry, Stat, Trainer,
};
use crystal_core::multiplayer::{
    DeterministicInputJournal, DeterministicInputJournalFrame, DeterministicReplayBundle,
    LinkHello, LinkMessage, LinkSessionIdentity, LinkTradeTransferMode, LockstepFrame,
    MenuChoiceFrame, MenuChoiceResultFrame, PlayerId, PlayerIdentity, RuntimeCommandFrame,
    RuntimeCommandResultFrame, SaveCheckpointFrame, SaveResumeReplayBundle,
    SessionRuntimeCommandFrame, SessionRuntimeCommandResultFrame, SessionSaveCheckpointFrame,
    StateChecksum, StateChecksumFrame, TradeOutcome, game_state_checksum,
    validate_link_session_identity,
};
#[cfg(test)]
use crystal_core::random::ReplayDivider;
use crystal_core::random::{
    CrystalRandom, DividerSource, LINK_BATTLE_RANDOM_SEED_COUNT, LinkBattleRandomState,
    RecordingDivider, RuntimeDividerSource,
};
use crystal_core::save::{
    SaveGameSummary, SaveModpackIdentity, SaveSlotSummary, list_save_game_summaries_for_modpack,
    read_save_game_for_modpack, read_save_game_summary_for_modpack, write_save_game_for_modpack,
};
#[cfg(any(test, feature = "test-fixtures"))]
use crystal_core::state::ScriptRuntimeElevatorFloor;
use crystal_core::state::{
    Badges, BattleMemory, DayCareInput, GameState, ItemUseRuntimeEvent, LinkSerialConnectionStatus,
    Options, OverworldMemory, PendingFieldTravel, PendingMoveLearn, SavedTrainerMetadata,
    ScriptControlRuntimeEvent, ScriptEndState, ScriptGraphicsRuntimeEvent, ScriptLocation,
    ScriptMapLoadRequest, ScriptMapRefreshRequest, ScriptMapRuntimeEvent, ScriptMoneyRuntimeEvent,
    ScriptMusicFade, ScriptReturnFrame, ScriptRuntimeDelay, ScriptRuntimeEarthquake,
    ScriptRuntimeEmote, ScriptRuntimeQueuedCommand, ScriptRuntimeStoneTableEntry, ScriptScreenFade,
    ScriptShopRequest, ScriptShopRuntimeEvent, ScriptTextRuntimeEvent, ScriptTextWait,
    ScriptWarpRequest, ScriptYesNoPrompt, is_engine_flag_name, saved_delay_command_payload,
    saved_earthquake_command_payload, saved_emote_command_payload, saved_map_load_command_payload,
    saved_map_refresh_command_payload, saved_music_fade_command_payload, saved_queued_command_args,
    saved_shop_event_command_payload, saved_shop_request_command_payload,
    validate_saved_trainer_metadata,
};
use crystal_core::systems::battle_escape::{BattleEscapeAttempt, BattleEscapeRules};
use crystal_core::systems::battle_items::{
    BattleItemEffectPlan, BattleItemOutcome, PartyItemOutcome, active_battle_item_effect_plan,
    battle_pp_item_effect_plan, party_special_item_effect_plan, party_wide_item_effect_plan,
};
use crystal_core::systems::battle_rewards::{
    BattleRewardOutcome, BattleRewardRules, PendingMoveLearnResolution,
};
use crystal_core::systems::economy::ScriptEconomyOutcome;
use crystal_core::systems::evolution::{
    EvolutionContext, EvolutionReport, LinkMode, check_and_evolve,
};
use crystal_core::systems::field_items::{
    FieldItemPickupOutcome, ItemfinderHiddenItem as CoreItemfinderHiddenItem,
};
use crystal_core::systems::field_moves::{
    FieldEscapeItemRule, FieldItemRule, FieldMoveBadgeRequirement, FieldMoveBlockOutcome,
    FieldMoveBlockRule, FieldMoveCatalog, FieldMoveFlagOutcome, FieldMoveFlagRule,
    FieldMoveMoveRule, FieldMoveReplacement, FieldMoveRule, FieldMoveTravelOutcome,
    FieldMoveTravelRule,
};
use crystal_core::systems::gift_pokemon::{GiftPokemonOutcome, GiftPokemonScript};
use crystal_core::systems::item_use::{ItemUseContext, ItemUseOutcome};
use crystal_core::systems::map_context::{SpawnMemoryUpdate, commit_overworld_snapshot};
use crystal_core::systems::phone::{ScriptPhoneInputs, ScriptPhoneOutcome};
use crystal_core::systems::script_audio::ScriptAudioCue;
use crystal_core::systems::script_blocks::ScriptBlockChangeOutcome;
use crystal_core::systems::script_control::ScriptControlAction;
use crystal_core::systems::script_flags::{ScriptFlagCheckOutcome, ScriptFlagMutationOutcome};
use crystal_core::systems::script_items::{
    ScriptItemCheckOutcome, ScriptItemGrantOutcome, ScriptItemTakeOutcome,
};
use crystal_core::systems::script_objects::{ScriptMovementOutcome, ScriptObjectMutationOutcome};
use crystal_core::systems::script_runtime::{
    ScriptRuntimeInputs, ScriptRuntimeOutcome, commit_interaction_script_dispatch,
    parse_menu_coord_token,
};
use crystal_core::systems::script_scenes::ScriptSceneOutcome;
use crystal_core::systems::script_swarms::ScriptSwarmOutcome;
use crystal_core::systems::script_text::{ScriptMenuDefinition, ScriptTextAction, ScriptTextBody};
use crystal_core::systems::script_variables::ScriptVariableOutcome;
use crystal_core::systems::script_warps::ScriptMapAction;
use crystal_core::systems::shop::{ScriptShopOutcome, ShopResult};
use crystal_core::systems::special_routines::{
    SpecialRoutineOutcome, saved_special_battle_type_builtin_routines,
};
use crystal_core::systems::step_events::StepEventResult;
use crystal_core::systems::time::{ClockTime, GameDate};
use crystal_core::systems::tmhm::TmHmLearnOutcome;
use crystal_core::world::encounters::{
    EncounterSlotTables, EncounterSurface, FieldEncounterData, TimeOfDay, WildEncounterData,
};
use crystal_core::world::fishing::FishingSession;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{LedgeJumpOutcome, MovementMode, StepOutcome};
use crystal_core::world::session::{
    ConnectionTransition, CoordEventTrigger, OverworldInteraction, OverworldInteractionTarget,
    OverworldSession, OverworldSnapshot, WarpTransition, WildEncounterRoll,
};

pub use crystal_assets as assets;
pub use crystal_audio as audio;
pub use crystal_core as core;
pub use crystal_net as net;

#[cfg(target_arch = "wasm32")]
static BROWSER_RUNTIME_FILES: OnceLock<BTreeMap<String, Vec<u8>>> = OnceLock::new();

/// Read a pack asset through the native or installed browser asset store.
pub fn read_runtime_asset(path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    let path = path.as_ref();
    #[cfg(target_arch = "wasm32")]
    if let Some(files) = BROWSER_RUNTIME_FILES.get() {
        let key = browser_runtime_asset_key(path);
        return files.get(&key).cloned().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("embedded runtime asset {key}"),
            )
        });
    }
    std::fs::read(path)
}

/// Check whether a presentation asset exists in the current host store.
pub fn runtime_asset_exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    #[cfg(target_arch = "wasm32")]
    if let Some(files) = BROWSER_RUNTIME_FILES.get() {
        return files.contains_key(&browser_runtime_asset_key(path));
    }
    path.is_file()
}

#[cfg(target_arch = "wasm32")]
fn browser_runtime_asset_key(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    text.split("apps/web/assets/")
        .nth(1)
        .map(str::to_owned)
        .or_else(|| {
            text.split("vendor/")
                .nth(1)
                .map(|value| format!("vendor/{value}"))
        })
        .unwrap_or_else(|| text.trim_start_matches("./").to_owned())
}

/// Read a UTF-8 presentation asset through the current host store.
pub fn read_runtime_asset_to_string(path: impl AsRef<Path>) -> std::io::Result<String> {
    String::from_utf8(read_runtime_asset(path)?)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

/// Decode a PNG presentation asset without creating graphics resources.
pub fn open_runtime_image(
    path: impl AsRef<Path>,
) -> image::ImageResult<image::DynamicImage> {
    let path = path.as_ref();
    #[cfg(target_arch = "wasm32")]
    {
        let bytes = read_runtime_asset(path).map_err(image::ImageError::IoError)?;
        return image::load_from_memory(&bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    image::open(path)
}

const BATTLE_MOVE_SLOTS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameViewport {
    pub width: u32,
    pub height: u32,
    pub scale: u32,
}

impl Default for GameViewport {
    fn default() -> Self {
        Self {
            width: 160,
            height: 144,
            scale: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeAudioCatalog {
    manifest: ModpackAudioManifest,
    playback: ModpackAudioPlaybackPlan,
    music: BTreeMap<String, AudioProgram>,
    sound_effects: BTreeMap<String, AudioProgram>,
    sound_effect_priorities: BTreeMap<String, u8>,
    cries: BTreeMap<String, AudioProgram>,
}

impl RuntimeAudioCatalog {
    pub fn is_empty(&self) -> bool {
        self.music.is_empty() && self.sound_effects.is_empty() && self.cries.is_empty()
    }

    pub fn manifest(&self) -> &ModpackAudioManifest {
        &self.manifest
    }

    pub fn playback(&self) -> &ModpackAudioPlaybackPlan {
        &self.playback
    }

    pub fn music(&self) -> &BTreeMap<String, AudioProgram> {
        &self.music
    }

    pub fn sound_effects(&self) -> &BTreeMap<String, AudioProgram> {
        &self.sound_effects
    }

    pub fn sound_effect_priorities(&self) -> &BTreeMap<String, u8> {
        &self.sound_effect_priorities
    }

    pub fn cries(&self) -> &BTreeMap<String, AudioProgram> {
        &self.cries
    }

    pub fn music_count(&self) -> usize {
        self.music.len()
    }

    pub fn sound_effect_count(&self) -> usize {
        self.sound_effects.len()
    }

    pub fn cry_count(&self) -> usize {
        self.cries.len()
    }

    pub fn contains_music(&self, id: &str) -> bool {
        self.music.contains_key(id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrystalRuntime {
    modpack: SaveModpackIdentity,
    pack_identity: CompiledGamePackIdentity,
    data: GameDataSet,
    runtime_files: BTreeMap<String, Vec<u8>>,
    runtime_asset_mount: Arc<RuntimeAssetMountCache>,
    audio: RuntimeAudioCatalog,
    viewport: GameViewport,
    /// Immutable pack map catalogs shared by presentation snapshots. Runtime
    /// snapshots clone only the active/overridden map instead of deep-cloning
    /// every map, scene, event, object, and block table each movement frame.
    map_catalog: Vec<Arc<RuntimeMapCatalogSnapshot>>,
    catalog_cache: Arc<OnceLock<RuntimeStaticCatalogCache>>,
}

/// Shared by runtime clones, but excluded from their semantic equality.
/// Materializing presentation files must not change a runtime's identity.
#[derive(Debug, Default)]
struct RuntimeAssetMountCache {
    root: OnceLock<AssetRoot>,
}

impl PartialEq for RuntimeAssetMountCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeStaticCatalogCache {
    audio: Arc<RuntimeAudioCatalogSnapshot>,
    items: Arc<Vec<RuntimeItemCatalogSnapshot>>,
    item_effect_plans: Arc<Vec<RuntimeItemEffectPlanKey>>,
    moves: Arc<Vec<RuntimeMoveCatalogSnapshot>>,
    pokemon: Arc<Vec<RuntimePokemonCatalogSnapshot>>,
    trainers: Arc<Vec<RuntimeTrainerCatalogSnapshot>>,
    spawn_points: Arc<Vec<RuntimeSpawnPoint>>,
    tilesets: Arc<Vec<RuntimeTilesetCatalogSnapshot>>,
    encounters: Arc<RuntimeEncounterCatalogSnapshot>,
    battle_rules: Arc<RuntimeBattleRuleCatalogSnapshot>,
    world_rules: Arc<RuntimeWorldRuleCatalogSnapshot>,
    presentation: Arc<RuntimePresentationCatalogSnapshot>,
    special: Arc<RuntimeSpecialCatalogSnapshot>,
    story: Arc<RuntimeStoryCatalogSnapshot>,
    playability: Arc<crystal_assets::PlayabilityRules>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBootSummary {
    pub modpack_id: String,
    pub modpack_hash: String,
    pub pack_content_hash: String,
    pub pokemon_species: usize,
    pub moves: usize,
    pub maps: usize,
    pub items: usize,
    pub wild_encounter_tables: usize,
    pub music_tracks: usize,
    pub sound_effects: usize,
    pub cries: usize,
    pub viewport: GameViewport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOverworldSession {
    state: GameState,
    overworld: OverworldSession,
    joypad: JoypadState,
    divider: RuntimeDividerSource,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeGameShell {
    asset_root: AssetRoot,
    runtime: CrystalRuntime,
    session: RuntimeOverworldSession,
    last_frame: Option<RuntimeOverworldFrame>,
    linked_menu_results: Vec<MenuChoiceResultFrame>,
    runtime_command_sequence: u64,
    runtime_commands: Vec<RuntimeCommandFrame>,
    runtime_results: Vec<RuntimeCommandResultFrame>,
    /// Retained command frames are useful for deterministic link/replay
    /// diagnostics, but serializing every idle input frame is not gameplay.
    /// Bevy disables this for its live shell; the public runtime keeps the
    /// historical default enabled for replay callers.
    retain_runtime_journal: bool,
}

struct RecordedRuntimeMutation {
    command: RuntimeMutationCommand,
    state: GameState,
    overworld: OverworldSession,
    outcome: RuntimeMutationOutcome,
    divider_after: Option<RuntimeDividerSource>,
}

const RUNTIME_LOCAL_PLAYER_ID: PlayerId = 1;

fn reject_unexpected_gift_pokemon_inputs(
    source_script: &str,
    command_index: usize,
    command: &str,
    inputs: &ScriptRuntimeInputs,
) -> Result<()> {
    if inputs.gift_original_trainer_name.is_some()
        || inputs.gift_original_trainer_id.is_some()
        || inputs.gift_nickname_accepted.is_some()
        || inputs.gift_nickname.is_some()
    {
        anyhow::bail!(
            "compiled script command {}:{} '{}' must not declare gift Pokemon input fields",
            source_script,
            command_index,
            command
        );
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeShellSnapshot {
    pub boot: RuntimeBootSummary,
    pub overworld: OverworldSnapshot,
    pub overworld_player_hidden: bool,
    pub visible_objects: Vec<crystal_core::map::ObjectEvent>,
    pub visible_object_slots: Vec<usize>,
    pub visible_object_runtime_tiles: BTreeMap<String, TilePosition>,
    pub visible_object_facings: BTreeMap<String, Direction>,
    pub state_checksum: StateChecksum,
    /// Checksum of the render-relevant game state with the monotonically
    /// advancing frame counter removed.  The authoritative checksum must
    /// change every frame; using it to decide whether to rebuild the viewport
    /// made idle gameplay recreate every tile and NPC at 60 Hz.
    pub visual_state_hash: u32,
    pub phase: RuntimeShellPhase,
    pub trainer: RuntimeTrainerSnapshot,
    pub progression: RuntimeProgressionSnapshot,
    pub roaming_pokemon: [crystal_core::state::RoamingPokemonState; 3],
    pub map_name_sign: crystal_core::state::MapNameSignMemory,
    pub day_care: crystal_core::state::DayCareState,
    pub bug_contest: crystal_core::state::BugContestState,
    pub magikarp_record: crystal_core::state::MagikarpRecordState,
    pub buenas_password: crystal_core::state::BuenasPasswordState,
    pub mystery_gift: RuntimeMysteryGiftSnapshot,
    pub link_session: RuntimeLinkSessionSnapshot,
    pub battle_tower: crystal_core::state::BattleTowerState,
    pub mobile_link: crystal_core::state::MobileLinkState,
    pub audio: RuntimeShellAudioState,
    pub audio_catalog: Arc<RuntimeAudioCatalogSnapshot>,
    pub menu: Option<RuntimeMenuSnapshot>,
    pub ui: RuntimeUiSnapshot,
    pub battle: Option<RuntimeBattleSnapshot>,
    pub pending_move_learn: Option<RuntimePendingMoveLearnSnapshot>,
    pub party: RuntimePartySnapshot,
    pub storage: RuntimeStorageSnapshot,
    pub mailbox: Vec<crystal_core::state::MailboxMail>,
    pub bag: RuntimeBagSnapshot,
    pub items: Arc<Vec<RuntimeItemCatalogSnapshot>>,
    pub item_effect_plans: Arc<Vec<RuntimeItemEffectPlanKey>>,
    pub moves: Arc<Vec<RuntimeMoveCatalogSnapshot>>,
    pub pokemon: Arc<Vec<RuntimePokemonCatalogSnapshot>>,
    pub trainers: Arc<Vec<RuntimeTrainerCatalogSnapshot>>,
    pub maps: Vec<Arc<RuntimeMapCatalogSnapshot>>,
    pub spawn_points: Arc<Vec<RuntimeSpawnPoint>>,
    pub tilesets: Arc<Vec<RuntimeTilesetCatalogSnapshot>>,
    pub encounters: Arc<RuntimeEncounterCatalogSnapshot>,
    pub battle_rules: Arc<RuntimeBattleRuleCatalogSnapshot>,
    pub world_rules: Arc<RuntimeWorldRuleCatalogSnapshot>,
    pub presentation: Arc<RuntimePresentationCatalogSnapshot>,
    pub special: Arc<RuntimeSpecialCatalogSnapshot>,
    pub story: Arc<RuntimeStoryCatalogSnapshot>,
    pub playability: Arc<crystal_assets::PlayabilityRules>,
    pub script_events: RuntimeScriptEventsSnapshot,
    pub pending_shop: Option<ScriptShopRequest>,
    pub linked_menu_results: Vec<MenuChoiceResultFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMysteryGiftSnapshot {
    pub unlocked: bool,
    pub stored_item: Option<String>,
    pub backup_item: Option<String>,
    pub trainer_house_flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLinkSessionSnapshot {
    pub link_mode: u8,
    pub player_link_action: u8,
    pub chosen_cable_club_room: u8,
    pub other_player_link_mode: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLinkSessionDescriptor {
    pub session: LinkSessionIdentity,
    pub local_player: PlayerIdentity,
    pub hello: LinkHello,
    pub checksum: StateChecksumFrame,
    pub save_checkpoint: SessionSaveCheckpointFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInputJournal {
    pub journal: DeterministicInputJournal,
    pub terminal_checksum: StateChecksumFrame,
}

impl RuntimeInputJournal {
    pub fn fingerprint(&self) -> Result<u32> {
        self.journal
            .fingerprint()
            .context("fingerprint runtime input journal")
    }

    pub fn fingerprint_hex(&self) -> Result<String> {
        self.journal
            .fingerprint_hex()
            .context("fingerprint runtime input journal")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeShellAudioState {
    pub current_music: Option<String>,
    pub queued_events: Vec<crystal_core::state::ScriptAudioRuntimeEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLinkTradeApply {
    pub sent: Pokemon,
    pub received_party_index: usize,
    pub evolution: EvolutionReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeAudioEventDrain {
    pub events: Vec<crystal_core::state::ScriptAudioRuntimeEvent>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeResolvedAudioEventDrain {
    pub events: Vec<RuntimeResolvedAudioPlayback>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeResolvedAudioPlayback {
    pub event: crystal_core::state::ScriptAudioRuntimeEvent,
    pub kind: RuntimeResolvedAudioPlaybackKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeResolvedAudioPlaybackKind {
    StopMusic {
        audio_id: String,
    },
    Play {
        audio_id: String,
        playback: ModpackAudioPlaybackEntry,
    },
    FadeMusic {
        audio_id: String,
        fade_frames: u16,
    },
    WaitForSoundEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeAudioCatalogSnapshot {
    pub manifest: ModpackAudioManifest,
    pub playback: ModpackAudioPlaybackPlan,
    pub music: BTreeMap<String, RuntimeAudioProgramSnapshot>,
    pub sound_effects: BTreeMap<String, RuntimeAudioProgramSnapshot>,
    pub cries: BTreeMap<String, RuntimeAudioProgramSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeAudioProgramSnapshot {
    pub cache_key: String,
    pub source: RuntimeAudioProgramSourceSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeAudioProgramSourceSnapshot {
    Pcm {
        byte_len: usize,
        format: AudioPcmFormat,
        loop_start_sample: Option<usize>,
        loop_end_sample: Option<usize>,
    },
    PcmGzip {
        byte_len: usize,
        format: AudioPcmFormat,
        loop_start_sample: Option<usize>,
        loop_end_sample: Option<usize>,
    },
    Midi {
        midi_base64_len: usize,
        byte_len: usize,
        format: AudioPcmFormat,
        loop_start_sample: Option<usize>,
        loop_end_sample: Option<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTrainerSnapshot {
    pub player_name: String,
    pub player_id: u16,
    pub player_gender: u8,
    pub player_palette_id: u8,
    pub money: u32,
    pub moms_money: u32,
    pub coins: u16,
    pub blue_card_balance: u16,
    pub current_pc_box: usize,
    pub options: Options,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeProgressionSnapshot {
    pub backup_warp_map_name: Option<String>,
    pub badges: Badges,
    pub pokedex_seen: usize,
    pub pokedex_owned: usize,
    pub pokedex_seen_species: BTreeSet<String>,
    pub pokedex_caught_species: BTreeSet<String>,
    pub link_wins: u16,
    pub link_losses: u16,
    pub link_draws: u16,
    pub pending_special_battle_type: Option<String>,
    pub repel_steps_remaining: u16,
    pub active_repel_item: Option<String>,
    pub registered_key_item: Option<String>,
    pub radio_tuning_knob: u8,
    pub last_spawn_map_constant: Option<String>,
    pub hall_of_fame: crystal_core::state::HallOfFameState,
    pub time: crystal_core::systems::time::TimeState,
    pub active_event_flags: BTreeSet<String>,
    pub active_engine_flags: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptEventsSnapshot {
    pub script_value: Option<String>,
    pub variables: BTreeMap<String, String>,
    pub memory: BTreeMap<String, String>,
    pub named_buffers: BTreeMap<String, String>,
    pub variable_sprites: BTreeMap<String, String>,
    pub phone_numbers: BTreeSet<String>,
    pub phone_number_order: Vec<Option<String>>,
    pub last_special_routine: Option<String>,
    pub last_talked_object: Option<String>,
    pub active_menu: Option<String>,
    pub pending_delays: Vec<ScriptRuntimeDelay>,
    pub pending_earthquakes: Vec<ScriptRuntimeEarthquake>,
    pub pending_emotes: Vec<ScriptRuntimeEmote>,
    pub command_queue: Vec<ScriptRuntimeQueuedCommand>,
    pub call_stack: Vec<ScriptReturnFrame>,
    pub stone_table_entries: Vec<ScriptRuntimeStoneTableEntry>,
    pub special_phone_call: Option<String>,
    pub audio_events: Vec<crystal_core::state::ScriptAudioRuntimeEvent>,
    pub pending_music_fade: Option<ScriptMusicFade>,
    pub waiting_for_sound_effect: bool,
    pub map_music_restart_disabled: bool,
    pub map_music_requested: bool,
    pub graphics_events: Vec<ScriptGraphicsRuntimeEvent>,
    pub pending_screen_fade: Option<ScriptScreenFade>,
    pub money_events: Vec<ScriptMoneyRuntimeEvent>,
    pub map_events: Vec<ScriptMapRuntimeEvent>,
    pub pending_script_warp: Option<ScriptWarpRequest>,
    pub pending_map_load: Option<ScriptMapLoadRequest>,
    pub pending_map_refresh: Option<ScriptMapRefreshRequest>,
    pub text_events: Vec<ScriptTextRuntimeEvent>,
    pub window_open: bool,
    pub active_pokemon_picture: Option<String>,
    pub text_window_open: bool,
    pub active_text_label: Option<String>,
    pub pending_text_label: Option<String>,
    pub pending_text_wait: Option<ScriptTextWait>,
    pub pending_yes_no: Option<ScriptYesNoPrompt>,
    pub control_events: Vec<ScriptControlRuntimeEvent>,
    pub next_script: Option<ScriptLocation>,
    pub deferred_scripts: Vec<ScriptLocation>,
    pub map_reentry_script: Option<ScriptLocation>,
    pub script_ended: Option<ScriptEndState>,
    pub player_input_locked: bool,
    pub all_input_locked: bool,
    pub script_stop_requested: bool,
    pub item_notify_queued: bool,
    pub warp_sound_queued: bool,
    pub teleport_from_queued: bool,
    pub hall_of_fame_requested: bool,
    pub credits_requested: bool,
    pub reset_requested: bool,
    pub menu_2d_requested: bool,
    pub completed_trades: Vec<String>,
    pub shop_events: Vec<ScriptShopRuntimeEvent>,
    pub item_use_events: Vec<ItemUseRuntimeEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMenuClose {
    pub menu: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWindowClose {
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTextWindowClose {
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTextWaitAdvance {
    pub wait: ScriptTextWait,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeYesNoResolution {
    pub prompt: ScriptYesNoPrompt,
    pub accepted: bool,
    pub script_value: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeVerticalMenuOpen {
    pub map_name: String,
    pub menu_key: String,
    pub menu_id: String,
    pub source_script: String,
    pub loadmenu_command_index: usize,
    pub verticalmenu_command_index: usize,
    pub options: Vec<String>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeVerticalMenuOptionSelection {
    pub menu_id: String,
    pub source_script: String,
    pub verticalmenu_command_index: usize,
    pub option_index: usize,
    pub option: String,
    pub script_value: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeElevatorFloorSelection {
    pub map_name: String,
    pub data_label: String,
    pub source_script: String,
    pub elevator_command_index: usize,
    pub floor_index: usize,
    pub floor: String,
    pub warp: u16,
    pub target_map: String,
    pub destination_tile: crystal_core::world::map::TilePosition,
    pub script_value: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLinkedMenuChoice {
    pub frame: MenuChoiceFrame,
    pub selection: RuntimeVerticalMenuOptionSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePokemonPictureClose {
    pub species_id: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeShopClose {
    pub shop: ScriptShopRequest,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMenuSnapshot {
    pub menu_id: String,
    pub source: RuntimeMenuSource,
    pub definition: Option<ScriptMenuDefinition>,
    pub layout: RuntimeMenuLayoutSnapshot,
    pub window_open: bool,
    pub coords: Option<[i16; 4]>,
    pub menu_2d_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMenuLayoutSnapshot {
    pub declared_coords: Option<[i16; 4]>,
    pub data_commands: Vec<RuntimeMenuDataCommandSnapshot>,
    pub vertical_menus: Vec<RuntimeVerticalMenuSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMenuDataCommandSnapshot {
    pub command: String,
    pub args: Vec<String>,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeVerticalMenuSnapshot {
    pub source_script: String,
    pub loadmenu_command_index: usize,
    pub verticalmenu_command_index: usize,
    pub header_label: String,
    pub data_label: Option<String>,
    pub options: Vec<String>,
    pub two_dimensional: bool,
    pub rows: Option<usize>,
    pub columns: Option<usize>,
    pub spacing: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeElevatorSnapshot {
    pub map_name: String,
    pub source_script: String,
    pub elevator_command_index: usize,
    pub data_label: String,
    pub floors: Vec<RuntimeElevatorFloorSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeElevatorFloorSnapshot {
    pub floor_index: usize,
    pub floor: String,
    pub warp: u16,
    pub target_map: String,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeGiftPokemonSnapshot {
    pub map_name: String,
    pub source_script: String,
    pub command_index: usize,
    pub species_id: String,
    pub level: u8,
    pub level_token: String,
    pub held_item_id: Option<String>,
    pub nickname_label: Option<String>,
    pub ot_label: Option<String>,
    pub egg: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeUiSnapshot {
    pub menu: Option<RuntimeMenuSnapshot>,
    pub elevators: Vec<RuntimeElevatorSnapshot>,
    pub gift_pokemon: Vec<RuntimeGiftPokemonSnapshot>,
    pub text: Option<RuntimeTextSnapshot>,
    pub window_open: bool,
    pub text_window_open: bool,
    pub coords: Option<[i16; 4]>,
    pub active_pokemon_picture: Option<String>,
    pub pending_yes_no: Option<ScriptYesNoPrompt>,
    pub pending_text_wait: Option<ScriptTextWait>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTextSnapshot {
    pub label: String,
    pub source: RuntimeTextSource,
    pub asm_text: Option<String>,
    pub body: Option<ScriptTextBody>,
    pub queued_text_events: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeTextSource {
    AsmText,
    ScriptBody { map_name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeMenuSource {
    ScriptDefinition { map_name: String },
    SpecialRoutine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleSnapshot {
    pub kind: RuntimeBattleKind,
    pub battle_music: String,
    pub battle_type: String,
    pub enemy_pokemon: Pokemon,
    pub enemy_party: Vec<Pokemon>,
    pub active_player_party_index: Option<usize>,
    pub active_enemy_party_index: Option<usize>,
    pub player_spikes_zero_hp_unchecked: bool,
    pub enemy_spikes_zero_hp_unchecked: bool,
    pub player_transformed_species: Option<String>,
    pub enemy_transformed_species: Option<String>,
    pub player_transformed_dvs: Option<Dv>,
    pub enemy_transformed_dvs: Option<Dv>,
    pub player_substitute_hp: u16,
    pub enemy_substitute_hp: u16,
    pub player_semi_invulnerable: bool,
    pub enemy_semi_invulnerable: bool,
    pub player_moves: Vec<LearnedMove>,
    pub enemy_moves: Vec<LearnedMove>,
    pub player_last_move: Option<String>,
    pub player_used_moves: Vec<String>,
    pub enemy_last_move: Option<String>,
    pub enemy_toxic_turns: u8,
    pub player_turns_taken: u8,
    pub enemy_turns_taken: u8,
    pub enemy_switch_locked: bool,
    pub player_cannot_escape: bool,
    pub player_wrapped: bool,
    pub enemy_wrapped: bool,
    pub rewarded_enemy_party_indices: Vec<usize>,
    pub escape_attempts: u8,
    pub player_mist_active: bool,
    pub pay_day_money: u32,
    pub amulet_coin_active: bool,
    pub trainer_items_used: BTreeSet<String>,
    pub player_disabled_move: Option<String>,
    pub commands: RuntimeBattleCommandSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptedTrainerBattleKey {
    pub map_name: String,
    pub source_script: String,
    pub loadtrainer_command_index: usize,
    pub startbattle_command_index: usize,
    pub battle_type: String,
    pub trainer_class: String,
    pub trainer_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeWildEncounterOriginKey {
    pub map_name: String,
    pub species: String,
    pub level: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptCommandKey {
    pub script_label: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptCommandPayloadKey {
    pub script_label: String,
    pub command_index: usize,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptReturnKey {
    pub script_label: String,
    pub next_command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeWarpKey {
    pub map_name: String,
    pub warp_index: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeMapObjectKey {
    pub map_name: String,
    pub object_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeMapSceneKey {
    pub map_name: String,
    pub scene_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeMapMetadataKey {
    pub map_name: String,
    pub map_id: String,
    pub tileset_name: String,
    pub border_block: u8,
    pub width: u16,
    pub height: u16,
    pub time_of_day: Option<String>,
    pub phone_service: u8,
    pub phone_flag: bool,
    pub environment: Option<String>,
    pub location: Option<String>,
    pub music: Option<String>,
    pub palette: Option<String>,
    pub fishing_group: Option<String>,
    pub map_constant: Option<String>,
    pub map_group_constant: Option<String>,
    pub metadata_constant: Option<String>,
    pub metadata_group_name: Option<String>,
    pub metadata_group_id: Option<u16>,
    pub metadata_map_id: Option<u16>,
    pub metadata_environment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeTilesetKey {
    pub tileset_id: String,
    pub collision: BTreeMap<String, Vec<String>>,
    pub palette_map: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeAudioAssetKey {
    pub audio_id: String,
    pub kind: String,
    pub source: String,
    pub path: String,
    pub byte_len: usize,
    pub payload_hash: String,
    pub pcm_sample_rate_hz: Option<u32>,
    pub pcm_channels: Option<u8>,
    pub pcm_bits_per_sample: Option<u8>,
    pub pcm_frame_count: Option<usize>,
    pub cache_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeMartKey {
    pub mart_id: String,
    pub item_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeFruitTreeKey {
    pub fruit_tree_id: String,
    pub item_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeFieldMoveReplacementKey {
    pub replacement_block_id: u16,
    pub variant: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeFieldMoveRuleKey {
    pub rule_id: String,
    pub rule_kind: String,
    pub move_id: Option<String>,
    pub item_id: Option<String>,
    pub badge_region: Option<String>,
    pub badge_index: Option<usize>,
    pub engine_flag: Option<String>,
    pub escape_rope_mode: Option<String>,
    pub target_collisions: Vec<u8>,
    pub blocked_collisions: Vec<u8>,
    pub replacements: BTreeMap<String, BTreeMap<u16, RuntimeFieldMoveReplacementKey>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeFlyDestinationKey {
    pub flypoint_flag: String,
    pub destination_spawn_identifier: u16,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimePokemonCryKey {
    pub species_id: String,
    pub cry_id: String,
    pub pitch: i16,
    pub length: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimePcStringKey {
    pub string_id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeMenuIconKey {
    pub species_id: String,
    pub icon_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimePokedexEntryKey {
    pub species_id: String,
    pub species: String,
    pub classification: String,
    pub height_digits: u16,
    pub weight_digits: u16,
    pub pages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimePokegearLandmarkKey {
    pub landmark_id: u16,
    pub constant: String,
    pub label: String,
    pub name: String,
    pub x: i16,
    pub y: i16,
    pub region: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimePokegearMapLandmarkKey {
    pub map_name: String,
    pub landmark_constant: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptVerticalMenuKey {
    pub map_name: String,
    pub menu_key: String,
    pub source_script: String,
    pub loadmenu_command_index: usize,
    pub verticalmenu_command_index: usize,
    pub header_label: String,
    pub data_label: Option<String>,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptTextBodyCommandKey {
    pub command: String,
    pub args: Vec<String>,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptTextBodyKey {
    pub map_name: String,
    pub body_key: String,
    pub label: String,
    pub commands: Vec<RuntimeScriptTextBodyCommandKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptMenuCommandKey {
    pub command: String,
    pub args: Vec<String>,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptMenuDefinitionKey {
    pub map_name: String,
    pub menu_key: String,
    pub label: String,
    pub commands: Vec<RuntimeScriptMenuCommandKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptElevatorFloorKey {
    pub floor: String,
    pub warp: u16,
    pub target_map: String,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptElevatorKey {
    pub map_name: String,
    pub elevator_key: String,
    pub source_script: String,
    pub elevator_command_index: usize,
    pub data_label: String,
    pub floors: Vec<RuntimeScriptElevatorFloorKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeGiftPokemonKey {
    pub map_name: String,
    pub species_id: String,
    pub level_token: String,
    pub level: u8,
    pub held_item_id: Option<String>,
    pub nickname_label: Option<String>,
    pub ot_label: Option<String>,
    pub source_script: String,
    pub command_index: usize,
    pub egg: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptObjectCommandKey {
    pub map_name: String,
    pub command: String,
    pub object_id: Option<String>,
    pub target_object_id: Option<String>,
    pub x: Option<u16>,
    pub y: Option<u16>,
    pub direction: Option<String>,
    pub movement: Option<String>,
    pub emote: Option<String>,
    pub duration: Option<u16>,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptMovementStepKey {
    pub command: String,
    pub direction: Option<String>,
    pub duration: Option<u16>,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptMovementKey {
    pub map_name: String,
    pub label: String,
    pub source_script: Option<String>,
    pub steps: Vec<RuntimeScriptMovementStepKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeMapScriptSectionCommandKey {
    pub map_name: String,
    pub command: String,
    pub args: Vec<String>,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeMapEventSectionCommandKey {
    pub map_name: String,
    pub command: String,
    pub args: Vec<String>,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptMapCommandKey {
    pub map_name: String,
    pub command: String,
    pub target_map: Option<String>,
    pub x: Option<u16>,
    pub y: Option<u16>,
    pub facing: Option<String>,
    pub map_setup: Option<String>,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptVariableCommandKey {
    pub map_name: String,
    pub command: String,
    pub target: Option<String>,
    pub value_tokens: Vec<String>,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptControlCommandKey {
    pub map_name: String,
    pub command: String,
    pub compare_value: Option<String>,
    pub target_label: Option<String>,
    pub resolved_target_script: Option<String>,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptFieldPickupKey {
    pub map_name: String,
    pub command: String,
    pub item_id: Option<String>,
    pub quantity: u16,
    pub event_flag: Option<String>,
    pub fruit_tree_id: Option<String>,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptShopCommandKey {
    pub map_name: String,
    pub command: String,
    pub mart_type: String,
    pub mart_id: String,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptPhoneCommandKey {
    pub map_name: String,
    pub command: String,
    pub contact_id: String,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptSwarmCommandKey {
    pub map_name: String,
    pub command: String,
    pub swarm_token: String,
    pub map_id: String,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptRuntimeCommandKey {
    pub map_name: String,
    pub command: String,
    pub args: Vec<String>,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptItemGrantKey {
    pub map_name: String,
    pub command: String,
    pub item_id: String,
    pub quantity: u16,
    pub source_script: String,
    pub command_index: usize,
    pub verbose: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptItemAccessKey {
    pub map_name: String,
    pub command: String,
    pub item_id: String,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptEconomyCommandKey {
    pub map_name: String,
    pub command: String,
    pub account: Option<String>,
    pub amount_tokens: Vec<String>,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptFlagCommandKey {
    pub map_name: String,
    pub command: String,
    pub flag_id: String,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptSceneCommandKey {
    pub map_name: String,
    pub command: String,
    pub map_id: Option<String>,
    pub scene_id: Option<String>,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptBlockChangeKey {
    pub map_name: String,
    pub x: u16,
    pub y: u16,
    pub block_id: u16,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptAudioCommandKey {
    pub map_name: String,
    pub command: String,
    pub audio_id: Option<String>,
    pub fade_frames: Option<u16>,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeScriptTextCommandKey {
    pub map_name: String,
    pub command: String,
    pub text_label: Option<String>,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeCaptureBallRuleKey {
    pub ball_id: String,
    pub multiplier_numerator: u16,
    pub multiplier_denominator: u16,
    pub battle_type: String,
    pub skip_hp_calc: bool,
    pub use_heavy_ball_weight_modifier: bool,
    pub use_level_ball_multiplier: bool,
    pub require_same_species: bool,
    pub require_same_gender: bool,
    pub require_fast_species: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeHeavyBallModifierKey {
    pub species_id: String,
    pub modifier: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeCaptureStatusBonusKey {
    pub status: String,
    pub bonus: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeCaptureWobbleProbabilityKey {
    pub catch_rate: u8,
    pub chance: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeItemBattleUseKey {
    pub item_id: String,
    pub effect: String,
    pub battle_menu: String,
    pub battle_usable: bool,
    pub battle_stat_boost_stat: Option<String>,
    pub battle_stat_boost_stages: Option<u8>,
    pub battle_escape_mode: Option<String>,
    pub battle_focus_energy: Option<bool>,
    pub battle_stat_drop_guard: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeItemEffectPlanKey {
    pub item_id: String,
    pub effect_id: String,
    pub behavior_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeItemFieldUseKey {
    pub item_id: String,
    pub effect: String,
    pub field_menu: String,
    pub field_usable: bool,
    pub consumable: bool,
    pub repel_steps: Option<u16>,
    pub escape_rope_mode: Option<String>,
    pub tmhm_index: Option<usize>,
    pub tmhm_move: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeMoveBattleDataKey {
    pub move_id: String,
    pub name: String,
    pub move_type: String,
    pub power: u16,
    pub accuracy: u8,
    pub pp: u8,
    pub effect: String,
    pub effect_chance: u8,
    pub stat: Option<Stat>,
    pub amount: Option<i8>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeSpeciesBattleDataKey {
    pub species_id: String,
    pub int_id: u16,
    pub base_hp: u16,
    pub base_attack: u16,
    pub base_defense: u16,
    pub base_speed: u16,
    pub base_special_attack: u16,
    pub base_special_defense: u16,
    pub type1: String,
    pub type2: String,
    pub catch_rate: u8,
    pub base_exp: u16,
    pub item1: Option<String>,
    pub item2: Option<String>,
    pub gender_ratio: u8,
    pub step_cycles_to_hatch: u8,
    pub growth_rate: String,
    pub egg_group1: String,
    pub egg_group2: String,
    pub tmhm_learnset: Vec<String>,
    pub ability: String,
    pub weight: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeSpeciesLearnsetKey {
    pub species_id: String,
    pub level: u8,
    pub move_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeSpeciesEvolutionKey {
    pub source_species_id: String,
    pub method: String,
    pub target_species_id: String,
    pub level: Option<u8>,
    pub item: Option<String>,
    pub held_item: Option<String>,
    pub happiness: Option<String>,
    pub stat_ratio: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeTrainerBattleDataKey {
    pub trainer_id: String,
    pub name: String,
    pub trainer_class: String,
    pub win_quote: String,
    pub lose_quote: String,
    pub items: Vec<Option<String>>,
    pub base_reward: u32,
    pub ai_move_flags: u32,
    pub ai_item_switch_flags: u32,
    pub encounter_music: String,
    pub ai_layers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeTrainerPartyPokemonKey {
    pub trainer_id: String,
    pub party_index: usize,
    pub species: String,
    pub level: u8,
    pub item: Option<String>,
    pub move_names: Vec<String>,
    pub move_pp: Vec<u8>,
    pub move_pp_ups: Vec<u8>,
    pub dv_attack: u8,
    pub dv_defense: u8,
    pub dv_speed: u8,
    pub dv_special: u8,
    pub dv_hp: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeMovePriorityEffectKey {
    pub effect_id: String,
    pub priority: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeMovePriorityMoveKey {
    pub move_id: String,
    pub priority: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeBattleStatMultiplierKey {
    pub table: String,
    pub stage: i8,
    pub numerator: i32,
    pub denominator: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeBattleRewardRuleKey {
    pub field: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeBattleEscapeRuleKey {
    pub field: String,
    pub value: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeTypeEffectivenessKey {
    pub attacking_type: String,
    pub defending_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeWeatherTypeModifierKey {
    pub weather: String,
    pub type_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeWeatherMoveEffectModifierKey {
    pub weather: String,
    pub effect_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleCommandSnapshot {
    pub player_move_slots: Vec<usize>,
    pub player_forced_struggle: bool,
    /// The source battle loop bypasses command selection while an existing
    /// multi-turn move or recharge state owns the player's next action.
    pub player_turn_automatic: bool,
    /// Bide still exposes the four-command menu, but choosing FIGHT bypasses
    /// move selection and resumes the retained Bide move immediately.
    pub player_fight_automatic: bool,
    pub enemy_move_slots: Vec<usize>,
    pub switch_party_indices: Vec<usize>,
    pub can_use_items: bool,
    pub can_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeBattleKind {
    Wild {
        map_name: String,
        battle_music: String,
    },
    StaticWild {
        origin_map_name: String,
        species: String,
        level: u8,
        source_script: String,
        startbattle_command_index: usize,
        resume_command_index: usize,
        battle_music: String,
    },
    Trainer {
        trainer_class: String,
        trainer_id: String,
        trainer_name: String,
        event_flag: String,
        seen_text: String,
        win_text: String,
        loss_text: String,
        callback: String,
        source_script: String,
        reward: u32,
        encounter_music: String,
        ai_move_flags: u32,
        ai_item_switch_flags: u32,
        ai_layers: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePendingMoveLearnSnapshot {
    pub party_index: usize,
    pub species_id: String,
    pub level: u8,
    pub learned_move: LearnedMove,
    pub defer_level_evolution: bool,
}

impl RuntimePendingMoveLearnSnapshot {
    fn from_state(state: &GameState) -> Option<Self> {
        let pending = state.pending_move_learn.as_ref()?;
        Some(Self {
            party_index: pending.party_index,
            species_id: pending.species_id.clone(),
            level: pending.level,
            learned_move: pending.learned_move.clone(),
            defer_level_evolution: pending.defer_level_evolution,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePendingMoveLearnResolution {
    pub resolution: PendingMoveLearnResolution,
    pub deferred_evolution: Option<EvolutionReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePartySnapshot {
    pub slots: Vec<RuntimePartySlotSnapshot>,
    pub active_battle_slot: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePartySlotSnapshot {
    pub index: usize,
    pub pokemon: Pokemon,
    pub is_active_battle_pokemon: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStorageSnapshot {
    pub current_pc_box: usize,
    pub party_count: usize,
    pub boxes: Vec<RuntimePcBoxSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePcBoxSnapshot {
    pub index: usize,
    pub name: String,
    pub count: usize,
    pub slots: Vec<RuntimePcBoxSlotSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePcBoxSlotSnapshot {
    pub index: usize,
    pub pokemon: Pokemon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStorageBoxSwitch {
    pub box_index_before: usize,
    pub box_index_after: usize,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStorageBoxName {
    pub box_index: usize,
    pub previous_name: String,
    pub name: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStorageDeposit {
    pub party_index: usize,
    pub box_index: usize,
    pub box_slot: usize,
    pub pokemon: Pokemon,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStorageWithdraw {
    pub box_index: usize,
    pub box_slot: usize,
    pub party_index: usize,
    pub pokemon: Pokemon,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStorageRelease {
    pub box_index: usize,
    pub box_slot: usize,
    pub pokemon: Pokemon,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStorageMove {
    pub source: crystal_assets::RuntimePokemonStorageLocation,
    pub target: crystal_assets::RuntimePokemonStorageLocation,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePcItemTransfer {
    pub item_id: String,
    pub quantity: u16,
    pub bag_quantity_after: u16,
    pub pc_quantity_after: u16,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHeldItemTransfer {
    pub party_index: usize,
    pub item_id: String,
    pub bag_quantity_after: u16,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMailTransfer {
    pub party_index: Option<usize>,
    pub mailbox_index: Option<usize>,
    pub item_id: String,
    pub mail: crystal_core::models::pokemon::MailData,
    pub mailbox_count_after: usize,
    pub bag_quantity_after: u16,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBagItemMutation {
    pub item_id: String,
    pub quantity: u16,
    pub added: bool,
    pub quantity_before: u16,
    pub quantity_after: u16,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBadgeAward {
    pub region: RuntimeBadgeRegion,
    pub index: usize,
    pub already_awarded: bool,
    pub awarded_count_after: usize,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePokedexRecord {
    pub species_id: String,
    pub already_seen: bool,
    pub already_caught: bool,
    pub seen_count_after: usize,
    pub caught_count_after: usize,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCurrencyMutation {
    pub account: RuntimeCurrencyAccount,
    pub amount: u32,
    pub value_before: u32,
    pub value_after: u32,
    pub cap: u32,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLinkBattleRecord {
    pub result: RuntimeLinkBattleResult,
    pub wins_after: u16,
    pub losses_after: u16,
    pub draws_after: u16,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOptionsSet {
    pub options_before: Options,
    pub options_after: Options,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTrainerIdentitySet {
    pub player_name_before: String,
    pub player_id_before: u16,
    pub player_name_after: String,
    pub player_id_after: u16,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlayerGenderSet {
    pub player_gender_before: u8,
    pub player_gender_after: u8,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePartyNicknameSet {
    pub party_index: usize,
    pub species_id: String,
    pub nickname_before: String,
    pub nickname_after: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePartyRecoveryStateSet {
    pub outcome: RuntimePartyRecoverySetupOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePartyHpTransfer {
    pub outcome: RuntimePartyHpTransferOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePartyRecovery {
    pub party_index: usize,
    pub species_id: String,
    pub hp_before: u16,
    pub hp_after: u16,
    pub status_before: Option<String>,
    pub status_after: Option<String>,
    pub pp_restored: Vec<(String, u8, u8)>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBlackoutRecovery {
    pub spawn_identifier: Option<u16>,
    pub map_name: String,
    pub tile: TilePosition,
    pub healed: Vec<RuntimePartyRecovery>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePartySwap {
    pub first_party_index: usize,
    pub second_party_index: usize,
    pub first_species_after: String,
    pub second_species_after: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePartyMoveSwap {
    pub party_index: usize,
    pub first_move_index: usize,
    pub second_move_index: usize,
    pub first_move_after: String,
    pub second_move_after: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBagSnapshot {
    pub items: Vec<RuntimeBagItemSnapshot>,
    pub balls: Vec<RuntimeBagItemSnapshot>,
    pub key_items: Vec<RuntimeBagItemSnapshot>,
    pub tm_hm: Vec<RuntimeTmHmSnapshot>,
    pub pc_items: Vec<RuntimeBagItemSnapshot>,
    pub custom_pockets: BTreeMap<String, Vec<RuntimeBagItemSnapshot>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBagItemSnapshot {
    pub item_id: String,
    pub quantity: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeItemCatalogSnapshot {
    pub item_id: String,
    pub name: String,
    pub description: String,
    pub effect: String,
    pub status_heals: Vec<String>,
    pub revive_hp_percent: Option<u8>,
    pub party_revive_hp_percent: Option<u8>,
    pub pp_restore_scope: Option<String>,
    pub pp_restore_points: Option<u8>,
    pub pp_up_stages: Option<u8>,
    pub vitamin_stat: Option<String>,
    pub vitamin_stat_exp: Option<u16>,
    pub vitamin_max_stat_exp: Option<u16>,
    pub rare_candy_level_gain: Option<u8>,
    pub party_special_effect: bool,
    pub battle_stat_boost_stat: Option<String>,
    pub battle_stat_boost_stages: Option<u8>,
    pub battle_escape_mode: Option<String>,
    pub battle_focus_energy: Option<bool>,
    pub battle_stat_drop_guard: Option<bool>,
    pub confusion_heal: Option<bool>,
    pub repel_steps: Option<u16>,
    pub escape_rope_mode: Option<String>,
    pub price: u16,
    pub held_effect: String,
    pub parameter: i16,
    pub property: String,
    pub pocket: String,
    pub field_menu: String,
    pub field_usable: bool,
    pub battle_menu: String,
    pub battle_usable: bool,
    pub script_name: String,
    pub consumable: bool,
    pub tmhm_index: Option<usize>,
    pub tmhm_move: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMoveCatalogSnapshot {
    pub move_id: String,
    pub source_index: u8,
    pub name: String,
    pub move_type: String,
    pub power: u16,
    pub accuracy: u8,
    pub pp: u8,
    pub effect: String,
    pub effect_chance: u8,
    pub stat: Option<Stat>,
    pub amount: Option<i8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePokemonCatalogSnapshot {
    pub species_id: String,
    pub int_id: u16,
    pub base_stats: crystal_core::models::BaseStats,
    pub type1: String,
    pub type2: String,
    pub catch_rate: u8,
    pub base_exp: u16,
    pub item1: Option<String>,
    pub item2: Option<String>,
    pub gender_ratio: u8,
    pub unknown1: u8,
    pub step_cycles_to_hatch: u8,
    pub unknown2: u8,
    pub growth_rate: String,
    pub egg_group1: String,
    pub egg_group2: String,
    pub tmhm_learnset: Vec<String>,
    pub ability: String,
    pub pic_size: u8,
    pub front_pic: u16,
    pub back_pic: u16,
    pub weight: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTrainerCatalogSnapshot {
    pub trainer_id: String,
    pub name: String,
    pub trainer_class: String,
    pub party: Vec<RuntimeTrainerPartyPokemonSnapshot>,
    pub win_quote: String,
    pub lose_quote: String,
    pub items: Vec<Option<String>>,
    pub base_reward: u32,
    pub ai_move_flags: u32,
    pub ai_item_switch_flags: u32,
    pub encounter_music: String,
    pub ai_layers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTrainerPartyPokemonSnapshot {
    pub species: String,
    pub level: u8,
    pub item: Option<String>,
    pub moves: Vec<RuntimeLearnedMoveSnapshot>,
    pub dvs: Dv,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLearnedMoveSnapshot {
    pub name: String,
    pub current_pp: u8,
    pub pp_ups: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMapCatalogSnapshot {
    pub map_name: String,
    pub id: String,
    pub attributes: crystal_core::map::MapAttributes,
    pub metadata: Option<RuntimeMapMetadataSnapshot>,
    pub scenes: crystal_core::map::MapSceneTable,
    pub events: crystal_core::map::MapEvents,
    pub objects: Vec<crystal_core::map::ObjectEvent>,
    pub blocks: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMapMetadataSnapshot {
    pub constant: String,
    pub name: String,
    pub group_name: String,
    pub group_id: u16,
    pub map_id: u16,
    pub width: u16,
    pub height: u16,
    pub environment: String,
    pub phone_service: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCurrentSceneScript {
    pub map_name: String,
    pub scene_id: String,
    pub script_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTilesetCatalogSnapshot {
    pub tileset_id: String,
    pub collision: BTreeMap<String, Vec<String>>,
    pub palette_map: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEncounterCatalogSnapshot {
    pub wild: BTreeMap<String, WildEncounterData>,
    pub field: BTreeMap<String, FieldEncounterData>,
    pub slot_tables: EncounterSlotTables,
    pub fishing: crystal_core::world::fishing::FishingCatalog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleRuleCatalogSnapshot {
    pub capture_rules: CaptureRules,
    pub capture_wobble_probabilities: Vec<CaptureWobbleProbability>,
    pub stat_multipliers: BattleStatMultiplierTables,
    pub move_priorities: MovePriorityTable,
    pub type_categories: TypeCategories,
    pub type_effectiveness: TypeEffectivenessTable,
    pub weather_modifiers: WeatherModifiers,
    pub reward_rules: BattleRewardRules,
    pub escape_rules: BattleEscapeRules,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWorldRuleCatalogSnapshot {
    pub marts: crystal_core::systems::shop::MartCatalog,
    pub currency: crystal_core::systems::economy::CurrencyCatalog,
    pub fruit_trees: crystal_core::systems::field_items::FruitTreeCatalog,
    pub field_moves: crystal_core::systems::field_moves::FieldMoveCatalog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePresentationCatalogSnapshot {
    pub title_screen: RuntimeTitleScreen,
    pub pc_strings: BTreeMap<String, String>,
    pub menu_icons: BTreeMap<String, String>,
    pub pokedex_entries: BTreeMap<String, RuntimePokedexEntry>,
    pub pokemon_frontpic_anim: BTreeMap<String, FrontpicAnimProgram>,
    pub asm_text: BTreeMap<String, String>,
    pub move_names: Vec<String>,
    pub battle_animations: BTreeMap<String, Vec<String>>,
    pub battle_animation_table: Vec<String>,
    pub battle_anim_bundle: String,
    pub sprite_anim_bundle: String,
    pub sprite_palette_defaults: BTreeMap<String, i64>,
    pub pokegear_town_map_palette_map: BTreeMap<String, Vec<String>>,
    pub pokegear_landmarks: PokegearLandmarksPayload,
    pub pokemon_cries: BTreeMap<String, PokemonCryMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSpecialCatalogSnapshot {
    pub phone_contacts: crystal_core::systems::phone::PhoneContactCatalog,
    pub permanent_phone_numbers:
        BTreeMap<String, crystal_core::systems::phone::PermanentPhoneNumberRule>,
    pub special_phone_calls: BTreeMap<String, crystal_assets::SpecialPhoneCallRule>,
    pub npc_trades: BTreeMap<String, crystal_assets::NpcTradeRule>,
    pub special_routines: BTreeMap<String, crystal_assets::SpecialRoutineRule>,
    pub flee_mons: crystal_core::systems::flee_mons::FleeMonTables,
    pub buena_password_categories: crystal_core::systems::special_routines::BuenaPasswordCategories,
    pub roaming_pokemon: crystal_core::systems::special_routines::RoamingPokemonCatalog,
    pub buena_prizes: crystal_core::systems::special_routines::BuenaPrizeDefinitions,
    pub kurt_apricorn_recipes: crystal_core::systems::special_routines::KurtApricornRecipes,
    pub shuckie_gift: Option<crystal_core::systems::special_routines::ShuckieGiftDefinition>,
    pub dratini_move_sets: crystal_core::systems::special_routines::DratiniMoveSets,
    pub bug_contest_config: Option<crystal_core::systems::special_routines::BugContestConfig>,
    pub battle_tower_rules: Option<crystal_core::systems::special_routines::BattleTowerRules>,
    pub oak_ratings: Vec<crystal_core::systems::special_routines::OakRatingEntry>,
    pub odd_egg_definitions: Vec<crystal_core::systems::special_routines::OddEggDefinition>,
    pub magikarp_lengths: Vec<crystal_core::systems::special_routines::MagikarpLengthEntry>,
    pub happiness_data: Option<crystal_core::systems::special_routines::HappinessData>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStoryCatalogSnapshot {
    pub initialize_events: crystal_core::systems::script_runtime::InitializeEventsConfig,
    pub story_event_script_constants:
        crystal_core::systems::script_runtime::StoryEventScriptConstants,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTmHmSnapshot {
    pub item_id: String,
    pub tmhm_index: usize,
    pub move_id: Option<String>,
    pub quantity: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeShellPhase {
    Overworld,
    WildBattle,
    StaticWildBattle,
    TrainerBattle,
    Text,
    YesNo,
    Menu,
    Shop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOverworldFrame {
    pub snapshot: OverworldSnapshot,
    pub input_mask: u8,
    pub pressed_mask: u8,
    pub autonomous_objects_changed: bool,
    pub movement: Option<StepOutcome>,
    pub ledge_jump: Option<LedgeJumpOutcome>,
    pub grass_rustle: Option<crystal_assets::OverworldGrassRustle>,
    pub phone_call: Option<crystal_assets::IncomingPhoneCall>,
    pub step_events: Option<StepEventResult>,
    pub coord_event: Option<CoordEventTrigger>,
    pub trainer_sight: Option<OverworldInteraction>,
    pub interaction: Option<OverworldInteraction>,
    pub warp: Option<WarpTransition>,
    pub connection: Option<ConnectionTransition>,
    pub wild_encounter: Option<WildEncounterRoll>,
    pub wild_battle: Option<WildBattleStart>,
    pub state_checksum: StateChecksum,
}

/// A deterministic RTC sample supplied by the host or replay/oracle adapter.
/// Applying it immediately before the joypad frame preserves Crystal's
/// ordering: day-boundary resets are committed before movement and scripts
/// observe the new date/time in that same frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeRtcSample {
    pub date: GameDate,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl RuntimeOverworldFrame {
    fn from_input_frame(frame: OverworldInputFrame, state_checksum: StateChecksum) -> Self {
        Self {
            snapshot: frame.snapshot,
            input_mask: frame.input_mask,
            pressed_mask: frame.pressed_mask,
            autonomous_objects_changed: frame.autonomous_objects_changed,
            movement: frame.movement,
            ledge_jump: frame.ledge_jump,
            grass_rustle: frame.grass_rustle,
            phone_call: frame.phone_call,
            step_events: frame.step_events,
            coord_event: frame.coord_event,
            trainer_sight: frame.trainer_sight,
            interaction: frame.interaction,
            warp: frame.warp,
            connection: frame.connection,
            wild_encounter: frame.wild_encounter,
            wild_battle: frame.wild_battle,
            state_checksum,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInteractionScriptDispatch {
    pub next_script: String,
    pub last_talked_object: Option<String>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCompiledScriptCursor {
    pub origin_map_name: String,
    pub source_script: String,
    pub command_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCompiledScriptStep {
    pub origin_map_name: String,
    pub source_script: String,
    pub command_index: usize,
    pub command: String,
    pub mutation: RuntimeMutationOutcome,
    pub next_cursor: Option<RuntimeCompiledScriptCursor>,
    pub boundary: Option<RuntimeCompiledScriptBoundary>,
    pub next_script: Option<String>,
    pub ended: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeCompiledScriptBoundary {
    DayOfWeekPrompt,
    PhoneNumberPrompt,
    TextLabel(String),
    TextWait(ScriptTextWait),
    YesNo(ScriptYesNoPrompt),
    ActiveMenu(String),
    PendingShop(ScriptShopRequest),
    PendingScriptWarp(ScriptWarpRequest),
    PendingMapLoad(ScriptMapLoadRequest),
    PendingMapRefresh(ScriptMapRefreshRequest),
    Delay(ScriptRuntimeDelay),
    Earthquake(ScriptRuntimeEarthquake),
    Emote(ScriptRuntimeEmote),
    ScriptMovement,
    VerboseItemGrant,
    WaitForSoundEffect,
    PhoneCallasm(crystal_core::systems::script_runtime::ScriptPhoneCallasmPresentation),
    ActiveBattle(RuntimeShellPhase),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCompiledScriptRun {
    pub steps: Vec<RuntimeCompiledScriptStep>,
    pub next_cursor: Option<RuntimeCompiledScriptCursor>,
    pub boundary: Option<RuntimeCompiledScriptBoundary>,
    pub ended: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeQueuedCompiledScriptRun {
    pub queued: RuntimeQueuedScriptCommand,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePendingCompiledScriptRun {
    pub next_script: RuntimeNextScript,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStandardScriptRun {
    pub next_script: RuntimeNextScript,
    pub result: String,
    pub state_checksum: StateChecksum,
    pub boundary: Option<RuntimeCompiledScriptBoundary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDeferredCompiledScriptRun {
    pub deferred_script: RuntimeDeferredScript,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTextWaitCompiledScriptRun {
    pub wait: RuntimeTextWaitAdvance,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeYesNoCompiledScriptRun {
    pub resolution: RuntimeYesNoResolution,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePhonePromptCompiledScriptRun {
    pub step: RuntimeCompiledScriptStep,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMenuSelectionCompiledScriptRun {
    pub selection: RuntimeVerticalMenuOptionSelection,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeElevatorFloorCompiledScriptRun {
    pub selection: RuntimeElevatorFloorSelection,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeGiftPokemonCompiledScriptRun {
    pub grant: RuntimeGiftPokemonGrant,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptWarpCompiledScriptRun {
    pub warp: RuntimeScriptWarp,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptedWildBattleCompiledScriptRun {
    pub completion: RuntimeScriptedBattleCompletion,
    pub run: RuntimeCompiledScriptRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptedTrainerBattleCompiledScriptRun {
    pub completion: RuntimeScriptedBattleCompletion,
    pub run: RuntimeCompiledScriptRun,
}

fn compiled_script_boundary(state: &GameState) -> Option<RuntimeCompiledScriptBoundary> {
    if let Some(wait) = &state.script_runtime.pending_text_wait {
        return Some(RuntimeCompiledScriptBoundary::TextWait(wait.clone()));
    }
    if let Some(prompt) = &state.script_runtime.pending_yes_no {
        return Some(RuntimeCompiledScriptBoundary::YesNo(prompt.clone()));
    }
    if let Some(menu) = &state.script_runtime.active_menu {
        return Some(RuntimeCompiledScriptBoundary::ActiveMenu(menu.clone()));
    }
    if let Some(shop) = &state.script_runtime.pending_shop {
        return Some(RuntimeCompiledScriptBoundary::PendingShop(shop.clone()));
    }
    if let Some(warp) = &state.script_runtime.pending_script_warp {
        return Some(RuntimeCompiledScriptBoundary::PendingScriptWarp(
            warp.clone(),
        ));
    }
    if let Some(load) = &state.script_runtime.pending_map_load {
        return Some(RuntimeCompiledScriptBoundary::PendingMapLoad(load.clone()));
    }
    if let Some(refresh) = &state.script_runtime.pending_map_refresh {
        return Some(RuntimeCompiledScriptBoundary::PendingMapRefresh(
            refresh.clone(),
        ));
    }
    if let Some(delay) = state.script_runtime.pending_delays.first() {
        return Some(RuntimeCompiledScriptBoundary::Delay(delay.clone()));
    }
    if let Some(earthquake) = state.script_runtime.pending_earthquakes.first() {
        return Some(RuntimeCompiledScriptBoundary::Earthquake(
            earthquake.clone(),
        ));
    }
    if let Some(emote) = state.script_runtime.pending_emotes.first() {
        return Some(RuntimeCompiledScriptBoundary::Emote(emote.clone()));
    }
    if state.script_runtime.waiting_for_sound_effect {
        return Some(RuntimeCompiledScriptBoundary::WaitForSoundEffect);
    }
    let battle = match &state.battle {
        BattleMemory::Inactive => None,
        BattleMemory::Wild { .. } => Some(RuntimeCompiledScriptBoundary::ActiveBattle(
            RuntimeShellPhase::WildBattle,
        )),
        BattleMemory::StaticWild { .. } => Some(RuntimeCompiledScriptBoundary::ActiveBattle(
            RuntimeShellPhase::StaticWildBattle,
        )),
        BattleMemory::Trainer { .. } => Some(RuntimeCompiledScriptBoundary::ActiveBattle(
            RuntimeShellPhase::TrainerBattle,
        )),
    };
    if battle.is_some() {
        return battle;
    }
    // A text label is the synchronous PrintText presentation boundary. Keep
    // it behind every already-active timed, modal, movement, audio, and battle
    // boundary, then stop before any command following this writetext runs.
    state
        .script_runtime
        .pending_text_label
        .clone()
        .map(RuntimeCompiledScriptBoundary::TextLabel)
}

fn empty_compiled_script_run() -> RuntimeCompiledScriptRun {
    RuntimeCompiledScriptRun {
        steps: Vec::new(),
        next_cursor: None,
        boundary: None,
        ended: false,
    }
}

include!("runtime_shell.rs");
include!("runtime_shell_actions.rs");

/// Whether a presentation boundary pauses the compiled script interpreter.
pub fn compiled_script_boundary_stops_run(
    command: &str,
    boundary: &Option<RuntimeCompiledScriptBoundary>,
) -> bool {
    boundary.is_some()
        // ASM Script_loadmenu prepares the header and returns. The actual
        // input boundary is Script_verticalmenu/Script__2dmenu.
        && !(command == "loadmenu"
            && matches!(boundary, Some(RuntimeCompiledScriptBoundary::ActiveMenu(_))))
}

impl RuntimeShellAudioState {
    fn from_state(state: &GameState) -> Self {
        Self {
            current_music: state.script_runtime.current_music.clone(),
            queued_events: state.script_runtime.audio_events.clone(),
        }
    }
}

impl RuntimeTrainerSnapshot {
    fn from_state(state: &GameState) -> Self {
        Self {
            player_name: state.player_name.clone(),
            player_id: state.player_id,
            player_gender: state.player_gender,
            player_palette_id: state.player_palette_id,
            money: state.money,
            moms_money: state.moms_money,
            coins: state.coins,
            blue_card_balance: u16::from(state.blue_card_balance),
            current_pc_box: state.current_pc_box,
            options: state.options.clone(),
        }
    }
}

impl RuntimeProgressionSnapshot {
    fn from_state(state: &GameState) -> Self {
        Self {
            backup_warp_map_name: state.backup_warp_map_name.clone(),
            badges: state.badges.clone(),
            pokedex_seen: state.pokedex.seen_count(),
            pokedex_owned: state.pokedex.caught_count(),
            pokedex_seen_species: state.pokedex.seen_species.clone(),
            pokedex_caught_species: state.pokedex.caught_species.clone(),
            link_wins: state.link_battle_stats.wins,
            link_losses: state.link_battle_stats.losses,
            link_draws: state.link_battle_stats.draws,
            pending_special_battle_type: state.pending_special_battle_type.clone(),
            repel_steps_remaining: state.repel_steps_remaining,
            active_repel_item: state.active_repel_item.clone(),
            registered_key_item: state.registered_key_item.clone(),
            radio_tuning_knob: state.radio_tuning_knob,
            last_spawn_map_constant: state.last_spawn_map_constant.clone(),
            hall_of_fame: state.hall_of_fame.clone(),
            time: state.time.clone(),
            active_event_flags: state.flags.active_event_flags().cloned().collect(),
            active_engine_flags: state
                .flags
                .engine_flags
                .iter()
                .filter_map(|(flag, set)| set.then(|| flag.clone()))
                .collect(),
        }
    }
}

impl RuntimeLinkSessionSnapshot {
    fn from_state(state: &GameState) -> Self {
        let link = &state.link_session;
        Self {
            link_mode: link.link_mode,
            player_link_action: link.player_link_action,
            chosen_cable_club_room: link.chosen_cable_club_room,
            other_player_link_mode: link.other_player_link_mode,
        }
    }
}

impl RuntimeScriptEventsSnapshot {
    fn from_state(state: &GameState) -> Self {
        let runtime = &state.script_runtime;
        Self {
            script_value: runtime.script_value.clone(),
            variables: runtime.variables.clone(),
            memory: runtime.memory.clone(),
            named_buffers: runtime.named_buffers.clone(),
            variable_sprites: runtime.variable_sprites.clone(),
            phone_numbers: runtime.phone_numbers.clone(),
            phone_number_order: runtime.phone_number_order.clone(),
            last_special_routine: runtime.last_special_routine.clone(),
            last_talked_object: runtime.last_talked_object.clone(),
            active_menu: runtime.active_menu.clone(),
            pending_delays: runtime.pending_delays.clone(),
            pending_earthquakes: runtime.pending_earthquakes.clone(),
            pending_emotes: runtime.pending_emotes.clone(),
            command_queue: runtime.command_queue.clone(),
            call_stack: runtime.call_stack.clone(),
            stone_table_entries: runtime.stone_table_entries.clone(),
            special_phone_call: runtime.special_phone_call.clone(),
            audio_events: runtime.audio_events.clone(),
            pending_music_fade: runtime.pending_music_fade.clone(),
            waiting_for_sound_effect: runtime.waiting_for_sound_effect,
            map_music_restart_disabled: runtime.map_music_restart_disabled,
            map_music_requested: runtime.map_music_requested,
            graphics_events: runtime.graphics_events.clone(),
            pending_screen_fade: runtime.pending_screen_fade.clone(),
            money_events: runtime.money_events.clone(),
            map_events: runtime.map_events.clone(),
            pending_script_warp: runtime.pending_script_warp.clone(),
            pending_map_load: runtime.pending_map_load.clone(),
            pending_map_refresh: runtime.pending_map_refresh.clone(),
            text_events: runtime.text_events.clone(),
            window_open: runtime.window_open,
            active_pokemon_picture: runtime.active_pokemon_picture.clone(),
            text_window_open: runtime.text_window_open,
            active_text_label: runtime.active_text_label.clone(),
            pending_text_label: runtime.pending_text_label.clone(),
            pending_text_wait: runtime.pending_text_wait.clone(),
            pending_yes_no: runtime.pending_yes_no.clone(),
            control_events: runtime.control_events.clone(),
            next_script: runtime.next_script.clone(),
            deferred_scripts: runtime.deferred_scripts.clone(),
            map_reentry_script: runtime.map_reentry_script.clone(),
            script_ended: runtime.script_ended.clone(),
            player_input_locked: runtime.player_input_locked,
            all_input_locked: runtime.all_input_locked,
            script_stop_requested: runtime.script_stop_requested,
            item_notify_queued: runtime.item_notify_queued,
            warp_sound_queued: runtime.warp_sound_queued,
            teleport_from_queued: runtime.teleport_from_queued,
            hall_of_fame_requested: runtime.hall_of_fame_requested,
            credits_requested: runtime.credits_requested,
            reset_requested: runtime.reset_requested,
            menu_2d_requested: runtime.menu_2d_requested,
            completed_trades: runtime.completed_trades.clone(),
            shop_events: runtime.shop_events.clone(),
            item_use_events: runtime.item_use_events.clone(),
        }
    }
}

impl RuntimeMenuSnapshot {
    fn from_state(
        state: &GameState,
        menu_id: String,
        source: RuntimeMenuSource,
        definition: Option<ScriptMenuDefinition>,
        vertical_menus: Vec<RuntimeVerticalMenuSnapshot>,
    ) -> Result<Self> {
        let layout =
            RuntimeMenuLayoutSnapshot::from_definition(definition.as_ref(), vertical_menus)
                .with_context(|| format!("parse runtime menu layout for {menu_id}"))?;
        let coords = layout.declared_coords;
        Ok(Self {
            menu_id,
            source,
            definition,
            layout,
            window_open: state.script_runtime.window_open,
            coords,
            menu_2d_requested: state.script_runtime.menu_2d_requested,
        })
    }
}

impl RuntimeMenuLayoutSnapshot {
    fn from_definition(
        definition: Option<&ScriptMenuDefinition>,
        vertical_menus: Vec<RuntimeVerticalMenuSnapshot>,
    ) -> Result<Self> {
        let Some(definition) = definition else {
            return Ok(Self {
                declared_coords: None,
                data_commands: Vec::new(),
                vertical_menus,
            });
        };
        let mut declared_coords = None;
        let mut data_commands = Vec::new();
        for command in &definition.commands {
            match command.command.as_str() {
                "menu_coords" => {
                    let coords = parse_menu_coords(&command.args).with_context(|| {
                        format!("parse menu_coords command {}", command.command_index)
                    })?;
                    declared_coords = Some(coords);
                }
                "db" | "dw" => data_commands.push(RuntimeMenuDataCommandSnapshot {
                    command: command.command.clone(),
                    args: command.args.clone(),
                    command_index: command.command_index,
                }),
                _ => {}
            }
        }
        Ok(Self {
            declared_coords,
            data_commands,
            vertical_menus,
        })
    }
}

impl RuntimeVerticalMenuSnapshot {
    fn from_definition(definition: &crystal_assets::ScriptVerticalMenuDefinition) -> Self {
        Self {
            source_script: definition.source_script.clone(),
            loadmenu_command_index: definition.loadmenu_command_index,
            verticalmenu_command_index: definition.verticalmenu_command_index,
            header_label: definition.header_label.clone(),
            data_label: definition.data_label.clone(),
            options: definition.options.clone(),
            two_dimensional: definition.two_dimensional,
            rows: definition.rows,
            columns: definition.columns,
            spacing: definition.spacing,
        }
    }
}

impl RuntimeElevatorSnapshot {
    fn from_definition(
        map_name: &str,
        definition: &crystal_assets::ScriptElevatorDefinition,
    ) -> Self {
        Self {
            map_name: map_name.to_string(),
            source_script: definition.source_script.clone(),
            elevator_command_index: definition.elevator_command_index,
            data_label: definition.data_label.clone(),
            floors: definition
                .floors
                .iter()
                .enumerate()
                .map(|(floor_index, floor)| RuntimeElevatorFloorSnapshot {
                    floor_index,
                    floor: floor.floor.clone(),
                    warp: floor.warp,
                    target_map: floor.target_map.clone(),
                    source_script: floor.source_script.clone(),
                    command_index: floor.command_index,
                })
                .collect(),
        }
    }
}

impl RuntimeGiftPokemonSnapshot {
    fn from_script(map_name: &str, gift: &GiftPokemonScript) -> Self {
        Self {
            map_name: map_name.to_string(),
            source_script: gift.source_script.clone(),
            command_index: gift.command_index,
            species_id: gift.species_id.clone(),
            level: gift.level,
            level_token: gift.level_token.clone(),
            held_item_id: gift.held_item_id.clone(),
            nickname_label: gift.nickname_label.clone(),
            ot_label: gift.ot_label.clone(),
            egg: gift.egg,
        }
    }
}

fn parse_menu_coords(args: &[String]) -> Result<[i16; 4]> {
    if args.len() != 4 {
        anyhow::bail!("menu_coords requires 4 operands, got {}", args.len());
    }
    Ok([
        parse_menu_coord(&args[0], "left")?,
        parse_menu_coord(&args[1], "top")?,
        parse_menu_coord(&args[2], "right")?,
        parse_menu_coord(&args[3], "bottom")?,
    ])
}

fn parse_menu_coord(value: &str, name: &str) -> Result<i16> {
    parse_menu_coord_expression(value)
        .with_context(|| format!("menu coordinate {name} must be an exact i16, got {value:?}"))
}

fn parse_menu_coord_expression(value: &str) -> Result<i16> {
    parse_menu_coord_token("menu_coords", value).map_err(anyhow::Error::new)
}

impl RuntimeUiSnapshot {
    fn from_state(
        state: &GameState,
        menu: Option<RuntimeMenuSnapshot>,
        elevators: Vec<RuntimeElevatorSnapshot>,
        gift_pokemon: Vec<RuntimeGiftPokemonSnapshot>,
        text: Option<RuntimeTextSnapshot>,
    ) -> Self {
        let runtime = &state.script_runtime;
        let coords = menu.as_ref().and_then(|menu| menu.coords);
        Self {
            menu,
            elevators,
            gift_pokemon,
            text,
            window_open: runtime.window_open,
            text_window_open: runtime.text_window_open,
            coords,
            active_pokemon_picture: runtime.active_pokemon_picture.clone(),
            pending_yes_no: runtime.pending_yes_no.clone(),
            pending_text_wait: runtime.pending_text_wait.clone(),
        }
    }
}

fn battle_trainer_display_name(name: &str, rival_name: Option<&str>) -> String {
    name.trim_end_matches('@').replace(
        "<RIVAL>",
        rival_name
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("???"),
    )
}

impl RuntimeBattleSnapshot {
    pub fn phase(&self) -> RuntimeShellPhase {
        match &self.kind {
            RuntimeBattleKind::Wild { .. } => RuntimeShellPhase::WildBattle,
            RuntimeBattleKind::StaticWild { .. } => RuntimeShellPhase::StaticWildBattle,
            RuntimeBattleKind::Trainer { .. } => RuntimeShellPhase::TrainerBattle,
        }
    }

    fn from_state(state: &GameState) -> Result<Option<Self>> {
        match &state.battle {
            BattleMemory::Inactive => Ok(None),
            BattleMemory::Wild {
                battle_type,
                battle_music,
                map_name,
                enemy_pokemon,
                enemy_party,
                ..
            } => Self::from_parts(
                state,
                RuntimeBattleKind::Wild {
                    map_name: map_name.clone(),
                    battle_music: battle_music.clone(),
                },
                battle_type,
                enemy_pokemon,
                enemy_party,
            )
            .map(Some),
            BattleMemory::StaticWild {
                battle_type,
                battle_music,
                origin_map_name,
                species,
                level,
                source_script,
                startbattle_command_index,
                resume_command_index,
                enemy_pokemon,
                enemy_party,
                ..
            } => Self::from_parts(
                state,
                RuntimeBattleKind::StaticWild {
                    origin_map_name: origin_map_name.clone(),
                    species: species.clone(),
                    level: *level,
                    source_script: source_script.clone(),
                    startbattle_command_index: *startbattle_command_index,
                    resume_command_index: *resume_command_index,
                    battle_music: battle_music.clone(),
                },
                battle_type,
                enemy_pokemon,
                enemy_party,
            )
            .map(Some),
            BattleMemory::Trainer {
                battle_type,
                trainer_class,
                trainer_id,
                trainer_name,
                event_flag,
                seen_text,
                win_text,
                loss_text,
                callback,
                source_script,
                enemy_pokemon,
                enemy_party,
                reward,
                encounter_music,
                ai_move_flags,
                ai_item_switch_flags,
                ai_layers,
            } => Self::from_parts(
                state,
                RuntimeBattleKind::Trainer {
                    trainer_class: trainer_class.clone(),
                    trainer_id: trainer_id.clone(),
                    // Resolve presentation tokens without changing the pack-backed
                    // battle memory used by save validation.
                    trainer_name: battle_trainer_display_name(
                        trainer_name,
                        state.script_runtime.variables.get("_rival_name")
                            .map(String::as_str),
                    ),
                    event_flag: event_flag.clone(),
                    seen_text: seen_text.clone(),
                    win_text: win_text.clone(),
                    loss_text: loss_text.clone(),
                    callback: callback.clone(),
                    source_script: source_script.clone(),
                    reward: *reward,
                    encounter_music: encounter_music.clone(),
                    ai_move_flags: *ai_move_flags,
                    ai_item_switch_flags: *ai_item_switch_flags,
                    ai_layers: ai_layers.clone(),
                },
                battle_type,
                enemy_pokemon,
                enemy_party,
            )
            .map(Some),
        }
    }

    fn from_parts(
        state: &GameState,
        kind: RuntimeBattleKind,
        battle_type: &str,
        enemy_pokemon: &Pokemon,
        enemy_party: &[Pokemon],
    ) -> Result<Self> {
        let commands = RuntimeBattleCommandSnapshot::from_state(state, &kind, enemy_pokemon)?;
        let battle_music = battle_music_from_kind(&kind);
        let combat = state.script_runtime.active_battle_combat.as_ref();
        let active_player = state
            .battle_active_party_index
            .and_then(|index| state.storage.party.pokemon.get(index))
            .and_then(Option::as_ref)
            .with_context(|| "active battle snapshot is missing its player Pokemon")?;
        let player_moves = combat
            .map(|combat| {
                combat
                    .player_transform
                    .as_ref()
                    .map(|transform| transform.moves.clone())
                    .unwrap_or_else(|| combat.player.moves.clone())
            })
            .unwrap_or_else(|| active_player.moves.clone());
        let enemy_moves = combat
            .map(|combat| {
                combat
                    .enemy_transform
                    .as_ref()
                    .map(|transform| transform.moves.clone())
                    .unwrap_or_else(|| combat.enemy.moves.clone())
            })
            .unwrap_or_else(|| enemy_pokemon.moves.clone());
        Ok(Self {
            kind,
            battle_music,
            battle_type: battle_type.to_string(),
            enemy_pokemon: enemy_pokemon.clone(),
            enemy_party: enemy_party.to_vec(),
            active_player_party_index: state.battle_active_party_index,
            active_enemy_party_index: state.battle_active_enemy_party_index,
            player_spikes_zero_hp_unchecked: combat
                .is_some_and(|combat| combat.player_spikes_zero_hp_unchecked),
            enemy_spikes_zero_hp_unchecked: combat
                .is_some_and(|combat| combat.enemy_spikes_zero_hp_unchecked),
            player_transformed_species: combat
                .and_then(|combat| combat.player_transform.as_ref())
                .map(|transform| transform.species.id.clone()),
            enemy_transformed_species: combat
                .and_then(|combat| combat.enemy_transform.as_ref())
                .map(|transform| transform.species.id.clone()),
            player_transformed_dvs: combat.and_then(|combat| combat.player_transform.as_ref()).map(|transform| transform.dvs),
            enemy_transformed_dvs: combat.and_then(|combat| combat.enemy_transform.as_ref()).map(|transform| transform.dvs),
            player_substitute_hp: combat.map_or(0, |combat| combat.player_substitute_hp),
            enemy_substitute_hp: combat.map_or(0, |combat| combat.enemy_substitute_hp),
            player_semi_invulnerable: combat
                .is_some_and(|combat| combat.player_airborne_move.is_some()),
            enemy_semi_invulnerable: combat
                .is_some_and(|combat| combat.enemy_airborne_move.is_some()),
            player_moves,
            enemy_moves,
            player_last_move: combat.and_then(|combat| combat.player_last_move.clone()),
            player_used_moves: combat
                .map(|combat| combat.player_used_moves.clone())
                .unwrap_or_default(),
            enemy_last_move: combat.and_then(|combat| combat.enemy_last_move.clone()),
            enemy_toxic_turns: combat.map_or(0, |combat| combat.enemy_toxic_turns),
            player_turns_taken: combat.map_or(0, |combat| combat.player_turns_taken),
            enemy_turns_taken: combat.map_or(0, |combat| combat.enemy_turns_taken),
            enemy_switch_locked: combat.is_some_and(|combat| {
                combat.enemy_recharge_move.is_some()
                    || combat.enemy_airborne_move.is_some()
                    || combat.enemy_charging_move.is_some()
                    || combat.enemy.rampage_turns > 0
                    || combat.enemy_bide_turns > 0
                    || combat.enemy_rollout_turns > 0
            }),
            player_cannot_escape: combat.is_some_and(|combat| combat.player_escape_trap.is_some()),
            player_wrapped: combat.is_some_and(|combat| combat.player_trap.is_some()),
            enemy_wrapped: combat.is_some_and(|combat| combat.enemy_trap.is_some()),
            rewarded_enemy_party_indices: state
                .battle_rewarded_enemy_party_indices
                .iter()
                .copied()
                .collect(),
            escape_attempts: state.battle_escape_attempts,
            player_mist_active: combat.is_some_and(|combat| combat.player_mist_active),
            pay_day_money: state.battle_pay_day_money,
            amulet_coin_active: state.battle_amulet_coin_active,
            trainer_items_used: state
                .script_runtime
                .active_battle_combat
                .as_ref()
                .map(|combat| combat.trainer_items_used.clone())
                .unwrap_or_default(),
            player_disabled_move: state
                .script_runtime
                .active_battle_combat
                .as_ref()
                .and_then(|combat| combat.player_disable.as_ref())
                .filter(|disable| disable.turns_remaining > 0)
                .map(|disable| disable.move_name.clone()),
            commands,
        })
    }
}

fn battle_music_from_kind(kind: &RuntimeBattleKind) -> String {
    match kind {
        RuntimeBattleKind::Wild { battle_music, .. }
        | RuntimeBattleKind::StaticWild { battle_music, .. } => battle_music.clone(),
        RuntimeBattleKind::Trainer {
            encounter_music, ..
        } => encounter_music.clone(),
    }
}

impl RuntimeBattleCommandSnapshot {
    fn from_state(
        state: &GameState,
        kind: &RuntimeBattleKind,
        enemy_pokemon: &Pokemon,
    ) -> Result<Self> {
        let player_disable = state
            .script_runtime
            .active_battle_combat
            .as_ref()
            .and_then(|combat| combat.player_disable.as_ref())
            .filter(|disable| disable.turns_remaining > 0)
            .map(|disable| disable.move_name.as_str());
        let mut player_forced_struggle = false;
        let player_turn_automatic = state
            .script_runtime
            .active_battle_combat
            .as_ref()
            .is_some_and(|combat| {
                (combat.player.hp > 0 || combat.player_spikes_zero_hp_unchecked)
                    && (combat.player_recharge_move.is_some()
                        || combat.player_airborne_move.is_some()
                        || combat.player_charging_move.is_some()
                        || combat.player.rampage_turns > 0
                        || combat.player_rollout_turns > 0)
            });
        let player_fight_automatic = state
            .script_runtime
            .active_battle_combat
            .as_ref()
            .is_some_and(|combat| {
                (combat.player.hp > 0 || combat.player_spikes_zero_hp_unchecked)
                    && combat.player_bide_turns > 0
            });
        let player_move_slots = state
            .battle_active_party_index
            .map(|index| {
                let pokemon = state
                    .storage
                    .party
                    .pokemon
                    .get(index)
                    .with_context(|| {
                        format!("active battle party index {index} is outside saved party")
                    })?
                    .as_ref()
                    .with_context(|| {
                        format!("active battle party index {index} points to an empty party slot")
                    })?;
                let moves = state
                    .script_runtime
                    .active_battle_combat
                    .as_ref()
                    .map(|combat| {
                        combat
                            .player_transform
                            .as_ref()
                            .map(|transform| transform.moves.as_slice())
                            .unwrap_or(combat.player.moves.as_slice())
                    })
                    .unwrap_or(pokemon.moves.as_slice());
                player_forced_struggle = !moves.iter().take(BATTLE_MOVE_SLOTS).any(|learned| {
                    learned.current_pp > 0 && player_disable != Some(learned.name.as_str())
                });
                Ok::<Vec<usize>, anyhow::Error>((0..moves.len().min(BATTLE_MOVE_SLOTS)).collect())
            })
            .transpose()?
            .with_context(|| "active battle snapshot is missing active player party index")?;
        let enemy_moves = state
            .script_runtime
            .active_battle_combat
            .as_ref()
            .map(|combat| {
                combat
                    .enemy_transform
                    .as_ref()
                    .map(|transform| transform.moves.as_slice())
                    .unwrap_or(combat.enemy.moves.as_slice())
            })
            .unwrap_or(enemy_pokemon.moves.as_slice());
        let enemy_disable = state
            .script_runtime
            .active_battle_combat
            .as_ref()
            .and_then(|combat| combat.enemy_disable.as_ref())
            .filter(|disable| disable.turns_remaining > 0)
            .map(|disable| disable.move_name.as_str());
        let enemy_move_slots = available_learned_move_slots(enemy_moves, enemy_disable);
        let switch_party_indices = state
            .storage
            .party
            .pokemon
            .iter()
            .enumerate()
            .filter_map(|(index, pokemon)| {
                let pokemon = pokemon.as_ref()?;
                (Some(index) != state.battle_active_party_index && pokemon.hp > 0).then_some(index)
            })
            .collect();
        Ok(Self {
            player_move_slots,
            player_forced_struggle,
            player_turn_automatic,
            player_fight_automatic,
            enemy_move_slots,
            switch_party_indices,
            can_use_items: state.battle_active_party_index.is_some(),
            can_run: matches!(
                kind,
                RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. }
            ),
        })
    }
}

fn available_learned_move_slots(moves: &[LearnedMove], disabled_move: Option<&str>) -> Vec<usize> {
    moves
        .iter()
        .take(BATTLE_MOVE_SLOTS)
        .enumerate()
        .filter_map(|(slot, learned)| {
            (learned.current_pp > 0 && disabled_move != Some(learned.name.as_str())).then_some(slot)
        })
        .collect()
}

pub fn validate_deterministic_replay_runtime_authority(
    bundle: &DeterministicReplayBundle,
    player_id: PlayerId,
) -> Result<()> {
    bundle
        .validate()
        .context("validate deterministic replay framing")?;
    let journal = bundle.input_journal().journal();
    let mut covered_input_frames = BTreeSet::new();
    for command in bundle.runtime_commands() {
        let request = command.command();
        let decoded =
            decode_runtime_mutation_command_payload(request.payload()).with_context(|| {
                format!(
                    "decode deterministic runtime command sequence {}",
                    request.sequence()
                )
            })?;
        if let RuntimeMutationCommand::ApplyOverworldInput(input) = decoded {
            let frame = request.expected_state().frame();
            let journal_frame = journal
                .frames()
                .iter()
                .find(|candidate| candidate.frame() == frame)
                .with_context(|| {
                    format!(
                        "ApplyOverworldInput sequence {} has no input-journal frame {frame}",
                        request.sequence()
                    )
                })?;
            let expected_mask = journal_frame.joypad_mask_for(player_id).with_context(|| {
                format!("runtime input journal frame {frame} is missing local player {player_id}")
            })?;
            let actual_mask = JoypadState::compute_mask(input.buttons);
            if actual_mask != expected_mask {
                anyhow::bail!(
                    "ApplyOverworldInput sequence {} mask {actual_mask:#04x} does not match journal frame {frame} mask {expected_mask:#04x}",
                    request.sequence()
                );
            }
            if !covered_input_frames.insert(frame) {
                anyhow::bail!(
                    "input-journal frame {frame} is covered by more than one ApplyOverworldInput command"
                );
            }
        }
    }
    for frame in journal.frames() {
        if !covered_input_frames.contains(&frame.frame()) {
            anyhow::bail!(
                "input-journal frame {} has no ApplyOverworldInput command",
                frame.frame()
            );
        }
    }
    Ok(())
}

fn runtime_party_recovery(
    outcome: PartyRecoveryOutcome,
    state_checksum: StateChecksum,
) -> RuntimePartyRecovery {
    RuntimePartyRecovery {
        party_index: outcome.party_index,
        species_id: outcome.species_id,
        hp_before: outcome.hp_before,
        hp_after: outcome.hp_after,
        status_before: outcome.status_before,
        status_after: outcome.status_after,
        pp_restored: outcome.pp_restored,
        state_checksum,
    }
}

fn runtime_blackout_recovery(
    outcome: BlackoutRecoveryOutcome,
    state_checksum: StateChecksum,
) -> RuntimeBlackoutRecovery {
    RuntimeBlackoutRecovery {
        spawn_identifier: outcome.spawn_identifier,
        map_name: outcome.map_name,
        tile: outcome.tile,
        healed: outcome
            .healed
            .into_iter()
            .map(|healed| runtime_party_recovery(healed, state_checksum.clone()))
            .collect(),
        state_checksum,
    }
}

impl RuntimePartySnapshot {
    fn from_state(state: &GameState) -> Self {
        Self {
            slots: state
                .storage
                .party
                .pokemon
                .iter()
                .enumerate()
                .filter_map(|(index, pokemon)| {
                    pokemon.clone().map(|pokemon| RuntimePartySlotSnapshot {
                        index,
                        pokemon,
                        is_active_battle_pokemon: state.battle_active_party_index == Some(index),
                    })
                })
                .collect(),
            active_battle_slot: state.battle_active_party_index,
        }
    }
}

impl RuntimeStorageSnapshot {
    fn from_state(state: &GameState) -> Self {
        Self {
            current_pc_box: state.current_pc_box,
            party_count: state.storage.party.filled_slots(),
            boxes: state
                .storage
                .pc_boxes
                .iter()
                .enumerate()
                .map(RuntimePcBoxSnapshot::from_box)
                .collect(),
        }
    }
}

impl RuntimePcBoxSnapshot {
    fn from_box((index, pc_box): (usize, &PcBox)) -> Self {
        Self {
            index,
            name: pc_box.name.clone(),
            count: pc_box.count,
            slots: pc_box
                .pokemon
                .iter()
                .enumerate()
                .filter_map(|(slot, pokemon)| {
                    pokemon.clone().map(|pokemon| RuntimePcBoxSlotSnapshot {
                        index: slot,
                        pokemon,
                    })
                })
                .collect(),
        }
    }
}

impl RuntimeBagSnapshot {
    fn inventory<'a>(
        inventory: impl IntoIterator<Item = (&'a String, &'a u16)>,
    ) -> Vec<RuntimeBagItemSnapshot> {
        inventory
            .into_iter()
            .map(|(item_id, quantity)| RuntimeBagItemSnapshot {
                item_id: item_id.clone(),
                quantity: *quantity,
            })
            .collect()
    }

    fn tm_hm(
        items: &BTreeMap<String, Item>,
        quantities: &[u8],
    ) -> Result<Vec<RuntimeTmHmSnapshot>> {
        let mut entries = Vec::new();
        for (item_id, item) in items {
            if item.pocket != ITEM_POCKET_TM_HM {
                continue;
            }
            let Some(index) = item.tmhm_index else {
                continue;
            };
            let quantity = quantities.get(index).copied().with_context(|| {
                format!(
                    "saved TM/HM flags missing index {index} required by compiled item {item_id}"
                )
            })?;
            if quantity > 0 {
                entries.push(RuntimeTmHmSnapshot {
                    item_id: item_id.clone(),
                    tmhm_index: index,
                    move_id: item.tmhm_move.clone(),
                    quantity: u16::from(quantity),
                });
            }
        }
        entries.sort_by(|left, right| {
            left.tmhm_index
                .cmp(&right.tmhm_index)
                .then_with(|| left.item_id.cmp(&right.item_id))
        });
        Ok(entries)
    }
}

impl RuntimeItemCatalogSnapshot {
    fn from_item(
        (item_id, item): (&String, &Item),
        evolutions: &crystal_core::systems::evolution::EvolutionTable,
    ) -> Self {
        Self {
            item_id: item_id.clone(),
            name: item.name.clone(),
            description: item.description.clone(),
            effect: item.effect.clone(),
            status_heals: item.status_heals.clone(),
            revive_hp_percent: item.revive_hp_percent,
            party_revive_hp_percent: item.party_revive_hp_percent,
            pp_restore_scope: item.pp_restore_scope.clone(),
            pp_restore_points: item.pp_restore_points,
            pp_up_stages: item.pp_up_stages,
            vitamin_stat: item.vitamin_stat.clone(),
            vitamin_stat_exp: item.vitamin_stat_exp,
            vitamin_max_stat_exp: item.vitamin_max_stat_exp,
            rare_candy_level_gain: item.rare_candy_level_gain,
            party_special_effect: party_special_item_effect_plan(item, evolutions).is_some(),
            battle_stat_boost_stat: item.battle_stat_boost_stat.clone(),
            battle_stat_boost_stages: item.battle_stat_boost_stages,
            battle_escape_mode: item.battle_escape_mode.clone(),
            battle_focus_energy: item.battle_focus_energy,
            battle_stat_drop_guard: item.battle_stat_drop_guard,
            confusion_heal: item.confusion_heal,
            repel_steps: item.repel_steps,
            escape_rope_mode: item.escape_rope_mode.clone(),
            price: item.price,
            held_effect: item.held_effect.clone(),
            parameter: item.parameter,
            property: item.property.clone(),
            pocket: item.pocket.clone(),
            field_menu: item.field_menu.clone(),
            field_usable: item.field_usable,
            battle_menu: item.battle_menu.clone(),
            battle_usable: item.battle_usable,
            script_name: item.script_name.clone(),
            consumable: item.consumable,
            tmhm_index: item.tmhm_index,
            tmhm_move: item.tmhm_move.clone(),
        }
    }
}

impl RuntimeItemEffectPlanKey {
    fn from_plan(plan: BattleItemEffectPlan) -> Self {
        Self {
            item_id: plan.item_id,
            effect_id: plan.effect_id,
            behavior_id: plan.behavior_id,
        }
    }
}

impl RuntimeMoveCatalogSnapshot {
    fn from_move((move_id, move_data): (&String, &Move)) -> Self {
        Self {
            move_id: move_id.clone(),
            source_index: move_data.source_index,
            name: move_data.name.clone(),
            move_type: move_data.move_type.clone(),
            power: move_data.power,
            accuracy: move_data.accuracy,
            pp: move_data.pp,
            effect: move_data.effect.clone(),
            effect_chance: move_data.effect_chance,
            stat: move_data.stat.clone(),
            amount: move_data.amount,
        }
    }
}

impl RuntimePokemonCatalogSnapshot {
    fn from_species((species_id, species): (&String, &PokemonSpecies)) -> Self {
        Self {
            species_id: species_id.clone(),
            int_id: species.int_id,
            base_stats: species.base_stats,
            type1: species.type1.clone(),
            type2: species.type2.clone(),
            catch_rate: species.catch_rate,
            base_exp: species.base_exp,
            item1: species.item1.clone(),
            item2: species.item2.clone(),
            gender_ratio: species.gender_ratio,
            unknown1: species.unknown1,
            step_cycles_to_hatch: species.step_cycles_to_hatch,
            unknown2: species.unknown2,
            growth_rate: species.growth_rate.clone(),
            egg_group1: species.egg_group1.clone(),
            egg_group2: species.egg_group2.clone(),
            tmhm_learnset: species.tmhm_learnset.clone(),
            ability: species.ability.clone(),
            pic_size: species.pic_size,
            front_pic: species.front_pic,
            back_pic: species.back_pic,
            weight: species.weight,
        }
    }
}

impl RuntimeTrainerCatalogSnapshot {
    fn from_trainer(trainer: &Trainer) -> Self {
        Self {
            trainer_id: trainer.trainer_id.clone(),
            name: trainer.name.clone(),
            trainer_class: trainer.trainer_class.clone(),
            party: trainer
                .party
                .iter()
                .map(RuntimeTrainerPartyPokemonSnapshot::from_party_pokemon)
                .collect(),
            win_quote: trainer.win_quote.clone(),
            lose_quote: trainer.lose_quote.clone(),
            items: trainer.items.clone(),
            base_reward: trainer.base_reward,
            ai_move_flags: trainer.ai_move_flags,
            ai_item_switch_flags: trainer.ai_item_switch_flags,
            encounter_music: trainer.encounter_music.clone(),
            ai_layers: trainer.ai_layers.clone(),
        }
    }
}

impl RuntimeTrainerPartyPokemonSnapshot {
    fn from_party_pokemon(pokemon: &crystal_core::models::TrainerPartyPokemon) -> Self {
        Self {
            species: pokemon.species.clone(),
            level: pokemon.level,
            item: pokemon.item.clone(),
            moves: pokemon
                .moves
                .iter()
                .map(RuntimeLearnedMoveSnapshot::from_learned_move)
                .collect(),
            dvs: pokemon.dvs,
        }
    }
}

impl RuntimeLearnedMoveSnapshot {
    fn from_learned_move(move_data: &crystal_core::models::LearnedMove) -> Self {
        Self {
            name: move_data.name.clone(),
            current_pp: move_data.current_pp,
            pp_ups: move_data.pp_ups,
        }
    }
}

impl RuntimeMapCatalogSnapshot {
    fn from_module(
        map_name: &str,
        module: &crystal_assets::MapModule,
        metadata: Option<&crystal_assets::RuntimeMapMetadata>,
    ) -> Self {
        Self {
            map_name: map_name.to_string(),
            id: module.id.clone(),
            attributes: module.attributes.clone(),
            metadata: metadata.map(RuntimeMapMetadataSnapshot::from_metadata),
            scenes: module.scenes.clone(),
            events: module.events.clone(),
            objects: module.objects.clone(),
            blocks: module.blocks.clone(),
        }
    }
}

impl RuntimeMapMetadataSnapshot {
    fn from_metadata(metadata: &crystal_assets::RuntimeMapMetadata) -> Self {
        Self {
            constant: metadata.constant.clone(),
            name: metadata.name.clone(),
            group_name: metadata.group_name.clone(),
            group_id: metadata.group_id,
            map_id: metadata.map_id,
            width: metadata.width,
            height: metadata.height,
            environment: metadata.environment.clone(),
            phone_service: metadata.phone_service,
        }
    }
}

impl RuntimeTilesetCatalogSnapshot {
    fn from_tileset((tileset_id, tileset): (&String, &TilesetDefinition)) -> Self {
        Self {
            tileset_id: tileset_id.clone(),
            collision: tileset.collision.clone(),
            palette_map: tileset.palette_map.clone(),
        }
    }
}

impl RuntimeEncounterCatalogSnapshot {
    fn from_data(data: &GameDataSet) -> Self {
        Self {
            wild: data.wild_encounters.clone(),
            field: data.field_encounters.clone(),
            slot_tables: data.encounter_slot_tables.clone(),
            fishing: data.fishing.clone(),
        }
    }
}

impl RuntimeBattleRuleCatalogSnapshot {
    fn from_data(data: &GameDataSet) -> Self {
        Self {
            capture_rules: data.capture_rules.clone(),
            capture_wobble_probabilities: data.capture_wobble_probabilities.clone(),
            stat_multipliers: data.battle_stat_multipliers.clone(),
            move_priorities: data.move_priorities.clone(),
            type_categories: data.type_categories.clone(),
            type_effectiveness: data.type_effectiveness.clone(),
            weather_modifiers: data.weather_modifiers.clone(),
            reward_rules: data.battle_reward_rules.clone(),
            escape_rules: data.battle_escape_rules.clone(),
        }
    }
}

impl RuntimeWorldRuleCatalogSnapshot {
    fn from_data(data: &GameDataSet) -> Self {
        Self {
            marts: data.marts.clone(),
            currency: data.currency_constants.clone(),
            fruit_trees: data.fruit_trees.clone(),
            field_moves: data.field_moves.clone(),
        }
    }
}

impl RuntimePresentationCatalogSnapshot {
    fn from_data(data: &GameDataSet) -> Self {
        Self {
            title_screen: data.runtime_title_screen.clone(),
            pc_strings: data.pc_strings.clone(),
            menu_icons: data.menu_icons.clone(),
            pokedex_entries: data.pokedex_entries.clone(),
            pokemon_frontpic_anim: data.pokemon_frontpic_anim.clone(),
            asm_text: data.asm_text.clone(),
            move_names: data.move_names.clone(),
            battle_animations: data.battle_animations.clone(),
            battle_animation_table: data.battle_animation_table.clone(),
            battle_anim_bundle: data.battle_anim_bundle.clone(),
            sprite_anim_bundle: data.sprite_anim_bundle.clone(),
            sprite_palette_defaults: data.sprite_palette_defaults.clone(),
            pokegear_town_map_palette_map: data.pokegear_town_map_palette_map.clone(),
            pokegear_landmarks: data.pokegear_landmarks.clone(),
            pokemon_cries: data.pokemon_cries.clone(),
        }
    }
}

impl RuntimeSpecialCatalogSnapshot {
    fn from_data(data: &GameDataSet) -> Self {
        Self {
            phone_contacts: data.phone_contacts.clone(),
            permanent_phone_numbers: data.permanent_phone_numbers.clone(),
            special_phone_calls: data.special_phone_calls.clone(),
            npc_trades: data.npc_trades.clone(),
            special_routines: data.special_routines.clone(),
            flee_mons: data.flee_mons.clone(),
            buena_password_categories: data.buena_password_categories.clone(),
            roaming_pokemon: data.roaming_pokemon.clone(),
            buena_prizes: data.buena_prizes.clone(),
            kurt_apricorn_recipes: data.kurt_apricorn_recipes.clone(),
            shuckie_gift: data.shuckie_gift.clone(),
            dratini_move_sets: data.dratini_move_sets.clone(),
            bug_contest_config: data.bug_contest_config.clone(),
            battle_tower_rules: data.battle_tower_rules.clone(),
            oak_ratings: data.oak_ratings.clone(),
            odd_egg_definitions: data.odd_egg_definitions.clone(),
            magikarp_lengths: data.magikarp_lengths.clone(),
            happiness_data: data.happiness_data.clone(),
        }
    }
}

impl RuntimeStoryCatalogSnapshot {
    fn from_data(data: &GameDataSet) -> Self {
        Self {
            initialize_events: data.initialize_events.clone(),
            story_event_script_constants: data.story_event_script_constants.clone(),
        }
    }
}

impl RuntimeAudioCatalogSnapshot {
    fn from_catalog(catalog: &RuntimeAudioCatalog) -> Self {
        Self {
            manifest: catalog.manifest.clone(),
            playback: catalog.playback.clone(),
            music: catalog
                .music
                .iter()
                .map(|(id, program)| {
                    (
                        id.clone(),
                        RuntimeAudioProgramSnapshot::from_program(program),
                    )
                })
                .collect(),
            sound_effects: catalog
                .sound_effects
                .iter()
                .map(|(id, program)| {
                    (
                        id.clone(),
                        RuntimeAudioProgramSnapshot::from_program(program),
                    )
                })
                .collect(),
            cries: catalog
                .cries
                .iter()
                .map(|(id, program)| {
                    (
                        id.clone(),
                        RuntimeAudioProgramSnapshot::from_program(program),
                    )
                })
                .collect(),
        }
    }
}

impl RuntimeAudioProgramSnapshot {
    fn from_program(program: &AudioProgram) -> Self {
        Self {
            cache_key: program.cache_key.clone(),
            source: RuntimeAudioProgramSourceSnapshot::from_source(&program.source),
        }
    }
}

impl RuntimeAudioProgramSourceSnapshot {
    fn from_source(source: &AudioProgramSource) -> Self {
        match source {
            AudioProgramSource::Pcm {
                bytes,
                format,
                loop_start_sample,
                loop_end_sample,
            } => Self::Pcm {
                byte_len: bytes.len(),
                format: format.clone(),
                loop_start_sample: *loop_start_sample,
                loop_end_sample: *loop_end_sample,
            },
            AudioProgramSource::PcmGzip {
                bytes,
                format,
                loop_start_sample,
                loop_end_sample,
                ..
            } => Self::PcmGzip {
                byte_len: bytes.len(),
                format: format.clone(),
                loop_start_sample: *loop_start_sample,
                loop_end_sample: *loop_end_sample,
            },
            AudioProgramSource::Midi {
                midi_base64,
                format,
                byte_len,
                loop_start_sample,
                loop_end_sample,
                ..
            } => Self::Midi {
                midi_base64_len: midi_base64.len(),
                byte_len: *byte_len,
                format: format.clone(),
                loop_start_sample: *loop_start_sample,
                loop_end_sample: *loop_end_sample,
            },
        }
    }
}

fn battle_tower_opponent_is_staged(state: &GameState) -> bool {
    state
        .script_runtime
        .variables
        .get("_battle_tower_opponent_pending")
        .is_some_and(|value| value == "1")
}

impl RuntimeShellPhase {
    fn from_state(state: &GameState) -> Self {
        if state.script_runtime.pending_yes_no.is_some() {
            Self::YesNo
        } else if state.script_runtime.pending_text_wait.is_some()
            || state.script_runtime.pending_text_label.is_some()
            || state.script_runtime.text_window_open
        {
            Self::Text
        } else if state.script_runtime.pending_shop.is_some() {
            Self::Shop
        } else if state.script_runtime.active_menu.is_some() {
            Self::Menu
        } else if battle_tower_opponent_is_staged(state) {
            Self::Overworld
        } else {
            match state.battle {
                BattleMemory::Inactive => Self::Overworld,
                BattleMemory::Wild { .. } => Self::WildBattle,
                BattleMemory::StaticWild { .. } => Self::StaticWildBattle,
                BattleMemory::Trainer { .. } => Self::TrainerBattle,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptedBattleCompletion {
    pub continued_after_battle: bool,
    pub trainer_prize_money: Option<u32>,
    pub money_after: Option<u32>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCaptureCompletion {
    pub stored: Option<StoredCapture>,
    pub contest_pokemon: Option<crystal_core::models::Pokemon>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCaptureAttempt {
    pub outcome: Option<CaptureOutcome>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleTurn {
    pub outcome: BattleTurnOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleItemTurn {
    pub item_use: ItemUseOutcome,
    pub battle_item: BattleItemOutcome,
    pub enemy_action: BattleAction,
    pub turn: RuntimeBattleTurn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleBallTurn {
    pub capture: CaptureOutcome,
    pub enemy_action: BattleAction,
    pub turn: RuntimeBattleTurn,
}

fn player_battle_item_outcome(outcome: &BattleTurnOutcome) -> Result<BattleItemOutcome> {
    outcome
        .events
        .iter()
        .find_map(|event| match event {
            crystal_core::battle::turn::BattleEvent::BattleItemEffect {
                side: crystal_core::battle::turn::BattleSide::Player,
                outcome,
            }
            | crystal_core::battle::turn::BattleEvent::BattlePartyItemEffect {
                side: crystal_core::battle::turn::BattleSide::Player,
                outcome,
                ..
            } => Some(outcome.clone()),
            _ => None,
        })
        .context("battle item turn emitted no player item effect")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleCommand {
    pub outcome: ActiveBattleCommandOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattlePartySwitch {
    pub party_index: usize,
    pub spikes: Option<crystal_core::battle::turn::SwitchInSpikesOutcome>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleEscape {
    pub outcome: BattleEscapeAttempt,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleEscapeItemUse {
    pub item_use: ItemUseOutcome,
    pub battle_escape_mode: String,
    pub escaped: bool,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleStateItemUse {
    pub item_use: ItemUseOutcome,
    pub mist_active_before: bool,
    pub mist_active_after: bool,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleRewards {
    pub outcome: BattleRewardOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTrainerBattleAdvance {
    pub next_enemy: Option<crystal_core::models::Pokemon>,
    pub trainer_defeated: bool,
    pub spikes: Option<crystal_core::battle::turn::SwitchInSpikesOutcome>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFishingCast {
    pub session: FishingSession,
    pub bite: Option<bool>,
    pub wild_battle: Option<WildBattleStart>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFishingRodItemUse {
    pub item_use: ItemUseOutcome,
    pub rod: String,
    pub cast: RuntimeFishingCast,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTimeUpdate {
    pub time_of_day: TimeOfDay,
    pub day_of_week: u8,
    pub hour: u8,
    pub minute: u8,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeGiftPokemonGrant {
    pub outcome: GiftPokemonOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldMoveBlockUse {
    pub outcome: FieldMoveBlockOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldMoveFlagUse {
    pub outcome: FieldMoveFlagOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldMoveTravelUse {
    pub outcome: FieldMoveTravelOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFlyFieldMoveUse {
    pub actor_party_index: usize,
    pub actor_species: String,
    pub flypoint_flag: String,
    pub source_map: String,
    pub destination_spawn_identifier: u16,
    pub destination_map: String,
    pub destination_tile: TilePosition,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTeleportFieldMoveUse {
    pub actor_party_index: usize,
    pub actor_species: String,
    pub source_map: String,
    pub destination_spawn_identifier: u16,
    pub destination_map: String,
    pub destination_tile: TilePosition,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeItemUse {
    pub outcome: ItemUseOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRegisteredKeyItem {
    pub outcome: RuntimeRegisteredKeyItemOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBattleItemUse {
    pub item_use: ItemUseOutcome,
    pub battle_item: BattleItemOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePartyItemUse {
    pub item_use: ItemUseOutcome,
    pub item_effect: BattleItemOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWholePartyItemUse {
    pub item_use: ItemUseOutcome,
    pub item_effect: PartyItemOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTmHmItemUse {
    pub item_use: ItemUseOutcome,
    pub learned_move: TmHmLearnOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRepelItemUse {
    pub item_use: ItemUseOutcome,
    pub repel_steps_before: u16,
    pub repel_steps_after: u16,
    pub active_repel_item_before: Option<String>,
    pub active_repel_item_after: Option<String>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBicycleItemUse {
    pub item_use: ItemUseOutcome,
    pub map_name: String,
    pub permission: u8,
    pub mode_before: MovementMode,
    pub mode_after: MovementMode,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeItemfinderUse {
    pub item_use: ItemUseOutcome,
    pub player_tile: TilePosition,
    pub found: Option<CoreItemfinderHiddenItem>,
    pub itemfinder_sound_cues: usize,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSquirtBottleUse {
    pub item_use: ItemUseOutcome,
    pub player_tile: TilePosition,
    pub target_tile: TilePosition,
    pub target_object_identifier: Option<String>,
    pub target_movement: String,
    pub target_script: Option<String>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStoryKeyUse {
    pub item_use: ItemUseOutcome,
    pub map_name: String,
    pub player_tile: TilePosition,
    pub target_tile: TilePosition,
    pub target_script: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeKeyItemBalanceUse {
    pub item_use: ItemUseOutcome,
    pub balance_label: String,
    pub balance: u32,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTownMapUse {
    pub item_use: ItemUseOutcome,
    pub map_name: String,
    pub map_constant: String,
    pub environment: String,
    pub landmark: PokegearLandmark,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePokegearUse {
    pub item_use: ItemUseOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBoxItemUse {
    pub item_use: ItemUseOutcome,
    pub decoration_flag: String,
    pub already_owned: bool,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEscapeRopeUse {
    pub item_use: ItemUseOutcome,
    pub source_map: String,
    pub destination_map: String,
    pub destination_warp_index: u16,
    pub destination_tile: TilePosition,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDigFieldMoveUse {
    pub actor_party_index: usize,
    pub actor_species: String,
    pub source_map: String,
    pub destination_map: String,
    pub destination_warp_index: u16,
    pub destination_tile: TilePosition,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptItemGrant {
    pub outcome: ScriptItemGrantOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptItemCheck {
    pub outcome: ScriptItemCheckOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptItemTake {
    pub outcome: ScriptItemTakeOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSpecialRoutineUse {
    pub outcome: SpecialRoutineOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldPickup {
    pub outcome: FieldItemPickupOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptEconomy {
    pub outcome: ScriptEconomyOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePhoneCommand {
    pub outcome: ScriptPhoneOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePermanentPhoneNumbers {
    pub inserted: Vec<String>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFlagMutation {
    pub outcome: ScriptFlagMutationOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFlagCheck {
    pub outcome: ScriptFlagCheckOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSceneCommand {
    pub outcome: ScriptSceneOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBlockChange {
    pub outcome: ScriptBlockChangeOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptAudio {
    pub cue: ScriptAudioCue,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptMapCommand {
    pub action: ScriptMapAction,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptWarp {
    pub target_map: String,
    pub tile: TilePosition,
    pub facing: Option<Direction>,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptText {
    pub action: ScriptTextAction,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptVariable {
    pub outcome: ScriptVariableOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptSwarm {
    pub outcome: ScriptSwarmOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptControl {
    pub action: ScriptControlAction,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptObjectMutation {
    pub outcome: ScriptObjectMutationOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptMovement {
    pub outcome: ScriptMovementOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptRuntimeCommand {
    pub outcome: ScriptRuntimeOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeQueuedScriptCommand {
    pub queued: ScriptRuntimeQueuedCommand,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeNextScript {
    pub origin_map_name: String,
    pub script: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptReturnResume {
    pub frame: ScriptReturnFrame,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDeferredScript {
    pub origin_map_name: String,
    pub script: String,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptEnd {
    pub end: ScriptEndState,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptShop {
    pub outcome: ScriptShopOutcome,
    pub state_checksum: StateChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeShopTransaction {
    pub outcome: ShopResult,
    pub state_checksum: StateChecksum,
}

include!("runtime_catalog.rs");

include!("runtime_session.rs");

#[cfg(any(test, feature = "test-fixtures"))]
#[allow(dead_code, unused_imports)]
#[path = "runtime_tests.rs"]
pub mod test_support;
