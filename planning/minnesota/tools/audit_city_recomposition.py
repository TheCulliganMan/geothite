#!/usr/bin/env python3
"""Independent written-pack check for exact compact city/Falls changes.

Checks every native module, split index, catalogue, old program and source glyph.
This does not establish controller play, earned access or visual acceptance.
"""
import argparse
import base64
from copy import deepcopy
import hashlib
import io
import json
from pathlib import Path

from PIL import Image

from audit_city_expansions import module_hash, section_rows
from audit_winding_shelter_541 import raw_identity
from render_reference import layout_blocks, load_pack

if not __debug__:
    raise SystemExit('Native verification requires Python assertions; omit -O')


def command(name, *args):
    return dict(command=name, args=list(args))


def gate(module):
    check, refresh, opened, done, denied, text = [
        'MnMPUpperPoolBuoy' + suffix for suffix in
        ['Check', 'Refresh', 'Open', 'Done', 'Denied', 'DeniedText']]
    c = command
    scripts = {
        opened: [c('changeblock', str(x), '2', str(block))
                 for x, block in [(32, 88), (34, 53), (36, 89)]] + [c('end')],
        done: [c('end')],
        denied: [c('opentext'), c('writetext', text), c('waitbutton'),
                 c('closetext'), c('end')],
        text: [c('text', '"River gate closed."'), c('line', '"Earn CURRENT and"'),
               c('para', '"CAPITOL BADGES."'), c('done')],
    }
    for label, fail in [(check, denied), (refresh, done)]:
        scripts[label] = [c('checkflag', 'ENGINE_PLAINBADGE'), c('iffalse', fail),
                          c('checkflag', 'ENGINE_FOGBADGE'), c('iffalse', fail),
                          c('sjump', opened)]
    assert not set(scripts) & set(module['scripts'])
    module['scripts'].update(scripts)
    module['script_text_bodies'][text] = dict(label=text, commands=[
        dict(row, command_index=i) for i, row in enumerate(scripts[text])])
    # Native parsers visit a BTreeMap, retaining per-program command indices.
    for label, rows in sorted(scripts.items()):
        for index, row in enumerate(rows):
            name, args = row['command'], row['args']
            source = dict(source_script=label, command_index=index)
            if name == 'checkflag':
                module['script_flag_commands'].append(dict(source, command=name, flag_id=args[0]))
            if name in ['iffalse', 'sjump', 'end']:
                target = args[0] if args else None
                module['script_control_commands'].append(dict(source, command=name,
                    compare_value=None, target_label=target, resolved_target_script=target))
            if name in ['opentext', 'writetext', 'waitbutton', 'closetext']:
                module['script_text_commands'].append(dict(source, command=name,
                    text_label=args[0] if args else None))
            if name == 'changeblock':
                module['script_block_changes'].append(dict(source,
                    x=int(args[0]), y=int(args[1]), block_id=int(args[2])))
    module['events']['coord_events'] += [dict(x=x, y=y, scene_id='-1', script_name=check)
        for y in [1, 4] for x in range(32, 38)]
    section = module['map_script_section_commands']
    section.append(dict(command='callback', args=['MAPCALLBACK_TILES', refresh], command_index=len(section)))


