use anyhow::{Context, Result};
use crystal_core::systems::script_text::ScriptTextBodyCommand;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn code_part(line: &str) -> &str {
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && quoted {
            escaped = true;
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
        }
        if ch == ';' && !quoted {
            return line[..index].trim();
        }
    }
    line.trim()
}

fn source_arguments(source: &str) -> Result<Vec<String>> {
    if source.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in source.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && quoted {
            escaped = true;
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
        }
        if ch == ',' && !quoted {
            result.push(source[start..index].trim().to_string());
            start = index + 1;
        }
    }
    anyhow::ensure!(!quoted && !escaped, "unterminated radio text string");
    result.push(source[start..].trim().to_string());
    anyhow::ensure!(
        result.iter().all(|arg| !arg.is_empty()),
        "empty radio text operand"
    );
    Ok(result)
}

fn collect_text_bodies(
    source: &str,
    required: &BTreeSet<String>,
    bodies: &mut BTreeMap<String, Value>,
) -> Result<()> {
    let mut active: Option<(String, Vec<Value>)> = None;
    for raw in source.lines() {
        let code = code_part(raw);
        if code.is_empty() {
            continue;
        }
        if code.ends_with(':') {
            anyhow::ensure!(
                active.is_none(),
                "radio text body is missing its terminator"
            );
            let label = code.trim_end_matches(':');
            if required.contains(label) {
                active = Some((label.into(), Vec::new()));
            }
            continue;
        }
        let Some((label, commands)) = active.as_mut() else {
            continue;
        };
        let (command, operands) = code.split_once(char::is_whitespace).unwrap_or((code, ""));
        let args = source_arguments(operands)?;
        // Validate against the runtime's existing typed text grammar, keeping
        // source quotes, line control, RAM operands and pauses intact.
        let _: ScriptTextBodyCommand = serde_json::from_value(json!({
            "command": command, "args": args, "command_index": commands.len()
        }))
        .with_context(|| format!("source radio text {label}: {code}"))?;
        commands.push(json!({"command": command, "args": args}));
        if matches!(command, "done" | "prompt" | "text_end") {
            let (label, commands) = active.take().expect("active radio body");
            anyhow::ensure!(
                bodies.insert(label.clone(), json!(commands)).is_none(),
                "duplicate source radio text body {label}"
            );
        }
    }
    anyhow::ensure!(
        active.is_none(),
        "radio text body is missing its terminator"
    );
    Ok(())
}

fn source_text_bodies(repository: &Path) -> Result<BTreeMap<String, Value>> {
    let radio =
        std::fs::read_to_string(repository.join("vendor/pokecrystal/engine/pokegear/radio.asm"))?;
    let mut required = BTreeSet::new();
    for raw in radio.lines() {
        let code = code_part(raw);
        let (command, operands) = code.split_once(char::is_whitespace).unwrap_or((code, ""));
        if command == "text_far" {
            let operands = source_arguments(operands)?;
            anyhow::ensure!(
                operands.len() == 1,
                "radio text_far requires one source label"
            );
            required.insert(operands[0].clone());
        }
    }
    anyhow::ensure!(!required.is_empty(), "radio has no source text references");
    let mut files = std::fs::read_dir(repository.join("vendor/pokecrystal/data/text"))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    files.sort();
    let mut bodies = BTreeMap::new();
    for file in files {
        if file.extension().is_some_and(|extension| extension == "asm") {
            collect_text_bodies(&std::fs::read_to_string(&file)?, &required, &mut bodies)
                .with_context(|| format!("read radio text from {}", file.display()))?;
        }
    }
    for label in required {
        anyhow::ensure!(
            bodies.contains_key(&label),
            "source radio text {label} is missing"
        );
    }
    Ok(bodies)
}

