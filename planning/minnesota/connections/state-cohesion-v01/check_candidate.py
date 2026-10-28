#!/usr/bin/env python3
"""Detached source geometry proof, never a native installer/controller."""
import argparse,collections,copy,hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack,layout_blocks,SourceArt
from render_progress_atlas import scene,font
from render_archive import save_render
from audit_outdoor_connections import collision,neighbor_origin,project,OPPOSITE
from author_review import PATCH_STROKES,candidates,digest,write
from PIL import Image,ImageDraw
BASE=ROOT/'content-packs/minnesota-twin-cities-compact-v01.crystalpack'
BASE_HASH='2fe6fc2a2c59833886b9ffa2fa04584f1f95841efaf87e865f11230cf4cdf54e'
# A side, B, offset in metatiles, two exact A lanes; B lanes are projected.
LINKS=[
 ('MnBrainerd','west','MnR15C00R00',0,[[0,24],[0,25]]),
 ('MnR15C00R00','west','MnDetroitLakesApproach',3,[[0,24],[0,25]]),
 ('MnDetroitLakesApproach','west','MnR16C01R00',-3,[[0,18],[0,19]]),
 ('MnBrainerd','east','MnR18C00R00',7,[[63,24],[63,25]]),
 ('MnBemidji','north','MnBemidjiNorthwoods',8,[[24,0],[25,0]]),
 ('MnBemidjiNorthwoods','north','MnR17C00R02',-12,[[8,0],[9,0]]),
 ('MnLakeItasca','west','MnR19C00R00',8,[[0,28],[0,29]]),
 ('MnGrandMarais','east','MnMaraisCedarApproach',0,[[39,12],[39,13]]),
 ('MnMaraisCedarApproach','south','MnR11C00R00',8,[[56,37],[57,37]]),
 ('MnGrandMarais','west','MnR12C01R01',0,[[0,8],[0,9]]),
 ('MnBoundaryWaters','north','MnPreservePineEntry',8,[[32,0],[33,0]]),
 ('MnPreservePineEntry','north','MnBoundarySafariC00R02',1,[[16,0],[17,0]]),
 ('MnLakeItasca','east','MnR14C00R00',-9,[[47,20],[47,21]]),
]
GUARDS=[
 dict(map='MnBrainerd',id='MnCohesionBrainerdWest',side='west',bodies=[[2,24],[2,25]],inside=[[3,24],[3,25]],outside=[[1,24],[1,25]],flags=['ENGINE_MINERALBADGE']),
 dict(map='MnBrainerd',id='MnCohesionBrainerdEast',side='east',bodies=[[61,24],[61,25]],inside=[[60,24],[60,25]],outside=[[62,24],[62,25]],flags=['ENGINE_MINERALBADGE']),
 dict(map='MnBemidji',id='MnCohesionBemidjiNorth',side='north',bodies=[[24,2],[25,2]],inside=[[24,3],[25,3]],outside=[[24,1],[25,1]],flags=['ENGINE_GLACIERBADGE']),
 dict(map='MnLakeItasca',id='MnCohesionItascaWest',side='west',bodies=[[2,28],[2,29]],inside=[[3,28],[3,29]],outside=[[1,28],[1,29]],flags=['ENGINE_MINERALBADGE']),
 dict(map='MnGrandMarais',id='MnCohesionMaraisWest',side='west',bodies=[[2,8],[2,9]],inside=[[3,8],[3,9]],outside=[[1,8],[1,9]],flags=['ENGINE_ZEPHYRBADGE','ENGINE_HIVEBADGE','ENGINE_PLAINBADGE','ENGINE_FOGBADGE','ENGINE_STORMBADGE','ENGINE_MINERALBADGE','ENGINE_GLACIERBADGE','ENGINE_RISINGBADGE'],event='EVENT_MN_LIGHTHOUSE_SIGNAL'),
 dict(map='MnGrandMarais',id='MnCohesionMaraisEast',side='east',bodies=[[37,12],[37,13]],inside=[[36,12],[36,13]],outside=[[38,12],[38,13]],flags=['ENGINE_ZEPHYRBADGE','ENGINE_HIVEBADGE','ENGINE_PLAINBADGE','ENGINE_FOGBADGE','ENGINE_STORMBADGE','ENGINE_MINERALBADGE','ENGINE_GLACIERBADGE','ENGINE_RISINGBADGE'],event='EVENT_MN_LIGHTHOUSE_SIGNAL'),
]

