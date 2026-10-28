#!/usr/bin/env python3
"""Detached four-metatile terminal bend; source-art preview, never a pack writer."""
import cbor2
import hashlib
import json
from pathlib import Path
from PIL import Image
from render_reference import SourceArt
ROOT=Path(__file__).resolve().parents[6]
OUT=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/coast-quality-v01/compact-bend-v02'
OUT.mkdir(parents=True,exist_ok=False)
PACK=ROOT/'content-packs/minnesota-superior-inland-winter-v01.crystalpack'
with PACK.open('rb') as stream:stream.read(22);pack=cbor2.load(stream)
art=SourceArt(pack)
old=pack['data']['maps']['MnR11C01R00'];blocks=[0x35]*96
for y in range(7):
 for x in range(4):blocks[y*4+x]=0x05
for x,y in [(0,6),(1,6),(1,5),(2,5),(2,4),(3,4)]:blocks[y*4+x]=0x01
for y in range(7,24):blocks[y*4+3]=0x05 if y==7 else 0x35 if y<18 else 0x0a
# Small visible native bank directly below the walking turn.
for x in range(3):blocks[7*4+x]=0x58
map_image=art.render('mn_north_shore',4,24,blocks);map_image.save(OUT/'compact-terminal-original.png')
scene=Image.new('RGB',(56*32,43*32),'#17222b')
# Actual old C00 and town art at proposed footprints, no terrain in the gap.
for name,origin in [('MnR11C00R00',(0,19)),('MnGrandMarais',(36,19))]:
 m=pack['data']['maps'][name];a=m['attributes'];scene.paste(art.render(a['tileset_name'],a['width'],a['height'],m['blocks']),(origin[0]*32,origin[1]*32))
scene.paste(map_image,(32*32,19*32));scene.save(OUT/'compact-headland-town-original.png')
scene.resize((896,688)).save(OUT/'compact-headland-town-review.png')
# Native-scale source camera centered on the weak arrival / turn / actual town seam.
for name,x,y in [('original-trail',40,6),('short-turn',65,11),('town-arrival',73,8)]:
 # Core coordinates measured from original C00 map origin.
 cx=x*16;cy=19*32+y*16
 scene.crop((cx-80,cy-72,cx+80,cy+72)).save(OUT/f'{name}-source-camera-original.png')
contract={'schema_version':1,'status':'DETACHED_GEOMETRY_PROPOSAL_NOT_NATIVE_PUBLISHED','parent_sha256':hashlib.sha256(PACK.read_bytes()).hexdigest(),'old_module_sha256':hashlib.sha256(json.dumps(old,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),'proposed_map':'MnR11C01R00','dimensions_metatiles':[4,24],'origin_metatiles':[222,-400],'rows':[' '.join(f'{b:02x}' for b in blocks[y*4:y*4+4]) for y in range(24)],'town_origin_metatiles':[226,-400],'cedar_origin_if_retained':[207,-419],'neck_width_if_retained':45,'walking_seams_core':{'west':[[0,12],[0,13]],'east':[[7,8],[7,9]]},'shared_superior_east_core_rows':[16,35],'footprints_fixed':['MnR11C00R00','all lower R11 chunks','R12','R13','BoundaryWaters'],'limitations':['C00 original terminal approach still retained; root may choose further crop after reviewing.','Global graph, shore boundary and native controller proof remain pending.','No new long path, grass quota, encounter, reward or elevation is introduced into this four-metatile turn.']}
(Path(__file__).parent/'compact-bend.contract.json').write_text(json.dumps(contract,indent=2)+'\n')
print(OUT)
