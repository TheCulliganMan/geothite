#!/usr/bin/env python3
"""Independent, read-only written681 state-cohesion preservation audit.

Checks exact parent, declared source cells, native indexed old records and raw
new checkpoint programs. Does not build packs or execute gameplay. The native
parser/controller regressions remain separate evidence for new compiled indexes.
"""
import argparse
import base64
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from audit_city_expansions import command, kind, module_hash, section_rows, text_body
from audit_outdoor_connections import neighbor_origin, project, OPPOSITE
from audit_winding_shelter_541 import raw_identity
from render_reference import layout_blocks, load_pack

ROOT = Path(__file__).resolve().parents[3]
BASE = '6b36b4b835873e4c4508a7da7727ecb759273c26e219ec026c061c0e66f24775'
CONTENT = 'de528e31f3d06f37c41b47595806d492d8279768f71d7b508ae7ab5315b66b5e'
CONTRACT = '2d758bfe1894ee42ddcd2e176d25fcd7147fb4f3632f066ac16468318c26756b'
PATCHES = '717f7744da7a5c1cfc13509144f0d2dd4f8d6f2b0fa0a6359efe4579252957c4'
FACING = dict(north='DOWN',south='UP',east='LEFT',west='RIGHT')


def sha(raw): return hashlib.sha256(raw).hexdigest()
def encoded(rows): return base64.b64encode(bytes(rows)).decode()


def inputs(path):
    raw = path.read_bytes(); assert sha(raw)==CONTRACT,'frozen cohesion contract differs'; plan = json.loads(raw)
    assert plan['base_pack_sha256'] == BASE and plan['base_content_hash'] == CONTENT
    assert plan['map_count_before'] == 677 and len(plan['new_maps']) == 4
    hashes = {str(path): sha(raw)}; layouts = {}; dossiers = {}
    for row in plan['new_maps']:
        for key, dest in [('layout',layouts),('dossier',dossiers)]:
            file = ROOT / row[key+'_file']; data = file.read_bytes()
            assert sha(data) == row[key+'_sha256'], str(file)
            hashes[str(file)] = sha(data); dest[row['id']] = json.loads(data)
    path = ROOT / plan['native_port_patches']; raw = path.read_bytes()
    assert sha(raw) == plan['native_port_patches_sha256'] == PATCHES
    hashes[str(path)] = sha(raw)
    return plan, layouts, dossiers, json.loads(raw), hashes


def guard_scripts(g):
    root = g['id']; badge = root+'Badges'; evidence = root+'Evidence'
    def checks(b,e):
        rows = []
        for flag in g['flags']: rows += [command('checkflag',flag),command('iffalse',b)]
        if g.get('event'): rows += [command('checkevent',g['event']),command('iffalse',e)]
        return rows
    def visible(c): return [command(c,id_) for id_ in g['actor_ids']]
    result = {root:checks(badge,evidence)+visible('disappear')+[command('end')],
        root+'Inward':[command('readvar','VAR_FACING'),command('ifequal',dict(north='UP',south='DOWN',east='RIGHT',west='LEFT')[g['side']],root+'End')]+visible('disappear')+[command('applymovement','PLAYER',root+'TwoSteps'),command('sjump',root+'Reconcile')],
        root+'Reconcile':checks(root+'Show',root+'Show')+[command('end')],
        root+'Show':visible('appear')+[command('end')],root+'End':[command('end')],
        root+'OneStep':[command('step',g['facing']),command('step_end')],
        root+'TwoSteps':[command('step',g['facing']),command('step',g['facing']),command('step_end')]}
    for label, exists in [(badge,bool(g['flags'])),(evidence,bool(g.get('event')))]:
        if exists:
            result[label+'Text'] = text_body(g['denial_text'])
            result[label] = [command('opentext'),command('writetext',label+'Text'),command('waitbutton'),command('closetext'),command('applymovement','PLAYER',root+'OneStep'),command('end')]
    return result


