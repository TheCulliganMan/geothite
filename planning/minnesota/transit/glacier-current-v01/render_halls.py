#!/usr/bin/env python3
"""Render every complete installed hall at a shared native scale."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import time

from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_archive import save_render
from render_progress_atlas import native_room
from render_reference import load_pack, room_sheet


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    source = json.loads(Path(__file__).with_name('service.json').read_text())
    started = time.perf_counter()
    rooms = []
    for stop in source['stations']:
        image = native_room(pack['data']['maps'][stop['map']], pack,
                            archive=args.output / stop['map'], native_origin=True)
        save_render(image, args.output / (stop['map'] + '-whole-original.png'))
        save_render(image.resize((image.width*2, image.height*2), Image.Resampling.NEAREST),
                    args.output / (stop['map'] + '-whole-2x.png'))
        rooms.append((stop['menu_label'], image))
    save_render(room_sheet(rooms, 3), args.output / 'all-five-halls-original.png')
    result = dict(pack=str(args.pack), pack_sha256=hashlib.sha256(args.pack.read_bytes()).hexdigest(),
                  native_scale='Per-hall originals are native1x; contact sheet is exact2x for every hall. Labels and margins are review annotation.',
                  warm_render_seconds=round(time.perf_counter()-started, 3),
                  maps=[s['map'] for s in source['stations']], gameplay_verified=False)
    (args.output / 'receipt.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
