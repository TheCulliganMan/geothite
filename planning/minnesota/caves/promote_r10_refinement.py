#!/usr/bin/env python3
"""Guarded R10 refinement promotion PREVIEW ONLY; no shared/native writes."""
import argparse
from copy import deepcopy
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import shutil
import time

ROOT=Path(__file__).resolve().parents[3]
ROOM='MnR10Cave1F'
CANDIDATE_SHA='be41b5fb81f3ef8536214e903a07d28685bd71b4383cb4c3fade9ebf8ab73161'
PREVIOUS_SHA='2e320c5676fcbb330152cb088e75a688ef344942add7402f48b58e2265b4b5f7'
PACK_SHA='fe57f83bdddf43fd1e617abb4cac7855f6118691ee07479dd77be27c1c815fb0'
PROOF_SHA='86d9ea05d3fd410c91bf9072f319cbebe6ad371cdd36fbeee81d6d40d8ab4c0b'
CASES=('r10_gallery_controller_original_mouth_passive_return_four_stairs_and_short_native_drop','r10_gallery_controller_original_reader_pages_revive_once_and_semantic_continue','r10_gallery_controller_full_bag_refusal_native_potion_use_and_original_pickup_retry')
spec=importlib.util.spec_from_file_location('existing_cave_promotion',Path(__file__).with_name('promote_r20_cleft.py'))
helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper);helper.ROOM=ROOM
require=helper.require
read=helper.read
digest=helper.digest


