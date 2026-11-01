#!/usr/bin/env python3
"""Read-only three-way geometry/source inventory; never writes packs or sources."""
import argparse
import base64
from collections import Counter
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re

import cbor2
from render_reference import layout_blocks, load_pack

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / 'planning/minnesota'
QUALITY_SHA = '6071912eeb2fde6982f9e857bed9157d8d86da0f7ae56dba38665c809ab100e1'
STORY_SHA = 'da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50'
NEW_TOWNS = ['MnWinona', 'MnRochester', 'MnMankato', 'MnMinneapolis', 'MnStPaul', 'MnStCloud']
OLD_TOWNS = ['MnGrandMarais', 'MnBoundaryWaters', 'MnBrainerd', 'MnBemidji', 'MnDuluth', 'MnGrandRapids']
CAVE_IDS = ['MnR01Cave1F', 'MnR03Cave1F', 'MnR05RockCut', 'MnR09Cave1F', 'MnR10Cave1F', 'MnR12Cave1F', 'MnR13Cave1F', 'MnR17Cave1F', 'MnR20Cave1F']
CONTRACTS = ['playable/rochester-links.json', 'playable/prairie-story.json', 'playable/river-city-links.json', 'playable/gym-four-approach.json', 'playable/courier-story.json', 'playable/lake-country-story.json']


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def fingerprint(value):
    return sha(cbor2.dumps(value, canonical=True))


def actor_roster(module, origin=(0, 0)):
    return [dict(id=o.get('object_identifier') or o['script'], tile=[o['x']+origin[0], o['y']+origin[1]], sprite=o['sprite'], facing=o['spritemovedata'], event_flag=o['event_flag'], complete_record_sha256=fingerprint(o)) for o in module['objects']]


def source_actor_conflicts(source, modules, pack, exterior):
    native = {}
    for name, module in modules.items():
        match = re.fullmatch(r'MnR\d\dC(\d\d)R(\d\d)', name)
        ox, oy = (int(match[1])*64, int(match[2])*48) if exterior and match else (0, 0)
        for obj in module['objects']:
            native[obj.get('object_identifier') or obj['script']] = (module, obj, [obj['x']+ox, obj['y']+oy])
    conflicts = []
    for role in ['trainers', 'residents', 'items']:
        for actor in source.get(role, []):
            identifier = actor.get('id')
            if not identifier:
                conflicts.append(dict(id=None, role=role, issue='source actor has no stable id', source_tile=actor.get('tile')))
                continue
            if identifier not in native:
                conflicts.append(dict(id=identifier, role=role, issue='source actor missing natively'))
                continue
            module, obj, tile = native[identifier]
            expected = dict(tile=actor['tile'], sprite=actor.get('sprite', 'SPRITE_POKE_BALL'))
            actual = dict(tile=tile, sprite=obj['sprite'])
            if role != 'items' or 'facing' in actor:
                expected['facing'] = 'SPRITEMOVEDATA_STANDING_' + {'north':'UP','south':'DOWN','east':'RIGHT','west':'LEFT'}[actor.get('facing', 'south')]
                actual['facing'] = obj['spritemovedata']
            if role == 'items':
                expected['flag'] = actor['flag']; actual['flag'] = obj['event_flag']
            if role == 'trainers':
                trainer = pack['data']['trainers']['trainers'].get(identifier, {})
                expected.update(name=actor['display_name'].upper(), party=[[v['species'],v['level']] for v in actor['party']], flag=actor['defeated_flag'])
                actual.update(name=trainer.get('name'), party=[[v['species'],v['level']] for v in trainer.get('party', [])], flag=module['trainer_scripts'].get(identifier, {}).get('event_flag'))
            fields = [key for key in expected if expected[key] != actual[key]]
            if fields:
                conflicts.append(dict(id=identifier, role=role, issue='source/native actor fields differ', fields=fields, source={k:expected[k] for k in fields}, native={k:actual[k] for k in fields}))
    return conflicts


