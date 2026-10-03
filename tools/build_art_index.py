#!/usr/bin/env python3
"""Build an honest Git-browsable source/model index from actual checked-in files."""
from collections import defaultdict
from pathlib import Path
import json
import struct
from johto_art_sources import load_sources
from animated_glb import load_catalog
from model_asset_storage import read_model_bytes, stored_model_size

ROOT=Path(__file__).resolve().parent.parent

def build(root=ROOT):
    art=root/'art/johto';source=art/'source'
    entries=dict(load_sources(source))
    lines=['# Johto source and model index','',
           'Original editable Blender scenes, animated GLB characters, and raw JSON static models are ordinary files in this repository. No reconstruction is needed to open a `.blend` file in Blender. Source previews are authoring views, not gameplay screenshots.','',
           'Only PNG files that exist are displayed below. A missing preview does not imply that a scene has been visually reviewed.','',
           '## Editable scenes','',
           '| Source | Native bytes | Authoring preview |','| --- | ---: | --- |']
    previews=0
    for name,payload in sorted(entries.items()):
        png=art/'previews'/(Path(name).stem+'.png')
        preview='Not rendered'
        if png.exists():
            data=png.read_bytes()
            if data[:8] != b'\x89PNG\r\n\x1a\n' or data[12:16] != b'IHDR' or len(data)<33:
                raise ValueError(f'invalid PNG preview: {png}')
            width,height=struct.unpack('>II',data[16:24])
            if not width or not height:raise ValueError(f'empty PNG preview: {png}')
            relative=png.relative_to(art).as_posix()
            preview=f'[![{Path(name).stem} authoring preview]({relative})]({relative})'
            previews+=1
        lines.append(f'| [{name}](source/{name}) | {len(payload):,} | {preview} |')
    lines += ['',f'{len(entries)} editable scenes; {previews} rendered PNG previews.','',
              '## Runtime models','',
              'Human characters share one standard `catalog.glb` with named scenes, articulated joints, materials and animation clips. Pidgeotto has its own articulated `pidgeotto.glb`; other models have directly readable `.json` geometry documents. Duplicate static STL previews are local, ignored output. Compression only exists in ignored Cargo build output. The generator scripts remain in [`tools/`](../../tools/).','',
              '| Catalog | Files | Stored bytes | Decoded JSON bytes |','| --- | ---: | ---: | ---: |']
    catalogs=defaultdict(lambda:[0,0,0])
    modelroot=root/'crates/crystal-voxel-view/models'
    for p in sorted(modelroot.rglob('*.json')):
        stats=catalogs[p.parent.relative_to(modelroot).as_posix()]
        stats[0]+=1;stats[1]+=stored_model_size(p);stats[2]+=len(read_model_bytes(p))
    for p in sorted(modelroot.rglob('*.glb')):
        stats=catalogs[p.parent.relative_to(modelroot).as_posix()]
        stats[0]+=1;stats[1]+=p.stat().st_size
    for name,(count,stored,decoded) in sorted(catalogs.items()):
        lines.append(f'| [{name}](../../crates/crystal-voxel-view/models/{name}) | {count} | {stored:,} | {decoded:,} |')
    catalog=modelroot/'johto_characters/catalog.glb'
    if catalog.exists():
        lines += ['', f'The human GLB contains {len(load_catalog(catalog.parent))} distinct character scenes; geometry accessors are shared across scenes. GLB bytes are native binary data, so they are not counted as decoded JSON.']
    lines += ['', 'Storage identity and validation details: [model storage](../../docs/art/model-storage.md).','']
    (art/'ASSETS.md').write_text('\n'.join(lines))
    return len(entries),previews

if __name__=='__main__':
    count,previews=build()
    print(f'Indexed {count} editable sources and {previews} existing PNG previews')