def write_json(path,value):path.write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--snapshot',type=Path)
    args=parser.parse_args()
    candidate_path=ROOT/'planning/minnesota/caves/MnR10Cave1F.refinement.candidate.json'
    previous_path=ROOT/'planning/minnesota/caves/MnR10Cave1F.refinement.previous.json'
    pack_path=ROOT/'content-packs/minnesota-northstar-tight-gallery-review.crystalpack'
    proof_path=ROOT/'planning/minnesota/renders/caves/r10-refinement-native/fe57f83bdddf43fd/review-01'/('native-verification-and-captures-'+PROOF_SHA+'.json')
    require(digest(candidate_path)==CANDIDATE_SHA and digest(previous_path)==PREVIOUS_SHA,'exact accepted candidate/current18x14 preimage required')
    require(digest(pack_path)==PACK_SHA and digest(proof_path)==PROOF_SHA,'exact native pack and frozen150/eight proof required')
    proof=read(proof_path)
    require(proof['status']=='PASS'and len(proof['checks'])==150 and all(v['passed']for v in proof['checks'])and len(proof['captures'])==8 and proof['all_capture_processes_and_pixel_checks_passed']and proof['pack']['file_sha256']==PACK_SHA and proof['pack']['maps']==510 and proof['source_sha256']==CANDIDATE_SHA and proof['previous_sha256']==PREVIOUS_SHA and proof['personally_visually_reviewed'],'complete exact native preservation/presentation proof required')
    require(all(c['exit_code']==0 and c['exact_pixel_diagnostics']['player_opaque_pixel_mismatches']==0 and c['exact_pixel_diagnostics']['terrain_pixel_mismatches_outside_oam']==0 and c['raw_gpu_archive']for c in proof['captures']),'every actual native image must have zero pixel mismatches and raw GPU receipt')
    controller_log=ROOT/'.local/r10-refinement-controller-both-lanes.log'
    controller_receipt=ROOT/'.local/r10-refinement-controller-both-lanes-receipt.json'
    assets_log=ROOT/'.local/r10-refinement-native-course-shoulders-regression.log'
    regression_log=ROOT/'.local/r10-course-r12-regression.log'
    logs=controller_log.read_text()
    require('test result: ok. 3 passed; 0 failed' in logs and 'test result: FAILED'not in logs and all(re.search(r'::'+re.escape(case)+r' \.\.\. ok',logs)for case in CASES),'all3 actual R10 controller cases required')
    receipt=read(controller_receipt)
    require(receipt['exit_code']==0 and receipt['pack_sha256']==PACK_SHA and receipt['filter']=='r10_gallery_controller_'and receipt['private_binary_removed'],'actual controller receipt must identify exact native pack')
    require('test result: ok. 7 passed; 0 failed'in assets_log.read_text()and 'test cave_revision_accepts_native_floor_lips_but_rejects_open_course_shoulders ... ok'in assets_log.read_text(),'seven native asset cases/course-side regression required')
    require('test result: ok. 5 passed; 0 failed'in regression_log.read_text(),'five R12 regression cases required')
    candidate,old=read(candidate_path),read(previous_path)
    retained=deepcopy(candidate);refinement=retained['review_contract'].pop('refinement')
    require({k:v for k,v in retained.items()if k!='rows'}=={k:v for k,v in old.items()if k!='rows'},'every original nonterrain value/review subfield must remain exact')
    require(candidate['dimensions_metatiles']==[18,14]and candidate['exits']==[[3,13]]and candidate['clear_arrival']==[3,12]and candidate['residents']==old['residents']and candidate['items']==old['items'],'original extent/mouth/reader/currentRevive anchors required')
    paths=[ROOT/'planning/minnesota/routes/MN_R10.json',ROOT/'planning/minnesota/interiors.json']
    before_bytes=[p.read_bytes()for p in paths];before=[raw.decode('utf-8')for raw in before_bytes]
    route,interiors=map(json.loads,before)
    live=next(v for v in route['interiors']if v['id']==ROOM)
    contract=next(v for v in interiors['contracts']if v['interior']==ROOM)
    require(live==old,'live R10 owned room differs from exact installed18x14 freeze')
    require(contract['dimensions_metatiles']==[18,14]and contract['native_palette']=='nite'and contract['arrival_core']==[3,12]and contract['native_warp_arrival_core']==[3,13]and contract['exterior_door_core']==[36,17]and contract['safe_exterior_return_core']==[36,18],'original shared interior/warp contract required')
    promoted=deepcopy(old);promoted['rows']=candidate['rows']
    refinement['status']='Native review candidate installed; Root accepted bounded whole/player visuals. All3 local production-controller cases PASS. User final approval and earned regional progression are not claimed.'
    refinement['native_acceptance']={'source_candidate_sha256':CANDIDATE_SHA,'pack_sha256':PACK_SHA,'proof_sha256':PROOF_SHA,'preservation_checks':150,'production_captures':8,'controller_log_sha256':digest(controller_log),'controller_receipt_sha256':digest(controller_receipt),'controller_cases':list(CASES),'both_hop_lanes_and_actual_returns':[13,15],'full_bag_retry_and_Continue':True,'root_bounded_visual_acceptance':True,'user_final_approval_claimed':False,'earned_regional_progress_claimed':False}
    promoted['review_contract']['refinement']=refinement
    normalized=deepcopy(promoted);normalized['rows']=old['rows'];normalized['review_contract'].pop('refinement')
    require(normalized==old,'proposed room changes only rows and new refinement annotation')
    revised_contract=deepcopy(contract)
    revised_contract['candidate_source_sha256']=CANDIDATE_SHA
    revised_contract['native_status']='Native510-map tight-gallery review candidate: unchanged18x14/NITE;16 terrain metatiles refine ceiling and lower return. Exact original reader/text, Revive29,23/program/event/quantity, four stairs, both native hop lanes and actual13/15-step ordinary returns, mouth3,13/clear3,12/exterior passive36,18 preserved. Three actual local controller cases PASS7.14s, including both hop lanes/reverse/returns, full-bag refusal/native Potion use/original pickup retry and Continue;150 written-pack checks and eight headless native captures PASS. Root accepted bounded visuals. User final approval and earned Q06/regional progression are not claimed.'
    require({k:v for k,v in revised_contract.items()if k not in {'candidate_source_sha256','native_status'}}=={k:v for k,v in contract.items()if k not in {'candidate_source_sha256','native_status'}},'only two owned contract annotations may change')
    replacements=[promoted,revised_contract];keys=['id','interior'];after=[];outside={}
    for p,raw,expected,replacement,key in zip(paths,before,[old,contract],replacements,keys):
        proposed,evidence=helper.splice_record(raw,key,expected,replacement)
        after.append(proposed);outside[str(p.relative_to(ROOT))]=evidence
    require(all(p.read_bytes()==raw for p,raw in zip(paths,before_bytes)),'source changed while preparing preflight')
    snapshot=(args.snapshot or ROOT/'planning/minnesota/renders/caves/r10-refinement/promotion'/('preflight-'+str(time.time_ns()))).resolve()
    snapshot.mkdir(parents=True,exist_ok=False)
    for p,raw,proposed in zip(paths,before_bytes,after):
        (snapshot/(p.name+'.before')).write_bytes(raw);(snapshot/(p.name+'.proposed')).write_bytes(proposed.encode('utf-8'))
    for name,value in [('room.before.json',old),('contract.before.json',contract),('room.proposed.json',promoted),('contract.proposed.json',revised_contract)]:write_json(snapshot/name,value)
    for p in [candidate_path,previous_path,proof_path,controller_log,controller_receipt,assets_log,regression_log]:shutil.copy2(p,snapshot/p.name)
    write_json(snapshot/'root-viewed-receipt.json',{'authority':'Trusted Root task message for this preflight','native_pack_sha256':PACK_SHA,'native_report_sha256':PROOF_SHA,'root_personally_viewed':'All8 actual R10 native frames and spring phase audit','decision':'Accepted bounded refinement for guarded source preflight review','controller_evidence':'All3 actual controller casesPASS7.14s, both hop lanes/reverse/13+15 returns and full-bag/retry','native_assets':'7 generic asset casesPASS;5 R12 regression casesPASS','source_apply_authorized':False,'user_final_approval_claimed':False,'earned_progress_claimed':False})
    freeze={'status':'PREFLIGHT ONLY; shared source unchanged','applied':False,'candidate_sha256':CANDIDATE_SHA,'previous_sha256':PREVIOUS_SHA,'native_pack_sha256':PACK_SHA,'native_report_sha256':PROOF_SHA,'source_promoter_sha256':digest(Path(__file__)),'shared_preimage_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(raw).hexdigest()for p,raw in zip(paths,before_bytes)},'proposed_complete_file_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(raw.encode()).hexdigest()for p,raw in zip(paths,after)},'outside_owned_record_evidence':outside,'owned_change':{'room':'rows + review_contract.refinement only; all original review subfields exact','contract':['candidate_source_sha256','native_status']},'evidence_files_sha256':{p.name:digest(p)for p in snapshot.iterdir()if p.is_file()},'current_sources_still_exact':all(p.read_bytes()==raw for p,raw in zip(paths,before_bytes))}
    require(freeze['current_sources_still_exact'],'source changed before preflight completed')
    write_json(snapshot/'freeze.json',freeze);shutil.copy2(Path(__file__),snapshot/'promote_r10_refinement.py.snapshot')
    print(json.dumps({'applied':False,'snapshot':str(snapshot),'candidate_sha256':CANDIDATE_SHA,'complete_outside_bytes_preserved':True,'freeze_sha256':digest(snapshot/'freeze.json')},indent=2))


if __name__=='__main__':main()
