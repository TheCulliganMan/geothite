#!/usr/bin/env python3
"""Bounded western R15 junction review, composed after frozen shore v03."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image, ImageDraw
from render_archive import save_render
from render_reference import layout_blocks, load_pack
import review_r15_refinement as shore

ROOT=Path(__file__).resolve().parents[3]
BEFORE=ROOT/'planning/minnesota/renders/central-lakes/west-apron-20261027/before-v03'
VIEWS=[('WESTERN_APRON',(22,26)),('LANDING_JUNCTION',(22,24)),('MEADOW_LANDING',(23,21)),('FLOWER_BEND',(21,29)),('RON_SOUTH_APPROACH',(17,25)),('BRIDGE_APPROACH',(27,29)),('SHORE_WALK_SOUTH',(47,23)),('LAKE_CABIN',(53,15))]

def inspect(d,l,pack,od,ol):
 started=time.perf_counter();ob,ok,oa,oreach,oc,oh=shore.graph(od,ol,pack);b,kind,allowed,reach,cost,hazards=shore.graph(d,l,pack);checks=[]
 def check(name,value):checks.append({'check':name,'passed':bool(value)})
 check('all noneditorial dossier fields exact',{k:v for k,v in d.items()if k!='review_contract'}=={k:v for k,v in od.items()if k!='review_contract'})
 check('all original shore-v03 dossier review fields remain exact',all(d['review_contract'].get(k)==v for k,v in od['review_contract'].items()))
 check('all layout metadata exact',all(l.get(k)==ol.get(k)for k in set(l)|set(ol)if k!='layers'))
 check('entire frozen shore-v03 layer prefix exact',l['layers'][:len(ol['layers'])]==ol['layers'])
 changes=[{'metatile':[i%32,i//32],'before':f'{a:02x}','after':f'{z:02x}'}for i,(a,z)in enumerate(zip(ob,b))if a!=z]
 check('western bounded changes only meta9..13/11..14',bool(changes)and all(9<=c['metatile'][0]<14 and 11<=c['metatile'][1]<15 for c in changes))
 check('bounded native forest ordinary edge and timber bank art only',all(c['after']in{'0f','6d','5f'}for c in changes))
 check('complete approved eastern shore terrain remains exact',all(b[y*32+x]==ob[y*32+x]for y in range(18)for x in range(14,32)))
 check('no placed Cut quadrant',not any(kind((x,y))=='CUT_TREE'for y in range(36)for x in range(64)))
 check('every original water/shore cell and closed outer perimeter exact',all(b[i]==ob[i]for i in range(len(b))if any(t in{'WATER','BUOY'}for t in pack['data']['tilesets']['kanto']['collision'][f'{ob[i]:02x}']))and all(kind((x,y))==ok((x,y))for y in range(36)for x in range(64)if x in{0,63}or y in{0,35}))
 for name,cb in [('original docks bridges and directed Ron/meadow contract',shore.check_piers),('original actual Surf closure',shore.check_water_boundaries)]:
  try:cb(d,l,pack);check(name,True)
  except AssertionError as e:check(name,False);checks[-1]['reason']=str(e)
 common,_,_=shore.generic_check(d,l,pack,b);check('shared full route geometry has no errors',not common['errors'])
 east,west=[tuple(v*2 for v in e['gate_metatile'])for e in d['endpoints']];seen=reach(east);oldseen=oreach(east)
 check('both actual endpoints physically reachable both ways',west in seen and east in reach(west))
 check('no retained original reachable walking cell stranded',not((oldseen&allowed)-seen))
 for label,st,target in [('west-to-east',west,{east}),('east-to-west',east,{west})]:check(label+' exact trainer avoidance and grass price',oc(st,target,oh)==cost(st,target,hazards))
 check('Ron remains mandatory westbound and avoidable eastbound',west not in reach(east,hazards)and east in reach(west,hazards))
 prices=[]
 for a in d['items']:
  for label,st in [('west',west),('east',east)]:
   target=set(shore.adjacent(tuple(a['tile'])));before=oc(st,target);after=cost(st,target);check(a['id']+' '+label+' exact resource grass cost',before==after);prices.append({'id':a['id']+'-'+label,'before':before,'candidate':after})
 check('every original grass quadrant exact',all(kind((x,y))==ok((x,y))for y in range(36)for x in range(64)if kind((x,y))=='TALL_GRASS'or ok((x,y))=='TALL_GRASS'))
 check('entire native meadow hop upper face course landing unchanged',all(kind((x,y))==ok((x,y))for y in range(16,22)for x in range(21,27)))
 ordinary={p for p in allowed if kind(p)in{'FLOOR','TALL_GRASS'}}
 def shortest(start,target,points):
  seen={start:0};q=deque([start])
  while q:
   p=q.popleft()
   if p==target:return seen[p]
   for n in shore.adjacent(p):
    if n in points and n not in seen:seen[n]=seen[p]+1;q.append(n)
  return None
 returns=[];oldordinary={p for p in oa if ok(p)in{'FLOOR','TALL_GRASS'}}
 for x in range(22,26):
  upper=(x,17);landing=(x,20);a=shortest(landing,upper,oldordinary);z=shortest(landing,upper,ordinary);check(f'hop lane{x} legal ordinary return retained',z is not None and z==a and z>=25);returns.append({'lane':x,'before_steps':a,'candidate_steps':z})
 check('landing stem remains three core lanes',all(kind((x,y))=='FLOOR'for y in range(22,28)for x in range(22,25)))
 check('attached timber shore bank closes right skirt',all(kind((25,y))=='WALL'for y in range(22,28))and all(kind((26,y))=='WALL'for y in range(22,28)))
 check('Ron road remains two core rows and joins landing stem',all(kind((x,y))=='FLOOR'for y in[24,25]for x in range(18,25)))
 check('lower meadow bank makes one readable road turn',all(kind((x,y))=='WALL'for y in[26,27]for x in range(18,21))and all(kind((x,29))=='FLOOR'for x in range(21,30)))
 for name,at in VIEWS:check(name+' reachable unoccupied Floor',kind(at)=='FLOOR'and at in seen)
 wall={p for p in[(x,y)for y in range(36)for x in range(64)]if kind(p)=='WALL'};oldwall={p for p in wall if ok(p)=='WALL'}
 def attached(seed):
  found={seed};q=[seed]
  while q:
   for p in shore.adjacent(q.pop()):
    if p in wall and p not in found:found.add(p);q.append(p)
  return bool(found&oldwall)
 check('every new solid quadrant attaches to original bank',all(attached(p)for p in wall-oldwall))
 path_contracts=[]
 cases=[('west-to-bridge',(22,20),[(22,24),(22,28),(46,28)]),('return-from-bridge',(46,28),[(21,28),(21,25),(14,25)]),('exact31-ordinary-hopreturn',(22,20),[(22,25),(15,25),(15,15),(22,15),(22,17)]),('exit-after-hop',(22,20),[(22,25),(4,25)]),('approved-east-S-course',(46,28),[(46,21),(50,21),(50,20),(54,20),(54,24),(60,24)])]
 for name,start,targets in cases:
  cursor=start;steps=[];valid=start in ordinary
  for target in targets:
   if cursor[0]!=target[0]and cursor[1]!=target[1]:valid=False;break
   dx=(target[0]>cursor[0])-(target[0]<cursor[0]);dy=(target[1]>cursor[1])-(target[1]<cursor[1])
   while cursor!=target:
    cursor=(cursor[0]+dx,cursor[1]+dy);steps.append(cursor);valid=valid and cursor in ordinary
  check(name+' every explicit input step has legal native collision/no actor',valid)
  if name=='exact31-ordinary-hopreturn':check('explicit cardinal return is exactly31tiles',len(steps)==31)
  path_contracts.append({'id':name,'start':start,'targets':targets,'steps':steps,'distance_core_steps':len(steps),'ordinary_native_walkable':valid})
 check('old20,28 and18,22 waypoints are now honestly blocked',kind((20,28))=='WALL'and kind((18,22))=='WALL')
 return {'status':'PASS'if all(c['passed']for c in checks)else'FAIL','checks':checks,'terrain_changes':changes,'resource_grass_prices':prices,'original_hop_ordinary_returns':returns,'explicit_controller_path_contracts':path_contracts,'shared_route_check':common,'warm_source_check_ms':round((time.perf_counter()-started)*1000,3)},kind,seen

def main():
 p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--self-test',action='store_true');args=p.parse_args();out=args.output;out.mkdir(parents=True,exist_ok=False)
 packpath=ROOT/'content-packs/minnesota-northstar-lake-country.crystalpack';assert shore.sha(packpath.read_bytes())==shore.PACK_SHA;pack=load_pack(packpath)
 source=ROOT/'planning/minnesota/routes';dpath=source/'MN_R15.west-apron.candidate.json';lpath=source/'MN_R15.west-apron.layout.json';beforepaths=[BEFORE/'MN_R15.refinement.candidate.json',BEFORE/'MN_R15.refinement.layout.json'];assert [shore.sha(f.read_bytes())for f in beforepaths]==['57bdf7662fadc71473d4a9ca610f62141326abaf69fb6923093796d0ab3c3773','04c1a8ba6e19e0e267c7876596f53d9a0a50dfb65fbe17b20a4f5c05d4a5b335']
 od,ol=[json.loads(f.read_bytes())for f in beforepaths];d,l=[json.loads(f.read_bytes())for f in[dpath,lpath]];report,_,_=inspect(d,l,pack,od,ol)
 for f in[dpath,lpath,Path(__file__)]: (out/f.name).write_bytes(f.read_bytes())
 shore.VIEWS=VIEWS;started=time.perf_counter();before=shore.render(od,ol,pack,out,'before');after=shore.render(d,l,pack,out,'candidate');pair=Image.new('RGB',(1024,1200),(19,29,40));draw=ImageDraw.Draw(pair);draw.text((12,6),'BEFORE frozen shore v03 | source composite',fill='white');pair.paste(before,(0,24));draw.text((12,606),'WESTERN CANDIDATE | landing junction + fitted timber shore bank | not native gameplay',fill='white');pair.paste(after,(0,624));save_render(pair,out/'before-after-whole-native32.png');report['complete_positive_render_ms']=round((time.perf_counter()-started)*1000,3)
 if args.self_test:
  negatives=[]
  def reject(name,mutate):
   nd,nl=deepcopy(d),deepcopy(l);mutate(nd,nl);r,_,_=inspect(nd,nl,pack,od,ol);fails=[v for v in r['checks']if not v['passed']];assert fails,name+' accepted';folder=out/'rejected-inputs'/name;folder.mkdir(parents=True);shore.write(folder/'dossier.json',nd);shore.write(folder/'layout.json',nl);shore.write(folder/'reasons.json',fails);negatives.append({'id':name,'rejected':True,'failed':[v['check']for v in fails]})
  point=lambda at,b:{'kind':'point','name':'Adversarial source','position':at,'block':b}
  reject('restore-empty-western-apron',lambda nd,nl:nl['layers'].append({'kind':'block_rect','name':'Erase bank','bounds':[9,11,14,15],'block':'31'}))
  reject('isolate-timber-toe',lambda nd,nl:nl['layers'].append(point([12,11],'31')))
  reject('block-three-lane-landing-stem',lambda nd,nl:nl['layers'].append(point([11,11],'0f')))
  reject('bypass-ron-through-north-copse',lambda nd,nl:nl['layers'].append(point([8,11],'31')))
  reject('reverse-original-meadow-hop',lambda nd,nl:nl['layers'].append(point([11,9],'31')))
  reject('new-incidental-cut',lambda nd,nl:nl['layers'].append(point([10,13],'06')))
  reject('change-original-grass-price',lambda nd,nl:nl['layers'].append(point([4,12],'31')))
  reject('damage-approved-shore-turn',lambda nd,nl:nl['layers'].append(point([24,9],'31')))
  reject('move-ron',lambda nd,nl:nd['trainers'][0].update(tile=[18,24]))
  reject('alter-original-item',lambda nd,nl:nd['items'][0].update(item='RARE_CANDY'))
  report['negative_mutations']=negatives
 
 # Keep the historical centered crop and add an honest clamped edge camera.
 for label in ['before','candidate']:
  for probe in shore.weak_cameras(od if label=='before'else d,ol if label=='before'else l,pack):
   x,y=probe['tile'];full=Image.open(out/(label+'-'+probe['id']+'-foreground-player-rgba.png'));left=max(0,min(x*16-80,full.width-160));top=max(0,min(y*16-72,full.height-144));clamped=shore.render_stage(full,'crop',out,label+'-'+probe['id']+'-clamped',(left,top,left+160,top+144));save_render(clamped,out/(label+'-'+probe['id']+'-clamped-native160-rgba.png'));clamped=shore.render_stage(clamped,'convert',out,label+'-'+probe['id']+'-clamped-rgb','RGB');save_render(clamped,out/(label+'-'+probe['id']+'-clamped-native160.png'))
 report.update(source_sha256={f.name:shore.sha(f.read_bytes())for f in[dpath,lpath]},before_sha256={f.name:shore.sha(f.read_bytes())for f in beforepaths},pack_sha256=shore.PACK_SHA,scope='Source-only composition after approved frozen shore v03, current538 art/native western preimage; no written candidate pack, GPU or controller proof.');shore.write(out/'review.json',report);print(json.dumps({'status':report['status'],'checks':len(report['checks']),'changes':len(report['terrain_changes']),'negatives':len(report.get('negative_mutations',[])),'failed':[c for c in report['checks']if not c['passed']],'returns':report['original_hop_ordinary_returns'],'source_sha256':report['source_sha256']},indent=2))
 if report['status']!='PASS':raise SystemExit(1)
if __name__=='__main__':main()
