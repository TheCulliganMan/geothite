"""Plan legal fixture courses on combined source-v05; no runtime game writes."""
from pathlib import Path
from heapq import heappop,heappush
import hashlib,json,cbor2
ROOT=Path(__file__).resolve().parents[4];folder=ROOT/'planning/minnesota/revisions/r04-route-quality-v01/source-v05'
raw=(ROOT/'content-packs/minnesota-river-long-v02.crystalpack').read_bytes();assert hashlib.sha256(raw).hexdigest()=='3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243'
base=cbor2.loads(raw[22:]);m=base['data']['maps']['MnR04C00R00'];contract=json.loads((folder/'native.contract.json').read_bytes());grid=json.loads((folder/'compiled-candidate.blocks.json').read_bytes())['blocks']
c=dict(base['data']['tilesets']['mn_river_park']['collision'])
for b in contract['blocks']:c[f"{b['target']:02x}"]=b['collision']
kind=lambda p:c[f"{grid[p[1]//2*28+p[0]//2]:02x}"][p[1]%2*2+p[0]%2]
occupied={(o['x'],o['y'])for o in m['objects']};sight={(18,18),(42,25)}
walk={'FLOOR','TALL_GRASS','LONG_GRASS'}
def route(start,target):
 heap=[(0,0,start)];best={start:(0,0)};parent={}
 while heap:
  grass,steps,p=heappop(heap)
  if (grass,steps)!=best[p]:continue
  if p==target:break
  for q in [(p[0]-1,p[1]),(p[0]+1,p[1]),(p[0],p[1]-1),(p[0],p[1]+1)]:
   if not(0<=q[0]<56 and 0<=q[1]<36)or q in occupied or q in sight or kind(q)not in walk:continue
   candidate=(grass+int(kind(q)!='FLOOR'),steps+1)
   if candidate<best.get(q,(10**9,10**9)):
    best[q]=candidate;parent[q]=p;heappush(heap,(*candidate,q))
 assert target in best,(start,target)
 path=[target]
 while path[-1]!=start:path.append(parent[path[-1]])
 path.reverse();buttons={(-1,0):'Left',(1,0):'Right',(0,-1):'Up',(0,1):'Down'}
 return dict(from_core=list(start),to_core=list(target),ordinary_steps=best[target][1],grass_steps=best[target][0],inputs=[{'from':list(a),'to':list(b),'button':buttons[(b[0]-a[0],b[1]-a[1])]} for a,b in zip(path,path[1:])])
old=json.loads((ROOT/'planning/minnesota/connections/river-continuity-r04-v01/source-v04/controller-input-plan.json').read_bytes())
courses=[dict(id=p['id'],**route(tuple(p['from_core']),tuple(p['to_core'])))for p in old['courses']]
for name,start,end in [('both-ports-west-east',(0,18),(55,18)),('both-ports-east-west',(55,18),(0,18)),('picnic-cache-paid-approach',(42,27),(48,27))]:courses.append(dict(id=name,**route(start,end)))
assert [c['grass_steps']for c in courses[:2]]==[4,4]
assert courses[-1]['grass_steps']==4
plan=dict(schema_version=1,status='PLANNED_PRODUCTION_INPUTS_NOT_EXECUTED',map='MnR04C00R00',parent_pack_sha256=contract['base_pack_sha256'],parent_identity=contract['base_identity'],candidate_contract_sha256=hashlib.sha256((folder/'native.contract.json').read_bytes()).hexdigest(),publication_identity_binding='Bind these fixture plans to the actual written candidate identity after publication; never rewrite a saved-game identity.',fixture_setup_only=old['fixture_setup_only'],press_rule=old['press_rule'],scope=dict(native_cell_changes=57,old_glyph_append_recipes=4,new_glyphs=0,sign_first_line='PARK POND BRIDGE'),actor_present_bodies=len(occupied),avoided_original_trainer_sight_tiles=[list(p)for p in sorted(sight)],courses=courses,bridge_collision_attempts=[dict(from_core=[28,18],button='Up',blocked_core=[28,17],expect='WALL gold north rail; position unchanged'),dict(from_core=[28,19],button='Down',blocked_core=[28,20],expect='WALL gold south rail; position unchanged')],park_port_cases=[dict(from_core=[0,18],button='Left',expect_map='MnMinneapolis',unchanged_connection_offset_metatiles=-9),dict(from_core=[55,18],button='Right',expect_map='MnStPaul',unchanged_connection_offset_metatiles=-15)],native_acceptance_cases=['Execute both directions, bridge walking rows18/19 and blocked gold rails17/20. Retain original bank passing room, both grass choices, bodies, trainer parties, reward/item flags, story and encounter tables.','Reach north and south closed-pond views on the planned ordinary courses. No new Surf mount witness or unconnected reachable water boundary.','At36,15 faceUp then A. Require PARK POND BRIDGE and Keep to the deck. through the same existing root. Dialogue must own input; unchanged flags, inventory and second text line.','Existing western trainer sight at18,18 still challenges; four-grass alternative avoids it. Existing optional trainer sight at42,25 remains. Bench24,12 and GreatBall49,27 retain their actual scripts and paid four-grass access.','Cross each actual native town seam, then return via the paired seam. Resolve and assert the existing native arrival coordinates; this source plan does not invent arrival coordinates from a diagram.','Save/Continue on the actual candidate identity, retaining party, flags, inventory and position. Fixture setup is not proof of earned campaign progression.','Full-pack residual: exactly57cells and encoded block mirror, four appended old-glyph metatile recipes/collision mirrors, one existing first sign line across raw/global/cached/typed mirrors, derived identity only. Every other map/program/art/audio/report/recovery record exact.'],new_surf_or_warp_route=False,static_paths_are_not_runtime_execution=True)
(folder/'controller-input-plan.json').write_text(json.dumps(plan,indent=2)+'\n')
print([(p['id'],p['ordinary_steps'],p['grass_steps'])for p in courses])
