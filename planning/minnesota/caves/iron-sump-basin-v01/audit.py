#!/usr/bin/env python3
"""Read-only full-pack and native shore-pixel audit of the sump revision."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re
import sys

import cbor2
from PIL import Image, ImageChops, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import SourceArt, npc_palette_pack
from review_passage_native import actors


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=False)
    source = Path(__file__).parent
    contract = json.loads((source/'contract.json').read_text())
    parent = ROOT/contract['parent_pack']
    assert sha(parent) == contract['parent_pack_sha256']
    old,new = [cbor2.loads(p.read_bytes()[22:]) for p in [parent,args.pack]]
    room,alias,base = 'MnIronSump','mn_iron_sump_rock','mn_iron_haulage_rock'
    before,after = [p['data']['maps'][room] for p in [old,new]]
    assert hashlib.sha256(json.dumps(before,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest() == contract['native_module_sha256']
    for module,name in [(before,'before.json'),(after,'after.json')]:
        assert module['blocks'] == [int(v,16) for row in json.loads((source/name).read_text())['rows'] for v in row.split()]
    expected = deepcopy(old)
    expected['identity'] = new['identity']
    expected['data']['maps'][room]['blocks'] = after['blocks']
    for key,value in dict(tileset_name=alias,border_block=46,time_of_day='nite',palette='PALETTE_NITE',fishing_group='FISHGROUP_MN_LAKE').items():
        expected['data']['maps'][room]['attributes'][key] = value
        expected['data']['map_attributes'][room][key] = value
    label = before['attributes']['blocks_label']
    expected['data']['map_blocks'][label] = new['data']['map_blocks'][label]
    assert base64.b64decode(new['data']['map_blocks'][label]) == bytes(after['blocks'])
    definition = deepcopy(old['data']['tilesets'][base])
    for block in ['43','44']:
        definition['collision'][block] = ['WATER','WATER','WALL','WALL']
    assert new['data']['tilesets'][alias] == definition
    expected['data']['tilesets'][alias] = definition
    files = {f'data/tilesets/{alias}.json',f'data/tilesets/{alias}_metatiles.bin',
             f'data/tilesets/{alias}_palette_map.json',f'gfx/tilesets/{alias}.png',f'gfx/tilesets/{alias}.2bpp'}
    assert set(new['runtime_files'])-set(old['runtime_files']) == files
    for path in files:
        expected['runtime_files'][path] = new['runtime_files'][path]
    expected['report']['manifests'].append('minnesota-iron-sump-native-shore-v1')
    assert expected == new,'unowned content changed'
    for suffix in ['.2bpp','.png']:
        assert new['runtime_files'][f'gfx/tilesets/{alias}{suffix}'] == old['runtime_files'][f'gfx/tilesets/{base}{suffix}']
    assert bytes(new['runtime_files'][f'data/tilesets/{alias}_metatiles.bin'])[:0x43*16] == bytes(old['runtime_files'][f'data/tilesets/{base}_metatiles.bin'])
    assert json.loads(bytes(new['runtime_files'][f'data/tilesets/{alias}.json'])) == definition['collision']
    art = SourceArt(new)
    original,private = art.blocks(base,'nite'),art.blocks(alias,'nite')
    for i in range(0x43):
        assert ImageChops.difference(original[i],private[i]).getbbox() is None
    for block,water in [(0x43,0x26),(0x44,0x25)]:
        wanted = Image.new('RGB',(32,32))
        wanted.paste(original[water].crop((0,0,32,16)),(0,0))
        wanted.paste(original[0x21].crop((0,16,32,32)),(0,16))
        assert ImageChops.difference(private[block],wanted).getbbox() is None
    roster = json.loads((ROOT/'planning/minnesota/pokemon/roster.json').read_text())
    curation = json.loads((ROOT/'planning/minnesota/pokemon/roster-curation.json').read_text())
    retired = {f['id'] for f in curation['retired_families']}
    normalize = lambda s:re.sub('[^A-Z0-9]','',s.upper())
    allowed = {normalize(m['name']) for f in roster['families'] if f['id'] not in retired for m in f['members']}
    parties = {}
    for trainer in ['MN_LODE_ARC_SUMP','MN_LODE_ARC_DRAIN']:
        party = new['data']['trainers']['trainers'][trainer]['party']
        assert all(normalize(p['species']) in allowed for p in party)
        parties[trainer] = [{'species':p['species'],'level':p['level']} for p in party]
    for species,evolutions in new['data']['evolutions'].items():
        if normalize(species) in allowed:
            assert all(normalize(e['species']) in allowed for e in evolutions)
    fish,groups = set(),set()
    def scan(v):
        if isinstance(v,list):
            for x in v:scan(x)
        elif isinstance(v,dict):
            if v.get('species'):fish.add(v['species'])
            if v.get('time_group') and v['time_group'] not in groups:
                groups.add(v['time_group']);scan(new['data']['fishing']['time_groups'][v['time_group']])
            for k,x in v.items():
                if k not in ['species','time_group']:scan(x)
    scan(new['data']['fishing']['groups']['FISHGROUP_MN_LAKE'])
    assert all(normalize(s) in allowed for s in fish)
    for pack,module,group,name in [(old,before,'day','before'),(new,after,'nite','after')]:
        a = module['attributes']
        scene = actors(npc_palette_pack(pack,group),SourceArt(pack).render(a['tileset_name'],a['width'],a['height'],module['blocks'],group),module)
        scene.save(args.output/(name+'-original.png'))
        scene.resize((scene.width*2,scene.height*2),Image.Resampling.NEAREST).save(args.output/(name+'-2x.png'))
    left,right = [Image.open(args.output/(name+'-original.png')).convert('RGB') for name in ['before','after']]
    sheet = Image.new('RGB',(left.width+right.width+24,left.height+30),'#14171c')
    draw = ImageDraw.Draw(sheet)
    draw.text((8,7),'Previous warehouse sump',fill='white')
    draw.text((left.width+16,7),'Flooded native rock drift',fill='white')
    sheet.paste(left,(8,24));sheet.paste(right,(left.width+16,24));sheet.save(args.output/'whole-before-after.png')
    receipt = dict(status='PASS',parent_pack=str(parent.relative_to(ROOT)),parent_sha256=sha(parent),
                   pack=str(args.pack),pack_sha256=sha(args.pack),content_identity=new['identity']['content_hash'],
                   unchanged_other_maps=len(new['data']['maps'])-1,original_programs_ports_guards_and_gate_exact=True,
                   all_existing_runtime_files_and_audio_exact=True,native_graphics_and_original_metatiles_exact=True,
                   new_native_quadrant_shores_verified=True,new_runtime_files=sorted(files),guard_parties=parties,
                   active_regional_species=len(allowed),active_evolution_closure=True,fishing_species=sorted(fish))
    (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print('PASS full pack, native shores and regional encounters')


if __name__ == '__main__':
    main()
