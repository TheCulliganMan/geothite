#!/usr/bin/env python3
"""Capture native party-menu grass Cut with matched pre-input clock palettes.

Requires the Rust fixture example, a current native renderer, Pillow and the
completed gameplay case inventory. Setup is explicit; only ordinary keyboard
inputs run after the native save is loaded. All originals/intermediates remain.
"""
import argparse
import csv
from datetime import datetime
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess
from zoneinfo import ZoneInfo

from PIL import Image, ImageDraw
from render_archive import save_render


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--fixture-writer', type=Path, required=True)
    parser.add_argument('--renderer', type=Path, required=True)
    parser.add_argument('--cases', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--timezone', default='Etc/GMT-12')
    args = parser.parse_args()
    cases = json.loads(args.cases.read_text())
    out = args.output
    out.mkdir(parents=True, exist_ok=False)
    world = Path('planning/minnesota/world.json').read_bytes()
    identities = {str(p): sha(p) for p in [args.pack, args.fixture_writer, args.renderer, args.cases]}
    zone = ZoneInfo(args.timezone)
    hour = datetime.now(zone).hour
    assert 10 <= hour < 17, 'choose a stable daylight capture timezone'
    env = {**os.environ, 'TZ': args.timezone,
           'CRYSTAL_RENDER_TEST_HEADLESS': '1', 'CRYSTAL_RENDER_TEST_CAPTURE_TARGET': 'image',
           'CRYSTAL_RENDER_TEST_WINDOW_SIZE': '640x576', 'CRYSTAL_PREVIEW_MUTE': '1',
           'RUST_MIN_STACK': '67108864', 'CRYSTAL_RENDER_TEST_WALK_INPUT': 'synthetic',
           'CRYSTAL_RENDER_TEST_WALK_PC_PULSE': '1'}
    records = []
    with open('.local/graphics-qa.lock', 'a+') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        for case in cases:
            assert datetime.now(zone).hour == hour, 'capture hour changed; retain this batch and review'
            save = out / (case['id'] + '-initial.crystalsave')
            command = [str(args.fixture_writer), 'fixture', str(args.pack), str(save), case['map'],
                       *map(str, case['approach']), case['facing'], '1', '1', str(hour)]
            fixture = subprocess.run(command, capture_output=True, text=True)
            assert fixture.returncode == 0, fixture.stderr
            original = save.read_bytes()
            for stage, keys in [('before', 'SBBBBBBB'), ('after', 'SAAABBBB')]:
                png = out / (case['id'] + '-' + stage + '.png')
                command = [str(args.renderer), '--pack', str(args.pack), '--save', str(save),
                           '--view', '2d', '--walk', keys, '--screenshot', str(png)]
                with png.with_suffix('.log').open('x') as log:
                    result = subprocess.run(command, env=env, stdout=log, stderr=subprocess.STDOUT)
                assert result.returncode == 0, str(png.with_suffix('.log'))
                assert save.read_bytes() == original, 'native input fixture was overwritten'
                trace = list(csv.DictReader(png.with_suffix('.movement.csv').open()))
                assert len(trace) == 256 and all(
                    row['runtime_map'] == case['map'] and
                    [int(row['tile_x']), int(row['tile_y'])] == case['approach'] for row in trace)
                records.append(dict(case=case['id'], stage=stage, command=command,
                                    png_sha256=sha(png), initial_save_sha256=sha(save), frames=len(trace)))
                receipt = dict(status='CAPTURED_REQUIRES_VISUAL_REVIEW', initial_hour=hour,
                               capture_timezone=args.timezone, input_hashes=identities,
                               original_fixtures_unchanged=True, captures=records,
                               world_unchanged=world == Path('planning/minnesota/world.json').read_bytes(),
                               scope='Explicit local clock/party/badge/position setup; ordinary native keyboard after load. Before opens and closes Start; after uses party Cut without a post-Cut reload. Weather and animation phase can differ. This is not earned progression.')
                (out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
                print(case['id'], stage, result.returncode, flush=True)
            pair = Image.new('RGB', (1280, 606), '#151515')
            draw = ImageDraw.Draw(pair)
            for column, stage in enumerate(['before', 'after']):
                pair.paste(Image.open(out / (case['id'] + '-' + stage + '.png')), (column * 640, 30))
                draw.text((column * 640 + 6, 7), case['id'] + ' ' + stage + ' native Cut', fill='white')
            save_render(pair, out / 'pairs' / (case['id'] + '.png'))
    assert all(sha(Path(path)) == digest for path, digest in identities.items())
    assert world == Path('planning/minnesota/world.json').read_bytes()


if __name__ == '__main__':
    main()
