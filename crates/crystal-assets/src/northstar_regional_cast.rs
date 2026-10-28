//! Bind the authored Minnesota campaign to its active original-species roster.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::northstar_regional_roster::RegionalRoster;

/// Rebinds Minnesota teams, grants, encounters and visible species references.
///
/// Uses the current curated source roster. Ordinary trainer levels, party sizes,
/// items and DVs remain fixed. Explicit faction teams use authored party sizes,
/// coverage and legal native level/TM moves, with source-certified boss AI.
/// Unrelated source regions and all audio bytes remain intact.
///
/// # Errors
/// Returns an error for inconsistent curation, unmapped or missing species,
/// invalid native programs, illegal level bindings or failed pack verification.
pub fn build_northstar_regional_cast(
    base: &CompiledGamePack,
    source: &str,
    roster: &str,
    curation: &str,
    profiles: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|m| m == "northstar-minnesota-only-v1"),
        "regional cast is already bound"
    );
    let policy = RegionalRoster::parse(source, roster, curation, profiles)?;
    let source: Value = serde_json::from_str(source)?;
    let mut pack = base.clone();
    let trainers = pack
        .data
        .trainers
        .trainers
        .keys()
        .filter(|id| id.starts_with("MN_"))
        .cloned()
        .collect::<Vec<_>>();
    for id in trainers {
        let mut trainer = pack.data.trainers.trainers[&id].clone();
        let rival: Option<Vec<(String, u8)>> = source["rival_parties"]
            .get(&id)
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?;
        if let Some(party) = &rival {
            ensure!(
                party.len() == trainer.party.len(),
                "rival party shape differs"
            );
        }
        for (index, pokemon) in trainer.party.iter_mut().enumerate() {
            if let Some(party) = &rival {
                ensure!(
                    party[index].1 == pokemon.level,
                    "rival migration changes level"
                );
                pokemon.species.clone_from(&party[index].0);
            }
            pokemon.species = policy.target(&pack, &pokemon.species, Some(pokemon.level))?;
            pokemon.moves = pack
                .data
                .create_pokemon(&pokemon.species, pokemon.level, pokemon.dvs)?
                .moves;
        }
        *pack
            .data
            .trainers
            .trainers
            .get_mut(&id)
            .context("installed Minnesota trainer")? = trainer;
    }
    crate::northstar_faction_teams::install(&mut pack, &policy, &source)?;
    let maps = pack
        .data
        .maps
        .keys()
        .filter(|name| name.starts_with("Mn"))
        .cloned()
        .collect::<Vec<_>>();
    for name in maps {
        let mut map = pack.data.maps[&name].clone();
        let mut changed = false;
        let mut text_changes = Vec::new();
        for (label, program) in &mut map.scripts {
            let Some(rows) = program.as_array_mut() else {
                continue;
            };
            let is_text = rows.iter().all(|r| {
                matches!(
                    r["command"].as_str(),
                    Some("text" | "line" | "para" | "cont" | "done" | "prompt")
                )
            });
            if is_text {
                let mut lines = Vec::new();
                let mut text_changed = false;
                for row in rows {
                    if let Some(line) = row["args"]
                        .as_array()
                        .and_then(|a| a.first())
                        .and_then(Value::as_str)
                    {
                        let original = line.trim_matches('"');
                        let updated = rewrite_line(&pack, &policy, original)?;
                        text_changed |= updated != original;
                        lines.push(updated);
                    }
                }
                if text_changed {
                    text_changes.push((label.clone(), lines));
                }
                continue;
            }
            for row in rows {
                let command = row["command"].as_str().unwrap_or("").to_owned();
                if !matches!(
                    command.as_str(),
                    "givepoke" | "giveegg" | "loadwildmon" | "cry" | "pokepic" | "checkpoke"
                ) {
                    continue;
                }
                let args = row["args"]
                    .as_array_mut()
                    .context("native species command arguments")?;
                let original = args
                    .first()
                    .and_then(Value::as_str)
                    .context("native species argument")?;
                if !pack.data.pokemon.contains_key(original) {
                    continue;
                }
                let level = if matches!(command.as_str(), "givepoke" | "giveegg" | "loadwildmon") {
                    Some(
                        args.get(1)
                            .and_then(Value::as_str)
                            .context("native species level")?
                            .parse::<u8>()?,
                    )
                } else {
                    None
                };
                let replacement = policy.target(&pack, original, level)?;
                if replacement != original {
                    args[0] = Value::String(replacement);
                    changed = true;
                }
            }
        }
        for (label, lines) in text_changes {
            authoring::text(
                &mut map,
                &label,
                &lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
            changed = true;
        }
        let audio = crate::parse_script_audio_commands(&name, &map.scripts)?;
        if map.script_audio_commands != audio {
            map.script_audio_commands = audio;
            changed = true;
        }
        if changed {
            crate::ely_league_rooms::finish(&pack, &mut map)?;
            crate::authored_town_publication::replace(&mut pack, map)?;
        }
    }
    for name in pack
        .data
        .wild_encounters
        .keys()
        .filter(|n| n.starts_with("Mn"))
        .cloned()
        .collect::<Vec<_>>()
    {
        let mut value = serde_json::to_value(&pack.data.wild_encounters[&name])?;
        rewrite_encounters(&pack, &policy, &mut value)?;
        pack.data
            .wild_encounters
            .insert(name, serde_json::from_value(value)?);
    }
    for name in pack
        .data
        .field_encounters
        .keys()
        .filter(|n| n.starts_with("Mn"))
        .cloned()
        .collect::<Vec<_>>()
    {
        let mut value = serde_json::to_value(&pack.data.field_encounters[&name])?;
        rewrite_encounters(&pack, &policy, &mut value)?;
        pack.data
            .field_encounters
            .insert(name, serde_json::from_value(value)?);
    }
    // Fishing groups are shared by source regions. Clone the groups used by
    // Minnesota instead of mutating the original Johto/Kanto fishing tables.
    regional_fishing(&mut pack, &policy)?;
    let mut intro = serde_json::to_value(&pack.data.runtime_title_screen.program)?;
    rewrite_intro(&pack, &policy, &mut intro)?;
    pack.data.runtime_title_screen.program = serde_json::from_value(intro)?;
    audit(&pack, &policy)?;
    pack.report
        .manifests
        .push("northstar-minnesota-only-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn rewrite_line(pack: &CompiledGamePack, policy: &RegionalRoster, line: &str) -> Result<String> {
    let mut result = String::new();
    let mut word = String::new();
    for character in line.chars().chain(std::iter::once('\0')) {
        if character.is_ascii_alphabetic() || character == '_' {
            word.push(character);
            continue;
        }
        if !word.is_empty() {
            let upper = word.to_ascii_uppercase();
            if pack.data.pokemon.contains_key(&upper) && !policy.allowed.contains(&upper) {
                result.push_str(&policy.target(pack, &upper, None)?);
            } else {
                result.push_str(&word);
            }
            word.clear();
        }
        if character != '\0' {
            result.push(character);
        }
    }
    Ok(result)
}

fn rewrite_encounters(
    pack: &CompiledGamePack,
    policy: &RegionalRoster,
    value: &mut Value,
) -> Result<()> {
    match value {
        Value::Object(object) => {
            for (species, level) in [
                ("species", "level"),
                ("day_species", "day_level"),
                ("night_species", "night_level"),
            ] {
                if let Some(original) = object.get(species).and_then(Value::as_str) {
                    let level = object
                        .get(level)
                        .and_then(Value::as_u64)
                        .map(u8::try_from)
                        .transpose()?;
                    let replacement = policy.target(pack, original, level)?;
                    object.insert(species.into(), Value::String(replacement));
                }
            }
            for child in object.values_mut() {
                rewrite_encounters(pack, policy, child)?;
            }
        }
        Value::Array(array) => {
            for child in array {
                rewrite_encounters(pack, policy, child)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn regional_fishing(pack: &mut CompiledGamePack, policy: &RegionalRoster) -> Result<()> {
    let mut groups = std::collections::BTreeMap::new();
    for map in pack.data.maps.values().filter(|m| m.id.starts_with("Mn")) {
        if let Some(group) = &map.attributes.fishing_group
            && group != "FISHGROUP_NONE"
        {
            groups.insert(
                group.clone(),
                format!("FISHGROUP_MN_{}", group.trim_start_matches("FISHGROUP_")),
            );
        }
    }
    let swarm_rules = pack.data.fishing.swarm_rules.clone();
    for rule in swarm_rules.values() {
        if groups.contains_key(&rule.base_group) {
            groups.entry(rule.swarm_group.clone()).or_insert_with(|| {
                format!(
                    "FISHGROUP_MN_{}",
                    rule.swarm_group.trim_start_matches("FISHGROUP_")
                )
            });
        }
    }
    let mut times = std::collections::BTreeMap::<String, String>::new();
    for (original, name) in &groups {
        let mut group = pack
            .data
            .fishing
            .groups
            .get(original)
            .context("native fishing group")?
            .clone();
        group.source_index = pack
            .data
            .fishing
            .groups
            .values()
            .map(|g| g.source_index)
            .max()
            .context("fishing indices")?
            .checked_add(1)
            .context("fishing group overflow")?;
        for table in group.rod_tables.values_mut() {
            for slot in &mut table.slots {
                if let Some(species) = &slot.species {
                    slot.species = Some(policy.target(pack, species, Some(slot.level))?);
                }
                if let Some(original) = &slot.time_group {
                    let name = if let Some(name) = times.get(original) {
                        name.clone()
                    } else {
                        let mut time = serde_json::to_value(
                            pack.data
                                .fishing
                                .time_groups
                                .get(original)
                                .context("native time fish")?,
                        )?;
                        rewrite_encounters(pack, policy, &mut time)?;
                        let name = format!("MN_TIME_{original}");
                        ensure!(
                            !pack.data.fishing.time_groups.contains_key(&name),
                            "occupied regional time fish"
                        );
                        pack.data
                            .fishing
                            .time_groups
                            .insert(name.clone(), serde_json::from_value(time)?);
                        times.insert(original.clone(), name.clone());
                        name
                    };
                    slot.time_group = Some(name);
                }
            }
        }
        ensure!(
            pack.data
                .fishing
                .groups
                .insert(name.clone(), group)
                .is_none(),
            "occupied regional fishing group"
        );
    }
    for (id, mut rule) in swarm_rules {
        if let Some(base) = groups.get(&rule.base_group) {
            rule.base_group = base.clone();
            rule.swarm_group = groups
                .get(&rule.swarm_group)
                .context("regional swarm group")?
                .clone();
            ensure!(
                pack.data
                    .fishing
                    .swarm_rules
                    .insert(format!("MN_{id}"), rule)
                    .is_none(),
                "occupied regional fishing swarm"
            );
        }
    }
    for name in pack
        .data
        .maps
        .keys()
        .filter(|n| n.starts_with("Mn"))
        .cloned()
        .collect::<Vec<_>>()
    {
        let mut map = pack.data.maps[&name].clone();
        if let Some(group) = map
            .attributes
            .fishing_group
            .as_ref()
            .and_then(|g| groups.get(g))
        {
            map.attributes.fishing_group = Some(group.clone());
            crate::authored_town_publication::replace(pack, map)?;
        }
    }
    Ok(())
}

fn rewrite_intro(
    pack: &CompiledGamePack,
    policy: &RegionalRoster,
    value: &mut Value,
) -> Result<()> {
    match value {
        Value::Object(object) => {
            for key in ["species", "cry"] {
                if let Some(name) = object.get(key).and_then(Value::as_str)
                    && pack.data.pokemon.contains_key(name)
                {
                    object.insert(key.into(), Value::String(policy.target(pack, name, None)?));
                }
            }
            for child in object.values_mut() {
                rewrite_intro(pack, policy, child)?;
            }
        }
        Value::Array(array) => {
            for child in array {
                rewrite_intro(pack, policy, child)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn audit(pack: &CompiledGamePack, policy: &RegionalRoster) -> Result<()> {
    let check = |name: &str| -> Result<()> {
        ensure!(
            policy.allowed.contains(name),
            "out-of-region Pokemon remains in the Minnesota story: {name}"
        );
        Ok(())
    };
    for trainer in pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|t| t.trainer_id.starts_with("MN_"))
    {
        for pokemon in &trainer.party {
            check(&pokemon.species)?;
        }
    }
    for map in pack.data.maps.values().filter(|m| m.id.starts_with("Mn")) {
        for audio in &map.script_audio_commands {
            if audio.command == "cry" {
                check(audio.audio_id.as_deref().context("regional cry species")?)?;
            }
        }
        for object in &map.objects {
            if let Some(name) = object.sprite.strip_prefix("SPRITE_")
                && pack.data.pokemon.contains_key(name)
            {
                check(name)?;
            }
        }
        for (label, program) in &map.scripts {
            if let Some(rows) = program.as_array() {
                for row in rows {
                    if row["command"] == "trade" {
                        anyhow::bail!(
                            "regional trade needs an explicit audited binding: {}/{label}",
                            map.id
                        );
                    }
                    if matches!(row["command"].as_str(), Some("loadtrainer" | "trainer")) {
                        let id = row["args"]
                            .get(1)
                            .and_then(Value::as_str)
                            .context("story trainer reference")?;
                        let trainer = pack
                            .data
                            .trainers
                            .trainers
                            .get(id)
                            .context("audited story trainer record")?;
                        for pokemon in &trainer.party {
                            check(&pokemon.species)?;
                        }
                    }
                    if let Some(args) = row["args"].as_array() {
                        for arg in args.iter().filter_map(Value::as_str) {
                            if pack.data.pokemon.contains_key(arg) {
                                check(arg)?;
                            }
                            if matches!(
                                row["command"].as_str(),
                                Some("text" | "line" | "para" | "cont")
                            ) {
                                let line = arg.trim_matches('"');
                                ensure!(
                                    rewrite_line(pack, policy, line)? == line,
                                    "legacy name remains in visible text {}/{label}",
                                    map.id
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    // Re-running the same strict field transform must have no work left in
    // every reachable regional encounter table, including time fish and swarms.
    for (kind, values) in [
        ("wild", serde_json::to_value(&pack.data.wild_encounters)?),
        ("field", serde_json::to_value(&pack.data.field_encounters)?),
        ("fishing", serde_json::to_value(&pack.data.fishing.groups)?),
        (
            "time fish",
            serde_json::to_value(&pack.data.fishing.time_groups)?,
        ),
    ] {
        for (name, value) in values.as_object().context("encounter catalog")? {
            if name.starts_with("Mn")
                || name.starts_with("FISHGROUP_MN_")
                || name.starts_with("MN_TIME_")
            {
                let mut checked = value.clone();
                rewrite_encounters(pack, policy, &mut checked)?;
                ensure!(&checked == value, "unbound regional {kind} table {name}");
            }
        }
    }
    let original_intro = serde_json::to_value(&pack.data.runtime_title_screen.program)?;
    let mut intro = original_intro.clone();
    rewrite_intro(pack, policy, &mut intro)?;
    ensure!(intro == original_intro, "unbound introductory Pokemon");
    Ok(())
}
