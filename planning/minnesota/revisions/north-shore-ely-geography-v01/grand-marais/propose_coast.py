"""Hand-authored Grand Marais coast proposal; source previews only, no pack writes."""
from pathlib import Path
from copy import deepcopy
from collections import deque
import sys,json,hashlib,cbor2,time
from PIL import Image,ImageChops
ROOT=Path(__file__).resolve().parents[5];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt,room_sheet,layout_blocks
from review_major_city import ReviewArt,render_map,player_camera
from review_north_shore import oriented_sprite
from render_archive import save_render
sha=lambda b:hashlib.sha256(b).hexdigest()
canonical=lambda v:json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
write=lambda p,v:p.write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
started=time.perf_counter()
folder=Path(__file__).resolve().parent
out=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/grand-marais'/sys.argv[1];out.mkdir(parents=True,exist_ok=False)
raw=(ROOT/'content-packs/minnesota-superior-warm-coast-v01.crystalpack').read_bytes()
assert sha(raw)=='ebea77ccb9b9dbe54aa5101e32590c1b075ed8130f6e7fcfe2d8c8451dab11d8'
base=cbor2.loads(raw[22:]);assert base['identity']['content_hash']=='5009f2898563ecf699d7036b19d988ef5f4d43a8b39cbd48f8616cc9c45eca32'
m=base['data']['maps']['MnGrandMarais'];assert sha(canonical(m))=='377210a771f41efaa9c5bd18a9035462a61a14da7e34e0967d79421b5e6c93c1'
TS='mn_marais_shore';preview=deepcopy(base);original=bytes(base['runtime_files']['data/tilesets/johto_metatiles.bin']);assert len(original)==128*16
collision=deepcopy(base['data']['tilesets']['johto']['collision']);definition=deepcopy(base['data']['tilesets']['johto']);metatiles=bytearray(original)
source=json.loads((ROOT/'planning/minnesota/towns/MnGrandMarais.connections.json').read_bytes())
recipes=[]
for item in source['reef_recipes']:
 recipes.append(dict(id=item['id'],base=item['base'],quadrants=item['quadrants'],role='existing reviewed native reef recipe'))
recipes.extend([
 dict(id='8a',base='35',quadrants=[dict(index=q,source='46')for q in [1,3]],role='quay west rail: water outside, blocking native fence inside'),
 dict(id='8b',base='35',quadrants=[dict(index=q,source='44')for q in [0,2]],role='quay east rail: blocking native fence inside, water outside'),
 dict(id='8c',base='35',quadrants=[dict(index=1,source='46'),dict(index=3,source='46'),dict(index=2,source='39')],role='west quay foot cap; unmatched south outer-water quadrant becomes native BUOY'),
 dict(id='8d',base='35',quadrants=[dict(index=0,source='44'),dict(index=2,source='44'),dict(index=3,source='39')],role='east quay foot cap; unmatched south outer-water quadrant becomes native BUOY'),
 dict(id='8e',base='55',quadrants=[dict(index=q,source='34')for q in [0,2]],role='preserve the native northwest harbor corner while closing its west-facing water')])
