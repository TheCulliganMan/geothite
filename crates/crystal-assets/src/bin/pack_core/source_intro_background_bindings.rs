//! Reconstruct the authored intro BG VRAM bindings, including carried banks.
use super::{Source,evaluate,symbols};
use anyhow::{Context,Result,ensure};
use serde_json::{Value,json};
use std::{collections::BTreeMap,path::Path};

pub(super) fn lower(vendor:&Path, constants:&BTreeMap<String,i64>, setup:&Value)->Result<BTreeMap<String,Value>> {
    let source=Source::read(vendor,"engine/movie/intro.asm")?;
    let (addresses,banks)=symbols(vendor)?;
    let mut constants=constants.clone();constants.extend(addresses);
    let mut tiles:Vec<Vec<Option<(String,u16)>>> = vec![vec![None;256];2];
    let mut tilemap=None;let mut attrmap=None;let mut palette=None;
    let mut result=BTreeMap::new();
    for number in 1..=28 {
        let label=format!("IntroScene{number}");
        let mut vram_bank=0usize;
        if let Some(operations)=setup[&label]["operations"].as_array(){
            let mut decompressed=None;
            for operation in operations {
                match operation["op"].as_str().context("intro setup operation")? {
                    "write_memory_byte" if operation["target"]=="rVBK"=>{vram_bank=operation["value"].as_u64().context("intro VRAM bank")? as usize;ensure!(vram_bank<2,"intro VRAM bank range");},
                    "decompress_lz3_resource"=>{decompressed=Some(operation["resource"].as_str().context("compressed intro resource")?.to_owned());},
                    "request_2bpp_transfer"=>{
                        let resource=if operation["source"]=="wDecompressScratch"{decompressed.as_ref().context("intro BG request lacks decompression")?.clone()}else{operation["source"].as_str().context("intro direct graphics source")?.to_owned()};
                        let address=evaluate(operation["target"].as_str().context("intro VRAM target")?,&constants,&banks)?;
                        let bank=operation["target_vram_bank"].as_u64().map(|x|x as usize).unwrap_or(vram_bank);
                        ensure!(bank<2,"intro graphics bank");
                        if address==constants["vBGMap0"] {
                            if bank==0{tilemap=Some(resource);}else{attrmap=Some(resource);}
                        }else{
                            let count=operation["tile_count"].as_u64().context("intro tile count")?;
                            ensure!((0x8000..0x9800).contains(&address) && address%16==0,"intro graphics VRAM address {address:#x}");
                            for index in 0..count {
                                let destination=address+index as i64*16;
                                // Signed BG IDs address 0x9000..0x97f0 for 0..127,
                                // and 0x8800..0x8ff0 for 128..255. The first OBJ
                                // page (0x8000..0x87ff) cannot be a BG binding.
                                let tile_id=if (0x9000..0x9800).contains(&destination){Some((destination-0x9000)/16)}else if (0x8800..0x9000).contains(&destination){Some(128+(destination-0x8800)/16)}else{None};
                                if let Some(tile_id)=tile_id{tiles[bank][tile_id as usize]=Some((resource.clone(),index as u16));}
                            }
                        }
                    },
                    "copy_memory" if operation["target"]=="wBGPals1"=>{palette=Some(operation["source"].as_str().context("intro palette copy source")?.to_owned());},
                    _=>{}
                }
            }
        }
        let mut bindings=Vec::new();
        for (bank,bank_tiles) in tiles.iter().enumerate(){
            let mut index=0;
            while index<256 {
                let Some((resource,start))=&bank_tiles[index] else{index+=1;continue};
                let begin=index;let resource=resource.clone();let resource_start=*start;
                index+=1;
                while index<256 && bank_tiles[index].as_ref().is_some_and(|(next,tile)|*next==resource && *tile==resource_start+(index-begin) as u16){index+=1;}
                bindings.push(json!({"tile_id_start":begin,"tile_id_end":index-1,"target_vram_bank":bank,"resource":resource,"resource_tile_start":resource_start}));
            }
        }
        ensure!(!bindings.is_empty(),"intro scene {number} has no authored BG graphics");
        result.insert(label.clone(),json!({"op":"intro_background_binding","dispatcher_entry":number-1,"tile_addressing":"signed_8800","tilemap_resource":tilemap.as_ref().with_context(||format!("intro scene {number} tilemap"))?,"attrmap_resource":attrmap.as_ref().with_context(||format!("intro scene {number} attrmap"))?,"palette_resource":palette.as_ref().with_context(||format!("intro scene {number} palette"))?,"tile_bindings":bindings,"source_span":source.span(&label)?}));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn all_scene_backgrounds_bind_source_maps_and_resolved_tile_bytes(){
  let Some(root)=std::env::var_os("CRYSTAL_EXPORT_WORKSPACE")else{return};let root=Path::new(&root);let vendor=root.join("vendor/pokecrystal");
  let constants:Value=serde_json::from_slice(&std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json")).unwrap()).unwrap();let constants=serde_json::from_value(constants["global"].clone()).unwrap();
  let setup=super::super::setup_phases(&vendor,&constants).unwrap();let bindings=lower(&vendor,&constants,&setup).unwrap();assert_eq!(bindings.len(),28);
  let assets=root.join("apps/web/assets");let read=|path:&str|std::fs::read(assets.join(path.strip_suffix(".lz").unwrap_or(path))).unwrap();
  for (scene,binding) in &bindings {
   let tilemap=read(binding["tilemap_resource"].as_str().unwrap());let attrmap=read(binding["attrmap_resource"].as_str().unwrap());assert_eq!(tilemap.len(),1024);assert_eq!(attrmap.len(),1024);
   for (offset,(&tile,&attr)) in tilemap.iter().zip(&attrmap).enumerate() {
    let bank=(attr>>3)&1;
    let range=binding["tile_bindings"].as_array().unwrap().iter().find(|range|range["target_vram_bank"]==bank && range["tile_id_start"].as_u64().unwrap()<=u64::from(tile) && range["tile_id_end"].as_u64().unwrap()>=u64::from(tile));
    let Some(range)=range else {
     // Source maps retain uninitialized padding outside the 20x18 LCD.
     assert!(offset%32>=20 || offset/32>=18,"{scene}: visible tile{tile} bank{bank} not bound");
     continue;
    };
    let index=range["resource_tile_start"].as_u64().unwrap()+u64::from(tile)-range["tile_id_start"].as_u64().unwrap();let resource=range["resource"].as_str().unwrap();
    assert!((index as usize+1)*16<=read(resource).len(),"{scene}: {resource} tile{index} outside authored bytes");
   }
  }
  assert_eq!(bindings["IntroScene3"]["tilemap_resource"],bindings["IntroScene4"]["tilemap_resource"]);
  assert_eq!(bindings["IntroScene17"]["tile_bindings"].as_array().unwrap().iter().find(|range|range["resource"]=="gfx/intro/suicune_close.2bpp.lz"&&range["tile_id_start"]==0).unwrap()["resource_tile_start"],128);
  assert!(bindings["IntroScene19"]["tile_bindings"].as_array().unwrap().iter().any(|range|range["resource"]=="gfx/intro/grass4.2bpp"&&range["tile_id_start"]==255&&range["tile_id_end"]==255));
 }
}
