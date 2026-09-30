#!/usr/bin/env python3
"""Verify repository-complete editable sources and embedded runtime model inputs."""
from pathlib import Path
from model_asset_storage import read_model_bytes, read_model_json, read_model_text, validate_storage
import gzip
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent.parent
from johto_art_sources import load_sources
editable = list(load_sources())

models = ROOT / 'crates/crystal-voxel-view/models'
model_files = sorted([*models.rglob('*.mesh.json'), *models.rglob('*.rig.json')])
validate_storage(models)
assert model_files
for path in model_files:
    model = read_model_json(path)
    assert 'primitives' in model or 'joints' in model, path
    if 'joints' in model:
        versions = (1, 2) if path.parent.name == 'johto_characters' else (1,)
        assert model['version'] in versions and len(model['joints']) == 16, path
    assert not path.read_bytes().startswith(b'version https://git-lfs'), path

references = 0
for source in (ROOT / 'crates/crystal-voxel-view/src').rglob('*.rs'):
    # Literal paths may be arguments to a small lazy-loader macro. Inspect the
    # actual paths rather than depending on rustfmt's include_str layout.
    for relative in re.findall(r'"([^"\n]*models/[^"\n]+\.json(?:\.include\.rs)?)"', source.read_text()):
        path = (source.parent / relative.removesuffix('.include.rs')).resolve()
        assert path.is_file(), (source, relative)
        if path not in model_files:
            assert path in (models / 'battle/arenas.json', models / 'johto_characters/shared.geometry.json'), path
            read_model_json(path)
            continue
        if not (source.name == 'johto_actor_props.rs' and relative.endswith('.include.rs')):
            references += 1
    # The prop catalog uses a declarative macro so source identities and
    # embedded mesh paths cannot drift. Verify every concrete expansion too.
    if source.name == 'johto_actor_props.rs':
        declaration = re.search(r'authored_props!\s*\{(.*?)\n\}', source.read_text(), re.S)
        assert declaration, 'missing source-scoped actor model catalog'
        for label in re.findall(r'=>\s*"([^"\n]+)"', declaration.group(1)):
            path = models / 'actor_props' / (label + '.mesh.json')
            assert path.is_file(), (source, label)
            assert path in model_files, path
            references += 1
    if source.name == 'battle_species_models.rs':
        declaration = re.search(r'species_models!\s*\{(.*?)\n\}', source.read_text(), re.S)
        assert declaration, 'missing exact battle species catalog'
        for label in re.findall(r'=>\s*"([^"\n]+)"', declaration.group(1)):
            path = models / 'battle_species' / (label + '.mesh.json')
            assert path.is_file(), (source, label)
            assert path in model_files, path
            references += 1
assert references >= len(model_files) - 3  # retained original character meshes are superseded by rigs
print(f'Validated {len(editable)} complete editable Blender sources, '
      f'{len(model_files)} runtime models and {references} embedded references')
