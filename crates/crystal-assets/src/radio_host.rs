//! Runtime effects for the source radio program interpreter.
use crate::{GameDataSet, radio_text_memory::RadioTextMemory};
use crystal_core::{
    random::DividerSource,
    state::GameState,
    systems::{
        radio_playback::{RadioPlaybackError, RadioPlaybackHost},
        radio_program::{RadioProgramEffect, RadioProgramTextSource},
        radio_text::{
            RadioLinePrinter, RadioTextEnvironment, RadioTextError, RadioTextWindow,
            encode_radio_string,
        },
        special_routines::{
            SpecialRoutineEffect, apply_random_special_routine, apply_special_routine,
        },
    },
};

/// These distinguish the three source music routines. The presentation owner
/// executes them in order, including MUSIC_NONE before either restart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadioMusicEffect {
    Stop,
    Restart(&'static str),
    PokemonChannel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioHostState {
    pub memory: RadioTextMemory,
    pub oak_segment_counter: u8,
    /// An override for the three name rows (8..10), set by NoRadioName or Buena.
    pub name_tiles: Option<[[u8; 18]; 3]>,
    pub music: Vec<RadioMusicEffect>,
    pub music_mode: Option<RadioMusicEffect>,
    /// Invalidates the game snapshot only when this frame writes cartridge state.
    pub game_state_changed: bool,
    bg_map_wait: Option<u8>,
    textbox_printer: Option<RadioLinePrinter>,
}

impl RadioHostState {
    pub fn new(weekday: u8) -> anyhow::Result<Self> {
        Ok(Self {
            memory: RadioTextMemory::new(weekday)?,
            oak_segment_counter: 0,
            name_tiles: None,
            music: Vec::new(),
            music_mode: None,
            game_state_changed: false,
            bg_map_wait: None,
            textbox_printer: None,
        })
    }

    fn queue_music(&mut self, effect: RadioMusicEffect) {
        self.music_mode = Some(effect.clone());
        self.music.push(effect);
    }
}

pub struct RadioHost<'a, S: DividerSource + ?Sized> {
    pub data: &'a GameDataSet,
    pub game: &'a mut GameState,
    pub divider: &'a mut S,
    pub state: &'a mut RadioHostState,
    pub held_ab: bool,
}

fn host_error(error: impl std::fmt::Display) -> RadioPlaybackError {
    RadioPlaybackError::Host(error.to_string())
}

/// Textbox at (0,12), interior dimensions 18 by 4.
pub fn radio_textbox() -> RadioTextWindow {
    let mut tiles = [[0x7f; 20]; 6];
    tiles[0].fill(0x7a);
    tiles[5].fill(0x7a);
    tiles[0][0] = 0x79;
    tiles[0][19] = 0x7b;
    tiles[5][0] = 0x7d;
    tiles[5][19] = 0x7e;
    for row in &mut tiles[1..5] {
        row[0] = 0x7c;
        row[19] = 0x7c;
    }
    RadioTextWindow { tiles }
}

impl<S: DividerSource + ?Sized> RadioTextEnvironment for RadioHost<'_, S> {
    fn ram_text(&self, address: u16) -> Result<Vec<u8>, RadioTextError> {
        self.state.memory.ram_text(address)
    }
    fn weekday(&self) -> u8 {
        self.state.memory.weekday()
    }
}

