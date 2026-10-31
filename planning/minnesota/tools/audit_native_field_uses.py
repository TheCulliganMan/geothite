#!/usr/bin/env python3
"""Independently bound the native gallery/current delta; never writes game content."""
import argparse
import hashlib
from copy import deepcopy
import json
from render_reference import load_pack

HOLLOW = 'MnR01Cave1F'
GALLERY = 'MnDriftlessLampGallery'
LAKE = 'MnLeechLake'
BEMIDJI = 'MnBemidji'
FLAGS = {'EVENT_MN_LAMP_GALLERY_READ', 'EVENT_MN_LAMP_GALLERY_CACHE', 'EVENT_MN_LAMP_GALLERY_HIDDEN', 'EVENT_MN_LEECH_EDDY_CACHE'}
HINTS = {
    HOLLOW: ('MN_R01_HOLLOW_GUIDEFieldHMGivenText', ['A lower gallery is', 'unlit. FLASH shows', 'the fossil rings.']),
    LAKE: ('MN_LEECH_SHORE_FISHERText', ['An eddy hides a', 'tiny eastern cove.', 'WHIRLPOOL clears', 'the current.', 'Try ITEMFINDER at', 'the eastern rocks.']),
    BEMIDJI: ('MN_BEMIDJI_PIER_FISHERFieldHMGivenText', ['Try the eddy on', "LEECH LAKE's east", 'shore. A rock cove', 'lies behind it.'])}
ANIMATION = 'data/tilesets/mn_headwaters_animations.json'


def command(name, *args):
    return dict(command=name, args=list(args))


def origin(row):
    return next((row[key] for key in ['source_script', 'script_label', 'script_name'] if key in row), None)


def check_native_whirlpool_root(old, new, native_source):
    assert hashlib.sha256(native_source.encode()).hexdigest() == 'ebc02a71ca708b145292687150ae1c3e8617b2a32d042c090e6c68d16e0a7abb'
    forced = {}
    active = None
    for raw in native_source.splitlines():
        line = raw.split(';')[0].strip()
        if not line:continue
        if line.endswith(':') or line.startswith('.'):
            label = line.rstrip(':')
            active = label + '@Script_ForcedMovement' if label.startswith('.') else label
            assert active not in forced
            forced[active] = []
        else:
            parts = line.split(None, 1)
            forced[active].append(command(parts[0], *([arg.strip() for arg in parts[1].split(',')] if len(parts)>1 else [])))
    assert len(forced) == 9
    expected_events = deepcopy(old['story_events'])
    standard = next(event['StandardScripts'] for event in expected_events if 'StandardScripts' in event)
    if 'Script_UsedWhirlpool' not in standard['GlobalScriptRoots']:
        standard['GlobalScriptRoots'].append('Script_UsedWhirlpool')
    if 'Script_ForcedMovement' not in standard['GlobalScriptRoots']:
        standard['GlobalScriptRoots'].append('Script_ForcedMovement')
    if 'Script_UseFlash' not in standard['GlobalScriptRoots']:
        standard['GlobalScriptRoots'].append('Script_UseFlash')
    overworld = next(event['OverworldEvents'] for event in expected_events if 'OverworldEvents' in event)
    for label, body in forced.items():
        assert label not in overworld or overworld[label] == body
        overworld[label] = body
    assert new['story_events'] == expected_events, 'native current roots and pinned pushback source only'
    expected = deepcopy(old['global_scripts'])
    expected['definitions'].update(forced)
    expected['scripts'].update(forced)
    labels = {'Script_UsedWhirlpool', 'UseWhirlpoolText', '_UseWhirlpoolText',
              'Script_UseFlash', 'UseFlashTextScript', '_BlindingFlashText'}
    for label in labels:
        expected['scripts'][label] = expected['definitions'][label]
    for label in ['UseWhirlpoolText', '_UseWhirlpoolText', 'UseFlashTextScript', '_BlindingFlashText']:
        body = expected['definitions'][label]
        cutoff = next((index+1 for index,row in enumerate(body) if row['command']=='text_asm'), len(body))
        expected['script_text_bodies'][label] = dict(label=label, commands=[
            dict(command=row['command'], args=row['args'], command_index=index)
            for index, row in enumerate(body[:cutoff])])
    rows = {
        'script_map_commands': [dict(command='refreshmap', target_map=None, x=None, y=None,
            facing=None, map_setup=None, source_script='Script_UsedWhirlpool', command_index=2)],
        'script_text_commands': [dict(command='writetext', text_label='UseWhirlpoolText',
            source_script='Script_UsedWhirlpool', command_index=1),
            dict(command='closetext', text_label=None, source_script='Script_UsedWhirlpool', command_index=4)],
        'script_control_commands': [dict(command='end', compare_value=None, target_label=None,
            resolved_target_script=None, source_script='Script_UsedWhirlpool', command_index=5)],
        'script_runtime_commands': [dict(command='callasm', args=[target],
            source_script='Script_UsedWhirlpool', command_index=index)
            for target, index in [('GetPartyNickname', 0), ('DisappearWhirlpool', 3)]]}
    rows['script_map_commands'].append(dict(command='refreshmap', target_map=None, x=None, y=None,
        facing=None, map_setup=None, source_script='Script_UseFlash', command_index=0))
    rows['script_text_commands'] += [dict(command='writetext', text_label='UseFlashTextScript', source_script='Script_UseFlash', command_index=2),
        dict(command='closetext', text_label=None, source_script='Script_UseFlash', command_index=4)]
    rows['script_control_commands'].append(dict(command='end', compare_value=None, target_label=None,
        resolved_target_script=None, source_script='Script_UseFlash', command_index=5))
    rows['script_runtime_commands'] += [dict(command='special', args=['UpdateTimePals'], source_script='Script_UseFlash', command_index=1),
        dict(command='callasm', args=['BlindingFlash'], source_script='Script_UseFlash', command_index=3)]
    for label, body in forced.items():
        for index, row in enumerate(body):
            name, args = row['command'], row['args']
            origin_fields = dict(source_script=label, command_index=index)
            if name == 'readvar':
                rows.setdefault('script_variable_commands', []).append(dict(command=name, target=args[0], value_tokens=[], **origin_fields))
            elif name in {'end', 'ifequal'}:
                rows['script_control_commands'].append(dict(command=name,
                    compare_value=args[0] if args else None, target_label=args[1] if args else None,
                    resolved_target_script=args[1]+'@Script_ForcedMovement' if args else None, **origin_fields))
            elif name == 'applymovement':
                rows.setdefault('script_object_commands', []).append(dict(command=name, object_id=args[0],
                    target_object_id=None, x=None, y=None, direction=None, movement=args[1], emote=None, duration=None, **origin_fields))
        if label.startswith('.MovementData_'):
            rows.setdefault('script_movements', []).append(dict(label=label.split('@')[0], source_script='Script_ForcedMovement',
                steps=[dict(command=row['command'], direction=row['args'][0] if row['command'] in {'turn_in','turn_head'} else None,
                    duration=int(row['args'][0]) if row['command']=='step_dig' else None, index=index) for index,row in enumerate(body)]))
    for field, additions in rows.items():
        expected[field] = [row for row in expected[field] if origin(row) not in forced and origin(row) not in {'Script_UsedWhirlpool','Script_UseFlash'}] + additions
        assert sorted(expected[field], key=lambda row: json.dumps(row, sort_keys=True)) == sorted(
            new['global_scripts'][field], key=lambda row: json.dumps(row, sort_keys=True)), field
        expected[field] = new['global_scripts'][field]
    assert new['global_scripts'] == expected, 'native Whirlpool executable closure only'


