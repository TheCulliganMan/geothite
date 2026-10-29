//! Native text controls retained alongside the displayed glyphs.
use std::collections::BTreeMap;

use crate::core::systems::script_text::ScriptTextBody;

use super::{
    normalize_visible_script_text_with_context, render_named_text_buffer, render_script_text_args,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum PageTransition {
    #[default]
    Initial,
    Paragraph,
    Scroll,
    Prompt,
}

impl PageTransition {
    pub fn frames(self) -> u8 {
        match self {
            Self::Paragraph => 20,
            Self::Scroll => 10,
            Self::Initial | Self::Prompt => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TextEffect {
    Pause,
    Sound(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TextCue {
    pub after_chars: usize,
    pub effect: TextEffect,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct TextPage {
    pub text: String,
    pub transition: PageTransition,
    pub cues: Vec<TextCue>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct TextProgram {
    pub pages: Vec<TextPage>,
}

impl TextProgram {
    pub fn plain(pages: Vec<String>) -> Self {
        let mut result = Vec::<TextPage>::new();
        for text in pages {
            let transition = result.last().map_or(PageTransition::Initial, |previous| {
                if previous
                    .text
                    .lines()
                    .last()
                    .is_some_and(|line| !line.is_empty() && Some(line) == text.lines().next())
                {
                    PageTransition::Scroll
                } else {
                    PageTransition::Paragraph
                }
            });
            result.push(TextPage {
                text,
                transition,
                cues: Vec::new(),
            });
        }
        Self { pages: result }
    }
}

#[derive(Default)]
struct Builder {
    pages: Vec<TextPage>,
    lines: Vec<String>,
    current: String,
    cues: Vec<TextCue>,
    transition: PageTransition,
}

impl Builder {
    fn line(&mut self) {
        if !self.current.is_empty() {
            self.lines.push(std::mem::take(&mut self.current));
        }
    }

    fn page(&mut self, normalize: &impl Fn(&str) -> String) {
        self.line();
        if !self.lines.is_empty() || !self.cues.is_empty() {
            self.pages.push(TextPage {
                text: normalize(&self.lines.join("\n")),
                transition: self.transition,
                cues: std::mem::take(&mut self.cues),
            });
            self.lines.clear();
        }
    }

    fn effect(&mut self, effect: TextEffect, normalize: &impl Fn(&str) -> String) {
        let mut lines = self.lines.clone();
        if !self.current.is_empty() {
            lines.push(self.current.clone());
        }
        self.cues.push(TextCue {
            after_chars: normalize(&lines.join("\n")).chars().count(),
            effect,
        });
    }
}

pub(super) fn compile(
    body: &ScriptTextBody,
    buffers: &BTreeMap<String, String>,
    player: &str,
    rival: &str,
    day: u8,
) -> TextProgram {
    let normalize =
        |text: &str| normalize_visible_script_text_with_context(text, player, rival, day);
    let mut builder = Builder::default();
    for command in &body.commands {
        let rendered = match command.command.as_str() {
            "text_ram" | "text_decimal" => {
                render_named_text_buffer(command.args.first().map(String::as_str), buffers)
            }
            _ => render_script_text_args(&command.args),
        };
        match command.command.as_str() {
            "text" | "text_start" | "text_block" | "text_ram" | "text_decimal" => {
                builder.current.push_str(&rendered)
            }
            "text_today" => builder.current.push_str("<TODAY>"),
            "text_pause" => builder.effect(TextEffect::Pause, &normalize),
            "text_asm" | "text_far" => {}
            "text_low" => builder.line(),
            "line" | "next" => {
                builder.line();
                builder.current.push_str(&rendered);
            }
            "para" => {
                builder.page(&normalize);
                builder.transition = PageTransition::Paragraph;
                builder.current.push_str(&rendered);
            }
            "cont" => {
                builder.line();
                let carry = builder.lines.last().cloned();
                builder.page(&normalize);
                builder.transition = PageTransition::Scroll;
                if let Some(carry) = carry {
                    builder.lines.push(carry);
                }
                builder.current.push_str(&rendered);
            }
            "text_promptbutton" => {
                builder.page(&normalize);
                builder.transition = PageTransition::Prompt;
            }
            "prompt" | "done" | "text_end" => {
                builder.page(&normalize);
                break;
            }
            opcode if opcode.starts_with("sound_") => {
                if let Some(sound) = sound(opcode) {
                    builder.effect(TextEffect::Sound(sound), &normalize);
                }
            }
            _ => {
                builder.line();
                builder.lines.push(if rendered.is_empty() {
                    format!("[{}]", command.command)
                } else {
                    format!("[{} {rendered}]", command.command)
                });
            }
        }
    }
    builder.page(&normalize);
    if builder.pages.is_empty() {
        builder.pages.push(TextPage::default());
    }
    TextProgram {
        pages: builder.pages,
    }
}

fn sound(opcode: &str) -> Option<&'static str> {
    match opcode {
        "sound_dex_fanfare_50_79" => Some("SFX_DEX_FANFARE_50_79"),
        "sound_fanfare" => Some("SFX_FANFARE"),
        "sound_dex_fanfare_20_49" => Some("SFX_DEX_FANFARE_20_49"),
        "sound_item" => Some("SFX_ITEM"),
        "sound_caught_mon" => Some("SFX_CAUGHT_MON"),
        "sound_dex_fanfare_80_109" => Some("SFX_DEX_FANFARE_80_109"),
        "sound_slot_machine_start" => Some("SFX_SLOT_MACHINE_START"),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub(super) struct TextTiming {
    pub identity: String,
    pub label: String,
    pub program: TextProgram,
    pub page_index: usize,
    pub next_cue: usize,
    pub frames_remaining: u8,
    pub waiting_sound: bool,
    pub transition_active: bool,
}

impl TextTiming {
    pub fn new(identity: String, program: TextProgram, page_index: usize) -> Self {
        Self {
            identity,
            label: String::new(),
            program,
            page_index,
            next_cue: 0,
            frames_remaining: 0,
            waiting_sound: false,
            transition_active: false,
        }
    }

    pub fn enter(&mut self, page_index: usize) {
        self.page_index = page_index;
        self.next_cue = 0;
        self.frames_remaining = self
            .program
            .pages
            .get(page_index)
            .map_or(0, |page| page.transition.frames());
        self.waiting_sound = false;
        self.transition_active = self.frames_remaining > 0;
    }

    pub fn complete(&self) -> bool {
        self.frames_remaining == 0
            && !self.waiting_sound
            && self
                .program
                .pages
                .get(self.page_index)
                .is_none_or(|page| self.next_cue >= page.cues.len())
    }
}
