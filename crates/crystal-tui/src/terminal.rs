use crate::{LineKind, SnapshotLine, TextSnapshot, TuiTheme, wrap_lines};
#[cfg(not(target_arch = "wasm32"))]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crystal_core::input::GameButton;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalAction {
    GameButton(GameButton),
    MoveSelection(isize),
    Wait,
    Refresh,
    BeginCommand,
    ToggleHelp,
    Save,
    Quit,
    None,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn map_key_event(event: KeyEvent, modal_selection: bool) -> TerminalAction {
    if event.modifiers.contains(KeyModifiers::CONTROL) && matches!(event.code, KeyCode::Char('c')) {
        return TerminalAction::Quit;
    }
    match event.code {
        KeyCode::Char(':') => TerminalAction::BeginCommand,
        KeyCode::Char('?') => TerminalAction::ToggleHelp,
        KeyCode::F(5) => TerminalAction::Save,
        KeyCode::Up | KeyCode::Char('w' | 'W' | 'k') if modal_selection => {
            TerminalAction::MoveSelection(-1)
        }
        KeyCode::Down | KeyCode::Char('s' | 'S') if modal_selection => {
            TerminalAction::MoveSelection(1)
        }
        KeyCode::Char('a' | 'A') if modal_selection => TerminalAction::GameButton(GameButton::A),
        KeyCode::Up | KeyCode::Char('w' | 'W' | 'k') => TerminalAction::GameButton(GameButton::Up),
        KeyCode::Down | KeyCode::Char('s' | 'S') => TerminalAction::GameButton(GameButton::Down),
        KeyCode::Left | KeyCode::Char('a' | 'h' | 'H') => {
            TerminalAction::GameButton(GameButton::Left)
        }
        KeyCode::Right | KeyCode::Char('d' | 'D' | 'l' | 'L') => {
            TerminalAction::GameButton(GameButton::Right)
        }
        KeyCode::Char('z' | 'Z' | 'j' | 'J' | 'A' | ' ') => {
            TerminalAction::GameButton(GameButton::A)
        }
        KeyCode::Char('x' | 'X' | 'b' | 'B' | 'K') | KeyCode::Esc => {
            TerminalAction::GameButton(GameButton::B)
        }
        KeyCode::Enter => TerminalAction::GameButton(GameButton::Start),
        KeyCode::Tab | KeyCode::BackTab => TerminalAction::GameButton(GameButton::Select),
        KeyCode::Char('.') => TerminalAction::Wait,
        KeyCode::Char('r' | 'R') => TerminalAction::Refresh,
        _ => TerminalAction::None,
    }
}

#[derive(Debug, Clone)]
pub struct TerminalUi {
    pub theme: TuiTheme,
    pub show_help: bool,
    pub compact: bool,
    pub notice: Option<String>,
}

impl TerminalUi {
    pub fn new(theme: TuiTheme) -> Self {
        Self {
            theme,
            show_help: false,
            compact: false,
            notice: None,
        }
    }

