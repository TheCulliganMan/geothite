#!/usr/bin/env python3
"""Promote four reviewed route sources after native/controller proof, with guards."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BATCH = ROOT / 'renders/route-quality-next'
PACK_HASH = '1186a24371251f741db88cf35a7863145ed5423da4ad3d7c6485c8f242753ecb'
SPECS = [
    ('04','r04/v04/before.layout.json','r04/v04/after.layout.json','r04/v02/dossier.json','r04/v04/dossier.json',{'description','main_path_metatiles','viewpoints','status'}),
    ('08','r08/v05/MN_R08.previous.layout.json','r08/v05/MN_R08.candidate.layout.json','r08/v05/MN_R08.previous.json','r08/v05/MN_R08.candidate.json',{'one_way_ledges'}),
    ('18','r18/v03/before-MN_R18.layout.json','r18/v03/MN_R18.layout.candidate.json','r18/v03/before-MN_R18.json','r18/v03/MN_R18.candidate.json',{'surf_viewpoints','main_path_metatiles','review_contract','loops','viewpoints'}),
    ('19','r19/v07-pine-habitat-watch/baseline-MN_R19.layout.json','r19/v07-pine-habitat-watch/MN_R19.layout.candidate.json','r19/v07-pine-habitat-watch/baseline-MN_R19.json','r19/v07-pine-habitat-watch/MN_R19.candidate.json',{'main_path_metatiles','one_way_ledges','viewpoints'}),
]


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--controller-proof',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--apply',action='store_true')
    args=parser.parse_args(); proof=read(args.controller_proof)
    assert proof['status']=='PASS' and proof['pack_sha256']==PACK_HASH
    assert len(proof['cases'])==20 and all(case['passed'] for case in proof['cases'])
    captures=[read(BATCH/name/'capture-progress.json') for name in ['native-r04-v02','native-r08-v01','native-r18-v01','native-r19-v01']]
    assert sum(len(value['captures']) for value in captures)==38
    assert all(value['status']=='PASS' and value['pack_sha256']==PACK_HASH and all(row['pixel_mismatches']==0 for row in value['captures']) for value in captures)
    changes=[]
    for route,old_layout,new_layout,old_dossier,new_dossier,allowed in SPECS:
        old,new=read(BATCH/old_layout),read(BATCH/new_layout)
        assert new['layers'][:len(old['layers'])]==old['layers']
        assert {key:val for key,val in old.items() if key!='layers'}=={key:val for key,val in new.items() if key!='layers'}
        prior,after=read(BATCH/old_dossier),read(BATCH/new_dossier)
        delta={key for key in set(prior)|set(after) if prior.get(key)!=after.get(key)}
        assert delta<=allowed, (route,delta-allowed)
        after['terrain_revision']=dict(id='route-quality-next-v01',pack_sha256=PACK_HASH,
             native_publication_inputs='renders/route-quality-next/integration-inputs',
             controller_proof=str(args.controller_proof.resolve().relative_to(ROOT)),
             runbook='playable/route-quality-next.md',scope='Reviewed bounded terrain only; no full-route final acceptance or earned regional progression claim.')
        for kind,baseline,candidate,source in [('layout',old,new,new_layout),('dossier',prior,after,new_dossier)]:
            path=ROOT/f'routes/MN_R{route}{".layout" if kind=="layout" else ""}.json'
            before=path.read_bytes();assert json.loads(before)==baseline, str(path)+': canonical source drift'
            raw=(json.dumps(candidate,indent=2)+'\n').encode()
            changes.append(dict(path=path,before=before,after=raw,source=source,route=route,kind=kind))
    args.output.mkdir(parents=True,exist_ok=False)
    receipt=dict(mode='apply' if args.apply else 'dry-run',pack_sha256=PACK_HASH,
                 controller_proof_sha256=sha(args.controller_proof.read_bytes()),files=[])
    for change in changes:
        path=change['path']; relative=path.relative_to(ROOT)
        for stage in ['before','after']:
            target=args.output/stage/relative;target.parent.mkdir(parents=True,exist_ok=True)
            target.write_bytes(change[stage])
        receipt['files'].append(dict(path=str(relative),before_sha256=sha(change['before']),after_sha256=sha(change['after']),frozen_candidate=change['source']))
    assert all(change['path'].read_bytes()==change['before'] for change in changes), 'source changed during preflight'
    if args.apply:
        for change in changes:
            change['path'].write_bytes(change['after'])
        assert all(change['path'].read_bytes()==change['after'] for change in changes)
    raw=(json.dumps(receipt,indent=2)+'\n').encode()
    output=args.output/('receipt-'+sha(raw)+'.json');output.write_bytes(raw)
    print(str(output))


if __name__=='__main__':
    main()
