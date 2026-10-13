use anyhow::{Context,Result,ensure};
use crystal_audio::{AudioCommand,AudioSource,synth::{Program,MusicData,SynthContext,link,render,encode_midi}};
use std::{collections::BTreeMap,path::Path};
use super::source_syntax::{code_part,source_arguments};

pub(super) fn export(root:&Path)->Result<()> {
    let vendor=root.join("vendor/pokecrystal");let audio=vendor.join("audio");
    let mut paths=std::fs::read_dir(audio.join("music"))?.map(|entry|entry.map(|entry|entry.path())).collect::<std::io::Result<Vec<_>>>()?;
    paths.extend([audio.join("sfx.asm"),audio.join("sfx_crystal.asm"),audio.join("cries.asm")]);paths.sort();
    let mut catalog=BTreeMap::<String,AudioSource>::new();let mut headers=BTreeMap::<String,Vec<(u8,String)>>::new();
    for path in paths {
        if path.extension().is_none_or(|ext|ext!="asm") {continue;}
        let source=std::fs::read_to_string(&path)?;let mut owner=String::new();let mut order=Vec::new();
        for line in source.lines().map(code_part).filter(|line|!line.is_empty()) {
            if line.ends_with(':') || (line.starts_with('.') && !line.contains(char::is_whitespace)) {
                let label=line.trim_end_matches(':');
                if label.starts_with('.') {
                    catalog.get_mut(&owner).context("audio local label has no owner")?.commands.push(AudioCommand{command:"label".into(),args:vec![format!("{owner}{label}")]});
                } else {
                    owner=label.to_owned();order.push(owner.clone());
                    ensure!(catalog.insert(owner.clone(),AudioSource{number:None,commands:Vec::new()}).is_none(),"duplicate source audio label {owner}");
                }continue;
            }
            if owner.is_empty() {continue;}
            let (command,raw)=line.split_once(char::is_whitespace).unwrap_or((line,""));let args=source_arguments(raw)?;
            match command {
                "channel_count" => {},
                "channel"=>{ensure!(args.len()==2,"invalid source audio channel");headers.entry(owner.clone()).or_default().push((args[0].parse()?,args[1].clone()));},
                "db"|"DEF"|"REDEF"|"assert"|"assert_table_length"|"INCLUDE"|"SECTION"=>{},
                _=>catalog.get_mut(&owner).unwrap().commands.push(AudioCommand{command:command.to_owned(),args}),
            }
        }
        // Consecutive labels alias the same emitted byte sequence.
        let mut following=None;
        for name in order.iter().rev() {
            if catalog[name].commands.is_empty() && !headers.contains_key(name) {
                if let Some(next)=following.as_ref() {
                    let mut body=catalog[next].commands.clone();
                    for command in &mut body {
                        if command.command=="label" {
                            for argument in &mut command.args {
                                if let Some(local)=argument.strip_prefix(&format!("{next}.")) {*argument=format!("{name}.{local}");}
                            }
                        }
                    }
                    catalog.get_mut(name).unwrap().commands=body;
                }
                if let Some(next)=following.as_ref() {
                    if let Some(channels)=headers.get(next).cloned() {headers.insert(name.clone(),channels);}
                }
            }
            following=Some(name.clone());
        }
    }
    let data=root.join("apps/web/assets/data");let modular=data.join("content-packs/core-modular/audio");std::fs::create_dir_all(&modular)?;
    let context=SynthContext::cartridge()?;let mut assets=BTreeMap::new();
    for (namespace,constant_file,pointer_file,kind,prefix) in [
        ("music","music_constants.asm","music_pointers.asm","music","MUSIC_"),
        ("sfx","sfx_constants.asm","sfx_pointers.asm","sound_effect","SFX_"),
        ("cries","cry_constants.asm","cry_pointers.asm","cry","CRY_"),
    ] {
        let names=std::fs::read_to_string(vendor.join("constants").join(constant_file))?.lines().map(code_part).filter_map(|line|line.strip_prefix("const ").filter(|name|name.starts_with(prefix)).map(str::to_owned)).collect::<Vec<_>>();
        let pointers=std::fs::read_to_string(audio.join(pointer_file))?.lines().map(code_part).filter_map(|line|line.strip_prefix("dba ").map(str::to_owned)).collect::<Vec<_>>();
        ensure!(names.len()==pointers.len(),"source {namespace} identity table mismatch {} vs {}",names.len(),pointers.len());
        for (index,(id,label)) in names.iter().zip(pointers.iter()).enumerate() {
            if id=="MUSIC_NONE" {continue;}
            let channels=headers.get(label).with_context(||format!("source audio header missing {label}"))?;
            let mut selected=BTreeMap::new();
            for (number,name) in channels {let mut body=catalog.get(name).with_context(||format!("missing channel body {name}"))?.clone();body.number=Some(*number);selected.insert(name.clone(),body);}
            let input=Program{profile:"pokecrystal-midi-v1".into(),music_data:MusicData{channel_count:u8::try_from(selected.len())?,channels:selected,subroutines:BTreeMap::new(),shared_sources:BTreeMap::new()},cry_pitch:None,cry_length:None};
            let program=link(&input,&catalog).with_context(||format!("link source audio {id}"))?;
            let rendered=render(&program,context.clone()).with_context(||format!("synthesize source audio {id}"))?;
            let pcm=rendered.downsample();ensure!(!pcm.is_empty(),"source audio {id} rendered no samples");
            let frames=pcm.len()/2;let hash=pcm.iter().flat_map(|sample|sample.to_le_bytes()).fold(2166136261u32,|hash,byte|(hash^u32::from(byte)).wrapping_mul(16777619));
            let primary=*channels.iter().map(|(number,_)|number).min().context("audio channels")?;
            let start=if kind=="music" {rendered.loop_samples.get(&primary).map(|value|value.div_ceil(2))}else{None};
            let midi=encode_midi(&program)?;
            let audio_path=format!("audio/{namespace}/{id}.pcm");
            let output=data.join(&audio_path);
            std::fs::create_dir_all(output.parent().context("external source audio directory")?)?;
            std::fs::write(output,pcm.iter().flat_map(|sample|sample.to_le_bytes()).collect::<Vec<_>>())?;
            let value=serde_json::json!({"id":id,"path":audio_path,"kind":kind,"source":"pcm","sfx_priority":if kind=="sound_effect" {Some(u8::try_from(index)?)}else{None},"pcm_format":{"sample_rate_hz":22050,"channels":2,"bits_per_sample":16},"pcm_frame_count":frames,"payload_hash":format!("{hash:08x}"),"loop_start_sample":start,"loop_end_sample":start.map(|_|frames),"midi_program":{"profile":"pokecrystal-midi-v1","midi_base64":midi}});
            let _:crystal_assets::ModpackAudioAsset=serde_json::from_value(value.clone()).with_context(||format!("validate source audio {id}"))?;
            std::fs::write(modular.join(format!("{id}.json")),serde_json::to_vec_pretty(&serde_json::json!({id:value}))?)?;
            assets.insert(id.clone(),value);println!("exported audio {id}: {frames} frames, hash {hash:08x}");
        }
    }
    std::fs::write(data.join("audio.json"),serde_json::to_vec_pretty(&assets.into_values().collect::<Vec<_>>())?)?;
    Ok(())
}
