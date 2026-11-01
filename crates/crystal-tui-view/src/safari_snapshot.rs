//! Read-only projection of the same saved ranger/capture owners as graphics.

use crystal_runtime::RuntimeShellSnapshot;
use crystal_runtime::core::systems::safari::SafariPhase;
use crystal_runtime::core::systems::safari_catch_presentation::SafariCatchStage as Stage;
use crystal_runtime::core::systems::safari_prompt::SafariPromptKind;

use super::{LineKind, SnapshotLine};

fn lines(text: impl IntoIterator<Item = String>) -> Vec<SnapshotLine> {
    text.into_iter()
        .map(|text| SnapshotLine::new(text, LineKind::Normal))
        .collect()
}

pub(super) fn viewport(source: &RuntimeShellSnapshot) -> Option<(String, Vec<SnapshotLine>)> {
    let session = source.safari.as_ref()?;
    if let Some(saved) = &source.safari_catch_presentation {
        if saved.stage == Stage::Naming {
            return Some((
                "NICKNAME".into(),
                lines(vec![saved.name_draft.value.clone()]),
            ));
        }
        if saved.stage == Stage::DexEntry {
            let SafariPhase::Presenting { result, .. } = &session.phase else {
                return None;
            };
            let crystal_runtime::core::systems::safari::SafariResolution::Caught(stored) =
                &result.resolution
            else {
                return None;
            };
            let entry = source
                .presentation
                .pokedex_entries
                .get(&stored.pokemon.species.id)?;
            let mut text = vec![
                stored.pokemon.nickname.clone(),
                entry.classification.clone(),
            ];
            text.extend(entry.pages.get(saved.dex_page)?.lines().map(str::to_string));
            return Some(("POKéDEX".into(), lines(text)));
        }
    }
    let wild = match &session.phase {
        SafariPhase::Encounter(wild)
        | SafariPhase::Presenting {
            encounter: wild, ..
        } => Some(wild),
        SafariPhase::Ending(_) => None,
        _ => return None,
    };
    let mut text = vec![
        "BOUNDARY WATERS SAFARI".into(),
        format!(
            "BALLS {:02}  STEPS {:03}",
            session.balls_remaining, session.steps_remaining
        ),
    ];
    if let Some(wild) = wild {
        text.push(format!(
            "Wild {}  Lv{}",
            wild.pokemon.nickname, wild.pokemon.level
        ));
        text.push("Watching you.".into());
    }
    Some(("Safari".into(), lines(text)))
}

pub(super) fn menu(source: &RuntimeShellSnapshot) -> Option<Vec<SnapshotLine>> {
    if let Some(saved) = &source.safari_catch_presentation
        && saved.stage == Stage::Naming
    {
        let layout = crystal_runtime::core::systems::name_entry::name_entry_layout(
            saved.name_draft.uppercase,
        );
        return Some(lines(layout.iter().enumerate().map(|(row, label)| {
            let column = if row == layout.len() - 1 {
                match saved.name_draft.cursor_column {
                    0..=4 => 0,
                    5..=6 => 1,
                    _ => 2,
                }
            } else {
                saved.name_draft.cursor_column
            };
            label
                .split_whitespace()
                .enumerate()
                .map(|(index, token)| {
                    format!(
                        "{}{token}",
                        if row == saved.name_draft.cursor_row && index == column {
                            ">"
                        } else {
                            " "
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        })));
    }
    let session = source.safari.as_ref()?;
    if session.phase == SafariPhase::Exploring {
        return None;
    }
    if !matches!(session.phase, SafariPhase::Encounter(_)) {
        return Some(Vec::new());
    }
    Some(lines(
        ["BALL", "BAIT", "ROCK", "RUN"]
            .iter()
            .enumerate()
            .map(|(index, label)| {
                format!(
                    "{}{label}",
                    if index == session.command_cursor as usize {
                        "> "
                    } else {
                        "  "
                    }
                )
            }),
    ))
}

pub(super) fn prompt(source: &RuntimeShellSnapshot) -> Option<Vec<SnapshotLine>> {
    if let Some(saved) = &source.safari_catch_presentation
        && saved.stage == Stage::NicknamePrompt
    {
        return Some(lines(vec![
            format!("{}YES", if saved.selected_yes { "> " } else { "  " }),
            format!("{}NO", if saved.selected_yes { "  " } else { "> " }),
        ]));
    }
    let prompt = source.safari_prompt.as_ref()?;
    Some(lines(vec![
        format!("{}YES", if prompt.selected_yes { "> " } else { "  " }),
        format!("{}NO", if prompt.selected_yes { "  " } else { "> " }),
    ]))
}

pub(super) fn dialogue(source: &RuntimeShellSnapshot) -> Option<Vec<SnapshotLine>> {
    if let Some(saved) = &source.safari_catch_presentation {
        return Some(if matches!(saved.stage, Stage::DexEntry | Stage::Naming) {
            Vec::new()
        } else {
            lines(crystal_runtime::safari_catch_presentation_lines(source)?)
        });
    }
    if let Some(prompt) = &source.safari_prompt {
        return Some(lines(match prompt.kind {
            SafariPromptKind::Admission => {
                vec!["Enter for 500?".into(), "30 BALLS, 500 STEPS.".into()]
            }
            SafariPromptKind::Retire => {
                vec!["End the expedition?".into(), "Your catches stay.".into()]
            }
        }));
    }
    match &source.safari.as_ref()?.phase {
        SafariPhase::Presenting { encounter, result } => Some(lines(
            crystal_runtime::safari_result_lines(encounter, result),
        )),
        SafariPhase::Ending(_) => Some(lines(vec![
            "Expedition over!".into(),
            "Return to basecamp.".into(),
        ])),
        _ => None,
    }
}
