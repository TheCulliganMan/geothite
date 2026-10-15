"""Resolve canonical battle models without retaining parallel geometry copies."""
from pathlib import Path
from importlib import import_module


def _lazy_reader(module, name):
    # A focused model check must not construct every unrelated sculpture.
    # Loading on demand still uses each canonical module's strict reader.
    def read(path):
        return getattr(import_module(module), name)(path)
    read.__name__ = name
    return read

read_jigglypuff = _lazy_reader('jigglypuff_glb', 'read_jigglypuff')
read_clefairy = _lazy_reader('clefairy_glb', 'read_clefairy')
read_sentret = _lazy_reader('sentret_glb', 'read_sentret')
read_staryu = _lazy_reader('staryu_glb', 'read_staryu')
read_voltorb = _lazy_reader('voltorb_glb', 'read_voltorb')
read_onix = _lazy_reader('onix_glb', 'read_onix')
read_diglett = _lazy_reader('diglett_glb', 'read_diglett')
read_slowpoke = _lazy_reader('slowpoke_glb', 'read_slowpoke')
read_entei = _lazy_reader('entei_glb', 'read_entei')
read_raikou = _lazy_reader('raikou_glb', 'read_raikou')
read_furret = _lazy_reader('furret_glb', 'read_furret')
read_suicune = _lazy_reader('suicune_glb', 'read_suicune')
read_unown = _lazy_reader('unown_glb', 'read_unown')
read_hoothoot = _lazy_reader('hoothoot_glb', 'read_hoothoot')
read_butterfree = _lazy_reader('wing_insect_glb', 'read_butterfree')
read_beedrill = _lazy_reader('wing_insect_glb', 'read_beedrill')
read_zubat = _lazy_reader('bat_glb', 'read_zubat')
read_golbat = _lazy_reader('bat_glb', 'read_golbat')
read_crobat = _lazy_reader('bat_glb', 'read_crobat')
read_psyduck = _lazy_reader('duck_glb', 'read_psyduck')
read_golduck = _lazy_reader('duck_glb', 'read_golduck')
read_raticate = _lazy_reader('raticate_glb', 'read_raticate')
read_rattata = _lazy_reader('rattata_glb', 'read_rattata')
read_poliwag = _lazy_reader('poliwag_glb', 'read_poliwag')
read_oddish = _lazy_reader('oddish_glb', 'read_oddish')
read_pidgey = _lazy_reader('pidgey_glb', 'read_pidgey')
read_geodude = _lazy_reader('geodude_glb', 'read_geodude')
read_squirtle = _lazy_reader('squirtle_glb', 'read_squirtle')
read_charmander = _lazy_reader('charmander_glb', 'read_charmander')
read_bulbasaur = _lazy_reader('bulbasaur_glb', 'read_bulbasaur')
read_pikachu = _lazy_reader('pikachu_glb', 'read_pikachu')
read_snorlax = _lazy_reader('snorlax_glb', 'read_snorlax')
read_lapras = _lazy_reader('lapras_glb', 'read_lapras')
read_sudowoodo = _lazy_reader('sudowoodo_glb', 'read_sudowoodo')
read_gyarados = _lazy_reader('gyarados_glb', 'read_gyarados')
read_wooper = _lazy_reader('wooper_glb', 'read_wooper')
read_meganium = _lazy_reader('meganium_glb', 'read_meganium')
read_bayleef = _lazy_reader('bayleef_glb', 'read_bayleef')
read_gengar = _lazy_reader('gengar_glb', 'read_gengar')
read_pidgeotto = _lazy_reader('pidgeotto_glb', 'read_pidgeotto')
read_spearow = _lazy_reader('spearow_glb', 'read_spearow')
read_chikorita = _lazy_reader('chikorita_glb', 'read_chikorita')
read_cyndaquil = _lazy_reader('cyndaquil_glb', 'read_cyndaquil')
read_totodile = _lazy_reader('totodile_glb', 'read_totodile')


from model_asset_storage import read_model_json

