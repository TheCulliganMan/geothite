#!/usr/bin/env python3
"""Read-only compiled-pack preservation and native-size art review."""
import argparse
import copy
import hashlib
import json
import sys
from pathlib import Path

import cbor2
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import SourceArt, npc_palette_pack
from review_north_shore import oriented_sprite

parser = argparse.ArgumentParser()
parser.add_argument('before', type=Path)
parser.add_argument('after', type=Path)
parser.add_argument('output', type=Path)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=False)
read = lambda path: cbor2.loads(path.read_bytes()[22:])
before, after = read(args.before), read(args.after)
name = 'MnR09Cave1F'
a, b = before['data']['maps'][name], after['data']['maps'][name]
assert a['blocks'] != b['blocks']
assert {k:v for k,v in a.items() if k != 'blocks'} == {k:v for k,v in b.items() if k != 'blocks'}
label = a['attributes']['blocks_label']
comparison = copy.deepcopy(after)
comparison['identity'] = before['identity']
comparison['data']['maps'][name]['blocks'] = a['blocks']
comparison['data']['map_blocks'][label] = before['data']['map_blocks'][label]
assert comparison == before, 'Change outside mine blocks/payload/identity'
# Entrances, including deliberately gated drill warps, retain exact geometry.
col = before['data']['tilesets']['cave']['collision']
def kind(m,x,y):
    return col[f"{m['blocks'][y//2*12+x//2]:02x}"][y%2*2+x%2]
for warp in a['events']['warps']:
    x,y=warp['x'],warp['y']
    assert kind(a,x,y)==kind(b,x,y)
    target=next(m for key,m in after['data']['maps'].items() if key==warp['target_map'] or m['attributes'].get('map_constant')==warp['target_map'])
    dest=next(w for w in target['events']['warps'] if w['index']==warp['target_warp_id'])
    assert dest['target_map'] in [name, a['attributes'].get('map_constant')]
for x in range(24):
    assert kind(b,x,0)=='WALL' and kind(b,x,21)=='WALL'
for y in range(22):
    assert kind(b,0,y)=='WALL' and kind(b,23,y)=='WALL'
# The jumped-over wall course is continuous, including both flanks.
assert all(kind(b,x,13)=='WALL' for x in range(15,19))
views=[('nell-and-upper-rim',(18,9)),('drop-and-lower-drift',(16,14)),('arrival-junction',(12,17))]
whole=[]
for title,pack in [('before',before),('after',after)]:
    m=pack['data']['maps'][name]
    im=SourceArt(pack).render('cave',12,11,m['blocks'],group='nite').convert('RGBA')
    sprites=npc_palette_pack(pack,'nite')
    for obj in m['objects']:
        frame=oriented_sprite(sprites,obj['sprite'],'south')
        im.alpha_composite(frame,(obj['x']*16,obj['y']*16-4))
    im=im.convert('RGB');im.save(args.output/f'{title}-whole-native.png');whole.append(im)
    for label,(x,y) in views:
        left=max(0,min(im.width-160,x*16-72));top=max(0,min(im.height-144,y*16-64))
        im.crop((left,top,left+160,top+144)).save(args.output/f'{title}-{label}-160x144.png')
out=Image.new('RGB',(768,376),'#eee');d=ImageDraw.Draw(out)
for i,im in enumerate(whole):out.paste(im,(i*384,24));d.text((i*384+8,6),['Before','Lower return drift'][i],fill='black')
out.save(args.output/'whole-comparison.png')
report={'before_sha256':hashlib.sha256(args.before.read_bytes()).hexdigest(),'after_sha256':hashlib.sha256(args.after.read_bytes()).hexdigest(),'changed_maps':[name],'changed_blocks':sum(x!=y for x,y in zip(a['blocks'],b['blocks'])),'preserved':'All other maps, events, scripts, actors, trainers, encounters, roster, audio and runtime files exactly equal','entrances':'All three warp locations/collisions and reciprocal destinations preserved','boundary':'Enclosed; upper rim and drop flanks verified','play':'No game session or controller executed'}
(args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
