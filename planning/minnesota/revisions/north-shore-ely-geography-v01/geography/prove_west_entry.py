#!/usr/bin/env python3
"""Read-only exact rectangle/connection proposal; no terrain or canonical mutation."""
import copy,hashlib,json,cbor2
from collections import deque
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[5]
META=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/current-v01/all-exterior-connection-metadata.json'
ATLAS=ROOT/'planning/minnesota/renders/river-continuity-long-v01/whole-state-native-v01/native-connection-audit.json'
OUT=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/west-entry-placement-v02'
assert not OUT.exists();OUT.mkdir()
base=json.loads(META.read_bytes());old=base['exteriors']
wp=ROOT/'content-packs/minnesota-superior-warm-coast-v01.crystalpack';raw=wp.read_bytes();warmsha='ebea77ccb9b9dbe54aa5101e32590c1b075ed8130f6e7fcfe2d8c8451dab11d8';assert hashlib.sha256(raw).hexdigest()==warmsha;actual=cbor2.loads(raw[22:]);current={n:dict(dimensions_metatiles=[actual['data']['maps'][n]['attributes']['width'],actual['data']['maps'][n]['attributes']['height']],connections=actual['data']['maps'][n]['attributes']['connections'])for n in old};assert current==old
base['actual_pack_sha256']=warmsha;maps=copy.deepcopy(current)
def conn(n,d,t,o):
 c=maps[n]['connections'];c[:]=[x for x in c if x['direction']!=d]
 if t:c.append(dict(direction=d,target_map=t,offset=o))
neck='MnMaraisInlandNeck';maps[neck]=dict(dimensions_metatiles=[65,19],connections=[])
for n,d,t,o in [('MnR11C00R00','north',None,0),('MnR11C01R00','east','MnGrandMarais',0),('MnGrandMarais','west','MnR11C01R00',0),('MnGrandMarais','east',None,0),('MnGrandMarais','north','MnMaraisCedarApproach',-19),('MnMaraisCedarApproach','south','MnGrandMarais',19),('MnMaraisCedarApproach','west',neck,0),(neck,'east','MnMaraisCedarApproach',0),(neck,'west','MnR12C01R01',0),('MnR12C01R01','east',neck,0)]:conn(n,d,t,o)
def projected(n,origin,c):
 x,y=origin;w,h=maps[n]['dimensions_metatiles'];tw,th=maps[c['target_map']]['dimensions_metatiles'];o=c['offset']
 return {'north':(x+o,y-th),'south':(x+o,y+h),'east':(x+w,y+o),'west':(x-tw,y+o)}[c['direction']]
def prove(ms):
 global maps
 maps=ms;positions={'MnMinneapolis':(0,0)};q=deque(positions);bad=[];reverse=[]
 opposite=dict(north='south',south='north',east='west',west='east')
 while q:
  n=q.popleft();assert len({c['direction']for c in maps[n]['connections']})==len(maps[n]['connections'])
  for c in maps[n]['connections']:
   t=c['target_map'];assert t in maps;tpos=projected(n,positions[n],c)
   if t in positions:
    if positions[t]!=tpos:bad.append(dict(source=n,connection=c,expected=tpos,actual=positions[t]))
   else:positions[t]=tpos;q.append(t)
   if dict(direction=opposite[c['direction']],target_map=n,offset=-c['offset'])not in maps[t]['connections']:reverse.append([n,c])
 overlaps=[];names=sorted(positions)
 for i,n in enumerate(names):
  x,y=positions[n];w,h=maps[n]['dimensions_metatiles']
  for m in names[i+1:]:
   xx,yy=positions[m];ww,hh=maps[m]['dimensions_metatiles'];a=max(x,xx);b=max(y,yy);c=min(x+w,xx+ww);d=min(y+h,yy+hh)
   if a<c and b<d:overlaps.append(dict(maps=[n,m],intersection=[a,b,c,d]))
 return dict(map_count=len(maps),reached=len(positions),inconsistent_cycles=bad,nonreciprocal_connections=reverse,overlaps=overlaps,positions=positions)
proposal=copy.deepcopy(maps);r=prove(proposal);assert r['reached']==100 and not r['inconsistent_cycles']and not r['overlaps']and not r['nonreciprocal_connections']
oldpos={x['map']:x['origin_metatiles']for x in json.loads(ATLAS.read_bytes())['placements']};moved={n:dict(before=oldpos.get(n),proposed=list(p))for n,p in r['positions'].items()if oldpos.get(n)!=list(p)}
assert set(moved)=={'MnGrandMarais','MnMaraisCedarApproach',neck}
# Existing Ely builder's exact paired seam, without moving any other footprint.
ely=copy.deepcopy(proposal);ely['MnEly']=dict(dimensions_metatiles=[20,18],connections=[dict(direction='north',target_map='MnR13C00R02',offset=-5)])
ely['MnR13C00R02']['connections'].append(dict(direction='south',target_map='MnEly',offset=5));er=prove(ely)
negative=[]
def rejects(bad):
 try:
  q=prove(bad);return bool(q['inconsistent_cycles']or q['overlaps']or q['nonreciprocal_connections']or q['reached']!=len(bad)or any(q['positions'].get(n)!=tuple(p)for n,p in oldpos.items()if n not in {'MnGrandMarais','MnMaraisCedarApproach'}))
 except AssertionError:return True
