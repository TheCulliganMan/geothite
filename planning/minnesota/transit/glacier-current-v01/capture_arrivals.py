#!/usr/bin/env python3
"""Capture actual train-arrival saves headlessly without scene overrides."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_archive import retain_render_file, save_render
from render_reference import room_sheet


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for field in ['pack', 'binary', 'saves', 'output']:
        parser.add_argument('--'+field, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    stops = json.loads(Path(__file__).with_name('service.json').read_text())['stations']
    private = Path('.local') / f'glacier-arrival-capture-{os.getpid()}'
    shutil.copy2(args.binary, private)
    report = dict(pack=str(args.pack), pack_sha256=sha(args.pack), binary_sha256=sha(private),
                  scope=__doc__, captures=[])
    env = dict(os.environ, CRYSTAL_RENDER_TEST_HEADLESS='1',
               CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image',
               CRYSTAL_RENDER_TEST_WINDOW_SIZE='640x576', CRYSTAL_PREVIEW_MUTE='1')
    for key in ['CRYSTAL_RENDER_TEST_PARTY_SPECIES', 'CRYSTAL_RENDER_TEST_PARTY_SIZE',
                'CRYSTAL_RENDER_TEST_SECOND_VIEW']:
        env.pop(key, None)
    frames = []
    try:
        for destination, stop in enumerate(stops):
            origin = 1 if destination == 0 else 0
            files = list(args.saves.glob(f'glacier-{origin}-to-{destination}-*.crystalsave'))
            assert len(files) == 1, (stop['id'], files)
            save = files[0]
            folder = args.output / stop['id']
            folder.mkdir()
            png = folder / 'production.png'
            cmd = [str(private.resolve()), '--pack', str(args.pack.resolve()),
                   '--save', str(save.resolve()), '--view', '2d',
                   '--screenshot', str(png.resolve())]
            frozen = sha(save)
            with (folder / 'capture.log').open('w') as log:
                result = subprocess.run(cmd, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=120)
            assert result.returncode == 0, stop['id']
            assert sha(save) == frozen
            retain_render_file(png)
            frames.append((stop['menu_label'], Image.open(png).convert('RGB')))
            report['captures'].append(dict(id=stop['id'], map=stop['map'],
                                           core=stop['ride_arrival_core'], save=str(save),
                                           save_sha256=frozen, production=str(png),
                                           production_sha256=sha(png), command=cmd))
            (args.output / 'progress.json').write_text(json.dumps(report, indent=2)+'\n')
            print(stop['id'], 'captured actual arrival', flush=True)
        save_render(room_sheet(frames, 3), args.output / 'all-five-actual-arrivals.png')
        report['status'] = 'PASS'
    finally:
        private.unlink()
        report['private_binary_removed'] = True
        assert sha(args.pack) == report['pack_sha256']
        (args.output / 'receipt.json').write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