def payload_state(pack, name):
    data = pack['data']; module = data['maps'][name]; attrs = module['attributes']; label = attrs.get('blocks_label')
    encoded = data['map_blocks'].get(label)
    if isinstance(encoded, str):
        blocks = list(base64.b64decode(encoded))
    else:
        blocks = None
    script_drift = [label for label, body in module['scripts'].items() if data['map_scripts'].get(label) != body]
    return dict(block_payload_label=label, cached_blocks_sha256=fingerprint(module['blocks']), encoded_payload_sha256=fingerprint(encoded), block_payload_matches_cache=blocks == module['blocks'], attributes_match_split=data['map_attributes'].get(name) == attrs, npcs_match_split=data['npcs'].get(name) == module['objects'], script_cache_split_mismatch_labels=script_drift)


def cells(left, right, width):
    if len(left) != len(right):
        return None
    return [dict(metatile=[i%width,i//width], left=f'{a:02x}', right=f'{b:02x}') for i,(a,b) in enumerate(zip(left,right)) if a != b]


def module_delta(old, new):
    fields = [key for key in sorted(set(old)|set(new)) if old.get(key) != new.get(key)]
    return dict(changed_fields=fields, field_hashes={key:dict(quality=fingerprint(old.get(key)), story=fingerprint(new.get(key))) for key in fields}, attributes_changed=[key for key in sorted(set(old['attributes'])|set(new['attributes'])) if old['attributes'].get(key) != new['attributes'].get(key)], actor_records_identical=old['objects'] == new['objects'], events_identical=old['events'] == new['events'], scripts_identical=old['scripts'] == new['scripts'])


def kind(pack, name, tile):
    module = pack['data']['maps'][name]; a = module['attributes']; x,y = tile
    if not (0 <= x < a['width']*2 and 0 <= y < a['height']*2):
        return None
    block = module['blocks'][y//2*a['width']+x//2]
    return pack['data']['tilesets'][a['tileset_name']]['collision'][f'{block:02x}'][y%2*2+x%2]


def connections(pack, towns):
    result = []
    maps = pack['data']['maps']; reverse = dict(north='south', south='north', east='west', west='east')
    for town in towns:
        module = maps[town]; a = module['attributes']
        for c in a['connections']:
            target = c['target_map']
            if not re.fullmatch(r'MnR\d\dC\d\dR\d\d', target):
                continue
            b = maps[target]['attributes']; lanes = []
            if c['direction'] in ['north','south']:
                for x in range(a['width']*2):
                    city = [x, 0 if c['direction']=='north' else a['height']*2-1]
                    route = [x-c['offset']*2, b['height']*2-1 if c['direction']=='north' else 0]
                    if kind(pack,town,city)=='FLOOR' and kind(pack,target,route)=='FLOOR':
                        lanes.append(dict(town=city, route=route))
            else:
                for y in range(a['height']*2):
                    city = [0 if c['direction']=='west' else a['width']*2-1, y]
                    route = [b['width']*2-1 if c['direction']=='west' else 0, y-c['offset']*2]
                    if kind(pack,town,city)=='FLOOR' and kind(pack,target,route)=='FLOOR':
                        lanes.append(dict(town=city, route=route))
            reciprocal = dict(direction=reverse[c['direction']], target_map=town, offset=-c['offset']) in b['connections']
            result.append(dict(town=town, route=target, outward=c['direction'], offset=c['offset'], reciprocal=reciprocal, legal_floor_lane_pairs=lanes))
    return result


def gym_blocks(gym):
    needed = set(''.join(gym['rows'])) | {'#'}
    needed.update(v['open_symbol'] for v in gym.get('gates', []))
    needed.update(v['replacement_symbol'] for v in gym.get('cut_gates', []))
    symbols = {symbol:index for index,symbol in enumerate(sorted(k for k in gym['art'] if k in needed))}
    return [symbols[symbol] for row in gym['rows'] for symbol in row]


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--output', type=Path, required=True); args = parser.parse_args(); assert not args.output.exists()
    paths = [ROOT/'content-packs/minnesota-northstar-rock-stairs-review.crystalpack', ROOT/'content-packs/minnesota-northstar-lake-country.crystalpack']
    assert [sha(p.read_bytes()) for p in paths] == [QUALITY_SHA, STORY_SHA]
    quality, story = [load_pack(p) for p in paths]; sources = {}
    def read(relative):
        path = PLAN/relative; raw = path.read_bytes(); sources[relative] = sha(raw); return json.loads(raw)
    report = dict(schema_version=1, status='READ_ONLY_INVENTORY', scope='Three-way cached native/source geometry inventory; no Rust build, publication, input/controller, save-migration or story repair claim.', packs=[dict(path=str(p.relative_to(ROOT)), file_sha256=h, maps=len(pack['data']['maps']), content_identity=pack['identity']['content_hash']) for p,h,pack in zip(paths,[QUALITY_SHA,STORY_SHA],[quality,story])], routes=[], caves=[], towns=[], native_changed_modules={}, source_files_sha256=sources)
    a,b = quality['data'],story['data']
    report['new_story_map_ids'] = sorted(set(b['maps'])-set(a['maps'])); report['missing_story_map_ids'] = sorted(set(a['maps'])-set(b['maps']))
    for name in sorted(set(a['maps'])&set(b['maps'])):
        if a['maps'][name] != b['maps'][name]:
            report['native_changed_modules'][name] = module_delta(a['maps'][name], b['maps'][name])
    river = read('playable/river-city-links.json'); lake = read('playable/lake-country-story.json')
    overlays = {}
    for cap in river['route_edge_closures']:
        x,y = cap['metatile']; overlays[('MnR03C00R01',x,y-24)] = dict(source='playable/river-city-links.json:route_edge_closures', previous=f'{cap["from"]:02x}', replacement=f'{cap["to"]:02x}')
    for cap in lake['caps']:
        x,y = cap['metatile']; overlays[(cap['map'],x,y)] = dict(source='playable/lake-country-story.json:caps', previous=f'{cap["previous"]:02x}', replacement=f'{cap["replacement"]:02x}')
    report['declared_story_caps'] = [dict(map=n, local_metatile=[x,y], **v) for (n,x,y),v in overlays.items()]
    for rid in range(1,21):
        prefix=f'MnR{rid:02}'; dossier=read(f'routes/MN_R{rid:02}.json'); layout=read(f'routes/MN_R{rid:02}.layout.json'); w,h=layout['dimensions_metatiles']; source=layout_blocks(layout)
        row = dict(id=dossier['id'], name=dossier['name'], source_extent_metatiles=[w,h], source_compiled_tileset=layout['compiled_tileset'], source_required_flags=dossier.get('required_flags'), source_actor_count=sum(len(dossier.get(k,[])) for k in ['trainers','residents','items']), chunks=[], actor_conflicts={}, interior_ids=[v['id'] for v in dossier['interiors']])
        modules = []
        for pack in [quality, story]:
            modules.append({name:m for name,m in pack['data']['maps'].items() if re.fullmatch(prefix+r'C\d\dR\d\d', name)})
        for name in sorted(set(modules[0])|set(modules[1])):
            match=re.fullmatch(prefix+r'C(\d\d)R(\d\d)',name); col,chunkrow=map(int,match.groups()); x0,y0=col*32,chunkrow*24; old,new=modules[0][name],modules[1][name]; ow,oh=old['attributes']['width'],old['attributes']['height']; nw,nh=new['attributes']['width'],new['attributes']['height']
            expected=[source[y*w+x] for y in range(y0,min(y0+24,h)) for x in range(x0,min(x0+32,w))]
            chunk=dict(id=name, quality_extent=[ow,oh], story_extent=[nw,nh], quality_tileset=old['attributes']['tileset_name'], story_tileset=new['attributes']['tileset_name'], active_blocks_sha256=fingerprint(expected), quality_vs_story_cells=cells(old['blocks'],new['blocks'],ow), quality_vs_active_cells=cells(old['blocks'],expected,ow), story_vs_active_cells=cells(new['blocks'],expected,nw), quality_payload=payload_state(quality,name), story_payload=payload_state(story,name), quality_connections=old['attributes']['connections'], story_connections=new['attributes']['connections'], native_actor_count=[len(old['objects']),len(new['objects'])], quality_actors=actor_roster(old,(col*64,chunkrow*48)), story_actors=actor_roster(new,(col*64,chunkrow*48)))
            for cell in chunk['quality_vs_story_cells'] or []:
                x,y=cell['metatile']; overlay=overlays.get((name,x,y)); cell['classification']='declared_story_boundary_cap' if overlay and cell['right']==overlay['replacement'] else 'completed_geometry_missing_from_story'; cell['global_metatile']=[x+x0,y+y0]
            for cell in chunk['story_vs_active_cells'] or []:
                x,y=cell['metatile']; overlay=overlays.get((name,x,y)); cell['classification']='preserve_story_cap_over_active_source' if overlay and cell['left']==overlay['replacement'] else 'active_source_geometry_or_existing_seam_difference'
            row['chunks'].append(chunk)
        for label,pack,mods in [('quality',quality,modules[0]),('story',story,modules[1])]:
            row['actor_conflicts'][label]=source_actor_conflicts(dossier,mods,pack,True)
        report['routes'].append(row)
        for room in dossier['interiors']:
            if room['id'] not in CAVE_IDS: continue
            name=room['id']; old,new=a['maps'][name],b['maps'][name]; expected=[int(v,16) for line in room['rows'] for v in line.split()]
            report['caves'].append(dict(id=name, source_route=dossier['id'], source_extent=room['dimensions_metatiles'], quality_extent=[old['attributes']['width'],old['attributes']['height']], story_extent=[new['attributes']['width'],new['attributes']['height']], quality_tileset=old['attributes']['tileset_name'], story_tileset=new['attributes']['tileset_name'], source_rows_match_quality=old['blocks']==expected, source_rows_match_story=new['blocks']==expected, quality_vs_story_cells=cells(old['blocks'],new['blocks'],old['attributes']['width']), source_vs_quality_cells=cells(expected,old['blocks'],room['dimensions_metatiles'][0]), source_vs_story_cells=cells(expected,new['blocks'],room['dimensions_metatiles'][0]), original_actor_records_same=old['objects']==new['objects'], quality_actors=actor_roster(old), story_actors=actor_roster(new), quality_vs_story=module_delta(old,new), quality_payload=payload_state(quality,name), story_payload=payload_state(story,name), actor_conflicts_quality=source_actor_conflicts(room,{name:old},quality,False), actor_conflicts_story=source_actor_conflicts(room,{name:new},story,False)))
    assert len(report['routes'])==20 and len(report['caves'])==9
    for town in NEW_TOWNS:
        dossier=read(f'towns/{town}.json'); layout=read(f'towns/{town}.layout.json'); module=b['maps'][town]; extent=dossier['dimensions_metatiles']; expected=layout_blocks(layout); roomids=[v['id'] for v in dossier['interiors']]
        extras=[]
        if town=='MnStPaul':
            depot=read('transit/MnStPaulUnionDepot.json'); roomids.append(depot['id']); extras=['transit/MnStPaulUnionDepot.json']
        report['towns'].append(dict(id=town, source_dossier=f'towns/{town}.json', source_layout=f'towns/{town}.layout.json', additional_room_sources=extras, source_extent=extent, native_extent=[module['attributes']['width'],module['attributes']['height']], native_tileset=module['attributes']['tileset_name'], source_ports=dossier['ports'], source_native_cells=cells(expected,module['blocks'],extent[0]), native_actors=actor_roster(module), source_actor_conflicts=source_actor_conflicts(dossier,{town:module},story,False), source_room_ids=roomids, source_room_count=len(roomids), newly_added_room_ids=[n for n in roomids if n in report['new_story_map_ids']], native_outdoor_payload=payload_state(story,town)))
    gyms=read('gyms/interiors.json'); report['canonical_gyms']=[dict(id=g['id'],active_extent=g['dimensions_metatiles'],quality_extent=[a['maps'][g['id']]['attributes']['width'],a['maps'][g['id']]['attributes']['height']],story_extent=[b['maps'][g['id']]['attributes']['width'],b['maps'][g['id']]['attributes']['height']],active_rows_match_quality=a['maps'][g['id']]['blocks']==gym_blocks(g),active_rows_match_story=b['maps'][g['id']]['blocks']==gym_blocks(g)) for g in gyms['gyms']]
    for contract in CONTRACTS: read(contract)
    for extra in ['world.json','interiors.json','story.json','transit/glacier-line.json']: read(extra)
    report['actual_story_town_route_pairs']=connections(story,OLD_TOWNS+NEW_TOWNS)
    report['actual_quality_town_route_pairs']=connections(quality,OLD_TOWNS)
    report['source_connection_contracts']=[dict(path=p,sha256=sources[p]) for p in CONTRACTS]
    report['art']=dict(missing_from_story=sorted(set(a['tilesets'])-set(b['tilesets'])),story_only=sorted(set(b['tilesets'])-set(a['tilesets'])),changed_common_tilesets=[dict(id=k,quality_sha256=fingerprint(a['tilesets'][k]),story_sha256=fingerprint(b['tilesets'][k])) for k in sorted(set(a['tilesets'])&set(b['tilesets'])) if a['tilesets'][k]!=b['tilesets'][k]],missing_runtime_files=[dict(path=k,quality_sha256=sha(bytes(quality['runtime_files'][k]))) for k in sorted(set(quality['runtime_files'])-set(story['runtime_files']))],changed_common_runtime_files=[dict(path=k,quality_sha256=sha(bytes(quality['runtime_files'][k])),story_sha256=sha(bytes(story['runtime_files'][k]))) for k in sorted(set(quality['runtime_files'])&set(story['runtime_files'])) if quality['runtime_files'][k]!=story['runtime_files'][k]])
    report['unchanged_pack_fields']=[k for k in quality if quality[k]==story.get(k)]
    report['other_changed_catalogs']=[dict(id=k,quality_sha256=fingerprint(a[k]),story_sha256=fingerprint(b[k])) for k in a if k not in ['maps','map_attributes','map_blocks','npcs','map_scripts','tilesets'] and a[k]!=b[k]]
    report['repel']=dict(quality_steps=a['items']['REPEL']['repel_steps'],story_steps=b['items']['REPEL']['repel_steps'],only_changed_item_ids=[k for k in a['items'] if a['items'][k]!=b['items'].get(k)])
    report['summary']=dict(quality_maps=len(a['maps']),story_maps=len(b['maps']),added_maps=len(report['new_story_map_ids']),routes=20,caves=9,route_exterior_chunks=sum(len(r['chunks']) for r in report['routes']),story_towns=len(OLD_TOWNS+NEW_TOWNS),new_towns=len(NEW_TOWNS),new_town_rooms=sum(t['source_room_count'] for t in report['towns']),newly_added_town_rooms=sum(len(t['newly_added_room_ids']) for t in report['towns']),quality_native_town_route_pairs=len(report['actual_quality_town_route_pairs']),story_native_town_route_pairs=len(report['actual_story_town_route_pairs']),missing_cave_refinements=[r['id'] for r in report['caves'] if r['source_rows_match_quality'] and not r['source_rows_match_story']],source_actor_conflicts=[dict(route=r['id'],conflicts=r['actor_conflicts']) for r in report['routes'] if any(r['actor_conflicts'].values())])
    report['input_hashes_still_exact']=[sha(p.read_bytes()) for p in paths]==[QUALITY_SHA,STORY_SHA]
    report['source_files_unchanged_during_audit']=all(sha((PLAN/name).read_bytes())==digest for name,digest in sources.items())
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report['summary'],indent=2));print('REPORT_SHA',sha(args.output.read_bytes()))


if __name__=='__main__':main()
