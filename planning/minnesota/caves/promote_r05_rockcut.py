#!/usr/bin/env python3
"""Guarded RockCut promotion; default preflight, requiring both actual cases to apply."""
import argparse
from copy import deepcopy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sys
import time

ROOT = Path(__file__).resolve().parents[3]
ROOM = 'MnR05RockCut'
SOURCE_SHA = '485652d7bdd92e37660acc187a71478d355cffde3e6e106801c22a125339c86b'
PREVIOUS_SHA = '1108fe31248453baffd28642d196be3976532592074616cd4ec546abc1342932'
GEOMETRY_SHA = '96a10295af34deeb57cd8ebdaf882d8eb80a70e49f63f8bc27ef84980dc1bcc1'
PACK_SHA = '31258a1788a986b158d0679e4f3a26c4dd476964311f707a6e09c9640cad48c6'
NATIVE_PROOF_SHA = 'adfcd9541ad40ecfb710aa0d541820af5a1511d6e73c9321c2406e53ef6daf2a'
SOURCE_PROOF_SHA = '02d60ec86087d53bc898aa11b00fcba71f03e477ca65a7dd1ff2ad6ae3e3d071'
CASES = (
    'r05_rockcut_controller_both_mouths_stair_healer_two_pages_and_continue',
    'r05_rockcut_controller_original_repel_once_pack_use_movement_and_continue',
)
RETURN = ('Original south insidewarp1 at (5,17), clear Floor (5,16), returns to '
          'MnR05C00R01 passive exteriorwarp2 at local (16,30), global (16,78). '
          'Original north insidewarp2 at (11,5), clear Floor (11,4), returns to '
          'MnR05C00R00 passive exteriorwarp2 at (30,28). Active exterior CAVE '
          'tiles (16,77) and (30,27) target insidewarps1 and2 respectively. '
          'Both inactive passive records retain their original target insidewarp1. '
          'No warp record changes; waiting or revisiting actual passive Floor cannot reenter.')

spec = importlib.util.spec_from_file_location('r03_source_splice', Path(__file__).with_name('promote_r03_bank.py'))
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
base.ROOM = ROOM
require, digest, read, splice_record = base.require, base.digest, base.read, base.splice_record
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack


def save_hashed(path, value):
    raw = (json.dumps(value,indent=2)+'\n').encode()
    frozen = path.with_name(path.stem+'-'+hashlib.sha256(raw).hexdigest()+path.suffix)
    require(not frozen.exists() or frozen.read_bytes() == raw,'immutable report collision')
    frozen.write_bytes(raw)
    path.write_bytes(raw)


def final_inheritance(snapshot, geometry_path, pack_path):
    require(digest(geometry_path) == GEOMETRY_SHA and digest(pack_path) == PACK_SHA,'exact written geometry and final100 packs required')
    before,after = [load_pack(path) for path in [geometry_path,pack_path]]
    checks = []
    def check(name, passed):
        checks.append({'check':name,'passed':bool(passed)})
    check('all510 native modules exact after isolated item correction',len(before['data']['maps']) == len(after['data']['maps']) == 510 and before['data']['maps'] == after['data']['maps'])
    check('only actual Repel duration200 to100 changes',before['data']['items']['REPEL']['repel_steps'] == 200 and after['data']['items']['REPEL']['repel_steps'] == 100)
    data = deepcopy(after['data'])
    data['items']['REPEL']['repel_steps'] = 200
    check('entire GameDataSet exact after normalizing only Repel duration',data == before['data'])
    check('Super and Max Repel remain200 and250',after['data']['items']['SUPER_REPEL']['repel_steps'] == 200 and after['data']['items']['MAX_REPEL']['repel_steps'] == 250)
    for key in sorted(set(before)|set(after)):
        if key not in {'data','identity'}:
            check('entire unrelated pack field '+key,before.get(key) == after.get(key))
    identity = deepcopy(after['identity'])
    check('derived item correction content hash changes',identity['content_hash'] != before['identity']['content_hash'])
    identity['content_hash'] = before['identity']['content_hash']
    check('every other identity field exact',identity == before['identity'])
    require(all(v['passed'] for v in checks),'final100 pack differs beyond isolated Repel duration/derived identity')
    result = {'status':'PASS','before_sha256':GEOMETRY_SHA,'after_sha256':PACK_SHA,'checks':checks,'limits':'Read-only written-pack inheritance proof; actual pinned-source certification/parser tests are independently owned by Root.'}
    save_hashed(snapshot/'final-repel-inheritance-proof.json',result)
    return result


