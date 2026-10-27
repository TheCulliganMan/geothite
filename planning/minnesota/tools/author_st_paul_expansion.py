#!/usr/bin/env python3
"""Explicit hand-authored St Paul expansion source; never writes a game pack."""
import copy
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
BASE=ROOT/'planning/minnesota'
OUT=BASE/'towns'
def write(path,value):path.write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n')
def c(command,*args):return dict(command=command,args=list(args))
def text(lines):return [c('text' if i==0 else 'line' if i%2 else 'para',json.dumps(line)) for i,line in enumerate(lines)]+[c('done')]
def actor(id,tile,lines,sprite='SPRITE_POKEFAN_M',service=None,**kw):return dict(id='MN_SPX_'+id,sprite=sprite,tile=tile,facing='south',lines=lines,service=service,**kw)
def main():
 old=json.loads((OUT/'MnStPaul.json').read_text());plan=json.loads((OUT/'MnStPaul.expansion-plan.json').read_text());before=json.loads((OUT/'MnStPaul.layout.json').read_text())
 l=dict(schema_version=2,id='MnStPaul',tileset='kanto',compiled_tileset='kanto',dimensions_metatiles=[96,72],shore_fitting='kanto_8_neighbors',materials=dict(before['materials'],floor='55',pebble='31',lawn='01',green='0a',civic='55'),additional_water_blocks=before['additional_water_blocks'],layers=[])
 layers=l['layers']
 def rect(name,bounds,material='lawn'):layers.append(dict(name=name,kind='rect',material=material,bounds=bounds))
 def block(name,bounds,b):layers.append(dict(name=name,kind='block_rect',bounds=bounds,block=b))
 def point(name,p,b):layers.append(dict(name=name,kind='point',position=p,block=b))
 def poly(name,pts,material):layers.append(dict(name=name,kind='polygon',material=material,points=pts))
 def path(name,pts,width=1,material='floor'):layers.append(dict(name=name,kind='path',material=material,points=pts,width_metatiles=width))
 # Every district outline and street course is deliberate, not random geography.
 poly('Fair campus and Hamline public grounds',[[4,3],[35,3],[35,8],[37,8],[37,23],[32,23],[32,25],[4,25]],'lawn')
 poly('Capitol terrace and Summit street walls',[[39,3],[92,3],[92,24],[67,24],[67,27],[39,27]],'lawn')
 poly('West Seventh and Lowertown inhabited blocks',[[3,26],[63,26],[63,44],[44,44],[44,42],[3,42]],'lawn')
 poly('East side and actual Union Depot north bank',[[66,25],[92,25],[92,54],[77,54],[77,55],[64,55],[64,48],[61,48],[61,44],[66,44]],'lawn')
 poly('West Side river neighborhood',[[6,54],[22,54],[22,57],[54,57],[54,70],[6,70]],'lawn')
 rect('South landing community',[59,64,92,70])
 # Connected civic streets: local lanes two core tiles; major junctions four.
 for name,pts,w in [
 ('West public entry',[[0,36],[18,36],[18,30],[23,30],[23,24]],1),
 ('Fair south entrance and bend',[[18,30],[18,26],[7,26],[7,13],[12,13],[12,3]],1),
 ('Fair public midway',[[7,23],[24,23],[24,13],[33,13],[33,9],[39,9]],2),
 ('Closed northern annex approach',[[24,0],[24,3],[24,7],[18,7],[18,13],[12,13]],1),
 ('Fair north exhibitors lane',[[12,7],[29,7],[29,13]],1),
 ('Capitol approach',[[33,13],[39,13],[39,17],[49,17],[49,15]],1),
 ('Summit avenue offset course',[[39,17],[45,17],[45,23],[68,23],[68,17],[80,17],[80,10],[89,10]],2),
 ('Capitol north lane',[[39,9],[45,9],[45,5],[66,5],[66,13],[72,13],[72,10],[80,10]],1),
 ('Capitol library court',[[49,17],[56,17],[56,13],[60,13]],1),
 ('West Seventh arterial',[[18,36],[29,36],[29,29],[39,29],[39,34],[47,34]],2),
 ('Clinic and West Seventh storefront lane',[[12,36],[12,30],[20,30],[20,29]],1),
 ('Market exchange circuit',[[29,36],[37,36],[37,33],[43,33],[43,38],[52,38],[52,36],[60,36],[60,29],[68,29]],1),
 ('Market river street',[[18,40],[29,40],[29,45],[39,45],[39,43],[49,43],[49,45],[60,45],[60,39],[68,39]],1),
 ('East service street',[[68,23],[68,29],[76,29],[76,33],[85,33],[85,30]],1),
 ('East workshop lane',[[76,33],[76,40],[87,40],[87,43],[81,43],[81,47]],1),
 ('Depot approach from Capitol',[[60,29],[64,29],[64,43],[65,43],[65,48],[70,48],[70,53]],2),
 ('Depot river forecourt',[[65,54],[75,54],[75,47],[86,47]],1),
 ('West river crossing approaches',[[29,40],[29,58],[23,58],[23,64],[9,64],[9,68]],1),
 ('West Side inhabited lane',[[23,64],[31,64],[31,62],[44,62],[44,68],[40,68]],1),
 ('South landing bridge approach',[[65,53],[65,67],[70,67],[70,69],[88,69]],1),
 ('South bank link',[[44,68],[54,68],[54,66],[59,66],[65,66]],1),
 ]:path(name,pts,w)
 # Attached planting and civic back walls break wide traversable lawns.
 for name,b in [('Fair west planted limit',[4,3,7,23]),('Fair barn back',[9,17,22,19]),('Hamline rear trees',[31,3,37,6]),('Capitol rear grove',[40,3,45,8]),('Capitol east planted divider',[54,6,57,15]),('Summit northern backyards',[68,3,91,6]),('Summit music block back',[68,6,72,15]),('Summit press rear',[70,18,73,22]),('Clinic rear garden',[15,25,23,27]),('West urban backyard',[5,27,8,34]),('Market attached orchard',[25,26,29,34]),('Exchange rear strip',[45,27,54,29]),('East gym planted back',[71,25,79,28]),('Workshop rear',[73,34,81,36]),('Freight yard back',[79,48,88,49]),('Depot sheltered east',[89,49,94,55]),('West side north gardens',[6,54,12,58]),('West side quiet rear',[31,56,40,59]),('South side limit',[59,70,93,72])]:rect(name,b,'forest')
 for name,b in [('Fair growing plots',[15,9,19,12]),('Fair flower exhibit',[24,16,27,20]),('Capitol west formal bed',[46,11,48,15]),('Capitol east formal bed',[52,11,54,15]),('Summit press garden',[74,21,78,23]),('Clinic healing garden',[19,31,23,34]),('River botany bed',[34,41,38,44]),('South landing flowers',[72,65,77,67])]:rect(name,b,'flowers')
 # Closed finite river dominates the south without becoming an empty city moat.
 poly('Mississippi working bends',[[5,43],[27,43],[27,45],[43,45],[43,46],[60,46],[60,52],[64,52],[64,55],[87,55],[87,57],[92,57],[92,65],[78,65],[78,63],[59,63],[59,60],[53,60],[53,56],[38,56],[38,54],[20,54],[20,52],[5,52]],'water')
 for name,x,y0,y1 in [('High Bridge west working span',29,43,57),('Robert Street east working span',65,51,66)]:
  block(name+' deck',[x,y0,x+1,y1],'31');block(name+' west rails',[x-1,y0,x,y1],'1b');block(name+' east rails',[x+1,y0,x+2,y1],'5f')
  path(name+' north landing',[[x,y0-1],[x,y0]],1);path(name+' south landing',[[x,y1-1],[x,y1]],1)
 # Actual guard rails visible and collision-bearing; broad banks stay inside map.
 block('Fisher north-bank railing',[44,46,57,47],'77');path('Fisher bank approach',[[49,43],[49,45],[46,45]],1)
 block('Depot river edge railing',[67,55,77,56],'77')
 path('Como garden public side lane',[[7,13],[5,13],[5,10]],1)
 path('Garden stand actual south access',[[18,13],[18,17],[15,17]],1)
 rect('Fair shared seated food court',[8,12,12,16],'civic')
 point('Food court north native bench',[9,13],'08')
 point('Food court east native bench',[11,14],'08')
 # Original directional low bank now connects Capitol terrace and market road.
 block('Capitol retaining face',[46,20,65,21],'1c');block('Capitol attached bank west cap',[42,20,45,21],'1c')
 block('Original ordinary brick return',[45,17,46,24],'05')
 point('Preserved true Capitol down hop',[51,20],'07');path('Capitol hop head',[[51,17],[51,19]],1);path('Capitol hop landing',[[51,21],[51,23]],1)
 # Forecourts and attached mixed-use street walls distinguish city districts.
 rect('Capitol actual public court',[47,15,54,18],'civic')
 rect('Fair exhibition gathering floor',[8,22,16,24],'civic')
 rect('Exchange front trading court',[46,36,53,38],'civic')
 rect('Union Depot passenger forecourt',[67,53,75,55],'civic')

 # Inhabited block backs: exact native roof/window/wall art with NO door.
 # These are architectural masses attached to the district street blocks,
 # not fake landmarks or random obstacles. Entrances remain on named fronts.
 for name,x,y,w,h in [
  ('Fair livestock wing',7,14,7,3),('Agriculture service wing',20,14,3,3),
  ('Fair craft rear wing',29,9,8,3),('Exchange east loading wing',52,30,4,5),('East market street-wall wing',66,31,6,3),('Hamline campus north wing',12,3,7,3),
  ('Fair exhibition west wing',5,19,4,3),('Capitol west office wing',39,21,5,4),
  ('Capitol records rear wing',54,19,5,4),('Summit north apartments',76,3,9,2),
  ('Summit courtyard rear apartments',75,9,4,5),('Summit eastern street wall',83,18,8,5),
  ('West Seventh north apartments',5,25,7,2),('West Seventh corner apartments',10,31,7,3),
  ('West Seventh riverward block',8,38,8,4),('Lowertown warehouse rear',30,32,6,3),
  ('Lowertown east warehouse',55,26,6,3),('Market southern workshop wing',31,39,5,3),
  ('East bank northern apartments',85,23,7,3),('East bank residential back',80,34,5,4),
  ('Depot freight warehouse wing',85,48,5,5),('Depot western loading wall',61,47,3,5),
  ('West Side riverside school wing',21,55,7,3),('West Side residential wall',7,59,5,4),
  ('West Side south block',16,66,9,3),('South neighborhood rear court',37,64,5,3),
  ('South Landing service block',75,64,7,2),
  ('River school rear workshop',54,43,3,3),('Capitol east administrative wing',61,23,6,4),('Clinic south attached neighborhood block',19,37,5,4),('Library quiet west wing',57,5,6,2),
 ]:
  for dy in range(h):
   row=(['20']+['54']*(w-2)+['21'] if dy==0 else ['37']+['7d']*(w-2)+['7e'] if dy==h-1 else ['68']+['7f']*(w-2)+['69'])
   for dx,b in enumerate(row):point(name+' closed native wall '+str((dx,dy)),[x+dx,y+dy],b)
 # Native fenced exhibits have attached planted backs and an open public side.
 for name,b in [('Midway seed garden',[19,8,21,12]),('Summit shared garden',[82,6,85,8]),('West Side food garden',[27,60,30,63]),('Depot passenger garden',[77,52,79,54]),('Summit shared reading garden',[78,19,80,23])]:
  rect(name,b,'green');block(name+' back wall',[b[0],b[1],b[2],b[1]+1],'1c')

 # Purposeful planted courts and working gardens bound routine pedestrian lanes.
 # Native fences are physical walls, not loose decorative bollards.
 for name,b in [
  ('Fair food hall rear growing plot',[13,9,15,12]),('Fair midway west planted shoulder',[12,12,16,14]),
  ('Fair midway east growing shoulder',[20,12,23,15]),('Fair exhibition side plot',[14,23,17,25]),
  ('Fair eastern nursery',[28,14,31,17]),('Hamline south courtyard beds',[33,19,37,22]),
  ('Summit club planted forecourt east',[88,15,92,18]),('Summit central garden west',[75,14,79,17]),
  ('West Seventh north garden',[12,28,13,31]),('West Seventh south planted block',[13,34,18,36]),
  ('Clinic garden west shelter',[16,29,19,34]),('Clinic garden east shelter',[23,29,25,34]),
  ('Market residential courtyard',[39,31,42,33]),('Lowertown riverbed shelter',[40,43,44,45]),
  ('Gym forecourt west planted shoulder',[70,30,73,33]),('Gym forecourt east planted shoulder',[78,29,81,33]),
  ('East bank shared garden',[80,38,84,41]),('Depot forecourt east shelter',[75,53,78,55]),
  ('West Side shared court north',[33,62,35,64]),('West Side shared court south',[32,65,36,68]),
  ('West Side arena planted shoulder',[47,64,50,67]),('South landing inn front shelter',[68,68,70,70]),
  ('South landing survey front garden',[77,69,80,71]),
  ('East community outer shared garden',[89,42,94,49]),('Summit press garden attached shelter',[80,18,83,24]),('Transit heritage quiet western garden',[62,43,64,47]),('Library east reading garden',[62,12,65,16]),('Bluff landing west garden',[46,21,51,23]),('Bluff landing east garden',[52,21,56,23]),('Capitol west attached garden',[40,15,44,17]),('Capitol west bluff garden',[40,18,44,20]),
 ]:
  rect(name,b,'forest')
 # Bounded cultivation plots with real public entrances, rather than empty plazas.
 for name,b in [('Fair bee garden',[25,20,28,23]),('Midway working seed beds',[17,9,20,12]),('West Side school garden',[17,59,20,62])]:
  rect(name,b,'flowers');block(name+' north fence',[b[0],b[1],b[2],b[1]+1],'1c')
 path('Hamline public west shoulder',[[33,9],[32,9],[32,12],[33,12],[33,13]],1)
 path('Clinic garden useful loop',[[19,29],[22,29],[22,30],[20,30],[20,32],[19,32],[19,29]],1)
 path('West Side courtyard actual front',[[33,62],[34,62],[34,64],[31,64]],1)
 path('South landing inn return street',[[67,68],[67,69],[72,69]],1)
 path('South landing survey threshold lane',[[79,69],[83,69]],1)
 # Old doors are moved, never renumbered. Native source facades are complete.
 def facade(name,x,y,w,h,style='commercial',doorcol=1):
  if style=='booth':rows=[['02','03']];dc=0
  elif style=='house':rows=[['38','39'],['37','3a']];dc=1
  elif style=='shop':rows=[['20','54','21'],['7c','73','7e']];dc=0
  elif style=='clinic':rows=[['20','54','21'],['7c','72','7e']];dc=0
  elif style=='gym':rows=[['0c','0d','0e'],['10','11','12']];dc=2
  else:
   rows=[['20']+['54']*(w-2)+['21']]+[['68']+['7f']*(w-2)+['69'] for _ in range(h-2)]+[['37']+['7d']*(w-2)+['7e']];dc=doorcol;rows[-1][dc]='3a'
  for dy,row in enumerate(rows):
   for dx,b in enumerate(row):point(name+' exact facade '+str((dx,dy)),[x+dx,y+dy],b)
  at=[(x+dc)*2,(y+len(rows)-1)*2+1]
  if style in {'shop','clinic','booth'}:at[0]+=1
  path(name+' front threshold',[[at[0]//2,y+len(rows)],[at[0]//2,y+len(rows)+1]],1)
  return at,[at[0],at[1]+1]
 old_buildings=[('Clinic',19,27,3,2,'clinic'),('Mart',35,31,3,2,'shop'),('Capitol',48,10,4,5,'commercial'),('Gym',73,28,3,2,'gym'),('Depot',67,50,8,3,'commercial')]
 for name,x,y,w,h,style in old_buildings:facade(name,x,y,w,h,style,3 if name=='Depot' else 1)
 # Active facades have a real room; no ornamental extra doors.
 specs=[
 ('FairFoodHall',9,9,2,1,'booth','Drinks for the midway.','food'),
 ('FairGardenStand',15,15,2,1,'booth','Seeds and berry stock.','garden'),
 ('FairAgricultureHall',22,9,5,3,'commercial','Crop and water exhibits.','agriculture'),
 ('FairExhibitionHall',10,19,6,3,'commercial','A voluntary show match.','battle0'),
 ('FairCraftHouse',28,17,2,2,'house','Handmade seed ribbons.','craft'),
 ('FairQuizPavilion',26,5,2,2,'house','Three fair questions.','quiz'),
 ('FairInfoHouse',9,5,2,2,'house','Public fair wayfinding.','info'),
 ('FairBreederBarn',18,18,4,3,'commercial','Care before competition.','battle1'),
 ('HamlineCollege',32,6,4,3,'commercial','A native move field guide.','study'),
 ('CapitolArchive',41,10,5,3,'commercial','Collect three archive stamps.','archive'),
 ('CapitolLibrary',58,7,4,5,'commercial','Reading room above the stacks.','library'),
 ('SummitMusicHouse',72,8,2,2,'house','A home for songbirds.','music'),
 ('SummitMoveStudyHouse',79,6,2,2,'house','A Headbutt field note.','headbutt'),
 ('SummitExhibitionClub',84,12,4,3,'commercial','An optional club match.','battle4'),
 ('SummitPressHouse',74,18,2,2,'house','Local streets in print.','press'),
 ('CivicMuseum',60,16,4,3,'commercial','River and capitol history.','museum'),
 ('NorthMarketHome',8,28,2,2,'house','A family on the west road.','home'),
 ('WestSeventhBakery',10,35,3,2,'shop','Picnic food for the river.','bakery'),
 ('LowertownExchange',47,29,5,7,'commercial','Three floors of real destinations.','exchange'),
 ('ArchiveAnnex',37,39,4,3,'commercial','A walking archive stop.','annex'),
 ('LowertownCrafts',58,32,2,2,'house','Trade supplies and craftwork.','tools'),
 ('StreetGardenHouse',25,38,2,2,'house','A pocket garden at the bend.','gardenhouse'),
 ('RiverScienceMuseum',44,41,5,3,'commercial','Read three river measures.','river'),
 ('EastAidClinic',83,28,3,2,'clinic','Native free care.','heal'),
 ('EastWorkshop',75,36,4,3,'commercial','An optional workshop match.','battle2'),
 ('CommunityHall',84,39,5,3,'commercial','Club notices and river walkers.','community'),
 ('TransitHeritageHouse',68,43,5,4,'commercial','Read the actual depot notice.','transit'),
 ('FreightOffice',80,49,4,3,'commercial','Local freight records.','freight'),
 ('LowertownCafe',84,45,3,2,'shop','Rest before the river walk.','cafe'),
 ('AnglerHouse',13,58,2,2,'house','A river fishing challenge.','battle5'),
 ('RiversideSupplies',22,60,3,2,'shop','River-walk supplies.','riverstock'),
 ('WestsideCommunity',32,59,5,3,'commercial','A neighborhood gathering room.','westcommunity'),
 ('WestsideArena',43,61,4,3,'commercial','A voluntary training bout.','battle6'),
 ('RiverRecordsHouse',8,65,2,2,'house','Old river levels in print.','records'),
 ('BoatCoop',40,67,2,2,'house','Boat care on land.','boats'),
 ('SouthLandingInn',66,66,3,2,'shop','Native party rest.','rest'),
 ('SouthLandingSurvey',78,66,4,3,'commercial','A survey team exhibition.','battle7'),

 ('FairBeeHouse',30,21,2,2,'house','Native pollinator records.','bees'),
 ('FairSeedExchange',21,3,2,2,'house','Garden exchange history.','seeds'),
 ('ComoGardenOffice',4,8,2,2,'house','A planted city walk.','como'),
 ('HamlineReadingHouse',32,16,2,2,'house','A quiet reading stop.','reading'),
 ('CapitolPrintOffice',47,4,4,3,'commercial','Read the city register.','printing'),
 ('SummitTeaHouse',67,6,3,2,'shop','A neighborhood tea room.','tea'),
 ('SummitHistoryHouse',87,5,3,4,'commercial','Streets of the upper bluff.','history'),
 ('SummitWestHome',68,18,2,2,'house','A lookout on Summit.','summithome'),
 ('WestSeventhMusicRoom',4,31,3,3,'commercial','A local rehearsal room.','rehearsal'),
 ('WestSeventhBooks',13,27,3,3,'commercial','Secondhand field guides.','bookshop'),
 ('WestSeventhCornerHome',5,38,2,2,'house','A family at the river turn.','cornerhome'),
 ('LowertownArtistLoft',30,27,3,4,'commercial','Working artists upstairs.','artist'),
 ('LowertownMailRoom',39,27,3,3,'commercial','Neighborhood mail supplies.','mail'),
 ('LowertownToolCoop',54,38,3,3,'commercial','Tools for field walks.','cooperative'),
 ('EastBankFieldOffice',87,34,3,3,'commercial','Water marks and weather.','weather'),
 ('EastBankHome',80,25,2,2,'house','A home near the clinic.','easthome'),
 ('WestsideSchool',16,55,4,3,'commercial','A native move classroom.','school'),
 ('WestsideBookRoom',28,66,3,3,'commercial','Books by the south lane.','southbooks'),
 ('SouthMarketGrocer',49,59,3,2,'shop','Berries for long walks.','grocer'),
 ('SouthLandingHome',88,66,2,2,'house','A home at the river bend.','southhome'),

 ('RailwayArchive',75,43,5,4,'commercial','Read working rail history.','railarchive'),
 ('EastMarketArcade',66,35,6,4,'commercial','Native utility supplies.','arcade'),
 ('LowertownCivicSchool',60,32,3,4,'commercial','Practice type matchups.','civicschool'),
 ('RiverEcologySchool',58,41,4,3,'commercial','A safe river field room.','ecology'),
 ('EastMoveSupply',88,28,4,4,'commercial','Native field move supplies.','tmstock'),

 ('FairBerrySnack',11,11,2,1,'booth','Berries for the midway.','berrysnack'),
 ('FairFestivalMail',18,12,2,1,'booth','Send a fair postcard.','fairmail'),
 ]
 contract=dict(schema_version=1,city='MnStPaul',native_module_sha256=plan['native_city_sha256'],old_dimensions_metatiles=[32,24],dimensions_metatiles=[96,72],translation_core=[64,48],arrival_core=[6,72],clinic_constant='MN_ST_PAUL_POKECENTER_1F',default_mart='MART_GOLDENROD_2F_1',anchor_overrides=[],connections=[dict(direction='west',target='MnR04C00R00',old_offset=3,offset=27,town_edge=[[0,72],[0,73]],route_edge=[[55,18],[55,19]])],spawn_overrides=[dict(identifier=271,before=[6,24],after=[6,72])],residents=[],signs=[],doors=[],rooms=[],event_flags=[],programs={},trainers=[],items=[],text_revisions=[])
 for a in plan['native_objects']:contract['anchor_overrides'].append(dict(kind='object',index=a['native_object_index_zero_based'],before=a['old_core'],after=a['new']['global_core']))
 for a in plan['native_bg_records']:contract['anchor_overrides'].append(dict(kind='background',index=a['native_bg_index_zero_based'],before=a['old_core'],after=a['new']['global_core']))
 for a in plan['doors']:
  for rec,after in zip(a['native_indexed_preimages'],[a['new_active']['global_core'],a['new_passive']['global_core']]):contract['anchor_overrides'].append(dict(kind='warp',index=rec['index']-1,before=[rec['x'],rec['y']],after=after))
 fairtexts={'MN_ST_PAUL_FAIR_STEWARDText':['The public fair','is down this road.','North annex shut.'],'MN_ST_PAUL_FAIR_GUARD_WESTText':['North annex shut.','Public fair opens','south of this gate.'],'MN_ST_PAUL_FAIR_GUARD_EASTText':['North annex shut.','Public fair opens','south of this gate.'],'MN_ST_PAUL_SIGN_FAIRText':['STATE FAIR CAMPUS','Public shows south.','North annex shut.']}
 for a in plan['native_objects']+plan['native_bg_records']:
  label=a['id']+'Text'
  if label in fairtexts:contract['text_revisions'].append(dict(label=label,before=a['source_lines_before'],after=fairtexts[label]))
 stocks={'berrysnack':['BERRY_JUICE','BERRY','GOLD_BERRY'],'fairmail':['FLOWER_MAIL','SURF_MAIL','LITEBLUEMAIL'],'arcade':['ESCAPE_ROPE','REPEL','SUPER_REPEL','POKE_DOLL'],'tmstock':['TM_HEADBUTT','TM_PROTECT','TM_MUD_SLAP'],'tea':['FRESH_WATER','MINT_BERRY','BURNT_BERRY'],'mail':['FLOWER_MAIL','SURF_MAIL','LITEBLUEMAIL'],'grocer':['BERRY','GOLD_BERRY','BITTER_BERRY'],'food':['FRESH_WATER','SODA_POP','LEMONADE'],'garden':['BERRY','BITTER_BERRY','MINT_BERRY','FLOWER_MAIL'],'bakery':['BERRY_JUICE','BERRY','PRZCUREBERRY'],'tools':['POKE_BALL','GREAT_BALL','REPEL','ESCAPE_ROPE'],'cafe':['SODA_POP','BERRY_JUICE','FRESH_WATER'],'riverstock':['LURE_BALL','REPEL','SUPER_REPEL','AWAKENING']}
 house=['04 1e 05 1d','0f 01 02 0f','0f 0c 0d 0f','06 0b 0f 07']
 mart=['14 27 14 13 13 13','2b 22 2b 2b 2b 2b','15 29 13 13 13 28','2b 2a 2b 2b 2b 23']
 # Public rooms are individually furnished: vertical shelf bays, two-table
 # workshops, exhibit rows or a central demonstration floor. Native tile art.
 public=['04 05 04 05 04 05 04 1d','0f 0f 0f 0f 0f 0f 0f 0f','0f 01 02 0f 0f 01 02 0f','0f 0c 0d 0f 0f 0c 0d 0f','0f 0f 0f 0f 0f 0f 0f 0f','06 0b 0f 0f 0f 0f 0f 07']
 battle=['04 05 04 05 04 05 04 05','0f 0f 0f 0f 0f 0f 0f 0f','01 02 0f 0f 0f 0f 01 02','0c 0d 0f 0f 0f 0f 0c 0d','0f 0f 0f 0f 0f 0f 0f 0f','06 0b 0f 0f 0f 0f 0f 07']
 parties=[('PICNICKER','SPRITE_LASS',[('HOPPIP',18),('SKIPLOOM',20)]),('POKEFANF','SPRITE_POKEFAN_F',[('SNUBBULL',20),('MILTANK',21)]),('SUPER_NERD','SPRITE_SUPER_NERD',[('MAGNEMITE',20),('PORYGON',21)]),('BIRD_KEEPER','SPRITE_YOUNGSTER',[('HOOTHOOT',20),('NOCTOWL',22)]),('GENTLEMAN','SPRITE_GENTLEMAN',[('FARFETCH_D',21),('PSYDUCK',21)]),('FISHER','SPRITE_FISHER',[('GOLDEEN',20),('QWILFISH',21)]),('BLACKBELT_T','SPRITE_BLACK_BELT',[('MACHOP',21),('TYROGUE',20)]),('CAMPER','SPRITE_YOUNGSTER',[('PHANPY',20),('NIDORINO',21)])]
 def flag(s):f='EVENT_MN_SPX_'+s;contract['event_flags'].append(f);return f
 def warp(index,tile,target,target_warp,passive=False):return dict(index=index,tile=tile,target=target,target_warp=target_warp,passive=passive)
 def room(id,constant,rows,tileset='house',template='MrPsychicsHouse',arrival=[3,5],warps=None):return dict(id=id,native_constant=constant,template=template,tileset=tileset,border_block=0,dimensions_metatiles=[len(rows[0].split()),len(rows)],rows=rows,arrival_core=arrival,warps=warps or [],residents=[],signs=[],trainers=[],items=[],programs={})
 building_records=[]
 for n,(name,x,y,w,h,style,desc,role) in enumerate(specs):
  at,ret=facade(name,x,y,w,h,style)
  # Short sidewalk attaches to explicit district avenue; retain address.
  street_y=y+h+1
  roomid='MnStPaul'+name;constant='MN_SPX_'+''.join('_'+a if a.isupper() else a.upper() for a in name).lstrip('_')
  wi=11+n*2
  contract['doors'].append(dict(warp_index=wi,return_warp_index=wi+1,tile=at,target=roomid,target_warp=3,return_tile=ret,arrival_tile=ret,collision='DOOR'))
  isshop=role in stocks
  rows=mart if isshop else battle if role.startswith('battle') else house if style=='house' else public
  r=room(roomid,constant,rows,'mart' if isshop else 'house','SaffronMart' if isshop else 'MrPsychicsHouse',arrival=[3,9] if len(rows)==6 else [3,5],warps=[warp(1,[2,len(rows)*2-1],'MN_ST_PAUL',wi+1),warp(2,[3,len(rows)*2-1],'MN_ST_PAUL',wi+1),warp(3,[3,9] if len(rows)==6 else [3,5],constant,3,True)])
  a=actor(name.upper()+'_HOST',[1,3] if isshop else [5,3],[desc],service='pokemart' if isshop else 'heal_party' if role in {'heal','rest'} else None)
  if isshop:a.update(stock=stocks[role],approach=[3,3],counter_tile=[2,3])
  r['residents'].append(a)
  if role.startswith('battle'):
   ti=int(role[6:]);klass,sprite,party=parties[ti];r['trainers'].append(dict(id='MN_SPX_TRAINER_'+str(ti+1),display_name=['ELLA','JUNE','EDISON','ROBIN','BERT','FINN','LEE','DALE'][ti],trainer_class=klass,sprite=sprite,tile=[9,7],facing='south',sight_range=0,defeated_flag=flag('TRAINER_'+str(ti+1)),party=[dict(species=s,level=lv) for s,lv in party],before_lines=['A friendly match?','Speak when ready!'],after_lines=['Good preparation!','Enjoy the city.'],loss_lines=['A fine exhibition!']))
  if role=='headbutt':r['items'].append(dict(id='MN_SPX_HEADBUTT',tile=[5,5],item='TM_HEADBUTT',quantity=1,flag=flag('HEADBUTT')))
  r['residents'][0]['lines']=[{'info':['Public shows open.','The north annex','is still closed.'],'transit':['The depot is east.','Read its notices','for real services.'],'music':['Bird songs change','with the daylight.'],'press':['Two bridges link','the river streets.'],'boats':['These are working','boats, not a ferry.'],'freight':['The manifest stays','inside UNION DEPOT.'],'home':['West Seventh runs','to the clinic.'],'gardenhouse':['Small gardens fit','between city homes.'],'records':['High river marks','are in the museum.'],'westcommunity':['A home for walkers','on the west bank.'],'community':['Our neighborhood','meets by the river.'],'study':['Native move books','are on these desks.']}.get(role, [desc[:18]])][0]
  # Useful native mark-and-reward programs attached only to new actors.
  if role in {'archive','river','quiz'}:
   prefix='MN_SPX_'+role.upper();reward={'archive':'MAGNET','river':'MYSTIC_WATER','quiz':'SILVERPOWDER'}[role];done=flag(role.upper()+'_REWARDED');needs=[flag(role.upper()+'_'+str(j)) for j in range(1,4)]
   a['lines']=['Find three records','then return here.'];a['id']=prefix+'_HOST'
   p=r['programs'];p[a['id']]=[c('faceplayer'),c('opentext'),c('checkevent',done),c('iftrue',prefix+'Done')]
   for f in needs:p[a['id']]+=[c('checkevent',f),c('iffalse',prefix+'Need')]
   p[a['id']]+=[c('giveitem',reward,'1'),c('iffalse',prefix+'Full'),c('setevent',done),c('writetext',prefix+'RewardText'),c('waitbutton'),c('closetext'),c('end')]
   for suffix,lines in [('Done',['Our work is saved.','Enjoy the city.']),('Need',['Read three records','around this block.']),('Full',['Your PACK is full.','Return with room.'])]:p[prefix+suffix]=[c('writetext',prefix+suffix+'Text'),c('waitbutton'),c('closetext'),c('end')];p[prefix+suffix+'Text']=text(lines)
   p[prefix+'RewardText']=text(['A field keepsake!','Received '+reward.replace('_',' ')[:9]+'.'])
   # Quiz uses acknowledged instructional records rather than fake yes/no UI.
   prompts={'archive':[['Early rail roads','followed the river.'],['The CAPITOL stands','on the high bluff.'],['Two banks share','one public city.']], 'river':[['Measure north-bank','water at the rail.'],['The bridge spans','both river banks.'],['Read high marks','from the safe path.']], 'quiz':[['Which berry wakes?','MINT BERRY does.'],['Which ball fits','anglers? LURE BALL.'],['Which party move','can cross water?','SURF, when earned.']]}[role]
   for j,(f,lines) in enumerate(zip(needs,prompts)):
    aid=prefix+'_RECORD_'+str(j+1);ra=actor(aid.removeprefix('MN_SPX_'),[3+4*j,9] if len(rows)==6 else [5,5],lines)
    if len(rows)==4:ra['tile']=[2+j,5]
    r['residents'].append(ra);p[aid]=[c('faceplayer'),c('opentext'),c('writetext',aid+'Text'),c('waitbutton'),c('setevent',f),c('closetext'),c('end')];p[aid+'Text']=text(lines)
  contract['rooms'].append(r);building_records.append(dict(id=name,origin_metatile=[x,y],dimensions_metatiles=[w,h],style=style,door_core=at,passive_core=ret,room=roomid,constant=constant,activity=role,description=desc))

 care=next(r for r in contract['rooms'] if r['id']=='MnStPaulEastAidClinic');sourcecare=next(r for r in old['interiors'] if r['id']=='MnStPaulPokecenter1F');care.update(template=sourcecare['template'],tileset=sourcecare['tileset'],dimensions_metatiles=sourcecare['dimensions_metatiles'],rows=sourcecare['rows'],arrival_core=[3,5])
 extreturn=next(d['return_warp_index'] for d in contract['doors'] if d['target']==care['id']);care['warps']=[warp(1,[3,7],'MN_ST_PAUL',extreturn),warp(2,[4,7],'MN_ST_PAUL',extreturn),warp(3,[3,5],care['native_constant'],3,True)]
 care['residents']=[actor('EAST_AID_NURSE',[3,1],['Welcome! Rest your','POKEMON here.'],sprite='SPRITE_NURSE',service='heal_party',approach=[3,3],counter_tile=[3,2])]
 care['signs']=[dict(id='MN_SPX_EAST_AID_PC',tile=[9,1],lines=['A public PC.'])];care['programs']={'MN_SPX_EAST_AID_PC':[c('opentext'),c('special','PokemonCenterPC'),c('closetext'),c('end')]}
 # Authentic multilevel mart architecture, real indexed stair arrivals.
 exchange=next(r for r in contract['rooms'] if r['id']=='MnStPaulLowertownExchange');ec=exchange['native_constant'];
 f1=['03 03 03 03 08 09 0a 02','0b 04 04 04 0c 0d 0e 04','04 07 07 04 04 04 04 04','0b 07 07 06 25 04 04 04']
 f2=['03 03 13 13 13 03 18 19','04 04 04 04 04 04 04 04','04 12 04 12 04 04 10 2c','04 12 04 12 04 04 2e 2d']
 f3=['03 03 17 09 0a 03 01 03','04 04 0c 0d 0e 04 04 04','12 04 12 04 12 04 04 04','12 04 12 04 12 04 13 13']
 extreturn=next(d['return_warp_index'] for d in contract['doors'] if d['target']==exchange['id'])
 exchange.update(template='GoldenrodDeptStore1F',tileset='mart',dimensions_metatiles=[8,4],rows=f1,arrival_core=[7,5],warps=[warp(1,[7,7],'MN_ST_PAUL',extreturn),warp(2,[8,7],'MN_ST_PAUL',extreturn),warp(3,[7,5],ec,3,True),warp(4,[15,0],ec+'_2F',4),warp(5,[15,1],ec,5,True)],residents=[actor('EXCHANGE_RECEPTION',[1,5],['Supplies upstairs.','Club matches on 3F.'])])
 next(d for d in contract['doors'] if d['target']==exchange['id'])['target_warp']=3
 for floor,rows,warps,arrival in [(2,f2,[warp(1,[15,0],ec,5),warp(2,[12,0],ec+'_3F',2),warp(3,[12,1],ec+'_2F',3,True),warp(4,[15,1],ec+'_2F',4,True)],[15,1]),(3,f3,[warp(1,[12,0],ec+'_2F',3),warp(2,[12,1],ec+'_3F',2,True)],[12,1])]:
  r=room(exchange['id']+str(floor)+'F',ec+'_'+str(floor)+'F',rows,'mart','GoldenrodDeptStore'+str(floor)+'F',arrival,warps)
  if floor==2:
   r['residents']=[actor('EXCHANGE_MEDICINE',[13,5],[],service='pokemart',stock=['POTION','SUPER_POTION','ANTIDOTE','PARLYZ_HEAL'],approach=[11,5],counter_tile=[12,5]),actor('EXCHANGE_BALLS',[13,6],[],service='pokemart',stock=['POKE_BALL','GREAT_BALL','LURE_BALL'],approach=[11,6],counter_tile=[12,6])]
  else:
   ti=3;klass,sprite,party=parties[ti];r['trainers']=[dict(id='MN_SPX_TRAINER_4',display_name='ROBIN',trainer_class=klass,sprite=sprite,tile=[12,5],facing='south',sight_range=0,defeated_flag=flag('TRAINER_4'),party=[dict(species=s,level=lv) for s,lv in party],before_lines=['A rooftop club','match indoors?'],after_lines=['Well flown!'],loss_lines=['A fine contest!'])]
  contract['rooms'].append(r)

 # Real yes/no quiz: choices are controller-owned native questions. Wrong or
 # full-bag paths grant nothing and leave the reward flag unset for retry.
 qr=next(r for r in contract['rooms'] if r['id']=='MnStPaulFairQuizPavilion');qr['residents']=qr['residents'][:1];qr['programs']={};qp=qr['programs'];q='MN_SPX_QUIZ';done='EVENT_MN_SPX_QUIZ_REWARDED'
 contract['event_flags']=[f for f in contract['event_flags'] if not f.startswith('EVENT_MN_SPX_QUIZ_') or f==done]
 qp[q+'_HOST']=[c('faceplayer'),c('opentext'),c('checkevent',done),c('iftrue',q+'Done')]
 for i,(lines,wrong) in enumerate([(['MINT BERRY wakes','a sleeping friend?'],'iffalse'),(['A POTION restores','a fainted friend?'],'iftrue'),(['A LURE BALL suits','a fishing catch?'],'iffalse')],1):
  qp[q+'Question'+str(i)+'Text']=text(lines);qp[q+'_HOST'] += [c('writetext',q+'Question'+str(i)+'Text'),c('yesorno'),c(wrong,q+'Wrong')]
 qp[q+'_HOST'] += [c('giveitem','SILVERPOWDER','1'),c('iffalse',q+'Full'),c('setevent',done),c('writetext',q+'RewardText'),c('waitbutton'),c('closetext'),c('end')]
 for suffix,lines in [('Wrong',['Try the field guide','and visit again.']),('Full',['Your PACK is full.','Return with room.']),('Done',['Three good answers!','Enjoy the fair.']),('Reward',['Three good answers!','SILVERPOWDER gift.'])]:
  qp[q+suffix+'Text']=text(lines)
  if suffix!='Reward':qp[q+suffix]=[c('writetext',q+suffix+'Text'),c('waitbutton'),c('closetext'),c('end')]
 # Archive discoveries are distributed among three actual entered landmarks.
 ar=next(r for r in contract['rooms'] if r['id']=='MnStPaulCapitolArchive')
 for j,target in [(2,'MnStPaulCivicMuseum'),(3,'MnStPaulArchiveAnnex')]:
  ac=next(v for v in ar['residents'] if v['id']=='MN_SPX_ARCHIVE_RECORD_'+str(j));ar['residents'].remove(ac);ac['tile']=[11,9];dest=next(r for r in contract['rooms'] if r['id']==target);dest['residents'].append(ac)
  for lab in [ac['id'],ac['id']+'Text']:dest['programs'][lab]=ar['programs'].pop(lab)
 # River measurements are three real outdoor signs: both bridge landings and
 # the old fisher's safe waterfront. The host/reward remains in the museum.
 rr=next(r for r in contract['rooms'] if r['id']=='MnStPaulRiverScienceMuseum')
 for j,at in [(1,[55,87]),(2,[135,129]),(3,[99,91])]:
  ac=next(v for v in rr['residents'] if v['id']=='MN_SPX_RIVER_RECORD_'+str(j));rr['residents'].remove(ac)
  contract['signs'].append(dict(id=ac['id'],tile=at,lines=ac['lines']))
  for lab in [ac['id'],ac['id']+'Text']:contract['programs'][lab]=rr['programs'].pop(lab)
  # Background programs have no actor to turn; all other commands preserved.
  contract['programs'][ac['id']]=[v for v in contract['programs'][ac['id']] if v['command']!='faceplayer']
  point(ac['id']+' actual meter sign',[at[0]//2,at[1]//2],'79')
 # Visible ordinary neighborhood residents and useful river circuit records.
 exterior_actors=[('MIDWAY_VISITOR',[31,29],['Food, flowers and','matches are inside.']),('BRIDGE_WALKER',[58,118],['Both bridges cross','to real streets.']),('EAST_HARBOR_WALKER',[131,132],['A safe river walk','ends at the inn.']),('SUMMIT_READER',[159,37],['The city archive','has three records','to find on foot.'])]
 for name,p,lines in exterior_actors:contract['residents'].append(actor(name,p,lines))
 # Small optional discoveries in meaningful side courts, not mandatory paving.
 for j,(p,item) in enumerate([([17,47],'SUPER_POTION'),([55,35],'BERRY_JUICE'),([91,13],'AWAKENING'),([183,45],'ETHER'),([45,83],'LURE_BALL'),([77,121],'PP_UP'),([101,137],'SUPER_REPEL'),([179,137],'MAX_ETHER')],1):contract['items'].append(dict(id='MN_SPX_ITEM_'+str(j),item=item,quantity=1,flag=flag('ITEM_'+str(j)),tile=p))
 # Exterior sign collision belongs to the sign, with ordinary reachable shoulder.
 pass # Signs authored after pedestrian paths below.
 # Explicit fixed native old anchor sidewalks; no geometry is inferred from pixels.
 for name,pts in [('Civic guide court',[[49,17],[51,17]]),('Bluff walker path',[[45,19],[49,19]]),('Clinic gardener walk',[[20,30],[22,30]]),('Old reader market walk',[[37,36],[40,36]]),('Old river fisher safe top bank',[[46,45],[49,45]]),('Old depot passenger waiting path',[[70,53],[73,53],[73,54]])]:path(name,pts,1)
 path('Heritage district street opening',[[65,42],[67,42],[67,47],[74,47],[74,48],[76,48]],1)
 path('Depot genuine east passenger opening',[[75,52],[75,54],[73,54]],1)
 path('South landing public continuous lane',[[73,69],[83,69]],1)
 path('West Seventh bookshop south alley',[[14,30],[16,30],[16,34],[18,34],[18,36]],1)
 for a in plan['native_bg_records']:
  p=a['new']['global_core'];point(a['id']+' relocated sign',[p[0]//2,p[1]//2],'79')
 # Close every exterior edge except the true west seam and guarded annex.
 for name,b in [('North outer city wall',[0,0,96,1]),('South outer city wall',[0,71,96,72]),('West outer city wall',[0,0,1,72]),('East outer city wall',[95,0,96,72])]:block(name,b,'13')
 point('Actual R04 two-lane walking seam',[0,36],'31');point('Guarded northern annex beyond public fair',[24,0],'31');point('Guard pair floor',[24,1],'31')
 # Ordinary source-only PC functionality must be native-bound separately; old
 # clinic PC remains exact. No fake exterior doors or unimplemented train UI.
 dossier=copy.deepcopy(old);dossier.update(id='MnStPaul',dimensions_metatiles=[96,72],compiled_tileset='kanto',status='SOURCE_ONLY_HAND_AUTHORED_CITY_EXPANSION; pending native/controller/visual acceptance',expansion_contract='planning/minnesota/towns/MnStPaul.expansion.contract.json',expansion_plan='planning/minnesota/towns/MnStPaul.expansion-plan.json',new_buildings=building_records,new_activities=dict(optional_trainers=8,shops=len(stocks)+2,acknowledged_record_errands=['archive','river'],native_yes_no_quiz=True,source_only=True,rail_travel_added=False),new_rooms=contract['rooms'],source_review=dict(native_gameplay_proved=False,reference_cities=['GoldenrodCity','SaffronCity'],old_room_modules_preserved=True))

 # Full candidate dossier coordinates agree with the executable overrides.
 for d in dossier['doors']:
  move=next(v for v in plan['doors'] if v['id']==d['id']);d['tile']=move['new_active']['global_core'];d['return_tile']=move['new_passive']['global_core'];d['arrival_tile']=d['return_tile']
 for key,source in [('residents','native_objects'),('signs','native_bg_records')]:
  for v in dossier[key]:
   move=next(q for q in plan[source] if q['id']==v['id']);v['tile']=move['new']['global_core']
   revised=next((q for q in contract['text_revisions'] if q['label']==v['id']+'Text'),None)
   if revised:v['lines']=revised['after']
 dossier['one_way_ledges']=[dict(id='CAPITOL_BLUFF_DROP',bounds_core=[102,40,104,42],direction='down',approach=[103,39],landing=[103,42],ordinary_return='west native brick return45,17..23; native05 remains ordinary walking ladder collision')]
 dossier['ports'][0].update(gate_metatile=[1,36],landing_metatile=[3,36])
 dossier['source_review']['limits']=['Source-only, no native/controller acceptance','Every original room/program is an unchanged native dependency','Foundational activity transactions require native roundtrip/controller tests','No rail travel or main quest added']
 # Move two voluntary exhibitions into visible public courts; retain exact flags/parties.
 for roomid,tile in [('MnStPaulFairExhibitionHall',[27,47]),('MnStPaulSouthLandingSurvey',[171,137])]:
  r=next(v for v in contract['rooms'] if v['id']==roomid)
  if r['trainers']:trainer=r['trainers'].pop();trainer['tile']=tile;contract['trainers'].append(trainer)
 dossier['new_rooms']=contract['rooms']
 write(OUT/'MnStPaul.expanded.layout.json',l);write(OUT/'MnStPaul.expansion.contract.json',contract);write(OUT/'MnStPaul.expanded.candidate.json',dossier)
 print('Hand-authored',len(building_records),'new street facades,',len(contract['rooms']),'rooms;',len(layers),'explicit strokes')
if __name__=='__main__':main()
