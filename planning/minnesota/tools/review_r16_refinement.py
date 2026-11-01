#!/usr/bin/env python3
"""Bounded R16 candidate source/art review; never edits an active source or pack."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
import time

from PIL import Image,ImageDraw
from render_reference import SourceArt,layout_blocks,load_pack,npc_palette_pack
from render_archive import save_render
from render_stages import render_stage
from review_north_shore import oriented_sprite
from review_league_approach import check as check_route,verify_published
from ledge_paths import shortest_walk,draw_ledge_paths

ROOT=Path(__file__).resolve().parents[3]
BEFORE=ROOT/'planning/minnesota/renders/red-river-prairie/refinement/before-31258'
PACK_SHA='31258a1788a986b158d0679e4f3a26c4dd476964311f707a6e09c9640cad48c6'
OLD_LAYOUT_SHA='6301787fb6db5d929d39092751d4587eff8f4a406af8a32eaaae7789ded970fd'
OLD_DOSSIER_SHA='6750bec82241d5205d34ae06aa704ef18fc0570a308e9151f33cddf0e434aa0a'

def sha(raw):return hashlib.sha256(raw).hexdigest()
def write(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
def adjacent(p):return [(p[0]-1,p[1]),(p[0]+1,p[1]),(p[0],p[1]-1),(p[0],p[1]+1)]

def metric_path(start,end,allowed,price):
 costs={start:0};parents={start:None};queue=deque([start])
 while queue:
  p=queue.popleft()
  for q in adjacent(p):
   if q not in allowed:continue
   step=price(q);cost=costs[p]+step
   if cost<costs.get(q,10**9):costs[q]=cost;parents[q]=p;(queue.append if step else queue.appendleft)(q)
 if end not in costs:return None,None
 path=[];p=end
 while p is not None:path.append(p);p=parents[p]
 return costs[end],path[::-1]


def inspect(d,l,pack,old_d,old_l):
 old_blocks=layout_blocks(old_l);b=layout_blocks(l);w,h=l['dimensions_metatiles'];coll=pack['data']['tilesets'][l['compiled_tileset']]['collision']
 checks=[]
 def check(label,value):checks.append({'check':label,'passed':bool(value)})
 check('same48x24 native prairie identity and exact original art recipes',all(l.get(k)==old_l.get(k)for k in set(l)|set(old_l)if k!='layers')and[w,h]==[48,24])
 check('all original layer strokes remain exact prefix',l['layers'][:len(old_l['layers'])]==old_l['layers'])
 normalized=deepcopy(d);normalized.pop('review_contract',None)
 normalized['one_way_ledges'][0]['bounds_core']=old_d['one_way_ledges'][0]['bounds_core'];normalized['one_way_ledges'][0]['side_barriers']=old_d['one_way_ledges'][0]['side_barriers']
 check('every dossier field exact except explicit bank bounds/side barriers and editorial review',normalized==old_d)
 check('exact new two-core bank bounds and corresponding course-end barriers',d['one_way_ledges'][0]['bounds_core']==[52,16,54,18]and d['one_way_ledges'][0]['side_barriers']==[[51,17],[54,17]])
 changes=[{'metatile':[i%w,i//w],'before':f'{x:02x}','after':f'{y:02x}'}for i,(x,y)in enumerate(zip(old_blocks,b))if x!=y]
 inside=lambda x,y:(16<=x<28 and y==8)or(35<=x<40 and 13<=y<17)
 check('every terrain delta stays in approved bank/farm rectangles',bool(changes)and all(inside(*v['metatile'])for v in changes))
 def kind(x,y):return coll[f'{b[y//2*w+x//2]:02x}'][y%2*2+x%2]
 def oldkind(x,y):return coll[f'{old_blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
 occupied={tuple(a['tile'])for key in ['trainers','residents','items']for a in d[key]}
 ordinary={(x,y)for y in range(h*2)for x in range(w*2)if kind(x,y)in{'FLOOR','TALL_GRASS','LADDER'}and(x,y)not in occupied}
 # Native HOP_DOWN remains land. It is entered from the upper Floor; down input
 # while standing on it jumps two core cells over its WALL complement. It is
 # not an ordinary edge across the course; no WALL occupancy is invented.
 hop={(x,y)for y in range(h*2)for x in range(w*2)if kind(x,y).startswith('HOP_')}
 check('exactly two native hop-face core cells remain',hop=={(52,16),(53,16)}and all(kind(*p)=='HOP_DOWN'for p in hop))
 grass={(x,y)for x,y in ordinary if kind(x,y)=='TALL_GRASS'}
 lane_reports=[]
 for x in [52,53]:
  upper=(x,15);face=(x,16);course=(x,17);landing=(x,18)
  check(f'lane{x} real upper/face/course/landing native quadrants',[kind(*p)for p in [upper,face,course,landing]]==['FLOOR','HOP_DOWN','WALL','FLOOR']and not occupied.intersection([upper,face,course,landing]))
  check(f'lane{x} actual two-core jump landing and blocked reverse',face[1]+2==landing[1]and course not in ordinary and landing in ordinary)
  path=shortest_walk(landing,upper,ordinary)
  gate={(gx,gy)for gy in range(10,12)for gx in range(30,32)}
  check(f'lane{x} ordinary uphill return uses original west working gate',path is not None and len(path)-1>=25 and shortest_walk(landing,upper,ordinary-gate)is None)
  local={(px,py)for px,py in ordinary|hop if 50<=px<=55 and 14<=py<=19}
  check(f'lane{x} no ordinary course-side/reverse bypass',shortest_walk(landing,upper,local)is None)
  cost,costpath=metric_path(landing,(56,30),ordinary,lambda p:int(p in grass))
  check(f'lane{x} at least six actual grass steps after drop',cost is not None and cost>=6)
  check(f'lane{x} no Floor-only road escape',shortest_walk(landing,(56,30),ordinary-grass)is None)
  lane_reports.append({'id':f'HAYFIELD_BANK_{x}','upper':upper,'face':face,'course':course,'landing':landing,'return_steps':None if path is None else len(path)-1,'ordinary_return_path':path,'minimum_grass_steps_to56_30':cost,'minimum_grass_path':costpath})
 check('both fitted course ends are genuinely WALL',all(kind(*p)=='WALL'for p in [(51,17),(54,17)]))
 for x in [70,74]:check(f'horizontal working lane{x} only two dry core rows',kind(x,28)=='WALL'and kind(x,29)==kind(x,30)=='FLOOR'and kind(x,31)=='WALL')
 for y in [27,29]:check(f'vertical working lane y{y} only two dry core columns',kind(75,y)=='WALL'and kind(76,y)==kind(77,y)=='FLOOR'and kind(78,y)in{'WALL','HEADBUTT_TREE'})
 check('turn south shoulder closes broad dry apron',all(kind(x,32)=='WALL'for x in range(76,80)))
 check('original bridges/rails/decks metadata exact',l['crossings']==old_l['crossings']and all(b[y*w+x]==old_blocks[y*w+x]for crossing in old_l['crossings']for key in ['north_rail_blocks','south_rail_blocks','deck_blocks']for x,y in crossing[key]))
 water={'WATER','WATERFALL','WHIRLPOOL','BUOY'}
 check('all actual river/ditch water and visible Surf closures exact',all(kind(x,y)==oldkind(x,y)for y in range(h*2)for x in range(w*2)if kind(x,y)in water or oldkind(x,y)in water))
 check('no added conifer/canopy/boulder/Cut blocks',all(v['after']in{'40','41','42','46','49'}for v in changes)and not any(kind(x,y)=='CUT_TREE'for y in range(h*2)for x in range(w*2)))
 check('golden lawn83/grass84 pixels and collision remain original native definitions',all(l['derived_blocks'][i]==old_l['derived_blocks'][i]for i in range(len(old_l['derived_blocks'])))and coll['83']==['FLOOR']*4 and coll['84']==['TALL_GRASS']*4)
 common,_,seen=check_route(d,l,pack,b)
 check('shared full actor-present route/anchors/grass/ledge-return checker has no errors',not common['errors'])
 x0,y0,x1,y1=d['required_grass_crossings'][0]['bounds_core'];band={(x,y)for y in range(y0,y1)for x in range(x0,x1)}
 start,end=[tuple(v*2 for v in e['gate_metatile'])for e in d['endpoints']]
 cost,_=metric_path(start,end,ordinary,lambda p:int(p in band))
 check('cottonwood neck retains all native grass and exactly four required steps',band<=grass and cost==4 and shortest_walk(start,end,ordinary-band)is None)
 choice=d['battle_or_grass_choices'][0];trainer=next(t for t in d['trainers']if t['id']==choice['trainer_id']);dx,dy={'north':(0,-1),'south':(0,1),'west':(-1,0),'east':(1,0)}[trainer['facing']]
 sight={(trainer['tile'][0]+dx*n,trainer['tile'][1]+dy*n)for n in range(1,trainer['sight_range']+1)}
 x0,y0,x1,y1=choice['bounds_core'];band={(x,y)for y in range(y0,y1)for x in range(x0,x1)}
 cost,_=metric_path(start,end,ordinary-sight,lambda p:int(p in band))
 check('original bridge battle-versus-two-grass choice still priced',cost==2 and shortest_walk(start,end,ordinary-sight-band)is None and shortest_walk(start,end,ordinary-band)is not None)
 check('all original actors/rewards/signs/endpoints/viewpoints preserved and approachable',all(d[k]==old_d[k]for k in ['trainers','residents','items','signs','endpoints','viewpoints'])and not any('no clear same-chunk approach'in e for e in common['errors']))
 outer={(x,0)for x in range(w*2)}|{(x,h*2-1)for x in range(w*2)}|{(0,y)for y in range(h*2)}|{(w*2-1,y)for y in range(h*2)}
 check('entire unconnected Surf outer boundary remains visibly closed',not any(kind(*p)in{'WATER','WATERFALL','WHIRLPOOL'}for p in outer))
 return {'status':'PASS'if all(c['passed']for c in checks)else'FAIL','checks':checks,'terrain_changes':changes,'lane_probes':lane_reports,'shared_route_check':common,'cottonwood_grass_steps':4,'battle_grass_steps':2},kind


def render(d,l,pack,out,label):
 b=layout_blocks(l);w,h=l['dimensions_metatiles'];art=SourceArt(pack);terrain=art.render(l['compiled_tileset'],w,h,b,group='day');save_render(terrain,out/(label+'-terrain-native.png'))
 scene=render_stage(terrain,'convert',out,label+'-terrain-rgba','RGBA');pal=npc_palette_pack(pack,'day')
 for key in ['trainers','residents','items']:
  for a in d[key]:
   frame=oriented_sprite(pal,a.get('sprite','SPRITE_POKE_BALL'),a.get('facing','south'))
   # Native OAM begins at tile top-left. Historical shared previews used y-4.
   scene.alpha_composite(frame,(a['tile'][0]*16,a['tile'][1]*16))
 save_render(scene,out/(label+'-whole-rgba.png'));rgb=render_stage(scene,'convert',out,label+'-whole-rgb','RGB');save_render(rgb,out/(label+'-whole-route.png'))
 views=[('widest-farm-lane',(75,30)),('hayfield-bank',(53,15)),('drop-left-upper',(52,15)),('drop-left-lower',(52,18)),('drop-right-lower',(53,18)),('farm-north-approach',(77,27)),('farm-south-shoulder',(77,33))]
 contact=Image.new('RGB',(3*344,((len(views)+2)//3)*332),(19,29,40));draw=ImageDraw.Draw(contact)
 player=oriented_sprite(pal,'SPRITE_CHRIS','south');save_render(player,out/(label+'-chris-south-original-rgba.png'))
 for i,(name,at)in enumerate(views):
  framed=scene.copy();framed.alpha_composite(player,(at[0]*16,at[1]*16));save_render(framed,out/(label+'-'+name+'-full-player-rgba.png'))
  x,y=at;crop=render_stage(framed,'crop',out,label+'-'+name,(x*16-80,y*16-72,x*16+80,y*16+72));save_render(crop,out/(label+'-'+name+'-native160-rgba.png'))
  crop=render_stage(crop,'convert',out,label+'-'+name+'-rgb','RGB');save_render(crop,out/(label+'-'+name+'-native160.png'))
  zoom=render_stage(crop,'resize',out,label+'-'+name+'-2x',(320,288),Image.Resampling.NEAREST);save_render(zoom,out/(label+'-'+name+'-2x.png'))
  cx=i%3*344+12;cy=i//3*332+28;draw.text((cx,cy-18),name+' '+str(at),fill='white');contact.paste(zoom,(cx,cy))
 save_render(contact,out/(label+'-decision-cameras.png'))
 return rgb


def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-northstar-repel-review.crystalpack');parser.add_argument('--dossier',type=Path,default=ROOT/'planning/minnesota/routes/MN_R16.refinement.candidate.json');parser.add_argument('--layout',type=Path,default=ROOT/'planning/minnesota/routes/MN_R16.refinement.layout.json');parser.add_argument('--output',type=Path,required=True);parser.add_argument('--self-test',action='store_true');args=parser.parse_args()
 assert sha(args.pack.read_bytes())==PACK_SHA
 old_paths=[BEFORE/'MN_R16.json',BEFORE/'MN_R16.layout.json'];assert[sha(p.read_bytes())for p in old_paths]==[OLD_DOSSIER_SHA,OLD_LAYOUT_SHA]
 old_d,old_l=[json.loads(p.read_bytes())for p in old_paths];d,l=[json.loads(p.read_bytes())for p in [args.dossier,args.layout]];pack=load_pack(args.pack)
 verify_published(old_d,old_l,pack)
 out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
 for p in [*old_paths,args.dossier,args.layout,Path(__file__)]: (out/p.name).write_bytes(p.read_bytes())
 start=time.perf_counter();report,kind=inspect(d,l,pack,old_d,old_l);report['warm_source_check_ms']=round((time.perf_counter()-start)*1000,3);write(out/'source-check.json',report)
 # Render even a rejected candidate so visual history remains reusable.
 original=render(old_d,old_l,pack,out,'before');candidate=render(d,l,pack,out,'candidate')
 paired=Image.new('RGB',(original.width,candidate.height*2+54),(19,29,40));draw=ImageDraw.Draw(paired);draw.text((12,8),'BEFORE: actual31258 source terrain/actors | SOURCE COMPOSITES, day, native32px/metatile',fill='white');paired.paste(original,(0,24));draw.text((12,original.height+31),'CANDIDATE: bounded two-core hayfield hop + fitted farm work lane | not published or controller-proven',fill='white');paired.paste(candidate,(0,original.height+54));save_render(paired,out/'before-after-whole-native32.png')
 report.update(warm_complete_render_loop_ms=round((time.perf_counter()-start)*1000,3),pack_sha256=PACK_SHA,pack_internal_identity=pack['identity']['content_hash'],baseline_source_sha256={p.name:sha(p.read_bytes())for p in old_paths},candidate_source_sha256={p.name:sha(p.read_bytes())for p in [args.dossier,args.layout]},render_scope='Whole explicit native source-art/OAM composite; centered day160x144 player previews, not actual GPU/controller proof.')
 if args.self_test:
  negatives=[]
  def reject(name,mutate):
   nd,nl=deepcopy(d),deepcopy(l);mutate(nd,nl)
   result,_=inspect(nd,nl,pack,old_d,old_l);failed=[v['check']for v in result['checks']if not v['passed']]
   assert failed,name+' accepted'
   folder=out/'rejected-inputs'/name;folder.mkdir(parents=True);write(folder/'dossier.json',nd);write(folder/'layout.json',nl);write(folder/'rejection.json',failed);negatives.append({'mutation':name,'rejected':True,'failed_checks':failed})
  point=lambda x,y,b:{'name':'Adversarial physical mutation','kind':'point','position':[x,y],'block':b}
  reject('old-wide-hop-face',lambda nd,nl:nl['layers'].append({'name':'Restore old lip','kind':'block_rect','bounds':[16,8,28,9],'block':'57'}))
  reject('missing-left-hop',lambda nd,nl:nl['layers'].append(point(26,8,'49')))
  reject('open-east-course-end',lambda nd,nl:nl['layers'].append(point(27,8,'75')))
  reject('dry-hayfield-grass-bypass',lambda nd,nl:nl['layers'].append({'name':'Dry all former swale grass','kind':'block_remap','bounds':[18,9,30,13],'from':'84','to':'83'}))
  reject('empty-wide-farm-apron',lambda nd,nl:nl['layers'].append({'name':'Clear entire bend','kind':'block_rect','bounds':[35,13,40,17],'block':'01'}))
  reject('changed-trainer-anchor',lambda nd,nl:nd['trainers'][0]['tile'].__setitem__(0,0))
  reject('changed-golden-derived-palette',lambda nd,nl:nl['derived_blocks'][-1].update(palette_override=0))
  reject('lost-drain-bridge-rail',lambda nd,nl:nl['layers'].append(point(30,14,'01')))
  reject('dry-cottonwood-bypass',lambda nd,nl:nl['layers'].append({'name':'Dry neck','kind':'block_rect','bounds':[41,9,43,14],'block':'83'}))
  report['negative_mutations']=negatives
 write(out/'review.json',report);print(json.dumps({'status':report['status'],'checks':len(report['checks']),'negative_mutations':len(report.get('negative_mutations',[])),'warm_source_check_ms':report['warm_source_check_ms'],'lane_probes':[{k:v for k,v in p.items()if k not in ['ordinary_return_path','minimum_grass_path']}for p in report['lane_probes']],'errors':[v for v in report['checks']if not v['passed']],'shared_errors':report['shared_route_check']['errors'],'output':str(out)},indent=2))
 assert sha(args.pack.read_bytes())==PACK_SHA
 if report['status']!='PASS':raise SystemExit(1)


if __name__=='__main__':main()
