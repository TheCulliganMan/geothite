#!/usr/bin/env python3
"""Independent read-only source/preimage and written Mississippi preservation audit.

Never constructs or writes a pack. New-source static evidence is separate from
production controller, animation, and full native parser regression evidence.
"""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from audit_city_expansions import command, kind, module_hash, section_rows, text_body
from audit_winding_shelter_541 import raw_identity
from render_reference import layout_blocks, load_pack

ROOT = Path(__file__).resolve().parents[3]
BASE_FILE = '2fe6fc2a2c59833886b9ffa2fa04584f1f95841efaf87e865f11230cf4cdf54e'
MANIFEST = '449ce63508b11513b187e5741ac8b8dc669d9fe420b1c922907dd301d4e2b814'


def sha(raw): return hashlib.sha256(raw).hexdigest()
def encode(blocks): return base64.b64encode(bytes(blocks)).decode()
def source(path): return ROOT / path

def read_inputs(manifest):
    raw = manifest.read_bytes()
    assert sha(raw) == MANIFEST, 'frozen source manifest differs'
    plan = json.loads(raw)
    layouts = {}
    inputs = {str(manifest): sha(raw)}
    for row in plan['new_maps']:
        path = source(row['layout']); data = path.read_bytes()
        assert sha(data) == row['layout_sha256']
        inputs[str(path)] = sha(data)
        layouts[row['layout']] = json.loads(data)
    path = source(plan['bridge_inventory']); data = path.read_bytes()
    inputs[str(path)] = sha(data)
    return plan, layouts, json.loads(data), inputs


def clipped(row, layouts):
    layout = layouts[row['layout']]
    assert layout['schema_version'] == 2 and layout['compiled_tileset'] == row['tileset']
    assert layout['source_pixels_modified'] is False and not layout.get('derived_blocks')
    assert layout['shore_fitting'] == 'explicit_source_edges' and layout['reject_cliff_cuts'] is True
    assert all(layer['kind'] == 'block_rect' for layer in layout['layers'])
    width, height = layout['dimensions_metatiles']
    x0, y0 = row['source_origin_metatiles']; w, h = row['dimensions_metatiles']
    assert 0 < w <= 32 and 0 < h <= 24 and x0+w <= width and y0+h <= height
    # These explicit source rectangles share the exact native inclusive/exclusive rule.
    blocks = [5] * (width * height)
    for layer in layout['layers']:
        a,b,c,d = layer['bounds']; block = int(layer['block'],16)
        assert 0 <= a < c <= width and 0 <= b < d <= height and 0 <= block <= 255
        for y in range(b,d): blocks[y*width+a:y*width+c] = [block]*(c-a)
    assert blocks == layout_blocks(layout)
    chunk = [blocks[y*width+x] for y in range(y0,y0+h) for x in range(x0,x0+w)]
    assert sha(bytes(chunk)) == row['source_blocks_sha256']
    return chunk


def anchors(module):
    return dict(objects=[[o['x'],o['y']] for o in module['objects']],
        backgrounds=[[o['x'],o['y']] for o in module['events']['bg_events']],
        coordinates=[[o['x'],o['y']] for o in module['events']['coord_events']],
        warps=[[o['x'],o['y']] for o in module['events']['warps']])


