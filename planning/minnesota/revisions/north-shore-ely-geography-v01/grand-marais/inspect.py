"""Inspect actual Grand Marais lineages without modifying native content."""
from pathlib import Path
import sys,json,hashlib,cbor2,gc
ROOT=Path(__file__).resolve().parents[5];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt,room_sheet
from review_major_city import ReviewArt,render_map,player_camera
from render_archive import save_render
sha=lambda b:hashlib.sha256(b).hexdigest()
canonical=lambda v:json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
out=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/grand-marais/inspection-v01';out.mkdir(parents=True,exist_ok=False)
records=[];wholes=[]
for filename in ['minnesota-river-long-v02.crystalpack','minnesota-northstar-north-shore-story.crystalpack','minnesota-northstar-grand-marais.crystalpack']:
 path=ROOT/'content-packs'/filename;raw=path.read_bytes();pack=cbor2.loads(raw[22:]);del raw
 m=pack['data']['maps']['MnGrandMarais'];a=m['attributes'];ts=a['tileset_name'];name=filename.removesuffix('.crystalpack')
 actors=[dict(id=o.get('object_identifier')or o['script'],sprite=o['sprite'],tile=[o['x'],o['y']],facing={'UP':'north','DOWN':'south','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south'),native_palette_override=o.get('pal',0))for o in m['objects']]
 art=ReviewArt(pack);ground,scene,_=render_map(art,m,actors,'day',out/name);wholes.append((name,scene))
 c=pack['data']['tilesets'][ts]['collision'];kind=lambda x,y:c[f"{m['blocks'][y//2*a['width']+x//2]:02x}"][y%2*2+x%2]
 occupied={tuple(o['tile'])for o in actors};cameras=[];frames=[]
 for view,requested in [('services',[15,11]),('west-quay',[5,18]),('lighthouse',[15,28]),('south-promendade',[17,29]),('east-coast',[36,20])]:
  x,y=requested
  valid=[(abs(xx-x)+abs(yy-y),xx,yy)for yy in range(a['height']*2)for xx in range(a['width']*2)if kind(xx,yy)in {'FLOOR','TALL_GRASS','LONG_GRASS'}and(xx,yy)not in occupied]
  _,x,y=min(valid);core=[x,y]
  frame,_=player_camera(art,m,actors,ground,scene,'day',dict(core=core),out/name/'views'/view);frames.append((view+' '+str(core),frame));cameras.append(dict(view=view,requested_core=requested,actual_core=core,collision=kind(x,y)))
 save_render(room_sheet(frames,3),out/name/'players.png')
 blocks=SourceArt(pack).blocks(ts);save_render(room_sheet([(f'{i:02x}',im)for i,im in enumerate(blocks)],12),out/name/'native-blocks.png')
 record=dict(pack=filename,pack_sha256=sha(path.read_bytes()),identity=pack['identity']['content_hash'],maps=len(pack['data']['maps']),module_sha256=sha(canonical(m)),attributes=a,objects=[dict(id=o['id'],tile=o['tile'],sprite=o['sprite'])for o in actors],events=m['events'],room_ids=[k for k in pack['data']['maps']if k.startswith('MnGrandMarais')],cameras=cameras,water_edge_core=dict(south=[x for x in range(a['width']*2)if kind(x,a['height']*2-1).startswith('WATER')],east=[y for y in range(a['height']*2)if kind(a['width']*2-1,y).startswith('WATER')]),alias_runtime_files={k:sha(bytes(v))for k,v in pack['runtime_files'].items()if ts in k},block_collision={f'{i:02x}':c.get(f'{i:02x}')for i in range(len(blocks))})
 records.append(record)
 del art,pack,m,ground,scene,blocks;gc.collect()
save_render(room_sheet(wholes,3),out/'town-lineage-comparison.png')
(out/'inspection.json').write_text(json.dumps(records,indent=2)+'\n')
print(json.dumps([{k:r[k]for k in ['pack','maps','module_sha256','attributes','water_edge_core','room_ids']}for r in records]))
