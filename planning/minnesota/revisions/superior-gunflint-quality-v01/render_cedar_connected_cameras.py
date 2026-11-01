#!/usr/bin/env python3
"""Retain four centered Cedar source-art cameras with real connected neighbors.

Camera-only evidence: no authored terrain, native pack, gameplay state or GPU
capture is changed. Original clamped source views remain immutable history.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import SourceArt, load_pack
from review_city_native import priority_scene
from review_north_shore import oriented_sprite
from review_passage_native import actors


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--expected-sha256', required=True)
    parser.add_argument('--native-camera-epoch', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'choose a fresh immutable camera-only epoch'
    assert sha(args.pack) == args.expected_sha256
    pack = load_pack(args.pack)
    plan = json.loads((args.native_camera_epoch / 'camera-plan.json').read_bytes())
    views = [v for v in plan['cameras'] if v['name'].startswith('cedar-')]
    assert len(views) == 4 and all(v['map'] == 'MnMaraisCedarApproach' and v['hour'] == 12 for v in views)
    base = pack['data']['maps']['MnMaraisCedarApproach']
    attrs = base['attributes']
    assert [attrs['width'], attrs['height']] == [20,12]
    art = SourceArt(pack)
    assert attrs['border_block'] == 0x8b, ('actual native border differs', attrs['border_block'])
    border = art.blocks(attrs['tileset_name'])[attrs['border_block']]
    terrain = art.render(attrs['tileset_name'],attrs['width'],attrs['height'],base['blocks'])
    regions = []
    for edge in attrs['connections']:
        neighbor = pack['data']['maps'][edge['target_map']]
        a = neighbor['attributes']
        offset = edge['offset']*32
        origin = dict(west=(-a['width']*32,offset), east=(attrs['width']*32,offset),
                      north=(offset,-a['height']*32), south=(offset,attrs['height']*32))[edge['direction']]
        image = art.render(a['tileset_name'],a['width'],a['height'],neighbor['blocks'])
        regions.append((neighbor['id'],origin,image))
    assert {(n,o) for n,o,_ in regions} == {('MnR12C00R01',(-1024,128)),('MnGrandMarais',(128,384))}
    args.output.mkdir(parents=True)
    receipt = dict(schema_version=1,pack_sha256=args.expected_sha256,
                   scope='Fixed centered160x144 native source art, same noon coordinates and South-facing player; direct actual neighbors only. No camera clamping, diagonal terrain, GPU modification or gameplay acceptance claim.',
                   script_sha256=sha(Path(__file__)),native_camera_plan_sha256=sha(args.native_camera_epoch/'camera-plan.json'),
                   native_border_block='8b',neighbors=[dict(map=n,origin_pixels=list(o))for n,o,_ in regions],cameras=[])
    for view in views:
        x,y=view['x'],view['y']
        box=(x*16-80,y*16-72,x*16+80,y*16+72)
        camera=Image.new('RGBA',(160,144))
        # Preserve the current map's actual metatile modulo phase outside its
        # rectangle. Padding does not invent a missing or diagonal map.
        for yy in range(box[1]//32*32,box[3],32):
            for xx in range(box[0]//32*32,box[2],32):
                camera.alpha_composite(border.convert('RGBA'),(xx-box[0],yy-box[1]))
        visible=[]
        for name,(ox,oy),image in regions:
            if ox < box[2] and oy < box[3] and ox+image.width > box[0] and oy+image.height > box[1]:
                camera.alpha_composite(image.convert('RGBA'),(ox-box[0],oy-box[1]))
                visible.append(name)
        active=actors(pack,terrain,base)
        active.alpha_composite(oriented_sprite(pack,'SPRITE_CHRIS','south'),(x*16,y*16))
        foreground_pixels=priority_scene(pack,base,active,terrain,box)
        camera.alpha_composite(active,(-box[0],-box[1]))
        destination=args.output/(view['name']+'-source160-original.png')
        camera.convert('RGB').save(destination)
        native=args.native_camera_epoch/view['name']/'native-2d-original.png'
        receipt['cameras'].append(dict(name=view['name'],map=view['map'],core=[x,y],hour=12,
            player_facing='south',crop_box_pixels=list(box),player_sprite_origin_in_camera=[80,72],
            visible_direct_neighbor_terrain=visible,active_npc_count=len(base['objects']),
            priority_masked_pixels=foreground_pixels,source=str(destination),source_sha256=sha(destination),
            native_original=str(native),native_original_sha256=sha(native)))
    (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(dict(output=str(args.output),cameras=len(views),pack_sha256=args.expected_sha256),indent=2))


if __name__ == '__main__':
    main()