def prepare(before, plan, layouts, inventory):
    assert before['identity']['content_hash'] == plan['base_content_hash']
    assert plan['base_pack_sha256'] == inventory['base_pack_sha256'] == BASE_FILE
    assert plan['schema_version'] == inventory['schema_version'] == 1
    assert plan['registry'] == dict(group_id=82,group_name='MINNESOTA_MISSISSIPPI',
        map_group_constant='MAPGROUP_MINNESOTA_MISSISSIPPI', reservation_evidence=plan['registry']['reservation_evidence'])
    assert len(plan['new_maps']) == 17 and not plan['backgrounds']
    a = before['data']; expected = deepcopy(a); files = {k:bytes(v) for k,v in before['runtime_files'].items()}
    assert len(a['maps']) == 660
    assert all(v['groupId'] != 82 for v in a['runtime_map_metadata'].values())
    assert 'MAPGROUP_MINNESOTA_MISSISSIPPI' not in a['story_event_script_constants']['global']
    cells = set(); changed = set(); recipes = {}; rows = {}
    for revision in plan['terrain_revisions']:
        old = a['maps'][revision['map']]; module = expected['maps'][revision['map']]
        assert module_hash(old) == revision['native_module_sha256']
        assert [old['attributes']['width'],old['attributes']['height']] == revision['dimensions_metatiles']
        assert anchors(old) == revision['anchor_preimages']
        for change in revision['changes']:
            x,y = change['metatile']; key = revision['map'],x,y
            assert key not in cells; cells.add(key)
            assert 0 <= x < old['attributes']['width'] and 0 <= y < old['attributes']['height']
            index = y*old['attributes']['width']+x
            assert old['blocks'][index] == change['before'] != change['after']
            module['blocks'][index] = change['after']; changed.add(revision['map'])
    for link in plan['existing_connection_additions']:
        module = expected['maps'][link['map']]
        assert link['direction'] not in [v['direction'] for v in module['attributes']['connections']]
        module['attributes']['connections'].append({k:link[k] for k in ['direction','target_map','offset']})
        module['attributes']['connection_flags'] = ' | '.join(v['direction'].upper() for v in module['attributes']['connections'])
        changed.add(link['map'])
    for bridge in inventory['bridges']:
        old = a['maps'][bridge['map']]; module = expected['maps'][bridge['map']]; alias = bridge['tileset']
        assert module_hash(old) == bridge['native_module_sha256']
        assert old['attributes']['tileset_name'] == alias and bridge['existing_foot_collision_must_remain_exact'] is True
        x0,y0,x1,y1 = bridge['metatile_bounds_exclusive']
        assert len(bridge['metatiles']) == (x1-x0)*(y1-y0)
        assert bridge['axis'] in ['north_south','east_west']
        for cell in bridge['metatiles']:
            x,y = cell['metatile']; index = y*old['attributes']['width']+x
            assert x0 <= x < x1 and y0 <= y < y1
            assert (bridge['map'],x,y) not in cells; cells.add((bridge['map'],x,y))
            block = cell['block_decimal']; assert int(cell['block_hex'],16) == block == old['blocks'][index]
            collision = a['tilesets'][alias]['collision'][cell['block_hex']]
            assert collision == cell['foot_collision'] and cell['proposed_surf_underpass_collision'] == ['WATER']*4
            assert set(collision) <= {'FLOOR','WALL'}
            recipe = alias,bridge['axis'],block
            meta = f'data/tilesets/{alias}_metatiles.bin'
            if recipe not in recipes:
                new = len(files[meta])//16; assert new <= 255
                assert len(files[meta]) % 16 == 0
                files[meta] += bytes(before['runtime_files'][meta])[block*16:block*16+16]
                axis = 'NS' if bridge['axis'] == 'north_south' else 'EW'
                expected['tilesets'][alias]['collision'][f'{new:02x}'] = [f'BRIDGE_{axis}_'+('DECK' if c=='FLOOR' else 'RAIL') for c in collision]
                recipes[recipe] = new
            assert module['blocks'][index] == block
            module['blocks'][index] = recipes[recipe]; changed.add(bridge['map'])
    for name in changed:
        module = expected['maps'][name]; attrs = module['attributes']
        expected['map_blocks'][attrs['blocks_label']] = encode(module['blocks'])
        expected['map_attributes'][name] = deepcopy(attrs)
    for row in plan['new_maps']:
        assert row['id'] not in a['maps'] and row['map_constant'] not in a['runtime_map_metadata']
        assert row['map_id'] not in [v['map_id'] for v in rows.values()]
        rows[row['id']] = row
        clipped(row,layouts)
    return expected, files, rows, changed, recipes