for recipe in recipes:
 i=int(recipe['id'],16);b=int(recipe['base'],16);assert i*16==len(metatiles)
 glyphs=bytearray(original[b*16:(b+1)*16]);tokens=list(collision[recipe['base']])
 for quadrant in recipe['quadrants']:
  q=quadrant['index'];donor=int(quadrant['source'],16)
  for dy in range(2):
   for dx in range(2):
    t=(q//2*2+dy)*4+q%2*2+dx;glyphs[t]=original[donor*16+t]
  tokens[q]=collision[quadrant['source']][q]
 recipe['glyphs']=list(glyphs);recipe['collision']=tokens;metatiles.extend(glyphs);collision[recipe['id']]=tokens
existing_alias=base['data']['tilesets'].get(TS)
assert existing_alias is None,'inspect any already installed shore alias before replacing it'
definition['collision']=collision;preview['data']['tilesets'][TS]=definition
for suffix in ['.2bpp','.png']:
 preview['runtime_files'][f'gfx/tilesets/{TS}{suffix}']=base['runtime_files'][f'gfx/tilesets/johto{suffix}']
preview['runtime_files'][f'data/tilesets/{TS}_metatiles.bin']=bytes(metatiles)
preview['runtime_files'][f'data/tilesets/{TS}.json']=canonical(collision)
preview['runtime_files'][f'data/tilesets/{TS}_palette_map.json']=canonical(definition['palette_map'])
blocks=list(m['blocks']);purpose={}
def place(x,y,value,reason):
 i=y*20+x
 if blocks[i]!=value:blocks[i]=value;purpose[(x,y)]=reason
for y in range(19):
 b=blocks[y*20]
 if any(collision[f'{b:02x}'][q].startswith('WATER')for q in [0,2]):
  replacement={0x55:0x8e,0x59:0x82,0x7a:0x83,0x35:0x84}.get(b);assert replacement is not None,(y,b);place(0,y,replacement,'close unmatched west harbor water beside the inland-only neighbor')
 b=blocks[y*20+19]
 if any(collision[f'{b:02x}'][q].startswith('WATER')for q in [1,3]):
  replacement={0x54:0x80,0x58:0x81,0x79:0x85,0x35:0x86}.get(b);assert replacement is not None,(y,b);place(19,y,replacement,'close unmatched east Superior boundary after removing Cedar east detour')
place(19,6,0x05,'close obsolete east foot port visibly with ordinary native pine canopy')
for x in range(12,19):place(x,18,0x35,'open the broad same-Lake-Superior seam to actual R11 north water')
for y in range(15,19):
 place(7,y,0x8c if y==18 else 0x8a,'native fenced west edge of narrow coastal quay')
 place(8,y,0x01,'stone quay links lighthouse headland directly to the R11 coastal road')
 place(9,y,0x8d if y==18 else 0x8b,'native fenced east edge of narrow coastal quay')
attrs=deepcopy(m['attributes']);attrs['tileset_name']=TS;attrs['connections']=[dict(direction='south',target_map='MnR11C00R00',offset=-12),dict(direction='west',target_map='MnMaraisCedarApproach',offset=0)];attrs['connection_flags']='SOUTH | WEST'
candidate=dict(m,attributes=attrs,blocks=blocks)
def objects(module):
 return [dict(id=o.get('object_identifier')or o['script'],sprite=o['sprite'],tile=[o['x'],o['y']],facing={'UP':'north','DOWN':'south','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south'),native_palette_override=o.get('pal',0))for o in module['objects']]
before_actors=objects(m);closed=deepcopy(before_actors)
relocations=[]
for o in closed:
 if o['id']in ['MNCOHESIONMARAISEAST_A','MNCOHESIONMARAISEAST_B']:
  old=list(o['tile']);o['tile']=[16 if o['id'].endswith('_A')else 17,35];o['facing']='north';relocations.append(dict(id=o['id'],before_core=old,after_core=o['tile'],after_movement='SPRITEMOVEDATA_STANDING_UP'))
open_actors=[o for o in closed if not o['id'].startswith('MNCOHESIONMARAIS')]
kind=lambda x,y:collision[f'{blocks[y//2*20+x//2]:02x}'][y%2*2+x%2]
assert all(kind(x,37)=='FLOOR'for x in [16,17]);assert [x for x in range(40)if kind(x,37).startswith('WATER')]==list(range(24,38))
water_out=[(x,35)for x in range(40)if kind(x,35).startswith('WATER')];water_in=[(x,36)for x in range(40)if kind(x,36).startswith('WATER')]
assert len(water_out)==34 and len(water_in)==34
foot={'FLOOR','TALL_GRASS','LONG_GRASS','DOOR'};regular_occupied={tuple(o['tile'])for o in open_actors};closed_occupied=regular_occupied|{tuple(o['tile'])for o in closed if o['id'].startswith('MNCOHESIONMARAIS')}
def reachable(start,closed_gate=False,inward=False,water=True):
 blocked=closed_occupied if closed_gate else regular_occupied
 barrier=set(water_out)if closed_gate else set();seen={start};todo=deque([start])
 while todo:
  x,y=todo.popleft();adjacent=[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]
  if closed_gate and inward and y==36 and(x in [16,17]or(x,y)in water_in):adjacent.append((x,34))
  for q in adjacent:
   if not(0<=q[0]<40 and 0<=q[1]<38)or q in blocked or q in barrier:continue
   k=kind(*q)
   if k not in foot and not(water and k.startswith('WATER')):continue
   if q not in seen:seen.add(q);todo.append(q)
 return seen
land_ports=[(16,37),(17,37)];sea_ports=[(x,37)for x in range(24,38)];outlets=land_ports+sea_ports
closed_reach=reachable((15,11),True,True);opened_reach=reachable((15,11),False)
assert not any(p in closed_reach for p in outlets);assert all(p in opened_reach for p in outlets)
assert (15,11)in reachable((16,37),True,True) and(15,11)in reachable((24,37),True,True)
doors=[]
for w in m['events']['warps']:
 x,y=w['x'],w['y'];approach=(x,y+1);assert kind(*approach)in foot and approach in reachable((16,37),True,True,False)
 doors.append(dict(index=w['index'],core=[x,y],approach_core=list(approach),target=w['target_map'],unchanged=True))
for y in [35,36,37]:
 for x in range(40):
  if kind(x,y)in foot:
   assert all(not(0<=xx<40 and 0<=yy<38)or not kind(xx,yy).startswith('WATER')for xx,yy in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]),'no Surf launch below checkpoint'
