#!/usr/bin/env python3
"""Read-only whole-pack, native cave and active regional guard audit."""
import argparse
import base64
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re
import sys

import cbor2
from PIL import Image, ImageChops, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt, npc_palette_pack
from review_passage_native import actors


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    source=Path(__file__).parent;contract=json.loads((source/'contract.json').read_text())
    parent=ROOT/contract['parent_pack'];assert sha(parent)==contract['parent_pack_sha256']
    old,new=[cbor2.loads(path.read_bytes()[22:])for path in [parent,args.pack]]
    room='MnIronPumpGallery';alias='mn_iron_haulage_rock'
    before,after=[p['data']['maps'][room]for p in [old,new]]
    assert hashlib.sha256(json.dumps(before,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()==contract['native_module_sha256']
    for module,name in [(before,'before.json'),(after,'after.json')]:
        assert module['blocks']==[int(v,16)for row in json.loads((source/name).read_text())['rows']for v in row.split()]
    expected=deepcopy(old);expected['identity']=new['identity'];expected['data']['maps'][room]['blocks']=after['blocks']
    for key,value in dict(tileset_name=alias,border_block=46,time_of_day='nite',palette='PALETTE_NITE',fishing_group='FISHGROUP_MN_LAKE').items():
        expected['data']['maps'][room]['attributes'][key]=value;expected['data']['map_attributes'][room][key]=value
    label=before['attributes']['blocks_label'];expected['data']['map_blocks'][label]=new['data']['map_blocks'][label]
    assert base64.b64decode(new['data']['map_blocks'][label])==bytes(after['blocks'])
    expected['report']['manifests'].append('minnesota-iron-pump-gallery-rock-v1')
    assert expected==new,'unowned content changed'
    art=SourceArt(new)
    for block in [0x30,0x36]:
        assert ImageChops.difference(art.blocks(alias,'nite')[block],art.blocks('cave','nite')[block]).getbbox() is None,'native stair art changed'
    assert new['data']['tilesets'][alias]['collision']['0d']==['FLOOR']*4
    bodies={(o['x'],o['y'])for o in after['objects']}
    def nodes(opened):
        blocks=list(after['blocks'])
        if opened:
            for x,y in contract['gate_metatiles']:blocks[y*16+x]=13
        floor=set()
        for y in range(24):
            for x in range(32):
                kind=new['data']['tilesets'][alias]['collision'][f'{blocks[y//2*16+x//2]:02x}'][y%2*2+x%2]
                assert kind in {'FLOOR','WALL','UP_WALL','LADDER'}
                if x in {0,31}or y in {0,23}:assert kind=='WALL'
                if kind=='FLOOR'and(x,y)not in bodies:floor.add((x,y))
        return floor
    def flood(floor,start):
        distances={start:0};queue=deque([start])
        while queue:
            x,y=queue.popleft()
            for p in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if p in floor and p not in distances:distances[p]=distances[x,y]+1;queue.append(p)
        return distances
    lengths={}
    for opened in [False,True]:
        floor=nodes(opened);main=flood(floor,(4,19));core=flood(floor,(24,5))
        assert set(main)|set(core)==floor
        assert (set(main)==set(core)) if opened else not(set(main)&set(core))
        lengths['open'if opened else'closed']=main[25,18]
    assert lengths['closed']>=lengths['open']+12
    closed=nodes(False)
    for sight in [{(12,6),(12,7),(12,8)},{(26,13),(27,13),(28,13)}]:
        assert (25,18)not in flood(closed-sight,(4,19)), 'guard free bypass'
    roster=json.loads((ROOT/'planning/minnesota/pokemon/roster.json').read_text());curation=json.loads((ROOT/'planning/minnesota/pokemon/roster-curation.json').read_text());retired={f['id']for f in curation['retired_families']};normalize=lambda s:re.sub('[^A-Z0-9]','',s.upper());allowed={normalize(m['name'])for f in roster['families']if f['id']not in retired for m in f['members']}
    parties={}
    for trainer in ['MN_LODE_ARC_PUMP','MN_LODE_ARC_INTAKE']:
        party=new['data']['trainers']['trainers'][trainer]['party'];assert all(normalize(p['species'])in allowed for p in party);parties[trainer]=[dict(species=p['species'],level=p['level'])for p in party]
    for species,evolutions in new['data']['evolutions'].items():
        if normalize(species)in allowed:assert all(normalize(e['species'])in allowed for e in evolutions)
    fish,groups=set(),set()
    def scan(v):
        if isinstance(v,list):
            for x in v:scan(x)
        elif isinstance(v,dict):
            if v.get('species'):fish.add(v['species'])
            if v.get('time_group')and v['time_group']not in groups:
                groups.add(v['time_group']);scan(new['data']['fishing']['time_groups'][v['time_group']])
            for k,x in v.items():
                if k not in ['species','time_group']:scan(x)
    scan(new['data']['fishing']['groups']['FISHGROUP_MN_LAKE']);assert all(normalize(s)in allowed for s in fish)
    for pack,module,group,name in [(old,before,'day','before'),(new,after,'nite','after')]:
        a=module['attributes'];scene=actors(npc_palette_pack(pack,group),SourceArt(pack).render(a['tileset_name'],a['width'],a['height'],module['blocks'],group),module);scene.save(args.output/(name+'-original.png'));scene.resize((scene.width*2,scene.height*2),Image.Resampling.NEAREST).save(args.output/(name+'-2x.png'))
    left,right=[Image.open(args.output/(name+'-original.png')).convert('RGB')for name in ['before','after']];sheet=Image.new('RGB',(left.width+right.width+24,left.height+30),'#14171c');draw=ImageDraw.Draw(sheet);draw.text((8,7),'Previous facility floor',fill='white');draw.text((left.width+16,7),'Native pump maintenance gallery',fill='white');sheet.paste(left,(8,24));sheet.paste(right,(left.width+16,24));sheet.save(args.output/'whole-before-after.png')
    receipt=dict(status='PASS',parent_pack=str(parent.relative_to(ROOT)),parent_sha256=sha(parent),pack=str(args.pack),pack_sha256=sha(args.pack),content_identity=new['identity']['content_hash'],unchanged_other_maps=len(new['data']['maps'])-1,original_programs_ports_guards_and_control_exact=True,all_runtime_files_and_audio_exact=True,native_stair_pixels_verified=True,closed_core_partition=True,guard_approaches_required=True,return_lengths=lengths,guard_parties=parties,active_regional_species=len(allowed),fishing_species=sorted(fish))
    (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print('PASS whole root, physical openings, native stairs and regional guards',lengths)


if __name__=='__main__':main()