def physical_seams(expected, plan, layouts):
    maps = deepcopy(expected['maps'])
    for row in plan['new_maps']:
        module = deepcopy(maps['Route30']); attrs = module['attributes']
        attrs.update(width=row['dimensions_metatiles'][0],height=row['dimensions_metatiles'][1],tileset_name=row['tileset'],connections=row['connections'])
        module['blocks'] = clipped(row,layouts); module['id'] = row['id']; maps[row['id']] = module
    pack = dict(data=dict(maps=maps,tilesets=expected['tilesets']))
    results = []
    links = [dict(map=row['id'],**link) for row in plan['new_maps'] for link in row['connections']] + plan['existing_connection_additions']
    for link in links:
        name = link['map']; module = maps[name]; target = maps[link['target_map']]
        side = link['direction']; offset = link['offset']; w,h = [module['attributes'][v]*2 for v in ['width','height']]; tw,th = [target['attributes'][v]*2 for v in ['width','height']]
        reverse = dict(north='south',south='north',east='west',west='east')[side]
        assert any(v == dict(direction=reverse,target_map=name,offset=-offset) for v in target['attributes']['connections'])
        points = [(x,0 if side=='north' else h-1) for x in range(w)] if side in ['north','south'] else [(0 if side=='west' else w-1,y) for y in range(h)]
        counts = dict(water_pairs=0,walking_pairs=0,closed_banks=0,wall_pairs=0)
        for x,y in points:
            tx,ty = (x-offset*2, th-1 if side=='north' else 0) if side in ['north','south'] else (tw-1 if side=='west' else 0,y-offset*2)
            first = kind(pack,module,[x,y])
            if not (0 <= tx < tw and 0 <= ty < th):
                assert first in ['WALL','BUOY'], (name,side,[x,y],first,'outside neighbor')
                continue
            second = kind(pack,target,[tx,ty])
            if first=='WATER' and second=='WATER': counts['water_pairs'] += 1
            elif first in ['FLOOR','TALL_GRASS'] and second in ['FLOOR','TALL_GRASS']: counts['walking_pairs'] += 1
            elif first in ['WALL','BUOY'] or second in ['WALL','BUOY']: counts['wall_pairs' if first==second else 'closed_banks'] += 1
            else: assert False,(name,side,[x,y],first,second)
        results.append(dict(map=name,direction=side,target=link['target_map'],**counts))
    return results


def activity_scripts(plan,name):
    scripts = {}
    for actor in plan['objects']:
        if actor['map'] != name: continue
        assert actor['role'] == 'optional_trainer'
        root = actor['id']; seen,won,lost,after = [root+s for s in ['Seen','Won','Lost','After']]
        scripts[root] = [command('trainer',actor['trainer_class'],root,actor['event_flag'],seen,won,lost,after)]
        scripts[after] = [command('endifjustbattled'),command('jumptextfaceplayer',won)]
        for label,lines in [(seen,actor['text']),(won,['Safe travels!']),(lost,['A strong current!'])]: scripts[label] = text_body(lines)
    return scripts


