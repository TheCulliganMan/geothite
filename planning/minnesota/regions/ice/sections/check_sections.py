"""Native collision source proof; Foot ICE over-approximation is not runtime execution."""
from pathlib import Path
import collections,copy,hashlib,importlib.util,json,re,sys
sys.path.insert(0,'planning/minnesota/tools')
import cbor2
from render_reference import layout_blocks
SOURCE=Path(sys.argv[1]);OUT=Path(sys.argv[2]);OUT.mkdir(parents=True,exist_ok=False)
packpath=Path('content-packs/minnesota-cohesion-harbor-water-v01.crystalpack');raw=packpath.read_bytes();assert hashlib.sha256(raw).hexdigest()=='3d1cbdd18b05ce10ed66db7ab56d41f0032f6e045a0e72ad753064c5dc6ec8df';pack=cbor2.loads(raw[22:])
helper=Path('planning/minnesota/regions/ice/preview_winter_art_v02.py');specpath=Path('planning/minnesota/regions/ice/winter-art-v01.json');spec=json.loads(specpath.read_bytes());mod=importlib.util.spec_from_file_location('winter',helper);winter=importlib.util.module_from_spec(mod);mod.loader.exec_module(winter);preview,_=winter.derive(pack,spec)
sys.path.insert(0,'planning/minnesota/regions/ice')
import preview_winter_edges_v01 as edges
preview,_=edges.derive_edges(preview,json.loads(Path('planning/minnesota/regions/ice/winter-mixed-edges-v01.json').read_bytes()))
DIRS=[(0,1),(0,-1),(1,0),(-1,0)];DRY={'FLOOR','TALL_GRASS','ICE'};sha=lambda x:hashlib.sha256(x).hexdigest()
def path(start,end,allowed):
 start,end=tuple(start),tuple(end);q=collections.deque([start]);prev={start:None}
 while q:
  p=q.popleft()
  if p==end:
   result=[]
   while p is not None:result.append(p);p=prev[p]
   return result[::-1]
  for dx,dy in DIRS:
   at=p[0]+dx,p[1]+dy
   if at in allowed and at not in prev:prev[at]=p;q.append(at)
 return None
