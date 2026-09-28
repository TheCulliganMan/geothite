use crystal_core::systems::script_text::ScriptTextBody;
use crystal_core::world::{
    map::{Direction, TilePosition, determine_quadrant_index},
    session::raw_event_tile_to_runtime_tile_checked,
};
use crystal_runtime::{RuntimeMapCatalogSnapshot, RuntimeShellPhase, RuntimeShellSnapshot};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ACTION_LOG_LIMIT: usize = 8;
const MAX_RENDER_CHARS: usize = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineKind {
    Normal,
    Heading,
    Selected,
    Hint,
    Danger,
    Water,
    Grass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotLine {
    pub text: String,
    pub kind: LineKind,
}

impl SnapshotLine {
    fn new(text: impl Into<String>, kind: LineKind) -> Self {
        Self {
            text: text.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextSnapshot {
    pub viewport_title: String,
    pub viewport: Vec<SnapshotLine>,
    pub info_title: String,
    pub info: Vec<SnapshotLine>,
    pub menu: Vec<SnapshotLine>,
    pub prompt: Vec<SnapshotLine>,
    pub dialogue: Vec<SnapshotLine>,
    pub action_log: Vec<SnapshotLine>,
    pub input_hints: Vec<SnapshotLine>,
    pub marker: Option<(i16, i16, char)>,
    pub status_line: String,
    pub frame: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportSize {
    pub width: u16,
    pub height: u16,
}

impl Default for ViewportSize {
    fn default() -> Self {
        Self {
            width: 21,
            height: 15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeTextRenderer {
    viewport: ViewportSize,
    menu_index: usize,
    yes_no_index: usize,
    action_log: Vec<String>,
}

impl Default for RuntimeTextRenderer {
    fn default() -> Self {
        Self::new(ViewportSize::default())
    }
}

impl RuntimeTextRenderer {
    pub fn new(viewport: ViewportSize) -> Self {
        Self {
            viewport,
            menu_index: 0,
            yes_no_index: 0,
            action_log: Vec::new(),
        }
    }

    pub fn menu_index(&self) -> usize {
        self.menu_index
    }
    pub fn yes_no_index(&self) -> usize {
        self.yes_no_index
    }

    pub fn move_selection(&mut self, delta: isize, option_count: usize, yes_no: bool) {
        if option_count == 0 {
            return;
        }
        let cursor = if yes_no {
            &mut self.yes_no_index
        } else {
            &mut self.menu_index
        };
        *cursor = ((*cursor as isize + delta).rem_euclid(option_count as isize)) as usize;
    }

    pub fn record_action(&mut self, action: impl Into<String>) {
        self.action_log.push(action.into());
        if self.action_log.len() > ACTION_LOG_LIMIT {
            self.action_log
                .drain(..self.action_log.len() - ACTION_LOG_LIMIT);
        }
    }

    pub fn render(&mut self, source: &RuntimeShellSnapshot) -> TextSnapshot {
        let (viewport_title, viewport) = if let Some(battle) = &source.battle {
            ("Battle".to_string(), battle_lines(source, battle))
        } else {
            (map_title(source), overworld_lines(source, self.viewport))
        };
        let menu = menu_lines(source, self.menu_index);
        if !menu.is_empty() {
            self.menu_index = self.menu_index.min(menu.len().saturating_sub(1));
        }
        let prompt = prompt_lines(source, self.yes_no_index);
        let dialogue = dialogue_lines(source);
        let input_hints = hint_lines(
            source,
            !menu.is_empty(),
            !prompt.is_empty(),
            !dialogue.is_empty(),
        );
        let status_line = format!(
            "{:?}  {}  ({}, {})  facing {:?}",
            source.phase,
            source.overworld.map_name,
            source.overworld.tile.x,
            source.overworld.tile.y,
            source.overworld.facing
        );
        TextSnapshot {
            viewport_title,
            viewport,
            info_title: "Trainer".to_string(),
            info: info_lines(source),
            menu,
            prompt,
            dialogue,
            action_log: self
                .action_log
                .iter()
                .map(|line| SnapshotLine::new(line, LineKind::Hint))
                .collect(),
            input_hints,
            marker: (!source.overworld_player_hidden).then_some((
                source.overworld.tile.x,
                source.overworld.tile.y,
                '@',
            )),
            status_line,
            frame: source.overworld.frame,
        }
    }
}

fn map_title(source: &RuntimeShellSnapshot) -> String {
    source
        .maps
        .iter()
        .find(|map| map.map_name == source.overworld.map_name)
        .and_then(|map| map.metadata.as_ref().map(|metadata| metadata.name.clone()))
        .unwrap_or_else(|| source.overworld.map_name.clone())
}

fn overworld_lines(source: &RuntimeShellSnapshot, viewport: ViewportSize) -> Vec<SnapshotLine> {
    let Some(map) = source
        .maps
        .iter()
        .find(|map| map.map_name == source.overworld.map_name)
    else {
        return vec![SnapshotLine::new(
            "(active map unavailable)",
            LineKind::Danger,
        )];
    };
    let map_width = i16::try_from(u32::from(map.attributes.width) * 2).unwrap_or(i16::MAX);
    let map_height = i16::try_from(u32::from(map.attributes.height) * 2).unwrap_or(i16::MAX);
    let width = i16::try_from(viewport.width)
        .unwrap_or(i16::MAX)
        .min(map_width);
    let height = i16::try_from(viewport.height)
        .unwrap_or(i16::MAX)
        .min(map_height);
    let left = (source.overworld.tile.x - width / 2).clamp(0, map_width.saturating_sub(width));
    let top = (source.overworld.tile.y - height / 2).clamp(0, map_height.saturating_sub(height));
    let visible_objects = source
        .visible_object_runtime_tiles
        .values()
        .copied()
        .collect::<Vec<_>>();
    let mut rows = Vec::with_capacity(height as usize + 1);
    let mut header = String::from("   ");
    for x in left..left + width {
        header.push_str(&format!("{x:02} "));
    }
    rows.push(SnapshotLine::new(header.trim_end(), LineKind::Hint));
    for y in top..top + height {
        let mut text = format!("{y:02} ");
        for x in left..left + width {
            let tile = TilePosition::new(x, y);
            let token = if tile == source.overworld.tile && !source.overworld_player_hidden {
                player_token(source.overworld.facing).to_string()
            } else if visible_objects.contains(&tile) {
                "N".to_string()
            } else if map
                .events
                .warps
                .iter()
                .any(|event| event_runtime_tile(event.x, event.y) == Some(tile))
            {
                "D".to_string()
            } else if let Some(event) = map
                .events
                .bg_events
                .iter()
                .find(|event| event_runtime_tile(event.x, event.y) == Some(tile))
            {
                background_event_token(&event.script).to_string()
            } else {
                terrain_glyph(source, map, tile).0.to_string()
            };
            text.push_str(&format!("{token:<2} "));
        }
        rows.push(SnapshotLine::new(text.trim_end(), LineKind::Normal));
    }
    rows
}

fn background_event_token(script: &str) -> char {
    let normalized = script.to_ascii_lowercase();
    if normalized.contains("pcscript") || normalized.ends_with("pc") {
        'P'
    } else if normalized.contains("bookshelf") {
        'B'
    } else if normalized.contains("sign")
        || normalized.contains("poster")
        || normalized.contains("radio")
    {
        'S'
    } else {
        '!'
    }
}

fn event_runtime_tile(x: u16, y: u16) -> Option<TilePosition> {
    raw_event_tile_to_runtime_tile_checked(x, y)
}

fn player_token(facing: Direction) -> &'static str {
    match facing {
        Direction::Up => "@^",
        Direction::Down => "@v",
        Direction::Left => "@<",
        Direction::Right => "@>",
    }
}

fn terrain_glyph(
    source: &RuntimeShellSnapshot,
    map: &RuntimeMapCatalogSnapshot,
    tile: TilePosition,
) -> (char, LineKind) {
    if tile.x < 0 || tile.y < 0 {
        return (' ', LineKind::Normal);
    }
    let metatile_x = usize::try_from(tile.x / 2).ok();
    let metatile_y = usize::try_from(tile.y / 2).ok();
    let map_width = usize::from(map.attributes.width);
    let Some(block) = metatile_y
        .and_then(|y| metatile_x.and_then(|x| y.checked_mul(map_width)?.checked_add(x)))
        .and_then(|index| map.blocks.get(index))
        .copied()
    else {
        return (' ', LineKind::Normal);
    };
    let Some(quadrant) = determine_quadrant_index(tile.x, tile.y) else {
        return (' ', LineKind::Normal);
    };
    let Some(tileset) = source
        .tilesets
        .iter()
        .find(|tileset| tileset.tileset_id == map.attributes.tileset_name)
    else {
        return ('.', LineKind::Normal);
    };
    let key = format!("{block:02x}");
    let Some(token) = tileset
        .collision
        .get(&key)
        .and_then(|entries| entries.get(quadrant))
    else {
        return ('.', LineKind::Normal);
    };
    classify_collision_token(token)
}

fn classify_collision_token(token: &str) -> (char, LineKind) {
    let upper = token.to_ascii_uppercase();
    if upper.contains("WATER") || upper.contains("WHIRLPOOL") || upper.contains("BUOY") {
        ('≈', LineKind::Water)
    } else if upper.contains("GRASS") {
        ('\"', LineKind::Grass)
    } else if upper.contains("WARP")
        || upper.contains("DOOR")
        || upper.contains("STAIR")
        || upper.contains("LADDER")
        || upper.contains("CAVE")
    {
        ('D', LineKind::Hint)
    } else if upper.contains("WALL")
        || upper.contains("COUNTER")
        || upper.contains("BOOKSHELF")
        || upper.contains("SHELF")
        || upper.contains("WINDOW")
        || upper == "COLL_07"
    {
        ('#', LineKind::Normal)
    } else {
        ('.', LineKind::Normal)
    }
}

fn battle_lines(
    source: &RuntimeShellSnapshot,
    battle: &crystal_runtime::RuntimeBattleSnapshot,
) -> Vec<SnapshotLine> {
    let enemy = &battle.enemy_pokemon;
    let player = source
        .party
        .slots
        .iter()
        .find(|slot| slot.is_active_battle_pokemon)
        .or_else(|| source.party.slots.first())
        .map(|slot| &slot.pokemon);
    let mut lines = vec![
        SnapshotLine::new(
            format!("{}  Lv{}", enemy.nickname, enemy.level),
            LineKind::Heading,
        ),
        SnapshotLine::new(
            hp_bar(enemy.hp, enemy.max_hp, 24),
            hp_kind(enemy.hp, enemy.max_hp),
        ),
        SnapshotLine::new(
            format!(
                "HP {}/{}{}",
                enemy.hp,
                enemy.max_hp,
                status_suffix(enemy.status.as_deref())
            ),
            LineKind::Normal,
        ),
        SnapshotLine::new("", LineKind::Normal),
        SnapshotLine::new("                 ╭──────╮", LineKind::Hint),
        SnapshotLine::new("                 │  VS  │", LineKind::Selected),
        SnapshotLine::new("                 ╰──────╯", LineKind::Hint),
        SnapshotLine::new("", LineKind::Normal),
    ];
    if let Some(player) = player {
        lines.extend([
            SnapshotLine::new(
                format!("{}  Lv{}", player.nickname, player.level),
                LineKind::Heading,
            ),
            SnapshotLine::new(
                hp_bar(player.hp, player.max_hp, 24),
                hp_kind(player.hp, player.max_hp),
            ),
            SnapshotLine::new(
                format!(
                    "HP {}/{}{}",
                    player.hp,
                    player.max_hp,
                    status_suffix(player.status.as_deref())
                ),
                LineKind::Normal,
            ),
        ]);
    }
    lines
}

fn hp_bar(hp: u16, max_hp: u16, width: usize) -> String {
    let filled = if max_hp == 0 {
        0
    } else {
        usize::from(hp).saturating_mul(width) / usize::from(max_hp)
    }
    .min(width);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(width - filled))
}

fn hp_kind(hp: u16, max_hp: u16) -> LineKind {
    if max_hp > 0 && hp.saturating_mul(4) <= max_hp {
        LineKind::Danger
    } else {
        LineKind::Grass
    }
}

fn status_suffix(status: Option<&str>) -> String {
    status
        .map(|status| format!("  {status}"))
        .unwrap_or_default()
}

fn info_lines(source: &RuntimeShellSnapshot) -> Vec<SnapshotLine> {
    let badges = source
        .progression
        .badges
        .johto
        .iter()
        .filter(|value| **value)
        .count()
        + source
            .progression
            .badges
            .kanto
            .iter()
            .filter(|value| **value)
            .count();
    let mut lines = vec![
        SnapshotLine::new(
            format!("{} · ${}", source.trainer.player_name, source.trainer.money),
            LineKind::Heading,
        ),
        SnapshotLine::new(
            format!(
                "Badges {badges}/16 · Dex {}/{}",
                source.progression.pokedex_owned, source.progression.pokedex_seen
            ),
            LineKind::Normal,
        ),
    ];
    for slot in &source.party.slots {
        let pokemon = &slot.pokemon;
        let marker = if slot.is_active_battle_pokemon {
            '▶'
        } else {
            ' '
        };
        lines.push(SnapshotLine::new(
            format!(
                "{marker} {:<10} L{:>2} {:>3}/{:<3}{}",
                pokemon.nickname,
                pokemon.level,
                pokemon.hp,
                pokemon.max_hp,
                status_suffix(pokemon.status.as_deref())
            ),
            hp_kind(pokemon.hp, pokemon.max_hp),
        ));
    }
    lines
}

fn menu_lines(source: &RuntimeShellSnapshot, selected: usize) -> Vec<SnapshotLine> {
    let Some(menu) = &source.ui.menu else {
        return Vec::new();
    };
    let Some(vertical) = menu.layout.vertical_menus.last() else {
        return Vec::new();
    };
    let selected = selected.min(vertical.options.len().saturating_sub(1));
    vertical
        .options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let shell_selected = option.trim_start().starts_with('>');
            let active = shell_selected
                || (!vertical
                    .options
                    .iter()
                    .any(|option| option.trim_start().starts_with('>'))
                    && index == selected);
            let option = if shell_selected {
                option.trim_start().trim_start_matches('>').trim_start()
            } else {
                option.as_str()
            };
            SnapshotLine::new(
                format!("{} {option}", if active { ">" } else { " " }),
                if active {
                    LineKind::Selected
                } else {
                    LineKind::Normal
                },
            )
        })
        .collect()
}

fn prompt_lines(source: &RuntimeShellSnapshot, selected: usize) -> Vec<SnapshotLine> {
    if source.ui.pending_yes_no.is_none() {
        return Vec::new();
    }
    ["YES", "NO"]
        .into_iter()
        .enumerate()
        .map(|(index, option)| {
            let active = index == selected;
            SnapshotLine::new(
                format!("{} {option}", if active { ">" } else { " " }),
                if active {
                    LineKind::Selected
                } else {
                    LineKind::Normal
                },
            )
        })
        .collect()
}

fn dialogue_lines(source: &RuntimeShellSnapshot) -> Vec<SnapshotLine> {
    let Some(text) = &source.ui.text else {
        return Vec::new();
    };
    if let Some(value) = &text.asm_text {
        return resolve_embedded_text(
            value,
            &source.script_events.named_buffers,
            &source.trainer.player_name,
            rival_name(source),
            source.progression.time.day_of_week,
        )
        .lines()
        .map(|line| SnapshotLine::new(line.trim(), LineKind::Normal))
        .collect();
    }
    text.body
        .as_ref()
        .map(|body| render_script_text_body(source, body))
        .unwrap_or_default()
}

fn render_script_text_body(
    source: &RuntimeShellSnapshot,
    body: &ScriptTextBody,
) -> Vec<SnapshotLine> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let flush = |lines: &mut Vec<String>, current: &mut String| {
        if !current.is_empty() {
            lines.push(std::mem::take(current));
        }
    };
    for command in &body.commands {
        let rendered = match command.command.as_str() {
            "text_ram" | "text_decimal" => command
                .args
                .first()
                .and_then(|id| named_text_buffer(&source.script_events.named_buffers, id))
                .cloned()
                .unwrap_or_default(),
            _ => render_text_args(&command.args),
        };
        match command.command.as_str() {
            "text" | "text_start" | "text_block" | "text_ram" | "text_decimal" => {
                current.push_str(&rendered);
            }
            "text_today" => current.push_str("<TODAY>"),
            "text_pause" | "text_asm" | "text_far" => {}
            "text_low" => flush(&mut lines, &mut current),
            "line" | "next" | "cont" => {
                flush(&mut lines, &mut current);
                current.push_str(&rendered);
            }
            "para" | "text_promptbutton" => {
                flush(&mut lines, &mut current);
                if !lines.last().is_some_and(String::is_empty) {
                    lines.push(String::new());
                }
                current.push_str(&rendered);
            }
            "prompt" | "done" | "text_end" => {
                flush(&mut lines, &mut current);
                break;
            }
            opcode if opcode.starts_with("sound_") => {}
            _ => {}
        }
    }
    flush(&mut lines, &mut current);
    lines
        .into_iter()
        .map(|line| {
            SnapshotLine::new(
                normalize_visible_text(
                    &line,
                    &source.trainer.player_name,
                    rival_name(source),
                    source.progression.time.day_of_week,
                ),
                LineKind::Normal,
            )
        })
        .collect()
}

fn render_text_args(args: &[String]) -> String {
    args.iter()
        .map(|arg| arg.trim().trim_matches('"').trim_end_matches('@'))
        .collect::<Vec<_>>()
        .join(" ")
}

fn rival_name(source: &RuntimeShellSnapshot) -> &str {
    source
        .script_events
        .variables
        .get("_rival_name")
        .map(String::as_str)
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("???")
}

fn named_text_buffer<'a>(
    buffers: &'a BTreeMap<String, String>,
    buffer_id: &str,
) -> Option<&'a String> {
    buffers.get(buffer_id).or_else(|| {
        let alias = match buffer_id {
            "wStringBuffer1" => "STRING_BUFFER_1",
            "wStringBuffer2" => "STRING_BUFFER_2",
            "wStringBuffer3" => "STRING_BUFFER_3",
            "wStringBuffer4" => "STRING_BUFFER_4",
            "wStringBuffer5" => "STRING_BUFFER_5",
            "STRING_BUFFER_1" => "wStringBuffer1",
            "STRING_BUFFER_2" => "wStringBuffer2",
            "STRING_BUFFER_3" => "wStringBuffer3",
            "STRING_BUFFER_4" => "wStringBuffer4",
            "STRING_BUFFER_5" => "wStringBuffer5",
            _ => return None,
        };
        buffers.get(alias)
    })
}

fn resolve_embedded_text(
    text: &str,
    buffers: &BTreeMap<String, String>,
    player_name: &str,
    rival_name: &str,
    day_of_week: u8,
) -> String {
    let mut resolved = String::with_capacity(text.len());
    let mut remainder = text;
    while let Some(start) = remainder.find('<') {
        resolved.push_str(&remainder[..start]);
        let token = &remainder[start..];
        let Some(end) = token.find('>') else {
            resolved.push_str(token);
            remainder = "";
            break;
        };
        let token_body = &token[1..end];
        let buffer_id = token_body
            .strip_prefix("RAM:")
            .or_else(|| {
                token_body
                    .strip_prefix("DECIMAL:")
                    .and_then(|arguments| arguments.split(',').next())
                    .map(str::trim)
            })
            .unwrap_or(token_body);
        let is_buffer = token_body.starts_with("STRING_BUFFER_")
            || token_body.starts_with("RAM:")
            || token_body.starts_with("DECIMAL:");
        if is_buffer {
            if let Some(value) = named_text_buffer(buffers, buffer_id) {
                resolved.push_str(value);
            }
        } else {
            resolved.push_str(&token[..=end]);
        }
        remainder = &token[end + 1..];
    }
    resolved.push_str(remainder);
    normalize_visible_text(&resolved, player_name, rival_name, day_of_week)
}

fn normalize_visible_text(
    text: &str,
    player_name: &str,
    rival_name: &str,
    day_of_week: u8,
) -> String {
    const DAY_NAMES: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
    text.replace("<……>", "……")
        .replace("<POKE>", "#")
        .replace('#', "POKé")
        .replace("<PLAYER>", player_name)
        .replace("<PLAY_G>", player_name)
        .replace("<RIVAL>", rival_name)
        .replace("<TODAY>", DAY_NAMES[usize::from(day_of_week % 7)])
}

fn hint_lines(
    source: &RuntimeShellSnapshot,
    has_menu: bool,
    has_prompt: bool,
    has_dialogue: bool,
) -> Vec<SnapshotLine> {
    let hints: &[&str] = if has_prompt {
        &["↑/↓ choose", "Z/J/Space confirm", "X/K/B/Esc cancel"]
    } else if has_menu {
        &["↑/↓ choose", "Z/J/Space select", "X/K/B/Esc back"]
    } else if has_dialogue || matches!(source.phase, RuntimeShellPhase::Text) {
        &["Z/J/Space advance", "X/K/B skip/back"]
    } else {
        &[
            "Arrows/WASD/HKL move",
            "Z/J/Space A",
            "X/K/B back",
            "Enter Start · Tab Select",
            "? help",
        ]
    };
    hints
        .iter()
        .map(|line| SnapshotLine::new(*line, LineKind::Hint))
        .collect()
}

pub fn render_snapshot_text(snapshot: &TextSnapshot, compact: bool) -> String {
    let mut lines = Vec::new();
    push_section(&mut lines, &snapshot.viewport_title, &snapshot.viewport);
    if !compact {
        push_section(&mut lines, &snapshot.info_title, &snapshot.info);
        if let Some((x, y, marker)) = snapshot.marker {
            push_section(
                &mut lines,
                "Marker",
                &[SnapshotLine::new(
                    format!("({x}, {y}) {marker}"),
                    LineKind::Hint,
                )],
            );
        }
    }
    push_section(&mut lines, "Dialogue", &snapshot.dialogue);
    push_section(&mut lines, "Menu", &snapshot.menu);
    push_section(&mut lines, "Prompt", &snapshot.prompt);
    if !compact {
        push_section(&mut lines, "Action log", &snapshot.action_log);
    }
    push_section(&mut lines, "Controls", &snapshot.input_hints);
    lines.join("\n")
}

fn push_section(output: &mut Vec<String>, title: &str, entries: &[SnapshotLine]) {
    if entries.is_empty() {
        return;
    }
    if !output.is_empty() {
        output.push(String::new());
    }
    output.push(title.to_ascii_uppercase());
    output.extend(entries.iter().map(|line| line.text.clone()));
}

pub fn wrap_lines(lines: &[SnapshotLine], max_chars: usize) -> Vec<SnapshotLine> {
    let width = max_chars.clamp(1, MAX_RENDER_CHARS);
    let mut output = Vec::new();
    for line in lines {
        if line.text.is_empty() {
            output.push(line.clone());
            continue;
        }
        let mut remaining = line.text.as_str();
        while remaining.chars().count() > width {
            let byte_end = remaining
                .char_indices()
                .nth(width)
                .map(|(index, _)| index)
                .unwrap_or(remaining.len());
            let candidate = &remaining[..byte_end];
            let split = candidate
                .rfind(' ')
                .filter(|index| *index > 0)
                .unwrap_or(byte_end);
            output.push(SnapshotLine::new(remaining[..split].trim_end(), line.kind));
            remaining = remaining[split..].trim_start();
        }
        output.push(SnapshotLine::new(remaining, line.kind));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collision_tokens_have_distinct_accessible_glyphs() {
        assert_eq!(classify_collision_token("WATER"), ('≈', LineKind::Water));
        assert_eq!(
            classify_collision_token("TALL_GRASS"),
            ('\"', LineKind::Grass)
        );
        assert_eq!(classify_collision_token("WALL"), ('#', LineKind::Normal));
        assert_eq!(classify_collision_token("DOOR"), ('D', LineKind::Hint));
    }

    #[test]
    fn selection_wraps_in_both_directions() {
        let mut renderer = RuntimeTextRenderer::default();
        renderer.move_selection(-1, 3, false);
        assert_eq!(renderer.menu_index(), 2);
        renderer.move_selection(1, 3, false);
        assert_eq!(renderer.menu_index(), 0);
    }

    #[test]
    fn action_log_is_bounded() {
        let mut renderer = RuntimeTextRenderer::default();
        for index in 0..20 {
            renderer.record_action(format!("action {index}"));
        }
        assert_eq!(renderer.action_log.len(), ACTION_LOG_LIMIT);
        assert_eq!(renderer.action_log[0], "action 12");
    }

    #[test]
    fn wrapping_preserves_kind_blanks_and_unicode_boundaries() {
        let lines = vec![
            SnapshotLine::new("alpha beta gamma", LineKind::Selected),
            SnapshotLine::new("", LineKind::Normal),
            SnapshotLine::new("▲▲▲▲", LineKind::Hint),
        ];
        let wrapped = wrap_lines(&lines, 6);
        assert_eq!(
            wrapped
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "beta", "gamma", "", "▲▲▲▲"]
        );
        assert!(
            wrapped[..3]
                .iter()
                .all(|line| line.kind == LineKind::Selected)
        );
    }

    #[test]
    fn text_helpers_resolve_buffers_aliases_and_player_tokens() {
        let buffers = BTreeMap::from([
            ("STRING_BUFFER_3".to_string(), "CHIKORITA".to_string()),
            ("wStringBuffer4".to_string(), "RARE CANDY".to_string()),
        ]);
        assert_eq!(
            named_text_buffer(&buffers, "wStringBuffer3").map(String::as_str),
            Some("CHIKORITA")
        );
        assert_eq!(
            resolve_embedded_text(
                "<PLAYER> found <RAM:wStringBuffer4> on <TODAY>!",
                &buffers,
                "KRIS",
                "SILVER",
                2,
            ),
            "KRIS found RARE CANDY on TUE!"
        );
        assert_eq!(render_text_args(&["#MON!@".to_string()]), "#MON!");
    }
}