def audit(before, after, plan, layouts, inventory):
    expected, files, rows, changed, recipes = prepare(before,plan,layouts,inventory)
    a,b = before['data'],after['data']; normalized = deepcopy(b); checks = []
    def equal(label,left,right):
        assert left == right, label
        checks.append(label)
    equal('exact original660 plus17 new maps',set(a['maps'])|set(rows),set(b['maps']))
    for name,old in a['maps'].items():
        equal(name+' exact original module except declared terrain/connections',expected['maps'][name],b['maps'][name])
    for name,row in rows.items():
        module = b['maps'][name]; attrs = module['attributes']; scripts = activity_scripts(plan,name)
        equal(name+' exact clipped source terrain',clipped(row,layouts),module['blocks'])
        equal(name+' native extent',row['dimensions_metatiles'],[attrs['width'],attrs['height']])
        equal(name+' native alias',row['tileset'],attrs['tileset_name'])
        parent = ('MnWinona' if name.startswith('MnMississippiLowerWoods') or name=='MnMississippiWinonaMouth' else 'MnMinneapolis' if name.startswith('MnMississippiConfluence') else 'MnStCloud')
        location = a['pokegear_landmarks']['map_to_landmark'][parent]
        landmark = next(v for v in a['pokegear_landmarks']['landmarks'] if v['constant']==location)
        assert 1 <= landmark['id'] <= 127
        equal(name+' exact existing Minnesota parent location',location,attrs['location'])
        equal(name+' exact native town-map binding',location,b['pokegear_landmarks']['map_to_landmark'][name])
        normalized['pokegear_landmarks']['map_to_landmark'].pop(name)

        equal(name+' native connections',row['connections'],attrs['connections'])
        equal(name+' native identity',(row['map_constant'],'MAPGROUP_MINNESOTA_MISSISSIPPI'),(attrs['map_constant'],attrs['map_group_constant']))
        equal(name+' exact new raw activity programs',scripts,module['scripts'])
        equal(name+' no new warps/coord events/BG/events',dict(warps=[],coord_events=[],bg_events=[]),module['events'])
        actors = [v for v in plan['objects'] if v['map']==name]
        objects = []
        for actor in actors:
            obj = deepcopy(a['maps']['NewBarkTown']['objects'][0])
            obj.update(sprite=actor['sprite'],x=actor['core'][0],y=actor['core'][1],spritemovedata='SPRITEMOVEDATA_STANDING_'+dict(north='UP',south='DOWN',east='RIGHT',west='LEFT')[actor['facing']],move_range_x=0,move_range_y=0,pal=0,script=actor['id'],object_identifier=actor['id'],event_flag='-1',object_type='OBJECTTYPE_TRAINER',radius=0)
            objects.append(obj)
        equal(name+' exact authored actor records',objects,module['objects'])
        for label,script in scripts.items(): equal(label+' exact split source',script,b['map_scripts'][label])
        equal(name+' split event wire',section_rows(module),b['map_scripts'][attrs['map_events_label']])
        # Split NPC structure is checked against the native cached object records.
        equal(name+' typed event wire',section_rows(module),[{k:v for k,v in c.items() if k!='command_index'} for c in module['map_event_section_commands']])
        equal(name+' exact metadata',dict(name=name,constant=row['map_constant'],groupName='MINNESOTA_MISSISSIPPI',groupId=82,mapId=row['map_id'],width=attrs['width'],height=attrs['height'],environment='ROUTE',phoneService=attrs['phone_service']),b['runtime_map_metadata'][row['map_constant']])
        equal(name+' exact encoded source blocks',encode(module['blocks']),b['map_blocks'][attrs['blocks_label']])
        equal(name+' mirrored attributes',attrs,b['map_attributes'][name])
        normalized['maps'].pop(name); normalized['map_attributes'].pop(name);normalized['map_dimensions'].pop(row['map_constant']);normalized['runtime_map_metadata'].pop(row['map_constant']);normalized['map_blocks'].pop(attrs['blocks_label']);normalized['npcs'].pop(name);normalized['wild_encounters'].pop(name,None)
        for label in set(scripts)|{attrs['map_scripts_label'],attrs['map_events_label']}: normalized['map_scripts'].pop(label)
    global_ = normalized['story_event_script_constants']['global']
    equal('new group82 identity',82,global_.pop('MAPGROUP_MINNESOTA_MISSISSIPPI'))
    next_flag = max(v for k,v in a['story_event_script_constants']['global'].items() if k.startswith('EVENT_'))
    for actor in plan['objects']:
        next_flag += 1; equal(actor['id']+' sequential fresh event',next_flag,global_.pop(actor['event_flag']))
        actual = normalized['trainers']['trainers'].pop(actor['id'])
        template = next(v for k,v in sorted(a['trainers']['trainers'].items()) if v['trainer_class']==actor['trainer_class'])
        old_numbers = [v['trainer_id'] for v in a['trainers']['trainers'].values() if v['trainer_class']==actor['trainer_class']]
        before_global = a['story_event_script_constants']['global']
        expected_number = max([before_global[id_] for id_ in old_numbers if id_ in before_global]+[b['story_event_script_constants']['global'][v['id']] for v in plan['objects'][:plan['objects'].index(actor)] if v['trainer_class']==actor['trainer_class']])+1
        equal(actor['id']+' fresh native class constant',expected_number,global_.pop(actor['id']))
        equal(actor['id']+' exact party species/levels',[(v['species'],v['level']) for v in actor['party']],[(v['species'],v['level']) for v in actual['party']])
        assert all(v['item'] is None and v['dvs']==template['party'][0]['dvs'] for v in actual['party'])
        expected_trainer = deepcopy(template);expected_trainer.update(trainer_id=actor['id'],name=actor['trainer_name'].upper(),party=actual['party'],win_quote='',lose_quote='')
        equal(actor['id']+' all other native trainer template fields',expected_trainer,actual)
    for alias in {v['tileset'] for v in inventory['bridges']}:
        equal(alias+' exact appended collision copies',expected['tilesets'][alias],b['tilesets'][alias])
        collision_file = f'data/tilesets/{alias}.json'
        equal(alias+' collision JSON semantic equality',expected['tilesets'][alias]['collision'],json.loads(bytes(after['runtime_files'][collision_file])))
        files[collision_file] = bytes(after['runtime_files'][collision_file])
    equal('all remaining original catalogues + exact declared old modifications',expected,normalized)
    equal('all original audio PCM/storage manifest',before['audio_manifest'],after['audio_manifest'])
    equal('all audio program payloads',before['compiled_audio'],after['compiled_audio'])
    equal('unchanged audio compression',before.get('audio_compression'),after.get('audio_compression'))
    equal('all source art/presentation files except exact bridge append+collision JSON',files,{k:bytes(v) for k,v in after['runtime_files'].items()})
    report = deepcopy(before['report']); report['maps'] = 677
    report['manifests'] += [f"{plan['id']}-{row['map_id']}" for row in plan['new_maps']]
    for key in ['reachable_maps','solvable_maps']: report[key] += list(rows)
    equal('exact bounded report additions',report,after['report'])
    return dict(checks_passed=len(checks),checks=checks,old_maps=660,new_maps=17,changed_old_maps=sorted(changed),bridge_copy_count=len(recipes),native_gameplay_proven=False,limitations=['Read-only audit does not execute native scripts or inputs.','New trainer moves and compiled index correctness are separately covered by Rust compiler/parser regressions, not reconstructed by this preservation audit.'])


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--before',type=Path,required=True);parser.add_argument('--manifest',type=Path,default=source('planning/minnesota/connections/mississippi-v02/manifest.json'));parser.add_argument('--pack',type=Path);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
    plan,layouts,inventory,inputs=read_inputs(args.manifest)
    assert sha(args.before.read_bytes())==BASE_FILE
    before=load_pack(args.before);expected,files,rows,changed,recipes=prepare(before,plan,layouts,inventory)
    seams=physical_seams(expected,plan,layouts)
    report=dict(status='SOURCE_PREFLIGHT_PASS',physical_seams=seams,manifest_sha256=MANIFEST,new_map_count=len(rows),old_changed_maps=sorted(changed),bridge_copies=len(recipes),clipped_sources=[dict(id=v['id'],source_origin=v['source_origin_metatiles'],dimensions=v['dimensions_metatiles'],blocks_sha256=v['source_blocks_sha256']) for v in plan['new_maps']],inputs=inputs,native_gameplay_proven=False)
    if args.pack:
        after=load_pack(args.pack);report.update(audit(before,after,plan,layouts,inventory));report['status']='WRITTEN_PACK_PASS';report['file_sha256']=sha(args.pack.read_bytes());report['identity']=after['identity'];report['raw_identities']=[raw_identity(args.before),raw_identity(args.pack)];assert all(v['exact'] for v in report['raw_identities'])
        negatives=[]
        for label in ['old_room','old_story','old_party','old_audio','outside_terrain','bridge_pixels','old_catalog','new_actor','new_bridge_collision','missing_landmark','wrong_location']:
            bad=deepcopy(after)
            if label=='old_room':bad['data']['maps']['MnStPaulUnionDepot']['blocks'][0]^=1
            elif label=='old_story':bad['data']['maps']['MnMinneapolis']['script_control_commands'][0]['command']='audit_corruption'
            elif label=='old_party':next(iter(bad['data']['trainers']['trainers'].values()))['party'][0]['level']^=1
            elif label=='old_audio':bad['audio_manifest']={}
            elif label=='outside_terrain':bad['data']['maps']['MnR05C00R00']['blocks'][0]^=1
            elif label=='bridge_pixels':
                file='data/tilesets/johto_metatiles.bin';raw=bytearray(bad['runtime_files'][file]);raw[-1]^=1;bad['runtime_files'][file]=bytes(raw)
            elif label=='old_catalog':bad['data']['items']['POTION']['price']+=1
            elif label=='new_actor':bad['data']['maps'][plan['objects'][0]['map']]['objects'][0]['x']+=1
            elif label=='missing_landmark':bad['data']['pokegear_landmarks']['map_to_landmark'].pop(plan['new_maps'][0]['id'])
            elif label=='wrong_location':bad['data']['maps'][plan['new_maps'][0]['id']]['attributes']['location']='LANDMARK_ROUTE_30'
            else:
                alias=inventory['bridges'][0]['tileset'];key=max(bad['data']['tilesets'][alias]['collision'],key=lambda x:int(x,16));bad['data']['tilesets'][alias]['collision'][key][0]='WATER'
            try:audit(before,bad,plan,layouts,inventory)
            except (AssertionError,KeyError,StopIteration):negatives.append(label)
            else:raise AssertionError('accepted corruption '+label)
        report['corruption_negatives_rejected']=negatives
    args.output.mkdir(parents=True,exist_ok=False);raw=(json.dumps(report,indent=2)+'\n').encode();out=args.output/('proof-'+sha(raw)+'.json');out.write_bytes(raw)
    print(json.dumps(dict(status=report['status'],proof=str(out),checks=report.get('checks_passed'),new_maps=len(rows))))

if __name__=='__main__':main()
