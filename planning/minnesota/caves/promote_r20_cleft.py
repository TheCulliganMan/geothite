#!/usr/bin/env python3
"""Guarded R20 source promotion; default preflight, never builds a native pack."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import time

ROOT = Path(__file__).resolve().parents[3]
ROOM = 'MnR20Cave1F'
CANDIDATE_SHA = '05617d4863db095ed7664ce6408f1d7f35e4ce3c6ed360d08bdfd36a35db2577'
PREVIOUS_SHA = 'fdf68fbac6dd8da99437cf9541607c59a7bbb3736246cc8694d8775764bd0707'
PACK_SHA = '1630ad063aa4a1cf252087bdf743ca48a2a8cb6c9c46ad95cb7aceb1c74c3b05'
PROOF_SHA = 'e133a133a919828ba4584f6e743a6931ba492d526991dcdcdab4d20b7b9f7a37'
CASES = (
    'r20_cleft_controller_both_mouths_passive_returns_and_two_stairs_bidirectional',
    'r20_cleft_controller_both_left_hop_lanes_reverse_denial_and_ordinary_returns',
    'r20_cleft_controller_original_hyper_potion_prints_once_heals_and_continues',
)
RETURN = ('Original west room warp1 returns to MnR20C00R00 passive warp2 global60,34 FLOOR; '
          'east room warp2 returns to MnR20C01R00 passive warp2 global76,32/local12,32 FLOOR. '
          'Active west60,33 targets roomwarp1; active east76,31 targets roomwarp2. '
          'Preserve the existing inactive east Floor record\'s own targetwarp1; no warp records are normalized.')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_text())


def splice_record(raw, key, expected, replacement):
    pattern = r'\{\s*"' + re.escape(key) + r'"\s*:\s*"' + ROOM + r'"'
    found = list(re.finditer(pattern, raw))
    require(len(found) == 1, 'exactly one owned room record required')
    start = found[0].start()
    actual, length = json.JSONDecoder().raw_decode(raw[start:])
    require(actual == expected, 'owned record no longer matches frozen preimage')
    prefix = raw[:start]
    suffix = raw[start + length:]
    indent = raw[raw.rfind('\n', 0, start) + 1:start]
    require(not indent.strip(), 'owned record must start on its own line')
    new_record = json.dumps(replacement, indent=2, ensure_ascii=False).replace('\n', '\n' + indent)
    proposed = prefix + new_record + suffix
    json.loads(proposed)
    require(proposed[:start] == prefix and proposed[start + len(new_record):] == suffix,
            'bytes outside owned record must remain exact')
    return proposed, {'prefix_sha256': hashlib.sha256(prefix.encode()).hexdigest(),
                      'suffix_sha256': hashlib.sha256(suffix.encode()).hexdigest(),
                      'outside_owned_record_bytes_exact': True}


def controller_cases(logs):
    passed = set()
    for path in logs:
        text = path.read_text()
        require('test result: ok.' in text and 'test result: FAILED' not in text,
                'supplied controller log must be an actual passing run')
        passed.update(case for case in CASES if re.search(r'::' + re.escape(case) + r' \.\.\. ok', text))
    return passed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--snapshot', type=Path, required=True)
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--controller-log', type=Path, action='append', default=[])
    parser.add_argument('--native-pack', type=Path,
                        default=ROOT/'content-packs/minnesota-northstar-glacial-cleft-review.crystalpack')
    args = parser.parse_args()
    snapshot = args.snapshot.resolve()
    snapshot.mkdir(parents=True, exist_ok=True)
    candidate_path = ROOT/'planning/minnesota/caves/MnR20Cave1F.candidate.json'
    previous_path = ROOT/'planning/minnesota/caves/MnR20Cave1F.previous.json'
    proof_path = ROOT/'planning/minnesota/renders/caves/r20-native/1630ad063aa4a1cf/review-01'/('native-verification-and-captures-'+PROOF_SHA+'.json')
    assets_log = ROOT/'.local/r20-cleft-native-assets-directional-fixture-fixed.log'
    require(digest(candidate_path) == CANDIDATE_SHA and digest(previous_path) == PREVIOUS_SHA,
            'accepted source/original bytes changed')
    require(digest(args.native_pack) == PACK_SHA, 'exact native written candidate required')
    require(digest(proof_path) == PROOF_SHA, 'exact immutable native preservation report required')
    proof = read(proof_path)
    require(proof['status'] == 'PASS' and len(proof['checks']) == 150
            and all(v['passed'] for v in proof['checks'])
            and proof['pack']['maps'] == 510 and proof['pack']['file_sha256'] == PACK_SHA
            and proof['source_sha256'] == CANDIDATE_SHA and proof['previous_sha256'] == PREVIOUS_SHA
            and len(proof['captures']) == 9 and proof['all_capture_processes_and_pixel_checks_passed'],
            'actual 510-map/150-check/nine-capture preservation evidence required')
    require('test result: ok. 6 passed; 0 failed' in assets_log.read_text(),
            'six actual directional/native roundtrip asset cases must pass')
    candidate, old = read(candidate_path), read(previous_path)
    editorial = {'rows', 'purpose', 'candidate_status', 'review_contract'}
    require(all(old.get(key) == candidate.get(key)
                for key in set(old) | set(candidate) if key not in editorial),
            'original item/ports/room fields must remain exact')
    require(candidate['dimensions_metatiles'] == [9, 8]
            and candidate['exits'] == [[5, 13], [13, 13]]
            and candidate['exit_doors'] == [[60, 33], [76, 31]], 'original two-mouth geometry changed')
    paths = [ROOT/'planning/minnesota/routes/MN_R20.json', ROOT/'planning/minnesota/interiors.json']
    before = [path.read_text() for path in paths]
    require(all(path.read_bytes() == raw.encode('utf-8') for path, raw in zip(paths, before)),
            'preflight must preserve exact source UTF-8 bytes/newlines')
    route = json.loads(before[0])
    contract = next(v for v in json.loads(before[1])['contracts'] if v['interior'] == ROOM)
    require(next(v for v in route['interiors'] if v['id'] == ROOM) == old,
            'live owned room differs from full original freeze')
    require([v['exterior_door_core'] for v in contract['mouths']] == [[60,33],[76,31]]
            and [v['safe_exterior_return_core'] for v in contract['mouths']] == [[60,34],[76,32]]
            and [v['interior_target_warp'] for v in contract['mouths']] == [1,2]
            and all(v['exterior_return_warp'] == 2 for v in contract['mouths']),
            'shared original mouth contracts changed')
    freeze_path = snapshot/'freeze.json'
    if freeze_path.exists():
        freeze = read(freeze_path)
        require(all(digest(path) == freeze['shared_preimage_sha256'][str(path.relative_to(ROOT))]
                    for path in paths), 'shared files changed since this preflight; create a fresh review snapshot')
    else:
        freeze = {'candidate_source_sha256': CANDIDATE_SHA, 'previous_source_sha256': PREVIOUS_SHA,
                  'native_pack_sha256': PACK_SHA, 'native_report': str(proof_path.relative_to(ROOT)),
                  'native_report_sha256': PROOF_SHA, 'assets_log_sha256': digest(assets_log),
                  'shared_preimage_sha256': {str(path.relative_to(ROOT)):digest(path) for path in paths}}
        freeze_path.write_text(json.dumps(freeze, indent=2)+'\n')
        for path, raw in zip(paths, before):
            (snapshot/(path.name+'.before')).write_text(raw)
        for name, value in [('room.before.json',old),('contract.before.json',contract)]:
            (snapshot/name).write_text(json.dumps(value,indent=2)+'\n')
        (snapshot/'assets-native-six-pass.log').write_bytes(assets_log.read_bytes())
    passed = controller_cases(args.controller_log)
    complete = passed == set(CASES)
    if complete:
        receipt_path = ROOT/'.local/r20-cleft-controller-first-receipt.json'
        receipt = read(receipt_path)
        require(receipt['exit_code'] == 0 and receipt['private_binary_removed']
                and digest(Path(receipt['pack'])) == PACK_SHA
                and receipt['binary_sha256'] == '88fb827f92ee1e35d7b88450bdb8e2db2181429ae23dce6e4e9aace8238dc1c5',
                'actual Root controller execution receipt must match this pack epoch')
        (snapshot/'controller-pass-receipt.json').write_bytes(receipt_path.read_bytes())
        for log_path in args.controller_log:
            (snapshot/('controller-'+log_path.name)).write_bytes(log_path.read_bytes())
    require(not args.apply or complete, '--apply is blocked until all three actual controller cases PASS')
    promoted = copy.deepcopy(candidate)
    promoted['candidate_status'] = ('Installed 510-map native review; all three local production-controller cases PASS.'
                                    if complete else 'Installed native terrain review; production-controller acceptance pending, source apply blocked.')
    promoted['review_contract']['unpublished'] = False
    promoted['review_contract']['controller_probes']['exterior_return_semantics'] = RETURN
    promoted['review_contract']['native_review'] = {
        'pack_sha256':PACK_SHA,'palette':'nite','candidate_sha256':CANDIDATE_SHA,
        'preservation_checks':150,'asset_cases_passed':6,'controller_cases':list(CASES),
        'controller_cases_passed':sorted(passed),'earned_SQ03_progression_claimed':False,
    }
    proposed_contract = copy.deepcopy(contract)
    proposed_contract.update({
        'dimensions_metatiles':[9,8],'native_palette':'nite','candidate_source_sha256':CANDIDATE_SHA,
        'native_review_pack_sha256':PACK_SHA,'runtime_settled_exterior_cores':[[60,34],[76,32]],
        'return_semantics':RETURN,
        'validation':'Actual written 510-map terrain/preservation150, nine native captures and six directional assets PASS; both original mouths/item/flag/program/art/attributes preserved.' + (' All three production-controller cases PASS.' if complete else ' Controller cases pending; do not apply.'),
        'native_status':'Installed dry9x8 Voyageurs cleft on1630ad06; original forced NITE. Leftward two-lane native drop, useful stairs and ordinary returns; no added field-move requirement or SQ03 authority.' + (' Native input/once/item use/Continue PASS.' if complete else ' Production input acceptance pending.'),
    })
    after, preservation = [], []
    for raw,key,expected,replacement in [(before[0],'id',old,promoted),(before[1],'interior',contract,proposed_contract)]:
        revised, details = splice_record(raw,key,expected,replacement)
        after.append(revised);preservation.append(details)
    for name,value in [('room.proposed.json',promoted),('contract.proposed.json',proposed_contract)]:
        (snapshot/name).write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n')
    for path,raw in zip(paths,after):
        (snapshot/(path.name+'.proposed')).write_text(raw)
    summary={'proposed_live_files':{str(path.relative_to(ROOT)):hashlib.sha256(raw.encode('utf-8')).hexdigest()for path,raw in zip(paths,after)},
             'owned_before_after':{name:digest(snapshot/name)for name in ['room.before.json','room.proposed.json','contract.before.json','contract.proposed.json']},
             'applied':False,'controller_acceptance_complete':complete,'native_pack_sha256':PACK_SHA,
             'candidate_source_sha256':CANDIDATE_SHA,'outside_owned_records':preservation,
             'unchanged_live_files':{str(path.relative_to(ROOT)):digest(path) for path in paths}}
    (snapshot/'preflight.json').write_text(json.dumps(summary,indent=2)+'\n')
    if args.apply:
        require(all(path.read_text()==raw for path,raw in zip(paths,before)), 'sources changed during preflight')
        application=snapshot/('application-'+str(time.time_ns()));application.mkdir()
        for path,raw in zip(paths,before):(application/(path.name+'.before')).write_text(raw)
        for path in args.controller_log:(application/path.name).write_bytes(path.read_bytes())
        staged,written=[],[]
        try:
            for path,raw in zip(paths,after):
                temporary=path.with_name('.'+path.name+'.r20-promotion.tmp')
                require(not temporary.exists(),'existing source staging file');temporary.write_text(raw);staged.append(temporary)
            require(all(path.read_text()==raw for path,raw in zip(paths,before)), 'sources changed before replacement')
            for temporary,path in zip(staged,paths):os.replace(temporary,path);written.append(path)
        except BaseException:
            for path,old_raw,new_raw in zip(paths,before,after):
                if path in written:
                    require(path.read_text()==new_raw,'external edit prevents rollback; before snapshots retained')
                    rollback=path.with_name('.'+path.name+'.r20-rollback.tmp')
                    require(not rollback.exists(),'existing rollback staging file');rollback.write_text(old_raw);os.replace(rollback,path)
            raise
        finally:
            for temporary in staged:temporary.unlink(missing_ok=True)
        summary['applied']=True;summary['after_files']={str(path.relative_to(ROOT)):digest(path)for path in paths}
        summary['controller_logs']={str(path):digest(path)for path in args.controller_log}
        (application/'promotion.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps(summary))


if __name__=='__main__':
    main()
