#!/usr/bin/env python3
"""Review the explicit Cedar Approach gate using the supplied native source pack."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_archive import save_render
from render_reference import SourceArt, load_pack, room_sheet
from review_boundary_basecamp import flood
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'towns/MnGrandMaraisShoreGate.json'


def validate(pack, source):
    assert pack['identity']['content_hash'] == source['base_content_hash']
    assert source['id'] == 'MnGrandMaraisShoreGate'
    assert source['template'] == 'IlexForestAzaleaGate'
    assert source['dimensions_metatiles'] == [5, 4]
    exterior = pack['data']['maps']['MnMaraisCedarApproach']
    assert exterior['attributes']['connections'] == source['exterior_binding']['retained_connections']
    assert not exterior['events']['warps']
    terrain = list(exterior['blocks'])
    expected = [([19,12],'05','02'),([20,12],'05','08'),([21,12],'05','09'),([22,12],'05','02'),([19,13],'01','0e'),
                ([20,13],'01','10'),([21,13],'01','11'),([22,13],'01','0b')]
    patches = source['exterior_binding']['patches']
    assert [(p['position'],p['before'],p['after']) for p in patches] == expected
    for patch in patches:
        x,y=patch['position'];assert terrain[y*32+x]==int(patch['before'],16)
        terrain[y*32+x]=int(patch['after'],16)
    blocks=[int(b,16) for row in source['rows'] for b in row.split()]
    assert blocks == pack['data']['maps']['IlexForestAzaleaGate']['blocks']
    def ground(tileset, width, height, blocks):
        collision=pack['data']['tilesets'][tileset]['collision']
        def kind(at):
            x,y=at
            assert 0<=x<width*2 and 0<=y<height*2
            return collision[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]
        floor={(x,y) for y in range(height*2) for x in range(width*2)
               if kind((x,y)) in {'FLOOR','TALL_GRASS','WARP_CARPET_LEFT','WARP_CARPET_RIGHT'}}
        return kind,floor
    kind,walk=ground('johto',32,19,terrain)
    bodies={(o['x'],o['y']) for o in exterior['objects']}
    west=flood((38,26),walk-bodies,set());east=flood((45,26),walk-bodies,set())
    assert {(0,12),(0,13),(39,26),(39,27)}<=west and (45,26) not in west
    assert {(56,37),(57,37),(44,26),(44,27)}<=east and (38,26) not in east
    sides=source['exterior_binding']['sides']
    for side,carpet in zip(sides,['WARP_CARPET_RIGHT','WARP_CARPET_LEFT']):
        assert all(kind(tuple(p))==carpet for p in side['entrances'])
        assert all(kind(tuple(p))=='FLOOR' for p in side['returns'])
    kind,walk=ground('gate',5,4,blocks)
    bodies={tuple(a['tile']) for a in source['residents']}
    assert len(bodies)==2
    for arrival in source['passive_arrivals']:
        at=tuple(arrival['tile']);assert kind(at)=='FLOOR' and at not in bodies
        seen=flood(at,walk-bodies,set())
        assert {tuple(e['tile']) for e in source['exits']}<=seen
        for a in source['residents']:
            assert kind(tuple(a['tile']))=='FLOOR' and tuple(a['approach']) in seen
            assert all(0<len(line)<=18 for line in a['lines'])
    assert source['residents'][0]['tile']==[5,2] and kind((5,3))=='COUNTER'
    assert [a['index'] for a in source['passive_arrivals']]==[5,6,7,8]
    assert [e['index'] for e in source['exits']]==[1,2,3,4]
    return terrain,blocks


def review(pack, source, output):
    terrain,blocks=validate(pack,source);art=SourceArt(pack);images=[]
    for name,tileset,width,height,grid,people in [
        ('cedar-approach','johto',32,19,terrain,[]),
        ('shore-gate','gate',5,4,blocks,source['residents']),
    ]:
        image=art.render(tileset,width,height,grid).convert('RGBA')
        save_render(image,output/(name+'-native-terrain.png'))
        for person in people:
            sprite=oriented_sprite(pack,person['sprite'],person['facing'])
            image.alpha_composite(sprite,tuple(c*16 for c in person['tile']))
        save_render(image,output/(name+'-with-residents.png'));images.append((name,image))
    save_render(room_sheet(images,2),output/'whole-place-review.png')
    rejects=0
    for mutate in [lambda v:v['exterior_binding']['patches'][0].update(after='09'),
                   lambda v:v['passive_arrivals'][0].update(tile=[0,4]),
                   lambda v:v['residents'][1].update(tile=[8,5]),
                   lambda v:v['residents'][0].update(tile=[5,3])]:
        bad=deepcopy(source);mutate(bad)
        try:validate(pack,bad)
        except AssertionError:rejects+=1
    assert rejects==4
    print('Whole gate and split approach geometry pass; four unsafe sources rejected.')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    review(load_pack(args.pack),json.loads(SOURCE.read_text()),args.output)
