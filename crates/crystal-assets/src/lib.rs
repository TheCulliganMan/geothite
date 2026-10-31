#![recursion_limit = "256"]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, Result};
#[cfg(any(test, feature = "test-fixtures"))]
use crystal_core::battle::capture::throw_ball_from_bag as core_throw_ball_from_bag;
use crystal_core::battle::capture::{
    CaptureAttemptContext, CaptureBallRule, CaptureBallRuleIssue, CaptureCompletion,
    CaptureOutcome, CaptureRulesIssue, CaptureWobbleProbability, CaptureWobbleProbabilityIssue,
    capture_rules_issues, capture_wobble_probability_issues,
    complete_active_wild_capture_result_with_transformed_replacement as core_complete_active_wild_capture,
    resolve_capture_attempt_exact as core_resolve_capture_attempt_exact,
    resolve_capture_attempt_with_battle_rng as core_resolve_capture_attempt_with_battle_rng,
    validate_capture_ball_item,
};
use crystal_core::battle::damage::{
    TypeCategories, TypeCategoryIssue, TypeEffectivenessTable, TypeEffectivenessTableIssue,
    TypeEffectivenessTableKind, WeatherModifierIssue, WeatherModifiers, type_category_issues,
    type_effectiveness_table_issues, weather_modifier_issues,
};
use crystal_core::battle::start::{
    ActiveBattlePartySwitchOutcome, StaticWildBattleOrigin, StaticWildBattleRequest,
    StaticWildBattleStart, TrainerBattleAdvanceOutcome, TrainerBattleCompletion,
    TrainerBattleCompletionOutcome, TrainerBattleRequest, TrainerBattleStartStatus,
    WildBattleStart, activate_static_wild_battle_start, activate_trainer_battle_start_status,
    activate_wild_battle_start,
    advance_active_trainer_battle as core_advance_active_trainer_battle,
    complete_trainer_battle as core_complete_trainer_battle, first_available_battle_party_index,
    materialize_non_roaming_wild_battle_with_rng, materialize_roaming_wild_battle_with_rng,
    materialize_staged_roaming_wild_battle_with_rng, materialize_trainer_party,
    require_active_battle_enemy_party_index, require_active_battle_party_index,
    require_active_battle_party_slot_index, static_wild_battle_start,
    switch_active_battle_party_index, trainer_battle_start,
};
use crystal_core::battle::stats::{
    BattleStatMultiplier, BattleStatMultiplierTableIssue, BattleStatMultiplierTables,
    battle_stat_multiplier_table_issues,
};
use crystal_core::battle::turn::{
    BattleAction, BattleCombatState, BattleEvent, BattleSide, BattleTurnError, BattleTurnInput,
    BattleTurnOutcome, EnemyMoveSelection, EnemyMoveSelector, EnemyPostOrderActionSelector,
    MovePriorityTable, MovePriorityTableIssue, active_battle_combat_state,
    apply_active_battle_item_effect_to_combat, battle_action_locked_before_menu,
    battle_move_effect_is_supported, battle_moves, battle_speed, commit_battle_turn_outcome,
    commit_wild_battle_escape_attempt, move_priority_table_issues,
    resolve_battle_enemy_action_with_items as core_resolve_battle_enemy_action_with_items,
    resolve_battle_turn_with_ball_action_and_enemy_ai_actions as core_resolve_battle_turn_with_ball_action_and_enemy_ai_actions,
    resolve_battle_turn_with_enemy_ai_actions as core_resolve_battle_turn_with_enemy_ai_actions,
    resolve_battle_turn_with_items as core_resolve_battle_turn_with_items,
    resolve_wild_battle_run as core_resolve_wild_battle_run,
    resolve_wild_battle_turn_with_enemy_ai_actions as core_resolve_wild_battle_turn_with_enemy_ai_actions,
    resolve_wild_battle_turn_with_items as core_resolve_wild_battle_turn_with_items,
    select_wild_enemy_move_slot as core_select_wild_enemy_move_slot,
};
use crystal_core::input::{B_PAD_A, B_PAD_B, GameButton, JoypadState, direction_from_pad_mask};
use crystal_core::map::{
    BackgroundEvent, CoordEvent, MapAttributes, MapConnection, MapEventSectionCommand,
    MapEventSectionCommandIssue, MapEvents, MapScene, MapSceneTable, MapScriptSectionCommand,
    MapScriptSectionCommandIssue, ObjectEvent, WarpEvent, map_event_section_command_arg_counts,
    map_event_section_command_issues, map_script_section_command_arg_counts,
    map_script_section_command_issues,
};
pub use crystal_core::models::RuntimePokedexEntry;
#[cfg(test)]
use crystal_core::models::TrainerPartyPokemon;
use crystal_core::models::pokemon::CaughtData;
use crystal_core::models::{
    Bag, BattleAnimationCatalogIssue, Dv, FrontpicAnimCatalogIssue, FrontpicAnimCommand,
    FrontpicAnimCommandIssue, FrontpicAnimProgram, ITEM_POCKET_BALL, ITEM_POCKET_ITEM,
    ITEM_POCKET_KEY_ITEM, ITEM_POCKET_TM_HM, Item, LearnedMove, MAX_BOX_MONS, MAX_PC_BOXES,
    MenuIconCatalogIssue, Move, MoveNameCatalogIssue, MovePayloadIssue,
    MoveSourceIndexCatalogIssue, Party, PcBox, PcStringCatalogIssue, PokedexEntryCatalogIssue,
    PokedexState, PokegearLandmarkIssue, PokegearTownMapPaletteIssue, Pokemon, PokemonSpecies,
    PokemonStorage, RuntimeBundleIssue, SpritePaletteDefaultIssue, Trainer, TrainerCatalog,
    TrainerCatalogIssue, battle_animation_catalog_issues, calculate_stats,
    create_pokemon_from_known_dvs, frontpic_anim_catalog_issues, frontpic_anim_command_issue,
    menu_icon_catalog_issues, move_name_catalog_issues, move_payload_issues,
    move_source_index_catalog_issues, pc_string_catalog_issues, pokedex_entry_catalog_issues,
    pokegear_landmark_issues, pokegear_town_map_palette_issues, pokemon_species_display_name,
    runtime_bundle_issues, sprite_palette_default_issues, trainer_catalog_issues, trainer_key,
    validate_saved_bag_pocket_references, validate_saved_pc_item_references,
    validate_saved_pokedex_references,
};
pub use crystal_core::models::{PokegearLandmark, PokegearLandmarksPayload};
use crystal_core::multiplayer::{
    PlayerId, RuntimeCommandFrame, RuntimeCommandFrameError, RuntimeCommandPayload,
    RuntimeCommandResultFrame, StateChecksum, StateChecksumFrame, fnv1a32_bytes,
    game_state_checksum, game_state_checksum_unchecked,
};
#[cfg(any(test, feature = "test-fixtures"))]
use crystal_core::random::Random;
use crystal_core::random::{
    BattleRandomSource, CrystalRandom, DividerSource, ExactBattleRandom, ReplayDivider,
    RuntimeDividerSource,
};
use crystal_core::save::SaveModpackIdentity;
use crystal_core::state::{
    BattleMemory, BattleTowerState, BuenasPasswordState, BugContestState, DayCareInput,
    DayCareState, EventFlagMemory, FishingMemory, GameState, ItemUseRuntimeEvent,
    LINK_MODE_COLOSSEUM, LinkSerialConnectionStatus, MagikarpRecordState, MysteryGiftState,
    Options, OverworldMemory, OverworldObjectMapMemory, PendingFieldTravel, PendingMoveLearn,
    RoamingPokemonState, SavedTrainerBattleFields, SceneMemory, ScriptAudioRuntimeEvent,
    ScriptAudioRuntimeKind, ScriptControlRuntimeEvent, ScriptControlRuntimeKind, ScriptEndState,
    ScriptGraphicsRuntimeEvent, ScriptLocation, ScriptMapLoadRequest, ScriptMapRefreshRequest,
    ScriptMapRuntimeEvent, ScriptMoneyRuntimeEvent, ScriptMusicFade, ScriptReturnFrame,
    ScriptRuntimeDelay, ScriptRuntimeEarthquake, ScriptRuntimeElevatorFloor, ScriptRuntimeEmote,
    ScriptRuntimeQueuedCommand, ScriptRuntimeStoneTableEntry, ScriptScreenFade,
    ScriptShopRuntimeEvent, ScriptTextRuntimeEvent, ScriptTextWait, ScriptWarpRequest,
    ScriptYesNoPrompt, SwarmMemory, saved_audio_runtime_event_command_args,
    saved_map_runtime_event_command_args, saved_pending_text_wait_command_args,
    saved_script_end_command, saved_text_runtime_event_command_args,
    validate_saved_audio_reference as core_validate_saved_audio_reference,
    validate_saved_block_overrides as core_validate_saved_block_overrides,
    validate_saved_bug_contest_references as core_validate_saved_bug_contest_references,
    validate_saved_catalog_reference, validate_saved_compiled_script_command_name_reference,
    validate_saved_compiled_script_command_payload_reference,
    validate_saved_compiled_script_command_reference,
    validate_saved_compiled_script_return_reference, validate_saved_control_runtime_event_shape,
    validate_saved_day_care_references as core_validate_saved_day_care_references,
    validate_saved_engine_flag_reference as core_validate_saved_engine_flag_reference,
    validate_saved_event_flag_reference as core_validate_saved_event_flag_reference,
    validate_saved_exact_catalog_reference,
    validate_saved_flag_references as core_validate_saved_flag_references,
    validate_saved_graphics_runtime_event_shape,
    validate_saved_last_talked_object_reference as core_validate_saved_last_talked_object_reference,
    validate_saved_map_object_reference as core_validate_saved_map_object_reference,
    validate_saved_map_reference as core_validate_saved_map_reference,
    validate_saved_money_runtime_event_shape,
    validate_saved_mystery_gift_references as core_validate_saved_mystery_gift_references,
    validate_saved_object_overrides_with_source_positions as core_validate_saved_object_overrides,
    validate_saved_optional_catalog_reference,
    validate_saved_overworld_references as core_validate_saved_overworld_references,
    validate_saved_pending_screen_fade_shape, validate_saved_player_gender,
    validate_saved_pokemon_party_references as core_validate_saved_pokemon_party_references,
    validate_saved_pokemon_reference as core_validate_saved_pokemon_reference,
    validate_saved_roaming_references as core_validate_saved_roaming_references,
    validate_saved_scene_references as core_validate_saved_scene_references,
    validate_saved_static_wild_battle_origin_reference as core_validate_saved_static_wild_battle_origin_reference,
    validate_saved_storage_references as core_validate_saved_storage_references,
    validate_saved_trainer_battle_request_field as core_validate_saved_trainer_battle_request_field,
    validate_saved_trainer_battle_request_fields, validate_saved_trainer_battle_source_reference,
    validate_saved_trainer_enemy_party_identity,
    validate_saved_warp_reference as core_validate_saved_warp_reference,
    validate_saved_wild_battle_origin_reference as core_validate_saved_wild_battle_origin_reference,
};
use crystal_core::systems::battle_escape::{
    BattleEscapeAttempt, BattleEscapeRules, BattleEscapeRulesIssue,
    attempt_wild_battle_escape_exact_with_loaded_speeds as core_attempt_wild_battle_escape_exact_with_loaded_speeds,
    battle_escape_rules_issues,
};
use crystal_core::systems::battle_items::{
    BattleItemOutcome, ItemPayloadIssue, ItemReferenceIssue, PartyItemOutcome,
    apply_active_battle_item_effect as core_apply_active_battle_item_effect,
    apply_battle_escape_item_use, apply_battle_pp_item_effect as core_apply_battle_pp_item_effect,
    apply_party_special_item_effect as core_apply_party_special_item_effect,
    apply_party_wide_item_effect as core_apply_party_wide_item_effect,
    clone_active_battle_party_pokemon, clone_field_party, clone_field_party_pokemon,
    item_payload_issues_with_known_field_rules, item_reference_issues,
    require_active_battle_party_pokemon_mut, require_field_party_mut,
    require_field_party_pokemon_mut, require_wild_battle_for_escape_item,
    validate_battle_escape_item, validate_battle_stat_drop_guard_item,
};
use crystal_core::systems::battle_rewards::{
    BattleLevelUpHappinessContext, BattleRewardError, BattleRewardOutcome, BattleRewardRules,
    BattleRewardRulesIssue, MomPurchaseKind, PendingMoveLearnResolution,
    apply_happiness_change as core_apply_happiness_change, battle_reward_rules_issues,
    claim_active_trainer_battle_rewards as core_claim_active_trainer_battle_rewards,
    claim_active_wild_battle_rewards as core_claim_active_wild_battle_rewards,
    decline_pending_move_learn as core_decline_pending_move_learn, promote_next_pending_move_learn,
    rebase_pending_move_learns_for_party,
    replace_pending_move_learn as core_replace_pending_move_learn,
    select_mom_purchase as core_select_mom_purchase,
    settle_pending_mom_purchase as core_settle_pending_mom_purchase,
    sync_active_combat_player_party_from_storage,
};
use crystal_core::systems::economy::{
    CurrencyCatalog, SCRIPT_COIN_CHECK_COMMANDS, SCRIPT_COIN_MUTATION_COMMANDS,
    SCRIPT_MONEY_CHECK_COMMANDS, SCRIPT_MONEY_MUTATION_COMMANDS, ScriptEconomyCommand,
    ScriptEconomyCommandIssue, ScriptEconomyOutcome,
    apply_script_economy_command as core_apply_script_economy_command,
    script_economy_command_issues, validate_save_currency_for_runtime_pack,
};
use crystal_core::systems::evolution::{
    EvolutionContext, EvolutionEntry, EvolutionReport, EvolutionTable, EvolutionTableIssue,
    LinkMode, TRADE_ANY_ITEM, check_and_evolve, evolution_table_issues, is_known_evolution_method,
    is_known_happiness_window, is_known_stat_evolution_ratio,
};
use crystal_core::systems::experience::{GrowthRateCatalogIssue, growth_rate_catalog_issues};
use crystal_core::systems::field_items::{
    FieldItemPickupOutcome, FruitTreeCatalog, FruitTreeCatalogIssue,
    ItemfinderHiddenItem as CoreItemfinderHiddenItem, SCRIPT_FIELD_FRUIT_TREE_PICKUP_COMMANDS,
    SCRIPT_FIELD_HIDDEN_ITEM_PICKUP_COMMANDS, SCRIPT_FIELD_ITEM_PICKUP_COMMANDS,
    SCRIPT_FIELD_ITEMBALL_PICKUP_COMMANDS, ScriptFieldPickup, ScriptFieldPickupIssue,
    find_itemfinder_hidden_item as core_find_itemfinder_hidden_item, fruit_tree_catalog_issues,
    fruit_tree_collected_flag, pickup_script_field_item as core_pickup_script_field_item,
    script_field_pickup_issues,
};
use crystal_core::systems::field_moves::{
    FieldEscapeItemRule, FieldItemRule, FieldMoveBlockOutcome, FieldMoveBlockRule,
    FieldMoveCatalog, FieldMoveCatalogIssue, FieldMoveError, FieldMoveFlagOutcome,
    FieldMoveFlagRule, FieldMoveMoveRule, FieldMoveRule, FieldMoveTravelOutcome,
    FieldMoveTravelRule, FieldMoveUseOutcome, FieldStoryKeyRule, SavedDigWarpDestination,
    apply_cut_field_move as core_apply_cut_field_move, apply_dig_warp_memory_for_transition,
    apply_escape_rope_chamber_effect, apply_flash_field_move as core_apply_flash_field_move,
    apply_flash_map_effect, apply_repel_item_use,
    apply_strength_field_move as core_apply_strength_field_move,
    apply_surf_field_move as core_apply_surf_field_move,
    apply_waterfall_field_move as core_apply_waterfall_field_move,
    apply_whirlpool_field_move as core_apply_whirlpool_field_move, blue_card_balance,
    field_move_catalog_issues, is_bicycle_environment, is_dig_field_move_environment,
    is_escape_rope_environment, is_fly_source_environment, is_teleport_source_environment,
    resolve_squirtbottle_target, saved_dig_warp_destination as core_saved_dig_warp_destination,
    validate_bicycle_item, validate_blue_card_item, validate_coin_case_item,
    validate_dig_field_move as core_validate_dig_field_move,
    validate_direct_field_move_actor as core_validate_direct_field_move_actor,
    validate_field_escape_item, validate_fly_field_move as core_validate_fly_field_move,
    validate_itemfinder_item, validate_pokegear_item, validate_repel_item,
    validate_saved_active_repel_item as core_validate_saved_active_repel_item,
    validate_saved_blue_card_balance, validate_squirtbottle_item,
    validate_teleport_field_move as core_validate_teleport_field_move, validate_town_map_item,
};
pub use crystal_core::systems::flee_mons::FleeMonTables;
use crystal_core::systems::flee_mons::{FleeMonCatalogIssue, flee_mon_catalog_issues};
use crystal_core::systems::gift_pokemon::{
    GiftPokemonOutcome, GiftPokemonRequest, GiftPokemonScript, GiftPokemonScriptIssue, NO_ITEM,
    gift_pokemon_script_issues, grant_gift_pokemon_to_state as core_grant_gift_pokemon_to_state,
};
use crystal_core::systems::item_use::{
    ItemUseContext, ItemUseOutcome, ItemUseRequest, use_bag_item as core_use_bag_item,
};
use crystal_core::systems::learnsets::{
    LearnsetCatalogIssue, LearnsetEntry, SpeciesLearnsets, learnset_catalog_issues,
    level_up_moves_for_species,
};
use crystal_core::systems::map_context::{
    SpawnMemoryUpdate, apply_map_music_context, apply_map_scene_context,
    apply_state_block_overrides, apply_state_object_overrides, commit_overworld_snapshot,
    sync_state_object_overrides,
};
use crystal_core::systems::phone::{
    PermanentPhoneNumberRule, PhoneContactCatalog, PhoneContactCatalogIssue, PhoneContactRecord,
    SCRIPT_PHONE_CHECK_COMMANDS, SCRIPT_PHONE_MUTATION_COMMANDS,
    SCRIPT_PHONE_REGISTRATION_COMMANDS, ScriptPhoneCommand, ScriptPhoneCommandIssue,
    ScriptPhoneError, ScriptPhoneInputs, ScriptPhoneOutcome,
    apply_script_phone_command as core_apply_script_phone_command,
    initialize_permanent_phone_numbers as core_initialize_permanent_phone_numbers,
    phone_contact_catalog_issues, script_phone_command_issues,
};
use crystal_core::systems::roaming::{
    RoamingBattleEndInput, RoamingEngineState, battle_end_handle_roam_mons,
};
use crystal_core::systems::runtime_pack::{
    RuntimePackPresenceIssue, RuntimePackSections, runtime_pack_presence_issues,
};
use crystal_core::systems::script_audio::{
    SCRIPT_AUDIO_CRY_COMMANDS, SCRIPT_AUDIO_MUSIC_COMMANDS, SCRIPT_AUDIO_MUSIC_FADE_COMMANDS,
    SCRIPT_AUDIO_NO_PAYLOAD_COMMANDS, SCRIPT_AUDIO_SOUND_EFFECT_COMMANDS, ScriptAudioCommand,
    ScriptAudioCommandIssue, ScriptAudioCue,
    apply_script_audio_command as core_apply_script_audio_command, script_audio_command_issues,
};
use crystal_core::systems::script_blocks::{
    ScriptBlockChange, ScriptBlockChangeIssue, ScriptBlockChangeOutcome,
    apply_script_block_change as core_apply_script_block_change, script_block_change_issues,
};
use crystal_core::systems::script_control::{
    ScriptControlAction, ScriptControlCommand, ScriptControlCommandIssue,
    apply_script_control_command as core_apply_script_control_command,
    script_control_command_issues,
};
use crystal_core::systems::script_flags::{
    ScriptFlagCheckOutcome, ScriptFlagCommand, ScriptFlagCommandIssue, ScriptFlagMutationOutcome,
    apply_script_flag_mutation as core_apply_script_flag_mutation,
    check_script_flag as core_check_script_flag, is_known_script_flag_command,
    script_flag_command_issues,
};
use crystal_core::systems::script_items::{
    ScriptItemAccess, ScriptItemAccessIssue, ScriptItemCheckOutcome, ScriptItemGrant,
    ScriptItemGrantIssue, ScriptItemGrantOutcome, ScriptItemTakeOutcome,
    check_script_item as core_check_script_item, grant_script_item as core_grant_script_item,
    script_item_access_issues, script_item_grant_issues, take_script_item as core_take_script_item,
};
use crystal_core::systems::script_objects::{
    SCRIPT_MOVEMENT_DIRECTION_COMMANDS, SCRIPT_MOVEMENT_NO_ARG_COMMANDS,
    SCRIPT_MOVEMENT_PLAYER_FACING_DIRECTION, SCRIPT_MOVEMENT_REQUIRED_DURATION_COMMANDS,
    SCRIPT_OBJECT_COORDINATE_COMMANDS, SCRIPT_OBJECT_DIRECT_MOVEMENT_COMMANDS,
    SCRIPT_OBJECT_DIRECTION_COMMANDS, SCRIPT_OBJECT_EMOTE_COMMANDS,
    SCRIPT_OBJECT_LAST_TALKED_MOVEMENT_COMMANDS, SCRIPT_OBJECT_MOVEMENT_COMMANDS,
    SCRIPT_OBJECT_NO_PAYLOAD_COMMANDS, SCRIPT_OBJECT_TARGET_COMMANDS,
    SCRIPT_OBJECT_VISIBILITY_COMMANDS, SCRIPT_OBJECT_WRITE_COORDINATE_COMMANDS, ScriptMovement,
    ScriptMovementOutcome, ScriptMovementStep, ScriptMovementStepIssue, ScriptObjectCommand,
    ScriptObjectCommandIssue, ScriptObjectMutationOutcome,
    apply_script_movement as core_apply_script_movement,
    apply_script_object_mutation as core_apply_script_object_mutation,
    is_hideable_object_event_flag, is_known_script_movement_command, is_script_movement_terminator,
    script_movement_step_issues, script_movement_step_runtime_stride, script_object_command_issues,
};
pub use crystal_core::systems::script_runtime::{
    InitializeEventsConfig, StoryEventScriptConstants,
};
use crystal_core::systems::script_runtime::{
    InitializeEventsIssue, SCRIPT_RUNTIME_USE_SCRIPT_VAR_ID, ScriptRuntimeCommand,
    ScriptRuntimeCommandError, ScriptRuntimeCommandIssue, ScriptRuntimeCpuCondition,
    ScriptRuntimeDecorationResolution, ScriptRuntimeInputs, ScriptRuntimeOutcome,
    ScriptRuntimeReferenceCatalog, StoryEventScriptConstantIssue, apply_initialize_events,
    apply_script_random_command_in_map as core_apply_script_random_command,
    apply_script_runtime_command_in_map as core_apply_script_runtime_command,
    commit_interaction_script_dispatch, initialize_events_issues, parse_menu_coord_token,
    parse_script_i32_token, script_runtime_command_arg_counts, script_runtime_command_issues,
    story_event_script_constant_issues, validate_script_runtime_command,
};
use crystal_core::systems::script_scenes::{
    SCRIPT_SCENE_CHECK_COMMANDS, SCRIPT_SCENE_CURRENT_MAP_MUTATION_COMMANDS,
    SCRIPT_SCENE_TARGET_MAP_CHECK_COMMANDS, SCRIPT_SCENE_TARGET_MAP_MUTATION_COMMANDS,
    ScriptSceneCommand, ScriptSceneCommandIssue, ScriptSceneOutcome,
    apply_script_scene_command as core_apply_script_scene_command, script_scene_command_issues,
};
use crystal_core::systems::script_swarms::{
    ScriptSwarmCommand, ScriptSwarmCommandIssue, ScriptSwarmOutcome,
    apply_script_swarm_command as core_apply_script_swarm_command, script_swarm_command_issues,
};
use crystal_core::systems::script_text::{
    AsmTextCatalogIssue, SCRIPT_TEXT_LABEL_COMMANDS, SCRIPT_TEXT_NO_LABEL_COMMANDS,
    ScriptMenuCommand, ScriptMenuDefinition, ScriptMenuDefinitionIssue, ScriptTextAction,
    ScriptTextBody, ScriptTextBodyCommand, ScriptTextBodyIssue, ScriptTextCommand,
    ScriptTextCommandError, apply_script_text_command as core_apply_script_text_command,
    asm_text_catalog_issues, menu_definition_command_arg_counts,
    resolve_script_text_command as core_resolve_script_text_command, script_menu_definition_issues,
    script_text_body_issues, script_text_command_issues, text_body_command_arg_counts,
};
use crystal_core::systems::script_variables::{
    ScriptVariableCommand, ScriptVariableCommandIssue, ScriptVariableOutcome,
    apply_script_variable_command as core_apply_script_variable_command,
    script_variable_command_issues,
};
use crystal_core::systems::script_warps::{
    MAP_CALLBACK_NEWMAP, MAP_CALLBACK_TILES, SCRIPT_MAP_FACING_WARP_COMMANDS,
    SCRIPT_MAP_NEW_LOAD_COMMANDS, SCRIPT_MAP_NO_PAYLOAD_COMMANDS, SCRIPT_MAP_REANCHOR_COMMANDS,
    SCRIPT_MAP_WARP_COMMANDS, ScriptMapAction, ScriptMapCommand, ScriptMapCommandError,
    apply_script_map_command as core_apply_script_map_command, apply_script_warp_arrival_facing,
    complete_pending_script_warp, map_setup_callback_kinds, parse_script_warp_facing,
    script_map_command_issues,
};
use crystal_core::systems::shop::{
    MartCatalog, MartCatalogIssue, SCRIPT_SHOP_COMMANDS, ScriptShopCommand, ScriptShopCommandIssue,
    ScriptShopOutcome, ShopError, ShopResult,
    apply_script_shop_command as core_apply_script_shop_command,
    buy_active_shop_item as core_buy_active_shop_item, close_active_shop as core_close_active_shop,
    mart_catalog_issues, script_shop_command_issues,
    sell_active_shop_item as core_sell_active_shop_item,
};
#[cfg(test)]
use crystal_core::systems::special_routines::BattleTowerBannedSpeciesRule;
pub use crystal_core::systems::special_routines::RuntimeSpawnPointRef as RuntimeSpawnPoint;
#[cfg(test)]
use crystal_core::systems::special_routines::{
    BattleTowerMonDefinition, BattleTowerTrainerDefinition, RoamingPokemonInitWrite,
    RoamingPokemonRoute,
};
use crystal_core::systems::special_routines::{
    BattleTowerRules, BattleTowerRulesIssue, BuenaPasswordCategories,
    BuenaPasswordCategoryDefinition, BuenaPasswordCategoryIssue, BuenaPrizeDefinitionIssue,
    BuenaPrizeDefinitions, BugContestConfig, BugContestConfigIssue, DratiniMoveSetIssue,
    DratiniMoveSets, EXECUTABLE_SPECIAL_ROUTINES, HappinessData, HappinessDataIssue,
    KurtApricornRecipeIssue, KurtApricornRecipes, MagikarpLengthEntry, MagikarpLengthTableIssue,
    OakRatingEntry, OakRatingTableIssue, OddEggDefinition, OddEggDefinitionIssue,
    RoamingMapLocation, RoamingPokemonCatalog, RoamingPokemonCatalogIssue,
    RuntimeSpawnPointCatalogIssue, ShuckieGiftDefinition, ShuckieGiftIssue,
    SpecialRoutineCatalogIssue, SpecialRoutineContext, SpecialRoutineEffect, SpecialRoutineOutcome,
    apply_random_special_routine_with_context, apply_special_routine_with_context,
    battle_tower_rules_issues, buena_password_category_issues, buena_prize_definition_issues,
    bug_contest_config_issues, checked_runtime_spawn_expected_tile, dratini_move_set_issues,
    happiness_data_issues, is_known_special_routine, kurt_apricorn_recipe_issues,
    magikarp_length_table_issues, oak_rating_table_issues, odd_egg_definition_issues,
    resolve_bug_contest_caught_mon, roaming_pokemon_catalog_issues, runtime_spawn_expected_tile,
    runtime_spawn_point_catalog_issues, runtime_spawn_subtiles_are_valid,
    saved_battle_tower_state_is_active, saved_special_battle_type_builtin_routine,
    shuckie_gift_issues, special_routine_catalog_issues,
    validate_saved_battle_tower_state as core_validate_saved_battle_tower_state,
    validate_saved_buena_password_references, validate_saved_magikarp_record_references,
    validate_saved_pending_special_battle_type,
};
use crystal_core::systems::step_events::{
    StepEventResult, StepEventRules, StepEventRulesIssue,
    process_overworld_step as core_process_overworld_step, step_event_rules_issues,
};
use crystal_core::systems::time::{ClockTime, GameDate};
use crystal_core::systems::tmhm::{
    TmHmLearnOutcome, teach_tmhm_move as core_teach_tmhm_move, validate_saved_tmhm_references,
};
use crystal_core::world::collision::{
    MetatileCollision, PlayerTraversalState, Terrain, TilesetCollision, can_enter_tile,
    describe_collision, is_permission_passable, permissions, sample_collision,
    spawns_shaking_grass_object, standard_interaction_script,
};
#[cfg(any(test, feature = "test-fixtures"))]
use crystal_core::world::encounters::roll_headbutt_encounter as core_roll_headbutt_encounter;
use crystal_core::world::encounters::{
    ENCOUNTER_TIME_KEYS, EncounterMusicModifierIssue, EncounterMusicModifiers, EncounterSlotChance,
    EncounterSlotTableIssue, EncounterSlotTables, EncounterSurface, FieldEncounterCatalogIssue,
    FieldEncounterData, FieldEncounterKind, HeadbuttEncounterOutcome, ResolvedWildEncounter,
    RockMonEncounterOutcome, TimeOfDay, WildEncounter, WildEncounterCatalogIssue,
    WildEncounterData, WildEncounterTable, apply_surf_level_variance,
    encounter_music_modifier_issues, encounter_slot_table_issues, field_encounter_catalog_issues,
    resolve_headbutt_encounter as core_resolve_headbutt_encounter,
    resolve_rock_mon_encounter as core_resolve_rock_mon_encounter, select_wild_encounter,
    table_for_surface, wild_encounter_catalog_issues,
};
use crystal_core::world::fishing::{
    FISHING_RODS, FishingCatalog, FishingCatalogIssue, FishingGroup, FishingSession, RodTable,
    do_fishing_exact as core_do_fishing_exact, fishing_battle_trigger, fishing_bite,
    fishing_catalog_issues, fishing_rod_for_item_id,
    validate_saved_fishing_references as core_validate_saved_fishing_references,
};
pub use crystal_core::world::map::RuntimeMapMetadata;
use crystal_core::world::map::{
    Direction, METATILE_WIDTH, OverworldMapData, RuntimeMapMetadataIssue, TilePosition,
    runtime_map_metadata_issues,
};
use crystal_core::world::movement::{
    DEFAULT_RUNTIME_TILE_STRIDE, LedgeJumpOutcome, MovementMode, PlayerMovementState, StepOptions,
    StepOutcome, checked_move_by_stride,
};
use crystal_core::world::session::{
    ConnectionDestination, ConnectionTransition, ConnectionTrigger, CoordEventTrigger,
    EncounterCheckOptions, ExactEncounterContext, OverworldInteraction, OverworldInteractionTarget,
    OverworldSession, OverworldSnapshot, WarpDestination, WarpTransition, WarpTrigger,
    WildEncounterRoll, leading_usable_party_level, object_event_initial_facing,
    raw_event_tile_to_runtime_tile_checked, runtime_tile_to_raw_event_tile,
};
use crystal_core::world::session::{
    background_event_tile_position_checked, warp_tile_position_checked,
};
use flate2::{Compression, write::GzEncoder};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crystal_core::battle::capture::CaptureRules;

