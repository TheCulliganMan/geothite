#!/usr/bin/env python3
"""Compare a Minnesota Crossings pack to its released Wayfinding parent, without playing."""
import argparse
import base64
from collections import deque
from io import BytesIO
import json
import re
from pathlib import Path

from PIL import Image

from audit_regional_design import DIRS, POLICY, Terrain, check, load, same_except
from render_reference import SourceArt

ROCK_MAPS = {'MnR12C00R00', 'MnR18C00R00'}
MAPS = {'MnR15C00R00', 'MnFormerShores', 'MnLeagueHallOfFame'} | ROCK_MAPS
PARENTS = {'e0638a9327e3296b214cdb8d3fdb6e25c33616cdacd2adac583bdd4eb70958bd',
           'e17218b24ad2b8292d17658b1176e7e093851f7502b2b5a6177f67edf27d97c4'}
MARKER = 'northstar-crossings-v1'
MENU = 'Script_StrengthFromMenu'
USED = 'Script_UsedStrength'
HALL = 'MnLeagueHallOfFame'
REGISTER = 'MnElyReadRegister'
PHOTO = 'MnElyReadFamilyPortrait'
AWARD = 'MnElyRecordVictory'
ROOM_ATLAS = 'mn_ely_register'


def hardware_bank(files, atlas, time):
    private = f'gfx/tilesets/{atlas}.pal'
    source = files.get(private, files['gfx/tilesets/bg_tiles.pal'])
    banks, group = [], None
    for raw in bytes(source).decode('ascii').splitlines():
        line, _, comment = raw.strip().partition(';')
        if not line:
            group = comment.strip().lower()
        elif line.startswith('RGB') and (private in files or group == time):
            components = [int(n) for n in line[3:].replace(',', ' ').split()]
            check(len(components) == 12 and all(0 <= n <= 31 for n in components), 'not four RGB555 colors')
            srgb = bytes((n << 3) | (n >> 2) for n in components)
            banks.append([srgb[i:i+3] for i in range(0, 12, 3)])
    check(len(banks) == 8, f'{atlas}/{time} eight native BG palettes')
    return banks


