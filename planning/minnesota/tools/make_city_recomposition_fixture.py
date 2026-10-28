"""Read-only coordinate adapter for production activity tests on recomposed cities.

Usage: python make_city_recomposition_fixture.py --base BASE.crystalpack
  --activities ORIGINAL.expansion.contract.json --recomposition RECOMPOSITION.json
  --published CURRENT.crystalpack --output UNIQUE_FOLDER

Only the exact optional Minneapolis north-water gate is recognized. Existing
programs/tables remain strict; geometry and gameplay require their own native
acceptance. Omitting --published produces an unverified source fixture.

Writes only a fixture plan/receipt into a NEW review directory. Never builds,
exports or writes a pack and never changes canonical source or runtime state.
"""
from pathlib import Path
import argparse,copy,hashlib,json

from render_reference import load_pack

if not __debug__:
 raise SystemExit("Fixture verification requires ordinary Python; -O disables safety assertions")
sha=lambda b:hashlib.sha256(b).hexdigest()
canon=lambda v:json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def gate_scripts():
 """The six fixed authored labels; this is not a generic ASM interpreter."""
 c=lambda command,*args:dict(command=command,args=list(args))
 check='MnMPUpperPoolBuoyCheck';refresh='MnMPUpperPoolBuoyRefresh'
 opened='MnMPUpperPoolBuoyOpen';denied='MnMPUpperPoolBuoyDenied'
 done='MnMPUpperPoolBuoyDone';text='MnMPUpperPoolBuoyDeniedText'
 def checks(failure):
  return [c('checkflag','ENGINE_PLAINBADGE'),c('iffalse',failure),
          c('checkflag','ENGINE_FOGBADGE'),c('iffalse',failure),c('sjump',opened)]
 return {check:checks(denied),refresh:checks(done),
         opened:[c('changeblock','32','2','88'),c('changeblock','34','2','53'),
                 c('changeblock','36','2','89'),c('end')],done:[c('end')],
         denied:[c('opentext'),c('writetext',text),c('waitbutton'),c('closetext'),c('end')],
         text:[c('text','"River gate closed."'),c('line','"Earn CURRENT and"'),
               c('para','"CAPITOL BADGES."'),c('done')]}

def gate_coordinates():
 return [dict(x=x,y=y,scene_id='-1',script_name='MnMPUpperPoolBuoyCheck')
         for y in [1,4]for x in range(32,38)]

def gate_typed_additions():
 """Exact encodings of this fixed recipe from current native parser types."""
 tables={field:[]for field in ['script_flag_commands','script_control_commands',
                              'script_text_commands','script_block_changes']}
 text='MnMPUpperPoolBuoyDeniedText'
 for label,body in sorted(gate_scripts().items()):
  for index,entry in enumerate(body):
   command,args=entry['command'],entry['args'];origin=dict(source_script=label,command_index=index)
   if command=='checkflag':
    tables['script_flag_commands'].append(dict(command=command,flag_id=args[0],**origin))
   elif command in ['iffalse','sjump','end']:
    target=args[0]if args else None
    tables['script_control_commands'].append(dict(command=command,compare_value=None,
      target_label=target,resolved_target_script=target,**origin))
   elif command in ['opentext','writetext','waitbutton','closetext']:
    tables['script_text_commands'].append(dict(command=command,text_label=args[0]if args else None,**origin))
   elif command=='changeblock':
    tables['script_block_changes'].append(dict(x=int(args[0]),y=int(args[1]),block_id=int(args[2]),**origin))
 tables['script_text_bodies']={text:dict(label=text,commands=[dict(**entry,command_index=i)
     for i,entry in enumerate(gate_scripts()[text])])}
 return tables

