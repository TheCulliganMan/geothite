"""Whole native-art source winter-section review; no native publication or generator."""
from pathlib import Path
import cbor2,copy,hashlib,importlib.util,json,re,sys,time
sys.dont_write_bytecode=True
sys.path.insert(0,'planning/minnesota/tools')
from PIL import Image
from render_archive import save_render
from render_stages import render_stage
from render_reference import layout_blocks,room_sheet
from review_major_city import ReviewArt,render_map,player_camera
SOURCE=Path(sys.argv[1]);OUT=Path(sys.argv[2]);assert not OUT.exists();OUT.mkdir(parents=True)
p=Path('content-packs/minnesota-cohesion-harbor-water-v01.crystalpack');raw=p.read_bytes();assert hashlib.sha256(raw).hexdigest()=='3d1cbdd18b05ce10ed66db7ab56d41f0032f6e045a0e72ad753064c5dc6ec8df';pack=cbor2.loads(raw[22:]);specpath=Path('planning/minnesota/regions/ice/winter-art-v01.json');specraw=specpath.read_bytes();helperpath=Path('planning/minnesota/regions/ice/preview_winter_art_v02.py');helperraw=helperpath.read_bytes();assert hashlib.sha256(specraw).hexdigest()=='4efd8ef08fc06bfc870144014602aca5ca639b34015bade3dddb5ac769aedd74';assert hashlib.sha256(helperraw).hexdigest()=='b6b2b08fd6ee056fd00e9897ca99d7b354c414004a7010573dfff421acd5eb56';(OUT/'inputs').mkdir();(OUT/'inputs/winter-art-v01.json').write_bytes(specraw);(OUT/'inputs/preview_winter_art_v02.py').write_bytes(helperraw);spec=json.loads(specraw);loader=importlib.util.spec_from_file_location('winter_art','planning/minnesota/regions/ice/preview_winter_art_v02.py');winter=importlib.util.module_from_spec(loader);loader.loader.exec_module(winter);warm=time.perf_counter();derived,receipts=winter.derive(pack,spec)
sys.path.insert(0,'planning/minnesota/regions/ice')
import preview_winter_edges_v01 as edges
ep=Path('planning/minnesota/regions/ice/winter-mixed-edges-v01.json');eraw=ep.read_bytes();eh=Path('planning/minnesota/regions/ice/preview_winter_edges_v01.py');ehraw=eh.read_bytes();assert hashlib.sha256(eraw).hexdigest()=='a6d8f46a813555c5daff1deffe0d2cb0682570f244971c28fc19f985593df40c';assert hashlib.sha256(ehraw).hexdigest()=='ac254d961bf18e2022114d7e849e87d28956305a4221e9022020b549d3ebf553';(OUT/'inputs/winter-mixed-edges-v01.json').write_bytes(eraw);(OUT/'inputs/preview_winter_edges_v01.py').write_bytes(ehraw);derived,edge_receipts=edges.derive_edges(derived,json.loads(eraw));art=ReviewArt(derived);frames=[];report={}
for r,views in [('17',[('north-snow-entry',[36,5]),('old-pines-habitat',[19,24]),('cedar-gutter-north-stop',[48,20]),('west-marsh-bank',[11,55]),('frozen-marsh-west-launch',[13,55]),('frozen-marsh-south-stop',[20,60]),('timber-crossing-approach',[30,64]),('cut-unchanged-approach',[36,68]),('southern-snowline',[24,84])]),('11',[('northern-dry-turn',[40,8]),('upper-bank-south-stop',[36,24]),('ryan-house-branch',[28,24]),('old-icy-habitat',[46,29]),('shore-frost-item-stop',[48,33]),('spring-gorge-upper-shelf',[25,64]),('snowline-stream',[33,78]),('southern-snowline',[32,82])])]:
 layout=json.loads((SOURCE/f'MN_R{r}.layout.candidate.json').read_bytes());old=json.loads((SOURCE/f'MN_R{r}.layout.json.before').read_bytes());w,h=layout['dimensions_metatiles'];before=layout_blocks(old);after=layout_blocks(layout);objects=[];sample=None;native=[None]*(w*h);module_hashes={}
 for name,m in pack['data']['maps'].items():
  match=re.fullmatch(f'MnR{r}C(\\d+)R(\\d+)',name)
  if not match:continue
  c,row=map(int,match.groups());a=m['attributes'];sample=m;module_hashes[name]=hashlib.sha256(json.dumps(m,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
  for y in range(a['height']):native[(row*24+y)*w+c*32:(row*24+y)*w+c*32+a['width']]=m['blocks'][y*a['width']:(y+1)*a['width']]
  for o in m['objects']:objects.append(dict(id=o.get('object_identifier')or o['script'],sprite=o['sprite'],tile=[c*64+o['x'],row*48+o['y']],facing={'UP':'north','DOWN':'south','LEFT':'west','RIGHT':'east'}.get(o['spritemovedata'].rsplit('_',1)[-1],'south'),native_palette_override=o.get('pal',0)))
 assert None not in native
 caps=[dict(at=[i%w,i//w],source_block=f'{a:02x}',installed_block=f'{b:02x}')for i,(a,b)in enumerate(zip(before,native))if a!=b]
 assert caps==([]if r=='17'else[dict(at=[20,0],source_block='05',installed_block='01')])
 # The current native cap overlay is preserved on BOTH preview states.
 for cap in caps:
  i=cap['at'][1]*w+cap['at'][0];before[i]=int(cap['installed_block'],16);after[i]=int(cap['installed_block'],16)
 delta=[dict(at=[i%w,i//w],before=f'{a:02x}',after=f'{b:02x}')for i,(a,b)in enumerate(zip(before,after))if a!=b]
 module=dict(id='MN_R'+r,attributes=dict(sample['attributes'],width=w,height=h,tileset_name=layout['compiled_tileset']),blocks=after)
 before_module=dict(module,blocks=before)
 oldcol=pack['data']['tilesets'][old['compiled_tileset']]['collision'];newcol=derived['data']['tilesets'][layout['compiled_tileset']]['collision']
 ground,scene,masked=render_map(art,module,objects,'day',OUT/f'r{r}/after')
 bg,bscene,_=render_map(art,before_module,objects,'day',OUT/f'r{r}/before')
 save_render(render_stage(scene,'resize',OUT/f'r{r}','wholefit',(round(scene.width*min(1,850/scene.width,1500/scene.height)),round(scene.height*min(1,850/scene.width,1500/scene.height))),Image.Resampling.NEAREST),OUT/f'r{r}/whole-review-fit.png')
 occupied={tuple(o['tile'])for o in objects}
 for name,at in views:
  assert tuple(at)not in occupied
  k=newcol[f'{after[at[1]//2*w+at[0]//2]:02x}'][at[1]%2*2+at[0]%2]
  assert k in ['FLOOR','TALL_GRASS','ICE'],(name,k)
  for label,mod,g,sc in [('before',before_module,bg,bscene),('after',module,ground,scene)]:
   # Reuse the native player/priority/crop renderer on an exact small metatile window.
   # Full original whole scenes remain archived once; no duplicate map-size PNG for each crop.
   left=max(0,min(sc.width-160,at[0]*16-80));top=max(0,min(sc.height-144,at[1]*16-72))
   x0,y0=left//32,top//32;x1,y1=min(w,(left+160+31)//32),min(h,(top+144+31)//32)
   subblocks=[mod['blocks'][yy*w+xx]for yy in range(y0,y1)for xx in range(x0,x1)]
   sub=dict(mod,attributes=dict(mod['attributes'],width=x1-x0,height=y1-y0),blocks=subblocks)
   box=(x0*32,y0*32,x1*32,y1*32);folder=OUT/f'r{r}/views/{name}/{label}'
   subground=render_stage(g,'crop',folder,'exact-terrain-window',box);subscene=render_stage(sc,'crop',folder,'exact-actor-window',box)
   local=[at[0]-x0*2,at[1]-y0*2]
   assert max(0,min(subscene.width-160,local[0]*16-80))+x0*32==left
   assert max(0,min(subscene.height-144,local[1]*16-72))+y0*32==top
   frame,_=player_camera(art,sub,[],subground,subscene,'day',dict(id=name,core=local),folder);frames.append((f'R{r} {name} {label}',frame))
 report[r]=dict(status='SOURCE_RECONSTRUCTION_ONLY_NATIVE_PENDING',native_module_sha256=module_hashes,cap_overlay=caps,changed_metatiles=delta,views=views,alias=layout['compiled_tileset'],new_block_collisions={b:newcol[b]for b in set(c['after']for c in delta)})
save_render(room_sheet(frames,4),OUT/'decision-before-after-sheet.png');assert specpath.read_bytes()==specraw and helperpath.read_bytes()==helperraw,'art inputs changed while rendering';report['art_source_sha256']=hashlib.sha256(specraw).hexdigest();report['art_helper_sha256']=hashlib.sha256(helperraw).hexdigest();report['art_receipts']=receipts;report['mixed_edge_receipts']=edge_receipts;assert ep.read_bytes()==eraw and eh.read_bytes()==ehraw,'mixed art inputs changed';report['warm_render_elapsed_seconds']=round(time.perf_counter()-warm,3);(OUT/'review.json').write_text(json.dumps(report,indent=2)+'\n');print('source renders saved',OUT,report['warm_render_elapsed_seconds'])
