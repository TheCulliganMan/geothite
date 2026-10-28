#!/usr/bin/env python3
"""Check frozen river and detached cohesion offset equations without pack writes."""
import argparse,copy,hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from audit_outdoor_connections import neighbor_origin,OPPOSITE
from render_archive import save_render
from render_progress_atlas import font
from PIL import Image,ImageDraw
HERE=Path(__file__).resolve().parent

def main():
 p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
 inv=json.loads((ROOT/'planning/minnesota/renders/outdoor-connections/state-cohesion/v01/native-state-inventory.json').read_text());co=json.loads((HERE/'source-v01/integration.contract.json').read_text());mp=ROOT/'planning/minnesota/connections/mississippi-v02/manifest.json';mb=mp.read_bytes();assert hashlib.sha256(mb).hexdigest()=='449ce63508b11513b187e5741ac8b8dc669d9fe420b1c922907dd301d4e2b814';river=json.loads(mb);pos={n:tuple(v)for n,v in co['proposed_origins_metatiles'].items()};dims={n:dict(width=m['dimensions_metatiles'][0],height=m['dimensions_metatiles'][1])for n,m in inv['outdoors'].items()};dims['MnR03C00R01']['width']=21
 for m in co['new_maps']:
  d=json.loads((HERE/'source-v01'/(m['id']+'.layout.json')).read_text());w,h=d['dimensions_metatiles'];dims[m['id']]=dict(width=w,height=h)
 for m in river['new_maps']:
  w,h=m['dimensions_metatiles'];dims[m['id']]=dict(width=w,height=h);pos[m['id']]=tuple(m['world_origin_metatiles'])
 checks=[]
 def check(id,ok,**details):checks.append(dict(id=id,pass_=bool(ok),**details))
 for m in river['new_maps']:
  n=m['id']
  for edge in m['connections']:
   t=edge['target_map'];implied=neighbor_origin(pos[n],dims[n],dims[t],edge);check(n+'-'+edge['direction']+'-exact-position',implied==pos[t],implied=implied,actual=pos[t])
   if t.startswith('MnMississippi'):
    rev=next((e for other in river['new_maps']if other['id']==t for e in other['connections']if e['direction']==OPPOSITE[edge['direction']]and e['target_map']==n),None)
   else:rev=next((e for e in river['existing_connection_additions']if e['map']==t and e['direction']==OPPOSITE[edge['direction']]and e['target_map']==n),None)
   check(n+'-'+edge['direction']+'-reciprocal',rev is not None and rev['offset']==-edge['offset'])
 for edge in river['existing_connection_additions']:
  n=edge['map'];t=edge['target_map'];check(n+'-'+edge['direction']+'-existing-map-position',neighbor_origin(pos[n],dims[n],dims[t],edge)==pos[t])
 overlaps=[]
 for i,n in enumerate(pos):
  x,y=pos[n]
  for t in list(pos)[i+1:]:
   u,v=pos[t];r=[max(x,u),max(y,v),min(x+dims[n]['width'],u+dims[t]['width']),min(y+dims[n]['height'],v+dims[t]['height'])]
   if r[0]<r[2]and r[1]<r[3]:overlaps.append(dict(maps=[n,t],rectangle_metatiles=r))
 check('all98-source-footprints-no-overlaps',not overlaps,total=len(pos),overlaps=overlaps);check('seventeen-river-maps-exact',len(river['new_maps'])==17);check('all98-proposed-maps-accounted',len(pos)==98)
 # A true one-scale footprint plan; rectangles never masquerade as river terrain.
 scale=6;mnx=min(x for x,y in pos.values());mny=min(y for x,y in pos.values());mxx=max(pos[n][0]+d['width']for n,d in dims.items());mxy=max(pos[n][1]+d['height']for n,d in dims.items());im=Image.new('RGB',((mxx-mnx)*scale+80,(mxy-mny)*scale+120),'#17232c');dr=ImageDraw.Draw(im)
 for n,(x,y)in pos.items():
  xx=40+(x-mnx)*scale;yy=90+(y-mny)*scale;rr=(xx,yy,xx+dims[n]['width']*scale-1,yy+dims[n]['height']*scale-1);dr.rectangle(rr,fill='#244962'if n.startswith('MnMississippi')else'#315440',outline='#56b5df'if n.startswith('MnMississippi')else'#72a77b');dr.text((xx+2,yy+2),n,font=font(8),fill='white')
 dr.text((18,10),'98-MAP PHYSICAL FOOTPRINT PROPOSAL — NO NEW NATIVE CLAIM',font=font(19,True),fill='white');dr.text((18,40),'Blue:17 frozen Mississippi source windows. Green:77 existing+4 detached pine connectors.',font=font(13),fill='white');dr.text((18,60),'Exact native offset equations; footprints are not substitute terrain. Preserve admission stays closed.',font=font(13),fill='#e9bf6a');save_render(im,a.output/'all98-footprints-proposal.png')
 report=dict(status='STATIC_SOURCE_FOOTPRINT_ONLY_NOT_NATIVE_OR_CONTROLLER_PROOF',parent_pack_sha256=inv['pack_sha256'],river_manifest_sha256=hashlib.sha256(mb).hexdigest(),cohesion_contract_sha256=hashlib.sha256((HERE/'source-v01/integration.contract.json').read_bytes()).hexdigest(),map_count=98,origins_metatiles=pos,dimensions_metatiles={n:[d['width'],d['height']]for n,d in dims.items()},checks=checks,failures=[c for c in checks if not c['pass_']]);(a.output/'river-cohesion-embedding-proof.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(checks=len(checks),failures=report['failures'],output=str(a.output))))
if __name__=='__main__':main()
