//! The browser paints the very same Ratatui cell buffer as the native client.
//! HTML is only a transport for cells; all layout and gameplay remain Rust.
use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier},
};
use std::fmt::Write;

fn session_save_path(modpack: &str, session: &str) -> Result<std::path::PathBuf, &'static str> {
    if session.len() > 64 || !session.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_') {
        return Err("Session must contain at most 64 ASCII letters, digits, hyphens or underscores");
    }
    let suffix = if session.is_empty() { "local".to_owned() } else { format!("session-{session}") };
    Ok(format!("saves/{modpack}-tui-{suffix}.crystalsave").into())
}

fn css_color(color: Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Reset => "inherit".into(),
        _ => "inherit".into(),
    }
}

pub(crate) fn buffer_html(buffer: &Buffer) -> String {
    let mut html = String::new();
    for y in buffer.area.top()..buffer.area.bottom() {
        let mut style = None;
        for x in buffer.area.left()..buffer.area.right() {
            let cell = &buffer[(x, y)];
            let next = (cell.fg, cell.bg, cell.modifier);
            if style != Some(next) {
                if style.is_some() {
                    html.push_str("</span>");
                }
                let _ = write!(
                    html,
                    "<span style=\"color:{};background-color:{};font-weight:{}\">",
                    css_color(cell.fg),
                    css_color(cell.bg),
                    if cell.modifier.contains(Modifier::BOLD) {
                        "bold"
                    } else {
                        "normal"
                    }
                );
                style = Some(next);
            }
            for ch in cell.symbol().chars() {
                match ch {
                    '&' => html.push_str("&amp;"),
                    '<' => html.push_str("&lt;"),
                    '>' => html.push_str("&gt;"),
                    '"' => html.push_str("&quot;"),
                    '\'' => html.push_str("&#39;"),
                    _ => html.push(ch),
                }
            }
        }
        if style.is_some() {
            html.push_str("</span>");
        }
        html.push('\n');
    }
    html
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::buffer_html;
    use crate::{RuntimeTextRenderer, TerminalUi, TuiTheme, ViewportSize};
    use crystal_assets::{AssetRoot, load_verified_compiled_game_pack_bytes};
    use crystal_bevy::VisibleShellController;
    use crystal_core::input::GameButton;
    use crystal_runtime::CrystalRuntime;
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;
    use wasm_bindgen::prelude::*;

    fn js_error(error: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&error.to_string())
    }

    #[wasm_bindgen]
    pub struct BrowserTui {
        game: VisibleShellController,
        renderer: RuntimeTextRenderer,
        ui: TerminalUi,
        save_path: PathBuf,
        painted: crate::painted::PaintedRenderer,
    }

    #[wasm_bindgen]
    impl BrowserTui {
        /// Open the identity-matched browser save, or start CHRIS immediately.
        #[wasm_bindgen(js_name = open)]
        pub fn open(pack: &[u8]) -> Result<BrowserTui, JsValue> {
            Self::open_session(pack, "")
        }

        /// Named slots are browser-local, not public bearer credentials. Keep
        /// the empty/default slot exactly where existing players saved it.
        pub fn open_session(pack: &[u8], session: &str) -> Result<BrowserTui, JsValue> {
            let loaded =
                load_verified_compiled_game_pack_bytes("browser-tui.crystalpack", pack.to_vec())
                    .map_err(js_error)?;
            let root = AssetRoot::new(".");
            let runtime =
                CrystalRuntime::from_loaded_compiled_pack(&root, loaded).map_err(js_error)?;
            let path = super::session_save_path(runtime.modpack().id(), session).map_err(js_error)?;
            let storage = web_sys::window()
                .ok_or_else(|| js_error("No browser window"))?
                .local_storage()?
                .ok_or_else(|| js_error("Browser save storage unavailable"))?;
            let resume = storage
                .get_item(&format!("crystal.save.v1.{}", path.display()))?
                .is_some() || storage.get_item(&format!("crystal.save.v1.{}.bak", path.display()))?.is_some();
            let mut tui = Self::from_runtime(root, runtime, path, "CHRIS", resume)?;
            tui.checkpoint()?;
            Ok(tui)
        }
        #[wasm_bindgen(constructor)]
        pub fn new(pack: &[u8], name: &str, resume: bool) -> Result<BrowserTui, JsValue> {
            let loaded =
                load_verified_compiled_game_pack_bytes("browser-tui.crystalpack", pack.to_vec())
                    .map_err(js_error)?;
            let root = AssetRoot::new(".");
            let runtime =
                CrystalRuntime::from_loaded_compiled_pack(&root, loaded).map_err(js_error)?;
            let save_path = super::session_save_path(runtime.modpack().id(), "").map_err(js_error)?;
            Self::from_runtime(root, runtime, save_path, name, resume)
        }

        pub fn checkpoint(&mut self) -> Result<bool, JsValue> {
            self.game.autosave(&self.save_path).map_err(js_error)
        }

        pub fn session_path(&self) -> String { self.save_path.display().to_string() }
    }

    impl BrowserTui {
        fn from_runtime(root: AssetRoot, runtime: CrystalRuntime, save_path: PathBuf, name: &str, resume: bool) -> Result<Self, JsValue> {
            if name.trim().is_empty() || name.chars().count() > 7 {
                return Err(js_error("Trainer name must contain 1–7 characters"));
            }
            let mut game = if resume {
                VisibleShellController::load_save(
                    root,
                    runtime,
                    save_path.clone(),
                    Some(save_path.clone()),
                )
            } else {
                VisibleShellController::new_game(root, runtime, name, Some(save_path.clone()))
            }
            .map_err(js_error)?;
            game.set_runtime_journal_enabled(false);
            game.set_battle_replay_enabled(true);
            Ok(Self {
                game,
                renderer: RuntimeTextRenderer::default(),
                ui: TerminalUi::new(TuiTheme::default()),
                save_path,
                painted: crate::painted::PaintedRenderer::default(),
            })
        }
    }

    #[wasm_bindgen]
    impl BrowserTui {

        /// Game Boy inputs, suitable for both humans and browser agents.
        pub fn press(&mut self, button: &str) -> Result<(), JsValue> {
            self.painted.cancel_replay();
            let button = match button.to_ascii_lowercase().as_str() {
                "up" => GameButton::Up,
                "down" => GameButton::Down,
                "left" => GameButton::Left,
                "right" => GameButton::Right,
                "a" => GameButton::A,
                "b" => GameButton::B,
                "start" => GameButton::Start,
                "select" => GameButton::Select,
                _ => return Err(js_error("Expected up/down/left/right/a/b/start/select")),
            };
            let snapshot = self.game.presentation_snapshot().map_err(js_error)?;
            if matches!(button, GameButton::Up | GameButton::Down) {
                let delta = if button == GameButton::Up { -1 } else { 1 };
                if snapshot.ui.pending_yes_no.is_some() {
                    self.renderer.move_selection(delta, 2, true);
                } else if let Some(menu) = snapshot.ui.menu.as_ref()
                    && let Some(vertical) = menu.layout.vertical_menus.first()
                    && !vertical
                        .options
                        .iter()
                        .any(|option| option.trim_start().starts_with('>'))
                {
                    self.renderer
                        .move_selection(delta, vertical.options.len(), false);
                }
            }
            self.game.press(button).map_err(js_error)?;
            self.painted.replay(self.game.take_battle_replays());
            self.ui.notice = None;
            self.renderer.record_action(format!("input: {button:?}"));
            self.checkpoint()?;
            Ok(())
        }

        pub fn observe(&mut self) -> Result<String, JsValue> {
            let snapshot = self.game.presentation_snapshot().map_err(js_error)?;
            serde_json::to_string(&self.renderer.render(&snapshot)).map_err(js_error)
        }
        /// Code mode runs in the same Rust interpreter as native stdio MCP.
        pub fn search(&self, query: &str) -> Result<String, JsValue> {
            if query.len() > 512 {
                return Err(js_error("query exceeds 512 bytes"));
            }
            serde_json::to_string(&crate::code_mode_catalog(query)).map_err(js_error)
        }
        pub fn execute_code(&mut self, code: &str) -> Result<String, JsValue> {
            let result = crate::execute_code(code, |name, args| {
                let result = crate::call_code_mode_tool(
                    &mut self.game,
                    &mut self.renderer,
                    name,
                    args,
                    |game| {
                        game.save(&self.save_path)?;
                        Ok(())
                    },
                );
                // Batching never makes animation input-owning. Retain only the
                // last input's cosmetic replay, not an unbounded replay queue.
                if matches!(name, "press" | "move") {
                    self.painted.cancel_replay();
                    self.painted.replay(self.game.take_battle_replays());
                    self.ui.notice = None;
                    // Persist even when an earlier tap applied before a tool
                    // error. Never save an unfinished battle/script boundary.
                    self.game.autosave(&self.save_path)?;
                }
                result
            });
            serde_json::to_string(&result.map_err(js_error)?).map_err(js_error)
        }
        /// A cosmetic clock only. Never ticks the controller or owns input.
        pub fn advance_visual(&mut self) -> bool { self.painted.advance_replay() }
        pub fn visual_active(&self) -> bool { self.painted.replay_active() }
        pub fn cancel_visual(&mut self) { self.painted.cancel_replay(); }
        /// High-density dot layer, same camera/tiles as the semantic terminal.
        pub fn world_dots(&mut self) -> Result<String,JsValue> {
            let source=self.game.presentation_snapshot().map_err(js_error)?;
            Ok(self.painted.detailed_world(&source).map(|b|buffer_html(&b)).unwrap_or_default())
        }
        pub fn world_bounds(&self) -> Vec<u16> {self.painted.scene_bounds()}
        pub fn advance_ink(&mut self, seconds: f64) {self.painted.advance_ink_by(seconds);}
        pub fn world_dot_sizes(&self)->Vec<f64> {self.painted.animated_dot_sizes()}

        pub fn render(&mut self, columns: u16, rows: u16) -> Result<String, JsValue> {
            let columns = columns.clamp(40, 240);
            let rows = rows.clamp(12, 100);
            // Match TerminalUi's two-pane layout. Keep map cells on one line
            // on phones instead of wrapping the world into misleading rows.
            let wide = columns >= 96;
            let panel_width = if wide { columns * 66 / 100 } else { columns };
            let panel_height = if wide {
                rows - 5
            } else {
                (rows - 5) * 62 / 100
            };
            self.renderer.set_viewport(ViewportSize {
                width: ((panel_width.saturating_sub(6) / 3).clamp(7, 21) - 1) | 1,
                height: (panel_height.saturating_sub(4).clamp(5, 15) - 1) | 1,
            });
            let snapshot = self.game.presentation_snapshot().map_err(js_error)?;
            let snapshot = self.renderer.render(&snapshot);
            let mut terminal = Terminal::new(TestBackend::new(columns, rows)).map_err(js_error)?;
            terminal
                .draw(|frame| self.ui.draw(frame, &snapshot))
                .map_err(js_error)?;
            Ok(buffer_html(terminal.backend().buffer()))
        }

        pub fn save(&mut self) -> Result<(), JsValue> {
            self.game.save(&self.save_path).map_err(js_error)?;
            self.ui.notice =
                Some("Saved in this browser (separate from the graphical game)".into());
            Ok(())
        }
        /// The text toggle changes presentation only, reserving phone controls.
        pub fn render_text(
            &mut self,
            columns: u16,
            rows: u16,
            touchscreen: bool,
        ) -> Result<String, JsValue> {
            if !touchscreen {
                return self.render(columns, rows);
            }
            let columns = columns.clamp(40, 160);
            let mut html = self.render(columns, rows.saturating_sub(9))?;
            for _ in 0..9 {
                html.push_str(&" ".repeat(usize::from(columns)));
                html.push('\n');
            }
            Ok(html)
        }

        pub fn toggle_help(&mut self) {
            self.ui.show_help = !self.ui.show_help;
        }
        /// Compatibility entry point for the responsive phone composition.
        pub fn render_mobile(&mut self, columns: u16, rows: u16) -> Result<String, JsValue> {
            self.render_painted(columns, rows, true)
        }
        /// Shared painted view. Touch targets reserve space only on phones.
        pub fn render_painted(
            &mut self,
            columns: u16,
            rows: u16,
            touchscreen: bool,
        ) -> Result<String, JsValue> {
            let source = self.game.presentation_snapshot().map_err(js_error)?;
            self.renderer.set_viewport(ViewportSize {
                width: 9,
                height: 9,
            });
            let text = self.renderer.render(&source);
            let buffer = self.painted.draw(
                &source,
                &text,
                columns.clamp(40, 160),
                rows.clamp(26, 70),
                self.ui.notice.as_deref(),
                self.ui.show_help,
                touchscreen,
            );
            if let Some(viewport) = self.painted.viewport {
                self.renderer.set_viewport(viewport);
            }
            Ok(buffer_html(&buffer))
        }
        pub fn wait(&mut self, frames: u16) -> Result<(), JsValue> {
            self.game
                .wait_frames(usize::from(frames.min(600)))
                .map_err(js_error)?;
            self.checkpoint()?;
            Ok(())
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::BrowserTui;

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;
    #[test]
    fn session_slots_are_isolated_and_keep_legacy_default() {
        assert_eq!(session_save_path("core", "").unwrap().to_string_lossy(), "saves/core-tui-local.crystalsave");
        assert_ne!(session_save_path("core", "alice").unwrap(), session_save_path("core", "bob").unwrap());
        for id in ["../alice", "a/b", "a.b", "🎮", &"a".repeat(65)] {
            assert!(session_save_path("core", id).is_err());
        }
    }
    #[test]
    fn html_escapes_game_text_and_preserves_cell_colors() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 1));
        buffer[(0, 0)]
            .set_symbol("<")
            .set_fg(Color::Rgb(72, 202, 228));
        buffer[(1, 0)].set_symbol("&");
        let html = buffer_html(&buffer);
        assert!(html.contains("color:#48cae4"));
        assert!(html.contains("&lt;</span>"));
        assert!(html.contains("&amp;"));
        assert!(!html.contains("<script"));
    }
}