pub mod radio_broadcast;
pub mod radio_catalog;
pub mod radio_host;
pub mod radio_text_memory;

mod gen3_modpack;
pub use gen3_modpack::{GEN3_MANIFEST_ID, GEN3_SPECIES_COUNT, build_gen3_modpack};
mod fairy_modpack;
mod minnesota_cries;
mod minnesota_pokemon_curation;
mod minnesota_pokemon_graphics;
mod minnesota_pokemon_modpack;
#[allow(dead_code)]
mod minnesota_pokemon_pixel_art;
mod minnesota_pokemon_profiles;
#[allow(dead_code)]
pub mod minnesota_signature_animations;
pub mod starter_signature_animations;
pub use fairy_modpack::{
    FAIRY_MOVES_MANIFEST_ID, FAIRY_TYPES_MANIFEST_ID, build_fairy_moves_modpack,
    build_fairy_moves_modpack_from_json, build_fairy_type_modpack,
};
pub mod northwoods_move_animations;
pub use northwoods_move_animations::{
    NORTHWOODS_ANIMATIONS_MANIFEST_ID, build_northwoods_animations_modpack,
};
mod northwoods_moves_modpack;
pub use northwoods_moves_modpack::{NORTHWOODS_MOVES_MANIFEST_ID, build_northwoods_moves_modpack};
mod northwoods_starter_signatures;
pub use northwoods_starter_signatures::{
    NORTHWOODS_STARTER_SIGNATURES_MANIFEST_ID, build_northwoods_starter_signatures_modpack,
    build_northwoods_starter_signatures_modpack_from_json,
    verify_northwoods_starter_signatures_preservation,
};
pub mod minnesota_overworld_sprites;
mod minnesota_sprite_refinement;
mod minnesota_sprite_sheet;
mod minnesota_tile_animation;
mod minnesota_tile_art;
pub use minnesota_cries::build_minnesota_cries;
pub use minnesota_pokemon_modpack::MinnesotaPokemonCatalog;
pub use minnesota_tile_animation::{
    AuthoredTileAnimation, MinnesotaTileAnimationRecipe, TileAnimationSource,
    with_minnesota_tile_animation,
};
pub use minnesota_tile_art::{
    MinnesotaTileCatalog, MinnesotaTileItem, MinnesotaTilePixels, build_minnesota_tile_art_pack,
    minnesota_tile_item_preview, minnesota_tile_preview,
};
mod state_fair_buildings;
mod state_fair_encounters;
mod state_fair_modpack;
mod state_fair_patrols;
mod state_fair_scripts;
pub use state_fair_buildings::revise_state_fair_buildings;
pub use state_fair_modpack::build_state_fair_modpack;
mod nuzlocke;
pub use nuzlocke::{NUZLOCKE_MANIFEST_ID, NuzlockeRules};
mod realtime_clock_modpack;
pub use realtime_clock_modpack::{REALTIME_CLOCK_MANIFEST_ID, build_realtime_clock_modpack};
mod outdoor_terrain_modpack;
pub use outdoor_terrain_modpack::build_outdoor_terrain_modpack;
mod nuzlocke_modpack;
pub use nuzlocke_modpack::build_nuzlocke_modpack;

