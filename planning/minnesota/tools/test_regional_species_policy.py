#!/usr/bin/env python3
"""Exercise the regional species checker against an external pack and isolated mutations."""
from pathlib import Path
import sys
from unittest.mock import patch

from audit_region_encounters import active_roster
from audit_regional_species_policy import audit, has_fishing_water, water_terrain
from render_reference import load_pack


def rejected(pack, active, source, species):
    try:
        audit(pack, active)
    except AssertionError as error:
        assert source in str(error) and species in str(error), error
    else:
        raise AssertionError(f'accepted {source} containing {species}')


def verify(pack):
    active = active_roster()
    data = pack['data']
    maps = data['maps']
    counts = audit(pack, active)
    assert counts['active_species'] == len(active)
    gym = maps['MnStCloudGym']
    leader = next(o for o in gym['objects'] if o['script'] == 'MnStCloudGymLeader')
    assert leader['sprite'] == 'SPRITE_MN_LEADER_SAVEN'
    assert data['trainers']['trainers']['MN_GYM_5']['trainer_class'] == 'MN_LEADER_SAVEN'
    with patch.dict(leader, sprite='SPRITE_BRISTUB'):
        audit(pack, active)
    for sprite in ('MAGIKARP', 'MN_LEADER_FICTITIOUS'):
        with patch.dict(leader, sprite=f'SPRITE_{sprite}'):
            rejected(pack, active, 'MnStCloudGym:visible-actor:MnStCloudGymLeader', sprite)
    with patch.dict(data['trainers']['trainers']['MN_GYM_5']['party'][0], species='MAGIKARP'):
        rejected(pack, active, 'trainer:MN_GYM_5', 'MAGIKARP')

    assert not has_fishing_water(data, gym) and has_fishing_water(data, maps['MnBrainerd'])
    assert all(water_terrain(t) for t in ('WATER', '24', 'C4', 'DOWN_BUOY', 'WATERFALL'))
    assert not any(water_terrain(t) for t in ('FLOOR', 'WALL', '23', '2B', 'BUOY', 'ICE'))
    # Dry indoor templates can carry a stock rod group. Once the same header
    # serves water, every species in it must be regional. Include script changes.
    with patch.dict(gym['attributes'], fishing_group='FISHGROUP_SHORE'):
        audit(pack, active)
        collision = data['tilesets'][gym['attributes']['tileset_name']]['collision']
        block = max(int(key, 16) for key in collision) + 1
        with patch.dict(collision, {f'{block:02x}': ['WATER'] * 4}):
            with patch.dict(gym, script_block_changes=[{'block_id': block, 'x': 2, 'y': 2}]):
                rejected(pack, active, 'fishing:FISHGROUP_SHORE', 'MAGIKARP')
    print('Accepted registered gym leaders, a regional creature and dry template fishing headers.')
    print('Rejected a nonregional actor, an unknown actor, a nonregional trainer partner and stock fish after a map change.')


if __name__ == '__main__':
    verify(load_pack(Path(sys.argv[1])))
