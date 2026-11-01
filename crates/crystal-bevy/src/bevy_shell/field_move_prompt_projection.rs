//! Read-only text projection of the graphical contextual field-move chooser.

use anyhow::Result;
use crystal_runtime::{
    RuntimeMenuLayoutSnapshot, RuntimeMenuSnapshot, RuntimeMenuSource, RuntimeShellPhase,
    RuntimeShellSnapshot, RuntimeVerticalMenuSnapshot,
};

use super::{BevyRuntimeShell, scene_dialog_yes_no_cursor_index};

pub(super) fn project(shell: &BevyRuntimeShell, snapshot: &mut RuntimeShellSnapshot) -> Result<()> {
    if shell.pending_contextual_field_move.is_none() {
        return Ok(());
    }
    let selected = scene_dialog_yes_no_cursor_index(snapshot, shell)?;
    let options = ["YES", "NO"]
        .into_iter()
        .enumerate()
        .map(|(index, label)| format!("{}{label}", if index == selected { ">" } else { " " }))
        .collect();
    // This is a visible-controller menu, not a compiled `yesorno` command.
    // Keep its native text above the options and mirror the production cursor
    // explicitly; the text renderer never owns or resolves this selection.
    snapshot.phase = RuntimeShellPhase::Menu;
    snapshot.ui.menu = Some(RuntimeMenuSnapshot {
        menu_id: "field:move-confirm".into(),
        source: RuntimeMenuSource::SpecialRoutine,
        definition: None,
        layout: RuntimeMenuLayoutSnapshot {
            declared_coords: None,
            data_commands: Vec::new(),
            vertical_menus: vec![RuntimeVerticalMenuSnapshot {
                source_script: "field:move-confirm".into(),
                loadmenu_command_index: 0,
                verticalmenu_command_index: 0,
                header_label: "field:move-confirm".into(),
                data_label: None,
                options,
                two_dimensional: false,
                rows: None,
                columns: None,
                spacing: None,
            }],
        },
        window_open: true,
        coords: None,
        menu_2d_requested: false,
    });
    Ok(())
}
