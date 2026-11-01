//! Explicit native dialogue pages. Finished prose is never silently reflowed.
use std::sync::OnceLock;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::MapModule;
use crate::grand_marais_modpack::authoring::command;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub(crate) struct AuthoredDialoguePage {
    pub lines: Vec<String>,
}

/// Installs a Minnesota conversation without reflowing its reviewed pages.
pub(crate) fn install_canonical(map: &mut MapModule, label: &str) -> Result<()> {
    static SOURCE: OnceLock<Value> = OnceLock::new();
    let source = SOURCE.get_or_init(|| {
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))
            .expect("canonical Minnesota story must be valid JSON")
    });
    let pages: Vec<AuthoredDialoguePage> =
        serde_json::from_value(source["dialogue_overhaul"]["texts"][&map.id][label].clone())
            .with_context(|| format!("canonical dialogue {}/{label}", map.id))?;
    map.scripts.insert(label.into(), compile(&pages)?);
    Ok(())
}

pub(crate) fn compile(pages: &[AuthoredDialoguePage]) -> Result<Value> {
    ensure!(!pages.is_empty(), "dialogue needs at least one page");
    let mut commands = Vec::new();
    for (page_index, page) in pages.iter().enumerate() {
        ensure!(
            (1..=2).contains(&page.lines.len()),
            "native page needs one or two lines"
        );
        for (line_index, line) in page.lines.iter().enumerate() {
            ensure!(
                !line.is_empty()
                    && line.is_ascii()
                    && line.len() <= 18
                    && !line
                        .chars()
                        .any(|c| c.is_control() || matches!(c, '"' | '<' | '>')),
                "invalid native dialogue line: {line:?}"
            );
            let opcode = if line_index == 1 {
                "line"
            } else if page_index == 0 {
                "text"
            } else {
                "para"
            };
            commands.push(command(opcode, &[&format!("\"{line}\"")]));
        }
    }
    commands.push(command("done", &[]));
    Ok(Value::Array(commands))
}

#[cfg(test)]
mod tests {
    use super::{AuthoredDialoguePage, compile};

    #[test]
    fn authored_dialogue_preserves_one_line_pages_and_rejects_overflow() {
        let pages = vec![
            AuthoredDialoguePage {
                lines: vec!["Take your time.".into()],
            },
            AuthoredDialoguePage {
                lines: vec!["I will wait here.".into(), "Come back anytime.".into()],
            },
        ];
        let body = compile(&pages).expect("valid native pages");
        let commands = body.as_array().expect("commands");
        assert_eq!(commands[1]["command"], "para");
        assert_eq!(commands[2]["command"], "line");
        assert!(
            compile(&[AuthoredDialoguePage {
                lines: vec!["This line is much too long.".into()]
            }])
            .is_err()
        );
        assert!(
            compile(&[AuthoredDialoguePage {
                lines: vec!["One".into(), "Two".into(), "Three".into()]
            }])
            .is_err()
        );
    }
}
