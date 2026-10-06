#!/usr/bin/env python3
"""Verify repository-complete editable sources and embedded runtime model inputs."""
from pathlib import Path
from model_asset_storage import read_model_json, validate_storage
from animated_glb import load_catalog
from pidgeotto_glb import read_pidgeotto
from battle_model_assets import species_paths
from bayleef_glb import read_bayleef
from gengar_glb import read_gengar
from chikorita_glb import read_chikorita
from cyndaquil_glb import read_cyndaquil
from totodile_glb import read_totodile
from spearow_glb import read_spearow
import re

ROOT = Path(__file__).resolve().parent.parent
from johto_art_sources import load_sources
editable = list(load_sources())

models = ROOT / 'crates/crystal-voxel-view/models'
weighted_skins = {
    models / 'battle_species/bayleef.glb': (read_bayleef, 33),
    models / 'actor_props/battle_chikorita.glb': (read_chikorita, 19),
    models / 'actor_props/battle_cyndaquil.glb': (read_cyndaquil, 30),
    models / 'actor_props/battle_totodile.glb': (read_totodile, 27),
    models / 'battle_species/gengar.glb': (read_gengar, 27),
    models / 'battle_species/spearow.glb': (read_spearow, 28),
}
model_files = sorted([*models.rglob('*.mesh.json'), *models.rglob('*.rig.json'), *models.rglob('*.glb')])
model_counts = {}
species_files = species_paths(models / 'battle_species')
validate_storage(models)
assert model_files
for path in model_files:
    if path.suffix == '.glb':
        if path in weighted_skins:
            reader, part_count = weighted_skins[path]
            model = reader(path)
            assert model['name'] == path.stem and len(model['primitives']) == part_count, path
            model_counts[path] = 1
            continue
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
        # Test readers may build a filesystem path with format!; that is not
        # an embedded asset reference. Production include paths stay literal.
        if '{' in relative or '}' in relative:
            assert source.name.endswith('_tests.rs'), (source, relative)
            continue
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
            skin = path.with_name(label + '.glb')
            if skin in weighted_skins:
                assert not path.exists(), 'canonical skin must not retain duplicate JSON geometry'
                path = skin
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
