use anyhow::{Context, Result, ensure};
use serde_json::json;
use std::{collections::BTreeMap, path::{Path, PathBuf}};

fn files(root: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() { files(&path, output)?; }
        else if path.extension().is_some_and(|extension| extension == "asm") { output.push(path); }
    }
    Ok(())
}
pub fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let mut paths = Vec::new();
    for directory in ["data/text", "data/phone/text"] { files(&vendor.join(directory), &mut paths)?; }
    paths.sort();
    let mut texts = BTreeMap::new();
    for path in paths {
        for (label, commands) in super::source_scripts::parse(&std::fs::read_to_string(&path)?)? {
            let mut value = String::new();
            for row in commands.as_array().context("source text program")? {
                let command = row["command"].as_str().context("source text opcode")?;
                let args = row["args"].as_array().context("source text operands")?;
                if matches!(command, "text" | "line" | "cont" | "next" | "para") {
                    if matches!(command, "line" | "cont" | "next") { value.push('\n'); }
                    if command == "para" { value.push_str("\n\n"); }
                    let text = args.first().and_then(|value| value.as_str()).context("source quoted text")?;
                    ensure!(text.starts_with('"') && text.ends_with('"'), "source quoted text {label}");
                    value.push_str(text.trim_matches('"').trim_end_matches('@'));
                } else if command == "text_ram" {
                    let reference = args.first().and_then(|value| value.as_str()).context("source text RAM reference")?;
                    if let Some(index) = reference.strip_prefix("wStringBuffer") { value.push_str(&format!("<STRING_BUFFER_{index}>")); }
                    else { value.push_str(&format!("<{reference}>")); }
                } else if command == "text_decimal" {
                    let count = args.last().and_then(|value| value.as_str()).context("source decimal digit count")?.parse::<usize>()?;
                    value.push_str(&"@".repeat(count));
                } else if command == "text_today" { value.push_str("<TODAY>"); }
                else if command == "text_pause" { value.push('…'); }
                else if matches!(command, "done" | "prompt" | "text_end" | "text_promptbutton") { break; }
            }
            let value = value.trim().to_owned();
            if !value.is_empty() { ensure!(texts.insert(label.clone(), value).is_none(), "duplicate global text {label}"); }
        }
    }
    ensure!(!texts.is_empty(), "source ASM text catalog");
    ensure!(crystal_core::systems::script_text::asm_text_catalog_issues(&texts).is_empty(), "source ASM text shape");
    let directory = data.join("content-packs/core-modular/asm_text");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("Source.json"), serde_json::to_vec_pretty(&json!(texts))?)?;
    std::fs::write(data.join("asm_text.json"), serde_json::to_vec_pretty(&json!(texts))?)?;
    Ok(())
}
