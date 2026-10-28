#!/usr/bin/env python3
"""Read-only Ely builder preimage/allocation certification on exact warm parent."""
import cbor2,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5]
PATH=ROOT/'content-packs/minnesota-superior-warm-coast-v01.crystalpack';SHA='ebea77ccb9b9dbe54aa5101e32590c1b075ed8130f6e7fcfe2d8c8451dab11d8'
OUT=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/ely-parent-v03';assert not OUT.exists();OUT.mkdir()
raw=PATH.read_bytes();assert hashlib.sha256(raw).hexdigest()==SHA;assert raw[:12]==b'CRYSTALPACK\0'and int.from_bytes(raw[14:18],'big')==len(raw)-22
pack=cbor2.loads(raw[22:]);d=pack['data'];src=(ROOT/'crates/crystal-assets/src/ely_league_rebase.rs').read_text()
def h(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
moduletext=src.split('const TEMPLATES:')[1].split('const TILESETS:')[0].replace('ROAD,','"MnR13C00R02",')
tstext=src.split('const TILESETS:')[1].split('fn hash(')[0]
modules=[dict(name=n,expected=e,actual=h(d['maps'][n]))for n,e in re.findall(r'\(\s*"([^"]+)",\s*"([0-9a-f]{64})"',moduletext)]
tilesets=[dict(name=n,expected=e,actual=h(d['tilesets'][n]))for n,e in re.findall(r'\(\s*"([^"]+)",\s*"([0-9a-f]{64})"',tstext)]
assert len(modules)==11and len(tilesets)==8
for row in modules+tilesets:row['pass']=row['expected']==row['actual']
const=d['story_event_script_constants']['global'];events=[v for k,v in const.items()if k.startswith('EVENT_')];spawns=[s['identifier']for s in d['runtime_spawn_points'].values()];elynames=['MnEly','MnElyPokecenter1F','MnElyMart','MnLeagueLobby','MnLeagueRoom1','MnLeagueRoom2','MnLeagueRoom3','MnLeagueRoom4','MnLeagueChampion','MnLeagueHallOfFame'];trainers=['MN_LEAGUE_1','MN_LEAGUE_2','MN_LEAGUE_3','MN_LEAGUE_4','MN_CHAMPION'];flags=['EVENT_MN_LEAGUE_RUN_ACTIVE','EVENT_MN_LEAGUE_CHAMPION'];groups=[k for k,v in d['runtime_map_metadata'].items()if v['groupId']==81]
checks=dict(all_11_module_preimages=all(x['pass']for x in modules),all_8_tileset_preimages=all(x['pass']for x in tilesets),maps_683=len(d['maps'])==683,pokemon_251=len(d['pokemon'])==251,moves_251=len(d['moves'])==251,max_spawn_277=max(spawns)==277,next_event_2385=max(events)+1==2385,group_81_free=not groups,group_constant_free='MAPGROUP_MINNESOTA_ELY'not in const,ely_mapnames_free=all(n not in d['maps']for n in elynames),ely_mart_free='MART_MN_ELY'not in d['marts'],ely_flags_trainers_free=all(n not in const and n not in d['trainers']['trainers']for n in flags+trainers),restored_survey_present='EVENT_MN_SURVEY_RESTORED'in const)
story=json.loads((ROOT/'planning/minnesota/story.json').read_bytes());storyhash=h(story['league']);checks['exact_league_source']=storyhash=='9fe88edb2abada391a04d6c612bf474a2768932f54339c0e039bd43b1ed71c1b'
sources=[]
for f in ['planning/minnesota/towns/MnEly.json','planning/minnesota/towns/MnEly.layout.json']:
 sources.append(dict(path=f,sha256=hashlib.sha256((ROOT/f).read_bytes()).hexdigest()))
# Installed source profiles only; no source programs, raw catalog, or bitmap exported.
profiles=[]
for prefix in ['MnR12','MnR13']:
 for n,m in sorted(d['maps'].items()):
  if n.startswith(prefix)and'C'in n:
   a=m['attributes'];profiles.append(dict(map=n,dimensions=[a['width'],a['height']],tileset=a['tileset_name'],time_of_day=a.get('time_of_day'),block_counts={f'{i:02x}':m['blocks'].count(i)for i in sorted(set(m['blocks']))}))
result=dict(status='PASS'if all(checks.values())else'FAIL',actual_pack_sha256=SHA,identity=pack['identity'],checks=checks,module_preimages=modules,tileset_preimages=tilesets,source_byte_hashes=sources,league_subtree_sha256=storyhash,current_allocation=dict(maps=len(d['maps']),max_spawn=max(spawns),next_event=max(events)+1,group81_occupants=groups),recommended_new_allocations=dict(group=81,event_active=max(events)+1,event_champion=max(events)+2,spawn_town=max(spawns)+1,spawn_clinic=max(spawns)+2,new_maps=10,result_maps=693),existing_parent_whitelist_accepts_actual_identity=False,required_code_change='Narrowly certify this exact actual warm identity in parent() with 683/251/251/maxspawn277/nextevent2385 gate; do not alter any native template/tileset/source preimage or residual validator.',profiles=profiles)
(OUT/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k]for k in['status','checks','current_allocation','recommended_new_allocations','source_byte_hashes']}));print('MODULEDIFF',[(x['name'],x['actual'])for x in modules if not x['pass']]);print('TS DIFF',[(x['name'],x['actual'])for x in tilesets if not x['pass']])
