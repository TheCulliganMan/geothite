#!/usr/bin/env python3
"""Hand-authored warm terminal headland; exact native cells, never edits a pack."""
import cbor2,copy,hashlib,json,sys
from collections import deque
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[5];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt
from render_progress_atlas import native_room
SHA='1c6ba5d59097546ff8b94e09c809d270727ce1272acb93759b89f3b3c7d9b90a';raw=(ROOT/'content-packs/minnesota-superior-inland-art-v01.crystalpack').read_bytes();assert hashlib.sha256(raw).hexdigest()==SHA;p=cbor2.loads(raw[22:]);names=['MnR11C00R00','MnR11C01R00'];original=[p['data']['maps'][n]for n in names];candidates=copy.deepcopy(original);W,H=56,24;before=[0]*(W*H)
for i,m in enumerate(original):
 w=m['attributes']['width'];ox=0 if i==0 else 32
 for y in range(H):before[y*W+ox:y*W+ox+w]=m['blocks'][y*w:(y+1)*w]
b=before.copy();land=Image.new('1',(W,H));ImageDraw.Draw(land).polygon([(22,0),(55,0),(55,7),(47,7),(47,9),(42,9),(42,7),(34,7),(34,6),(27,6),(27,5),(22,5)],fill=1)
for y in range(H):
 for x in range(W):
  if land.getpixel((x,y)):b[y*W+x]=5
b[20]=5
pts=[(20,3),(25,3),(25,4),(31,4),(31,6),(39,6),(39,3),(45,3),(45,4),(55,4)]
for (x,y),(xx,yy)in zip(pts,pts[1:]):
 assert x==xx or y==yy
 for a in range(min(x,xx),max(x,xx)+1):
  for z in range(min(y,yy),max(y,yy)+1):b[z*W+a]=1
for x,y in[(28,4),(29,4),(35,6),(36,6),(43,3),(44,3)]:b[y*W+x]=3
# Fenced lake overlook sits one short spur off the road; actual Superior is visible.
for x in range(48,51):
 for y in range(5,7):b[y*W+x]=2
b[7*W+48]=0x48;b[7*W+49]=0x49;b[7*W+50]=0x4a
# Explicit east Superior aperture; preserve the old reef below town's south cap.
for y in range(8,18):b[y*W+55]=0x35
# Re-fit only ordinary lake cells touching this new hand-drawn land contour.
# Existing falls, creeks, city art and every lower row remain outside this box.
old=b.copy();water={0x35,0x54,0x55,0x58,0x59,0x76,0x79,0x7a,0x30,0x31,0x32,0x36}
def is_land(x,y):return 0<=x<W and 0<=y<H and old[y*W+x]not in water
for y in range(1,11):
 for x in range(22,W):
  if old[y*W+x]not in water:continue
  # Preserve closed east reef outside the paired GM window.
  if x==55 and y not in range(8,18):continue
  # Preserve north boundary reef outside the peninsula (none expected inside).
  n,we,e=is_land(x,y-1),is_land(x-1,y),is_land(x+1,y)
  v=0x54 if n and we else 0x55 if n and e else 0x58 if we else 0x59 if e else 0x76 if n else 0x79 if is_land(x-1,y-1)else 0x7a if is_land(x+1,y-1)else 0x35
  b[y*W+x]=v
