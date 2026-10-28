#!/usr/bin/env python3
"""Bind an input plan to actual native river collision; never execute gameplay.

Only an in-memory copy of the declared existing badge gate is opened. Native
controller execution must satisfy actual Surf/Waterfall prerequisites and run
the real gate callbacks. The content pack is read-only.
"""
import argparse
import sys,json,hashlib
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from render_reference import load_pack
from audit_outdoor_connections import collision,project
ap=argparse.ArgumentParser(description=__doc__)
ap.add_argument('--pack',type=Path,required=True)
ap.add_argument('--expected-sha256',required=True)
ap.add_argument('--expected-map-count',type=int,choices=[677,681],default=677)
ap.add_argument('--source-plan',type=Path,required=True)
ap.add_argument('--manifest',type=Path,required=True)
ap.add_argument('--output',type=Path,required=True)
args=ap.parse_args()
PACK,EXPECTED,OUT,SOURCE,MANIFEST=args.pack,args.expected_sha256,args.output,args.source_plan,args.manifest
OUT.mkdir(parents=True,exist_ok=True)
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(PACK.read_bytes())==EXPECTED
pack=load_pack(PACK);maps=pack['data']['maps'];assert len(maps)==args.expected_map_count
cohesion_ids={'MnDetroitLakesApproach','MnMaraisCedarApproach','MnPreservePineEntry','MnBemidjiNorthwoods'}
if args.expected_map_count==681:
 parent=PACK.parent/'minnesota-mississippi-connected-v02.crystalpack'
 assert sha(parent.read_bytes())=='6b36b4b835873e4c4508a7da7727ecb759273c26e219ec026c061c0e66f24775'
 parent_names=set(load_pack(parent)['data']['maps'])
 assert len(parent_names)==677 and set(maps)==parent_names|cohesion_ids
else:
 assert not cohesion_ids & set(maps)
manifest=json.loads(MANIFEST.read_bytes());plan=json.loads(SOURCE.read_bytes());assert plan['source_manifest_sha256']==sha(MANIFEST.read_bytes())
for row in manifest['new_maps']:
 module=maps[row['id']]
 assert [module['attributes']['width'],module['attributes']['height']]==row['dimensions_metatiles']
 assert sha(bytes(module['blocks']))==row['source_blocks_sha256'],row['id']+': frozen river terrain changed'
# A geometry fixture only: controller regression must open this existing gate
# through native callbacks and actual declared initial badges.
from copy import deepcopy
mpls=deepcopy(maps['MnMinneapolis']);maps=dict(maps,MnMinneapolis=mpls)
gates=[r for r in mpls['script_block_changes'] if r['source_script']=='MnMPUpperPoolBuoyOpen'];assert len(gates)==3
for r in gates:mpls['blocks'][(r['y']//2)*mpls['attributes']['width']+r['x']//2]=r['block_id']
kinds={n:collision(pack,m) for n,m in maps.items() if n.startswith('Mn')}
inv=json.loads(Path(manifest['bridge_inventory']).read_bytes());assert inv['base_pack_sha256']==manifest['base_pack_sha256']
apertures=[];aperture_nodes={}
for i,row in enumerate(inv['bridges']):
 cells=[r['metatile'] for r in row['metatiles']];bounds=[min(x for x,y in cells),min(y for x,y in cells),max(x for x,y in cells)+1,max(y for x,y in cells)+1]
 entry=dict(id=row.get('id',row['map']+'-bridge-'+str(i+1)),map=row['map'],metatile_bounds_exclusive=bounds)
 apertures.append(entry)
 for x,y in cells:
  for dx,dy in [(0,0),(1,0),(0,1),(1,1)]:
   p=x*2+dx,y*2+dy;kind=kinds[row['map']][p]
   axis='NS' if row['axis']=='north_south' else 'EW'
   assert kind in {'BRIDGE_'+axis+'_DECK','BRIDGE_'+axis+'_RAIL'},(row['map'],p,kind)
   aperture_nodes[(row['map'],*p)]=entry['id']
start=plan['start'];launch=plan['launch'];node=(start['map'],*start['point']);assert kinds[node[0]][node[1:]]=='FLOOR'
dirs={'up':(0,-1,'north'),'down':(0,1,'south'),'left':(-1,0,'west'),'right':(1,0,'east')}
dx,dy,_=dirs[launch['direction']];assert (launch['map'],*launch['point'])==(node[0],node[1]+dx,node[2]+dy)
node=(launch['map'],*launch['point']);assert kinds[node[0]][node[1:]]=='WATER'
visited=set();waterfall_checks=[]
for index,step in enumerate(plan['steps']):
 dx,dy,compass=dirs[step['direction']];expected=(step['map'],*step['point']);a=maps[node[0]]['attributes']
 if step.get('action')=='waterfall_up':
  assert step['direction']=='up' and expected[0]==node[0] and expected[1]==node[1] and expected[2]<node[2]-1
  assert kinds[node[0]][node[1:]]==kinds[expected[0]][expected[1:]]=='WATER'
  assert all(kinds[node[0]].get((node[1],y))=='WATERFALL' for y in range(expected[2]+1,node[2]))
  waterfall_checks.append(dict(input_index=index,before=list(node),after=list(expected)))
 else:
  q=node[1]+dx,node[2]+dy
  if not(0<=q[0]<a['width']*2 and 0<=q[1]<a['height']*2):
   edge=next(e for e in a['connections'] if e['direction']==compass)
   actual=(edge['target_map'],*project(a,maps[edge['target_map']]['attributes'],edge,node[1:]))
  else:actual=(node[0],*q)
  assert actual==expected,(index,node,actual,expected)
  for n in [node,expected]:
   k=kinds[n[0]][n[1:]]
   assert k in {'WATER','BRIDGE_NS_DECK','BRIDGE_NS_RAIL','BRIDGE_EW_DECK','BRIDGE_EW_RAIL'},(index,n,k)
   assert not(k.startswith('BRIDGE_NS') and dx or k.startswith('BRIDGE_EW') and dy),(index,n,k,step['direction'])
 node=expected
 if node in aperture_nodes:visited.add(aperture_nodes[node])
 assert (node[1],node[2]) not in {(o['x'],o['y']) for o in maps[node[0]]['objects']},(index,'actor blocks planned water')
assert node==(plan['target']['map'],*plan['target']['point'])
assert len(visited)==len(apertures)==7
plan.update(status='Native output collision/seam geometry verified; production controller execution still pending',pack=str(PACK),pack_sha256=EXPECTED,source_input_plan_sha256=sha(SOURCE.read_bytes()),bridge_apertures=apertures)
(OUT/'all-bridges-input-plan.json').write_text(json.dumps(plan,indent=2)+'\n')
proof=dict(status='NATIVE_COLLISION_PLAN_BINDING_PASS_NOT_CONTROLLER_PROOF',map_count=len(maps),frozen_river_maps_checked=len(manifest['new_maps']),pack_sha256=EXPECTED,source_manifest_sha256=sha(MANIFEST.read_bytes()),source_input_plan_sha256=sha(SOURCE.read_bytes()),input_count=len(plan['steps']),bridge_count=7,visited_bridges=sorted(visited),waterfall_checks=waterfall_checks,gate_fixture='Only in-memory declared existing native gate-open block changes; no pack changes; controller must use native badge callbacks',pack_unchanged=sha(PACK.read_bytes())==EXPECTED)
(OUT/'native-plan-binding-proof.json').write_text(json.dumps(proof,indent=2)+'\n')
print(json.dumps(proof))
