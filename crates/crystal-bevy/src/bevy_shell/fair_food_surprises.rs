// Restore from authoritative pending state, never from a presentation-only flag.
fn settle_visible_pending_fair_food(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(pending) = runtime_shell
        .shell
        .session()
        .state()
        .pending_fair_food
        .as_ref()
    else {
        return Ok(());
    };
    let reveal = pending.reveal();
    if let Some(shop) = runtime_shell
        .shell
        .session()
        .state()
        .script_runtime
        .pending_shop
        .clone()
    {
        // Shop UI cursors are process-local. Continue must also restore the
        // vendor's native closetext/end continuation before releasing battle.
        if runtime_shell.active_script_cursor.is_none() {
            arm_visible_active_script_cursor(
                runtime_shell,
                &shop.source_script,
                shop.command_index
                    .checked_add(1)
                    .context("Fair shop command index overflow")?,
            );
        }
        if runtime_shell.shop_notice.is_none() {
            runtime_shell.shop_notice = Some(reveal.into());
            runtime_shell.shop_close_after_notice = true;
            runtime_shell.shop_return_to_top_after_notice = false;
            runtime_shell.shop_welcome_seen = true;
            mark_runtime_snapshot_dirty(runtime_shell);
        }
        return Ok(());
    }
    let state = runtime_shell.shell.session().state();
    if runtime_shell.active_script_cursor.is_some()
        || state.script_runtime.text_window_open
        || state.script_runtime.next_script.is_some()
        || state.script_runtime.active_menu.is_some()
    {
        return Ok(());
    }
    if runtime_shell.shell.start_pending_fair_food_battle()? {
        prepare_visible_battle_entry(runtime_shell)?;
    }
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}