edges=dict(north=[x for x in range(40)if kind(x,0).startswith('WATER')],south=[x for x in range(40)if kind(x,37).startswith('WATER')],west=[y for y in range(38)if kind(0,y).startswith('WATER')],east=[y for y in range(38)if kind(39,y).startswith('WATER')]);assert edges==dict(north=[],south=list(range(24,38)),west=[],east=[])
actual_r11=base['data']['maps']['MnR11C00R00'];r11=deepcopy(actual_r11);r11_patches=[dict(metatile=[x,0],before=f"{r11['blocks'][x]:02x}",after='58'if x==24 else'35')for x in range(24,31)]
for cell in r11_patches:r11['blocks'][cell['metatile'][0]]=int(cell['after'],16)
rk=lambda x,y:base['data']['tilesets'][r11['attributes']['tileset_name']]['collision'][f"{r11['blocks'][y//2*32+x//2]:02x}"][y%2*2+x%2]
assert all(rk(x,0)=='FLOOR'for x in [40,41]);assert all(rk(x,0).startswith('WATER')for x in range(48,62))
assert all(kind(x,34).startswith('WATER')and kind(x,35).startswith('WATER')for x,_ in water_out)
assert all(kind(x,y).startswith('WATER')for x,_ in water_in for y in [34,35,36])
proof=dict(status='DETACHED_SOURCE_GEOMETRY_AND_PLANNED_GATE_GRAPH_NOT_NATIVE_EXECUTION',parent_pack_sha256=sha(raw),native_town_module_sha256=sha(canonical(m)),new_alias=TS,new_metatiles=15,new_glyphs=0,changed_metatiles=sum(a!=b for a,b in zip(m['blocks'],blocks)),closed_foot_and_surf_have_zero_south_port_reachability=True,open_foot_and_surf_reach_all_declared_ports=True,closed_gate_free_inward_foot_and_surf_to_clinic=True,all_six_original_door_approaches_reachable_on_foot_after_inward=True,water_outbound_gate_core=water_out,water_inward_gate_core=water_in,unmatched_water_edges=edges,incidental_cut_quadrants=sum(k=='CUT_TREE'for b in blocks for k in collision[f'{b:02x}']),source_only=True,pack_written=False)
proof['paired_r11_source_ports']=dict(owner='river_native',native_module_before_sha256=sha(canonical(actual_r11)),required_source_cells=r11_patches,foot_core=[dict(town=[x-24,37],r11=[x,0],town_collision=kind(x-24,37),r11_candidate_collision=rk(x,0))for x in [40,41]],water_core=[dict(town=[x-24,37],r11=[x,0],town_collision=kind(x-24,37),r11_candidate_collision=rk(x,0))for x in range(48,62)],native_water_before='BUOY: must apply the seven explicit R11 patches before publishing connections')
write(out/'geometry-proof.json',proof)
patches=[dict(metatile=[i%20,i//20],before=f'{old:02x}',after=f'{new:02x}',purpose=purpose[(i%20,i//20)])for i,(old,new)in enumerate(zip(m['blocks'],blocks))if old!=new]
contract=dict(schema_version=1,status='DETACHED_SOURCE_PROPOSAL_NOT_ACCEPTED_OR_NATIVE',base_pack_sha256=sha(raw),base_identity=base['identity']['content_hash'],map='MnGrandMarais',base_module_sha256=sha(canonical(m)),dimensions_metatiles=[20,19],proposed_world_origin=[202,-419],native_alias=TS,native_alias_parent='johto',native_art_recipes=recipes,cells=patches,connections_before=m['attributes']['connections'],connections_after=attrs['connections'],preserved_doors=doors,preserved_rooms_sha256={k:sha(canonical(v))for k,v in base['data']['maps'].items()if k.startswith('MnGrandMarais')and k!='MnGrandMarais'},guard_relocations=relocations,gate_rotation=dict(root='MnCohesionMaraisEast',prerequisites='unchanged all eight native badges and EVENT_MN_LIGHTHOUSE_SIGNAL; local lamp flag remains separate',outbound_land_core=[[16,34],[17,34]],inward_land_core=[[16,36],[17,36]],argument_edits=[]),water_gate=dict(outbound_core=water_out,inward_core=water_in,outbound_wrapper=dict(label='MnMaraisCoastWaterOut',native_commands=[dict(command='readvar',args=['VAR_FACING']),dict(command='ifequal',args=['DOWN','MnCohesionMaraisEast']),dict(command='end',args=[])]),inward_wrapper=dict(label='MnMaraisCoastWaterIn',native_commands=[dict(command='readvar',args=['VAR_FACING']),dict(command='ifequal',args=['UP','MnCohesionMaraisEastInward']),dict(command='end',args=[])]),purpose='full-width directional water checkpoint; free inward scripted two-step crossing; no Surf bypass around land officers'),wayfinding_sign=dict(label='MnGrandMaraisSign1Text',core=[12,14],before_lines=['HARBOR PARK','FOLLOW SHORE EAST'],after_lines=['SUPERIOR HARBOR','NORTH SHORE: SOUTH','GUNFLINT: WEST','ELY: INLAND SNOW']),shared_geometry_owner='river_native; placement-v01 proof; R11/Cedar/neck edits are not owned here',all_other_town_and_room_scripts_events_services_quests_and_audio='unchanged except exact declared gate rotation/wrappers/event bindings and one existing sign text')
contract['r11_counterpart_required']=proof['paired_r11_source_ports']
contract['gate_rotation']['argument_edits']=[dict(label='MnCohesionMaraisEastInward',command_index=1,arg_index=0,before='RIGHT',after='DOWN'),dict(label='MnCohesionMaraisEastOneStep',command_index=0,arg_index=0,before='LEFT',after='UP')]+[dict(label='MnCohesionMaraisEastTwoSteps',command_index=i,arg_index=0,before='LEFT',after='UP')for i in [0,1]]
write(out/'native-edit.contract.json',contract);write(out/'compiled-candidate.blocks.json',dict(map='MnGrandMarais',tileset=TS,dimensions_metatiles=[20,19],blocks=blocks))
whole=[];states={}
for name,p,local,actors in [('actual-warm-parent',base,m,before_actors),('proposal-closed',preview,candidate,closed),('proposal-open-fixture',preview,candidate,open_actors)]:
 art=ReviewArt(p);ground,scene,_=render_map(art,local,actors,'day',out/name);whole.append((name,scene));states[name]=(ground,scene,art)
 frames=[]
 for label,core in [('services',[15,11]),('inland-departure',[4,8]),('lighthouse',[15,28]),('quay-head',[17,29])]+([]if name=='actual-warm-parent'else[('coastal-quay',[16,32]),('direct-arrival',[16,36])]):
  look=local['attributes']['tileset_name'];grid=local['blocks'];x,y=core;token=p['data']['tilesets'][look]['collision'][f'{grid[y//2*20+x//2]:02x}'][y%2*2+x%2];assert token in foot and core not in [o['tile']for o in actors],(name,label,core,token)
  frame,_=player_camera(art,local,actors,ground,scene,'day',dict(core=core),out/name/'views'/label);frames.append((label+' '+str(core),frame))
 save_render(room_sheet(frames,3),out/name/'players.png')
save_render(room_sheet(whole,3),out/'whole-town-before-closed-open.png')
coast={}
for state in ['actual-warm-parent','proposal-open-fixture']:
 coast[state]=[]
 for name in ['MnR11C00R00','MnR11C01R00']:
  local=r11 if name=='MnR11C00R00'and state=='proposal-open-fixture'else base['data']['maps'][name];art=ReviewArt(base);_,scene,_=render_map(art,local,objects(local),'day',out/'warm-coast'/state/name);coast[state].append((local,scene))
for state in ['actual-warm-parent','proposal-open-fixture']:
 joint=Image.new('RGBA',(56*32,43*32),(19,29,40,255));joint.alpha_composite(states[state][1],(12*32,0))
 for local,scene in coast[state]:joint.alpha_composite(scene,(0 if local['id']=='MnR11C00R00'else 32*32,19*32))
 save_render(joint,out/(state+'-joined-coast.png'))
 for label,core,sprite in [('direct-foot-seam',[40,38],'SPRITE_CHRIS')]+([]if state=='actual-warm-parent'else[('superior-water-seam',[52,38],'SPRITE_SURF')]):
  marked=joint.copy();marker=oriented_sprite(preview,sprite,'south');save_render(marker,out/'source-markers'/(sprite+'.png'));marked.alpha_composite(marker,(core[0]*16,core[1]*16));save_render(marked,out/state/(label+'-whole-marker.png'))
  x=core[0]*16-80;y=core[1]*16-72;frame=marked.crop((x,y,x+160,y+144));save_render(frame,out/state/(label+'-source160.png'));save_render(frame.resize((640,576),Image.Resampling.NEAREST),out/state/(label+'-source4x.png'))
# Preserve a complete authoring layout derived from the already-reviewed native
# harbor source, then append only these explicit hand-placed coastal strokes.
legacy=ROOT/'planning/minnesota/connections/grand-marais-harbor-closure-v01/MnGrandMarais.candidate.layout.json'
layout=json.loads(legacy.read_bytes());assert layout_blocks(layout)==m['blocks'],'legacy full layout must match the actual warm-parent town'
layout['id']='MnGrandMarais';layout['compiled_tileset']=TS
for patch in patches:layout['layers'].append(dict(kind='point',name=patch['purpose'],position=patch['metatile'],block=patch['after']))
assert layout_blocks(layout)==blocks
write(out/'MnGrandMarais.layout.candidate.json',layout)
dossier_path=ROOT/'planning/minnesota/towns/MnGrandMarais.json';dossier=json.loads(dossier_path.read_bytes())
dossier['composition']='Green Lake Superior harbor and lighthouse quay; direct North Shore arrival south, inland Gunflint/Ely departure west.'
dossier['ports']=[dict(route='MN_R11',side='S',gate_metatile=[8,17],landing_metatile=[8,18],physical_facade='Open-air fenced stone quay; direct Foot and Superior Surf map seams; no gatehouse detour',native_connection='MnR11C00R00 north offset+12 / town south offset-12; core16/17 to40/41, water24..37 to48..61'),dict(route='MN_R12',side='W',gate_metatile=[0,4],landing_metatile=[2,4],native_connection='MnMaraisCedarApproach east; offsets0/0; core8/9. Cedar is the inland section to the new neck and existing Gunflint, not the coastal approach.')]
dossier['interiors']=[room for room in dossier['interiors']if room['id']in contract['preserved_rooms_sha256']]
assert len(dossier['interiors'])==6
for room in dossier['interiors']:
 room['source_behavior']='Preserve actual installed warm-parent room module and all existing behavior byte-for-byte. No alternate shore-successor room programs are imported.'
dossier['door_contract']='Six original native indexed facade/room pairs and automatic outdoor exit steps are preserved. The regional coast uses direct outdoor Foot/Surf seams; no seventh gatehouse room.'
dossier['native_connections_source']='MnGrandMarais.connections.candidate.json'
write(out/'MnGrandMarais.dossier.candidate.json',dossier)
connections=dict(schema_version=1,status='DETACHED_CURRENT_683_MAP_LINEAGE_GEOGRAPHY_CANDIDATE_NOT_PUBLISHED',map='MnGrandMarais',base_pack_sha256=sha(raw),base_identity=base['identity']['content_hash'],native_module_before_sha256=sha(canonical(m)),world_origin_proposed=[202,-419],attributes_before=m['attributes'],attributes_after=attrs,foot_seams=[dict(side='south',town_core=[[16,37],[17,37]],target_map='MnR11C00R00',target_core=[[40,0],[41,0]],offset=-12),dict(side='west',town_core=[[0,8],[0,9]],target_map='MnMaraisCedarApproach',target_core=[[63,8],[63,9]],offset=0)],surf_seam=dict(side='south',town_core_x=list(range(24,38)),town_core_y=37,target_map='MnR11C00R00',target_core_x=list(range(48,62)),target_core_y=0,requires_r11_seven_cell_aperture=True),preserved_room_ids=list(contract['preserved_rooms_sha256']),new_rooms=[],native_edit_contract='native-edit.contract.json',shared_placement_owner='river_native',canonical_world_edited=False)
write(out/'MnGrandMarais.connections.candidate.json',connections)
canonical_paths=[ROOT/'planning/minnesota/world.json',dossier_path,ROOT/'planning/minnesota/towns/MnGrandMarais.connections.json',legacy,ROOT/'planning/minnesota/playable/grand-marais.json']
manifest=dict(status='NARROW_CURRENT_NATIVE_GEOGRAPHY_CANDIDATE_ONLY',actual_maps=len(base['data']['maps']),native_town_module_sha256=sha(canonical(m)),canonical_before_sha256={str(p.relative_to(ROOT)):sha(p.read_bytes())for p in canonical_paths},native_town_attributes_before_sha256=sha(canonical(m['attributes'])),native_block_bytes_before_sha256=sha(bytes(m['blocks'])),current_native_rooms=contract['preserved_rooms_sha256'],alternate_lineage_reference='The544-map north-shore-story shore alias/reef art was studied; its seventh gatehouse and room/story/recovery datasets are not imported.',world_canonical_patch_requires_owner_guarded_review=True,canonical_or_world_writes=0)
write(out/'source-preimages.json',manifest)
# Source-only pixel proof uses exact same-quadrant donor pixels in all banks.
pixel_frames=[];pixel_banks=[]
for bank in ['morn','day','nite','dark']:
 old_art=SourceArt(base).blocks('johto',bank);new_art=SourceArt(preview).blocks(TS,bank)
 assert len(old_art)==128 and len(new_art)==143
 for old_image,new_image in zip(old_art,new_art):assert ImageChops.difference(old_image,new_image).getbbox()is None
 for recipe in recipes:
  expected=old_art[int(recipe['base'],16)].copy()
  for q in recipe['quadrants']:
   x=q['index']%2*16;y=q['index']//2*16;expected.paste(old_art[int(q['source'],16)].crop((x,y,x+16,y+16)),(x,y))
  actual=new_art[int(recipe['id'],16)];assert ImageChops.difference(expected,actual).getbbox()is None
  pixel_frames.append((bank+' '+recipe['id'],actual))
 pixel_banks.append(dict(bank=bank,old_pixels=128*1024,new_recipe_pixels=15*1024,different_pixels=0))
save_render(room_sheet(pixel_frames,15),out/'four-bank-native-quadrant-recipes.png')
write(out/'source-art-proof.json',dict(status='SOURCE_PIXELS_NOT_GPU',banks=pixel_banks,old_pixels=128*1024*4,new_recipe_pixels=15*1024*4,different_pixels=0,new_glyphs=0,new_source_pixels=0,behavior_parent_required='johto'))
write(out/'render-loop.json',dict(elapsed_seconds=round(time.perf_counter()-started,3),native_parent_sha256=sha(raw),source_only=True,pack_written=False,cargo_executed=False,source_script_sha256=sha(Path(__file__).read_bytes()),native_contract_sha256=sha((out/'native-edit.contract.json').read_bytes()),images_preserved_as_fresh_epoch=True))
print('render',out);print('cells',len(patches),'recipes',len(recipes),'contract',sha((out/'native-edit.contract.json').read_bytes()))

