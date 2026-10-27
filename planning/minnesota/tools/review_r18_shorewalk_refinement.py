#!/usr/bin/env python3
"""Source-only R18 shore-watch circuit with native art and bounded route proof."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import importlib.util
import json
from pathlib import Path
import time

from PIL import Image,ImageDraw
from render_archive import save_render
from render_reference import SourceArt,layout_blocks,load_pack,npc_palette_pack,room_sheet
from render_stages import render_stage
from review_north_shore import oriented_sprite
from review_league_approach import check as generic_check
from review_mille_lacs_shore import check_lake,walking_design
from review_r15_refinement import graph,sha,write,adjacent

ROOT=Path(__file__).resolve().parents[3]
BEFORE=ROOT/'planning/minnesota/renders/mille-lacs-shore/refinement-20261027/before-da27'
PACK_SHA='da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50'

def shortest(start,target,allowed):
 seen={start:None};q=deque([start])
 while q:
  p=q.popleft()
  if p==target:
   route=[]
   while p is not None:route.append(p);p=seen[p]
   return list(reversed(route))
  for n in adjacent(p):
   if n in allowed and n not in seen:seen[n]=p;q.append(n)
 return None

def inspect(d,l,p,od,ol):
 started=time.perf_counter();ob,ok,oa,orr,oc,oh=graph(od,ol,p);b,kind,allowed,reach,cost,hazards=graph(d,l,p);checks=[]
 def check(name,value):checks.append({'check':name,'passed':bool(value)})
 check('all existing gameplay/story/event/art source fields exact',all(d.get(k)==od.get(k)for k in set(d)|set(od)if k not in{'loops','viewpoints','review_contract'}))
 check('all existing views exact prefix',d['viewpoints'][:len(od['viewpoints'])]==od['viewpoints'])
 check('exact one ordinary two-ended shore loop added',d['loops'][:len(od['loops'])]==od['loops']and len(d['loops'])==len(od['loops'])+1 and d['loops'][-1]['id']=='PUBLIC_SHORE_WATCH_LOOP')
 check('all source art extent materials and piers metadata exact',all(l.get(k)==ol.get(k)for k in set(l)|set(ol)if k!='layers'))
 check('all original terrain is exact layer prefix',l['layers'][:len(ol['layers'])]==ol['layers'])
 delta=[{'metatile':[i%40,i//40],'before':f'{a:02x}','after':f'{z:02x}'}for i,(a,z)in enumerate(zip(ob,b))if a!=z]
 check('new terrain stays only shore link/dune margin/shore-fit halo meta28..36/6..10',bool(delta)and all(28<=c['metatile'][0]<37 and 6<=c['metatile'][1]<11 for c in delta))
 check('exact existing deck rail and closed corner art only',all(c['after']in{'82','81','40','49','42','03','54','55','76','79','7a'}for c in delta))
 common,_,_=generic_check(walking_design(d),l,p,b);check('complete ordinary source geometry has no errors',not common['errors'])
 try:lake=check_lake(d,l,p);check('all original flat shore quest piers Surf island closures and seam proofs',True)
 except AssertionError as e:lake={'error':str(e)};check('all original flat shore quest piers Surf island closures and seam proofs',False)
 west,east=[tuple(2*v for v in e['gate_metatile'])for e in d['endpoints']];seen=reach(west);oldseen=orr(west)
 check('all retained reachable ordinary cells remain connected',not((oldseen&allowed)-seen))
 check('ordinary cross-route travel works both ways',east in seen and west in reach(east))
 check('all original grass quadrants exact',all(kind((x,y))=='TALL_GRASS'for y in range(48)for x in range(80)if ok((x,y))=='TALL_GRASS'))
 check('no Cut or directional hop collision added',not any(kind((x,y))=='CUT_TREE'or'HOP'in kind((x,y))for y in range(48)for x in range(80)))
 prices=[]
 for label,start,target in [('west-east',west,east),('east-west',east,west)]:
  a=oc(start,{target});z=cost(start,{target});check(label+' exact necessary habitat entry price',a==z and z>=6);prices.append({'id':label,'before':a,'candidate':z})
 for a in d['items']:
  if a['id']in d['surf_only_actor_ids']:continue
  for label,st in [('west',west),('east',east)]:
   target=set(adjacent(tuple(a['tile'])));old=oc(st,target);new=cost(st,target);check(a['id']+' '+label+' exact optional grass price',old==new);prices.append({'id':a['id']+'-'+label,'before':old,'candidate':new})
 for label,a in [('west',west),('east',east)]:check(label+' original trainer avoidance remains possible',bool(orr(a,oh)&{east if a==west else west})==bool(reach(a,hazards)&{east if a==west else west}))
 newfoot={(x,y)for y in range(16,20)for x in range(58,72)if kind((x,y))=='FLOOR'}
 exposed=[{'at':a,'water':n}for a in newfoot for n in adjacent(a)if kind(n)=='WATER']
 check('all new shorewalk footing protected from incidental Surf gaps',not exposed)
 gates=sorted((a,n)for a in newfoot for n in adjacent(a)if n in allowed and n not in newfoot)
 check('exactly two ordinary shorewalk entrances and no third rail gap',gates==[((58,18),(57,18)),((71,18),(72,18))])
 check('central shorewalk exactly two native walking rows',all(kind((x,16))==kind((x,19))=='WALL'and kind((x,17))==kind((x,18))=='FLOOR'for x in range(60,70)))
 start,target=(59,18),(72,18);oldroute=shortest(start,target,oa);newroute=shortest(start,target,allowed);check('shorewalk makes an actual shorter walking circuit',oldroute is not None and newroute is not None and len(newroute)<len(oldroute))
 def water_pixels(at):
  x,y=at;return sum(kind((a,z))=='WATER'for z in range(y-4,y+5)for a in range(x-5,x+5))
 check('new lake-view frame actually shows substantial open water',water_pixels((65,18))>=20)
 for name,at in [('west-gate',(59,18)),('lake-view',(65,18)),('beach-gate',(71,18))]:check(name+' new ordinary Floor camera reachable',kind(at)=='FLOOR'and at in seen)
 return {'status':'PASS'if all(c['passed']for c in checks)else'FAIL','checks':checks,'terrain_changes':delta,'shared_geometry':common,'lake_contract':lake,'grass_prices':prices,'new_foot_water_gaps':exposed,'exact_shorewalk_gate_edges':gates,'coast_shortcut':{'start':start,'target':target,'before_steps':len(oldroute)-1 if oldroute else None,'candidate_steps':len(newroute)-1 if newroute else None,'before_path':oldroute,'candidate_path':newroute},'lake_view_water_core_cells':water_pixels((65,18)),'warm_source_check_ms':round((time.perf_counter()-started)*1000,3)},kind,seen

def render(d,l,p,out,label):
 w,h=l['dimensions_metatiles'];terrain=SourceArt(p).render(l['compiled_tileset'],w,h,layout_blocks(l),group='day');save_render(terrain,out/(label+'-original-native-terrain.png'));scene=render_stage(terrain,'convert',out,label+'-original-rgba','RGBA');pal=npc_palette_pack(p,'day')
 for k in['trainers','residents','items']:
  for a in d[k]:scene.alpha_composite(oriented_sprite(pal,a.get('sprite','SPRITE_POKE_BALL'),a.get('facing','south')),(a['tile'][0]*16,a['tile'][1]*16))
 save_render(scene,out/(label+'-whole-source-rgba.png'));rgb=render_stage(scene,'convert',out,label+'-whole-rgb','RGB');save_render(rgb,out/(label+'-whole-route.png'));player=oriented_sprite(pal,'SPRITE_CHRIS','south');save_render(player,out/(label+'-player-original-rgba.png'));frames=[]
 helper=ROOT/'planning/minnesota/renders/caves/r13-native/review_native.py';spec=importlib.util.spec_from_file_location('r18_priority',helper);native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native);adapted=dict(p);adapted['data']=dict(p['data']);adapted['data']['tilesets']=dict(p['data']['tilesets']);adapted['data']['tilesets']['cave']=p['data']['tilesets'][l['compiled_tileset']];adapted['runtime_files']=dict(p['runtime_files'])
 for old,new in [('gfx/tilesets/cave.png','gfx/tilesets/'+l['compiled_tileset']+'.png'),('data/tilesets/cave_metatiles.bin','data/tilesets/'+l['compiled_tileset']+'_metatiles.bin')]:adapted['runtime_files'][old]=p['runtime_files'][new]
 module={'blocks':layout_blocks(l),'attributes':{'width':w,'height':h}};priority=[]
 _,kind,allowed,reach,_,_=graph(d,l,p);start=tuple(v*2 for v in d['endpoints'][0]['gate_metatile']);seen=reach(start);sizes={};best=(0,0,0)
 for y in range(h*2):
  for x in range(w*2):
   size=1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0))if(x,y)in seen and kind((x,y))=='FLOOR'else 0;sizes[x,y]=size
   if size>best[0]:best=(size,x-size+1,y-size+1)
 size,x,y=best;weak={'id':'WEAKEST_GLOBAL_CLEAR','tile':[x+size//2,y+size//2],'largest_floor_square_bounds':[x,y,x+size,y+size],'width_core':size};write(out/(label+'-weakest-camera.json'),weak)
 views=d['viewpoints']+[weak]
 for v in views:
  x,y=v['tile'];framed=scene.copy();framed.alpha_composite(player,(x*16,y*16));save_render(framed,out/(label+'-'+v['id']+'-full-player-rgba.png'))
  try:cover=native.cave_foreground_pixels(adapted,module,(x,y),player)
  except AssertionError:cover=set();priority.append({'id':v['id'],'foreground_scope':'Native non-wall/ordinary palette effect outside this bounded Floor/Wall helper; source-only, native capture required.'})
  for px,py in cover:framed.putpixel((x*16+px,y*16+py),terrain.getpixel((x*16+px,y*16+py))+(255,))
  save_render(framed,out/(label+'-'+v['id']+'-foreground-player-rgba.png'));priority.append({'id':v['id'],'wall_foreground_pixels':len(cover)});left=max(0,min(x*16-80,framed.width-160));top=max(0,min(y*16-72,framed.height-144));im=render_stage(framed,'crop',out,label+'-'+v['id'],(left,top,left+160,top+144));save_render(im,out/(label+'-'+v['id']+'-original160-rgba.png'));im=render_stage(im,'convert',out,label+'-'+v['id']+'-rgb','RGB');save_render(im,out/(label+'-'+v['id']+'-native160.png'));frames.append((v['id'],im))
 save_render(room_sheet(frames,columns=3),out/(label+'-decision-cameras.png'));write(out/(label+'-foreground-priority.json'),priority);return rgb

def main():
 a=argparse.ArgumentParser();a.add_argument('--output',type=Path,required=True);a.add_argument('--self-test',action='store_true');args=a.parse_args();out=args.output;out.mkdir(parents=True,exist_ok=False);packpath=ROOT/'content-packs/minnesota-northstar-lake-country.crystalpack';assert sha(packpath.read_bytes())==PACK_SHA;p=load_pack(packpath);beforepaths=[BEFORE/'MN_R18.json',BEFORE/'MN_R18.layout.json'];assert [sha(f.read_bytes())for f in beforepaths]==['c8b56362ddd07924284e7b2004d1a566cfef02cf141e66d7afa0fbc1aed69e91','bb122560bb6f12cba044a9caa710c3146e63d58810ddcce674e05c13f4b535f5'];od,ol=[json.loads(f.read_bytes())for f in beforepaths];sources=[ROOT/'planning/minnesota/routes'/n for n in['MN_R18.shorewalk.candidate.json','MN_R18.shorewalk.layout.json']];d,l=[json.loads(f.read_bytes())for f in sources];report,_,_=inspect(d,l,p,od,ol)
 for f in sources+[Path(__file__)]: (out/f.name).write_bytes(f.read_bytes())
 started=time.perf_counter();before=render(od,ol,p,out,'before');after=render(d,l,p,out,'candidate');pair=Image.new('RGB',(1280,1584),(19,29,40));draw=ImageDraw.Draw(pair);draw.text((12,6),'BEFORE current538 native-art source composite',fill='white');pair.paste(before,(0,24));draw.text((12,798),'CANDIDATE public shore-watch circuit | source composite, not gameplay',fill='white');pair.paste(after,(0,816));save_render(pair,out/'before-after-whole-native32.png');report['complete_positive_render_ms']=round((time.perf_counter()-started)*1000,3)
 if args.self_test:
  negatives=[]
  def reject(name,mutation):
   nd,nl=deepcopy(d),deepcopy(l);mutation(nd,nl);r,_,_=inspect(nd,nl,p,od,ol);failed=[c for c in r['checks']if not c['passed']];assert failed,name+' unexpectedly accepted';folder=out/'rejected-inputs'/name;folder.mkdir(parents=True);write(folder/'dossier.json',nd);write(folder/'layout.json',nl);write(folder/'reasons.json',failed);negatives.append({'id':name,'rejected':True,'failed_checks':[c['check']for c in failed]})
  point=lambda at,b:{'kind':'point','name':'Adversarial physical source','position':at,'block':b}
  reject('north-rail-third-entrance',lambda nd,nl:nl['layers'].append(point([32,8],'81')))
  reject('south-rail-surf-gap',lambda nd,nl:nl['layers'].append(point([32,9],'71')))
  reject('close-west-gate',lambda nd,nl:nl['layers'].append(point([29,9],'42')))
  reject('block-coast-course',lambda nd,nl:nl['layers'].append(point([32,9],'82')))
  reject('open-east-corner-water-gap',lambda nd,nl:nl['layers'].append(point([35,8],'81')))
  reject('erase-required-grass-neck',lambda nd,nl:nl['layers'].append({'kind':'block_rect','name':'Dry bypass','bounds':[18,5,21,9],'block':'02'}))
  reject('dry-link-to-surf-only-islet',lambda nd,nl:nl['layers'].append({'kind':'block_rect','name':'Free islet','bounds':[30,9,31,18],'block':'01'}))
  reject('incidental-cut-in-margin',lambda nd,nl:nl['layers'].append(point([32,7],'5b')))
  reject('move-reed-report-actor',lambda nd,nl:nd['residents'][0].update(tile=[65,18]))
  reject('alter-quest-reward-flag',lambda nd,nl:nd['quest_contract'].update(reward_flag='EVENT_MN_SHORE_WATCH_01'))
  reject('change-native-trainer-party',lambda nd,nl:nd['trainers'][0]['party'][0].update(level=50))
  report['negative_mutations']=negatives
 report.update(source_sha256={f.name:sha(f.read_bytes())for f in sources},baseline_pack_sha256=PACK_SHA,scope='Unpublished source-art/layout/native collision proof only; no GPU/controller/publication/earned progression claim.');write(out/'review.json',report);print(json.dumps({'status':report['status'],'checks':len(report['checks']),'changed_metatiles':len(report['terrain_changes']),'failed':[v for v in report['checks']if not v['passed']],'source_shortcut':{k:v for k,v in report['coast_shortcut'].items()if not k.endswith('_path')},'water_cells_visible':report['lake_view_water_core_cells'],'shared_errors':report['shared_geometry']['errors'],'lake_error':report['lake_contract'].get('error')},indent=2))
 if report['status']!='PASS':raise SystemExit(1)
if __name__=='__main__':main()