def source_art(before, after, expected):
    name = 'mn_minneapolis_city'
    old = before['data']['tilesets'][name]
    cave = before['data']['tilesets']['cave']
    a, b = [{key: bytes(raw) for key, raw in pack['runtime_files'].items()}
            for pack in [before, after]]
    meta, graphics = f'data/tilesets/{name}_metatiles.bin', f'gfx/tilesets/{name}.2bpp'
    assert len(a[meta]) == 153 * 16 and len(a[graphics]) == 192 * 16
    assert b[meta][:len(a[meta])] == a[meta] and len(b[meta]) == len(a[meta]) + 16
    glyph = b[meta][-1]
    assert b[meta][-16:] == bytes([glyph]) * 16 and glyph not in a[meta]
    palette = after['data']['tilesets'][name]['palette_map'][glyph]
    physical = lambda tile, color: tile if color & 8 == 0 else (tile & 127) + 96
    slot = physical(glyph, palette)
    assert slot not in {physical(t, old['palette_map'][t]) for t in a[meta]}
    source_glyph = a['data/tilesets/cave_metatiles.bin'][0x2c * 16]
    pixels = a['gfx/tilesets/cave.2bpp'][source_glyph * 16:(source_glyph + 1) * 16]
    wanted = bytearray(a[graphics]); wanted[slot * 16:(slot + 1) * 16] = pixels
    assert bytes(wanted) == b[graphics]
    definition = deepcopy(old)
    definition['palette_map'][glyph] = cave['palette_map'][source_glyph] | (palette & 8)
    definition['collision']['99'] = cave['collision']['2c']
    assert definition == after['data']['tilesets'][name]
    assert json.loads(b[f'data/tilesets/{name}.json']) == definition['collision']
    assert json.loads(b[f'data/tilesets/{name}_palette_map.json']) == definition['palette_map']
    image = Image.open(io.BytesIO(b[f'gfx/tilesets/{name}.png'])).convert('L')
    assert image.size == (128, 96)
    # Palette-index PNG pixels must encode precisely these 192 2bpp glyphs.
    # Existing native PNG uses grayscale 255,170,85,0 for indices0..3.
    for tile in range(192):
        for y in range(8):
            lo, hi = wanted[tile * 16 + y * 2:tile * 16 + y * 2 + 2]
            for x in range(8):
                color = ((lo >> (7-x)) & 1) | (((hi >> (7-x)) & 1) << 1)
                assert image.getpixel(((tile % 16) * 8+x, (tile // 16) * 8+y)) == 255-color*85
    expected['tilesets'][name] = definition
    changed = {meta, graphics, f'gfx/tilesets/{name}.png', f'data/tilesets/{name}.json',
               f'data/tilesets/{name}_palette_map.json'}
    assert set(a) == set(b)
    assert {k for k in a if a[k] != b[k]} == changed


def audit(before, after, layout, contract):
    a, b = before['data'], after['data']
    expected = deepcopy(a)
    city = contract['city']; module = expected['maps'][city]
    assert module_hash(module) == contract['native_module_sha256']
    width, height = contract['dimensions_metatiles']
    module['attributes'].update(width=width, height=height)
    module['blocks'] = layout_blocks(layout)
    fields = {'object': module['objects'], 'warp': module['events']['warps'],
              'background': module['events']['bg_events'], 'coordinate': module['events']['coord_events']}
    complete = {(kind, i) for kind, rows in fields.items() for i in range(len(rows))}
    anchors = contract['anchor_overrides']
    assert {(v['kind'], v['index']) for v in anchors} == complete and len(anchors) == len(complete)
    for anchor in anchors:
        row = fields[anchor['kind']][anchor['index']]
        assert [row['x'], row['y']] == anchor['before']
        row['x'], row['y'] = anchor['after']
    owned = {city}
    for link in contract['connections']:
        neighbor = expected['maps'][link['target']]; owned.add(link['target'])
        local = next(c for c in module['attributes']['connections'] if c['direction'] == link['direction'])
        assert local['target_map'] == link['target'] and local['offset'] == link['old_offset']
        local['offset'] = link['offset']
        remote = next(c for c in neighbor['attributes']['connections'] if c['target_map'] == city)
        assert remote['offset'] == -link['old_offset']; remote['offset'] = -link['offset']
    for revision in contract.get('neighbor_terrain', []):
        neighbor = expected['maps'][revision['map']]
        assert module_hash(a['maps'][revision['map']]) == revision['native_module_sha256']
        for change in revision['changes']:
            x, y = change['metatile']; index = y * neighbor['attributes']['width'] + x
            assert neighbor['blocks'][index] == change['before']; neighbor['blocks'][index] = change['after']
    if contract.get('north_water_checkpoint'):
        assert [module['blocks'][40 + x] for x in [16, 17, 18]] == [49] * 3
        gate(module)
    module['map_event_section_commands'] = [dict(row, command_index=i) for i, row in enumerate(section_rows(module))]
    for name in owned:
        native = expected['maps'][name]; attrs = native['attributes']
        expected['map_attributes'][name] = deepcopy(attrs)
        expected['npcs'][name] = deepcopy(native['objects'])
        expected['map_blocks'][attrs['blocks_label']] = base64.b64encode(bytes(native['blocks'])).decode()
        for field, attr in [('map_event_section_commands', 'map_events_label'), ('map_script_section_commands', 'map_scripts_label')]:
            expected['map_scripts'][attrs[attr]] = [{k:v for k,v in row.items() if k != 'command_index'} for row in native[field]]
        expected['map_scripts'].update(native['scripts'])
    constant = module['attributes']['map_constant']
    expected['map_dimensions'][constant] = dict(width=width, height=height)
    expected['runtime_map_metadata'][constant].update(width=width, height=height)
    spawns = contract['spawn_overrides']
    assert {s['identifier'] for s in spawns} == {s['identifier'] for s in a['runtime_spawn_points'].values() if s['mapName'] == city}
    for spawn in spawns:
        record = expected['runtime_spawn_points'][str(spawn['identifier'])]
        assert [record['tileX'], record['tileY']] == spawn['before']
        x, y = spawn['after']
        record.update(tileX=x, tileY=y, metatileX=x//2, metatileY=y//2, subtileX=x%2, subtileY=y%2)
    if contract.get('waterfall_block') is not None:
        source_art(before, after, expected)
    else:
        assert before['runtime_files'] == after['runtime_files']
    for field in expected:
        assert expected[field] == b[field], 'unexpected native dataset drift: ' + field
    assert set(expected) == set(b)
    for field in before:
        if field not in ['data', 'runtime_files', 'identity']:
            assert before[field] == after[field], 'native top-level drift: ' + field
    assert before['identity'] == dict(after['identity'], content_hash=before['identity']['content_hash'])
    return dict(maps=len(a['maps']), completely_preserved_maps=len(a['maps'])-len(owned),
                changed_maps=sorted(owned), all_rooms_programs_catalogues_exact_except_bounded_gate=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['before', 'pack', 'layout', 'contract', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    inputs = {name: getattr(args, name).read_bytes() for name in ['before', 'pack', 'layout', 'contract']}
    before, after = load_pack(args.before), load_pack(args.pack)
    layout, contract = json.loads(inputs['layout']), json.loads(inputs['contract'])
    report = audit(before, after, layout, contract)
    negatives = []
    for label in ['room', 'old_program', 'gate_prerequisite', 'neighbor', 'audio']:
        bad = deepcopy(after)
        city = bad['data']['maps'][contract['city']]
        if label == 'room':
            name = next(n for n in bad['data']['maps'] if n.startswith(contract['city']) and n != contract['city'])
            bad['data']['maps'][name]['blocks'][0] ^= 1
        elif label == 'old_program':
            city['scripts']['MnMPNorthCheck'][0]['args'] = ['ENGINE_RISINGBADGE']
        elif label == 'gate_prerequisite':
            city['scripts']['MnMPUpperPoolBuoyCheck'][0]['args'] = ['ENGINE_RISINGBADGE']
        elif label == 'neighbor':
            bad['data']['maps'][contract['connections'][0]['target']]['blocks'][0] ^= 1
        else:
            bad['audio_manifest'] = {}
        try:
            audit(before, bad, layout, contract)
        except (AssertionError, KeyError):
            negatives.append(label)
        else:
            raise AssertionError('accepted corruption: '+label)
    identities = [raw_identity(args.before), raw_identity(args.pack)]
    assert all(v['exact'] for v in identities)
    assert all(getattr(args, name).read_bytes() == raw for name, raw in inputs.items())
    report.update(status='PASS', negatives_rejected=negatives, raw_identities=identities,
                  scope=__doc__, inputs={name:dict(path=str(getattr(args,name)), sha256=hashlib.sha256(raw).hexdigest()) for name,raw in inputs.items()})
    raw = (json.dumps(report, indent=2)+'\n').encode()
    args.output.mkdir(parents=True, exist_ok=False)
    proof = args.output / ('proof-'+hashlib.sha256(raw).hexdigest()+'.json'); proof.write_bytes(raw)
    print(json.dumps(dict(status='PASS', proof=str(proof), maps=report['maps'], negatives=len(negatives))))


if __name__ == '__main__':
    main()