impl<S: DividerSource + ?Sized> RadioPlaybackHost for RadioHost<'_, S>
where
    S::Error: std::fmt::Display,
{
    fn effect(
        &mut self,
        effect: &RadioProgramEffect,
        window: &mut RadioTextWindow,
    ) -> Result<bool, RadioPlaybackError> {
        use RadioProgramEffect::*;
        match effect {
            ClearTextbox => *window = radio_textbox(),
            NoRadioName => {
                self.state.name_tiles = Some([[0x7f; 18]; 3]);
                *window = radio_textbox();
            }
            NoRadioMusic => self.state.queue_music(RadioMusicEffect::Stop),
            RestartMusic(song) => self.state.queue_music(RadioMusicEffect::Restart(song)),
            RestartPokemonChannelMusic => self.state.queue_music(RadioMusicEffect::PokemonChannel),
            StoreBuenaPassword(password) => {
                self.state.game_state_changed = true;
                self.game.buenas_password.category_index = usize::from(password >> 4);
                self.game.buenas_password.option_index = usize::from(password & 15);
            }
            SetBuenaPasswordGenerated(value) => {
                self.game
                    .flags
                    .set_engine_flag("ENGINE_BUENAS_PASSWORD", *value)
                    .map_err(host_error)?;
                self.state.game_state_changed = true;
            }
            FormatBuenaPassword(password) => self
                .state
                .memory
                .format_buena_password(self.data, *password)
                .map_err(host_error)?,
            SetOakSegmentCounter(value) => self.state.oak_segment_counter = *value,
            FormatOakEncounter {
                route_index,
                time_of_day,
                slot,
            } => self
                .state
                .memory
                .format_oak_encounter(self.data, *route_index, *time_of_day, *slot)
                .map_err(host_error)?,
            FormatCurrentSpecies => self
                .state
                .memory
                .format_current_species(self.data)
                .map_err(host_error)?,
            FormatPokedexSpecies { species } => self
                .state
                .memory
                .select_pokedex_species(self.data, *species)
                .map_err(host_error)?,
            FormatPeoplePlacesTrainer {
                class_id,
                trainer_id,
            } => self
                .state
                .memory
                .format_source_trainer(self.data, *class_id, *trainer_id)
                .map_err(host_error)?,
            FormatPeoplePlacesLandmark { index } => self
                .state
                .memory
                .format_source_place(self.data, *index)
                .map_err(host_error)?,
            FormatLuckyNumber => self
                .state
                .memory
                .format_lucky_number(self.game.lucky_id_number)
                .map_err(host_error)?,
            CheckAndResetLuckyNumberShow => {
                self.state.game_state_changed = true;
                let result =
                    apply_special_routine(self.game, &self.data.moves, "CheckLuckyNumberShowFlag")
                        .map_err(host_error)?;
                let SpecialRoutineEffect::CheckLuckyNumberShowFlag { flag } = result.effect else {
                    return Err(host_error(
                        "CheckLuckyNumberShowFlag returned a different routine effect",
                    ));
                };
                if flag {
                    apply_random_special_routine(
                        self.game,
                        &self.data.moves,
                        "ResetLuckyNumberShowFlag",
                        self.divider,
                    )
                    .map_err(host_error)?;
                }
            }
            WaitBgMap => {
                let remaining = self.state.bg_map_wait.get_or_insert(4);
                if *remaining != 0 {
                    *remaining -= 1;
                    return Ok(false);
                }
                self.state.bg_map_wait = None;
            }
            PrintTextboxLabel(label) => {
                if self.state.textbox_printer.is_none() {
                    let text = self
                        .state
                        .memory
                        .label_text(self.data, label)
                        .map_err(host_error)?;
                    self.state.textbox_printer = Some(RadioLinePrinter::text_at(&text, 1, 14)?);
                }
                let printer = self
                    .state
                    .textbox_printer
                    .as_mut()
                    .expect("textbox printer initialized above");
                if !printer.advance_frame(window, &self.state.memory, self.held_ab)? {
                    return Ok(false);
                }
                self.state.textbox_printer = None;
            }
            PlaceRadioString { column, row, text } => {
                let mut commands = vec![0];
                commands.extend(encode_radio_string(text)?);
                commands.push(0x50);
                if *row >= 12 {
                    RadioLinePrinter::text_at(&commands, *column, *row)?.advance_frame(
                        window,
                        &self.state.memory,
                        self.held_ab,
                    )?;
                } else {
                    // The only source use outside the textbox is Buena's
                    // station name. Use the same PlaceString expansion.
                    if !(8..11).contains(row) || !(1..19).contains(column) {
                        return Err(host_error(
                            "radio name coordinate is outside the source name area",
                        ));
                    }
                    let mut name = RadioTextWindow {
                        tiles: [[0x7f; 20]; 6],
                    };
                    if let Some(rows) = self.state.name_tiles {
                        for (target, source) in name.tiles.iter_mut().zip(rows) {
                            target[1..19].copy_from_slice(&source);
                        }
                    }
                    RadioLinePrinter::text_at(&commands, *column, row + 4)?.advance_frame(
                        &mut name,
                        &self.state.memory,
                        self.held_ab,
                    )?;
                    self.state.name_tiles = Some(std::array::from_fn(|row| {
                        name.tiles[row][1..19].try_into().expect("18 columns")
                    }));
                }
            }
            SetCurrentLine(_) | SetNextLine(_) | SetRadioDelay(_) | SetPrintedLines(_) => {
                return Err(host_error("radio control effect belongs to RadioPlayback"));
            }
        }
        Ok(true)
    }

    fn text(&mut self, source: &RadioProgramTextSource) -> Result<Vec<u8>, RadioPlaybackError> {
        self.state
            .memory
            .text(self.data, source)
            .map_err(host_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_core::random::ReplayDivider;

    #[test]
    fn radio_host_preserves_source_wait_name_and_music_effect_order() {
        let data = GameDataSet::default();
        let mut game = GameState::default();
        let mut divider = ReplayDivider::new([]);
        let mut state = RadioHostState::new(0).unwrap();
        let mut host = RadioHost {
            data: &data,
            game: &mut game,
            divider: &mut divider,
            state: &mut state,
            held_ab: false,
        };
        let mut window = RadioTextWindow {
            tiles: [[0; 20]; 6],
        };
        for _ in 0..4 {
            assert!(
                !host
                    .effect(&RadioProgramEffect::WaitBgMap, &mut window)
                    .unwrap()
            );
        }
        assert!(
            host.effect(&RadioProgramEffect::WaitBgMap, &mut window)
                .unwrap()
        );
        host.effect(&RadioProgramEffect::NoRadioName, &mut window)
            .unwrap();
        assert_eq!(window, radio_textbox());
        host.effect(
            &RadioProgramEffect::PlaceRadioString {
                column: 2,
                row: 9,
                text: "BUENA'S PASSWORD@",
            },
            &mut window,
        )
        .unwrap();
        let mut name = [[0x7f; 18]; 3];
        let encoded = encode_radio_string("BUENA'S PASSWORD").unwrap();
        name[1][1..1 + encoded.len()].copy_from_slice(&encoded);
        assert_eq!(host.state.name_tiles, Some(name));
        for effect in [
            RadioProgramEffect::NoRadioMusic,
            RadioProgramEffect::RestartPokemonChannelMusic,
            RadioProgramEffect::RestartMusic("MUSIC_POKEMON_TALK"),
        ] {
            host.effect(&effect, &mut window).unwrap();
        }
        assert_eq!(
            host.state.music,
            [
                RadioMusicEffect::Stop,
                RadioMusicEffect::PokemonChannel,
                RadioMusicEffect::Restart("MUSIC_POKEMON_TALK")
            ]
        );
        assert_eq!(divider.consumed(), 0);
    }

    #[test]
    fn radio_host_lucky_check_uses_source_script_result_and_timer() {
        let data = GameDataSet::default();
        let mut game = GameState::default();
        game.time.current_day = 5; // Friday must restart at seven days.
        game.flags
            .set_engine_flag("ENGINE_LUCKY_NUMBER_SHOW", true)
            .unwrap();
        let mut divider = ReplayDivider::new([1, 2, 3, 4]);
        let mut state = RadioHostState::new(5).unwrap();
        let mut host = RadioHost {
            data: &data,
            game: &mut game,
            divider: &mut divider,
            state: &mut state,
            held_ab: false,
        };
        let mut window = radio_textbox();
        host.effect(
            &RadioProgramEffect::CheckAndResetLuckyNumberShow,
            &mut window,
        )
        .unwrap();
        assert_eq!(host.game.lucky_number_countdown.remaining_days, 7);
        assert_eq!(host.game.script_runtime.script_value.as_deref(), Some("1"));
        assert!(
            !host
                .game
                .flags
                .is_engine_flag_set("ENGINE_LUCKY_NUMBER_SHOW")
                .unwrap()
        );
        assert_eq!(host.divider.consumed(), 4);
        host.effect(&RadioProgramEffect::FormatLuckyNumber, &mut window)
            .unwrap();
        assert_eq!(
            host.state
                .memory
                .ram_text(crate::radio_text_memory::RADIO_STRING_1)
                .unwrap(),
            encode_radio_string(&format!("{:05}@", host.game.lucky_id_number)).unwrap()
        );
        host.effect(
            &RadioProgramEffect::CheckAndResetLuckyNumberShow,
            &mut window,
        )
        .unwrap();
        assert_eq!(host.game.script_runtime.script_value.as_deref(), Some("0"));
        assert_eq!(
            host.divider.consumed(),
            4,
            "an unexpired show does not sample Random again"
        );
    }
}