    pub fn draw(&self, frame: &mut Frame<'_>, snapshot: &TextSnapshot) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(6),
                Constraint::Length(2),
            ])
            .split(area);
        self.draw_header(frame, chunks[0], snapshot);
        self.draw_body(frame, chunks[1], snapshot);
        self.draw_footer(frame, chunks[2], snapshot);
        if self.show_help {
            self.draw_help(frame, centered_rect(72, 20, area));
        }
    }

    fn draw_header(&self, frame: &mut Frame<'_>, area: Rect, snapshot: &TextSnapshot) {
        let title = Line::from(vec![
            Span::styled(
                " GEOTHITE ",
                Style::default()
                    .fg(self.theme.background.into())
                    .bg(self.theme.accent.into())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                &snapshot.status_line,
                Style::default().fg(self.theme.foreground.into()),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(title).block(self.block("GEOTHITE", false)),
            area,
        );
    }

    fn draw_body(&self, frame: &mut Frame<'_>, area: Rect, snapshot: &TextSnapshot) {
        let wide = area.width >= 96 && !self.compact;
        let regions = Layout::default()
            .direction(if wide {
                Direction::Horizontal
            } else {
                Direction::Vertical
            })
            .constraints(if wide {
                [Constraint::Percentage(66), Constraint::Percentage(34)]
            } else {
                [Constraint::Percentage(62), Constraint::Percentage(38)]
            })
            .split(area);
        let mut game_lines = vec![SnapshotLine {
            text: snapshot.viewport_title.clone(),
            kind: LineKind::Heading,
        }];
        game_lines.extend(snapshot.viewport.iter().cloned());
        self.draw_panel(frame, regions[0], "GAME BOY", &game_lines, false);
        let context = context_lines(snapshot, self.compact);
        self.draw_panel(
            frame,
            regions[1],
            "INFO",
            &context,
            !snapshot.menu.is_empty() || !snapshot.prompt.is_empty(),
        );
    }

    fn draw_footer(&self, frame: &mut Frame<'_>, area: Rect, snapshot: &TextSnapshot) {
        let hint = self.notice.as_deref().unwrap_or_else(|| {
            snapshot
                .input_hints
                .first()
                .map(|line| line.text.as_str())
                .unwrap_or("? help · :q! quit")
        });
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw(hint),
                Line::raw(if area.width >= 96 {
                    "@ you · N person · D door · # wall · T tree · ║ barrier · ↓↘↙ ledges · ≈ water · \" grass · ? unknown"
                } else {
                    "# wall · T tree · ↓ ledge · ║ barrier · ? unknown"
                }),
            ])
                .alignment(Alignment::Center)
                .style(Style::default().fg(self.theme.muted.into())),
            area,
        );
    }

    fn draw_panel(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        title: &str,
        lines: &[SnapshotLine],
        highlight: bool,
    ) {
        let content_width = usize::from(area.width.saturating_sub(4)).max(1);
        let display = wrap_lines(lines, content_width)
            .into_iter()
            .map(|line| self.styled_line(line))
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(display)
                .block(self.block(title, highlight))
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn block<'a>(&self, title: &'a str, highlight: bool) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(if highlight {
                self.theme.accent.into()
            } else {
                self.theme.border.into()
            }))
            .style(Style::default().bg(self.theme.background.into()))
    }

    fn line_style(&self, kind: LineKind) -> Style {
        let base = Style::default()
            .fg(self.theme.foreground.into())
            .bg(self.theme.background.into());
        match kind {
            LineKind::Heading => base
                .fg(self.theme.accent.into())
                .add_modifier(Modifier::BOLD),
            LineKind::Selected => base
                .fg(self.theme.selected_foreground.into())
                .bg(self.theme.selected_background.into())
                .add_modifier(Modifier::BOLD),
            LineKind::Hint => base.fg(self.theme.muted.into()),
            LineKind::Danger => base
                .fg(self.theme.danger.into())
                .add_modifier(Modifier::BOLD),
            LineKind::Water => base.fg(self.theme.water.into()),
            LineKind::Grass => base.fg(self.theme.grass.into()),
            LineKind::Normal => base,
        }
    }

    fn styled_line<'a>(&self, line: SnapshotLine) -> Line<'a> {
        if line.kind != LineKind::Normal {
            return Line::from(Span::styled(
                if line.text.is_empty() {
                    " ".to_string()
                } else {
                    line.text
                },
                self.line_style(line.kind),
            ));
        }
        let spans = line
            .text
            .chars()
            .map(|character| {
                let kind = match character {
                    '▲' | '▼' | '◀' | '▶' => LineKind::Selected,
                    '≈' => LineKind::Water,
                    '"' => LineKind::Grass,
                    'D' | '!' => LineKind::Hint,
                    _ => LineKind::Normal,
                };
                Span::styled(character.to_string(), self.line_style(kind))
            })
            .collect::<Vec<_>>();
        if spans.is_empty() {
            Line::from(" ")
        } else {
            Line::from(spans)
        }
    }

    fn draw_help(&self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);
        let text = [
            "ARROWS/WASD/HKL Move or choose",
            "Z/J/SPACE       A · confirm · interact",
            "X/K/B/ESC       B · cancel",
            "ENTER           START",
            "TAB             SELECT",
            ".               Wait 8 frames",
            "R               Refresh",
            #[cfg(not(target_arch = "wasm32"))]
            "F5              Save (when --save is set)",
            #[cfg(target_arch = "wasm32")]
            "F5 / Save       Save in this browser",
            "?               Close this help",
            "MAP: # wall · T tree · D door · N person",
            "     arrows: one-way ledges · ║ side barrier",
            "     ≈ water · \" grass · ? unknown terrain",
            #[cfg(target_arch = "wasm32")]
            "TOUCH: swipe move · tap A · hold Start · 2 fingers B",
            #[cfg(not(target_arch = "wasm32"))]
            ":q! / CTRL-C    Quit",
            #[cfg(target_arch = "wasm32")]
            "Save before closing this browser tab",
        ]
        .join("\n");
        frame.render_widget(
            Paragraph::new(text)
                .block(self.block("CONTROLS", true))
                .alignment(Alignment::Left),
            area,
        );
    }
}

