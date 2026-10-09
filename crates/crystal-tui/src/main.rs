use anyhow::{Context, Result, bail};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController as RuntimeGameShell;
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeCompiledScriptCursor};
use geothite::{
    PaintedRenderer, RuntimeTextRenderer, TerminalAction, TerminalCanvas, TerminalUi, TuiTheme,
    map_key_event, render_snapshot_text,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use serde_json::{Value, json};
use std::{
    env,
    io::{self, BufRead, BufReader, IsTerminal, Stdout, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Debug)]
struct Options {
    command: Command,
    pack: PathBuf,
    load: Option<PathBuf>,
    save: Option<PathBuf>,
    theme: Option<PathBuf>,
    compact: bool,
    player_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Play,
    Mcp,
    Dump,
}

fn main() -> Result<()> {
    let options = parse_options(env::args().skip(1))?;
    let pack = options
        .pack
        .canonicalize()
        .with_context(|| format!("resolve game content pack {}", options.pack.display()))?;
    let asset_root = AssetRoot::new(pack.parent().context("content pack path has no parent")?);
    let loaded = read_loaded_verified_compiled_game_pack(&pack)
        .with_context(|| format!("load game content pack {}", pack.display()))?;
    let painter = if options.command == Command::Play {
        PaintedRenderer::from_pack_assets(loaded.pack().runtime_files())
    } else {
        PaintedRenderer::default()
    };
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&asset_root, loaded)?;
    let mut game = match &options.load {
        Some(load) => {
            RuntimeGameShell::load_save(asset_root, runtime, load.clone(), options.save.clone())?
        }
        None => RuntimeGameShell::new_game(
            asset_root,
            runtime,
            &options.player_name,
            options.save.clone(),
        )?,
    };
    game.set_runtime_journal_enabled(false);
    let theme = match &options.theme {
        Some(path) => TuiTheme::from_path(path)?,
        None => TuiTheme::default(),
    };
    if options.command == Command::Dump {
        let mut renderer = RuntimeTextRenderer::default();
        let mut active_cursor = None;
        finish_noninteractive_work(&mut game, &mut active_cursor, &mut renderer)?;
        let runtime_snapshot = game.snapshot()?;
        let snapshot = renderer.render(&runtime_snapshot);
        println!("{}", render_snapshot_text(&snapshot, options.compact));
        return Ok(());
    }
    if options.command == Command::Mcp {
        return run_mcp(game, options.save);
    }
    anyhow::ensure!(
        io::stdin().is_terminal() && io::stdout().is_terminal(),
        "geothite play requires an interactive terminal; use `geothite dump PACK` for plain text"
    );
    run_tui(game, theme, options.save, painter)
}

fn parse_options(arguments: impl IntoIterator<Item = String>) -> Result<Options> {
    let usage = "usage: geothite <play|mcp|dump> <game.crystalpack> [--load SAVE] [--save SAVE] [--name NAME] [--theme THEME] [--compact]";
    let mut pack = None;
    let mut load = None;
    let mut save = None;
    let mut compact = false;
    let mut player_name = "CHRIS".to_string();
    let mut theme = None;
    let mut arguments = arguments.into_iter();
    let command = match arguments.next().as_deref() {
        Some("play") => Command::Play,
        Some("mcp") => Command::Mcp,
        Some("dump") => Command::Dump,
        Some("-h" | "--help") | None => {
            println!("{usage}");
            println!(
                "\nCommands:\n  play  Interactive terminal game\n  mcp   stdio MCP server\n  dump  Plain-text snapshot"
            );
            std::process::exit(0);
        }
        Some(command) => bail!("unknown command {command}\n{usage}"),
    };
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--load" => {
                load = Some(PathBuf::from(
                    arguments.next().context("--load requires a path")?,
                ))
            }
            "--save" => {
                save = Some(PathBuf::from(
                    arguments.next().context("--save requires a path")?,
                ))
            }
            "--name" => {
                player_name = arguments.next().context("--name requires a trainer name")?;
                anyhow::ensure!(
                    !player_name.trim().is_empty() && player_name.chars().count() <= 7,
                    "--name must contain 1 to 7 characters"
                );
            }
            "--theme" => {
                theme = Some(PathBuf::from(
                    arguments.next().context("--theme requires a path")?,
                ))
            }
            "--compact" => compact = true,
            "-h" | "--help" => {
                println!("{usage}");
                std::process::exit(0);
            }
            value if value.starts_with('-') => bail!("unknown option {value}"),
            value if pack.is_none() => pack = Some(PathBuf::from(value)),
            value => bail!("unexpected argument {value}"),
        }
    }
    Ok(Options {
        command,
        pack: pack.context(usage)?,
        load,
        save,
        theme,
        compact,
        player_name,
    })
}

