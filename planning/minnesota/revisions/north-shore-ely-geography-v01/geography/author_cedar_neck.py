#!/usr/bin/env python3
"""Detached hand-authored Cedar/neck native-cell proposal, never modifies a pack."""
import cbor2,copy,hashlib,json,sys
from collections import deque
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[5];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_progress_atlas import native_room
from render_reference import SourceArt
SHA='ebea77ccb9b9dbe54aa5101e32590c1b075ed8130f6e7fcfe2d8c8451dab11d8';raw=(ROOT/'content-packs/minnesota-superior-warm-coast-v01.crystalpack').read_bytes();assert hashlib.sha256(raw).hexdigest()==SHA;pack=cbor2.loads(raw[22:]);base=pack['data']['maps']['MnMaraisCedarApproach'];candidate=copy.deepcopy(base)
OUT=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/cedar-neck-v03';assert not OUT.exists();OUT.mkdir()
SOURCE=ROOT/'planning/minnesota/revisions/north-shore-ely-geography-v01/geography/cedar-neck-v03';assert not SOURCE.exists();SOURCE.mkdir()
def jhash(x):return hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
def put(b,w,x,y,v):b[y*w+x]=v
def path(b,w,pts,v=1):
 for (x,y),(xx,yy)in zip(pts,pts[1:]):
  assert x==xx or y==yy
  for a in range(min(x,xx),max(x,xx)+1):
   for c in range(min(y,yy),max(y,yy)+1):put(b,w,a,c,v)
b=list(base['blocks'])
# Preserve both old trainer clearings and the long existing trail; connect its end
# back uphill to the east port. The former southern coastal port is visibly shut.
path(b,32,[(0,4),(1,4),(1,6),(6,6)])
put(b,32,0,6,5)
path(b,32,[(31,4),(25,4),(25,8),(21,8),(21,12),(27,12),(27,16),(28,16)])
put(b,32,28,17,5);put(b,32,28,18,5)
# Tiny clearings beside the new ridge lane give the entrance a readable bend.
for x,y in [(26,5),(26,6),(24,9),(24,10),(22,10),(22,11),(26,14),(28,15)]:put(b,32,x,y,2)
for x,y in [(25,6),(25,7),(21,10),(21,11)]:put(b,32,x,y,3)
# An optional, fully capped granite overlook climbs by authentic cave stairs.
# The main east/west lane stays below the ridge; the rocky top has one entry.
outline=[(26,6),(28,6),(28,7),(29,7),(29,9),(28,9),(28,10),(26,10)]
mask=Image.new('1',(32,19));ImageDraw.Draw(mask).polygon(outline,fill=1)
def rock(x,y):return 0<=x<32 and 0<=y<19 and mask.getpixel((x,y))
for yy in range(19):
 for xx in range(32):
  if not rock(xx,yy):continue
  n,so,we,e=rock(xx,yy-1),rock(xx,yy+1),rock(xx-1,yy),rock(xx+1,yy)
  v=(0x6a if not n and not we else 0x6b if not n and not e else 0x6c if not so and not we else 0x6d if not so and not e else 0x70 if not n else 0x72 if not so else 0x68 if not we else 0x69 if not e else 0x6e if not rock(xx-1,yy+1) else 0x6f if not rock(xx+1,yy+1) else 0x71)
  put(b,32,xx,yy,v)
put(b,32,27,10,0x81);put(b,32,27,11,1)
# Existing WALKER trainer becomes the optional summit interaction; no new actor,
# party, event flag or trainer/text program is allocated.
assert candidate['objects'][1]['object_identifier']=='MN_COHESION_MARAIS_WALKER'
candidate['objects'][1]['x']=55;candidate['objects'][1]['y']=17
# Snowline is hand stepped through pines, east eight columns stay entirely warm.
def cold(x,y):return x<24 and not(20<=x<24 and y<4)
for y in range(19):
 for x in range(32):
  i=y*32+x;v=b[i]
  if cold(x,y):
   b[i]={1:0x83,2:0x83,3:0x84,4:0x83,5:0x86,0x45:0x88}.get(v,v)
