#!/usr/bin/env python3
"""Explicit west-entry Cedar and 65x19 inland woods proposal; no pack mutation."""
import cbor2,copy,hashlib,heapq,json,sys
from collections import deque
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[5];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt
from render_progress_atlas import native_room
SHA='1c6ba5d59097546ff8b94e09c809d270727ce1272acb93759b89f3b3c7d9b90a';raw=(ROOT/'content-packs/minnesota-superior-inland-art-v01.crystalpack').read_bytes();assert hashlib.sha256(raw).hexdigest()==SHA;p=cbor2.loads(raw[22:]);base=p['data']['maps']['MnMaraisCedarApproach'];cedar=copy.deepcopy(base);alias='mn_winter_north_shore';coll=p['data']['tilesets'][alias]['collision'];assert coll['8b']==['WALL']*4
OUT=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/west-inland-v02';SOURCE=ROOT/'planning/minnesota/revisions/north-shore-ely-geography-v01/geography/west-inland-v02';assert not OUT.exists()and not SOURCE.exists();OUT.mkdir();SOURCE.mkdir()
def jhash(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
def at(b,w,x,y,v):b[y*w+x]=v
def path(b,w,points,v=0x83):
 for(x,y),(xx,yy)in zip(points,points[1:]):
  assert x==xx or y==yy
  for a in range(min(x,xx),max(x,xx)+1):
   for z in range(min(y,yy),max(y,yy)+1):at(b,w,a,z,v)
# Preserve original winding west-to-south Cedar geometry and both actual trainers.
# Warm eastern8 columns are the town-side forest; winter starts farther inland.
for y in range(19):
 for x in range(32):
  if x<24 and not(20<=x<24 and y<4):
   i=y*32+x;cedar['blocks'][i]={1:0x83,2:0x83,3:0x84,4:0x83,5:0x8b,0x45:0x88}.get(cedar['blocks'][i],cedar['blocks'][i])
a=cedar['attributes'];a.update(tileset_name=alias,border_block=0x8b,connections=[dict(direction='west',target_map='MnMaraisInlandNeck',offset=0),dict(direction='south',target_map='MnGrandMarais',offset=19)],connection_flags='WEST | SOUTH')
b=[0x8b]*(65*19);main=[(0,4),(7,4),(7,7),(17,7),(17,11),(26,11),(26,8),(36,8),(36,5),(46,5),(46,9),(57,9),(57,6),(64,6)];path(b,65,main)
for x,y in[(15,7),(16,7),(29,8),(30,8),(49,9),(50,9)]:at(b,65,x,y,0x84)
# Two short downhill ledges provide one-way choices with real walking returns.
path(b,65,[(7,4),(12,4)]);path(b,65,[(10,6),(12,6),(12,7)])
for x in range(10,13):at(b,65,x,5,0x56)
path(b,65,[(40,7),(46,7)])
for x in range(40,43):at(b,65,x,6,0x56)
# Small wind-sheltered campsite, with a real declared native supply pickup.
path(b,65,[(32,8),(32,13)])
for y in range(12,15):
 for x in range(30,36):at(b,65,x,y,0x83)
for x,y in[(30,12),(35,14)]:at(b,65,x,y,0x0a)
at(b,65,34,12,0x88)
neck=copy.deepcopy(cedar);neck['id']='MnMaraisInlandNeck';neck['blocks']=b
for k,v in list(neck.items()):
 if k not in{'id','attributes','blocks'}:neck[k]=[]if isinstance(v,list)else{}
neck['scenes']={'scenes':[]};neck['events']={'warps':[],'coord_events':[],'bg_events':[dict(x=69,y=25,event_type='BGEVENT_READ',script='MN_INLAND_CAMP_SIGN')]};neck['attributes']=copy.deepcopy(a);neck['attributes'].update(width=65,height=19,map_constant='MN_MARAIS_INLAND_NECK',map_group_constant='MAPGROUP_MINNESOTA',blocks_label='MnMaraisInlandNeck_Blocks',map_scripts_label='MnMaraisInlandNeck_MapScripts',map_events_label='MnMaraisInlandNeck_MapEvents',connections=[dict(direction='west',target_map='MnR12C01R01',offset=0),dict(direction='east',target_map='MnMaraisCedarApproach',offset=0)],connection_flags='WEST | EAST')
item=copy.deepcopy(base['objects'][0]);item.update(sprite='SPRITE_POKE_BALL',sprite_has_facings=False,x=63,y=27,spritemovedata='SPRITEMOVEDATA_STILL',object_type='OBJECTTYPE_ITEMBALL',radius=0,script='MN_INLAND_CAMP_SUPPLY',event_flag='EVENT_MN_GUNFLINT_CAMP_SUPPLY',object_identifier='MN_INLAND_CAMP_SUPPLY');neck['objects']=[item]
# This is a source module for exact-art preview only; native finish/publication
# builds the typed item/sign programs from the explicit bounded contract below.
DIRECTIONS=[('left',-1,0),('right',1,0),('up',0,-1),('down',0,1)]
def review(m,start,end,ledges=()):
 w=m['attributes']['width'];h=m['attributes']['height'];bs=m['blocks'];occ={(o['x'],o['y'])for o in m['objects']}
 def k(pos):x,y=pos;return coll[f'{bs[y//2*w+x//2]:02x}'][y%2*2+x%2]
 def okay(pos):x,y=pos;return 0<=x<w*2 and 0<=y<h*2 and pos not in occ and(k(pos)in{'FLOOR','TALL_GRASS'}or k(pos).startswith('HOP_'))
 def edges(pos):
  x,y=pos
  for button,dx,dy in DIRECTIONS:
   jump=k(pos)=='HOP_DOWN'and button=='down';target=(x+dx*(2 if jump else 1),y+dy*(2 if jump else 1))
   if okay(target):yield target,button,jump
 def bfs(s,e=None):
  q=deque([s]);dist={s:0};prev={s:None}
  while q:
   pos=q.popleft()
   for target,button,jump in edges(pos):
    if target not in dist:dist[target]=dist[pos]+1;prev[target]=(pos,button,jump);q.append(target)
  return dist,prev
 dist,prev=bfs(start);assert end in dist;reverse,_=bfs(end);assert start in reverse
 ports=sorted(pos for pos in dist if pos[0]in{0,w*2-1}or pos[1]in{0,h*2-1});expected=sorted([(0,12),(0,13),(56,37),(57,37)])if w==32 else sorted([(0,8),(0,9),(129,12),(129,13)]);assert ports==expected
 grass={start:0};queue=[(0,start)]
 while queue:
  cost,pos=heapq.heappop(queue)
  if cost!=grass[pos]:continue
  for target,_,_ in edges(pos):
   cc=cost+(k(target)=='TALL_GRASS')
   if cc<grass.get(target,10**9):grass[target]=cc;heapq.heappush(queue,(cc,target))
 assert grass[end]>0
 ledgereports=[]
 for takeoff,landing in ledges:
  assert k(takeoff)=='HOP_DOWN'and k(landing)=='FLOOR';assert(landing,'down',True)in list(edges(takeoff));ret,_=bfs(landing);assert takeoff in ret and ret[takeoff]>=9;assert(takeoff,'up',False)not in list(edges(landing));ledgereports.append(dict(takeoff_core=list(takeoff),landing_core=list(landing),one_down_input_jumps_two_core_tiles=True,reverse_up_blocked=True,walking_return_inputs=ret[takeoff]))
 actions=[];pos=end
 while pos!=start:old,button,jump=prev[pos];actions.append(dict(button=button,jump=jump,from_core=list(old),to_core=list(pos)));pos=old
 actions.reverse();actorapproaches={o['object_identifier']:any((o['x']+dx,o['y']+dy)in dist for _,dx,dy in DIRECTIONS)for o in m['objects']};assert all(actorapproaches.values())
 return dict(forward_inputs=dist[end],reverse_inputs=reverse[start],minimum_grass_steps=grass[end],reachable_core_tiles=len(dist),all_ports=ports,actor_approaches=actorapproaches,ledges=ledgereports),actions
cedarproof,cedarinputs=review(cedar,(0,12),(56,37));neckproof,neckinputs=review(neck,(0,8),(129,12),[((24,10),(24,12)),((82,12),(82,14))])
art=SourceArt(p)
for label,m in[('Cedar',cedar),('Neck',neck)]:native_room(m,p,native_origin=True).convert('RGB').save(OUT/f'{label}-candidate-native.png')
whole=Image.new('RGB',((65+32)*32,19*32));whole.paste(native_room(neck,p,native_origin=True).convert('RGB'),(0,0));whole.paste(native_room(cedar,p,native_origin=True).convert('RGB'),(65*32,0));whole.save(OUT/'whole-inland-actual-8b-source.png')
cameras=[('neck-west-grass-start',5,8),('first-short-ledge',24,10),('camp-cache',64,26),('second-short-ledge',82,12),('neck-east-port',125,12),('cedar-west-arrival',130+3,12),('cedar-snowline',130+47,29),('town-side-warm-exit',130+56,33)];sheet=Image.new('RGB',(4*660,2*626),'#14212c');draw=ImageDraw.Draw(sheet)
for i,(name,x,y)in enumerate(cameras):
 px=x*16+8;py=y*16+8;view=whole.crop((px-80,py-72,px+80,py+72));view.save(OUT/f'{name}-native160.png');xx=i%4*660+10;yy=i//4*626+36;sheet.paste(view.resize((640,576),Image.Resampling.NEAREST),(xx,yy));draw.text((xx,yy-25),name+' / exact8b source crop, not GPU',fill='white')
sheet.save(OUT/'eight-player-source-cameras.png')
changes=[dict(metatile=[i%32,i//32],before=f'{old:02x}',after=f'{new:02x}')for i,(old,new)in enumerate(zip(base['blocks'],cedar['blocks']))if old!=new];flag_next=max(v for k,v in p['data']['story_event_script_constants']['global'].items()if k.startswith('EVENT_'))+1;assert flag_next==2387
contract=dict(schema_version=1,status='DETACHED WEST-ENTRY SOURCE; native publication and player acceptance pending',parent_pack_sha256=SHA,parent_identity=p['identity']['content_hash'],cedar=dict(map='MnMaraisCedarApproach',native_module_sha256=jhash(base),dimensions_metatiles=[32,19],origin_metatiles=[227,-419],attributes=a,cells=changes,rows=[' '.join(f'{v:02x}'for v in cedar['blocks'][y*32:(y+1)*32])for y in range(19)],actors_events_programs_exact_except_sign_directions=True,text_changes=[dict(label='MN_COHESION_MARAIS_SIGNText',command_index=1,before='"GRAND MARAIS west."',after='"GRAND MARAIS south"'),dict(label='MN_COHESION_MARAIS_SIGNText',command_index=2,before='"NORTH SHORE south."',after='"GUNFLINT west."')]),neck=dict(map='MnMaraisInlandNeck',constant='MN_MARAIS_INLAND_NECK',dimensions_metatiles=[65,19],origin_metatiles=[162,-419],group_name='MINNESOTA',group_id=40,map_id=8,attributes=neck['attributes'],rows=[' '.join(f'{v:02x}'for v in b[y*65:(y+1)*65])for y in range(19)],objects=neck['objects'],events=neck['events'],pickup=dict(id='MN_INLAND_CAMP_SUPPLY',core=[63,27],item='BERRY',quantity=3,event_flag='EVENT_MN_GUNFLINT_CAMP_SUPPLY',fresh_event=flag_next),sign=dict(id='MN_INLAND_CAMP_SIGN',core=[69,25],lines=['PORTAGE CAMP','Three BERRIES for','the trail ahead.']),wild_encounters=dict(source='MnMaraisCedarApproach',source_sha256=jhash(p['data']['wild_encounters']['MnMaraisCedarApproach']),preserve_table_except_map_name=True),temporary_spawn=dict(identifier=280,core=[0,8],remove_after_registration=True),landmark='LANDMARK_MN_GRAND_MARAIS'),physical=dict(cedar=cedarproof,neck=neckproof),strict_residual_policy=['Only three existing geography terrain modules plus R12east connection may change; town parent is consumed unchanged','New Neck adds one map registration and ordinary compiled mirrors, one copied native encounter entry, one native Berry pickup and three-line camp sign','Exactly one fresh event2387; all original constants, parties, species, moves, audio, art, recovery and unrelated maps stay exact','Old south-town aperture/8-wide connector/Walker summit source epochs are superseded and must not be merged'],limitations=['Source HOP graph follows current-permission jump with2-core displacement, but only production controller proves execution','Native56 ledge retains its original green grass lip; no shared art or glyph changed','Final native contract must bind actual GM-town parent SHA/identity; owned preimages must remain exact'])
(SOURCE/'contract.json').write_text(json.dumps(contract,indent=2)+'\n');(OUT/'source-physical-proof.json').write_text(json.dumps(contract['physical'],indent=2)+'\n');(OUT/'source-input-candidates.json').write_text(json.dumps(dict(cedar=cedarinputs,neck=neckinputs),indent=2)+'\n');print(json.dumps(dict(cedar_cells=len(changes),physical=contract['physical'],contract_sha256=hashlib.sha256((SOURCE/'contract.json').read_bytes()).hexdigest())))
