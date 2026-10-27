#!/usr/bin/env python3
"""Hand-authored R18 dune margin, winding dry approach and grass-shortcut proof."""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import time

from PIL import Image,ImageDraw
from render_archive import save_render
from render_reference import layout_blocks,load_pack
from review_league_approach import check as generic_check
from review_mille_lacs_shore import check_lake,walking_design
from review_r15_refinement import graph,sha,write,adjacent
from review_r18_shorewalk_refinement import render,shortest

ROOT=Path(__file__).resolve().parents[3]
BEFORE=ROOT/'planning/minnesota/renders/mille-lacs-shore/ribbon-20261027/before-shorewalk-v04'
PACK_SHA='da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50'

def largest_square(kind,allowed,bounds):
 x0,y0,x1,y1=bounds;table={};best=(0,0,0)
 for y in range(y0,y1):
  for x in range(x0,x1):
   n=1+min(table.get((x-1,y),0),table.get((x,y-1),0),table.get((x-1,y-1),0))if(x,y)in allowed and kind((x,y))=='FLOOR'else 0;table[x,y]=n
   if n>best[0]:best=(n,x-n+1,y-n+1)
 return {'width_core':best[0],'bounds':[best[1],best[2],best[1]+best[0],best[2]+best[0]]}

def inspect(d,l,p,od,ol):
 started=time.perf_counter();ob,ok,oa,oreach,oc,oh=graph(od,ol,p);b,kind,allowed,reach,cost,hazards=graph(d,l,p);checks=[]
 def check(name,value):checks.append({'check':name,'passed':bool(value)})
 check('all existing events story actors parties rewards habitats metadata exact',all(d.get(k)==od.get(k)for k in set(d)|set(od)if k not in{'main_path_metatiles','viewpoints','review_contract'}))
 check('original shoreloop and all views exact prefixes',d['loops']==od['loops']and d['viewpoints'][:len(od['viewpoints'])]==od['viewpoints'])
 check('all original shorewalk editorial records retained',all(d['review_contract'].get(k)==v for k,v in od['review_contract'].items()))
 check('all layout metadata and source art recipes exact',all(l.get(k)==ol.get(k)for k in set(l)|set(ol)if k!='layers'))
 check('full approved shorewalk layer prefix exact',l['layers'][:len(ol['layers'])]==ol['layers'])
 delta=[{'metatile':[i%40,i//40],'before':f'{a:02x}','after':f'{z:02x}'}for i,(a,z)in enumerate(zip(ob,b))if a!=z]
 check('larger central-ribbon changes bounded meta21..26/3..9',bool(delta)and all(21<=c['metatile'][0]<27 and 3<=c['metatile'][1]<10 for c in delta))
 check('only original grass road rail sign blocks used',all(c['after']in{'03','01','48','49','45'}for c in delta))
 check('approved new shorewalk and dune road bytes exact',all(b[y*40+x]==ob[y*40+x]for y in range(6,11)for x in range(28,37)))
 check('all old water and native sea-edge art exact',all(b[i]==ob[i]for i in range(len(b))if any(t in{'WATER','BUOY'}for t in p['data']['tilesets'][l['compiled_tileset']]['collision'][f'{ob[i]:02x}'])))
 check('every original grass quadrant preserved',all(kind((x,y))=='TALL_GRASS'for y in range(48)for x in range(80)if ok((x,y))=='TALL_GRASS'))
 check('no incidental Cut hop or new unsupported collision',not any(kind((x,y))=='CUT_TREE'or'HOP'in kind((x,y))for y in range(48)for x in range(80)))
 common,_,_=generic_check(walking_design(d),l,p,b);check('complete native source geometry has no errors',not common['errors'])
 try:lake=check_lake(d,l,p);check('original flatshore piers all walking quest actors Surf islet seam boundary contract',True)
 except AssertionError as e:lake={'error':str(e)};check('original flatshore piers all walking quest actors Surf islet seam boundary contract',False)
 west,east=[tuple(2*v for v in e['gate_metatile'])for e in d['endpoints']];seen=reach(west);oldseen=oreach(west)
 check('every retained old reachable cell remains connected',not((oldseen&allowed)-seen))
 check('both regional endpoints reachable both ways',east in seen and west in reach(east))
 prices=[]
 for label,a,z in [('west-east',west,east),('east-west',east,west)]:
  old=oc(a,{z});new=cost(a,{z});check(label+' still exactly six necessary grass steps',old==new==6);prices.append({'id':label,'before':old,'candidate':new})
 for a in d['items']:
  if a['id']in d['surf_only_actor_ids']:continue
  for label,st in [('west',west),('east',east)]:
   old=oc(st,set(adjacent(tuple(a['tile']))));new=cost(st,set(adjacent(tuple(a['tile']))));check(a['id']+'-'+label+' original resource grass cost retained',old==new);prices.append({'id':a['id']+'-'+label,'before':old,'candidate':new})
 for actor in d['residents']:
  approaches=set(adjacent(tuple(actor['tile'])));check(actor['id']+' walking report approach preserved',bool(approaches&seen))
 floor={at for at in allowed if kind(at)=='FLOOR'};start,target=(45,13),(48,13);dry=shortest(start,target,floor);direct=shortest(start,target,allowed);direct_grass=sum(kind(at)=='TALL_GRASS'for at in direct[1:])if direct else None
 check('meaningful choice: nine-step dry bend or three steps through two grass',dry is not None and direct is not None and len(dry)-1==9 and len(direct)-1==3 and direct_grass==2)
 check('dry approach really visits offset lakeview row16',dry is not None and all(at in dry for at in[(45,16),(46,16),(47,16),(48,16)]))
 check('main dry road annotations follow actual ordinary Floor',d['main_path_metatiles']==[[1,5],[6,5],[6,6],[22,6],[22,8],[24,8],[24,5],[38,5]])
 check('offset lookout protected with fitted wall cap and rail',kind((42,16))=='WALL'and all(kind((x,16))=='FLOOR'and kind((x,17))=='WALL'for x in range(43,48)))
 water_gaps=[(at,n)for at in[(x,16)for x in range(43,48)]for n in adjacent(at)if kind(n)=='WATER'];check('no new Surf hole in lookout rail',not water_gaps)
 check('original launch sign reading footing and Surf launch exact',all(kind(at)==ok(at)for at in[(52,16),(53,16),(52,17),(53,17),(49,19),(49,20)]))
 water=sum(kind((x,y))=='WATER'for y in range(12,21)for x in range(40,50));check('offset native player view actually shows lake',water>=15)
 oldsq=largest_square(ok,oa,(42,6,54,18));newsq=largest_square(kind,allowed,(42,6,54,18));check('old ten-core central Floor square meaningfully reduced',oldsq['width_core']==10 and newsq['width_core']<=4)
 return {'status':'PASS'if all(c['passed']for c in checks)else'FAIL','checks':checks,'terrain_changes':delta,'shared_geometry':common,'lake_contract':lake,'grass_prices':prices,'grass_choice':{'start':start,'target':target,'dry_steps':len(dry)-1 if dry else None,'direct_steps':len(direct)-1 if direct else None,'direct_grass':direct_grass,'dry_path':dry,'direct_path':direct},'lookout_native_view_water_core_cells':water,'lookout_water_gaps':water_gaps,'central_clear_square':{'before':oldsq,'candidate':newsq},'warm_source_check_ms':round((time.perf_counter()-started)*1000,3)}

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True);parser.add_argument('--self-test',action='store_true');args=parser.parse_args();out=args.output;out.mkdir(parents=True,exist_ok=False);packpath=ROOT/'content-packs/minnesota-northstar-lake-country.crystalpack';assert sha(packpath.read_bytes())==PACK_SHA;p=load_pack(packpath);oldpaths=[BEFORE/n for n in ['MN_R18.shorewalk.candidate.json','MN_R18.shorewalk.layout.json']];assert [sha(v.read_bytes())for v in oldpaths]==['21a5bd33acb1760380b1a94494728965f8760d267ee4f969f9bee7e795842c4c','2b5b4a542513691e07dc35845c16af4c6f6a20bad879f3156ed76c9a29290618'];od,ol=[json.loads(v.read_bytes())for v in oldpaths];sources=[ROOT/'planning/minnesota/routes'/n for n in ['MN_R18.ribbon.candidate.json','MN_R18.ribbon.layout.json']];d,l=[json.loads(v.read_bytes())for v in sources];r=inspect(d,l,p,od,ol)
 for f in sources+[Path(__file__)]: (out/f.name).write_bytes(f.read_bytes())
 started=time.perf_counter();before=render(od,ol,p,out,'before');candidate=render(d,l,p,out,'candidate');pair=Image.new('RGB',(1280,1584),(19,29,40));draw=ImageDraw.Draw(pair);draw.text((12,6),'BEFORE approved shorewalk v04 source composite',fill='white');pair.paste(before,(0,24));draw.text((12,798),'CANDIDATE inhabited dune ribbon / dry lakeview bend / short grass choice | source only',fill='white');pair.paste(candidate,(0,816));save_render(pair,out/'before-after-whole-native32.png');r['complete_positive_render_ms']=round((time.perf_counter()-started)*1000,3)
 if args.self_test:
  negatives=[]
  def reject(name,mutate):
   nd,nl=deepcopy(d),deepcopy(l);mutate(nd,nl);rr=inspect(nd,nl,p,od,ol);failed=[v for v in rr['checks']if not v['passed']];assert failed,name+' accepted';folder=out/'rejected-inputs'/name;folder.mkdir(parents=True);write(folder/'dossier.json',nd);write(folder/'layout.json',nl);write(folder/'reasons.json',failed);negatives.append({'id':name,'rejected':True,'failed_checks':[v['check']for v in failed]})
  point=lambda at,b:{'kind':'point','name':'Adversarial native source','position':at,'block':b}
  reject('dry-shortcut-free-of-grass',lambda nd,nl:nl['layers'].append(point([23,6],'01')))
  reject('erase-winding-dry-road',lambda nd,nl:nl['layers'].append(point([22,7],'03')))
  reject('open-lakeview-rail-to-water',lambda nd,nl:nl['layers'].append(point([22,8],'01')))
  reject('restore-empty-central-apron',lambda nd,nl:nl['layers'].append({'kind':'block_rect','name':'Plain lawn','bounds':[21,3,27,8],'block':'02'}))
  reject('damage-approved-shorewalk',lambda nd,nl:nl['layers'].append(point([32,9],'71')))
  reject('erase-original-six-grass-neck',lambda nd,nl:nl['layers'].append({'kind':'block_rect','name':'Dry original habitat','bounds':[18,5,21,9],'block':'02'}))
  reject('incidental-cut-dune',lambda nd,nl:nl['layers'].append(point([23,4],'5b')))
  reject('block-public-launch-footing',lambda nd,nl:nl['layers'].append(point([24,9],'49')))
  reject('alter-quest-completion',lambda nd,nl:nd['quest_contract'].update(reward_flag='EVENT_MN_SHORE_WATCH_02'))
  reject('move-native-dory',lambda nd,nl:nd['trainers'][2].update(tile=[53,10]))
  r['negative_mutations']=negatives
 r.update(candidate_source_sha256={v.name:sha(v.read_bytes())for v in sources},before_source_sha256={v.name:sha(v.read_bytes())for v in oldpaths},reference_pack_sha256=PACK_SHA,scope='Separate source-only composition after approved shorewalk; no native pack/controller/GPU/promotion/whole-route acceptance claim.');write(out/'review.json',r);print(json.dumps({'status':r['status'],'checks':len(r['checks']),'negatives':len(r.get('negative_mutations',[])),'changes':len(r['terrain_changes']),'failed':[v for v in r['checks']if not v['passed']],'choice':{k:v for k,v in r['grass_choice'].items()if not k.endswith('_path')},'central_clear_square':r['central_clear_square'],'water_visible':r['lookout_native_view_water_core_cells'],'shared_errors':r['shared_geometry']['errors'],'lake_error':r['lake_contract'].get('error')},indent=2))
 if r['status']!='PASS':raise SystemExit(1)
if __name__=='__main__':main()
