//! Source PlayMusic/PlaySFX caller tapes, selected by live SFX ownership.
use super::super::source_intro_oracle::{measure_routine_state, symbols};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) fn export(vendor: &Path) -> Result<Value> {
    let symbols = symbols(&std::fs::read_to_string(vendor.join("pokecrystal.sym"))?)?;
    let source = std::fs::read_to_string(vendor.join("engine/movie/intro.asm"))?;
    let used = source
        .lines()
        .flat_map(|line| {
            line.split(';')
                .next()
                .unwrap_or("")
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        })
        .filter(|word| word.starts_with("MUSIC_") || word.starts_with("SFX_"))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let mut ids = BTreeMap::new();
    for file in [
        "constants/music_constants.asm",
        "constants/sfx_constants.asm",
    ] {
        let mut id = 0u16;
        for line in std::fs::read_to_string(vendor.join(file))?.lines() {
            let line = line.split(';').next().unwrap_or("").trim();
            if let Some(name) = line
                .strip_prefix("const ")
                .and_then(|s| s.split_whitespace().next())
            {
                if used.contains(name) {
                    ids.insert(name.to_string(), id);
                }
                id += 1;
            }
        }
    }
    ensure!(
        ids.len() == used.len(),
        "intro caller assets have unresolved authored identities"
    );
    let off = *symbols.get("MusicOff").context("source MusicOff")?;
    let on = *symbols.get("MusicOn").context("source MusicOn")?;
    let build = |name: &str, id: u16, mask: u8, rejected: bool| -> Result<Value> {
        let mut writes = vec![
            ("hCGB".to_owned(), 1),
            ("hROMBank".to_owned(), 58),
            ("wCurSFX".to_owned(), if rejected { 0 } else { id as u8 }),
        ];
        for bit in 0..4 {
            writes.push((
                format!("wChannel{}Flags1", bit + 5),
                if mask & (1 << bit) != 0 { 9 } else { 0 },
            ));
        }
        let label = if name.starts_with("MUSIC_") {
            "PlayMusic"
        } else {
            "PlaySFX"
        };
        let measured = measure_routine_state(vendor, label, &writes, &[("DE", id)])?;
        let mut tape = Vec::new();
        let mut effects = Vec::new();
        let mut playback = None;
        for (index, instruction) in measured["instructions"]
            .as_array()
            .context("source audio caller tape")?
            .iter()
            .enumerate()
        {
            tape.push(
                instruction["machine_cycles"]
                    .as_u64()
                    .context("source caller instruction cost")?,
            );
            let location = (
                instruction["bank"].as_u64().context("caller bank")? as usize,
                instruction["address"].as_u64().context("caller address")? as u16,
            );
            if location == (off.0, off.1 + 1) {
                effects.push(json!({"instruction_index":index,"music_playing":0}));
            }
            if location == (on.0, on.1 + 2) {
                effects.push(json!({"instruction_index":index,"music_playing":1}));
                playback = Some(index);
            }
        }
        ensure!(
            rejected == playback.is_none(),
            "source caller accept guard disagrees for {name}, mask {mask}"
        );
        Ok(
            json!({"instruction_machine_cycles":tape,"music_playing_effects":effects,"playback_instruction_index":playback,"accepted":!rejected}),
        )
    };
    let mut entries = BTreeMap::new();
    for (name, id) in ids {
        if name.starts_with("MUSIC_") {
            entries.insert(
                name.clone(),
                json!({"source_id":id,"music":build(&name,id,0,false)?}),
            );
        } else {
            let mut accepted = Vec::new();
            let mut rejected = Vec::new();
            for mask in 0..16 {
                accepted.push(build(&name, id, mask, false)?);
                rejected.push(if mask == 0 {
                    Value::Null
                } else {
                    build(&name, id, mask, true)?
                });
            }
            entries.insert(
                name.clone(),
                json!({"source_id":id,"sfx_accepted":accepted,"sfx_rejected":rejected}),
            );
        }
    }
    Ok(json!({"schema_version":1,"callers":entries}))
}
