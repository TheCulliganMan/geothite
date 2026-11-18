#!/usr/bin/env python3
"""Compare the installed Minnesota wayfinding revision with its exact released parent."""
import argparse
import base64
from collections import deque
import json
from pathlib import Path

from audit_regional_design import DIRS, POLICY, Terrain, check, load, same_except
from render_reference import SourceArt

REGION = Path(__file__).resolve().parents[1]
PARENTS = {
    'ea89ae2a3df712b16018c3f7d89f96cfef5486f24778476cff5f7df490af74dd',
    '19caa20f4616a72a378f757cbef7052c5bf2a91d079782e267a7b43f6b91a64e',
}
MANIFEST = 'northstar-wayfinding-v1'
GUIDE = 'MN_R10_GALLERY_READER'
TM = 'TM_ROCK_SMASH'
RECEIPT = 'EVENT_MN_R10_RECEIVED_ROCK_SMASH'
CARGO = 'MnIceCargoBasement'


def check_origin(old, new, sha):
    same_except(old, new, {'data', 'identity', 'report', 'runtime_files'}, 'pack')
    same_except(old['report'], new['report'], {'manifests'}, 'compiler report')
    manifests = new['report']['manifests']
    check(manifests.count(MANIFEST) == 1 and [m for m in manifests if m != MANIFEST]
          == old['report']['manifests'], 'release lineage')
    policy = json.loads(bytes(new['runtime_files'][POLICY]))
    check(policy['schema_version'] == 6 and policy['target_manifest'] == MANIFEST
          and policy['inherited'] == json.loads(bytes(old['runtime_files'][POLICY])), 'save lineage')
    check({s['modpack_hash'] for s in policy['sources']} == PARENTS and any(
        s['modpack_hash'] == sha and s['content_hash'] == old['identity']['content_hash']
        and s['modpack_id'] == old['identity']['runtime_modpack_id'] for s in policy['sources']),
        'direct save parents')
    check(new['identity']['runtime_modpack_id'] in policy['target_modpack_ids'], 'target save identity')


def check_original_art(old, new, tilesets):
    previous, revised = SourceArt(old), SourceArt(new)
    for name in tilesets:
        oldset, newset = (p['data']['tilesets'][name] for p in (old, new))
        for block, collision in oldset['collision'].items():
            check(newset['collision'][block] == collision, f'{name}/{block}: prior collision')
        # An atlas can change a glyph shared by hundreds of otherwise unchanged maps.
        # Compare every old block at both day and night, including unplaced blocks.
        for time in ('day', 'nite'):
            a, b = previous.blocks(name, time), revised.blocks(name, time)
            check(len(b) >= len(a), name + ' shortened native atlas')
            for block, image in enumerate(a):
                check(image.tobytes() == b[block].tobytes(), f'{name}/{block:02x}/{time}: prior art')


def bluff_distances(pack):
    """Outdoor paths only, with trainers in their starting positions and cave mouths excluded."""
    cliff = Terrain(pack, 'MnR01C00R00')
    start, goal = (25, 14), (24, 18)
    walk = {'FLOOR', 'TALL_GRASS', 'UP_WALL', 'HOP_DOWN'}
    def distance(a, b):
        queue, seen = deque([(a, 0)]), {a}
        while queue:
            (x, y), n = queue.popleft()
            if (x, y) == b:
                return n
            for dx, dy in DIRS.values():
                current = cliff.tiles[x, y]
                dest = (x + dx, y + dy)
                if current == 'HOP_DOWN' and dy == 1:
                    dest = (x, y + 2)
                if ((current == 'UP_WALL' and dy == -1) or
                        (cliff.tiles.get(dest) == 'UP_WALL' and dy == 1)):
                    continue
                if (dest not in seen and cliff.tiles.get(dest) in walk
                        and dest not in cliff.warps and dest not in cliff.occupied):
                    seen.add(dest)
                    queue.append((dest, n + 1))
        raise ValueError(f'Bluff route {a} to {b} disconnected')
    return distance(start, goal), distance(goal, start)


def check_lookouts(old, new):
    before, after = (p['data']['maps'][CARGO] for p in (old, new))
    grid = Terrain(new, CARGO)
    rotations = {'MN_ICE_HQ_CARGO_SEARCH': 'SPRITEMOVEDATA_SPINCLOCKWISE',
                 'MN_ICE_HQ_CARGO_MANIFEST': 'SPRITEMOVEDATA_SPINCOUNTERCLOCKWISE'}
    for a, b in zip(before['objects'], after['objects'], strict=True):
        if a['script'] not in rotations:
            check(a == b, 'unrelated cargo actor')
            continue
        check(dict(a, spritemovedata=rotations[a['script']]) == b and not b.get('sightline_direction_override')
              and b['object_type'] == 'OBJECTTYPE_TRAINER', b['script'] + ' must watch its live facing')
        lanes = []
        for dx, dy in DIRS.values():
            lane = set()
            for n in range(1, max(8, b['radius']) + 1):
                at = b['x'] + n * dx, b['y'] + n * dy
                if at in grid.occupied or grid.tiles.get(at) not in {'FLOOR', 'LADDER'}:
                    break
                lane.add(at)
            check(not lane & grid.warps, 'arrival falls in a guard sightline')
            lanes.append(len(lane))
        check(min(lanes) >= 2, 'cargo lookout has a wasted turn')