fn run_mcp(mut game: RuntimeGameShell, save_path: Option<PathBuf>) -> Result<()> {
    let mut renderer = RuntimeTextRenderer::default();
    let mut active_cursor = None;
    finish_noninteractive_work(&mut game, &mut active_cursor, &mut renderer)?;
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in BufReader::new(stdin.lock()).lines() {
        let line = line.context("read MCP stdio request")?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = serde_json::from_str(&line).context("parse MCP JSON-RPC request")?;
        let Some(id) = request.get("id").cloned() else {
            continue;
        };
        let response = match request.get("method").and_then(Value::as_str) {
            Some("initialize") => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": request.pointer("/params/protocolVersion")
                        .and_then(Value::as_str)
                        .unwrap_or("2025-06-18"),
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "geothite", "version": env!("CARGO_PKG_VERSION") }
                }
            }),
            Some("ping") => json!({ "jsonrpc": "2.0", "id": id, "result": {} }),
            Some("tools/list") => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "tools": mcp_tools() }
            }),
            Some("tools/call") => match call_mcp_tool(
                &mut game,
                &mut renderer,
                &mut active_cursor,
                request.pointer("/params/name").and_then(Value::as_str),
                request.pointer("/params/arguments").unwrap_or(&Value::Null),
                save_path.as_ref(),
            ) {
                Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
                Err(error) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": error.to_string() }],
                        "isError": true
                    }
                }),
            },
            Some(method) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("method not found: {method}") }
            }),
            None => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32600, "message": "missing method" }
            }),
        };
        serde_json::to_writer(&mut stdout, &response).context("write MCP response")?;
        stdout.write_all(b"\n").context("finish MCP response")?;
        stdout.flush().context("flush MCP response")?;
    }
    Ok(())
}

