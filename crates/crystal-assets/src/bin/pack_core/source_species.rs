//! Pokemon and move catalogs generated exclusively from external source tables.
use super::radio_text_export::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn rows(source: &str, command: &str) -> Result<Vec<Vec<String>>> {
    source
        .lines()
        .filter_map(|line| code_part(line).strip_prefix(&format!("{command} ")))
        .map(source_arguments)
        .collect()
}
fn number(value: &str) -> Result<u64> {
    Ok(value.trim().parse()?)
}
fn write(data: &Path, category: &str, values: &BTreeMap<String, Value>) -> Result<()> {
    let directory = data.join("content-packs/core-modular").join(category);
    std::fs::create_dir_all(&directory)?;
    for (id, value) in values {
        std::fs::write(
            directory.join(format!("{id}.json")),
            serde_json::to_vec_pretty(&json!({id: value}))?,
        )?;
    }
    std::fs::write(
        data.join(format!("{category}.json")),
        serde_json::to_vec_pretty(values)?,
    )?;
    Ok(())
}
pub(super) fn export(repository: &Path) -> Result<()> {
    let vendor = repository.join("vendor/pokecrystal");
    let read = |name: &str| -> Result<String> { Ok(std::fs::read_to_string(vendor.join(name))?) };
    let mut ids = BTreeMap::new();
    let mut index = 0u64;
    for line in read("constants/pokemon_constants.asm")?.lines() {
        if code_part(line).starts_with("DEF NUM_POKEMON ") {
            break;
        }
        let words = code_part(line).split_whitespace().collect::<Vec<_>>();
        match words.as_slice() {
            ["const_def"] => index = 0,
            ["const_def", value] => index = number(value)?,
            ["const", name] => {
                if (1..=251).contains(&index) {
                    ensure!(
                        ids.insert(name.to_string(), index).is_none(),
                        "duplicate species identity"
                    );
                }
                index += 1;
            }
            _ => {}
        }
    }
    ensure!(
        ids.len() == 251,
        "source species identities must contain 251 entries"
    );
    let mut species = BTreeMap::new();
    for (id, index) in ids {
        let source = read(&format!(
            "data/pokemon/base_stats/{}.asm",
            id.to_lowercase()
        ))?;
        let fields = rows(&source, "db")?;
        ensure!(
            fields.len() == 11 && fields[0] == [id.clone()],
            "invalid base-stat fields for {id}"
        );
        ensure!(
            fields[1].len() == 6 && fields[2].len() == 2 && fields[5].len() == 2,
            "invalid base-stat shape for {id}"
        );
        let stats = fields[1]
            .iter()
            .map(|value| number(value))
            .collect::<Result<Vec<_>>>()?;
        let gender = match fields[6][0].as_str() {
            "GENDER_F0" => 0,
            "GENDER_F12_5" => 31,
            "GENDER_F25" => 63,
            "GENDER_F50" => 127,
            "GENDER_F75" => 191,
            "GENDER_F100" => 254,
            "GENDER_UNKNOWN" => 255,
            other => anyhow::bail!("unsupported gender constant {other}"),
        };
        let eggs = rows(&source, "dn")?;
        ensure!(
            eggs.len() == 1 && eggs[0].len() == 2,
            "missing source egg groups for {id}"
        );
        let learnset = rows(&source, "tmhm")?;
        ensure!(learnset.len() <= 1, "duplicate TM/HM learnset for {id}");
        let png = source
            .lines()
            .find_map(|line| code_part(line).strip_prefix("INCBIN "))
            .context("frontpic dimensions source")?;
        let png = png
            .trim_matches('"')
            .strip_suffix(".dimensions")
            .context("source dimensions path")?;
        let png_bytes = std::fs::read(vendor.join(format!("{png}.png")))?;
        ensure!(
            png_bytes.len() >= 24 && &png_bytes[..8] == b"\x89PNG\r\n\x1a\n",
            "invalid source frontpic PNG for {id}"
        );
        let width = u32::from_be_bytes(png_bytes[16..20].try_into()?);
        ensure!(
            matches!(width, 40 | 48 | 56),
            "invalid frontpic width for {id}"
        );
        let dimensions = (width / 8) * 17;
        let dex = read(&format!(
            "data/pokemon/dex_entries/{}.asm",
            id.to_lowercase()
        ))?;
        let measures = rows(&dex, "dw")?;
        ensure!(
            measures.len() == 1 && measures[0].len() == 2,
            "missing source weight for {id}"
        );
        let held = |name: &str| {
            if name == "NO_ITEM" {
                Value::Null
            } else {
                json!(name)
            }
        };
        let value = json!({"id":id,"int_id":index,
            "base_stats":{"hp":stats[0],"attack":stats[1],"defense":stats[2],"speed":stats[3],"special_attack":stats[4],"special_defense":stats[5]},
            "type1":fields[2][0],"type2":fields[2][1],"catch_rate":number(&fields[3][0])?,"base_exp":number(&fields[4][0])?,
            "item1":held(&fields[5][0]),"item2":held(&fields[5][1]),"gender_ratio":gender,"unknown1":number(&fields[7][0])?,
            "step_cycles_to_hatch":number(&fields[8][0])?,"unknown2":number(&fields[9][0])?,"growth_rate":fields[10][0],
            "egg_group1":eggs[0][0],"egg_group2":eggs[0][1],"tmhm_learnset":learnset.first().cloned().unwrap_or_default(),
            "ability":"NONE","pic_size":dimensions,"front_pic":0,"back_pic":0,"weight":number(&measures[0][1])?});
        let _: crystal_core::models::PokemonSpecies = serde_json::from_value(value.clone())
            .with_context(|| format!("validate source species {id}"))?;
        species.insert(id, value);
    }
    let mut moves = BTreeMap::new();
    for (index, fields) in rows(&read("data/moves/moves.asm")?, "move")?
        .into_iter()
        .enumerate()
    {
        ensure!(fields.len() == 7, "invalid move source row");
        let effect = fields[1]
            .strip_prefix("EFFECT_")
            .context("source move effect")?;
        let effect = effect.replace("SP_ATK_", "SPECIAL_ATTACK_").replace("SP_DEF_", "SPECIAL_DEFENSE_");
        let value = json!({"source_index":index + 1,"name":fields[0],"effect":effect,"power":number(&fields[2])?,
            "type":fields[3],"accuracy":number(&fields[4])?,"pp":number(&fields[5])?,"effect_chance":number(&fields[6])?,"stat":null,"amount":null});
        let _: crystal_core::models::Move = serde_json::from_value(value.clone())
            .with_context(|| format!("validate source move {}", fields[0]))?;
        ensure!(
            moves.insert(fields[0].clone(), value).is_none(),
            "duplicate source move"
        );
    }
    ensure!(
        moves.len() == 251,
        "source move table must contain 251 entries"
    );
    let data = repository.join("apps/web/assets/data");
    std::fs::create_dir_all(&data)?;
    write(&data, "pokemon", &species)?;
    write(&data, "moves", &moves)?;
    Ok(())
}