def checkpoint_indexes(scripts):
    """Independent exact records for this bounded checkpoint command vocabulary."""
    fields={key:[] for key in ['script_flag_commands','script_object_commands',
        'script_movements','script_text_commands','script_variable_commands','script_control_commands']}
    movements={}
    for label,body in sorted(scripts.items()):
        for i,c in enumerate(body):
            op,args=c['command'],c['args'];identity=dict(source_script=label,command_index=i)
            if op in ['checkflag','checkevent']:
                fields['script_flag_commands'].append(dict(command=op,flag_id=args[0],**identity))
            elif op in ['end','iffalse','ifequal','sjump']:
                target=args[-1] if args else None
                fields['script_control_commands'].append(dict(command=op,compare_value=args[0] if op=='ifequal' else None,target_label=target,resolved_target_script=target,**identity))
            elif op in ['appear','disappear','applymovement']:
                movement=args[1] if op=='applymovement' else None
                fields['script_object_commands'].append(dict(command=op,object_id=args[0],target_object_id=None,x=None,y=None,direction=None,movement=movement,emote=None,duration=None,**identity))
                if movement:movements[movement,label]=scripts[movement]
            elif op=='readvar':
                fields['script_variable_commands'].append(dict(command=op,target=args[0],value_tokens=[],**identity))
            elif op in ['opentext','writetext','waitbutton','closetext']:
                fields['script_text_commands'].append(dict(command=op,text_label=args[0] if op=='writetext' else None,**identity))
    for (label,parent),body in sorted(movements.items()):
        fields['script_movements'].append(dict(label=label,source_script=parent,steps=[dict(command=c['command'],direction=c['args'][0] if c['args'] else None,duration=None,index=i) for i,c in enumerate(body)]))
    return fields


def trainer_scripts(dossier):
    scripts = {}
    for a in dossier['actors']:
        root = a['id']; labels = [root+s for s in ['Seen','Won','Lost','After']]
        scripts[root] = [command('trainer',a['trainer_class'],root,a['event_flag'],*labels)]
        scripts[labels[3]] = [command('endifjustbattled'),command('jumptextfaceplayer',labels[1])]
        for label, lines in zip(labels[:3],[a['text'],['Safe travels!'],['A good forest walk!']]): scripts[label] = text_body(lines)
    for s in dossier['signs']:
        scripts[s['id']] = [command('jumptext',s['id']+'Text')]
        scripts[s['id']+'Text'] = text_body(s['lines'])
    return scripts


def object_record(template, id_, tile, sprite, facing, script, trainer=False):
    obj = deepcopy(template)
    obj.update(x=tile[0],y=tile[1],sprite=sprite,sprite_has_facings=True,
        spritemovedata='SPRITEMOVEDATA_STANDING_'+facing,move_range_x=0,move_range_y=0,
        hram_x=-1,hram_y=-1,pal=0,object_type='OBJECTTYPE_TRAINER' if trainer else 'OBJECTTYPE_SCRIPT',
        radius=0,script=script,label=None,event_flag='-1',object_identifier=id_,sightline_direction_override=None)
    return obj


def topology(after, plan):
    maps = {n:m for n,m in after['data']['maps'].items() if n.startswith('Mn') and str(m['attributes']['environment']).lower() in ['town','route','outdoor']}
    assert len(maps)==98
    positions={'MnMinneapolis':[0,0]}; queue=deque(positions)
    while queue:
        name=queue.popleft(); attrs=maps[name]['attributes']
        for edge in attrs['connections']:
            target=edge['target_map']; assert target in maps
            other=maps[target]['attributes']
            assert dict(direction=OPPOSITE[edge['direction']],target_map=name,offset=-edge['offset']) in other['connections']
            at=list(neighbor_origin(positions[name],attrs,other,edge))
            if target in positions: assert positions[target]==at, ('inconsistent cycle',name,target)
            else: positions[target]=at;queue.append(target)
    assert positions==plan['proposed_origins_metatiles'], '98 exact connected origins'
    names=sorted(maps)
    for i,n in enumerate(names):
        x,y=positions[n];a=maps[n]['attributes']
        for t in names[i+1:]:
            u,v=positions[t];b=maps[t]['attributes']
            assert x+a['width']<=u or u+b['width']<=x or y+a['height']<=v or v+b['height']<=y, ('overlap',n,t)
    for link in plan['new_reciprocal_links']:
        for name,target,side,offset,start,end in [(link['from_map'],link['to_map'],link['direction'],link['offset'],link['from_core'],link['to_core']), (link['to_map'],link['from_map'],OPPOSITE[link['direction']],link['reverse_offset'],link['to_core'],link['from_core'])]:
            module=maps[name];dest=maps[target];edge=dict(direction=side,target_map=target,offset=offset)
            assert edge in module['attributes']['connections']
            for a,b in zip(start,end):
                assert list(project(module['attributes'],dest['attributes'],edge,a))==b
                assert kind(after,module,a)==kind(after,dest,b)=='FLOOR'
    return dict(outdoors=98,components=1,overlaps=0,reciprocal_two_lane_links=len(plan['new_reciprocal_links']))


