#!/usr/bin/env python3
"""Require the ten reviewed rooms, their native split data and the three flower placements."""
import argparse
import base64
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / 'tools'))
from audit_story_flow import require
from render_reference import load_pack

TRAIL='MnR19C00R00'
GATE='MnR19DetroitGate'
MAPS={TRAIL,GATE,'MnDetroitLakes','MnDetroitLakesRangerCabin','MnDuluth','MnDuluthHarborWorkshop',
      'MnBoundaryWaters','MnBoundaryWatersSurveyShelter','MnLeagueLobby','MnLeagueChampion'}


def changed(old,new):
    return {k for k in old.keys() | new.keys() if k not in old or k not in new or old[k]!=new[k]}


def check(base,final):
    require(changed(base,final)=={'data','identity','report'}, 'graphics, audio, archive format or other pack content changed')
    require(changed(base['report'],final['report'])=={'manifests'}
            and final['report']['manifests']==[*base['report']['manifests'],'northstar-lake-observations-v1'], 'build history')
    a,b=base['data'],final['data']
    require(changed(a,b)=={'maps','map_scripts','map_blocks'}, 'changes outside map scripts and terrain')
    require(changed(a['maps'],b['maps'])==MAPS, 'unexpected room changes')
    expected_labels=set()
    for name in sorted(MAPS):
        x,y=a['maps'][name],b['maps'][name]
        labels=changed(x['scripts'],y['scripts'])
        expected_labels |= labels
        for label in labels:
            require(b['map_scripts'].get(label)==y['scripts'].get(label), f'{name} split binding for {label}')
        allowed={'scripts','script_text_bodies','script_text_commands','script_control_commands','script_flag_commands','script_runtime_commands'}
        if name==TRAIL:
            allowed={'blocks'}
            width=x['attributes']['width']
            require(len(x['blocks'])==len(y['blocks']), 'route size')
            delta={(i%width,i//width,old,new) for i,(old,new) in enumerate(zip(x['blocks'],y['blocks'])) if old!=new}
            require(delta=={(8,22,2,4),(9,22,2,4),(11,23,2,4)}, 'native flower changes')
            key=x['attributes']['tileset_name']
            require(a['tilesets'][key]['collision']['02']==b['tilesets'][key]['collision']['04'], 'new flower collision')
        if name==GATE:
            allowed.update({'events','map_event_section_commands','script_movements','script_object_commands','script_variable_commands'})
            events=x['attributes']['map_events_label']
            expected_labels.add(events)
            require(changed(x['events'],y['events'])=={'coord_events'} and len(x['events']['coord_events'])==2 and not y['events']['coord_events'], 'gatehouse coordinates')
            require(b['map_scripts'][events]==[{k:r[k] for k in ['command','args']} for r in y['map_event_section_commands']], 'split gatehouse events')
        require(changed(x,y)<=allowed, f'{name} changed outside intended scripts: {changed(x,y)-allowed}')
    require(changed(a['map_scripts'],b['map_scripts'])==expected_labels,'unrelated or missing world script change')
    key=TRAIL+'_Blocks'
    require(changed(a['map_blocks'],b['map_blocks'])=={key} and base64.b64decode(b['map_blocks'][key])==bytes(b['maps'][TRAIL]['blocks']),'split route terrain')
    print(f"Compared all {len(b['maps'])} maps, {len(b['trainers']['trainers'])} trainer parties and {len(base['runtime_files'])} runtime files. Only ten reviewed rooms changed; all art, animations, warps, actors, item flags, habitats, battle teams and audio match.")


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent',type=Path)
    parser.add_argument('child',type=Path)
    paths=parser.parse_args()
    check(load_pack(paths.parent),load_pack(paths.child))