coll=p['data']['tilesets']['mn_north_shore']['collision'];occupied={(o['x']+(64 if i else 0),o['y'])for i,m in enumerate(original)for o in m['objects']}
def kind(x,y):return coll[f'{b[y//2*W+x//2]:02x}'][y%2*2+x%2]
allowed={(x,y)for y in range(H*2)for x in range(W*2)if kind(x,y)in{'FLOOR','TALL_GRASS'}and(x,y)not in occupied};start=(40,6);end=(111,8);q=deque([start]);dist={start:0};prev={start:None}
while q:
 x,y=q.popleft()
 for nn in[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
  if nn in allowed and nn not in dist:dist[nn]=dist[x,y]+1;prev[nn]=(x,y);q.append(nn)
assert end in dist
assert all(kind(x,0)=='WALL'for x in range(40,112))
assert all(kind(111,y)=='WATER'for y in range(16,36));assert all(kind(111,y)=='FLOOR'for y in[8,9])
# Both existing internal seams are synthesized from this single exact grid.
changes=[]
for i,m in enumerate(candidates):
 w=m['attributes']['width'];ox=0 if i==0 else 32
 m['blocks']=[b[y*W+ox+x]for y in range(H)for x in range(w)]
 cells=[dict(metatile=[j%w,j//w],before=f'{a:02x}',after=f'{z:02x}')for j,(a,z)in enumerate(zip(original[i]['blocks'],m['blocks']))if a!=z]
 changes.append(dict(map=m['id'],native_module_sha256=hashlib.sha256(json.dumps(original[i],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),dimensions_metatiles=[w,H],cells=cells))
OUT=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/west-r11-headland-v02';SOURCE=ROOT/'planning/minnesota/revisions/north-shore-ely-geography-v01/geography/west-r11-headland-v02';assert not OUT.exists()and not SOURCE.exists();OUT.mkdir();SOURCE.mkdir();art=SourceArt(p)
for label,ms in [('before',original),('candidate',candidates)]:
 im=Image.new('RGB',(W*32,H*32));ox=0
 for m in ms:im.paste(native_room(m,p,native_origin=True).convert('RGB'),(ox,0));ox+=m['attributes']['width']*32
 im.save(OUT/f'R11-terminal-{label}-native.png')
 if label=='candidate':
  sheet=Image.new('RGB',(3*660,2*626),'#14212c');d=ImageDraw.Draw(sheet)
  for i,(name,x,y)in enumerate([('existing-trail-turn',42,6),('first-grass-saddle',57,8),('middle-bank-bend',70,12),('second-pine-bend',80,6),('fenced-Superior-view',98,12),('town-west-arrival',106,8)]):
   px=x*16+8;py=y*16+8;view=im.crop((px-80,py-72,px+80,py+72));view.save(OUT/f'{name}-native160.png');xx=i%3*660+10;yy=i//3*626+36;sheet.paste(view.resize((640,576),Image.Resampling.NEAREST),(xx,yy));d.text((xx,yy-25),name+' / candidate source crop only',fill='white')
  sheet.save(OUT/'six-player-source-cameras.png')
contract=dict(schema_version=1,status='DETACHED WEST-ENTRY CANDIDATE; supersedes the seven-cell north opening',parent_pack_sha256=SHA,changes=changes,connection_replacements=[dict(map='MnR11C00R00',direction='north',before=dict(target_map='MnMaraisCedarApproach',offset=-8),after=None),dict(map='MnR11C01R00',direction='east',before=None,after=dict(target_map='MnGrandMarais',offset=0))],paired_town_ports=dict(foot=dict(r11=[[47,8],[47,9]],gm=[[0,8],[0,9]]),water=dict(r11=[[47,16],[47,35]],gm=[[0,16],[0,35]])),source_physical=dict(existing_trail_core=list(start),town_edge_whole_core=list(end),steps=dist[end],all_proposed_north_edges_closed=True,all_20_Superior_seam_lanes_water=True,all_original_events_actors_programs=True),features=['Winding two-core-wide pine trail over a warm hand-drawn headland','Three mandatory grass saddles, subject to source minimum-grass check','Fenced actual-Superior overlook one short spur from the main path','All old falls, creeks, cabin and lower routes remain outside the new terminal footprint'],limitations=['Actual GM paired west water gate belongs to town author','Source crops not GPU/controller/clamp acceptance','Rebind exact native preimages to final town/art parent before publication'])
(SOURCE/'contract.json').write_text(json.dumps(contract,indent=2)+'\n');print(json.dumps(dict(cells={r['map']:len(r['cells'])for r in changes},steps=dist[end],contract_sha256=hashlib.sha256((SOURCE/'contract.json').read_bytes()).hexdigest())))
