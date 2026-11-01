use anyhow::{Context,Result,ensure};
use serde_json::json;
use std::{collections::BTreeMap,path::Path,process::Command};
use super::radio_text_export::code_part;

// macros/scripts/maps.asm resets the scene counter to zero and advances it
// for every scene_script, even when that slot exports no named constant.
fn scene_constants(source: &str) -> Result<BTreeMap<String,i64>> {
    let mut constants = BTreeMap::new();
    let mut ordinal = None::<i64>;
    let mut macro_body = false;
    for line in source.lines().map(code_part) {
        if line.starts_with("MACRO ") { macro_body = true; continue; }
        if line == "ENDM" { macro_body = false; continue; }
        if macro_body { continue; }
        if line == "def_scene_scripts" { ordinal = Some(0); continue; }
        if line == "def_callbacks" { ordinal = None; continue; }
        if let Some(arguments) = line.strip_prefix("scene_script ") {
            let arguments = super::radio_text_export::source_arguments(arguments)?;
            ensure!((1..=2).contains(&arguments.len()), "source scene_script must have pointer and optional constant");
            let index = ordinal.as_mut().context("source scene_script precedes def_scene_scripts")?;
            if let Some(name) = arguments.get(1) {
                ensure!(constants.insert(name.clone(),*index).is_none(), "duplicate source scene constant {name}");
            }
            *index = index.checked_add(1).context("source scene ordinal overflowed")?;
        }
    }
    Ok(constants)
}

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
        let map_source = std::fs::read_to_string(&path)?;
        let mut locals = scene_constants(&map_source).with_context(|| format!("map scene constants {}",path.display()))?;
        for (name,value) in &locals {
            if let Some(previous) = global.insert(name.clone(),*value) {
                ensure!(previous == *value,"conflicting exported source scene constant {name}");
            }
        }
        let mut counter=None; let mut macro_body=false;
        for line in map_source.lines().map(code_part) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_script_constants_preserve_named_and_unnamed_slot_ordinals() {
        let constants = scene_constants("Map_MapScripts:\n def_scene_scripts\n scene_script First, SCENE_FIRST\n scene_script Unnamed\n scene_script Third, SCENE_THIRD ; source comment\n def_callbacks\n callback MAPCALLBACK_OBJECTS, Callback\n").unwrap();
        assert_eq!(constants,BTreeMap::from([("SCENE_FIRST".to_string(),0),("SCENE_THIRD".to_string(),2)]));
        assert!(scene_constants("scene_script MissingHeader, SCENE_INVALID\n").is_err());
    }

    #[test]
    fn ruins_research_scene_constants_match_pinned_assembler_symbols() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else { return; };
        let vendor = Path::new(&vendor);
        let constants = scene_constants(&std::fs::read_to_string(vendor.join("maps/RuinsOfAlphResearchCenter.asm")).unwrap()).unwrap();
        assert_eq!(constants["SCENE_RUINSOFALPHRESEARCHCENTER_NOOP"],0);
        assert_eq!(constants["SCENE_RUINSOFALPHRESEARCHCENTER_GET_UNOWN_DEX"],1);
        let symbols = std::fs::read_to_string(vendor.join("pokecrystal.sym")).unwrap();
        for (name,value) in constants {
            let words = symbols.lines().map(|line|line.split_whitespace().collect::<Vec<_>>())
                .find(|words| words.len() == 2 && words[1] == name).unwrap();
            assert_eq!(i64::from_str_radix(words[0],16).unwrap(),value,"assembler scene constant {name}");
        }
    }

    #[test]
    fn authored_map_scene_ordinals_match_all_exported_assembler_constants() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else { return; };
        let vendor = Path::new(&vendor);
        let source = std::fs::read_to_string(vendor.join("pokecrystal.sym")).unwrap();
        let symbols = source.lines().filter_map(|line| {
            let words = line.split_whitespace().collect::<Vec<_>>();
            if words.len() != 2 || words[0].contains(':') { return None; }
            i64::from_str_radix(words[0],16).ok().map(|value|(words[1].to_string(),value))
        }).collect::<BTreeMap<_,_>>();
        let mut compared = 0;
        for entry in std::fs::read_dir(vendor.join("maps")).unwrap() {
            let path = entry.unwrap().path();
            if !path.extension().is_some_and(|extension|extension == "asm") { continue; }
            for (name,value) in scene_constants(&std::fs::read_to_string(&path).unwrap()).unwrap() {
                // Source contains dormant, unassembled maps too. Compare every
                // constant independently published by the pinned assembler.
                if let Some(assembled) = symbols.get(&name) {
                    assert_eq!(*assembled,value,"{} scene constant {name}",path.display());
                    compared += 1;
                }
            }
        }
        assert!(compared > 100,"insufficient independent scene constant coverage: {compared}");
    }
}
