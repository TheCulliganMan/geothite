//! Responsive terminal illustration. Reads real pack art and controller
//! collision, but never dispatches input or replaces authoritative state.
use crate::{TextSnapshot, snapshot::terrain_glyph};
use crystal_core::world::{
    encounters::TimeOfDay,
    map::{Direction, TilePosition},
    movement::MovementMode,
};
use crystal_runtime::RuntimeShellSnapshot;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};
use std::collections::BTreeMap;
const INK: Color = Color::Rgb(232, 230, 218);
const MUTED: Color = Color::Rgb(159, 177, 179);
const GOLD: Color = Color::Rgb(246, 194, 113);
const GREEN: Color = Color::Rgb(129, 202, 161);
const PAPER: Color = Color::Rgb(3, 3, 18);
struct Art {
    width: usize,
    pixels: Vec<u8>,
    metatiles: Vec<u8>,
    palette: Vec<u8>,
}
type Palette = [[u8; 3]; 4];
#[derive(Default)]
pub struct PaintedRenderer {
    // Native clients read directly from the supplied pack, never the build checkout.
    assets: Option<BTreeMap<String, Vec<u8>>>,
    art: BTreeMap<String, Option<Art>>,
    sprites: BTreeMap<(String, bool), Option<Sprite>>,
    actors: BTreeMap<String, Option<Actor>>,
    palettes: BTreeMap<(String, String), Vec<Palette>>,
    pub viewport: Option<crate::ViewportSize>,
    replay: std::collections::VecDeque<crystal_bevy::bevy_shell::VisibleBattleReplayFrame>,
    replay_frame: Option<crystal_bevy::bevy_shell::VisibleBattleReplayFrame>,
    scene: Option<SceneGeometry>,
    dense: bool,
    dot_sizes: Vec<u8>,
    dot_width: u16,
    dot_height: u16,
    ink_phase: f64,
    native_detail: Option<Buffer>,
}
#[derive(Clone, Copy)]
struct SceneGeometry {
    area: Rect,
    left: i16,
    top: i16,
    nx: u16,
    ny: u16,
    cw: u16,
    ch: u16,
}
struct Actor {
    width: usize,
    pixels: Vec<u8>,
    frames: usize,
}
struct Sprite {
    size: u32,
    width: u32,
    pixels: Vec<u8>,
}
fn put(b: &mut Buffer, x: u16, y: u16, text: &str, color: Color) {
    if x < b.area.width && y < b.area.height {
        b.set_stringn(
            x,
            y,
            text,
            usize::from(b.area.width - x),
            Style::default().fg(color).bg(PAPER),
        );
    }
}
fn rule(b: &mut Buffer, y: u16, title: &str) {
    put(
        b,
        1,
        y,
        &"─".repeat(usize::from(b.area.width - 2)),
        Color::Rgb(63, 93, 98),
    );
    put(b, 2, y, &format!(" {title} "), GOLD);
}
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = vec![];
    for paragraph in text.lines() {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if line.chars().count() + usize::from(!line.is_empty()) + word.chars().count() > width
                && !line.is_empty()
            {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}
impl PaintedRenderer {
    /// Same layout, camera, source art and controller snapshot; denser glyphs
    /// only for real terminals. Browser circles/ASCII battle art stay unchanged.
    pub fn draw_native(
        &mut self,
        s: &RuntimeShellSnapshot,
        t: &TextSnapshot,
        cols: u16,
        rows: u16,
        notice: Option<&str>,
        help: bool,
    ) -> Buffer {
        let mut buffer = self.draw(s, t, cols, rows, notice, help, false);
        self.native_detail = None;
        let scene = self.scene;
        if let Some(fine) = self.detailed_world(s) {
            let scene = scene.expect("fine scene geometry");
            crate::ink::pack_terminal_dots(
                &mut buffer,
                scene.area,
                &fine,
                &self.animated_dot_sizes(),
                scene.left,
                scene.top,
            );
            self.native_detail = Some(fine);
        }
        buffer
    }
    /// Exact circle sizes/colors from the shared detailed dot field. Native
    /// terminal graphics is only another output transport, not another scene.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn circle_image(&self, width: u32, height: u32) -> Option<image::RgbImage> {
        let fine = self.native_detail.as_ref()?;
        Some(crate::ink::rasterize_dots(
            fine,
            &self.animated_dot_sizes(),
            width,
            height,
        ))
    }
    pub fn advance_ink(&mut self) {
        self.advance_ink_by(0.1);
    }
    /// Shared seconds-based cosmetic clock. Never advances the controller.
    /// Bound discontinuities after hidden tabs, suspension or slow transports.
    pub fn advance_ink_by(&mut self, seconds: f64) {
        if seconds.is_finite() && seconds > 0. {
            self.ink_phase = (self.ink_phase + seconds.min(0.25) * 0.8)
                % (std::f64::consts::TAU * 100.);
        }
    }
    pub fn animated_dot_sizes(&self) -> Vec<u8> {
        let Some(scene) = self.scene else {
            return Vec::new();
        };
        self.dot_sizes
            .iter()
            .enumerate()
            .map(|(i, size)| {
                if *size == 0 {
                    return 0;
                }
                let x = f64::from(scene.left)
                    + (i % usize::from(self.dot_width)) as f64 / f64::from(scene.cw * 2);
                let y = f64::from(scene.top)
                    + (i / usize::from(self.dot_width)) as f64 / f64::from(scene.ch * 4);
                // Slow travelling breaths: roughly eight seconds per cycle,
                // with half the original radius swing, not a pulsing fade-out.
                // A downward-biased range gives large dots room to move rather
                // than pinning their whole animation against the 250 cap.
                let wave = 0.92
                    + 0.14 * (x * 0.43 + y * 0.31 + self.ink_phase).sin()
                    + 0.06 * (x * 0.21 - y * 0.37 - self.ink_phase * 0.71).sin();
                (f64::from(*size) * wave).clamp(0., 250.) as u8
            })
            .collect()
    }
    pub fn scene_bounds(&self) -> Vec<u16> {
        self.scene
            .map(|s| vec![s.area.x, s.area.y, s.area.width, s.area.height])
            .unwrap_or_default()
    }
    pub fn detailed_world(&mut self, source: &RuntimeShellSnapshot) -> Option<Buffer> {
        if source.battle.is_some() {
            return None;
        }
        let scene = self.scene?;
        let mut b = Buffer::empty(Rect::new(0, 0, scene.area.width * 2, scene.area.height * 4));
        b.set_style(b.area, Style::default().fg(INK).bg(PAPER));
        self.dot_width = b.area.width;
        self.dot_height = b.area.height;
        self.dot_sizes = vec![0; b.content.len()];
        self.dense = true;
        let height = b.area.height;
        self.world(&mut b, source, height);
        self.dense = false;
        Some(b)
    }
    pub fn replay(&mut self, replays: Vec<crystal_bevy::bevy_shell::VisibleBattleReplay>) {
        self.cancel_replay();
        self.replay = replays
            .into_iter()
            .flat_map(|r| r.frames)
            .take(160)
            .collect();
        self.advance_replay();
    }
    pub fn cancel_replay(&mut self) {
        self.replay.clear();
        self.replay_frame = None;
    }
    pub fn advance_replay(&mut self) -> bool {
        self.replay_frame = self.replay.pop_front();
        self.replay_frame.is_some()
    }
    pub fn replay_active(&self) -> bool {
        self.replay_frame.is_some()
    }
    pub fn from_pack_assets(files: &BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            assets: Some(
                files
                    .iter()
                    .filter(|(path, _)| {
                        path.starts_with("gfx/") || path.starts_with("data/tilesets/")
                    })
                    .map(|(path, bytes)| (path.clone(), bytes.clone()))
                    .collect(),
            ),
            ..Self::default()
        }
    }
    fn bytes(&self, path: &str) -> Option<Vec<u8>> {
        match &self.assets {
            Some(files) => files.get(path).cloned(),
            None => crystal_runtime::read_runtime_asset(format!("apps/web/assets/{path}")).ok(),
        }
    }
    fn image(&self, path: &str) -> Option<image::RgbaImage> {
        image::load_from_memory(&self.bytes(path)?)
            .ok()
            .map(|image| image.to_rgba8())
    }
    fn palette_bank(&mut self, path: &str, group: &str) -> Vec<Palette> {
        let key = (path.to_owned(), group.to_owned());
        if !self.palettes.contains_key(&key) {
            let bank = self
                .bytes(path)
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .map(|text| parse_palettes(&text, group))
                .unwrap_or_default();
            self.palettes.insert(key.clone(), bank);
        }
        self.palettes[&key].clone()
    }
    fn load(&mut self, s: &RuntimeShellSnapshot, name: &str) {
        if self.art.contains_key(name) {
            return;
        }
        let art = (|| {
            let mut image = self.image(&format!("gfx/tilesets/{name}.png"))?;
            if name == "battle_tower_outside" {
                // Same authored roof substitution as the graphical frontend.
                let roof = self.image("gfx/tilesets/roofs/olivine.png")?;
                if roof.width() * roof.height() != 9 * 64 || roof.width() % 8 != 0 {
                    return None;
                }
                for tile in 0..9 {
                    let dst = tile + 10;
                    for y in 0..8 {
                        for x in 0..8 {
                            image.put_pixel(
                                dst % (image.width() / 8) * 8 + x,
                                dst / (image.width() / 8) * 8 + y,
                                *roof.get_pixel(
                                    tile % (roof.width() / 8) * 8 + x,
                                    tile / (roof.width() / 8) * 8 + y,
                                ),
                            );
                        }
                    }
                }
            }
            let metatiles = self.bytes(&format!("data/tilesets/{name}_metatiles.bin"))?;
            let palette = s
                .tilesets
                .iter()
                .find(|t| t.tileset_id == name)?
                .palette_map
                .clone();
            Some(Art {
                width: image.width() as usize,
                pixels: image.into_raw(),
                metatiles,
                palette,
            })
        })();
        self.art.insert(name.to_owned(), art);
    }
    pub fn draw(
        &mut self,
        s: &RuntimeShellSnapshot,
        t: &TextSnapshot,
        cols: u16,
        rows: u16,
        notice: Option<&str>,
        help: bool,
        touchscreen: bool,
    ) -> Buffer {
        self.scene = None;
        let mut b = Buffer::empty(Rect::new(0, 0, cols, rows));
        b.set_style(b.area, Style::default().fg(INK).bg(PAPER));
        put(&mut b, 2, 0, "G E O T H I T E", GOLD);
        put(
            &mut b,
            2,
            1,
            &format!("{} · {:?}", s.overworld.map_name, s.phase),
            INK,
        );
        let location = format!("{},{}", s.overworld.tile.x, s.overworld.tile.y);
        put(&mut b, 20, 0, &location, MUTED);
        if touchscreen && cols >= 68 && rows <= 32 {
            return self.landscape(s, t, b, notice, help);
        }
        let context: Vec<String> = if help && touchscreen {
            vec![
                "Swipe: move / choose. Tap: A.".into(),
                "Hold: START. Two fingers: B.".into(),
                "Buttons below are touch targets.".into(),
                "Arrows on terrain are ledges.".into(),
                "# wall  T tree  | barrier  ? unknown".into(),
            ]
        } else if help {
            vec![
                "Arrows / WASD / HJKL: move or choose.".into(),
                "Z / J / Space: A. X / Esc: B. Enter: START.".into(),
                "A confirms while dialogue or battle owns input.".into(),
                "V: text/paint. F5: save. :wq: save+quit.".into(),
                "# wall  T tree  arrows: ledges  | barrier  ? unknown".into(),
            ]
        } else if !t.menu.is_empty() || !t.prompt.is_empty() {
            t.menu
                .iter()
                .chain(&t.prompt)
                .flat_map(|l| wrap(&l.text, usize::from(cols - 4)))
                .collect()
        } else if !t.dialogue.is_empty() {
            t.dialogue
                .iter()
                .flat_map(|l| wrap(&l.text, usize::from(cols - 4)))
                .collect()
        } else {
            vec![
                format!("{}  ${}", s.trainer.player_name, s.trainer.money),
                "# wall  T tree  v ledge  | barrier".into(),
            ]
        };
        let height =
            (context.len() as u16 + 2).min(rows.saturating_sub(if touchscreen { 14 } else { 10 }));
        let controls = rows - if touchscreen { 9 } else { 2 };
        let cy = controls.saturating_sub(height + 1);
        if s.battle.is_some() {
            self.battle(&mut b, s, Rect::new(2, 3, cols - 4, cy.saturating_sub(4)));
        } else {
            self.world(&mut b, s, cy.saturating_sub(3));
        }
        rule(
            &mut b,
            cy,
            if help {
                "FIELD GUIDE"
            } else if !t.menu.is_empty() {
                "CHOOSE"
            } else if !t.dialogue.is_empty() {
                "DIALOGUE"
            } else {
                "TRAINER"
            },
        );
        let capacity = usize::from(height.saturating_sub(1));
        let selected = context
            .iter()
            .position(|line| {
                line.trim_start().starts_with('>') || line.trim_start().starts_with('▶')
            })
            .unwrap_or(0);
        let start = selected.saturating_sub(capacity.saturating_sub(1));
        for (i, line) in context.iter().skip(start).take(capacity).enumerate() {
            put(
                &mut b,
                2,
                cy + 1 + i as u16,
                line,
                if line.trim_start().starts_with('>') || line.trim_start().starts_with('▶') {
                    GOLD
                } else {
                    INK
                },
            );
        }
        if let Some(notice) = notice {
            put(&mut b, 2, controls - 1, notice, GREEN);
        }
        rule(
            &mut b,
            controls,
            if touchscreen {
                "TOUCH"
            } else {
                "ARROWS move · Z confirm · X back · ENTER start · V text/paint"
            },
        );
        b
    }
    fn world(&mut self, b: &mut Buffer, s: &RuntimeShellSnapshot, height: u16) {
        let Some(map) = s.maps.iter().find(|m| m.map_name == s.overworld.map_name) else {
            return;
        };
        self.load(s, &map.attributes.tileset_name);
        let declared = map.attributes.time_of_day.as_deref();
        let indoor =
            map.attributes.environment.as_deref().is_some_and(|v| {
                v.eq_ignore_ascii_case("indoor") || v.eq_ignore_ascii_case("gate")
            });
        let group = match if indoor && !matches!(declared, Some("dark" | "darkness")) {
            Some("indoor")
        } else {
            declared
        } {
            Some("indoor" | "indoors") => "indoor",
            Some("dark" | "darkness")
                if s.progression
                    .active_engine_flags
                    .contains("STATUSFLAGS_FLASH") =>
            {
                "nite"
            }
            Some("dark" | "darkness") => "dark",
            Some("morn" | "morning") => "morn",
            Some("nite" | "night") => "nite",
            Some("day") => "day",
            _ => match s.progression.time.time_of_day {
                TimeOfDay::Morning => "morn",
                TimeOfDay::Day => "day",
                TimeOfDay::Night => "nite",
            },
        };
        let mut bank = self.palette_bank(
            &format!("gfx/tilesets/{}.pal", map.attributes.tileset_name),
            "",
        );
        if bank.is_empty() {
            bank = self.palette_bank("gfx/tilesets/bg_tiles.pal", group);
        }
        if bank.is_empty() {
            bank = self.palette_bank("gfx/tilesets/bg_tiles.pal", "day");
        }
        let actors_bank = self.palette_bank(
            "gfx/overworld/npc_sprites.pal",
            if group == "indoor" { "day" } else { group },
        );
        let mut actors = BTreeMap::new();
        for object in &s.visible_objects {
            let Some(id) = object.object_identifier.as_ref() else {
                continue;
            };
            let Some(tile) = s.visible_object_runtime_tiles.get(id) else {
                continue;
            };
            let sprite = s
                .script_events
                .variable_sprites
                .get(&object.sprite)
                .unwrap_or(&object.sprite);
            let facing = s
                .visible_object_facings
                .get(id)
                .copied()
                .unwrap_or(Direction::Down);
            let palette = actor_palette(s, sprite, object.pal);
            if let Some(pixels) = self.actor(
                sprite,
                facing,
                object.sprite_has_facings,
                &actors_bank,
                palette,
            ) {
                actors.insert((tile.x, tile.y), pixels);
            }
        }
        if !s.overworld_player_hidden {
            let female = s.trainer.player_gender == 1;
            let (sprite, override_id) = match s.overworld.mode {
                MovementMode::Normal | MovementMode::Skate => (
                    if female {
                        "SPRITE_KRIS"
                    } else {
                        "SPRITE_CHRIS"
                    },
                    s.trainer.player_palette_id,
                ),
                MovementMode::Bike => (
                    if female {
                        "SPRITE_KRIS_BIKE"
                    } else {
                        "SPRITE_CHRIS_BIKE"
                    },
                    s.trainer.player_palette_id,
                ),
                MovementMode::Surf => ("SPRITE_SURF", 1),
                MovementMode::SurfPika => ("SPRITE_SURFING_PIKACHU", 0),
            };
            if let Some(pixels) = self.actor(
                sprite,
                s.overworld.facing,
                true,
                &actors_bank,
                actor_palette(s, sprite, override_id),
            ) {
                actors.insert((s.overworld.tile.x, s.overworld.tile.y), pixels);
            }
        }
        let art = self
            .art
            .get(&map.attributes.tileset_name)
            .and_then(Option::as_ref);
        let mw = map.attributes.width.saturating_mul(2) as i16;
        let mh = map.attributes.height.saturating_mul(2) as i16;
        // The real pack tiles and actor pixels, with world-anchored dithering.
        let old = self.scene.filter(|_| self.dense);
        let cw = old.map_or(if b.area.width >= 96 { 8 } else { 6 }, |s| s.cw * 2);
        let ch = old.map_or(cw / 2, |s| s.ch * 4);
        let nx = old.map_or(
            ((b.area.width.saturating_sub(4)) / cw)
                .max(1)
                .min(21)
                .min(mw as u16),
            |s| s.nx,
        );
        let ny = old.map_or((height / ch).max(1).min(15).min(mh as u16), |s| s.ny);
        self.viewport = Some(crate::ViewportSize {
            width: nx,
            height: ny,
        });
        let left = old.map_or(
            (s.overworld.tile.x - nx as i16 / 2).clamp(0, mw - nx as i16),
            |s| s.left,
        );
        let top = old.map_or(
            (s.overworld.tile.y - ny as i16 / 2).clamp(0, mh - ny as i16),
            |s| s.top,
        );
        let ox = if self.dense {
            0
        } else {
            (b.area.width - nx * cw) / 2
        };
        let oy = if self.dense {
            0
        } else {
            3 + height.saturating_sub(ny * ch) / 2
        };
        if !self.dense {
            self.scene = Some(SceneGeometry {
                area: Rect::new(ox, oy, nx * cw, ny * ch),
                left,
                top,
                nx,
                ny,
                cw,
                ch,
            });
        }
        for y in 0..ny {
            for x in 0..nx {
                let tile = TilePosition::new(left + x as i16, top + y as i16);
                let (glyph, _) = terrain_glyph(s, map, tile);
                let bx = ox + x * cw;
                let by = oy + y * ch;
                let actor = actors.get(&(tile.x, tile.y));
                let mut pixels = [[0u8; 3]; 256];
                for py in 0..16 {
                    for px in 0..16 {
                        let index = py * 16 + px;
                        pixels[index] = art
                            .and_then(|a| {
                                a.sample(
                                    &map.blocks,
                                    usize::from(map.attributes.width),
                                    tile,
                                    px,
                                    py,
                                )
                            })
                            .map(|(gray, id)| {
                                bank.get(usize::from(id))
                                    .or_else(|| bank.first())
                                    .map_or([gray; 3], |p| p[shade(gray)])
                            })
                            .unwrap_or([36, 42, 54]);
                        if let Some(color) = actor.and_then(|a| a[index]) {
                            pixels[index] = color;
                        }
                    }
                }
                let area = Rect::new(bx, by, cw, ch);
                let sizes = crate::ink::tile(
                    b,
                    area,
                    &pixels,
                    tile.x,
                    tile.y,
                    if self.dense { 0. } else { self.ink_phase },
                );
                // No floating map labels in painted art. Text/MCP retain the
                // exact collision tokens; painted ledges use small dot chevrons.
                crate::ink::collision(b, area, glyph);
                if self.dense {
                    for row in 0..ch {
                        for col in 0..cw {
                            let dst = usize::from(by + row) * usize::from(self.dot_width)
                                + usize::from(bx + col);
                            let local = usize::from(row) * usize::from(cw) + usize::from(col);
                            self.dot_sizes[dst] =
                                if sizes[local] == 0 && b[(bx + col, by + row)].symbol() == "●" {
                                    210
                                } else {
                                    sizes[local]
                                };
                        }
                    }
                }
            }
        }
    }
    fn actor(
        &mut self,
        token: &str,
        facing: Direction,
        has_facings: bool,
        bank: &[Palette],
        palette: u8,
    ) -> Option<Vec<Option<[u8; 3]>>> {
        let name = token.trim_start_matches("SPRITE_").to_ascii_lowercase();
        if !self.actors.contains_key(&name) {
            let path = if let Some(icon) = name.strip_prefix("icon_") {
                format!("gfx/icons/{icon}.png")
            } else {
                format!("gfx/sprites/{name}.png")
            };
            let actor = self.image(&path).map(|image| {
                let width = image.width() as usize;
                let pixels = image.into_raw();
                let frames = pixels.len() / (width * width * 4);
                let uniform = |frame| {
                    let data = &pixels[frame * width * width * 4..(frame + 1) * width * width * 4];
                    data.chunks_exact(4).all(|p| p == &data[..4])
                };
                let padded = frames > 1 && !uniform(0) && (1..frames).all(|frame| uniform(frame));
                Actor {
                    width,
                    pixels,
                    frames: if padded { 1 } else { frames },
                }
            });
            self.actors.insert(name.clone(), actor);
        }
        let actor = self.actors.get(&name)?.as_ref()?;
        let frame = if has_facings {
            match facing {
                Direction::Up if actor.frames >= 2 => 1,
                Direction::Left | Direction::Right if actor.frames >= 3 => 2,
                _ => 0,
            }
        } else {
            0
        };
        let mut pixels = vec![None; 256];
        for y in 0..16 {
            for x in 0..16 {
                let sx = if facing == Direction::Right && has_facings {
                    15 - x
                } else {
                    x
                } * actor.width
                    / 16;
                let sy = y * actor.width / 16 + frame * actor.width;
                let p = actor
                    .pixels
                    .get((sy * actor.width + sx) * 4..(sy * actor.width + sx) * 4 + 4)?;
                let index = shade(p[0]);
                if p[3] != 0 && index != 0 {
                    pixels[y * 16 + x] = Some(
                        bank.get(usize::from(palette))
                            .map_or([p[0]; 3], |palette| palette[index]),
                    );
                }
            }
        }
        Some(pixels)
    }
    fn landscape(
        &mut self,
        s: &RuntimeShellSnapshot,
        t: &TextSnapshot,
        mut b: Buffer,
        notice: Option<&str>,
        help: bool,
    ) -> Buffer {
        let split = b.area.width / 2;
        let controls = b.area.height - 9;
        let mut scene = Buffer::empty(Rect::new(0, 0, split - 2, b.area.height));
        scene.set_style(scene.area, Style::default().fg(INK).bg(PAPER));
        if let Some(battle) = s.battle.as_ref() {
            let half = controls.saturating_sub(4) / 2;
            let bounds = Rect::new(0, 3, scene.area.width, controls.saturating_sub(3));
            let enemy_offset = self
                .replay_frame
                .as_ref()
                .map_or([0; 2], |f| f.enemy_offset);
            let player_offset = self
                .replay_frame
                .as_ref()
                .map_or([0; 2], |f| f.player_offset);
            self.portrait(
                &mut scene,
                &battle.enemy_pokemon.species.id,
                false,
                portrait_motion_rect(bounds, Rect::new(2, 3, split - 6, half), enemy_offset),
            );
            put(
                &mut b,
                split,
                3,
                &format!(
                    "{} Lv{}",
                    battle.enemy_pokemon.nickname, battle.enemy_pokemon.level
                ),
                GOLD,
            );
            health(
                &mut b,
                Rect::new(split, 4, split - 2, 2),
                battle.enemy_pokemon.hp,
                battle.enemy_pokemon.max_hp,
            );
            if let Some(index) = battle.active_player_party_index
                && let Some(slot) = s.party.slots.iter().find(|slot| slot.index == index)
            {
                put(
                    &mut b,
                    split,
                    7,
                    &format!("{} Lv{}", slot.pokemon.nickname, slot.pokemon.level),
                    GREEN,
                );
                health(
                    &mut b,
                    Rect::new(split, 8, split - 2, 2),
                    slot.pokemon.hp,
                    slot.pokemon.max_hp,
                );
                self.portrait(
                    &mut scene,
                    &slot.pokemon.species.id,
                    true,
                    portrait_motion_rect(
                        bounds,
                        Rect::new(2, 3 + half, split - 6, half),
                        player_offset,
                    ),
                );
            }
            if let Some(frame) = &self.replay_frame {
                for piece in &frame.pieces {
                    paint_effect(&mut scene, bounds, piece);
                }
            }
        } else {
            self.world(&mut scene, s, controls.saturating_sub(4));
        }
        for y in 3..controls {
            for x in 0..scene.area.width {
                b[(x, y)] = scene[(x, y)].clone();
            }
        }
        let lines = if help {
            vec![
                "Swipe to move. Tap A.".to_owned(),
                "Two fingers B. Hold START.".to_owned(),
            ]
        } else if !t.menu.is_empty() || !t.prompt.is_empty() {
            t.menu
                .iter()
                .chain(&t.prompt)
                .map(|line| line.text.clone())
                .collect()
        } else if !t.dialogue.is_empty() {
            t.dialogue.iter().map(|line| line.text.clone()).collect()
        } else {
            vec![
                format!("{} ${}", s.trainer.player_name, s.trainer.money),
                "# wall  T tree  v ledge".to_owned(),
            ]
        };
        let start = if s.battle.is_some() { 11 } else { 3 };
        let lines: Vec<_> = lines
            .iter()
            .flat_map(|line| wrap(line, usize::from(b.area.width - split - 2)))
            .collect();
        let capacity = usize::from(controls.saturating_sub(start + 1));
        let selected = lines
            .iter()
            .position(|line| line.trim_start().starts_with('>'))
            .unwrap_or(0);
        for (i, line) in lines
            .iter()
            .skip(selected.saturating_sub(capacity.saturating_sub(1)))
            .take(capacity)
            .enumerate()
        {
            put(
                &mut b,
                split,
                start + i as u16,
                line,
                if line.trim_start().starts_with('>') {
                    GOLD
                } else {
                    INK
                },
            );
        }
        if let Some(notice) = notice {
            put(&mut b, split, controls - 1, notice, GREEN);
        }
        rule(&mut b, controls, "TOUCH");
        b
    }
    fn battle(&mut self, b: &mut Buffer, s: &RuntimeShellSnapshot, area: Rect) {
        let Some(battle) = &s.battle else { return };
        let half = area.width / 2;
        let height = area.height / 2;
        let enemy = &battle.enemy_pokemon;
        put(
            b,
            area.x,
            area.y,
            &format!("{} Lv{}", enemy.nickname, enemy.level),
            GOLD,
        );
        health(
            b,
            Rect::new(area.x, area.y + 1, half.saturating_sub(1), 2),
            enemy.hp,
            enemy.max_hp,
        );
        let shift = |r, offset| portrait_motion_rect(area, r, offset);
        let enemy_offset = self
            .replay_frame
            .as_ref()
            .map_or([0; 2], |f| f.enemy_offset);
        let player_offset = self
            .replay_frame
            .as_ref()
            .map_or([0; 2], |f| f.player_offset);
        self.portrait(
            b,
            &enemy.species.id,
            false,
            shift(Rect::new(area.x + half, area.y, half, height), enemy_offset),
        );
        if let Some(index) = battle.active_player_party_index
            && let Some(slot) = s.party.slots.iter().find(|slot| slot.index == index)
        {
            let player = &slot.pokemon;
            self.portrait(
                b,
                &player.species.id,
                true,
                shift(
                    Rect::new(area.x, area.y + height, half, height),
                    player_offset,
                ),
            );
            let hud_y = area.y + area.height.saturating_sub(4);
            put(
                b,
                area.x + half,
                hud_y,
                &format!("{} Lv{}", player.nickname, player.level),
                GREEN,
            );
            health(
                b,
                Rect::new(area.x + half, hud_y + 1, half, 2),
                player.hp,
                player.max_hp,
            );
        }
        if let Some(frame) = &self.replay_frame {
            for piece in &frame.pieces {
                paint_effect(b, area, piece);
            }
        }
        // Effects must not obscure the live, readable health HUD.
        put(
            b,
            area.x,
            area.y,
            &format!("{} Lv{}", enemy.nickname, enemy.level),
            GOLD,
        );
        health(
            b,
            Rect::new(area.x, area.y + 1, half.saturating_sub(1), 2),
            enemy.hp,
            enemy.max_hp,
        );
        if let Some(index) = battle.active_player_party_index
            && let Some(slot) = s.party.slots.iter().find(|slot| slot.index == index)
        {
            let y = area.y + area.height.saturating_sub(4);
            put(
                b,
                area.x + half,
                y,
                &format!("{} Lv{}", slot.pokemon.nickname, slot.pokemon.level),
                GREEN,
            );
            health(
                b,
                Rect::new(area.x + half, y + 1, half, 2),
                slot.pokemon.hp,
                slot.pokemon.max_hp,
            );
        }
    }
    fn portrait(&mut self, b: &mut Buffer, species: &str, back: bool, area: Rect) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let species = species.to_ascii_lowercase().replace('-', "_");
        let species = if species == "unown" {
            "unown_a".to_owned()
        } else {
            species
        };
        let key = (species.clone(), back);
        if !self.sprites.contains_key(&key) {
            let path = format!(
                "gfx/pokemon/{species}/{}.png",
                if back { "back" } else { "front" }
            );
            let sprite = self.image(&path).map(|image| Sprite {
                size: image.width().min(image.height()),
                width: image.width(),
                pixels: image.into_raw(),
            });
            self.sprites.insert(key.clone(), sprite);
        }
        let sprite = &self.sprites[&key];
        let Some(sprite) = sprite else {
            put(
                b,
                area.x,
                area.y,
                if back {
                    "(backsprite unavailable)"
                } else {
                    "(frontsprite unavailable)"
                },
                MUTED,
            );
            return;
        };
        let width = area.width.min(area.height * 2);
        let height = area.height.min(width.div_ceil(2));
        let start = area.x + (area.width - width) / 2;
        for row in 0..height {
            for col in 0..width {
                let x0 = u32::from(col) * sprite.size / u32::from(width);
                let x1 = (u32::from(col + 1) * sprite.size / u32::from(width))
                    .max(x0 + 1)
                    .min(sprite.size);
                let y0 = u32::from(row) * sprite.size / u32::from(height);
                let y1 = (u32::from(row + 1) * sprite.size / u32::from(height))
                    .max(y0 + 1)
                    .min(sprite.size);
                let mut colored = [0_u32; 3];
                let mut count = 0_u32;
                let mut edge = 0_u32;
                for yy in y0..y1 {
                    for xx in x0..x1 {
                        let i = ((yy * sprite.width + xx) * 4) as usize;
                        let p = &sprite.pixels[i..i + 4];
                        // Exported sprites are already species-palette colored.
                        // Only palette-zero white/alpha is the transparent key.
                        if p[3] == 0 || p[..3].iter().all(|v| *v >= 248) {
                            continue;
                        }
                        count += 1;
                        if p[..3].iter().all(|v| *v < 48) {
                            edge += 1;
                        } else {
                            for c in 0..3 {
                                colored[c] += u32::from(p[c]);
                            }
                        }
                    }
                }
                if count == 0 {
                    continue;
                }
                let coverage = count * 255 / ((x1 - x0) * (y1 - y0));
                let ch = match coverage {
                    0..=40 => '.',
                    41..=90 => ':',
                    91..=150 => '+',
                    151..=210 => '#',
                    _ => '@',
                };
                let color = if edge * 2 >= count {
                    INK
                } else {
                    let n = (count - edge).max(1);
                    Color::Rgb(
                        (colored[0] / n).max(65) as u8,
                        (colored[1] / n).max(65) as u8,
                        (colored[2] / n).max(65) as u8,
                    )
                };
                put(b, start + col, area.y + row, &ch.to_string(), color);
            }
        }
    }
}
fn health(b: &mut Buffer, area: Rect, hp: u16, max_hp: u16) {
    if area.width < 4 {
        return;
    }
    let width = area.width.min(32).saturating_sub(2);
    let hp = hp.min(max_hp);
    let filled = if max_hp == 0 {
        0
    } else {
        (u32::from(hp) * u32::from(width)).div_ceil(u32::from(max_hp)) as u16
    };
    let color = if u32::from(hp) * 5 <= u32::from(max_hp) {
        Color::Rgb(239, 108, 112)
    } else if u32::from(hp) * 2 <= u32::from(max_hp) {
        GOLD
    } else {
        GREEN
    };
    put(b, area.x, area.y, "[", MUTED);
    for x in 0..width {
        put(
            b,
            area.x + 1 + x,
            area.y,
            if x < filled { "█" } else { "░" },
            if x < filled {
                color
            } else {
                Color::Rgb(63, 93, 98)
            },
        );
    }
    put(b, area.x + width + 1, area.y, "]", MUTED);
    put(b, area.x, area.y + 1, &format!("HP {hp}/{max_hp}"), INK);
}
impl Art {
    fn sample(
        &self,
        blocks: &[u16],
        map_width: usize,
        tile: TilePosition,
        x: usize,
        y: usize,
    ) -> Option<(u8, u8)> {
        let block = *blocks.get(tile.y as usize / 2 * map_width + tile.x as usize / 2)? as usize;
        let px = x + tile.x as usize % 2 * 16;
        let py = y + tile.y as usize % 2 * 16;
        let tile_id = *self.metatiles.get(block * 16 + py / 8 * 4 + px / 8)? as usize;
        let count = self.pixels.len() / 4 / 64;
        let palette = self.palette.get(tile_id).copied().unwrap_or(0);
        let id = if palette & 8 != 0 && count % 2 == 0 {
            (tile_id & 127) + count / 2
        } else if palette & 8 != 0 && (tile_id & 127) + 128 < count {
            (tile_id & 127) + 128
        } else if tile_id < count {
            tile_id
        } else {
            tile_id.saturating_sub(128)
        };
        let columns = self.width / 8;
        if columns == 0 {
            return None;
        }
        let index = ((id / columns * 8 + py % 8) * self.width + id % columns * 8 + px % 8) * 4;
        Some((
            if *self.pixels.get(index + 3)? == 0 {
                255
            } else {
                *self.pixels.get(index)?
            },
            palette & 7,
        ))
    }
}
fn shade(gray: u8) -> usize {
    match gray {
        213..=255 => 0,
        128..=212 => 1,
        43..=127 => 2,
        _ => 3,
    }
}
fn rgb([r, g, b]: [u8; 3]) -> Color {
    Color::Rgb(r, g, b)
}
fn portrait_motion_rect(area: Rect, r: Rect, offset: [i16; 2]) -> Rect {
    let width = r.width.min(r.height * 2);
    let height = r.height.min(width.div_ceil(2));
    let r = Rect::new(r.x + (r.width - width) / 2, r.y, width, height);
    Rect::new(
        (i32::from(r.x) + i32::from(offset[0]) * i32::from(area.width) / 160).clamp(
            i32::from(area.x),
            i32::from(area.right().saturating_sub(r.width)),
        ) as u16,
        (i32::from(r.y) + i32::from(offset[1]) * i32::from(area.height) / 112).clamp(
            i32::from(area.y),
            i32::from(area.bottom().saturating_sub(r.height)),
        ) as u16,
        r.width,
        r.height,
    )
}
fn actor_palette(s: &RuntimeShellSnapshot, sprite: &str, override_id: u8) -> u8 {
    if override_id != 0 {
        override_id & 7
    } else {
        s.presentation
            .sprite_palette_defaults
            .get(sprite)
            .copied()
            .unwrap_or(0) as u8
            & 7
    }
}
fn paint_effect(
    b: &mut Buffer,
    area: Rect,
    piece: &crystal_bevy::bevy_shell::VisibleBattleReplayPiece,
) {
    // The actual production OAM raster is reinterpreted as colored ASCII.
    // Anchor sampling to source coordinates, never screen-space random noise.
    for sy in (0..piece.height).step_by(4) {
        for sx in (0..piece.width).step_by(2) {
            let i = (usize::from(sy) * usize::from(piece.width) + usize::from(sx)) * 4;
            let Some(p) = piece.rgba.get(i..i + 4) else {
                continue;
            };
            if p[3] == 0 {
                continue;
            }
            let x = i32::from(area.x)
                + (i32::from(piece.x) + i32::from(sx)) * i32::from(area.width) / 160;
            let y = i32::from(area.y)
                + (i32::from(piece.y) + i32::from(sy)) * i32::from(area.height) / 112;
            if x < i32::from(area.x)
                || y < i32::from(area.y)
                || x >= i32::from(area.x + area.width)
                || y >= i32::from(area.y + area.height)
            {
                continue;
            }
            let density = (u16::from(p[0]) + u16::from(p[1]) + u16::from(p[2])) / 3;
            b[(x as u16, y as u16)]
                .set_symbol(match density {
                    0..=60 => "@",
                    61..=120 => "#",
                    121..=190 => "+",
                    _ => "*",
                })
                .set_fg(rgb([p[0].max(65), p[1].max(65), p[2].max(65)]))
                .set_bg(PAPER);
        }
    }
}
#[cfg(test)]
fn sample_square(pixels: &[[u8; 3]; 256], x: u16, y: u16, size: u16) -> [u8; 3] {
    let x0 = usize::from(x) * 16 / usize::from(size);
    let x1 = usize::from(x + 1) * 16 / usize::from(size);
    let y0 = usize::from(y) * 16 / usize::from(size);
    let y1 = usize::from(y + 1) * 16 / usize::from(size);
    let mut total = [0u32; 3];
    for py in y0..y1 {
        for px in x0..x1 {
            for (channel, sum) in total.iter_mut().enumerate() {
                *sum += u32::from(pixels[py * 16 + px][channel]);
            }
        }
    }
    total.map(|value| (value / ((x1 - x0) * (y1 - y0)) as u32) as u8)
}
/// The same RGB5 palette file format used by the graphical client. No extracted
/// palette catalog: read the supplied pack and honor its authored time groups.
fn parse_palettes(text: &str, group: &str) -> Vec<Palette> {
    let mut active = String::new();
    let mut pending = Vec::new();
    let mut result = Vec::new();
    for raw in text.lines() {
        let trimmed = raw.trim();
        if trimmed.starts_with(';') && !trimmed.to_ascii_uppercase().contains("RGB") {
            active = trimmed
                .trim_start_matches(';')
                .trim()
                .to_ascii_lowercase()
                .replace(' ', "_");
            pending.clear();
            continue;
        }
        let line = raw.split(';').next().unwrap_or("").trim();
        if !line.to_ascii_uppercase().starts_with("RGB") || (!group.is_empty() && active != group) {
            continue;
        }
        let values: Option<Vec<u8>> = line[3..]
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|p| !p.is_empty())
            .map(|p| p.parse().ok())
            .collect();
        let Some(values) = values else {
            return Vec::new();
        };
        if values.len() % 3 != 0 {
            return Vec::new();
        }
        for chunk in values.chunks_exact(3) {
            pending.push(
                [chunk[0], chunk[1], chunk[2]]
                    .map(|v| if v <= 31 { (v << 3) | (v >> 2) } else { v }),
            );
            if pending.len() == 4 {
                result.push([pending[0], pending[1], pending[2], pending[3]]);
                pending.clear();
            }
        }
    }
    result.into_iter().take(8).collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portrait_motion_uses_actual_art_bounds_not_empty_slot_margins() {
        let area = Rect::new(2, 3, 124, 28);
        let enemy = Rect::new(64, 3, 62, 14);
        let resting = portrait_motion_rect(area, enemy, [0, 0]);
        assert!(portrait_motion_rect(area, enemy, [8, 0]).x > resting.x);
        let player = Rect::new(2, 17, 62, 14);
        assert!(
            portrait_motion_rect(area, player, [-8, 0]).x
                < portrait_motion_rect(area, player, [0, 0]).x
        );
    }
    #[test]
    fn painted_health_tracks_live_hp_and_thresholds() {
        for (hp, count, color) in [
            (100, 10, GREEN),
            (50, 5, GOLD),
            (10, 1, Color::Rgb(239, 108, 112)),
            (0, 0, Color::Rgb(239, 108, 112)),
        ] {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 3));
            health(&mut buffer, Rect::new(0, 0, 12, 2), hp, 100);
            assert_eq!(
                buffer.content.iter().filter(|c| c.symbol() == "█").count(),
                count
            );
            if hp > 0 {
                assert_eq!(buffer[(1, 0)].fg, color);
            }
            let text: String = buffer.content.iter().map(|c| c.symbol()).collect();
            assert!(text.contains(&format!("HP {hp}/100")));
        }
    }
    #[test]
    fn cosmetic_clock_is_elapsed_time_not_frontend_frame_count() {
        let mut terminal = PaintedRenderer::default();
        let mut browser = PaintedRenderer::default();
        for _ in 0..8 { terminal.advance_ink_by(0.1); }
        for _ in 0..5 { browser.advance_ink_by(0.16); }
        assert!((terminal.ink_phase - browser.ink_phase).abs() < 1e-10);
        assert!((terminal.ink_phase - 0.64).abs() < 1e-10, "Keep the slower eight-second breath");
        let phase = terminal.ink_phase;
        for seconds in [f64::NAN, f64::INFINITY, -1., 0.] { terminal.advance_ink_by(seconds); }
        assert_eq!(terminal.ink_phase, phase);
        terminal.advance_ink_by(1000.);
        assert!((terminal.ink_phase - phase - 0.2).abs() < 1e-10, "No resume catch-up burst");
    }
    #[test]
    fn ambient_ink_is_gentle_but_not_static() {
        let mut painter = PaintedRenderer::default();
        painter.scene = Some(SceneGeometry {
            area: Rect::new(0, 0, 1, 1), left: 0, top: 0,
            nx: 1, ny: 1, cw: 1, ch: 1,
        });
        painter.dot_width = 2;
        painter.dot_height = 2;
        painter.dot_sizes = vec![0, 50, 125, 250];
        let first = painter.animated_dot_sizes();
        let mut previous = first.clone();
        let mut changed = false;
        for _ in 0..80 {
            painter.advance_ink();
            let next = painter.animated_dot_sizes();
            assert_eq!(next[0], 0, "Empty paper never flickers into ink");
            for ((base, now), last) in painter.dot_sizes.iter().zip(&next).zip(&previous) {
                assert!(f64::from(*now) >= f64::from(*base) * 0.72 - 1.);
                assert!(f64::from(*now) <= f64::from(*base) * 1.12);
                assert!(now.abs_diff(*last) <= 5, "No aggressive radius jump per 100ms");
            }
            changed |= next[3].abs_diff(first[3]) >= 25;
            previous = next;
        }
        assert!(changed, "A slow full breath must remain visible");
    }
    #[test]
    fn palettes_and_square_sampling_keep_real_colors() {
        let palettes = parse_palettes(
            "; day\nRGB 31, 31, 31, 20, 10, 0, 5, 5, 5, 0, 0, 0\n; nite\nRGB 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3",
            "day",
        );
        assert_eq!(palettes.len(), 1);
        assert_eq!(palettes[0][1], [165, 82, 0]);
        assert_eq!([shade(255), shade(170), shade(85), shade(0)], [0, 1, 2, 3]);
        assert_eq!(
            sample_square(&[[81, 145, 210]; 256], 2, 3, 8),
            [81, 145, 210]
        );
        assert_eq!(
            wrap("Oh, CHRIS! Our neighbor, PROF.", 20),
            vec!["Oh, CHRIS! Our", "neighbor, PROF."]
        );
    }
    #[test]
    fn phone_context_and_controls_fit_without_touching_game_state() {
        let root = crystal_assets::AssetRoot::new(".");
        let pack = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content-packs/realtime-clock.browser.crystalpack");
        if !pack.is_file() {
            return;
        }
        let loaded =
            crystal_assets::read_loaded_verified_compiled_game_pack(pack.canonicalize().unwrap())
                .unwrap();
        let mut painter = PaintedRenderer::from_pack_assets(loaded.pack().runtime_files());
        let runtime =
            crystal_runtime::CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
        let mut controller =
            crystal_bevy::VisibleShellController::new_game(root, runtime, "CHRIS", None).unwrap();
        controller
            .press(crystal_core::input::GameButton::Start)
            .unwrap();
        let source = controller.presentation_snapshot().unwrap();
        let checksum = source.state_checksum.clone();
        let text = crate::RuntimeTextRenderer::default().render(&source);
        // All supplied maps share this renderer, not just the starting room.
        // Require every tileset/metatile to resolve directly from pack bytes.
        for map in &source.maps {
            painter.load(&source, &map.attributes.tileset_name);
            let art = painter.art[&map.attributes.tileset_name]
                .as_ref()
                .expect("pack tileset art");
            for y in 0..map.attributes.height * 2 {
                for x in 0..map.attributes.width * 2 {
                    for (px, py) in [(0, 0), (15, 15)] {
                        assert!(
                            art.sample(
                                &map.blocks,
                                usize::from(map.attributes.width),
                                TilePosition::new(x as i16, y as i16),
                                px,
                                py
                            )
                            .is_some(),
                            "{} ({x},{y})",
                            map.map_name
                        );
                    }
                }
            }
        }
        for (cols, rows) in [(40, 26), (45, 49), (80, 26)] {
            let buffer = painter.draw(&source, &text, cols, rows, None, false, true);
            let visible: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
            assert!(visible.contains("PACK"));
            assert!(visible.contains("SAVE"));
            assert!(visible.contains("TOUCH"));
            assert_eq!(buffer.content.len(), usize::from(cols * rows));
            assert!(
                buffer
                    .content
                    .iter()
                    .any(|cell| matches!(cell.symbol(), "·" | "•" | "●") && cell.fg != cell.bg)
            );
            assert!(
                !visible.contains('▀'),
                "overworld is ASCII illustration, not raster pixels"
            );
            let viewport = painter.viewport;
            let scene = painter.scene.unwrap();
            let fine = painter.detailed_world(&source).unwrap();
            assert_eq!(fine.area.width, scene.area.width * 2);
            assert_eq!(fine.area.height, scene.area.height * 4);
            assert_eq!(
                painter.viewport, viewport,
                "fine art has the same semantic map bounds"
            );
            assert!(
                fine.content
                    .iter()
                    .filter(|c| matches!(c.symbol(), "·" | "•" | "●"))
                    .count()
                    > 100
            );
            assert!(
                fine.content
                    .iter()
                    .all(|c| matches!(c.symbol(), " " | "·" | "•" | "●")),
                "No stray labels in the painted map"
            );
            assert!(
                fine.content.iter().all(|c| c.bg == PAPER),
                "Halftone is dots on flat paper, not pixel rectangles"
            );
            let mut hidden = source.clone();
            hidden.overworld_player_hidden = true;
            let without_player = painter.detailed_world(&hidden).unwrap();
            assert!(
                fine.content
                    .iter()
                    .zip(&without_player.content)
                    .filter(|(a, b)| a != b)
                    .count()
                    > 2,
                "The actual player sprite changes the painted scene at its authoritative tile"
            );
            painter.detailed_world(&source).unwrap();
            let sizes = painter.animated_dot_sizes();
            for _ in 0..20 {
                painter.advance_ink();
            }
            let after = painter.animated_dot_sizes();
            let visible_changes = sizes.iter().zip(&after)
                .filter(|(a, b)| **a > 0 && a.abs_diff(**b) >= 24).count();
            assert!(visible_changes > sizes.len() / 10,
                "Two seconds of gentle motion must change dot diameters, not just frame hashes");
        }
        for (cols, rows) in [(80, 24), (93, 34), (128, 43)] {
            let buffer = painter.draw(&source, &text, cols, rows, None, false, false);
            let viewport = painter.viewport;
            let area = painter.scene.unwrap().area;
            let terminal = painter.draw_native(&source, &text, cols, rows, None, false);
            let phase = painter.ink_phase;
            let image = painter.circle_image(u32::from(area.width) * 12, u32::from(area.height) * 24).unwrap();
            for _ in 0..20 { painter.advance_ink(); }
            let animated = painter.circle_image(image.width(), image.height()).unwrap();
            let pixel_change = image.pixels().zip(animated.pixels()).map(|(a,b)|
                (0..3).map(|c| a[c].abs_diff(b[c])).max().unwrap() as f64
            ).sum::<f64>() / f64::from(image.width() * image.height());
            assert!(pixel_change > 6., "Native circle motion must be visibly substantial: {pixel_change}");
            // Restore the phase for the existing unchanged-adapter comparison.
            painter.ink_phase = phase;
            assert_eq!(painter.viewport, viewport);
            assert!(
                terminal.content.iter().any(|cell| cell
                    .symbol()
                    .chars()
                    .any(|c| ('\u{2801}'..='\u{28ff}').contains(&c))),
                "Native terminal uses the shared high-resolution dot field"
            );
            for y in 0..rows {
                for x in 0..cols {
                    if x < area.x || x >= area.right() || y < area.y || y >= area.bottom() {
                        assert_eq!(
                            buffer[(x, y)],
                            terminal[(x, y)],
                            "Menus/dialogue are not compressed into dots"
                        );
                    }
                }
            }
            let browser_again = painter.draw(&source, &text, cols, rows, None, false, false);
            assert_eq!(
                buffer, browser_again,
                "Terminal encoding must not change approved browser artwork"
            );
            let visible: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
            assert!(visible.contains("PACK") && visible.contains("SAVE"));
            assert!(!visible.contains("TOUCH"));
            assert!(
                buffer
                    .content
                    .iter()
                    .any(|cell| matches!(cell.symbol(), "·" | "•" | "●") && cell.fg != cell.bg)
            );
        }
        assert_eq!(
            controller.presentation_snapshot().unwrap().state_checksum,
            checksum
        );
    }
}
