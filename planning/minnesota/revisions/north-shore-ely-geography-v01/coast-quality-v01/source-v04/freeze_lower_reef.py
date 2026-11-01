#!/usr/bin/env python3
"""Freeze the actual lower-neighbor overhang closure; no native pack writing."""
import cbor2
import hashlib
import json
from pathlib import Path
from PIL import Image
from render_reference import SourceArt, layout_blocks
ROOT=Path(__file__).resolve().parents[6]
HERE=Path(__file__).resolve().parent
PACK=ROOT/'content-packs/minnesota-superior-inland-winter-v01.crystalpack'
with PACK.open('rb')as f:f.read(22);pack=cbor2.load(f)
data=pack['data'];maps=data['maps'];old=maps['MnR11C01R01']
def sha(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
c=json.loads((HERE/'compact-bend.contract.json').read_text());c['lower_north_boundary']={'map':'MnR11C01R01','old_module_sha256':sha(old),'cells':[{'metatile':[x,0],'before':old['blocks'][x],'after':10}for x in range(4,24)]}
c['limitations']=[v for v in c['limitations']if 'Global graph' not in v]
c['limitations'].append('Source/native controller and production-camera acceptance pending; lower C01R01 now has an explicit20-cell north reef correction.')
(HERE/'compact-bend.contract.json').write_text(json.dumps(c,indent=2)+'\n')
layout=json.loads((HERE/'MN_R11.layout.candidate.json').read_text())
for x in range(4,24):layout['layers'].append({'name':'Visible lower-neighbor north overhang reef after compact crop','kind':'point','position':[32+x,24],'block':'0a'})
(HERE/'MN_R11.layout.candidate.json').write_text(json.dumps(layout,indent=2)+'\n')
d=json.loads((HERE/'MN_R11.dossier.candidate.json').read_text());d['compact_terminal']['reciprocal_lower_neighbor_reef']={'map':'MnR11C01R01','metatile_bounds':[4,0,24,1],'block':'0a','collision':'WALL in all quadrants','reason':'Cropped northern neighbor has no target beyond corex7. Close actual lower-map north edge visibly instead of relying on its broad north connection.'}
(HERE/'MN_R11.dossier.candidate.json').write_text(json.dumps(d,indent=2)+'\n')
f=json.loads((HERE/'ragged-footprint.contract.json').read_text());f['source_sha256']={p.name:hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(HERE.glob('MN_R11.*.json'))};f['changed_native_chunks']=['MnR11C01R00','MnR11C01R01'];f['all_other_six_r11_chunks_unchanged']=True
(HERE/'ragged-footprint.contract.json').write_text(json.dumps(f,indent=2)+'\n')
# This is an in-memory source-art review only. Native writing stays in Rust.
revised=json.loads(json.dumps(maps));revised['MnR11C01R00']['blocks']=[int(b,16)for row in c['rows']for b in row.split()];revised['MnR11C01R00']['attributes']['width']=4
for x in range(4,24):revised['MnR11C01R01']['blocks'][x]=10
art=SourceArt(pack);out=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/coast-quality-v01/compact-bend-v03';out.mkdir(parents=True,exist_ok=False)
scene=Image.new('RGB',(56*32,48*32),'#17222b')
for name,x,y in [('MnR11C00R00',0,0),('MnR11C01R00',32,0),('MnGrandMarais',36,0),('MnR11C00R01',0,24),('MnR11C01R01',32,24)]:
 m=revised[name];a=m['attributes'];im=art.render(a['tileset_name'],a['width'],a['height'],m['blocks']);im.save(out/f'{name}-terrain-original.png');scene.paste(im,(x*32,y*32))
scene.save(out/'whole-coast-town-original.png');scene.resize((896,768)).save(out/'whole-coast-town-review.png')
for label,x,y in [('western-water-aperture',67,48),('middle-closed-north-reef',87,49),('eastern-reef-corner',109,49),('compact-town-arrival',71,8)]:scene.crop((x*16-80,y*16-72,x*16+80,y*16+72)).save(out/f'{label}-source-camera-original.png')
# Two-sided boundary audit: inspect every exposed core on BOTH actual neighbors,
# including the lower edge beyond the cropped upper width.
def kind(name,x,y):
 m=revised[name];a=m['attributes'];b=m['blocks'][y//2*a['width']+x//2]
 return data['tilesets'][a['tileset_name']]['collision'][f'{b:02x}'][y%2*2+x%2]
for x in range(8,48):assert kind('MnR11C01R01',x,0)=='WALL',(x,'leaked lower north')
for x in range(8):
 a=kind('MnR11C01R00',x,47);b=kind('MnR11C01R01',x,0)
 assert a in ['WATER','WALL','BUOY']and b in ['WATER','WALL','BUOY']
 if a=='WATER':assert b=='WATER'
 if b=='WATER'and a!='WATER':assert a in ['WALL','BUOY']
# Every lower east tile is visibly blocked; every lower south Water has a
# real same-row target in the unchanged next chunk. West remains unchanged.
for y in range(48):assert kind('MnR11C01R01',47,y)in['BUOY','WALL'],('lower east',y)
for x in range(48):
 if kind('MnR11C01R01',x,47)=='WATER':assert kind('MnR11C01R02',x,0)=='WATER',('lower south',x)
for y in range(48):
 if kind('MnR11C01R01',0,y)=='WATER':assert kind('MnR11C00R01',63,y)=='WATER',('lower west',y)
compiled=layout_blocks(layout)
for name in ['MnR11C01R00','MnR11C01R01']:
 m=revised[name];a=m['attributes'];oy=24 if name.endswith('R01')else 0
 assert [compiled[(oy+y)*56+32+x]for y in range(a['height'])for x in range(a['width'])]==m['blocks']
proof={'schema_version':1,'status':'SOURCE_COLLISION_PASS_NOT_CONTROLLER_ACCEPTANCE','new_changed_chunk':'MnR11C01R01','lower_north_preimage_open_water_core_x':list(range(8,47)),'lower_north_closed_core_x':list(range(8,48)),'reciprocal_shared_water_core_x':list(range(6)),'shared_x6_x7_target':'Visible upper0a rock blocks approach; no outside-target movement','lower_north_closed_count':40,'lower_east_all48_core_rows_closed':True,'lower_south_all_open_water_reciprocal':True,'lower_west_all_open_water_reciprocal':True,'changed_native_chunks':['MnR11C01R00','MnR11C01R01'],'unchanged_six_r11_module_sha256':{n:sha(m)for n,m in maps.items()if n.startswith('MnR11C')and n not in ['MnR11C01R00','MnR11C01R01']},'coast_contract_sha256':hashlib.sha256((HERE/'compact-bend.contract.json').read_bytes()).hexdigest()}
(HERE/'boundary-and-anchor-proof.json').write_text(json.dumps(proof,indent=2)+'\n')
print(out)
