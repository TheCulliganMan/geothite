#!/usr/bin/env python3
"""Fast, read-only Duluth source previews; native publication remains Rust-only."""
import argparse
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_reference import SourceArt, layout_blocks, load_pack
from review_passage_native import actors

ROOT = Path(__file__).resolve().parents[3]


def current_layout(pack_path, explicit):
    """Resolve the selected pack's authored layout; retain explicit preimages."""
    if explicit is not None:
        return explicit
    world = json.loads((ROOT / 'planning/minnesota/world.json').read_text())
    selected = world.get(world.get('active_local_review_revision'), {})
    index = selected.get('authoring_source_index')
    if index and (ROOT / selected['pack']).resolve() == pack_path.resolve():
        assert hashlib.sha256(pack_path.read_bytes()).hexdigest() == selected['pack_sha256']
        sources = json.loads((ROOT / index).read_text())
        assert sources['pack_sha256'] == selected['pack_sha256']
        return ROOT / sources['current_layouts']['MnDuluth']['path']
    return ROOT / 'planning/minnesota/towns/MnDuluth.layout.json'


def render(pack, art, layout_path, output):
    started = time.perf_counter()
    raw = layout_path.read_bytes()
    layout = json.loads(raw)
    assert layout['id'] == 'MnDuluth' and layout['source_pixels_modified'] is False
    width, height = layout['dimensions_metatiles']
    terrain = art.render(layout['tileset'], width, height, layout_blocks(layout),
                         derived=layout['derived_blocks'])
    image = actors(pack, terrain, pack['data']['maps']['MnDuluth'])
    digest = hashlib.sha256(raw).hexdigest()
    destination = output / (digest[:12] + '-' + str(time.time_ns()))
    destination.mkdir(parents=True, exist_ok=False)
    terrain.save(destination / 'terrain-original.png')
    image.save(destination / 'whole-town-original-rgba.png')
    image.convert('RGB').save(destination / 'whole-town.png')
    image.convert('RGB').resize((width * 16, height * 16), Image.Resampling.NEAREST).save(
        destination / 'preview.png')
    record = dict(layout=str(layout_path), layout_sha256=digest,
                  output=str(destination), source_pixels=terrain.size,
                  render_seconds=round(time.perf_counter() - started, 3),
                  status='Source preview only; no pack or gameplay state is written.')
    (destination / 'source.json').write_text(json.dumps(record, indent=2) + '\n')
    output.mkdir(parents=True, exist_ok=True)
    (output / 'latest.json').write_text(json.dumps(record, indent=2) + '\n')
    return record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--layout', type=Path,
                        help='Explicit layout; otherwise resolve selected authoring sources.')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--watch', action='store_true')
    args = parser.parse_args()
    args.layout = current_layout(args.pack, args.layout)
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    previous = None
    while True:
        digest = hashlib.sha256(args.layout.read_bytes()).hexdigest()
        if digest != previous:
            try:
                print(json.dumps(render(pack, art, args.layout, args.output)), flush=True)
            except (AssertionError, KeyError, ValueError) as error:
                if not args.watch:
                    raise
                print(json.dumps(dict(error=str(error), layout_sha256=digest)), flush=True)
            previous = digest
        if not args.watch:
            break
        time.sleep(0.25)


if __name__ == '__main__':
    main()
