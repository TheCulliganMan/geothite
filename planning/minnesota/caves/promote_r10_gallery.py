#!/usr/bin/env python3
"""Guarded source-only R10 promotion. Default is preflight; never builds a pack."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import time

ROOT = Path(__file__).resolve().parents[3]
ROOM = 'MnR10Cave1F'
CANDIDATE_SHA = 'a3ea9a75a23c0b8e0a335aa47ceeffd5762efdccee16512168523260f115aef6'
CASES = (
    'r10_gallery_controller_original_mouth_passive_return_four_stairs_and_short_native_drop',
    'r10_gallery_controller_original_reader_pages_revive_once_and_semantic_continue',
    'r10_gallery_controller_full_bag_refusal_native_potion_use_and_original_pickup_retry',
)


def read(path):
    return json.loads(path.read_text())


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def replace_record(raw, key, expected, replacement):
    pattern = r'\{\s*"' + re.escape(key) + r'"\s*:\s*"' + ROOM + r'"'
    matches = list(re.finditer(pattern, raw))
    require(len(matches) == 1, 'exactly one owned record must exist')
    start = matches[0].start()
    actual, length = json.JSONDecoder().raw_decode(raw[start:])
    require(actual == expected, 'owned record changed after source freeze')
    line_start = raw.rfind('\n', 0, start) + 1
    indent = raw[line_start:start]
    require(not indent.strip(), 'record must start on its own indented line')
    proposed = json.dumps(replacement, indent=2).replace('\n', '\n' + indent)
    after = raw[:start] + proposed + raw[start + length:]
    require(json.loads(after), 'proposed complete source must parse')
    # Splicing a single decoded object preserves every byte outside that object.
    require(after[:start] == raw[:start] and after[start + len(proposed):] == raw[start + length:],
            'unrelated source bytes changed')
    return after


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--snapshot', type=Path, required=True)
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--controller-log', type=Path, action='append', default=[])
    parser.add_argument('--native-pack', type=Path, default=ROOT / '.local/minnesota-city-revisions/gallery-e0cf8cf00b18c1b25c7cab70e37ebbcc93f1ff25d67fb7aa7f4c4371d567085e.crystalpack')
    args = parser.parse_args()
    snapshot = args.snapshot.resolve()
    freeze = read(snapshot / 'freeze.json')
    candidate_path = ROOT / 'planning/minnesota/caves/MnR10Cave1F.candidate.json'
    require(digest(candidate_path) == CANDIDATE_SHA, 'reviewed candidate bytes changed')
    require(digest(args.native_pack) == freeze['native_candidate_pack_sha256'], 'native review pack changed')
    candidate = read(candidate_path)
    old = read(snapshot / 'room.before.json')
    contract = read(snapshot / 'contract.before.json')
    require(candidate['dimensions_metatiles'] == [18, 14], 'exact reviewed extent required')
    require(candidate['residents'] == old['residents'], 'original whole Nerd/text changed')
    reward = copy.deepcopy(old['items'])
    reward[0]['tile'] = [29, 23]
    require(candidate['items'] == reward, 'Revive identity/quantity/flag changed')
    require(candidate['exits'] == [[3, 13]] and candidate['clear_arrival'] == [3, 12], 'mouth changed')
    require(contract['exterior_door_core'] == [36, 17]
            and contract['safe_exterior_return_core'] == [36, 18]
            and contract['native_warp_arrival_core'] == [3, 13], 'paired return changed')
    native = read(ROOT / freeze['native_comparison'])
    capture = read(ROOT / freeze['native_capture_summary'])
    require(native['pack_sha256'] == freeze['native_candidate_pack_sha256']
            and native['source_sha256'] == CANDIDATE_SHA
            and native['native_source_blocks_equal'] == 252
            and native['unrelated_maps_equal'] == 504
            and native['original_reader_record_and_program_equal']
            and native['original_pickup_program_and_flag_equal']
            and native['original_warp_events_equal'], 'native preservation evidence mismatch')
    require(capture['native_declared_palette'] == 'nite'
            and capture['native_palette_nite_preserved_from_base']
            and capture['whole_native_nite_terrain_source_pixels_equal'], 'exact native NITE evidence required')
    preservation_path = ROOT / 'planning/minnesota/renders/northwoods-native/proof/r10-gallery/comparison-ae43e7811f5c2cd5c2e88c1441ecf2d95053f144396119899111a3d1e505f3b8.json'
    preservation = read(preservation_path)
    require(preservation['status'] == 'PASS'
            and preservation['after']['file_sha256'] == freeze['native_candidate_pack_sha256']
            and len(preservation['checks']) == 125
            and all(c['passed'] for c in preservation['checks']), '125 written-pack checks required')
    promoted = copy.deepcopy(candidate)
    promoted['candidate_status'] = 'Native review candidate installed; all three local production-controller cases pass. User visual approval and earned regional progress are not claimed.'
    promoted['review_contract']['unpublished'] = False
    revised_contract = copy.deepcopy(contract)
    revised_contract.update({
        'dimensions_metatiles': [18, 14],
        'native_palette': 'nite',
        'candidate_source_sha256': CANDIDATE_SHA,
        'native_status': 'Native505-map gallery review candidate:18x14; inherited NITE palette; exact original Nerd/text and Revive program/flag/quantity, moved only to29,23. Native mouth3,13, clear3,12 and exterior passive36,18 retained. Three actual local production-controller cases PASS, including native item presentation, full-bag retry and Continue;125 written-pack preservation checks and six headless production captures retained. Earned Q06/regional hub progress remains unverified.',
    })
    paths = [ROOT / 'planning/minnesota/routes/MN_R10.json', ROOT / 'planning/minnesota/interiors.json']
    before = [path.read_text() for path in paths]
    after = [replace_record(before[0], 'id', old, promoted),
             replace_record(before[1], 'interior', contract, revised_contract)]
    for name, record in [('room.proposed.json', promoted), ('contract.proposed.json', revised_contract)]:
        (snapshot / name).write_text(json.dumps(record, indent=2) + '\n')
    if args.apply:
        passed = set()
        require(args.controller_log, '--apply requires actual passing controller logs')
        for path in args.controller_log:
            log = path.read_text()
            require('test result: ok.' in log and 'test result: FAILED' not in log, 'controller evidence must be a passing run')
            passed.update(case for case in CASES if re.search(r'::' + re.escape(case) + r' \.\.\. ok', log))
        require(passed == set(CASES), 'all three actual R10 controller cases must pass')
        # Parent explicitly runs --apply only after approving the test result.
        # Recheck both complete files before writing; preserve immutable recovery.
        require(all(p.read_text() == value for p, value in zip(paths, before)), 'source changed during preflight')
        application = snapshot / ('application-' + str(time.time_ns()))
        application.mkdir()
        for path, raw in zip(paths, before):
            (application / (path.name + '.before')).write_text(raw)
        for path in args.controller_log:
            (application / path.name).write_bytes(path.read_bytes())
        staged = []
        written = []
        try:
            for path, raw in zip(paths, after):
                temporary = path.with_name('.' + path.name + '.r10-promotion.tmp')
                require(not temporary.exists(), 'existing promotion staging file')
                temporary.write_text(raw)
                staged.append(temporary)
            require(all(p.read_text() == value for p, value in zip(paths, before)), 'source changed before replacement')
            for temporary, path in zip(staged, paths):
                os.replace(temporary, path)
                written.append(path)
        except BaseException:
            # Restore only our own successful replacement, never external edits.
            for path, old_raw, new_raw in zip(paths, before, after):
                if path in written:
                    require(path.read_text() == new_raw, 'external source edit prevents automatic rollback; frozen before files retained')
                    rollback = path.with_name('.' + path.name + '.r10-rollback.tmp')
                    require(not rollback.exists(), 'existing rollback staging file')
                    rollback.write_text(old_raw)
                    os.replace(rollback, path)
            raise
        finally:
            for temporary in staged:
                temporary.unlink(missing_ok=True)
        (application / 'promotion.json').write_text(json.dumps({
            'status': 'SOURCE PROMOTION APPLIED AFTER THREE NATIVE CONTROLLER PASSES',
            'candidate_sha256': CANDIDATE_SHA,
            'native_pack_sha256': freeze['native_candidate_pack_sha256'],
            'native_palette': 'nite',
            'preservation_report': str(preservation_path.relative_to(ROOT)),
            'checks': 125,
            'after_files': {str(p.relative_to(ROOT)): digest(p) for p in paths},
            'controller_logs': {str(p): digest(p) for p in args.controller_log},
        }, indent=2) + '\n')
    print(json.dumps({'applied': args.apply, 'source_candidate_sha256': CANDIDATE_SHA,
                      'native_palette': 'nite', 'dimensions_metatiles': [18, 14],
                      'snapshot': str(snapshot), 'controller_logs': [str(p) for p in args.controller_log]}))


if __name__ == '__main__':
    main()
