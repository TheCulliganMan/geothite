#!/usr/bin/env python3
"""Read-only cast/wiring audit; counts do not prove campaign reachability or balance."""
import argparse
from collections import Counter
import json
from pathlib import Path
from render_reference import load_pack
ROOT=Path(__file__).resolve().parents[3]


def audit(pack,story,active,network):
    data=pack['data'];catalog=data['trainers']['trainers']
    def trainer(identity):
        found=[t for t in catalog.values() if t['trainer_id']==identity]
        assert len(found)==1,('unique story trainer',identity,len(found))
        party=found[0]['party'];assert 1<=len(party)<=6
        for member in party:
            assert member['species'] in active,(identity,member['species'])
            assert 1<=member['level']<=100 and member['moves'],('legal party',identity)
        return found[0]
    guards=story['grunts'];assert len(guards)==24
    floors=Counter()
    for guard in guards:
        identity,name=guard['id'],guard['map'];room=data['maps'][name]
        trainer(identity)
        actors=[o for o in room['objects'] if o['script']==identity]
        assert len(actors)==1 and actors[0]['object_type']=='OBJECTTYPE_TRAINER'
        assert 1<=actors[0]['radius']<=4
        commands=room['scripts'][identity]
        assert any(c['command']=='trainer' and identity in c['args'] for c in commands)
        assert f'EVENT_{identity}_DEFEATED' in data['story_event_script_constants']['global']
        floors[name]+=1
    rivals=[scene['battle'] for scene in story['scene_contracts'] if 'battle' in scene]
    assert len(rivals)==6 and len(set(rivals))==6
    league=['MN_LEAGUE_1','MN_LEAGUE_2','MN_LEAGUE_3','MN_LEAGUE_4','MN_CHAMPION']
    bound={identity:set() for identity in rivals+league}
    for name,room in data['maps'].items():
        if not name.startswith('Mn'):continue
        for program in room['scripts'].values():
            if not isinstance(program,list):continue
            for command in program:
                if command['command'] not in ['loadtrainer','trainer']:continue
                for identity in bound:
                    if identity in command['args']:bound[identity].add(name)
    for identity in bound:
        trainer(identity);assert bound[identity],('missing native battle binding',identity)
    pads=Counter()
    for link in network['links']:
        for source,destination in [(link['a'],link['b']),(link['b'],link['a'])]:
            room=data['maps'][source['map']];target=data['maps'][destination['map']]
            at=source['active'];warps=[w for w in room['events']['warps'] if [w['x'],w['y']]==at]
            assert len(warps)==1,(link['id'],source['map'],'active warp')
            warp=warps[0];constant=target['attributes']['map_constant']
            assert warp['target_map_constant']==constant
            arrival=target['events']['warps'][warp['target_warp_id']-1]
            assert [arrival['x'],arrival['y']]==destination['arrival'],(link['id'],'native arrival')
        pads[link['kind']]+=1
    return dict(authored_guard_actors_verified=len(guards),guard_floors=dict(floors),
        six_motivated_rival_battles={identity:sorted(bound[identity]) for identity in rivals},
        elite_four_and_champion={identity:dict(name=trainer(identity)['name'],party_size=len(trainer(identity)['party']),maps=sorted(bound[identity])) for identity in league},
        reciprocal_hq_links=dict(pads),native_hq_card_doors=len(network['doors']),
        limitation='Cast, active roster and static native bindings only; not earned travel, battle balance, all dialogue acknowledgment or full-campaign acceptance')


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('pack');args=parser.parse_args()
    roster=json.loads((ROOT/'planning/minnesota/pokemon/roster.json').read_text())
    retired={f['id'] for f in json.loads((ROOT/'planning/minnesota/pokemon/roster-curation.json').read_text())['retired_families']}
    active={m['name'].upper() for f in roster['families'] if f['id'] not in retired for m in f['members']}
    story=json.loads((ROOT/'planning/minnesota/story-arc-v3.json').read_text())
    network=json.loads((ROOT/'planning/minnesota/lairs/ice-headquarters/network.json').read_text())
    print(json.dumps(audit(load_pack(args.pack),story,active,network),indent=2))


if __name__=='__main__':main()
