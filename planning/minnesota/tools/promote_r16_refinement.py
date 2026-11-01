#!/usr/bin/env python3
"""Guarded R16 layout/ledge-contract source promotion. Freeze is read-only to live files."""
import argparse
from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import re
import time

ROOT=Path(__file__).resolve().parents[3]
NATIVE=ROOT/'planning/minnesota/renders/red-river-prairie/refinement/native/8826137c74842b68-review01'
SOURCE=ROOT/'planning/minnesota/renders/red-river-prairie/refinement/314d972a78bcf479'
FILES=[ROOT/'planning/minnesota/routes/MN_R16.json',ROOT/'planning/minnesota/routes/MN_R16.layout.json']
BEFORE=['6750bec82241d5205d34ae06aa704ef18fc0570a308e9151f33cddf0e434aa0a','6301787fb6db5d929d39092751d4587eff8f4a406af8a32eaaae7789ded970fd']
PACK_SHA='8826137c74842b68a184a957a14d15cdb3f9ed3bdaca349a046924a049a20a41'
CONTROLLER_SHA='6071912eeb2fde6982f9e857bed9157d8d86da0f7ae56dba38665c809ab100e1'
CANDIDATE=['211812ad52f7d11c863eda2ba48b03bf77821faffad190f7de1c734d3036cb02','cf9cb707c3f416064488b86acb2f7fc8c4f1cce575bd2d40e0ce748991a233b5']

def sha(raw):return hashlib.sha256(raw).hexdigest()
def read(p):return json.loads(p.read_bytes())
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
def require(v,message):
 if not v:raise ValueError(message)

def value_span(raw,key):
 text=raw.decode();matches=list(re.finditer(r'"'+re.escape(key)+r'"\s*:\s*',text));require(len(matches)==1,'exactly one key required: '+key);start=matches[0].end();value,size=json.JSONDecoder().raw_decode(text[start:]);return value,len(text[:start].encode()),len(text[:start+size].encode())

def splice_value(raw,key,value):
 old,start,end=value_span(raw,key);line=raw[:start].rfind(b'\n')+1;indent=raw[line:start].split(b'"',1)[0].decode();replacement=json.dumps(value,indent=2).replace('\n','\n'+indent).encode();result=raw[:start]+replacement+raw[end:]
 require(result[:start]==raw[:start]and result[start+len(replacement):]==raw[end:],'outside-key bytes changed')
 return result,{'key':key,'start':start,'old_end':end,'new_end':start+len(replacement),'old_value_sha256':sha(raw[start:end]),'prefix_sha256':sha(raw[:start]),'suffix_sha256':sha(raw[end:])}


def evidence():
 paths=[NATIVE/'native-verification-and-captures.json',NATIVE/'controller-pack-alignment.json',NATIVE/'visual-review.json',SOURCE/'review.json',ROOT/'.local/exterior-bends-final-controller-pocket-receipts.json',ROOT/'.local/exterior-bends-final-red_river_controller-pocket.log']
 proof=read(paths[0]);require(sha(paths[0].read_bytes())=='5a65443a0ff06d2b9e0f0c2e881fa92d79e4b2e10ba58a088e7ca8cd8ea198c6'and proof['status']=='PASS'and len(proof['checks'])==188 and all(c['passed']for c in proof['checks'])and len(proof['captures'])==7 and proof['full_lcd_pixels_compared']==161280 and all(c['full_lcd_mismatches']==0 for c in proof['captures']),'exact188/seven native proof required')
 alignment=read(paths[1]);require(alignment['status']=='PASS'and alignment['native_gpu_pack_sha256']==PACK_SHA and alignment['controller_pack_sha256']==CONTROLLER_SHA and all(c['passed']for c in alignment['checks']),'actual controller pack must preserve exact R16 native module/art/programs')
 require(sha(paths[4].read_bytes())==alignment['controller_receipt_sha256']and sha(paths[5].read_bytes())==alignment['controller_log_sha256'],'exact passing execution receipts required')
 require('test result: ok. 2 passed'in paths[5].read_text()and all('::'+case+' ... ok'in paths[5].read_text()for case in alignment['cases']),'both real production cases required')
 require(sha(paths[3].read_bytes())=='d4ceed1653276505c496b0a22fabbcbcfb869794096a22bb64114871c17b2cea','frozen33+9 source proof changed')
 for path,expected in [(ROOT/'content-packs/minnesota-northstar-prairie-bend-review.crystalpack',PACK_SHA),(ROOT/'content-packs/minnesota-northstar-rock-stairs-review.crystalpack',CONTROLLER_SHA)]:require(sha(path.read_bytes())==expected,'native evidence pack changed')
 return {str(p.relative_to(ROOT)):sha(p.read_bytes())for p in paths}