fn mcp_tools() -> Value {
    json!([
        {
            "name": "observe",
            "description": "Verbose text snapshot of the current game screen, dialogue, menus, and visible surroundings.",
            "inputSchema": { "type": "object", "properties": { "detail": { "enum": ["full", "compact"] } } }
        },
        {
            "name": "status",
            "description": "Compact structured session state for routine checks.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "move",
            "description": "Move an exact number of overworld tiles, or send directional input to the active menu.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "direction": { "enum": ["up", "down", "left", "right"] },
                    "steps": { "type": "integer", "minimum": 1, "maximum": 25 }
                },
                "required": ["direction"]
            }
        },
        {
            "name": "press",
            "description": "Press and release a Game Boy button.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "button": { "enum": ["a", "b", "start", "select", "up", "down", "left", "right"] },
                    "times": { "type": "integer", "minimum": 1, "maximum": 25 }
                },
                "required": ["button"]
            }
        },
        {
            "name": "hold_button",
            "description": "Hold a Game Boy button for a bounded number of frames.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "button": { "enum": ["a", "b", "start", "select", "up", "down", "left", "right"] },
                    "frames": { "type": "integer", "minimum": 1, "maximum": 120 }
                },
                "required": ["button", "frames"]
            }
        },
        {
            "name": "execute_macro",
            "description": "Execute a bounded built-in macro or explicit short action list.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "macro": { "enum": ["advance_dialog", "mash_a", "interact"] },
                    "max_presses": { "type": "integer", "minimum": 1, "maximum": 120 },
                    "actions": {
                        "type": "array",
                        "maxItems": 120,
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": { "enum": ["move", "button"] },
                                "value": { "type": "string" },
                                "times": { "type": "integer", "minimum": 1, "maximum": 120 }
                            },
                            "required": ["type", "value"]
                        }
                    }
                }
            }
        },
        {
            "name": "recent_events",
            "description": "Return the recent player action log and current compact state.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "journal",
            "description": "Alias for recent_events.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "map_info",
            "description": "Return the current map, facing direction, visible objects, and authored warp tiles.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "flow_state",
            "description": "Return the current presentation phase and available original Game Boy inputs.",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

fn call_mcp_tool(
    game: &mut RuntimeGameShell,
    renderer: &mut RuntimeTextRenderer,
    active_cursor: &mut Option<RuntimeCompiledScriptCursor>,
    name: Option<&str>,
    arguments: &Value,
    save_path: Option<&PathBuf>,
) -> Result<Value> {
    match name.context("MCP tools/call is missing params.name")? {
        "observe" => {
            let source = game.presentation_snapshot()?;
            let snapshot = renderer.render(&source);
            let compact = arguments.get("detail").and_then(Value::as_str) == Some("compact");
            let text = render_snapshot_text(&snapshot, compact);
            Ok(json!({
                "content": [{ "type": "text", "text": text }],
                "structuredContent": snapshot,
                "isError": false
            }))
        }
        "status" => mcp_status_result(game, renderer),
        "move" => {
            let direction = arguments
                .get("direction")
                .and_then(Value::as_str)
                .context("move requires direction")?;
            let button = parse_game_button(direction)?;
            let steps = bounded_argument(arguments, "steps", 1, 25, 1)?;
            move_exact_tiles(game, renderer, active_cursor, button, steps)?;
            save_after_mcp_action(game, save_path)?;
            mcp_status_result(game, renderer)
        }
        "press" => {
            let button = parse_game_button(
                arguments
                    .get("button")
                    .and_then(Value::as_str)
                    .context("press requires button")?,
            )?;
            let times = bounded_argument(arguments, "times", 1, 25, 1)?;
            for _ in 0..times {
                apply_button(game, renderer, active_cursor, button)?;
                finish_noninteractive_work(game, active_cursor, renderer)?;
            }
            save_after_mcp_action(game, save_path)?;
            mcp_status_result(game, renderer)
        }
        "hold_button" => {
            let button = parse_game_button(
                arguments
                    .get("button")
                    .and_then(Value::as_str)
                    .context("hold_button requires button")?,
            )?;
            let frames = bounded_argument(arguments, "frames", 1, 120, 1)?;
            for _ in 0..frames {
                apply_button(game, renderer, active_cursor, button)?;
                finish_noninteractive_work(game, active_cursor, renderer)?;
            }
            save_after_mcp_action(game, save_path)?;
            mcp_status_result(game, renderer)
        }
        "execute_macro" => {
            if let Some(actions) = arguments.get("actions").and_then(Value::as_array) {
                anyhow::ensure!(
                    actions.len() <= 120,
                    "actions must contain at most 120 entries"
                );
                for action in actions {
                    let kind = action
                        .get("type")
                        .and_then(Value::as_str)
                        .context("macro action requires type")?;
                    let value = action
                        .get("value")
                        .and_then(Value::as_str)
                        .context("macro action requires value")?;
                    let times = bounded_argument(action, "times", 1, 120, 1)?;
                    let button = parse_game_button(value)?;
                    if kind == "move" {
                        move_exact_tiles(game, renderer, active_cursor, button, times)?;
                    } else if kind == "button" {
                        for _ in 0..times {
                            apply_button(game, renderer, active_cursor, button)?;
                            finish_noninteractive_work(game, active_cursor, renderer)?;
                        }
                    } else {
                        bail!("macro action type must be move or button");
                    }
                }
            } else {
                let macro_name = arguments
                    .get("macro")
                    .and_then(Value::as_str)
                    .context("execute_macro requires macro or actions")?;
                let presses = bounded_argument(
                    arguments,
                    "max_presses",
                    1,
                    120,
                    if macro_name == "interact" { 1 } else { 16 },
                )?;
                anyhow::ensure!(
                    matches!(macro_name, "advance_dialog" | "mash_a" | "interact"),
                    "unknown macro {macro_name}"
                );
                for _ in 0..presses {
                    apply_button(game, renderer, active_cursor, GameButton::A)?;
                    finish_noninteractive_work(game, active_cursor, renderer)?;
                    if macro_name == "advance_dialog" {
                        let snapshot = game.presentation_snapshot()?;
                        if snapshot.ui.pending_text_wait.is_none()
                            && snapshot.ui.pending_yes_no.is_none()
                        {
                            break;
                        }
                    }
                }
            }
            save_after_mcp_action(game, save_path)?;
            mcp_status_result(game, renderer)
        }
        "recent_events" | "journal" => {
            let source = game.presentation_snapshot()?;
            let snapshot = renderer.render(&source);
            let payload = json!({
                "actions": snapshot.action_log,
                "status": mcp_status_payload(&source)
            });
            Ok(mcp_json_result(payload))
        }
        "map_info" => {
            let source = game.presentation_snapshot()?;
            let map = source
                .maps
                .iter()
                .find(|map| map.map_name == source.overworld.map_name);
            let player = source.overworld.tile;
            let visible_objects = source
                .visible_object_runtime_tiles
                .iter()
                .map(|(id, tile)| {
                    json!({
                        "id": id,
                        "offset_x": tile.x - player.x,
                        "offset_y": tile.y - player.y
                    })
                })
                .collect::<Vec<_>>();
            let warps = map
                .into_iter()
                .flat_map(|map| map.events.warps.iter())
                .filter_map(|warp| {
                    event_runtime_position(warp.x, warp.y).map(|(x, y)| {
                        json!({
                            "x": x,
                            "y": y,
                            "target_map": warp.target_map
                        })
                    })
                })
                .collect::<Vec<_>>();
            Ok(mcp_json_result(json!({
                "name": source.overworld.map_name,
                "facing": format!("{:?}", source.overworld.facing).to_lowercase(),
                "visible_objects": visible_objects,
                "warps": warps
            })))
        }
        "flow_state" => {
            let source = game.presentation_snapshot()?;
            Ok(mcp_json_result(json!({
                "phase": format!("{:?}", source.phase).to_lowercase(),
                "animating": game.has_pending_script_work(),
                "buttons": ["up", "down", "left", "right", "a", "b", "start", "select"]
            })))
        }
        other => bail!("unknown tool {other}"),
    }
}

fn event_runtime_position(x: u16, y: u16) -> Option<(i16, i16)> {
    crystal_core::world::session::raw_event_tile_to_runtime_tile_checked(x, y)
        .map(|tile| (tile.x, tile.y))
}

fn mcp_status_result(
    game: &mut RuntimeGameShell,
    renderer: &mut RuntimeTextRenderer,
) -> Result<Value> {
    let source = game.presentation_snapshot()?;
    let payload = mcp_status_payload(&source);
    renderer.render(&source);
    Ok(mcp_json_result(payload))
}

fn mcp_status_payload(source: &crystal_runtime::RuntimeShellSnapshot) -> Value {
    let badges = source
        .progression
        .badges
        .johto
        .iter()
        .filter(|badge| **badge)
        .count()
        + source
            .progression
            .badges
            .kanto
            .iter()
            .filter(|badge| **badge)
            .count();
    json!({
        "mode": format!("{:?}", source.phase).to_lowercase(),
        "map": source.overworld.map_name,
        "position": { "x": source.overworld.tile.x, "y": source.overworld.tile.y },
        "facing": format!("{:?}", source.overworld.facing).to_lowercase(),
        "frame": source.overworld.frame,
        "player": source.trainer.player_name,
        "money": source.trainer.money,
        "badges": badges,
        "pokedex": { "seen": source.progression.pokedex_seen, "owned": source.progression.pokedex_owned },
        "menu": source.ui.menu.as_ref().map(|menu| menu.menu_id.as_str()),
        "dialogue": source.ui.text_window_open || source.ui.pending_text_wait.is_some(),
    })
}

fn mcp_json_result(payload: Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "structuredContent": payload,
        "isError": false
    })
}