def verify_new_glyphs(old, new):
    art, files = SourceArt(new), new['runtime_files']
    counts = {}
    for name in ('kanto', 'mn_prairie_headwaters', 'mn_headwaters', ROOM_ATLAS):
        parent_atlas = 'traditional_house' if name == ROOM_ATLAS else name
        inherited = bytes(old['runtime_files'][f'data/tilesets/{parent_atlas}_metatiles.bin'])
        layouts = bytes(files[f'data/tilesets/{name}_metatiles.bin'])
        new_ids = set(layouts[len(inherited):]) - set(inherited)
        check(layouts.startswith(inherited) and new_ids, name + ' expected appended native art')
        substitutions = set()
        for suffix in ('foregrounds', 'native_backgrounds'):
            overrides = files.get(f'data/tilesets/{name}_{suffix}.json', b'{}')
            substitutions.update(map(int, json.loads(bytes(overrides))))
        check(not (new_ids & substitutions), name + ' substituted colors on an authored 8×8 glyph')
        planes = bytes(files[f'gfx/tilesets/{name}.2bpp'])
        sheet = Image.open(BytesIO(bytes(files[f'gfx/tilesets/{name}.png']))).convert('L')
        count = len(planes) // 16
        check(len(planes) % 16 == 0 and 0 < count <= 256 and sheet.width * sheet.height == count * 64,
              name + ' native 2bpp atlas shape')
        attrs = new['data']['tilesets'][name]['palette_map']
        times = ('indoor',) if name == ROOM_ATLAS else ('morn', 'day', 'nite')
        banks = {time: hardware_bank(files, name, time) for time in times}
        for glyph in sorted(new_ids):
            attr = attrs[glyph]
            slot = (glyph & 0x7f) + count // 2 if attr & 8 else glyph
            check(attr < 16 and slot < count, f'{name}/{glyph:02x} physical VRAM address')
            # Prairie's pre-existing seasonal tint affects palette seven only.
            check(name != 'mn_prairie_headwaters' or attr & 7 != 7, 'new prairie art inherited a computed tint')
            bitmap = planes[slot*16:slot*16+16]
            levels = [((bitmap[y*2] >> shift) & 1) | (((bitmap[y*2+1] >> shift) & 1) << 1)
                      for y in range(8) for shift in range(7, -1, -1)]
            left, top = (slot % (sheet.width//8)) * 8, (slot // (sheet.width//8)) * 8
            check(sheet.crop((left, top, left+8, top+8)).tobytes() == bytes(255 - 85*i for i in levels),
                  f'{name}/{glyph:02x} physical bitplanes and PNG disagree')
            cell = layouts.index(glyph, len(inherited))
            block, part = divmod(cell, 16)
            x, y = (part % 4) * 8, (part // 4) * 8
            for time in times:
                palette = banks[time][attr & 7]
                expected = b''.join(palette[i] for i in levels)
                actual = art.blocks(name, time)[block].crop((x, y, x+8, y+8)).tobytes()
                check(actual == expected, f'{name}/{glyph:02x}/{time} glyph must use one four-color palette')
        counts[name] = len(new_ids)
    return counts


def verify_art(old, new, prior_atlases):
    previous, revised = SourceArt(old), SourceArt(new)
    total = 0
    for name in sorted(prior_atlases | {'traditional_house', 'ice_path'}):
        a, b = (pack['data']['tilesets'][name] for pack in (old, new))
        same_except(a, b, {'collision', 'palette_map'}, name + ' tileset')
        path = f'data/tilesets/{name}_metatiles.bin'
        check(bytes(new['runtime_files'][path]).startswith(bytes(old['runtime_files'][path])),
              name + ' original native block composition')
        for block, collision in a['collision'].items():
            check(b['collision'].get(block) == collision, f'{name}/{block} original collision')
        for time in ('morn', 'day', 'nite', 'indoor'):
            before, after = previous.blocks(name, time), revised.blocks(name, time)
            check(len(after) >= len(before), name + ' shortened atlas')
            for block, image in enumerate(before):
                check(image.tobytes() == after[block].tobytes(), f'{name}/{block:02x}/{time} original pixels')
            if time == 'day':
                total += len(before)
    return total


def verify_hall(old, new):
    before, after = (Terrain(pack, HALL) for pack in (old, new))
    check((before.width, after.width, len(before.tiles), len(after.tiles)) == (5, 7, 140, 196),
          'Hall of Fame dimensions')
    check(all(after.tiles.get(at) == kind for at, kind in before.tiles.items()), 'changed old Hall footing')
    for door in [(4, 11), (5, 11)]:
        old_floor, floor = before.reach([door]), after.reach([door])
        gallery = {(x, y) for x in range(10, 13) for y in range(7, 10)} | {(10, 10)}
        check(floor == old_floor | gallery, 'Hall access or east aisle')
        check({(5, 6), (6, 5), (3, 9), (11, 7), (12, 7), (4, 11), (5, 11)} <= floor,
              'recorder, book, family, portrait and return to both doorways')
    a, b = before.map, after.map
    events = b['events']['bg_events']
    check(not a['events']['bg_events'] and [(e['x'], e['y'], e['script']) for e in events]
          == [(6, 4, REGISTER), (11, 6, PHOTO), (12, 6, PHOTO)], 'Hall exhibit positions')
    check(all(e['event_type'] == 'BGEVENT_READ' and after.tiles[e['x'], e['y']] == 'WALL' for e in events),
          'readable book and portrait positions')
    check(all(b['scripts'][k] == body for k, body in a['scripts'].items()),
          'original family story, victory checks, saved script indices and rematch behavior')
    actor = next(o for o in a['objects'] if o['script'] == AWARD)
    def cmds(name):
        return [(r['command'], r['args']) for r in b['scripts'][name]]
    check(cmds(REGISTER) == [('setlasttalked', [actor['object_identifier']]), ('sjump', [AWARD])],
          'public book must run the recorder’s original program')
    check(any(r['source_script'] == REGISTER and r['resolved_target_script'] == AWARD
              for r in b['script_control_commands']), 'unresolved register jump')
    check(cmds(PHOTO) == [('pokepic', ['JIGLET']), ('waitbutton', []), ('closepokepic', []),
                          ('jumptext', ['MnElyPortraitCaption'])], 'optional Jiglet portrait and caption')
    files = new['runtime_files']
    check(files['gfx/pokemon/jiglet/front.dimensions'][0] == 0x77
          and len(files['gfx/pokemon/jiglet/front.2bpp']) >= 7 * 7 * 16
          and 'JIGLET' in new['data']['pokemon'], 'native photo portrait')
    check(before.tileset == 'ice_path' and after.tileset == ROOM_ATLAS, 'private Hall atlas')
    text = bytes(files[f'gfx/tilesets/{ROOM_ATLAS}.pal']).decode('ascii')
    banks = [[int(n) for n in re.findall(r'\d+', line.partition(';')[0])] for line in text.splitlines() if line.strip()]
    check(len(banks) == 8 and all(len(row) == 12 and all(0 <= c <= 31 for c in row) for row in banks),
          'eight real RGB555 palettes, four colors each')
    check(not any(f'data/tilesets/{ROOM_ATLAS}_{kind}.json' in files
                  for kind in ('foregrounds', 'native_backgrounds')), 'Hall must use only its native palettes')
    tileset = new['data']['tilesets'][ROOM_ATLAS]
    gfx = files[f'gfx/tilesets/{ROOM_ATLAS}.2bpp']
    layout = files[f'data/tilesets/{ROOM_ATLAS}_metatiles.bin']
    check(len(gfx) == 192 * 16, 'native 8×8 two-bit Hall glyphs')
    check(all(0 <= tileset['palette_map'][i] <= 15 for i in layout), 'bank and palette attributes')
    check(json.loads(bytes(files[f'data/tilesets/{ROOM_ATLAS}_palette_map.json'])) == tileset['palette_map']
          and json.loads(bytes(files[f'data/tilesets/{ROOM_ATLAS}.json'])) == tileset['collision'],
          'native Hall runtime tables')


def changed_placements(before, after):
    def layout(room):
        width = room['attributes']['width']
        return {(i % width, i // width): block for i, block in enumerate(room['blocks'])}
    old, new = layout(before), layout(after)
    return sum(old.get(at) != new.get(at) for at in old.keys() | new.keys())


def verify_rocks(old, new):
    edits = {
        'MnR12C00R00': [(16, 6, 81, 45), (17, 6, 82, 74), (17, 5, 83, 50),
                         (18, 5, 84, 50), (17, 7, 85, 60), (24, 3, 86, 77)],
        'MnR18C00R00': [(1, 10, 0x86, 0x87)],
    }
    for name, placements in edits.items():
        before, after = (Terrain(pack, name) for pack in (old, new))
        check(before.tiles == after.tiles, name + ' changed playable ground under stone')
        check(len(placements) == changed_placements(before.map, after.map), name + ' unrelated art change')
        for x, y, left, right in placements:
            check(before.block(2*x, 2*y) == left and after.block(2*x, 2*y) == right, name + ' rock location')


def verify_shores(old, new):
    before, after = (Terrain(pack, 'MnFormerShores') for pack in (old, new))

    def distances(grid, start):
        dist, todo = {start: 0}, deque([start])
        blocked = grid.occupied | grid.warps
        while todo:
            x, y = point = todo.popleft()
            for dx, dy in DIRS.values():
                dest = (x + dx, y + dy)
                entry = grid.tiles.get(dest)
                if dest in dist or dest in blocked or entry not in {'FLOOR', 'TALL_GRASS', 'UP_WALL'}:
                    continue
                if (dy == -1 and grid.tiles[point] == 'UP_WALL') or (dy == 1 and entry == 'UP_WALL'):
                    continue
                dist[dest] = dist[point] + 1
                todo.append(dest)
        return dist

    dry_before, dry_after = (distances(g, (14, 0)).keys() for g in (before, after))
    check(dry_before < dry_after and len(dry_after - dry_before) == 8, 'eight new banks and prior dry access')
    surveyor, high, middle, low, den = (17, 6), (16, 9), (20, 15), (12, 21), (18, 22)
    legs = [(surveyor, high), (high, middle), (middle, low), (low, surveyor), (surveyor, den)]
    for grid, expected in [(before, [18, 18, 16, 36, 37]), (after, [6, 10, 14, 22, 23])]:
        check([distances(grid, a).get(b) for a, b in legs] == expected, 'dry survey loop and return')
    waters = {at for at, kind in after.tiles.items() if kind == 'WATER'}
    check(len(waters) == 22 and all(28 <= x <= 33 and 4 <= y <= 11 for x, y in waters)
          and (30, 8) in dry_after and (31, 8) in waters and before.map['events'] == after.map['events']
          and after.map['attributes']['fishing_group'] == 'FISHGROUP_MN_QWILFISH', 'enclosed fishing swale and observations')


def verify_lake(new):
    grid = Terrain(new, 'MnR15C00R00')
    channel = ['WATER', 'BRIDGE_NS_RAIL', *['BRIDGE_NS_DECK'] * 4, 'BRIDGE_NS_RAIL', 'WATER', 'WATER']
    check(all([grid.tiles[x, y] for y in range(24, 33)] == channel for x in range(38, 46)),
          'eight unobstructed channels beneath the bridge')
    check(grid.tiles[37, 28] == grid.tiles[46, 28] == grid.tiles[49, 32] == 'FLOOR'
          and grid.tiles[50, 32] == 'WALL', 'bridge approaches and creel interaction')
    reward = 'MN_R15_SHINER_CREEL'
    check([rec for rec in grid.map['events']['bg_events'] if rec['script'] == reward] ==
          [{'x': 50, 'y': 32, 'event_type': 'BGEVENT_ITEM', 'script': reward}], 'hidden creel event')
    pickups = [r for r in grid.map['script_field_pickups'] if r['source_script'] == reward]
    check(len(pickups) == 1 and (pickups[0]['item_id'], pickups[0]['quantity'], pickups[0]['event_flag'])
          == ('LURE_BALL', 1, 'EVENT_' + reward), 'one-time Lure Ball pickup')


def verify_strength(left, right):
    before, after = left['global_scripts'], right['global_scripts']
    old, new = before['definitions'][MENU], after['definitions'][MENU]
    check(MENU not in before['scripts'] and new == [*old, {'command': 'sjump', 'args': [USED]}],
          'missing native Strength fallthrough')
    check(after['scripts'] == {**before['scripts'], MENU: new}
          and after['definitions'] == {**before['definitions'], MENU: new}, 'global script changes')
    expected = {'script_map_commands': (0, 'refreshmap'), 'script_runtime_commands': (1, 'special'),
                'script_control_commands': (2, 'sjump')}
    for k in before.keys() - {'definitions', 'scripts'}:
        if k not in expected:
            check(before[k] == after[k], 'global program index: ' + k)
            continue
        check([r for r in after[k] if r['source_script'] != MENU] == before[k], 'earlier program index: ' + k)
        added = [r for r in after[k] if r['source_script'] == MENU]
        check([(r['command_index'], r['command']) for r in added] == [expected[k]], 'party Strength ' + k)
    jumps = [r for r in after['script_control_commands'] if r['source_script'] == MENU]
    check(jumps[0]['resolved_target_script'] == USED, 'wrong boulder activation program')
    catalogs = set()
    for old_event, new_event in zip(left['story_events'], right['story_events'], strict=True):
        for catalog in same_except(old_event, new_event, {'OverworldEvents', 'StandardScripts'}, 'story catalogs'):
            a, b = old_event[catalog], new_event[catalog]
            check(catalog not in catalogs, 'duplicate catalog'); catalogs.add(catalog)
            if catalog == 'OverworldEvents':
                check(b == {**a, MENU: new}, 'unexpected field program replacement')
            else:
                check(b == {**a, 'GlobalScriptRoots': [*a['GlobalScriptRoots'], MENU]}, 'changed script roots')
    check(catalogs == {'OverworldEvents', 'StandardScripts'}, 'missing menu entry')


def audit(parent, candidate):
    sha, old = load(parent); digest, new = load(candidate)
    check(sha in PARENTS, 'expected released Wayfinding parent')
    a, b = old['data'], new['data']
    same_except(old, new, {'data', 'identity', 'report', 'runtime_files'}, 'pack')
    same_except(old['report'], new['report'], {'manifests'}, 'compiler report')
    manifests = old['report']['manifests'].copy()
    start = manifests.index('northstar-wayfinding-v1') + 1
    manifests.insert(start, MARKER)
    check(new['report']['manifests'] == manifests, 'manifest lineage')
    policy = json.loads(bytes(new['runtime_files'][POLICY]))
    check(policy['schema_version'] == 7 and policy['target_manifest'] == MARKER
          and policy['inherited'] == json.loads(bytes(old['runtime_files'][POLICY]))
          and {s['modpack_hash'] for s in policy['sources']} == PARENTS
          and any(s['modpack_hash'] == sha and s['content_hash'] == old['identity']['content_hash']
                  and s['modpack_id'] == old['identity']['runtime_modpack_id'] for s in policy['sources'])
          and new['identity']['runtime_modpack_id'] in policy['target_modpack_ids'], 'save lineage')
    same_except(a, b, {'maps', 'map_attributes', 'map_dimensions', 'runtime_map_metadata',
                       'map_blocks', 'map_scripts', 'npcs', 'tilesets',
                       'story_events', 'global_scripts', 'story_event_script_constants'}, 'game data')
    check(same_except(a['maps'], b['maps'], MAPS, 'changed maps') == MAPS, 'expected five changed maps')
    verify_strength(a, b)
    flag = 'EVENT_MN_R15_SHINER_CREEL'
    old_constants, new_constants = a['story_event_script_constants'], b['story_event_script_constants']
    check(same_except(old_constants, new_constants, {'global'}, 'script constants') == {'global'}
          and same_except(old_constants['global'], new_constants['global'], {flag}, 'event flags') == {flag}
          and flag not in old_constants['global'], 'one new creel reward flag')
    tilesets = {b['maps'][n]['attributes']['tileset_name'] for n in MAPS}
    check(tilesets == {'kanto', 'mn_prairie_headwaters', 'mn_winter_gunflint', 'mn_headwaters', ROOM_ATLAS},
          'expected affected art families')
    changed_atlases = same_except(a['tilesets'], b['tilesets'], tilesets, 'tileset scope')
    check(changed_atlases == tilesets - {'mn_winter_gunflint'}, 'Gunflint must reuse its existing cliff blocks')
    files = {POLICY}
    for ts in tilesets:
        files.update('data/tilesets/' + ts + suffix for suffix in ('.json', '_metatiles.bin', '_palette_map.json',
                     '_foregrounds.json', '_native_backgrounds.json'))
        files.update('gfx/tilesets/' + ts + suffix for suffix in ('.png', '.2bpp'))
    files.add(f'gfx/tilesets/{ROOM_ATLAS}.pal')
    same_except(old['runtime_files'], new['runtime_files'], files, 'runtime files')
    prior_atlases = tilesets - {ROOM_ATLAS}
    old_blocks = verify_art(old, new, prior_atlases)
    glyphs = verify_new_glyphs(old, new)
    same_except(a['map_attributes'], b['map_attributes'], MAPS, 'map attributes')
    same_except(a['npcs'], b['npcs'], MAPS, 'actor table')
    same_except(a['map_blocks'], b['map_blocks'], {b['maps'][m]['attributes']['blocks_label'] for m in MAPS}, 'block index')
    event_tables = {b['maps'][m]['attributes']['map_events_label'] for m in MAPS}
    new_labels = {label for m in MAPS for label in b['maps'][m]['scripts'].keys() - a['maps'][m]['scripts'].keys()}
    same_except(a['map_scripts'], b['map_scripts'], event_tables | new_labels, 'script index')
    constant = b['maps'][HALL]['attributes']['map_constant']
    for table in ('map_dimensions', 'runtime_map_metadata'):
        check(same_except(a[table], b[table], {constant}, table) == {constant}, 'Hall dimension propagation')
        check(b[table][constant] == {**a[table][constant], 'width': 7}, 'Hall catalog width ' + table)
    changes = 0
    for name in MAPS:
        l, r = a['maps'][name], b['maps'][name]; attr = r['attributes']
        if name in ROCK_MAPS:
            same_except(l, r, {'blocks'}, name + ' terrain-only edit')
        check(l['events']['warps'] == r['events']['warps'] and l['objects'] == r['objects'], name + ' warps and actors')
        for key in ('trainer_scripts', 'scripted_trainer_battles', 'scripted_wild_battles', 'gift_pokemon_scripts'):
            check(l[key] == r[key], name + ' ' + key)
        check(b['npcs'][name] == r['objects'] and b['map_attributes'][name] == attr
              and base64.b64decode(b['map_blocks'][attr['blocks_label']], validate=True) == bytes(r['blocks']),
              name + ' published map indices')
        check(all(p in r['script_field_pickups'] for p in l['script_field_pickups']), name + ' existing reward')
        changes += changed_placements(l, r)
    verify_hall(old, new)
    verify_rocks(old, new)
    verify_shores(old, new)
    verify_lake(new)
    print(f'{len(a["maps"])} map modules compared, {len(MAPS)} changed. {changes} changed block placements.')
    print(f'{old_blocks} original blocks and their footing verified across {len(prior_atlases) + 2} atlases and four palette banks.')
    print('New 8×8 two-bit glyphs verified against PNG, physical VRAM addresses and RGB555 palettes: '
          + ', '.join(f'{k} {n}' for k, n in glyphs.items()) + '.')
    print(f'140 original Hall squares retained, ten new gallery squares accessible. SHA-256 {digest}.')
    print('Content ' + new['identity']['content_hash'])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('candidate', type=Path)
    args = parser.parse_args()
    audit(args.parent, args.candidate)
