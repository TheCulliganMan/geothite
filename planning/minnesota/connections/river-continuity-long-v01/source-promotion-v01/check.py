#!/usr/bin/env python3
"""Read-only canonical-source promotion check. There is deliberately no apply mode."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[5]
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack,layout_blocks


def sha(raw):return hashlib.sha256(raw).hexdigest()

def metadata(before,after,allowed):
    a=deepcopy(before);b=deepcopy(after)
    a.pop('layers');b.pop('layers')
    for key in allowed:a.pop(key);b.pop(key)
    assert a==b,'Unrelated source metadata changed'
    if allowed:
        assert allowed==['shore_fitting']
        assert before['id']=='MN_R05'and before['shore_fitting']=='johto_8_neighbors'
        assert after['shore_fitting']=='explicit_source_edges'


def native_blocks(actual,expected,label):
    assert actual==expected,('Candidate source does not reproduce native terrain',label)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-river-long-v02.crystalpack')
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    assert args.output.resolve().is_relative_to(ROOT/'planning/minnesota/renders/river-continuity-long-v01')
    assert not args.output.exists(),'Retain every promotion-check epoch.'
    proposal=json.loads((HERE/'promotion.json').read_text())
    assert sha(args.pack.read_bytes())==proposal['accepted_native_pack_sha256']
    pack=load_pack(args.pack)
    checked=[];all_chunks=[]
    for row in proposal['files']:
        canonical=ROOT/row['source'];raw=canonical.read_bytes()
        assert sha(raw)==row['source_sha256'],'Canonical source changed since proposal'
        assert raw==(HERE/row['before_file']).read_bytes(),'Frozen before bytes differ'
        target=ROOT/row['canonical_target']
        if row['expected_target_sha256']is None:
            assert not target.exists(),('New current-source target appeared',str(target))
        else:
            assert sha(target.read_bytes())==row['expected_target_sha256'],'Target source preimage drift'
        proposed=HERE/row['candidate_file'];assert sha(proposed.read_bytes())==row['candidate_sha256']
        before=json.loads(raw);after=json.loads(proposed.read_text())
        metadata(before,after,row['allowed_metadata_changes'])
        assert after['dimensions_metatiles']==row['dimensions_metatiles']
        assert after['shore_fitting']=='explicit_source_edges'
        assert all(l['kind']=='block_rect'for l in after['layers'])
        width,height=after['dimensions_metatiles'];grid=layout_blocks(after);old=layout_blocks(before)
        actual_changes=[dict(metatile=[i%width,i//width],before=f'{a:02x}',after=f'{b:02x}')for i,(a,b)in enumerate(zip(old,grid))if a!=b]
        assert actual_changes==row['changes']and len(actual_changes)==row['changed_metatiles']
        for chunk in row['chunks']:
            x0,y0=chunk['source_origin_metatiles'];w,h=chunk['dimensions_metatiles'];name=chunk['id'];m=pack['data']['maps'][name]
            clipped=[grid[(y+y0)*width+x+x0]for y in range(h)for x in range(w)]
            assert [m['attributes']['width'],m['attributes']['height']]==[w,h]
            native_blocks(clipped,m['blocks'],name)
            all_chunks.append(dict(map=name,dimensions_metatiles=[w,h],full_source_stride=width,source_origin_metatiles=[x0,y0],blocks_sha256=sha(bytes(clipped))))
        checked.append(dict(target=row['canonical_target'],source_sha256=row['source_sha256'],candidate_sha256=row['candidate_sha256'],changed_metatiles=row['changed_metatiles'],unrelated_source_metadata_exact=True))
    for guard in proposal['guards']:
        assert sha((ROOT/guard['path']).read_bytes())==guard['sha256'],('Story/world/interior/historical source guard drift',guard['path'])
    inventory=HERE/proposal['current_inventory_candidate']
    assert sha(inventory.read_bytes())==proposal['current_inventory_sha256']
    assert not(ROOT/proposal['current_inventory_target']).exists()
    r05=json.loads((HERE/'candidate/MN_R05.layout.json').read_text());full=layout_blocks(r05)
    alternative=json.loads((HERE/'alternatives/MN_R05.before-native-bridge-art.layout.json').read_text());metadata(r05,alternative,[]);base=layout_blocks(alternative)
    for cell in proposal['r05_existing_derived_bridge_substitutions']:
        x,y=cell['metatile'];index=y*32+x
        assert base[index]==int(cell['base_source'],16)
        assert full[index]==int(cell['installed_native'],16)
        base[index]=int(cell['installed_native'],16)
    assert base==full,'Base-art alternative differs beyond three existing bridge substitutions'
    assert len(checked)==3 and len(all_chunks)==15
    assert sum(r['changed_metatiles']for r in checked)==1699
    assert len(proposal['r05_older_approved_source_catchup'])==21
    assert next(r for r in all_chunks if r['map']=='MnR05C00R02')['dimensions_metatiles']==[32,8]
    assert next(r for r in all_chunks if r['map']=='MnMississippiLowerWoodsC01R02')['dimensions_metatiles']==[32,20]
    # Actual semantic counterexamples exercise source metadata and stride guards.
    rejected=[]
    bad=deepcopy(r05);bad['compiled_tileset']='johto'
    try:metadata(r05,bad,[])
    except AssertionError:rejected.append('Unrelated compiled tileset edit')
    else:raise AssertionError('Metadata corruption accepted')
    lower=json.loads((HERE/'candidate/MnMississippiLowerWoods.layout.json').read_text());g=layout_blocks(lower)
    wrong_stride=[g[(y+48)*32+x+32]for y in range(20)for x in range(32)]
    try:native_blocks(wrong_stride,pack['data']['maps']['MnMississippiLowerWoodsC01R02']['blocks'],'bad lower stride')
    except AssertionError:rejected.append('Lower east chunk incorrectly clipped with 32 instead of 64 stride')
    else:raise AssertionError('Wrong source stride accepted')
    bad=full.copy();bad[-1]^=1
    try:native_blocks(bad,full,'bad cropped R05 row')
    except AssertionError:rejected.append('Final cropped R05 row terrain corruption')
    else:raise AssertionError('Final cropped row corruption accepted')
    args.output.mkdir(parents=True)
    result=dict(status='GUARDED_SOURCE_PROMOTION_GEOMETRY_PASS_NO_APPLY',proposal_sha256=sha((HERE/'promotion.json').read_bytes()),accepted_native_pack_sha256=sha(args.pack.read_bytes()),files=checked,native_chunks=all_chunks,canonical_source_changes=1699,current_native_revision_changes=1678,older_approved_source_catchup=21,world_story_interiors_historical_inputs_exact=True,historical_manifest_not_modified=True,metadata_and_stride_counterexamples_rejected=rejected,current_inventory_candidate_sha256=sha(inventory.read_bytes()),no_canonical_write_performed=True,required_rust_profile_change_not_applied=proposal['required_shared_rust_change'],rebuild_certified=False,rebuild_limit='This checks current-source promotion geometry, exact preimages and existing installed-art capability. It does not execute a fresh historical or full cold rebuild. Preserve frozen historical inputs; consume current sources through an explicitly reviewed current overlay/build stage.',checker_sha256=sha(Path(__file__).read_bytes()))
    raw=(json.dumps(result,indent=2)+'\n').encode();h=sha(raw);(args.output/'receipt.json').write_bytes(raw);(args.output/f'receipt-{h}.json').write_bytes(raw)
    print(json.dumps(dict(status=result['status'],proposal_sha256=result['proposal_sha256'],receipt_sha256=h,native_chunks=len(all_chunks),canonical_source_changes=1699),indent=2))


if __name__=='__main__':main()
