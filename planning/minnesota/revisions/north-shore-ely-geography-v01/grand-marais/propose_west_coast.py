"""Hand-authored WEST coast arrival and separate NORTH inland departure."""
from pathlib import Path
from copy import deepcopy
from collections import deque
import sys,json,hashlib,cbor2,time
from PIL import Image,ImageChops
ROOT=Path(__file__).resolve().parents[5];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt,room_sheet,layout_blocks
from review_major_city import ReviewArt,render_map,player_camera
from render_archive import save_render
sha=lambda b:hashlib.sha256(b).hexdigest()
canon=lambda v:json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
write=lambda p,v:p.write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
started=time.perf_counter();folder=Path(__file__).resolve().parent
out=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/grand-marais'/sys.argv[1];out.mkdir(parents=True,exist_ok=False)
raw=(ROOT/'content-packs/minnesota-superior-inland-art-v01.crystalpack').read_bytes();assert sha(raw)=='1c6ba5d59097546ff8b94e09c809d270727ce1272acb93759b89f3b3c7d9b90a'
base=cbor2.loads(raw[22:]);m=base['data']['maps']['MnGrandMarais'];assert sha(canon(m))=='377210a771f41efaa9c5bd18a9035462a61a14da7e34e0967d79421b5e6c93c1'
TS='mn_marais_shore';preview=deepcopy(base);definition=deepcopy(base['data']['tilesets']['johto']);collision=definition['collision'];oldraw=bytes(base['runtime_files']['data/tilesets/johto_metatiles.bin']);newraw=bytearray(oldraw);assert len(oldraw)==128*16 and TS not in base['data']['tilesets']
reference=json.loads((ROOT/'planning/minnesota/towns/MnGrandMarais.connections.json').read_bytes());recipes=[]
for item in reference['reef_recipes']:
 b=int(item['base'],16);glyphs=bytearray(oldraw[b*16:(b+1)*16]);tokens=list(collision[item['base']]);assert int(item['id'],16)*16==len(newraw)
 for q in item['quadrants']:
  index=q['index'];donor=int(q['source'],16);assert tokens[index]=='WATER' and collision[q['source']][index]=='BUOY'
  for dy in range(2):
   for dx in range(2):
    t=(index//2*2+dy)*4+index%2*2+dx;glyphs[t]=oldraw[donor*16+t]
  tokens[index]='BUOY'
 newraw.extend(glyphs);collision[item['id']]=tokens;recipes.append(dict(item,glyphs=list(glyphs),collision=tokens))
preview['data']['tilesets'][TS]=definition
for suffix in ['.2bpp','.png']:preview['runtime_files'][f'gfx/tilesets/{TS}{suffix}']=base['runtime_files'][f'gfx/tilesets/johto{suffix}']
preview['runtime_files'][f'data/tilesets/{TS}_metatiles.bin']=bytes(newraw)
preview['runtime_files'][f'data/tilesets/{TS}.json']=canon(collision);preview['runtime_files'][f'data/tilesets/{TS}_palette_map.json']=canon(definition['palette_map'])
blocks=list(m['blocks']);purposes={}
def place(x,y,new,purpose):
 i=y*20+x
 if blocks[i]!=new:blocks[i]=new;purposes[x,y]=purpose
# Keep the real west regional entry. Close the retired east exit visibly.
place(19,6,0x05,'close former east Foot exit with ordinary native pines')
for y in range(19):
 b=blocks[y*20+19]
 if any(collision[f'{b:02x}'][q].startswith('WATER')for q in [1,3]):
  new={0x54:0x80,0x58:0x81,0x79:0x85,0x35:0x86}.get(b);assert new is not None,(y,b)
  place(19,y,new,'native reefs close unmatched east water boundary')
# Clear an actual inland lane between the existing Center and Mart, never roofs.
for y in range(6):place(9,y,0x01,'north inland lane between Center and Mart; green coastal town ends at this port')
# Move the west lookout bank one metatile east, preserving its inner Floor lane.
# This creates three native Water columns for safe gated Surf scripts; there is
# no forced Surf move onto Floor and no new south quay.
for y in range(8,13):place(0,y,0x35,'open Superior water outside the shifted west lookout bank')
for y,b in [(8,0x55),(9,0x59),(10,0x59),(11,0x59),(12,0x7a)]:place(1,y,b,'shift existing west bank inward; retain inner lookout walking lane and native shoreline corners')
attrs=deepcopy(m['attributes']);attrs['tileset_name']=TS;attrs['connections']=[dict(direction='north',target_map='MnMaraisCedarApproach',offset=-19),dict(direction='west',target_map='MnR11C01R00',offset=0)];attrs['connection_flags']='NORTH | WEST';candidate=dict(m,attributes=attrs,blocks=blocks)
def objects(module):
 return [dict(id=o.get('object_identifier')or o['script'],sprite=o['sprite'],tile=[o['x'],o['y']],facing={'UP':'north','DOWN':'south','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south'),native_palette_override=o.get('pal',0))for o in module['objects']]
before=objects(m);closed=deepcopy(before);relocations=[]
for o in closed:
 if o['id'].startswith('MNCOHESIONMARAISEAST_'):
  prior=list(o['tile']);o['tile']=[18 if o['id'].endswith('_A')else 19,2];o['facing']='south';relocations.append(dict(id=o['id'],before_core=prior,after_core=o['tile'],after_movement='SPRITEMOVEDATA_STANDING_DOWN'))
opened=[o for o in closed if not o['id'].startswith('MNCOHESIONMARAIS')]
kind=lambda x,y:collision[f'{blocks[y//2*20+x//2]:02x}'][y%2*2+x%2]
foot={'FLOOR','TALL_GRASS','LONG_GRASS','DOOR'};water=lambda p:kind(*p).startswith('WATER')
assert [y for y in range(38)if water((0,y))]==list(range(16,36))
assert [x for x in range(40)if kind(x,0)in foot]==[18,19]
assert not any(water((39,y))for y in range(38)) and not any(water((x,37))for x in range(40))
water_out=[(2,y)for y in range(38)if water((2,y))];water_in=[(1,y)for y in range(38)if water((1,y))]
assert water_out==[(2,y)for y in range(16,37)] and water_in==[(1,y)for y in range(16,37)]
assert all(water((x,y))for _,y in water_out for x in [1,2,3])
regular={tuple(o['tile'])for o in opened};guards={tuple(o['tile'])for o in closed if o['id'].startswith('MNCOHESIONMARAIS')}
def reach(start,gate_closed=False,allow_inward=True,allow_water=True):
 occupied=regular|(guards if gate_closed else set());barrier=set(water_out)if gate_closed else set();seen={start};todo=deque([start])
 while todo:
  x,y=todo.popleft();adj=[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]
  if gate_closed and allow_inward:
   if x==1 and(y in [8,9]or(x,y)in water_in):adj.append((3,y))
   if y==1 and x in [18,19]:adj.append((x,3))
  for q in adj:
   if not(0<=q[0]<40 and 0<=q[1]<38)or q in occupied or q in barrier:continue
   k=kind(*q)
   if k not in foot and not(allow_water and k.startswith('WATER')):continue
   if q not in seen:seen.add(q);todo.append(q)
 return seen
ports=[(0,8),(0,9),(18,0),(19,0)]+[(0,y)for y in range(16,36)]
cr=reach((15,11),True);op=reach((15,11),False);assert not any(p in cr for p in ports);assert all(p in op for p in ports)
for entry in [(0,8),(18,0),(0,20)]:assert(15,11)in reach(entry,True),entry
doors=[]
for warp in m['events']['warps']:
 x,y=warp['x'],warp['y'];approach=(x,y+1);assert approach in reach((0,8),True,True,False) and approach in reach((18,0),True,True,False)
 doors.append(dict(index=warp['index'],core=[x,y],approach_core=list(approach),target=warp['target_map'],unchanged=True))
for o in opened:assert kind(*o['tile'])in foot,o
for o in closed:
 if o['id'].startswith('MNCOHESIONMARAIS'):assert any(q in cr for q in [(o['tile'][0]-1,o['tile'][1]),(o['tile'][0]+1,o['tile'][1]),(o['tile'][0],o['tile'][1]-1),(o['tile'][0],o['tile'][1]+1)]),o
patches=[dict(metatile=[i%20,i//20],before=f'{a:02x}',after=f'{b:02x}',purpose=purposes[i%20,i//20])for i,(a,b)in enumerate(zip(m['blocks'],blocks))if a!=b]
proof=dict(status='SOURCE_GRAPH_WITH_PLANNED_NATIVE_GATE_EDGES_NOT_CONTROLLER_EXECUTION',west_foot_core=[[0,8],[0,9]],west_water_core=[[0,y]for y in range(16,36)],north_inland_core=[[18,0],[19,0]],closed_all_foot_surf_outlets_unreachable=True,open_all_outlets_reachable=True,unqualified_foot_both_entries_and_surf_can_reach_clinic=True,all6_original_door_approaches_reachable_on_foot_after_both_inward_entries=True,water_gate_out_core=water_out,water_gate_in_core=water_in,water_inward_and_denial_positions_all_water=True,water_out_events_run_existing_west_check_regardless_facing='Prevents mounting directly into gate column from the northern bank; denial alwaysRIGHT to safeWater column3.',incidental_cut_quadrants=sum(k=='CUT_TREE'for b in blocks for k in collision[f'{b:02x}']),changed_town_metatiles=len(patches),new_old_quadrant_metatiles=10,new_glyphs=0,south_quay_superseded=True)
write(out/'geometry-proof.json',proof)
contract=dict(schema_version=2,status='WEST_ENTRY_SOURCE_PROPOSAL_NOT_NATIVE',base_pack_sha256=sha(raw),base_identity=base['identity']['content_hash'],map='MnGrandMarais',base_module_sha256=sha(canon(m)),dimensions_metatiles=[20,19],world_origin_proposed=[246,-400],native_alias=TS,behavior_source='johto',native_art_recipes=recipes,cells=patches,attributes_before=m['attributes'],attributes_after=attrs,guards=relocations,preserved_doors=doors,preserved_rooms_sha256={k:sha(canon(v))for k,v in base['data']['maps'].items()if k.startswith('MnGrandMarais')and k!='MnGrandMarais'},west_gate_programs='Preserve installed west programs, bodies2,8/9, outbound3/inward1 and flag predicates exactly; now the coast entry.',north_rotation=dict(old_root='MnCohesionMaraisEast',guards_core=[[18,2],[19,2]],outbound_land_core=[[18,3],[19,3]],inward_land_core=[[18,1],[19,1]],argument_edits=[dict(label='MnCohesionMaraisEastInward',command_index=1,arg_index=0,before='RIGHT',after='UP'),dict(label='MnCohesionMaraisEastOneStep',command_index=0,arg_index=0,before='LEFT',after='DOWN')]+[dict(label='MnCohesionMaraisEastTwoSteps',command_index=i,arg_index=0,before='LEFT',after='DOWN')for i in [0,1]]),water_gate=dict(outbound_core=water_out,outbound_existing_script='MnCohesionMaraisWest',outbound_any_facing=True,inward_core=water_in,inward_wrapper=dict(label='MnMaraisCoastWaterIn',native_commands=[dict(command='readvar',args=['VAR_FACING']),dict(command='ifequal',args=['RIGHT','MnCohesionMaraisWestInward']),dict(command='end',args=[])]),native_mode='Allpositions1→2→3 and denial2→3 areWater; existing native movement must preserveSurf mode in production tests.'),wayfinding_sign=dict(label='MnGrandMaraisSign1Text',core=[12,14],before_lines=['HARBOR PARK','FOLLOW SHORE EAST'],after_lines=['SUPERIOR HARBOR','NORTH SHORE: WEST','GUNFLINT: NORTH','ELY: INLAND SNOW']),shared_geometry='west-entry-placement-v02: GM246,-400; Cedar227,-419; N offsets-19/+19, W offsets0/0. R11terminal headland source is owned by river_native and must supply real Floor+Water counterpart apertures before publication.',canonical_and_world_writes=0,all_other_native_programs_services_quests_warps_and_rooms='exactly preserved')
write(out/'native-edit.contract.json',contract);write(out/'compiled-candidate.blocks.json',dict(map='MnGrandMarais',tileset=TS,dimensions_metatiles=[20,19],blocks=blocks))
layout_path=ROOT/'planning/minnesota/connections/grand-marais-harbor-closure-v01/MnGrandMarais.candidate.layout.json';layout=json.loads(layout_path.read_bytes());assert layout_blocks(layout)==m['blocks'];layout['id']='MnGrandMarais';layout['compiled_tileset']=TS
for p in patches:layout['layers'].append(dict(kind='point',name=p['purpose'],position=p['metatile'],block=p['after']))
assert layout_blocks(layout)==blocks;write(out/'MnGrandMarais.layout.candidate.json',layout)
wholes=[]
for name,p,local,actors in [('actual-warm-parent',base,m,before),('west-north-proposal-closed',preview,candidate,closed),('west-north-proposal-open-fixture',preview,candidate,opened)]:
 art=ReviewArt(p);ground,scene,_=render_map(art,local,actors,'day',out/name);wholes.append((name,scene));frames=[]
 for label,core in [('services',[15,11]),('west-arrival',[4,8]),('lighthouse',[15,28])]+([]if name=='actual-warm-parent'else[('north-inland-lane',[18,4]),('west-lookout',[4,20]),('north-entry',[18,0])]):
  x,y=core;cols=p['data']['tilesets'][local['attributes']['tileset_name']]['collision'];b=local['blocks'][y//2*20+x//2];assert cols[f'{b:02x}'][y%2*2+x%2]in foot and core not in[o['tile']for o in actors]
  frame,_=player_camera(art,local,actors,ground,scene,'day',dict(core=core),out/name/'views'/label);frames.append((label+' '+str(core),frame))
 save_render(room_sheet(frames,3),out/name/'players.png')
save_render(room_sheet(wholes,3),out/'whole-before-west-north-closed-open.png')
artframes=[]
for bank in ['morn','day','nite','dark']:
 old=SourceArt(base).blocks('johto',bank);new=SourceArt(preview).blocks(TS,bank);assert len(new)==138
 for a,b in zip(old,new):assert ImageChops.difference(a,b).getbbox()is None
 for recipe in recipes:
  expected=old[int(recipe['base'],16)].copy()
  for q in recipe['quadrants']:
   x=q['index']%2*16;y=q['index']//2*16;expected.paste(old[int(q['source'],16)].crop((x,y,x+16,y+16)),(x,y))
  actual=new[int(recipe['id'],16)];assert ImageChops.difference(expected,actual).getbbox()is None;artframes.append((bank+' '+recipe['id'],actual))
save_render(room_sheet(artframes,10),out/'four-bank-native-reef-recipes.png')
write(out/'source-art-proof.json',dict(old_pixels=128*1024*4,new_recipe_pixels=10*1024*4,different_pixels=0,new_source_glyphs=0,source_only_not_gpu=True))
# Exact neighbor authoring, never invented connecting terrain.
headpath=folder.parent/'geography/west-r11-headland-v01/contract.json'
head=json.loads(headpath.read_bytes());joined_scenes=[];joined_cameras=[];seam=[]
for state,town_actors in [('closed',closed),('open-fixture',opened)]:
 joined=Image.new('RGBA',(76*32,24*32),'#14202b');review=ReviewArt(preview)
 for change in head['changes']:
  original=base['data']['maps'][change['map']];assert sha(canon(original))==change['native_module_sha256']
  local=deepcopy(original);w=local['attributes']['width']
  for cell in change['cells']:
   x,y=cell['metatile'];i=y*w+x;assert local['blocks'][i]==int(cell['before'],16);local['blocks'][i]=int(cell['after'],16)
  ground,scene,_=render_map(review,local,objects(local),'day',out/'joined'/state/change['map'])
  joined.alpha_composite(scene,(0 if change['map']=='MnR11C00R00'else 32*32,0))
  if change['map']=='MnR11C01R00':
   c=preview['data']['tilesets'][local['attributes']['tileset_name']]['collision']
   for y in [8,9]+list(range(16,36)):
    token=c[f"{local['blocks'][y//2*w+23]:02x}"][y%2*2+1];town=kind(0,y)
    assert token==town and token==('FLOOR'if y in[8,9]else'WATER'),(y,token,town)
    if state=='closed':seam.append(dict(r11_core=[47,y],town_core=[0,y],collision=token))
 ground,scene,_=render_map(review,candidate,town_actors,'day',out/'joined'/state/'MnGrandMarais')
 joined.alpha_composite(scene,(56*32,0));save_render(joined,out/f'joined-coast-{state}-native.png');joined_scenes.append((state,joined))
 for label,core,sprite in [('west-foot-arrival',[111,8],'SPRITE_CHRIS'),('superior-surf-seam',[111,20],'SPRITE_SURF')]:
  whole=joined.copy();whole.alpha_composite(review.sprite(sprite,'east','day'),(core[0]*16,core[1]*16));px=core[0]*16;py=core[1]*16
  camera=whole.crop((px-80,py-72,px+80,py+72));save_render(camera,out/f'{state}-{label}-native160.png');joined_cameras.append((state+' '+label,camera))
save_render(room_sheet(joined_cameras,2),out/'joined-coast-seam-players.png')
write(out/'paired-seam-proof.json',dict(status='SOURCE_COLLISION_PAIRS_NOT_CONTROLLER_EXECUTION',counterpart_contract_sha256=sha(headpath.read_bytes()),paired_west_lanes=seam,north_town_core=[[18,0],[19,0]],cedar_south_core=[[56,37],[57,37]],north_offsets=[-19,19],source_only=True))
write(out/'render-loop.json',dict(elapsed_seconds=round(time.perf_counter()-started,3),source_only=True,pack_written=False,cargo_executed=False,native_contract_sha256=sha((out/'native-edit.contract.json').read_bytes())))
print('render',out,'cells',len(patches),'contract',sha((out/'native-edit.contract.json').read_bytes()))
