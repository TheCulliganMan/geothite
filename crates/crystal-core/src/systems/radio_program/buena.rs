use super::*;

fn opening(effects: &mut Vec<RadioProgramEffect>, printed: u8) {
    if printed == 0 {
        start(effects, "MUSIC_BUENAS_PASSWORD");
    }
    effects.push(RadioProgramEffect::PlaceRadioString {
        column: 2,
        row: 9,
        text: "BUENA'S PASSWORD@",
    });
}

fn off_air(effects: &mut Vec<RadioProgramEffect>) {
    effects.extend([
        RadioProgramEffect::NoRadioMusic,
        RadioProgramEffect::NoRadioName,
        RadioProgramEffect::SetBuenaPasswordGenerated(false),
        RadioProgramEffect::SetCurrentLine(4),
        RadioProgramEffect::SetPrintedLines(0),
    ]);
}

pub(super) fn step(
    line: u8,
    context: RadioProgramContext<'_>,
    random: &mut impl FnMut(bool) -> Result<u8, RadioProgramError>,
) -> Result<RadioProgramStep, RadioProgramError> {
    let data = context.buena.ok_or(RadioProgramError::SelectionData(
        "Buena clock and daily password",
    ))?;
    if data.hour >= 24 {
        return Err(RadioProgramError::SelectionData("Buena hour"));
    }
    let night = data.hour >= 18; // BuenasPasswordCheckTime: cp NITE_HOUR.
    let mut effects = Vec::new();
    let (label, next_line) = match line {
        4 => {
            if night {
                opening(&mut effects, context.printed);
                ("_BuenaRadioText1", 64)
            } else if context.printed == 0 {
                off_air(&mut effects);
                ("_BuenaOffTheAirText", 83)
            } else {
                effects.push(RadioProgramEffect::SetBuenaPasswordGenerated(false));
                ("_BuenaRadioMidnightText10", 71)
            }
        }
        64 => ("_BuenaRadioText2", 65),
        65 | 69 => {
            if !night {
                effects.push(RadioProgramEffect::SetBuenaPasswordGenerated(false));
            }
            (
                if line == 65 {
                    "_BuenaRadioText3"
                } else {
                    "_BuenaRadioText7"
                },
                if !night {
                    70
                } else if line == 65 {
                    66
                } else {
                    4
                },
            )
        }
        66 => {
            if !night {
                effects.push(RadioProgramEffect::SetBuenaPasswordGenerated(false));
                ("_BuenaRadioMidnightText10", 71)
            } else {
                let password = if data.password_generated {
                    data.password
                } else {
                    let category = loop {
                        let value = random(false)? & 15;
                        if value < 11 {
                            break value;
                        }
                    };
                    let word = loop {
                        let value = random(false)? & 3;
                        if value < 3 {
                            break value;
                        }
                    };
                    let password = (category << 4) | word;
                    effects.push(RadioProgramEffect::StoreBuenaPassword(password));
                    effects.push(RadioProgramEffect::SetBuenaPasswordGenerated(true));
                    password
                };
                if password >> 4 >= 11 || password & 15 >= 3 {
                    return Err(RadioProgramError::SelectionData(
                        "Buena saved password index",
                    ));
                }
                effects.push(RadioProgramEffect::FormatBuenaPassword(password));
                ("_BuenaRadioText4", 67)
            }
        }
        67 => ("_BuenaRadioText5", 68),
        68 => ("_BuenaRadioText6", 69),
        70 => {
            effects.push(RadioProgramEffect::SetBuenaPasswordGenerated(false));
            ("_BuenaRadioMidnightText10", 71)
        }
        71..=79 => {
            const LABELS: [&str; 9] = [
                "_BuenaRadioMidnightText1",
                "_BuenaRadioMidnightText2",
                "_BuenaRadioMidnightText3",
                "_BuenaRadioMidnightText4",
                "_BuenaRadioMidnightText5",
                "_BuenaRadioMidnightText6",
                "_BuenaRadioMidnightText7",
                "_BuenaRadioMidnightText8",
                "_BuenaRadioMidnightText9",
            ];
            (LABELS[usize::from(line - 71)], line + 1)
        }
        80 | 81 => ("_BuenaRadioMidnightText10", line + 1),
        82 => {
            off_air(&mut effects);
            ("_BuenaOffTheAirText", 83)
        }
        83 => {
            effects.extend([
                RadioProgramEffect::SetCurrentLine(4),
                RadioProgramEffect::SetPrintedLines(0),
            ]);
            if night {
                opening(&mut effects, 0);
                ("_BuenaRadioText1", 64)
            } else {
                ("_BuenaOffTheAirText", 83)
            }
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
