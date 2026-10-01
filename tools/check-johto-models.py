#!/usr/bin/env python3
"""Verify repository-complete editable sources and embedded runtime model inputs."""
from pathlib import Path
from model_asset_storage import read_model_json, validate_storage
from animated_glb import load_catalog
from pidgeotto_glb import read_pidgeotto, species_paths
import re

ROOT = Path(__file__).resolve().parent.parent
from johto_art_sources import load_sources
editable = list(load_sources())

models = ROOT / 'crates/crystal-voxel-view/models'
model_files = sorted([*models.rglob('*.mesh.json'), *models.rglob('*.rig.json'), *models.rglob('*.glb')])
model_counts = {}
species_files = species_paths(models / 'battle_species')
validate_storage(models)
assert model_files
for path in model_files:
    if path.suffix == '.glb':
        if path == models / 'battle_species/pidgeotto.glb':
            assert len(read_pidgeotto(path)['primitives']) == 34
            model_counts[path] = 1
            continue
        assert path == models / 'johto_characters/catalog.glb', path
        catalog = load_catalog(path.parent)
        assert len(catalog) == 75
        assert all(len(rig['joints']) == 16 for rig in catalog.values())
        model_counts[path] = len(catalog)
        continue
    model_counts[path] = 1
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
    for relative in re.findall(r'"([^"\n]*models/[^"\n]+\.(?:json|glb)(?:\.include\.rs)?)"', source.read_text()):
        base = models.parent if relative.startswith('models/') else source.parent
        path = (base / relative.removesuffix('.include.rs')).resolve()
        assert path.is_file(), (source, relative)
        if path not in model_files:
            assert path in (models / 'battle/arenas.json', models / 'johto_characters/shared.geometry.json'), path
            read_model_json(path)
            continue
        if not (source.name == 'johto_actor_props.rs' and relative.endswith('.include.rs')):
            references += model_counts[path]
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
            path = species_files[label]
            assert path.is_file(), (source, label)
            assert path in model_files, path
            references += 1
model_count = sum(model_counts.values())
assert references >= model_count - 3  # retained original character meshes are superseded by rigs
print(f'Validated {len(editable)} complete editable Blender sources, '
      f'{model_count} runtime models in {len(model_files)} files and {references} embedded references')