fn context_lines(snapshot: &TextSnapshot, compact: bool) -> Vec<SnapshotLine> {
    let mut output = Vec::new();
    append(&mut output, "DIALOGUE", &snapshot.dialogue);
    append(&mut output, "MENU", &snapshot.menu);
    append(&mut output, "PROMPT", &snapshot.prompt);
    if !compact {
        append(
            &mut output,
            &snapshot.info_title.to_ascii_uppercase(),
            &snapshot.info,
        );
        append(&mut output, "ACTION LOG", &snapshot.action_log);
    }
    if output.is_empty() {
        output.push(SnapshotLine {
            text: "No modal UI. Explore the map.".to_string(),
            kind: LineKind::Hint,
        });
    }
    output
}

fn append(output: &mut Vec<SnapshotLine>, title: &str, lines: &[SnapshotLine]) {
    if lines.is_empty() {
        return;
    }
    if !output.is_empty() {
        output.push(SnapshotLine {
            text: String::new(),
            kind: LineKind::Normal,
        });
    }
    output.push(SnapshotLine {
        text: title.to_string(),
        kind: LineKind::Heading,
    });
    output.extend_from_slice(lines);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width.saturating_sub(2));
    let height = height.min(area.height.saturating_sub(2));
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;
    use ratatui::{Terminal, backend::TestBackend};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new_with_kind(code, KeyModifiers::NONE, KeyEventKind::Press)
    }

    #[test]
    fn directional_keys_control_game_until_a_modal_is_open() {
        assert_eq!(
            map_key_event(key(KeyCode::Up), false),
            TerminalAction::GameButton(GameButton::Up)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Up), true),
            TerminalAction::MoveSelection(-1)
        );
    }

    #[test]
    fn game_boy_and_shell_keys_are_distinct() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('z')), false),
            TerminalAction::GameButton(GameButton::A)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('?')), false),
            TerminalAction::ToggleHelp
        );
        assert_eq!(
            map_key_event(key(KeyCode::F(5)), false),
            TerminalAction::Save
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char(' ')), false),
            TerminalAction::GameButton(GameButton::A)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Enter), false),
            TerminalAction::GameButton(GameButton::Start)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Tab), false),
            TerminalAction::GameButton(GameButton::Select)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char(':')), false),
            TerminalAction::BeginCommand
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('q')), false),
            TerminalAction::None
        );
    }

    #[test]
    fn lowercase_a_confirms_menu_less_battle_and_dialogue() {
        let mut view = snapshot();
        view.menu.clear();
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(
            map_key_event(key, view.confirmation_input_owned()),
            TerminalAction::GameButton(GameButton::Left)
        );
        for phase in ["WildBattle", "TrainerBattle", "StaticWildBattle", "Text"] {
            view.status_line = phase.to_owned();
            assert_eq!(
                map_key_event(key, view.confirmation_input_owned()),
                TerminalAction::GameButton(GameButton::A)
            );
        }
        view.status_line = "Overworld".into();
        view.dialogue.push(SnapshotLine {
            text: "Wild RATTATA appeared!".into(),
            kind: LineKind::Normal,
        });
        assert_eq!(
            map_key_event(key, view.confirmation_input_owned()),
            TerminalAction::GameButton(GameButton::A)
        );
    }

    fn snapshot() -> TextSnapshot {
        TextSnapshot {
            viewport_title: "New Bark Town".to_string(),
            viewport: vec![
                SnapshotLine {
                    text: "..≈..".to_string(),
                    kind: LineKind::Normal,
                },
                SnapshotLine {
                    text: ".N▶D.".to_string(),
                    kind: LineKind::Normal,
                },
            ],
            info_title: "Trainer".to_string(),
            info: vec![SnapshotLine {
                text: "KRIS · $3000".to_string(),
                kind: LineKind::Heading,
            }],
            menu: vec![SnapshotLine {
                text: "> PACK".to_string(),
                kind: LineKind::Selected,
            }],
            prompt: Vec::new(),
            dialogue: Vec::new(),
            action_log: Vec::new(),
            input_hints: vec![SnapshotLine {
                text: "Z select".to_string(),
                kind: LineKind::Hint,
            }],
            marker: Some((4, 6, '@')),
            status_line: "Overworld".to_string(),
            frame: 1,
        }
    }

    #[test]
    fn responsive_ui_renders_in_wide_and_narrow_terminals() {
        let ui = TerminalUi::new(TuiTheme::default());
        for (width, height) in [(120, 32), (48, 24)] {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).expect("test terminal");
            terminal
                .draw(|frame| ui.draw(frame, &snapshot()))
                .expect("draw");
            let rendered = terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            assert!(rendered.contains("GEOTHITE"));
            assert!(rendered.contains("New Bark Town"));
            assert!(rendered.contains("PACK"));
        }
    }
}