def check(parent, child, site, native_source):
    old, new = parent['data'], child['data']
    for key in parent.keys() | child.keys():
        if key not in {'identity', 'data', 'report', 'runtime_files'}:
            assert parent.get(key) == child.get(key), key
    files = child['runtime_files']
    assert files.keys() == parent['runtime_files'].keys() | {ANIMATION}
    assert all(files[path] == value for path, value in parent['runtime_files'].items()), 'original art'
    clips = json.loads(bytes(files[ANIMATION]))
    assert set(clips) == {'50', '51', '66', '67'}, 'four native glyphs only'
    for index, glyph in enumerate(['50', '51', '66', '67'], 1):
        strip = list(parent['runtime_files'][f'gfx/tilesets/whirlpool/{index}.2bpp'])
        assert clips[glyph] == dict(frame_ticks=11, frames=[strip[n:n+16] for n in range(0, 64, 16)])
    report = deepcopy(parent['report'])
    report['manifests'] += ['minnesota-leech-eddy-native-animation-v1', 'northstar-lair-mndriftlesslampgallery', 'minnesota-native-field-uses-v1']
    report['reachable_maps'].append(GALLERY)
    report['solvable_maps'].append(GALLERY)
    report['maps'] += 1
    assert child['report'] == report
    mutable = {'maps', 'map_scripts', 'npcs', 'story_event_script_constants', 'wild_encounters',
               'runtime_map_metadata', 'map_dimensions', 'map_attributes', 'map_blocks', 'pokegear_landmarks',
               'story_events', 'global_scripts'}
    for key in old.keys() | new.keys():
        if key not in mutable:
            assert old.get(key) == new.get(key), key
    check_native_whirlpool_root(old, new, native_source)
    assert new['maps'].keys() == old['maps'].keys() | {GALLERY}
    for name, before in old['maps'].items():
        after = new['maps'][name]
        if name not in HINTS:
            assert before == after, ('unrelated map', name)
            continue
        assert before['attributes'] == after['attributes'], (name, 'attributes')
        assert before['objects'] == after['objects'], (name, 'actors')
        assert before['scenes'] == after['scenes'], (name, 'scenes')
        blocks = list(before['blocks'])
        if name == HOLLOW:
            assert blocks[4*8+5] == 2
            blocks[4*8+5] = 11
        elif name == LAKE:
            assert blocks[9*32+30] == blocks[10*32+30] == 89
            blocks[9*32+30], blocks[10*32+30] = 54, 7
        assert after['blocks'] == blocks, (name, 'bounded terrain')
        events = deepcopy(before['events'])
        if name == HOLLOW:
            assert len(after['events']['warps']) == len(events['warps']) + 2
            assert after['events']['warps'][:1] == events['warps'], 'original world exit'
            events['warps'] = after['events']['warps']
            assert [(r['x'], r['y'], r['target_map_constant'], r['target_warp_id']) for r in events['warps'][1:]] == [(11,9,'MN_DRIFTLESS_LAMP_GALLERY',2), (10,9,before['attributes']['map_constant'],3)]
        elif name == LAKE:
            events['bg_events'].append(dict(x=61, y=20, event_type='BGEVENT_ITEM', script='MN_LEECH_EDDY_CACHE'))
        assert after['events'] == events, (name, 'events')
        label, lines = HINTS[name]
        expected = deepcopy(before['scripts'])
        assert expected[label].pop() == command('done')
        expected[label] += [command('para' if index % 2 == 0 else 'line', '"'+line+'"') for index, line in enumerate(lines)]
        expected[label].append(command('done'))
        changed = {label}
        if name == LAKE:
            expected['MN_LEECH_EDDY_CACHE'] = [command('hiddenitem', 'PP_UP', 'EVENT_MN_LEECH_EDDY_CACHE')]
            changed.add('MN_LEECH_EDDY_CACHE')
        assert after['scripts'] == expected, (name, 'programs')
        for field in before.keys() | after.keys():
            if field in {'id', 'attributes', 'blocks', 'scenes', 'objects', 'events', 'scripts', 'map_event_section_commands'}:
                continue
            a, b = before[field], after[field]
            if isinstance(a, dict):
                for key, value in a.items():
                    if key not in changed:
                        assert b.get(key) == value, (name, field, key)
                assert b.keys() - a.keys() <= changed, (name, field, 'new labels')
            elif isinstance(a, list) and all(isinstance(row, dict) for row in a+b):
                assert [r for r in a if origin(r) not in changed] == [r for r in b if origin(r) not in changed], (name, field)
            else:
                assert a == b, (name, field)
    gallery = new['maps'][GALLERY]
    assert (gallery['attributes']['width'], gallery['attributes']['height'], gallery['attributes']['palette']) == (8,8,'dark')
    assert gallery['attributes']['time_of_day'] == 'dark'
    assert gallery['blocks'] == [int(v,16) for row in site['gallery']['rows'] for v in row.split()]
    assert gallery['attributes']['tileset_name'] == 'cave'
    assert gallery['attributes']['environment'] == 'CAVE'
    assert gallery['attributes']['connections'] == []
    assert [(r['x'],r['y'],r['target_map_constant'],r['target_warp_id']) for r in gallery['events']['warps']] == [(9,13,old['maps'][HOLLOW]['attributes']['map_constant'],3),(8,13,'MN_DRIFTLESS_LAMP_GALLERY',2)]
    assert len(gallery['objects']) == 1
    actor = gallery['objects'][0]
    assert (actor['sprite'], actor['x'], actor['y'], actor['event_flag']) == ('SPRITE_POKE_BALL',8,4,'EVENT_MN_LAMP_GALLERY_HIDDEN')
    assert actor['object_type'] == 'OBJECTTYPE_SCRIPT'
    gift = gallery['scripts']['MN_LAMP_GALLERY_CACHE']
    assert gift[:4] == [command('checkevent','EVENT_MN_LAMP_GALLERY_CACHE'),command('iftrue','MnLampCacheEnd'),command('checkevent','EVENT_MN_LAMP_GALLERY_READ'),command('iffalse','MnLampTooDark')]
    assert gift[5:9] == [command('verbosegiveitem','RARE_CANDY','1'),command('iffalse','MnLampCacheClose'),command('setevent','EVENT_MN_LAMP_GALLERY_CACHE'),command('disappear','MN_LAMP_GALLERY_CACHE')]
    read = gallery['scripts']['MnLampFossils']
    assert read[:2] == [command('checkflag','STATUSFLAGS_FLASH'),command('iffalse','MnLampTooDark')]
    assert read.index(command('waitbutton')) < read.index(command('setevent','EVENT_MN_LAMP_GALLERY_READ'))
    programs = {}
    for name in [HOLLOW, LAKE, BEMIDJI, GALLERY]:
        room = new['maps'][name]
        programs.update(room['scripts'])
        for attr, field in [('map_scripts_label','map_script_section_commands'), ('map_events_label','map_event_section_commands')]:
            programs[room['attributes'][attr]] = [command(row['command'], *row['args']) for row in room[field]]
        for label, body in room['script_text_bodies'].items():
            programs.setdefault(label, [command(row['command'], *row['args']) for row in body['commands']])
    assert new['map_scripts'].keys() == old['map_scripts'].keys() | programs.keys()
    for label, program in new['map_scripts'].items():
        assert program == programs.get(label, old['map_scripts'].get(label)), ('global mirror',label)
    constants = new['story_event_script_constants']
    assert constants.keys() == old['story_event_script_constants'].keys()
    for key, value in old['story_event_script_constants'].items():
        if key != 'global':assert constants[key] == value
    a, b = old['story_event_script_constants']['global'], constants['global']
    assert all(b.get(key) == value for key, value in a.items())
    assert b.keys() - a.keys() == FLAGS
    assert len({b[key] for key in FLAGS}) == 4
    assert not {b[key] for key in FLAGS} & {value for key,value in a.items() if key.startswith('EVENT_')}
    if HOLLOW in old['wild_encounters']:
        expected = deepcopy(old['wild_encounters'][HOLLOW]); expected['map_name'] = GALLERY
        assert new['wild_encounters'][GALLERY] == expected
    normalized = deepcopy(child)
    data = normalized['data']
    for table, key in [('maps',GALLERY), ('map_attributes',GALLERY),
                       ('map_dimensions','MN_DRIFTLESS_LAMP_GALLERY'),
                       ('runtime_map_metadata','MN_DRIFTLESS_LAMP_GALLERY'),
                       ('map_blocks',gallery['attributes']['blocks_label']), ('npcs',GALLERY)]:
        del data[table][key]
    assert data['pokegear_landmarks']['map_to_landmark'].pop(GALLERY) == old['pokegear_landmarks']['map_to_landmark'][HOLLOW]
    if HOLLOW in old['wild_encounters']:del data['wild_encounters'][GALLERY]
    for name in HINTS:
        for table,key in [('maps',name), ('map_attributes',name), ('map_blocks',old['maps'][name]['attributes']['blocks_label']), ('npcs',name)]:
            data[table][key] = deepcopy(old[table][key])
    for label in programs:
        if label in old['map_scripts']:data['map_scripts'][label] = deepcopy(old['map_scripts'][label])
        else:del data['map_scripts'][label]
    for flag in FLAGS:del data['story_event_script_constants']['global'][flag]
    data['story_events'] = deepcopy(old['story_events'])
    data['global_scripts'] = deepcopy(old['global_scripts'])
    del normalized['runtime_files'][ANIMATION]
    normalized['report'] = parent['report']
    normalized['identity'] = parent['identity']
    assert normalized == parent, 'unrelated complete content, metadata, HOME, catalogs, art or audio changed'
    return len(old['maps']) - 3


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['parent','child','site','native_source']:parser.add_argument(name)
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    site = json.loads(open(args.site).read())
    native_source = open(args.native_source).read()
    untouched = check(parent,child,site,native_source)
    rejected = []
    for case in ['unrelated-map','roster','audio','original-art','old-dialogue','cache-bypass']:
        altered = deepcopy(child)
        if case == 'unrelated-map':altered['data']['maps']['MnWinona']['blocks'][0] ^= 1
        elif case == 'roster':altered['data']['pokemon'].pop('VERDIGRIM')
        elif case == 'audio':altered['audio_manifest'] = None
        elif case == 'original-art':
            path = 'gfx/tilesets/mn_headwaters.2bpp'
            value = list(altered['runtime_files'][path]); value[0] ^= 1
            altered['runtime_files'][path] = value
        elif case == 'old-dialogue':
            room = altered['data']['maps'][BEMIDJI]
            label = next(k for k in parent['data']['maps'][BEMIDJI]['scripts'] if k != HINTS[BEMIDJI][0])
            room['scripts'][label] = []
        else:altered['data']['maps'][LAKE]['blocks'][9*32+30] = 89
        try:check(parent,altered,site,native_source)
        except AssertionError:rejected.append(case)
        else:raise AssertionError('failed to reject '+case)
    print(json.dumps(dict(unchanged_complete_maps=untouched,rejected_negative_cases=rejected,
        caution='Independent preservation only. Native permissions, teaching, Itemfinder, capacity/retry, visuals and earned campaign require separate evidence.'),indent=2))


if __name__ == '__main__':main()