def propose(raws,candidates):
 old_d,old_l=[json.loads(v)for v in raws];new_d,new_l=map(deepcopy,candidates)
 normalized=deepcopy(new_d);normalized.pop('review_contract',None);normalized['one_way_ledges'][0]['bounds_core']=old_d['one_way_ledges'][0]['bounds_core'];normalized['one_way_ledges'][0]['side_barriers']=old_d['one_way_ledges'][0]['side_barriers'];require(normalized==old_d,'dossier changed outside approved ledge fields/review annotations')
 require(all(new_l.get(k)==old_l.get(k)for k in set(new_l)|set(old_l)if k!='layers')and new_l['layers'][:len(old_l['layers'])]==old_l['layers'],'layout changed original layers/art/identity')
 new_d['review_contract']['status']='Installed native review candidate; both real local production-controller cases PASS on composed6071. User final approval and earned regional progression remain unclaimed.'
 new_d['review_contract']['native_review']={'pack_sha256':PACK_SHA,'controller_pack_sha256':CONTROLLER_SHA,'preservation_checks':188,'gpu_captures':7,'full_lcd_exact_pixels':161280,'native_source_report_sha256':'d4ceed1653276505c496b0a22fabbcbcfb869794096a22bb64114871c17b2cea','controller_cases':read(NATIVE/'controller-pack-alignment.json')['cases'],'return_contract':'Left52 ordinary return57 steps; original right53 return59 steps; both native hops/reverses and at least6grass after landing remain exact.','item_notice_limit':'Production settling internally prints automatic FoundText/fanfare; completed PocketText and actual quantities/flags/input ownership are the observed native evidence.'}
 changed,ledge=splice_value(raws[0],'one_way_ledges',new_d['one_way_ledges'])
 # Append only the new editorial key. Preserve original source bytes outside
 # the explicit ledge value and this insertion, including trailing whitespace.
 insertion=len(changed.rstrip())-1;require(changed[insertion:insertion+1]==b'}','top-level source closing brace required');at=insertion
 while at>0 and changed[at-1:at].isspace():at-=1
 annotation=b',\n  "review_contract": '+json.dumps(new_d['review_contract'],indent=2).replace('\n','\n  ').encode();dossier=changed[:at]+annotation+changed[at:]
 require(json.loads(dossier)==new_d,'proposed dossier semantic mismatch')
 # Append only explicit new layer strokes before the original closing array.
 _,start,end=value_span(raws[1],'layers');pos=end-1
 while pos>start and raws[1][pos-1:pos].isspace():pos-=1
 extra=new_l['layers'][len(old_l['layers']):];require(extra,'new terrain strokes required');insert=b',\n'+',\n'.join('\n'.join('    '+line for line in json.dumps(layer,indent=2).splitlines())for layer in extra).encode();layout=raws[1][:pos]+insert+raws[1][pos:]
 require(json.loads(layout)==new_l,'proposed layout semantic mismatch')
 # Independent exact reversal of the only permitted lexical edits.
 unannotated=dossier[:at]+dossier[at+len(annotation):];restored,_=splice_value(unannotated,'one_way_ledges',old_d['one_way_ledges']);require(restored==raws[0],'dossier outside-approved bytes differ')
 require(layout[:pos]+layout[pos+len(insert):]==raws[1],'layout original layer bytes/outer metadata differ')
 return [dossier,layout],{'dossier':{'ledge_value':ledge,'annotation_insert_byte':at,'annotation_bytes':len(annotation),'outside_allowed_edits_exact':True},'layout':{'new_layer_insert_byte':pos,'new_layer_bytes':len(insert),'every_original_byte_preserved_in_order':True}}


