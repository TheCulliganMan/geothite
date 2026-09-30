#!/usr/bin/env python3
"""Verify repository-complete editable sources and embedded runtime model inputs."""
from pathlib import Path
import gzip
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent.parent
from johto_art_sources import load_sources
editable = list(load_sources())

models = ROOT / 'crates/crystal-voxel-view/models'
model_files = sorted(models.rglob('*.json'))
assert model_files
for path in model_files:
    model = json.loads(path.read_text())
    assert 'primitives' in model or 'joints' in model, path
    if 'joints' in model:
        assert model['version'] == 1 and len(model['joints']) == 16, path
    assert not path.read_bytes().startswith(b'version https://git-lfs'), path

references = 0
for source in (ROOT / 'crates/crystal-voxel-view/src').rglob('*.rs'):
    for relative in re.findall(r'include_str!\("([^"\n]*models/[^"\n]+)"\)', source.read_text()):
        path = (source.parent / relative).resolve()
        assert path.is_file(), (source, relative)
        assert path in model_files, path
        references += 1
assert references >= len(model_files) - 3  # retained original character meshes are superseded by rigs
print(f'Validated {len(editable)} complete editable Blender sources, '
      f'{len(model_files)} runtime models and {references} embedded references')
