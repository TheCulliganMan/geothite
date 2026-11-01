#!/usr/bin/env python3
"""Read-only installed exterior/port inventory; no terrain or pack generation."""
import argparse,collections,hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack
from audit_outdoor_connections import collision,boundary,project,OPPOSITE,neighbor_origin

def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--pack',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
 pack=load_pack(a.pack);maps=pack['data']['maps'];outdoors={n:m for n,m in maps.items() if n.startswith('Mn') and str(m['attributes']['environment']).lower() in ('town','route','outdoor')}
 kinds={n:collision(pack,m) for n,m in outdoors.items()};const={m['attributes']['map_constant']:n for n,m in maps.items()}
 wpath=ROOT/'planning/minnesota/world.json';ipath=ROOT/'planning/minnesota/interiors.json';w=json.loads(wpath.read_text());interiors=json.loads(ipath.read_text());riverpath=ROOT/'planning/minnesota/connections/mississippi-v02/manifest.json';river=json.loads(riverpath.read_text())
 groups=[];position={};overlaps=[];cycles=[]
 for root in ['MnMinneapolis']+sorted(outdoors):
  if root in position:continue
  position[root]=(0,0);queue=collections.deque([root]);group=[]
  while queue:
   n=queue.popleft();group.append(n)
   for e in outdoors[n]['attributes']['connections']:
    t=e['target_map']
    if t not in outdoors:continue
    at=neighbor_origin(position[n],outdoors[n]['attributes'],outdoors[t]['attributes'],e)
    if t in position:
     if position[t]!=at:cycles.append({'from':n,'to':t,'existing':position[t],'implied':at})
    else:position[t]=at;queue.append(t)
  groups.append(group)
 for gi,g in enumerate(groups):
  for i,n in enumerate(g):
   x,y=position[n];na=outdoors[n]['attributes']
   for t in g[i+1:]:
    u,v=position[t];ta=outdoors[t]['attributes'];r=[max(x,u),max(y,v),min(x+na['width'],u+ta['width']),min(y+na['height'],v+ta['height'])]
    if r[0]<r[2] and r[1]<r[3]:overlaps.append({'group':gi,'maps':[n,t],'rectangle_metatiles':r})
 inventory={}
 for n,m in outdoors.items():
  at=m['attributes'];edges={}
  for side in OPPOSITE:
   cells=boundary(at,side);e=next((e for e in at['connections'] if e['direction']==side),None)
   edges[side]={'connection':e,'floor_core':[list(q) for q in cells if kinds[n][q]=='FLOOR'],'water_core':[list(q) for q in cells if kinds[n][q]in('WATER','WATERFALL')],'other_core':[{'tile':q,'collision':kinds[n][q]} for q in cells if kinds[n][q] not in ('FLOOR','WATER','WATERFALL','WALL','HEADBUTT_TREE','HEADBUTT_TREE_1D','BUOY')]}
  warps=[]
  for warp in m['events']['warps']:
   target=const.get(warp['target_map'],warp['target_map']);index=warp['target_warp_id'];targetwarp=next((v for v in maps.get(target,{}).get('events',{}).get('warps',[]) if v['index']==index),None)
   warps.append({'index':warp['index'],'core':[warp['x'],warp['y']],'collision':kinds[n].get((warp['x'],warp['y'])),'target':target,'target_warp':index,'target_core':[targetwarp['x'],targetwarp['y']]if targetwarp else None})
  inventory[n]={'dimensions_metatiles':[at['width'],at['height']],'map_constant':at['map_constant'],'metadata':m.get('metadata'),'origin_in_component_metatiles':position[n],'component':next(i for i,g in enumerate(groups)if n in g),'edges':edges,'warps':warps,'native_module_sha256':hashlib.sha256(json.dumps(m,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()}
 routedata={}
 for r in w['routes']:
  f=ROOT/'planning/minnesota'/r['file'];d=json.loads(f.read_text());routedata[r['id']]={'file':str(f.relative_to(ROOT)),'sha256':sha(f),'dimensions_metatiles':d['dimensions_metatiles'],'endpoints':d.get('endpoints'),'entrances':d.get('entrances'),'required_flags':d.get('required_flags'),'interiors':[{'id':i['id'],'dimensions_metatiles':i.get('dimensions_metatiles'),'entrances':i.get('entrances'),'exits':i.get('exits')} for i in d.get('interiors',[])]}
 towns=[{'id':t['id'],'installed':t['id']in outdoors,'planned_dimensions_metatiles':t['dimensions_metatiles'],'actual_dimensions_metatiles':inventory.get(t['id'],{}).get('dimensions_metatiles'),'ports':t['ports'],'source_file':t.get('authoring_file',t.get('source_file')),'layout_file':t.get('layout_file')}for t in w['towns']]
 crosswarps=[]
 for n,m in maps.items():
  for warp in m['events']['warps']:
   t=const.get(warp['target_map'],warp['target_map'])
   if n!=t and(n in outdoors or t in outdoors):crosswarps.append({'from':n,'index':warp['index'],'core':[warp['x'],warp['y']],'to':t,'target_warp':warp['target_warp_id']})
 report={'status':'READ_ONLY_STATIC_NATIVE_INVENTORY_NOT_CONTROLLER_PROOF','pack':str(a.pack),'pack_sha256':sha(a.pack),'map_count':len(maps),'outdoor_count':len(outdoors),'source_hashes':{'world.json':sha(wpath),'interiors.json':sha(ipath),'river_manifest':sha(riverpath)},'components':groups,'inconsistent_cycles':cycles,'overlaps':overlaps,'outdoors':inventory,'towns':towns,'routes':routedata,'world_connections':w['connections'],'native_cross_map_warps_touching_outdoors':crosswarps,'river_candidate':{'manifest':str(riverpath.relative_to(ROOT)),'status':river['status'],'map_count':len(river['new_maps']),'connections':river['existing_connection_additions'],'new_maps':[{'id':m['id'],'dimensions_metatiles':m['dimensions_metatiles'],'world_origin_metatiles':m['world_origin_metatiles'],'connections':m['connections']}for m in river['new_maps']]}}
 (a.output/'native-state-inventory.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'pack_sha256':report['pack_sha256'],'maps':len(maps),'outdoors':len(outdoors),'components':len(groups),'overlaps':len(overlaps),'missing_towns':[t['id']for t in towns if not t['installed']]}))
if __name__=='__main__':main()
