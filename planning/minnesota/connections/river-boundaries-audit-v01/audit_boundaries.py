#!/usr/bin/env python3
"""Classify actual exterior water edges using production directional semantics.

Conservative static geometry, never an assertion of earned campaign access.
Includes bridge Surf planes; current/HM behavior is explicitly qualified.
"""
import argparse
from collections import Counter,deque
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
sys.path.insert(0,str(ROOT/'planning/minnesota/connections/river-continuity-v01'))
from render_reference import load_pack,room_sheet
from render_progress_atlas import native_room
from review_north_shore import oriented_sprite
from audit_outdoor_connections import collision,project,boundary
from audit_water_edges import production_permissions,numeric,blocked

PACK_SHA='3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243'
ATLAS=ROOT/'planning/minnesota/renders/river-continuity-long-v01/whole-state-native-v01/native-connection-audit.json'
DIRS={'west':(-1,0),'east':(1,0),'north':(0,-1),'south':(0,1)}
OPPOSITE={'west':'east','east':'west','north':'south','south':'north'}


def sha(raw):return hashlib.sha256(raw).hexdigest()


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--pack',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args();assert not args.output.exists()
 assert sha(args.pack.read_bytes())==PACK_SHA
 atlas=json.loads(ATLAS.read_bytes());assert atlas['pack_sha256']==PACK_SHA
 tokens,water,wall,source_hashes=production_permissions()
 for name in ['crates/crystal-assets/src/game_data_connections.rs','crates/crystal-core/src/world/movement.rs','crates/crystal-core/src/world/session.rs','crates/crystal-core/src/systems/field_moves.rs','crates/crystal-core/src/systems/script_blocks.rs']:
  source_hashes[name]=sha((ROOT/name).read_bytes())
 print('Reading exact actual pack; classifying directional Surf boundaries.',flush=True)
 pack=load_pack(args.pack);maps=pack['data']['maps'];placements={r['map']:r for r in atlas['placements']};names=sorted(placements)
 kinds={n:collision(pack,maps[n])for n in names};numbers={n:{p:numeric(t,tokens)for p,t in g.items()}for n,g in kinds.items()}
 edges={n:{e['direction']:e for e in maps[n]['attributes']['connections']}for n in names}
 surf_exclusions=set(pack['data']['field_moves']['surf']['blocked_collisions'])
 def wet(v):return v in water or 0xe0<=v<=0xe3
 def exit_block(v,d):return blocked(v,d,True)or(v in {0xe0,0xe1}and d in {'west','east'})or(v in {0xe2,0xe3}and d in {'north','south'})
 def enter(v,d):return not(blocked(v,d)or v in wall or(v in {0xe0,0xe1}and d in {'west','east'})or(v in {0xe2,0xe3}and d in {'north','south'}))
 def target(name,point,direction):
  dx,dy=DIRS[direction];q=point[0]+dx,point[1]+dy
  if q in numbers[name]:return name,q,False
  edge=edges[name].get(direction)
  if not edge:return None,None,True
  other=edge['target_map'];q=project(maps[name]['attributes'],maps[other]['attributes'],edge,point)
  return other,q,True
 # Multi-source over-approximation: every geometrically valid native Surf mount
 # is available, regardless of campaign reachability, actors, time, or HM badge.
 # Water currents are unrestricted here, and bridges use their real water axis.
 # Thus absence of a route is useful; presence still needs current/gate input.
 previous={};seed_witness={};todo=deque();mounts=[]
 for n in names:
  for point,v in numbers[n].items():
   if wet(v)or v in wall:continue
   for d,(dx,dy)in DIRS.items():
    q=point[0]+dx,point[1]+dy;w=numbers[n].get(q)
    if w in water and w not in surf_exclusions and not blocked(v,d,True)and not blocked(w,d):
     key=(n,*q);mount=dict(map=n,land_core=list(point),water_core=list(q),direction=d,land_permission=kinds[n][point],water_permission=kinds[n][q]);mounts.append(mount)
     if key not in previous:previous[key]=None;seed_witness[key]=mount;todo.append(key)
 # A warp that places a player on water is an additional possible Surf arrival.
 warp_seeds=[]
 for source,m in maps.items():
  for event in m['events']['warps']:
   dest=event.get('target_map');index=event.get('target_warp_id')
   if dest not in numbers or not isinstance(index,int):continue
   arrivals=maps[dest]['events']['warps'];arrival=next((r for r in arrivals if r.get('index')==index),None)
   if arrival is None and 0<index<=len(arrivals):arrival=arrivals[index-1]
   if arrival is None:continue
   q=arrival['x'],arrival['y'];v=numbers[dest].get(q)
   if v is not None and wet(v):
    key=(dest,*q);witness=dict(source_map=source,target_map=dest,target_core=list(q),kind='POSSIBLE_WARP_WATER_ARRIVAL');warp_seeds.append(witness)
    if key not in previous:previous[key]=None;seed_witness[key]=witness;todo.append(key)
 script_warp_seeds=[]
 for source,m in maps.items():
  for command in m.get('script_map_commands',[]):
   dest=command.get('target_map');x,y=command.get('x'),command.get('y')
   if dest not in numbers or x is None or y is None:continue
   q=x,y;v=numbers[dest].get(q)
   if v is not None and wet(v):
    key=(dest,*q);summary=dict(source_map=source,target_map=dest,target_core=list(q),kind='POSSIBLE_TYPED_SCRIPT_WATER_ARRIVAL',source_script=command['source_script']);script_warp_seeds.append(summary)
    if key not in previous:previous[key]=None;seed_witness[key]=summary;todo.append(key)
 while todo:
  name,x,y=todo.popleft();point=x,y;v=numbers[name][point]
  for d in DIRS:
   if exit_block(v,d):continue
   other,q,_=target(name,point,d)
   if other not in numbers or q not in numbers[other]:continue
   w=numbers[other][q];key=(other,*q)
   if key in previous or not wet(w)or not enter(w,d):continue
   previous[key]=(name,x,y);todo.append(key)
 def witness(key):
  if key not in previous:return None
  path=[]
  while previous[key]is not None:path.append(dict(map=key[0],core=list(key[1:])));key=previous[key]
  return dict(seed=seed_witness[key],steps=len(path),water_course=list(reversed(path)))
 def classify(name,point,d):
  v=numbers[name][point];other,q,cross=target(name,point,d);w=numbers.get(other,{}).get(q);reached=(name,*point)in previous
  if exit_block(v,d):label='SOURCE_DIRECTION_OR_BRIDGE_AXIS_BLOCKED'
  elif other is None:label='UNCONNECTED_OUTER_WATER_POTENTIALLY_REACHABLE'if reached else'UNCONNECTED_OUTER_WATER_NO_MOUNT_OR_WATER_ROUTE'
  elif w is None:label='CONNECTION_DESTINATION_OUTSIDE_RECTANGLE_BLOCKED'
  elif not enter(w,d):label='CONNECTED_SHORE_BLOCKED_BY_NATIVE_DESTINATION'
  elif wet(w):label='LEGAL_BRIDGE_OR_WATER_SEAM'
  else:label='PASSABLE_CONNECTED_LAND_DESTINATION_NOT_AN_OUTER_ESCAPE'
  forced=None
  if v&0xf0==0x30:forced=['east','west','north','south'][v&3]
  return dict(map=name,core=list(point),direction=d,permission=kinds[name][point],numeric_permission=f'{v:02x}',classification=label,potentially_reachable=reached,neighbor=other,neighbor_core=list(q)if q is not None else None,neighbor_permission=kinds.get(other,{}).get(q),forced_current_direction=forced,opposing_current_input_needs_native_HM_or_current_review=forced is not None and forced!=d,water_route_witness=witness((name,*point))if label.startswith('UNCONNECTED')and reached else None)
 classified=[]
 for row in atlas['exposed_or_nonwater_neighbor_boundary']:classified.append(classify(row['map'],tuple(row['core']),row['direction']))
 assert len(classified)==162
 perimeter=[]
 for name in names:
  a=maps[name]['attributes']
  for d in DIRS:
   for point in boundary(a,d):
    if wet(numbers[name][point]):perimeter.append(classify(name,point,d))
 outer=[r for r in perimeter if r['classification'].startswith('UNCONNECTED')]
 # Inspect all geometrically mountless water components that touch a bare edge.
 components=[];seen=set()
 for row in outer:
  key=(row['map'],*row['core'])
  if key in seen:continue
  local={key};queue=deque([key]);seen.add(key)
  while queue:
   n,x,y=queue.popleft();v=numbers[n][x,y]
   for d in DIRS:
    if exit_block(v,d):continue
    other,q,_=target(n,(x,y),d)
    if other not in numbers or q not in numbers[other]or not wet(numbers[other][q])or not enter(numbers[other][q],d):continue
    k=(other,*q)
    if k in seen:continue
    seen.add(k);local.add(k);queue.append(k)
  components.append(dict(seed=dict(map=key[0],core=list(key[1:])),wet_cells=len(local),potentially_reachable=any(k in previous for k in local),maps=sorted({k[0]for k in local}),touching_outer_lanes=[dict(map=r['map'],core=r['core'],direction=r['direction'])for r in outer if(r['map'],*r['core'])in local]))
 # Summarize callback behavior without exporting source programs/catalogues.
 callback_summary=[]
 for name in sorted({r['map']for r in outer}|{n for n in names if any(v&0xf0==0x30 for v in numbers[n].values())}|{'MnR04C00R00','MnStPaul'}):
  m=maps[name];callbacks=[dict(kind=c['args'][0],script=c['args'][1])for c in m['map_script_section_commands']if c['command']=='callback'and len(c['args'])>=2];ops=Counter(c['command']for commands in m.get('scripts',{}).values()if isinstance(commands,list)for c in commands if isinstance(c,dict)and'command'in c)
  block_changes=[dict(source_script=c['source_script'],script_core_coordinates=[c['x'],c['y']],metatile=[c['x']//2,c['y']//2],before_block=f"{m['blocks'][(c['y']//2)*m['attributes']['width']+c['x']//2]:02x}"if 0<=c['x']//2<m['attributes']['width']and 0<=c['y']//2<m['attributes']['height']else'OUTSIDE',after_block=f"{c['block_id']:02x}")for c in m['script_block_changes']]
  adjacent=Counter()
  for row in outer:
   if row['map']!=name:continue
   for d,(dx,dy)in DIRS.items():
    q=row['core'][0]+dx,row['core'][1]+dy
    if q in kinds[name]and not wet(numbers[name][q]):adjacent[kinds[name][q]]+=1
  callback_summary.append(dict(map=name,callback_headers=callbacks,typed_geometry_edits=block_changes,typed_map_travel_commands=len(m['script_map_commands']),typed_movements=len(m['script_movements']),outer_lane_adjacent_nonwater_permissions=dict(adjacent),script_roots=len(m.get('scripts',{})),geometry_or_travel_command_counts={k:v for k,v in ops.items()if k in {'changeblock','reloadmappart','warp','warpfacing','applymovement','special','surf','waterfall'}},warps_on_water=[dict(core=[e['x'],e['y']],permission=kinds[name].get((e['x'],e['y'])))for e in m['events']['warps']if wet(numbers[name].get((e['x'],e['y']),0xff))]))
 args.output.mkdir(parents=True);result=dict(status='DIRECTIONAL_STATIC_CLASSIFICATION_NOT_CONTROLLER_ESCAPE_PROOF',actual_pack_sha256=PACK_SHA,atlas_sha256=sha(ATLAS.read_bytes()),source_hashes=source_hashes,exteriors=len(names),original_lanes=len(classified),original_classification_counts=dict(Counter(r['classification']for r in classified)),numeric_perimeter_lanes=len(perimeter),numeric_classification_counts=dict(Counter(r['classification']for r in perimeter)),qualified_local_mounts=len(mounts),possible_warp_water_seeds=warp_seeds,possible_typed_script_water_seeds=script_warp_seeds,numeric_perimeter_permissions=dict(Counter(r['permission']for r in perimeter)),bridge_water_plane_quadrants=sum(0xe0<=v<=0xe3 for g in numbers.values()for v in g.values()),current_or_waterfall_quadrants=sum(v&0xf0==0x30 for g in numbers.values()for v in g.values()),reviewer_sha256=sha(Path(__file__).read_bytes()),globally_potentially_reachable_water_tiles=len(previous),all_162_lanes=classified,all_numeric_outer_water=outer,unconnected_outer_components=components,callback_review=callback_summary,limits=['Reachability intentionally over-approximates campaign/actors/ice/current/HM gates; a witness is a candidate for native input, not proof.', 'No-mount/no-water-route components still require checking callback/warp edits before calling permanently inaccessible.', 'Connected native destination Walls and valid land exits are not unconnected map escapes.', 'Waterfall/current direction is recorded; opposing current movement is never claimed executed.', 'Source cameras saved separately are native-art static crops, not reached GPU or gameplay.'])
 (args.output/'classification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(dict(original_counts=result['original_classification_counts'],numeric_counts=result['numeric_classification_counts'],outer_components=components,output=str(args.output))),flush=True)
 # Whole maps and exact native-sized cameras for actionable visible geography.
 scenes=[]
 for name in sorted({r['map']for r in outer}|{'MnR04C00R00','MnStPaul','MnMississippiConfluenceC03R00'}):
  image=native_room(maps[name],pack,native_origin=True);image.save(args.output/f'{name}-actual-whole-native.png');scenes.append((name,image))
 views=[];camera_rows=[]
 for label,name,requested in [('r02-north-drainage','MnR02C00R00',(27,4)),('r02-south-drainage','MnR02C00R00',(27,39)),('r04-north-phantom-canal','MnR04C00R00',(23,10)),('r04-south-phantom-canal','MnR04C00R00',(22,24)),('st-paul-southeast-stem','MnStPaul',(74,64)),('confluence-city-stem','MnMississippiConfluenceC03R00',(2,8))]:
  n=maps[name];a=n['attributes'];points=[q for q,v in numbers[name].items()if v not in wall and not(0xe0<=v<=0xe3 and v in{0xe1,0xe3})];point=min(points,key=lambda q:abs(q[0]-requested[0])+abs(q[1]-requested[1]));image=native_room(n,pack,native_origin=True).convert('RGBA');x,y=point;left=min(max(x*16-72,0),max(0,image.width-160));top=min(max(y*16-64,0),max(0,image.height-144));frame=image.crop((left,top,left+160,top+144));v=numbers[name][point]
  if(name,*point)in previous or not wet(v):frame.alpha_composite(oriented_sprite(pack,'SPRITE_SURF'if wet(v)else'SPRITE_CHRIS','north'),(x*16-left,y*16-top))
  camera_rows.append(dict(label=label,map=name,requested_core=list(requested),actual_center_core=list(point),camera_pixel_origin=[left,top],permission=kinds[name][point],potentially_surf_reachable=(name,*point)in previous));frame.save(args.output/f'{label}-native160.png');frame.resize((640,576),Image.Resampling.NEAREST).save(args.output/f'{label}-4x.png');views.append((label,frame.convert('RGB')))
 room_sheet(views,columns=3).save(args.output/'geography-issues-player-scale-source.png')
 (args.output/'camera-positions.json').write_text(json.dumps(dict(scope='Centered/clamped static native source crops; not actual controller/GPU. No sprite is placed on unreachable water.',views=camera_rows),indent=2)+'\n')
 assert sha(args.pack.read_bytes())==PACK_SHA


if __name__=='__main__':main()