# Three authored deep-pine stands use the second native canopy silhouette.
for x,y in [(3,1),(4,1),(5,1),(3,2),(4,2),(5,2),(8,12),(9,12),(10,12),(8,13),(9,13),(10,13),(18,2),(19,2),(18,3),(19,3)]:
 if b[y*32+x]==0x86:b[y*32+x]=0x87
candidate['blocks']=b;at=candidate['attributes'];at.update(tileset_name='mn_winter_north_shore',border_block=0x86,connections=[dict(direction='west',target_map='MnMaraisInlandNeck',offset=0),dict(direction='east',target_map='MnGrandMarais',offset=0)],connection_flags='WEST | EAST')
# The only existing authored text changes are the two now-reversed directions.
text_changes=[dict(label='MN_COHESION_MARAIS_SIGNText',command_index=1,arg_index=0,before='"GRAND MARAIS west."',after='"GRAND MARAIS east."'),dict(label='MN_COHESION_MARAIS_SIGNText',command_index=2,arg_index=0,before='"NORTH SHORE south."',after='"GUNFLINT west."')]
# Rendering uses exact unchanged programs; publication separately applies these two args.
neckblocks=[0x86]*(8*19)
path(neckblocks,8,[(0,4),(1,4),(1,7),(5,7),(5,4),(7,4)],0x83)
for x,y in [(2,7),(3,7),(5,5)]:put(neckblocks,8,x,y,0x84)
for x,y in [(2,2),(3,2),(2,3),(3,3),(6,12),(6,13),(5,12),(5,13)]:put(neckblocks,8,x,y,0x87)
# No hidden floor pocket or house glyph in this narrow connector.
neck=copy.deepcopy(candidate);neck['id']='MnMaraisInlandNeck';neck['blocks']=neckblocks;neck['objects']=[];neck['events']={k:[]for k in base['events']};neck['attributes']=copy.deepcopy(at);neck['attributes'].update(width=8,height=19,map_constant='MN_MARAIS_INLAND_NECK',map_group_constant='MAPGROUP_MINNESOTA',blocks_label='MnMaraisInlandNeck_Blocks',map_scripts_label='MnMaraisInlandNeck_MapScripts',map_events_label='MnMaraisInlandNeck_MapEvents',connections=[dict(direction='west',target_map='MnR12C01R01',offset=0),dict(direction='east',target_map='MnMaraisCedarApproach',offset=0)])
# Any empty transport headers must be assembled by native authoring::finish, not copied.
for key in list(neck):
 if key.startswith('script_')or key=='scripts':neck[key]=[]if isinstance(neck[key],list)else{}
collision=pack['data']['tilesets']['mn_winter_north_shore']['collision']
def physical(m):
 w=m['attributes']['width'];h=m['attributes']['height'];bs=m['blocks'];occupied={(o['x'],o['y'])for o in m['objects']}
 def k(x,y):return collision[f'{bs[y//2*w+x//2]:02x}'][y%2*2+x%2]
 allowed={(x,y)for y in range(h*2)for x in range(w*2)if k(x,y)in{'FLOOR','TALL_GRASS'}and(x,y)not in occupied};start=(0,8);end=(w*2-1,8);q=deque([start]);dist={start:0}
 while q:
  x,y=q.popleft()
  for p in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
   if p in allowed and p not in dist:dist[p]=dist[x,y]+1;q.append(p)
 outer=sorted(p for p in allowed if p[0]in{0,w*2-1}or p[1]in{0,h*2-1});expected=sorted([(0,8),(0,9),(w*2-1,8),(w*2-1,9)])
 actorapproach={o['object_identifier']:any((o['x']+dx,o['y']+dy)in dist for dx,dy in[(1,0),(-1,0),(0,1),(0,-1)])for o in m['objects']}
 assert end in dist and outer==expected and all(actorapproach.values())
 return dict(west_to_east_steps=dist[end],reachable_tiles=len(dist),walkable_tiles=len(allowed),outer_core_lanes=outer,actor_approaches=actorapproach,water_quadrants=sum(k(x,y).startswith('WATER')for y in range(h*2)for x in range(w*2)),ice_quadrants=sum(k(x,y)=='ICE'for y in range(h*2)for x in range(w*2)))
