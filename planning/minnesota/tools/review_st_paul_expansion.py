#!/usr/bin/env python3
"""Read-only source-art and physical review; no native playability claims."""
import argparse,collections,copy,hashlib,json,sys
from pathlib import Path
from PIL import Image,ImageDraw
from render_reference import load_pack,SourceArt,layout_blocks,room_sheet
from render_archive import save_render
from render_stages import render_stage
from review_north_shore import oriented_sprite
ROOT=Path(__file__).resolve().parents[3];B=ROOT/'planning/minnesota';T=B/'towns'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--revision',default='v01');args=ap.parse_args();out=B/'renders/st-paul/expanded-20261027'/args.revision;out.mkdir(parents=True,exist_ok=True)
 p=load_pack(ROOT/'content-packs/minnesota-northstar-seep-shelter-review.crystalpack');a=SourceArt(p);l=json.loads((T/'MnStPaul.expanded.layout.json').read_text());d=json.loads((T/'MnStPaul.expanded.candidate.json').read_text());ct=json.loads((T/'MnStPaul.expansion.contract.json').read_text());plan=json.loads((T/'MnStPaul.expansion-plan.json').read_text());bl=layout_blocks(l);w,h=l['dimensions_metatiles'];co=p['data']['tilesets']['kanto']['collision']
 def kind(at):x,y=at;return co[f'{bl[y//2*w+x//2]:02x}'][y%2*2+x%2]
 actors=[dict(id=v['id'],tile=v['new']['global_core'],sprite=p['data']['maps']['MnStPaul']['objects'][v['native_object_index_zero_based']]['sprite'],facing='south') for v in plan['native_objects']]+ct['residents']+ct['trainers']+[dict(v,sprite='SPRITE_POKE_BALL') for v in ct['items']]
 # Native stored sprite identifiers differ by runtime adapter; source field only.
 for ac in actors:
  if isinstance(ac['sprite'],dict):ac['sprite']=ac['sprite'].get('name','SPRITE_POKEFAN_M')
 occupied={tuple(v['tile']) for v in actors};walk={(x,y) for y in range(h*2) for x in range(w*2) if kind((x,y)) in {'FLOOR','TALL_GRASS','LADDER'} and (x,y) not in occupied}
 def flood(start,allowed):
  seen={tuple(start)};q=collections.deque(seen)
  while q:
   x,y=q.popleft()
   for v in [(x+1,y),(x-1,y),(x,y+1),(x,y-1)]:
    if v in allowed and v not in seen:seen.add(v);q.append(v)
  return seen
 seen=flood([6,72],walk);errors=[]
 doors=[dict(tile=v['new_active']['global_core'],return_tile=v['new_passive']['global_core'],target=v['id']) for v in plan['doors']]+ct['doors']
 for door in doors:
  if kind(door['tile'])!='DOOR':errors.append(['door_collision',door['target'],door['tile'],kind(door['tile'])])
  if tuple(door['return_tile']) not in seen:errors.append(['door_return',door['target'],door['return_tile'],kind(door['return_tile'])])
 for ac in actors:
  x,y=ac['tile'];adj=[(x+1,y),(x-1,y),(x,y+1),(x,y-1)]
  if kind((x,y))!='FLOOR' or not any(v in seen for v in adj):errors.append(['actor',ac['id'],[x,y],kind((x,y))])
 for sign in plan['native_bg_records']:
  x,y=sign['new']['global_core']
  if kind((x,y))!='WALL' or not any(v in seen for v in [(x+1,y),(x-1,y),(x,y+1),(x,y-1)]):errors.append(['sign',sign['id'],[x,y],kind((x,y))])
 allactualdoors={(x,y) for y in range(h*2) for x in range(w*2) if kind((x,y))=='DOOR'};expected={tuple(v['tile']) for v in doors}
 if allactualdoors!=expected:errors.append(['door_roster',sorted(allactualdoors-expected),sorted(expected-allactualdoors)])
 cut=[(x,y) for y in range(h*2) for x in range(w*2) if kind((x,y))=='CUT_TREE']
 if cut:errors.append(['incidental_cut',cut])
 outer=[(x,y) for y in range(h*2) for x in range(w*2) if (x in {0,w*2-1} or y in {0,h*2-1}) and (x,y) in seen]
 if set(outer)!={(0,72),(0,73)}:errors.append(['outer_walk',outer])
 surfedge=[(x,y) for y in range(h*2) for x in range(w*2) if (x in {0,w*2-1} or y in {0,h*2-1}) and kind((x,y)).startswith('WATER')]
 if surfedge:errors.append(['surfedge',surfedge])
 # Catalogue/rooms/native art and physically paired warps.
 roomimgs=[];roomstats=[]
 for r in ct['rooms']:
  rw,rh=r['dimensions_metatiles'];rb=[int(b,16) for row in r['rows'] for b in row.split()];rc=p['data']['tilesets'][r['tileset']]['collision']
  def rk(at):x,y=at;return rc[f'{rb[y//2*rw+x//2]:02x}'][y%2*2+x%2]
  bodies={tuple(v['tile']) for key in ['residents','trainers','items'] for v in r.get(key,[])}
  allowed={(x,y) for y in range(rh*2) for x in range(rw*2) if rk((x,y)) in {'FLOOR','STAIRCASE','WARP_CARPET_DOWN','LADDER'} and (x,y) not in bodies};reach=flood(r['arrival_core'],allowed)
  if rk(r['arrival_core'])!='FLOOR':errors.append(['room_arrival',r['id'],r['arrival_core'],rk(r['arrival_core'])])
  for key in ['residents','trainers','items']:
   for v in r.get(key,[]):
    x,y=v['tile'];adj=[(x+1,y),(x-1,y),(x,y+1),(x,y-1)]
    if rk((x,y))!='FLOOR' or not (tuple(v.get('approach',[-1,-1])) in reach or any(q in reach for q in adj)):errors.append(['room_actor',r['id'],v['id'],v['tile'],rk((x,y))])
    for item in v.get('stock',[]):
     if item not in p['data']['items']:errors.append(['unknown_stock',item])
    for member in v.get('party',[]):
     if member['species'] not in p['data']['pokemon']:errors.append(['unknown_species',member['species']])
  for warp in r['warps']:
   k=rk(warp['tile'])
   if warp['passive'] and k!='FLOOR':errors.append(['passive_warp',r['id'],warp,k])
   if not warp['passive'] and k not in {'WARP_CARPET_DOWN','STAIRCASE','LADDER','WARP_CARPET_UP'}:errors.append(['active_warp',r['id'],warp,k])
  im=a.render(r['tileset'],rw,rh,rb);rgba=Image.new('RGBA',im.size)
  for v in r['residents']+r['trainers']+[dict(v,sprite='SPRITE_POKE_BALL',facing='south') for v in r['items']]:
   try:sp=oriented_sprite(p,v['sprite'],v.get('facing','south'));rgba.alpha_composite(sp,(v['tile'][0]*16,v['tile'][1]*16))
   except (KeyError,AssertionError) as e:errors.append(['sprite',v['sprite'],str(e)])
  im.paste(rgba,(0,0),rgba);save_render(im,out/'rooms'/(r['id']+'-whole-original.png'));roomimgs.append((r['id'],im));roomstats.append(dict(id=r['id'],ordinary_reached=len(reach)))
 # Largest native-Floor through-space; do not hide it behind chosen landmarks.
 dp={};best=[]
 for y in range(h*2):
  for x in range(w*2):
   if (x,y) not in seen or kind((x,y))!='FLOOR':continue
   n=1+min(dp.get((x-1,y),0),dp.get((x,y-1),0),dp.get((x-1,y-1),0));dp[(x,y)]=n
   if n>=8:best.append((n,[x-n//2,y-n//2]))
 best.sort(reverse=True);weak=[]
 for n,at in best:
  if all(abs(at[0]-q[1][0])+abs(at[1]-q[1][1])>18 for q in weak):weak.append((n,at))
  if len(weak)==5:break
 im=a.render('kanto',w,h,bl);save_render(im,out/'whole-terrain-original.png');rgba=Image.new('RGBA',im.size)
 for ac in actors:
  try:sp=oriented_sprite(p,ac['sprite'],ac.get('facing','south'));rgba.alpha_composite(sp,(ac['tile'][0]*16,ac['tile'][1]*16))
  except (KeyError,AssertionError) as e:errors.append(['sprite',ac['sprite'],str(e)])
 save_render(rgba,out/'actors-source-rgba.png');im.paste(rgba,(0,0),rgba);save_render(im,out/'whole-city-original.png');save_render(im.resize((1536,1152),Image.Resampling.NEAREST),out/'whole-city-preview.png')
 cams=[('R04 true west arrival',[6,72]),('Fair midway',[34,27]),('Fair north annex',[49,5]),('Exhibition lane',[28,45]),('Capitol hall',[98,30]),('Original bluff hop',[103,39]),('Summit club',[173,31]),('West Seventh clinic',[39,58]),('Market three-floor exchange',[98,72]),('East gym',[150,60]),('Depot actual story entrance',[140,106]),('High Bridge',[59,99]),('Robert Street bridge',[131,119]),('West side neighborhood',[69,127]),('South landing',[143,137])]
 sheet=Image.new('RGB',(5*340,3*340),(20,26,35));dr=ImageDraw.Draw(sheet)
 for i,(label,at) in enumerate(cams):
  x,y=at;box=(x*16-72,y*16-64,x*16+88,y*16+80);crop=im.crop(box);save_render(crop,out/'cameras'/(f'{i:02d}-native160-original.png'));big=crop.resize((320,288),Image.Resampling.NEAREST);sheet.paste(big,(i%5*340,i//5*340+28));dr.text((i%5*340+4,i//5*340+6),label,fill='white')
 save_render(sheet,out/'decision-cameras.png')
 weakim=Image.new('RGB',(340*len(weak),340),(20,26,35));wd=ImageDraw.Draw(weakim)
 for i,(n,(x,y)) in enumerate(weak):
  crop=im.crop((x*16-72,y*16-64,x*16+88,y*16+80));save_render(crop,out/'cameras'/(f'weak-{i:02d}-native160-original.png'));weakim.paste(crop.resize((320,288),Image.Resampling.NEAREST),(i*340,28));wd.text((i*340+4,6),f'{n}core Floor square near {x},{y}',fill='white')
 save_render(weakim,out/'weakest-cameras.png');save_render(room_sheet(roomimgs,5),out/'all-new-interiors.png')
 report=dict(source_only=True,native_gameplay_proved=False,weakest_floor_squares=weak,errors=errors,doors=len(doors),new_rooms=len(ct['rooms']),ordinary_reached=len(seen),room_stats=roomstats,source_hashes={f.name:sha(f) for f in [T/'MnStPaul.expanded.layout.json',T/'MnStPaul.expansion.contract.json',T/'MnStPaul.expanded.candidate.json']})
 (out/'review.json').write_text(json.dumps(report,indent=2)+'\n')
 for f in report['source_hashes']:(out/f).write_bytes((T/f).read_bytes())
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()
