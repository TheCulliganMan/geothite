//! Source radio textbox primitives. Program selection and VBlank/CPU scheduling
//! belong to the caller; these operations model PrintRadioLine and RadioScroll.
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

mod encoding;
pub use encoding::{encode_radio_pokedex_entry, encode_radio_string, encode_radio_text_body};

pub const RADIO_SCROLL: u8 = 84;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RadioTextError {
    #[error("Pokédex entry {0} does not preserve the source radio line layout")]
    PokedexEntry(String),
    #[error("unsupported radio source character {character:?} at byte {offset}")]
    Encoding { offset: usize, character: char },
    #[error("radio text {label} command {index}: {detail}")]
    Body {
        label: String,
        index: usize,
        detail: String,
    },
    #[error("radio RAM symbol {0} has no binding")]
    MissingRamSymbol(String),
    #[error("radio text ends before its source terminator")]
    Truncated,
    #[error("unsupported radio text command {0:#04x}")]
    Command(u8),
    #[error("unsupported radio string control {0:#04x}")]
    Control(u8),
    #[error("radio text writes outside its textbox at {0}")]
    Position(usize),
    #[error("radio RAM text at {0:#06x} is unavailable")]
    Ram(u16),
    #[error("radio weekday {0} is outside 0..7")]
    Weekday(u8),
}

pub trait RadioTextEnvironment {
    /// Return the source string, including its @ byte ($50).
    fn ram_text(&self, address: u16) -> Result<Vec<u8>, RadioTextError>;
    fn weekday(&self) -> u8;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RadioTextWindow {
    /// Full tile rows 12..17, including the source textbox border.
    pub tiles: [[u8; 20]; 6],
}

impl RadioTextWindow {
    pub fn scroll(&mut self, printed: u8) {
        // CopyBottomLineToTopLine includes the side borders and blank row.
        if printed != 1 {
            self.tiles[1] = self.tiles[3];
            self.tiles[2] = self.tiles[4];
        }
        self.tiles[3][1..19].fill(0x7f);
        self.tiles[4][1..19].fill(0x7f);
    }

