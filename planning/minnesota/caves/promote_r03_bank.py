#!/usr/bin/env python3
"""Guarded R03 source promotion; default preflight, never builds a native pack."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import time

ROOT = Path(__file__).resolve().parents[3]
ROOM = 'MnR03Cave1F'
CANDIDATE_SHA = '50738c06417b03802e513a6076b5fbfba301c9b3698723e3588f0ab4fcd92243'
PREVIOUS_SHA = '7a67696a71b110ec6b2d52bb346b6e4445f8132c9b790008569b67e5197516ac'
PACK_SHA = '5fc652a6310beaf52d95e4ac339b4c12279e6fdae339d39c8c9597dc9b7a2df6'
PROOF_SHA = 'ea7f4e8cbce3a074aa4d49a4b76571692630c56bdc34de4bd9397ff6243d3267'
CASES = ('r03_bank_cave_controller_native_roof_walker_three_pages_great_ball_once_and_continue',)
RETURN = ('Original room warp1 at7,9 returns directly to MnR03C00R00 passive exteriorwarp2 at8,38 FLOOR. '
          'Active CAVE8,37 exteriorwarp1 targets roomwarp1; inside clear Floor7,8 remains exact. '
          'Waiting on or revisiting passive Floor does not reenter; no warp record is changed.')



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
                        default=ROOT/'content-packs/minnesota-northstar-bank-cave-review.crystalpack')
    args = parser.parse_args()
    snapshot = args.snapshot.resolve()
    snapshot.mkdir(parents=True, exist_ok=True)
    candidate_path = ROOT/'planning/minnesota/caves/MnR03Cave1F.candidate.json'
    previous_path = ROOT/'planning/minnesota/caves/MnR03Cave1F.previous.json'
    proof_path = ROOT/'planning/minnesota/renders/caves/r03-native/5fc652a6310beaf5/review-01'/('native-verification-and-captures-'+PROOF_SHA+'.json')
    source_proof_path = ROOT/'planning/minnesota/renders/caves/r03-candidate/v02/source-proof-4f2667416ad58fbf228559c92e4a7a1f2c70db03315f151fb49cf6b6d9ebff2f.json'
    require(digest(candidate_path) == CANDIDATE_SHA and digest(previous_path) == PREVIOUS_SHA,
            'accepted source/original bytes changed')
    require(digest(args.native_pack) == PACK_SHA, 'exact native written candidate required')
    require(digest(proof_path) == PROOF_SHA, 'exact immutable native preservation report required')
    proof = read(proof_path)
    require(proof['status'] == 'PASS' and len(proof['checks']) == 147
            and all(v['passed'] for v in proof['checks'])
            and proof['pack']['maps'] == 510 and proof['pack']['file_sha256'] == PACK_SHA
            and proof['source_sha256'] == CANDIDATE_SHA and proof['previous_sha256'] == PREVIOUS_SHA
            and len(proof['captures']) == 6 and proof['all_capture_processes_and_pixel_checks_passed'],
            'actual 510-map/147-check/six-capture preservation evidence required')
    require(digest(source_proof_path) == '4f2667416ad58fbf228559c92e4a7a1f2c70db03315f151fb49cf6b6d9ebff2f',
            'exact source directional collision proof required')
    source_proof = read(source_proof_path)
    require(source_proof['passed'] == source_proof['total'] == 31 and not source_proof['failed'],
            'all31 source checks/ten negatives must pass')
    candidate, old = read(candidate_path), read(previous_path)
    editorial = {'rows', 'purpose', 'candidate_status', 'review_contract'}
    require(all(old.get(key) == candidate.get(key)
                for key in set(old) | set(candidate) if key not in editorial),
            'original item/ports/room fields must remain exact')
    require(candidate['dimensions_metatiles'] == [8, 6]
            and candidate['exits'] == [[7, 9]] and candidate['clear_arrival'] == [7, 8],
            'original single mouth geometry changed')
    paths = [ROOT/'planning/minnesota/routes/MN_R03.json', ROOT/'planning/minnesota/interiors.json']
    before = [path.read_text() for path in paths]
    require(all(path.read_bytes() == raw.encode('utf-8') for path, raw in zip(paths, before)),
            'preflight must preserve exact source UTF-8 bytes/newlines')
    route = json.loads(before[0])
    contract = next(v for v in json.loads(before[1])['contracts'] if v['interior'] == ROOM)
    require(next(v for v in route['interiors'] if v['id'] == ROOM) == old,
            'live owned room differs from full original freeze')
    require(contract['exterior_map'] == 'MnR03C00R00'
            and contract['exterior_door_core'] == [8,37]
            and contract['exterior_return_core'] == [8,38]
            and contract['exterior_warp_index'] == 1 and contract['exterior_return_warp_index'] == 2,
            'shared original mouth/passive return changed')
    freeze_path = snapshot/'freeze.json'
    if freeze_path.exists():
        freeze = read(freeze_path)
        require(all(digest(path) == freeze['shared_preimage_sha256'][str(path.relative_to(ROOT))]
                    for path in paths), 'shared files changed since this preflight; create a fresh review snapshot')
    else:
        freeze = {'candidate_source_sha256': CANDIDATE_SHA, 'previous_source_sha256': PREVIOUS_SHA,
                  'native_pack_sha256': PACK_SHA, 'native_report': str(proof_path.relative_to(ROOT)),
                  'native_report_sha256': PROOF_SHA, 'source_proof_sha256': digest(source_proof_path),
                  'shared_preimage_sha256': {str(path.relative_to(ROOT)):digest(path) for path in paths}}
        freeze_path.write_text(json.dumps(freeze, indent=2)+'\n')
        for path, raw in zip(paths, before):
            (snapshot/(path.name+'.before')).write_text(raw)
        for name, value in [('room.before.json',old),('contract.before.json',contract)]:
            (snapshot/name).write_text(json.dumps(value,indent=2)+'\n')
        (snapshot/'source-31-checks.json').write_bytes(source_proof_path.read_bytes())
    passed = controller_cases(args.controller_log)
    complete = passed == set(CASES)
    if complete:
        receipt_path = ROOT/'.local/r03-bank-controller-first-receipt.json'
        receipt = read(receipt_path)
        require(receipt['exit_code'] == 0 and receipt['private_binary_removed']
                and digest(Path(receipt['pack'])) == PACK_SHA
                and receipt['binary_sha256'] == '8aaad57a8c3d44b9066c04b0c9dac6dbe18b8fa1f4b3c9123f9109ba3521d82c',
                'actual Root controller execution receipt must match this pack epoch')
        (snapshot/'controller-pass-receipt.json').write_bytes(receipt_path.read_bytes())
        for log_path in args.controller_log:
            (snapshot/('controller-'+log_path.name)).write_bytes(log_path.read_bytes())
    require(not args.apply or complete, '--apply is blocked until the actual controller case PASS')
    promoted = copy.deepcopy(candidate)
    promoted['candidate_status'] = ('Installed 510-map native review; the local production-controller case PASS.'
                                    if complete else 'Installed native terrain review; production-controller acceptance pending, source apply blocked.')
    promoted['review_contract']['unpublished'] = False
    promoted['review_contract']['controller_probes']['exterior_return_semantics'] = RETURN
    promoted['review_contract']['native_review'] = {
        'pack_sha256':PACK_SHA,'palette':'nite','candidate_sha256':CANDIDATE_SHA,
        'preservation_checks':147,'source_checks_passed':31,'controller_cases':list(CASES),
        'controller_cases_passed':sorted(passed),'earned_regional_progression_claimed':False,
    }
    proposed_contract = copy.deepcopy(contract)
    proposed_contract.update({
        'dimensions_metatiles':[8,6],'native_palette':'nite','candidate_source_sha256':CANDIDATE_SHA,
        'exterior_door_status':'Actual native entry, granite-nose walking with directional roof land, all three original walker pages, Great Ball pickup once, collected-tile safety, passive Floor return and Continue PASS through production controller input.',
        'native_review_pack_sha256':PACK_SHA,'runtime_settled_exterior_core':[8,38],
        'return_semantics':RETURN,
        'validation':'Actual written 510-map terrain/preservation147, six native captures and31 source checks PASS; original walker six lines/native program, Great Ball1/flag and sole mouth/art/attributes preserved.' + (' Production-controller case PASS.' if complete else ' Controller case pending; do not apply.'),
        'native_status':'Installed dry8x6 River Valley bank pocket on5fc652a6; original forced NITE. Granite-nose bend, actual directional roof land, original walker/Great Ball and genuine passive return; no added HM, service, story or field use.' + (' Native pages/input ownership/pickup once/cleared-tile safety/Continue PASS.' if complete else ' Production input acceptance pending.'),
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
                temporary=path.with_name('.'+path.name+'.r03-promotion.tmp')
                require(not temporary.exists(),'existing source staging file');temporary.write_text(raw);staged.append(temporary)
            require(all(path.read_text()==raw for path,raw in zip(paths,before)), 'sources changed before replacement')
            for temporary,path in zip(staged,paths):os.replace(temporary,path);written.append(path)
        except BaseException:
            for path,old_raw,new_raw in zip(paths,before,after):
                if path in written:
                    require(path.read_text()==new_raw,'external edit prevents rollback; before snapshots retained')
                    rollback=path.with_name('.'+path.name+'.r03-rollback.tmp')
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