def event_section(module):
 """Exact existing native event-wire shape, including original record ordering."""
 rows=[]
 def add(command,args=()):
  rows.append(dict(command=command,args=list(args),command_index=len(rows)))
 add('db',['0','0']);add('def_warp_events')
 for w in module['events']['warps']:
  add('warp_event',[str(w['x']),str(w['y']),w['target_map_constant'],str(w['target_warp_id'])])
 add('def_coord_events')
 for e in module['events']['coord_events']:
  add('coord_event',[str(e['x']),str(e['y']),e['scene_id']or'-1',e['script_name']])
 add('def_bg_events')
 for e in module['events']['bg_events']:
  add('bg_event',[str(e['x']),str(e['y']),e['event_type'],e['script']])
 add('def_object_events')
 for o in module['objects']:
  add('object_event',[str(o['x']),str(o['y']),o['sprite'],o['spritemovedata'],
      *[str(o[k])for k in ['move_range_x','move_range_y','hram_x','hram_y','pal']],
      o['object_type'],str(o['radius']),o['script'],o['event_flag']])
 return rows

def wire(rows):
 return [dict(command=r['command'],args=r['args'])for r in rows]

def verified_programs(old,actual,published,enabled):
 """Remove only exact declared gate additions; compare every other field."""
 normalized=copy.deepcopy(actual)
 if enabled:
  scripts=gate_scripts();labels=set(scripts)
  assert not labels&set(old['scripts']),'gate labels already exist in original city'
  assert actual['scripts']==dict(old['scripts'],**scripts),'fixed gate raw programs or original script bodies drifted'
  for label,body in scripts.items():
   assert published['data']['map_scripts'].get(label)==body,'gate global/raw mirror drift: '+label
   normalized['scripts'].pop(label)
  for field,new in gate_typed_additions().items():
   if isinstance(new,list):
    assert actual[field]==old[field]+new,'old parsed prefix or exact gate tail drifted: '+field
   else:
    assert not set(new)&set(old[field]),'gate parsed text label occupied'
    assert actual[field]==dict(old[field],**new),'old parsed text or exact gate body drifted'
   normalized[field]=copy.deepcopy(old[field])
  callback=dict(command='callback',args=['MAPCALLBACK_TILES','MnMPUpperPoolBuoyRefresh'],
                command_index=len(old['map_script_section_commands']))
  assert actual['map_script_section_commands']==old['map_script_section_commands']+[callback],'gate callback or old callback prefix drifted'
  normalized['map_script_section_commands']=copy.deepcopy(old['map_script_section_commands'])
 for field in old:
  if field not in ['attributes','blocks','objects','events','map_event_section_commands']:
   assert normalized[field]==old[field],'city original activity/program table drift: '+field
 # Whole event and script section wires are derived explicitly, never ignored.
 assert actual['map_event_section_commands']==event_section(actual),'typed native event section is incoherent'
 for field,attribute in [('map_event_section_commands','map_events_label'),
                         ('map_script_section_commands','map_scripts_label')]:
  label=actual['attributes'][attribute]
  assert published['data']['map_scripts'][label]==wire(actual[field]),'global native section mirror drift: '+label
 assert published['data']['npcs'][actual['id']]==actual['objects'],'global native object mirror drift'


