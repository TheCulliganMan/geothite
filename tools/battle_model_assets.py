"""Resolve canonical battle models without retaining parallel geometry copies."""
from pathlib import Path

from meganium_glb import read_meganium
from bayleef_glb import read_bayleef
from gengar_glb import read_gengar
from model_asset_storage import read_model_json
from pidgeotto_glb import read_pidgeotto
from spearow_glb import read_spearow
from chikorita_glb import read_chikorita
from cyndaquil_glb import read_cyndaquil
from totodile_glb import read_totodile

GLB_READERS = {'meganium.glb': read_meganium, 'bayleef.glb': read_bayleef, 'pidgeotto.glb': read_pidgeotto, 'gengar.glb': read_gengar, 'spearow.glb': read_spearow}
STARTER_READERS = {'battle_chikorita.glb': read_chikorita,
                   'battle_cyndaquil.glb': read_cyndaquil,
                   'battle_totodile.glb': read_totodile}


def species_paths(directory, *, include_starters=False, actor_directory=None):
    directory = Path(directory)
    paths = {p.name.removesuffix('.mesh.json'): p for p in directory.glob('*.mesh.json')}
    for path in directory.glob('*.glb'):
        if path.name not in GLB_READERS:
            raise ValueError(f'unsupported battle species GLB: {path.name}')
        if path.stem in paths:
            raise ValueError(f'duplicate canonical {path.stem.capitalize()} JSON and GLB geometry')
        paths[path.stem] = path
    # Production registers these exact rigs from actor_props. Resolve that
    # single source instead of demanding duplicate battle_species exports.
    actors = Path(actor_directory) if actor_directory is not None else directory.parent / 'actor_props'
    for filename in STARTER_READERS if include_starters else ():
        name = filename.removeprefix('battle_').removesuffix('.glb')
        path = actors / filename
        if path.is_file():
            if name in paths:
                raise ValueError(f'duplicate canonical {name.capitalize()} battle and actor geometry')
            paths[name] = path
    return paths


def read_species_model(path):
    path = Path(path)
    if path.suffix == '.glb':
        if path.name in STARTER_READERS:
            return STARTER_READERS[path.name](path)
        if path.name not in GLB_READERS:
            raise ValueError(f'unsupported battle species GLB: {path.name}')
        return GLB_READERS[path.name](path)
    if not path.name.endswith('.mesh.json'):
        raise ValueError(f'not a canonical species mesh: {path.name}')
    return read_model_json(path)
