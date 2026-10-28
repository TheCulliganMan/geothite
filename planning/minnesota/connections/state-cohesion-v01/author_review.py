#!/usr/bin/env python3
"""Explicit hand-authored cohesion source candidate, not a native publisher."""
import argparse,collections,copy,hashlib,json,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt,layout_blocks,load_pack
from render_progress_atlas import scene,native_room,font
from render_archive import save_render
from audit_outdoor_connections import collision,neighbor_origin,OPPOSITE,project
from PIL import Image,ImageDraw

def digest(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
def write(p,v):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,indent=2)+'\n')
def rect(name,bounds,material):return dict(name=name,kind='rect',bounds=bounds,material=material)
def path(name,points,material='floor'):return dict(name=name,kind='path',points=points,width_metatiles=1,material=material)
def layout(name,wh,layers):return dict(schema_version=2,id=name,tileset='johto',compiled_tileset='johto',dimensions_metatiles=wh,shore_fitting='explicit_source_edges',reject_cliff_cuts=True,source_pixels_modified=False,layers=layers)
def route(name,wh,layers,ports,actors,signs,views):return dict(id=name,layout=layout(name,wh,layers),ports=ports,actors=actors,signs=signs,views=views)
def actor(id,tile,sprite,name,cls,party):return dict(id=id,tile=tile,sprite=sprite,facing='south',role='optional_trainer',trainer_name=name,trainer_class=cls,party=[dict(species=s,level=l)for s,l in party],event_flag='EVENT_'+id+'_DEFEATED',text=['A short forest walk.','Battle with me?'])

def candidates():
 return [route('MnDetroitLakesApproach',[24,18],[
  rect('Connected upper pine meadow',[3,6,9,9],'lawn'),rect('Small lower passing pocket',[12,10,15,13],'lawn'),
  path('West trail winds into the meadow',[[0,9],[4,9],[4,7],[8,7],[8,10],[13,10],[13,12],[18,12],[18,9],[23,9]]),
  rect('Whole upper meadow grass crossing',[6,6,7,9],'encounter'),rect('Lower narrow grass neck',[16,12,17,13],'encounter'),
  rect('Field flowers beside occupied turnout',[3,6,4,7],'flowers'),rect('Lower turn flowers',[12,11,13,12],'flowers'),
 ],[dict(side='west',core=[[0,18],[0,19]]),dict(side='east',core=[[47,18],[47,19]])],[
  actor('MN_COHESION_DETROIT_KEEPER',[7,15],'SPRITE_YOUNGSTER','TAM','BIRD_KEEPER',[('NOCTOWL',30),('PIDGEOTTO',29)]),
  actor('MN_COHESION_DETROIT_WALKER',[24,21],'SPRITE_LASS','IRIS','LASS',[('FURRET',30),('HOPPIP',29)]),
 ],[dict(id='MN_COHESION_DETROIT_SIGN',tile=[8,13],lines=['DETROIT LAKES','WOODLAND APPROACH','Follow the pine trail.'])],
 [dict(id='west-meadow',core=[10,15]),dict(id='lower-pine-bend',core=[29,24]),dict(id='east-return',core=[39,18])]),
 route('MnMaraisCedarApproach',[32,19],[
  rect('West reading turnout attached to path',[2,4,5,8],'lawn'),rect('Cedar crossing small occupied meadow',[14,8,18,11],'lawn'),
  path('Coastal cedar trail around attached forest fingers',[[0,6],[6,6],[6,4],[11,4],[11,9],[17,9],[17,13],[23,13],[23,16],[28,16],[28,18]]),
  rect('Required west grass neck',[9,4,10,5],'encounter'),rect('Full cedar-meadow grass strip',[16,8,17,11],'encounter'),
  rect('Return-side flower bank',[14,8,15,9],'flowers'),rect('South approach attached turnout',[26,15,28,17],'lawn'),
 ],[dict(side='west',core=[[0,12],[0,13]]),dict(side='south',core=[[56,37],[57,37]])],[
  actor('MN_COHESION_MARAIS_KEEPER',[29,19],'SPRITE_YOUNGSTER','CEDAR','BIRD_KEEPER',[('NOCTOWL',36),('FARFETCH_D',35)]),
  actor('MN_COHESION_MARAIS_WALKER',[53,31],'SPRITE_LASS','MIRA','LASS',[('VENONAT',35),('FURRET',36)]),
 ],[dict(id='MN_COHESION_MARAIS_SIGN',tile=[7,9],lines=['CEDAR APPROACH','GRAND MARAIS west.','NORTH SHORE south.'])],
 [dict(id='west-arrival',core=[7,12]),dict(id='cedar-meadow',core=[31,19]),dict(id='south-pine-turn',core=[49,32])]),
 route('MnPreservePineEntry',[24,16],[
  rect('Small ranger reading pocket',[6,5,10,8],'lawn'),
  path('Public-facing pine approach to the closed preserve',[[8,0],[8,3],[5,3],[5,6],[9,6],[9,10],[6,10],[6,13],[8,13],[8,15]]),
  rect('Attached flower verge',[7,5,8,6],'flowers'),
 ],[dict(side='north',core=[[16,0],[17,0]]),dict(side='south',core=[[16,31],[17,31]])],[],[
  dict(id='MN_COHESION_PRESERVE_NOTICE',tile=[13,11],lines=['PRESERVE ENTRY','Expeditions closed.','No fee is charged.'])],
 [dict(id='north-court-approach',core=[16,5]),dict(id='ranger-reading',core=[16,13]),dict(id='south-pine-turn',core=[14,27])]),
 route('MnBemidjiNorthwoods',[8,14],[
  path('A narrow pine neck clears the old west and east roads',[[4,13],[4,11],[2,11],[2,8],[5,8],[5,5],[3,5],[3,2],[4,2],[4,0]]),
  rect('Small inside bend grass crossing',[3,5,4,6],'encounter'),
 ],[dict(side='north',core=[[8,0],[9,0]]),dict(side='south',core=[[8,27],[9,27]])],[],[],
 [dict(id='south-pine-entry',core=[8,23]),dict(id='middle-grass-bend',core=[7,11]),dict(id='north-forest-turn',core=[9,4])])]

