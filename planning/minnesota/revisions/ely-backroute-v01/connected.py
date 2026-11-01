"""Read-only equal-scale context from actual neighbor maps at their real origins."""
import json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[4]

def connected(pack,art):
    origins={'MnGrandMarais':(226,-400)}
    todo=['MnGrandMarais'];maps=pack['data']['maps']
    while todo:
        n=todo.pop();a=maps[n]['attributes'];x,y=origins[n]
        for link in a['connections']:
            target=link['target_map']
            if target=='MnElyShorePass':continue
            b=maps[target]['attributes'];d=link['direction'];offset=link['offset']
            point={'east':(x+a['width'],y+offset),'west':(x-b['width'],y+offset),
                   'north':(x+offset,y-b['height']),'south':(x+offset,y+a['height'])}[d]
            if target not in origins:origins[target]=point;todo.append(target)
    assert origins['MnEly']==(163,-342) and origins['MnR11C00R02']==(190,-352)
    origins['MnElyShorePass']=[183,-342]
    atlas=Image.new('RGBA',(64*32,34*32),'#17232c')
    for name,(x,y) in origins.items():
        if name not in pack['data']['maps']:continue
        m=pack['data']['maps'][name];a=m['attributes'];w=a['width'];h=a['height']
        if x>=222 or x+w<=158 or y>=-324 or y+h<=-358:continue
        # Crop only to the review bounds; do not stretch or add linking terrain.
        im=art.render(a['tileset_name'],w,h,m['blocks'],'day').convert('RGBA')
        from review_north_shore import oriented_sprite
        for o in m['objects']:
            face={'UP':'north','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south')
            im.alpha_composite(oriented_sprite(pack,o['sprite'],face),(o['x']*16,o['y']*16))
        atlas.alpha_composite(im,((x-158)*32,(y+358)*32))
    return atlas
