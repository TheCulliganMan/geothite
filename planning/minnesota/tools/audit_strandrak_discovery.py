#!/usr/bin/env python3
"""Independent read-only two-map basin, preservation and mutation audit."""
import argparse
import base64
from copy import deepcopy
import json
from pathlib import Path
from render_reference import load_pack
PARENT='MnR16C00R00'
RIDGE='MnFormerShores'
BASIN='MnInnerBasin'
CONSTANTS={RIDGE:'MN_FORMER_SHORES',BASIN:'MN_INNER_BASIN'}
FLAGS={'EVENT_MN_BASIN_'+key for key in [
    'HIGH_SHORE_READ','MIDDLE_SHORE_READ','LOW_SHORE_READ','SHORES_COMPARED',
    'INNER_OUTLET_ACKNOWLEDGED','SURVEYOR_READ','CAUGHT',
    *[dial+'_'+suffix for dial in ['UPPER','CENTER','LOWER'] for suffix in ['MIDDLE','HIGH','TOUCHED']]]}
ROOT=Path(__file__).resolve().parents[3]


def warp(index,x,y,target,destination):
    return dict(index=index,x=x,y=y,target_map_constant=target,target_map=target,target_warp_id=destination)


def check(parent,child,site):
    old,new=parent['data'],child['data']
    assert new['maps'].keys()-old['maps'].keys()=={RIDGE,BASIN}
    outside,original,ridge,basin=[new['maps'][PARENT],old['maps'][PARENT],new['maps'][RIDGE],new['maps'][BASIN]]
    attributes=deepcopy(original['attributes'])
    assert not any(c['direction'].lower()=='south' for c in attributes['connections'])
    attributes['connections'].append(dict(direction='south',target_map=RIDGE,offset=14))
    attributes['connection_flags']+=' | SOUTH'
    assert outside['attributes']==attributes
    blocks=list(original['blocks']);assert blocks[757]==5;blocks[757]=2
    assert outside['blocks']==blocks
    normalized=deepcopy(outside);normalized['attributes']=original['attributes'];normalized['blocks']=original['blocks']
    assert normalized==original
    assert ridge['attributes']['connections']==[dict(direction='north',target_map=PARENT,offset=-14)]
    assert ridge['attributes']['connection_flags']=='NORTH'
    assert not basin['attributes']['connections'] and basin['attributes']['connection_flags'] is None
    for field in ['environment','palette','music']:
        assert basin['attributes'][field]==old['maps']['SlowpokeWellB1F']['attributes'][field]
    assert basin['attributes']['palette']=='PALETTE_NITE'
    for module,key,width in [(ridge,'ridge',18),(basin,'basin',14)]:
        assert [module['attributes']['width'],module['attributes']['height']]==[width,14]
        assert module['attributes']['tileset_name']==site[key]['tileset']
        assert module['blocks']==[int(v,16) for row in site[key]['rows'] for v in row.split()]
        assert not module['events']['coord_events']
        assert module['events']['bg_events']==[dict(x=r['at'][0],y=r['at'][1],event_type='BGEVENT_READ',script=r['script'])
                                               for r in site[key]['reads' if key=='ridge' else 'controls'].values()]
    assert len(ridge['objects'])==1 and len(basin['objects'])==1
    for module,label,sprite,x,y in [(ridge,'MnBasinSurveyor','SPRITE_SCIENTIST',17,5),(basin,'MN_BASIN_STRANDRAK','SPRITE_STRANDRAK',14,2)]:
        actor=module['objects'][0];assert (actor['script'],actor['sprite'],actor['x'],actor['y'])==(label,sprite,x,y)
    assert ridge['events']['warps']==[warp(1,18,23,CONSTANTS[BASIN],2),warp(2,18,24,CONSTANTS[RIDGE],2)]
    assert basin['events']['warps']==[warp(1,3,25,CONSTANTS[RIDGE],2),warp(2,2,25,CONSTANTS[BASIN],2)]
    def kind(module,blocks,x,y):
        width=module['attributes']['width'];coll=new['tilesets'][module['attributes']['tileset_name']]['collision']
        return coll[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]
    def flood(module,blocks,start,surf=False):
        width,height=module['attributes']['width']*2,module['attributes']['height']*2
        occupied={(o['x'],o['y']) for o in module['objects']};seen={start};pending=[start]
        while pending:
            x,y=pending.pop()
            for point in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if (0<=point[0]<width and 0<=point[1]<height and point not in seen and point not in occupied
                    and kind(module,blocks,*point) in (['FLOOR','TALL_GRASS','LADDER','CAVE','WATER'] if surf else ['FLOOR','TALL_GRASS','LADDER','CAVE'])):
                    seen.add(point);pending.append(point)
        return seen
    ridge_reach=flood(ridge,ridge['blocks'],(14,0));assert len(ridge_reach)==482
    for read in site['ridge']['reads'].values():
        assert kind(ridge,ridge['blocks'],*read['at'])=='UP_WALL'
        assert tuple(read['approach']) in ridge_reach
    assert (17,6) in ridge_reach and (18,23) in ridge_reach and (18,24) in ridge_reach
    closed=flood(basin,basin['blocks'],(2,25),True);assert len(closed)==333
    for control in site['basin']['controls'].values():
        assert kind(basin,basin['blocks'],*control['at'])=='WALL'
        assert tuple(control['approach']) in closed
    assert (14,3) not in closed
    opened=list(basin['blocks']);changes=[]
    for row in site['basin']['terrain_changes']:
        x,y=row['metatile'];assert opened[y*14+x]==int(row['before'],16);opened[y*14+x]=2
        changes.append(dict(command='changeblock',args=[str(x*2),str(y*2),'2']))
    assert len(changes)==9
    restore=[dict(command='checkevent',args=['EVENT_MN_BASIN_INNER_OUTLET_ACKNOWLEDGED']),
             dict(command='iffalse',args=['MnBasinTerrainEnd']),*changes,dict(command='end',args=[])]
    assert basin['scripts']['MnBasinRestoreTerrain']==restore
    assert basin['scripts']['MnBasinTiles']==[dict(command='scall',args=['MnBasinRestoreTerrain']),dict(command='endcallback',args=[])]
    assert basin['scripts']['MnBasinOpenInnerOutlet']==[dict(command='scall',args=['MnBasinRestoreTerrain']),dict(command='refreshmap',args=[]),dict(command='end',args=[])]
    reached=flood(basin,opened,(2,25));assert len(reached)==432 and (14,3) in reached
    for module,blocks in [(ridge,ridge['blocks']),(basin,basin['blocks']),(basin,opened)]:
        width,height=module['attributes']['width']*2,module['attributes']['height']*2
        for y in range(height):
            for x in range(width):
                k=kind(module,blocks,x,y);assert k not in ['CUT_TREE','BUOY','PIT']
                if x in [0,width-1] or y in [0,height-1]:assert k=='WALL' or (module['id']==RIDGE and y==0 and x in [14,15])
    wild=[(c['args'][0],c['args'][1]) for program in basin['scripts'].values() if isinstance(program,list) for c in program if c['command']=='loadwildmon']
    assert wild==[('STRANDRAK','60')]
    assert not ridge['scripted_wild_battles']
    expected=deepcopy(old['wild_encounters'][PARENT]);expected['map_name']=RIDGE
    assert new['wild_encounters'][RIDGE]==expected and BASIN not in new['wild_encounters']
    for module in [outside,ridge,basin]:
        name=module['id'];assert new['npcs'][name]==module['objects'];assert new['map_attributes'][name]==module['attributes']
        assert new['map_blocks'][module['attributes']['blocks_label']]==base64.b64encode(bytes(module['blocks'])).decode()
        programs=dict(module['scripts'])
        for attr,section in [('map_scripts_label','map_script_section_commands'),('map_events_label','map_event_section_commands')]:
            programs[module['attributes'][attr]]=[dict(command=row['command'],args=row['args']) for row in module[section]]
        assert all(new['map_scripts'][label]==commands for label,commands in programs.items())
    anchor=old['runtime_map_metadata'][original['attributes']['map_constant']]
    next_id=max(m['mapId'] for m in old['runtime_map_metadata'].values() if m['groupId']==anchor['groupId'])
    for offset,module in enumerate([ridge,basin],1):
        meta=deepcopy(anchor);meta.update(name=module['id'],constant=CONSTANTS[module['id']],width=module['attributes']['width'],height=14,
            environment=module['attributes']['environment'],mapId=next_id+offset)
        assert new['runtime_map_metadata'][CONSTANTS[module['id']]]==meta
    before,after=old['story_event_script_constants']['global'],new['story_event_script_constants']['global']
    assert after.keys()-before.keys()==FLAGS and all(after.get(k)==v for k,v in before.items())
    ids={after[k] for k in FLAGS};assert len(ids)==16 and not ids&{v for k,v in before.items() if k.startswith('EVENT_')}
    report=deepcopy(parent['report']);report['maps']+=2
    for name in [RIDGE,BASIN]:report['reachable_maps'].append(name);report['solvable_maps'].append(name)
    report['manifests'].extend(['northstar-lair-mnformershores','northstar-lair-mninnerbasin','minnesota-strandrak-former-shores-v1'])
    assert child['report']==report
    normalized=deepcopy(child);data=normalized['data']
    for module in [ridge,basin]:
        name=module['id']
        for table,key in [('maps',name),('map_attributes',name),('map_dimensions',CONSTANTS[name]),('map_blocks',module['attributes']['blocks_label']),('runtime_map_metadata',CONSTANTS[name]),('npcs',name)]:del data[table][key]
        assert data['pokegear_landmarks']['map_to_landmark'].pop(name)==old['pokegear_landmarks']['map_to_landmark'][PARENT]
    del data['wild_encounters'][RIDGE]
    for table,key in [('maps',PARENT),('map_attributes',PARENT),('map_blocks',original['attributes']['blocks_label']),('npcs',PARENT)]:data[table][key]=deepcopy(old[table][key])
    labels=set(outside['scripts'])|set(ridge['scripts'])|set(basin['scripts'])
    for module in [outside,ridge,basin]:
        for attr in ['map_scripts_label','map_events_label']:labels.add(module['attributes'][attr])
    assert new['map_scripts'].keys()-old['map_scripts'].keys()==labels-old['map_scripts'].keys()
    for label in labels:
        if label in old['map_scripts']:data['map_scripts'][label]=deepcopy(old['map_scripts'][label])
        else:del data['map_scripts'][label]
    for flag in FLAGS:del data['story_event_script_constants']['global'][flag]
    normalized['identity'],normalized['report']=parent['identity'],parent['report']
    assert normalized==parent,'outside waters, world, scripts, art, species or audio changed'


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('parent');parser.add_argument('child');args=parser.parse_args()
    parent,child=load_pack(args.parent),load_pack(args.child)
    site=json.loads((ROOT/'planning/minnesota/pokemon/acquisition-v01/strandrak/site.json').read_text());check(parent,child,site)
    mutations=[('outside water',lambda p:p['data']['maps'][PARENT]['blocks'].__setitem__(648,2)),
        ('sill',lambda p:p['data']['maps'][BASIN]['blocks'].__setitem__(49,2)),
        ('old program',lambda p:p['data']['map_scripts'].__setitem__('AUDIT_FOREIGN',[])),
        ('terrain restoration',lambda p:p['data']['maps'][BASIN]['scripts']['MnBasinTiles'].clear()),
        ('foreign species',lambda p:p['data']['pokemon'].__setitem__('AUDIT_FOREIGN',{})),
        ('audio',lambda p:p.__setitem__('audio_manifest',None))]
    for label,mutate in mutations:
        changed=deepcopy(child);mutate(changed)
        try:check(parent,changed,site)
        except (AssertionError,KeyError,TypeError):pass
        else:raise AssertionError('accepted mutation: '+label)
        del changed
    print(f'PASS: {len(parent["data"]["maps"])-1} unrelated maps and outside waters exact; three shores, controls, reciprocal travel and nine local changes; original art/species/audio exact; six mutations rejected')


if __name__=='__main__':main()
