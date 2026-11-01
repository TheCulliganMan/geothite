#!/usr/bin/env python3
"""Check the cabin identity delta without accepting unrelated content changes."""
import argparse
from copy import deepcopy
import json

from render_grand_marais_pack import dialogue_rows
from render_reference import load_pack

ROOM = 'MnBoundaryRootBeerCabin'
OLD = 'MN_INTERIOR_DOROTHY'
NEW = 'MN_INTERIOR_CABIN_VOLUNTEER'
OLD_TEXT = OLD+'InteriorText'
NEW_TEXT = NEW+'InteriorText'
SERVICE = 'BWS_ROOT_BEER_LADYRootBeerService'


def renamed(value):
    names = {OLD: NEW, OLD_TEXT: NEW_TEXT}
    if isinstance(value,str):
        return names.get(value,value)
    if isinstance(value,list):
        return [renamed(v) for v in value]
    if isinstance(value,dict):
        return {names.get(k,k):renamed(v) for k,v in value.items()}
    return value


def audit(before, after, source):
    for key in before.keys() | after.keys():
        if key not in {'data','report','identity'}:
            assert before.get(key) == after.get(key), key
    assert after['report'] == dict(before['report'],manifests=before['report']['manifests']+[source['id']])
    old,new = before['data'],after['data']
    assert new['maps'].keys() == old['maps'].keys()
    original,room = old['maps'][ROOM],new['maps'][ROOM]
    expected = renamed(original)
    actors = [o for o in expected['objects'] if o['script'] == NEW]
    assert len(actors) == 1 and actors[0]['sprite'] == source['old_sprite']
    actors[0]['sprite'] = source['new_sprite']
    lines = dialogue_rows(source['lines'])
    assert all(len(line)<=18 for line in lines)
    text = [dict(command='text' if i==0 else 'line' if i%2 else 'para',args=['"'+line+'"']) for i,line in enumerate(lines)]
    text.append(dict(command='done',args=[]))
    expected['scripts'][NEW_TEXT] = text
    expected['script_text_bodies'][NEW_TEXT] = dict(label=NEW_TEXT,commands=[dict(row,command_index=i) for i,row in enumerate(text)])
    for row in expected['map_event_section_commands']:
        if row['command']=='object_event' and row['args'][11]==NEW:
            assert row['args'][2]==source['old_sprite']
            row['args'][2]=source['new_sprite']
    assert room == expected, [k for k in room if room[k]!=expected.get(k)]
    assert len(room['objects'])==2
    assert OLD not in room['scripts'] and OLD_TEXT not in room['scripts']
    assert new['maps'][ROOM]['events']==original['events']
    dorothy=[(name,o) for name,m in new['maps'].items() for o in m['objects'] if o['script']==SERVICE]
    assert len(dorothy)==1 and dorothy[0][0]!=ROOM and dorothy[0][1]['sprite']=='SPRITE_GRANNY'
    residual=deepcopy(new)
    residual['maps'][ROOM]=original
    assert new['npcs'][ROOM]==room['objects']
    residual['npcs'][ROOM]=old['npcs'][ROOM]
    assert new['map_scripts'].keys()-old['map_scripts'].keys()=={NEW,NEW_TEXT}
    assert old['map_scripts'].keys()-new['map_scripts'].keys()=={OLD,OLD_TEXT}
    for label in [NEW,NEW_TEXT]:
        assert new['map_scripts'][label]==room['scripts'][label]
        del residual['map_scripts'][label]
    for label in [OLD,OLD_TEXT]:
        residual['map_scripts'][label]=old['map_scripts'][label]
    for attribute,section in [('map_scripts_label','map_script_section_commands'),('map_events_label','map_event_section_commands')]:
        label=room['attributes'][attribute]
        assert new['map_scripts'][label]==[dict(command=r['command'],args=r['args']) for r in room[section]]
        residual['map_scripts'][label]=old['map_scripts'][label]
    assert residual==old,'unrelated compiled content changed'
    return dict(status='PASS',other_maps_exact=len(old['maps'])-1,unchanged_art_audio_items_spawns_and_flags=True,
                outdoor_dorothy_unchanged=True,distinct_volunteer=True,paddler_doors_and_care_semantics_preserved=True,
                gameplay_acceptance=False)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('before');parser.add_argument('after');parser.add_argument('source')
    args=parser.parse_args()
    with open(args.source) as handle:source=json.load(handle)
    before,after=load_pack(args.before),load_pack(args.after)
    report=audit(before,after,source);negatives=[]
    for label in ['outdoor Dorothy drift','door drift','care operation drift','compiled text drift']:
        bad=dict(after,data=dict(after['data'],maps=dict(after['data']['maps'])))
        if label=='outdoor Dorothy drift':
            name=next(n for n,m in after['data']['maps'].items() if any(o['script']==SERVICE for o in m['objects']))
            module=deepcopy(after['data']['maps'][name]);next(o for o in module['objects'] if o['script']==SERVICE)['x']+=1;bad['data']['maps'][name]=module
        else:
            module=deepcopy(after['data']['maps'][ROOM]);bad['data']['maps'][ROOM]=module
            if label=='door drift':module['events']['warps'][0]['x']+=1
            elif label=='care operation drift':module['scripts'][NEW][4]['args']=['PokemonCenterPC']
            else:module['script_text_bodies'][NEW_TEXT]['commands'][0]['args']=['"I am DOROTHY."']
        try:audit(before,bad,source)
        except AssertionError:negatives.append(dict(mutation=label,rejected=True))
        else:raise AssertionError('accepted corruption: '+label)
    print(json.dumps(dict(report,negative_cases=negatives)))
