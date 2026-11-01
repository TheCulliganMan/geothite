#!/usr/bin/env python3
"""Explicit seven-cell warm Superior aperture, detached from publication."""
import argparse,cbor2,copy,hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt
arg=argparse.ArgumentParser();arg.add_argument('--pack',type=Path,required=True);arg.add_argument('--sha256',required=True);arg.add_argument('--epoch',required=True);args=arg.parse_args();assert args.epoch.startswith('r11-aperture-v')and'/'not in args.epoch
SHA=args.sha256;raw=args.pack.read_bytes();assert hashlib.sha256(raw).hexdigest()==SHA;p=cbor2.loads(raw[22:]);m=p['data']['maps']['MnR11C00R00'];candidate=copy.deepcopy(m)
assert len(p['data']['maps'])==693 and'MnEly'in p['data']['maps']
previous=json.loads((ROOT/'planning/minnesota/revisions/north-shore-ely-geography-v01/geography/r11-aperture-v01/contract.json').read_bytes());assert hashlib.sha256(json.dumps(m,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()==previous['parent_r11_module_sha256']

def h(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
changes=[dict(metatile=[x,0],before=f'{0x30 if x==24 else 0x31:02x}',after=f'{0x58 if x==24 else 0x35:02x}')for x in range(24,31)]
for r in changes:x,y=r['metatile'];assert m['blocks'][y*32+x]==int(r['before'],16);candidate['blocks'][y*32+x]=int(r['after'],16)
c=p['data']['tilesets']['mn_north_shore']['collision'];assert all(c[f'{candidate["blocks"][x//2]:02x}'][x%2]=='WATER'for x in range(48,62));assert candidate['blocks'][31]==0x31;assert c[f'{candidate["blocks"][31]:02x}'][:2]==['BUOY','BUOY'];assert candidate['blocks'][20]==1
source=ROOT/'planning/minnesota/revisions/north-shore-ely-geography-v01/geography'/args.epoch;out=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01'/args.epoch;assert not source.exists()and not out.exists();source.mkdir();out.mkdir()
contract=dict(status='Guarded detached seven-cell candidate; actual GM counterpart owned separately',parent_pack_sha256=SHA,parent_r11_module_sha256=h(m),map='MnR11C00R00',tileset='mn_north_shore',cells=changes,old_north_connection=dict(direction='north',target_map='MnMaraisCedarApproach',offset=-8),new_north_connection=dict(direction='north',target_map='MnGrandMarais',offset=12),matched_gm_south=dict(direction='south',target_map='MnR11C00R00',offset=-12),foot_core=dict(r11=[[40,0],[41,0]],gm=[[16,37],[17,37]],unchanged_r11_terrain=True),water_core=dict(r11=[[48,0],[61,0]],gm=[[24,37],[37,37]]),preserve=dict(all_other_761_blocks=True,meta31_top_buoy_cap=True,all_c01_north_reef=True,all_other_attributes_programs_events_npcs=True),physical_note='This opens the actual installed Superior water below, not a new river. Land bank uses same actual58 as row1;6center metatiles use35. Every source lane remains WATER and adjacentrow1 WATER.',parent_rebind='Actual supplied additiveEly parent verified693maps and exactR11nativepreimage preserved; revalidate exacthash beforepublication.')
(source/'contract.json').write_text(json.dumps(contract,indent=2)+'\n');art=SourceArt(p)
for label,b in [('before',m['blocks']),('candidate',candidate['blocks'])]:im=art.render('mn_north_shore',32,24,b);im.save(out/f'R11C00R00-{label}-native.png');im.crop((18*32,0,32*32,6*32)).resize((14*64,6*64)).save(out/f'R11-north-aperture-{label}-2x.png')
print(json.dumps(dict(contract_sha256=hashlib.sha256((source/'contract.json').read_bytes()).hexdigest(),seven_cells=len(changes),water_lanes=14,remaining_metatiles_preserved=761)))