pub mod modpack {
    pub use super::ScriptElevatorDefinition;
    pub use super::ScriptVerticalMenuDefinition;
    pub use crystal_core::battle::capture::{CaptureRules, CaptureWobbleProbability};
    pub use crystal_core::battle::damage::{
        TypeCategories, TypeEffectivenessTable, WeatherModifiers,
    };
    pub use crystal_core::battle::stats::{BattleStatMultiplier, BattleStatMultiplierTables};
    pub use crystal_core::battle::turn::{MovePriorityOverride, MovePriorityTable};
    pub use crystal_core::map::{MapEventSectionCommand, MapScriptSectionCommand};
    pub use crystal_core::systems::battle_rewards::BattleRewardRules;
    pub use crystal_core::systems::economy::{CurrencyCatalog, ScriptEconomyCommand};
    pub use crystal_core::systems::evolution::{EvolutionEntry, EvolutionTable};
    pub use crystal_core::systems::experience::{GrowthRateCatalog, GrowthRateCurve};
    pub use crystal_core::systems::field_items::{FruitTreeCatalog, ScriptFieldPickup};
    pub use crystal_core::systems::gift_pokemon::GiftPokemonScript;
    pub use crystal_core::systems::phone::{
        PermanentPhoneNumberRule, PhoneContactCatalog, PhoneContactRecord, ScriptPhoneCommand,
    };
    pub use crystal_core::systems::script_audio::ScriptAudioCommand;
    pub use crystal_core::systems::script_blocks::ScriptBlockChange;
    pub use crystal_core::systems::script_control::ScriptControlCommand;
    pub use crystal_core::systems::script_flags::ScriptFlagCommand;
    pub use crystal_core::systems::script_items::{ScriptItemAccess, ScriptItemGrant};
    pub use crystal_core::systems::script_objects::{
        ScriptMovement, ScriptMovementStep, ScriptObjectCommand,
    };
    pub use crystal_core::systems::script_runtime::ScriptRuntimeCommand;
    pub use crystal_core::systems::script_scenes::ScriptSceneCommand;
    pub use crystal_core::systems::script_text::{
        ScriptMenuCommand, ScriptMenuDefinition, ScriptTextBody, ScriptTextBodyCommand,
        ScriptTextCommand,
    };
    pub use crystal_core::systems::script_variables::ScriptVariableCommand;
    pub use crystal_core::systems::script_warps::ScriptMapCommand;
    pub use crystal_core::systems::shop::{MartCatalog, ScriptShopCommand};
    pub use crystal_core::world::encounters::{
        EncounterMusicModifier, EncounterMusicModifiers, EncounterSlotChance, EncounterSlotTables,
    };
    pub use crystal_core::world::fishing::FishingCatalog;

