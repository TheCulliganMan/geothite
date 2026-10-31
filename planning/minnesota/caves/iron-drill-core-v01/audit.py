#!/usr/bin/env python3
"""Read-only whole-pack/native-machine preservation and scene-path audit."""
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
from PIL import Image,ImageChops,ImageDraw

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt,npc_palette_pack
from review_passage_native import actors


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True);parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    source=Path(__file__).parent;contract=json.loads((source/'contract.json').read_text());spec=json.loads((source/'after.json').read_text());parent=ROOT/contract['parent_pack'];assert sha(parent)==contract['parent_pack_sha256']
    old,new=[cbor2.loads(p.read_bytes()[22:])for p in [parent,args.pack]];room='MnIronDrillCore';base,alias='mn_iron_haulage_rock','mn_iron_core_rock';before,after=[p['data']['maps'][room]for p in [old,new]]
    assert hashlib.sha256(json.dumps(before,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()==contract['native_module_sha256']
    for module,name in [(before,'before.json'),(after,'after.json')]:assert module['blocks']==[int(v,16)for row in json.loads((source/name).read_text())['rows']for v in row.split()]
    expected=deepcopy(old);expected['identity']=new['identity'];target=expected['data']['maps'][room];target['blocks']=after['blocks']
    for key,value in dict(tileset_name=alias,border_block=46,time_of_day='nite',palette='PALETTE_NITE',fishing_group='FISHGROUP_MN_LAKE').items():target['attributes'][key]=value;expected['data']['map_attributes'][room][key]=value
    for x in [19,20]:
        label=f'MnNDrillWatchFrom{x}';directions=['DOWN']+['LEFT']*(x-9)+['UP']*4;commands=[dict(command='step',args=[v])for v in directions]+[dict(command='step_end',args=[])]
        target['scripts'][label]=commands;expected['data']['map_scripts'][label]=commands;definition=next(v for v in target['script_movements']if v['label']==label);definition['steps']=[dict(command='step',direction=v,duration=None,index=i)for i,v in enumerate(directions)]+[dict(command='step_end',direction=None,duration=None,index=len(directions))]
    label=before['attributes']['blocks_label'];expected['data']['map_blocks'][label]=new['data']['map_blocks'][label];assert base64.b64decode(expected['data']['map_blocks'][label])==bytes(after['blocks'])
    definition=deepcopy(old['data']['tilesets'][base]);definition['palette_map']=new['data']['tilesets'][alias]['palette_map']
    for id,collision in {'12':['WALL']*4,'16':['WALL']*4,'43':['WALL']*4,'44':['WALL']*4,'45':['WALL']*4,'46':['WALL','WALL','FLOOR','FLOOR']}.items():definition['collision'][id]=collision
    assert definition==new['data']['tilesets'][alias];expected['data']['tilesets'][alias]=definition
    files={f'data/tilesets/{alias}.json',f'data/tilesets/{alias}_palette_map.json',f'data/tilesets/{alias}_metatiles.bin',f'gfx/tilesets/{alias}.png',f'gfx/tilesets/{alias}.2bpp'};assert set(new['runtime_files'])-set(old['runtime_files'])==files
    for file in files:expected['runtime_files'][file]=new['runtime_files'][file]
    expected['report']['manifests'].extend(['minnesota-iron-core-native-rock-v1-expanded-city-art','minnesota-iron-core-rock-chamber-v1']);assert expected==new,'unowned whole-pack content changed'
    art=SourceArt(new);original,private=art.blocks(base,'nite'),art.blocks(alias,'nite')
    for i in range(0x43):
        if i not in {0x12,0x16}:assert ImageChops.difference(original[i],private[i]).getbbox()is None,f'private original block changed: {i:02x}'
    for to,src in [(0x12,0x12),(0x16,0x16),(0x43,0x13),(0x44,0x12),(0x45,0x16)]:assert ImageChops.difference(art.blocks('facility','nite')[src],private[to]).getbbox()is None,'native machine pixels/palettes differ'
    control=art.blocks('facility','nite')[0x28].copy();control.paste(original[13].crop((0,24,32,32)),(0,24));assert ImageChops.difference(control,private[0x46]).getbbox()is None,'console landing recipe differs'
    assert json.loads(bytes(new['runtime_files'][f'data/tilesets/{alias}.json']))==definition['collision'];assert json.loads(bytes(new['runtime_files'][f'data/tilesets/{alias}_palette_map.json']))==definition['palette_map']
    bodies={(o['x'],o['y'])for o in after['objects']};kind=lambda at:definition['collision'][f'{after["blocks"][at[1]//2*12+at[0]//2]:02x}'][at[1]%2*2+at[0]%2]
    floor={(x,y)for y in range(20)for x in range(24)if kind((x,y))=='FLOOR'and(x,y)not in bodies};seen={(4,15)};queue=deque(seen)
    while queue:
        x,y=queue.popleft()
        for at in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
            if at in floor and at not in seen:seen.add(at);queue.append(at)
    assert seen==floor
    traces=[]
    for name,start in [(f'MnNDrillWatchFrom{x}',(x,12))for x in range(3,21)if x!=18]+[('MnNDrillPlayerToCrew',(16,3)),('MnNDrillCrewOut',(15,9))]:
        at=start;path=[at]
        for cmd in after['scripts'][name]:
            if cmd['command']=='step_end':continue
            dx,dy={'UP':(0,-1),'DOWN':(0,1),'LEFT':(-1,0),'RIGHT':(1,0)}[cmd['args'][0]];at=(at[0]+dx,at[1]+dy);assert kind(at)=='FLOOR'and at not in bodies,(name,at);path.append(at)
        assert at==((8,7)if name=='MnNDrillCrewOut'else(9,9));traces.append(dict(script=name,path=path))
    roster=json.loads((ROOT/'planning/minnesota/pokemon/roster.json').read_text());curation=json.loads((ROOT/'planning/minnesota/pokemon/roster-curation.json').read_text());retired={f['id']for f in curation['retired_families']};normalize=lambda s:re.sub('[^A-Z0-9]','',s.upper());allowed={normalize(m['name'])for f in roster['families']if f['id']not in retired for m in f['members']};parties={}
    for trainer in ['MN_REED_MINE','MN_LODE_ARC_CORE_WEST','MN_LODE_ARC_CORE_EAST']:
        party=new['data']['trainers']['trainers'][trainer]['party'];assert all(normalize(p['species'])in allowed for p in party);parties[trainer]=[dict(species=p['species'],level=p['level'])for p in party]
    for pokemon,evolutions in new['data']['evolutions'].items():
        if normalize(pokemon)in allowed:assert all(normalize(e['species'])in allowed for e in evolutions)
    for p,m,group,name in [(old,before,'day','before'),(new,after,'nite','after')]:
        m=dict(m);m['objects']=[o for o in m['objects']if o['script']!='MN_NDRILL_FINN'];a=m['attributes'];scene=actors(npc_palette_pack(p,group),SourceArt(p).render(a['tileset_name'],a['width'],a['height'],m['blocks'],group),m);scene.save(args.output/(name+'-original.png'));scene.resize((scene.width*3,scene.height*3),Image.Resampling.NEAREST).save(args.output/(name+'-3x.png'))
    images=[Image.open(args.output/f'{name}-original.png').convert('RGB')for name in ['before','after']];sheet=Image.new('RGB',(images[0].width*2+24,images[0].height+30),'#171d24');draw=ImageDraw.Draw(sheet)
    for i,(label,img)in enumerate(zip(['Previous facility rectangle','Native rock-cut drill bay'],images)):draw.text((8+i*(img.width+8),7),label,fill='white');sheet.paste(img,(8+i*(img.width+8),24))
    sheet.save(args.output/'whole-before-after.png');receipt=dict(status='PASS',parent_pack=str(parent.relative_to(ROOT)),parent_pack_sha256=sha(parent),pack=str(args.pack),pack_sha256=sha(args.pack),content_identity=new['identity']['content_hash'],unchanged_other_maps=len(new['data']['maps'])-1,all_existing_native_files_audio_and_unowned_programs_exact=True,only_two_witness_programs_changed=True,private_machine_frames_and_control_pixels_native=True,all_scene_paths_clear_of_rock_and_actors=traces,native_guard_and_rival_parties=parties,default_crew_hidden=True)
    (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print('PASS whole root, native machine frames and',len(traces),'clear scene paths')


if __name__=='__main__':main()
