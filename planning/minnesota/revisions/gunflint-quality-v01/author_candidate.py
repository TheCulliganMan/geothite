#!/usr/bin/env python3
"""Hand-composed compact Gunflint source; no pack writes or random terrain."""
from pathlib import Path
import copy,json,hashlib,sys
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack,layout_blocks,room_sheet
from review_major_city import ReviewArt,render_map,player_camera
from PIL import Image
HERE=Path(__file__).resolve().parent
PACK=ROOT/'content-packs/minnesota-superior-inland-winter-v01.crystalpack'
SHA='65cf6f98e65ada839ec5afaa57728bf8289d3a50ba53ba6b9b50f08d3620bb17'
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 epoch=sys.argv[1] if len(sys.argv)>1 else 'source-v01'
 src=HERE/epoch;src.mkdir(exist_ok=True)
 out=ROOT/'planning/minnesota/renders/gunflint-quality-v01'/epoch
 assert not out.exists(), 'immutable render epoch required';out.mkdir(parents=True)
 assert sha(PACK.read_bytes())==SHA
 pack=load_pack(PACK)
 dossier_path=ROOT/'planning/minnesota/routes/MN_R12.json';layout_path=dossier_path.with_name('MN_R12.layout.json')
 for p in [dossier_path,layout_path]:
  target=src/(p.name+'.before')
  if target.exists():assert target.read_bytes()==p.read_bytes(), 'canonical changed while authoring'
  else:target.write_bytes(p.read_bytes())
 d=json.loads(dossier_path.read_text());old=json.loads(layout_path.read_text())
 layers=[]
 def poly(name,points,material='floor'):layers.append(dict(kind='polygon',name=name,points=points,material=material))
 def path(name,points,width=1,material='floor'):layers.append(dict(kind='path',name=name,points=points,width_metatiles=width,material=material))
 def rect(name,bounds,material='floor'):layers.append(dict(kind='rect',name=name,bounds=bounds,material=material))
 def point(name,position,block):layers.append(dict(kind='point',name=name,position=position,block=block))
 # Three differently shaped inhabited spaces, joined by a short bent lake trail.
 poly('South town-side trailhead and ranger forecourt',[[24,31],[28,31],[28,28],[29,28],[29,25],[28,25],[28,23],[26,23],[26,22],[23,22],[23,24],[22,24],[22,26],[24,26],[24,28]])
 path('Only actual south town approach',[[27,31],[27,27],[25,27],[25,25]])
 poly('Loons Lake walking shore, east landing and main westbound saddle',[[24,25],[26,25],[26,23],[25,23],[25,20],[26,20],[26,17],[27,17],[27,13],[26,13],[26,11],[24,11],[24,14],[22,14],[22,16],[20,16],[20,17],[16,17],[16,16],[14,16],[14,14],[11,14],[11,16],[9,16],[9,18],[12,18],[12,19],[16,19],[16,18],[20,18],[20,20],[23,20],[23,22],[24,22]])
 path('West Boundary Waters physical exit',[[0,14],[5,14],[5,16],[9,16],[9,17],[13,17]])
 poly('West bank and Windthrow fork',[[4,12],[6,12],[6,11],[9,11],[9,12],[11,12],[11,14],[12,14],[12,17],[10,17],[10,18],[7,18],[7,16],[4,16]])
 # The open lake is an irregular arm with opposing reachable shores, never
 # another rectangle enclosed by a one-cell forest moat.
 poly('Loons open-water basin with short south arm',[[12,11],[14,11],[14,10],[18,10],[18,11],[22,11],[22,12],[24,12],[24,14],[22,14],[22,15],[20,15],[20,16],[18,16],[18,15],[15,15],[15,16],[13,16],[13,15],[12,15]],'water')
 path('Actual north lake shore to shelter',[[12,10],[12,9],[15,9],[15,8],[18,8],[18,9],[21,9],[21,10],[25,10],[25,12]])
 # Cut clearing is fully ringed by forest except its native gate at14,17.
 rect('Useful Cut-sealed herb clearing',[6,5,8,8])
 path('Herb gate approach from west fork',[[8,12],[8,10],[7,10],[7,9]])
 point('Native Cut gate preserves forest0f replacement',[7,8],'0f')
 # Compact uneven Raven granite brow above its actual cave mouth.
 layers.append(dict(kind='cliff_polygon',name='Raven Rock jagged shelter brow',points=[[16,4],[19,4],[19,3],[21,3],[21,4],[23,4],[23,6],[22,6],[22,7],[18,7],[18,6],[16,6]]))
 point('Raven native cave front',[20,7],'34')
 poly('Shelter step and interpretation shoulder',[[17,8],[18,8],[18,7],[20,7],[20,8],[22,8],[22,9],[20,9],[20,10],[18,10],[18,9],[17,9]])
 point('Restore exact native cave face after shoulder',[20,7],'34')
 path('Raven ridge cache short climb',[[17,8],[17,5],[16,5]])
 # A coherent frost-slick basin with a full dry walking rim and openings to
 # both shores. Native slide stops are bounded by that rim, not item alcoves.
 poly('North frost basin dry rim',[[21,4],[26,4],[26,5],[29,5],[29,9],[27,9],[27,11],[24,11],[24,10],[20,10],[20,9],[18,9],[18,6],[21,6]])
 poly('Substantial frost-slick granite basin',[[23,5],[25,5],[25,6],[28,6],[28,8],[27,8],[27,9],[22,9],[22,8],[23,8]],'ice')
 path('East frost rim returns without field moves',[[28,9],[29,9],[29,5],[26,5],[26,4],[22,4],[22,5]])
 # A southern bog actually shares a shore with its grass neck, service stop
 # and side discoveries. All remain walkable without a mid-slide turn.
 poly('Southern bog shore and hammock clearing',[[7,21],[9,21],[9,20],[12,20],[12,21],[15,21],[15,23],[17,23],[17,25],[15,25],[15,27],[13,27],[13,29],[10,29],[10,28],[8,28],[8,26],[7,26]])
 path('Short bog loop joins main trail twice',[[9,17],[9,21],[8,21],[8,25],[11,25],[11,27],[14,27],[14,24],[17,24],[17,21],[20,21],[20,19],[23,19]])
 poly('Western bog inlet',[[6,22],[8,22],[8,23],[10,23],[10,25],[8,25],[8,26],[6,26],[6,25],[5,25],[5,23],[6,23]],'water')
 poly('Eastern bog pool',[[13,22],[15,22],[15,23],[16,23],[16,25],[14,25],[14,26],[13,26]],'water')
 rect('Bog grass neck makes the short route meaningful',[11,22,12,26],'encounter')
 rect('Windthrow habitat across west route',[9,14,11,15],'encounter')
 path('Ranger porch connects useful bog fork',[[17,24],[20,24],[20,25],[24,25]])
 # House rows preserve exact original Ilex structure, with real door quadrant.
 for y,row in enumerate([['1d','1e','1f'],['21','22','23'],['24','26','27']]):
  for x,b in enumerate(row):point('Original ranger hut native structure',[24+x,21+y],b)
 # Intentional partial canopies make clearings non-rectangular; no incidental Cut.
 for pos,b in [([5,11],'42'),([10,11],'43'),([4,15],'46'),([11,17],'48'),([15,18],'42'),([21,19],'43'),([28,20],'46'),([23,25],'48'),([10,27],'42'),([14,28],'43'),([17,9],'46')]:point('Deliberate native snow canopy shoulder',pos,b)
 # Coherent pine peninsulas shorten blank crossings and bring the bank into
 # player cameras. These are explicit landscape masses, not scattered props.
 poly('West shore pine spur',[[6,12],[8,12],[8,13],[9,13],[9,15],[8,15],[8,14],[6,14]],'forest')
 poly('South lake pine tongue',[[15,17],[17,17],[17,18],[16,18],[16,19],[14,19],[14,18],[15,18]],'forest')
 poly('Central bog ridge closes empty diagonal lawn',[[13,19],[16,19],[16,20],[17,20],[17,21],[15,21],[15,22],[13,22]],'forest')
 poly('East shore pine promontory',[[25,14],[26,14],[26,15],[27,15],[27,17],[26,17],[26,16],[25,16]],'forest')
 poly('Ranger northwood',[[22,20],[24,20],[24,21],[23,21],[23,23],[22,23]],'forest')
 poly('South approach west pine mass',[[23,27],[25,27],[25,28],[26,28],[26,30],[24,30],[24,29],[23,29]],'forest')
 rect('South approach east pine shoulder',[28,28,30,31],'forest')
 rect('Porch side pine shoulder',[26,25,28,27],'forest')
 # Reapply deliberate narrow thoroughfares after landscape masses.
 path('Lake south walking bank',[[14,18],[20,18],[20,17],[22,17],[22,19],[23,19],[23,25],[24,25]])
 path('Principal visible Loons east shore',[[24,25],[23,25],[23,21],[22,21],[22,19],[24,19],[24,17],[25,17],[25,13],[25,10],[21,10],[21,9],[18,9],[18,8],[15,8],[15,9],[12,9],[12,12],[10,12],[10,14],[5,14]])
 path('South approach bent final path',[[27,31],[27,27],[25,27],[25,25]])
 # Keep granite roof and native entrance visible beside, never under, the ice.
 layers.append(dict(kind='cliff_polygon',name='Raven shelter final native brow',points=[[16,4],[19,4],[19,3],[21,3],[21,4],[23,4],[23,6],[22,6],[22,7],[18,7],[18,6],[16,6]]))
 point('Raven exact native cave mouth',[20,7],'34')
 point('Raven safe exterior step',[20,8],'3c')
 # Final regional arrival is east54/55; the superseded south exit is
 # visibly enclosed by two complete rows of native pine forest.
 rect('Close superseded southern exterior port',[0,30,32,32],'forest')
 path('Short actual east arrival to ranger junction',[[31,27],[27,27],[25,27],[25,25]])
 rect('Required west portage habitat',[2,14,3,15],'encounter')
 # Restore all original house rows after final paths so the entrance keeps
 # its actual DOOR quadrant instead of a white floor overpaint.
 for y,row in enumerate([['1d','1e','1f'],['21','22','23'],['24','26','27']]):
  for x,b in enumerate(row):point('Final intact ranger hut',[24+x,21+y],b)
 rect('Close obsolete southern spur below east arrival',[23,28,32,30],'forest')
 residents=[[59,55],[48,9],[25,33],[22,55],[51,37],[23,27]]
 trainers=[[55,49],[51,29],[51,21],[37,19],[19,25],[39,43],[25,55],[25,47],[17,43],[17,21]]
 items=[[13,11],[55,11],[23,55],[34,9]]
 signs=[[61,53],[49,33],[43,17],[47,49],[23,45],[5,27]]
 for field,points in [('residents',residents),('trainers',trainers),('items',items),('signs',signs)]:
  assert len(d[field])==len(points)
  for obj,at in zip(d[field],points):
   obj['tile']=at
   if 'anchor_metatile' in obj:obj['anchor_metatile']=[v//2 for v in at]
 for sign in d['signs']:point('Real native winter sign '+sign['id'],[v//2 for v in sign['tile']],'41')
 # Explicit small actor feet, bounded within authored district clearings.
 for field in ['residents','trainers','items']:
  for obj in d[field]:
   at=[v//2 for v in obj['tile']]
   point('Stable footprint for '+obj['id'],at,'3c')
 # Reinstate gate walls after actors. Cache item lives west of the only gate.
 point('Native Cut gate',[7,8],'0f')
 d['items'].append(dict(id='MN_INLAND_CAMP_SUPPLY',item='BERRY',quantity=3,flag='EVENT_MN_GUNFLINT_CAMP_SUPPLY',tile=[51,53],access='Ranger junction trail stock; native event2387 moved from retired inland Neck.'))
 d['signs'].append(dict(id='MN_INLAND_CAMP_SIGN',tile=[55,53],lines=['PORTAGE CAMP','Three BERRIES for','the trail ahead.']))
 point('Camp stock stable footprint',[25,26],'3c')
 point('Relocated native camp sign',[27,26],'41')
 d['dimensions_metatiles']=[32,32];d['chunk_columns']=1;d['chunk_rows']=2
 d['endpoints']=[dict(place='CEDAR APPROACH',side='E',gate_metatile=[31,27],landing_metatile=[29,27]),dict(place='BOUNDARY WATERS',side='W',gate_metatile=[0,14],landing_metatile=[2,14])]
 d['main_path_metatiles']=[[31,27],[27,27],[25,27],[25,25],[23,25],[23,21],[22,21],[22,19],[24,19],[24,17],[25,17],[25,10],[21,10],[21,9],[18,9],[18,8],[15,8],[15,9],[12,9],[12,12],[10,12],[10,14],[0,14]]
 for obj,at in zip(d['doors'],[[40,15],[49,47]]):
  obj['tile']=at;obj['arrival_tile']=at;obj['return_tile']=[at[0],at[1]+1]
 d['cut_gates'][0]['tile']=[14,17];d['cut_gates'][0]['approach']=[14,18]
 if 'targets' in d['cut_gates'][0]:d['cut_gates'][0]['targets']=[[14,11]]
 d['status']='DETACHED compact Gunflint whole-route revision; no native publication or controller acceptance.'
 d['description']='A shorter inland winter route around the Loons lake shore: a Raven Rock shelter, coherent frost basin with dry rim, two useful bog banks and the ranger hut. The Cut herb clearing is optional and actually sealed.'
 d['districts']=[dict(id='LOONS_SHORES',purpose='One open lake with connected opposite walking banks, useful lookout and short main trail.'),dict(id='RAVEN_FROST_BROW',purpose='Real shelter cave beside a coherent slippery basin; continuous dry rim and clear native slide stops.'),dict(id='STONE_BOG_RANGER',purpose='Dry-access sign and trainer pocket, real grass neck, hammock resource and nearby ranger room.')]
 d['loops']=[dict(id='LOONS_CIRCUIT',purpose='North shelter/ice visit rejoins both walking banks without an empty route-wide detour.'),dict(id='BOG_BANK_RETURN',purpose='Walkable southern discovery circuit; every sign/trainer has ordinary dry access.')]
 d['viewpoints']=[dict(id=name,tile=at)for name,at in [('LAKE_WEST_BANK',[23,27]),('LOONS_BANK',[50,28]),('CLEFT_APPROACH',[40,16]),('RANGER_PORCH',[49,49]),('BOG_BANK',[22,49]),('CUT_APPROACH',[14,18]),('WIDEST_MAIN_CLEARING',[32,37]),('FROST_RIM',[55,19])]]
 d['required_grass_crossings']=[dict(id='WEST_PINE_SEDGE',bounds_core=[4,28,6,30],minimum_grass_steps=2)]
 d['portages']=[dict(id='BOG_GRASS_NECK',bounds_core=[22,44,24,52],purpose='Useful shorter passage between two ponds, with a dry outer circuit and an actual shore landing.')]
 d['implementation']='Detached complete32x32 candidate; native endpoint and room pairing integration pending. Original room content, parties, flags, rewards and story identities remain unchanged.'
 d['native_integration']=dict(compiled_tileset='mn_winter_gunflint',proposed_chunks=[dict(id='MnR12C00R00',dimensions=[32,24]),dict(id='MnR12C00R01',dimensions=[32,8])],native_status='NOT PUBLISHED; compare source and actual final native controller after integration.')
 d['revision_contract_changes']=['Gunflint shortened64x56 to32x32; source-v02 inland winter epoch retained as immutable history.','External ports are proposed east63,54/55 and west0,28/29; root integrates actual regional footprints.','All actor/room/reward/script identities retained with explicit relocated exterior anchors; old regional fixtures need new coordinates.','Pine8b copied from installed native winter art as one declared derived block49; Cut replacement prefix remains original.']
 d.pop('inland_winter_review',None);d.pop('coastal_geography_binding',None)
 for interior,at in zip(d['interiors'],[[40,15],[49,47]]):
  interior['exterior_anchor_metatile']=[v//2 for v in at];interior['exterior_door_core']=at
  interior['return']=f'Native paired exterior warp{tuple(at)} settles onto clear footprint{(at[0],at[1]+1)}; room rows/events are unchanged.'
 d['quality_revision']='gunflint-quality-v01'
 d['proposed_ports']=dict(east_core=[[63,54],[63,55]],west_core=[[0,28],[0,29]],origin_metatiles=[190,-432],east_native_map='MnR12C00R01',east_local_core=[[63,6],[63,7]],east_offset=-4,west_native_map='MnR12C00R00',west_offset=2,global_geometry_owner='root')
 layout=dict(schema_version=2,id='MN_R12',tileset='forest',compiled_tileset='mn_winter_gunflint',dimensions_metatiles=[32,32],materials=dict(floor='3c',lawn='3c',encounter='3d',forest='49',water='02',ice='3e'),cliff_blocks=old['cliff_blocks'],derived_blocks=old['derived_blocks']+[dict(id='49',source_tileset='mn_winter_north_shore',source_block='8b',collision=['WALL']*4)],shore_fitting='forest_8_neighbors',layers=layers)
 blocks=layout_blocks(layout)
 recipes=[r for r in layout['derived_blocks'] if int(r['id'],16)>=len(ReviewArt(pack).art.blocks('mn_winter_gunflint'))]
 for r in recipes:r.setdefault('collision',pack['data']['tilesets'][r['source_tileset']]['collision'][r['source_block']])
 module=dict(id='MnR12CompactReview',attributes=dict(pack['data']['maps']['MnR12C00R00']['attributes'],width=32,height=32),blocks=blocks)
 objects=[]
 for field in ['residents','trainers','items']:
  for o in d[field]:objects.append(dict(id=o['id'],tile=o['tile'],sprite=o.get('sprite','SPRITE_POKE_BALL'),facing=o.get('facing','south')))
 art=ReviewArt(pack);ground,scene,_=render_map(art,module,objects,'day',out/'whole',recipes)
 scene.resize((1024,1024),Image.Resampling.NEAREST).save(out/'whole-display.png')
 views=[('east-arrival',[59,54]),('ranger-shore',[49,49]),('loons-east-bank',[50,28]),('lake-west-bank',[23,27]),('frost-entry',[38,17]),('frost-rim',[55,19]),('raven-shelter',[40,16]),('cut-approach',[14,18]),('bog-neck',[22,49]),('hammock',[23,54]),('west-exit',[3,28])]
 cameras=[]
 for name,at in views:
  frame,_=player_camera(art,module,objects,ground,scene,'day',dict(core=at),out/'cameras'/name,recipes);cameras.append((name,frame))
 room_sheet(cameras,3).save(out/'player-contact.png')
 write(src/'MN_R12.candidate.layout.json',layout);write(src/'MN_R12.candidate.json',d)
 write(src/'receipt.json',dict(status='DETACHED_SOURCE_ONLY',pack_sha256=SHA,canonical_preimages={p.name:sha(p.read_bytes())for p in[dossier_path,layout_path]},dimensions=[32,32],old_area=64*56,new_area=32*32,derived_art_imports=recipes,views=views))
 print(out)
if __name__=='__main__':main()
