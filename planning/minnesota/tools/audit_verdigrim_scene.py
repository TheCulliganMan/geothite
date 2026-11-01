#!/usr/bin/env python3
"""Check scene preservation only; controller and rendered review remain required."""
import argparse
import json

from render_reference import load_pack


def audit(before, after):
    room_id = 'MnBoundaryIntake'
    tileset = 'mn_verdigrim_intake'
    for key in before.keys() | after.keys():
        if key not in {'data', 'runtime_files', 'report', 'identity'}:
            assert before.get(key) == after.get(key), key
    for key, value in before['runtime_files'].items():
        assert after['runtime_files'].get(key) == value, key
    additions = after['runtime_files'].keys() - before['runtime_files'].keys()
    assert additions == {
        f'gfx/tilesets/{tileset}.2bpp', f'gfx/tilesets/{tileset}.png',
        f'data/tilesets/{tileset}.json',
        *(f'data/tilesets/{tileset}_{suffix}' for suffix in [
            'foregrounds.json', 'metatiles.bin', 'palette_map.json', 'animations.json'])}
    assert before['report'].keys() == after['report'].keys()
    for key, value in before['report'].items():
        if key != 'manifests':
            assert after['report'][key] == value, key
    assert sorted(after['report']['manifests']) == sorted(
        before['report']['manifests'] + [
            'mn_verdigrim_intake-native-v1', 'verdigrim-intake-scene-v1'])
    old, new = before['data'], after['data']
    changed = {'maps', 'map_attributes', 'map_blocks', 'npcs', 'map_scripts',
               'tilesets', 'story_event_script_constants'}
    for key in old.keys() | new.keys():
        if key not in changed:
            assert old.get(key) == new.get(key), key
    assert old['maps'].keys() == new['maps'].keys()
    for name, value in old['maps'].items():
        if name != room_id:
            assert new['maps'][name] == value, name
    room = new['maps'][room_id]
    original_room = old['maps'][room_id]
    expected_attributes = dict(original_room['attributes'], tileset_name=tileset)
    assert room['attributes'] == expected_attributes
    expected_blocks = list(original_room['blocks'])
    for index, block in [(4 * 16 + 11, 66), (9 * 16 + 10, 0x16),
                         (5 * 16 + 6, 0x16)]:
        expected_blocks[index] = block
    assert room['blocks'] == expected_blocks
    assert room['events']['warps'] == original_room['events']['warps']
    assert room['events']['coord_events'] == original_room['events']['coord_events']
    assert room['objects'][:len(original_room['objects'])] == original_room['objects']
    assert {o['object_identifier'] for o in room['objects'][len(original_room['objects']):]} == {
        'MN_VERDIGRIM_INTAKE', 'MN_VERDIGRIM_FOREMAN'}
    replaced = {'MnArcIntakeEntry', 'MnArcIntakeActors', 'MnArcIntakeStop',
                'MnArcIntakeFinnText', 'MnArcIntakeIvyText'}
    for label, program in old['maps'][room_id]['scripts'].items():
        if label not in replaced:
            assert room['scripts'].get(label) == program, label
    for label, body in old['maps'][room_id]['script_text_bodies'].items():
        if label not in replaced:
            assert room['script_text_bodies'].get(label) == body, label
    assert all(label.startswith(('MnVerdigrim', 'MN_VERDIGRIM_'))
               for label in room['scripts'].keys() - old['maps'][room_id]['scripts'].keys())
    for table, permitted in [('map_attributes', room_id), ('npcs', room_id),
                             ('map_blocks', room['attributes']['blocks_label'])]:
        assert old[table].keys() == new[table].keys(), table
        for name, value in old[table].items():
            if name != permitted:
                assert new[table][name] == value, (table, name)
    assert new['tilesets'].keys() - old['tilesets'].keys() == {tileset}
    assert all(new['tilesets'].get(k) == v for k, v in old['tilesets'].items())
    source = 'mn_arc_intake'
    files = after['runtime_files']
    native_blocks = bytes(files[f'data/tilesets/{source}_metatiles.bin'])
    private_blocks = bytes(files[f'data/tilesets/{tileset}_metatiles.bin'])
    assert private_blocks[:len(native_blocks)] == native_blocks
    native_graphics = bytes(files[f'gfx/tilesets/{source}.2bpp'])
    private_graphics = bytes(files[f'gfx/tilesets/{tileset}.2bpp'])
    for glyph in set(native_blocks):
        physical = (glyph & 127) + (96 if glyph & 128 else 0)
        span = slice(physical * 16, (physical + 1) * 16)
        assert private_graphics[span] == native_graphics[span], glyph
    assert all(new['tilesets'][tileset]['collision'].get(k) == v
               for k, v in old['tilesets'][source]['collision'].items())
    # Import repacks the VRAM banks and clears unused palette slots. Only glyphs
    # referenced by original metatiles contribute to their displayed pixels.
    floor_glyphs = set(native_blocks[7 * 16:8 * 16])
    for glyph in set(native_blocks):
        expected = old['tilesets'][source]['palette_map'][glyph]
        if glyph in floor_glyphs:
            expected &= 8  # Native gray floor; preserve the original VRAM bank.
        assert (new['tilesets'][tileset]['palette_map'][glyph]
                == expected), glyph
    clips = json.loads(bytes(files[f'data/tilesets/{tileset}_animations.json']))
    native_water = bytes(files['gfx/tilesets/water/water.2bpp'])
    expected_frames = [list(native_water[i:i + 16]) for i in range(0, 64, 16)]
    assert len(clips) == 2
    for clip in clips.values():
        assert clip == dict(frame_ticks=22, frames=expected_frames)
    programs = dict(room['scripts'])
    for attribute, section in [('map_scripts_label', 'map_script_section_commands'),
                               ('map_events_label', 'map_event_section_commands')]:
        programs[room['attributes'][attribute]] = [
            dict(command=row['command'], args=row['args']) for row in room[section]]
    permitted = set(old['maps'][room_id]['scripts']) | set(programs)
    for label in old['map_scripts'].keys() | new['map_scripts'].keys():
        if label not in permitted:
            assert old['map_scripts'].get(label) == new['map_scripts'].get(label), label
        elif label in programs:
            assert new['map_scripts'][label] == programs[label], label
    a, b = old['story_event_script_constants'], new['story_event_script_constants']
    assert a.keys() == b.keys()
    for key in a:
        if key != 'global':
            assert a[key] == b[key], key
    assert all(b['global'].get(k) == v for k, v in a['global'].items())
    flags = {'EVENT_MN_VERDIGRIM_' + suffix for suffix in [
        'EXPOSED', 'GREEN_WATER', 'COUNTER_CURRENT', 'FOREMAN_FLED', 'DRILL_STOPPED',
        'OUTLET_CLOSED', 'CLEAN_BYPASS', 'PINNED', 'RETREATED', 'CONTAINED', 'CAUGHT']}
    assert b['global'].keys() - a['global'].keys() == flags
    values = [b['global'][flag] for flag in flags]
    old_event_values = {v for k, v in a['global'].items() if k.startswith('EVENT_')}
    assert len(set(values)) == len(flags) and not old_event_values.intersection(values)
    return dict(other_maps_exact=len(old['maps']) - 1, original_runtime_files_exact=True,
                species_and_audio_exact=True, private_runtime_files=len(additions),
                original_intake_glyph_pixels_exact=True, native_gray_floor=True,
                native_water_animation_clips=2,
                distinct_new_scene_flags=len(flags), gameplay_acceptance=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    args = parser.parse_args()
    print(json.dumps(audit(load_pack(args.parent), load_pack(args.child))))
