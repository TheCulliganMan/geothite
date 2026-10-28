#!/usr/bin/env python3
"""Inventory native exterior water mouths using production numeric permissions.

Read-only static evidence. Forced currents, event gates and earned HM/badge
progression still require production controller tests. Never writes a pack.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack
from audit_outdoor_connections import project

DIRS = {'west':(-1,0),'east':(1,0),'north':(0,-1),'south':(0,1)}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def production_permissions():
    collision_path = ROOT/'crates/crystal-core/src/world/collision.rs'
    parser_path = ROOT/'crates/crystal-assets/src/script_parsing.rs'
    bridge_path = ROOT/'crates/crystal-core/src/world/bridge.rs'
    collision, parser, bridge = [p.read_text() for p in [collision_path,parser_path,bridge_path]]
    constants = {k:int(v,16)for k,v in re.findall(r'pub const (\w+): u8 = 0x([0-9a-f]+);',collision)}
    bridge_constants = {k:int(v,16)for k,v in re.findall(r'pub const (\w+): u8 = 0x([0-9a-f]+);',bridge)}
    function = parser.split('fn resolve_collision_token(',1)[1].split('fn parse_metatile_id(',1)[0]
    tokens = {}
    for name,value in re.findall(r'"([A-Z0-9_]+)" => ([^,\n]+)',function):
        if value.startswith('0x'):
            tokens[name] = int(value,16)
        elif value.startswith('permissions::'):
            tokens[name] = constants[value.split('::')[-1]]
        elif value.startswith('crystal_core::world::bridge::'):
            tokens[name] = bridge_constants[value.split('::')[-1]]
        else:
            raise AssertionError(('Unknown production token expression',name,value))
    body = collision.split('pub fn describe_collision(',1)[1].split('pub fn sample_collision(',1)[0]
    def values(text):
        result = set()
        for first,last in re.findall(r'0x([0-9a-f]+)(?:\.\.=0x([0-9a-f]+))?',text):
            a,b = int(first,16),int(last or first,16)
            result.update(range(a,b+1))
        return result
    water_expression = re.search(r'([\s0-9a-fx|.=]+) => \{\s*Terrain::Water',body).group(1)
    wall_expression = re.search(r'([\s0-9a-fx|.=]+) => Terrain::Wall',body).group(1)
    water,wall = values(water_expression),values(wall_expression)
    assert 0x21 in water and 0x23 not in water and 0x27 in wall
    assert set(range(0x30,0x40)) <= water and set(range(0xc0,0xd0)) <= water
    return tokens,water,wall,{str(p.relative_to(ROOT)):sha(p.read_bytes()) for p in [collision_path,parser_path,bridge_path]}


def numeric(token,tokens):
    return int(token,16) if re.fullmatch('[0-9A-Fa-f]+',token) else tokens[token]


def blocked(value,direction,leaving=False):
    if value & 0xf0 not in {0xb0,0xc0}:
        return False
    masks = {'south':{3,4,5},'north':{2,6,7},'west':{1,5,7},'east':{0,4,6}} if leaving else {
        'south':{2,6,7},'north':{3,4,5},'west':{0,4,6},'east':{1,5,7}}
    return value & 7 in masks[direction]


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--pack',type=Path,required=True)
    ap.add_argument('--output',type=Path,required=True)
    args = ap.parse_args()
    assert not args.output.exists()
    args.output.mkdir(parents=True)
    tokens,water,wall,source_hashes = production_permissions()
    pack = load_pack(args.pack)
    maps = pack['data']['maps']
    placements = json.loads((ROOT/'planning/minnesota/renders/ice/native-geography-v03/atlas/native-connection-audit.json').read_bytes())['placements']
    exteriors = {r['map'] for r in placements}
    blocked_mounts = set(pack['data']['field_moves']['surf']['blocked_collisions'])
    summaries = []
    for name in sorted(exteriors):
        m = maps[name];a=m['attributes'];width,height=a['width']*2,a['height']*2
        tile = {}
        for y in range(height):
            for x in range(width):
                b = m['blocks'][(y//2)*a['width']+x//2]
                token = pack['data']['tilesets'][a['tileset_name']]['collision'][f'{b:02x}'][(y%2)*2+x%2]
                tile[x,y] = numeric(token,tokens)
        # Every mount witness starts on a native land tile and honors entry,
        # departure and actual pack Surf exclusions. It is a local geometric
        # possibility, not proof this land tile is reachable from campaign entry.
        launches = []
        bodies = {(o['x'],o['y'])for o in m['objects']}
        for p,value in tile.items():
            if value in water or value in wall or value in range(0xe0,0xe4) or p in bodies:
                continue
            for direction,(dx,dy) in DIRS.items():
                q=p[0]+dx,p[1]+dy
                other=tile.get(q)
                if other in water and other not in blocked_mounts and not blocked(value,direction,True) and not blocked(other,direction):
                    launches.append((p,q,direction))
        wet = {p for p,v in tile.items() if v in water}
        components=[]
        while wet:
            seed=min(wet);wet.remove(seed);component={seed};queue=deque([seed])
            while queue:
                p=queue.popleft()
                for direction,(dx,dy) in DIRS.items():
                    q=p[0]+dx,p[1]+dy
                    if q in wet and not blocked(tile[p],direction,True) and not blocked(tile[q],direction):
                        wet.remove(q);component.add(q);queue.append(q)
            mouths=[]
            for side,points in [('west',[(0,y)for y in range(height)]),('east',[(width-1,y)for y in range(height)]),('north',[(x,0)for x in range(width)]),('south',[(x,height-1)for x in range(width)])]:
                edge=next((e for e in a['connections'] if e['direction']==side),None)
                for p in points:
                    if p not in component or blocked(tile[p],side,True):
                        continue
                    target=None
                    if edge:
                        q=project(a,maps[edge['target_map']]['attributes'],edge,p)
                        target=dict(map=edge['target_map'],core=list(q))
                        other=maps[edge['target_map']];oa=other['attributes']
                        if 0<=q[0]<oa['width']*2 and 0<=q[1]<oa['height']*2:
                            b=other['blocks'][(q[1]//2)*oa['width']+q[0]//2]
                            token=pack['data']['tilesets'][oa['tileset_name']]['collision'][f'{b:02x}'][(q[1]%2)*2+q[0]%2]
                            value=numeric(token,tokens)
                            target.update(permission=f'{value:02x}',terrain='water' if value in water else 'wall' if value in wall else 'land',
                                          directional_entry_blocked=blocked(value,side))
                        else:
                            target['terrain']='OUTSIDE_NEIGHBOR_RECTANGLE'
                    mouths.append(dict(side=side,core=list(p),permission=f'{tile[p]:02x}',target=target))
            component_launches=[dict(from_core=list(p),water_core=list(q),direction=d)for p,q,d in launches if q in component]
            if mouths:
                components.append(dict(water_tiles=len(component),mouths=mouths,local_mount_witnesses=component_launches,
                    classification='LOCALLY_MOUNTABLE_WATER' if component_launches else 'NO_LOCAL_LAND_TO_WATER_MOUNT_WITNESS'))
        if components:
            summaries.append(dict(map=name,dimensions_core=[width,height],tileset=a['tileset_name'],connections=a['connections'],components=components))
    result=dict(schema_version=1,status='NUMERIC_STATIC_INVENTORY_NOT_CONTROLLER_PROOF',
        parent_pack_sha256=sha(args.pack.read_bytes()),source_hashes=source_hashes,exterior_count=len(exteriors),
        method='Collision token resolution and water/wall classification parsed from production Rust; directional masks mirrored explicitly; current forced movement, ledges, script gates and cross-map reachability are not simulated.',
        surf_blocked_mount_permissions=sorted(blocked_mounts),maps=summaries)
    (args.output/'numeric-water-edge-inventory.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(maps_with_water_mouths=len(summaries),water_components=sum(len(m['components'])for m in summaries))))


if __name__=='__main__':
    main()