fn bounded_argument(
    arguments: &Value,
    key: &str,
    minimum: u64,
    maximum: u64,
    default: u64,
) -> Result<usize> {
    let value = arguments
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or(default);
    anyhow::ensure!(
        (minimum..=maximum).contains(&value),
        "{key} must be between {minimum} and {maximum}"
    );
    Ok(value as usize)
}

fn parse_game_button(value: &str) -> Result<GameButton> {
    match value.to_ascii_lowercase().as_str() {
        "a" => Ok(GameButton::A),
        "b" => Ok(GameButton::B),
        "start" => Ok(GameButton::Start),
        "select" => Ok(GameButton::Select),
        "up" => Ok(GameButton::Up),
        "down" => Ok(GameButton::Down),
        "left" => Ok(GameButton::Left),
        "right" => Ok(GameButton::Right),
        _ => bail!("unknown Game Boy button {value}"),
    }
}

fn move_exact_tiles(
    game: &mut RuntimeGameShell,
    renderer: &mut RuntimeTextRenderer,
    active_cursor: &mut Option<RuntimeCompiledScriptCursor>,
    button: GameButton,
    steps: usize,
) -> Result<()> {
    for _ in 0..steps {
        let start = game.presentation_snapshot()?.overworld.tile;
        let mut moved = false;
        for _ in 0..4 {
            apply_button(game, renderer, active_cursor, button)?;
            finish_noninteractive_work(game, active_cursor, renderer)?;
            if game.presentation_snapshot()?.overworld.tile != start {
                moved = true;
                break;
            }
        }
        if !moved {
            renderer.record_action(format!("blocked: {:?}", button));
            break;
        }
    }
    Ok(())
}

fn save_after_mcp_action(game: &mut RuntimeGameShell, save_path: Option<&PathBuf>) -> Result<()> {
    if let Some(path) = save_path {
        game.save(path)?;
    }
    Ok(())
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    canvas: TerminalCanvas,
}

/// Cosmetic deadlines are independent of input arrival. Never tick the game
/// here, and never catch up a paused view with a burst of invisible frames.
struct VisualClock {
    next: Instant,
    period: Duration,
}

impl VisualClock {
    fn new(now: Instant, period: Duration) -> Self {
        Self {
            next: now + period,
            period,
        }
    }

    fn tick(&mut self, now: Instant, enabled: bool) -> bool {
        if !enabled {
            self.next = now + self.period;
            return false;
        }
        if now < self.next {
            return false;
        }
        self.next = now + self.period;
        true
    }

    fn wait(&self, now: Instant) -> Duration {
        self.next.saturating_duration_since(now)
    }
}

