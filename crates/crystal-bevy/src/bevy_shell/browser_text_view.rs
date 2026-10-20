// A read-only renderer of the live visible controller, never another game.
#[cfg(target_arch = "wasm32")]
struct BrowserTextView {
    enabled: bool,
    columns: u16,
    rows: u16,
    painted: bool,
    renderer: crystal_tui_view::RuntimeTextRenderer,
    painter: crystal_tui_view::PaintedRenderer,
    ui: crystal_tui_view::TerminalUi,
    frame: u64,
    serialized: String,
}
#[cfg(target_arch = "wasm32")]
impl Default for BrowserTextView {
    fn default() -> Self {
        Self {
            enabled: false,
            columns: 80,
            rows: 40,
            painted: true,
            renderer: Default::default(),
            painter: Default::default(),
            ui: crystal_tui_view::TerminalUi::new(Default::default()),
            frame: u64::MAX,
            serialized: String::new(),
        }
    }
}
#[cfg(target_arch = "wasm32")]
thread_local! {
    static BROWSER_TEXT_VIEW: std::cell::RefCell<BrowserTextView> = std::cell::RefCell::new(BrowserTextView::default());
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_configure_text_view(enabled: bool, columns: u16, rows: u16, painted: bool) {
    BROWSER_TEXT_VIEW.with_borrow_mut(|view| {
        let columns = columns.clamp(40, 128);
        let rows = rows.clamp(26, 70);
        if (view.enabled, view.columns, view.rows, view.painted)
            != (enabled, columns, rows, painted)
        {
            view.frame = u64::MAX;
        }
        view.enabled = enabled;
        view.columns = columns;
        view.rows = rows;
        view.painted = painted;
    });
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_text_view_frame() -> String {
    BROWSER_TEXT_VIEW.with_borrow(|view| view.serialized.clone())
}
#[cfg(target_arch = "wasm32")]
fn publish_browser_text_view(mut runtime: ResMut<BevyRuntimeShell>) {
    use crystal_tui_view::{LineKind, SnapshotLine, buffer_html};
    BROWSER_TEXT_VIEW.with_borrow_mut(|view| {
        if !view.enabled { return; }
        // Ten presentation samples per second. Sampling never advances input
        // or settles scripts; the normal game loop continues in every view.
        if view.frame != u64::MAX && runtime.lcd_animation_frame.saturating_sub(view.frame) < 6 { return; }
        let result = (|| -> Result<String> {
            let snapshot = visible_controller_snapshot(&mut runtime)?;
            view.renderer.set_viewport(crystal_tui_view::ViewportSize {width: 9, height: 9});
            let menu_index = snapshot.ui.menu.as_ref().and_then(|menu| {
                menu.layout.vertical_menus.first().and_then(|vertical| {
                    strict_readonly_cursor_index(&runtime.menu_cursor,
                        &vertical_menu_surface_id(menu,vertical),vertical.options.len())
                })
            }).unwrap_or(0);
            let yes_no_index = strict_readonly_cursor_index(&runtime.yes_no_cursor,"ui:yes-no",2).unwrap_or(0);
            view.renderer.set_selection(menu_index,yes_no_index);
            let mut text = view.renderer.render(&snapshot);
            text.input_hints = vec![SnapshotLine { text: "Arrows/WASD move · Z confirm · X back · Start/Select: game controls".into(), kind: LineKind::Hint }];
            let opening = if runtime.intro_screen.is_some() { Some(format_intro_dialog(&runtime)) }
                else if runtime.title_menu.is_some() { Some(format_title_dialog(&runtime)) }
                else if runtime.pending_gender_selection.is_some() { Some(format_gender_dialog(&runtime)) }
                else if runtime.pending_time_set.is_some() { Some(format_time_set_dialog_overlay(&runtime)) }
                else if runtime.pending_oak_intro.is_some() { Some(format_oak_intro_dialog_overlay(&runtime)) }
                else { None };
            let columns = view.columns; let rows = view.rows;
            let use_paint = view.painted && opening.is_none();
            let mut detail = None;
            let html = if use_paint {
                view.painter.advance_ink_by(0.1);
                let buffer = view.painter.draw(&snapshot, &text, columns, rows, None, false, false);
                let html = buffer_html(&buffer);
                if let Some(buffer) = view.painter.detailed_world(&snapshot) {
                    detail = Some(serde_json::json!({"html":buffer_html(&buffer), "bounds":view.painter.scene_bounds(), "sizes":view.painter.animated_dot_sizes()}));
                }
                html
            } else {
                if let Some(opening) = opening {
                    text.viewport_title = "Geothite".into();
                    text.viewport = opening.lines().filter(|line| !line.starts_with("SPRITE:")).map(|line| SnapshotLine {text:line.into(),kind:LineKind::Normal}).collect();
                    text.menu.clear(); text.dialogue.clear(); text.prompt.clear();
                }
                let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(columns,rows))?;
                terminal.draw(|frame| view.ui.draw(frame,&text))?;
                buffer_html(terminal.backend().buffer())
            };
            Ok(serde_json::json!({"frame":runtime.lcd_animation_frame, "html":html,"detail":detail,"observation":text}).to_string())
        })();
        view.serialized = match result { Ok(value) => value, Err(error) => serde_json::json!({"error":error.to_string()}).to_string() };
        view.frame = runtime.lcd_animation_frame;
    });
}