def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--snapshot',type=Path,required=True);parser.add_argument('--freeze',action='store_true');parser.add_argument('--apply',action='store_true');args=parser.parse_args();require(not(args.freeze and args.apply),'freeze and apply are distinct')
 paths=[SOURCE/'MN_R16.refinement.candidate.json',SOURCE/'MN_R16.refinement.layout.json'];require([sha(p.read_bytes())for p in paths]==CANDIDATE,'frozen candidate source bytes changed');candidates=[read(p)for p in paths];ev=evidence();raws=[p.read_bytes()for p in FILES];require([sha(v)for v in raws]==BEFORE,'complete live R16 source differs from reviewed original; do not overwrite')
 after,outside=propose(raws,candidates);snapshot=args.snapshot.resolve()
 if args.freeze:
  snapshot.mkdir(parents=True,exist_ok=False)
  for p,raw,result in zip(FILES,raws,after):(snapshot/(p.name+'.before')).write_bytes(raw);(snapshot/(p.name+'.proposed')).write_bytes(result)
  for p in [*paths,Path(__file__)]: (snapshot/p.name).write_bytes(p.read_bytes())
  for name in ev:
   target=snapshot/'evidence'/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((ROOT/name).read_bytes())
  write(snapshot/'freeze.json',{'source_files':{str(p.relative_to(ROOT)):sha(raw)for p,raw in zip(FILES,raws)},'proposed_files':{str(p.relative_to(ROOT)):sha(raw)for p,raw in zip(FILES,after)},'evidence_sha256':ev,'script_sha256':sha(Path(__file__).read_bytes()),'native_pack_sha256':PACK_SHA,'controller_pack_sha256':CONTROLLER_SHA,'status':'PREFLIGHT ONLY; apply requires separate explicit Root authorization'})
  write(snapshot/'outside-owned-bytes.proposed.json',outside)
 freeze=read(snapshot/'freeze.json');require(freeze['evidence_sha256']==ev and freeze['script_sha256']==sha(Path(__file__).read_bytes()),'frozen evidence/script changed')
 for p,raw,result in zip(FILES,raws,after):require(sha(raw)==freeze['source_files'][str(p.relative_to(ROOT))]and result==(snapshot/(p.name+'.proposed')).read_bytes(),'reviewed source/proposal drift')
 summary={'applied':False,'snapshot':str(snapshot),'before':freeze['source_files'],'proposed':freeze['proposed_files'],'native_pack_sha256':PACK_SHA,'controller_pack_sha256':CONTROLLER_SHA}
 if args.apply:
  application=snapshot/('application-'+str(time.time_ns()));application.mkdir();staged=[];written=[]
  for p,raw,result in zip(FILES,raws,after):(application/(p.name+'.before')).write_bytes(raw);(application/(p.name+'.after')).write_bytes(result)
  try:
   for p,result in zip(FILES,after):
    temp=p.with_name('.'+p.name+'.r16-promotion.tmp');require(not temp.exists(),'existing staging file');temp.write_bytes(result);staged.append(temp)
   require(all(p.read_bytes()==raw for p,raw in zip(FILES,raws)),'concurrent writer changed source')
   for temp,p in zip(staged,FILES):os.replace(temp,p);written.append(p)
  except BaseException:
   for p,raw,result in zip(FILES,raws,after):
    if p in written:
     require(p.read_bytes()==result,'concurrent writer blocks rollback; snapshots retained');temp=p.with_name('.'+p.name+'.r16-rollback.tmp');require(not temp.exists(),'existing rollback staging file');temp.write_bytes(raw);os.replace(temp,p)
   raise
  finally:
   for temp in staged:temp.unlink(missing_ok=True)
  require(all(p.read_bytes()==raw for p,raw in zip(FILES,after)),'full applied byte readback differs');_,independent=propose(raws,candidates);require(independent==outside,'independent outside-byte proof differs');write(application/'outside-owned-bytes.independent.json',independent)
  summary.update(applied=True,application=str(application),after={str(p.relative_to(ROOT)):sha(p.read_bytes())for p in FILES});write(application/'promotion.json',summary)
  for p in [snapshot/'freeze.json',Path(__file__)]: (application/p.name).write_bytes(p.read_bytes())
 write(snapshot/('application-summary.json'if args.apply else'preflight.json'),summary);print(json.dumps(summary,indent=2))


if __name__=='__main__':main()
