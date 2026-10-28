#!/usr/bin/env python3
"""Retain actual North Shore buoy pixels and a conservative local launch witness."""
import argparse
from collections import Counter, deque
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image, ImageDraw
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack
from audit_outdoor_connections import collision,project,OPPOSITE
from audit_reachable_surf_boundaries import permits,DIRECTIONS,FOOT
from render_archive import save_render
from render_progress_atlas import font


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--atlas',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
    sha=hashlib.sha256(args.pack.read_bytes()).hexdigest();pack=load_pack(args.pack)
    names=[f'MnR11C0{x}R0{y}' for y in range(4) for x in range(2)]
    maps={n:pack['data']['maps'][n] for n in names};k={n:collision(pack,m) for n,m in maps.items()}
    occ={n:{(o['x'],o['y']) for o in m['objects']} for n,m in maps.items()}
    edges={(n,e['direction']):e for n,m in maps.items() for e in m['attributes']['connections'] if e['target_map'] in maps}
    def step(state,d):
        n,p=state;dx,dy=DIRECTIONS[d];q=p[0]+dx,p[1]+dy
        if q in k[n]:return n,q
        e=edges.get((n,d))
        if e is None:return None
        t=e['target_map'];q=project(maps[n]['attributes'],maps[t]['attributes'],e,p);inv=edges.get((t,OPPOSITE[d]))
        if q not in k[t] or inv is None or inv['target_map']!=n or project(maps[t]['attributes'],maps[n]['attributes'],inv,q)!=p:return None
        return t,q
    seeds={(n,(w['x'],w['y'])) for n,m in maps.items() for w in m['events']['warps'] if k[n].get((w['x'],w['y'])) in FOOT and (w['x'],w['y']) not in occ[n]}
    foot={s:None for s in seeds};queue=deque(sorted(seeds))
    while queue:
        s=queue.popleft()
        for d in DIRECTIONS:
            t=step(s,d)
            if t and t not in foot and k[t[0]][t[1]] in FOOT and t[1] not in occ[t[0]]:foot[t]=s;queue.append(t)
    target=('MnR11C01R01',(46,24));water={target:None};queue=deque([target]);bank=None
    while queue and bank is None:
        s=queue.popleft()
        for d,(dx,dy) in DIRECTIONS.items():
            t=step(s,d)
            if t is None:continue
            if t in foot and s[0]==t[0] and k[s[0]][s[1]]=='WATER' and k[t[0]][t[1]] in {'FLOOR','TALL_GRASS'}:bank=(t,s);break
            if t not in water and t[1] not in occ[t[0]] and permits(k[s[0]][s[1]],dx,dy) and permits(k[t[0]][t[1]],dx,dy):water[t]=s;queue.append(t)
    assert bank,'No conservative local launch witness'
    def path(parents,s):
        result=[]
        while s is not None:result.append({'map':s[0],'core':list(s[1])});s=parents[s]
        return result
    floor_path=list(reversed(path(foot,bank[0])));water_path=path(water,bank[1])
    args.output.mkdir(parents=True,exist_ok=False)
    coast=Image.new('RGB',(1792,3072));records=[]
    for row in range(4):
        for col in range(2):
            n=f'MnR11C0{col}R0{row}';im=Image.open(args.atlas/'maps'/n/'scene-rgb.png').convert('RGB');assert im.size==((1024,768) if col==0 else (768,768));coast.paste(im,(0 if col==0 else 1024,row*768))
        n=f'MnR11C01R0{row}';m=maps[n];a=m['attributes'];assert a['width']==24 and a['height']==24
        tokens=[k[n][47,y] for y in range(48)];assert set(tokens)=={'BUOY'}
        records.append({'map':n,'outer_core_x':47,'core_y_inclusive':[0,47],'tokens':dict(Counter(tokens)),'outer_metatile_column':23,'block_ids':dict(Counter(f'{m["blocks"][y*24+23]:02x}' for y in range(24)))})
    save_render(coast,args.output/'north-shore-whole-original.png')
    # Exact coast pixels at the eastern guard, including two joins and endcaps.
    boxes={'east-native-buoys':(1696,1120,1792,1280),'east-internal-seam':(1696,704,1792,832),'northeast-endcap':(1696,0,1792,128),'southeast-endcap':(1696,2944,1792,3072)}
    board=Image.new('RGB',(1800,830),'#111e29');d=ImageDraw.Draw(board);d.text((24,18),'ACTUAL NORTH SHORE · VISIBLE NATIVE BUOY CLOSURE',font=font(24,True),fill='white');d.text((24,54),f'pack {sha[:16]} · exact pixels; 4× nearest-neighbor crops · no invented border art',font=font(18),fill='#bdd8d4')
    for i,(name,box) in enumerate(boxes.items()):
        crop=coast.crop(box);save_render(crop,args.output/(name+'-original.png'));zoom=crop.resize((crop.width*4,crop.height*4),Image.Resampling.NEAREST);save_render(zoom,args.output/(name+'-4x.png'));x=24+i*440;board.paste(zoom,(x,130));d.text((x,96),name,font=font(17,True),fill='white')
    d.text((24,790),'Outer x47: BUOY on all 192 east-edge core cells. Water x46 lies inside the visible float line.',font=font(20),fill='#e7ca91');save_render(board,args.output/'visible-boundary-review.png')
    report={'status':'STATIC_NATIVE_GEOMETRY_AND_PIXELS_NOT_CONTROLLER_PLAY','pack_sha256':sha,'east_guard':records,
        'native_art_note':'Native white float/rock-shaped BUOY glyphs; no Water-looking WALL used on this east guard.',
        'mount_witness':{'floor':floor_path[-1],'water':water_path[0],'foot_from_actual_outdoor_warp':floor_path,'surf_to_inside_buoy':water_path,'inside_buoy':{'map':target[0],'core':list(target[1])},'blocked_outward_neighbor':{'map':target[0],'core':[47,24],'collision':k[target[0]][47,24]}},
        'limits':['Ordinary local geometry from a real room warp, not earned story access or executed inputs.','Foot search excludes Cut, ledges and sliding ICE and treats current actors as occupied.','Native WATERFALL/bridge direction permissions respected in the water witness.']}
    (args.output/'proof.json').write_text(json.dumps(report,indent=2)+'\n');assert hashlib.sha256(args.pack.read_bytes()).hexdigest()==sha
    print(json.dumps({'bank':report['mount_witness']['floor'],'water':report['mount_witness']['water'],'foot_steps':len(floor_path)-1,'water_steps':len(water_path)-1,'east_buoy_cells':192}))


if __name__=='__main__':main()
