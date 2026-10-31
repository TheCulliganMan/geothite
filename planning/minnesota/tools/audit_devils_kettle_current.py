#!/usr/bin/env python3
"""Read-only composed falls/clinic audit; virtual intermediate is never exported."""
import argparse
from copy import deepcopy
import json
from audit_devils_kettle import audit as falls_audit
from audit_devils_kettle_clinic import audit as clinic_audit, CONSTANT, MANIFEST, NURSE, ROOM
from render_reference import load_pack


def check(parent,child,site):
    added=MANIFEST not in parent['report']['manifests'] and MANIFEST in child['report']['manifests']
    intermediate=child
    clinic=None
    if added:
        intermediate=dict(child,data=dict(child['data']),report=deepcopy(child['report']))
        intermediate['report']['manifests'].remove(MANIFEST)
        data=intermediate['data'];old=parent['data']
        data['maps']=dict(data['maps']);data['maps'][ROOM]=old['maps'][ROOM]
        data['map_scripts']=dict(data['map_scripts']);data['map_scripts'][NURSE]=old['map_scripts'][NURSE]
        additions=data['runtime_spawn_points'].keys()-old['runtime_spawn_points'].keys()
        assert len(additions)==1
        key=next(iter(additions));assert data['runtime_spawn_points'][key]['mapConstant']==CONSTANT
        data['runtime_spawn_points']=dict(data['runtime_spawn_points']);del data['runtime_spawn_points'][key]
        clinic=clinic_audit(intermediate,child)
    falls=falls_audit(parent,intermediate,site)
    return dict(falls=falls,clinic=clinic,actual_untouched_maps=len(parent['data']['maps'])-(2 if added else 1),
                virtual_intermediate_exported=False,gameplay_acceptance=False)


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('parent');parser.add_argument('child');parser.add_argument('site');args=parser.parse_args()
    parent,child=load_pack(args.parent),load_pack(args.child)
    with open(args.site) as stream:site=json.load(stream)
    report=check(parent,child,site);rejected=[]
    for label in ['unrelated-map','native-art','native-habitat','wrong-clinic-recovery']:
        bad=dict(child,data=dict(child['data']))
        if label=='unrelated-map':
            bad['data']['maps']=dict(child['data']['maps']);room=deepcopy(child['data']['maps']['NewBarkTown']);room['blocks'][0]^=1;bad['data']['maps']['NewBarkTown']=room
        elif label=='native-art':
            bad['runtime_files']=dict(child['runtime_files']);key=next(iter(bad['runtime_files']));bad['runtime_files'][key]=b'unreviewed native art'
        elif label=='native-habitat':
            bad['data']['wild_encounters']=dict(child['data']['wild_encounters']);bad['data']['wild_encounters']['AUDIT_FOREIGN']={}
        else:
            bad['data']['runtime_spawn_points']=dict(child['data']['runtime_spawn_points'])
            key=next(key for key,value in child['data']['runtime_spawn_points'].items() if value['mapConstant']==CONSTANT)
            spawn=dict(child['data']['runtime_spawn_points'][key]);spawn['tileY']+=1;bad['data']['runtime_spawn_points'][key]=spawn
        try:check(parent,bad,site)
        except (AssertionError,KeyError,TypeError):rejected.append(label)
        else:raise AssertionError('accepted corruption: '+label)
    print(json.dumps(dict(report,negative_cases_rejected=rejected),indent=2))


if __name__=='__main__':main()
