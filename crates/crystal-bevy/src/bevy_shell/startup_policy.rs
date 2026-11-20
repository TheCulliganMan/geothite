//! Regional startup selection, independent of Minnesota's host-clock policy.

use anyhow::{Context, Result};
use crystal_runtime::CrystalRuntime;

use super::BevyRuntimeShell;

pub(super) fn is_minnesota(runtime: &CrystalRuntime) -> bool {
    runtime
        .modpack()
        .id()
        .split('+')
        .any(|id| id.starts_with("minnesota"))
        || runtime.map_ids().iter().any(|name| name.starts_with("Mn"))
}

pub(super) fn skips_title_sequence(runtime: &CrystalRuntime) -> bool {
    is_minnesota(runtime) && !super::northstar_opening::enabled(runtime)
}

pub(super) fn plays_source_movie(runtime: &CrystalRuntime, reduced_motion: bool) -> bool {
    !skips_title_sequence(runtime)
        && (runtime
            .data()
            .runtime_title_screen
            .static_title_image
            .is_none()
            || (super::northstar_opening::enabled(runtime) && !reduced_motion))
}

pub(super) fn open_main_menu(shell: &mut BevyRuntimeShell) -> Result<()> {
    let title = shell
        .title_menu
        .as_mut()
        .context("startup has no game menu")?;
    title
        .presentation_machine
        .memory
        .insert("wJumptableIndex".into(), 0x80);
    title
        .presentation_machine
        .memory
        .insert("wTitleScreenSelectedOption".into(), 0);
    super::activate_visible_title_main_menu(shell)?;
    // Complete the menu's own entry program before the first rendered frame.
    for _ in 0..64 {
        if shell
            .title_menu
            .as_ref()
            .is_some_and(super::visible_title_main_menu_ready)
        {
            return Ok(());
        }
        super::advance_visible_main_menu_entry(shell)?;
    }
    anyhow::bail!("Minnesota startup did not reach the game menu")
}