checks=dict(cedar=physical(candidate),neck=physical(neck));checks['cedar_events_programs_preserved']=all(candidate[k]==base[k]for k in base if k not in{'attributes','blocks','objects'});assert checks['cedar_events_programs_preserved']
art=SourceArt(pack)
for label,m in [('Cedar',candidate),('Neck',neck)]:
 im=art.render('mn_winter_north_shore',m['attributes']['width'],19,m['blocks']);im.save(OUT/f'{label}-candidate-whole-native.png')
# Actual source actor overlay on the existing Cedar module.
native_room(candidate,pack,native_origin=True).convert('RGB').save(OUT/'Cedar-candidate-actors-native.png')
combined=Image.new('RGB',(40*32,19*32),'#152332');combined.paste(art.render('mn_winter_north_shore',8,19,neckblocks),(0,0));combined.paste(native_room(candidate,pack,native_origin=True).convert('RGB'),(8*32,0));combined.save(OUT/'neck-cedar-connected-native.png')
changes=[dict(metatile=[i%32,i//32],before=f'{old:02x}',after=f'{new:02x}')for i,(old,new)in enumerate(zip(base['blocks'],b))if old!=new]
contract=dict(schema_version=1,status='DETACHED CANDIDATE; not native publication or controller acceptance',parent_pack_sha256=SHA,parent_content_hash=pack['identity']['content_hash'],parent_cedar_module_sha256=jhash(base),render_tileset='mn_winter_north_shore',cedar=dict(id=base['id'],dimensions=[32,19],proposed_origin=[170,-419],attributes=at,changes=changes,rows=[' '.join(f'{v:02x}'for v in b[y*32:(y+1)*32])for y in range(19)],preserve_fields='all fields except attributes, blocks, WALKER trainer position only, and the two declared sign text arguments plus their native global/cache mirrors',actor_position_changes=[dict(id='MN_COHESION_MARAIS_WALKER',before=[53,31],after=[55,17])],text_changes=text_changes,warm_east_buffer_meta_x=[24,31],cold_extent_note='Pine-stepped inland snowline at x24, with warm x20..23 northern stand'),neck=dict(id='MnMaraisInlandNeck',constant='MN_MARAIS_INLAND_NECK',group_name='MINNESOTA',group_id=40,map_id=8,dimensions=[8,19],proposed_origin=[162,-419],attributes=neck['attributes'],rows=[' '.join(f'{v:02x}'for v in neckblocks[y*8:(y+1)*8])for y in range(19)],events=neck['events'],objects=[],programs='No authored actor, text, runtime, map callback or encounter program. Use ordinary native empty script/event transport headers and authoring::finish.',wild_encounters=[],temporary_registration_spawn=dict(identifier=280,core=[0,8],remove_after_insertion=True),landmark='LANDMARK_MN_GRAND_MARAIS'),physical=checks,registry_changes=['maps new MnMaraisInlandNeck; existing Cedar blocks/attributes plus WALKER position and two sign direction arguments','map_attributes, map_dimensions MN_MARAIS_INLAND_NECK, runtime_map_metadata MINNESOTA group40 map8','new map_blocks label MnMaraisInlandNeck_Blocks; exact modified Cedar block mirror','new empty script/event transport headers via native compiler; only two existing Cedar sign args and necessary mirrors','npcs empty for neck; Cedar WALKER only coordinate pair53,31→55,17; all other NPC fields exact','pokegear map_to_landmark neck→LANDMARK_MN_GRAND_MARAIS, existing landmark definition unchanged','report maps+1, one fresh manifest/reachable/solvable entry; no retained recovery or game-start mutation','runtime_files, tileset definitions, audio, species, moves, trainers and unrelated maps exact'],limitations=['Static Foot BFS includes native Floor/TallGrass and rejects actors, but does not substitute for production ledge/stair/camera traversal.','Final contract must be rebound to actual additive Ely parent SHA and registration maxspawn279.','All changed town and R11 seam terrain are owned by town author, not included here.'])
(SOURCE/'contract.json').write_text(json.dumps(contract,indent=2)+'\n');(OUT/'physical-proof.json').write_text(json.dumps(checks,indent=2)+'\n');print(json.dumps(dict(changes=len(changes),checks=checks,contract_sha256=hashlib.sha256((SOURCE/'contract.json').read_bytes()).hexdigest())))
