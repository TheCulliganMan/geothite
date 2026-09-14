use super::*;

pub(super) fn step(
    line: u8,
    context: RadioProgramContext<'_>,
    random: &mut impl FnMut(bool) -> Result<u8, RadioProgramError>,
) -> Result<RadioProgramStep, RadioProgramError> {
    let mut effects = Vec::new();
    let (source, next_line) = match line {
        0 => {
            effects.push(RadioProgramEffect::SetOakSegmentCounter(5));
            if context.printed == 0 {
                start(&mut effects, "MUSIC_POKEMON_TALK");
            }
            (RadioProgramTextSource::Label("_OPT_IntroText1"), 11)
        }
        11 => (RadioProgramTextSource::Label("_OPT_IntroText2"), 12),
        12 => (RadioProgramTextSource::Label("_OPT_IntroText3"), 13),
        13 => {
            let data = context
                .oak
                .ok_or(RadioProgramError::SelectionData("Oak route data"))?;
            if data.grass_routes.is_empty() || data.grass_routes.len() > 32 {
                return Err(RadioProgramError::SelectionData("Oak route count"));
            }
            let route_index = loop {
                let index = random(false)? & 31;
                if usize::from(index) < data.grass_routes.len() {
                    break index;
                }
            };
            if !data.grass_routes[usize::from(route_index)] {
                // The source overflow path prints the previous radio buffer
                // and returns to the intro; it does not sample another route.
                (RadioProgramTextSource::RetainedBuffer, 0)
            } else {
                let time_of_day = loop {
                    let time = random(false)? & 3;
                    if time != 3 {
                        break time;
                    }
                };
                let mut carry = false;
                let slot = loop {
                    let slot = random(carry)? & 7;
                    if (2..5).contains(&slot) {
                        break slot;
                    }
                    carry = slot < 2;
                };
                effects.push(RadioProgramEffect::FormatOakEncounter {
                    route_index,
                    time_of_day,
                    slot,
                });
                (RadioProgramTextSource::Label("_OPT_OakText1"), 14)
            }
        }
        14 => (RadioProgramTextSource::Label("_OPT_OakText2"), 15),
        15 => (RadioProgramTextSource::Label("_OPT_OakText3"), 16),
        16 => {
            effects.push(RadioProgramEffect::FormatCurrentSpecies);
            (RadioProgramTextSource::Label("_OPT_MaryText1"), 17)
        }
        17 => {
            const LABELS: [&str; 16] = [
                "_OPT_SweetAdorablyText",
                "_OPT_WigglySlicklyText",
                "_OPT_AptlyNamedText",
                "_OPT_UndeniablyKindOfText",
                "_OPT_UnbearablyText",
                "_OPT_WowImpressivelyText",
                "_OPT_AlmostPoisonouslyText",
                "_OPT_SensuallyText",
                "_OPT_MischievouslyText",
                "_OPT_TopicallyText",
                "_OPT_AddictivelyText",
                "_OPT_LooksInWaterText",
                "_OPT_EvolutionMustBeText",
                "_OPT_ProvocativelyText",
                "_OPT_FlippedOutText",
                "_OPT_HeartMeltinglyText",
            ];
            (
                RadioProgramTextSource::Label(LABELS[usize::from(random(false)? & 15)]),
                18,
            )
        }
        18 => {
            const LABELS: [&str; 16] = [
                "_OPT_CuteText",
                "_OPT_WeirdText",
                "_OPT_PleasantText",
                "_OPT_BoldSortOfText",
                "_OPT_FrighteningText",
                "_OPT_SuaveDebonairText",
                "_OPT_PowerfulText",
                "_OPT_ExcitingText",
                "_OPT_GroovyText",
                "_OPT_InspiringText",
                "_OPT_FriendlyText",
                "_OPT_HotHotHotText",
                "_OPT_StimulatingText",
                "_OPT_GuardedText",
                "_OPT_LovelyText",
                "_OPT_SpeedyText",
            ];
            let label = LABELS[usize::from(random(false)? & 15)];
            let counter = context
                .oak
                .ok_or(RadioProgramError::SelectionData("Oak segment counter"))?
                .segment_counter
                .wrapping_sub(1);
            effects.push(RadioProgramEffect::SetOakSegmentCounter(if counter == 0 {
                5
            } else {
                counter
            }));
            (
                RadioProgramTextSource::Label(label),
                if counter == 0 { 59 } else { 13 },
            )
        }
        59 => {
            effects.extend([
                RadioProgramEffect::RestartPokemonChannelMusic,
                RadioProgramEffect::ClearTextbox,
                RadioProgramEffect::WaitBgMap,
                RadioProgramEffect::PrintTextboxLabel("_OPT_PokemonChannelText"),
                RadioProgramEffect::SetCurrentLine(60),
                RadioProgramEffect::SetRadioDelay(100),
            ]);
            return Ok(RadioProgramStep {
                effects,
                text: None,
            });
        }
        60..=63 => {
            let delay = context
                .oak
                .ok_or(RadioProgramError::SelectionData("Oak bumper delay"))?
                .delay
                .wrapping_sub(1);
            effects.push(RadioProgramEffect::SetRadioDelay(delay));
            if delay == 0 {
                if line == 63 {
                    effects.extend([
                        RadioProgramEffect::RestartMusic("MUSIC_POKEMON_TALK"),
                        RadioProgramEffect::ClearTextbox,
                        RadioProgramEffect::SetNextLine(13),
                        RadioProgramEffect::SetPrintedLines(0),
                        RadioProgramEffect::SetCurrentLine(84),
                        RadioProgramEffect::SetRadioDelay(10),
                    ]);
                } else {
                    effects.extend([
                        RadioProgramEffect::SetCurrentLine(line + 1),
                        RadioProgramEffect::SetRadioDelay(100),
                    ]);
                    let (column, row, text) = match line {
                        60 => (9, 14, "#MON@"),
                        61 => (1, 16, "#MON Channel@"),
                        _ => (12, 16, "@"),
                    };
                    effects.push(RadioProgramEffect::PlaceRadioString { column, row, text });
                }
            }
            return Ok(RadioProgramStep {
                effects,
                text: None,
            });
        }
        _ => return Err(RadioProgramError::UnimplementedLine(line)),
    };
    Ok(RadioProgramStep {
        effects,
        text: Some(RadioProgramText { source, next_line }),
    })
}