    fn write(&mut self, position: &mut usize, tile: u8) -> Result<(), RadioTextError> {
        let row = self
            .tiles
            .get_mut(*position / 20)
            .ok_or(RadioTextError::Position(*position))?;
        row[*position % 20] = tile;
        *position += 1;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RadioScrollState {
    pub current_line: u8,
    pub next_line: u8,
    pub delay: u8,
    pub printed: u8,
}

impl RadioScrollState {
    pub fn finish_print(&mut self) {
        self.current_line = RADIO_SCROLL;
        self.delay = 100;
    }

    /// One source RadioScroll call. The call which decrements 1 to 0 does not
    /// transition; the following call changes the line and scrolls the window.
    pub fn step(&mut self, window: &mut RadioTextWindow) {
        if self.delay != 0 {
            self.delay -= 1;
            return;
        }
        self.current_line = self.next_line;
        window.scroll(self.printed);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RadioLinePrinter {
    text: Vec<u8>,
    offset: usize,
    cursor: usize,
    string_origin: usize,
    in_string: bool,
    expansion: VecDeque<u8>,
    ram_string: Option<VecDeque<u8>>,
    wait_frames: u8,
    done: bool,
}

impl RadioLinePrinter {
    /// `text` is the source wRadioText buffer, before PrintRadioLine mutates it.
    pub fn new(
        text: &[u8],
        state: &mut RadioScrollState,
        next_line: u8,
    ) -> Result<Self, RadioTextError> {
        let mut text = text.to_vec();
        let mut offset = 0;
        let mut cursor = 41; // (1,14), relative to textbox row 12.
        if state.printed < 2 {
            *text.get_mut(1).ok_or(RadioTextError::Truncated)? = 0; // TX_START
            offset = 1;
            state.printed += 1;
            if state.printed == 2 {
                cursor = 81;
            } // (1,16)
        }
        state.next_line = next_line;
        Ok(Self {
            text,
            offset,
            cursor,
            string_origin: cursor,
            in_string: false,
            expansion: VecDeque::new(),
            ram_string: None,
            wait_frames: 0,
            done: false,
        })
    }

    /// PrintText/PlaceString at an explicit textbox coordinate, without
    /// PrintRadioLine's prefix mutation or printed-line counter changes.
    pub fn text_at(text: &[u8], column: u8, row: u8) -> Result<Self, RadioTextError> {
        if column >= 20 || !(12..18).contains(&row) {
            return Err(RadioTextError::Position(
                usize::from(row) * 20 + usize::from(column),
            ));
        }
        let cursor = usize::from(row - 12) * 20 + usize::from(column);
        Ok(Self {
            text: text.to_vec(),
            offset: 0,
            cursor,
            string_origin: cursor,
            in_string: false,
            expansion: VecDeque::new(),
            ram_string: None,
            wait_frames: 0,
            done: false,
        })
    }

    pub fn is_done(&self) -> bool {
        self.done
    }
    pub fn wait_frames(&self) -> u8 {
        self.wait_frames
    }

    fn byte(&mut self) -> Result<u8, RadioTextError> {
        let byte = *self
            .text
            .get(self.offset)
            .ok_or(RadioTextError::Truncated)?;
        self.offset += 1;
        Ok(byte)
    }

    fn expand_word(&mut self, word: &str) {
        self.expansion.extend(word.bytes().map(|byte| match byte {
            b'A'..=b'Z' => byte - b'A' + 0x80,
            _ => unreachable!("source command expansion contains only uppercase ASCII"),
        }));
    }

    /// Execute until a source pause or terminator. Call once per frame while
    /// paused. Held A/B is sampled when TextCommand_PAUSE is encountered, not
    /// during its ensuing DelayFrames. This does not model CPU execution time.
    pub fn advance_frame(
        &mut self,
        window: &mut RadioTextWindow,
        environment: &impl RadioTextEnvironment,
        held_ab: bool,
    ) -> Result<bool, RadioTextError> {
        if self.done {
            return Ok(true);
        }
        if self.wait_frames != 0 {
            self.wait_frames -= 1;
            if self.wait_frames != 0 {
                return Ok(false);
            }
        }
        loop {
            if let Some(tile) = self.expansion.pop_front() {
                window.write(&mut self.cursor, tile)?;
                continue;
            }
            let from_ram = self.ram_string.is_some();
            let byte = if let Some(ram) = self.ram_string.as_mut() {
                ram.pop_front().ok_or(RadioTextError::Truncated)?
            } else {
                self.byte()?
            };
            if !self.in_string && !from_ram {
                match byte {
                    0 => {
                        self.in_string = true;
                        self.string_origin = self.cursor;
                    }
                    1 => {
                        let address = u16::from_le_bytes([self.byte()?, self.byte()?]);
                        let text = environment.ram_text(address)?;
                        let end = text
                            .iter()
                            .position(|byte| *byte == 0x50)
                            .ok_or(RadioTextError::Truncated)?;
                        self.ram_string = Some(text[..=end].iter().copied().collect());
                        self.string_origin = self.cursor;
                    }
                    0x0a => {
                        if !held_ab {
                            self.wait_frames = 30;
                            return Ok(false);
                        }
                    }
                    0x15 => {
                        let day = environment.weekday();
                        let word = [
                            "SUNDAY",
                            "MONDAY",
                            "TUESDAY",
                            "WEDNESDAY",
                            "THURSDAY",
                            "FRIDAY",
                            "SATURDAY",
                        ]
                        .get(usize::from(day))
                        .ok_or(RadioTextError::Weekday(day))?;
                        self.expand_word(word);
                    }
                    0x50 => {
                        self.done = true;
                        return Ok(true);
                    }
                    other => return Err(RadioTextError::Command(other)),
                }
                continue;
            }
            match byte {
                0 => window.write(&mut self.cursor, 0xe6)?, // source NullChar writes '?'
                0x1f => window.write(&mut self.cursor, 0x7f)?,
                0x25 => {} // word-break opportunity
                0x22 | 0x4e => {
                    self.string_origin += if byte == 0x22 { 20 } else { 40 };
                    self.cursor = self.string_origin;
                }
                0x4f => {
                    self.cursor = 81;
                    self.string_origin = 81;
                }
                0x50 => {
                    if from_ram {
                        self.ram_string = None;
                    } else {
                        self.in_string = false;
                    }
                }
                0x57 => {
                    self.done = true;
                    return Ok(true);
                }
                0x54 => {
                    self.expand_word("POK");
                    self.expansion.push_back(0xea);
                }
                0x24 => self.expansion.extend([0x70, 0x71]),
                0x4a => self.expansion.extend([0xe1, 0xe2]),
                0x56 => self.expansion.extend([0x75, 0x75]),
                0x5b => self.expand_word("PC"),
                0x5c => self.expand_word("TM"),
                0x5d => self.expand_word("TRAINER"),
                0x5e => self.expand_word("ROCKET"),
                0x60..=0xff => window.write(&mut self.cursor, byte)?,
                other => return Err(RadioTextError::Control(other)),
            }
        }
    }
}