    pub use super::{
        COMPILED_GAME_PACK_EXTENSION, COMPILED_GAME_PACK_FORMAT_VERSION, CompiledGamePack,
        CompiledGamePackIdentity, CompiledMapExtension, CompiledModpack, CompiledTilesetExtension,
        ContentPack, ContentPackCategory, ContentPackFiles, ContentPackIndex, GameDataSet,
        LoadedCompiledGamePack, MapAccessRule, MapModule, ModpackAudioAsset, ModpackAudioKind,
        ModpackAudioLoopPolicy, ModpackAudioManifest, ModpackAudioManifestEntry,
        ModpackAudioPlaybackEntry, ModpackAudioPlaybackMode, ModpackAudioPlaybackPlan,
        ModpackAudioSource, ModpackCompileOptions, ModpackCompileReport, ModpackCompiler,
        ModpackManifest, ModpackMetadata, ModpackPayload, PACK_AUDIO_COMPRESSION_GZIP,
        PACK_AUDIO_COMPRESSION_MIDI, PlayabilityGraphEdge, PlayabilityRules, PlayabilityStart,
        ProgressionGrants, ProgressionRequirements, ProgressionRule,
        REQUIRED_VENDOR_RUNTIME_FILE_KEYS, VerificationError, VerificationSeverity,
        load_verified_compiled_game_pack_bytes, read_loaded_verified_compiled_game_pack,
        read_verified_compiled_game_pack, validate_compiled_runtime_files,
    };
    pub use super::{
        GEN3_MANIFEST_ID, GEN3_SPECIES_COUNT, NUZLOCKE_MANIFEST_ID, NuzlockeRules,
        build_gen3_modpack, build_nuzlocke_modpack,
    };
    #[cfg(any(test, feature = "test-fixtures"))]
    pub use super::{
        write_compiled_game_pack_for_tests, write_compiled_game_pack_with_midi_audio_for_tests,
    };
    pub use crystal_core::models::{Trainer, TrainerCatalog};
    pub use crystal_core::systems::special_routines::{
        BattleTowerBannedSpeciesRule, BattleTowerRules, BuenaPasswordCategories,
        BuenaPasswordCategoryDefinition, BuenaPrizeDefinitions, BugContestConfig, DratiniMoveSets,
        HappinessChangeEntry, HappinessData, HappinessServiceOutcome, KurtApricornRecipes,
        MagikarpLengthEntry, OakRatingEntry, OddEggDefinition, RoamingMapLocation,
        RoamingPokemonCatalog, RoamingPokemonInitWrite, RoamingPokemonRoute, ShuckieGiftDefinition,
    };
    pub use crystal_core::systems::step_events::StepEventRules;
}

