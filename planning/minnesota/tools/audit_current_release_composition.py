#!/usr/bin/env python3
"""Read-only provenance and accepted-terrain audit for the release composition.

This does not synthesize or write a pack, or establish gameplay acceptance.
The resulting audio must also pass the Rust PCM/frame/loop verifier.
"""
import argparse
import json
from pathlib import Path

from render_reference import load_pack

ABSENT = object()


def check(current, before, released, result):
    assert result['format_version'] == current['format_version']
    current_maps = current['data']['maps']
    outside = {name for name, module in current_maps.items()
               if module['attributes']['environment'] in ['ROUTE', 'TOWN']}
    protected = set()
    for name in outside:
        module = current_maps[name]
        for field in ['attributes', 'blocks']:
            protected.add(('data', 'maps', name, field))
        protected.add(('data', 'map_attributes', name))
        protected.add(('data', 'map_blocks', module['attributes']['blocks_label']))
        for table in ['runtime_map_metadata', 'map_dimensions']:
            protected.add(('data', table, module['attributes']['map_constant']))
        actual = result['data']['maps'][name]
        assert actual['attributes'] == module['attributes'], name + ': exterior attributes'
        assert actual['blocks'] == module['blocks'], name + ': exterior terrain'
    assert all(name in current_maps for name, module in result['data']['maps'].items()
               if module['attributes']['environment'] in ['ROUTE', 'TOWN'])
    counts = dict(untouched_subtrees=0, released_delta_subtrees=0,
                  protected_spatial_subtrees=0)

    def visit(old, base, donor, actual, path):
        if path in protected:
            assert actual == old, 'accepted spatial data changed: ' + '.'.join(path)
            counts['protected_spatial_subtrees'] += 1
            return
        if base == donor:
            assert actual == old, 'unrelated current content changed: ' + '.'.join(path)
            counts['untouched_subtrees'] += 1
            return
        protected_descendant = any(key[:len(path)] == path and len(key) > len(path)
                                   for key in protected)
        if (old == base or old == donor) and not protected_descendant:
            assert actual == donor, 'released content missing: ' + '.'.join(path)
            counts['released_delta_subtrees'] += 1
            return
        assert all(isinstance(value, dict) for value in [old, base, donor, actual]), (
            'unresolved conflicting content: ' + '.'.join(path))
        for key in set(old) | set(base) | set(donor) | set(actual):
            visit(old.get(key, ABSENT), base.get(key, ABSENT),
                  donor.get(key, ABSENT), actual.get(key, ABSENT), path + (str(key),))

    # These three domains have stable serialized values independent of storage
    # compression. Inspect the complete data and art, not just named scene maps.
    for table in ['data', 'runtime_files']:
        visit(current[table], before[table], released[table], result[table], (table,))
    old_report = dict(current['report'])
    base_report = dict(before['report'])
    donor_report = dict(released['report'])
    actual_report = dict(result['report'])
    for report in [old_report, base_report, donor_report, actual_report]:
        del report['manifests']
    visit(old_report, base_report, donor_report, actual_report, ('report',))
    expected = list(current['report']['manifests'])
    for marker in released['report']['manifests']:
        if marker not in expected:
            expected.append(marker)
    expected.append('minnesota-current-release-composition-v1')
    assert result['report']['manifests'] == expected, 'publication markers'
    assert current['audio_manifest'] == before['audio_manifest'], 'unreviewed current audio delta'
    assert result['audio_manifest'] == released['audio_manifest'], 'released audio metadata'
    original_audio = 0
    total_audio = 0
    for kind, entries in current['audio_manifest'].items():
        for name, entry in entries.items():
            assert result['audio_manifest'][kind][name] == entry, 'original audio metadata: ' + name
            original_audio += 1
    for entries in result['audio_manifest'].values():
        total_audio += len(entries)
    assert result['compiled_audio'].keys() == released['compiled_audio'].keys()
    assert len(result['identity']['runtime_modpack_id'].encode()) <= 16384, 'hosted identity limit'
    return dict(content_hash=result['identity']['content_hash'],
                accepted_outdoor_maps_exact=len(outside),
                original_audio_entries_exact=original_audio,
                resulting_audio_entries=total_audio,
                species=len(result['data']['pokemon']), maps=len(result['data']['maps']),
                **counts,
                caution='Provenance audit only; Rust PCM verification and earned gameplay remain required.')


def reject_mutations(current, before, released, result):
    rejected = []
    for name in ['accepted-terrain', 'current-native-art', 'released-species', 'original-audio']:
        changed = dict(result)
        if name in ['accepted-terrain', 'released-species']:
            changed['data'] = dict(result['data'])
        if name == 'accepted-terrain':
            changed['data']['maps'] = dict(result['data']['maps'])
            module = dict(result['data']['maps']['MnDuluth'])
            module['blocks'] = list(module['blocks'])
            module['blocks'][0] ^= 1
            changed['data']['maps']['MnDuluth'] = module
        elif name == 'released-species':
            changed['data']['pokemon'] = dict(result['data']['pokemon'])
            del changed['data']['pokemon']['VERDIGRIM']
        elif name == 'current-native-art':
            path = 'data/tilesets/mn_duluth_expanded_metatiles.bin'
            changed['runtime_files'] = dict(result['runtime_files'])
            value = result['runtime_files'][path]
            altered = list(value)
            altered[0] ^= 1
            changed['runtime_files'][path] = bytes(altered) if isinstance(value, bytes) else altered
        else:
            kind = next(kind for kind, rows in current['audio_manifest'].items() if rows)
            key = next(iter(current['audio_manifest'][kind]))
            changed['audio_manifest'] = dict(result['audio_manifest'])
            changed['audio_manifest'][kind] = dict(result['audio_manifest'][kind])
            entry = dict(result['audio_manifest'][kind][key])
            entry['byte_len'] += 1
            changed['audio_manifest'][kind][key] = entry
        try:
            check(current, before, released, changed)
        except AssertionError:
            rejected.append(name)
        else:
            raise AssertionError('audit accepted corruption: ' + name)
    return rejected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['current', 'before', 'released', 'result']:
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    packs = [load_pack(getattr(args, name)) for name in ['current', 'before', 'released', 'result']]
    report = check(*packs)
    report["rejected_negative_cases"] = reject_mutations(*packs)
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
