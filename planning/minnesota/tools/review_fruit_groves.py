#!/usr/bin/env python3
"""Read-only written-pack placement, preservation and native source-art review."""
import argparse
import collections
import copy
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw
from audit_outdoor_connections import collision
from render_progress_atlas import native_room
from render_reference import load_pack as verified_load_pack
import pickle


def load_pack(path):
    cache = Path('.local') / ('fruit-review-' + hashlib.sha256(path.read_bytes()).hexdigest() + '.pickle')
    if cache.exists():
        return pickle.loads(cache.read_bytes())
    pack = verified_load_pack(path)
    cache.write_bytes(pickle.dumps(pack))
    return pack
from review_river_fun import flood


from river_content_lineage import validate

def review(output,source,path,controller_pack=None,bank_source=None):
    output.mkdir(parents=True, exist_ok=False)
    design = json.loads(source.read_bytes())
    base = load_pack(Path(design['base_pack']))
    pack = load_pack(path)
    validate(base);validate(pack)
    assert base['identity']['content_hash'] == design['base_content_hash']
    trees = design['trees']
    by_map = collections.defaultdict(list)
    for tree in trees:
        by_map[tree['map']].append(tree)
    rivers = {row['map']: row for row in design['rivers']}
    if bank_source:
        banks=json.loads(bank_source.read_bytes())
        for row in banks['maps']:
            river=rivers[row['id']]
            assert hashlib.sha256(bytes(river['blocks'])).hexdigest()==row['expected_blocks_sha256']
            width=pack['data']['maps'][row['id']]['attributes']['width']
            for cell in row['cells']:
                x,y=cell['at'];index=y*width+x
                assert river['blocks'][index]==cell['before']==5 and cell['after'] in [1,3]
                river['blocks'][index]=cell['after']

    edited = set(by_map) | set(rivers)
    assert pack['data']['maps'].keys() == base['data']['maps'].keys()
    for name, old in base['data']['maps'].items():
        new = pack['data']['maps'][name]
        if name not in edited:
            assert new == old, ('unrelated map changed', name)
            continue
        for field in ['attributes', 'events']:
            assert new[field] == old[field], (name, field)
        assert all(obj in new['objects'] for obj in old['objects']), ('old actor changed', name)
        for key, body in old['scripts'].items():
            assert new['scripts'][key] == body, ('old script changed', name, key)
        assert new['blocks'] == (rivers[name]['blocks'] if name in rivers else old['blocks'])
    for field in ['runtime_files', 'audio_manifest', 'compiled_audio', 'audio_compression', 'format_version']:
        assert pack[field] == base[field], field
    allowed = {'maps', 'map_blocks', 'npcs', 'map_scripts', 'trainers', 'fruit_trees',
               'story_event_script_constants', 'wild_encounters'}
    for key, value in base['data'].items():
        if key not in allowed:
            assert pack['data'][key] == value, ('unrelated catalog changed', key)
    old_trainers = base['data']['trainers']['trainers']
    new_trainers = pack['data']['trainers']['trainers']
    river_scripts = {o['script'] for n in rivers for o in pack['data']['maps'][n]['objects']}
    for key, value in old_trainers.items():
        if key not in river_scripts:
            assert new_trainers[key] == value, ('unrelated trainer changed', key)
    for key, value in base['data']['wild_encounters'].items():
        if key not in rivers:
            assert pack['data']['wild_encounters'][key] == value
    for name, rows in by_map.items():
        module = pack['data']['maps'][name]
        kinds = collision(pack, module)
        bodies = {(o['x'], o['y']) for o in module['objects']}
        w, h = module['attributes']['width']*2, module['attributes']['height']*2
        starts = {v for v, k in kinds.items() if k in ['FLOOR', 'TALL_GRASS'] and (v[0] in [0,w-1] or v[1] in [0,h-1])}
        for event in module['events']['warps']:
            x,y = event['x'],event['y']
            starts.update(q for q in [(x,y),(x-1,y),(x+1,y),(x,y-1),(x,y+1)] if kinds.get(q) in ['FLOOR','TALL_GRASS'])
        for v, k in kinds.items():
            if k == 'FLOOR' and any(kinds.get(q) in ['WATER','CUT_TREE','WHIRLPOOL','WATERFALL'] for q in [(v[0]-1,v[1]),(v[0]+1,v[1]),(v[0],v[1]-1),(v[0],v[1]+1)]):
                starts.add(v)
        reachable = flood(kinds, starts, {'FLOOR', 'TALL_GRASS'}, bodies)
        for tree in rows:
            ident = tree['id']
            target, approach = tuple(tree['tile']), tuple(tree['approach'])
            objects = [o for o in module['objects'] if o['object_identifier'] == ident]
            assert len(objects) == 1
            obj = objects[0]
            assert (obj['x'], obj['y']) == target
            assert obj['sprite'] == 'SPRITE_FRUIT_TREE' and obj['event_flag'] == '-1'
            assert pack['data']['fruit_trees'][ident] == tree['item']
            pickups = [p for p in module['script_field_pickups'] if p.get('fruit_tree_id') == ident]
            assert len(pickups) == 1
            assert kinds[target] == kinds[approach] == 'FLOOR'
            bodies = {(o['x'], o['y']) for o in module['objects']}
            assert approach not in bodies
            assert approach in reachable, ('grove isolated from native route/warp/shore', tree)
            ring = {(target[0]+dx, target[1]+dy) for dx in [-1, 0, 1] for dy in [-1, 0, 1]} - {target}
            assert all(kinds[q] in ['FLOOR', 'TALL_GRASS'] and q not in bodies for q in ring)
            assert ring <= flood(kinds, [approach], {'FLOOR', 'TALL_GRASS'}, bodies).keys()
    roster = json.loads(Path('planning/minnesota/pokemon/roster.json').read_bytes())
    curated = json.loads(Path('planning/minnesota/pokemon/roster-curation.json').read_bytes())
    retired = {f['id'] for f in curated['retired_families']}
    active = {m['name'].upper() for f in roster['families'] if f['id'] not in retired for m in f['members']}
    for name in rivers:
        for obj in pack['data']['maps'][name]['objects']:
            if obj['script'] in new_trainers:
                assert all(m['species'] in active for m in new_trainers[obj['script']]['party']), ('river team roster', obj['script'])
        def check_wild(value):
            if isinstance(value, dict):
                if 'species' in value:
                    assert value['species'] in active, ('river wild roster', name, value)
                for child in value.values():
                    check_wild(child)
            elif isinstance(value, list):
                for child in value:
                    check_wild(child)
        check_wild(pack['data']['wild_encounters'].get(name, {}))
    reaches = {}
    cases = []
    for reach in ['Lower', 'Upstream']:
        prefix = f'MnMississippi{reach}Woods'
        kinds, bodies = {}, set()
        trainer_count = 0
        for name in rivers:
            if not name.startswith(prefix):
                continue
            module = pack['data']['maps'][name]
            ox, oy = int(name[-5:-3])*64, int(name[-2:])*48
            kinds.update({(x+ox, y+oy): k for (x, y), k in collision(pack, module).items()})
            bodies.update((o['x']+ox, o['y']+oy) for o in module['objects'])
            trainer_count += sum(o['object_type'] == 'OBJECTTYPE_TRAINER' for o in module['objects'])
        assert trainer_count == 20
        landings = [v for v, kind in kinds.items() if kind == 'FLOOR' and v not in bodies
                    and any(kinds.get(q) == 'WATER' for q in [(v[0]-1,v[1]),(v[0]+1,v[1]),(v[0],v[1]-1),(v[0],v[1]+1)])]
        reached = flood(kinds, landings, {'FLOOR', 'TALL_GRASS'}, bodies)
        for name in rivers:
            if not name.startswith(prefix):
                continue
            ox, oy = int(name[-5:-3])*64, int(name[-2:])*48
            for obj in pack['data']['maps'][name]['objects']:
                x, y = obj['x']+ox, obj['y']+oy
                assert any(q in reached for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]), ('river activity unreachable', name, obj)
        old_kinds = {}
        for name in rivers:
            if name.startswith(prefix):
                ox,oy=int(name[-5:-3])*64,int(name[-2:])*48
                old_kinds.update({(x+ox,y+oy):k for (x,y),k in collision(base,base['data']['maps'][name]).items()})
        assert all(kinds[v]=='WATER' for v,k in old_kinds.items() if k=='WATER'), 'original river water changed'
        assert all(kinds[v]==k for v,k in old_kinds.items() if v[0] in [0,127] or v[1] in [0,135]), 'river boundary changed'
        activities=[];sight=set()
        for row in design['river_activities']['maps']:
            name=row['id']
            if not name.startswith(prefix):continue
            ox,oy=int(name[-5:-3])*64,int(name[-2:])*48
            for original in row['activities']:
                activity=copy.deepcopy(original);activity.update(map=name,global_tile=[original['tile'][0]+ox,original['tile'][1]+oy]);activities.append(activity)
                if activity['kind']=='trainer':
                    x,y=activity['global_tile'];dx,dy={'north':(0,-1),'south':(0,1),'east':(1,0),'west':(-1,0)}[activity['facing']]
                    sight.update((x+dx*i,y+dy*i) for i in [1,2])
        land=flood(kinds,[v for v in landings if v not in sight],{'FLOOR','TALL_GRASS'},bodies|sight)
        dry=flood(kinds,landings,{'FLOOR'},bodies)
        def location(v):
            x,y=v
            return dict(map=f'{prefix}C{x//64:02}R{y//48:02}',point=[x%64,y%48])
        for activity in activities:
            x,y=activity['global_tile'];neighbors=[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]
            goals=[v for v in neighbors if v in land];assert goals,activity['id']
            goal=min(goals,key=lambda v:(kinds[v]=='TALL_GRASS',v));route=[goal]
            while land[route[-1]] is not None:route.append(land[route[-1]])
            route.reverse();x,y=route[0];water=next(q for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)] if kinds.get(q)=='WATER')
            if activity['kind']=='item':assert not any(v in dry for v in neighbors),'supply bypasses grass'
            cases.append({**activity,'path':[location(v) for v in route],'water':location(water)})
        grass={v for v,k in kinds.items() if k=='TALL_GRASS'};unseen=set(grass);components=0
        while unseen:
            unseen-=flood(kinds,[min(unseen)],{'TALL_GRASS'}).keys();components+=1
        image = Image.new('RGB', (2048, 2176))
        for row in range(3):
            for col in range(2):
                module = pack['data']['maps'][f'{prefix}C{col:02}R{row:02}']
                image.paste(native_room(module, pack, native_origin=True), (col*1024, row*768))
        image.save(output/f'{reach}-whole-original.png')
        image.resize((768,816)).save(output/f'{reach}-preview.png')
        reaches[reach] = {'trainers': trainer_count, 'berry_trees': sum(n.startswith(prefix) for n in [t['map'] for t in trees]), 'all_activities_reachable': True, 'grass_gameplay_tiles':len(grass),'grass_patches':components}
    examples = ['MnWinona', 'MnMinneapolis', 'MnStPaul', 'MnDuluth', 'MnEly', 'MnBemidji',
                'MnGrandMarais', 'MnInternationalFalls', 'MnR01C00R00', 'MnR05C00R00', 'MnR11C00R00', 'MnR13C00R00']
    tiles = []
    for name in examples:
        if name not in by_map:
            continue
        image = native_room(pack['data']['maps'][name], pack, native_origin=True)
        image.save(output/f'{name}-whole-original.png')
        preview = image.copy()
        preview.thumbnail((512,512))
        tile = Image.new('RGB', (532,552), '#eee5cc')
        tile.paste(preview, (10,32))
        ImageDraw.Draw(tile).text((10,10), f'{name}: {len(by_map[name])} trees', fill='black')
        tiles.append(tile)
    if tiles:
        gallery = Image.new('RGB', (532*3,552*((len(tiles)+2)//3)), '#eee5cc')
        for i, tile in enumerate(tiles):
            gallery.paste(tile, ((i%3)*532,(i//3)*552))
        gallery.save(output/'regional-groves.png')
    if not trees:
        assert pack['data']['fruit_trees']==base['data']['fruit_trees'], 'river-only stage changed fruit catalog'
    native_room(base['data']['maps']['Route29'], base, native_origin=True).save(output/'native-Route29-reference.png')
    receipt = dict(pack=str(path),pack_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                   source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),bank_source_sha256=hashlib.sha256(bank_source.read_bytes()).hexdigest() if bank_source else None,trees=len(trees),
                   outdoor_maps=len(by_map),items=dict(collections.Counter(t['item'] for t in trees)),
                   reaches=reaches, all_tree_rings_clear=True, original_actors_and_events_preserved=True,
                   unrelated_maps_and_catalogs_preserved=True, art_and_audio_preserved=True)
    controller_pack = controller_pack or path
    if controller_pack != path:
        controller=load_pack(controller_pack);validate(controller)
        assert all(controller['data']['maps'][name]==pack['data']['maps'][name] for name in rivers)
    for launch in design['river_activities'].get('preserved_launches',[]):
        name=launch['map'];v=tuple(launch['core'])
        assert collision(base,base['data']['maps'][name])[v]==collision(pack,pack['data']['maps'][name])[v]=='FLOOR'
    (output/'river-controller-proof.json').write_text(json.dumps(dict(pack=str(controller_pack),pack_sha256=hashlib.sha256(controller_pack.read_bytes()).hexdigest(),reaches=reaches,cases=cases),indent=2)+'\n')
    (output/'proof.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(receipt,indent=2),flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True, help='Fresh immutable review epoch')
    parser.add_argument('--source',type=Path,default=Path('planning/minnesota/fruit-groves/groves.compact-v01.json'))
    parser.add_argument('--written',type=Path,required=True)
    parser.add_argument('--controller-pack',type=Path)
    parser.add_argument('--bank-source',type=Path)
    args=parser.parse_args();review(args.output,args.source,args.written,args.controller_pack,args.bank_source)