report={}
for r in ['17','11']:
 old=json.loads((SOURCE/f'MN_R{r}.layout.json.before').read_bytes());new=json.loads((SOURCE/f'MN_R{r}.layout.candidate.json').read_bytes());d=json.loads((SOURCE/f'MN_R{r}.json.before').read_bytes());nd=json.loads((SOURCE/f'MN_R{r}.candidate.json').read_bytes());w,h=old['dimensions_metatiles'];before=layout_blocks(old);after=layout_blocks(new);oc=pack['data']['tilesets'][old['compiled_tileset']]['collision'];nc=preview['data']['tilesets'][new['compiled_tileset']]['collision'];native=[None]*(w*h);objects=[];preimages={}
 for name,m in pack['data']['maps'].items():
  match=re.fullmatch(f'MnR{r}C(\\d+)R(\\d+)',name)
  if not match:continue
  c,row=map(int,match.groups());a=m['attributes'];preimages[name]=sha(json.dumps(m,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())
  for yy in range(a['height']):native[(row*24+yy)*w+c*32:(row*24+yy)*w+c*32+a['width']]=m['blocks'][yy*a['width']:(yy+1)*a['width']]
  for o in m['objects']:objects.append((c*64+o['x'],row*48+o['y']))
 assert None not in native
 caps=[dict(at=[i%w,i//w],source=a,native=b)for i,(a,b)in enumerate(zip(before,native))if a!=b];assert caps==([]if r=='17'else[dict(at=[20,0],source=5,native=1)])
 for cap in caps:before[cap['at'][1]*w+cap['at'][0]]=cap['native']
 assert {k:v for k,v in nd.items()if k not in ['compiled_tileset','winter_section_review']}==d
 assert new['layers'][:len(old['layers'])]==old['layers']
 assert {k:v for k,v in new.items()if k not in ['compiled_tileset','winter_section_review','shore_fitting','layers']}=={k:v for k,v in old.items()if k not in ['compiled_tileset','shore_fitting','layers']}
 def lookup(blocks,col,p):
  x,y=p
  return col[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]if 0<=x<w*2 and 0<=y<h*2 else'OUTSIDE'
 ok=lambda p:lookup(before,oc,p);nk=lambda p:lookup(after,nc,p)
 occupied=set(objects);ice={(x,y)for y in range(h*2)for x in range(w*2)if nk((x,y))=='ICE'};assert ice and not ice&occupied,'ice replaces native actor footing'
 entries=[]
 for p in sorted(ice):
  for dx,dy in DIRS:
   start=p[0]-dx,p[1]-dy
   if nk(start)not in {'FLOOR','TALL_GRASS'}or start in occupied:continue
   slide=[];end=p
   while nk(end)=='ICE':slide.append(end);end=end[0]+dx,end[1]+dy;assert len(slide)<256
   blocked_stop=nk(end)not in {'FLOOR','TALL_GRASS'}
   turns=[]
   if blocked_stop:
    assert nk(end)in {'WALL','HEADBUTT_TREE','UP_WALL'} and end not in occupied,('unreadable hazard stop',r,start,end,nk(end))
    stop=slide[-1]
    for tx,ty in DIRS:
     if (tx,ty)==(dx,dy):continue
     q=stop[0]+tx,stop[1]+ty;turn_path=[]
     while nk(q)=='ICE':turn_path.append(q);q=q[0]+tx,q[1]+ty
     if nk(q)in {'FLOOR','TALL_GRASS'}and q not in occupied:turns.append(dict(direction=[tx,ty],ice_path_core=[list(a)for a in turn_path],dry_stop_core=list(q)))
    assert turns,('stranded ice wall stop',r,start,stop)
   else:stop=end;assert stop not in occupied,('occupied stop',r,start,stop)
   entries.append(dict(from_core=list(start),direction=[dx,dy],ice_path_core=[list(p)for p in slide],stop_core=list(stop),stop_kind='intentional_wall_stop_on_ICE'if blocked_stop else'dry_or_grass_stop',blocked_target_core=list(end)if blocked_stop else None,ordinary_turn_exits=turns))
 def surface_guards(candidate,design):
  assert {k:v for k,v in design.items()if k not in ['compiled_tileset','winter_section_review']}==d,'original actor/party/item/story dossier changed'
  ck=lambda p:lookup(candidate,nc,p)
  limits=new['winter_section_review'].get('snowline_exclusive_y_by_meta_x')
  if limits:
   assert len(limits)==w
   for i,(a,b)in enumerate(zip(before,candidate)):
    if a!=b:assert i//w<limits[i%w],'winter change outside authored geographic snowline'
  for y in range(h*2):
   for x in range(w*2):
    p=x,y;a,b=ok(p),ck(p)
    assert(a=='TALL_GRASS')==(b=='TALL_GRASS'),('grass changed',p)
    if a not in {'FLOOR','TALL_GRASS','WATER'}:assert a==b,('gate/ledge/warp/wall changed',r,p,a,b)
    if a=='WATER'and b!='WATER':assert r=='17'and 6<=x//2<15 and 25<=y//2<32,('water changed outside explicit upper marsh',p)
    if x in [0,w*2-1]or y in [0,h*2-1]:assert a==b,('boundary collision changed',r,p)
    if p in occupied:assert a==b,('actor footing changed',p)
 surface_guards(after,nd)
 negatives=[]
 def reject(label,blocks=None,design=None):
  try:surface_guards(after if blocks is None else blocks,nd if design is None else design)
  except AssertionError as error:negatives.append(dict(mutation=label,rejected=True,reason=str(error)))
  else:raise AssertionError('Uncaught source negative:'+label)
 b=after.copy();b[0]=0x3e if r=='17'else 0x85;reject('ICE opens outer pine boundary',blocks=b)
 grass_index=next(i for i,b in enumerate(before)if oc[f'{b:02x}']==['TALL_GRASS']*4);b=after.copy();b[grass_index]=0x3c if r=='17'else 0x83;reject('Original encounter habitat erased',blocks=b)
 x,y=objects[0];b=after.copy();b[y//2*w+x//2]=0x3e if r=='17'else 0x85;reject('Original native actor standing tile becomes forced ICE',blocks=b)
 broken=copy.deepcopy(nd);broken['residents'][0]['tile'][0]+=1;reject('Moved original resident',design=broken)
 b=after.copy();outside=next(i for i,v in enumerate(before)if i//w>=45 and v==5);b[outside]=0x3c if r=='17'else 0x83;reject('Southern unreviewed forest cleared',blocks=b)
 if r=='17':
  b=after.copy();b[34*w+18]=0x3c;reject('Cut tree replaced by Snow Floor',blocks=b)
  b=after.copy();b[34*w+11]=0x3c;reject('Frozen lower lake opens unapproved bridge bypass',blocks=b)
  b=after.copy();b[32*w+11]=0x3c;reject('Timber rail erased',blocks=b)
 else:
  b=after.copy();i=next(i for i,v in enumerate(before)if 'WATERFALL'in oc[f'{v:02x}']);b[i]=0x85;reject('Animated waterfall frozen and badge route changed',blocks=b)
  b=after.copy();b[20]=5;reject('Installed northern cohesion opening lost',blocks=b)
  b=after.copy();b[20*w+30]=0x85;reject('Open Lake Superior frozen',blocks=b)
 dry={(x,y)for y in range(h*2)for x in range(w*2)if nk((x,y))in DRY and(x,y)not in occupied};old_dry={(x,y)for y in range(h*2)for x in range(w*2)if ok((x,y))in DRY and(x,y)not in occupied}
 endpoints=[tuple(v*2 for v in e['gate_metatile'])for e in d['endpoints']];main=path(*endpoints,dry);old_main=path(*endpoints,old_dry)
 if r=='17':assert main,'main Foot path severed'
 else:assert dry==old_dry,'R11 old directional walking footprint changed'
 summary=dict(status='STATIC_SOURCE_MODEL_ONLY_NOT_NATIVE_CONTROLLER',native_module_sha256=preimages,native_source_cap_overlays=caps,old_dossier_gameplay_exact=True,changed_metatiles=sum(a!=b for a,b in zip(before,after)),ice_core_tiles=len(ice),all_legal_ice_entries_safe=entries,legal_slide_entries=len(entries),whole_grass_footprint_exact=True,all_original_actor_footings_exact=True,boundary_collision_exact=True,main_floor_only_overapproximate_steps=len(main)-1 if main else None,old_floor_only_overapproximate_steps=len(old_main)-1 if old_main else None,graph_limit='Floor/Grass/ICE orthogonal over-approximation: preserves gate checks on R17, not a full R11 native ledge/stair/room journey. R11 all directional collision tokens and walking footprint remain exact.')
 # Every old actor/sign/room approach was already reachable without Surf; no newly ungated old island reward.
 old_targets=[]
 for key in ['items','residents','trainers','doors','signs']:
  for actor in d.get(key,[]):
   at=tuple(actor['tile']);approaches=[(at[0]+dx,at[1]+dy)for dx,dy in DIRS]
   old_access=any(path(endpoints[0],a,old_dry)for a in approaches if a in old_dry);new_access=any(path(endpoints[0],a,dry)for a in approaches if a in dry)
   assert old_access==new_access,('old target access changed',actor.get('id',actor.get('map',actor.get('target_map','unnamed-door'))),old_access,new_access)
   old_targets.append(dict(id=actor.get('id',actor.get('map',actor.get('target_map','unnamed-door'))),old_and_new_foot_access=old_access))
 summary['old_targets']=old_targets
 if r=='17':
  cut=d['shortcut'];post=after.copy();x,y=cut['tile'];post[y//2*w+x//2]=int(cut['replacement_block'],16);pk=lambda p:lookup(post,nc,p)
  for surf in [False,True]:
   preallowed={(x,y)for y in range(h*2)for x in range(w*2)if nk((x,y))in(DRY|({'WATER'}if surf else set()))and(x,y)not in occupied};postallowed={(x,y)for y in range(h*2)for x in range(w*2)if pk((x,y))in(DRY|({'WATER'}if surf else set()))and(x,y)not in occupied}
   pre=path(cut['compare_from'],cut['compare_to'],preallowed);opened=path(cut['compare_from'],cut['compare_to'],postallowed);assert pre and opened and len(pre)-len(opened)>=cut['minimum_saved_steps'],('Cut bypass',surf,len(pre)if pre else None,len(opened)if opened else None)
   summary['cut_'+('surf'if surf else'foot')]=dict(before_steps=len(pre)-1,opened_steps=len(opened)-1,saved_steps=len(pre)-len(opened))
  crossing=old['crossings'][0];closed={(x*2+dx,y*2+dy)for field in ['north_rail_blocks','south_rail_blocks']for x,y in crossing[field]for dx in [0,1]for dy in [0,1]};assert not path(*endpoints,dry-closed),'Frozen marsh creates free timber bridge bypass';summary['bridge_still_required_without_Cut']=True
  grass=d['required_grass_crossings'][0];x0,y0,x1,y1=grass['bounds_core'];band={(x,y)for y in range(y0,y1)for x in range(x0,x1)};assert not path(*endpoints,dry-band),'Old sedge neck bypassed';summary['required_sedge_neck_still_required']=True
 summary['source_negative_checks']=negatives
 report[r]=summary
(OUT/'proof.json').write_text(json.dumps(dict(status='SOURCE_MODEL_PASS_NATIVE_PENDING',routes=report,parent_pack=str(packpath),parent_pack_sha256=sha(raw),art_spec_sha256=sha(specpath.read_bytes()),art_helper_sha256=sha(helper.read_bytes()),tool_sha256=sha(Path(__file__).read_bytes())),indent=2)+'\n');print({r:{k:v for k,v in row.items()if k in ['changed_metatiles','ice_core_tiles','legal_slide_entries','cut_foot','cut_surf','bridge_still_required_without_Cut']}for r,row in report.items()})
