#!/usr/bin/env python3
"""Check Minnesota's darkness and early Flash contract in a compiled pack.

Requires cbor2. This conservative structural check does not prove that the
surface route is traversable or replace an earned gameplay and visual review.
"""
import argparse
import hashlib
from pathlib import Path

import cbor2


def require(condition, message):
    if not condition:
        raise ValueError(message)


def check(pack):
    data = pack['data']
    maps = data['maps']
    metadata = data['runtime_map_metadata']
    regional_names = {entry['name'] for entry in metadata.values()
                      if entry.get('groupName', '').upper().startswith('MINNESOTA')}
    regional = {name: room for name, room in maps.items()
                if name.startswith('Mn') or name in regional_names}
    dark = set()
    for name, room in regional.items():
        attrs = room['attributes']
        runtime = metadata.get(attrs.get('map_constant'))
        require(runtime is not None, f'{name}: missing runtime map metadata')
        palette = str(attrs.get('palette', '')).lower().removeprefix('palette_')
        is_dark = palette == 'dark'
        require(is_dark == (attrs['phone_service'] & 0x0f == 4)
                == (runtime['phoneService'] & 0x0f == 4),
                f'{name}: palette and native darkness selector disagree')
        if is_dark:
            dark.add(name)
    require(dark == {'MnDriftlessLampGallery'},
            f'unreviewed Minnesota dark maps: {sorted(dark)}')

    hollow = maps['MnR01Cave1F']
    gallery = maps['MnDriftlessLampGallery']
    require(hollow['attributes']['palette'].lower() in ('nite', 'palette_nite'),
            'Driftless Hollow must remain lit')
    require(not gallery['attributes']['connections'], 'dark gallery has a surface connection')
    incoming = {name for name, room in maps.items()
                for warp in room['events']['warps']
                if warp['target_map'] in ('MnDriftlessLampGallery', 'MN_DRIFTLESS_LAMP_GALLERY')}
    require(incoming == {'MnR01Cave1F', 'MnDriftlessLampGallery'},
            f'dark gallery has an unreviewed entrance: {sorted(incoming)}')
    exits = {warp['target_map'] for warp in gallery['events']['warps']}
    require(exits == {'MnR01Cave1F', 'MN_DRIFTLESS_LAMP_GALLERY'},
            f'dark gallery has an unreviewed exit: {sorted(exits)}')
    require({w['target_map'] for w in hollow['events']['warps']} ==
            {'MnR01C00R00', 'MnR01Cave1F', 'MN_DRIFTLESS_LAMP_GALLERY'},
            'hollow connections changed; review before accepting')
    route = maps['MnR01C00R00']['attributes']['connections']
    require({(c['direction'], c['target_map']) for c in route} >=
            {('east', 'MnWinona'), ('west', 'MnRochester')},
            'the surface route no longer connects Winona and Rochester')

    flash = data['field_moves']['flash']
    require(flash['move_id'] == 'FLASH' and flash['badge'] == {'region': 'johto', 'index': 0},
            'native Flash permission changed; review progression')
    require(flash['engine_flag'] == 'STATUSFLAGS_FLASH', 'Flash status flag changed')
    actor = next((o for o in hollow['objects'] if o['script'] == 'MN_R01_HOLLOW_GUIDEFieldHM'), None)
    require(actor is not None, 'Flash guide is missing')
    script = hollow['scripts'][actor['script']]
    def position(command, argument):
        return next((i for i, entry in enumerate(script)
                     if entry['command'] == command and argument in entry['args']), -1)
    badge = position('checkflag', 'ENGINE_ZEPHYRBADGE')
    gift = position('verbosegiveitem', 'HM_FLASH')
    require(badge >= 0 and gift > badge and script[badge + 1]['command'] == 'iffalse',
            'Flash gift no longer checks the Care Badge before awarding')
    denied = script[badge + 1]['args'][0]
    denial_script = hollow['scripts'].get(denied, [])
    require(denial_script and all(entry['command'] != 'verbosegiveitem'
                                  for entry in denial_script),
            'Flash badge denial no longer follows the reviewed refusal script')
    item = data['items']['HM_FLASH']
    require(item['tmhm_move'] == 'FLASH' and not item['consumable'], 'Flash HM is not reusable')
    wild = data['wild_encounters']['MnR01C00R00']
    require(set(wild['grass']) == {'morning', 'day', 'night'},
            'R01 grass encounters must cover all three time periods')
    for period, slots in wild['grass'].items():
        sources = [('ordinary grass', slots)] + [
            (zone['id'], zone['grass'].get(period, []))
            for zone in wild.get('zones', []) if zone.get('grass')
        ]
        eligible = [(source, slot['species']) for source, entries in sources for slot in entries
                    if 'FLASH' in data['pokemon'][slot['species']]['tmhm_learnset']]
        require(eligible, f'no Flash-compatible R01 grass encounter in {period}')
        print(f'{period}: Flash-compatible R01 encounters: {sorted(set(eligible))}')
    print(f'PASS: {len(regional)} Minnesota maps checked; only optional gallery is dark')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pack', type=Path)
    args = parser.parse_args()
    raw = args.pack.read_bytes()
    print(f'{args.pack}: sha256 {hashlib.sha256(raw).hexdigest()}')
    check(cbor2.loads(raw[22:]))


if __name__ == '__main__':
    main()