include!("content_pack.rs");
mod cbor_identity_hash;
include!("map_modules.rs");
include!("runtime_pack.rs");
include!("verification.rs");
include!("runtime_commands.rs");
mod game_data_encounter_surfaces;
include!("game_data.rs");
mod pack_load_trace;
pub use pack_load_trace::with_compiled_pack_load_observer;
include!("mutation_protocol.rs");
include!("merge.rs");
include!("script_parsing.rs");
mod scoped_text_save;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

mod competitive_battle_modpack;
mod competitive_trainer_profiles;
mod default_battle_modpack;
mod deferred_evolution;
mod modern_move_split_modpack;
pub use competitive_battle_modpack::{
    COMPETITIVE_BATTLES_MANIFEST_ID, build_competitive_battle_modpack,
};
pub use competitive_trainer_profiles::TrainerBattleProfile;
pub use default_battle_modpack::build_default_battle_modpack;
mod wild_challenge;
pub use wild_challenge::{
    WILD_CHALLENGE_MANIFEST_ID, WildChallengeRules, build_wild_challenge_modpack,
};
mod trainer_rematches;
pub use modern_move_split_modpack::{
    MODERN_MOVE_SPLIT_MANIFEST_ID, build_modern_move_split_modpack,
};
pub use trainer_rematches::{TRAINER_REMATCHES_MANIFEST_ID, build_trainer_rematches_modpack};

mod player_customization_modpack;
pub use player_customization_modpack::{
    PLAYER_CUSTOMIZATION_MANIFEST_ID, build_player_customization_modpack,
};

mod text_tui_modpack;
pub use text_tui_modpack::{TEXT_TUI_MANIFEST_ID, build_text_tui_modpack};

mod all_pokemon_modpack;
pub use all_pokemon_modpack::{
    ALL_251_MANIFEST_ID, EVOLUTION_STONES, LINK_CABLE_ITEM_ID, SOLO_EVOLUTION_MANIFEST_ID,
    STONE_MART_ID, build_all_251_catchable_modpack, build_all_pokemon_modpack,
    build_solo_evolution_modpack, verify_all_251_coverage,
};

mod natural_movement_modpack;
pub use natural_movement_modpack::{NATURAL_MOVEMENT_MANIFEST_ID, build_natural_movement_modpack};

mod grand_marais_harbor_revision;
pub use grand_marais_harbor_revision::revise_grand_marais_harbor_boundary;
mod grand_marais_water_repair;
pub use grand_marais_water_repair::repair_grand_marais_water_encounters;
mod grand_marais_modpack;
pub use grand_marais_modpack::{
    build_grand_marais_modpack, build_grand_marais_region_extension,
    refresh_grand_marais_story_dialogue,
};
mod gym_art;
mod gym_cut;
mod gym_design;
mod gym_ledges;
mod gym_lifts;
mod gym_publication;
mod gym_routing;
mod gym_scripts;
mod gym_switches;
mod gym_wind;
mod minnesota_gyms;
pub use minnesota_gyms::{build_minnesota_gym_review_pack, build_minnesota_gym_review_room};
mod boundary_basecamp_story;
pub use boundary_basecamp_story::refresh_boundary_basecamp_story_dialogue;
mod northstar_relay;
mod northstar_relay_art;
pub use northstar_relay::build_northstar_relay_scene;
mod city_note_mimics;
mod wild_level_refuges;
pub use city_note_mimics::build_city_note_mimics;

mod northstar_supplies;
pub use northstar_supplies::build_northstar_supplies;
mod authored_dialogue;
mod northstar_clinic_revision;
mod northstar_courier_revision;
mod northstar_current_story;
mod northstar_dispatch_revision;
mod northstar_holding_fence;
mod northstar_opening_revision;
mod northstar_report_revision;
mod northstar_story_cast;
mod northstar_story_connections;
mod northstar_story_pacing;
mod northstar_story_revision;
mod northstar_story_staff;
pub use northstar_story_connections::align_northstar_league_entrance;
pub use northstar_story_revision::revise_northstar_story;
mod northstar_dialogue;
pub use northstar_current_story::{
    build_northstar_current_story, build_northstar_current_story_from_composition,
};
mod northstar_league_dialogue;
pub use northstar_dialogue::refresh_northstar_dialogue;
pub use northstar_league_dialogue::refresh_northstar_league_dialogue;
mod northstar_returns;
pub use northstar_returns::build_northstar_returns;
mod northstar_story_events;
mod northstar_story_opening;
pub use northstar_story_events::build_northstar_story_events;
mod northstar_faction_teams;
mod northstar_regional_cast;
#[cfg(test)]
mod northstar_regional_cast_tests;
mod northstar_regional_roster;
pub use northstar_regional_cast::build_northstar_regional_cast;
mod northstar_ice_lair;
#[cfg(test)]
mod northstar_ice_lair_tests;
mod northstar_lair_authoring;
pub use northstar_ice_lair::build_northstar_ice_lair;
mod northstar_ice_headquarters;
mod northstar_ice_headquarters_events;
#[cfg(test)]
mod northstar_ice_headquarters_tests;
pub use northstar_ice_headquarters::build_northstar_ice_headquarters;
mod northstar_iron_workings;
#[cfg(test)]
mod northstar_iron_workings_tests;
pub use northstar_iron_workings::build_northstar_iron_workings;
mod northstar_progression_rewards;
pub use northstar_progression_rewards::build_northstar_progression_rewards;
mod northstar_legendary_completion;
pub use northstar_legendary_completion::build_northstar_legendary_completion;
mod minnesota_fossils;
pub use minnesota_fossils::build_minnesota_fossils;
mod minnesota_fossil_lab;
pub use minnesota_fossil_lab::build_minnesota_fossil_lab;
mod minnesota_castorock;
pub use minnesota_castorock::build_minnesota_castorock;
mod castorock_gallery_revision;
pub use castorock_gallery_revision::revise_castorock_gallery;
mod minnesota_dex_roster_sync;
pub use minnesota_dex_roster_sync::build_minnesota_dex_roster_sync;
mod grand_marais_partner_retry;
pub use grand_marais_partner_retry::build_grand_marais_partner_retry;
mod minnesota_starter_return;
pub use minnesota_starter_return::build_minnesota_starter_return;
mod minnesota_research_access;
pub use minnesota_research_access::build_minnesota_research_access;
mod northstar_publication_geometry;
mod northstar_rival_arc;
mod northstar_rival_art;
mod northstar_rival_early;
mod northstar_rival_finale;
mod northstar_rival_grunts;
mod northstar_rival_middle;
mod northstar_rival_returns;
pub use northstar_publication_geometry::verify_northstar_publication_geometry;
mod northstar_ancient_swamp;
pub use northstar_ancient_swamp::build_northstar_ancient_swamp;
#[cfg(test)]
mod northstar_rival_arc_tests;
pub use northstar_rival_arc::build_northstar_rival_arc;
mod northstar_drill_lair;
#[cfg(test)]
mod northstar_drill_lair_tests;
mod northstar_drill_scene;
mod northstar_story_bridges;
pub use northstar_drill_lair::build_northstar_drill_lair;
mod minnesota_fishing_references;
pub use minnesota_fishing_references::repair_minnesota_fishing_references;
mod minnesota_fishing_hints;
mod minnesota_fishing_houses;
mod minnesota_fishing_rods;
pub use minnesota_fishing_rods::build_minnesota_fishing_rods;

mod minnesota_fly_destinations;
pub use minnesota_fly_destinations::build_minnesota_fly_destinations;
mod minnesota_habitat_fishing;
mod minnesota_habitats;
pub use minnesota_habitats::build_minnesota_habitats;
mod minnesota_sweet_scent_source;
mod minnesota_water_cave_access;
mod minnesota_water_cave_habitats;
mod minnesota_biome_ecology;
mod minnesota_ecology_fishing;
mod minnesota_ecology_coverage;
mod minnesota_habitat_review;
mod minnesota_clinic_companion;
mod minnesota_encounter_progression;
pub use minnesota_encounter_progression::build_minnesota_encounter_progression;
pub use minnesota_water_cave_habitats::build_minnesota_water_cave_habitats;
pub use minnesota_biome_ecology::build_minnesota_biome_ecology;
mod minnesota_hm_progression;
pub use minnesota_hm_progression::build_minnesota_hm_progression;
mod minnesota_tmhm;
mod minnesota_tmhm_caches;
pub use minnesota_tmhm::build_minnesota_tmhm;
mod boundary_basecamp_connections;
mod boundary_basecamp_modpack;
pub use boundary_basecamp_connections::connect_boundary_basecamp;
pub use boundary_basecamp_modpack::{
    BOUNDARY_BASECAMP_CONSTANT, BOUNDARY_BASECAMP_GROUP, BOUNDARY_BASECAMP_MANIFEST,
    BOUNDARY_BASECAMP_MAP, build_boundary_basecamp_modpack,
};

mod north_shore_art;
pub use north_shore_art::build_north_shore_art_pack;

mod gunflint_art;
mod native_field_scripts_modpack;
mod native_tile_png;
pub use gunflint_art::build_gunflint_art_pack;
pub use native_field_scripts_modpack::build_native_field_scripts_modpack;

mod north_shore_modpack;
pub use north_shore_modpack::{build_north_shore_modpack, expand_north_shore_modpack};

