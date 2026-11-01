#!/usr/bin/env python3
"""Review an explicit expanded R11 candidate; never publishes or edits a pack."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from ledge_paths import draw_ledge_paths
from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_stages import render_stage
import review_duluth
from review_gunflint import flood, neighbors
from review_league_approach import check as common_check
from review_north_shore import check_river_catchments

ROOT = Path(__file__).resolve().parents[3]


def validate_orthogonal_geometry(layout):
    """Match the native compiler's axis-aligned polygon and path contract."""
    for layer in layout['layers']:
        kind = layer['kind']
        if kind not in {'polygon', 'cliff_polygon', 'path'}:
            continue
        points = layer['points']
        edges = list(zip(points, points[1:]))
        if kind != 'path':
            edges.append((points[-1], points[0]))
        for start, end in edges:
            assert start[0] == end[0] or start[1] == end[1], (
                f"{layer.get('name', kind)} must be orthogonal: {start} -> {end}")


def inspect(design, layout, pack, blocks):
    report, kind, seen = common_check(design, layout, pack, blocks)
    width, height = layout['dimensions_metatiles']
    occupied = {tuple(a['tile']) for key in ['trainers', 'residents', 'items'] for a in design[key]}
    land = {(x, y) for y in range(height * 2) for x in range(width * 2)
            if kind(x, y) in {'FLOOR', 'TALL_GRASS', 'LADDER'} and (x, y) not in occupied}
    water = {(x, y) for y in range(height * 2) for x in range(width * 2)
             if kind(x, y) in {'WATER', 'WATERFALL'}}
    start = tuple(v * 2 for v in design['endpoints'][0]['gate_metatile'])
    surf = flood(start, land | water)
    chunks = {(x // 64, y // 48) for x, y in surf}
    # An enlarged lake has four deliberately water-only chunks. They require
    # native reciprocal Surf seams, not artificial walking islands as a quota.
    if len(chunks) == design['chunk_columns'] * design['chunk_rows']:
        report['errors'] = [e for e in report['errors'] if not e.startswith('Unused chunks:')]
    else:
        report['errors'].append('Expanded walking/Surf graph does not visit every chunk')
    outer = {(x, y) for y in range(height * 2) for x in range(width * 2)
             if x in {0, width * 2 - 1} or y in {0, height * 2 - 1}}
    if water & outer:
        report['errors'].append('Unclosed outer Water/Waterfall: ' + str(sorted(water & outer)[:12]))
    if any(kind(x, y) == 'CUT_TREE' for y in range(height * 2) for x in range(width * 2)):
        report['errors'].append('Incidental Cut quadrant')
    for view in design['surf_viewpoints']:
        if tuple(view['tile']) not in surf & water:
            report['errors'].append(view['id'] + ': not reachable water')
    for crossing in layout['crossings']:
        cells = {(2 * x + dx, 2 * y + dy) for x, y in crossing['blocks'] for dx in [0, 1] for dy in [0, 1]}
        if any(kind(*p) != 'LADDER' or p not in seen for p in cells):
            report['errors'].append(crossing['id'] + ': deck/approach invalid')
        end = tuple(v * 2 for v in design['endpoints'][1]['gate_metatile'])
        if crossing['necessary'] and end in flood(start, land - cells):
            report['errors'].append(crossing['id'] + ': dry bypass')
    try:
        report.update(check_river_catchments(design, layout, kind, seen))
    except AssertionError as error:
        report['errors'].append(str(error))
    for ledge in design['one_way_ledges']:
        x0, y0, x1, _ = ledge['bounds_core']
        if any(kind(x, y0) != 'HOP_DOWN' or kind(x, y0 + 1) != 'WALL' for x in range(x0, x1)):
            report['errors'].append(ledge['id'] + ': native lip/reverse face invalid')
        if any(kind(*p) != 'WALL' for p in ledge['side_barriers']):
            report['errors'].append(ledge['id'] + ': side cap invalid')
    costs_report=[]
    for choice in design.get('optional_grass_rewards',[]):
        x0,y0,x1,y1=choice['bounds_core']
        band={(x,y) for y in range(y0,y1) for x in range(x0,x1)}
        if any(kind(*p)!='TALL_GRASS' for p in band):
            report['errors'].append(choice['id']+': declared encounter band damaged')
        costs={start:0};todo=deque([start])
        while todo:
            at=todo.popleft()
            for to in neighbors(at):
                if to not in land:continue
                cost=costs[at]+int(to in band)
                if cost<costs.get(to,10**9):
                    costs[to]=cost
                    (todo.append if to in band else todo.appendleft)(to)
        target=next(a for a in design['items'] if a['id']==choice['actor_id'])
        steps=min(costs.get(p,10**9) for p in neighbors(tuple(target['tile'])))
        if not choice['minimum_grass_steps']<=steps<10**9:
            report['errors'].append(choice['id']+': optional cache has a free grass bypass or no approach')
        costs_report.append(dict(id=choice['id'],actual_minimum_grass_steps=steps))
    probes=design['expansion_contract']['controller_probes']
    for stair in probes['stair_lanes']:
        path=[stair['before']]+stair['steps']+[stair['after']]
        if stair.get('after_collision','FLOOR') not in {'FLOOR','TALL_GRASS'} or any(kind(*p)!='FLOOR' for p in path[:-1]) or kind(*path[-1])!=stair.get('after_collision','FLOOR') or any(abs(a[0]-b[0])+abs(a[1]-b[1])!=1 for a,b in zip(path,path[1:])):
            report['errors'].append(stair['id']+': stair path invalid')
        if kind(*stair['blocked_side']['target_core'])!='WALL':
            report['errors'].append(stair['id']+': native stair wall lost')
    bank,launch=probes['sea_bank'],probes['sea_launch']
    if tuple(bank) not in seen or kind(*bank)!='FLOOR' or kind(*launch)!='WATER' or sum(abs(a-b) for a,b in zip(bank,launch))!=1:
        report['errors'].append('Main sea bank/launch invalid')
    sea_route=[]
    for a,b in zip(probes['sea_waypoints'],probes['sea_waypoints'][1:]):
        assert a[0]==b[0] or a[1]==b[1], 'sea probe must be an orthogonal input path'
        dx=(b[0]>a[0])-(b[0]<a[0]);dy=(b[1]>a[1])-(b[1]<a[1])
        sea_route.extend((a[0]+dx*i,a[1]+dy*i) for i in range(max(abs(b[0]-a[0]),abs(b[1]-a[1]))+1))
    if any(kind(*p)!='WATER' for p in sea_route) or len({(x//64,y//48) for x,y in sea_route})!=8:
        report['errors'].append('Ordinary Water probe does not safely visit every chunk')
    for probe in probes['sea_boundary_probes']:
        if probe.get('expected', 'BUOY') not in {'WALL', 'BUOY'} or kind(*probe['at'])!='WATER' or kind(*probe['target'])!=probe.get('expected','BUOY'):
            report['errors'].append('Sea boundary probe has invalid water/rock collision')
    boundary_views=[]
    boundary_ids = ['SUPERIOR_ROCK_BOUNDARY', 'SOUTHERN_SEA_BOUNDARY']
    if 'compact_terminal' not in design:
        boundary_ids.append('NORTHERN_SEA_ROCK_BOUNDARY')
    for identifier in boundary_ids:
        view=next(v for v in design['surf_viewpoints'] if v['id']==identifier)
        x,y=view['tile']
        visible=sum(kind(cx,cy)=='BUOY' for cy in range(max(0,y-5),min(height*2,y+5))
                    for cx in range(max(0,x-5),min(width*2,x+5)))
        if visible<8:report['errors'].append(identifier+': camera does not show the actual blocking rock boundary')
        boundary_views.append(dict(id=identifier,visible_buoy_core_tiles=visible))
    if 'compact_terminal' in design:
        assert design['compact_terminal']['native_width_metatiles'] == 4
        compact_probes = [p for p in probes['sea_boundary_probes'] if p.get('expected') == 'WALL']
        assert len(compact_probes) == 1, 'One exact compact-bank collision probe required'
        probe = compact_probes[0]
        x, y = probe['at']
        assert 64 <= x < 72 and y < 48, 'Compact water probe must remain in its real native chunk'
        assert sum(abs(a-b) for a,b in zip(probe['at'],probe['target'])) == 1
        assert kind(x,y) == 'WATER' and kind(*probe['target']) == 'WALL'
        boundary_views.append(dict(id='COMPACT_BANK', at=probe['at'],
                                   target=probe['target'], blocking_collision='WALL'))
    report.update(dimensions_metatiles=[width, height],source_only=True,native_playability_proved=False,
                  whole_surface_chunks=len(chunks),walkable_chunks=len({(x//64,y//48) for x,y in seen}),
                  reachable_water_core_tiles=len(surf & water),outer_water_core_tiles=len(water & outer),
                  native_waterfall_metatiles=sum(kind(x*2,y*2)=='WATERFALL' for y in range(height) for x in range(width)),
                  native_metatile_varieties=len(set(blocks)),optional_grass_costs=costs_report,
                  boundary_camera_collision_probes=boundary_views)
    return report, kind, seen, land, surf


def negative_checks(design,layout,pack,blocks):
    width=layout['dimensions_metatiles'][0]
    cases=[('opened eastern Surf boundary',55,30,0x35),
           ('erased original tall fall',10,34,0x35),
           ('erased offset Stoney lower fall',11,77,0x35),
           ('removed Stoney bridge',20,81,0x05),
           ('fake Floor bridge',20,81,0x01),
           ('incidental decorative Cut',1,1,0x5b),
           ('ordinary Floor replaces directional Birch face',16,45,0x01),
           ('Palisade cache free upper grass bypass',10,23,0x71),
           ('removed native Birch return stair',18,45,0x05)]
    rejected=[]
    for name,x,y,block in cases:
        changed=list(blocks);changed[y*width+x]=block
        report,*_=inspect(design,layout,pack,changed)
        assert report['errors'], 'negative fixture accepted: '+name
        rejected.append(name)
    wrong=deepcopy(design)
    next(v for v in wrong['surf_viewpoints'] if v['id']=='SUPERIOR_ROCK_BOUNDARY')['tile']=[59,81]
    report,*_=inspect(wrong,layout,pack,blocks)
    assert any('actual blocking rock boundary' in error for error in report['errors']), 'stale boundary camera accepted'
    rejected.append('old boundary camera left inside enlarged lake')
    for layer_kind in ['polygon', 'cliff_polygon', 'path']:
        changed = deepcopy(layout)
        changed['layers'].append(dict(kind=layer_kind, name='diagonal negative fixture',
                                      points=[[1, 1], [2, 2], [1, 2]], material='floor'))
        try:
            validate_orthogonal_geometry(changed)
        except AssertionError as error:
            assert 'must be orthogonal' in str(error)
        else:
            raise AssertionError('diagonal source geometry accepted: ' + layer_kind)
        rejected.append('diagonal ' + layer_kind + ' rejected before rasterization')
    return rejected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('dossier', type=Path)
    parser.add_argument('layout', type=Path)
    parser.add_argument('--pack', type=Path, default=ROOT/'content-packs/minnesota-cities-review.crystalpack')
    parser.add_argument('--output-root', type=Path, default=ROOT/'planning/minnesota/renders/north-shore/expanded-candidate')
    parser.add_argument('--baseline-dir', type=Path, default=ROOT/'planning/minnesota/routes')
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    source = args.dossier.read_bytes(); strokes = args.layout.read_bytes()
    design, layout = json.loads(source), json.loads(strokes)
    validate_orthogonal_geometry(layout)
    digest = hashlib.sha256(source + strokes).hexdigest()
    output = args.output_root/digest[:16]
    output.mkdir(parents=True, exist_ok=True)
    for name, payload in [('MN_R11.json',source),('MN_R11.layout.json',strokes)]:
        path = output/name
        if path.exists():
            assert path.read_bytes() == payload, 'immutable candidate changed'
        else:
            path.write_bytes(payload)
    for name in ['MN_R11.json','MN_R11.layout.json']:
        target=output/('baseline-'+name)
        if not target.exists():target.write_bytes((args.baseline_dir/name).read_bytes())
    pack = load_pack(args.pack)
    definition=deepcopy(pack['data']['tilesets']['mn_north_shore'])
    for recipe in layout['derived_blocks'][1:]:
        definition['collision'][recipe['id']] = pack['data']['tilesets'][recipe['source_tileset']]['collision'][recipe['source_block']]
    pack['data']['tilesets'][layout['compiled_tileset']] = definition
    baseline=json.loads((output/'baseline-MN_R11.json').read_bytes())
    for key in ['trainers','residents','doors','interiors','required_flags']:
        assert design[key]==baseline[key], 'expanded R11 changed original '+key
    assert (design['items'][:len(baseline['items'])] == baseline['items']
            and design['signs'][:len(baseline['signs'])] == baseline['signs']), 'original resources/signs changed'
    assert design['required_grass_crossings'][0]==baseline['required_grass_crossings'][0], 'original required habitat changed'
    assert design['waterfall_summit']==baseline['waterfall_summit'], 'original summit contract changed'
    started=time.perf_counter(); blocks=layout_blocks(layout)
    report,kind,seen,land,surf=inspect(design,layout,pack,blocks)
    art=SourceArt(pack)
    assert art.blocks('mn_north_shore')[128].tobytes() == art.blocks('cave')[0x2c].tobytes()
    for block in [0x30,0x31,0x32,0x36,0x38,0x39,0x3a]:
        block_image=art.blocks('johto')[block]
        save_render(block_image,output/'boundary-source-art'/f'johto-{block:02x}-native32.png')
        save_render(render_stage(block_image,'resize',output/'boundary-source-art',f'johto-{block:02x}-4x',(128,128),Image.Resampling.NEAREST),output/'boundary-source-art'/f'johto-{block:02x}-4x.png')
    terrain=art.render('johto',*layout['dimensions_metatiles'],blocks,derived=layout['derived_blocks'])
    save_render(terrain,output/'terrain-original.png')
    scene=render_stage(terrain,'convert',output,'whole-rgba','RGBA')
    review_duluth.OUTPUT=output
    for key in ['trainers','residents','items']:
        for actor in design[key]:
            token='SPRITE_POKE_BALL' if key=='items' else actor['sprite']
            frame=review_duluth.oriented_sprite(pack,token,actor.get('facing','south'))
            scene.alpha_composite(frame,(actor['tile'][0]*16,actor['tile'][1]*16-4))
    save_render(scene,output/'whole-original-rgba.png')
    whole=render_stage(scene,'convert',output,'whole-rgb','RGB');save_render(whole,output/'whole-route.png')
    overview=render_stage(whole,'resize',output,'whole-half',(whole.width//2,whole.height//2),Image.Resampling.NEAREST)
    save_render(overview,output/'whole-route-half.png')
    views=[]
    for view in design['viewpoints']+design['surf_viewpoints']:
        x,y=view['tile'];name=view['id'].lower()
        camera=render_stage(whole,'crop',output,name,(x*16-80,y*16-72,x*16+80,y*16+72))
        save_render(camera,output/(name+'-native160.png'))
        save_render(render_stage(camera,'resize',output,name+'-4x',(640,576),Image.Resampling.NEAREST),output/(name+'-4x.png'))
        views.append((view['id'],camera))
    save_render(room_sheet(views,columns=4),output/'decision-cameras.png')
    # Find the largest actual ordinary ground footprint, including new extension.
    occupied={tuple(a['tile']) for key in ['trainers','residents','items'] for a in design[key]}
    sizes={};worst=(0,0,0)
    width,height=layout['dimensions_metatiles']
    for y in range(height):
        for x in range(width):
            valid=blocks[y*width+x] in {1,2,0x71} and (x*2,y*2) in seen and all((x*2+dx,y*2+dy) not in occupied for dx in [0,1] for dy in [0,1])
            n=1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0)) if valid else 0
            sizes[x,y]=n
            if n>worst[0]:worst=(n,x-n+1,y-n+1)
    n,x,y=worst;at=(x*2+n,y*2+n)
    camera=render_stage(whole,'crop',output,'worst-corridor',(at[0]*16-80,at[1]*16-72,at[0]*16+80,at[1]*16+72))
    save_render(camera,output/'worst-corridor-native160.png')
    save_render(render_stage(camera,'resize',output,'worst-corridor-4x',(640,576),Image.Resampling.NEAREST),output/'worst-corridor-4x.png')
    if report['ledge_returns']:
        save_render(draw_ledge_paths(whole,design,report['ledge_returns']),output/'ledge-return-paths.png')
    report.update(worst_corridor_core=list(at),largest_blank_ground_square_metatiles=n,
                  dossier_sha256=hashlib.sha256(source).hexdigest(),layout_sha256=hashlib.sha256(strokes).hexdigest(),
                  pack_file_sha256=hashlib.sha256(args.pack.read_bytes()).hexdigest(),render_ms=round((time.perf_counter()-started)*1000,1))
    if args.self_test:report['negative_cases_rejected']=negative_checks(design,layout,pack,blocks)
    payload=(json.dumps(report,indent=2)+'\n').encode()
    old_report=output/'review.json'
    if old_report.exists():
        prior=old_report.read_bytes()
        (output/('review-'+hashlib.sha256(prior).hexdigest()+'.json')).write_bytes(prior)
    (output/('review-'+hashlib.sha256(payload).hexdigest()+'.json')).write_bytes(payload)
    old_report.write_bytes(payload)
    print(json.dumps(dict(output=str(output),errors=report['errors'],render_ms=report['render_ms'],worst=at),indent=2))


if __name__=='__main__':
    main()
