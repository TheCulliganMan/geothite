#!/usr/bin/env python3
"""Bounded whole-trip R18 source review and native-preimage handoff; no pack writes."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image, ImageDraw
from render_archive import save_render
from render_reference import layout_blocks, load_pack
from review_league_approach import check as generic_check
from review_mille_lacs_shore import check_lake, walking_design
from review_r15_refinement import graph, adjacent
from review_r18_shorewalk_refinement import render, shortest
from review_r18_inhabited_ribbon import largest_square

ROOT=Path(__file__).resolve().parents[3]
BASE=ROOT/'planning/minnesota'
PACK=ROOT/'content-packs/minnesota-three-cities-tight-final-v02.crystalpack'
PACK_SHA='052de08898bef175959d7beeab75409b949612d71266e59c86561221a8c137e6'
REGIONS=[[4,3,6,5],[4,7,5,13],[9,4,17,8],[21,3,27,10],[28,6,37,11],[29,16,31,17]]

def sha(b):return hashlib.sha256(b).hexdigest()
def canonical(v):return sha(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')

def inspect(d,l,p,od,ol):
 start=time.perf_counter();ob,ok,oa,orr,oc,oh=graph(od,ol,p);b,kind,allowed,reach,cost,hazards=graph(d,l,p);checks=[]
 def test(n,v):checks.append(dict(check=n,passed=bool(v)))
 test('all original actor/event/story/party/reward/encounter/room/endpoints exact',all(d.get(k)==od.get(k)for k in set(d)|set(od)if k not in{'main_path_metatiles','loops','viewpoints','surf_viewpoints','review_contract'}))
 test('old Surf views exact prefix',d['surf_viewpoints'][:len(od['surf_viewpoints'])]==od['surf_viewpoints'])
 test('old views exact prefix and one shorewalk loop retained',d['viewpoints'][:len(od['viewpoints'])]==od['viewpoints'] and len(d['loops'])==1 and d['loops'][0]['id']=='PUBLIC_SHORE_WATCH_LOOP')
 test('extent tilesets native art recipes materials piers and metadata exact',all(l.get(k)==ol.get(k)for k in set(l)|set(ol)if k!='layers'))
 test('complete original layer prefix exact',l['layers'][:len(ol['layers'])]==ol['layers'])
 delta=[dict(metatile=[i%40,i//40],before=f'{a:02x}',after=f'{z:02x}')for i,(a,z)in enumerate(zip(ob,b))if a!=z]
 test('all terrain deltas inside six explicitly authored regions',bool(delta) and all(any(x0<=v['metatile'][0]<x1 and y0<=v['metatile'][1]<y1 for x0,y0,x1,y1 in REGIONS)for v in delta))
 test('native original grass quadrants all retained',all(kind((x,y))=='TALL_GRASS'for y in range(48)for x in range(80)if ok((x,y))=='TALL_GRASS'))
 test('no incidental Cut or unsupported hop/stair/warp collision',not any(kind((x,y))in{'CUT_TREE','LADDER','STAIRCASE','DOOR','WARP_PANEL'}or'HOP'in kind((x,y))for y in range(48)for x in range(80)))
 shared,_,_=generic_check(walking_design(d),l,p,b);test('complete authored route geometry checker clean',not shared['errors'])
 try:lake=check_lake(d,l,p);test('full original piers Surf boundary islet SQ04 and flatshore contract',True)
 except AssertionError as error:lake={'error':str(error)};test('full original piers Surf boundary islet SQ04 and flatshore contract',False)
 west,east=[tuple(2*v for v in a['gate_metatile'])for a in d['endpoints']];seen=reach(west)
 test('retained ordinary Floor connected with all native actor bodies',not((orr(west)&allowed)-seen))
 test('regional endpoint travel in both directions',east in seen and west in reach(east))
 prices=[]
 for label,a,z in [('west-east',west,east),('east-west',east,west)]:
  old,new=oc(a,{z}),cost(a,{z});prices.append(dict(id=label,before=old,candidate=new));test(label+' six necessary grass steps exact',old==new==6)
 for a in d['items']:
  if a['id']in d['surf_only_actor_ids']:continue
  for label,st in [('west',west),('east',east)]:
   target=set(adjacent(tuple(a['tile'])));old,new=oc(st,target),cost(st,target);prices.append(dict(id=a['id']+'-'+label,before=old,candidate=new));test(a['id']+' '+label+' optional resource grass price exact',old==new)
 test('ordinary travel avoiding every trainer sight tile still exists',east in reach(west,hazards)and west in reach(east,hazards))
 for a in d['residents']:test(a['id']+' reachable walking-only',bool(set(adjacent(tuple(a['tile'])))&seen))
 # Main narrative annotation is a contract, not a line painted over collision.
 test('every annotated main-route waypoint ordinary Floor',all(kind((x*2,y*2))=='FLOOR' for x,y in d['main_path_metatiles']))
 test('new western narrow bank ordinary and lake-facing',all(kind((x,14))=='FLOOR'and kind((x,15))=='WALL'for x in range(20,32)))
 test('new western grove is attached to north grove and cuts old straight course',all(kind(at)=='WALL'for at in[(19,9),(20,9),(20,10),(21,12),(22,12),(23,13)]))
 test('western bank main course reachable without trainer battle',all(at in reach(west,hazards)for at in[(19,14),(23,14),(27,14),(27,12)]))
 newwater=sum(kind((x,y))=='WATER'for y in range(10,19)for x in range(22,32));test('new western player frame visibly includes lake',newwater>=20)
 test('central ribbon nine-step dry bend retained',len(shortest((45,13),(48,13),{a for a in allowed if kind(a)=='FLOOR'})or[])-1==9)
 test('central grass route still three steps/two grass',len(shortest((45,13),(48,13),allowed)or[])-1==3 and sum(kind(a)=='TALL_GRASS'for a in(shortest((45,13),(48,13),allowed)or[])[1:])==2)
 test('protected shoreloop crosses real neighbor seam on both rows',all(kind(at)=='FLOOR'for at in[(63,17),(64,17),(63,18),(64,18)]))
 test('shoreloop upper lower rails are real barriers',all(kind((x,16))=='WALL'and kind((x,19))=='WALL'and kind((x,17))==kind((x,18))=='FLOOR'for x in range(60,70)))
 test('new islet pine never blocks native reward/Surf arrival',kind((58,32))=='WALL'and kind((61,34))=='FLOOR'and kind((61,35))=='FLOOR')
 beforeclear=largest_square(ok,oa,(0,0,80,48));afterclear=largest_square(kind,allowed,(0,0,80,48));test('whole-route largest clear Floor square shrinks',afterclear['width_core']<beforeclear['width_core'])
 return dict(status='PASS'if all(v['passed']for v in checks)else'FAIL',checks=checks,terrain_changes=delta,shared_geometry=shared,lake_contract=lake,grass_prices=prices,western_frame_water_core_cells=newwater,largest_clear_square=dict(before=beforeclear,candidate=afterclear),warm_check_ms=round((time.perf_counter()-start)*1000,3)),kind,allowed,hazards

def course(kind,allowed,blocked,waypoints):
 result=[]
 for a,z in zip(waypoints,waypoints[1:]):
  path=shortest(tuple(a),tuple(z),allowed-blocked);assert path,(a,z)
  buttons=[]
  for f,t in zip(path,path[1:]):buttons.append({(-1,0):'Left',(1,0):'Right',(0,-1):'Up',(0,1):'Down'}[(t[0]-f[0],t[1]-f[1])])
  result.append(dict(from_core=a,to_core=z,steps=len(buttons),buttons=buttons,path_core=path))
 return result

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--source',type=Path,required=True);ap.add_argument('--self-test',action='store_true');a=ap.parse_args();out=a.source
 assert sha(PACK.read_bytes())==PACK_SHA;p=load_pack(PACK)
 od=json.loads((out/'before-MN_R18.json').read_text());ol=json.loads((out/'before-MN_R18.layout.json').read_text());d=json.loads((out/'MN_R18.candidate.json').read_text());l=json.loads((out/'MN_R18.layout.candidate.json').read_text())
 blocks=layout_blocks(ol);native=[]
 for y in range(24):native+=p['data']['maps']['MnR18C00R00']['blocks'][y*32:(y+1)*32]+p['data']['maps']['MnR18C01R00']['blocks'][y*8:(y+1)*8]
 assert native==blocks,'Current native R18 preimage differs from canonical source'
 r,kind,allowed,hazards=inspect(d,l,p,od,ol)
 preimage=dict(parent_pack=str(PACK.relative_to(ROOT)),parent_file_sha256=PACK_SHA,map_count=len(p['data']['maps']),native_source_rows_exact=True,modules={name:dict(module_sha256=canonical(p['data']['maps'][name]),field_sha256={k:canonical(v)for k,v in p['data']['maps'][name].items()if k!='blocks'},block_sha256=sha(bytes(p['data']['maps'][name]['blocks'])))for name in ['MnR18C00R00','MnR18C01R00']},source_sha256={f.name:sha(f.read_bytes())for f in [out/'before-MN_R18.json',out/'before-MN_R18.layout.json',out/'MN_R18.candidate.json',out/'MN_R18.layout.candidate.json']},allowed_changes='Only these two existing modules.blocks and their uniquely owned map_blocks payloads, plus derived pack identity. All other module fields/maps/global programs/art/audio/HOME/recovery/report/catalogs exact.',exact_proposed_terrain_delta=r['terrain_changes'],native_art_sha256=canonical(p['data']['tilesets'][l['compiled_tileset']]))
 write(out/'native-preimage-contract.json',preimage)
 before=render(od,ol,p,out,'before-current660');drawd=deepcopy(d);drawd['viewpoints'] += [v for v in d['surf_viewpoints']if v['id']=='ISLET_PINE_DISCOVERY'];candidate=render(drawd,l,p,out,'candidate')
 pair=Image.new('RGB',(1280,1584),(20,26,35));dr=ImageDraw.Draw(pair);dr.text((8,4),'CURRENT660 installed R18 source composite; native preimage exact',fill='white');pair.paste(before,(0,24));dr.text((8,796),'CANDIDATE whole shore trip; source composite only, not native publication',fill='white');pair.paste(candidate,(0,816));save_render(pair,out/'before-after-whole-native32.png')
 if a.self_test:
  assert r['status']=='PASS', [v for v in r['checks']if not v['passed']]
  negatives=[]
  def reject(name,change):
   nd,nl=deepcopy(d),deepcopy(l);change(nd,nl);nr,_,_,_=inspect(nd,nl,p,od,ol);fail=[v['check']for v in nr['checks']if not v['passed']];assert fail,name+' accepted';folder=out/'rejected-inputs'/name;folder.mkdir(parents=True,exist_ok=False);write(folder/'dossier.json',nd);write(folder/'layout.json',nl);write(folder/'failed-checks.json',fail);negatives.append(dict(id=name,rejected=True,failed_checks=fail))
  def point(l,at,b):l['layers'].append(dict(kind='point',name='Adversarial physical fixture',position=at,block=b))
  reject('original-grass-neck-dried',lambda d,l:l['layers'].append(dict(kind='block_rect',name='Bad dry neck',bounds=[18,5,21,9],block='02')))
  reject('western-bank-severed',lambda d,l:point(l,[11,7],'05'))
  reject('western-bank-water-hole',lambda d,l:point(l,[11,7],'35'))
  reject('old-straight-grove-bypass',lambda d,l:point(l,[10,5],'01'))
  reject('return-to-empty-central-lawn',lambda d,l:l['layers'].append(dict(kind='block_rect',name='Bad empty lawn',bounds=[21,3,27,8],block='02')))
  reject('central-shortcut-free-of-grass',lambda d,l:point(l,[23,6],'01'))
  reject('shoreloop-course-blocked',lambda d,l:point(l,[32,9],'82'))
  reject('shoreloop-unsafe-water-gap',lambda d,l:point(l,[32,9],'71'))
  reject('islet-dry-bridge',lambda d,l:l['layers'].append(dict(kind='block_rect',name='Bad free islet',bounds=[30,9,31,18],block='01')))
  reject('incidental-Cut',lambda d,l:point(l,[12,5],'5b'))
  reject('outside-water-escape',lambda d,l:point(l,[39,15],'35'))
  reject('pier-end-open',lambda d,l:point(l,[15,11],'71'))
  reject('report-actor-relocated',lambda d,l:d['residents'][0].update(tile=[59,35]))
  reject('trainer-party-changed',lambda d,l:d['trainers'][0]['party'][0].update(level=100))
  reject('quest-flag-changed',lambda d,l:d['quest_contract'].update(reward_flag='EVENT_WRONG'))
  r['negative_cases']=negatives
 if r['status']=='PASS':
  waypoints=[[2,10],[11,11],[19,14],[27,14],[31,10],[35,13],[42,13],[45,13],[45,16],[48,16],[48,13],[49,19],[59,18],[63,18],[64,18],[71,18],[75,18],[76,10]]
  walk=course(kind,allowed,hazards,waypoints);write(out/'controller-walk-input-plan.json',dict(status='UNEXECUTED genuine input plan; Root production-controller acceptance required',initial_fixture='Local start2,10; legal party/Badges/RNG only at setup, no subsequent state writes; not earned access.',main_no_trainer_sight=walk,execution='For each planned direction use production press, repeat only for native facing/turn until the expected visible step (bounded8 attempts), settle movement, assert expected map/local core. Never rewrite state.',central_choice=dict(dry=course(kind,{at for at in allowed if kind(at)=='FLOOR'},set(),[[45,13],[48,13]]),direct_grass=course(kind,allowed,set(),[[45,13],[48,13]])),coordinates=[dict(global_core=v,map='MnR18C'+str(v[0]//64).zfill(2)+'R00',local_core=[v[0]%64,v[1]])for v in waypoints],additional_acceptance=['Both west meadow bank approaches and all protective Wall tiles','Both central9stepdry and3steptwoGrass options; actual native encounters/resources','Both shoreloop entrances; walking63↔64 seam on rows17/18; blocked rails/corners','Original SQ04acknowledgment/badge/report-subsets/GOOD_ROD/adequate-rod/once/Continue cases unchanged; current catalog cannot construct a valid full key pocket without an adequate rod, so retain that honest original fixture limit','Original public Surf launch49,19→49,20; water63↔64 seam; native islet landing61,32→61,34 and Revive pickup61,35; pine blocked58,32; closed outer water edges','Original upper/lower working pier decks/rails/endcaps; all actor conversations, battle/PP and item flags retained']))
 r.update(source_only=True,native_published=False,pack_unchanged=sha(PACK.read_bytes())==PACK_SHA,canonical_sources_unchanged=all((BASE/'routes'/n).read_bytes()==(out/('before-'+n)).read_bytes()for n in ['MN_R18.json','MN_R18.layout.json']))
 write(out/'review.json',r);(out/'review_r18_quality_next.py.snapshot').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(status=r['status'],checks=len(r['checks']),changes=len(r['terrain_changes']),failed=[v for v in r['checks']if not v['passed']],shared_errors=r['shared_geometry']['errors'],lake_error=r['lake_contract'].get('error'),largest_clear=r['largest_clear_square'],western_lake=r['western_frame_water_core_cells'],negatives=len(r.get('negative_cases',[]))),indent=2))
 assert r['status']=='PASS'
if __name__=='__main__':main()