mod authored_cave_revision;
mod authored_route_cut_gates;
mod authored_route_design;
mod authored_route_events;
mod authored_route_layout;
mod authored_route_modpack;
mod authored_route_revision;
mod bemidji_northwoods_grass;
mod coastal_cedar_warmth;
mod confluence_grass;
mod detroit_approach_grass;
mod duluth_cliff_stairs;
mod duluth_harbor;
mod duluth_terrace_stairs;
mod ely_shore_pass_grass;
mod iron_cliff_stairs;
mod itasca_grass;
mod lower_woods_grass;
mod mille_lacs_grass;
mod original_repel_revision;
mod outdoor_cliff_corners;
mod outdoor_cliff_joins;
mod outdoor_ramp_stairs;
mod outdoor_stair_landings;
mod outdoor_trainer_sightlines;
mod preserve_pine_grass;
mod r09_current_mine;
mod r09_mine_revision;
mod r10_cache_grass_art;
mod r10_carrier_corner_art;
mod r10_gallery_revision;
mod r10_stairs_revision;
mod rainy_icefields_grass;
mod upstream_woods_grass;
pub use authored_cave_revision::revise_authored_cave_terrain;
pub use authored_route_revision::revise_authored_route_terrain;
pub use bemidji_northwoods_grass::revise_bemidji_northwoods_grass;
pub use coastal_cedar_warmth::restore_coastal_cedar_warmth;
pub use confluence_grass::revise_confluence_grass;
pub use detroit_approach_grass::revise_detroit_approach_grass;
pub use duluth_cliff_stairs::revise_duluth_cliff_stairs;
pub use duluth_harbor::{clear_duluth_inner_buoys, revise_duluth_harbor, revise_duluth_shoreline};
pub use duluth_terrace_stairs::revise_duluth_terrace_stairs;
pub use ely_shore_pass_grass::revise_ely_shore_pass_grass;
pub use iron_cliff_stairs::revise_iron_cliff_stairs;
pub use itasca_grass::revise_itasca_grass;
pub use lower_woods_grass::revise_lower_woods_grass;
pub use mille_lacs_grass::revise_mille_lacs_grass;
pub use original_repel_revision::{parse_source_repel_duration, repair_original_repel_duration};
pub use outdoor_cliff_joins::revise_minnesota_outdoor_cliff_joins;
pub use outdoor_ramp_stairs::{
    revise_minnesota_outdoor_north_stairs, revise_minnesota_outdoor_ramp_stairs,
};
pub use outdoor_stair_landings::revise_minnesota_outdoor_stair_landings;
pub use preserve_pine_grass::revise_preserve_pine_grass;
pub use r09_current_mine::build_current_r09_mine_pack;
pub use r09_mine_revision::revise_r09_mine_terrain;
pub use r10_cache_grass_art::{add_r10_cache_art, add_r10_west_terrace_art};
pub use r10_carrier_corner_art::add_r10_carrier_corner_art;
pub use r10_gallery_revision::revise_r10_gallery_modpack;
pub use r10_stairs_revision::revise_r10_rock_stairs;
pub use rainy_icefields_grass::revise_rainy_icefields_grass;
pub use upstream_woods_grass::revise_upstream_woods_grass;
mod authored_route_reward_grass;
mod authored_route_surface;
mod authored_route_traversal;
mod gunflint_modpack;
pub use gunflint_modpack::build_gunflint_modpack;
mod league_approach_modpack;
pub use league_approach_modpack::build_league_approach_modpack;
mod boundary_safari_art;
mod safari_admission;
mod safari_departure;
mod safari_encounters;
mod safari_fishing;
mod safari_guidance;
mod safari_mutation;
mod safari_pack;
mod safari_prompts;
mod safari_public_trail;
mod safari_roster;
mod safari_zone;
pub use safari_admission::SafariAdmissionOutcome;
pub use safari_mutation::{RuntimeSafariCommand, RuntimeSafariOutcome};
pub use safari_zone::{
    SafariGateCourt, SafariHabitat, SafariMap, SafariSurface, SafariWildSlot, SafariZone,
};
mod boundary_safari_modpack;
pub use boundary_safari_modpack::{
    BOUNDARY_SAFARI_GROUP, BOUNDARY_SAFARI_MANIFEST, BOUNDARY_SAFARI_PENDING_GUARDS,
    build_boundary_safari_modpack, expand_boundary_safari_modpack,
};
mod headwaters_art;
pub use boundary_safari_art::build_boundary_safari_art_pack;
mod iron_range_modpack;
pub use iron_range_modpack::build_iron_range_modpack;
mod iron_range_story;
pub use iron_range_story::build_iron_range_story_modpack;
mod grand_rapids_modpack;
pub use grand_rapids_modpack::build_grand_rapids_modpack;
mod authored_checkpoint;
mod authored_city_expansion;
mod authored_city_expansion_anchors;
mod authored_city_expansion_art;
mod authored_city_expansion_design;
mod authored_city_expansion_events;
mod authored_city_expansion_geometry;
mod authored_town_design;
mod authored_town_geometry;
mod authored_town_publication;
mod authored_town_rooms;
mod authored_town_scripts;
mod minnesota_geometry_publication;
pub use authored_city_expansion::expand_authored_city;
mod authored_city_recomposition;
mod authored_city_recomposition_art;
mod authored_city_recomposition_design;
mod authored_city_recomposition_gate;
mod authored_city_recomposition_water;
mod authored_city_retained_programs;
pub use authored_city_recomposition::recompose_authored_city;
mod authored_river_connections;
mod authored_river_connections_art;
mod authored_river_connections_design;
mod authored_river_connections_events;
mod authored_river_connections_validation;
pub use authored_river_connections::build_authored_river_connections;
mod state_cohesion_design;
mod state_cohesion_events;
mod state_cohesion_modpack;
mod state_cohesion_validation;
pub use state_cohesion_modpack::build_state_cohesion;
mod authored_region_geometry;
mod itasca_connections;
mod itasca_events;
mod itasca_gatehouse;
mod itasca_geometry;
mod itasca_story;
mod leech_shore;
mod leech_shore_connections;
mod leech_shore_geometry;
mod marais_regional_services;
mod marais_shore_art;
mod voyageurs_ice;
pub use voyageurs_ice::{build_voyageurs_ice_modpack, revise_voyageurs_ice_turns};
mod ely_league;
mod ely_league_rooms;
mod ely_league_scripts;
pub use ely_league::build_ely_league_modpack;
mod north_shore_story;
pub use itasca_story::build_itasca_story_modpack;
pub use leech_shore::build_leech_shore_modpack;
pub use north_shore_story::build_north_shore_story_modpack;
mod canonical_city_gym;
mod gym_refresh;
mod gym_refresh_names;
mod gym_refresh_roster;
pub use gym_refresh::refresh_installed_minnesota_gyms;
mod regional_gyms_modpack;
mod runtime_landmark_verification;
pub use regional_gyms_modpack::{RegionalGymSources, build_regional_gyms_modpack};
mod st_paul_connections;
mod st_paul_design;
mod st_paul_modpack;
pub use st_paul_modpack::build_st_paul_modpack;
mod detroit_lakes_connections;
mod detroit_lakes_modpack;
mod detroit_lakes_quest;
mod st_cloud_design;
mod st_cloud_modpack;
pub use detroit_lakes_modpack::build_detroit_lakes_modpack;
mod international_falls_connections;
mod international_falls_modpack;
mod international_falls_story;
pub use international_falls_modpack::build_international_falls_modpack;
mod moorhead_connections;
mod moorhead_modpack;
mod moorhead_story;
pub use moorhead_modpack::build_moorhead_modpack;
mod winona_station_edges;
mod winona_station_modpack;
mod winona_station_story;
pub use winona_station_modpack::build_winona_station_modpack;
mod shore_gate_geometry;
mod shore_gate_modpack;
pub use shore_gate_modpack::build_shore_gate_modpack;
mod split_rock_clinic;
mod split_rock_clinic_geometry;
pub use split_rock_clinic::build_split_rock_clinic_pack;
mod authored_regional_approach;
mod regional_town_fishing;
mod staples_station;
mod staples_station_geometry;
mod staples_station_roster;
pub use staples_station::build_staples_station_pack;
mod glacier_line_geometry;
mod glacier_line_preservation;
mod glacier_line_scripts;
mod glacier_line_service;
pub use glacier_line_service::build_glacier_line_service;
#[cfg(test)]
mod glacier_line_tests;
mod st_cloud_connections;
pub use st_cloud_modpack::build_st_cloud_modpack;
mod mankato_gym;
mod mankato_modpack;
mod mankato_publication;
pub use mankato_modpack::build_mankato_modpack;
mod minneapolis_modpack;
mod rochester_connections;
pub use rochester_connections::connect_winona_rochester;
mod courier_connections;
mod courier_events;
mod courier_story;
mod courier_trainer;
mod prairie_story;
mod prairie_story_connections;
mod river_city_connections;
mod river_city_geometry;
mod rochester_gym;
mod rochester_modpack;
mod rochester_publication;
mod rochester_story;
pub use courier_story::build_courier_story_modpack;
mod lake_country_checkpoints;
mod lake_country_geometry;
mod lake_country_story;
pub use lake_country_story::build_lake_country_story_modpack;
pub use river_city_connections::connect_river_cities;
mod gym_four_approach;
pub use gym_four_approach::build_gym_four_approach;
mod prairie_notebook;
pub use prairie_story::build_prairie_story_modpack;
pub use rochester_modpack::build_rochester_modpack;
mod winona_modpack;
mod winona_opening;
mod winona_publication;
pub use winona_modpack::build_winona_opening_modpack;
mod minneapolis_gym;
mod minneapolis_gym_revision;
pub use minneapolis_gym_revision::replace_installed_minneapolis_gym;
pub use minneapolis_modpack::build_minneapolis_modpack;
mod grand_rapids_connections;
pub use grand_rapids_connections::connect_grand_rapids;
mod grand_rapids_sign_anchors;
pub use grand_rapids_sign_anchors::revise_grand_rapids_sign_anchors;
mod grand_rapids_depot_facade;
pub use grand_rapids_depot_facade::revise_grand_rapids_building_facades;
mod grand_marais_mart_facade;
pub use grand_marais_mart_facade::revise_grand_marais_mart_facade;
mod rochester_mart_facade;
pub use rochester_mart_facade::revise_rochester_mart_facade;
mod mankato_mart_facade;
pub use mankato_mart_facade::revise_mankato_mart_facade;
mod north_shore_footbridges;
mod north_shore_passing_width;
mod r05_cache_bank;
mod r06_south_woods_grass;
mod r07_north_pocket_grass;
mod r07_stair_guidance;
mod r08_shore_grass;
mod r09_southwest_grass;
mod r10_neck_grass;
mod r11_upper_grass;
mod r13_pine_grass;
pub use north_shore_footbridges::rebuild_north_shore_footbridges;
pub use north_shore_passing_width::rebuild_north_shore_passing_width;
mod r08_stair_guidance;
mod r10_stair_guidance;
mod r11_stair_guidance;
pub use r05_cache_bank::revise_r05_cache_bank;
pub use r06_south_woods_grass::revise_r06_south_woods_grass;
pub use r08_shore_grass::revise_r08_shore_grass;
pub use r09_southwest_grass::revise_r09_southwest_grass;
pub use r10_neck_grass::revise_r10_neck_grass;
pub use r11_upper_grass::revise_r11_upper_grass;
pub use r13_pine_grass::revise_r13_pine_grass;
mod r15_shore_guidance;
pub use r15_shore_guidance::revise_r15_shore_guidance;
mod r15_staples_spur;
pub use r15_staples_spur::revise_r15_staples_spur;
mod r16_bank_guidance;
pub use r16_bank_guidance::revise_r16_bank_guidance;
mod r17_trail_guidance;
pub use r17_trail_guidance::revise_r17_trail_guidance;
mod r13_boundary_alignment;
pub use r07_north_pocket_grass::revise_r07_north_pocket_grass;
pub use r07_stair_guidance::revise_r07_stair_guidance;
pub use r08_stair_guidance::revise_r08_stair_guidance;
pub use r10_stair_guidance::revise_r10_stair_guidance;
pub use r11_stair_guidance::revise_r11_stair_guidance;
pub use r13_boundary_alignment::align_r13_boundary;
mod r03_stair_guidance;
pub use r03_stair_guidance::revise_r03_stair_guidance;
mod r01_stair_guidance;
pub use r01_stair_guidance::revise_r01_stair_guidance;
mod brainerd_mart_facade;
pub use brainerd_mart_facade::revise_brainerd_mart_facade;
mod winona_mart_facade;
pub use winona_mart_facade::revise_winona_mart_facade;
mod bemidji_mart_facade;
pub use bemidji_mart_facade::revise_bemidji_mart_facade;
mod bemidji_plateau_stairs;
pub use bemidji_plateau_stairs::revise_bemidji_plateau_stairs;
mod bemidji_roof_plateau;
pub use bemidji_roof_plateau::rebuild_bemidji_roof_plateau;
mod duluth_mart_facade;
pub use duluth_mart_facade::revise_duluth_mart_facade;
mod st_cloud_service_fronts;
pub use st_cloud_service_fronts::revise_st_cloud_service_fronts;
mod r14_forest_bank;
pub use r14_forest_bank::revise_r14_forest_bank;
mod iron_range_descent_modpack;
pub use iron_range_descent_modpack::build_iron_range_descent_modpack;
mod red_river_prairie_modpack;
pub use red_river_prairie_modpack::build_red_river_prairie_modpack;
mod border_pines_art;
pub use border_pines_art::build_border_pines_art_pack;
mod border_pines_modpack;
pub use border_pines_modpack::build_border_pines_modpack;
mod mille_lacs_park;
mod mille_lacs_shore_modpack;
pub use mille_lacs_park::build_mille_lacs_park;
pub use mille_lacs_shore_modpack::{
    SHORE_WATCH_REPORT_EVENTS, SHORE_WATCH_REWARDED_EVENT, build_mille_lacs_shore_modpack,
};
mod voyageurs_border_loop_modpack;
pub use voyageurs_border_loop_modpack::build_voyageurs_border_loop_modpack;
mod prairie_forest_link_modpack;
pub use prairie_forest_link_modpack::{LAKE_OBSERVATION_EVENTS, build_prairie_forest_link_modpack};
mod root_beer_cabin;
pub use root_beer_cabin::{
    PreparedRootBeerCabin, ROOT_BEER_CABIN_CONSTANT, ROOT_BEER_CABIN_MAP, prepare_root_beer_cabin,
};
mod iron_range_art;
pub use iron_range_art::build_iron_range_art_pack;
mod prairie_grass_art;
pub use prairie_grass_art::build_prairie_grass_art_pack;
mod winter_outdoor_art;
pub use winter_outdoor_art::build_winter_outdoor_art_pack;
mod winter_mixed_edges_art;
pub use winter_mixed_edges_art::append_winter_mixed_edges_art;
mod winter_region_design;
mod winter_region_events;
mod winter_region_modpack;
mod winter_region_validation;
pub use winter_region_modpack::build_winter_region_modpack;
mod winter_sections_design;
mod winter_sections_modpack;
mod winter_sections_validation;
pub use winter_sections_modpack::build_winter_sections_modpack;
mod winter_roof_fidelity;
pub use winter_roof_fidelity::build_winter_roof_fidelity_pack;
mod river_continuity_bend;
pub use river_continuity_bend::build_river_continuity_bend_pack;
mod minnesota_fruit_groves;
pub use minnesota_fruit_groves::{
    build_minnesota_fruit_groves, build_mississippi_activity_revision,
};

