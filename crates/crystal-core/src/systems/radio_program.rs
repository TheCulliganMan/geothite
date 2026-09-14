//! Native radio program transitions. Effects are returned in source call order;
//! the owner executes them before passing the text to the radio printer.
use thiserror::Error;
mod buena;
mod oak;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioProgramContext<'a> {
    pub buena: Option<&'a RadioBuenaContext>,
    pub oak: Option<&'a RadioOakContext<'a>>,
    pub printed: u8,
    pub weekday: u8,
    pub caught_pokemon: Option<&'a [bool; 251]>,
    pub people_places: Option<&'a RadioPeoplePlacesContext<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioBuenaContext {
    /// Updated clock hour at BuenasPasswordCheckTime.
    pub hour: u8,
    pub password: u8,
    pub password_generated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioOakContext<'a> {
    pub segment_counter: u8,
    pub delay: u8,
    /// OaksPKMNTalkRoutes order; whether each map exists in JohtoGrassWildMons.
    pub grass_routes: &'a [bool],
}

/// Source table inputs and progression bytes used by PeoplePlaces4/6.
/// Exclusion slices contain trainer IDs before the shared -1 terminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioPeoplePlacesContext<'a> {
    pub hall_of_fame: bool,
    pub kanto_badges: u8,
    pub trainer_class_count: u8,
    pub place_count: u8,
    pub hidden_people: &'a [u8],
    pub hidden_people_beat_e4: &'a [u8],
    pub hidden_people_beat_kanto: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadioProgramEffect {
    ClearTextbox,
    NoRadioMusic,
    NoRadioName,
    StoreBuenaPassword(u8),
    SetBuenaPasswordGenerated(bool),
    FormatBuenaPassword(u8),
    SetOakSegmentCounter(u8),
    FormatOakEncounter {
        route_index: u8,
        time_of_day: u8,
        slot: u8,
    },
    FormatCurrentSpecies,
    RestartPokemonChannelMusic,
    WaitBgMap,
    PrintTextboxLabel(&'static str),
    SetCurrentLine(u8),
    SetNextLine(u8),
    SetRadioDelay(u8),
    PlaceRadioString {
        column: u8,
        row: u8,
        text: &'static str,
    },
    /// RadioMusicRestartDE: stop music, update radio/map music IDs, then start this song.
    RestartMusic(&'static str),
    CheckAndResetLuckyNumberShow,
    /// Source LuckyNumberShow8 writes five zero-padded digits and @ to wStringBuffer1.
    FormatLuckyNumber,
    /// GetPokemonName after the zero-based caught-bit selection.
    FormatPokedexSpecies {
        species: u8,
    },
    /// GetTrainerClassName -> wStringBuffer2; trainer #1 name -> wStringBuffer1.
    FormatPeoplePlacesTrainer {
        class_id: u8,
        trainer_id: u8,
    },
    /// Resolve PnP_Places[index] through GetWorldMapLocation/GetLandmarkName.
    FormatPeoplePlacesLandmark {
        index: u8,
    },
    SetPrintedLines(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadioProgramTextSource {
    Label(&'static str),
    RetainedBuffer,
    PokedexCategory,
    PokedexEntryLine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioProgramText {
    pub source: RadioProgramTextSource,
    pub next_line: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioProgramStep {
    pub effects: Vec<RadioProgramEffect>,
    pub text: Option<RadioProgramText>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RadioProgramError {
    #[error("radio Random: {0}")]
    Random(String),
    #[error("radio program line {0} has not been implemented")]
    UnimplementedLine(u8),
    #[error("radio weekday {0} is outside 0..7")]
    Weekday(u8),
    #[error("radio selection data is missing or invalid: {0}")]
    SelectionData(&'static str),
}

pub fn radio_music_channel_song(weekday: u8) -> Result<&'static str, RadioProgramError> {
    if weekday >= 7 {
        return Err(RadioProgramError::Weekday(weekday));
    }
    Ok(if weekday & 1 == 0 {
        "MUSIC_POKEMON_MARCH"
    } else {
        "MUSIC_POKEMON_LULLABY"
    })
}

fn start(effects: &mut Vec<RadioProgramEffect>, song: &'static str) {
    effects.push(RadioProgramEffect::ClearTextbox);
    effects.push(RadioProgramEffect::RestartMusic(song));
}

/// One source radio program call; the caller owns printing and scrolling.
/// The random callback receives the CPU carry input and returns the result
/// of the source Random routine, not a seed.
pub fn radio_program_step(
    line: u8,
    context: RadioProgramContext<'_>,
    random: &mut impl FnMut(bool) -> Result<u8, RadioProgramError>,
) -> Result<RadioProgramStep, RadioProgramError> {
    if matches!(line, 4 | 64..=83) {
        return buena::step(line, context, random);
    }
    if matches!(line, 0 | 11..=18 | 59..=63) {
        return oak::step(line, context, random);
    }
    let mut effects = Vec::new();
    let (label, next_line) = match line {
        1 => {
            let caught = context
                .caught_pokemon
                .ok_or(RadioProgramError::SelectionData("Pokédex caught flags"))?;
            if !caught.iter().any(|caught| *caught) {
                return Err(RadioProgramError::SelectionData(
                    "no caught Pokémon for Pokédex Show",
                ));
            }
            if context.printed == 0 {
                start(&mut effects, "MUSIC_POKEMON_CENTER");
            }
            let species = loop {
                let index = random(false)?;
                if index < 251 && caught[usize::from(index)] {
                    break index + 1;
                }
            };
            effects.push(RadioProgramEffect::FormatPokedexSpecies { species });
            ("_PokedexShowText", 19)
        }
        19 | 20 | 21 | 22 | 85 | 86 | 87 => {
            let next_line = match line {
                19 => 20,
                20 => 21,
                21 => 22,
                22 => 85,
                85 => 86,
                86 => 87,
                87 => 1,
                _ => unreachable!(),
            };
            return Ok(RadioProgramStep {
                effects,
                text: Some(RadioProgramText {
                    source: if line == 19 {
                        RadioProgramTextSource::PokedexCategory
                    } else {
                        RadioProgramTextSource::PokedexEntryLine
                    },
                    next_line,
                }),
            });
        }
        5 => {
            if context.printed == 0 {
                start(&mut effects, "MUSIC_VIRIDIAN_CITY");
            }
            ("_PnP_Text1", 44)
        }
        44 => ("_PnP_Text2", 45),
        45 => ("_PnP_Text3", if random(false)? < 123 { 46 } else { 48 }),
        46 => {
            let data = context
                .people_places
                .ok_or(RadioProgramError::SelectionData("Places & People"))?;
            if data.trainer_class_count < 2 {
                return Err(RadioProgramError::SelectionData("trainer class count"));
            }
            let hidden = if !data.hall_of_fame {
                data.hidden_people
            } else if data.kanto_badges != 0xff {
                data.hidden_people_beat_e4
            } else {
                data.hidden_people_beat_kanto
            };
            if (1..data.trainer_class_count).all(|id| hidden.contains(&id)) {
                return Err(RadioProgramError::SelectionData(
                    "all trainer classes excluded",
                ));
            }
            let mask = (u16::from(data.trainer_class_count).next_power_of_two() - 1) as u8;
            let mut carry = false;
            let class_id = loop {
                let candidate = (random(carry)? & mask).wrapping_add(1);
                if candidate >= data.trainer_class_count {
                    carry = false;
                    continue;
                }
                carry = hidden.contains(&candidate);
                if !carry {
                    break candidate;
                }
            };
            effects.push(RadioProgramEffect::FormatPeoplePlacesTrainer {
                class_id,
                trainer_id: 1,
            });
            ("_PnP_Text4", 47)
        }
        48 => {
            let data = context
                .people_places
                .ok_or(RadioProgramError::SelectionData("Places & People"))?;
            if data.place_count == 0 {
                return Err(RadioProgramError::SelectionData("place count"));
            }
            let index = loop {
                let candidate = random(false)?;
                if candidate < data.place_count {
                    break candidate;
                }
            };
            effects.push(RadioProgramEffect::FormatPeoplePlacesLandmark { index });
            ("_PnP_Text5", 49)
        }
        47 | 49 => {
            // Both source adjective tables have this order. Each branch
            // consumes its own Random call, including the optional third draw.
            const ADJECTIVES: [&str; 16] = [
                "_PnP_CuteText",
                "_PnP_LazyText",
                "_PnP_HappyText",
                "_PnP_NoisyText",
                "_PnP_PrecociousText",
                "_PnP_BoldText",
                "_PnP_PickyText",
                "_PnP_SortOfOKText",
                "_PnP_SoSoText",
                "_PnP_GreatText",
                "_PnP_MyTypeText",
                "_PnP_CoolText",
                "_PnP_InspiringText",
                "_PnP_WeirdText",
                "_PnP_RightForMeText",
                "_PnP_OddText",
            ];
            let label = ADJECTIVES[usize::from(random(false)? & 15)];
            let next = if random(false)? < 10 {
                5
            } else if random(false)? < 123 {
                46
            } else {
                48
            };
            (label, next)
        }
        7 => {
            if context.printed == 0 {
                start(&mut effects, "MUSIC_ROCKET_OVERTURE");
            }
            ("_RocketRadioText1", 50)
        }
        50 => ("_RocketRadioText2", 51),
        51 => ("_RocketRadioText3", 52),
        52 => ("_RocketRadioText4", 53),
        53 => ("_RocketRadioText5", 54),
        54 => ("_RocketRadioText6", 55),
        55 => ("_RocketRadioText7", 56),
        56 => ("_RocketRadioText8", 57),
        57 => ("_RocketRadioText9", 58),
        58 => ("_RocketRadioText10", 7),
        3 => {
            if context.printed == 0 {
                start(&mut effects, "MUSIC_GAME_CORNER");
            }
            effects.push(RadioProgramEffect::CheckAndResetLuckyNumberShow);
            ("_LC_Text1", 30)
        }
        30 => ("_LC_Text2", 31),
        31 => ("_LC_Text3", 32),
        32 => ("_LC_Text4", 33),
        33 => ("_LC_Text5", 34),
        34 => ("_LC_Text6", 35),
        35 => ("_LC_Text7", 36),
        36 => {
            effects.push(RadioProgramEffect::FormatLuckyNumber);
            ("_LC_Text8", 37)
        }
        37 => ("_LC_Text9", 38),
        38 => ("_LC_Text7", 39),
        39 => ("_LC_Text8", 40), // The source reuses the already formatted number.
        40 => ("_LC_Text10", 41),
        41 => ("_LC_Text11", if random(false)? == 0 { 42 } else { 3 }),
        42 => ("_LC_DragText1", 43),
        43 => ("_LC_DragText2", 3),
        2 | 6 => {
            // Both use StartPokemonMusicChannel, not RadioChannelSongs.
            start(&mut effects, radio_music_channel_song(context.weekday)?);
            if line == 2 {
                ("_BenIntroText1", 23)
            } else {
                ("_FernIntroText1", 29)
            }
        }
        23 => ("_BenIntroText2", 24),
        24 => ("_BenIntroText3", 25),
        29 => ("_FernIntroText2", 25),
        25 => ("_BenFernText1", 26),
        26 | 27 => {
            radio_music_channel_song(context.weekday)?;
            match (line, context.weekday & 1) {
                (26, 0) => ("_BenFernText2A", 27),
                (26, _) => ("_BenFernText2B", 27),
                (27, 0) => ("_BenFernText3A", 28),
                (27, _) => ("_BenFernText3B", 28),
                _ => unreachable!(),
            }
        }
        28 => {
            return Ok(RadioProgramStep {
                effects,
                text: None,
            });
        }
        8..=10 => {
            if context.printed == 0 {
                start(
                    &mut effects,
                    match line {
                        8 => "MUSIC_POKE_FLUTE_CHANNEL",
                        9 => "MUSIC_RUINS_OF_ALPH_RADIO",
                        10 => "MUSIC_LAKE_OF_RAGE_ROCKET_RADIO",
                        _ => unreachable!(),
                    },
                );
            }
            effects.push(RadioProgramEffect::SetPrintedLines(1));
            return Ok(RadioProgramStep {
                effects,
                text: None,
            });
        }
        _ => return Err(RadioProgramError::UnimplementedLine(line)),
    };
    Ok(RadioProgramStep {
        effects,
        text: Some(RadioProgramText {
            source: RadioProgramTextSource::Label(label),
            next_line,
        }),
    })
}

/// CopyDexEntryPart1/2 retain the source byte boundaries rather than wrapping
/// a flattened description. The owner resets this cursor for each species.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RadioPokedexEntryCursor {
    pub offset: usize,
}

impl RadioPokedexEntryCursor {
    pub fn copy_text(
        &mut self,
        entry: &[u8],
        category: bool,
    ) -> Result<Vec<u8>, RadioProgramError> {
        if category && self.offset != 0 {
            return Err(RadioProgramError::SelectionData("Pokédex category cursor"));
        }
        let remaining = entry
            .get(self.offset..)
            .ok_or(RadioProgramError::SelectionData("Pokédex entry pointer"))?;
        let length = remaining
            .iter()
            .take(19)
            .position(|byte| matches!(byte, 0x50 | 0x4e | 0x5f))
            .ok_or(RadioProgramError::SelectionData("Pokédex line terminator"))?;
        let mut text = vec![0, 0x4f]; // TX_START, <LINE>
        text.extend_from_slice(&remaining[..length]);
        text.push(0x57); // Replace the source delimiter with <DONE>.
        let next = self.offset + length + 1 + if category { 4 } else { 0 };
        if next > entry.len() {
            return Err(RadioProgramError::SelectionData(
                "Pokédex height and weight",
            ));
        }
        self.offset = next;
        Ok(text)
    }
}