fn terminal_poll_timeout(remaining: Duration) -> Duration {
    // The use-dev-tty poller skips even buffered keys when its timeout is
    // zero. Large PNG frames can overrun a visual deadline: always give the
    // input source time to check readiness instead of starving controls.
    remaining.max(Duration::from_millis(1))
}

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode().context("enable terminal raw mode")?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(error).context("enter alternate terminal screen");
        }
        match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => Ok(Self {
                terminal,
                canvas: TerminalCanvas::detect(),
            }),
            Err(error) => {
                let mut stdout = io::stdout();
                let _ = execute!(stdout, LeaveAlternateScreen);
                let _ = disable_raw_mode();
                Err(error).context("create terminal")
            }
        }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.canvas.clear(self.terminal.backend_mut());
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn run_tui(
    mut game: RuntimeGameShell,
    theme: TuiTheme,
    save_path: Option<PathBuf>,
    mut painter: PaintedRenderer,
) -> Result<()> {
    let mut terminal = TerminalGuard::enter()?;
    let mut renderer = RuntimeTextRenderer::default();
    let mut ui = TerminalUi::new(theme);
    let mut active_cursor = None;
    let mut command_buffer: Option<String> = None;
    let mut painted = true;
    let reduced_motion = std::env::var("GEOTHITE_REDUCED_MOTION").as_deref() == Ok("1");
    let mut ink_clock = VisualClock::new(Instant::now(), Duration::from_secs_f64(1. / 30.));
    let mut last_ink = Instant::now();
    let mut replay_clock = VisualClock::new(Instant::now(), Duration::from_millis(50));
    game.set_battle_replay_enabled(!reduced_motion);
    finish_noninteractive_work(&mut game, &mut active_cursor, &mut renderer)?;
    let mut runtime_snapshot = game.presentation_snapshot()?;
    let mut snapshot = renderer.render(&runtime_snapshot);
    let mut state_dirty = false;
    let mut surface_dirty = true;
    let mut surface = None;
    let mut painted_this_frame = false;
    'tui: loop {
        let now = Instant::now();
        let ink_changed = ink_clock.tick(now, painted && !reduced_motion);
        if ink_changed {
            painter.advance_ink_by(now.duration_since(last_ink).as_secs_f64());
            last_ink = now;
        } else if !painted || reduced_motion {
            last_ink = now;
        }
        if replay_clock.tick(now, painted && !reduced_motion && painter.replay_active()) {
            painter.advance_replay();
            surface_dirty = true;
        }
        if state_dirty {
            runtime_snapshot = game.presentation_snapshot()?;
            snapshot = renderer.render(&runtime_snapshot);
            state_dirty = false;
            surface_dirty = true;
        }
        let redraw =
            surface_dirty || (ink_changed && painted_this_frame && !terminal.canvas.enabled());
        if redraw {
            terminal
                .terminal
                .draw(|frame| {
                    if painted && frame.area().width >= 40 && frame.area().height >= 24 {
                        painted_this_frame = true;
                        let area = frame.area();
                        if surface_dirty
                            || surface
                                .as_ref()
                                .is_none_or(|b: &ratatui::buffer::Buffer| b.area != area)
                        {
                            surface = Some(painter.draw_native_surface(
                                &runtime_snapshot,
                                &snapshot,
                                area.width,
                                area.height,
                                ui.notice.as_deref(),
                                ui.show_help,
                                terminal.canvas.enabled(),
                            ));
                        } else if let Some(buffer) = surface.as_mut() {
                            painter.refresh_native_dots(buffer);
                        }
                        frame
                            .buffer_mut()
                            .merge(surface.as_ref().expect("painted surface"));
                    } else {
                        painted_this_frame = false;
                        ui.draw(frame, &snapshot);
                    }
                })
                .context("draw terminal UI")?;
        }
        if painted_this_frame && terminal.canvas.enabled() && (surface_dirty || ink_changed) {
            let bounds = painter.scene_bounds();
            if let [x, y, width, height] = bounds.as_slice() {
                let size = crossterm::terminal::window_size().ok();
                let cw = size
                    .as_ref()
                    .filter(|s| s.width >= s.columns && s.columns != 0)
                    .map_or(12, |s| u32::from(s.width) / u32::from(s.columns));
                let ch = size
                    .as_ref()
                    .filter(|s| s.height >= s.rows && s.rows != 0)
                    .map_or(24, |s| u32::from(s.height) / u32::from(s.rows));
                let (pixels_w, pixels_h) =
                    geothite::terminal_canvas_size(u32::from(*width) * cw, u32::from(*height) * ch);
                if let Some(image) = painter.circle_image(pixels_w, pixels_h) {
                    terminal
                        .canvas
                        .paint(
                            terminal.terminal.backend_mut(),
                            ratatui::layout::Rect::new(*x, *y, *width, *height),
                            &image,
                        )
                        .context("paint terminal dot canvas")?;
                } else {
                    terminal.canvas.clear(terminal.terminal.backend_mut())?;
                }
            } else {
                terminal.canvas.clear(terminal.terminal.backend_mut())?;
            }
        } else if !painted_this_frame {
            terminal.canvas.clear(terminal.terminal.backend_mut())?;
        }
        surface_dirty = false;
        if painted {
            if let Some(viewport) = painter.viewport {
                renderer.set_viewport(viewport);
            }
        } else {
            renderer.set_viewport(geothite::ViewportSize::default());
        }
        let now = Instant::now();
        let timeout = if painter.replay_active() {
            ink_clock.wait(now).min(replay_clock.wait(now))
        } else {
            ink_clock.wait(now)
        };
        if !event::poll(terminal_poll_timeout(timeout)).context("poll terminal input")? {
            continue;
        }
        let input = event::read().context("read terminal input")?;
        if matches!(input, Event::Resize(..)) {
            surface_dirty = true;
        }
        let Event::Key(key) = input else {
            continue;
        };
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            continue;
        }
        if painter.replay_active() {
            surface_dirty = true;
        }
        painter.cancel_replay();
        if let Some(buffer) = command_buffer.as_mut() {
            surface_dirty = true;
            match key.code {
                KeyCode::Esc => {
                    command_buffer = None;
                    ui.notice = None;
                }
                KeyCode::Backspace => {
                    buffer.pop();
                    ui.notice = Some(format!("COMMAND> :{buffer}"));
                }
                KeyCode::Char(character) => {
                    buffer.push(character);
                    ui.notice = Some(format!("COMMAND> :{buffer}"));
                }
                KeyCode::Enter => {
                    let command = std::mem::take(buffer);
                    command_buffer = None;
                    match command.trim() {
                        "q!" => break 'tui,
                        "wq" | "wq!" | "x" | "x!" => {
                            if let Some(path) = save_path.as_ref() {
                                game.save(path)?;
                            }
                            break 'tui;
                        }
                        "q" => {
                            ui.notice = Some(
                                "Use :wq to save+quit or :q! to quit without saving.".to_string(),
                            )
                        }
                        "c" => ui.show_help = !ui.show_help,
                        "v" => ui.compact = !ui.compact,
                        other => ui.notice = Some(format!("Unknown command: {other}")),
                    }
                }
                _ => {}
            }
            continue;
        }
        if matches!(key.code, KeyCode::Char('v' | 'V')) {
            painted = !painted;
            surface_dirty = true;
            continue;
        }
        let modal_selection = snapshot.confirmation_input_owned();
        let action = map_key_event(key, modal_selection);
        if !matches!(action, TerminalAction::None) {
            state_dirty = true;
        }
        match action {
            TerminalAction::Quit => break,
            TerminalAction::BeginCommand => {
                command_buffer = Some(String::new());
                ui.notice = Some("COMMAND> :".to_string());
            }
            TerminalAction::ToggleHelp => ui.show_help = !ui.show_help,
            TerminalAction::Refresh => {}
            TerminalAction::Wait => {
                game.wait_frames(8)?;
                renderer.record_action("wait: 8 frames");
            }
            TerminalAction::Save => match save_path.as_ref() {
                Some(path) => {
                    game.save(path)?;
                    ui.notice = Some(format!("Saved {}", path.display()));
                    renderer.record_action(format!("save: {}", path.display()));
                }
                None => ui.notice = Some("Start with --save PATH to enable F5 saves".to_string()),
            },
            TerminalAction::MoveSelection(delta) => {
                if !snapshot.prompt.is_empty() {
                    renderer.move_selection(delta, 2, true);
                } else {
                    renderer.move_selection(delta, snapshot.menu.len(), false);
                }
                game.press(if delta < 0 {
                    GameButton::Up
                } else {
                    GameButton::Down
                })?;
            }
            TerminalAction::GameButton(button) => {
                ui.notice = None;
                apply_button(&mut game, &mut renderer, &mut active_cursor, button)?;
                finish_noninteractive_work(&mut game, &mut active_cursor, &mut renderer)?;
            }
            TerminalAction::None => {}
        }
        let replays = game.take_battle_replays();
        if painted && !reduced_motion {
            painter.replay(replays);
        }
    }
    Ok(())
}

