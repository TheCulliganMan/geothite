#!/usr/bin/env python3
"""Apply only the exact Root-authorized R10 layout, with complete byte guards."""
import hashlib
import json
import os
from pathlib import Path
import re
import time

ROOT=Path(__file__).resolve().parents[3]
BEFORE='2db59b63a1184b5f0975c30f3ff8fec1bebbce2bf6cf18c81895410f3ff94e5a'
AFTER='293c2a1d4ba6e7141074bc6f5f26c83d754d8b42d6c4de0014cbec1c2c7317f3'
DOSSIER='0fd088086912e9a13ec64bae47750e856ac01d1faaedad34f80720b89a268ffa'
PACK='6071912eeb2fde6982f9e857bed9157d8d86da0f7ae56dba38665c809ab100e1'
REPORT='c30a43b8859dd9bd40beaf86533458c18fcef0219c90230ba60965675b7e4d25'


def sha(raw):return hashlib.sha256(raw).hexdigest()


def value_span(raw,key):
    matches=list(re.finditer(r'"'+re.escape(key)+r'"\s*:\s*',raw));assert len(matches)==1,(key,'one owned field required')
    start=matches[0].end();value,length=json.JSONDecoder().raw_decode(raw[start:]);return start,start+length,value


def outside_bytes(before,after):
    old=before.decode();new=after.decode();a,b,oldlayers=value_span(old,'layers');c,d,newlayers=value_span(new,'layers')
    assert newlayers[:-2]==oldlayers and len(newlayers)==len(oldlayers)+2
    oldarray=old[a:b];newarray=new[c:d];assert oldarray.endswith('\n  ]')
    assert newarray.startswith(oldarray[:-4]+',\n'),'every original layer byte must remain exact'
    new=new[:c]+oldarray+new[d:]
    a,b,oldalias=value_span(old,'compiled_tileset');c,d,newalias=value_span(new,'compiled_tileset');assert oldalias=='johto'and newalias=='mn_r10_rockstairs';new=new[:c]+old[a:b]+new[d:]
    c,d,recipe=value_span(new,'derived_blocks');assert recipe==[{'id':'80','source_tileset':'cave','source_block':'12','collision':'FLOOR/FLOOR/WALL/FLOOR','required_typed_extension':'r10_stairs_revision::revise_r10_rock_stairs'}]
    field=',\n  "derived_blocks": '+new[c:d];assert new.count(field)==1;new=new.replace(field,'',1)
    assert new.encode()==before,'bytes outside the alias value, two appended strokes and appended recipe changed'
    return dict(original_layer_prefix_bytes_exact=True,entire_outside_owned_bytes_exact=True,independent_normalized_file_sha256=sha(new.encode()),normalized_fields=['compiled_tileset value','two trailing layer objects','derived_blocks appended field'])


def main():
    target=ROOT/'planning/minnesota/routes/MN_R10.layout.json';dossier=target.with_name('MN_R10.json');folder=ROOT/'planning/minnesota/renders/iron-range-descent/rock-stairs-candidate/v01-right-lane';candidate=folder/'MN_R10.layout.candidate.json'
    before=target.read_bytes();after=candidate.read_bytes();dossier_before=dossier.read_bytes();assert [sha(before),sha(after),sha(dossier_before)]==[BEFORE,AFTER,DOSSIER],'live full-file preimage or candidate drift; reject/rebase'
    assert before==(folder/'baseline-MN_R10.layout.json').read_bytes();assert dossier_before==(folder/'baseline-MN_R10.json').read_bytes();bounded=outside_bytes(before,after)
    pack=ROOT/'content-packs/minnesota-northstar-rock-stairs-review.crystalpack';reportpath=ROOT/'planning/minnesota/renders/iron-range-descent/rock-stairs-native/6071912eeb2fde69/review-01/native-verification-and-captures.json';report=json.loads(reportpath.read_bytes());assert sha(pack.read_bytes())==PACK and sha(reportpath.read_bytes())==REPORT
    assert report['status']=='PASS'and len(report['checks'])==180 and all(v['passed']for v in report['checks'])and len(report['captures'])==5 and report['personally_visually_reviewed']and report['full_lcd_audit']['complete_lcd_pixels_compared']==115200 and report['full_lcd_audit']['pixel_mismatches']==0
    logs=[ROOT/'.local/r10-outdoor-stairs-assets-corrected-fixtures.log',ROOT/'.local/exterior-bends-final-iron_range_descent_controller-pocket.log'];assert '5 passed; 0 failed'in logs[0].read_text()and '4 passed; 0 failed'in logs[1].read_text();assert all('test result: FAILED'not in p.read_text()for p in logs)
    receiptpath=ROOT/'.local/exterior-bends-final-controller-pocket-receipts.json';controller=json.loads(receiptpath.read_bytes())['runs'][2];assert controller['exit_code']==0 and controller['pack_sha256']==PACK and controller['binary_sha256']=='250c34008027211e8e7f1b44c8f5620a57cf956a7f8280ba47168082faf5d9bb'
    stamp=str(time.time_ns());out=folder.parent/('promotion-'+stamp);out.mkdir();(out/'MN_R10.layout.json.before').write_bytes(before);(out/'MN_R10.layout.json.proposed').write_bytes(after);(out/'MN_R10.json.before').write_bytes(dossier_before)
    freeze=dict(authorization='Root explicitly authorized layout-only exactv01 source promotion after personally viewing whole/five actual native frames and actual5assets/4controllers PASS. No user final approval claim.',before_sha256=BEFORE,proposed_sha256=AFTER,dossier_sha256=DOSSIER,native_pack_sha256=PACK,native_report_sha256=REPORT,logs={str(p.relative_to(ROOT)):sha(p.read_bytes())for p in logs},controller_receipt_sha256=sha(receiptpath.read_bytes()),outside_byte_proof=bounded,applied=False)
    (out/'preflight.json').write_text(json.dumps(freeze,indent=2)+'\n');tmp=target.with_name('.'+target.name+'.rockstairs-'+stamp+'.tmp')
    try:
        with tmp.open('xb')as f:f.write(after);f.flush();os.fsync(f.fileno())
        os.chmod(tmp,target.stat().st_mode&0o777)
        assert target.read_bytes()==before and dossier.read_bytes()==dossier_before,'live source changed before atomic write'
        os.replace(tmp,target)
        actual=target.read_bytes();dossier_after=dossier.read_bytes();assert actual==after and dossier_after==dossier_before,'independent full readback differs'
        independent=outside_bytes(before,actual);assert independent==bounded
        (out/'MN_R10.layout.json.after').write_bytes(actual);(out/'MN_R10.json.after').write_bytes(dossier_after)
        freeze.update(applied=True,after_sha256=sha(actual),dossier_after_sha256=sha(dossier_after),independent_readback_outside_byte_proof=independent,native_pack_still_exact=sha(pack.read_bytes())==PACK)
        (out/'application-receipt.json').write_text(json.dumps(freeze,indent=2)+'\n');print(out);print('RECEIPT_SHA',sha((out/'application-receipt.json').read_bytes()))
    finally:tmp.unlink(missing_ok=True)


if __name__=='__main__':main()
