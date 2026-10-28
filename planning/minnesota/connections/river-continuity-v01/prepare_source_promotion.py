#!/usr/bin/env python3
"""Freeze and independently check a detached R03 source reconciliation.

This helper has no apply mode. It never changes canonical layouts, packs, or
shared manifests. The integration owner must separately accept native evidence
and compare the exact canonical preimages before applying either candidate.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import layout_blocks, load_pack

PLAN = Path(__file__).resolve().parent
PACK_SHA = '57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62'
PACK_IDENTITY = '6ba03931721e00e50adbf8f058ef81f0ab9a94b75c2b8899ded9a198c75ffb2b'
SOURCE_LAYOUT = Path('planning/minnesota/routes/MN_R03.layout.json')
SOURCE_DOSSIER = Path('planning/minnesota/routes/MN_R03.json')


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def cells(before, after, width):
    assert len(before) == len(after)
    return [dict(metatile=[i % width, i // width], before=f'{a:02x}',
                 after=f'{b:02x}') for i, (a, b) in enumerate(zip(before, after))
            if a != b]


def validation(before_layout, candidate_layout, before_dossier,
               candidate_dossier, target, expected_cells):
    before = layout_blocks(before_layout)
    after = layout_blocks(candidate_layout)
    expected_meta = deepcopy(before_layout)
    expected_meta['shore_fitting'] = 'explicit_source_edges'
    expected_meta['layers'] = candidate_layout['layers']
    old_sign = next(s for s in before_dossier['signs'] if s['id'] == 'MN_R03_NORTH_SIGN')
    expected_dossier = deepcopy(before_dossier)
    sign = next(s for s in expected_dossier['signs'] if s['id'] == 'MN_R03_NORTH_SIGN')
    sign['tile'] = [22, 8]
    return [
        dict(check='24 by 32 canonical extent exact', passed=candidate_layout['dimensions_metatiles'] == [24, 32]),
        dict(check='all 768 target cells exact', passed=after == target),
        dict(check='exact declared effective cell delta', passed=cells(before, after, 24) == expected_cells),
        dict(check='all 192 lower canonical cells exact', passed=after[576:] == before[576:]),
        dict(check='all metadata except shore fitting and layers exact', passed=candidate_layout == expected_meta),
        dict(check='all original authoring layers retained as exact prefix', passed=candidate_layout['layers'][:len(before_layout['layers'])] == before_layout['layers']),
        dict(check='explicit edge mode prevents neighbor refitting', passed=candidate_layout['shore_fitting'] == 'explicit_source_edges'),
        dict(check='original north sign source coordinate exact', passed=old_sign['tile'] == [22, 4]),
        dict(check='entire dossier exact except north sign tile', passed=candidate_dossier == expected_dossier),
    ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'Use a fresh immutable epoch.'
    assert sha(args.pack.read_bytes()) == PACK_SHA, 'Wrong accepted native candidate.'
    before_layout_raw = (ROOT / SOURCE_LAYOUT).read_bytes()
    before_dossier_raw = (ROOT / SOURCE_DOSSIER).read_bytes()
    before_layout = json.loads(before_layout_raw)
    before_dossier = json.loads(before_dossier_raw)
    before = layout_blocks(before_layout)
    assert len(before) == 768
    pack = load_pack(args.pack)
    assert pack['identity']['content_hash'] == PACK_IDENTITY
    maps = pack['data']['maps']
    upper = maps['MnR03C00R00']
    assert [upper['attributes'][k] for k in ['width', 'height']] == [24, 24]
    target = list(before)
    target[:576] = upper['blocks']
    lower = maps['MnR03C00R01']
    width, height = [lower['attributes'][k] for k in ['width', 'height']]
    assert [width, height] == [21, 8]
    assert all(target[(y + 24) * 24 + x] == lower['blocks'][y * width + x]
               for y in range(height) for x in range(width)), 'Clipped native lower chunk changed.'
    old_native_layout = json.loads((PLAN / 'source-v01/MnR03C00R00.before.layout.json').read_bytes())
    old_native = layout_blocks(old_native_layout)
    historical = cells(before[:576], old_native, 24)
    native_delta = cells(old_native, target[:576], 24)
    delta = cells(before, target, 24)
    assert len(historical) == 15 and len(native_delta) == 41 and len(delta) == 55

    # Preserve every old authored stroke. Materialize only cells whose raw
    # stroke result differs from the explicit accepted grid, including old
    # fitted edges. This is source compilation, never terrain generation.
    candidate_layout = deepcopy(before_layout)
    candidate_layout['shore_fitting'] = 'explicit_source_edges'
    raw = layout_blocks(candidate_layout)
    materialized = cells(raw, target, 24)
    for change in materialized:
        x, y = change['metatile']
        candidate_layout['layers'].append(dict(
            kind='point', name=f'Explicit accepted river/retained shore {x},{y}',
            position=[x, y], block=change['after']))
    candidate_dossier = deepcopy(before_dossier)
    sign = next(s for s in candidate_dossier['signs'] if s['id'] == 'MN_R03_NORTH_SIGN')
    sign['tile'] = [22, 8]
    checks = validation(before_layout, candidate_layout, before_dossier,
                        candidate_dossier, target, delta)
    checks += [dict(check='native upper grid exact', passed=layout_blocks(candidate_layout)[:576] == upper['blocks']),
               dict(check='native sign coordinate agrees', passed=any(e['script'] == 'MN_R03_NORTH_SIGN' and [e['x'], e['y']] == [22, 8] for e in upper['events']['bg_events']))]
    negatives = []
    for label, mutate in [
        ('unlisted lower terrain', lambda l, d: l['layers'].append(dict(kind='point', position=[0, 31], block='01'))),
        ('wrong river cell', lambda l, d: l['layers'].append(dict(kind='point', position=[8, 2], block='05'))),
        ('automatic refit restored', lambda l, d: l.update(shore_fitting='johto_8_neighbors')),
        ('wrong compiled tileset', lambda l, d: l.update(compiled_tileset='johto')),
        ('old authoring stroke altered', lambda l, d: l['layers'][0].update(name='unexpected change')),
        ('old sign source left behind', lambda l, d: next(s for s in d['signs'] if s['id'] == 'MN_R03_NORTH_SIGN').update(tile=[22, 4])),
        ('unrelated dossier change', lambda l, d: d.update(status='undeclared')),
    ]:
        bad_layout, bad_dossier = deepcopy(candidate_layout), deepcopy(candidate_dossier)
        mutate(bad_layout, bad_dossier)
        failed = [r['check'] for r in validation(before_layout, bad_layout, before_dossier,
                  bad_dossier, target, delta) if not r['passed']]
        negatives.append(dict(case=label, rejected=bool(failed), failed_checks=failed))
    assert all(r['passed'] for r in checks) and all(r['rejected'] for r in negatives)
    # Guard again after the potentially slow read. No canonical writes occur.
    assert (ROOT / SOURCE_LAYOUT).read_bytes() == before_layout_raw, 'Concurrent layout edit.'
    assert (ROOT / SOURCE_DOSSIER).read_bytes() == before_dossier_raw, 'Concurrent dossier edit.'
    args.output.mkdir(parents=True)
    (args.output / 'MN_R03.before.layout.json').write_bytes(before_layout_raw)
    (args.output / 'MN_R03.before.json').write_bytes(before_dossier_raw)
    write_json(args.output / 'MN_R03.candidate.layout.json', candidate_layout)
    write_json(args.output / 'MN_R03.candidate.json', candidate_dossier)
    contract = dict(schema_version=1, status='DETACHED_CANONICAL_RECONCILIATION_NOT_APPLIED',
        native_pack=str(args.pack), native_pack_sha256=PACK_SHA, native_identity=PACK_IDENTITY,
        canonical_layout=str(SOURCE_LAYOUT), canonical_layout_before_sha256=sha(before_layout_raw),
        canonical_dossier=str(SOURCE_DOSSIER), canonical_dossier_before_sha256=sha(before_dossier_raw),
        dimensions_metatiles=[24, 32], effective_changed_cells=delta,
        inherited_native_overlay_differences=historical, newly_published_native_delta=native_delta,
        explicit_edge_materialization=materialized,
        explanation='The 55-cell effective union comprises 15 inherited source/native differences and 41 newly published cells, overlapping at one cell. All 192 lower canonical cells stay exact; the clipped 21 by 8 native lower map is checked at its actual stride.',
        sign=dict(id='MN_R03_NORTH_SIGN', before_core=[22, 4], after_core=[22, 8]),
        required_parser_change='Narrowly allow MN_R03 shore_fitting explicit_source_edges. Integration owner only; this helper does not edit Rust.',
        apply_guard='No apply mode. Integration owner must verify accepted native controller/GPU evidence, candidate hashes, canonical preimage byte equality, full target grid and exact dossier delta before writing.',
        target_grid_sha256=sha(bytes(target)), original_grid_sha256=sha(bytes(before)),
        checks=checks, negative_cases=negatives)
    write_json(args.output / 'promotion.contract.json', contract)
    files = {p.name: sha(p.read_bytes()) for p in sorted(args.output.iterdir()) if p.is_file()}
    write_json(args.output / 'freeze-receipt.json', dict(status='PASS_DETACHED_NOT_APPLIED', files=files,
        canonical_before_current=True, checks=len(checks), negatives=len(negatives)))
    print(json.dumps(dict(status='PASS_DETACHED_NOT_APPLIED', changed_cells=len(delta),
        inherited=len(historical), native_delta=len(native_delta), materialized=len(materialized),
        checks=len(checks), negatives=len(negatives), output=str(args.output))), flush=True)


if __name__ == '__main__':
    main()
