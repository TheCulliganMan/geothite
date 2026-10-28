#!/usr/bin/env python3
"""Compare retained ice captures against one complete native animation phase.

No pack or production image is changed. Reuses the production-camera reference
compositor; only native Ice Path glyphs $b5/$b1 receive their source scrolls.
"""
import argparse
import json
from pathlib import Path

from PIL import Image

from render_reference import SourceArt, load_pack
from review_passage_native import digest
from review_voyageurs_ice_native import compare


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--captures', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    prior = json.loads((args.captures / 'capture-progress.json').read_text())
    assert digest(args.pack) == prior['pack_sha256'], 'same captured pack'
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    report = {'pack_sha256': prior['pack_sha256'], 'source_sha256': digest(args.source),
              'production_binary_sha256': prior['binary_sha256'], 'captures': []}
    for room in json.loads(args.source.read_text())['rooms']:
        module = pack['data']['maps'][room['id']]
        assert module['blocks'] == [int(v, 16) for row in room['rows'] for v in row.split()]
        for view in room['review_viewpoints']:
            label = room['id'] + '-' + view['id']
            production = args.captures / label / 'production.png'
            original = next(r for r in prior['captures'] if r['id'] == label)
            assert digest(production) == original['production_sha256'], 'unchanged production pixels'
            actual = Image.open(production).convert('RGB')
            assert actual.size == (640, 576)
            output = args.output / label
            output.mkdir()
            errors, tick = compare(pack, art, module, view['tile'], actual, output)
            report['captures'].append({'id': label, 'pixel_mismatches': errors,
                                       'coherent_tick': tick,
                                       'production_sha256': original['production_sha256']})
            print(label, errors, flush=True)
    assert len(report['captures']) == len(prior['captures']) == 9
    report['status'] = 'PASS' if all(r['pixel_mismatches'] == 0 for r in report['captures']) else 'REVIEW'
    (args.output / 'phase-proof.json').write_text(json.dumps(report, indent=2) + '\n')
    assert report['status'] == 'PASS', 'inspect retained coherent-phase differences'


if __name__ == '__main__':
    main()
