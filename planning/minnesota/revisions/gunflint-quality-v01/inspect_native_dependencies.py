#!/usr/bin/env python3
"""Inspect the local pack into a bounded planning inventory; never writes packs."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import load_pack

PACK = ROOT / 'content-packs/minnesota-superior-inland-winter-v01.crystalpack'
EXPECTED = '65cf6f98e65ada839ec5afaa57728bf8289d3a50ba53ba6b9b50f08d3620bb17'
OUTPUT = Path(__file__).with_name('native-dependencies-v01.md')


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()


def cell(value):
    return str(value).replace('|', '\\|').replace('\n', ' ')


def table(headers, rows):
    return ['| ' + ' | '.join(headers) + ' |', '| ' + ' | '.join('---' for _ in headers) + ' |'] + ['| ' + ' | '.join(cell(v) for v in row) + ' |' for row in rows]


def references(value, tokens, path=''):
    if isinstance(value, dict):
        for key, child in value.items():
            yield from references(child, tokens, f'{path}/{key}')
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from references(child, tokens, f'{path}/{index}')
    elif isinstance(value, str) and value in tokens:
        yield path, value


def main():
    assert hashlib.sha256(PACK.read_bytes()).hexdigest() == EXPECTED
    pack = load_pack(PACK)
    data = pack['data']
    maps = data['maps']
    names = sorted(n for n in maps if n.startswith('MnR12')) + ['MnMaraisInlandNeck', 'MnMaraisCedarApproach']
    constants = {maps[n]['attributes']['map_constant']: n for n in names}
    tokens = set(names) | set(constants)
    resolve = lambda v: constants.get(v, v)
    out = ['# Native R12 / inland connector dependency inventory', '',
           f'Inspected native parent: `{PACK.relative_to(ROOT)}`; file SHA256 `{EXPECTED}`.',
           f'Content identity: `{pack["identity"]["content_hash"]}`. No pack writes, source promotions or Cargo runs are performed by this inspection.', '',
           'This is a bounded relocation planning inventory, not a replacement program catalog. Coordinates below are 16px gameplay tiles unless dimensions explicitly say metatiles. Hashes use sorted-key compact JSON of each existing native module. Geometry and retained/retired choices are not approved by this inventory.', '',
           '## Native identities and preimages', '']
    rows=[]
    for n in names:
        m=maps[n];a=m['attributes'];meta=data['runtime_map_metadata'][a['map_constant']]
        rows.append([n,a['map_constant'],f'{meta["groupId"]}:{meta["mapId"]}',f'{a["width"]}×{a["height"]}',a['tileset_name'],digest(m)])
    out += table(['Map','Constant','Group:map','Metatile extent','Tileset','Whole module SHA256'],rows)
    out += ['', '## Every incoming and outgoing native connection', '']
    rows=[]
    for n,m in sorted(maps.items()):
        for c in m['attributes']['connections']:
            if n in names or c['target_map'] in names:
                rows.append([n,c['direction'],c['target_map'],c['offset']])
    out += table(['Source','Direction','Target','Offset'],rows)
    out += ['', '## Every incoming and outgoing native warp', '']
    rows=[]
    for n,m in sorted(maps.items()):
        for w in m['events']['warps']:
            target=resolve(w.get('target_map_constant') or w.get('target_map'))
            if n in names or target in names:
                rows.append([n,w['index'],f'({w["x"]},{w["y"]})',target,w['target_warp_id']])
    out += table(['Source','Warp index','Local tile','Target','Target warp'],rows)
    out += ['', 'The cave exterior mouth currently belongs to `MnR12C01R00` (global R12 `(66,23)`), with native passive dry return `(66,24)`. The hut mouth belongs to `MnR12C01R01` (global `(85,83)`), with native passive return `(85,84)`. Both hut carpets return to the same exterior warp index 1. Moving either mouth requires rebinding its existing room return(s) to the new owning map and exterior warp index; the interior identities and programs can remain unchanged.', '',
            '## Spawns, recovery and non-map registries', '']
    spawns=[(k,v) for k,v in data['runtime_spawn_points'].items() if v['mapName'] in names or v['mapConstant'] in constants]
    out.append(f'Runtime spawn/recovery entries targeting these maps: **{len(spawns)}**. The native spawn catalog was checked by both map name and map constant. No spawn identifier needs migration for these maps in this exact parent.')
    rows=[]
    for field in ['runtime_title_screen','fly_destinations','playability','field_box_items','story_events','phone_scripts','global_scripts','special_routines']:
        for path,value in references(data.get(field),tokens,field):
            rows.append([path,value])
    out += ['', 'Additional exact map-name/constant references in title, Fly, playability, field-box items, story, phone/global scripts and special routines:', '']
    out += table(['Source path','Reference'],rows) if rows else ['None in those registries.']
    out += ['', '## Script map references and coordinate-bearing instructions', '']
    rows=[]
    for n,m in sorted(maps.items()):
        for field in ['scripts','script_map_commands','script_object_commands','script_block_changes','script_variable_commands','script_runtime_commands','script_control_commands']:
            for path,value in references(m[field],tokens,f'{n}/{field}'):
                rows.append([path,value])
    for path,value in references(data['map_scripts'],tokens,'global split map_scripts'):
        rows.append([path,value])
    out += table(['Source path','Map reference'],rows) if rows else ['No map references to these names/constants exist in cached raw programs, typed script command tables, or global split scripts. Native map links are therefore the connections and warps listed above.']
    coordinate_opcodes={'warp','warpfacing','warpmod','blackoutmod','moveobject','moveperson','changeblock','applymovement','applymovementlasttalked','follow','stopfollow','writecmdqueue','delcmdqueue','xycompare'}
    rows=[]
    for n in names:
        m=maps[n]
        for field in ['script_map_commands','script_object_commands','script_block_changes','script_movements']:
            if m[field]: rows.append([n,field,'nonempty: inspect exact typed preimage before relocation'])
        for label,commands in m['scripts'].items():
            for i,command in enumerate(commands):
                if command.get('command') in coordinate_opcodes:
                    rows.append([n,f'{label}[{i}]',f'{command["command"]} args={command.get("args",[])}'])
    out += ['', 'Coordinate-bearing program instruction/table scan (event-header coordinates are inventoried separately below):', '']
    out += table(['Map','Source','Coordinate dependency'],rows) if rows else ['No coordinate-bearing script instructions or nonempty typed map/object/block/movement tables occur in the ten inventoried modules. There are no map scenes or coordinate events in these modules. Terrain interactions such as Cut use native collision, not a bespoke coordinate program.']
    out += ['', '## Existing actors, encounter flags and pickup identities', '']
    rows=[]
    global_flags=data['story_event_script_constants']['global']
    for n in names:
        m=maps[n]
        match=re.fullmatch(r'MnR12C(\d\d)R(\d\d)',n)
        dx,dy=(int(match[1])*64,int(match[2])*48) if match else (0,0)
        for o in m['objects']:
            ident=o.get('object_identifier') or o['script']
            trainer=m['trainer_scripts'].get(o['script'])
            flag=trainer['event_flag'] if trainer else o['event_flag']
            pickup=next((p for p in m['script_field_pickups'] if p['source_script']==o['script']),None)
            activity=f'{pickup["item_id"]}×{pickup["quantity"]}' if pickup else (f'{trainer["trainer_class"]}/{trainer["trainer_id"]}' if trainer else 'existing dialogue/service')
            rows.append([n,ident,f'({o["x"]},{o["y"]})',f'({o["x"]+dx},{o["y"]+dy})' if match else 'not R12',o['spritemovedata'],activity,f'{flag}={global_flags.get(flag,"not global / no flag")}'])
    out += table(['Owner','Actor/script identity','Local tile','Whole R12 tile','Facing/movement','Activity','Event flag / native number'],rows)
    out += ['', '## Background signs and text-program guards', '']
    rows=[]
    for n in names:
        m=maps[n]
        for sign in m['events']['bg_events']:
            labels=sorted({c['text_label'] for c in m['script_text_commands'] if c.get('source_script')==sign['script'] and c.get('text_label')})
            rows.append([n,sign['script'],f'({sign["x"]},{sign["y"]})',', '.join(labels)])
    out += table(['Owner','Sign script','Local tile','Native text label(s)'],rows)
    out += ['', 'Program tables to preserve/move with their owning actors (hashes certify content without exporting a duplicate command catalog):', '']
    rows=[]
    for n in names:
        m=maps[n]
        counts=f'{len(m["scripts"])} script labels; {len(m["trainer_scripts"])} trainers; {len(m["script_text_bodies"])} text bodies'
        program={k:v for k,v in m.items() if k not in ['id','attributes','blocks','objects','events','map_script_section_commands','map_event_section_commands']}
        rows.append([n,counts,digest(program)])
    out += table(['Owner','Installed content','Retained program tables SHA256'],rows)
    out += ['', '## Feasible chunk retirement, without choosing geometry', '',
            'The current R12 footprint is 64×56 metatiles: six exterior chunks. Rooms `MnR12Cave1F` (42:7) and `MnR12RangerHut` (42:8) remain separately installed and need no internal resizing.', '',
            '- A 32×32 route using the established 32×24 clipping grid has two exterior chunks: retain `MnR12C00R00` (42:1), resize `MnR12C00R01` (42:3) to 32×8. Retire four exterior chunks: `C01R00`, `C01R01`, `C00R02`, `C01R02`. Existing actors/program labels/flags from retired chunks must migrate to explicit legal new anchors. Both room returns currently target retired right-column chunks and must be rebound.',
            '- A 32×24 route has one exterior chunk: retain `MnR12C00R00` (42:1), retire the other five. Keeping two exterior chunks at that extent would require a different deliberately specified clipping scheme; it is not the existing 32×24 chunk contract.',
            '- Retiring `MnMaraisInlandNeck` is an additional retirement, independent of the R12 chunk count. Its BERRY×3 flag2387 pickup and campsite sign are existing content. Root must explicitly choose relocation or retirement of that activity; removing a map must not silently discard its reward or reset its flag.',
            '- Cedar (83:2) and Neck (40:8) each have distinct existing numeric identities. Neither a proposed direct port nor a revised dimension is frozen by this inventory.', '',
            'For each retired map, remove its uniquely owned cached module, raw attributes/dimensions, metadata, blocks, NPC wire entry, encounter entry, landmark mapping and owned split labels only after migrating retained programs/events. Keep group/map numbers of retained identities stable; do not renumber unrelated maps. Reject every dangling warp, connection, typed/raw map-command, spawn/recovery and playability reference. Rebuild report map membership/counts only within the explicit removal scope. Full inverse restoration must recover the original parent, including art/audio, programs, catalogs and recovery data.', '',
            'Old saves may contain visited-map/position/override data for retired identities; resizing is not a save-compatibility claim. Production tests must use fresh fixtures bound to the resulting pack and retain all reached saves.', '']
    OUTPUT.write_text('\n'.join(out))
    print(f'Wrote {OUTPUT.relative_to(ROOT)}; {len(names)} existing native maps; no pack mutation.')


if __name__ == '__main__':
    main()