fn finish_noninteractive_work(
    _game: &mut RuntimeGameShell,
    _active_cursor: &mut Option<RuntimeCompiledScriptCursor>,
    _renderer: &mut RuntimeTextRenderer,
) -> Result<()> {
    Ok(())
}

fn apply_button(
    game: &mut RuntimeGameShell,
    renderer: &mut RuntimeTextRenderer,
    _active_cursor: &mut Option<RuntimeCompiledScriptCursor>,
    button: GameButton,
) -> Result<()> {
    if matches!(button, GameButton::Up | GameButton::Down) {
        let snapshot = game.presentation_snapshot()?;
        let delta = if button == GameButton::Up { -1 } else { 1 };
        if snapshot.ui.pending_yes_no.is_some() {
            renderer.move_selection(delta, 2, true);
        } else if let Some(menu) = snapshot.ui.menu.as_ref()
            && let Some(vertical) = menu.layout.vertical_menus.first()
            && !vertical
                .options
                .iter()
                .any(|option| option.trim_start().starts_with('>'))
        {
            renderer.move_selection(delta, vertical.options.len(), false);
        }
    }
    game.press(button)?;
    renderer.record_action(format!("input: {:?}", button));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overrun_visual_frames_still_poll_real_input() {
        let start = Instant::now();
        let clock = VisualClock::new(start, Duration::from_secs_f64(1. / 30.));
        let expired = clock.wait(start + Duration::from_millis(80));
        assert!(
            expired.is_zero(),
            "A large frame missed its visual deadline"
        );
        assert_eq!(terminal_poll_timeout(expired), Duration::from_millis(1));
        assert_eq!(
            terminal_poll_timeout(Duration::from_millis(20)),
            Duration::from_millis(20)
        );
    }

    #[test]
    fn cosmetic_deadlines_survive_continuous_input() {
        let start = Instant::now();
        let mut clock = VisualClock::new(start, Duration::from_millis(100));
        let mut frames = 0;
        // Input every 10ms: the old poll-timeout branch never animated here.
        for millis in (10..=1000).step_by(10) {
            frames += usize::from(clock.tick(start + Duration::from_millis(millis), true));
        }
        assert_eq!(frames, 10);
        assert_eq!(
            clock.wait(start + Duration::from_millis(1050)),
            Duration::from_millis(50)
        );
    }

    #[test]
    fn paused_cosmetic_clocks_do_not_replay_hidden_frames() {
        let start = Instant::now();
        let mut clock = VisualClock::new(start, Duration::from_millis(100));
        assert!(!clock.tick(start + Duration::from_secs(10), false));
        assert!(!clock.tick(start + Duration::from_millis(10050), true));
        assert!(clock.tick(start + Duration::from_millis(10100), true));
        assert!(clock.tick(start + Duration::from_secs(30), true));
        assert!(!clock.tick(start + Duration::from_secs(30), true));
    }

    #[test]
    fn default_theme_needs_no_build_checkout() {
        let options = parse_options(["play".into(), "game.crystalpack".into()]).unwrap();
        assert!(options.theme.is_none());
        let custom = parse_options([
            "play".into(),
            "game.crystalpack".into(),
            "--theme".into(),
            "custom.json".into(),
        ])
        .unwrap();
        assert_eq!(custom.theme, Some(PathBuf::from("custom.json")));
    }

    #[test]
    fn installed_save_resumes_real_movement() {
        let pack = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../content-packs/realtime-clock.browser.crystalpack")
            .canonicalize()
            .unwrap();
        let assets = AssetRoot::new(pack.parent().unwrap());
        let runtime = CrystalRuntime::from_loaded_compiled_pack(
            &assets,
            read_loaded_verified_compiled_game_pack(&pack).unwrap(),
        )
        .unwrap();
        let path = std::env::temp_dir().join(format!(
            "geothite-install-resume-{}.crystalsave",
            std::process::id()
        ));
        let mut game = RuntimeGameShell::new_game(
            assets.clone(),
            runtime.clone(),
            "CHRIS",
            Some(path.clone()),
        )
        .unwrap();
        game.press(GameButton::Start).unwrap();
        game.press(GameButton::B).unwrap();
        game.save(&path).unwrap();
        let mut resumed =
            RuntimeGameShell::load_save(assets, runtime, path.clone(), Some(path.clone())).unwrap();
        let before = resumed.snapshot().unwrap();
        resumed.press(GameButton::Right).unwrap();
        resumed.press(GameButton::Right).unwrap();
        let after = resumed.snapshot().unwrap();
        std::fs::remove_file(path).unwrap();
        assert_ne!(
            before.overworld.tile, after.overworld.tile,
            "resume must move: {:?}",
            after.phase
        );
    }

    #[test]
    fn mcp_exposes_the_play_surface_under_the_geothite_server() {
        let tools = mcp_tools();
        let names = tools
            .as_array()
            .expect("tool catalog")
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "observe",
                "status",
                "move",
                "press",
                "hold_button",
                "execute_macro",
                "recent_events",
                "journal",
                "map_info",
                "flow_state"
            ]
        );
        assert_eq!(env!("CARGO_PKG_NAME"), "geothite");
    }

    #[test]
    fn mcp_battle_exit_restores_movement_and_start_menu() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let pack = repo.join("content-packs/text-tui.crystalpack");
        let assets = AssetRoot::new(pack.parent().unwrap());
        let loaded = read_loaded_verified_compiled_game_pack(&pack).expect("external TUI pack");
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&assets, loaded).unwrap();
        let spawn = runtime.title_new_game_spawn_identifier().unwrap();
        let mut fixture = crystal_runtime::RuntimeGameShell::new_game_at_runtime_tile(
            assets.clone(),
            runtime.clone(),
            spawn,
            "Route29",
            46,
            12,
        )
        .unwrap();
        fixture
            .add_party_pokemon(
                "TOTODILE",
                50,
                None,
                None,
                "MCP_EXIT",
                1,
                crystal_core::models::pokemon::Dv::from_non_hp(10, 10, 10, 10),
            )
            .unwrap();
        let save = env::temp_dir().join(format!(
            "geothite-mcp-battle-exit-{}.crystalsave",
            std::process::id()
        ));
        fixture.save(&save).unwrap();
        let loaded = RuntimeGameShell::load_save(assets, runtime, save.clone(), None);
        std::fs::remove_file(&save).unwrap();
        let mut game = loaded.unwrap();
        let mut renderer = RuntimeTextRenderer::default();
        let mut cursor = None;
        for step in 0..256 {
            call_mcp_tool(&mut game, &mut renderer, &mut cursor, Some("move"),
                &json!({"direction": if (step / 2) % 2 == 0 { "right" } else { "left" }, "steps": 1}), None).unwrap();
            if game.snapshot().unwrap().battle.is_some() {
                break;
            }
        }
        assert!(
            game.snapshot().unwrap().battle.is_some(),
            "MCP did not enter a grass encounter"
        );
        for _ in 0..256 {
            let shown = game.presentation_snapshot().unwrap();
            let view = renderer.render(&shown);
            if shown.battle.is_none() && !view.confirmation_input_owned() {
                break;
            }
            // Level-50 learnsets need not put a damaging move in slot zero.
            // Choose a displayed attack through the real menu instead of
            // spending 256 A presses on SCREECH instead of attacking.
            let attack = view.menu.iter().position(|line| {
                line.text.contains("PP")
                    && [
                        "SLASH",
                        "WATER GUN",
                        "HYDRO PUMP",
                        "RAGE",
                        "BITE",
                        "SCRATCH",
                    ]
                    .iter()
                    .any(|name| line.text.contains(name))
            });
            let selected = view
                .menu
                .iter()
                .position(|line| line.kind == geothite::LineKind::Selected)
                .unwrap_or(0);
            let button = attack
                .filter(|index| *index != selected)
                .map_or("a", |index| if index > selected { "down" } else { "up" });
            call_mcp_tool(
                &mut game,
                &mut renderer,
                &mut cursor,
                Some("press"),
                &json!({"button": button}),
                None,
            )
            .unwrap();
        }
        let before = game.snapshot().unwrap();
        assert!(
            before.battle.is_none(),
            "MCP battle did not finish: {}",
            render_snapshot_text(
                &renderer.render(&game.presentation_snapshot().unwrap()),
                false
            )
        );
        call_mcp_tool(
            &mut game,
            &mut renderer,
            &mut cursor,
            Some("move"),
            &json!({"direction": "up", "steps": 2}),
            None,
        )
        .unwrap();
        let after = game.snapshot().unwrap();
        assert_ne!(
            before.overworld.tile, after.overworld.tile,
            "MCP movement is blocked after battle"
        );
        call_mcp_tool(
            &mut game,
            &mut renderer,
            &mut cursor,
            Some("press"),
            &json!({"button": "start"}),
            None,
        )
        .unwrap();
        let observed = call_mcp_tool(
            &mut game,
            &mut renderer,
            &mut cursor,
            Some("observe"),
            &json!({"detail": "compact"}),
            None,
        )
        .unwrap();
        assert!(
            observed["structuredContent"]["menu"]
                .as_array()
                .unwrap()
                .iter()
                .any(|line| {
                    line["text"]
                        .as_str()
                        .is_some_and(|text| text.contains("SAVE"))
                }),
            "MCP Start menu is blocked after battle: {observed}"
        );
    }

    #[test]
    fn mcp_drives_real_movement_and_production_start_menu() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let pack_path = repo_root.join("content-packs/text-tui.crystalpack");
        if !pack_path.exists() {
            eprintln!(
                "skipping external-pack MCP regression; build {} first",
                pack_path.display()
            );
            return;
        }
        let asset_root = AssetRoot::new(pack_path.parent().expect("pack parent"));
        let loaded =
            read_loaded_verified_compiled_game_pack(&pack_path).expect("read text TUI pack");
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&asset_root, loaded)
            .expect("load text TUI pack");
        let mut game =
            RuntimeGameShell::new_game(asset_root, runtime, "CHRIS", None).expect("start MCP game");
        let mut renderer = RuntimeTextRenderer::default();
        let mut cursor = None;

        let movement = call_mcp_tool(
            &mut game,
            &mut renderer,
            &mut cursor,
            Some("move"),
            &json!({ "direction": "right", "steps": 1 }),
            None,
        )
        .expect("MCP movement");
        assert_eq!(movement["structuredContent"]["position"]["x"], 4);
        assert_eq!(movement["structuredContent"]["position"]["y"], 3);

        call_mcp_tool(
            &mut game,
            &mut renderer,
            &mut cursor,
            Some("press"),
            &json!({ "button": "start" }),
            None,
        )
        .expect("MCP Start press");
        let observed = call_mcp_tool(
            &mut game,
            &mut renderer,
            &mut cursor,
            Some("observe"),
            &json!({ "detail": "compact" }),
            None,
        )
        .expect("MCP observe");
        let menu = observed["structuredContent"]["menu"]
            .as_array()
            .expect("visible production Start menu");
        let has_option = |label: &str| {
            menu.iter().any(|line| {
                line["text"]
                    .as_str()
                    .is_some_and(|text| text.trim_start_matches([' ', '>']).trim() == label)
            })
        };
        assert!(has_option("PACK"), "missing PACK in {menu:?}");
        assert!(has_option("SAVE"), "missing SAVE in {menu:?}");

        call_mcp_tool(
            &mut game,
            &mut renderer,
            &mut cursor,
            Some("press"),
            &json!({ "button": "down" }),
            None,
        )
        .expect("MCP Start-menu navigation");
        call_mcp_tool(
            &mut game,
            &mut renderer,
            &mut cursor,
            Some("press"),
            &json!({ "button": "a" }),
            None,
        )
        .expect("MCP Start-menu selection");
        let card = call_mcp_tool(
            &mut game,
            &mut renderer,
            &mut cursor,
            Some("observe"),
            &json!({ "detail": "compact" }),
            None,
        )
        .expect("MCP Trainer Card observe");
        let card_lines = card["structuredContent"]["dialogue"]
            .as_array()
            .expect("visible Trainer Card");
        assert!(card_lines.iter().any(|line| line["text"] == "NAME/ CHRIS"));
        assert!(card_lines.iter().any(|line| line["text"] == "TIME   0:00"));
    }
}