mod river_bank_refinement;
pub use river_bank_refinement::refine_mississippi_banks;
mod frozen_water;
mod river_content_lineage;
pub use frozen_water::build_frozen_water;
mod river_route_fun;
pub use river_route_fun::build_river_route_fun;
mod river_continuity_long;
pub use river_continuity_long::build_river_continuity_long_pack;
mod minneapolis_house_joins;
pub use minneapolis_house_joins::build_minneapolis_house_joins_pack;
mod minneapolis_waterfront;
pub use minneapolis_waterfront::build_minneapolis_waterfront_pack;
mod park_pond_route;
pub use park_pond_route::build_park_pond_route_pack;
mod north_shore_warm;
pub use north_shore_warm::build_north_shore_warm_pack;
mod north_shore_coves;
pub use north_shore_coves::build_north_shore_coves_pack;
mod inland_snow_pines;
pub use inland_snow_pines::build_inland_snow_pines_pack;
mod winter_gunflint_alias;
pub use winter_gunflint_alias::build_winter_gunflint_alias_pack;
mod grand_marais_superior;
pub use grand_marais_superior::build_grand_marais_superior_pack;
mod superior_inland_geography;
pub use superior_inland_geography::build_superior_inland_geography_modpack;
mod inland_winter;
pub use inland_winter::build_inland_winter_pack;
mod compact_superior;
mod compact_superior_geography_validation;
pub use compact_superior::build_compact_superior_pack;
pub use compact_superior_geography_validation::validate_compact_superior_geography;
mod prairie_grass_modpack;
pub use prairie_grass_modpack::{PrairieGrassRevision, build_prairie_grass_modpack};
mod duluth_connections;
pub use duluth_connections::connect_duluth;
mod duluth_story;
pub use duluth_story::{build_duluth_story_modpack, refresh_duluth_story_dialogue};

mod ryans_house;
pub use ryans_house::build_ryans_house_modpack;
mod north_shore_shops;
pub use north_shore_shops::build_north_shore_shops;
mod minnesota_game_corner;
pub use minnesota_game_corner::build_minnesota_game_corner;
mod duluth_city_modpack;
pub use duluth_city_modpack::build_duluth_city_modpack;
mod brainerd_city_geometry;
mod brainerd_city_modpack;
mod brainerd_city_publication;
mod brainerd_city_scripts;
pub use brainerd_city_modpack::build_brainerd_city_modpack;
mod bemidji_city_art;
mod bemidji_city_modpack;
pub use bemidji_city_modpack::{BEMIDJI_CITY_GROUP, BEMIDJI_CITY_MAP, build_bemidji_city_modpack};
mod root_beer_lady_modpack;
pub use root_beer_lady_modpack::{
    PreparedRootBeerLadyService, ROOT_BEER_FIRST_GIFT_EVENT, ROOT_BEER_ITEM_ID,
    ROOT_BEER_MANIFEST_ID, ROOT_BEER_MART_ID, ROOT_BEER_SERVICE_MANIFEST_ID, RootBeerRestPolicy,
    build_root_beer_lady_modpack, prepare_root_beer_lady_service,
    publish_root_beer_lady_service_map,
};
mod headwaters_modpack;
pub use headwaters_modpack::build_headwaters_modpack;
mod driftless_modpack;
mod leech_lake_modpack;
mod northern_lakes_modpack;
mod pine_country_modpack;
mod prairie_farm_modpack;
mod river_bluffs_modpack;
mod river_valley_modpack;
mod twin_cities_art;
mod twin_cities_modpack;
pub use driftless_modpack::build_driftless_modpack;
pub use leech_lake_modpack::build_leech_lake_modpack;
pub use northern_lakes_modpack::build_northern_lakes_modpack;
pub use pine_country_modpack::build_pine_country_modpack;
pub use prairie_farm_modpack::build_prairie_farm_modpack;
pub use river_bluffs_modpack::build_river_bluffs_modpack;
pub use river_valley_modpack::build_river_valley_modpack;
pub use twin_cities_modpack::build_twin_cities_modpack;
mod central_lakes_modpack;
pub use central_lakes_modpack::build_central_lakes_modpack;

mod rochester_maze_revision;
pub use rochester_maze_revision::revise_rochester_maze;
mod rochester_gate_revision;
pub use rochester_gate_revision::revise_rochester_gates;
mod gym_room_revision;
mod gym_selection;
pub use gym_selection::read_minnesota_gym_design;
mod current_gym_refresh;
pub use current_gym_refresh::refresh_current_minnesota_gyms;
mod mankato_terrace_revision;
pub use mankato_terrace_revision::revise_mankato_terraces;
mod minneapolis_circuit_revision;
pub use minneapolis_circuit_revision::revise_minneapolis_circuit;

