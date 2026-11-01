#!/usr/bin/env python3
"""Check detached geography candidates against a final native pack, then promote.

Requires cbor2/Pillow used by the existing repository source renderer. Default is
read-only. --apply is reserved for an already accepted exact native pack.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import layout_blocks, load_pack


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pack', type=Path)
    parser.add_argument('--expected-pack-sha256', required=True)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    assert digest(args.pack) == args.expected_pack_sha256, 'explicit final pack SHA differs'
    guards = json.loads((HERE / 'source-preimages.json').read_text())
    current_guards = json.loads((HERE / 'current-source-guards-v01.json').read_text())['read_only_current_guards']
    for name, expected in guards['canonical_preimages_sha256'].items():
        if name in current_guards:
            expected = current_guards[name]['sha256']
        assert digest(ROOT / name) == expected, f'canonical preimage changed: {name}'
    for name, expected in guards['candidate_sha256'].items():
        assert digest(HERE / name) == expected, f'detached candidate changed: {name}'
    dossier_guards = json.loads((HERE / 'promotion-dossier-guards-v01.json').read_text())['candidate_sha256']
    assert len(dossier_guards) == 3, 'exact three dossier guards required'
    for name, expected in dossier_guards.items():
        assert digest(HERE / name) == expected, f'dossier preimage changed: {name}'
    contract_path = HERE.parent / 'native-west-v02/contract.json'
    assert digest(contract_path) == guards['contract_sha256'], 'frozen contract changed'
    contract = json.loads(contract_path.read_text())
    pack = load_pack(args.pack)
    data = pack['data']
    assignments = {
        'MN_R11.layout.candidate.json': 'planning/minnesota/routes/MN_R11.layout.json',
        'MN_R11.dossier.candidate.json': 'planning/minnesota/routes/MN_R11.json',
        'MnMaraisCedarApproach.layout.candidate.json': 'planning/minnesota/towns/MnMaraisCedarApproach.layout.json',
        'MnMaraisInlandNeck.layout.candidate.json': 'planning/minnesota/towns/MnMaraisInlandNeck.layout.json',
        'MnMaraisCedarApproach.dossier.candidate.json': 'planning/minnesota/towns/MnMaraisCedarApproach.json',
        'MnMaraisInlandNeck.dossier.candidate.json': 'planning/minnesota/towns/MnMaraisInlandNeck.json',
    }
    normalized_layouts = {}
    for source in assignments:
        if 'layout' not in source:
            continue
        layout = json.loads((HERE / source).read_text())
        original_cells = layout_blocks(layout)
        if source == 'MnMaraisInlandNeck.layout.candidate.json':
            # Schema-only normalization for the Rust source compiler. Frozen
            # reviewed candidate bytes remain immutable. Point terrain is exact.
            assert 'shore_fitting' not in layout and 'reject_cliff_cuts' not in layout
            layout['shore_fitting'] = 'explicit_source_edges'
            layout['reject_cliff_cuts'] = True
        cells = layout_blocks(layout)
        assert cells == original_cells, f'schema normalization changed cells: {source}'
        normalized_layouts[source] = layout
        width, height = layout['dimensions_metatiles']
        if source.startswith('MN_R11'):
            for row in range(4):
                for column in range(2):
                    name = f'MnR11C{column:02}R{row:02}'
                    native = data['maps'][name]
                    a = native['attributes']
                    expected = [cells[(row*24+y)*width+column*32+x]
                                for y in range(a['height']) for x in range(a['width'])]
                    assert expected == native['blocks'], f'native/source cell drift: {name}'
                    assert a['tileset_name'] == layout['compiled_tileset'], f'R11 alias drift: {name}'
        else:
            name = layout['id']
            native = data['maps'][name]
            assert native['blocks'] == cells, f'native/source cell drift: {name}'
            assert [native['attributes']['width'], native['attributes']['height']] == [width, height]
            assert native['attributes'] == contract['inland']['cedar' if name.endswith('Approach') else 'neck']['attributes']
    for name, side, target, offset in [
        ('MnR11C01R00','east','MnGrandMarais',0),
        ('MnR12C01R01','east','MnMaraisInlandNeck',0),
        ('MnGrandMarais','west','MnR11C01R00',0),
        ('MnGrandMarais','north','MnMaraisCedarApproach',-19),
    ]:
        assert any(c['direction']==side and c['target_map']==target and c['offset']==offset
                   for c in data['maps'][name]['attributes']['connections']), f'connection drift: {name}'
    assert not any(c['direction']=='north' for c in data['maps']['MnR11C00R00']['attributes']['connections'])
    for source, target in assignments.items():
        if 'MN_R11' not in source:
            assert not (ROOT / target).exists(), f'new canonical source already exists: {target}'
    # Normalize publication metadata only after all exact candidate/native checks.
    # Local fixture/controller acceptance does not imply earned campaign access.
    for source in assignments:
        if 'dossier' not in source:
            continue
        dossier = json.loads((HERE / source).read_text())
        dossier['status'] = 'MATCHES_COMPOSED_NATIVE_GEOGRAPHY'
        dossier['native_geography_publication'] = dict(
            final_pack_sha256=args.expected_pack_sha256,
            content_identity=pack['identity']['content_hash'],
            source_contract='revisions/north-shore-ely-geography-v01/geography/native-west-v02/contract.json',
            source_contract_sha256=guards['contract_sha256'],
            verification='revisions/north-shore-ely-geography-v01/VERIFICATION.md',
            progression_scope='Local explicit fixtures and production inputs; earned campaign progression is not established.')
        if source.startswith('MN_R11'):
            dossier['anchor_status'] = 'Original native actors/doors/parties retained; west-entry terrain and connected Foot/Surf verified. See revisions/north-shore-ely-geography-v01/VERIFICATION.md.'
            dossier['warm_coast_restoration_review'].update(
                status='COMPOSED_NATIVE_WARM_WEST_ENTRY',
                publication_identity=pack['identity']['content_hash'],
                preserved='Original warm restoration plus306 bounded terminal headland cells; original actors/programs/falls/stairs/ledges retained, obsolete north port closed and west-town connection installed.',
                source_policy='Canonical warm strokes plus exact frozen west-entry headland edits; source raster is checked against every final R11 chunk.')
        if source.startswith('MnMaraisCedarApproach'):
            for sign in dossier.get('signs', []):
                if sign['id'] == 'MN_COHESION_MARAIS_SIGN':
                    assert sign['lines'] == ['CEDAR APPROACH', 'GRAND MARAIS west.', 'NORTH SHORE south.']
                    sign['lines'] = ['CEDAR APPROACH', 'GRAND MARAIS south', 'GUNFLINT west.']
        if source.startswith('MnMaraisInlandNeck'):
            dossier['native_registration'] = data['runtime_map_metadata']['MN_MARAIS_INLAND_NECK']
            assert (dossier['native_registration']['groupId'], dossier['native_registration']['mapId']) == (40, 8)
            dossier['retained_recovery'] = None
        normalized_layouts[source] = dossier
    # Preload every byte before writing. Source history and all candidate epochs stay immutable.
    writes = [(ROOT / target,
               (json.dumps(normalized_layouts[source], indent=2) + '\n').encode()
               if source in normalized_layouts else (HERE / source).read_bytes())
              for source, target in assignments.items()]
    if args.apply:
        for target, content in writes:
            target.write_bytes(content)
    print('Neck schema normalization: shore_fitting=explicit_source_edges; reject_cliff_cuts=true; all1235 cells unchanged')
    print(('Promoted' if args.apply else 'Verified without applying'), len(writes), 'geography source files')


if __name__ == '__main__':
    main()
