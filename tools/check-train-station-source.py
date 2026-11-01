#!/usr/bin/env python3
"""Exercise the exact production train matcher with the ignored external pack.
No game art, script dump, collision file or source fixture is stored in the repo.
"""
import argparse,importlib.util,json,os,shutil,subprocess,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
BODY=r'''
fn check(map:&str,cells:Vec<Identity<'static>>,w:usize,h:usize) {
 let blocked=vec![false;cells.len()];let out=source::resolve(map,&cells,w,h,[0,0],&blocked);
 assert_eq!(out.len(),1,"{map}");let p=&out[0];assert!(p.coherent(&cells,w,h));
 assert_eq!(p.indices(w).count(),60);assert_eq!((p.column,p.row),(8,8));
 let owned=p.indices(w).collect::<Vec<_>>();
 for y in 2..4 {for x in [4,5,14,15] {assert!(!owned.contains(&((8+y)*w+8+x)),"owned live door");}}
 assert_eq!((0..4).flat_map(|y|(0..20).map(move|x|(x,y))).filter(|&(x,y)|cells[(8+y)*w+8+x].tile==0x1f).count(),12);
 assert!((12..16).all(|y|(8..28).all(|x|!owned.contains(&(y*w+x)))));
 assert!(source::resolve("ViridianGym",&cells,w,h,[0,0],&blocked).is_empty());
 assert!(source::resolve("GoldenrodCity",&cells,w,h,[0,0],&blocked).is_empty());
 assert!(source::resolve(map,&cells,w,h,[1,0],&blocked).is_empty());
 assert!(source::resolve(map,&cells,w,h,[0,1],&blocked).is_empty());
 assert!(source::resolve(map,&cells,w+1,h,[0,0],&blocked).is_empty());
 assert!(source::resolve(map,&cells,w,h,[0,0],&blocked[..blocked.len()-1]).is_empty());
 // Every single guard cell is tested, including all retained doors, backing,
 // and platform approach. Atomic fallback protects live/custom content.
 for i in p.guard_indices(w) {
  for mode in 0..6 {
   let mut changed=cells.clone();let mut custom=blocked.clone();
   match mode {0=>changed[i].tile^=1,1=>changed[i].metatile^=1,
    2=>changed[i].column^=1,3=>changed[i].row^=1,
    4=>changed[i].tileset="pokecom",_=>custom[i]=true}
   if mode<5 {assert!(!p.coherent(&changed,w,h));}
   assert!(source::resolve(map,&changed,w,h,[0,0],&custom).is_empty(),"partial source or custom overlap survived");
  }
 }
 let mut no_ground=cells.clone();for s in &mut no_ground {if source::valid_ground(s){s.tile=0xffff;}}
 assert!(!p.coherent(&no_ground,w,h));assert!(source::resolve(map,&no_ground,w,h,[0,0],&blocked).is_empty());
 let mut reserved_ground=blocked.clone();reserved_ground[p.ground]=true;
 assert!(source::resolve(map,&cells,w,h,[0,0],&reserved_ground).is_empty());
 // Ordinary scrolling/padding changes canvas indices but not map identity.
 let pad=3;let pw=w+2*pad;let ph=h+2*pad;let blank=Identity{tileset:"none",metatile:0,column:0,row:0,tile:0};let mut padded=vec![blank;pw*ph];
 for y in 0..h {padded[(y+pad)*pw+pad..(y+pad)*pw+pad+w].copy_from_slice(&cells[y*w..(y+1)*w]);}
 let padded_out=source::resolve(map,&padded,pw,ph,[-3,-3],&vec![false;padded.len()]);assert_eq!(padded_out.len(),1);assert_eq!(padded_out[0].indices(pw).count(),60);
 for (cw,ch) in [(27,h),(w,15)] {
  let clipped=(0..ch).flat_map(|y|cells[y*w..y*w+cw].iter().copied()).collect::<Vec<_>>();
  assert!(source::resolve(map,&clipped,cw,ch,[0,0],&vec![false;clipped.len()]).is_empty());
 }
 println!("PASS {map}: 60 body, 8 live door, 12 rail/backing; every art/map/phase/custom/native-ground/platform/clipping guard");
}
'''
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--pack',type=Path,required=True);parser.add_argument('--rustc',default=os.getenv('RUSTC')or shutil.which('rustc'));args=parser.parse_args();assert args.rustc,'set RUSTC or use --rustc'
 spec=importlib.util.spec_from_file_location('pack_reader',ROOT/'tools/check-gym-scenery-source.py');reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader);pack=reader.load(args.pack)
 module=ROOT/'crates/crystal-voxel-view/src/mesh/train_station_source.rs';code=f'#[path={json.dumps(str(module))}] mod source;\nuse source::Identity;\n'+BODY+'\nfn main(){\n'
 for name,other,flag in [('GoldenrodMagnetTrainStation','SAFFRON_MAGNET_TRAIN_STATION','FALSE'),('SaffronMagnetTrainStation','GOLDENROD_MAGNET_TRAIN_STATION','TRUE')]:
  m=pack['data']['maps'][name];assert m['attributes']['tileset_name']=='train_station'and not m['script_block_changes'];w=m['attributes']['width']*4;h=m['attributes']['height']*4;layout=pack['runtime_files']['data/tilesets/train_station_metatiles.bin']
  warps=m['events']['warps'];assert [(e['x'],e['y'],e['target_warp_id'])for e in warps[2:]]==[(6,5,4),(11,5,3)]
  assert all(e['target_map_constant']==other for e in warps[2:]);assert any(e['x']==11 and e['y']==6 for e in m['events']['coord_events'])
  scripts=m['scripts'];officer=scripts[name+'OfficerScript'];assert any(c['command']=='checkevent'and c['args']==['EVENT_RESTORED_POWER_TO_KANTO']for c in officer)
  ride=next(value for label,value in scripts.items()if label.startswith('.MagnetTrainTo'))
  assert any(c['command']=='checkitem'and c['args']==['PASS']for c in ride)
  sequence=[(c['command'],c['args'])for c in ride];start=sequence.index(('setval',[flag]))
  assert sequence[start:start+4]==[('setval',[flag]),('special',['MagnetTrain']),('warpcheck',[]),('newloadmap',['MAPSETUP_TRAIN'])]
  walk=scripts[name+'PlayerApproachAndEnterTrainMovement'];assert [c['args'][0]for c in walk if c['command']=='step']==['UP','UP','UP','LEFT','LEFT','LEFT','UP','UP']
  collision=pack['data']['tilesets']['train_station']['collision'];assert collision['11']==['WALL','WALL','DOOR','WALL']and collision['12']==['WALL','WALL','WALL','DOOR']
  code+='check('+json.dumps(name)+',vec![\n'
  for y in range(h):
   for x in range(w):
    b=m['blocks'][(y//4)*(w//4)+x//4];tile=layout[b*16+(y%4)*4+x%4]
    code+=f'Identity{{tileset:"train_station",metatile:{b},column:{x%4},row:{y%4},tile:{tile}}},\n'
  code+=f'],{w},{h});\n'
 code+='}\n'
 with tempfile.TemporaryDirectory(prefix='geothite-train-source-')as td:
  path=Path(td);(path/'check.rs').write_text(code);subprocess.run([args.rustc,'--edition=2024','-O','-A','dead_code',str(path/'check.rs'),'-o',str(path/'check')],check=True);subprocess.run([str(path/'check')],check=True)
 print('PASS native power/pass gates, both authoritative boarding warps, arrival coordinate events, movement and live MagnetTrain/MAPSETUP_TRAIN script sequence; no block-change states')
if __name__=='__main__':main()