def prepare(before,plan,patches):
    assert before['identity']['content_hash']==CONTENT and len(before['data']['maps'])==677
    a=before['data']; expected=deepcopy(a['maps']); changed=set(); cells=set()
    for p in plan['native_preimages']+plan['river_module_preservation_preimages']:
        assert module_hash(a['maps'][p['map']])==p['module_sha256'], ('preimage',p['map'])
    assert len(plan['native_preimages'])==len(plan['river_module_preservation_preimages'])==17
    for patch in patches['patches']:
        name=patch['map'];old=a['maps'][name];m=expected[name];changed.add(name)
        assert module_hash(old)==patch['native_module_sha256']
        if patch.get('kind'):
            assert name=='MnR03C00R01' and patch['before_dimensions_metatiles']==[24,8] and patch['after_dimensions_metatiles']==[21,8] and patch['removed_bounds_metatiles']==[21,0,24,8]
            for y in range(16):
                for x in range(42,48):assert kind(before,old,[x,y])=='WALL'
            assert all(o['x']<42 for o in old['objects']+old['events']['warps']+old['events']['coord_events']+old['events']['bg_events'])
            m['blocks']=[old['blocks'][y*24+x] for y in range(8) for x in range(21)]
            m['attributes']['width']=21
        else:
            for c in patch['changes']:
                x,y=c['metatile'];key=name,x,y;assert key not in cells;cells.add(key)
                idx=y*m['attributes']['width']+x
                assert old['blocks'][idx]==c['before']!=c['after'];m['blocks'][idx]=c['after']
    assert len(cells)==42
    for row in plan['existing_connection_updates']:
        m=expected[row['map']];changed.add(row['map'])
        edge=next(e for e in m['attributes']['connections'] if e['direction']==row['side'] and e['target_map']==row['target'])
        assert edge['offset']==row['old_offset'];edge['offset']=row['new_offset']
    for link in plan['new_reciprocal_links']:
        for name,side,target,offset in [(link['from_map'],link['direction'],link['to_map'],link['offset']),(link['to_map'],OPPOSITE[link['direction']],link['from_map'],link['reverse_offset'])]:
            if name not in expected:continue
            m=expected[name];changed.add(name)
            assert not any(e['direction']==side for e in m['attributes']['connections'])
            m['attributes']['connections'].append(dict(direction=side,target_map=target,offset=offset))
    for name in changed:
        attrs=expected[name]['attributes'];attrs['connection_flags']=' | '.join(e['direction'].upper() for e in attrs['connections']) or None
    return expected,changed


