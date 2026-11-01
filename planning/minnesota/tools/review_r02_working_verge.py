#!/usr/bin/env python3
"""Review the bounded nursery verge with installed art and directed farm physics."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image, ImageDraw
from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, npc_palette_pack, room_sheet
from render_stages import render_stage
from review_city_native import priority_scene
from review_league_approach import check as shared_check
from review_north_shore import oriented_sprite
from review_passage_native import actors
from review_prairie_farm import check_piers, check_water_boundary

ROOT=Path(__file__).resolve().parents[3]
PACK=ROOT/'content-packs/minnesota-three-cities-tight-final-v02.crystalpack'
PACK_SHA='052de08898bef175959d7beeab75409b949612d71266e59c86561221a8c137e6'

def sha(raw):return hashlib.sha256(raw).hexdigest()
def canonical(value):return sha(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())
def write(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
def adjacent(at):return[(at[0]-1,at[1]),(at[0]+1,at[1]),(at[0],at[1]-1),(at[0],at[1]+1)]

def geometry(d,l,p):
 blocks=layout_blocks(l);coll=p['data']['tilesets'][l['compiled_tileset']]['collision']
 def kind(at):
  x,y=at
  return coll[f'{blocks[y//2*30+x//2]:02x}'][y%2*2+x%2]if 0<=x<60 and 0<=y<44 else'OUTSIDE'
 occupied={tuple(a['tile'])for key in ['trainers','residents','items']for a in d[key]}
 walk={(x,y)for y in range(44)for x in range(60)if kind((x,y))in{'FLOOR','TALL_GRASS'}and(x,y)not in occupied}
 hops={(x,y)for y in range(44)for x in range(60)if kind((x,y))=='HOP_LEFT'}
 hazards=set();vec={'north':(0,-1),'south':(0,1),'east':(1,0),'west':(-1,0)}
 for a in d['trainers']:
  x,y=a['tile'];dx,dy=vec[a['facing']]
  for n in range(1,a['sight_range']+1):
   at=x+dx*n,y+dy*n
   if at not in walk:break
   hazards.add(at)
 return blocks,kind,walk,hops,hazards

def shortest(start,target,allowed,hops=frozenset()):
 parent={start:None};todo=deque([start])
 while todo:
  at=todo.popleft()
  if at==target:
   result=[]
   while at is not None:result.append(at);at=parent[at]
   return list(reversed(result))
  for to in adjacent(at):
   if to in hops:
    if to!=(at[0]-1,at[1]):continue
    to=at[0]-3,at[1]
   if to in allowed and to not in parent:parent[to]=at;todo.append(to)
 return None

def reachable(start,allowed,hops=frozenset()):
 seen={start};todo=[start]
 while todo:
  at=todo.pop()
  for to in adjacent(at):
   if to in hops:
    if to!=(at[0]-1,at[1]):continue
    to=at[0]-3,at[1]
   if to in allowed and to not in seen:seen.add(to);todo.append(to)
 return seen

def grass_cost(start,targets,walk,hops,kind):
 costs={start:0};todo=deque([start])
 while todo:
  at=todo.popleft()
  for to in adjacent(at):
   if to in hops:
    if to!=(at[0]-1,at[1]):continue
    to=at[0]-3,at[1]
   if to not in walk:continue
   price=int(kind(to)=='TALL_GRASS');v=costs[at]+price
   if v<costs.get(to,10**9):costs[to]=v;(todo.append if price else todo.appendleft)(to)
 return min((costs[q]for q in targets if q in costs),default=None)

def inspect(d,l,p,od,ol):
 start=time.perf_counter();checks=[]
 def test(name,value):checks.append(dict(check=name,passed=bool(value)))
 ob,ok,ow,oh,oz=geometry(od,ol,p);b,kind,walk,hops,hazards=geometry(d,l,p)
 test('all gameplay source fields exact; only path annotations/editorial review differ',all(d.get(k)==od.get(k)for k in set(d)|set(od)if k not in{'main_path_metatiles','loops','review_contract'}))
 test('only existing loop path annotation changed',len(d['loops'])==len(od['loops'])and all({k:v for k,v in n.items()if k!='path_metatiles'}=={k:v for k,v in o.items()if k!='path_metatiles'}for n,o in zip(d['loops'],od['loops'])))
 test('extent art recipes palette authority and all layout metadata exact',all(l.get(k)==ol.get(k)for k in set(l)|set(ol)if k!='layers'))
 test('every original terrain layer exact prefix',l['layers'][:len(ol['layers'])]==ol['layers'])
 delta=[dict(metatile=[i%30,i//30],before=f'{a:02x}',after=f'{z:02x}')for i,(a,z)in enumerate(zip(ob,b))if a!=z]
 test('all deltas inside approved meta24,6..28,11 study',bool(delta)and all(24<=v['metatile'][0]<28 and 6<=v['metatile'][1]<11 for v in delta))
 test('only existing native worked-soil and wooden-stake art added',all(v['after']in{'2c','77'}for v in delta))
 test('all original encounter cells exact and no added grass toll',{at for at in walk if kind(at)=='TALL_GRASS'}=={at for at in ow if ok(at)=='TALL_GRASS'})
 test('all original Cut quadrants and all hop quadrants exact',all(kind((x,y))==ok((x,y))for y in range(44)for x in range(60)if ok((x,y))=='CUT_TREE'or'HOP'in ok((x,y)))and not any(kind((x,y))=='CUT_TREE'and ok((x,y))!='CUT_TREE'for y in range(44)for x in range(60)))
 for label,callback in [('original bridge/four-hop-direction/long-return checks',check_piers),('zero mountable drain/closed8outerWater cells',check_water_boundary)]:
  try:value=callback(d,l,p);test(label,True)
  except AssertionError as error:test(label,False);checks[-1]['reason']=str(error)
 common,_,_=shared_check(d,l,p,b);test('shared source actor/door/path/grass checks clean',not common['errors'])
 test('ordinary main and retained Floor remain connected with native actors',not((reachable((58,14),ow,oh)&walk)-reachable((58,14),walk,hops)))
 prices=[]
 for label,a,z in [('east-west',(58,14),(2,30)),('west-east',(2,30),(58,14))]:
  before=grass_cost(a,{z},ow,oh,ok);after=grass_cost(a,{z},walk,hops,kind);prices.append(dict(id=label,before=before,candidate=after));test(label+' exact original whole-route7grass price',before==after==7)
  band={(x,y)for y in range(28,34)for x in range(8,14)};band_kind=lambda at:'TALL_GRASS'if at in band else'FLOOR';old_band=grass_cost(a,{z},ow,oh,band_kind);new_band=grass_cost(a,{z},walk,hops,band_kind);test(label+' exactly6requiredPastureGate entries',old_band==new_band==6);prices.append(dict(id=label+'-requiredPastureGateOnly',before=old_band,candidate=new_band))
 for item in d['items']:
  for label,st in [('east',(58,14)),('west',(2,30))]:
   targets=set(adjacent(tuple(item['tile'])));old=grass_cost(st,targets,ow,oh,ok);new=grass_cost(st,targets,walk,hops,kind);prices.append(dict(id=item['id']+'-'+label,before=old,candidate=new));test(item['id']+' '+label+' original optional grass price exact',old==new)
 dry={a for a in walk if kind(a)=='FLOOR'}-hazards;old_dry={a for a in ow if ok(a)=='FLOOR'}-oz
 approach=shortest((54,14),(52,22),dry);old_approach=shortest((54,14),(52,22),old_dry)
 test('actual native body/sight-aware dry approach changes10to16steps',old_approach is not None and approach is not None and len(old_approach)==11 and len(approach)==17)
 reverse=shortest((52,22),(54,14),dry);test('actual trainer-sight-aware dry return is16steps',reverse is not None and len(reverse)==17)
 test('working stakes rooted to existing east windbreak',all(kind((x,17))=='WALL'for x in range(50,58))and all(b[6*30+x]==0x2c for x in range(25,28)))
 test('two bank-head lanes48and49 remain ordinary at all four hop rows',all(kind((x,y))=='FLOOR'for x in[48,49]for y in range(14,18)))
 lane_proof=[]
 for y in range(14,18):
  legal=kind((48,y))=='FLOOR'and kind((47,y))=='HOP_LEFT'and kind((46,y))=='WALL'and kind((45,y))=='FLOOR'
  route=shortest((45,y),(48,y),walk-hazards)
  test(f'original hop lane{y} exact native jump reverseWall and ordinary>=33return',legal and route is not None and len(route)-1>=33)
  lane_proof.append(dict(upper=[48,y],hop=[47,y],reverse_wall=[46,y],landing=[45,y],ordinary_steps=len(route)-1 if route else None,ordinary_path=route))
 test('both original ledge side caps remain Wall',kind((47,13))==kind((47,18))=='WALL')
 test('zero new unsupported collisions',all(kind((x,y))==ok((x,y))or kind((x,y))in{'FLOOR','WALL'}for y in range(44)for x in range(60)))
 native_trainers=[]
 for a in d['trainers']:
  native=p['data']['trainers']['trainers'][a['id']];typed=p['data']['maps']['MnR02C00R00']['trainer_scripts'][a['id']]
  test(a['id']+' actual installed class and entire species/level roster match source',native['trainer_id']==a['id']and native['trainer_class']==a['trainer_class']==typed['trainer_class']and[(v['species'],v['level'])for v in native['party']]==[(v['species'],v['level'])for v in a['party']])
  native_trainers.append(dict(id=a['id'],name=native['name'],trainer_class=native['trainer_class'],party=[dict(species=v['species'],level=v['level'])for v in native['party']],full_native_template_sha256=canonical(native),typed_binding_sha256=canonical(typed),event_flag=typed['event_flag']))
 return dict(status='PASS'if all(v['passed']for v in checks)else'FAIL',checks=checks,terrain_delta=delta,shared_errors=common['errors'],grass_prices=prices,approach_before=old_approach,approach_candidate=approach,reverse_candidate=reverse,all_four_original_hop_lanes=lane_proof,native_trainer_authority=native_trainers,warm_check_ms=round((time.perf_counter()-start)*1000,3)),kind

def render_pair(out,p,ol,l):
 art=SourceArt(p);m=p['data']['maps']['MnR02C00R00'];player=oriented_sprite(npc_palette_pack(p,'day'),'SPRITE_CHRIS','south');save_render(player,out/'player-source-original.png')
 views=[('weak-nursery-apron',(48,15)),('east-working-field',(54,14)),('stake-end-turn',(49,16)),('reopened-board-return',(52,20)),('worked-nursery-shoulder',(51,12)),('fourth-hop-lane',(48,17)),('unchanged-toolcourt',(52,36))]
 pairs=[];wholes=[]
 for label,layout in [('before',ol),('candidate',l)]:
  module=dict(m);module['blocks']=layout_blocks(layout);ground=art.render('mn_prairie_kanto',30,22,module['blocks']);save_render(ground,out/(label+'-native-terrain-original.png'));scene=actors(p,ground,module);priority_scene(p,module,scene,ground,(0,0,960,704));save_render(scene,out/(label+'-whole-source-rgba.png'));wholes.append((label,scene));frames=[]
  for name,(x,y)in views:
   image=scene.copy();image.alpha_composite(player,(x*16,y*16));priority_scene(p,module,image,ground,(x*16,y*16,(x+2)*16,(y+2)*16));save_render(image,out/f'{label}-{name}-full-player-original-rgba.png');crop=render_stage(image,'crop',out,f'{label}-{name}',(x*16-80,y*16-72,x*16+80,y*16+72));save_render(crop,out/f'{label}-{name}-native160.png');frames.append((name,crop));pairs.append((label+' '+name,crop))
  save_render(room_sheet(frames,3),out/(label+'-source160-sheet.png'))
 whole=Image.new('RGB',(960,1440),(20,27,34));dr=ImageDraw.Draw(whole)
 for n,(label,image)in enumerate(wholes):dr.text((8,n*720+3),label+' installed native-art source composite; candidate not GPU/controller proof',fill='white');whole.paste(image,(0,n*720+16))
 save_render(whole,out/'whole-before-after-native32.png');ordered=[]
 for n in range(len(views)):ordered.extend([pairs[n],pairs[len(views)+n]])
 save_render(room_sheet(ordered,2),out/'same-position-before-after-source160.png');write(out/'native-views.json',[dict(id=name,map='MnR02C00R00',core=at)for name,at in views])

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--source',type=Path,required=True);ap.add_argument('--self-test',action='store_true');args=ap.parse_args();out=args.source
 assert sha(PACK.read_bytes())==PACK_SHA;p=load_pack(PACK);od=json.loads((out/'before-MN_R02.json').read_text());ol=json.loads((out/'before-MN_R02.layout.json').read_text());d=json.loads((out/'MN_R02.candidate.json').read_text());l=json.loads((out/'MN_R02.layout.candidate.json').read_text());m=p['data']['maps']['MnR02C00R00'];assert m['blocks']==layout_blocks(ol),'Native preimage differs from frozen source'
 r,kind=inspect(d,l,p,od,ol);render_pair(out,p,ol,l)
 contract=dict(parent_pack=str(PACK.relative_to(ROOT)),parent_file_sha256=PACK_SHA,map_count=len(p['data']['maps']),native_source_preimage_exact=True,module='MnR02C00R00',module_sha256=canonical(m),field_sha256={k:canonical(v)for k,v in m.items()if k!='blocks'},native_art_sha256=canonical(p['data']['tilesets']['mn_prairie_kanto']),source_sha256={name:sha((out/name).read_bytes())for name in['before-MN_R02.json','before-MN_R02.layout.json','MN_R02.candidate.json','MN_R02.layout.candidate.json']},allowed_changes='This existing exterior blocks + uniquely owned map_blocks payload + derived pack identity only. All other module fields, rooms, actors, scripts, programs, trainers, items, flags, art/audio/HOME/recovery/report/catalogs exact. No new lowering.',exact_metatile_changes=r['terrain_delta'],original_native_trainers=r['native_trainer_authority']);write(out/'native-preimage-contract.json',contract)
 if args.self_test:
  assert r['status']=='PASS',[v for v in r['checks']if not v['passed']]
  negative=[]
  def point(nl,at,block):nl['layers'].append(dict(kind='point',name='Adversarial physical fixture',position=at,block=block))
  def reject(name,change):
   nd,nl=deepcopy(d),deepcopy(l);change(nd,nl);nr,_=inspect(nd,nl,p,od,ol);failed=[v['check']for v in nr['checks']if not v['passed']];assert failed,name+' accepted';folder=out/'rejected-inputs'/name;folder.mkdir(parents=True,exist_ok=False);write(folder/'dossier.json',nd);write(folder/'layout.json',nl);write(folder/'failed-checks.json',failed);negative.append(dict(id=name,rejected=True,failed_checks=failed))
  for label,x in [('left',25),('middle',26),('rooted-right',27)]:reject(label+'-stake-gap',lambda d,l,x=x:point(l,[x,8],'31'))
  reject('first-two-hop-head-lanes-blocked',lambda d,l:point(l,[24,7],'0f'))
  reject('last-two-hop-head-lanes-blocked',lambda d,l:point(l,[24,8],'0f'))
  reject('reverse-bank-made-ordinary',lambda d,l:point(l,[23,7],'31'))
  reject('ledge-south-cap-gap',lambda d,l:point(l,[23,9],'01'))
  reject('drain-fence-Surf-mount-gap',lambda d,l:point(l,[14,5],'01'))
  reject('bridge-south-rail-removed',lambda d,l:point(l,[16,13],'43'))
  reject('required-six-grass-neck-dried',lambda d,l:l['layers'].append(dict(kind='block_rect',name='Bad dry pasture',bounds=[4,14,7,17],block='80')))
  reject('extra-grass-tax-added',lambda d,l:point(l,[26,6],'81'))
  reject('trainer-party-drift',lambda d,l:d['trainers'][0]['party'][0].update(level=99))
  reject('resident-anchor-drift',lambda d,l:d['residents'][0].update(tile=[38,24]))
  reject('room-return-moved-onto-new-stakes',lambda d,l:d['doors'][1].update(return_tile=[52,17]))
  r['negative_cases']=negative
 r.update(source_only=True,native_published=False,parent_pack_unchanged=sha(PACK.read_bytes())==PACK_SHA,canonical_sources_unchanged=all((ROOT/'planning/minnesota/routes'/name).read_bytes()==(out/('before-'+name)).read_bytes()for name in['MN_R02.json','MN_R02.layout.json']))
 write(out/'review.json',r);(out/'review_r02_working_verge.py.snapshot').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(status=r['status'],checks=len(r['checks']),changes=len(r['terrain_delta']),failed=[v for v in r['checks']if not v['passed']],shared_errors=r['shared_errors'],approach_steps=len(r['approach_candidate'])-1 if r['approach_candidate']else None,negatives=len(r.get('negative_cases',[]))),indent=2));assert r['status']=='PASS'
if __name__=='__main__':main()
