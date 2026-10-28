#!/usr/bin/env python3
"""Whole-map native-art inspection for the detached inland winter proposal.

Never writes a pack or canonical authoring. Every output epoch is immutable.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import sys

from PIL import Image

ROOT=Path(__file__).resolve().parents[5]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_archive import save_render
from render_reference import load_pack,room_sheet
from review_major_city import ReviewArt,render_map


def sha(raw):return hashlib.sha256(raw).hexdigest()


def whole(pack,prefix):
    maps={n:m for n,m in pack['data']['maps'].items()if re.fullmatch(prefix+r'C\d+R\d+',n)}
    placements=[]
    for name,m in sorted(maps.items()):
        c,r=map(int,re.fullmatch(prefix+r'C(\d+)R(\d+)',name).groups())
        placements.append(dict(map=name,origin=[c*32,r*24],dimensions=[m['attributes']['width'],m['attributes']['height']]))
    width=max(r['origin'][0]+r['dimensions'][0]for r in placements)
    height=max(r['origin'][1]+r['dimensions'][1]for r in placements)
    blocks=[None]*(width*height);objects=[]
    for row in placements:
        m=maps[row['map']];x0,y0=row['origin'];w,h=row['dimensions']
        for y in range(h):blocks[(y+y0)*width+x0:(y+y0)*width+x0+w]=m['blocks'][y*w:(y+1)*w]
        for actor in m['objects']:
            objects.append(dict(id=actor.get('object_identifier')or actor['script'],sprite=actor['sprite'],
                tile=[x0*2+actor['x'],y0*2+actor['y']],
                facing={'UP':'north','DOWN':'south','LEFT':'west','RIGHT':'east'}.get(actor['spritemovedata'].rsplit('_',1)[-1],'south'),
                native_palette_override=actor.get('pal',0)))
    assert None not in blocks
    sample=next(iter(maps.values()))
    return dict(id=prefix,attributes=dict(sample['attributes'],width=width,height=height),blocks=blocks),objects,placements


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--expected-sha256',required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();assert not args.output.exists()
    assert sha(args.pack.read_bytes())==args.expected_sha256
    pack=load_pack(args.pack);args.output.mkdir(parents=True)
    art=ReviewArt(pack);report={};contact=[]
    for prefix in ['MnR12','MnR13']:
        m,objects,rows=whole(pack,prefix)
        _,scene,_=render_map(art,m,objects,'day',args.output/prefix/'actual-warm')
        scale=min(1,850/scene.width,1300/scene.height)
        display=scene.resize((round(scene.width*scale),round(scene.height*scale)),Image.Resampling.NEAREST)
        save_render(display,args.output/prefix/'whole-warm-display.png');contact.append((prefix+' actual native warm baseline',display))
        report[prefix]=dict(dimensions_metatiles=[m['attributes']['width'],m['attributes']['height']],
            alias=m['attributes']['tileset_name'],native_block_counts={f'{b:02x}':n for b,n in sorted(Counter(m['blocks']).items())},
            native_modules_sha256={r['map']:sha(json.dumps(pack['data']['maps'][r['map']],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())for r in rows},
            actors=len(objects),placements=rows)
    save_render(room_sheet(contact,2),args.output/'whole-inland-warm-baselines.png')
    (args.output/'receipt.json').write_text(json.dumps(dict(status='ACTUAL_NATIVE_WARM_SOURCE_ART_ONLY_NOT_GPU',
        pack_sha256=args.expected_sha256,identity=pack['identity']['content_hash'],regions=report,
        tool_sha256=sha(Path(__file__).read_bytes())),indent=2)+'\n')
    (args.output/'review_inland.snapshot.py').write_bytes(Path(__file__).read_bytes())
    print(json.dumps({p:r['dimensions_metatiles']for p,r in report.items()}))


if __name__=='__main__':main()
