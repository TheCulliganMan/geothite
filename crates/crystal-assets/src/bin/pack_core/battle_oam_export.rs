use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

fn number(value: &str) -> Result<i64> {
    let value = value.trim();
    Ok(if let Some(hex) = value.strip_prefix('$') {
        i64::from_str_radix(hex, 16)?
    } else {
        value.parse()?
    })
}

fn parse_oam(source: &str) -> Result<Value> {
    let mut labels = BTreeMap::new();
    let mut pieces = Vec::new();
    let mut table = Vec::new();
    for raw in source.lines() {
        let (code, comment) = raw.split_once(';').unwrap_or((raw, ""));
        let code = code.trim();
        if let Some(label) = code.strip_suffix(':') {
            labels.insert(label.to_string(), pieces.len());
        } else if let Some(args) = code.strip_prefix("battleanimoam ") {
            let args: Vec<_> = args.split(',').map(str::trim).collect();
            anyhow::ensure!(args.len() == 3, "invalid battleanimoam row");
            table.push((
                comment.trim().to_string(),
                number(args[0])?,
                usize::try_from(number(args[1])?)?,
                args[2].to_string(),
            ));
        } else if let Some(args) = code.strip_prefix("dbsprite ") {
            let args: Vec<_> = args.split(',').map(str::trim).collect();
            anyhow::ensure!(args.len() == 6, "invalid dbsprite row");
            let flags = args[5]
                .split('|')
                .try_fold(0_i64, |flags, token| -> Result<i64> {
                    Ok(flags
                        | match token.trim() {
                            "OAM_XFLIP" => 32,
                            "OAM_YFLIP" => 64,
                            "OAM_PAL1" => 16,
                            value => number(value)?,
                        })
                })?;
            let x = (number(args[0])? * 8 + number(args[2])?) as u8 as i8;
            let y = (number(args[1])? * 8 + number(args[3])?) as u8 as i8;
            pieces.push(json!({"x":x,"y":y,"tile_id":number(args[4])?,"xflip":flags & 32 != 0,"yflip":flags & 64 != 0,"obp":(flags >> 4)&1,"attributes":flags}));
        }
    }
    let mut result = serde_json::Map::new();
    for (name, offset, count, label) in table {
        anyhow::ensure!(
            name.starts_with("BATTLE_ANIM_OAMSET_"),
            "missing OAM constant"
        );
        let start = *labels
            .get(&label)
            .with_context(|| format!("missing OAM label {label}"))?;
        let entries = pieces
            .get(start..start + count)
            .context("OAM count exceeds source data")?;
        result.insert(
            name.clone(),
            json!({"name":name,"tile_offset":offset,"entries":entries}),
        );
    }
    anyhow::ensure!(!result.is_empty(), "missing battle OAM table");
    Ok(Value::Object(result))
}

pub fn export(repository: &Path) -> Result<()> {
    use sha2::{Digest, Sha256};
    let program = repository.join("rust/crates/crystal-bevy/src/battle_anim_program");
    let manifest: Value = serde_json::from_slice(&std::fs::read(program.join("provenance.json"))?)?;
    for (name, expected) in manifest["sources"]
        .as_object()
        .context("missing battle program source hashes")?
    {
        let bytes = std::fs::read(repository.join("vendor/pokecrystal").join(name))?;
        let actual = format!("{:x}", Sha256::digest(bytes));
        anyhow::ensure!(
            Some(actual.as_str()) == expected.as_str(),
            "battle animation source {name} changed; rebuild the reference ROM and regenerate export_battle_program.py"
        );
    }
    let actual = format!(
        "{:x}",
        Sha256::digest(std::fs::read(program.join("bank.bin"))?)
    );
    anyhow::ensure!(
        Some(actual.as_str()) == manifest["bank_sha256"].as_str(),
        "battle instruction bank differs from generated provenance"
    );
    let source =
        std::fs::read_to_string(repository.join("vendor/pokecrystal/data/battle_anims/oam.asm"))?;
    let oam = parse_oam(&source)?;
    for relative in [
        "battle_anim_bundle.json",
        "content-packs/core-modular/battle_anim_bundle/bundle.json",
    ] {
        let path = repository.join("apps/web/assets/data").join(relative);
        let mut bundle: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        bundle["oam_sets"] = oam.clone();
        let mut bytes = serde_json::to_vec_pretty(&bundle)?;
        bytes.push(b'\n');
        std::fs::write(path, bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn count_continues_across_labels_and_preserves_attributes() {
        let source = "battleanimoam $05, 2, .first ; BATTLE_ANIM_OAMSET_60\n.first:\n dbsprite -1, -1, 4, 0, $00, $0\n.second:\n dbsprite -1, 0, 4, 0, $00, $1 | OAM_PAL1\n";
        let data = parse_oam(source).unwrap();
        let entries = data["BATTLE_ANIM_OAMSET_60"]["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1]["y"], 0);
        assert_eq!(entries[1]["attributes"], 17);
    }
    #[test]
    fn every_oam_piece_matches_pinned_cartridge() {
        let data = parse_oam(include_str!(
            "../../../../../../vendor/pokecrystal/data/battle_anims/oam.asm"
        ))
        .unwrap();
        let bank = include_bytes!("../../../../crystal-bevy/src/battle_anim_program/bank.bin");
        for (id, (_, value)) in data.as_object().unwrap().iter().enumerate() {
            let row = &bank[0x2eae + id * 4..0x2eae + id * 4 + 4];
            assert_eq!(value["tile_offset"], row[0]);
            let entries = value["entries"].as_array().unwrap();
            assert_eq!(entries.len(), usize::from(row[1]));
            let address = usize::from(u16::from_le_bytes([row[2], row[3]])) - 0x4000;
            for (piece, entry) in entries.iter().enumerate() {
                let actual = [
                    entry["y"].as_i64().unwrap() as u8,
                    entry["x"].as_i64().unwrap() as u8,
                    entry["tile_id"].as_u64().unwrap() as u8,
                    entry["attributes"].as_u64().unwrap() as u8,
                ];
                assert_eq!(
                    actual,
                    bank[address + piece * 4..address + piece * 4 + 4],
                    "set {id} piece {piece}"
                );
            }
        }
    }
}
