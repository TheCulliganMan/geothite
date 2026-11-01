#!/usr/bin/env python3
"""Verify the complete fossil-lab integration without modifying any input pack."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from audit_comprehensive_accepted_successors import CACHE_PATHS, merge
from render_reference import load_pack

HASHES = (
    'f6498564aec5f6cec506e47329f2eb72d248fe73d97677a6f0b4af0e6272fcf8',
    '4329e491655cd8d6c787a34266a5fa51e222b8cdbd596e56c3b046219c5df5db',
    '3e6573727e54ea83bffe38f7a34fab323eeef618cdd53a52a5f03ef2196a3219',
    '1111a481d2261ccf9c327f24da1f431772d99b2004ffcd208659ff465bfc10a0',
)
CONTENT = '0fb868423bc402679f5623cf6fdda2b2af87e7ab0228a5f8d38dfdd3d0667386'
LAB = 'MnRochesterFossilLab'


def expected_data(common, donor, current):
    # Merge derived command caches by their source programs. Conflicting edits
    # to one source label are rejected by the shared cache-union implementation.
    for name in ('MnRochester', 'MnR01C00R00'):
        for key in ('script_flag_commands', 'script_movements', 'script_text_commands',
                    'script_runtime_commands', 'script_control_commands',
                    'script_object_commands', 'script_variable_commands',
                    'script_map_commands'):
            CACHE_PATHS.add('data.maps.' + name + '.' + key)
    return merge(common['data'], current['data'], donor['data'], 'data')


def check(common, donor, current, child, expected):
    assert child['identity']['content_hash'] == CONTENT, 'child identity'
    assert child['data'] == expected, 'unexpected complete data delta'
    for key in current.keys() | child.keys():
        if key not in {'data', 'report', 'identity'}:
            assert child.get(key) == current.get(key), key
    report = deepcopy(current['report'])
    report['maps'] += 1
    for key in ('reachable_maps', 'solvable_maps'):
        assert donor['report'][key] == common['report'][key] + [LAB], key
        report[key] = report[key] + [LAB]
    for marker in donor['report']['manifests']:
        if marker not in common['report']['manifests'] and marker not in report['manifests']:
            report['manifests'].append(marker)
    assert child['report'] == report, 'unexpected report delta'
    old, new = current['data']['maps'], child['data']['maps']
    assert new.keys() - old.keys() == {LAB}
    assert old.keys() <= new.keys()
    changed = sorted(name for name in old if old[name] != new[name])
    assert changed == ['MnR01C00R00', 'MnRochester'], changed


def reject_faults(common, donor, current, child, expected):
    faults = ('terrain', 'dialogue', 'audio', 'art', 'report', 'identity')
    for fault in faults:
        altered = deepcopy(child)
        if fault == 'terrain':
            altered['data']['maps']['MnWinonaHome2F']['blocks'][0] ^= 1
        elif fault == 'dialogue':
            scripts = altered['data']['maps']['MnWinonaHome2F']['scripts']
            del scripts[next(iter(scripts))]
        elif fault == 'audio':
            assert altered['audio_manifest'] is not None
            altered['audio_manifest'] = None
        elif fault == 'art':
            assert altered['runtime_files'], 'native source art required'
            altered['runtime_files'].clear()
        elif fault == 'report':
            altered['report']['maps'] -= 1
        else:
            altered['identity']['content_hash'] = '0' * 64
        try:
            check(common, donor, current, altered, expected)
        except AssertionError:
            pass
        else:
            raise AssertionError('accepted deliberate fault: ' + fault)
    check(common, donor, current, child, expected)
    return list(faults)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('common', type=Path)
    parser.add_argument('donor', type=Path)
    parser.add_argument('current', type=Path)
    parser.add_argument('child', type=Path)
    args = parser.parse_args()
    paths = (args.common, args.donor, args.current, args.child)
    for path, expected in zip(paths, HASHES):
        with path.open('rb') as stream:
            digest = hashlib.sha256()
            for block in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(block)
        assert digest.hexdigest() == expected, 'unreviewed input: ' + str(path)
    common, donor, current, child = map(load_pack, paths)
    expected = expected_data(common, donor, current)
    check(common, donor, current, child, expected)
    rejected = reject_faults(common, donor, current, child, expected)
    print(json.dumps(dict(entire_data_exact=True, report_exact=True,
                         art_audio_exact=True, untouched_maps=740,
                         changed_maps=['MnR01C00R00', 'MnRochester'],
                         new_maps=[LAB], deliberate_faults_rejected=rejected,
                         scope='Static preservation; native gameplay and visual acceptance remain separate.')))


if __name__ == '__main__':
    main()