fn collect_selection_table(
    source: &str,
    labels: &[&str],
    command: &str,
) -> Result<BTreeMap<String, Value>> {
    let mut starts = Vec::new();
    let mut rows = Vec::new();
    let mut ended = false;
    for code in source
        .lines()
        .map(code_part)
        .filter(|line| !line.is_empty())
    {
        anyhow::ensure!(
            !ended,
            "radio selection table has data after its terminator"
        );
        if code.ends_with(':') {
            let label = code.trim_end_matches(':');
            anyhow::ensure!(
                labels.get(starts.len()).copied() == Some(label),
                "unexpected radio selection label {label}"
            );
            starts.push((label.to_string(), rows.len()));
            continue;
        }
        anyhow::ensure!(!starts.is_empty(), "radio selection table has no label");
        if command == "map_id" && code == ".End" {
            ended = true;
            continue;
        }
        let (op, operands) = code.split_once(char::is_whitespace).unwrap_or((code, ""));
        let args = source_arguments(operands)?;
        anyhow::ensure!(
            op == command && args.len() == 1,
            "invalid radio selection row {code}"
        );
        let value = &args[0];
        let sentinel = command == "db" && value == "-1";
        anyhow::ensure!(
            sentinel
                || (value.starts_with(|ch: char| ch.is_ascii_uppercase())
                    && value
                        .chars()
                        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')),
            "invalid radio selection constant {value}"
        );
        rows.push(json!({"command": command, "args": args}));
        ended = sentinel;
    }
    anyhow::ensure!(
        ended && starts.len() == labels.len(),
        "incomplete radio selection table"
    );
    let mut tables = BTreeMap::new();
    for (label, start) in starts {
        anyhow::ensure!(
            start < rows.len() - usize::from(command == "db"),
            "empty radio selection table {label}"
        );
        // ASM searches from the selected label through the shared -1 sentinel.
        tables.insert(label, json!(&rows[start..]));
    }
    Ok(tables)
}

fn collect_landmark_tables(source: &str) -> Result<BTreeMap<String, Value>> {
    let mut tables = BTreeMap::new();
    let mut rows = Vec::new();
    let mut in_table = false;
    let mut ended = false;
    for code in source
        .lines()
        .map(code_part)
        .filter(|code| !code.is_empty())
    {
        if code == "Landmarks:" {
            anyhow::ensure!(!in_table && !ended, "duplicate Landmarks table");
            in_table = true;
            continue;
        }
        if in_table {
            if code == "assert_table_length NUM_LANDMARKS" {
                in_table = false;
                ended = true;
                continue;
            }
            if code.starts_with("table_width ") || code == "assert_table_length KANTO_LANDMARK" {
                continue;
            }
            let operands = code
                .strip_prefix("landmark ")
                .context("unexpected Landmarks row")?;
            let args = source_arguments(operands)?;
            anyhow::ensure!(args.len() == 3, "landmark requires x, y, name");
            args[0].parse::<i16>().context("invalid landmark x")?;
            args[1].parse::<i16>().context("invalid landmark y")?;
            rows.push(json!({"command":"landmark", "args":args}));
            continue;
        }
        if !ended {
            continue;
        }
        let (label, body) = code
            .split_once(':')
            .context("landmark name must have a label")?;
        let operands = body
            .trim()
            .strip_prefix("db ")
            .context("landmark name must use db")?;
        let args = source_arguments(operands)?;
        anyhow::ensure!(args.len() == 1, "landmark name must be one source string");
        let name: String =
            serde_json::from_str(&args[0]).context("invalid landmark name string")?;
        anyhow::ensure!(name.ends_with('@'), "unterminated landmark name {label}");
        anyhow::ensure!(
            tables
                .insert(label.to_string(), json!([{"command":"db", "args":args}]))
                .is_none(),
            "duplicate landmark name {label}"
        );
    }
    anyhow::ensure!(ended && !rows.is_empty(), "incomplete Landmarks table");
    for row in &rows {
        let label = row["args"][2]
            .as_str()
            .context("invalid landmark name reference")?;
        anyhow::ensure!(
            tables.contains_key(label),
            "missing source landmark name {label}"
        );
    }
    tables.insert("Landmarks".into(), json!(rows));
    Ok(tables)
}

fn collect_pokemon_names(source: &str) -> Result<Value> {
    let mut rows = Vec::new();
    let mut header = 0;
    let mut assertions = 0;
    for code in source
        .lines()
        .map(code_part)
        .filter(|line| !line.is_empty())
    {
        anyhow::ensure!(
            assertions < 3,
            "PokemonNames has data after its final assertion"
        );
        if header == 0 {
            anyhow::ensure!(code == "PokemonNames::", "missing PokemonNames label");
            header = 1;
            continue;
        }
        if header == 1 {
            anyhow::ensure!(
                code == "table_width NAME_LENGTH - 1",
                "invalid PokemonNames slot width"
            );
            header = 2;
            continue;
        }
        let (command, operands) = code.split_once(char::is_whitespace).unwrap_or((code, ""));
        if command == "assert_table_length" {
            let (symbol, count) = [("NUM_POKEMON", 251), ("EGG", 253), ("$100", 256)][assertions];
            anyhow::ensure!(
                operands == symbol && rows.len() == count,
                "invalid PokemonNames assertion {code}"
            );
            assertions += 1;
            continue;
        }
        let args = source_arguments(operands)?;
        anyhow::ensure!(
            command == "dname" && args.len() == 1,
            "invalid PokemonNames row {code}"
        );
        let name: String = serde_json::from_str(&args[0]).context("invalid PokemonNames string")?;
        let bytes = crystal_core::systems::radio_text::encode_radio_string(&name)?;
        anyhow::ensure!(
            !bytes.is_empty() && bytes.len() <= 10 && !bytes.contains(&0x50),
            "invalid fixed-width PokemonNames string {name}"
        );
        rows.push(json!({"command":"dname", "args":args}));
    }
    anyhow::ensure!(
        assertions == 3 && rows.len() == 256,
        "incomplete PokemonNames table"
    );
    Ok(json!(rows))
}

fn collect_radio_trainer_indices(source: &str) -> Result<BTreeMap<String, Value>> {
    let mut classes = Vec::new();
    let mut ids = BTreeMap::<String, Vec<Value>>::new();
    let mut current = None::<String>;
    let mut ended = false;
    for code in source
        .lines()
        .map(code_part)
        .filter(|line| !line.is_empty())
    {
        if current.is_none() && code != "trainerclass TRAINER_NONE" {
            continue;
        }
        anyhow::ensure!(
            !ended,
            "trainer constants continue after NUM_TRAINER_CLASSES"
        );
        if code == "DEF NUM_TRAINER_CLASSES EQU __trainer_class__ - 1" {
            ended = true;
            continue;
        }
        if matches!(
            code,
            "DEF KRIS EQU __trainer_class__"
                | "DEF NUM_NONTRAINER_PHONECONTACTS EQU const_value - 1"
        ) {
            continue;
        }
        let (command, operands) = code.split_once(char::is_whitespace).unwrap_or((code, ""));
        let args = source_arguments(operands)?;
        anyhow::ensure!(
            args.len() == 1
                && args[0]
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'),
            "invalid source trainer constant {code}"
        );
        match command {
            "trainerclass" => {
                let class = args[0].clone();
                anyhow::ensure!(
                    ids.insert(class.clone(), Vec::new()).is_none(),
                    "duplicate trainer class {class}"
                );
                classes.push(json!({"command":"trainerclass", "args":args}));
                current = Some(class);
            }
            "const" => {
                let slots = ids
                    .get_mut(current.as_ref().context("trainer slot has no class")?)
                    .context("trainer class is missing")?;
                anyhow::ensure!(
                    !slots.iter().any(|slot| slot["args"][0] == args[0]),
                    "duplicate trainer slot {code}"
                );
                slots.push(json!({"command":"const", "args":args}));
            }
            _ => anyhow::bail!("unsupported source trainer constant {code}"),
        }
    }
    anyhow::ensure!(
        ended && classes.len() == 68,
        "incomplete source trainer classes"
    );
    let mut tables = BTreeMap::from([("RadioTrainerClasses".to_string(), json!(classes))]);
    for (class, slots) in ids {
        tables.insert(format!("RadioTrainerIds_{class}"), json!(slots));
    }
    Ok(tables)
}

fn source_selection_tables(repository: &Path) -> Result<BTreeMap<String, Value>> {
    let directory = repository.join("vendor/pokecrystal/data/radio");
    let mut tables = BTreeMap::new();
    for (file, labels, command) in [
        (
            "oaks_pkmn_talk_routes.asm",
            &["OaksPKMNTalkRoutes"][..],
            "map_id",
        ),
        ("pnp_places.asm", &["PnP_Places"][..], "map_id"),
        (
            "pnp_hidden_people.asm",
            &[
                "PnP_HiddenPeople",
                "PnP_HiddenPeople_BeatE4",
                "PnP_HiddenPeople_BeatKanto",
            ][..],
            "db",
        ),
    ] {
        tables.extend(
            collect_selection_table(
                &std::fs::read_to_string(directory.join(file))?,
                labels,
                command,
            )
            .with_context(|| format!("read radio selection table {file}"))?,
        );
    }
    tables.extend(collect_landmark_tables(&std::fs::read_to_string(
        repository.join("vendor/pokecrystal/data/maps/landmarks.asm"),
    )?)?);
    tables.insert(
        "PokemonNames".into(),
        collect_pokemon_names(&std::fs::read_to_string(
            repository.join("vendor/pokecrystal/data/pokemon/names.asm"),
        )?)?,
    );
    tables.extend(collect_radio_trainer_indices(&std::fs::read_to_string(
        repository.join("vendor/pokecrystal/constants/trainer_constants.asm"),
    )?)?);
    Ok(tables)
}

pub fn export(repository: &Path) -> Result<()> {
    let bodies = source_text_bodies(repository)?;
    let tables = source_selection_tables(repository)?;
    let mut outputs = Vec::new();
    for relative in [
        "story_events/StandardScripts.json",
        "content-packs/core-modular/story_events/StandardScripts.json",
    ] {
        let path = repository.join("apps/web/assets/data").join(relative);
        let mut catalog: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        let scripts = catalog
            .get_mut("StandardScripts")
            .and_then(Value::as_object_mut)
            .with_context(|| format!("{} is missing StandardScripts", path.display()))?;
        // Radio is driven by its own program, not a map-script text reference.
        // Keep its text roots in the materialized global runtime catalog.
        let roots = scripts
            .get_mut("GlobalScriptRoots")
            .and_then(Value::as_array_mut)
            .with_context(|| format!("{} is missing GlobalScriptRoots", path.display()))?;
        for label in bodies.keys() {
            if !roots
                .iter()
                .any(|root| root.as_str() == Some(label.as_str()))
            {
                roots.push(json!(label));
            }
        }
        for (label, commands) in bodies.iter().chain(&tables) {
            scripts.insert(label.clone(), commands.clone());
        }
        let mut bytes = serde_json::to_vec_pretty(&catalog)?;
        bytes.push(b'\n');
        outputs.push((path, bytes));
    }
    for (path, bytes) in outputs {
        std::fs::write(path, bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_trainer_selection_uses_source_class_and_party_slot_indices() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let tables = source_selection_tables(&root).unwrap();
        let classes = tables
            .get("RadioTrainerClasses")
            .expect("numeric radio trainer classes need the source order")
            .as_array()
            .unwrap();
        assert_eq!(classes.len(), 68);
        for (index, name) in [
            (0, "TRAINER_NONE"),
            (1, "FALKNER"),
            (48, "FIREBREATHER"),
            (67, "MYSTICALMAN"),
        ] {
            assert_eq!(classes[index]["args"][0], name);
        }
        assert_eq!(tables["RadioTrainerIds_FALKNER"][0]["args"][0], "FALKNER1");
        assert_eq!(tables["RadioTrainerIds_FIREBREATHER"][0]["args"][0], "OTIS");
        assert_eq!(
            tables["RadioTrainerIds_MYSTICALMAN"][0]["args"][0],
            "EUSINE"
        );
    }

    #[test]
    fn radio_species_names_reject_truncation_wrong_width_and_embedded_terminators() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let source =
            std::fs::read_to_string(root.join("vendor/pokecrystal/data/pokemon/names.asm"))
                .unwrap();
        for invalid in [
            source.replace("assert_table_length $100", ""),
            source.replace("NAME_LENGTH - 1", "NAME_LENGTH"),
            source.replacen("BULBASAUR", "BULBASAUR@", 1),
            source.replacen("BULBASAUR", "BULBASAURXXX", 1),
        ] {
            assert!(collect_pokemon_names(&invalid).is_err());
        }
    }

    #[test]
    fn radio_species_names_keep_source_punctuation_and_all_fixed_slots() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let tables = source_selection_tables(&root).unwrap();
        let names = tables
            .get("PokemonNames")
            .expect("radio GetPokemonName needs its source table")
            .as_array()
            .unwrap();
        assert_eq!(names.len(), 256);
        let source_bytes =
            include_bytes!("../../../../../../tools/asm-oracle/fixtures/pokemon-names.bin");
        let mut exported_bytes = Vec::new();
        for row in names {
            let name: String = serde_json::from_str(row["args"][0].as_str().unwrap()).unwrap();
            let mut bytes = crystal_core::systems::radio_text::encode_radio_string(&name).unwrap();
            bytes.resize(10, 0x50);
            exported_bytes.extend(bytes);
        }
        assert_eq!(
            exported_bytes, source_bytes,
            "all 256 dname slots must match the ROM byte for byte"
        );

        for (index, expected) in [
            (28, "NIDORAN♀"),
            (31, "NIDORAN♂"),
            (82, "FARFETCH'D"),
            (121, "MR.MIME"),
            (250, "CELEBI"),
            (252, "EGG"),
        ] {
            assert_eq!(
                names[index],
                json!({"command":"dname", "args":[format!("\"{expected}\"")]})
            );
        }
    }

    #[test]
    fn radio_landmarks_keep_source_coordinates_names_and_controls() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let tables = source_selection_tables(&root).unwrap();
        let rows = tables
            .get("Landmarks")
            .expect("radio formatter requires source Landmarks")
            .as_array()
            .unwrap();
        assert_eq!(rows.len(), 96);
        assert_eq!(
            rows[1],
            json!({"command":"landmark", "args":["140", "100", "NewBarkTownName"]})
        );
        assert_eq!(
            tables["NewBarkTownName"],
            json!([{"command":"db", "args":["\"NEW BARK<BSP>TOWN@\""]}])
        );
        for row in rows {
            let label = row["args"][2].as_str().unwrap();
            let name = &tables[label][0]["args"][0];
            let text: String = serde_json::from_str(name.as_str().unwrap()).unwrap();
            assert!(text.ends_with('@'), "{label}");
        }
    }

    #[test]
    fn radio_selection_parser_keeps_fallthrough_and_rejects_incomplete_tables() {
        let tables = collect_selection_table(
            "First:\ndb WILL\nSecond:\ndb RED\ndb -1",
            &["First", "Second"],
            "db",
        )
        .unwrap();
        assert_eq!(tables["First"].as_array().unwrap().len(), 3);
        assert_eq!(
            tables["Second"],
            json!([
                {"command":"db", "args":["RED"]},
                {"command":"db", "args":["-1"]}
            ])
        );
        for source in [
            "First:\ndb RED",
            "First:\ndb -1",
            "First:\ndb RED, WILL\ndb -1",
            "First:\ndb RED\ndb -1\ndb WILL",
            "Wrong:\ndb RED\ndb -1",
        ] {
            assert!(
                collect_selection_table(source, &["First"], "db").is_err(),
                "{source}"
            );
        }
        for source in [
            "First:\nmap_id ROUTE_29",
            "First:\n.End",
            "First:\nmap_id -1\n.End",
        ] {
            assert!(
                collect_selection_table(source, &["First"], "map_id").is_err(),
                "{source}"
            );
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let tables = source_selection_tables(&root).unwrap();
        for (label, count) in [
            ("OaksPKMNTalkRoutes", 15),
            ("PnP_Places", 9),
            ("PnP_HiddenPeople", 19),
            ("PnP_HiddenPeople_BeatE4", 14),
            ("PnP_HiddenPeople_BeatKanto", 6),
        ] {
            assert_eq!(tables[label].as_array().unwrap().len(), count, "{label}");
        }
    }

    #[test]
    fn all_radio_text_bodies_preserve_source_control_commands() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let bodies = source_text_bodies(&root).unwrap();
        assert_eq!(bodies.len(), 113);
        assert_eq!(
            bodies["_RocketRadioText7"],
            json!([
                {"command":"text_start", "args":[]},
                {"command":"line", "args":["\"GIOVANNI! @\""]},
                {"command":"text_pause", "args":[]},
                {"command":"text", "args":["\"Can you\""]},
                {"command":"done", "args":[]}
            ])
        );
        assert_eq!(
            bodies
                .values()
                .flat_map(|body| body.as_array().unwrap())
                .filter(|command| command["command"] == "text_pause")
                .count(),
            5
        );
    }

    #[test]
    fn radio_text_parser_keeps_quoted_punctuation_and_rejects_invalid_bodies() {
        assert_eq!(
            source_arguments("\"a,b; c@\"").unwrap(),
            vec!["\"a,b; c@\""]
        );
        assert_eq!(code_part("text \"a; b\" ; comment"), "text \"a; b\"");
        let required = BTreeSet::from(["RadioText".to_string()]);
        for source in [
            "RadioText::\ntext_start",
            "RadioText::\nunknown_macro\ndone",
            "RadioText::\ntext_pause 30\ndone",
            "RadioText::\ntext \"broken\ndone",
        ] {
            assert!(
                collect_text_bodies(source, &required, &mut BTreeMap::new()).is_err(),
                "{source}"
            );
        }
    }
}
