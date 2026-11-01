#!/usr/bin/env python3
"""Review Pine Country's complete native route, ledge choices and retained Cut gate."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from render_progress_atlas import native_room
from render_reference import load_pack, room_sheet
from review_r05_wooded_terraces import distance, graph
from review_selection_lineage import require_selected_ancestor

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'planning/minnesota/routes/r06-pine-fork-v01/source-v04.json'


def compose(pack, source):
    assert source['schema_version']==1 and source['route']=='MN_R06'
    assert source['dimensions_metatiles']==[32,44] and not source['source_pixels_modified']
    old,new=[],[]
    repairs={(r['map'],tuple(r['metatile'])):r for r in source['cliff_repairs']}
    for row,record in enumerate(source['maps']):
        assert record['id']==f'MnR06C00R{row:02}'
        m=pack['data']['maps'][record['id']];a=m['attributes']
        assert hashlib.sha256(json.dumps(m,sort_keys=True,separators=(',',':')).encode()).hexdigest()==record['native_module_sha256']
        assert [a['width'],a['height']]==record['dimensions_metatiles']==[32,24 if row==0 else 20]
        assert a['tileset_name']==record['tileset']
        def parse(rows):
            assert len(rows)==a['height'] and all(len(r.split())==32 for r in rows)
            return [int(b,16) for r in rows for b in r.split()]
        before=parse(record['before_rows']);blocks=parse(record['rows']);assert m['blocks']==before
        changes=[i for i,(x,y) in enumerate(zip(before,blocks)) if x!=y]
        assert len(changes)==record['changed_metatiles']
        if row==1:assert not changes
        for i in changes:
            point=i%32,i//32
            if (record['id'],point) in repairs:
                repair=repairs[record['id'],point];assert before[i]==int(repair['before'],16) and blocks[i]==int(repair['after'],16)
            elif point==(21,13):assert before[i]==1 and blocks[i]==0x57
            else:assert before[i] in [1,2,3,0x71] and blocks[i] in [2,3,5] and 9<=point[0]<=27 and 10<=point[1]<=16
        revised=deepcopy(m);revised['blocks']=blocks;old.append(m);new.append(revised)
    before,old_edges=graph(pack,old,hops=True);terrain,edges=graph(pack,new,hops=True)
    _,ordinary=graph(pack,new,hops=False)
    start=(16,86);reached=distance(edges,start);old_reached=distance(old_edges,start)
    for point in [(16,87),(17,87),(42,0),(43,0),(37,40),(9,62),(26,66),(37,66),(26,67),(37,67),(32,43),(36,43)]:
        assert point in reached,('retained route/door/bridge/pier inaccessible',point)
    for row,m in enumerate(old):
        for y in range(m['attributes']['height']*2):
            for x in range(64):
                point=x,y+row*48
                owned=(m['id'],(x//2,y//2)) in repairs or (row==0 and (x//2,y//2)==(21,13))
                if x in [0,63] or y in [0,m['attributes']['height']*2-1] or (before[point] not in ['FLOOR','TALL_GRASS'] and not owned):
                    assert before[point]==terrain[point],('unowned cliff/stair/Cut/water/ledge/perimeter changed',point)
    # The original native 5b -> 3c override retains its other quadrants.
    cut=source['retained_cut_gate'];ox,oy=cut['metatile'];assert new[0]['blocks'][oy*32+ox]==int(cut['original'],16)
    opened_old,opened_new=deepcopy(old),deepcopy(new)
    for modules in [opened_old,opened_new]:modules[0]['blocks'][oy*32+ox]=int(cut['replacement'],16)
    _,old_open_edges=graph(pack,opened_old,hops=True);open_terrain,open_edges=graph(pack,opened_new,hops=True)
    old_open=distance(old_open_edges,start);new_open=distance(open_edges,start)
    assert tuple(cut['inside']) not in old_reached and tuple(cut['inside']) not in reached
    assert tuple(cut['inside']) in old_open and tuple(cut['inside']) in new_open
    for row,m in enumerate(old):
        for obj in m['objects']+m['events']['bg_events']:
            x,y=obj['x'],obj['y']+row*48;neighbors={(x+dx,y+dy) for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]}
            if neighbors & old_reached.keys():assert neighbors & reached.keys(),('lost public interaction',obj.get('script'))
            if neighbors & old_open.keys():assert neighbors & new_open.keys(),('lost Cut-open interaction',obj.get('script'))
    ledges=[]
    for lane in source['new_ledge']['lanes']:
        upper,lip,wall,landing=[tuple(lane[k]) for k in ['upper','lip','reverse_wall','landing']]
        assert [terrain[q] for q in [upper,lip,wall,landing]]==['FLOOR','HOP_DOWN','WALL','FLOOR']
        uphill=distance(ordinary,landing,terrain).get(upper)
        downhill=distance(edges,upper,terrain).get(landing)
        assert uphill>=source['new_ledge']['expected_minimum_return_grass'] and downhill==0,('new ledge grass bypass',uphill,downhill)
        open_price=distance(open_edges,landing,open_terrain).get(upper)
        assert open_price>=source['new_ledge']['expected_minimum_return_grass'], ('Cut return bypass',open_price)
        ledges.append(dict(upper=upper,lip=lip,landing=landing,uphill_grass=uphill,cut_open_return_grass=open_price,downhill_grass=downhill,ordinary_return_steps=distance(ordinary,landing).get(upper)))
    for lane in source['retained_north_ledge']['lanes']:
        for name in ['upper','lip','reverse_wall','landing']:assert terrain[tuple(lane[name])]==before[tuple(lane[name])]
        price=distance(ordinary,tuple(lane['landing']),terrain).get(tuple(lane['upper']))
        assert price is not None and price>=8,('original north ledge dry bypass',price)
    # The new drop is on ordinary grassland, far from every cliff recipe.
    assert all(new[0]['blocks'][y*32+x] not in range(0x68,0x74) for x in range(20,23) for y in range(12,15))
    costs=[]
    for a,b in [((16,86),(42,0)),((42,0),(16,86))]:
        old_cost=distance(old_edges,a,before).get(b);new_cost=distance(edges,a,terrain).get(b)
        assert new_cost is not None and new_cost>=old_cost
        costs.append(dict(start=a,target=b,before=old_cost,after=new_cost))
    return old,new,dict(new_ledge_lanes=ledges,retained_north_ledge_and_grassy_return=True,original_cut_gate_blocks_and_opens_reward=True,original_actor_door_bridge_pier_approaches_preserved=True,all_unowned_cliffs_stairs_water_and_perimeters_exact=True,whole_route_grass=costs,changed_metatiles=sum(r['changed_metatiles'] for r in source['maps']))


def main():
    cli=argparse.ArgumentParser(description=__doc__);cli.add_argument('--source',type=Path,default=SOURCE);cli.add_argument('--output',type=Path,required=True);args=cli.parse_args();assert not args.output.exists()
    source=json.loads(args.source.read_text());world=json.loads((ROOT/'planning/minnesota/world.json').read_text())
    require_selected_ancestor(world,'north_shore_passing_width_revision','e951fddafee3a0c09a2356d4dd683890ac498e23daeb266d08c5f12d79ec1167')
    parent=ROOT/source['parent_pack'];assert hashlib.sha256(parent.read_bytes()).hexdigest()==source['parent_pack_sha256'];pack=load_pack(parent);started=time.perf_counter();old,new,report=compose(pack,source)
    args.output.mkdir(parents=True);images=[]
    for label,modules in [('before',old),('after',new)]:
        image=Image.new('RGB',(1024,1408))
        for row,m in enumerate(modules):image.paste(native_room(m,pack,archive=args.output/label/m['id'],native_origin=True),(0,row*768))
        save_render(image,args.output/f'{label}-whole-original.png');images.append(image);cameras=[]
        for view in source['review_views']:
            x,y=view['core'];crop=image.crop((x*16-80,y*16-72,x*16+80,y*16+72));save_render(crop,args.output/label/f"{view['id']}-source160.png");cameras.append((view['id'],crop))
        save_render(room_sheet(cameras,columns=3),args.output/f'{label}-decisions.png')
    combined=Image.new('RGB',(2048,1408));combined.paste(images[0],(0,0));combined.paste(images[1],(1024,0));save_render(combined,args.output/'before-after-whole-original.png')
    report.update(status='PASS_SOURCE_ONLY',source=str(args.source),source_sha256=hashlib.sha256(args.source.read_bytes()).hexdigest(),seconds_after_load=round(time.perf_counter()-started,3),scope='Whole native source and dry/ledge/Cut collision graphs; native field input, publication and visual acceptance remain separate.')
    (args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))


if __name__=='__main__':main()
