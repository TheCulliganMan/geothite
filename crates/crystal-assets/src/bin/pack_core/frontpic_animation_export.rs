use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

fn source_byte(value: &str) -> Result<u8> {
    let value = value.trim();
    Ok(if let Some(hex) = value.strip_prefix('$') {
        u8::from_str_radix(hex, 16)?
    } else {
        value.parse()?
    })
}

fn parse_program(source: &str) -> Result<Value> {
    let mut commands = Vec::new();
    let mut ended = false;
    for raw in source.lines() {
        let code = raw.split_once(';').map_or(raw, |(code, _)| code).trim();
        if code.is_empty() {
            continue;
        }
        anyhow::ensure!(!ended, "frontpic source has commands after endanim");
        let (kind, args) = code
            .split_once(char::is_whitespace)
            .map_or((code, ""), |parts| parts);
        let operands = if args.trim().is_empty() {
            Vec::new()
        } else {
            args.split(',')
                .map(source_byte)
                .collect::<Result<Vec<_>>>()?
        };
        let command = match (kind, operands.as_slice()) {
            ("frame", [frame, duration]) => {
                anyhow::ensure!(*frame < 253, "frontpic frame uses a reserved opcode");
                json!({"kind":"frame", "frame":frame, "duration":duration})
            }
            ("setrepeat", [count]) => json!({"kind":"setrepeat", "count":count}),
            ("dorepeat", [target]) => json!({"kind":"dorepeat", "target":target}),
            ("endanim", []) => {
                ended = true;
                json!({"kind":"endanim"})
            }
            _ => anyhow::bail!("invalid source frontpic command {code:?}"),
        };
        commands.push(command);
    }
    anyhow::ensure!(
        ended && commands.len() <= 256,
        "frontpic source must end within its byte command range"
    );
    let program = json!({"commands":commands});
    // Use the same typed command/target validation as the compiled payload.
    let _: crystal_core::models::frontpic_anim::FrontpicAnimProgram =
        serde_json::from_value(program.clone())?;
    Ok(program)
}

fn source_programs(repository: &Path) -> Result<BTreeMap<String, Value>> {
    let mut programs = BTreeMap::new();
    for entry in std::fs::read_dir(repository.join("vendor/pokecrystal/gfx/pokemon"))? {
        let path = entry?.path();
        if !path.is_dir() || !path.join("anim.asm").exists() {
            continue;
        }
        let asset = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("frontpic source directory has no UTF-8 asset name")?
            .to_ascii_uppercase();
        for (suffix, source) in [("", "anim.asm"), ("_IDLE", "anim_idle.asm")] {
            let source_path = path.join(source);
            let source = std::fs::read_to_string(&source_path)
                .with_context(|| format!("read source animation {}", source_path.display()))?;
            let program = parse_program(&source)
                .with_context(|| format!("parse source animation {}", source_path.display()))?;
            anyhow::ensure!(
                programs
                    .insert(format!("{asset}{suffix}"), program)
                    .is_none(),
                "duplicate frontpic source asset {asset}{suffix}"
            );
        }
    }
    anyhow::ensure!(!programs.is_empty(), "source frontpic catalog is empty");
    Ok(programs)
}

pub fn export(repository: &Path) -> Result<()> {
    let programs = source_programs(repository)?;
    let mut bytes = serde_json::to_vec_pretty(&programs)?;
    bytes.push(b'\n');
    for relative in [
        "pokemon_frontpic_anim.json",
        "content-packs/core-modular/pokemon_frontpic_anim/programs.json",
    ] {
        std::fs::write(
            repository.join("apps/web/assets/data").join(relative),
            &bytes,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_main_and_idle_programs_match_rom_bytes() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let programs = source_programs(&repository).unwrap();
        assert_eq!(
            programs.len(),
            556,
            "278 source assets each have main and idle programs"
        );
        let trace: Value = serde_json::from_slice(
            &std::fs::read(
                repository.join("tools/asm-oracle/fixtures/frontpic-animation-cyndaquil.json"),
            )
            .unwrap(),
        )
        .unwrap();
        for call in trace["calls"].as_array().unwrap() {
            let asset = match call["before"]["scene"].as_u64().unwrap() {
                2 => "CYNDAQUIL",
                6 => "CYNDAQUIL_IDLE",
                other => panic!("unexpected source scene {other}"),
            };
            assert_eq!(programs[asset], call["program"], "{asset}");
        }
    }

    #[test]
    fn malformed_or_incomplete_animation_source_is_rejected() {
        for source in [
            "frame 1\nendanim",
            "frame 1, 256\nendanim",
            "endanim\nframe 0, 1",
            "frame 1, 2",
            "dorepeat 5\nendanim",
            "unknown 0\nendanim",
        ] {
            assert!(parse_program(source).is_err(), "{source}");
        }
        assert!(
            parse_program("frame 0, 0\nendanim").is_ok(),
            "zero duration is source-valid"
        );
    }
}
