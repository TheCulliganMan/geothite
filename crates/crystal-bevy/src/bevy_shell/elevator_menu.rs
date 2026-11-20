//! The field display of the active elevator panel, backed by its native floor table.
use anyhow::{Context, Result, ensure};
use bevy::prelude::{Assets, Commands, Image};
use crystal_runtime::{
    RuntimeMenuLayoutSnapshot, RuntimeMenuSnapshot, RuntimeMenuSource, RuntimeShellPhase,
    RuntimeShellSnapshot, RuntimeTextSnapshot, RuntimeTextSource, RuntimeVerticalMenuSnapshot,
};

use super::{
    AssetRoot, BevyRuntimeShell, RenderedTilesetArt, battle_hud_tile_origin,
    spawn_scene_dialog_bitmap_text, spawn_scene_dialog_text_box, spawn_scene_dialog_window,
    strict_readonly_cursor_index, visible_elevator_prompt_options,
};

const ROWS: usize = 4;
const QUESTION: &str = "Which floor?";

struct Selector<'a> {
    floors: Vec<&'a str>,
    current: &'a str,
    selected: usize,
}

impl<'a> Selector<'a> {
    fn from_shell(snapshot: &'a RuntimeShellSnapshot, shell: &BevyRuntimeShell) -> Result<Self> {
        let cursor = shell
            .elevator_cursor
            .as_ref()
            .context("elevator panel requires its cursor")?;
        let options = visible_elevator_prompt_options(snapshot, shell);
        let choices: Vec<_> = options
            .into_iter()
            .flat_map(|elevator| &elevator.floors)
            .collect();
        ensure!(
            !choices.is_empty(),
            "retained elevator surface {} has no matching compiled floors on map {}",
            cursor.surface_id,
            snapshot.overworld.map_name
        );
        let selected =
            strict_readonly_cursor_index(&shell.elevator_cursor, &cursor.surface_id, choices.len())
                .context("elevator prompt is active without a valid cursor")?;
        let origin = shell
            .shell
            .session()
            .state()
            .backup_warp_map_name
            .as_deref();
        let label = |name: &'a str| name.strip_prefix("FLOOR_").unwrap_or(name);
        let current = choices
            .iter()
            .find(|f| Some(f.target_map.as_str()) == origin)
            .context("elevator panel lost the current floor")?;
        Ok(Self {
            current: label(&current.floor),
            floors: choices.into_iter().map(|f| label(&f.floor)).collect(),
            selected,
        })
    }

    fn first_row(&self) -> usize {
        self.selected.saturating_sub(ROWS - 1)
    }

    fn rows(&self) -> impl Iterator<Item = (bool, &str)> {
        self.floors
            .iter()
            .enumerate()
            .skip(self.first_row())
            .take(ROWS)
            .map(|(i, &name)| (i == self.selected, name))
    }

    fn labels(&self) -> Vec<String> {
        self.rows()
            .map(|(selected, name)| format!("{}{name}", if selected { ">" } else { " " }))
            .collect()
    }
}

pub(super) fn entries(
    snapshot: &RuntimeShellSnapshot,
    shell: &BevyRuntimeShell,
) -> Result<Vec<String>> {
    let panel = Selector::from_shell(snapshot, shell)?;
    let mut entries = vec![QUESTION.to_owned(), format!("Now on: {}", panel.current)];
    entries.extend(panel.labels());
    Ok(entries)
}

pub(super) fn project(shell: &BevyRuntimeShell, snapshot: &mut RuntimeShellSnapshot) -> Result<()> {
    let panel = Selector::from_shell(snapshot, shell)?;
    let text = format!("{QUESTION}\nNow on: {}", panel.current);
    let options = panel.labels();
    let id = "visible-shell:elevator";
    snapshot.phase = RuntimeShellPhase::Menu;
    snapshot.ui.window_open = true;
    snapshot.ui.text_window_open = true;
    snapshot.ui.text = Some(RuntimeTextSnapshot {
        label: id.into(),
        source: RuntimeTextSource::AsmText,
        asm_text: Some(text),
        body: None,
        queued_text_events: 0,
    });
    snapshot.ui.menu = Some(RuntimeMenuSnapshot {
        menu_id: id.into(),
        source: RuntimeMenuSource::SpecialRoutine,
        definition: None,
        layout: RuntimeMenuLayoutSnapshot {
            declared_coords: None,
            data_commands: Vec::new(),
            vertical_menus: vec![RuntimeVerticalMenuSnapshot {
                source_script: id.into(),
                loadmenu_command_index: 0,
                verticalmenu_command_index: 0,
                header_label: id.into(),
                data_label: None,
                options,
                two_dimensional: false,
                rows: Some(ROWS),
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

pub(super) fn draw(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    shell: &BevyRuntimeShell,
    art: &mut RenderedTilesetArt,
    root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let panel = Selector::from_shell(snapshot, shell)?;
    // Elevator_GetCurrentFloorText and Elevator_MenuHeader reserve these LCD tiles.
    spawn_scene_dialog_window(commands, art, root, images, 0., 0., 10., 6., 4.);
    spawn_scene_dialog_window(commands, art, root, images, 12., 1., 7., 9., 4.);
    spawn_scene_dialog_text_box(commands, art, root, images, 4.);
    let mut print = |column, row, text: &str| {
        let (x, y) = battle_hud_tile_origin(column, row);
        spawn_scene_dialog_bitmap_text(commands, art, root, images, text, x, y, 4.2);
    };
    print(1., 2., "Now on:");
    print(4., 4., panel.current);
    print(1., 14., QUESTION);
    for (i, (selected, floor)) in panel.rows().enumerate() {
        let y = 2. + 2. * i as f32;
        if selected {
            print(13., y, ">");
        }
        print(14., y, floor);
    }
    if panel.first_row() > 0 {
        print(17., 1., "▲");
    }
    if panel.first_row() + ROWS < panel.floors.len() {
        print(17., 9., "▼");
    }
    Ok(())
}
