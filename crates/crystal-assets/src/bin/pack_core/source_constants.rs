use anyhow::{Context,Result,ensure};
use serde_json::json;
use std::{collections::BTreeMap,path::Path,process::Command};
use super::radio_text_export::code_part;

pub(super) fn export(root: &Path) -> Result<()> {
    let vendor=root.join("vendor/pokecrystal");
    let build=root.join("source-build"); std::fs::create_dir_all(&build)?;
    let assembly=build.join("constants-input.asm"); std::fs::write(&assembly,"")?;
    let definitions=build.join("constants.defs"); let object=build.join("constants.o");
    let executable=std::env::var_os("RGBASM").map(std::path::PathBuf::from).unwrap_or_else(|| {
        std::env::var_os("RGBDS").map(std::path::PathBuf::from).map_or_else(||"rgbasm".into(),|path|path.join("rgbasm"))
    });
    let version=Command::new(&executable).arg("--version").output().context("run pinned RGBDS assembler; set RGBDS to its directory")?;
    ensure!(version.status.success() && String::from_utf8_lossy(&version.stdout).trim()=="rgbasm v1.0.4","RGBASM must be pinned version 1.0.4");
    let status=Command::new(&executable).current_dir(&vendor).args(["-Q8","-P","includes.asm","-s"]).arg(format!("equ:{}",definitions.display())).arg("-o").arg(&object).arg(&assembly).output()?;
    ensure!(status.status.success(),"assemble source constants: {}",String::from_utf8_lossy(&status.stderr));
    let mut global=BTreeMap::new();
    for line in std::fs::read_to_string(&definitions)?.lines().map(code_part) {
        if line.is_empty() {continue;}
        let words=line.split_whitespace().collect::<Vec<_>>();
        ensure!(words.len()==4 && words[0]=="def" && words[2]=="equ","invalid assembler constant definition {line}");
        let value=u32::from_str_radix(words[3].strip_prefix('$').context("assembler constant value")?,16)?;
        if words[1].starts_with("__") || !words[1].bytes().all(|byte|byte.is_ascii_alphanumeric() || byte==b'_') {continue;}
        ensure!(global.insert(words[1].to_owned(),i64::from(value as i32)).is_none(),"duplicate source constant");
    }
    ensure!(global.contains_key("EGG_LEVEL"),"required battle constants absent");
    let mut maps=BTreeMap::new();
    for entry in std::fs::read_dir(vendor.join("maps"))? {
        let path=entry?.path(); if !path.extension().is_some_and(|extension|extension=="asm") {continue;}
        let mut locals=BTreeMap::new(); let mut counter=None; let mut macro_body=false;
        for line in std::fs::read_to_string(&path)?.lines().map(code_part) {
            if line.starts_with("MACRO ") { macro_body=true; continue; }
            if line == "ENDM" { macro_body=false; continue; }
            if macro_body { continue; }
            if let Some(definition) = line.strip_prefix("DEF ")
                && let Some((name, expression)) = definition.split_once(" EQU ") {
                let mut scope = global.clone();
                scope.extend(locals.clone());
                let value = super::source_expressions::evaluate(expression, &scope)
                    .with_context(|| format!("map source constant {name}"))?;
                locals.insert(name.trim().to_owned(), value);
                if name.ends_with("_PRICE") || name.ends_with("_COST") {
                    if let Some(previous) = global.insert(name.trim().to_owned(), value) {
                        ensure!(previous == value, "conflicting authored currency constant {name}");
                    }
                }
                continue;
            }
            if line=="object_const_def" {counter=Some(1);continue;}
            if line=="scene_const_def" {counter=Some(0);continue;}
            if let Some(name)=line.strip_prefix("const ") {
                if let Some(value)=counter.as_mut() {locals.insert(name.to_owned(),*value);*value+=1;}
            } else if !line.is_empty() {counter=None;}
        }
        maps.insert(path.file_stem().and_then(|name|name.to_str()).context("map name")?.to_owned(),locals);
    }
    let value=json!({"global":global,"maps":maps});
    let _:crystal_core::systems::script_runtime::StoryEventScriptConstants=serde_json::from_value(value.clone())?;
    let data=root.join("apps/web/assets/data"); let output=data.join("content-packs/core-modular/story_event_script_constants");std::fs::create_dir_all(&output)?;
    let bytes=serde_json::to_vec_pretty(&value)?;
    std::fs::write(data.join("story_event_script_constants.json"),&bytes)?;
    std::fs::write(output.join("constants.json"),bytes)?;
    Ok(())
}