GLB_READERS = {'sunkern.glb': _lazy_reader('sunflower_glb', 'read_sunkern'), 'sunflora.glb': _lazy_reader('sunflower_glb', 'read_sunflora'), 'aipom.glb': _lazy_reader('aipom_glb', 'read_aipom'), 'jumpluff.glb': _lazy_reader('jumpluff_glb', 'read_jumpluff'), 'skiploom.glb': _lazy_reader('skiploom_glb', 'read_skiploom'), 'hoppip.glb': _lazy_reader('hoppip_glb', 'read_hoppip'), 'quagsire.glb': _lazy_reader('quagsire_glb', 'read_quagsire'), 'spinarak.glb': _lazy_reader('spider_glb', 'read_spinarak'), 'ariados.glb': _lazy_reader('spider_glb', 'read_ariados'), 'ledyba.glb': _lazy_reader('ladybird_glb', 'read_ledyba'), 'ledian.glb': _lazy_reader('ladybird_glb', 'read_ledian'), 'noctowl.glb': _lazy_reader('noctowl_glb', 'read_noctowl'), 'furret.glb': read_furret, 'suicune.glb': read_suicune, 'raikou.glb': read_raikou, 'entei.glb': read_entei, 'slowpoke.glb': read_slowpoke, 'diglett.glb': read_diglett, 'onix.glb': read_onix, 'voltorb.glb': read_voltorb, 'staryu.glb': read_staryu, 'sentret.glb': read_sentret, 'hoothoot.glb': read_hoothoot, 'butterfree.glb': read_butterfree, 'beedrill.glb': read_beedrill, 'zubat.glb': read_zubat, 'golbat.glb': read_golbat, 'crobat.glb': read_crobat, 'psyduck.glb': read_psyduck, 'golduck.glb': read_golduck, 'raticate.glb': read_raticate, 'rattata.glb': read_rattata, 'poliwag.glb': read_poliwag, 'oddish.glb': read_oddish, 'pidgey.glb': read_pidgey, 'geodude.glb': read_geodude, 'squirtle.glb': read_squirtle, 'charmander.glb': read_charmander, 'bulbasaur.glb': read_bulbasaur, 'pikachu.glb': read_pikachu, 'snorlax.glb': read_snorlax, 'lapras.glb': read_lapras, 'sudowoodo.glb': read_sudowoodo, 'gyarados.glb': read_gyarados, 'wooper.glb': read_wooper, 'meganium.glb': read_meganium, 'bayleef.glb': read_bayleef, 'pidgeotto.glb': read_pidgeotto, 'gengar.glb': read_gengar, 'spearow.glb': read_spearow}
GLB_READERS.update({'unown_'+chr(letter)+'.glb': read_unown for letter in range(ord('a'),ord('z')+1)})

STARTER_READERS = {'battle_chikorita.glb': read_chikorita,
                   'battle_cyndaquil.glb': read_cyndaquil,
                   'battle_totodile.glb': read_totodile}

ACTOR_READERS = {**STARTER_READERS, 'icon_clefairy.glb': read_clefairy, 'icon_jigglypuff.glb': read_jigglypuff}

def species_paths(directory, *, include_starters=False, actor_directory=None):
    directory = Path(directory)
    paths = {p.name.removesuffix('.mesh.json'): p for p in directory.glob('*.mesh.json')}
    forms = sorted(directory.glob('unown_?.glb'))
    if forms and {p.stem for p in forms} != {'unown_'+chr(letter) for letter in range(ord('a'),ord('z')+1)}:
        raise ValueError('incomplete canonical Unown A..Z family')
    for path in directory.glob('*.glb'):
        if path.name not in GLB_READERS:
            raise ValueError(f'unsupported battle species GLB: {path.name}')
        if path.stem.startswith('unown_'):
            if 'unown' in paths and paths['unown'].suffix != '.glb':
                raise ValueError('duplicate canonical Unown JSON and GLB geometry')
            if path.stem == 'unown_i': paths['unown'] = path
            continue
        if path.stem in paths:
            raise ValueError(f'duplicate canonical {path.stem.capitalize()} JSON and GLB geometry')
        paths[path.stem] = path
    # The legacy include_starters flag includes canonical actor-space rigs.
    # Production registers these exact rigs from actor_props. Resolve that
    # single source instead of demanding duplicate battle_species exports.
    actors = Path(actor_directory) if actor_directory is not None else directory.parent / 'actor_props'
    for filename in ACTOR_READERS if include_starters else ():
        name = filename.removeprefix('battle_').removeprefix('icon_').removesuffix('.glb')
        path = actors / filename
        if path.is_file():
            if path.with_suffix(".mesh.json").exists():
                raise ValueError(f"duplicate canonical {name.capitalize()} actor JSON and GLB geometry")
            if name in paths:
                raise ValueError(f'duplicate canonical {name.capitalize()} battle and actor geometry')
            paths[name] = path
    return paths


def read_species_model(path):
    path = Path(path)
    if path.suffix == '.glb':
        if path.name in ACTOR_READERS:
            return ACTOR_READERS[path.name](path)
        if path.name not in GLB_READERS:
            raise ValueError(f'unsupported battle species GLB: {path.name}')
        return GLB_READERS[path.name](path)
    if not path.name.endswith('.mesh.json'):
        raise ValueError(f'not a canonical species mesh: {path.name}')
    return read_model_json(path)
