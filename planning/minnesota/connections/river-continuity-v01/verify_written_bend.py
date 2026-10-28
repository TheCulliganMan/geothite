#!/usr/bin/env python3
"""Independently audit a Rust-written first-bend pack against frozen authoring.

No pack publication, source extraction, catalogue export or gameplay mutation.
Only exact bounded terrain and the declared sign event coordinate may differ.
"""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from audit_city_expansions import module_hash
from audit_winding_shelter_541 import raw_identity
from render_reference import layout_blocks,load_pack

PLAN=Path(__file__).resolve().parent/'source-v01'
CONTRACT_SHA='32757c9db294ce6ae58bacfda7107d807b4ecc77a2123c07e5d09efae854bb89'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def expected_data(before,contract):
    source=before['data']
    expected=dict(source)
    expected['maps']=dict(source['maps'])
    expected['map_blocks']=dict(source['map_blocks'])
    expected['map_scripts']=dict(source['map_scripts'])
    for row in contract['changes']:
        name=row['map'];old=source['maps'][name]
        assert module_hash(old)==row['module_sha256'],('stale module',name)
        module=deepcopy(old)
        width=old['attributes']['width']
        layout=json.loads((PLAN/f'{name}.candidate.layout.json').read_bytes())
        before_layout=json.loads((PLAN/f'{name}.before.layout.json').read_bytes())
        assert layout_blocks(before_layout)==old['blocks']
        cells=set()
        for change in row['cells']:
            x,y=change['metatile'];assert (x,y)not in cells;cells.add((x,y))
            index=y*width+x
            assert module['blocks'][index]==int(change['before'],16)
            module['blocks'][index]=int(change['after'],16)
        assert layout_blocks(layout)==module['blocks']
        expected['map_blocks'][old['attributes']['blocks_label']]=base64.b64encode(bytes(module['blocks'])).decode()
        for event in row['event_changes']:
            assert name=='MnR03C00R00' and event['group']=='bg_events'
            assert event['script']=='MN_R03_NORTH_SIGN' and event['before_core']==[22,4] and event['after_core']==[22,8]
            matches=[e for e in module['events']['bg_events']if e['script']==event['script']]
            assert len(matches)==1 and [matches[0]['x'],matches[0]['y']]==[22,4]
            matches[0]['y']=8
            section=old['attributes']['map_events_label']
            commands=deepcopy(source['map_scripts'][section])
            assert commands[6]==dict(command='bg_event',args=['22','4','BGEVENT_READ','MN_R03_NORTH_SIGN'])
            commands[6]['args'][1]='8'
            expected['map_scripts'][section]=commands
            assert module['map_event_section_commands'][6]==dict(command='bg_event',args=['22','4','BGEVENT_READ','MN_R03_NORTH_SIGN'],command_index=6)
            module['map_event_section_commands'][6]['args'][1]='8'
        expected['maps'][name]=module
    return expected