mod st_paul_loop_revision;
pub use st_paul_loop_revision::revise_st_paul_loop;

mod gym_wind_art;

mod st_cloud_arrow_revision;
pub use st_cloud_arrow_revision::revise_st_cloud_arrows;

mod brainerd_winch_revision;
pub use brainerd_winch_revision::revise_brainerd_winch;

mod bemidji_return_revision;
pub use bemidji_return_revision::revise_bemidji_return;
mod duluth_gate_revision;
pub use duluth_gate_revision::revise_duluth_gates;

mod minnesota_city_life;
pub use minnesota_city_life::build_minnesota_city_life_modpack;
mod compact_interior_story;
pub use compact_interior_story::build_compact_interior_story_pack;
mod accepted_terrain_rebase;
pub use accepted_terrain_rebase::rebase_accepted_terrain;
mod interior_conversation_relocations;
mod minnesota_interior_life;
mod minnesota_interior_lineage;
pub use minnesota_interior_life::build_minnesota_interior_life_modpack;
mod interior_read_relocations;
mod minnesota_interior_layouts;
pub use minnesota_interior_layouts::build_minnesota_interior_layout_modpack;

mod minnesota_native_npc;
#[cfg(test)]
mod minnesota_original_npc_tests;
mod minnesota_trainer_art;
mod minnesota_trainer_distribution;
pub use minnesota_trainer_art::bind_minnesota_trainer_art;
pub use minnesota_trainer_distribution::distribute_minnesota_trainer_art;
mod minnesota_script_repair;
pub use minnesota_script_repair::repair_minnesota_script_commands;

mod minnesota_trainers;
pub use minnesota_trainers::{
    build_minneapolis_cyclists_modpack, build_minnesota_trainers_modpack,
    build_original_npc_modpack, revise_original_npc_overworld,
};

mod ely_shore_pass;
pub use ely_shore_pass::build_ely_shore_pass_pack;

mod ely_snow_art;

mod canoe_modpack;
mod canoe_sprites;
mod canoe_story_hints;
mod canoe_travel;
pub use canoe_modpack::build_canoe_modpack;

mod verdigrim_intake_art;
mod verdigrim_scene;
pub use verdigrim_scene::build_verdigrim_scene;

mod duluth_story_composition;
pub use duluth_story_composition::compose_duluth_story_terrain;

mod current_minnesota_release_composition;
pub use current_minnesota_release_composition::compose_current_minnesota_release;

mod northstar_rescue_refresh;
pub use northstar_rescue_refresh::refresh_northstar_rescue_evidence;

// Minnesota optional discovery and field-use publishers.
mod boundary_cabin_cast;
pub use boundary_cabin_cast::revise_boundary_cabin_cast;

mod devils_kettle;
pub use devils_kettle::build_devils_kettle;

mod devils_kettle_clinic;

mod discovery_dialogue;

mod flarkindle_discovery;
pub use flarkindle_discovery::{FlarkindleDiscoveryPrograms, build_flarkindle_discovery_programs};

mod flarkindle_discovery_pack;
pub use flarkindle_discovery_pack::build_flarkindle_discovery_pack;

mod icemyth_discovery;
pub use icemyth_discovery::{IcemythDiscoveryPrograms, build_icemyth_discovery_programs};

mod icemyth_discovery_pack;
pub use icemyth_discovery_pack::build_icemyth_discovery_pack;

mod kensigard_discovery;
pub use kensigard_discovery::{KensigardDiscoveryPrograms, build_kensigard_discovery_programs};

mod kensigard_discovery_pack;
pub use kensigard_discovery_pack::build_kensigard_discovery_pack;

mod minneapolis_cycling_dialogue;
pub use minneapolis_cycling_dialogue::refresh_minneapolis_cycling_dialogue;

mod minneapolis_cycling_routes;
pub use minneapolis_cycling_routes::revise_minneapolis_cycling_routes;

mod iron_crosscut;
pub use iron_crosscut::rebuild_iron_crosscut;

mod ely_pine_camp;
pub use ely_pine_camp::rebuild_ely_pine_camp;

mod iron_approach_revision;
pub use iron_approach_revision::revise_iron_drill_approach;
mod r10_exploration;
pub use r10_exploration::rebuild_r10_exploration;
mod r10_survey_spurs;
pub use r10_survey_spurs::revise_r10_survey_spurs;
mod international_falls_windbreak;
pub use international_falls_windbreak::rebuild_international_falls_windbreak;
mod r20_winding_cleft;
pub use r20_winding_cleft::rebuild_r20_winding_cleft;
mod r17_talus_loop;
pub use r17_talus_loop::revise_r17_talus_loop;
mod iron_haulage_rock;
mod iron_haulage_rock_art;
pub use iron_haulage_rock::rebuild_iron_haulage_rock;
mod iron_drill_core;
mod iron_drill_core_art;
mod iron_pump_gallery;
mod iron_sump_basin;
mod iron_sump_shore_art;
mod iron_ventilation_art;
mod iron_ventilation_cleft;
pub use iron_drill_core::rebuild_iron_drill_core;
mod r13_cedar_basin;
pub use r13_cedar_basin::rebuild_r13_cedar_basin;

mod gunflint_spring_shelter;
pub use gunflint_spring_shelter::rebuild_gunflint_spring_shelter;

mod gunflint_grotto;
pub use gunflint_grotto::rebuild_gunflint_grotto;
mod grand_rapids_riverfront;
mod grand_rapids_riverfront_art;
pub use grand_rapids_riverfront::rebuild_grand_rapids_riverfront;
pub use iron_pump_gallery::rebuild_iron_pump_gallery;
pub use iron_sump_basin::rebuild_iron_sump_basin;
pub use iron_ventilation_cleft::rebuild_iron_ventilation_cleft;

mod driftless_fossil_gallery;
pub use driftless_fossil_gallery::build_driftless_fossil_gallery;

mod leech_lake_island;
pub use leech_lake_island::build_leech_lake_island;

mod leech_lake_mainland;
pub use leech_lake_mainland::rebuild_leech_lake_mainland;

mod minnesota_field_use;
pub use minnesota_field_use::build_minnesota_field_use_pack;

mod minnesota_field_use_art;

mod comprehensive_league_seam;
pub use comprehensive_league_seam::repair_comprehensive_league_seam;
mod comprehensive_cave_successors;
pub use comprehensive_cave_successors::integrate_comprehensive_cave_successors;
mod comprehensive_story_support;
pub use comprehensive_story_support::integrate_comprehensive_story_support;
mod comprehensive_haulage_successors;
pub use comprehensive_haulage_successors::integrate_comprehensive_haulage_successors;
mod comprehensive_sump_successors;
pub use comprehensive_sump_successors::integrate_comprehensive_sump_successors;
mod comprehensive_core_shelter;
pub use comprehensive_core_shelter::integrate_comprehensive_core_shelter;
mod comprehensive_accepted_successors;
pub use comprehensive_accepted_successors::integrate_comprehensive_accepted_successors;
mod comprehensive_fossil_lab;
pub use comprehensive_fossil_lab::integrate_comprehensive_fossil_lab;

mod comprehensive_riverfront_gate_spurs;
pub use comprehensive_riverfront_gate_spurs::integrate_comprehensive_riverfront_gate_spurs;
mod comprehensive_vent_bridges;
pub use comprehensive_vent_bridges::integrate_comprehensive_vent_bridges;
mod comprehensive_r10_successor;
pub use comprehensive_r10_successor::integrate_comprehensive_r10_successor;
mod comprehensive_road_successors;
pub use comprehensive_road_successors::integrate_comprehensive_road_successors;
mod comprehensive_current_world;
pub use comprehensive_current_world::{
    integrate_comprehensive_brainerd_roof, integrate_comprehensive_current_world,
};
mod northstar_cast_integration;
pub use northstar_cast_integration::complete_northstar_story_cast;
mod northstar_story_refresh;
pub use northstar_story_refresh::refresh_northstar_current_story;

mod northwoods_discovery;
pub use northwoods_discovery::{NorthwoodsDiscoveryPrograms, build_northwoods_discovery_programs};

mod northwoods_discovery_pack;
pub use northwoods_discovery_pack::build_northwoods_discovery_pack;

mod pepin_discovery;
pub use pepin_discovery::{PepinDiscoveryPrograms, build_pepin_discovery_programs};

mod pepin_discovery_pack;
pub use pepin_discovery_pack::build_pepin_discovery_pack;

mod strandrak_discovery;
pub use strandrak_discovery::{StrandrakDiscoveryPrograms, build_strandrak_discovery_programs};

mod strandrak_discovery_pack;
pub use strandrak_discovery_pack::build_strandrak_discovery_pack;

mod minnesota_forced_movement_source;

mod r19_detroit_seam;
pub use r19_detroit_seam::restore_r19_detroit_seam;

mod r19_detroit_gate;
pub use r19_detroit_gate::build_r19_detroit_gate;

mod trainer_random_selection;

mod comprehensive_water_habitats;
pub use comprehensive_water_habitats::integrate_comprehensive_water_habitats;

mod comprehensive_leech_mainland;
pub use comprehensive_leech_mainland::integrate_comprehensive_leech_mainland;

mod comprehensive_research_grotto;
pub use comprehensive_research_grotto::integrate_comprehensive_research_grotto;

mod comprehensive_iron_crosscut;
pub use comprehensive_iron_crosscut::integrate_comprehensive_iron_crosscut;

mod northstar_title;
mod northstar_title_art;
pub use northstar_title::{NORTHSTAR_TITLE_IMAGE, build_northstar_title};

mod minnesota_daycares;
#[cfg(test)]
mod minnesota_daycares_tests;
pub use minnesota_daycares::build_minnesota_daycares;

mod encounter_refinement_composition;
pub use encounter_refinement_composition::compose_minnesota_encounter_refinements;

mod northstar_story_consolidation;
pub use northstar_story_consolidation::consolidate_northstar_story;

mod northstar_public_roads;
pub use northstar_public_roads::open_northstar_public_roads;
