#!/usr/bin/env python3
"""Read-only native fen preservation, collision and negative-control audit."""
import argparse
import base64
from copy import deepcopy
import json
from pathlib import Path
from render_reference import load_pack
PARENT = 'MnBemidjiNorthwoods'
FEN = 'MnLayeredFen'
CONSTANT = 'MN_LAYERED_FEN'
FLAGS = {'EVENT_MN_FEN_' + key for key in ['RIB_READ','POOL_READ','ROOT_READ','LAYERS_COMPARED','FOLDED_PATH_REVEALED','SEQUENCE_ROOT','SEQUENCE_POOL','OBSERVER_READ','CAUGHT']}
ROOT = Path(__file__).resolve().parents[3]


def check(parent, child, site):
    old, new = parent['data'], child['data']
    assert new['maps'].keys() - old['maps'].keys() == {FEN}
    outside, original, fen = new['maps'][PARENT], old['maps'][PARENT], new['maps'][FEN]
    attributes = deepcopy(original['attributes'])
    assert not any(c['direction'].lower() == 'west' for c in attributes['connections'])
    attributes['connections'].append(dict(direction='west', target_map=FEN, offset=0))
    attributes['connection_flags'] += ' | WEST'
    assert outside['attributes'] == attributes
    blocks = list(original['blocks'])
    assert blocks[64:67] == [5,5,1]
    blocks[64:66] = [1,1]
    assert outside['blocks'] == blocks
    normalized_outside = deepcopy(outside)
    normalized_outside['attributes'] = original['attributes']
    normalized_outside['blocks'] = original['blocks']
    assert normalized_outside == original
    assert (fen['attributes']['width'], fen['attributes']['height'], fen['attributes']['tileset_name']) == (20,13,'johto')
    assert fen['attributes']['connections'] == [dict(direction='east', target_map=PARENT, offset=0)]
    assert fen['attributes']['connection_flags'] == 'EAST'
    assert fen['blocks'] == [int(v,16) for row in site['fen']['rows'] for v in row.split()]
    assert {(o['script'],o['sprite'],o['x'],o['y']) for o in fen['objects']} == {
        ('MnFenObserver','SPRITE_SCIENTIST',36,17),('MN_FEN_FLARKINDLE','SPRITE_FLARKINDLE',32,5)}
    assert len(fen['objects']) == 2 and not fen['events']['warps'] and not fen['events']['coord_events']
    assert fen['events']['bg_events'] == [dict(x=r['at'][0],y=r['at'][1],event_type='BGEVENT_READ',script=r['script']) for r in site['fen']['reads'].values()]
    collision = new['tilesets']['johto']['collision']
    def kind(blocks,x,y):
        return collision[f"{blocks[y//2*20+x//2]:02x}"][y%2*2+x%2]
    def flood(blocks,surf=False):
        seen,pending={(39,16)},[(39,16)]
        while pending:
            x,y=pending.pop()
            for point in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if (0<=point[0]<40 and 0<=point[1]<26 and point not in seen and point not in [(36,17),(32,5)]
                    and kind(blocks,*point) in (['FLOOR','TALL_GRASS','WATER'] if surf else ['FLOOR','TALL_GRASS'])):
                    seen.add(point);pending.append(point)
        return seen
    closed=fen['blocks'];opened=list(closed);assert opened[93]==5;opened[93]=1
    before,after=flood(closed),flood(opened)
    assert len(before)==327
    assert len(after)==394
    for read in site['fen']['reads'].values():
        assert kind(closed,*read['at'])==read['kind'] and tuple(read['approach']) in before
        assert kind(closed,*read['approach'])=='FLOOR'
    assert (32,6) not in flood(closed,True) and (32,6) in after
    for blocks in [closed,opened]:
        assert all(kind(blocks,x,y) not in ['CUT_TREE','BUOY'] for y in range(26) for x in range(40))
        for x,y in flood(blocks,True):
            if x in [0,39] or y in [0,25]:assert x==39 and y in [16,17]
    wild=[(c['args'][0],c['args'][1]) for program in fen['scripts'].values() if isinstance(program,list) for c in program if c['command']=='loadwildmon']
    assert wild==[('FLARKINDLE','55')]
    assert fen['scripts']['MnFenRestoreTerrain']==[
        dict(command='checkevent',args=['EVENT_MN_FEN_FOLDED_PATH_REVEALED']),
        dict(command='iffalse',args=['MnFenTerrainEnd']),dict(command='changeblock',args=['26','8','1']),dict(command='end',args=[])]
    assert fen['scripts']['MnFenTiles']==[dict(command='scall',args=['MnFenRestoreTerrain']),dict(command='endcallback',args=[])]
    assert fen['scripts']['MnFenOpenPath']==[dict(command='scall',args=['MnFenRestoreTerrain']),dict(command='refreshmap',args=[]),dict(command='end',args=[])]
    assert PARENT in old['wild_encounters']
    for table in ['wild_encounters','field_encounters']:
        if PARENT in old[table]:
            expected=deepcopy(old[table][PARENT]);expected['map_name']=FEN
            assert new[table][FEN]==expected
        else:assert FEN not in new[table]
    for module in [outside, fen]:
        name = module['id']
        assert new['npcs'][name] == module['objects']
        assert new['map_attributes'][name] == module['attributes']
        assert new['map_blocks'][module['attributes']['blocks_label']] == base64.b64encode(bytes(module['blocks'])).decode()
        programs = dict(module['scripts'])
        for attr, section in [('map_scripts_label', 'map_script_section_commands'), ('map_events_label', 'map_event_section_commands')]:
            programs[module['attributes'][attr]] = [dict(command=row['command'], args=row['args']) for row in module[section]]
        assert all(new['map_scripts'][label] == commands for label, commands in programs.items())
    meta = deepcopy(old['runtime_map_metadata'][original['attributes']['map_constant']])
    meta.update(name=FEN, constant=CONSTANT, width=20, height=13, environment=fen['attributes']['environment'])
    meta['mapId'] = max(m['mapId'] for m in old['runtime_map_metadata'].values() if m['groupId'] == meta['groupId']) + 1
    assert new['runtime_map_metadata'][CONSTANT] == meta
    before, after = old['story_event_script_constants']['global'], new['story_event_script_constants']['global']
    assert after.keys() - before.keys() == FLAGS
    assert all(after.get(k) == v for k, v in before.items())
    ids = {after[k] for k in FLAGS}
    assert len(ids) == 9 and not ids & {v for k, v in before.items() if k.startswith('EVENT_')}
    report = deepcopy(parent['report'])
    report['maps'] += 1
    report['reachable_maps'].append(FEN)
    report['solvable_maps'].append(FEN)
    report['manifests'].extend(['northstar-lair-mnlayeredfen', 'minnesota-flarkindle-layered-fen-v1'])
    assert child['report'] == report
    normalized = deepcopy(child)
    data = normalized['data']
    for table, key in [('maps', FEN), ('map_attributes', FEN), ('map_dimensions', CONSTANT),
                       ('map_blocks', fen['attributes']['blocks_label']), ('runtime_map_metadata', CONSTANT),
                       ('npcs', FEN)]:
        del data[table][key]
    for table in ['wild_encounters', 'field_encounters']:
        if PARENT in old[table]:
            del data[table][FEN]
    assert data['pokegear_landmarks']['map_to_landmark'].pop(FEN) == old['pokegear_landmarks']['map_to_landmark'][PARENT]
    for table, key in [('maps', PARENT), ('map_attributes', PARENT), ('map_blocks', original['attributes']['blocks_label']), ('npcs', PARENT)]:
        data[table][key] = deepcopy(old[table][key])
    labels = set(outside['scripts']) | set(fen['scripts'])
    for module in [outside, fen]:
        for attr in ['map_scripts_label', 'map_events_label']:
            labels.add(module['attributes'][attr])
    assert new['map_scripts'].keys() - old['map_scripts'].keys() == labels - old['map_scripts'].keys()
    for label in labels:
        if label in old['map_scripts']:
            data['map_scripts'][label] = deepcopy(old['map_scripts'][label])
        else:
            del data['map_scripts'][label]
    for flag in FLAGS:
        del data['story_event_script_constants']['global'][flag]
    normalized['report'], normalized['identity'] = parent['report'], parent['identity']
    assert normalized == parent, 'unrelated world, scripts, encounters, art, species or audio changed'


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent');parser.add_argument('child');args=parser.parse_args()
    parent,child=load_pack(args.parent),load_pack(args.child)
    site=json.loads((ROOT/'planning/minnesota/pokemon/acquisition-v01/flarkindle/site.json').read_text())
    check(parent,child,site)
    mutations=[
        ('old corridor',lambda p:p['data']['maps'][PARENT]['blocks'].__setitem__(0,1)),
        ('old program',lambda p:p['data']['map_scripts'].__setitem__('AUDIT_FOREIGN_PROGRAM',[])),
        ('gate',lambda p:p['data']['maps'][FEN]['blocks'].__setitem__(93,1)),
        ('terrain callback',lambda p:p['data']['maps'][FEN]['scripts']['MnFenTiles'].clear()),
        ('species',lambda p:p['data']['pokemon'].__setitem__('AUDIT_FOREIGN',{})),
        ('audio',lambda p:p.__setitem__('audio_manifest',None)),
    ]
    for label,mutate in mutations:
        changed=deepcopy(child);mutate(changed)
        try:check(parent,changed,site)
        except (AssertionError,KeyError,TypeError):pass
        else:raise AssertionError('accepted mutation: '+label)
        del changed
    print(f'PASS: {len(parent["data"]["maps"])-1} unrelated maps, original corridor/travel, art/species/audio exact; closed/open/Surf gate and six read approaches; six mutations rejected')


if __name__=='__main__':main()