def check_tm(old, new):
    shop = 'MART_MN_MP_DEPT_5_CLERK'
    a, b = (p['data'] for p in (old, new))
    check(same_except(a['marts'], b['marts'], {shop}, 'shops') == {shop}
          and b['marts'][shop] == [*a['marts'][shop], TM], 'replacement TM counter')
    room = b['maps']['MnR10Cave1F']
    body = room['scripts'][GUIDE]
    commands = [(r['command'], r['args']) for r in body]
    check(commands.index(('checkevent', [RECEIPT])) < commands.index(('iftrue', ['MnGalleryDirections']))
          < commands.index(('verbosegiveitem', [TM, '1'])) < commands.index(('iffalse', ['MnGalleryKeepTM']))
          < commands.index(('setevent', [RECEIPT])), 'survey TM grant/receipt order')
    check(room['scripts']['MnGalleryDirections'] == a['maps'][room['id']]['scripts'][GUIDE]
          or room['scripts']['MnGalleryDirections'] == [{'command': 'jumptext', 'args': [GUIDE+'Text']}],
          'surveyor repeat directions')
    check(all(r['command'] != 'setevent' for r in room['scripts']['MnGalleryKeepTM']),
          'full pocket must allow retry')
    words = ' '.join(arg.strip(chr(34)) for row in room['scripts'][GUIDE+'Text'] for arg in row['args'])
    check('crew' in words and 'west on the road' in words and len(words.split()) <= 18,
          'concise directions to the working mine')
    check(any(r['item_id'] == TM and r['source_script'] == GUIDE for r in room['script_item_grants']),
          'missing compiled TM grant')
    check(any(r['args'][-1:] == [shop] and r['command'] == 'pokemart'
          for rows in b['maps']['MnMinneapolisDepartmentStore5F']['scripts'].values() for r in rows),
          'unreachable TM counter service')


def check_rock_scripts(old, new):
    root = 'RockSmashFromMenuScript'
    altered = 0
    for before, after in zip(old['story_events'], new['story_events'], strict=True):
        if before == after:
            continue
        same_except(before, after, {'StandardScripts'}, 'story catalogs')
        a, b = before['StandardScripts'], after['StandardScripts']
        same_except(a, b, {'GlobalScriptRoots'}, 'native standard programs')
        check(root not in a['GlobalScriptRoots'] and b['GlobalScriptRoots'] == [*a['GlobalScriptRoots'], root],
              'party entry must be the only new executable root')
        altered += 1
    check(altered == 1, 'missing native party entry')
    a, b = old['global_scripts'], new['global_scripts']
    check(a.keys() == b.keys() and a['definitions'] == b['definitions'], 'native script definitions changed')
    body = a['definitions'][root]
    check(root not in a['scripts'] and b['scripts'] == {**a['scripts'], root: body}
          and body[-1] == {'command': 'sjump', 'args': ['RockSmashScript']}, 'party Rock Smash dispatch')
    for name in ('AskRockSmashScript', 'SmashRockScript', 'RockSmashScript'):
        check(b['scripts'][name] == a['definitions'][name], name + ' executable')
    indices = {'script_map_commands': (0, 'refreshmap'), 'script_runtime_commands': (1, 'special'),
               'script_control_commands': (2, 'sjump')}
    for key in a.keys() - {'scripts', 'definitions'}:
        if key in indices:
            check([row for row in b[key] if row['source_script'] != root] == a[key],
                  'changed native command index: ' + key)
            added = [row for row in b[key] if row['source_script'] == root]
            check([(row['command_index'], row['command']) for row in added] == [indices[key]],
                  'missing native party command: ' + key)
        else:
            check(b[key] == a[key], 'changed native script metadata: ' + key)


