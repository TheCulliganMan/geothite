//! Reconcile old authored city dialogue with the original reviewed gym cast.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

fn words(text: &str, old: &str, new: &str) -> String {
    text.split_inclusive(|c: char| !c.is_ascii_alphabetic())
        .map(|part| {
            let word = part.trim_end_matches(|c: char| !c.is_ascii_alphabetic());
            if word == old {
                format!("{new}{}", &part[word.len()..])
            } else {
                part.to_owned()
            }
        })
        .collect()
}

pub(super) fn reconcile(
    pack: &mut CompiledGamePack,
    pairs: &[(String, String)],
) -> Result<Vec<String>> {
    let mut changed = Vec::new();
    let names = pack
        .data
        .maps
        .keys()
        .filter(|name| {
            name.starts_with("Mn") && !name.ends_with("Gym") && name.as_str() != "MnGymReviewHall"
        })
        .cloned()
        .collect::<Vec<_>>();
    for name in names {
        let mut module: MapModule = pack.data.maps[&name].clone();
        let labels = module
            .script_text_bodies
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let mut modified = false;
        for label in labels {
            let Some(body) = module.scripts.get(&label).and_then(Value::as_array) else {
                continue;
            };
            let mentions = body.iter().any(|command| {
                command["args"].as_array().is_some_and(|args| {
                    args.iter()
                        .filter_map(Value::as_str)
                        .any(|line| pairs.iter().any(|(old, new)| words(line, old, new) != line))
                })
            });
            if !mentions {
                continue;
            }
            let mut lines = Vec::new();
            for command in body {
                let op = command["command"]
                    .as_str()
                    .context("authored text command")?;
                if op == "done" {
                    continue;
                }
                ensure!(
                    matches!(op, "text" | "line" | "para" | "cont"),
                    "name refresh requires literal authored dialogue: {label}"
                );
                let mut line: String = serde_json::from_str(
                    command["args"][0]
                        .as_str()
                        .context("quoted authored line")?,
                )?;
                for (old, new) in pairs {
                    line = words(&line, old, new);
                }
                lines.push(line);
            }
            authoring::text(
                &mut module,
                &label,
                &lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
            modified = true;
        }
        if modified {
            crate::gym_scripts::finish(pack, &mut module)?;
            crate::authored_town_publication::replace(pack, module)?;
            changed.push(name);
        }
    }
    Ok(changed)
}
