//! Native text snapshots and terminal rendering for Geothite.
//!
//! The snapshot model deliberately has no terminal types. Agents, logs, tests,
//! accessibility adapters, and the interactive TUI can consume the same view.

mod menu_labels;
pub use menu_labels::visible_script_menu_option_label;
mod painted;
mod ink;
pub use painted::PaintedRenderer;
mod snapshot;
mod terminal;
mod theme;

pub use snapshot::{
    ACTION_LOG_LIMIT, LineKind, RuntimeTextRenderer, SnapshotLine, TextSnapshot, ViewportSize,
    render_snapshot_text, wrap_lines,
};
#[cfg(not(target_arch = "wasm32"))]
pub use terminal::map_key_event;
pub use terminal::{TerminalAction, TerminalUi};
pub use theme::{RgbColor, TuiTheme};

#[derive(Debug, Clone, Default)]
pub struct VisibleBattleReplayFrame {
    pub player_offset: [i16; 2],
    pub enemy_offset: [i16; 2],
    pub pieces: Vec<VisibleBattleReplayPiece>,
}
#[derive(Debug, Clone)]
pub struct VisibleBattleReplayPiece {
    pub x: i16, pub y: i16, pub width: u16, pub height: u16, pub rgba: Vec<u8>,
}
#[derive(Debug, Clone)]
pub struct VisibleBattleReplay { pub animation_label: String, pub frames: Vec<VisibleBattleReplayFrame> }

/// Escape the shared Ratatui cell buffer for the browser's presentation adapter.
pub fn buffer_html(buffer: &ratatui::buffer::Buffer) -> String {
    use ratatui::style::{Color, Modifier};
    use std::fmt::Write;
    let color = |color| match color { Color::Rgb(r,g,b) => format!("#{r:02x}{g:02x}{b:02x}"), _ => "inherit".to_string() };
    let mut html = String::new();
    for y in buffer.area.top()..buffer.area.bottom() {
        let mut style = None;
        for x in buffer.area.left()..buffer.area.right() {
            let cell = &buffer[(x,y)];
            let next = (cell.fg,cell.bg,cell.modifier);
            if style != Some(next) {
                if style.is_some() { html.push_str("</span>"); }
                let _ = write!(html, "<span style=\"color:{};background-color:{};font-weight:{}\">", color(cell.fg),color(cell.bg),if cell.modifier.contains(Modifier::BOLD) {"bold"}else{"normal"});
                style = Some(next);
            }
            for ch in cell.symbol().chars() {
                match ch { '&'=>html.push_str("&amp;"),'<'=>html.push_str("&lt;"),'>'=>html.push_str("&gt;"),'"'=>html.push_str("&quot;"),'\''=>html.push_str("&#39;"),_=>html.push(ch) }
            }
        }
        if style.is_some() { html.push_str("</span>"); }
        html.push('\n');
    }
    html
}