def audit(parent, child):
    parent_hash, old = load(parent)
    check(parent_hash in PARENTS, 'expected a released regional-design parent')
    child_hash, new = load(child)
    check_origin(old, new, parent_hash)
    a, b = old['data'], new['data']
    same_except(a, b, {'maps', 'map_blocks', 'map_scripts', 'npcs', 'tilesets', 'marts',
                       'field_moves', 'story_event_script_constants', 'story_events', 'global_scripts'}, 'game data')
    check(a['maps'].keys() == b['maps'].keys(), 'map inventory')
    check(bluff_distances(old) == (45, 45) and bluff_distances(new) == (4, 45),
          'fossil bluff downhill return and ordinary climb')
    check_lookouts(old, new)
    check_tm(old, new)
    check_rock_scripts(a, b)
    plans = [json.loads((REGION / file).read_text()) for file in (
        'caves/iron-mine-junctions-v01/junctions.json',
        'revisions/wayfinding-v01/small-water-caves.json', 'revisions/trail-landmarks-v01/caches.json')]
    maps = {r['map'] for plan in plans for r in plan} | {
        CARGO, 'MnR10Cave1F', 'MnR10C00R00', 'MnR12C00R00', 'MnR01C00R00', 'MnR02C00R00', 'MnR04C00R00', 'MnR18C00R00'}
    changed = same_except(a['maps'], b['maps'], maps, 'changed map inventory')
    check(changed == maps, f'missing map changes: {maps - changed}')
    tilesets = {b['maps'][name]['attributes']['tileset_name'] for name in changed}
    same_except(a['tilesets'], b['tilesets'], tilesets, 'tileset scope')
    files = {POLICY, 'data/tilesets/mn_headwaters_native_backgrounds.json'}
    for ts in tilesets:
        files.update(f'data/tilesets/{ts}{suffix}' for suffix in (
            '.json', '_metatiles.bin', '_palette_map.json', '_foregrounds.json'))
        files.update(f'gfx/tilesets/{ts}{suffix}' for suffix in ('.png', '.2bpp'))
    same_except(old['runtime_files'], new['runtime_files'], files, 'runtime resources')
    check_original_art(old, new, tilesets)
    same_except(a['field_moves'], b['field_moves'], {'cut'}, 'field moves')
    check(a['field_moves']['cut'].keys() == b['field_moves']['cut'].keys(), 'Cut definition')
    same_except(a['field_moves']['cut'], b['field_moves']['cut'], {'replacements'}, 'Cut rules')
    for ts, rules in a['field_moves']['cut']['replacements'].items():
        check(all(b['field_moves']['cut']['replacements'][ts][k] == v for k, v in rules.items()),
              'previous Cut replacement changed in ' + ts)
    globals_before, globals_after = (data['story_event_script_constants']['global'] for data in (a, b))
    added_flags = set(globals_after) - set(globals_before)
    check(added_flags == {RECEIPT, 'EVENT_MN_R10_STAR_SEAM', 'EVENT_MN_R10_CROWN_SUPPLY',
                          'EVENT_MN_R18_ERRATIC_STONE'} and
          all(globals_after[k] == v for k, v in globals_before.items()), 'story and reward flags')
    changed_blocks = 0
    for name in changed:
        left, right = a['maps'][name], b['maps'][name]
        check(left['attributes'] == right['attributes'] and left['events']['warps'] == right['events']['warps'],
              name + ' entrances and connections')
        for key in ('trainer_scripts','scripted_trainer_battles','scripted_wild_battles','gift_pokemon_scripts'):
            check(left[key] == right[key], name + ' / ' + key)
        check(all(pickup in right['script_field_pickups'] for pickup in left['script_field_pickups']),
              name + ' lost an existing pickup')
        check(b['npcs'][name] == right['objects']
              and base64.b64decode(b['map_blocks'][right['attributes']['blocks_label']], validate=True)
              == bytes(right['blocks']) and b['map_scripts'][right['attributes']['map_events_label']]
              == [{k: r[k] for k in ('command', 'args')} for r in right['map_event_section_commands']],
              name + ' published indices')
        initial, revised = Terrain(old, name), Terrain(new, name)
        for at, old_kind in initial.tiles.items():
            now = revised.tiles[at]
            compatible_bluff = (name == 'MnR01C00R00' and at in ((24, 16), (25, 16))
                                and old_kind == 'FLOOR' and now == 'HOP_DOWN')
            check(old_kind == now or old_kind == 'WALL' or compatible_bluff,
                  f'{name} {at}: existing {old_kind} replaced with {now}')
        changed_blocks += sum(x != y for x,y in zip(left['blocks'],right['blocks'],strict=True))
    print(f'{len(a["maps"])} maps compared, {len(changed)} changed, {changed_blocks} changed metatiles.')
    print(f'Original day/night metatile pixels and collisions verified in {len(tilesets)} atlases.')
    print(f'SHA-256 {child_hash}\nContent {new["identity"]["content_hash"]}')


if __name__ == '__main__':
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('parent', type=Path)
    cli.add_argument('child', type=Path)
    args = cli.parse_args()
    audit(args.parent, args.child)
