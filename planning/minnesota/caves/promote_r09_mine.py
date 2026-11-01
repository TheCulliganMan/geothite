#!/usr/bin/env python3
"""Guarded source-only R09 promotion. Preflight is default; never builds a pack."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import time

ROOT = Path(__file__).resolve().parents[3]
ROOM = 'MnR09Cave1F'
CANDIDATE_SHA = '243f04da49a01f59e6fe8fb004049fd942c3acac1e1e6a5ad6c865f89b650d43'
PREVIOUS_SHA = '8553987f7da3cfaa3e8bdb3a680686c9a32f9c869b8615bf01e87782b0f0e088'
CASES = (
    'r09_mine_controller_original_mouth_three_stairs_two_short_hops_scout_and_continue',
    'iron_range_story_controller_actual_win_separate_recovery_decline_repeat_and_continue',
    'iron_range_story_controller_played_loss_never_awards_victory_or_evidence',
    'iron_range_story_controller_refuses_missing_prerequisites_and_locked_cache',
)
ASSET_CASES = (
    'r09_mine_revision_preserves_all_other_data_and_exact_native_story',
    'r09_mine_revision_rejects_changed_and_matching_false_story_sources',
    'r09_mine_revision_rejects_typed_party_flag_and_noncanonical_stale_split_drift',
    'r09_mine_revision_rejects_unsafe_actor_stair_hop_mouth_and_boundary_geometry',
    'r09_mine_revision_true_uncached_and_actual_written_roundtrip_are_complete',
)
RETURN = 'Original mine warp1 at13,19 LADDER returns directly to exterior passive warp2 at40,42 FLOOR; stable after native frames and walking away/back. Exterior active CAVE warp1 at40,41 deliberately reenters. Clear indoor arrival13,18 is distinct from actual warp arrival13,19.'



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
    parser.add_argument('--native-pack', type=Path, default=ROOT / 'content-packs/minnesota-northstar-mine-cleft-review.crystalpack')
    args = parser.parse_args()
    snapshot = args.snapshot.resolve()
    freeze = read(snapshot / 'freeze.json')
    candidate_path = ROOT / 'planning/minnesota/caves/MnR09Cave1F.candidate.json'
    previous_path = ROOT / 'planning/minnesota/caves/MnR09Cave1F.previous.json'
    require(digest(candidate_path) == CANDIDATE_SHA, 'frozen candidate bytes changed')
    require(digest(previous_path) == PREVIOUS_SHA, 'original whole R09 room bytes changed')
    require(digest(args.native_pack) == freeze['native_candidate_pack_sha256'], 'exact installed native pack changed')
    candidate = read(candidate_path)
    old = read(snapshot / 'room.before.json')
    contract = read(snapshot / 'contract.before.json')
    require(old == read(previous_path), 'frozen shared preimage must equal original whole room')
    ignored = {'rows', 'purpose', 'candidate_status', 'review_contract'}
    require(all(old.get(k) == candidate.get(k) for k in set(old) | set(candidate) if k not in ignored),
            'candidate changes original actors/program/reward/warps/attributes')
    require(candidate['dimensions_metatiles'] == [12, 11], 'same12x11 extent required')
    require(candidate['exits'] == [[13, 19]] and candidate['clear_arrival'] == [13, 18], 'mouth changed')
    require(contract['exterior_door_core'] == [40, 41]
            and contract['safe_exterior_return_core'] == [40, 42], 'original passive warp2 return changed')
    proof_path = ROOT / freeze['preservation_report']
    require(digest(proof_path) == freeze['preservation_report_sha256'], 'frozen native preservation report changed')
    proof = read(proof_path)
    require(proof['status'] == 'PASS' and len(proof['checks']) == 199
            and all(c['passed'] for c in proof['checks'])
            and proof['pack']['file_sha256'] == freeze['native_candidate_pack_sha256']
            and proof['pack']['maps'] == 510
            and proof['source_sha256'] == CANDIDATE_SHA
            and proof['previous_sha256'] == PREVIOUS_SHA, 'exact CURRENT510 preservation proof required')
    require(any(c['passed'] and c['check'] == 'original12x11 forcedNITE attributes exact' for c in proof['checks']),
            'native NITE attribute preservation proof required')
    require(proof['personally_visually_reviewed'] and len(proof['captures']) == 9
            and proof['full_lcd_exact_pixels_compared'] == 207360, 'exact nine reviewed native captures required')
    viewed_path = ROOT / freeze['root_viewed_receipt']
    require(digest(viewed_path) == freeze['root_viewed_receipt_sha256'], 'frozen Root viewed receipt changed')
    viewed = read(viewed_path)
    require(viewed['native_pack_sha256'] == freeze['native_candidate_pack_sha256']
            and viewed['source_sha256'] == CANDIDATE_SHA
            and viewed['native_capture_report_sha256'] == freeze['preservation_report_sha256'],
            'Root viewed receipt must match exact native/source proof')
    for name, sha in freeze['execution_evidence_sha256'].items():
        require(digest(ROOT / name) == sha, 'frozen execution evidence changed: ' + name)
    assets = (ROOT / freeze['asset_log']).read_text()
    require('test result: ok. 5 passed' in assets and all(re.search(r'\b' + re.escape(case) + r' \.\.\. ok', assets) for case in ASSET_CASES),
            'all five native assets cases required')
    audio = (ROOT / freeze['audio_log']).read_text()
    require('377 audio programs; all PCM hashes, frame counts and loop ranges match' in audio,
            'exact native377 audio proof required')
    receipt = read(ROOT / freeze['controller_receipt'])
    require(Path(receipt['pack']).resolve() == args.native_pack.resolve()
            and len(receipt['results']) == 2 and all(r['exit_code'] == 0 for r in receipt['results'])
            and receipt['private_binary_removed'], 'actual geometry/story controller receipt required')
    observed = set()
    for name in freeze['controller_logs']:
        log = (ROOT / name).read_text()
        require('test result: ok.' in log and 'test result: FAILED' not in log,
                'frozen controller run must pass')
        observed.update(case for case in CASES if re.search(r'::' + re.escape(case) + r' \.\.\. ok', log))
    require(observed == set(CASES), 'all four native controller cases required before proposal')
    promoted = copy.deepcopy(candidate)
    promoted['candidate_status'] = 'Installed CURRENT510 native review candidate; four local production-controller cases PASS. User visual acceptance and earned regional Q06 progress remain unclaimed.'
    promoted['review_contract']['unpublished'] = False
    promoted['review_contract']['status'] = 'Installed native CURRENT510 mine candidate; all original Q06 programs/party/flags preserved and four local production-controller cases PASS.'
    promoted['review_contract'].setdefault('controller_probes', {})['exterior_return_semantics'] = RETURN
    promoted['review_contract']['native_review'] = {
        'pack_sha256': freeze['native_candidate_pack_sha256'], 'palette': 'nite',
        'candidate_sha256': CANDIDATE_SHA, 'preservation_checks': 199,
        'controller_cases': list(CASES), 'source_promoted_after_explicit_root_pass': True,
    }
    revised_contract = copy.deepcopy(contract)
    revised_contract.update({
        'dimensions_metatiles': [12, 11], 'native_palette': 'nite',
        'native_warp_arrival_core': [13, 19], 'clear_arrival_core': [13, 18],
        'exterior_clear_return_core': [40, 42], 'source_collision_checked': True,
        'requires_real_collision_check': False,
        'mouths': [{'exterior_door_core': [40, 41], 'interior_target_warp': 1,
                    'exterior_entry_warp': 1, 'exterior_return_warp': 2,
                    'safe_exterior_return_core': [40, 42]}],
        'candidate_source_sha256': CANDIDATE_SHA,
        'native_review_pack_sha256': freeze['native_candidate_pack_sha256'],
        'runtime_settled_exterior_core': [40, 42], 'return_semantics': RETURN,
        'validation': 'Actual CURRENT510199-check preservation/nine captures, five assets and four production-controller cases PASS; whole original scout, foreman battle/party/indexes, separate saved-event cache and all warp/art/attribute records retained; exact historical19-root/NPC/event mirrors synchronized.',
        'native_status': 'Installed12x11 R09 native mine review candidate on b3b0c678 CURRENT510; original forced NITE. Three useful stairs both ways, two short drops with9/11 ordinary returns, scout pages, foreman prerequisites/real battle/win-loss, separate recovery yes-no/acknowledgement/repeat and Continue PASS. Regional earned progression and final visual approval remain separate.',
    })
    paths = [ROOT / 'planning/minnesota/routes/MN_R09.json', ROOT / 'planning/minnesota/interiors.json']
    if args.apply:
        for path in paths:
            require(digest(path) == freeze['source_files'][str(path.relative_to(ROOT))],
                    'complete shared source changed since reviewed preflight: ' + str(path))
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
        require(all(str(path) in freeze['controller_logs'] or str(path.resolve().relative_to(ROOT)) in freeze['controller_logs'] for path in args.controller_log), 'controller logs must be frozen evidence')
        require(passed == set(CASES), 'all four actual R09 geometry and Q06 controller cases must pass')
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
                temporary = path.with_name('.' + path.name + '.r09-promotion.tmp')
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
                    rollback = path.with_name('.' + path.name + '.r09-rollback.tmp')
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
                      'dimensions_metatiles': [12, 11], 'return_semantics': RETURN}))


if __name__ == '__main__':
    main()
