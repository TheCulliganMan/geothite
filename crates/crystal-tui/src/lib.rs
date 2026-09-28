//! Native text snapshots and terminal rendering for Geothite.
//!
//! The snapshot model deliberately has no terminal types. Agents, logs, tests,
//! accessibility adapters, and the interactive TUI can consume the same view.

mod snapshot;
mod terminal;
mod theme;

pub use snapshot::{
    ACTION_LOG_LIMIT, LineKind, RuntimeTextRenderer, SnapshotLine, TextSnapshot, ViewportSize,
    render_snapshot_text, wrap_lines,
};
pub use terminal::{TerminalAction, TerminalUi, map_key_event};
pub use theme::{RgbColor, TuiTheme};
