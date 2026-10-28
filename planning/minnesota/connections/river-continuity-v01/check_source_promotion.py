#!/usr/bin/env python3
"""Read-only preflight for the frozen, unapplied R03 canonical candidate."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import layout_blocks
from prepare_source_promotion import PACK_SHA, PACK_IDENTITY, cells, validation


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'Choose a fresh preflight receipt.'
    receipt = json.loads((args.source / 'freeze-receipt.json').read_bytes())
    assert receipt['status'] == 'PASS_DETACHED_NOT_APPLIED'
    for name, digest in receipt['files'].items():
        assert sha((args.source / name).read_bytes()) == digest, ('Frozen candidate changed', name)
    contract = json.loads((args.source / 'promotion.contract.json').read_bytes())
    assert contract['native_pack_sha256'] == PACK_SHA and sha(args.pack.read_bytes()) == PACK_SHA
    assert contract['native_identity'] == PACK_IDENTITY
    files = {}
    for field, before_file in [('canonical_layout', 'MN_R03.before.layout.json'),
                               ('canonical_dossier', 'MN_R03.before.json')]:
        path = ROOT / contract[field]
        frozen = (args.source / before_file).read_bytes()
        assert path.read_bytes() == frozen, ('Canonical concurrent drift', str(path))
        assert sha(frozen) == contract[field + '_before_sha256']
        files[str(path.relative_to(ROOT))] = sha(frozen)
    before_layout = json.loads((args.source / 'MN_R03.before.layout.json').read_bytes())
    candidate_layout = json.loads((args.source / 'MN_R03.candidate.layout.json').read_bytes())
    before_dossier = json.loads((args.source / 'MN_R03.before.json').read_bytes())
    candidate_dossier = json.loads((args.source / 'MN_R03.candidate.json').read_bytes())
    original = layout_blocks(before_layout)
    target = list(original)
    frozen_native = json.loads((Path(__file__).resolve().parent / 'source-v01/MnR03C00R00.candidate.layout.json').read_bytes())
    target[:576] = layout_blocks(frozen_native)
    assert sha(bytes(original)) == contract['original_grid_sha256']
    assert sha(bytes(target)) == contract['target_grid_sha256']
    assert cells(original, target, 24) == contract['effective_changed_cells']
    checks = validation(before_layout, candidate_layout, before_dossier,
                        candidate_dossier, target, contract['effective_changed_cells'])
    checks += [dict(check='all frozen build-time validation passed', passed=all(r['passed'] for r in contract['checks'])),
               dict(check='all seven independent mutations were rejected', passed=len(contract['negative_cases']) == 7 and all(r['rejected'] for r in contract['negative_cases']))]
    assert all(r['passed'] for r in checks)
    # Snapshot canonical files once more immediately before writing the receipt.
    assert all(sha((ROOT / path).read_bytes()) == digest for path, digest in files.items())
    output = dict(status='PASS_PREFLIGHT_NO_APPLY', native_pack=str(args.pack),
        native_pack_sha256=PACK_SHA, native_identity=PACK_IDENTITY,
        frozen_receipt_sha256=sha((args.source / 'freeze-receipt.json').read_bytes()),
        canonical_before=files, checks=checks,
        candidate_layout_sha256=sha((args.source / 'MN_R03.candidate.layout.json').read_bytes()),
        candidate_dossier_sha256=sha((args.source / 'MN_R03.candidate.json').read_bytes()),
        limitation='Read-only source guard. Root must separately accept native controller and visual evidence before applying these exact candidate bytes.')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps(dict(status=output['status'], checks=len(checks), output=str(args.output))))


if __name__ == '__main__':
    main()
