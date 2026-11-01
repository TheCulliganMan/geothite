#!/usr/bin/env python3
"""Fast whole Brainerd native-art SOURCE review; never publish native content."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time
from PIL import Image, ImageDraw
from render_archive import save_render
from render_reference import SourceArt,layout_blocks,load_pack
from render_stages import render_stage
from ledge_paths import inspect_ledges,draw_ledge_paths
from review_north_shore import oriented_sprite
ROOT=Path(__file__).resolve().parents[3]
DESIGN=ROOT/'planning/minnesota/towns/MnBrainerd.json'
LAYOUT=ROOT/'planning/minnesota/towns/MnBrainerd.layout.json'
OUTPUT=ROOT/'planning/minnesota/renders/brainerd'
CANONICAL_BYTES=None
def adjacent(p):x,y=p;return[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]
def flood(start,cells,doors=frozenset()):
 seen={start};todo=deque([start])
 while todo:
  for q in adjacent(todo.popleft()):
   if q in cells and q not in seen:
    seen.add(q)
    if q not in doors:todo.append(q)
 return seen
def sampler(pack,ts,w,blocks):
 c=pack['data']['tilesets'][ts]['collision']
 def kind(p):x,y=p;return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
 return kind
def actor_check(pack,a):
 assert f'gfx/sprites/{a["sprite"].removeprefix("SPRITE_").lower()}.png'in pack['runtime_files'],'missing sprite '+a['sprite']
 assert all(len(t)>0 and (a.get('canonical_actor')or len(t)<=18)for t in a['lines']),a['id']+' text width'
 if 'battle'in a:
  assert a['trainer_class']in pack['data']['trainer_class_names'],'unknown native trainer class '+a['trainer_class']
  assert 1<=len(a['party'])<=6 and all(v['species']in pack['data']['pokemon']and 1<=v['level']<=100 for v in a['party']),'illegal native party'
 if 'reward'in a and isinstance(a['reward'],dict):assert a['reward']['item']in pack['data']['items']and a['reward']['quantity']>0,'unknown native item'
def canonical_gym(reference):
 assert reference['canonical_source']=='planning/minnesota/gyms/interiors.json'and reference['canonical_id']=='MnBrainerdGym','canonical gym authority drift'
 assert not any(key in reference for key in ['rows','art','residents','party','stations','gates','reward','tileset']),'city must not own an alternative functional Gym6'
 payload=CANONICAL_BYTES if CANONICAL_BYTES is not None else (ROOT/reference['canonical_source']).read_bytes()
 gym=next(g for g in json.loads(payload)['gyms']if g['id']==reference['canonical_id'])
 digest=hashlib.sha256(json.dumps(gym,sort_keys=True,separators=(',',':')).encode()).hexdigest()
 assert digest==reference['canonical_gym_sha256'],'canonical gym changed; inspect and refresh city reference'
 return gym

def preview_room(reference,pack):
 """Resolve canonical source art/collision in memory; never export a gym or pack."""
 if reference['id']!='MnBrainerdGym':return reference
 gym=canonical_gym(reference);room=deepcopy(reference)
 needed=set(''.join(gym['rows']))|{gate['open_symbol']for gate in gym['gates']}|{'#'}
 first=len(pack['runtime_files']['data/tilesets/elite_four_room_metatiles.bin'])//16
 symbols={s:first+i for i,s in enumerate(sorted(needed))}
 room.update(template='VioletGym',tileset='elite_four_room',border_block=symbols['#'],rows=[' '.join(f'{symbols[s]:02x}'for s in row)for row in gym['rows']],derived_blocks=[dict(id=f'{symbols[s]:02x}',source_tileset=gym['art'][s][0],source_block=gym['art'][s][1])for s in sorted(needed)],residents=[],signs=[],canonical_design=gym,canonical_symbols=symbols,canonical_art_sources={symbols[s]:gym['art'][s]for s in needed})
 def actor(id,sprite,tile,lines,**fields):return dict(id=id,sprite=sprite,tile=tile,facing='south',lines=lines,canonical_actor=True,**fields)
 room['residents']=[actor(gym['id']+'Guide','SPRITE_GYM_GUIDE',gym['guide_at'],gym['guide'])]
 room['residents'] += [actor(t['id'],t['sprite'],t['at'],["Let's practice",'before the leader!'],battle=t['id'],trainer_class=t['trainer_class'],party=t['party'],canonical_trainer=t,win_flag='EVENT_'+t['id']+'_DEFEATED')for t in gym['trainers']]
 room['residents'] += [actor(gym['id']+s['id']+'Station',s['sprite'],s['at'],s['prompt'],canonical_station=s)for s in gym['stations']]
 room['residents'].append(actor(gym['id']+'Leader',gym['leader_sprite'],gym['leader_at'],['I am '+gym['leader']+'.','Show me your team!'],battle='MN_GYM_6',trainer_class=gym['trainer_class'],party=gym['party'],requires=gym['requires'],reward=gym['reward'],phase='court_open'))
 # Derive actual legal station camera approaches from the canonical collision;
 # this chooses views, never changes terrain.
 w,h=room['dimensions_metatiles'];bs=[int(x,16)for row in room['rows']for x in row.split()];kind=room_sampler(pack,room,w,bs);bodies={tuple(a['tile'])for a in room['residents']};walk={(x,y)for y in range(h*2)for x in range(w*2)if kind((x,y))in{'FLOOR','LADDER','WARP_CARPET_DOWN'}};seen=flood(tuple(gym['arrival_core']),walk-bodies)
 room['viewpoints']=[dict(id='arrival-closed',tile=gym['arrival_core'])]+[dict(id=station['id'].lower()+'-closed',tile=list(next(p for p in adjacent(tuple(station['at']))if p in seen)))for station in gym['stations']]+[dict(id='leader-court-open',tile=[gym['leader_at'][0]-1,gym['leader_at'][1]],phase='court_open')]
 # Scalar reward on this preview actor is metadata from the canonical object,
 # not a second city-owned giveitem definition.
 return room

def room_sampler(pack,room,width,blocks):
 if 'canonical_art_sources'not in room:return sampler(pack,room['tileset'],width,blocks)
 def kind(p):
  x,y=p;source,block=room['canonical_art_sources'][blocks[y//2*width+x//2]]
  return pack['data']['tilesets'][source]['collision'][block][y%2*2+x%2]
 return kind

def canonical_phases(room,pack):
 """Check source-defined water phases; event sets here are static fixtures."""
 gym=room['canonical_design'];w,h=room['dimensions_metatiles'];original=[int(x,16)for row in room['rows']for x in row.split()];blocks=list(original);actors={tuple(a['tile'])for a in room['residents']};completed=set();reports=[];ever=set()
 def reach():
  kind=room_sampler(pack,room,w,blocks);walk={(x,y)for y in range(h*2)for x in range(w*2)if kind((x,y))in{'FLOOR','LADDER','WARP_CARPET_DOWN'}}
  return flood(tuple(gym['arrival_core']),walk-actors)
 def record(step):
  seen=reach();ever.update(seen);reports.append(dict(step=step,events=sorted(completed),open_gates=[g['event']for g in gym['gates']if g['event']in completed],reachable_walk=len(seen)))
 closed=reach();leader=tuple(gym['leader_at']);assert not set(adjacent(leader))&closed,'canonical closed court has a walking bypass'
 record('ARRIVAL_CLOSED')
 for step in gym['walkthrough']:
  trainer=next((t for t in gym['trainers']if t['id']==step),None)
  routing=next(((r,m)for r in gym.get('routers',[])for m in r['modes']if m['id']==step),None)
  target=trainer or (dict(at=routing[0]['at'],requires=routing[1]['requires'])if routing else next(s for s in gym['stations']if s['id']==step))
  assert set(target.get('requires',[]))<=completed,'canonical walkthrough skips prerequisites'
  assert set(adjacent(tuple(target['at'])))&reach(),'canonical walkthrough step inaccessible: '+step
  if routing:
   router,mode=routing
   completed.difference_update(other['event']for other in router['modes'])
   completed.add(mode['event'])
  else:completed.add('EVENT_'+trainer['id']+'_DEFEATED'if trainer else target['event'])
  for gate in gym['gates']:
   gx,gy=gate['at'];index=gy*w+gx
   if set(gate['requires'])<=completed:blocks[index]=room['canonical_symbols'][gate['open_symbol']];completed.add(gate['event'])
   else:blocks[index]=original[index];completed.discard(gate['event'])
  record(step)
 opened=reach();assert set(adjacent(leader))&opened,'canonical open court blocks leader'
 for actor in room['residents']:assert set(adjacent(tuple(actor['tile'])))&ever,'canonical actor inaccessible in every phase'
 court='EVENT_MN_GYM_6_COURT_OPEN'
 assert all(court not in phase['events']for phase in reports[:-1])and court in reports[-1]['events'],'Gym6 court must open only after both battles and the left crossing'
 return reports,closed,opened,blocks

def validate(d,l,pack):
 assert d['id']==l['id']=='MnBrainerd'and d['dimensions_metatiles']==l['dimensions_metatiles']==[32,24]
 assert l['compiled_tileset']=='johto'and not l.get('derived_blocks')and not l['source_pixels_modified']
 b=layout_blocks(l);k=sampler(pack,'johto',32,b);cells={(x,y)for y in range(48)for x in range(64)}
 actors={tuple(a['tile'])for a in d['residents']};doors={tuple(a['tile'])for a in d['doors']}
 assert len(actors)==len(d['residents'])and not actors&doors
 walk={p for p in cells if k(p)in{'FLOOR','TALL_GRASS','DOOR'}};water={p for p in cells if k(p)=='WATER'}
 seen=flood((32,44),walk-actors,doors);surf=flood((32,44),(walk|water)-actors,doors)
 for a in d['residents']:
  actor_check(pack,a);assert k(tuple(a['tile']))=='FLOOR',a['id']+' actor off FLOOR '+k(tuple(a['tile']))
  assert set(adjacent(tuple(a['tile'])))&seen,a['id']+' inaccessible'
 for s in d['signs']:
  assert k(tuple(s['tile']))=='WALL',s['id']+' sign not WALL'
  assert set(adjacent(tuple(s['tile'])))&seen,s['id']+' inaccessible'
  assert all(0<len(t)<=18 for t in s['lines'])
 assert doors=={p for p in cells if k(p)=='DOOR'},('unbound facade doors',doors^{p for p in cells if k(p)=='DOOR'})
 for door in d['doors']:
  p=tuple(door['tile']);q=tuple(door['return_tile']);assert p in seen and q in seen,door['id']+' door disconnected'
  assert k(p)=='DOOR'and k(q)=='FLOOR'and q==(p[0],p[1]+1),door['id']+' unsafe door return'
  assert door['target_warp']==3,'room arrival must be passive3'
 assert [v for door in d['doors']for v in[door['warp_index'],door['return_warp_index']]]==list(range(1,11))
 expected=[('MN_R06','S',[16,22],[16,20]),('MN_R07','N',[16,1],[16,3]),('MN_R15','W',[1,12],[3,12]),('MN_R18','E',[30,12],[28,12])]
 assert len(d['ports'])==4,'four exact regional ports'
 for p,e in zip(d['ports'],expected):
  assert(p['route'],p['side'],p['gate_metatile'],p['landing_metatile'])==e,'reserved port changed'
  assert all(tuple(v*2 for v in p[key])in seen for key in['gate_metatile','landing_metatile']),'port disconnected'
 outer={p for p in cells if p[0]in{0,63}or p[1]in{0,47}}
 assert seen&outer=={(0,24),(0,25),(63,24),(63,25),(32,0),(33,0),(32,47),(33,47)},('unexpected walk edge',sorted(seen&outer))
 assert not water&outer,'unconnected outer lake'
 assert not any(k(p)=='CUT_TREE'for p in cells),'incidental Cut gate'
 for v in d['viewpoints']:assert tuple(v['tile'])in seen and k(tuple(v['tile']))=='FLOOR',v['id']+' camera not accessible Floor'
 dock=d['public_dock'];assert tuple(dock['launch_tile'])in seen and k(tuple(dock['water_target']))=='WATER','lake launch not real'
 for x,y in dock['west_rail_metatiles']:assert k((2*x,2*y))==k((2*x,2*y+1))=='WALL','west pier rail gap'
 for x,y in dock['east_rail_metatiles']:assert k((2*x+1,2*y))==k((2*x+1,2*y+1))=='WALL','east pier rail gap'
 for x,y in dock['north_cap_metatiles']:assert k((2*x,2*y))==k((2*x+1,2*y))=='WALL','pier head open'
 t=d['gym_terrace'];rx,ry=t['ramp_metatile'];assert b[ry*32+rx]==0x74,'native gym ramp erased'
 altered=list(b);altered[ry*32+rx]=0x72;ak=sampler(pack,'johto',32,altered);aw={p for p in cells if ak(p)in{'FLOOR','DOOR'}}
 assert tuple(d['doors'][2]['tile'])not in flood((32,44),aw-actors,doors),'gym ramp has a dry bypass'
 lx,ly=t['ledge_metatile'];assert b[ly*32+lx]==0x4d and k(tuple(t['ledge_collision']))=='HOP_RIGHT'and k(tuple(t['ledge_approach']))=='FLOOR'and k(tuple(t['ledge_landing']))=='FLOOR','false gym one-way ledge'
 assert tuple(t['ledge_landing'])in seen,'gym ledge landing is isolated'
 assert d['gym_contract']['requires']==['BADGE_5']and d['gym_contract']['win_only']==['BADGE_6','EVENT_MN_GYM_6_DEFEATED']and d['gym_contract']['native_badge_index']==5
 assert d['story_contract']['ranger_anchor_core']==[25,19]and next(a for a in d['residents']if a['id']=='MN_BRAINERD_RANGER')['tile']==[25,19]
 ledges=inspect_ledges(d,lambda x,y:k((x,y)),seen)
 for ledge in d['one_way_ledges']:
  assert k(tuple(ledge['hop_from']))=='HOP_RIGHT'and k(tuple(ledge['reverse_blocked']))=='WALL'and all(k(tuple(p))=='WALL'for p in ledge['side_barriers']),'ledge has a reverse or side bypass'
 return dict(ledges=ledges,source_only=True,reachable_walk=len(seen),reachable_water=len(surf&water),doors=len(doors),native_metatile_varieties=len(set(b)),closed_outer_water=True,native_publication=False),k,seen,b

def validate_rooms(d,pack):
 assert[(r['id'],r['dimensions_metatiles'])for r in d['interiors']]==[('MnBrainerdPokecenter1F',[10,8]),('MnBrainerdMart',[6,4]),('MnBrainerdGym',d['interiors'][2]['dimensions_metatiles']),('MnBrainerdRangerStation',[8,6]),('MnBrainerdOutfitter',[6,4])],'complete exact room roster'
 assert{door['target']for door in d['doors']}=={r['id']for r in d['interiors']},'unbound room target'
 reports=[]
 for reference in d['interiors']:
  r=preview_room(reference,pack)
  assert r['template']in pack['data']['maps'],'missing source behavior template '+r['template']
  assert not r.get('receiver_contract'),'unrelated boundary story hook'
  if'canonical_art_sources'in r:
   source,block=r['canonical_art_sources'][r['border_block']]
   assert set(pack['data']['tilesets'][source]['collision'][block])=={'WALL'},'canonical border is not native WALL'
  else:assert set(pack['data']['tilesets'][r['tileset']]['collision'][f"{r['border_block']:02x}"])=={'WALL'},r['id']+' room border must be native WALL'
  w,h=r['dimensions_metatiles'];rows=[x.split()for x in r['rows']];assert len(rows)==h and all(len(x)==w for x in rows),r['id']+' rows'
  b=[int(x,16)for row in rows for x in row];k=room_sampler(pack,r,w,b);cells={(x,y)for y in range(h*2)for x in range(w*2)}
  actors={tuple(a['tile'])for a in r['residents']};walk={p for p in cells if k(p)in{'FLOOR','LADDER','WARP_CARPET_DOWN'}};arrival=tuple(r['passive_arrival']['tile'])
  assert len(actors)==len(r['residents']),'overlapping canonical actors'
  assert r['passive_arrival']['index']==3 and k(arrival)=='FLOOR'and arrival not in actors,'unsafe passive room arrival'
  seen=flood(arrival,walk-actors);open_seen=seen;phases=[]
  if'canonical_design'in r:
   phases,closed,open_seen,phase_blocks=canonical_phases(r,pack);assert seen==closed
   g=r['canonical_design'];contract=d['gym_contract']
   assert not any(key in contract for key in ['party','rows','art','residents','stations','gates','reward']),'city summary must not redefine canonical gym'
   assert contract['canonical_source']==reference['canonical_source']and contract['canonical_id']==g['id']and contract['canonical_gym_sha256']==reference['canonical_gym_sha256'],'Gym6 summary canonical authority drift'
   assert contract['requires']==g['requires']and g['badge_flag']=='ENGINE_MINERALBADGE'and contract['native_badge_index']==g['slot']-1,'Gym6 canonical badge drift'
  carpets={tuple(e['tile'])for e in r['exits']};assert carpets=={p for p in cells if k(p)=='WARP_CARPET_DOWN'}and carpets<=seen,r['id']+' carpet mismatch'
  door=next(x for x in d['doors']if x['target']==r['id']);assert all(e['target']=='MnBrainerd'and e['target_warp']==door['return_warp_index']for e in r['exits']),'wrong room return'
  if r['id']=='MnBrainerdGym':assert[(e['index'],e['tile'])for e in r['exits']]==list(enumerate(r['canonical_design']['exit_tiles_core'],1))and r['passive_arrival']['tile']==r['canonical_design']['arrival_core']and door['target_warp']==3,'native gym room warp1/2/3 mismatch'
  for a in r['residents']:
   actor_check(pack,a);q=tuple(a['tile']);assert k(q)=='FLOOR',a['id']+' not Floor'
   if'counter_tile'in a:assert k(tuple(a['counter_tile']))=='COUNTER'and tuple(a['approach'])in seen,'counter service inaccessible'
   else:assert a.get('canonical_actor')or set(adjacent(q))&(open_seen if a.get('phase')=='court_open'else seen),a['id']+' inaccessible'
  for sign in r.get('signs',[]):assert k(tuple(sign['tile']))in{'WALL','WINDOW','BOOKSHELF'}and tuple(sign['approach'])in seen,'room sign inaccessible'
  if'pc'in r:assert k(tuple(r['pc']['tile']))=='PC'and tuple(r['pc']['approach'])in seen,'PC inaccessible'
  assert not any(k(p)in({'PIT','STAIRCASE','DOOR'}if r.get('canonical_design',{}).get('enclosed_pool')else {'PIT','LADDER','STAIRCASE','DOOR'})for p in cells),'unbound interior trigger'
  outer={p for p in cells if p[0]in{0,w*2-1}or p[1]in{0,h*2-1}}
  if r.get('canonical_design',{}).get('enclosed_pool'):assert all(k(p)in {'FLOOR','WALL','WARP_CARPET_DOWN'}for p in outer),'pool water must not reach an outer edge'
  else:assert seen&outer==carpets and open_seen&outer==carpets,(r['id'],'unbound room edge')
  for v in r['viewpoints']:
   vk=room_sampler(pack,r,w,phase_blocks)if v.get('phase')=='court_open'and 'canonical_design'in r else k
   assert tuple(v['tile'])in(open_seen if v.get('phase')=='court_open'else seen)and vk(tuple(v['tile']))=='FLOOR','room camera not Floor'
  reports.append(dict(id=r['id'],public_tiles=len(seen),open_phase_public_tiles=len(open_seen),passive_arrival=list(arrival),water_tiles=sum(k(p)=='WATER'for p in cells),canonical_source=r.get('canonical_source'),canonical_gym_sha256=r.get('canonical_gym_sha256'),puzzle_source_phases=phases,canonical_text_lines_over18=[dict(actor=a['id'],line=t,characters=len(t))for a in r['residents']if a.get('canonical_actor')for t in a['lines']if len(t)>18]))
 return reports

def negatives(d,l,pack):
 results=[]
 def reject(name,mutate):
  dd,ll=deepcopy(d),deepcopy(l);mutate(dd,ll)
  try:validate(dd,ll,pack);validate_rooms(dd,pack)
  except(AssertionError,KeyError,IndexError):results.append(name);return
  raise AssertionError('negative accepted: '+name)
 def block(l,p,b):l['layers'].append(dict(name='NEGATIVE ONLY',kind='point',position=p,block=b))
 reject('wrong facade quadrant',lambda d,l:d['doors'][0].update(tile=[8,21]))
 reject('active carpet used as room arrival',lambda d,l:d['interiors'][0]['passive_arrival'].update(tile=[10,15]))
 reject('room returned onto active facade',lambda d,l:d['interiors'][1]['exits'][0].update(target_warp=3))
 reject('reserved route port drift',lambda d,l:d['ports'][0].update(gate_metatile=[15,22]))
 reject('unclosed Surf perimeter',lambda d,l:block(l,[31,5],'35'))
 reject('pier west rail removed',lambda d,l:block(l,[23,8],'01'))
 reject('pier head removed',lambda d,l:block(l,[23,6],'01'))
 reject('gym ramp removed',lambda d,l:block(l,[10,20],'72'))
 reject('false one-way ledge',lambda d,l:block(l,[12,19],'01'))
 reject('incidental Cut tree',lambda d,l:block(l,[18,13],'5b'))
 reject('badge6 granted before badge5',lambda d,l:d['gym_contract'].update(requires=[]))
 reject('unrelated boundary receiver hook',lambda d,l:d['interiors'][3].update(receiver_contract={'sets':['EVENT_MN_SURVEY_RESTORED']}))
 reject('room border is walkable',lambda d,l:d['interiors'][4].update(border_block=0))
 reject('alternative city-owned Gym6 rows',lambda d,l:d['interiors'][2].update(rows=['............']*12))
 reject('alternative city-owned Gym6 actors',lambda d,l:d['interiors'][2].update(residents=[]))
 reject('alternative Gym6 summary party',lambda d,l:d['gym_contract'].update(party=[{'species':'MAGIKARP','level':1}]))
 reject('stale canonical Gym6 signature',lambda d,l:d['interiors'][2].update(canonical_gym_sha256='0'*64))
 reject('unknown native HM item',lambda d,l:next(a for a in d['residents']if a['id']=='MN_BRAINERD_RANGER')['reward'].update(item='HM03_FAKE'))
 reject('visible cabin has no room',lambda d,l:d['interiors'].pop())
 reject('ranger story anchor moved',lambda d,l:d['story_contract'].update(ranger_anchor_core=[26,19]))
 return results

def render(d,l,pack):
 started=time.perf_counter();report,k,seen,b=validate(d,l,pack);report['rooms']=validate_rooms(d,pack);report['negative_cases_rejected']=negatives(d,l,pack)
 digest=hashlib.sha256(json.dumps([d,l],sort_keys=True).encode()).hexdigest()[:12];out=OUTPUT/digest;out.mkdir(parents=True,exist_ok=True)
 art=SourceArt(pack);terrain=art.render('johto',32,24,b);save_render(terrain,out/'terrain-original.png');scene=render_stage(terrain,'convert',out,'town-rgba','RGBA')
 for a in d['residents']:scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
 save_render(scene,out/'whole-town-rgba.png');rgb=render_stage(scene,'convert',out,'town-rgb','RGB');save_render(rgb,out/'whole-town.png');save_render(rgb,OUTPUT/'whole-town.png');save_render(draw_ledge_paths(scene,d,report['ledges']),out/'ledge-return-overlay.png')
 best=max((sum(k((x,y))=='FLOOR'for y in range(q[1]-4,q[1]+5)for x in range(q[0]-5,q[0]+5)),q)for q in seen if k(q)=='FLOOR'and 5<=q[0]<=58 and 4<=q[1]<=43)
 views=d['viewpoints']+[dict(id='WIDEST_FLOOR',tile=list(best[1]))]
 def cameras(im,views,prefix):
  frames=[]
  for v in views:
   x,y=v['tile'];cx,cy=x*16+8,y*16+8;name=prefix+v['id'];crop=render_stage(im,'crop',out,name,(cx-80,cy-72,cx+80,cy+72));frame=render_stage(crop,'convert',out,name+'-rgba','RGBA');frame.alpha_composite(oriented_sprite(pack,'SPRITE_CHRIS','south'),(72,60));save_render(frame,out/(name+'-160.png'));big=render_stage(frame,'resize',out,name+'-4x',(640,576),Image.Resampling.NEAREST);save_render(big,out/(name+'-4x.png'));frames.append((name,big))
  panel=Image.new('RGB',(1304,618*((len(frames)+1)//2)),(19,29,40));draw=ImageDraw.Draw(panel)
  for i,(name,frame)in enumerate(frames):x=i%2*652+6;y=i//2*618+24;draw.text((x,y-18),name,fill='white');panel.paste(frame,(x,y))
  save_render(panel,out/(prefix+'cameras.png'))
 cameras(scene,views,'town-')
 for reference in d['interiors']:
  r=preview_room(reference,pack)
  w,h=r['dimensions_metatiles'];rb=[int(v,16)for row in r['rows']for v in row.split()];im=art.render(r['tileset'],w,h,rb,group='indoor',derived=r.get('derived_blocks'));save_render(im,out/(r['id']+'-terrain-original.png'));im=render_stage(im,'convert',out,r['id']+'-rgba','RGBA')
  for a in r['residents']:im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
  save_render(im,out/(r['id']+'-whole-rgba.png'));save_render(render_stage(im,'convert',out,r['id']+'-rgb','RGB'),out/(r['id']+'-whole.png'));cameras(im,[v for v in r['viewpoints']if v.get('phase')!='court_open'],r['id']+'-')
  if'canonical_design'in r:
   phases,closed,opened,open_blocks=canonical_phases(r,pack);open_im=art.render(r['tileset'],w,h,open_blocks,group='indoor',derived=r['derived_blocks']);save_render(open_im,out/(r['id']+'-open-terrain-original.png'));open_im=render_stage(open_im,'convert',out,r['id']+'-open-rgba','RGBA')
   for a in r['residents']:open_im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
   save_render(open_im,out/(r['id']+'-open-whole-rgba.png'));save_render(render_stage(open_im,'convert',out,r['id']+'-open-rgb','RGB'),out/(r['id']+'-open-whole.png'));cameras(open_im,[v for v in r['viewpoints']if v.get('phase')=='court_open'],r['id']+'-open-')
   (out/'canonical-gym-source.json').write_text(json.dumps(r['canonical_design'],indent=2)+'\n');(out/'canonical-gym-puzzle-phases.json').write_text(json.dumps(phases,indent=2)+'\n')
   panel=Image.new('RGB',(w*128+24,h*64+78),(19,29,40));draw=ImageDraw.Draw(panel);draw.text((12,10),'SOURCE fixture: court CLOSED / no puzzle events',fill='white');draw.text((w*64+18,10),'SOURCE fixture: court OPEN / actual gate prerequisites declared complete',fill='white')
   for i,state in enumerate([im,open_im]):big=render_stage(state,'resize',out,r['id']+'-phase-comparison-'+str(i),(w*64,h*64),Image.Resampling.NEAREST);panel.paste(big,(i*(w*64+12)+6,36))
   draw.text((12,h*64+46),'Original canonical art and actor positions; this preview does not execute native scripts or award events.',fill='white');save_render(panel,out/'canonical-gym-closed-open-comparison.png')
 report.update(revision=digest,widest_floor_camera=list(best[1]),render_ms=round((time.perf_counter()-started)*1000,1),directory=str(out),image=str(out/'whole-town.png'),status='SOURCE ONLY; no native service, gym, story, reward or regional walking claim')
 (out/'review.json').write_text(json.dumps(report,indent=2)+'\n');(OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2));return report

def main():
 global CANONICAL_BYTES
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-quality-review.crystalpack');p.add_argument('--town-source',type=Path,default=DESIGN);p.add_argument('--layout-source',type=Path,default=LAYOUT);p.add_argument('--canonical-source',type=Path,default=ROOT/'planning/minnesota/gyms/interiors.json',help='Explicit frozen review candidate; never replaces the declared authority');p.add_argument('--watch',action='store_true');args=p.parse_args();pack=load_pack(args.pack);last=None
 while True:
  sources=[p.read_bytes()for p in[args.town_source,args.layout_source]];canonical_bytes=args.canonical_source.read_bytes();CANONICAL_BYTES=canonical_bytes;key=hashlib.sha256(b'\0'.join(sources+[canonical_bytes])).hexdigest()[:16]
  if key!=last:
   archive=OUTPUT/'source-passes'/key;archive.mkdir(parents=True,exist_ok=True)
   for name,payload in zip(['MnBrainerd.json','MnBrainerd.layout.json'],sources):target=archive/name;assert not target.exists()or target.read_bytes()==payload;target.write_bytes(payload)
   (archive/'canonical-gyms-source.json').write_bytes(canonical_bytes)
   (archive/'input-paths.json').write_text(json.dumps(dict(town=str(args.town_source.resolve()),layout=str(args.layout_source.resolve()),canonical=str(args.canonical_source.resolve()),frozen_candidate=args.canonical_source.resolve()!=(ROOT/'planning/minnesota/gyms/interiors.json').resolve()),indent=2)+'\n')
   try:render(*(json.loads(v)for v in sources),pack)
   except(AssertionError,KeyError,IndexError,ValueError,TypeError)as e:
    (archive/'rejected.json').write_text(json.dumps(dict(error=str(e),source_pass=key),indent=2)+'\n');print('INVALID SOURCE:',e,flush=True)
    if not args.watch:raise SystemExit(1)from e
   last=key
  if not args.watch:break
  time.sleep(.15)
if __name__=='__main__':main()