def build(pack):
 maps=copy.deepcopy({n:m for n,m in pack['data']['maps'].items()if n.startswith('Mn')and str(m['attributes']['environment']).lower()in('town','route','outdoor')})
 for name,strokes in PATCH_STROKES.items():
  m=maps[name];w=m['attributes']['width']
  for (x0,y0,x1,y1),block in strokes:
   for y in range(y0,y1):
    for x in range(x0,x1):m['blocks'][y*w+x]=block
 m=maps['MnR03C00R01'];w=m['attributes']['width'];m['blocks']=[b for y in range(m['attributes']['height'])for b in m['blocks'][y*w:y*w+21]];m['attributes']['width']=21
 for r in candidates():
  l=copy.deepcopy(r['layout'])
  for s in r['signs']:
   x,y=s['tile'];l['layers'].append(dict(name=s['id'],kind='point',position=[x//2,y//2],block='45'))
  w,h=l['dimensions_metatiles'];maps[r['id']]=dict(id=r['id'],attributes=dict(width=w,height=h,tileset_name='johto',map_constant='MN_'+r['id'][2:].upper(),environment='route',connections=[]),blocks=layout_blocks(l),objects=[dict(x=a['tile'][0],y=a['tile'][1],sprite=a['sprite'],object_identifier=a['id'])for a in r['actors']],events=dict(warps=[],bg_events=[dict(x=s['tile'][0],y=s['tile'][1])for s in r['signs']],coord_events=[]))
 def edge(n,side,t,offset,replace=False):
  es=maps[n]['attributes']['connections'];old=next((e for e in es if e['direction']==side),None)
  if old:
   assert replace and old['target_map']==t,(n,side,t,old)
   old['offset']=offset
  else:es.append(dict(direction=side,target_map=t,offset=offset))
 for n,side,t,offset in [('MnBemidji','south','MnR08C00R00',0),('MnR08C00R00','north','MnBemidji',0),('MnBoundaryWaters','south','MnR13C00R00',0),('MnR13C00R00','north','MnBoundaryWaters',0)]:edge(n,side,t,offset,True)
 for n,s,t,o,lanes in LINKS:edge(n,s,t,o);edge(t,OPPOSITE[s],n,-o)
 return maps

def flood(col,start,blocked=set()):
 seen={start};q=collections.deque([start])
 while q:
  x,y=q.popleft()
  for p in[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
   if p not in seen and p not in blocked and col.get(p)in('FLOOR','TALL_GRASS'):seen.add(p);q.append(p)
 return seen

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--output',type=Path,required=True);ap.add_argument('--render-atlas',action='store_true');ap.add_argument('--base',type=Path,default=BASE);ap.add_argument('--expected-sha',default=BASE_HASH);a=ap.parse_args();a.output.mkdir(parents=True,exist_ok=False)
 assert hashlib.sha256(a.base.read_bytes()).hexdigest()==a.expected_sha
 pack=load_pack(a.base);maps=build(pack);cols={n:collision(pack,m)for n,m in maps.items()};checks=[];links=[];negatives=[]
 def check(id,ok,**detail):checks.append(dict(id=id,pass_=bool(ok),**detail))
 for n,s,t,o,lanes in LINKS:
  edge=next(e for e in maps[n]['attributes']['connections']if e['direction']==s);target=[project(maps[n]['attributes'],maps[t]['attributes'],edge,p)for p in lanes]
  check(n+'-'+s+'-two-native-floor-lanes',all(cols[n][tuple(p)]=='FLOOR'and cols[t][tuple(q)]=='FLOOR'for p,q in zip(lanes,target)),source=lanes,target=target)
  check(n+'-'+s+'-reciprocal',any(e['direction']==OPPOSITE[s]and e['target_map']==n and e['offset']==-o for e in maps[t]['attributes']['connections']))
  links.append(dict(from_map=n,direction=s,to_map=t,offset=o,reverse_offset=-o,from_core=lanes,to_core=target,status='PROPOSED_NOT_INSTALLED'))
 for r in candidates():
  n=r['id'];ports=[tuple(p)for e in r['ports']for p in e['core']];blocked={(o['x'],o['y'])for o in maps[n]['objects']};seen=flood(cols[n],ports[0],blocked)
  check(n+'-actor-present-ordinary-travel-both-lanes',all(p in seen for p in ports))
  for o in maps[n]['objects']:
   p=(o['x'],o['y']);check(o['object_identifier']+'-floor-and-adjacent',cols[n][p]=='FLOOR'and any(q in seen for q in[(p[0]-1,p[1]),(p[0]+1,p[1]),(p[0],p[1]-1),(p[0],p[1]+1)]))
  check(n+'-no-incidental-HM-water-ledge',set(cols[n].values())<={'FLOOR','TALL_GRASS','WALL'})
  perimeter={(x,y)for x,y in cols[n]if x==0 or y==0 or x==maps[n]['attributes']['width']*2-1 or y==maps[n]['attributes']['height']*2-1};check(n+'-closed-all-other-borders',all(cols[n][p]=='WALL'for p in perimeter-set(ports)))
  dry={p:k if k!='TALL_GRASS'else'WALL'for p,k in cols[n].items()};check(n+'-grass-not-freely-avoidable',not all(p in flood(dry,ports[0],blocked)for p in ports)if any(k=='TALL_GRASS'for k in cols[n].values())else True)
 for g in GUARDS:
  pts=g['bodies']+g['inside']+g['outside'];check(g['id']+'-all-native-floor',all(cols[g['map']].get(tuple(p))=='FLOOR'for p in pts))
 # Every untouched source row/collision and old native anchor retains exact data.
 for n in PATCH_STROKES:
  before=pack['data']['maps'][n];after=maps[n];assert before['objects']==after['objects']and before['events']==after['events']
  w=before['attributes']['width'];allowed={(x,y)for bounds,_ in PATCH_STROKES[n]for y in range(bounds[1],bounds[3])for x in range(bounds[0],bounds[2])}
  check(n+'-all-outside-owned-blocks-exact',all(b==after['blocks'][i]for i,b in enumerate(before['blocks'])if(i%w,i//w)not in allowed))
  c0=collision(pack,before);pts={(o['x'],o['y'])for o in before['objects']}|{(o['x'],o['y'])for k in['warps','bg_events','coord_events']for o in before['events'][k]};check(n+'-all-old-anchor-collision-exact',all(c0[p]==cols[n][p]for p in pts))
  check(n+'-old-water-grass-cut-quadrants-exact',all(cols[n][p]==kind for p,kind in c0.items()if kind in('WATER','WATERFALL','TALL_GRASS')or'CUT'in kind))
 # Placement equations are native offset equations, not arbitrary art arrangement.
 pos={'MnMinneapolis':(0,0)};q=collections.deque(pos);cycle=[]
 while q:
  n=q.popleft()
  for e in maps[n]['attributes']['connections']:
   t=e['target_map']
   if t not in maps:continue
   v=neighbor_origin(pos[n],maps[n]['attributes'],maps[t]['attributes'],e)
   if t in pos:
    if pos[t]!=v:cycle.append(dict(from_map=n,to_map=t,existing=pos[t],implied=v))
   else:pos[t]=v;q.append(t)
 check('single-proposed-direct-seam-component',len(pos)==len(maps),reached=len(pos),total=len(maps));check('all-offset-equations-consistent',not cycle,conflicts=cycle)
 overlaps=[]
 for i,n in enumerate(pos):
  x,y=pos[n];aa=maps[n]['attributes']
  for t in list(pos)[i+1:]:
   u,v=pos[t];bb=maps[t]['attributes'];r=[max(x,u),max(y,v),min(x+aa['width'],u+bb['width']),min(y+aa['height'],v+bb['height'])]
   if r[0]<r[2]and r[1]<r[3]:overlaps.append(dict(maps=[n,t],bounds_metatiles=r))
 check('no-native-map-rectangles-overlap',not overlaps,overlaps=overlaps)
 # Actual ordinary gatehouse is evidence, not inferred from a filename.
 gate=pack['data']['maps']['MnItascaGatehouse'];commands=collections.Counter(c['command']for rows in gate['scripts'].values()for c in rows)
 check('Itasca-gatehouse-no-script-access-condition',set(commands)<={'jumptextfaceplayer','text','line','para','done'}and not gate['events']['coord_events']and not gate['scenes']['scenes'])
 itasca=dict(native_module_sha256=digest(gate),raw_command_kinds=dict(commands),coord_event_count=len(gate['events']['coord_events']),scene_count=len(gate['scenes']['scenes']),warp_count=len(gate['events']['warps']),retained_exact=True,existing_access='Bemidji west MineralBadge checkpoint remains exact; marker separately needs GlacierBadge.',parallel_trail='New ordinary woodland connection, not a permit bypass.')
 write(a.output/'proposed-integration.contract.json',dict(schema_version=1,status='DETACHED_SOURCE_PROPOSAL_NOT_NATIVE_OR_CONTROLLER_PROOF',base_pack_sha256=a.expected_sha,map_count_before=len(pack['data']['maps']),new_maps=[dict(id=r['id'],group_proposal=83,id_proposal=i+1,layout_file='source-v01/'+r['id']+'.layout.json',dossier_file='source-v01/'+r['id']+'.dossier.json')for i,r in enumerate(candidates())],new_reciprocal_links=links,existing_connection_updates=[dict(map='MnBemidji',side='south',target='MnR08C00R00',old_offset=-8,new_offset=0),dict(map='MnR08C00R00',side='north',target='MnBemidji',old_offset=8,new_offset=0),dict(map='MnBoundaryWaters',side='south',target='MnR13C00R00',old_offset=1,new_offset=0),dict(map='MnR13C00R00',side='north',target='MnBoundaryWaters',old_offset=-1,new_offset=0)],guard_proposals=GUARDS,itasca_gatehouse=itasca,proposed_origins_metatiles=pos,missing_hubs=['MnEly','MnDetroitLakes','MnMoorhead','MnInternationalFalls','MnMilleLacs'],explicit_limitations=['New maps and links are not installed.','Pending preserve admission remains closed; no capture/session/fee claim.','New chapter guards require native lowering and controller denial/success/free-return coverage.','Room warps and all existing event/script records must remain exact.','Group83 must be rechecked against fresh river parent before publication.']))
 write(a.output/'source-proof.json',dict(status='STATIC_SOURCE_GEOMETRY_NOT_CONTROLLER_PROOF',checks=checks,failures=[c for c in checks if not c['pass_']]))
 if a.render_atlas:
  scale=6;minx=min(x for x,y in pos.values());miny=min(y for x,y in pos.values());maxx=max(pos[n][0]+m['attributes']['width']for n,m in maps.items());maxy=max(pos[n][1]+m['attributes']['height']for n,m in maps.items());canvas=Image.new('RGB',((maxx-minx)*scale+80,(maxy-miny)*scale+140),'#17232c');draw=ImageDraw.Draw(canvas);art=SourceArt(pack)
  new={r['id']:r for r in candidates()}
  for n,(x,y)in pos.items():
   m=maps[n];aa=m['attributes'];actors=new[n]['actors']if n in new else[dict(tile=[o['x'],o['y']],sprite=o['sprite'],facing=o['spritemovedata'].removeprefix('SPRITEMOVEDATA_STANDING_').lower())for o in m['objects']];im=scene(art,pack,aa['tileset_name'],aa['width'],aa['height'],m['blocks'],actors,native_origin=True);im=im.resize((aa['width']*scale,aa['height']*scale),Image.Resampling.NEAREST);xx=(x-minx)*scale+40;yy=(y-miny)*scale+100;canvas.paste(im,(xx,yy));draw.rectangle((xx,yy,xx+im.width-1,yy+im.height-1),outline='#f0b95f'if n in new or n in PATCH_STROKES else'#526875');draw.text((xx+2,yy+2),n,fill='white',font=font(9))
  draw.text((18,10),'DETACHED COHESION TERRAIN PROPOSAL — NOT INSTALLED',font=font(20,True),fill='white');draw.text((18,40),str(len(maps))+' actual/proposed outdoor maps at one scale; amber = proposed source/port patch.',font=font(13),fill='#e5bd70');draw.text((18,62),'Preserve remains admission-closed. Five missing towns are not fabricated here.',font=font(13),fill='#e5bd70');save_render(canvas,a.output/'proposed-full-state-source.png')
 print(json.dumps(dict(checks=len(checks),failures=[c for c in checks if not c['pass_']],output=str(a.output))))
if __name__=='__main__':main()