def audit(before,after,plan,layouts,dossiers,patches):
    expected,changed=prepare(before,plan,patches);a=before['data'];b=after['data'];norm=deepcopy(b);checks=[]
    def eq(label,left,right):
        assert left==right,label
        checks.append(label)
    new={r['id']:r for r in plan['new_maps']};guards={}
    for g in plan['guard_proposals']:guards.setdefault(g['map'],[]).append(g)
    eq('exact677 plus four additions',set(a['maps'])|set(new),set(b['maps']))
    template=a['maps']['NewBarkTown']['objects'][0]
    # Geometry-only publication must retain BOTH original representations even
    # if an inherited cached/split discrepancy predates this revision.
    guard_event_labels={a['maps'][n]['attributes']['map_events_label'] for n in guards}
    for label,body in a['map_scripts'].items():
        if label not in guard_event_labels:eq('original global raw program '+label,body,b['map_scripts'].get(label))
    for name,objects in a['npcs'].items():
        if name not in guards:eq(name+' original global NPC mirror',objects,b['npcs'].get(name))
    for name,old in a['maps'].items():
        actual=b['maps'][name];want=expected[name]
        if name not in guards:
            eq(name+' exact original module except bounded terrain/seams',want,actual)
        else:
            scripts={};objs=[];coords=[]
            for g in guards[name]:
                assert g['facing']==FACING[g['side']] and g['outbound_only'] and g['inbound_free']
                scripts.update(guard_scripts(g))
                for id_,tile,inside,outside in zip(g['actor_ids'],g['bodies'],g['inside'],g['outside']):
                    objs.append(object_record(template,id_,tile,'SPRITE_OFFICER',g['facing'],g['id']))
                    coords += [dict(x=inside[0],y=inside[1],scene_id='-1',script_name=g['id']),dict(x=outside[0],y=outside[1],scene_id='-1',script_name=g['id']+'Inward')]
            eq(name+' original plus exact new guard objects',old['objects']+objs,actual['objects'])
            ev=deepcopy(old['events']);ev['coord_events']+=coords
            eq(name+' original events plus exact guard triggers',ev,actual['events'])
            eq(name+' exact old raw programs plus authored guards',old['scripts']|scripts,actual['scripts'])
            for field,previous in old.items():
                if field in ['attributes','blocks','objects','events','scripts','map_event_section_commands']:continue
                value=actual[field]
                if isinstance(previous,list):
                    # Only new root-owned rows may be inserted. Removing them
                    # must reproduce every old row and its relative order.
                    new_rows=[v for v in value if v.get('source_script',v.get('label')) in scripts]
                    retained=[v for v in value if v.get('source_script',v.get('label')) not in scripts]
                    eq(name+' exact retained original indexes '+field,previous,retained)
                    added=checkpoint_indexes(scripts).get(field,[])
                    eq(name+' exact inserted checkpoint indexes '+field,added,new_rows)
                    if field in {'script_control_commands','script_flag_commands','script_text_commands'}:
                        expected_order=deepcopy(previous)
                        for row_ in added:
                            key_=(row_['source_script'],row_['command_index'])
                            at_=next((i for i,v in enumerate(expected_order) if (v['source_script'],v['command_index'])>key_),len(expected_order))
                            expected_order.insert(at_,row_)
                        eq(name+' exact bounded new-index insertion order '+field,expected_order,value)
                    else:
                        eq(name+' unchanged old prefix with exact appended indexes '+field,previous+added,value)
                elif isinstance(previous,dict):
                    eq(name+' original keyed indexes '+field,previous,{k:value[k] for k in previous})
                    additions={k:v for k,v in value.items() if k not in previous}
                    expected_bodies={label:dict(label=label,commands=[dict(**c,command_index=i) for i,c in enumerate(body)]) for label,body in scripts.items() if body and body[0]['command']=='text'} if field=='script_text_bodies' else {}
                    eq(name+' exact new keyed indexes '+field,expected_bodies,additions)
                else:eq(name+' original scalar '+field,previous,value)
            eq(name+' exact intended attributes',want['attributes'],actual['attributes'])
            eq(name+' exact intended blocks',want['blocks'],actual['blocks'])
            rows=section_rows(actual)
            eq(name+' regenerated typed event section',rows,[{k:v for k,v in c.items() if k!='command_index'} for c in actual['map_event_section_commands']])
        if name in changed or name in guards:
            attrs=actual['attributes'];constant=attrs['map_constant']
            eq(name+' mirrored attrs',attrs,b['map_attributes'][name])
            eq(name+' dimension mirror',dict(width=attrs['width'],height=attrs['height']),b['map_dimensions'][constant])
            metadata=deepcopy(a['runtime_map_metadata'][constant]);metadata.update(width=attrs['width'],height=attrs['height'])
            eq(name+' metadata mirror',metadata,b['runtime_map_metadata'][constant])
            for point in old['objects']+old['events']['warps']+old['events']['coord_events']+old['events']['bg_events']:
                at_=[point['x'],point['y']]
                eq(name+' retained original anchor collision',kind(before,old,at_),kind(after,actual,at_))
            eq(name+' mirrored blocks',encoded(actual['blocks']),b['map_blocks'][attrs['blocks_label']])
            if name in guards:
                eq(name+' mirrored events',section_rows(actual),b['map_scripts'][attrs['map_events_label']])
                eq(name+' mirrored NPCs',actual['objects'],b['npcs'][name])
                for label,body in actual['scripts'].items():eq(label+' global raw source',body,b['map_scripts'][label])
            else:
                eq(name+' unchanged original event mirror',a['map_scripts'][attrs['map_events_label']],b['map_scripts'][attrs['map_events_label']])
            for field,key in [('maps',name),('map_attributes',name),('map_dimensions',constant),('runtime_map_metadata',constant),('map_blocks',attrs['blocks_label']),('npcs',name)]:
                if key in a[field]:norm[field][key]=deepcopy(a[field][key])
                else:norm[field].pop(key,None)
            for label in set(actual['scripts'])|set(actual['script_text_bodies'])|{attrs['map_scripts_label'],attrs['map_events_label']}:
                if label in a['map_scripts']:norm['map_scripts'][label]=deepcopy(a['map_scripts'][label])
                else:norm['map_scripts'].pop(label,None)
    for row in plan['river_module_preservation_preimages']:eq(row['map']+' complete17river preservation',a['maps'][row['map']],b['maps'][row['map']])
    for name,row in new.items():
        m=b['maps'][name];at=m['attributes'];d=dossiers[name];scripts=trainer_scripts(d)
        eq(name+' exact source terrain',layout_blocks(layouts[name]),m['blocks'])
        eq(name+' source dimensions',row['dimensions_metatiles'],[at['width'],at['height']])
        attrs=deepcopy(a['maps']['Route30']['attributes']);attrs.update(width=row['dimensions_metatiles'][0],height=row['dimensions_metatiles'][1],tileset_name='johto',environment='ROUTE',music=row['music'],map_group_constant='MAPGROUP_MINNESOTA_COHESION',map_constant=row['map_constant'],border_block=5,blocks_label=name+'_Blocks',map_scripts_label=name+'_MapScripts',map_events_label=name+'_MapEvents',connections=[],connection_flags=None)
        for link in plan['new_reciprocal_links']:
            if link['from_map']==name:attrs['connections'].append(dict(direction=link['direction'],target_map=link['to_map'],offset=link['offset']))
            if link['to_map']==name:attrs['connections'].append(dict(direction=OPPOSITE[link['direction']],target_map=link['from_map'],offset=link['reverse_offset']))
        attrs['connection_flags']=' | '.join(e['direction'].upper() for e in attrs['connections']) or None
        attrs['location']=next(v['landmark_constant'] for v in plan['landmark_bindings'] if v['map']==name)
        eq(name+' every native attribute',attrs,at)
        eq(name+' native dimensions mirror',dict(width=at['width'],height=at['height']),b['map_dimensions'][row['map_constant']])
        eq(name+' johto native alias','johto',at['tileset_name'])
        eq(name+' exact raw authored trainer/sign programs',scripts,m['scripts'])
        eq(name+' no fabricated warps/coord scripts',dict(warps=[],coord_events=[],bg_events=[dict(x=s['tile'][0],y=s['tile'][1],event_type='BGEVENT_READ',script=s['id']) for s in d['signs']]),m['events'])
        objects=[object_record(template,v['id'],v['tile'],v['sprite'],dict(north='UP',south='DOWN',east='RIGHT',west='LEFT')[v['facing']],v['id'],True) for v in d['actors']]
        eq(name+' exact four new trainers',objects,m['objects'])
        ports={tuple(p) for port in d['ports'] for p in port['core']}
        occupied={(o['x'],o['y']) for o in objects};w,h=at['width']*2,at['height']*2
        reached={next(iter(ports))};queue=deque(reached)
        while queue:
            x,y=queue.popleft()
            for point in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if point not in reached and point not in occupied and 0<=point[0]<w and 0<=point[1]<h and kind(after,m,point) in ['FLOOR','TALL_GRASS']:
                    reached.add(point);queue.append(point)
        assert ports<=reached,(name,'actor-present port route')
        for obj in objects:
            x,y=obj['x'],obj['y'];assert any(p in reached for p in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]),(name,'trainer approach')
        for y in range(h):
            for x in range(w):
                k=kind(after,m,[x,y]);assert k in ['FLOOR','TALL_GRASS','WALL'],(name,'unexpected collision')
                if x in [0,w-1] or y in [0,h-1]:assert k==('FLOOR' if (x,y) in ports else 'WALL'),(name,'unconnected passable perimeter')
        binding=next(l for l in plan['landmark_bindings'] if l['map']==name);location=a['pokegear_landmarks']['map_to_landmark'][binding['reference_native_map']]
        assert not binding['add_landmark_row'] and 1<=binding['existing_landmark_id']<=127
        eq(name+' existing native location',location,at['location']);eq(name+' town-map registration',location,b['pokegear_landmarks']['map_to_landmark'][name])
        norm['pokegear_landmarks']['map_to_landmark'].pop(name)
        eq(name+' exact group83 map identity',(row['map_constant'],'MAPGROUP_MINNESOTA_COHESION'),(at['map_constant'],at['map_group_constant']))
        eq(name+' native metadata',dict(name=name,constant=row['map_constant'],groupName='MINNESOTA_COHESION',groupId=83,mapId=row['map_id'],width=at['width'],height=at['height'],environment='ROUTE',phoneService=at['phone_service']),b['runtime_map_metadata'][row['map_constant']])
        eq(name+' mirrored attributes',at,b['map_attributes'][name]);eq(name+' mirrored blocks',encoded(m['blocks']),b['map_blocks'][at['blocks_label']]);eq(name+' mirrored NPCs',objects,b['npcs'][name]);eq(name+' mirrored event wire',section_rows(m),b['map_scripts'][at['map_events_label']])
        for label,body in scripts.items():eq(label+' exact global raw body',body,b['map_scripts'][label])
        binding=next((v for v in plan['encounter_bindings_proposed'] if v['map']==name),None)
        if binding:
            wild=deepcopy(a['wild_encounters'][binding['native_reference_map']]);wild.update(map_name=name,water=None,water_rate=None)
            # Native serde omits these cleared default containers entirely.
            wild.pop('swarm_overrides',None);wild.pop('zones',None)
            eq(name+' exact native encounter clone',wild,b['wild_encounters'][name])
        else:assert name not in b['wild_encounters']
        for field,key in [('maps',name),('map_attributes',name),('map_dimensions',row['map_constant']),('runtime_map_metadata',row['map_constant']),('map_blocks',at['blocks_label']),('npcs',name),('wild_encounters',name)]:norm[field].pop(key,None)
        for label in set(scripts)|{at['map_scripts_label'],at['map_events_label']}:norm['map_scripts'].pop(label)
    global_=norm['story_event_script_constants']['global'];eq('new group83',83,global_.pop('MAPGROUP_MINNESOTA_COHESION'))
    next_flag=max(v for k,v in a['story_event_script_constants']['global'].items() if k.startswith('EVENT_'))
    actors=[v for name in sorted(dossiers) for v in dossiers[name]['actors']]
    allocated={}
    for actor in actors:
        next_flag+=1;eq(actor['id']+' fresh defeat flag',next_flag,global_.pop(actor['event_flag']))
        cls=actor['trainer_class'];previous=[a['story_event_script_constants']['global'][v['trainer_id']] for v in a['trainers']['trainers'].values() if v['trainer_class']==cls and v['trainer_id'] in a['story_event_script_constants']['global']]
        number=max(previous+allocated.get(cls,[]))+1;allocated.setdefault(cls,[]).append(number)
        eq(actor['id']+' next native class index',number,global_.pop(actor['id']))
        trainer=norm['trainers']['trainers'].pop(actor['id']);template_=next(v for k,v in sorted(a['trainers']['trainers'].items()) if v['trainer_class']==cls)
        eq(actor['id']+' source party',[(v['species'],v['level']) for v in actor['party']],[(v['species'],v['level']) for v in trainer['party']])
        assert all(v['dvs']==template_['party'][0]['dvs'] and v['item'] is None for v in trainer['party'])
        want=deepcopy(template_);want.update(trainer_id=actor['id'],name=actor['trainer_name'].upper(),party=trainer['party'],win_quote='',lose_quote='')
        eq(actor['id']+' native trainer fields',want,trainer)
    eq('all residual original data/catalogues/programs/recovery/HOME exact',a,norm)
    for field in ['runtime_files','compiled_audio','audio_manifest','audio_compression']:eq('complete original '+field,before.get(field),after.get(field))
    report=deepcopy(before['report']);report['maps']=681;report['manifests'] += [f"{plan['id']}-{r['map_id']}" for r in plan['new_maps']]
    for key in ['reachable_maps','solvable_maps']:report[key] += list(new)
    eq('exact bounded report',report,after['report'])
    return dict(checks_passed=len(checks),checks=checks,geometry=topology(after,plan),old_maps=677,new_maps=4,declared_cells=42,native_gameplay_proven=False,limitations=['Read-only data/collision proof; no native controller execution.','New compiled typed-index completeness and legal learned moves are separately checked by native parser/asset regressions.'])


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--before',type=Path,required=True);p.add_argument('--contract',type=Path,default=ROOT/'planning/minnesota/connections/state-cohesion-v01/source-v01/integration.contract.json');p.add_argument('--pack',type=Path);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    plan,layouts,dossiers,patches,hashes=inputs(args.contract)
    assert sha(args.before.read_bytes())==BASE
    before=load_pack(args.before);prepare(before,plan,patches)
    report=dict(status='SOURCE_PREFLIGHT_PASS',inputs=hashes,base_file_sha256=BASE,contract_sha256=sha(args.contract.read_bytes()),native_gameplay_proven=False)
    if args.pack:
        after=load_pack(args.pack);report.update(audit(before,after,plan,layouts,dossiers,patches));report.update(status='WRITTEN_PACK_PASS',file_sha256=sha(args.pack.read_bytes()),identity=after['identity'],raw_identities=[raw_identity(args.before),raw_identity(args.pack)])
        assert all(row['exact'] for row in report['raw_identities'])
        print(json.dumps(dict(stage='positive_written_checks_passed',checks=report['checks_passed'],geometry=report['geometry'])),flush=True)
        negative=[]
        mutations={
            'old_room':lambda b:b['data']['maps']['MnStPaulUnionDepot']['blocks'].__setitem__(0,0),
            'old_raw_story':lambda b:b['data']['maps']['MnMinneapolis']['script_control_commands'][0].__setitem__('command','audit_corrupt'),
            'old_party':lambda b:next(iter(b['data']['trainers']['trainers'].values()))['party'][0].__setitem__('level',1),
            'audio':lambda b:b.__setitem__('audio_manifest',{}),
            'catalog_price':lambda b:b['data']['items']['POTION'].__setitem__('price',999),
            'HOME_recovery':lambda b:b['data']['runtime_spawn_points'].clear(),
            'outside_cell':lambda b:b['data']['maps']['MnR15C00R00']['blocks'].__setitem__(0,0),
            'river_preservation':lambda b:b['data']['maps']['MnMississippiWinonaMouth']['blocks'].__setitem__(0,0),
            'new_typed_guard_flag':lambda b:next(v for v in b['data']['maps']['MnBrainerd']['script_flag_commands'] if v['source_script']==plan['guard_proposals'][0]['id']).__setitem__('flag_id','ENGINE_ZEPHYRBADGE'),
            'injected_guard_reward':lambda b:b['data']['maps']['MnBrainerd']['script_item_grants'].append(dict(command='giveitem',item_id='MASTER_BALL',quantity=1,source_script=plan['guard_proposals'][0]['id'],command_index=0,verbose=False)),
            'guard_prerequisite':lambda b:b['data']['maps']['MnBrainerd']['scripts'][plan['guard_proposals'][0]['id']][0]['args'].__setitem__(0,'ENGINE_ZEPHYRBADGE'),
            'guard_event_mirror':lambda b:b['data']['map_scripts'][b['data']['maps']['MnBrainerd']['attributes']['map_events_label']].pop(),
            'landmark_binding':lambda b:b['data']['pokegear_landmarks']['map_to_landmark'].pop(plan['new_maps'][0]['id']),
            'new_actor':lambda b:b['data']['maps'][plan['new_maps'][0]['id']]['objects'][0].__setitem__('x',0),
            'bridge_art':lambda b:b['runtime_files'].__setitem__('data/tilesets/johto_metatiles.bin',b''),
            'new_offset':lambda b:b['data']['maps']['MnBrainerd']['attributes']['connections'][-1].__setitem__('offset',99)}
        for label,mutate in mutations.items():
            bad=deepcopy(after);mutate(bad)
            assert bad!=after,('ineffective corruption',label)
            try:audit(before,bad,plan,layouts,dossiers,patches)
            except (AssertionError,KeyError,StopIteration):
                negative.append(label)
                print(json.dumps(dict(stage='corruption_rejected',mutation=label)),flush=True)
            else:raise AssertionError('accepted '+label)
        report['corruption_negatives_rejected']=negative
    args.output.mkdir(parents=True,exist_ok=False);raw=(json.dumps(report,indent=2)+'\n').encode();file=args.output/('proof-'+sha(raw)+'.json');file.write_bytes(raw)
    print(json.dumps(dict(status=report['status'],proof=str(file),checks=report.get('checks_passed'))))

if __name__=='__main__':main()
