#!/usr/bin/env python3
"""Guarded source-only R12 promotion. Preflight is default; never builds a pack."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import time

ROOT = Path(__file__).resolve().parents[3]
ROOM = 'MnR12Cave1F'
CANDIDATE_SHA = '7fe83d8e9bd69f01f57c77dd1ea1a7c759de3e6d4ceefdb33d9f42fed8dd9048'
PREVIOUS_SHA = '39870d5222d8eda588f843d00e3710b4952a1a7d2c56e7e88ce3f20fd3efab73'
CASES = (
    'r12_shelter_controller_original_mouth_four_stairs_two_hop_lanes_and_ordinary_returns',
    'r12_shelter_controller_original_otta_max_ether_restores_legal_pp_once_and_continues',
    'r12_shelter_controller_native_ether_elixers_and_full_pp_refusals_preserve_resources',
)
RETURN = 'Original active exterior CAVE warp1 at global66,23/local2,23; native CheckTile may advance Down during presentation frames. If still on the active arrival, explicit Down reaches safe Floor66,24/local2,24 without reentry. Deliberate north input reenters. No passive exterior warp is added.'


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
    parser.add_argument('--native-pack', type=Path, default=ROOT / 'content-packs/minnesota-northstar-map-refinements.crystalpack')
    args = parser.parse_args()
    snapshot = args.snapshot.resolve()
    freeze = read(snapshot / 'freeze.json')
    candidate_path = ROOT / 'planning/minnesota/caves/MnR12Cave1F.candidate.json'
    previous_path = ROOT / 'planning/minnesota/caves/MnR12Cave1F.previous.json'
    require(digest(candidate_path) == CANDIDATE_SHA, 'frozen candidate bytes changed')
    require(digest(previous_path) == PREVIOUS_SHA, 'original3987 room bytes changed')
    require(digest(args.native_pack) == freeze['native_candidate_pack_sha256'], 'exact installed native pack changed')
    candidate = read(candidate_path)
    old = read(snapshot / 'room.before.json')
    contract = read(snapshot / 'contract.before.json')
    require(old == read(previous_path), 'frozen shared preimage must equal original whole room')
    ignored = {'rows', 'purpose', 'candidate_status', 'review_contract'}
    require(all(old.get(k) == candidate.get(k) for k in set(old) | set(candidate) if k not in ignored),
            'candidate changes original actors/program/reward/warps/attributes')
    require(candidate['dimensions_metatiles'] == [16, 12], 'same16x12 extent required')
    require(candidate['exits'] == [[17, 23]] and candidate['clear_arrival'] == [17, 22], 'mouth changed')
    require(contract['exterior_door_core'] == [66, 23]
            and contract['exterior_clear_return_core'] == [66, 24]
            and contract['mouths'][0]['exterior_return_warp'] == 1, 'active warp1 return changed')
    proof = read(ROOT / freeze['preservation_report'])
    require(proof['status'] == 'PASS' and len(proof['checks']) == 163
            and all(c['passed'] for c in proof['checks'])
            and proof['after']['file_sha256'] == freeze['native_candidate_pack_sha256']
            and proof['after']['maps'] == 510, 'exact CURRENT510 preservation proof required')
    require(any(c['passed'] and c['check'] == 'R12 exact16x12 forced native NITE' for c in proof['checks']),
            'native NITE attribute preservation proof required')
    promoted = copy.deepcopy(candidate)
    promoted['candidate_status'] = 'Installed CURRENT510 native review candidate; all three local production-controller cases PASS. User visual acceptance and earned regional SQ03/Q07 progress remain unclaimed.'
    promoted['review_contract']['unpublished'] = False
    promoted['review_contract']['controller_probes']['exterior_return_semantics'] = RETURN
    promoted['review_contract']['native_review'] = {
        'pack_sha256': freeze['native_candidate_pack_sha256'], 'palette': 'nite',
        'candidate_sha256': CANDIDATE_SHA, 'preservation_checks': 163,
        'controller_cases': list(CASES), 'source_promoted_after_explicit_root_pass': True,
    }
    revised_contract = copy.deepcopy(contract)
    revised_contract.update({
        'dimensions_metatiles': [16, 12], 'native_palette': 'nite',
        'candidate_source_sha256': CANDIDATE_SHA,
        'native_review_pack_sha256': freeze['native_candidate_pack_sha256'],
        'runtime_settled_exterior_core': [66, 24], 'return_semantics': RETURN,
        'validation': 'Actual CURRENT510 native terrain/preservation and three production-controller cases PASS; original whole Otta/text, MAX_ETHER1 event/program and all warp/art/attribute records retained.',
        'native_status': 'Installed16x12 R12 native review candidate on c16af00c CURRENT510; original forced NITE. Native four stairs/two hop lanes/ordinary returns, Otta pages, real Max Ether PP use/once/Continue and PP-item refusal coverage PASS. Regional earned progression and final visual approval remain separate.',
    })
    paths = [ROOT / 'planning/minnesota/routes/MN_R12.json', ROOT / 'planning/minnesota/interiors.json']
    before = [p.read_text() for p in paths]
    after = [replace_record(before[0], 'id', old, promoted),
             replace_record(before[1], 'interior', contract, revised_contract)]
    for name, value in [('room.proposed.json', promoted), ('contract.proposed.json', revised_contract)]:
        (snapshot / name).write_text(json.dumps(value, indent=2) + '\n')
    if args.apply:
        passed = set()
        require(args.controller_log, '--apply requires actual passing final510 controller logs')
        for path in args.controller_log:
            log = path.read_text()
            require('test result: ok.' in log and 'test result: FAILED' not in log, 'controller evidence must be a passing run')
            passed.update(case for case in CASES if re.search(r'::' + re.escape(case) + r' \.\.\. ok', log))
        require(passed == set(CASES), 'all three actual R12/PP controller cases must pass')
        require(all(p.read_text() == raw for p, raw in zip(paths, before)), 'source changed during preflight')
        application = snapshot / ('application-' + str(time.time_ns()))
        application.mkdir()
        for path, raw in zip(paths, before):
            (application / (path.name + '.before')).write_text(raw)
        for path in args.controller_log:
            (application / path.name).write_bytes(path.read_bytes())
        staged, written = [], []
        try:
            for path, raw in zip(paths, after):
                temporary = path.with_name('.' + path.name + '.r12-promotion.tmp')
                require(not temporary.exists(), 'existing source staging file')
                temporary.write_text(raw)
                staged.append(temporary)
            require(all(p.read_text() == raw for p, raw in zip(paths, before)), 'source changed before replacement')
            for temporary, path in zip(staged, paths):
                os.replace(temporary, path)
                written.append(path)
        except BaseException:
            for path, old_raw, new_raw in zip(paths, before, after):
                if path in written:
                    require(path.read_text() == new_raw, 'external edit blocks rollback; immutable before files retained')
                    rollback = path.with_name('.' + path.name + '.r12-rollback.tmp')
                    require(not rollback.exists(), 'existing rollback staging file')
                    rollback.write_text(old_raw)
                    os.replace(rollback, path)
            raise
        finally:
            for temporary in staged:
                temporary.unlink(missing_ok=True)
        (application / 'promotion.json').write_text(json.dumps({
            'status': 'APPLIED AFTER EXPLICIT ROOT PASS', 'candidate_sha256': CANDIDATE_SHA,
            'native_pack_sha256': freeze['native_candidate_pack_sha256'], 'native_palette': 'nite',
            'after_files': {str(p.relative_to(ROOT)): digest(p) for p in paths},
            'controller_logs': {str(p): digest(p) for p in args.controller_log},
        }, indent=2) + '\n')
    print(json.dumps({'applied': args.apply, 'snapshot': str(snapshot),
                      'candidate_sha256': CANDIDATE_SHA, 'native_palette': 'nite',
                      'dimensions_metatiles': [16, 12], 'return_semantics': RETURN}))


if __name__ == '__main__':
    main()
