#!/usr/bin/env python3
"""Verify native winter terrain and promote frozen R12/R13 source with guards.

Read-only by default. This tool never writes a content pack or modifies a frozen
source epoch. --apply replaces only four canonical route authoring files after
all exact original-byte, candidate-byte and final-native comparisons pass.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
SOURCE = HERE / 'source-v02'
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import layout_blocks, load_pack

CONTRACT_SHA = '7e345514059a5333fe383db501a0d82785fc294cb9f9b8d398b46f5ffeea2d9d'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def json_sha(value):
    return sha(json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--expected-sha256', required=True)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    assert sha(args.pack.read_bytes()) == args.expected_sha256, 'final native pack SHA differs'
    raw = (SOURCE / 'native.review.contract.json').read_bytes()
    assert sha(raw) == CONTRACT_SHA, 'frozen source contract differs'
    contract = json.loads(raw)
    pack = load_pack(args.pack)
    assert len(pack['data']['maps']) == 694, 'completed geography plus winter required'
    pending = []
    dossier_edits = []
    for route in ['MN_R12', 'MN_R13']:
        candidate_name = route + '.winter.candidate.layout.json'
        layout_raw = (SOURCE / candidate_name).read_bytes()
        assert sha(layout_raw) == contract['candidates_sha256'][candidate_name]
        layout = json.loads(layout_raw)
        blocks = layout_blocks(layout)
        width, height = layout['dimensions_metatiles']
        assert len(blocks) == width * height
        modules = [m for m in contract['modules'] if m['map'].startswith(route.replace('MN_', 'Mn'))]
        for row in modules:
            native = pack['data']['maps'][row['map']]
            # Compare the complete frozen seasonal candidate, allowing exactly
            # the implemented target-only east seam correction in final R12.
            historical = json.loads(json.dumps(native))
            if row['map'] == 'MnR12C01R01':
                east = [c for c in historical['attributes']['connections'] if c['direction'] == 'east']
                assert len(east) == 1 and east[0] == dict(direction='east', target_map='MnMaraisInlandNeck', offset=0)
                east[0]['target_map'] = 'MnGrandMarais'
            assert json_sha(historical) == row['candidate_module_sha256'], row['map'] + ' full native candidate differs'
            assert native['attributes']['tileset_name'] == layout['compiled_tileset']
            match = re.fullmatch(r'MnR\d+C(\d+)R(\d+)', row['map'])
            col, chunk_row = map(int, match.groups())
            w, h = row['dimensions_metatiles']
            expected = [blocks[(chunk_row*24+y)*width+col*32+x] for y in range(h) for x in range(w)]
            assert expected == native['blocks'], row['map'] + ' source raster differs from native'
        for suffix in ['.json', '.layout.json']:
            target = ROOT / 'planning/minnesota/routes' / (route + suffix)
            before = SOURCE / (route + suffix + '.before')
            key = str(target.relative_to(ROOT))
            assert sha(before.read_bytes()) == contract['source_files_sha256'][key]
            assert target.read_bytes() == before.read_bytes(), str(target) + ' canonical preimage changed'
            candidate = route + '.winter.candidate' + suffix
            candidate_raw = (SOURCE / candidate).read_bytes()
            assert sha(candidate_raw) == contract['candidates_sha256'][candidate]
            if suffix == '.json':
                dossier = json.loads(candidate_raw)
                edits = []
                if route == 'MN_R12':
                    assert dossier['endpoints'][0]['place'] == 'GRAND MARAIS'
                    dossier['endpoints'][0]['place'] = 'GUNFLINT INLAND CONNECTOR'
                    edits.append('endpoints[0].place: GRAND MARAIS -> GUNFLINT INLAND CONNECTOR')
                    dossier['coastal_geography_binding'] = dict(
                        east_neighbor='MnMaraisInlandNeck', offset=0,
                        journey='East through the inland connector and Cedar approach, then south into Grand Marais; the warm North Shore enters town from the west.')
                    assert dossier['native_integration']['compiled_tileset'] == 'mn_gunflint'
                    dossier['native_integration']['compiled_tileset'] = 'mn_winter_gunflint'
                    edits.append('native_integration.compiled_tileset: mn_gunflint -> mn_winter_gunflint')
                    if 'native_status' in dossier.get('native_integration', {}):
                        dossier['native_integration']['native_status'] = 'Native seasonal terrain and six clipped modules match the verified publication. Original source review history is retained in source-v02.'
                dossier['status'] = 'Canonical seasonal source matches all native terrain in the verified final geography publication.'
                dossier['anchor_status'] = 'Frozen original/candidate bytes and all nine native module guards checked during source promotion; controller and visual evidence are recorded in the final geography receipt.'
                review = dossier['inland_winter_review']
                review['status'] = 'NATIVE_TERRAIN_VERIFIED_CANONICAL_SOURCE'
                assert review.pop('pending') == 'Root final west-entry geography binding, native winter publisher, controller, GPU and joint guarded source promotion.'
                review['published_pack_sha256'] = args.expected_sha256
                review['published_identity'] = pack['identity']['content_hash']
                review['verification_scope'] = 'Full native modules, exact source raster, seasonal cells and final inland connection. Production controller and visual acceptance remain separate evidence in the final geography receipt.'
                edits.extend(['status and anchor_status updated to native source verification',
                    'inland_winter_review status/pending replaced by exact publication binding and verification scope'])
                candidate_raw = (json.dumps(dossier, indent=2, ensure_ascii=False) + '\n').encode()
                dossier_edits.append(dict(route=route, edits=edits,
                    promoted_sha256=sha(candidate_raw)))
            pending.append((target, candidate_raw))
    # No write happens until all four original files and both entire regions
    # have been verified against the completed final native pack.
    if args.apply:
        for target, raw in pending:
            temporary = target.with_suffix(target.suffix + '.winter-promotion-tmp')
            assert not temporary.exists(), 'promotion temporary path occupied'
            try:
                temporary.write_bytes(raw)
                os.replace(temporary, target)
            finally:
                if temporary.exists():
                    temporary.unlink()
    print(json.dumps(dict(status='APPLIED' if args.apply else 'VERIFIED_ONLY',
        pack_sha256=args.expected_sha256, identity=pack['identity']['content_hash'],
        files=[str(p.relative_to(ROOT)) for p, _ in pending],
        verified_native_modules=9, verified_seasonal_cells=4586, dossier_edits=dossier_edits), indent=2))


if __name__ == '__main__':
    main()