def adapt(base,activities,recomposition,published=None):
 city=recomposition['city'];old=base['data']['maps'][city]
 assert activities['city']==city
 assert sha(canon(old))==recomposition['native_module_sha256'],'stale installed city preimage'
 assert [old['attributes']['width'],old['attributes']['height']]==recomposition['old_dimensions_metatiles']==activities['dimensions_metatiles'],'activity/base extent mismatch'
 for room in activities['rooms']:
  actual=base['data']['maps'][room['id']]
  assert [actual['attributes']['width'],actual['attributes']['height']]==room['dimensions_metatiles'],'room fixture extent drift'
  assert [int(v,16)for row in room['rows']for v in row.split()]==actual['blocks'],'room fixture source rows drift'
  actors={o['object_identifier']:o for o in actual['objects']}
  for field in ['residents','trainers','items']:
   for actor in room[field]:
    native=actors[actor['id']];assert actor['tile']==[native['x'],native['y']],'room fixture actor coordinate drift'
  for label,commands in room['programs'].items():assert actual['scripts'].get(label)==commands,'room fixture program drift'
 enabled=recomposition.get('north_water_checkpoint',False)
 assert isinstance(enabled,bool),'north_water_checkpoint must be boolean'
 if enabled:
  assert city=='MnMinneapolis'and recomposition['dimensions_metatiles']==[40,32],'fixed gate requires compact40x32 Minneapolis'
 plan=copy.deepcopy(activities);city_new=copy.deepcopy(old)
 categories={'object':city_new['objects'],'background':city_new['events']['bg_events'],'coordinate':city_new['events']['coord_events'],'warp':city_new['events']['warps']}
 expected={(k,i)for k,rows in categories.items()for i in range(len(rows))};seen=set()
 for a in recomposition['anchor_overrides']:
  key=a['kind'],a['index'];assert key in expected and key not in seen,'missing/duplicate/unknown indexed native anchor';seen.add(key)
  row=categories[a['kind']][a['index']];assert [row['x'],row['y']]==a['before'],'stale indexed before coordinate'
  row['x'],row['y']=a['after']
 assert seen==expected,'incomplete indexed native anchors'
 assert len(recomposition['spawn_overrides'])==len([s for s in base['data']['runtime_spawn_points'].values()if s['mapName']==city]),'missing outdoor recovery'
 for field in ['schema_version','native_module_sha256','old_dimensions_metatiles','dimensions_metatiles','arrival_core','anchor_overrides','connections','spawn_overrides']:plan[field]=copy.deepcopy(recomposition[field])
 plan['translation_core']=[0,0]
 actorby={o['object_identifier']:i for i,o in enumerate(old['objects'])}
 for field in ['residents','trainers','items']:
  for a in plan[field]:
   i=actorby[a['id']];before=old['objects'][i];after=city_new['objects'][i];assert a['tile']==[before['x'],before['y']],'activity actor identity/preimage mismatch';delta=[after['x']-before['x'],after['y']-before['y']];a['tile']=[after['x'],after['y']]
   for key in ['approach','counter_tile']:
    if isinstance(a.get(key),list):a[key]=[a[key][j]+delta[j]for j in range(2)]
 bgby={o['script']:i for i,o in enumerate(old['events']['bg_events'])}
 for a in plan['signs']:
  i=bgby[a['id']];before=old['events']['bg_events'][i];after=city_new['events']['bg_events'][i];assert a['tile']==[before['x'],before['y']],'activity BG preimage mismatch';a['tile']=[after['x'],after['y']]
 oldwarps={w['index']:w for w in old['events']['warps']};newwarps={w['index']:w for w in city_new['events']['warps']}
 for d in plan['doors']:
  before=oldwarps[d['warp_index']];after=newwarps[d['warp_index']];assert d['tile']==[before['x'],before['y']],'door fixture preimage';d['tile']=[after['x'],after['y']]
  ret_before=oldwarps[d['return_warp_index']];ret=newwarps[d['return_warp_index']];assert d['return_tile']==[ret_before['x'],ret_before['y']],'passive fixture preimage';d['return_tile']=[ret['x'],ret['y']]
  # Existing Door.arrival_tile is the exterior return Floor, NOT room arrival.
  if 'arrival_tile'in d:d['arrival_tile']=d['return_tile'][:]
 assert plan['rooms']==activities['rooms'],'never mutate a room or its activity program'
 if enabled:
  assert len(city_new['events']['coord_events'])==8,'fixed gate requires exactly eight old coordinate rows'
  added=gate_coordinates();positions={(c['x'],c['y'])for c in city_new['events']['coord_events']}
  assert not positions&{(c['x'],c['y'])for c in added},'gate coordinates overlap an old event'
  assert not {(o['x'],o['y'])for o in city_new['objects']}&{(c['x'],c['y'])for c in added},'gate approach has an actor body'
  city_new['events']['coord_events'].extend(added)
 plan['north_water_checkpoint']=enabled
 plan['controller_coordinate_events']=copy.deepcopy(city_new['events']['coord_events'])
 plan['controller_fixture_scope']='EXPLICIT_LOCAL_SETUP_THEN_EXISTING_PRODUCTION_PRESS_TESTS; NOT_EARNED_PROGRESS'
 if published is not None:
  actual=published['data']['maps'][city];assert [actual['attributes']['width'],actual['attributes']['height']]==recomposition['dimensions_metatiles'],'published extent mismatch'
  for field in ['objects','events']:
   assert actual[field]==city_new[field],'published native indexed records drifted: '+field
  # Room modules (including originals), full scripts/text/runtime tables/catalog
  # ownership must remain exact; this helper does not bless new art or geometry.
  for name,room in base['data']['maps'].items():
   if name.startswith(city) and name!=city:assert published['data']['maps'][name]==room,'published owned room drift: '+name
  verified_programs(old,actual,published,enabled)
  owned_sections={actual['attributes']['map_scripts_label'],actual['attributes']['map_events_label']}
  for label in old['scripts']:
   if label not in owned_sections:
    assert published['data']['map_scripts'].get(label)==base['data']['map_scripts'].get(label),'old program globalmirror drift: '+label
  if enabled:
   assert actual['attributes']['tileset_name']=='mn_minneapolis_city','gate city alias drift'
   collision=published['data']['tilesets']['mn_minneapolis_city']['collision']
   assert collision['31']==['BUOY','BUOY','WATER','WATER'],'gate native Buoy art collision drift'
   assert [actual['blocks'][40+x]for x in [16,17,18]]==[49,49,49],'published gate base is not the exact closed31 recipe'
   for x,y in [(c['x'],c['y'])for c in gate_coordinates()]:
    block=actual['blocks'][y//2*40+x//2]
    assert collision[f'{block:02x}'][y%2*2+x%2]=='WATER','published gate approach must be Water'
   for block in ['58','35','59']:
    assert collision[block]==['WATER']*4,'gate exact native opening no longer Water'
  for field in ['items','marts','trainers','moves']:
   assert published['data'][field]==base['data'][field],'preserved activity catalog drift: '+field
  for a in recomposition['spawn_overrides']:
   record=published['data']['runtime_spawn_points'][str(a['identifier'])];assert [record['tileX'],record['tileY']]==a['after'],'published recovery mismatch'
  for link in recomposition['connections']:
   assert dict(direction=link['direction'],target_map=link['target'],offset=link['offset'])in actual['attributes']['connections'],'published actual road connection mismatch'
 return plan
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--base',type=Path,required=True);ap.add_argument('--activities',type=Path,required=True);ap.add_argument('--recomposition',type=Path,required=True);ap.add_argument('--published',type=Path);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
 all_inputs=[args.base,args.activities,args.recomposition,Path(__file__)]+([args.published]if args.published else[])
 input_hashes={str(p):sha(p.read_bytes())for p in all_inputs}
 activity_bytes=args.activities.read_bytes();recomposition_bytes=args.recomposition.read_bytes()
 base=load_pack(args.base);activities=json.loads(activity_bytes);recomposition=json.loads(recomposition_bytes);published=load_pack(args.published)if args.published else None;plan=adapt(base,activities,recomposition,published)
 assert input_hashes=={str(p):sha(p.read_bytes())for p in all_inputs},'input/pack/tool changed during adapter verification'
 args.output.mkdir(parents=True,exist_ok=False)
 for name,path in [('activities',args.activities),('recomposition',args.recomposition),('adapter',Path(__file__))]:
  dst=args.output/'inputs'/f'{name}{path.suffix}';dst.parent.mkdir(exist_ok=True);dst.write_bytes(path.read_bytes())
 raw=(json.dumps(plan,indent=2,ensure_ascii=False)+'\n').encode();(args.output/'city.controller.plan.json').write_bytes(raw)
 record=dict(status='PUBLISHED_PACK_COORDINATE_AND_ACTIVITY_VERIFIED_NOT_CONTROLLER_RUN'if published else'SOURCE_FIXTURE_ONLY_NOT_VALIDATED_AGAINST_A_COMPACT_NATIVE_PACK',base=dict(path=str(args.base),sha256=sha(args.base.read_bytes())),published=dict(path=str(args.published),sha256=sha(args.published.read_bytes()))if published else None,city=plan['city'],dimensions=plan['dimensions_metatiles'],arrival_core=plan['arrival_core'],room_count=len(plan['rooms']),door_count=len(plan['doors']),coordinate_event_count=len(plan['controller_coordinate_events']),north_water_checkpoint=plan['north_water_checkpoint'],plan_sha256=sha(raw),inputs=[dict(path=str(p),sha256=sha(p.read_bytes()))for p in[args.activities,args.recomposition,Path(__file__)]])
 (args.output/'receipt.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record,indent=2))
