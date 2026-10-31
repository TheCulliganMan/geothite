#!/usr/bin/env python3
"""Read-only complete-parent preservation audit for the native pine island."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'tools'))
from render_reference import load_pack
cli=argparse.ArgumentParser(description=__doc__)
for name in ['parent','child','source','output']:cli.add_argument('--'+name,type=Path,required=True)
a=cli.parse_args();assert not a.output.exists()
p,q=load_pack(a.parent),load_pack(a.child);before,after=p['data'],q['data'];name='MnLeechLake';trainer='MN_EXTRA_LEECH_ISLAND_NATURALIST';hint='MN_LEECH_SHORE_FISHERText'
assert before.keys()==after.keys()
changed={'maps','map_scripts','map_blocks','story_event_script_constants','wild_encounters','npcs','trainers','story_events','global_scripts'}
for key in before:
 if key not in changed:assert before[key]==after[key],key
assert before['maps'].keys()==after['maps'].keys()
for key,m in before['maps'].items():
 if key!=name:assert after['maps'][key]==m,key
old,new=before['maps'][name],after['maps'][name]
allowed={'scripts','scripted_trainer_battles','script_flag_commands','script_object_commands','script_map_commands','script_text_commands','script_text_bodies','script_control_commands','script_field_pickups','script_runtime_commands','map_event_section_commands','events','objects','blocks'}
assert old.keys()==new.keys()
for key in old:
 if key not in allowed:assert old[key]==new[key],key
source=json.loads(a.source.read_text());rows=[int(v,16)for row in source['rows']for v in row.split()];assert new['blocks']==rows
terrain_changes=[]
for i,(x,y)in enumerate(zip(old['blocks'],new['blocks'])):
 if x!=y:
  xx,yy=i%32,i//32;assert (22<=xx<=28 and 12<=yy<=20)or(xx==30 and yy in [9,10]);terrain_changes.append([xx,yy])
assert new['attributes']==old['attributes']
assert new['objects'][:len(old['objects'])]==old['objects'] and len(new['objects'])==len(old['objects'])+2
assert [o['script']for o in new['objects'][-2:]]==['MN_LEECH_ISLAND_WATER',trainer]
for key in old['events']:
 if key=='bg_events':assert new['events'][key][:-1]==old['events'][key] and new['events'][key][-1]==dict(x=61,y=20,event_type='BGEVENT_ITEM',script='MN_LEECH_EDDY_CACHE')
 else:assert old['events'][key]==new['events'][key]
for key in ['scripts','script_text_bodies']:
 for label,body in old[key].items():
  if label!=hint:assert new[key][label]==body,(key,label)
 assert all(label.startswith(trainer)or label in ['MN_LEECH_EDDY_CACHE','MN_LEECH_ISLAND_WATER']for label in new[key]if label not in old[key])
assert old['scripts'][hint][:-1]==new['scripts'][hint][:len(old['scripts'][hint])-1]
assert old['scripts'][hint][-1]==new['scripts'][hint][-1]
for key in allowed-{'scripts','script_text_bodies','map_event_section_commands','events','objects','blocks'}:
 assert new[key][:len(old[key])]==old[key],key
 for row in new[key][len(old[key]):]:
  assert row.get('source_script','').startswith(trainer)or row.get('source_script')in ['MN_LEECH_EDDY_CACHE','MN_LEECH_ISLAND_WATER'],(key,row)
for key,exceptions in [('map_scripts',{hint,'MnLeechLake_MapEvents'}),('map_blocks',{'MnLeechLake_Blocks'}),('npcs',{name})]:
 for label,value in before[key].items():
  if label not in exceptions:assert after[key][label]==value,(key,label)
 if key=='map_scripts':assert set(after[key])-set(before[key])==set(new['scripts'])-set(old['scripts'])
 else:assert before[key].keys()==after[key].keys()
expected=deepcopy(before['wild_encounters'].get(name,source['encounters']))
if name in before['wild_encounters']:expected.setdefault('zones',[]).extend(source['encounters']['zones'])
assert after['wild_encounters'][name]==expected
assert set(after['wild_encounters'])==set(before['wild_encounters'])|{name}
for label,value in before['wild_encounters'].items():
 if label!=name:assert after['wild_encounters'][label]==value
for label,value in before['trainers'].items():
 if label!='trainers':assert after['trainers'][label]==value
 else:
  assert set(after['trainers'][label])-set(value)=={trainer}
  for identity,original in value.items():assert after['trainers'][label][identity]==original
constants=before['story_event_script_constants'];added_constants={trainer,'EVENT_MN_EXTRA_LEECH_ISLAND_NATURALIST','EVENT_MN_LEECH_ISLAND_WATER','EVENT_MN_LEECH_EDDY_CACHE'}
for key,value in constants.items():
 if key!='global':assert after['story_event_script_constants'][key]==value
 else:
  revised=after['story_event_script_constants'][key];assert set(revised)-set(value)==added_constants
  for identity,n in value.items():assert revised[identity]==n
for original,revised in zip(before['story_events'],after['story_events']):
 assert original.keys()==revised.keys()
 for catalog,content in original.items():
  newer=revised[catalog]
  for label,body in content.items():
   if catalog=='StandardScripts'and label=='GlobalScriptRoots':assert [v for v in newer[label]if v not in ['Script_UsedWhirlpool','Script_ForcedMovement']]==[v for v in body if v not in ['Script_UsedWhirlpool','Script_ForcedMovement']]
   else:assert newer[label]==body,(catalog,label)
  additions=set(newer)-set(content)
  assert not additions or(catalog=='OverworldEvents'and len(additions)==9),catalog
old_global,new_global=before['global_scripts'],after['global_scripts'];labels=set(old_global['scripts'])
for key,value in old_global.items():
 candidate=new_global[key]
 if isinstance(value,dict):
  for label,body in value.items():assert candidate[label]==body,('global',key,label)
 elif isinstance(value,list):
  retained=[row for row in candidate if (row.get('source_script')or row.get('name'))in labels]if value else[]
  if value:assert retained==value,('global',key)
 else:assert candidate==value,('global',key)
assert p['audio_manifest']==q['audio_manifest'] and p['compiled_audio']==q['compiled_audio']
assert set(q['runtime_files'])-set(p['runtime_files'])=={'data/tilesets/mn_headwaters_animations.json'}
for key,value in p['runtime_files'].items():assert q['runtime_files'][key]==value,key
assert q['report']['manifests']==p['report']['manifests']+['minnesota-leech-eddy-native-animation-v1','minnesota-leech-pine-island-v1']
for key,value in p['report'].items():
 if key!='manifests':assert q['report'][key]==value,key
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
result=dict(status='PASS',parent=str(a.parent),parent_sha256=sha(a.parent),pack=str(a.child),pack_sha256=sha(a.child),content_identity=q['identity']['content_hash'],unchanged_complete_maps=len(before['maps'])-1,changed_map=name,changed_metatiles=terrain_changes,all_original_assets_audio_trainers_world_connections_and_services_preserved=True,original_global_programs_preserved=True,source_sha256=sha(a.source))
a.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
