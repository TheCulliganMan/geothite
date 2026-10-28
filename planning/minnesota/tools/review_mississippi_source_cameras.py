#!/usr/bin/env python3
"""Review full authored river layouts, six source cameras and exact footprints.

The standing Chris sprite is a source-view marker, not a native Surf/GPU
capture. This tool reads the declared parent pack and never publishes content.
"""
import argparse
import sys,json,hashlib,collections,copy
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from render_reference import load_pack,layout_blocks,SourceArt
from render_archive import save_render
from render_progress_atlas import font
from review_north_shore import oriented_sprite
from PIL import Image,ImageDraw
ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--manifest',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
mfile=args.manifest;manifest=json.loads(mfile.read_text());assert hashlib.sha256(Path(manifest['base_pack']).read_bytes()).hexdigest()==manifest['base_pack_sha256'];pack=load_pack(manifest['base_pack']);maps=pack['data']['maps'];out=args.output;out.mkdir(parents=True,exist_ok=True);art=SourceArt(pack)
by_layout={}
for row in manifest['new_maps']:
 assert hashlib.sha256(Path(row['layout']).read_bytes()).hexdigest()==row['layout_sha256']
 by_layout.setdefault(row['layout'],[]).append(row)
reviews=[]
for filename,leaves in by_layout.items():
 l=json.load(open(filename));w,h=l['dimensions_metatiles'];blocks=layout_blocks(l);im=art.render('johto',w,h,blocks).convert('RGBA')
 for a in manifest['objects']:
  leaf=next((r for r in leaves if r['id']==a['map']),None)
  if leaf:
   x,y=a['core'];ox,oy=leaf['source_origin_metatiles'];im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(x*16+ox*32,y*16+oy*32))
 save_render(im,out/(l['id']+'-whole-rgba.png'));rgb=im.convert('RGB');save_render(rgb,out/(l['id']+'-whole.png'))
 if 'LowerWoods' in l['id']:wanted=[(24,64),(82,21),(108,84)]
 elif 'UpstreamWoods' in l['id']:wanted=[(34,131),(110,108),(68,25)]
 else:continue
 c=pack['data']['tilesets']['johto']['collision'];water=[]
 for y in range(h*2):
  for x in range(w*2):
   if c[f'{blocks[(y//2)*w+x//2]:02x}'][(y%2)*2+x%2]=='WATER':water.append((x,y))
 board=Image.new('RGB',(3*640,616),'#121f2b');d=ImageDraw.Draw(board)
 for i,p in enumerate(wanted):
  x,y=min(water,key=lambda q:abs(q[0]-p[0])+abs(q[1]-p[1]));view=im.crop((x*16-72,y*16-64,x*16+88,y*16+80));view.alpha_composite(oriented_sprite(pack,'SPRITE_CHRIS','north'),(72,64));view=view.convert('RGB');dest=out/'player-views'/l['id']/f'view-{i+1:02}-{x}-{y}';save_render(view,dest.with_suffix('.source160.png'));large=view.resize((640,576),Image.Resampling.NEAREST);save_render(large,dest.with_suffix('.source4x.png'));board.paste(large,(i*640,40));d.text((i*640+12,10),f'{l["id"]} · core{x},{y}',font=font(16,True),fill='white');reviews.append({'layout':l['id'],'core':[x,y],'file':str(dest.with_suffix('.source160.png'))})
 save_render(board,out/(l['id']+'-three-player-cameras.png'))
# Every existing direct-seam component is enumerated. Only the component with
# an actual common native origin can have a global footprint comparison.
old_out={n:v for n,v in maps.items()if n.startswith('Mn')and(v['attributes']['environment']or'').lower()in{'town','route','outdoor'}}
def embed(modules,root):
 pos={root:(0,0)};q=collections.deque([root]);conflicts=[]
 while q:
  n=q.popleft();x,y=pos[n];a=modules[n]['attributes']
  for e in a['connections']:
   t=e['target_map']
   if t not in modules:continue
   b=modules[t]['attributes'];o=e['offset'];v={'north':(x+o,y-b['height']),'south':(x+o,y+a['height']),'west':(x-b['width'],y+o),'east':(x+a['width'],y+o)}[e['direction']]
   if t not in pos:pos[t]=v;q.append(t)
   elif pos[t]!=v:conflicts.append([n,t,list(pos[t]),list(v)])
 return pos,conflicts
before,_=embed(old_out,'MnMinneapolis');candidate=copy.deepcopy(old_out)
for row in manifest['new_maps']:
 w,h=row['dimensions_metatiles'];candidate[row['id']]={'attributes':{'width':w,'height':h,'connections':row['connections']}}
for row in manifest['existing_connection_additions']:candidate[row['map']]['attributes']['connections'].append({k:row[k]for k in ['direction','target_map','offset']})
pos,conflicts=embed(candidate,'MnMinneapolis');assert not conflicts,conflicts
for row in manifest['new_maps']:assert list(pos[row['id']])==row['world_origin_metatiles'],row['id']
newset={r['id']for r in manifest['new_maps']};overlaps=[];names=list(pos)
for i,n in enumerate(names):
 x,y=pos[n];a=candidate[n]['attributes']
 for t in names[i+1:]:
  u,v=pos[t];b=candidate[t]['attributes'];r=[max(x,u),max(y,v),min(x+a['width'],u+b['width']),min(y+a['height'],v+b['height'])]
  if r[0]<r[2]and r[1]<r[3]:overlaps.append({'maps':[n,t],'rectangle_metatiles':r,'new_map_involved':n in newset or t in newset})
assert not any(r['new_map_involved']for r in overlaps),overlaps
proof={'all_native_outdoors_enumerated':len(old_out),'native_with_shared_direct_seam_origin':len(before),'native_other_components_without_global_origin':sorted(set(old_out)-set(before)),'new_maps_checked':len(newset),'new_overlaps':[],'inherited_overlaps_in_main_component':overlaps,'connection_cycle_conflicts':conflicts,'interpretation':f'{len(old_out)} native outdoors enumerated; {len(before)} have an actual shared exterior origin. Other {len(old_out)-len(before)} are separate direct-seam components, so no physical relative origin is fabricated. Gatehouse linkage does not alone define an exterior metatile displacement.'}
(out/'footprint-proof.json').write_text(json.dumps(proof,indent=2)+'\n');(out/'player-camera-index.json').write_text(json.dumps(reviews,indent=2)+'\n')
print(json.dumps({'manifest_sha256':hashlib.sha256(mfile.read_bytes()).hexdigest(),'new_overlap_count':0,'player_cameras':len(reviews),'new_maps':len(newset)}))
