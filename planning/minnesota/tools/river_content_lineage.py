"""Select the current reviewed parent and reject retired or icy coast geometry."""
import json
from pathlib import Path


def selected_parent():
    world = json.loads(Path('planning/minnesota/world.json').read_bytes())
    return Path(world[world['active_local_review_revision']]['pack'])


def validate(pack):
    maps = pack['data']['maps']
    for name in ['MnMaraisInlandNeck', 'MnR12C01R00', 'MnR12C01R01',
                 'MnR12C00R02', 'MnR12C01R02']:
        assert name not in maps, ('retired northern geometry', name)
    for name, size in [('MnR11C01R00', (4, 24)), ('MnGrandMarais', (20, 19)),
                       ('MnMaraisCedarApproach', (20, 12)),
                       ('MnR12C00R00', (32, 24)), ('MnR12C00R01', (32, 8))]:
        a = maps[name]['attributes']
        assert (a['width'], a['height']) == size, ('stale northern geometry', name)
    coast = [m for name, m in maps.items() if name.startswith('MnR11C')]
    assert len(coast) == 8
    for m in coast:
        assert m['attributes']['tileset_name'] == 'mn_north_shore', 'North Shore must remain unfrozen'
        collision = pack['data']['tilesets']['mn_north_shore']['collision']
        assert all('ICE' not in collision[f'{b:02x}'] for b in m['blocks'])
