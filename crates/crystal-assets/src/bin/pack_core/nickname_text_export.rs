use anyhow::Result;
use serde_json::Value;
use std::{collections::{BTreeMap, BTreeSet}, path::Path};

use super::radio_text_export::{collect_text_bodies, export_global_text_bodies};

fn source_text_bodies(repository: &Path) -> Result<BTreeMap<String, Value>> {
    let labels = BTreeSet::from([
        "_CaughtAskNicknameText".to_string(),
        "_BreedAskNicknameText".to_string(),
    ]);
    let source = std::fs::read_to_string(
        repository.join("vendor/pokecrystal/data/text/common_2.asm"),
    )?;
    let mut bodies = BTreeMap::new();
    collect_text_bodies(&source, &labels, &mut bodies)?;
    for label in labels {
        anyhow::ensure!(bodies.contains_key(&label), "missing source nickname text {label}");
    }
    Ok(bodies)
}

pub fn export(repository: &Path) -> Result<()> {
    export_global_text_bodies(repository, &source_text_bodies(repository)?, &BTreeMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nickname_question_export_preserves_cont_and_ram_commands() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let bodies = source_text_bodies(&repository).expect("read ASM nickname questions");
        assert_eq!(bodies["_CaughtAskNicknameText"], json!([
            {"command":"text", "args":["\"Give a nickname to\""]},
            {"command":"line", "args":["\"the @\""]},
            {"command":"text_ram", "args":["wStringBuffer1"]},
            {"command":"text", "args":["\" you\""]},
            {"command":"cont", "args":["\"received?\""]},
            {"command":"done", "args":[]},
        ]));
        assert_eq!(bodies["_BreedAskNicknameText"][4]["command"], "done");
    }
}
