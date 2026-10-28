#!/usr/bin/env python3
"""Independent whole-written-pack roof residual and exact native-art review."""
import argparse
import base64
import copy
import hashlib
import io
import json
from pathlib import Path
import sys
from PIL import Image
sys.path.insert(0,str(Path(__file__).parent))
import preview_roof_fidelity as source
from render_archive import save_render
from render_reference import SourceArt
NAME=source.NAME;MAP=source.MAP


def validate(base,actual,expected):
    assert actual['data']==expected['data'],'Whole native data differs beyond exact roof alias/cells/block mirror'
    assert set(actual['runtime_files'])==set(expected['runtime_files'])
    png=f'gfx/tilesets/{NAME}.png'
    for path,value in expected['runtime_files'].items():
        if path!=png:assert bytes(actual['runtime_files'][path])==bytes(value),path
    native=Image.open(io.BytesIO(bytes(actual['runtime_files'][png])));preview=Image.open(io.BytesIO(bytes(expected['runtime_files'][png])))
    assert native.mode==preview.mode=='L' and native.size==preview.size==(128,128)
    assert native.tobytes()==preview.tobytes(),'Native PNG must reproduce exact authored bitplane indices'
    outside=copy.copy(actual);outside.pop('data');outside.pop('runtime_files');outside.pop('identity')
    old=copy.copy(base);old.pop('data');old.pop('runtime_files');old.pop('identity')
    assert outside==old,'Unrelated top-level report/audio/storage/format differs'
    assert set(actual['identity'])==set(base['identity']) and actual['identity']!=base['identity'],'Derived identity must change with roof art'


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--base',type=Path,required=True);p.add_argument('--actual',type=Path,required=True);p.add_argument('--contract',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    before_sha=source.sha(args.base.read_bytes());actual_sha=source.sha(args.actual.read_bytes());spec=json.loads(args.contract.read_text());assert before_sha==spec['base_pack_sha256'];base=source.load_pack(args.base);actual=source.load_pack(args.actual);expected=source.derive(base,spec)
    label=base['data']['maps'][MAP]['attributes']['blocks_label'];expected['data']['map_blocks'][label]=base64.b64encode(bytes(expected['data']['maps'][MAP]['blocks'])).decode()
    validate(base,actual,expected);args.output.mkdir(parents=True,exist_ok=False)
    art=SourceArt(actual);old=SourceArt(base);pixel_checks=[]
    for period in ['morn','day','nite','dark']:
        a=old.blocks(NAME,period);b=art.blocks(NAME,period)
        assert len(a)==137 and len(b)==139
        for block in range(137):assert a[block].tobytes()==b[block].tobytes()
        for original,new in [(0x2c,0x89),(0x2d,0x8a)]:assert a[original].crop((0,16,32,32)).tobytes()==b[new].crop((0,16,32,32)).tobytes()
        pixel_checks.append(dict(time_bank=period,old_recipes_exact=137,roof_timber_exact=2))
    m=actual['data']['maps'][MAP];scene=art.render(NAME,m['attributes']['width'],m['attributes']['height'],m['blocks']);save_render(scene,args.output/'written-roof-module-terrain.png')
    checks=[]
    for label,mutation in [('unrelated-road',lambda a:a['data']['maps'][MAP]['blocks'].__setitem__(0,5)),('old-art-glyph',lambda a:a['runtime_files'].__setitem__(f'gfx/tilesets/{NAME}.2bpp',bytes([1])+bytes(a['runtime_files'][f'gfx/tilesets/{NAME}.2bpp'])[1:])),('program',lambda a:a['data']['maps'][MAP]['scripts'].__setitem__('UnrelatedRoofProgram',[])),('extra-manifest',lambda a:a['report']['manifests'].append('unexpected'))]:
        changed=copy.deepcopy(actual);mutation(changed)
        try:validate(base,changed,expected)
        except AssertionError:checks.append(dict(case=label,rejected=True))
        else:raise AssertionError('Negative check not rejected: '+label)
    report=dict(status='PASS_WRITTEN_NATIVE_ROOF_RESIDUAL_AND_ART',base_pack_sha256=before_sha,actual_pack_sha256=actual_sha,actual_identity=actual['identity'],contract_sha256=source.sha(args.contract.read_bytes()),native_maps=len(actual['data']['maps']),full_native_data_exact_to_source_candidate=True,all_unowned_runtime_files_and_top_level_fields_exact=True,png_note='Native Rust and Pillow PNG encoders may differ; exact physical index pixels and original bitplanes checked, no color fitting.',old_prefix_all_four_timebanks=pixel_checks,negative_checks=checks,owned_metatiles=spec['cells'],limits=['Native GPU/controller acceptance is separate.','Complete identity derivation and uncached parser residual are independently enforced by the native Rust publisher; this consumer verifies written identity changed and all other fields exactly.'])
    assert source.sha(args.base.read_bytes())==before_sha and source.sha(args.actual.read_bytes())==actual_sha
    (args.output/'proof.json').write_text(json.dumps(report,indent=2)+'\n');print('PASS written683-map roof residual,548oldblock/time checks,8timber halves,4negative cases')


if __name__=='__main__':main()