def audit(before,after,expected):
    checks=[]
    def check(label,passed):
        checks.append(dict(check=label,passed=bool(passed)))
    a,b=before['data'],after['data']
    own={'MnR03C00R00','MnMississippiConfluenceC00R00'}
    check('683-map roster exact',len(a['maps'])==len(b['maps'])==683 and set(a['maps'])==set(b['maps']))
    for name,module in a['maps'].items():
        check(('reviewed full module ' if name in own else 'unrelated full module ')+name,b['maps'].get(name)==expected['maps'][name])
    for key,value in expected.items():
        check('complete GameData field '+key,b.get(key)==value)
    check('ENTIRE GameData equals exact bounded expectation',b==expected)
    for name in own:
        old,actual=a['maps'][name],b['maps'][name]
        expected_cells=41 if name=='MnR03C00R00' else 10
        check(name+' exact cell count',sum(x!=y for x,y in zip(old['blocks'],actual['blocks']))==expected_cells)
        check(name+' exact encoded block mirror',list(base64.b64decode(b['map_blocks'][actual['attributes']['blocks_label']],validate=True))==actual['blocks'])
    check('all runtime files exact',after['runtime_files']==before['runtime_files'])
    check('all audio manifest hashes counts and loops exact',after['audio_manifest']==before['audio_manifest'])
    check('entire report exact',after['report']==before['report'])
    check('pack format exact',after['format_version']==before['format_version'])
    expected_identity=dict(before['identity']);expected_identity['content_hash']=after['identity']['content_hash']
    check('all identity metadata unchanged except derived content hash',after['identity']==expected_identity)
    check('content hash changed',after['identity']['content_hash']!=before['identity']['content_hash'])
    check('all outer fields accounted',set(after)==set(before))
    return checks


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--before',type=Path,required=True)
    ap.add_argument('--after',type=Path,required=True)
    ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args()
    assert not args.output.exists(),'Choose a fresh proof epoch.'
    args.output.mkdir(parents=True)
    raw=(PLAN/'candidate.contract.json').read_bytes();assert sha(raw)==CONTRACT_SHA
    contract=json.loads(raw)
    assert sha(args.before.read_bytes())==contract['parent_pack_sha256']
    receipt=json.loads((PLAN/'freeze-receipt.json').read_bytes())
    for name,digest in receipt['files'].items():
        assert sha((PLAN/name).read_bytes())==digest,('frozen source changed',name)
    before=load_pack(args.before);after=load_pack(args.after)
    assert before['identity']['content_hash']==contract['parent_identity']
    expected=expected_data(before,contract)
    checks=audit(before,after,expected)
    identities={label:raw_identity(path)for label,path in [('before',args.before),('after',args.after)]}
    checks.extend(dict(check='independent exact stored CBOR identity '+label,passed=value['exact'])for label,value in identities.items())
    # Corruptions touch exactly one field on fresh shallow branches, then run
    # the independent whole residual comparison. No candidate pack is written.
    negatives=[]
    def rejects(label,broken):
        failed=[c['check']for c in audit(before,broken,expected)if not c['passed']]
        negatives.append(dict(case=label,rejected=bool(failed),failed_checks=failed))
    for label,name in [('unrelated map terrain','MnWinona'),('undeclared R03 terrain','MnR03C00R00')]:
        broken=dict(after);broken['data']=dict(after['data']);broken['data']['maps']=dict(after['data']['maps'])
        broken['data']['maps'][name]=deepcopy(after['data']['maps'][name]);broken['data']['maps'][name]['blocks'][0]^=1
        rejects(label,broken)
    broken=dict(after);broken['data']=dict(after['data']);broken['data']['map_scripts']=dict(after['data']['map_scripts'])
    key='MnR03C00R00_MapEvents';broken['data']['map_scripts'][key]=deepcopy(before['data']['map_scripts'][key]);rejects('forgot raw sign event relocation',broken)
    broken=dict(after);broken['data']=dict(after['data']);broken['data']['maps']=dict(after['data']['maps'])
    key='MnR03C00R00';broken['data']['maps'][key]=deepcopy(after['data']['maps'][key]);broken['data']['maps'][key]['map_event_section_commands'][6]['args'][1]='4';rejects('forgot cached indexed sign event',broken)
    broken=dict(after);broken['report']=dict(after['report']);broken['report']['undeclared_continuity_change']=True;rejects('unrelated report delta',broken)
    broken=dict(after);broken['runtime_files']=dict(after['runtime_files']);broken['runtime_files']['undeclared-continuity-file']=b'x';rejects('undeclared runtime file',broken)
    broken=dict(after);broken['audio_manifest']=deepcopy(after['audio_manifest']);music=broken['audio_manifest']['music'];music[sorted(music)[0]]['pcm_frame_count']+=1;rejects('unrelated PCM descriptor count',broken)
    broken=dict(after);broken['identity']=dict(after['identity']);broken['identity']['maps']+=1;rejects('stale identity map count',broken)
    result=dict(schema_version=1,status='PASS' if all(c['passed']for c in checks)and all(c['rejected']for c in negatives)else 'FAIL',
        before_pack=str(args.before),before_sha256=sha(args.before.read_bytes()),after_pack=str(args.after),after_sha256=sha(args.after.read_bytes()),
        source_contract_sha256=CONTRACT_SHA,checks=checks,negative_cases=negatives,identities=identities,
        gameplay='This is an independent written-pack preservation proof. Native input and visual captures remain separate evidence.')
    path=args.output/'proof.json';path.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(status=result['status'],checks=len(checks),negative_cases=len(negatives),proof=str(path))))
    if result['status']!='PASS':
        raise SystemExit(1)


if __name__=='__main__':
    main()
