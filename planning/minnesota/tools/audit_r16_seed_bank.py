#!/usr/bin/env python3
"""Compare the entire prairie child against the two source-bound native chunks."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_reference import load_pack
from review_r16_seed_bank import compose


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    for name in ['parent', 'candidate', 'source', 'output']:
        cli.add_argument('--' + name, type=Path, required=True)
    args = cli.parse_args()
    inputs = {str(p): sha(p) for p in [args.parent, args.candidate, args.source]}
    source = json.loads(args.source.read_text())
    assert sha(args.source) == '2c0cb252df3534b42595afc6c547ad2fa72a867067727bb769b14148dba53bac'
    assert sha(args.parent) == source['parent_pack_sha256']
    base, child = load_pack(args.parent), load_pack(args.candidate)
    _, modules, geometry = compose(base, source)
    expected = deepcopy(base)
    for module in modules:
        expected['data']['maps'][module['id']] = module
        label = module['attributes']['blocks_label']
        expected['data']['map_blocks'][label] = base64.b64encode(bytes(module['blocks'])).decode()
    marker = 'minnesota-r16-seed-bank-v1'
    expected['report']['manifests'].append(marker)
    expected['identity']['runtime_modpack_id'] += '+' + marker
    expected['identity']['content_hash'] = child['identity']['content_hash']
    assert child == expected, 'unrelated prairie child content differs'
    assert all(sha(Path(p)) == value for p, value in inputs.items())
    args.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(status='PASS_COMPLETE_PRESERVATION', inputs=inputs,
                   candidate_content_hash=child['identity']['content_hash'],
                   complete_other_maps_preserved=len(base['data']['maps']) - 1,
                   changed_metatiles=15, geometry=geometry,
                   scope='Whole root matches the fifteen western-chunk metatiles, encoded mirror, marker and identity. Every other map, actor, event, program, current regional party/encounter/fishing table, attribute, art and audio remains exact.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()
