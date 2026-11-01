//! Lower the decoration name dispatcher, preserving its separate name domains.
use anyhow::{Context, Result, ensure};
use crystal_assets::{DecorationCatalog, DecorationCategory, DecorationDefinition};
use serde_json::Value;
use std::path::Path;
#[path = "source_syntax.rs"]
mod source_syntax;
use source_syntax::{code_part as code, source_arguments};

fn strings(source: &str, command: &str) -> Result<Vec<String>> {
    source
        .lines()
        .filter_map(|line| code(line).strip_prefix(command))
        .map(|raw| {
            Ok(raw
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .context("authored quoted decoration name")?
                .to_owned())
        })
        .collect()
}

pub(super) fn display_name(
    kind: &str,
    name: &str,
    constants: &Value,
    names: &[String],
    pokemon: &[String],
) -> Result<String> {
    let index = |token: &str| -> Result<usize> {
        Ok(usize::try_from(constants[token].as_u64().with_context(
            || format!("missing decoration name constant {token}"),
        )?)?)
    };
    let deco = |token: &str| -> Result<&str> {
        Ok(names
            .get(index(token)?)
            .context("decoration name index")?
            .as_str())
    };
    let mon = || -> Result<&str> {
        Ok(pokemon
            .get(
                index(name)?
                    .checked_sub(1)
                    .context("zero decoration species index")?,
            )
            .context("decoration species name index")?
            .as_str())
    };
    // GetDecoName dispatches by attribute TYPE, not action or token spelling.
    Ok(match kind {
        "DECO_PLANT" => deco(name)?.to_owned(),
        "DECO_BED" => format!("{}{}", deco(name)?, deco("_BED")?),
        "DECO_CARPET" => format!("{}{}", deco(name)?, deco("_CARPET")?),
        "DECO_POSTER" => format!("{}{}", mon()?, deco("_POSTER")?),
        "DECO_DOLL" => format!("{}{}", mon()?, deco("_DOLL")?),
        // Source .bigdoll appends only the species name, without a DOLL suffix.
        "DECO_BIGDOLL" => format!("{}{}", deco("BIG_")?, mon()?),
        _ => anyhow::bail!("unsupported decoration name type {kind}"),
    })
}

pub(super) fn export(vendor: &Path, constants: &Value) -> Result<DecorationCatalog> {
    let read = |path: &str| -> Result<String> { Ok(std::fs::read_to_string(vendor.join(path))?) };
    let names = strings(&read("data/decorations/names.asm")?, "li ")?;
    let pokemon_source = read("data/pokemon/names.asm")?;
    let species_source = pokemon_source
        .split_once("assert_table_length NUM_POKEMON")
        .context("authored Pokemon name catalog boundary")?
        .0;
    let pokemon = strings(species_source, "dname ")?;
    ensure!(
        names.len() == 26 && pokemon.len() == 251,
        "authored decoration and species name table lengths"
    );
    let attributes = read("data/decorations/attributes.asm")?;
    let rows = attributes
        .lines()
        .filter_map(|line| code(line).strip_prefix("decoration "))
        .map(source_arguments)
        .collect::<Result<Vec<_>>>()?;
    let mut decorations = Vec::new();
    for line in read("data/decorations/decorations.asm")?.lines().map(code) {
        let Some(id) = line
            .strip_prefix("db ")
            .filter(|id| id.starts_with("DECO_"))
        else {
            continue;
        };
        let index = u8::try_from(
            constants[id]
                .as_u64()
                .with_context(|| format!("missing decoration index {id}"))?,
        )?;
        let row = rows
            .get(usize::from(index))
            .context("decoration attribute index")?;
        ensure!(row.len() == 5, "authored decoration fields for {id}");
        let category = match row[2].as_str() {
            "SET_UP_BED" => DecorationCategory::Bed,
            "SET_UP_CARPET" => DecorationCategory::Carpet,
            "SET_UP_PLANT" => DecorationCategory::Plant,
            "SET_UP_POSTER" => DecorationCategory::Poster,
            "SET_UP_CONSOLE" => DecorationCategory::GameConsole,
            "SET_UP_DOLL" => DecorationCategory::Ornament,
            "SET_UP_BIG_DOLL" => DecorationCategory::BigDoll,
            action => anyhow::bail!("unknown decoration action {action}"),
        };
        decorations.push(DecorationDefinition {
            index,
            id: id.into(),
            category,
            display_name: display_name(&row[0], &row[1], constants, &names, &pokemon)?,
            action: row[2].clone(),
            event_flag: row[3].clone(),
            sprite: row[4].clone(),
        });
    }
    ensure!(
        decorations.len() == 45,
        "authored decoration ownership list"
    );
    Ok(DecorationCatalog {
        category_order: vec![
            DecorationCategory::Bed,
            DecorationCategory::Carpet,
            DecorationCategory::Plant,
            DecorationCategory::Poster,
            DecorationCategory::GameConsole,
            DecorationCategory::Ornament,
            DecorationCategory::BigDoll,
        ],
        decorations,
    })
}
