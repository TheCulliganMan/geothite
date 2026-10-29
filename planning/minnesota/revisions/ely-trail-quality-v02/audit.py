#!/usr/bin/env python3
"""Audit actual native terrain, roster closure and retained source art; no pack writes."""
import argparse,copy,hashlib,json,sys
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[4]
HERE=Path(__file__).parent
sys.path[:0]=[str(ROOT/'planning/minnesota/tools'),str(ROOT/'planning/minnesota/revisions/ely-backroute-v01'),str(HERE)]
from render_reference import load_pack,SourceArt
from render_archive import save_render
from review_trainers import check_species
from review_north_shore import oriented_sprite
from connected import connected
from author import course


def main():
 ap=argparse.ArgumentParser();ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--geometry',type=Path,required=True);ap.add_argument('--source',type=Path,required=True);ap.add_argument('--epoch',required=True);a=ap.parse_args()
 out=ROOT/'planning/minnesota/renders/ely-trail-quality-v02'/a.epoch;out.mkdir(exist_ok=False,parents=True)
 g=load_pack(a.geometry);p=load_pack(a.pack);d=p['data'];before=g['data']
 assert set(before['maps'])==set(d['maps'])
 for name,old in before['maps'].items():
  new=d['maps'][name]
  assert old['blocks']==new['blocks'],('terrain changed by roster binding',name)
  assert old['events']==new['events'],('events changed by roster binding',name)
  for key,v in old['attributes'].items():
   if key!='fishing_group':assert new['attributes'][key]==v,(name,key)
  assert [(o['x'],o['y'],o['object_type'],o['event_flag'],o['object_identifier'])for o in old['objects']]==[(o['x'],o['y'],o['object_type'],o['event_flag'],o['object_identifier'])for o in new['objects']]
 assert p['compiled_audio']==g['compiled_audio'];assert p['audio_manifest']==g['audio_manifest']
 for path,v in g['runtime_files'].items():assert p['runtime_files'][path]==v,('original runtime bytes',path)
 r=json.load(open(ROOT/'planning/minnesota/pokemon/roster.json'));curation=json.load(open(ROOT/'planning/minnesota/pokemon/roster-curation.json'));cut={f['id']for f in curation['retired_families']};allowed={m['name'].upper()for f in r['families']if f['id']not in cut for m in f['members']}
 assert allowed<=set(d['pokemon'])
 for name,trainer in d['trainers']['trainers'].items():
  if name.startswith('MN_'):check_species(trainer['party'],allowed)
 for key in ['wild_encounters','field_encounters']:
  for name,table in d[key].items():
   if name.startswith('Mn'):check_species(table,allowed)
 for key,prefix in [('groups','FISHGROUP_MN_'),('time_groups','MN_TIME_')]:
  for name,table in d['fishing'][key].items():
   if name.startswith(prefix):check_species(table,allowed)
 for name,m in d['maps'].items():
  if name.startswith('Mn'):
   for o in m['objects']:
    n=o['sprite'].removeprefix('SPRITE_')
    if n in d['pokemon']:assert n in allowed,(name,n)
 proof=json.load(open(a.source/'input-plan.json'));check=course(p,(0,30),(33,34),True);back=course(p,(33,34),(0,30),True)
 assert check['cost']==proof['minimum_grass_east']==14
 assert back['cost']==proof['minimum_grass_west']==0
 for key,start,end,grass in [('eastbound',(0,30),(33,34),False),('westbound',(33,34),(0,30),False),('dry_return',(33,34),(0,30),True)]:assert course(p,start,end,grass)==proof[key]
 for key in ['eastbound','westbound','dry_return']:
  for step in proof[key]['steps']:
   x,y=step['from_core'];tx,ty=step['to_core'];stride=2 if step['jump'] else 1
   assert (tx-x,ty-y)==tuple(stride*v for v in {'up':(0,-1),'down':(0,1),'left':(-1,0),'right':(1,0)}[step['button']]),('wrong native button',step)
 # Corrupt each important authored feature to ensure the contract catches it.
 broken=copy.deepcopy(p);m=broken['data']['maps']['MnR11C00R02'];w=m['attributes']['width'];m['blocks'][17*w+11]=1
 assert course(broken,(0,30),(33,34),True)['cost']==0,'removing the lip should expose the prohibited dry shortcut'
 broken=copy.deepcopy(p);m=broken['data']['maps']['MnR11C00R02'];m['blocks'][17*w+10]=5
 try:course(broken,(33,34),(0,30),True)
 except AssertionError:pass
 else:assert course(broken,(33,34),(0,30),True)['cost']>0,'blocking landing should remove dry ledge return'
 sha=hashlib.sha256(a.pack.read_bytes()).hexdigest();proof.update(pack=str(a.pack),pack_sha256=sha)
 (out/'input-plan.json').write_text(json.dumps(proof,indent=2)+'\n')
 art=SourceArt(p)
 for name in ['MnEly','MnElyShorePass','MnR11C00R02','MnSplitRockPokecenter1F']:
  m=d['maps'][name];v=m['attributes'];im=art.render(v['tileset_name'],v['width'],v['height'],m['blocks'],'day').convert('RGBA')
  for o in m['objects']:
   face={'UP':'north','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south');im.alpha_composite(oriented_sprite(p,o['sprite'],face),(o['x']*16,o['y']*16))
  save_render(im,out/(name+'-whole.png'))
  if name=='MnR11C00R02':save_render(im.crop((0,12*32,19*32,20*32)).resize((1216,512),Image.Resampling.NEAREST),out/'pine-knoll-trail.png')
 save_render(connected(p,art),out/'connected-whole.png')
 receipt=dict(pack=str(a.pack),sha256=sha,identity=p['identity']['content_hash'],maps=len(d['maps']),active_regional_species=len(allowed),installed_species=len(d['pokemon']),all_geometry_blocks_events_positions_exact_to_geometry_stage=True,all_geometry_runtime_bytes_exact=True,all_audio_exact=True,all_minnesota_trainers_wild_fishing_visible_species_curated=True,minimum_eastbound_grass=14,minimum_westbound_grass=0,negative_cases_rejected=['missing-ledge','blocked-landing'],note='Static/native publication evidence. Controller and GPU evidence recorded separately.')
 (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt,indent=2))
if __name__=='__main__':main()
