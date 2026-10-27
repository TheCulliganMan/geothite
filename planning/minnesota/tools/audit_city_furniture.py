#!/usr/bin/env python3
"""Prove a written three-city furniture revision changes only declared room rows.

This is a read-only stored-pack/source-pixel comparison, not native gameplay or
GPU evidence. It uses existing source rendering and keeps all review originals.
"""
import argparse
import base64
from collections import Counter
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from audit_city_expansions import CITIES, fingerprint, kind
from audit_winding_shelter_541 import raw_identity
from render_archive import save_render
from render_reference import SourceArt, load_pack, room_sheet
from review_north_shore import oriented_sprite
from review_passage_native import actors


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def audit(before, after, contracts):
    checks, deltas = [], []

    def equal(label, expected, actual):
        valid = expected == actual
        checks.append(dict(check=label, passed=valid, **({} if valid else dict(
            expected_sha256=fingerprint(expected), actual_sha256=fingerprint(actual)))))

    a, b = before['data'], after['data']
    rooms, changed = {}, {}
    for old, new in contracts:
        equal(old['city'] + ' contract city identity exact', old['city'], new['city'])
        normalized = deepcopy(new)
        previous = {room['id']: room for room in old['rooms']}
        equal(old['city'] + ' exact room roster and ordering', [room['id'] for room in old['rooms']], [room['id'] for room in new['rooms']])
        for room in normalized['rooms']:
            name = room['id']
            rooms[name] = room
            prior = previous[name]
            if room['rows'] != prior['rows']:
                changed[name] = (prior, deepcopy(room))
            room['rows'] = prior['rows']
        equal(old['city'] + ' ENTIRE contract exact except explicit room rows', old, normalized)
    equal('same exact660 map roster', set(a['maps']), set(b['maps']))
    equal('native roster count remains660', 660, len(b['maps']))
    equal('exact29 declared changed room roster', 29, len(changed))
    equal('changed rooms divide6/6/17 across the three contracts', [6,6,17], sorted(Counter(
        next(old['city'] for old, _ in contracts if any(room['id'] == name for room in old['rooms']))
        for name in changed).values()))
    actual_changed = {name for name in a['maps'] if a['maps'][name] != b['maps'].get(name)}
    equal('ONLY the exact29 declared whole native room modules change', set(changed), actual_changed)
    normalized = dict(b)
    normalized['maps'] = dict(b['maps'])
    normalized['map_blocks'] = dict(b['map_blocks'])
    for name, (old_room, new_room) in changed.items():
        old, new = a['maps'][name], b['maps'][name]
        old_blocks = [int(value,16) for row in old_room['rows'] for value in row.split()]
        new_blocks = [int(value,16) for row in new_room['rows'] for value in row.split()]
        equal(name + ' exact v01 source native preimage', old_blocks, old['blocks'])
        equal(name + ' exact v02 authored native rows', new_blocks, new['blocks'])
        module = dict(new, blocks=old['blocks'])
        equal(name + ' ENTIRE native module exact except blocks', old, module)
        label = old['attributes']['blocks_label']
        equal(name + ' old native owned block payload', old_blocks, list(base64.b64decode(a['map_blocks'][label],validate=True)))
        equal(name + ' new native owned block payload', new_blocks, list(base64.b64decode(b['map_blocks'][label],validate=True)))
        normalized['maps'][name] = old
        normalized['map_blocks'][label] = a['map_blocks'][label]
        width = old_room['dimensions_metatiles'][0]
        deltas.append(dict(room=name, dimensions_metatiles=old_room['dimensions_metatiles'],
                           before_rows=old_room['rows'], after_rows=new_room['rows'],
                           source_cells=[dict(meta=[i % width,i // width],before=f'{x:02x}',after=f'{y:02x}')
                                         for i,(x,y) in enumerate(zip(old_blocks,new_blocks)) if x != y]))
    equal('ENTIRE GameDataSet exact after only29 block arrays/owned payloads restored', a, normalized)
    original = set(a['maps']) - set(rooms)
    equal('full original543 map roster remains', 543, len(original))
    for name in sorted(set(a['maps']) - set(changed)):
        equal(name + ' complete unchanged native module', a['maps'][name], b['maps'].get(name))
    for city in sorted(CITIES):
        equal(city + ' whole exterior module byte-value exact', a['maps'][city], b['maps'][city])
    for key in before:
        if key not in ['data','identity']:
            equal('entire native top-level field exact ' + key, before[key], after[key])
    identity = dict(after['identity'],content_hash=before['identity']['content_hash'])
    equal('only derived content hash changes in identity', before['identity'], identity)
    return checks, deltas


def camera_pixels(before, after, views, output):
    checks, witnesses, sheets = [], [], []
    art = [SourceArt(before), SourceArt(after)]
    scenes = [{},{}]
    for city in sorted(CITIES):
        for index, pack in enumerate([before,after]):
            module = pack['data']['maps'][city]
            attr = module['attributes']
            terrain = art[index].render(attr['tileset_name'],attr['width'],attr['height'],module['blocks'],'day')
            scene = actors(pack,terrain,module)
            scenes[index][city] = scene
            save_render(scene,output / f'{city}-v0{index+1}-whole-source-reconstruction.png')
        valid = scenes[0][city].tobytes() == scenes[1][city].tobytes()
        checks.append(dict(check=city + ' ENTIRE day exterior SourceArt/actor scene RGB exact',passed=valid))
    for view in views:
        city, (x,y), label = view['map'], view['core'], view['id']
        assert city in CITIES
        hashes = []
        for index, pack in enumerate([before,after]):
            scene = scenes[index][city].copy()
            module = pack['data']['maps'][city]
            valid = kind(pack,module,(x,y)) == 'FLOOR' and not any(obj['x'] == x and obj['y'] == y for obj in module['objects'])
            checks.append(dict(check=f'{city} v0{index+1} {label} explicit unoccupied Floor camera',passed=valid))
            scene.alpha_composite(oriented_sprite(pack,'SPRITE_CHRIS','south'),(x*16,y*16))
            left = max(0,min(scene.width-160,x*16-80))
            top = max(0,min(scene.height-144,y*16-72))
            frame = scene.crop((left,top,left+160,top+144))
            save_render(frame,output/'cameras'/f'{city}-{label}-v0{index+1}-source160.png')
            sheets.append((city.replace('Mn','')+' '+label+f' v0{index+1}',frame))
            hashes.append(sha(frame.tobytes()))
        checks.append(dict(check=city+' same-source160 camera exact '+label,passed=hashes[0] == hashes[1]))
        witnesses.append(dict(map=city,id=label,core=[x,y],time='day',facing='south',
                              before_rgba_sha256=hashes[0],after_rgba_sha256=hashes[1],
                              scope='Source reconstruction with identical authored object visibility; no GPU capture.'))
    for start in range(0,len(sheets),16):
        save_render(room_sheet(sheets[start:start+16],4),output/f'unchanged-source-cameras-{start//16+1}.png')
    return checks,witnesses


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before',type=Path,required=True)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--contracts',nargs=2,type=Path,action='append',required=True,metavar=('BEFORE','AFTER'))
    parser.add_argument('--views',type=Path,action='append',required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=False)
    paths = [args.before,args.pack] + [p for pair in args.contracts for p in pair] + args.views
    hashes = {str(path):sha(path.read_bytes()) for path in paths}
    contracts = [(json.loads(old.read_bytes()),json.loads(new.read_bytes())) for old,new in args.contracts]
    assert {old['city'] for old,_ in contracts} == CITIES
    views = [view for path in args.views for view in json.loads(path.read_bytes())]
    before,after = load_pack(args.before),load_pack(args.pack)
    checks,deltas = audit(before,after,contracts)
    pixel_checks,witnesses = camera_pixels(before,after,views,args.output)
    checks += pixel_checks
    negatives = []
    changed = {row['room'] for row in deltas}
    unowned = next(name for name in before['data']['maps'] if name.startswith('MnMinneapolis') and name not in changed and name not in CITIES)
    for label,name,mutate in [
        ('outdoor street block changed','MnMinneapolis',lambda module:module['blocks'].__setitem__(0,module['blocks'][0]^1)),
        ('quest program changed in furnished room','MnMinneapolisPrintShop',lambda module:module['scripts']['MN_MP_PRINTSHOP_HOST'].clear()),
        ('unowned original room changed',unowned,lambda module:module['blocks'].__setitem__(0,module['blocks'][0]^1)),
    ]:
        damaged = dict(after)
        damaged['data'] = dict(after['data'])
        damaged['data']['maps'] = dict(after['data']['maps'])
        damaged['data']['maps'][name] = deepcopy(after['data']['maps'][name])
        mutate(damaged['data']['maps'][name])
        failed = [row['check'] for row in audit(before,damaged,contracts)[0] if not row['passed']]
        negatives.append(dict(corruption=label,rejected=bool(failed),failed_checks=failed[:8]))
    identities = {str(path):raw_identity(path) for path in [args.before,args.pack]}
    checks.append(dict(check='both native CBOR content identities independently recomputed',passed=all(value['exact'] for value in identities.values())))
    checks.append(dict(check='all input bytes stable throughout proof',passed=hashes == {str(path):sha(path.read_bytes()) for path in paths}))
    report = dict(status='PASS' if all(row['passed'] for row in checks) and all(row['rejected'] for row in negatives) else 'FAIL',
                  scope='Read-only native furniture-only preservation and exact source camera reconstruction, not GPU/controller/save compatibility evidence.',
                  input_sha256=hashes,tool_sha256=sha(Path(__file__).read_bytes()),exact_native_content_identities=identities,
                  changed_rooms=len(deltas),changed_metatiles=sum(len(row['source_cells']) for row in deltas),
                  source_exact_deltas=deltas,checks=checks,corruption_negatives=negatives,outdoor_source_cameras=witnesses)
    raw=(json.dumps(report,indent=2)+'\n').encode()
    path=args.output/('proof-'+sha(raw)+'.json')
    path.write_bytes(raw)
    (args.output/'audit_city_furniture.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(dict(status=report['status'],report=str(path),checks=len(checks),negative_checks=len(negatives),
                          changed_rooms=len(deltas),changed_metatiles=report['changed_metatiles'],source_cameras=len(witnesses),
                          failed=[row for row in checks if not row['passed']]),indent=2))
    raise SystemExit(0 if report['status']=='PASS' else 1)


if __name__ == '__main__':
    main()
