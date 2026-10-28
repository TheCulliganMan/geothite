#!/usr/bin/env python3
"""Read-only whole-floor art and stateful cross-floor collision review."""
import argparse
from collections import deque
import json
from pathlib import Path
import time

import cbor2
from PIL import Image, ImageDraw
from render_reference import SourceArt
from render_archive import save_render

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'planning/minnesota/lairs/ice-headquarters'


def review(path):
    # The Rust publisher/runtime validates this external pack. This reader
    # only previews its native art and collision; it never exports a pack.
    pack = cbor2.loads(Path(path).read_bytes()[22:])
    network = json.loads((SOURCE / 'network.json').read_text())
    native_panels = 'mn_ice_headquarters' in pack['data']['tilesets']
    if native_panels:
        assert pack['data']['tilesets']['mn_ice_headquarters']['collision']['40'] == ['FLOOR','WARP_PANEL','FLOOR','FLOOR']
    rooms = {r['id']: r for r in [json.loads((SOURCE / (n+'.json')).read_text())
             for n in ['operations','records','maintenance','transfer']]}
    for name in ['MnIceCargoBasement','MnIceRelayFloor']:
        m = pack['data']['maps'][name]
        rooms[name] = dict(id=name, tileset=m['attributes']['tileset_name'],
            dimensions_metatiles=[m['attributes']['width'],m['attributes']['height']],
            rows=[' '.join(f'{b:02x}' for b in m['blocks'])])
    blocks = {k:[int(x,16) for row in r['rows'] for x in row.split()] for k,r in rooms.items()}
    cargo = json.loads((SOURCE.parent/'minneapolis-cargo.json').read_text())
    for change in cargo['block_changes']:
        x,y = change['metatile'];w=rooms['MnIceCargoBasement']['dimensions_metatiles'][0]
        blocks['MnIceCargoBasement'][y*w+x] = int(change['open'],16)
    ports = {};arrivals=[]
    for link in network['links']:
        for a,b in [(link['a'],link['b']),(link['b'],link['a'])]:
            name=a['map'];x,y=a['active'];w,h=rooms[name]['dimensions_metatiles']
            if name in network['maps']:
                assert blocks[name][y//2*w+x//2] == 7,(name,link['id'],'port overlaps wall')
                blocks[name][y//2*w+x//2] = 0x1b if link['kind']=='panel' else 4
                if native_panels:
                    actual=pack['data']['maps'][name]
                    assert actual['attributes']['tileset_name']=='mn_ice_headquarters'
                    assert actual['blocks'][y//2*w+x//2] == (0x40 if link['kind']=='panel' else 4)
            # Safe arrivals are ordinary floor, not extra active teleporters.
            key=(name,*a['active']);assert key not in ports,('overlapping warp',key)
            ports[key]=(b['map'],*b['arrival'],link['kind'])
            arrivals.append((name,*a['arrival']))
            if native_panels:
                origin=pack['data']['maps'][name];dest=pack['data']['maps'][b['map']]
                for tile in [a['active'],a['arrival']]:
                    warp=next(w for w in origin['events']['warps'] if [w['x'],w['y']]==tile)
                    assert warp['target_map_constant']==dest['attributes']['map_constant']
                    landing=dest['events']['warps'][warp['target_warp_id']-1]
                    assert [landing['x'],landing['y']]==b['arrival'],('compiled landing differs',name,link['id'])
    def floor(name,x,y,opened):
        w,h=rooms[name]['dimensions_metatiles']
        if not (0<=x<2*w and 0<=y<2*h):return False
        block=blocks[name][y//2*w+x//2]
        if opened and any(d['map']==name and d['metatile']==[x//2,y//2] for d in network['doors']):block=7
        collision=pack['data']['tilesets'][rooms[name]['tileset']]['collision']
        return collision[f'{block:02x}'][y%2*2+x%2] in ['FLOOR','LADDER','WARP_CARPET_DOWN']
    for origin, (dest,x,y,kind) in ports.items():
        assert floor(dest,x,y,False),('unsafe pre-callback arrival',origin,dest,x,y)
    def reachable(start,opened=False,panels=True):
        todo=deque([start]);seen={start}
        while todo:
            name,x,y=todo.popleft()
            for xx,yy in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if not floor(name,xx,yy,opened):continue
                at=(name,xx,yy)
                if at in ports:
                    dest,tx,ty,kind=ports[at]
                    if kind=='panel' and not panels:continue
                    at=(dest,tx,ty)
                if at not in seen:seen.add(at);todo.append(at)
        return seen
    start=('MnIceOperations',4,19)
    before=reachable(start);after=reachable(start,True);stairs=reachable(start,False,False)
    ada=('MnIceRecords',24,5);relay=('MnIceRelayFloor',4,15)
    assert ada in before and ada not in stairs,'Ada must require cross-floor pads'
    assert relay not in before and relay in after,'card must gate the final relay'
    for name,at in [('MnIceRecords',(24,17)),('MnIceMaintenance',(24,7)),('MnIceTransfer',(24,5))]:
        assert (name,*at) not in before and (name,*at) in after,('optional room gate',name)
    # Every reachable compartment can get home, including deceptive pad loops.
    for point in arrivals:
        if point in after:assert start in reachable(point,True),('return trap',point)
    print(f'PASS: {len(network["links"])} paired connections (8 pad pairs), six card doors; no stair-only card route; relay sealed until card; three optional rooms; all returns.')
    output=ROOT/'planning/minnesota/renders/story-events-v2'/f'ice-headquarters-{time.time_ns()}'
    output.mkdir(parents=True)
    art=SourceArt(pack);sheet=Image.new('RGB',(1024,820),'#17191c');draw=ImageDraw.Draw(sheet)
    for i,name in enumerate(network['maps']):
        room=rooms[name];w,h=room['dimensions_metatiles']
        img=art.render('facility',w,h,blocks[name],'day');d=ImageDraw.Draw(img)
        for link in network['links']:
            for a in [link['a'],link['b']]:
                if a['map']==name:
                    x,y=a['active'];d.text((x*16,y*16),link['id'] if link['kind']=='panel' else 'S',fill='#ff4040',stroke_width=1,stroke_fill='black')
        save_render(img,output/f'{name}-closed.png')
        x=i%2*512;y=i//2*410;sheet.paste(img,(x,y+24));draw.text((x+8,y+6),name,fill='white')
    save_render(sheet,output/'whole-complex.png')
    print(output)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--pack',required=True)
    review(parser.parse_args().pack)
