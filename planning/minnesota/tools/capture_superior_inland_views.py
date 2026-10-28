#!/usr/bin/env python3
"""Execute the saved player-camera plan with the production headless renderer.

Preserves each source reference, native output and log in a fresh evidence epoch.
Explicit coordinate fixtures establish presentation, not earned progression.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--expected-sha256', required=True)
    parser.add_argument('--renderer', type=Path, required=True)
    parser.add_argument('--plan', type=Path, default=ROOT / 'planning/minnesota/revisions/north-shore-ely-geography-v01/native-camera-plan-v01.json')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert sha(args.pack) == args.expected_sha256
    assert not args.output.exists(), 'choose a fresh immutable capture epoch'
    plan = json.loads(args.plan.read_bytes())
    assert all((ROOT / row['source']).exists() for row in plan['cameras'])
    args.output.mkdir(parents=True)
    shutil.copyfile(args.plan, args.output / 'camera-plan.json')
    env = dict(os.environ, CRYSTAL_RENDER_TEST_HEADLESS='1',
               CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image',
               CRYSTAL_RENDER_TEST_WINDOW_SIZE='640x576', RUST_MIN_STACK='33554432')
    receipt = dict(pack_sha256=args.expected_sha256, renderer_sha256=sha(args.renderer),
                   plan_sha256=sha(args.plan), scope=plan['scope'], captures=[])
    for row in plan['cameras']:
        folder = args.output / row['name']
        folder.mkdir()
        shutil.copyfile(ROOT / row['source'], folder / 'source160-original.png')
        output = folder / 'native-2d-original.png'
        command = [str(args.renderer.resolve()), '--pack', str(args.pack.resolve()),
                   '--map', row['map'], '--x', str(row['x']), '--y', str(row['y']),
                   '--hour', str(row['hour']), '--view', '2d', '--screenshot', str(output.resolve())]
        with (folder / 'renderer.log').open('w') as log:
            result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        entry = dict(row, command=command, returncode=result.returncode,
                     output_sha256=sha(output) if output.exists() else None)
        receipt['captures'].append(entry)
        (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
        print(row['name'], 'captured' if output.exists() else 'missing', result.returncode, flush=True)
        assert result.returncode == 0 and output.exists(), 'inspect preserved renderer.log'
    shutil.copyfile(Path(__file__), args.output / Path(__file__).name)


if __name__ == '__main__':
    main()
