// Renderer-neutral input has no graphical update schedule. Advance the existing
// item presentation sequence after its real printer completes, before returning
// ownership to the next button. The pocket/refusal pages still require A/B.
fn advance_visible_controller_item_notice(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    if advance_visible_field_item_fanfare_pause(runtime_shell) {
        return Ok(true);
    }
    let found_text_ready = runtime_shell
        .visible_field_item_notice
        .as_ref()
        .is_some_and(|notice| {
            notice.phase == VisibleFieldItemPhase::FoundText
                && runtime_shell.field_notice.as_deref() == Some(notice.sound_trigger_text.as_str())
        });
    if !found_text_ready {
        return Ok(false);
    }
    let presentation = runtime_shell.shell.presentation_snapshot()?;
    if !visible_field_dialogue_is_fully_revealed(runtime_shell, &presentation) {
        return Ok(false);
    }
    begin_visible_field_item_sound(runtime_shell)?;
    Ok(true)
}