def controller_evidence(logs, receipt_path, snapshot):
    passed = set()
    for path in logs:
        text = path.read_text()
        require('test result: ok.' in text and 'test result: FAILED' not in text,'only actual passing controller logs accepted')
        passed.update(case for case in CASES if re.search(r'::'+re.escape(case)+r' \.\.\. ok',text))
    complete = passed == set(CASES)
    if complete:
        require(receipt_path is not None,'matching real successful controller execution receipt required')
        receipt = read(receipt_path)
        require(receipt['exit_code'] == 0 and receipt['private_binary_removed'] and receipt['pack_sha256'] == PACK_SHA and digest(Path(receipt['pack'])) == PACK_SHA,'actual successful controller receipt must match final100 pack')
        require(re.fullmatch('[0-9a-f]{64}',receipt['binary_sha256']) is not None,'actual controller binary hash required')
        require(any(Path(receipt['log']).resolve() == path.resolve() for path in logs),'receipt must identify supplied successful log')
        (snapshot/'controller-pass-receipt.json').write_bytes(receipt_path.read_bytes())
        for path in logs:
            (snapshot/('controller-'+path.name)).write_bytes(path.read_bytes())
    return passed,complete


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--snapshot',type=Path,required=True)
    parser.add_argument('--apply',action='store_true')
    parser.add_argument('--controller-log',type=Path,action='append',default=[])
    parser.add_argument('--controller-receipt',type=Path)
    args = parser.parse_args()
    snapshot = args.snapshot.resolve()
    snapshot.mkdir(parents=True,exist_ok=True)
    source = ROOT/'planning/minnesota/caves/MnR05RockCut.candidate.json'
    previous = ROOT/'planning/minnesota/caves/MnR05RockCut.previous.json'
    require(digest(source) == SOURCE_SHA and digest(previous) == PREVIOUS_SHA,'frozen source/original changed')
    geometry = ROOT/'content-packs/minnesota-northstar-rockcut-review.crystalpack'
    pack = ROOT/'content-packs/minnesota-northstar-repel-review.crystalpack'
    native_path = ROOT/'planning/minnesota/renders/caves/r05-rockcut-native/96a10295af34deeb/review-01'/('native-verification-and-captures-'+NATIVE_PROOF_SHA+'.json')
    source_path = ROOT/'planning/minnesota/renders/caves/r05-rockcut-candidate/v02'/('source-proof-'+SOURCE_PROOF_SHA+'.json')
    require(digest(native_path) == NATIVE_PROOF_SHA and digest(source_path) == SOURCE_PROOF_SHA,'exact source/native evidence required')
    native_proof, source_proof = read(native_path),read(source_path)
    require(native_proof['status'] == 'PASS' and len(native_proof['checks']) == 163 and all(v['passed'] for v in native_proof['checks']) and native_proof['personally_visually_reviewed'] and len(native_proof['captures']) == 8 and native_proof['pixel_totals']['full_lcd_compared'] == 184320 and native_proof['pixel_totals']['full_lcd_mismatches'] == 0 and native_proof['pack']['file_sha256'] == GEOMETRY_SHA and native_proof['source_sha256'] == SOURCE_SHA,'actual native163/eight-camera reviewed proof required')
    require(source_proof['status'] == 'PASS' and len(source_proof['checks']) == 61 and all(v['passed'] for v in source_proof['checks']) and len(source_proof['negative_variants']) == 13,'source61/thirteen-negative proof required')
    final_inheritance(snapshot,geometry,pack)
    candidate,old = read(source),read(previous)
    editorial = {'rows','purpose','candidate_status','review_contract'}
    require(all(old.get(k) == candidate.get(k) for k in set(old)|set(candidate) if k not in editorial),'original actor/service/item/port fields must remain exact')
    require(candidate['dimensions_metatiles'] == [8,10] and candidate['exits'] == [[5,17],[11,5]] and candidate['clear_arrival'] == [5,16],'original two-mouth geometry changed')
    paths = [ROOT/'planning/minnesota/routes/MN_R05.json',ROOT/'planning/minnesota/interiors.json']
    before = [path.read_text() for path in paths]
    require(all(path.read_bytes() == raw.encode('utf-8') for path,raw in zip(paths,before)),'exact UTF8 source bytes/newlines required')
    require(next(v for v in json.loads(before[0])['interiors'] if v['id'] == ROOM) == old,'live owned room no longer matches full original freeze')
    contract = next(v for v in json.loads(before[1])['contracts'] if v['interior'] == ROOM)
    require(contract['exterior_doors_core'] == [[16,77],[30,27]] and contract['safe_exterior_returns_core'] == [[16,78],[30,28]] and contract['native_interior_warp_indices'] == [1,2] and contract['exterior_return_warp_indices'] == [2,2],'shared paired mouth/return contracts changed')
    freeze_path = snapshot/'freeze.json'
    if freeze_path.exists():
        freeze = read(freeze_path)
        require(all(digest(path) == freeze['shared_preimage_sha256'][str(path.relative_to(ROOT))] for path in paths),'shared source changed since preflight; create a fresh review snapshot')
    else:
        freeze = {'candidate_sha256':SOURCE_SHA,'previous_sha256':PREVIOUS_SHA,'geometry_pack_sha256':GEOMETRY_SHA,'final100_pack_sha256':PACK_SHA,'native_report_sha256':NATIVE_PROOF_SHA,'source_report_sha256':SOURCE_PROOF_SHA,'shared_preimage_sha256':{str(path.relative_to(ROOT)):digest(path) for path in paths}}
        save_hashed(freeze_path,freeze)
        for path,raw in zip(paths,before):
            (snapshot/(path.name+'.before')).write_text(raw)
        for name,value in [('room.before.json',old),('contract.before.json',contract)]:
            (snapshot/name).write_text(json.dumps(value,indent=2)+'\n')
        for name,path in [('frozen.candidate.json',source),('frozen.previous.json',previous),('source-61-checks.json',source_path),('native-163-checks.json',native_path)]:
            (snapshot/name).write_bytes(path.read_bytes())
    passed,complete = controller_evidence(args.controller_log,args.controller_receipt,snapshot)
    require(not args.apply or complete,'--apply blocked until both actual final100 controller cases PASS')
    promoted = deepcopy(candidate)
    promoted['candidate_status'] = ('Installed native review; both local production-controller cases PASS.' if complete else 'Installed native geometry review; controller acceptance pending, source apply blocked.')
    promoted['review_contract']['unpublished'] = False
    promoted['review_contract']['controller_probes']['exterior_return_semantics'] = RETURN
    promoted['review_contract']['native_review'] = {'geometry_pack_sha256':GEOMETRY_SHA,'final100_pack_sha256':PACK_SHA,'palette':'nite','candidate_sha256':SOURCE_SHA,'preservation_checks':163,'source_checks_passed':61,'captures':8,'full_lcd_pixels_exact':184320,'controller_cases':list(CASES),'controller_cases_passed':sorted(passed),'earned_regional_progression_claimed':False}
    revised_contract = deepcopy(contract)
    revised_contract.update({'dimensions_metatiles':[8,10],'native_palette':'nite','candidate_source_sha256':SOURCE_SHA,'native_geometry_pack_sha256':GEOMETRY_SHA,'native_review_pack_sha256':PACK_SHA,'return_semantics':RETURN,'requires_real_collision_check':not complete,'exterior_door_status':('Both original native mouth pairings, genuine passive Floor returns, useful bidirectional stair/sidewall, two printed healer pages/input lock, real HP/status/PP healing and Continue PASS.' if complete else 'Native two-mouth geometry/preservation and eight actual views PASS; both controller cases pending, source apply blocked.'),'validation':'Source61/thirteen negative variants and actual native163/eight captures PASS; original actors, services, programs, rewards, paired mouths, NITE, art and all unrelated content preserved. Final100 pack inherits all510 geometry modules exactly; only canonical Repel duration/derived identity changes.'+(' Both production-controller cases PASS, including original Repel pickup once, actual Pack USE100, walking countdown and Continue.' if complete else ' Production-controller acceptance pending; do not apply.'),'native_status':'Installed winding8x10 river-bluff RockCut on96a10295, inherited exact in31258a17 with source-correct100-step Repel; one actual stair, original healer/Repel/two mouths, no new actors, HM, reward or story condition.'})
    after,preservation = [],[]
    for raw,key,expected,replacement in [(before[0],'id',old,promoted),(before[1],'interior',contract,revised_contract)]:
        value,details = splice_record(raw,key,expected,replacement)
        after.append(value)
        preservation.append(details)
    for name,value in [('room.proposed.json',promoted),('contract.proposed.json',revised_contract)]:
        (snapshot/name).write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n')
    for path,raw in zip(paths,after):
        (snapshot/(path.name+'.proposed')).write_text(raw)
    (snapshot/'promote_r05_rockcut.py.snapshot').write_bytes(Path(__file__).read_bytes())
    summary = {'applied':False,'controller_acceptance_complete':complete,'controller_cases_passed':sorted(passed),'candidate_sha256':SOURCE_SHA,'native_geometry_sha256':GEOMETRY_SHA,'final100_pack_sha256':PACK_SHA,'proposed_live_files':{str(path.relative_to(ROOT)):hashlib.sha256(raw.encode()).hexdigest() for path,raw in zip(paths,after)},'owned_before_after':{name:digest(snapshot/name) for name in ['room.before.json','room.proposed.json','contract.before.json','contract.proposed.json']},'outside_owned_records':preservation,'unchanged_live_files':{str(path.relative_to(ROOT)):digest(path) for path in paths}}
    save_hashed(snapshot/'preflight.json',summary)
    if args.apply:
        require(all(path.read_text() == raw for path,raw in zip(paths,before)),'source changed during preflight')
        application = snapshot/('application-'+str(time.time_ns()))
        application.mkdir()
        for path,raw in zip(paths,before):
            (application/(path.name+'.before')).write_text(raw)
        for path in args.controller_log:
            (application/path.name).write_bytes(path.read_bytes())
        staged,written = [],[]
        try:
            for path,raw in zip(paths,after):
                temporary = path.with_name('.'+path.name+'.r05-rockcut-promotion.tmp')
                require(not temporary.exists(),'existing source staging file')
                temporary.write_text(raw)
                staged.append(temporary)
            require(all(path.read_text() == raw for path,raw in zip(paths,before)),'shared source changed before replacement')
            for temporary,path in zip(staged,paths):
                os.replace(temporary,path)
                written.append(path)
        except BaseException:
            for path,old_raw,new_raw in zip(paths,before,after):
                if path in written:
                    require(path.read_text() == new_raw,'external edit prevents rollback; snapshots retained')
                    rollback = path.with_name('.'+path.name+'.r05-rockcut-rollback.tmp')
                    require(not rollback.exists(),'existing rollback staging file')
                    rollback.write_text(old_raw)
                    os.replace(rollback,path)
            raise
        finally:
            for temporary in staged:
                temporary.unlink(missing_ok=True)
        summary['applied'] = True
        summary['after_files'] = {str(path.relative_to(ROOT)):digest(path) for path in paths}
        summary['controller_logs'] = {str(path):digest(path) for path in args.controller_log}
        save_hashed(application/'promotion.json',summary)
    print(json.dumps(summary),flush=True)


if __name__ == '__main__':
    main()
