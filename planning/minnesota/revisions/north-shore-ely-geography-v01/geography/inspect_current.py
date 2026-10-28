#!/usr/bin/env python3
"""Read-only current coast/inland native geography and exact port inventory."""
import argparse,hashlib,json,sys
from pathlib import Path
from collections import deque
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[5]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack
from render_progress_atlas import native_room
from audit_outdoor_connections import collision
SHA='3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243'
p=argparse.ArgumentParser();p.add_argument('--pack',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();assert not a.output.exists();assert hashlib.sha256(a.pack.read_bytes()).hexdigest()==SHA;a.output.mkdir(parents=True)
pack=load_pack(a.pack);maps=pack['data']['maps'];atlas=json.loads((ROOT/'planning/minnesota/renders/river-continuity-long-v01/whole-state-native-v01/native-connection-audit.json').read_bytes());by={r['map']:r for r in atlas['placements']};names=[n for n in by if n.startswith(('MnR11','MnR12'))or n in{'MnGrandMarais','MnMaraisCedarApproach','MnDuluth','MnBoundaryWaters','MnPreservePineEntry'}];report=[]
for n in sorted(names):
 m=maps[n];at=m['attributes'];k=collision(pack,m);w,h=at['width']*2,at['height']*2
 borders={d:[dict(core=[x,y],permission=k[x,y])for x,y in pts]for d,pts in [('north',[(x,0)for x in range(w)]),('south',[(x,h-1)for x in range(w)]),('west',[(0,y)for y in range(h)]),('east',[(w-1,y)for y in range(h)])]}
 report.append(dict(map=n,origin_metatiles=by[n]['origin_metatiles'],dimensions_metatiles=[at['width'],at['height']],tileset=at['tileset_name'],palette_group=at.get('time_of_day'),connections=at['connections'],borders=borders,warps=[dict(index=e['index'],core=[e['x'],e['y']],target=e['target_map'])for e in m['events']['warps']]))
summary=dict(actual_pack_sha256=SHA,maps=report,ely_maps=[n for n in maps if 'Ely'in n],map_count=len(maps));(a.output/'actual-geography.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(dict(maps=[dict(map=r['map'],origin=r['origin_metatiles'],size=r['dimensions_metatiles'],tileset=r['tileset'],connections=r['connections'])for r in report],ely_maps=summary['ely_maps'])),flush=True)
cache={n:native_room(maps[n],pack,native_origin=True).convert('RGB')for n in names}
for prefix,width,height in [('MnR11',56,96),('MnR12',64,56)]:
 origin=[min(by[n]['origin_metatiles'][i]for n in names if n.startswith(prefix))for i in range(2)];image=Image.new('RGB',(width*32,height*32),'#121f2b')
 for n in names:
  if n.startswith(prefix):x,y=by[n]['origin_metatiles'];image.paste(cache[n],((x-origin[0])*32,(y-origin[1])*32))
 image.save(a.output/f'{prefix}-actual-whole-native.png')
for n in ['MnGrandMarais','MnMaraisCedarApproach']:cache[n].save(a.output/f'{n}-actual-whole-native.png')
x0=min(by[n]['origin_metatiles'][0]for n in names);y0=min(by[n]['origin_metatiles'][1]for n in names);x1=max(by[n]['origin_metatiles'][0]+by[n]['dimensions_metatiles'][0]for n in names);y1=max(by[n]['origin_metatiles'][1]+by[n]['dimensions_metatiles'][1]for n in names);scale=10;image=Image.new('RGB',((x1-x0)*scale,(y1-y0)*scale+72),'#121f2b');d=ImageDraw.Draw(image);d.text((10,10),'ACTUAL3ddbad16 / NORTH SHORE, GRAND MARAIS AND INLAND ROUTES',fill='white');d.text((10,32),'Native source art at actual positions. Ely absent. No proposed terrain or filled atlas gaps.',fill='#e6c289')
for n in names:
 x,y=by[n]['origin_metatiles'];w,h=by[n]['dimensions_metatiles'];image.paste(cache[n].resize((w*scale,h*scale),Image.Resampling.NEAREST),((x-x0)*scale,(y-y0)*scale+72))
image.save(a.output/'current-coast-inland-composition.png')
