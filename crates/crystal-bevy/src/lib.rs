//! Bevy frontend for the shared Geothite runtime.

mod battle_anim_machine;
#[cfg(feature = "voxel-view")]
pub mod loading_pack_trace;
#[cfg(feature = "browser-webgpu")]
mod browser_shader;
#[cfg(all(target_arch = "wasm32", feature = "browser-webgpu"))]
pub mod browser_backend;
#[cfg(all(target_arch = "wasm32", feature = "allocation-tester"))]
mod browser_allocations;
#[cfg(all(target_arch = "wasm32", feature = "allocation-tester"))]
pub fn enable_loading_allocation_stacks() { browser_allocations::enable_stack_sampling(); }
#[cfg(feature = "operation-trace")]
pub mod operation_trace;

#[cfg(test)]
use crystal_assets::RuntimePendingScriptRequestKind;
use crystal_assets::{
    RuntimeMutationCommand, RuntimeMutationResult, RuntimePendingScriptRequest,
    RuntimeStaticWildBattleOrigin, RuntimeStoredPokemonNicknameCommand,
};
use crystal_core::battle::start::TrainerBattleStartStatus;
use crystal_runtime::*;

#[cfg(feature = "bevy-shell")]
// The visible shell keeps private deterministic smoke/control helpers that are
// exercised selectively by feature-specific and platform-specific builds.
#[allow(dead_code)]
pub mod bevy_shell;
#[cfg(feature = "bevy-shell")]
pub use bevy_shell::{
    BevyHostedMultiplayerConfig, BevyMeshtasticConfig, BevyMeshtasticConnection,
    BattleOriginContact, BattleOriginKind, BattleOriginVisualStep,
    BattlePresentationOrigin, BattlePresentationOriginFrame,
    BevyMultiplayerConfig, BevyShellConfig, BevyShellStart, VisibleShellBattleSmoke,
    VisibleShellBattleSmokeRef, VisibleShellController, VisibleShellOverworldSmoke,
    VisibleShellPartySmoke, VisibleShellSmokeItem, VisibleShellSmokePokemon,
    VisibleShellStartMenuSmoke, VisibleShellTitleNameInputSmoke, VisibleShellTitleSmoke,
    VisibleShellTrainerBattleSmoke, run_bevy_shell, smoke_visible_shell_overworld,
    smoke_visible_shell_party, smoke_visible_shell_start_menu, smoke_visible_shell_title,
    smoke_visible_shell_title_name_input, smoke_visible_shell_trainer_battle,
    smoke_visible_shell_wild_battle,
};
