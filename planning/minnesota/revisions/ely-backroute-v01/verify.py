#!/usr/bin/env python3
"""Read-only audit and equal-scale render of the actual Rust-published child."""
import argparse,cbor2,hashlib,json,sys
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[4];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack,SourceArt
from render_archive import save_render
from review_north_shore import oriented_sprite

def main():
 a=argparse.ArgumentParser();a.add_argument('--pack',type=Path,required=True);a.add_argument('--epoch',required=True);a.add_argument('--contract',type=Path,default=ROOT/'planning/minnesota/revisions/ely-backroute-v01/source-v08/native.contract.json');args=a.parse_args()
 out=ROOT/'planning/minnesota/renders/ely-backroute-v01'/args.epoch;out.mkdir(parents=True,exist_ok=False)
 b=load_pack(ROOT/'content-packs/minnesota-superior-compact-v03.crystalpack');p=load_pack(args.pack)
 c=json.load(open(args.contract))
 old=b['data']['maps'];new=p['data']['maps'];assert set(new)-set(old)=={'MnElyShorePass','MnSplitRockPokecenter1F'};assert not set(old)-set(new)
 for n,m in old.items():
  if n not in ['MnEly','MnR11C00R02']:assert new[n]==m,('changed original map',n)
 assert p['compiled_audio']==b['compiled_audio'];assert p['audio_manifest']==b['audio_manifest']
 for path,blob in b['runtime_files'].items():assert p['runtime_files'][path]==blob,('original art changed',path)
 for n,patches in c['patches'].items():
  expected=list(old[n]['blocks']);w=old[n]['attributes']['width']
  for v in patches:x,y=v['position'];assert expected[y*w+x]==v['before'];expected[y*w+x]=v['after']
  assert new[n]['blocks']==expected;assert new[n]['objects']==old[n]['objects'];assert new[n]['scripts']==old[n]['scripts']
 assert new['MnElyShorePass']['blocks']==[int(v,16)for row in c['pass']['rows']for v in row.split()]
 assert new['MnEly']['attributes']['connections'][0]==old['MnEly']['attributes']['connections'][0]
 assert new['MnR11C00R02']['events']['warps'][:2]==old['MnR11C00R02']['events']['warps']
 assert new['MnR11C00R02']['events']['warps'][2]['target_map_constant']=='MN_SPLIT_ROCK_POKECENTER_1F'
 assert {k:v for k,v in p['data']['tilesets']['mn_ely_snow']['collision'].items() if k!='83'}==b['data']['tilesets']['kanto']['collision']
 assert not any(v in [0x0f,0x32,0x33]for v in new['MnEly']['blocks'])
 assert p['data']['wild_encounters']['MnElyShorePass']['map_name']=='MnElyShorePass'
 from snow_art import snow_art
 expected_art=__import__('copy').deepcopy(b);definition,files=snow_art(b)
 expected_art['data']['tilesets']['mn_ely_snow']=definition;expected_art['runtime_files'].update(files)
 expected_blocks=SourceArt(expected_art).blocks('mn_ely_snow')
 art=SourceArt(p);assert all(a.tobytes()==b.tobytes()for a,b in zip(expected_blocks,art.blocks('mn_ely_snow'))),'native art differs from reviewed source'
 scenes={}
 for n in ['MnEly','MnElyShorePass','MnR11C00R02','MnSplitRockPokecenter1F']:
  m=new[n];v=m['attributes'];im=art.render(v['tileset_name'],v['width'],v['height'],m['blocks'],'day').convert('RGBA')
  for o in m['objects']:
   facing={'UP':'north','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south');im.alpha_composite(oriented_sprite(p,o['sprite'],facing),(o['x']*16,o['y']*16))
  scenes[n]=im;save_render(im,out/(n+'-whole.png'))
 from connected import connected
 atlas=connected(p,art)
 save_render(atlas,out/'connected-whole.png');save_render(atlas.resize((1024,544),Image.Resampling.NEAREST),out/'connected-review.png')
 receipt={'pack':str(args.pack),'pack_sha256':hashlib.sha256(args.pack.read_bytes()).hexdigest(),'identity':p['identity']['content_hash'],'maps':len(new),'unchanged_existing_maps':len(old)-2,'all_original_runtime_files_exact':True,'audio_bytes_and_manifest_exact':True,'source_blocks_exact':True,'new_maps':sorted(set(new)-set(old)),'note':'Static/native source publication evidence. Controller tests and actual GPU captures are separate.'}
 (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt,indent=2))
if __name__=='__main__':main()
