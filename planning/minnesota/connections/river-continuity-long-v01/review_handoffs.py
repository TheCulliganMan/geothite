#!/usr/bin/env python3
"""Retained native-art source cameras for river handoffs and quiet weak reaches."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image
ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT/'planning/minnesota/tools'))
from audit_outdoor_connections import collision
from render_progress_atlas import native_room
from render_reference import load_pack, room_sheet
from review_north_shore import oriented_sprite


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'Retain every prior render epoch.'
    args.output.mkdir(parents=True)
    contract = json.loads((args.candidate/'candidate.contract.json').read_text())
    assert hashlib.sha256(args.pack.read_bytes()).hexdigest() == contract['parent_pack_sha256']
    pack = load_pack(args.pack)
    for entry in contract['changes']:
        module = deepcopy(pack['data']['maps'][entry['map']])
        width = module['attributes']['width']
        for cell in entry['cells']:
            x, y = cell['metatile']
            assert module['blocks'][y*width+x] == int(cell['before'], 16)
            module['blocks'][y*width+x] = int(cell['after'], 16)
        pack['data']['maps'][entry['map']] = module
    placements = json.loads((ROOT/'planning/minnesota/renders/ice/native-geography-v03/atlas/native-connection-audit.json').read_text())['placements']
    by_name = {row['map']: row for row in placements}
    cache = {}
    def frame(name, point, facing):
        ox, oy = by_name[name]['origin_metatiles']
        x, y = ox*32+point[0]*16, oy*32+point[1]*16
        bounds = (x-72, y-64, x+88, y+80)
        image = Image.new('RGBA', (160,144), '#121f2b')
        for row in placements:
            mx,my = row['origin_metatiles']
            w,h = row['dimensions_metatiles']
            if mx*32 >= bounds[2] or (mx+w)*32 <= bounds[0] or my*32 >= bounds[3] or (my+h)*32 <= bounds[1]:
                continue
            mid = row['map']
            if mid not in cache:
                cache[mid] = native_room(pack['data']['maps'][mid], pack, native_origin=True).convert('RGBA')
            image.alpha_composite(cache[mid], (mx*32-bounds[0], my*32-bounds[1]))
        permission = collision(pack,pack['data']['maps'][name])[point]
        assert permission == 'WATER', (name, point, permission)
        image.alpha_composite(oriented_sprite(pack,'SPRITE_SURF',facing),(72,64))
        return image
    views = [
        ('st-cloud-r05-to-upper-woods','MnMississippiUpstreamWoodsC00R00',(2,40),'east'),
        ('upper-woods-to-minneapolis-bank','MnMississippiUpstreamWoodsC00R02',(2,36),'west'),
        ('minneapolis-river-arrival','MnR05C00R02',(16,15),'south'),
        ('st-paul-to-lower-woods','MnMississippiLowerWoodsC00R01',(2,24),'east'),
        ('lower-woods-to-winona-mouth','MnMississippiLowerWoodsC00R02',(36,39),'south'),
        ('weak-upper-quiet-water','MnMississippiUpstreamWoodsC01R01',(48,34),'north'),
        ('weak-lower-still-water','MnMississippiLowerWoodsC01R00',(4,34),'east'),
        ('ordinary-lower-wooded-descent','MnMississippiLowerWoodsC00R02',(44,19),'south'),
    ]
    images=[]
    for label,name,point,facing in views:
        image = frame(name,point,facing)
        image.save(args.output/f'{label}-native160.png')
        image.resize((640,576),Image.Resampling.NEAREST).save(args.output/f'{label}-4x.png')
        images.append((label,image.convert('RGB')))
    room_sheet(images,columns=4).save(args.output/'handoffs-and-weak-reaches.png')
    (args.output/'receipt.json').write_text(json.dumps(dict(status='SOURCE_ART_ONLY_NOT_GPU_OR_REACHED_INPUT',parent_pack_sha256=contract['parent_pack_sha256'],contract_sha256=hashlib.sha256((args.candidate/'candidate.contract.json').read_bytes()).hexdigest(),views=[dict(label=l,map=n,local_core=list(p),facing=f)for l,n,p,f in views],meaning='Actual current native positions and art. No fabricated transition terrain. Cameras are centered source crops, not production camera-clamp evidence.'),indent=2)+'\n')

if __name__ == '__main__':
    main()
