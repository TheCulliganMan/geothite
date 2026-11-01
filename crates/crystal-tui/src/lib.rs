//! Native and standalone browser hosts share the production text renderer.
pub use crystal_tui_view::*;
#[cfg(any(target_arch = "wasm32", test))]
mod browser;
mod codemode;
#[cfg(not(target_arch = "wasm32"))]
mod terminal_canvas;
#[cfg(target_arch = "wasm32")]
pub use browser::BrowserTui;
pub use codemode::{call_code_mode_tool, code_mode_catalog, execute_code};
#[cfg(not(target_arch = "wasm32"))]
pub use terminal_canvas::{TerminalCanvas, terminal_canvas_size};
