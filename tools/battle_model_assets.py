"""Resolve canonical battle models without retaining parallel geometry copies."""
from pathlib import Path

from gengar_glb import read_gengar
from model_asset_storage import read_model_json
from pidgeotto_glb import read_pidgeotto

GLB_READERS = {'pidgeotto.glb': read_pidgeotto, 'gengar.glb': read_gengar}


def species_paths(directory):
    directory = Path(directory)
    paths = {p.name.removesuffix('.mesh.json'): p for p in directory.glob('*.mesh.json')}
    for path in directory.glob('*.glb'):
        if path.name not in GLB_READERS:
            raise ValueError(f'unsupported battle species GLB: {path.name}')
        if path.stem in paths:
            raise ValueError(f'duplicate canonical {path.stem.capitalize()} JSON and GLB geometry')
        paths[path.stem] = path
    return paths


def read_species_model(path):
    path = Path(path)
    if path.suffix == '.glb':
        if path.name not in GLB_READERS:
            raise ValueError(f'unsupported battle species GLB: {path.name}')
        return GLB_READERS[path.name](path)
    if not path.name.endswith('.mesh.json'):
        raise ValueError(f'not a canonical species mesh: {path.name}')
    return read_model_json(path)
