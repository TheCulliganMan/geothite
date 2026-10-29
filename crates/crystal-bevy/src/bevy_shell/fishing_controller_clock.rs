// The graphical update schedule already runs this same casting/hook/pause clock.
// Neutral hosts need one clock tick per controller frame too. Timed casting owns
// the entire input frame and cannot advance the authoritative map, RNG or queue.
// AwaitText remains an explicit A/B boundary handled by the existing owner.
fn advance_visible_controller_fishing_clock(runtime_shell: &mut BevyRuntimeShell) -> bool {
    if !runtime_shell
        .visible_fishing_animation
        .as_ref()
        .is_some_and(|animation| animation.phase != VisibleFishingPhase::AwaitText)
    {
        return false;
    }
    advance_visible_fishing_animation(runtime_shell);
    true
}

// A successful cast has already prepared the core encounter. The graphical
// host retains the shore scene through casting and the bite notice; neutral
// snapshots must use that same owner until A/B acknowledges the notice.
fn visible_controller_fishing_scene(
    runtime_shell: &BevyRuntimeShell,
) -> Option<&RuntimeShellSnapshot> {
    runtime_shell.visible_fishing_animation.as_ref()?;
    runtime_shell.field_notice_scene.as_deref()
}
