#!/usr/bin/env python3
"""Bind frozen winter review cameras to an actual pack without running captures."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack


def bind(pack,views):
    for view in views:
        module=pack['data']['maps'][view['map']];a=module['attributes'];x,y=view['core']
        assert 0<=x<a['width']*2 and 0<=y<a['height']*2,view['id']
        assert a['tileset_name']==view['expected_tileset'],view['id']+' alias'
        block=module['blocks'][y//2*a['width']+x//2]
        if 'expected_block' in view:assert f'{block:02x}'==view['expected_block'],view['id']+' block'
        collision=pack['data']['tilesets'][a['tileset_name']]['collision'][f'{block:02x}'][(y%2)*2+x%2]
        assert collision==view['expected_collision'],view['id']+' collision'
        assert not any(o['x']==x and o['y']==y for o in module['objects']),view['id']+' occupied'
    return views


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--pack',type=Path,required=True);p.add_argument('--mode',choices=['baseline','winter'],required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    source=Path(__file__).parent/'native-review-plan-v01.json';plan=json.loads(source.read_text());raw=args.pack.read_bytes();sha=hashlib.sha256(raw).hexdigest()
    if args.mode=='baseline':assert sha==plan['baseline_pack_sha256']
    selected=bind(load_pack(args.pack),plan['native_baseline_views' if args.mode=='baseline' else 'winter_views'])
    args.output.mkdir(parents=True,exist_ok=False)
    (args.output/'views.json').write_text(json.dumps(selected,indent=2)+'\n')
    (args.output/'binding.json').write_text(json.dumps({'status':'CAMERA_GEOMETRY_BOUND_NOT_CAPTURED','pack_sha256':sha,'plan_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'mode':args.mode,'views':len(selected)},indent=2)+'\n')