PATCH_STROKES={
 'MnBrainerd': [([0,12,2,13],0x01),([31,12,32,13],0x01)],
 'MnBemidji': [([12,0,13,1],0x01)],
 'MnR18C00R00':[([0,5,1,6],0x01)],
 'MnLakeItasca':[([0,14,4,15],0x01),([20,10,24,11],0x01)],
 'MnR14C00R00':[([0,19,5,20],0x01),([4,18,8,19],0x01),([4,18,5,20],0x01)],
 'MnR19C00R00':[([30,6,32,7],0x01)],
 'MnGrandMarais':[([0,4,1,5],0x01),([18,6,20,7],0x01)],
 'MnR11C00R00':[([20,0,21,2],0x01)],
 'MnBoundaryWaters':[([16,0,17,3],0x01)],
 'MnBoundarySafariC00R02':[([7,5,8,8],0x01)],
 'MnR08C00R00':[([20,0,21,1],0x05),([12,0,13,3],0x01),([12,2,16,3],0x01),([15,1,21,2],0x01)],
}

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--output',type=Path,required=True);a=ap.parse_args();a.output.mkdir(parents=True,exist_ok=False)
 base=ROOT/'content-packs/minnesota-twin-cities-compact-v01.crystalpack';raw=base.read_bytes();assert hashlib.sha256(raw).hexdigest()=='2fe6fc2a2c59833886b9ffa2fa04584f1f95841efaf87e865f11230cf4cdf54e';pack=load_pack(base);art=SourceArt(pack);maps=copy.deepcopy({n:m for n,m in pack['data']['maps'].items()if n.startswith('Mn') and str(m['attributes']['environment']).lower() in ('town','route','outdoor')});checks=[];patches=[]
 for name,strokes in PATCH_STROKES.items():
  m=maps[name];before=copy.deepcopy(m);w=m['attributes']['width'];changed=[]
  for bounds,block in strokes:
   x0,y0,x1,y1=bounds
   for y in range(y0,y1):
    for x in range(x0,x1):m['blocks'][y*w+x]=block
  col_before=collision(pack,before);col_after=collision(pack,m)
  for i,(old,new)in enumerate(zip(before['blocks'],m['blocks'])):
   if old!=new:changed.append(dict(metatile=[i%w,i//w],before=old,after=new,before_quadrants=pack['data']['tilesets'][m['attributes']['tileset_name']]['collision'][f'{old:02x}'],after_quadrants=pack['data']['tilesets'][m['attributes']['tileset_name']]['collision'][f'{new:02x}']))
  conflicts=[c for c in changed if any(t in ('WATER','WATERFALL','TALL_GRASS','CUT_TREE','CUT_TREE_1A','CUT_08','CUT_28')for t in c['before_quadrants'])]
  points={(o['x'],o['y'])for o in before['objects']}|{(o['x'],o['y'])for key in ['warps','bg_events','coord_events']for o in before['events'][key]}
  changed_core={(2*c['metatile'][0]+dx,2*c['metatile'][1]+dy)for c in changed for dx in range(2)for dy in range(2)}
  checks.append(dict(id=name+'-protected-collision',pass_=not conflicts,conflicts=conflicts));checks.append(dict(id=name+'-native-anchor-collision',pass_=all(col_before[p]==col_after[p]for p in points)))
  patches.append(dict(map=name,native_module_sha256=digest(before),dimensions_metatiles=[w,m['attributes']['height']],changes=changed,conflicts=conflicts))
  native_room(before,pack,archive=a.output/'ports'/name/'before',native_origin=True);native_room(m,pack,archive=a.output/'ports'/name/'candidate',native_origin=True)
 # Removing an entirely blocking margin changes no original traversable quadrant.
 name='MnR03C00R01';m=maps[name];before=copy.deepcopy(m);w=m['attributes']['width'];removed=[b for y in range(m['attributes']['height'])for b in m['blocks'][y*w+21:(y+1)*w]];assert all(pack['data']['tilesets'][m['attributes']['tileset_name']]['collision'][f'{b:02x}']==['WALL']*4 for b in removed)
 m['blocks']=[b for y in range(m['attributes']['height'])for b in m['blocks'][y*w:y*w+21]];m['attributes']['width']=21
 patches.append(dict(map=name,native_module_sha256=digest(before),kind='remove_blocking_margin',before_dimensions_metatiles=[w,8],after_dimensions_metatiles=[21,8],removed_bounds_metatiles=[21,0,24,8],removed_quadrants={'WALL':96},native_anchors_unchanged=True))
 native_room(before,pack,archive=a.output/'ports'/name/'before',native_origin=True);native_room(m,pack,archive=a.output/'ports'/name/'candidate',native_origin=True)
 # Explicit source connectors; no fake facade, station or paid admission.
 for r in candidates():
  l=r['layout'];n=r['id'];
  for sign in r['signs']:
   x,y=sign['tile'];l['layers'].append(dict(name=sign['id']+' authentic signpost',kind='point',position=[x//2,y//2],block='45'))
  blocks=layout_blocks(l);w,h=l['dimensions_metatiles'];write(a.output/(n+'.layout.json'),l);write(a.output/(n+'.dossier.json'),{k:v for k,v in r.items()if k!='layout'})
  im=scene(art,pack,'johto',w,h,blocks,r['actors'],archive=a.output/'routes'/n,native_origin=True)
  for v in r['views']:
   cx,cy=v['core'];left=max(0,min(w*32-160,cx*16-80));top=max(0,min(h*32-144,cy*16-72));crop=im.crop((left,top,left+160,top+144));save_render(crop,a.output/'routes'/n/(v['id']+'-160.png'));save_render(crop.resize((640,576),Image.Resampling.NEAREST),a.output/'routes'/n/(v['id']+'-4x.png'))
 write(a.output/'bounded-native-port-patches.json',dict(status='DETACHED_SOURCE_PROPOSAL_NOT_NATIVE',parent_pack_sha256=hashlib.sha256(raw).hexdigest(),patches=patches))
 write(a.output/'initial-port-protection-checks.json',dict(status='SOURCE_ONLY_PROVISIONAL',checks=checks))
 print(json.dumps({'output':str(a.output),'changed_port_cells':sum(len(p.get('changes',[]))for p in patches),'protected_conflicts':[(p['map'],p['conflicts'])for p in patches if p.get('conflicts')],'failures':[c['id']for c in checks if not c['pass_']]}))
if __name__=='__main__':main()
