use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
use super::radio_text_export::{code_part, source_arguments};

pub(super) fn export(root: &Path, oam: &Value) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let read = |name: &str| std::fs::read_to_string(vendor.join(name));
    let constants: Value = serde_json::from_slice(&std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json"))?)?;
    let numeric = |token: &str| -> Result<i64> {
        if let Some(value) = token.strip_prefix('$') { return Ok(i64::from_str_radix(value,16)?); }
        if token == "ABSOLUTE_X" {return Ok(0);} if token == "RELATIVE_X" {return Ok(1);}
        token.parse().ok().or_else(|| constants["global"][token].as_i64()).with_context(|| format!("unknown battle constant {token}"))
    };
    let object_names: Vec<_> = read("constants/battle_anim_constants.asm")?.lines().map(code_part)
        .filter_map(|line|line.strip_prefix("const BATTLE_ANIM_OBJ_").map(|name|format!("BATTLE_ANIM_OBJ_{}",name.trim()))).collect();
    let mut objects = BTreeMap::new();
    for line in read("data/battle_anims/objects.asm")?.lines().map(code_part) {
        let Some(raw) = line.strip_prefix("battleanimobj ") else {continue};
        let args = source_arguments(raw)?; ensure!(args.len()==6,"invalid battle object");
        let name = object_names.get(objects.len()).context("battle object exceeds source constants")?;
        let flags = args[0].split('|').try_fold(0, |sum, token| Ok::<_,anyhow::Error>(sum | numeric(token.trim())?))?;
        objects.insert(name.clone(),json!({"object_id":name,"flags":flags,"fix_y":numeric(&args[1])?,"frameset":args[2],"function":if args[3]=="BATTLE_ANIM_FUNC_NULL" {None} else {Some(&args[3])},"palette":args[4],"gfx_id":args[5]}));
    }
    ensure!(objects.len()==object_names.len(),"incomplete source battle objects");
    let mut pointers=BTreeMap::new(); let mut labels:Vec<(String,Vec<Value>)>=Vec::new();
    for raw in read("data/battle_anims/framesets.asm")?.lines() {
        let line=code_part(raw);
        if let Some(target)=line.strip_prefix("dw ") {
            if let Some((_,name))=raw.split_once(';') {pointers.insert(name.trim().to_owned(),target.trim().to_owned());}
        } else if line.starts_with('.') && line.ends_with(':') {labels.push((line.trim_end_matches(':').to_owned(),Vec::new()));}
        else if let Some((_,entries))=labels.last_mut() {
            let (command, rest)=line.split_once(' ').unwrap_or((line,""));
            let value=match command {
                "oamframe" => {let args=source_arguments(rest)?; ensure!(args.len()>=2,"invalid source frame");json!({"command":"frame","oam_set":args[0],"duration":numeric(&args[1])?,"xflip":args.iter().skip(2).any(|flag|flag.contains("B_OAM_XFLIP")),"yflip":args.iter().skip(2).any(|flag|flag.contains("B_OAM_YFLIP"))})},
                "oamwait"=>json!({"command":"wait","oam_set":null,"duration":numeric(rest)?,"xflip":false,"yflip":false}),
                "oamend"|"oamrestart"|"oamdelete"=>json!({"command":command.trim_start_matches("oam"),"oam_set":null,"duration":0,"xflip":false,"yflip":false}),
                _=>continue,
            };entries.push(value);
        }
    }
    let mut framesets=BTreeMap::new();
    for (name,target) in pointers {
        let start=labels.iter().position(|(label,_)|label==&target).context("missing frameset target")?;
        let mut entries=Vec::new(); let mut terminated=false;
        for (_,body) in &labels[start..] {entries.extend(body.iter().cloned()); if body.iter().any(|entry|matches!(entry["command"].as_str(),Some("end"|"restart"|"delete"))) {terminated=true;break;}}
        ensure!(terminated,"unterminated source frameset {name}");framesets.insert(name,entries);
    }
    let mut gfx_entries=Vec::new();
    for line in read("data/battle_anims/object_gfx.asm")?.lines().map(code_part) {
        if let Some(raw)=line.strip_prefix("anim_obj_gfx ") {let args=source_arguments(raw)?;ensure!(args.len()==2,"invalid battle gfx");gfx_entries.push(json!([numeric(&args[0])?,args[1]]));}
    }
    let mut gfx_table=BTreeMap::from([("BATTLE_ANIM_GFX_0".to_owned(),gfx_entries.first().context("missing zero battle gfx")?.clone())]);
    for (name,value) in constants["global"].as_object().context("global constants")? {
        if name.starts_with("BATTLE_ANIM_GFX_") {let index=usize::try_from(value.as_i64().context("gfx index")?)?;gfx_table.insert(name.clone(),gfx_entries.get(index).context("battle gfx outside source table")?.clone());}
    }
    let mut gfx_sources=BTreeMap::new();let mut aliases=Vec::new();
    for line in read("gfx/battle_anims.asm")?.lines().map(code_part) {
        let mut remainder=line;
        if let Some((label,body))=line.split_once(':') {aliases.push(label.trim().to_owned());remainder=body.trim();}
        if let Some(raw)=remainder.strip_prefix("INCBIN ") {let path=raw.split('"').nth(1).context("battle gfx path")?.trim_end_matches(".lz");ensure!(vendor.join(path).is_file(),"missing generated graphics {path}");for label in aliases.drain(..) {gfx_sources.insert(label,path.to_owned());}}
    }
    let bundle=json!({"objects":objects,"framesets":framesets,"oam_sets":oam,"gfx_table":gfx_table,"gfx_sources":gfx_sources});
    let data=root.join("apps/web/assets/data");let modular=data.join("content-packs/core-modular/battle_anim_bundle");std::fs::create_dir_all(&modular)?;
    let bytes=serde_json::to_vec_pretty(&bundle)?;std::fs::write(data.join("battle_anim_bundle.json"),&bytes)?;std::fs::write(modular.join("bundle.json"),bytes)?;
    Ok(())
}
