#!/usr/bin/env python3
"""R15 attached woodland shore-walk candidate: exact source, directed collision and art."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import importlib.util
import json
from pathlib import Path
import time

from PIL import Image,ImageDraw
from render_reference import SourceArt,layout_blocks,load_pack,npc_palette_pack,room_sheet
from render_archive import save_render
from render_stages import render_stage
from review_north_shore import oriented_sprite
from review_league_approach import check as generic_check
from review_central_lakes import check_piers,check_water_boundaries

ROOT=Path(__file__).resolve().parents[3]
BEFORE=ROOT/'planning/minnesota/renders/central-lakes/refinement-20261027/before-da27'
PACK_SHA='da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50'
D_SHA='2d3d16ebd1e4488c378b893e84db134ba0e24054cd8d493c0ee329aaf294e9d8'
L_SHA='5cda48486308bcebef1937ee5a6045af45a6240ca76a44f93fdc5bab2321dd85'
PASSABLE={'FLOOR','TALL_GRASS','HOP_DOWN','LADDER','DOOR'}
VECTORS={'north':(0,-1),'south':(0,1),'west':(-1,0),'east':(1,0)}
VIEWS=[('SHORE_WALK_SOUTH',(47,23)),('LAKE_CABIN',(53,15)),('DOCK_BRANCH',(46,18)),('NORTH_TONGUE',(49,22)),('CENTER_TURN',(51,20)),('CABIN_GATE',(54,19)),('EAST_ROAD',(55,24)),('LAKE_SIGN',(49,26))]

def sha(raw):return hashlib.sha256(raw).hexdigest()
def write(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
def adjacent(p):return [(p[0]-1,p[1]),(p[0]+1,p[1]),(p[0],p[1]-1),(p[0],p[1]+1)]

def graph(d,l,pack):
 b=layout_blocks(l);w,h=l['dimensions_metatiles'];c=pack['data']['tilesets'][l['compiled_tileset']]['collision'];occupied={tuple(a['tile'])for key in ['trainers','residents','items']for a in d[key]}
 def kind(p):
  x,y=p
  return c[f'{b[y//2*w+x//2]:02x}'][y%2*2+x%2]if 0<=x<w*2 and 0<=y<h*2 else'WALL'
 allowed={(x,y)for y in range(h*2)for x in range(w*2)if kind((x,y))in PASSABLE and(x,y)not in occupied}
 def edges(p,forbidden):
  for q in adjacent(p):
   if kind(p)=='HOP_DOWN'and q==(p[0],p[1]+1):q=(p[0],p[1]+2)
   if q in allowed and q not in forbidden:yield q
 def reach(start,forbidden=frozenset()):
  seen={start};todo=[start]
  while todo:
   for q in edges(todo.pop(),forbidden):
    if q not in seen:seen.add(q);todo.append(q)
  return seen
 def cost(start,targets,forbidden=frozenset()):
  costs={start:0};queue=deque([start])
  while queue:
   p=queue.popleft()
   for q in edges(p,forbidden):
    step=int(kind(q)=='TALL_GRASS');v=costs[p]+step
    if v<costs.get(q,10**9):costs[q]=v;(queue.append if step else queue.appendleft)(q)
  values=[costs[q]for q in targets if q in costs]
  return min(values)if values else None
 hazards=set()
 for a in d['trainers']:
  x,y=a['tile'];dx,dy=VECTORS[a['facing']]
  for i in range(1,a['sight_range']+1):
   at=(x+dx*i,y+dy*i)
   if kind(at)not in{'FLOOR','TALL_GRASS'}:break
   hazards.add(at)
 return b,kind,allowed,reach,cost,hazards


def inspect(d,l,pack,old_d,old_l):
 started=time.perf_counter();old_b,old_kind,old_allowed,old_reach,old_cost,old_hazards=graph(old_d,old_l,pack);b,kind,allowed,reach,cost,hazards=graph(d,l,pack);checks=[]
 def check(label,value):checks.append({'check':label,'passed':bool(value)})
 nd=deepcopy(d);nd.pop('review_contract',None)
 check('every original dossier field exact; only editorial review added',nd==old_d)
 check('same32x18 Kanto route identity/art/shore/bridge metadata',l['dimensions_metatiles']==[32,18]and all(l.get(k)==old_l.get(k)for k in set(l)|set(old_l)if k!='layers'))
 check('every original layer is an exact unchanged prefix',l['layers'][:len(old_l['layers'])]==old_l['layers'])
 changes=[{'metatile':[i%32,i//32],'before':f'{x:02x}','after':f'{y:02x}'}for i,(x,y)in enumerate(zip(old_b,b))if x!=y]
 check('all changed terrain stays in eastern apron meta19..29/9..14',bool(changes)and all(19<=v['metatile'][0]<30 and 9<=v['metatile'][1]<15 for v in changes))
 check('new terrain is only native ordinary canopy edges and stone path',all(v['after']in{'0f','6c','6f','31'}for v in changes))
 check('no incidental Cut collision in any placed quadrant',not any(kind((x,y))=='CUT_TREE'for y in range(36)for x in range(64)))
 check('all lake/outlet water and visible boundary blocks exact',all(b[i]==old_b[i]for i in range(len(b))if any(t in{'WATER','BUOY'}for t in pack['data']['tilesets']['kanto']['collision'][f'{old_b[i]:02x}'])or any(t in{'WATER','BUOY'}for t in pack['data']['tilesets']['kanto']['collision'][f'{b[i]:02x}'])))
 for name,callback in [('original complete dock/bridge/Ron/ledge physical contract',check_piers),('original closed Surf boundary and native rock rows',check_water_boundaries)]:
  try:callback(d,l,pack);check(name,True)
  except AssertionError as e:check(name,False);checks[-1]['reason']=str(e)
 common,_,_=generic_check(d,l,pack,b);check('shared actor/door/grass/room/path/ledge check has no errors',not common['errors'])
 east,west=[tuple(v*2 for v in e['gate_metatile'])for e in d['endpoints']];seen=reach(east);old_seen=old_reach(east)
 check('both endpoints remain physically reachable in both directions',west in seen and east in reach(west))
 check('no new stranded retained Floor/grass cells',not((old_seen&allowed)-seen))
 check('all actual outer walking apertures unchanged',all(kind((x,y))==old_kind((x,y))for y in range(36)for x in range(64)if x in{0,63}or y in{0,35}))
 check('both dock entry lanes and all protected rails/decks unchanged',all(kind((42,y))=='FLOOR'for y in[14,15])and all(kind((42,y))==old_kind((42,y))for y in range(6,24)))
 protected=[(53,11),(53,12),(49,25),(49,26),(57,19),(57,20)]
 check('native door/passive arrival and both original sign reading steps exact',all(kind(p)==old_kind(p)for p in protected)and kind((53,11))=='DOOR'and all(p in seen for p in[(53,12),(49,26),(57,20)]))
 check('two cabin gate core lanes stay open',(54,19)in seen and(55,19)in seen)
 check('all actor/reward native anchors and same-chunk approaches remain',all(kind(tuple(a['tile']))=='FLOOR'and any(q in seen for q in adjacent(tuple(a['tile'])))for key in['trainers','residents','items']for a in d[key]))
 prices=[]
 for label,start,target,blocked in [('west-to-east avoid all trainer sight',west,{east},hazards),('east-to-west avoid all trainer sight',east,{west},hazards)]:
  old=old_cost(start,target,old_hazards);new=cost(start,target,blocked);check(label+' retains exact directional grass/battle cost',old==new);prices.append({'id':label,'before':old,'candidate':new})
 check('Ron remains mandatory westbound and optional eastbound',west not in reach(east,hazards)and east in reach(west,hazards))
 for a in d['items']:
  for label,start in [('west',west),('east',east)]:
   targets=set(adjacent(tuple(a['tile'])));old=old_cost(start,targets);new=cost(start,targets);check(a['id']+' '+label+' resource grass entry price exact',old==new);prices.append({'id':a['id']+'-'+label,'before':old,'candidate':new})
 bands=d['required_grass_crossings'][0]['bounds_core'];x0,y0,x1,y1=bands;band={(x,y)for y in range(y0,y1)for x in range(x0,x1)}
 check('western six-grass gate still exact and unavoidable',all(kind(p)=='TALL_GRASS'for p in band)and west not in reach(east,band))
 check('entire original meadow hop/course/landing/return terrain exact',all(kind((x,y))==old_kind((x,y))for y in range(8,28)for x in range(12,28)))
 # Each new solid canopy piece must join an existing solid bank, never form
 # an isolated prop island. This permits native half-canopy floor quadrants.
 wall={(x,y)for y in range(36)for x in range(64)if kind((x,y))=='WALL'};oldwall={p for p in wall if old_kind(p)=='WALL'};newwall=wall-oldwall
 def component(seed):
  result={seed};todo=[seed]
  while todo:
   for q in adjacent(todo.pop()):
    if q in wall and q not in result:result.add(q);todo.append(q)
  return result
 check('all added wall quadrants attach to original woodland/bank',all(component(p)&oldwall for p in newwall))
 check('northern tongue and opposing bank force the intended S course',all(kind(p)=='WALL'for p in[(48,20),(49,20),(48,23),(49,23),(50,22),(51,22),(52,22),(53,22)])and all(kind(p)=='FLOOR'for p in[(48,21),(49,21),(48,22),(49,22),(50,20),(51,20),(50,21),(51,21),(52,20),(53,20)]))
 check('right cabin stem is two core cells between attached banks',all(kind((x,y))=='FLOOR'for y in[22,23]for x in[54,55])and all(kind((x,y))=='WALL'for y in[22,23]for x in[53,56]))
 check('lower bridge approach is three core cells with attached north bank',all(kind((x,26))=='WALL'and kind((x,27))==kind((x,28))==kind((x,29))=='FLOOR'and kind((x,30))=='WALL'for x in range(38,46)))
 for name,at in VIEWS:check(name+' exact unoccupied native Floor camera',kind(at)=='FLOOR'and at in seen)
 return {'status':'PASS'if all(c['passed']for c in checks)else'FAIL','checks':checks,'terrain_changes':changes,'grass_resource_prices':prices,'shared_route_check':common,'warm_source_check_ms':round((time.perf_counter()-started)*1000,3)},kind,seen


def weak_cameras(d,l,pack):
 _,kind,allowed,_,_,_=graph(d,l,pack);result=[]
 for label,bounds in [('WEAKEST_GLOBAL_CLEAR',(0,0,64,36)),('WEAKEST_EASTERN_APRON',(42,12,60,30))]:
  x0,y0,x1,y1=bounds;sizes={};best=(0,0,0)
  for y in range(y0,y1):
   for x in range(x0,x1):
    size=1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0))if (x,y)in allowed and kind((x,y))=='FLOOR'else 0;sizes[x,y]=size
    if size>best[0]:best=(size,x-size+1,y-size+1)
  size,x,y=best;result.append({'id':label,'tile':[x+size//2,y+size//2],'floor_square_core_bounds':[x,y,x+size,y+size],'clear_floor_square_core_width':size,'meaning':'Largest all-Floor square without an actor body, not a claim that it lacks visible flowers/features.'})
 return result


def render(d,l,pack,out,label):
 b=layout_blocks(l);terrain=SourceArt(pack).render('kanto',32,18,b,group='day');save_render(terrain,out/(label+'-terrain-original.png'));scene=render_stage(terrain,'convert',out,label+'-terrain-rgba','RGBA');pal=npc_palette_pack(pack,'day')
 for key in['trainers','residents','items']:
  for a in d[key]:scene.alpha_composite(oriented_sprite(pal,a.get('sprite','SPRITE_POKE_BALL'),a.get('facing','south')),(a['tile'][0]*16,a['tile'][1]*16))
 save_render(scene,out/(label+'-whole-source-rgba.png'));rgb=render_stage(scene,'convert',out,label+'-whole-rgb','RGB');save_render(rgb,out/(label+'-whole-route.png'));player=oriented_sprite(pal,'SPRITE_CHRIS','south');save_render(player,out/(label+'-player-original-rgba.png'));views=[]
 helper=ROOT/'planning/minnesota/renders/caves/r13-native/review_native.py';spec=importlib.util.spec_from_file_location('r13_priority',helper);native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)
 adapted=dict(pack);adapted['data']=dict(pack['data']);adapted['data']['tilesets']=dict(pack['data']['tilesets']);adapted['data']['tilesets']['cave']=pack['data']['tilesets']['kanto'];adapted['runtime_files']=dict(pack['runtime_files'])
 for oldpath,newpath in [('gfx/tilesets/cave.png','gfx/tilesets/kanto.png'),('data/tilesets/cave_metatiles.bin','data/tilesets/kanto_metatiles.bin')]:adapted['runtime_files'][oldpath]=pack['runtime_files'][newpath]
 module={'blocks':b,'attributes':{'width':32,'height':18}};probes=VIEWS+[(p['id'],tuple(p['tile']))for p in weak_cameras(d,l,pack)];priority=[]
 for name,at in probes:
  full=scene.copy();full.alpha_composite(player,(at[0]*16,at[1]*16));save_render(full,out/(label+'-'+name+'-full-player-rgba.png'))
  cover=native.cave_foreground_pixels(adapted,module,at,player)
  for x,y in cover:full.putpixel((at[0]*16+x,at[1]*16+y),terrain.getpixel((at[0]*16+x,at[1]*16+y))+(255,))
  save_render(full,out/(label+'-'+name+'-foreground-player-rgba.png'));priority.append({'id':name,'masked_native_foreground_pixels':len(cover)})
  x,y=at;im=render_stage(full,'crop',out,label+'-'+name,(x*16-80,y*16-72,x*16+80,y*16+72));save_render(im,out/(label+'-'+name+'-original160-rgba.png'));im=render_stage(im,'convert',out,label+'-'+name+'-rgb','RGB');save_render(im,out/(label+'-'+name+'-native160.png'));views.append((name,im))
 save_render(room_sheet(views,columns=3),out/(label+'-decision-cameras.png'));write(out/(label+'-weakest-floor-cameras.json'),weak_cameras(d,l,pack));write(out/(label+'-foreground-priority.json'),priority)
 return rgb


def main():
 parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-northstar-lake-country.crystalpack');parser.add_argument('--dossier',type=Path,default=ROOT/'planning/minnesota/routes/MN_R15.refinement.candidate.json');parser.add_argument('--layout',type=Path,default=ROOT/'planning/minnesota/routes/MN_R15.refinement.layout.json');parser.add_argument('--output',type=Path,required=True);parser.add_argument('--self-test',action='store_true');args=parser.parse_args();assert sha(args.pack.read_bytes())==PACK_SHA
 oldpaths=[BEFORE/'MN_R15.json',BEFORE/'MN_R15.layout.json'];assert [sha(p.read_bytes())for p in oldpaths]==[D_SHA,L_SHA];old_d,old_l=[json.loads(p.read_bytes())for p in oldpaths];d,l=[json.loads(p.read_bytes())for p in[args.dossier,args.layout]];pack=load_pack(args.pack);assert len(pack['data']['maps'])==538 and pack['data']['maps']['MnR15C00R00']['blocks']==layout_blocks(old_l),'exact current story538/R15 preimage required'
 out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
 for path in [*oldpaths,args.dossier,args.layout,Path(__file__)]: (out/path.name).write_bytes(path.read_bytes())
 started=time.perf_counter();report,kind,seen=inspect(d,l,pack,old_d,old_l);write(out/'source-check.json',report)
 old=render(old_d,old_l,pack,out,'before');new=render(d,l,pack,out,'candidate');paired=Image.new('RGB',(1024,576*2+54),(19,29,40));draw=ImageDraw.Draw(paired);draw.text((12,8),'BEFORE current story538 | native Kanto source composite,32px/metatile',fill='white');paired.paste(old,(0,24));draw.text((12,607),'CANDIDATE attached woodland shore walk | source preview, not native gameplay',fill='white');paired.paste(new,(0,630));save_render(paired,out/'before-after-whole-native32.png');report['complete_positive_render_loop_ms']=round((time.perf_counter()-started)*1000,3)
 if args.self_test:
  negatives=[]
  def reject(name,mutate):
   nd,nl=deepcopy(d),deepcopy(l);mutate(nd,nl);r,_,_=inspect(nd,nl,pack,old_d,old_l);failed=[v for v in r['checks']if not v['passed']];assert failed,name+' accepted';folder=out/'rejected-inputs'/name;folder.mkdir(parents=True);write(folder/'dossier.json',nd);write(folder/'layout.json',nl);write(folder/'rejection.json',failed);negatives.append({'mutation':name,'rejected':True,'failed_checks':[v['check']for v in failed]})
  point=lambda at,b:{'name':'Adversarial physical source mutation','kind':'point','position':at,'block':b}
  reject('reopen-broad-blank-apron',lambda nd,nl:nl['layers'].append({'name':'Clear center bank','kind':'block_rect','bounds':[24,9,30,13],'block':'31'}))
  reject('block-center-course',lambda nd,nl:nl['layers'].append(point([25,10],'0f')))
  reject('incidental-cut-canopy',lambda nd,nl:nl['layers'].append(point([24,9],'06')))
  reject('close-dock-two-lane-gate',lambda nd,nl:nl['layers'].append(point([21,7],'1b')))
  reject('block-original-passive-cabin-return',lambda nd,nl:nl['layers'].append(point([26,6],'0f')))
  reject('change-original-cabin-door',lambda nd,nl:nd['doors'][0].update(tile=[54,11]))
  reject('move-original-pier-watcher',lambda nd,nl:nd['residents'][0].update(tile=[46,17]))
  reject('dry-west-six-grass-bypass',lambda nd,nl:nl['layers'].append({'name':'Dry west grass','kind':'block_rect','bounds':[3,11,6,14],'block':'31'}))
  reject('erase-original-meadow-ledge',lambda nd,nl:nl['layers'].append(point([11,9],'31')))
  reject('open-north-lake-rock',lambda nd,nl:nl['layers'].append(point([15,1],'43')))
  report['negative_mutations']=negatives
 report.update(weak_cameras_before=weak_cameras(old_d,old_l,pack),weak_cameras_candidate=weak_cameras(d,l,pack),pack_sha256=PACK_SHA,pack_identity=pack['identity']['content_hash'],source_sha256={p.name:sha(p.read_bytes())for p in[args.dossier,args.layout]},original_source_sha256={p.name:sha(p.read_bytes())for p in oldpaths},scope='Source/art/directed physical proof only. Exact static native palette/top-left actor frames; no native GPU/controller/earned progression or user approval claim.');write(out/'review.json',report);print(json.dumps({'status':report['status'],'checks':len(report['checks']),'negatives':len(report.get('negative_mutations',[])),'changes':len(report['terrain_changes']),'warm_check_ms':report['warm_source_check_ms'],'render_ms':report['complete_positive_render_loop_ms'],'failed':[v for v in report['checks']if not v['passed']],'shared_errors':report['shared_route_check']['errors'],'prices':report['grass_resource_prices']},indent=2));assert sha(args.pack.read_bytes())==PACK_SHA
 if report['status']!='PASS':raise SystemExit(1)


if __name__=='__main__':main()
