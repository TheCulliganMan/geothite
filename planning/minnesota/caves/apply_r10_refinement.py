#!/usr/bin/env python3
"""Apply only Root-approved R10 records, preserving unrelated rebased bytes."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import time

ROOT=Path(__file__).resolve().parents[3]
APPROVED=ROOT/'planning/minnesota/renders/caves/r10-refinement/promotion/preflight-1793073253959856000'
APPROVED_FREEZE_SHA='55db7b72a095634c398c568e2fc5b9f794a5cd63888a4b471062510b795c39cc'
APPROVED_BOUNDED_SHA='87bdf7453dac0c0a50ff033144ce37e120b2119698fd9344f38bdf82c2034eff'
spec=importlib.util.spec_from_file_location('preview',Path(__file__).with_name('promote_r10_refinement.py'));preview=importlib.util.module_from_spec(spec);spec.loader.exec_module(preview)
require=preview.require
digest=preview.digest
read=preview.read
ROOM=preview.ROOM


def span(raw,key):
    found=list(re.finditer(r'\{\s*"'+re.escape(key)+r'"\s*:\s*"'+ROOM+r'"',raw))
    require(len(found)==1,'one exact owned record required')
    start=found[0].start();value,length=json.JSONDecoder().raw_decode(raw[start:])
    return start,start+length,value


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--snapshot',type=Path,required=True);args=parser.parse_args();snapshot=args.snapshot.resolve()
    require(digest(APPROVED/'freeze.json')==APPROVED_FREEZE_SHA and digest(APPROVED/'bounded-record-proof.json')==APPROVED_BOUNDED_SHA,'Root-approved original review changed')
    freeze=read(snapshot/'freeze.json');require(not freeze['applied']and freeze['candidate_sha256']==preview.CANDIDATE_SHA and freeze['native_pack_sha256']==preview.PACK_SHA and freeze['native_report_sha256']==preview.PROOF_SHA,'exact accepted native/source epoch required')
    require(digest(ROOT/'planning/minnesota/caves/MnR10Cave1F.refinement.candidate.json')==preview.CANDIDATE_SHA,'source candidate drift')
    paths=[ROOT/'planning/minnesota/routes/MN_R10.json',ROOT/'planning/minnesota/interiors.json'];before=[];after=[];rebase=[]
    for path,key,record in zip(paths,['id','interior'],['room','contract']):
        current=path.read_bytes();proposed=(snapshot/(path.name+'.proposed')).read_bytes();original=(APPROVED/(path.name+'.before')).read_bytes()
        require(current==(snapshot/(path.name+'.before')).read_bytes()and hashlib.sha256(current).hexdigest()==freeze['shared_preimage_sha256'][str(path.relative_to(ROOT))],'current full file no longer equals fresh preimage; rebase again')
        require(hashlib.sha256(proposed).hexdigest()==freeze['proposed_complete_file_sha256'][str(path.relative_to(ROOT))],'proposed full file changed')
        for suffix in ['before','proposed']:require((snapshot/(record+'.'+suffix+'.json')).read_bytes()==(APPROVED/(record+'.'+suffix+'.json')).read_bytes(),'rebased owned '+record+' '+suffix+' bytes differ from approved record')
        start,end,value=span(current.decode(),key);a_start,a_end,a_value=span(original.decode(),key)
        require(current.decode()[start:end].encode()==original.decode()[a_start:a_end].encode()and value==a_value,'live owned source bytes differ from approved preimage')
        p_start,p_end,p_value=span(proposed.decode(),key);approved_after=(APPROVED/(path.name+'.proposed')).read_text();ap_start,ap_end,ap_value=span(approved_after,key)
        require(proposed.decode()[p_start:p_end]==approved_after[ap_start:ap_end]and p_value==ap_value,'proposed owned source bytes differ from approved proposal')
        rebuilt,outside=preview.helper.splice_record(current.decode(),key,value,p_value)
        require(rebuilt.encode()==proposed,'fresh proposal does not preserve current outside bytes')
        before.append(current);after.append(proposed);rebase.append({'file':str(path.relative_to(ROOT)),'approved_preimage_sha256':hashlib.sha256(original).hexdigest(),'fresh_preimage_sha256':hashlib.sha256(current).hexdigest(),'owned_before_bytes_identical_to_approved':True,'owned_proposed_bytes_identical_to_approved':True,**outside})
    require(all(p.read_bytes()==raw for p,raw in zip(paths,before)),'source changed while verifying rebase')
    application=snapshot/('application-'+str(time.time_ns()));application.mkdir()
    for path,old,new in zip(paths,before,after):(application/(path.name+'.before')).write_bytes(old);(application/(path.name+'.after')).write_bytes(new)
    preview.write_json(application/'rebase-proof.json',{'original_approved_snapshot':str(APPROVED.relative_to(ROOT)),'fresh_snapshot':str(snapshot.relative_to(ROOT)),'approved_freeze_sha256':APPROVED_FREEZE_SHA,'root_authorization':'Trusted Root task explicitly approved exact proposed R10 room/contract and permitted unrelated full-file rebase only with byte-identical owned records','records':rebase,'user_final_approval_claimed':False})
    staged=[];written=[]
    try:
        for path,payload in zip(paths,after):
            temporary=path.with_name('.'+path.name+'.r10-refinement-'+str(time.time_ns())+'.tmp')
            with temporary.open('xb')as stream:stream.write(payload);stream.flush();os.fsync(stream.fileno())
            staged.append(temporary)
        require(all(p.read_bytes()==raw for p,raw in zip(paths,before)),'full sources changed before replacement')
        for index,(temporary,path)in enumerate(zip(staged,paths)):
            require(path.read_bytes()==before[index],'source changed before its atomic replacement')
            os.replace(temporary,path);written.append(index)
    except BaseException:
        for index in reversed(written):
            require(paths[index].read_bytes()==after[index],'external edit prevents safe rollback; immutable before/after retained')
            temporary=paths[index].with_name('.'+paths[index].name+'.r10-refinement-rollback-'+str(time.time_ns())+'.tmp')
            temporary.write_bytes(before[index]);os.replace(temporary,paths[index])
        raise
    finally:
        for temporary in staged:temporary.unlink(missing_ok=True)
    readback=[]
    for path,key,old,new in zip(paths,['id','interior'],before,after):
        actual=path.read_bytes();require(actual==new,'independent complete file readback differs')
        before_text=old.decode();after_text=actual.decode();s,e,_=span(before_text,key);ps,pe,_=span(after_text,key)
        require(before_text[:s].encode()==after_text[:ps].encode()and before_text[e:].encode()==after_text[pe:].encode(),'independent outside-owned byte readback differs')
        readback.append({'file':str(path.relative_to(ROOT)),'sha256':hashlib.sha256(actual).hexdigest(),'full_readback_exact':True,'outside_owned_prefix_bytes_exact':True,'outside_owned_suffix_bytes_exact':True})
    preview.write_json(application/'application-receipt.json',{'status':'APPLIED Root-approved R10 source records after guarded unrelated-interiors rebase','candidate_source_sha256':preview.CANDIDATE_SHA,'native_pack_sha256':preview.PACK_SHA,'native_proof_sha256':preview.PROOF_SHA,'source_apply_script_sha256':digest(Path(__file__)),'readback':readback,'no_native_pack_writes':True,'user_final_approval_claimed':False})
    shutil.copy2(Path(__file__),application/'apply_r10_refinement.py.snapshot')
    print(json.dumps({'applied':True,'application':str(application),'receipt_sha256':digest(application/'application-receipt.json'),'readback':readback},indent=2))


if __name__=='__main__':main()