for label in ['coast_offset','neck_width','missing_reciprocal','duplicate_direction','stale_cedar_south','ely_offset']:
 bad=copy.deepcopy(ely)
 if label=='coast_offset':next(c for c in bad['MnGrandMarais']['connections']if c['direction']=='west')['offset']=1
 elif label=='neck_width':bad[neck]['dimensions_metatiles'][0]=66
 elif label=='missing_reciprocal':bad['MnGrandMarais']['connections']=[c for c in bad['MnGrandMarais']['connections']if c['direction']!='west']
 elif label=='duplicate_direction':bad['MnGrandMarais']['connections'].append(dict(direction='west',target_map='MnR11C01R00',offset=0))
 elif label=='stale_cedar_south':bad['MnMaraisCedarApproach']['connections'].append(dict(direction='south',target_map='MnR11C00R00',offset=8))
 else:bad['MnEly']['connections'][0]['offset']=-4
 assert rejects(bad),label;negative.append(dict(corruption=label,rejected=True))
result=dict(negative_corruptions=negative,warm_parent_connections_exactly_match_previous_99=True,status='PROPOSED GEOMETRY ONLY; terrain/ports/scripts not yet authored or played',pack_sha256=base['actual_pack_sha256'],inputs_sha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in[META,ATLAS]},direct_coast=r,changed_footprints=moved,ely_additive_geometry=er,ports_core=dict(north_shore_foot=[[47,8],[47,9]],town_foot=[[0,8],[0,9]],north_shore_water=[[47,16],[47,35]],town_water=[[0,16],[0,35]],town_north_foot=[[18,0],[19,0]],cedar_south_foot=[[56,37],[57,37]],neck_west=[[0,8],[0,9]],neck_east=[[129,12],[129,13]]),limitations=['Existing connection graph and rectangles checked globally; replacement collisions, actors, scripts, camera views and live controller still require authored candidate.','New 65x19 neck name provisional; no canonical native module exists yet.','Ely builder adds nine indoor maps as well as this exterior; they do not occupy outdoor atlas rectangles.'])
(OUT/'proof.json').write_text(json.dumps(result,indent=2)+'\n')
(OUT/'proposed-exterior-connections.json').write_text(json.dumps(dict(exteriors=proposal),indent=2)+'\n')
# Local geometry diagram explicitly uses no fabricated terrain.
selected=[n for n in er['positions']if n.startswith(('MnR11','MnR12','MnR13'))or n in{'MnGrandMarais','MnMaraisCedarApproach',neck,'MnEly','MnBoundaryWaters','MnPreservePineEntry'}]
x0=min(er['positions'][n][0]for n in selected);y0=min(er['positions'][n][1]for n in selected);x1=max(er['positions'][n][0]+ely[n]['dimensions_metatiles'][0]for n in selected);y1=max(er['positions'][n][1]+ely[n]['dimensions_metatiles'][1]for n in selected);s=8;im=Image.new('RGB',((x1-x0)*s+32,(y1-y0)*s+100),'#14202b');d=ImageDraw.Draw(im);d.text((12,10),'WEST ENTRY / SUPERSEDES OLD SOUTH-TOWN PLAN / rectangles only',fill='white');d.text((12,31),'WEST ENTRY: all100 consistent; Ely adds101st. Rectangles only; no authored terrain.',fill='white')
for n in selected:
 x,y=er['positions'][n];w,h=ely[n]['dimensions_metatiles'];box=(16+(x-x0)*s,72+(y-y0)*s,16+(x+w-x0)*s-1,72+(y+h-y0)*s-1);col='#426c55'if n.startswith('MnR11')or n=='MnGrandMarais'else'#7e9fab'if n.startswith(('MnR12','MnR13'))or n=='MnEly'else'#8c7e5d';d.rectangle(box,fill=col,outline='white');d.text((box[0]+3,box[1]+3),n.removeprefix('Mn'),fill='white')
im.save(OUT/'proposed-geography-rectangles.png');print(json.dumps(dict(proposal={k:r[k]for k in['map_count','reached','inconsistent_cycles','nonreciprocal_connections','overlaps']},moved=moved,ely={k:er[k]for k in['map_count','reached','inconsistent_cycles','nonreciprocal_connections','overlaps']},ely_origin=er['positions']['MnEly'])))
