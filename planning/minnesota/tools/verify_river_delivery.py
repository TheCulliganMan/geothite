#!/usr/bin/env python3
"""Read-only exact residual and warm-coast audit of the river delivery."""
import argparse
import copy
import hashlib
import json
from pathlib import Path

from audit_outdoor_connections import collision
from render_reference import load_pack
from river_content_lineage import validate


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fruit',type=Path,required=True)
    parser.add_argument('--banks',type=Path,required=True)
    parser.add_argument('--final',type=Path,required=True)
    parser.add_argument('--bank-source',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    fruit,banks,final=(load_pack(p) for p in [args.fruit,args.banks,args.final])
    for pack in [fruit,banks,final]:validate(pack)
    plan=json.loads(args.bank_source.read_bytes())
    residual=copy.deepcopy(banks['data'])
    changes=0
    for row in plan['maps']:
        name=row['id'];old=fruit['data']['maps'][name];new=banks['data']['maps'][name]
        assert hashlib.sha256(bytes(old['blocks'])).hexdigest()==row['expected_blocks_sha256']
        expected=copy.deepcopy(old)
        for cell in row['cells']:
            x,y=cell['at'];i=y*old['attributes']['width']+x
            assert expected['blocks'][i]==cell['before']==5 and cell['after']==3
            expected['blocks'][i]=3;changes+=1
        assert new==expected
        label=old['attributes']['blocks_label']
        residual['maps'][name]=old
        residual['map_blocks'][label]=fruit['data']['map_blocks'][label]
    assert residual==fruit['data'],'bank patch altered unrelated content'
    for key in ['runtime_files','audio_manifest','compiled_audio','audio_compression','format_version','report']:
        assert banks[key]==fruit[key],key
    coast=[n for n in final['data']['maps'] if n.startswith('MnR11C')]
    for name in coast:
        assert final['data']['maps'][name]==banks['data']['maps'][name]
    frozen=[n for n,m in final['data']['maps'].items() if n.startswith('Mn') and m['attributes']['tileset_name'].startswith('mn_winter_')]
    for name in frozen:
        assert not set(collision(final,final['data']['maps'][name]).values()) & {'WATER','WATER_21','WATERFALL','BUOY'},('liquid remains in snowy exterior',name)
    for key in ['compiled_audio','audio_manifest','audio_compression']:
        assert final[key]==fruit[key],key
    receipt=dict(pack=str(args.final),pack_sha256=hashlib.sha256(args.final.read_bytes()).hexdigest(),content_identity=final['identity']['content_hash'],maps=len(final['data']['maps']),bank_metatiles=changes,exact_bank_residual=True,north_shore_warm_and_unchanged=coast,all_snowy_exteriors_without_liquid=frozen,audio_exact=True)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    assert not args.output.exists(),'retain earlier receipts'
    args.output.write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(receipt,indent=2))


if __name__=='__main__':main()
