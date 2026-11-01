#!/usr/bin/env python3
"""Review the complete native River Bluffs route and wooded terrace candidate."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from render_progress_atlas import native_room
from render_reference import load_pack, room_sheet
from review_r19_prairie_approach import MASKS
from review_selection_lineage import require_selected_ancestor

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'planning/minnesota/routes/r05-wooded-terraces-v01/source-v05.json'
LAND = {'FLOOR', 'TALL_GRASS', 'BRIDGE_EW_DECK', *MASKS}


def graph(pack, modules, hops=False):
    terrain, bodies = {}, set()
    for row, module in enumerate(modules):
        a = module['attributes']
        table = pack['data']['tilesets'][a['tileset_name']]['collision']
        for y in range(a['height']*2):
            for x in range(64):
                terrain[x, y+row*48] = table[f"{module['blocks'][y//2*32+x//2]:02x}"][y%2*2+x%2]
        bodies |= {(o['x'],o['y']+row*48) for o in module['objects']}
    land = LAND | ({'HOP_DOWN'} if hops else set())
    edges = {}
    for at, kind in terrain.items():
        if kind not in land or at in bodies:
            continue
        edges[at] = []
        for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
            q = at[0]+dx, at[1]+dy
            if hops and kind == 'HOP_DOWN' and (dx,dy)==(0,1):
                q=q[0],q[1]+1
            target = terrain.get(q)
            if q not in bodies and target in land and (dx,dy) not in MASKS.get(target,set()) and (-dx,-dy) not in MASKS.get(kind,set()):
                edges[at].append(q)
    return terrain, edges


def distance(edges, start, terrain=None):
    costs, pending = {start:0}, deque([start])
    while pending:
        at = pending.popleft()
        for q in edges.get(at,[]):
            price = int(terrain[q]=='TALL_GRASS') if terrain else 1
            cost = costs[at]+price
            if cost < costs.get(q,10**9):
                costs[q] = cost
                (pending.append if price else pending.appendleft)(q)
    return costs


def compose(pack, source):
    old, new = [], []
    repairs={(r["map"],tuple(r["metatile"])):r for r in source.get("cliff_repairs",[])}
    assert source['route']=='MN_R05' and source['dimensions_metatiles']==[32,56]
    assert not source['source_pixels_modified']
    for row, record in enumerate(source['maps']):
        assert record['id']==f'MnR05C00R{row:02}'
        original=pack['data']['maps'][record['id']]
        assert hashlib.sha256(json.dumps(original,sort_keys=True,separators=(',',':')).encode()).hexdigest()==record['native_module_sha256']
        assert [original['attributes']['width'],original['attributes']['height']]==record['dimensions_metatiles']
        before=[int(b,16) for line in record['before_rows'] for b in line.split()]
        blocks=[int(b,16) for line in record['rows'] for b in line.split()]
        assert original['blocks']==before and len(blocks)==len(before)
        changes=[i for i,(a,b) in enumerate(zip(before,blocks)) if a!=b]
        assert len(changes)==record['changed_metatiles']
        assert all((record["id"],(i%32,i//32)) in repairs or (before[i] in [1,2,3,4,0x71] and blocks[i] in [1,2,3,4,5]) for i in changes)
        assert all((record["id"],(i%32,i//32)) in repairs or (6<=i%32<=20 and (8<=i//32<=19 if row==0 else 3<=i//32<=13)) for i in changes)
        assert row!=2 or not changes
        revised=deepcopy(original);revised['blocks']=blocks
        for (owner,(x,y)),repair in repairs.items():
            if owner==record['id']:
                assert before[y*32+x]==int(repair['before'],16) and blocks[y*32+x]==int(repair['after'],16)
        old.append(original);new.append(revised)
    before, old_edges=graph(pack,old);terrain, edges=graph(pack,new)
    start=(22,106);old_reached=distance(old_edges,start);reached=distance(edges,start)
    for at in [(24,0),(25,0),(22,111),(23,111),(0,104),(0,105),(30,28),(16,78),(25,14),(25,15),(15,40),(15,41),(29,52),(29,53),(13,76),(13,77),(37,39),(37,42),(37,75),(37,78),(51,21),(55,89),(60,89)]:
        assert at in reached,('protected destination unreachable',at)
    for row,module in enumerate(old):
        a=module['attributes']
        for obj in module['objects']+module['events']['bg_events']:
            x,y=obj['x'],obj['y']+row*48
            approaches={(x+dx,y+dy) for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]} & old_reached.keys()
            assert approaches & reached.keys(),('lost original interaction',obj.get('script'))
        for y in range(a['height']*2):
            for x in range(64):
                at=x,y+row*48
                if x in [0,63] or y in [0,a['height']*2-1] or (before[at] not in {'FLOOR','TALL_GRASS'} and (module['id'],(x//2,y//2)) not in repairs):
                    assert before[at]==terrain[at],('protected cliff/stair/shore/perimeter',at)
    prices=[]
    for start,target in [((15,41),(25,14)),((25,14),(15,41)),((13,77),(29,52)),((29,52),(13,77))]:
        a=distance(old_edges,start,before).get(target);b=distance(edges,start,terrain).get(target)
        assert b is not None and b>=a,('lost grass cost',start,target,a,b)
        prices.append(dict(start=start,target=target,before=a,after=b))
    reward=(37,19)
    neighbors=[(reward[0]+dx,reward[1]+dy) for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]]
    old_costs=distance(old_edges,(25,14),before);new_costs=distance(edges,(25,14),terrain)
    old_price=min(old_costs[q] for q in neighbors if q in old_costs)
    new_price=min(new_costs[q] for q in neighbors if q in new_costs)
    assert [old_price,new_price]==[2,4],('Great Ball grass bypass',old_price,new_price)
    hop_terrain,hop_edges=graph(pack,new,hops=True)
    hop_costs=distance(hop_edges,(25,14),hop_terrain)
    hop_price=min(hop_costs[q] for q in neighbors if q in hop_costs)
    assert hop_price>=4,('existing ledge bypassed reward grass',hop_price)
    return old,new,dict(reward_grass_with_native_hop=hop_price,reward_grass_before=old_price,reward_grass_after=new_price,protected_destinations_reachable=True,original_interactions_reachable=True,cliff_repair_cells=len(repairs),unowned_cliffs_all_stairs_shores_perimeters_exact=True,minimum_terrace_grass=prices,changed_metatiles=sum(r['changed_metatiles'] for r in source['maps']))


def main():
    cli=argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--source',type=Path,default=SOURCE)
    cli.add_argument('--output',type=Path,required=True)
    args=cli.parse_args();assert not args.output.exists()
    source=json.loads(args.source.read_text());world=json.loads((ROOT/'planning/minnesota/world.json').read_text())
    require_selected_ancestor(world,'north_shore_passing_width_revision','e951fddafee3a0c09a2356d4dd683890ac498e23daeb266d08c5f12d79ec1167')
    parent=ROOT/source['parent_pack'];assert hashlib.sha256(parent.read_bytes()).hexdigest()==source['parent_pack_sha256']
    pack=load_pack(parent);started=time.perf_counter();old,new,report=compose(pack,source)
    args.output.mkdir(parents=True)
    images=[]
    for name,modules in [('before',old),('after',new)]:
        image=Image.new('RGB',(1024,1792))
        for row,module in enumerate(modules):
            image.paste(native_room(module,pack,archive=args.output/name/module['id'],native_origin=True),(0,row*768))
        save_render(image,args.output/f'{name}-whole-original.png');images.append(image)
        cameras=[]
        for view in source['review_views']:
            x,y=view['core'];crop=image.crop((x*16-80,y*16-72,x*16+80,y*16+72))
            save_render(crop,args.output/name/f"{view['id']}-source160.png");cameras.append((view['id'],crop))
        save_render(room_sheet(cameras,columns=3),args.output/f'{name}-decisions.png')
    combined=Image.new('RGB',(2048,1792));combined.paste(images[0],(0,0));combined.paste(images[1],(1024,0))
    save_render(combined,args.output/'before-after-whole-original.png')
    report.update(status='PASS_SOURCE_ONLY',source=str(args.source),source_sha256=hashlib.sha256(args.source.read_bytes()).hexdigest(),seconds_after_load=round(time.perf_counter()-started,3),scope='Complete native source and directional dry collision review. Native controller, publication and visual acceptance remain separate.')
    (args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))


if __name__=='__main__':
    main()
