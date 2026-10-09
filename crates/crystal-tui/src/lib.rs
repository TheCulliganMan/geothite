//! Native text snapshots and terminal rendering for Geothite.
//!
//! The snapshot model deliberately has no terminal types. Agents, logs, tests,
//! accessibility adapters, and the interactive TUI can consume the same view.

#[cfg(any(target_arch = "wasm32", test))]
mod browser;
mod codemode;
mod ink;
mod painted;
mod snapshot;
mod terminal;
#[cfg(not(target_arch = "wasm32"))]
mod terminal_canvas;
mod theme;
#[cfg(not(target_arch = "wasm32"))]
pub use terminal_canvas::{TerminalCanvas, terminal_canvas_size};

#[cfg(target_arch = "wasm32")]
pub use browser::BrowserTui;
pub use codemode::{call_code_mode_tool, code_mode_catalog, execute_code};
pub use painted::PaintedRenderer;
pub use snapshot::{
    ACTION_LOG_LIMIT, LineKind, RuntimeTextRenderer, SnapshotLine, TextSnapshot, ViewportSize,
    render_snapshot_text, wrap_lines,
};
#[cfg(not(target_arch = "wasm32"))]
pub use terminal::map_key_event;
pub use terminal::{TerminalAction, TerminalUi};
pub use theme::{RgbColor, TuiTheme};
