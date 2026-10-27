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
 old=json.loads((OUT/'MnStPaul.json').read_text());plan=json.loads((OUT/'MnStPaul.tight.plan.json').read_text());before=json.loads((OUT/'MnStPaul.layout.json').read_text())
 l=dict(schema_version=2,id='MnStPaul',tileset='kanto',compiled_tileset='kanto',source_pixels_modified=False,reject_cliff_cuts=True,dimensions_metatiles=[64,48],shore_fitting='kanto_8_neighbors',materials=dict(before['materials'],floor='55',lawn='01'),additional_water_blocks=before['additional_water_blocks'],layers=[])
 layers=l['layers']
 def rect(name,bounds,material='lawn'):layers.append(dict(name=name,kind='rect',material=material,bounds=bounds))
 def block(name,bounds,b):layers.append(dict(name=name,kind='block_rect',bounds=bounds,block=b))
 def point(name,p,b):layers.append(dict(name=name,kind='point',position=p,block=b))
 def poly(name,pts,material):layers.append(dict(name=name,kind='polygon',material=material,points=pts))
 def path(name,pts,width=1,material='floor'):layers.append(dict(name=name,kind='path',material=material,points=pts,width_metatiles=width))

 # Explicit compact city district landforms. Main streets2–4core; local alleys2.
 poly('Fair exhibition grounds',[[2,2],[23,2],[23,7],[24,7],[24,18],[2,18]],'lawn')
 poly('Capitol terrace and Summit',[[26,2],[61,2],[61,17],[44,17],[44,19],[26,19]],'lawn')
 poly('West Seventh and market streets',[[2,19],[39,19],[39,25],[41,25],[41,30],[2,30]],'lawn')
 poly('East side and Depot bank',[[44,18],[61,18],[61,38],[43,38],[43,35],[40,35],[40,26],[44,26]],'lawn')
 poly('West Side streets',[[5,38],[20,38],[20,39],[37,39],[37,46],[5,46]],'lawn')
 rect('South landing inhabited bank',[43,43,61,47])

 # Exact native0a urban lawns distinguish narrow paved streets from the
 # planted river/civic shoulders. They do not invent encounter grass.
 for name,bb in [('Fair shared food green',[4,8,11,14]),('Fair agriculture lawn',[16,3,23,9]),('Capitol bluff public green',[35,10,42,13]),('Summit north shared lawn',[51,2,56,8]),('Summit club garden',[54,8,61,13]),('Clinic garden turf',[6,23,11,27]),('West Seventh planted sidewalk margin',[11,20,13,26]),('Archive riverward lawn',[25,29,30,31]),('Gym upper approach green',[47,16,54,22]),('East clinic shared green',[54,18,61,23]),('Depot bank green',[42,35,54,39]),('West Side community green',[11,41,19,44]),('South landing inn green',[43,43,52,47])]:block(name,bb,'0a')
 for name,pts,w in [
  ('R04 West Seventh entry',[[0,24],[14,24],[14,20],[24,20],[24,17],[14,17],[14,3]],1),
  ('Fair closed northern annex',[[14,0],[14,4],[18,4]],1),
  ('Fair food and exhibition circuit',[[5,5],[5,8],[10,8],[10,10],[14,10],[14,17],[5,17],[5,16]],1),
  ('Fair booth side lane',[[8,10],[8,12],[15,12],[15,13],[19,13],[19,17]],1),
  ('Fair north crop hall lane',[[14,8],[18,8],[18,7],[22,7],[22,11],[22,12],[19,12]],1),
  ('Capitol west approach',[[14,17],[26,17],[26,10],[31,10],[31,8]],1),
  ('Capitol upper court',[[28,7],[28,9],[38,9],[38,8],[42,8],[42,10],[48,10]],1),
  ('Original useful brick return',[[28,10],[28,17],[36,17]],1),
  ('Capitol true hop approach',[[36,10],[36,12]],1),
  ('Capitol lower landing',[[36,14],[36,17],[42,17],[42,20],[48,20]],1),
  ('Summit inhabited ribbon',[[48,10],[48,6],[53,6],[53,8],[58,8],[58,12],[52,12],[52,15],[49,15],[48,15],[48,17]],1),
  ('East gym service street',[[48,17],[51,17],[51,21],[56,21],[56,22],[60,22]],1),
  ('Clinic front garden loop',[[6,23],[10,23],[10,24],[14,24],[14,23]],1),
  ('West bakery and garden lane',[[3,27],[6,27],[6,24],[10,24],[10,28],[15,28],[15,29],[18,29]],1),
  ('Market principal street',[[14,24],[18,24],[18,26],[24,26],[24,24],[31,24]],1),
  ('Exchange real front court',[[31,25],[31,26],[37,26],[37,24],[38,24]],1),
  ('Market south alley',[[18,26],[18,29],[27,29],[27,30],[31,30],[31,29],[38,29],[38,24]],1),
  ('Market to East Workshop',[[38,24],[44,24],[44,29],[48,29],[48,28]],1),
  ('East clinic and community lane',[[56,21],[60,21],[60,26],[55,26],[55,29],[53,29]],1),
  ('Depot upper approach',[[38,29],[41,29],[41,31],[42,31],[42,35],[46,35]],1),
  ('Depot eastern road',[[53,29],[53,35],[50,35],[50,37],[47,37],[47,36]],1),
  ('West bridge north approach',[[18,29],[18,30]],1),
  ('West Side public lane',[[18,37],[18,39],[18,43],[24,43],[24,45],[30,45],[34,45]],1),
  ('West Side community loop',[[8,40],[8,42],[13,42],[13,43],[18,43]],1),
  ('West Side arena approach',[[24,45],[26,45],[26,44]],1),
  ('West Side Angler lane',[[6,44],[6,45],[13,45],[13,43]],1),
  ('South crossbank route',[[34,45],[38,45],[38,44],[41,44],[41,35]],1),
  ('South landing inn street',[[41,44],[43,44],[43,46],[59,46]],1),
 ]:path(name,pts,w)
 # Urban backs physically frame streets; no doors or false enterable buildings.
 for name,x,y,w,h in [
  ('Fair north hall wing',9,2,4,3),('Fair east barn rear',19,12,4,2),
  ('Capitol west records wing',26,12,2,4),('Summit north apartments',46,2,5,3),
  ('Summit press east courtyard',55,13,6,3),('Summit eastern apartments',56,3,5,4),('Summit west street frontage',44,7,3,5),('Workshop west service wing',43,26,3,3),('Gym north urban block',44,17,3,3),('Summit attached central street wall',50,9,4,3),('Market workshop rear',34,27,3,3),('West Seventh clinic service wing',3,19,7,2),
  ('Market residential back',20,18,7,3),('Exchange loading wing',35,20,2,5),
  ('East workshop wing',51,26,2,3),('Depot eastern warehouse wing',58,34,3,4),
  ('West Side school back',9,38,3,3),('South landing inn rear',44,41,6,2),
 ]:
  for dy in range(h):
   row=['20']+['54']*(w-2)+['21'] if dy==0 else ['37']+['7d']*(w-2)+['7e'] if dy==h-1 else ['68']+['7f']*(w-2)+['69']
   for dx,bb in enumerate(row):point(name+' closed native wall '+str((dx,dy)),[x+dx,y+dy],bb)
 for name,b in [
  ('Fair growing beds attached hedge',[11,5,13,7]),('Fair west planted limit',[2,3,4,14]),
  ('Fair bee garden back',[16,10,18,12]),('Fair craft planted side',[12,14,14,16]),('Fair breeder growing shoulder',[15,14,18,16]),('Capitol east reading garden',[41,3,43,8]),
  ('Capitol bluff west plot',[29,14,35,16]),('Capitol bluff east plot',[37,14,41,16]),
  ('Summit press quiet shoulder',[50,13,52,16]),('Clinic west planted shoulder',[4,21,6,25]),
  ('Clinic east garden back',[9,21,11,23]),('Clinic community growing garden',[11,25,14,27]),('East community outer garden',[58,27,61,31]),('Market west green corner',[20,22,22,25]),
  ('Gym west shoulder',[47,19,49,22]),('Gym east shoulder',[52,19,54,22]),
  ('Depot passenger east shelter',[51,36,53,38]),('West Side herb garden back',[20,40,23,43]),
  ('South landing north bank pines',[50,41,52,44]),
 ]:rect(name,b,'forest')
 for name,b in [('Fair seeded display',[11,7,13,10]),('Fair bee flowers',[16,12,18,15]),('Capitol formal west flowerbed',[28,5,30,8]),('Capitol formal east flowerbed',[34,5,36,8]),('Clinic patient garden',[8,25,10,27]),('West Side growing garden',[20,43,23,45])]:rect(name,b,'flowers')
 rect('Fair shared seated food court',[5,9,10,13],'floor');point('Food court north bench',[6,11],'08');point('Food court east bench',[9,12],'08')
 rect('Capitol public court',[30,8,35,11],'floor');rect('Exchange compact trading court',[30,25,34,27],'floor');rect('Union Depot actual passenger forecourt',[43,35,51,38],'floor')
 poly('Mississippi enclosed working bends',[[3,30],[17,30],[17,31],[31,31],[31,33],[39,33],[39,36],[43,36],[43,39],[60,39],[60,44],[48,44],[48,42],[38,42],[38,40],[31,40],[31,38],[20,38],[20,37],[7,37],[7,35],[3,35]],'water')
 for name,x,y0,y1 in [('High Bridge working span',18,30,39),('Robert Street working span',41,35,44)]:
  block(name+' ordinary deck',[x,y0,x+1,y1],'31');block(name+' west rail',[x-1,y0,x,y1],'1b');block(name+' east rail',[x+1,y0,x+2,y1],'5f')
  path(name+' north landing',[[x,y0-1],[x,y0]],1);path(name+' south landing',[[x,y1-1],[x,y1]],1)
 block('Original Capitol low retaining bank',[29,13,44,14],'1c');point('Original actual down hop',[36,13],'07');block('Original ordinary brick return',[28,10,29,17],'05')
 block('Actual fisher north-bank rail',[30,31,37,32],'77');block('Depot river safety rail',[44,38,54,39],'77')
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
 old_buildings=[('Clinic',6,21,3,2,'clinic'),('Mart',22,22,3,2,'shop'),('Capitol',30,4,4,4,'commercial'),('Gym',49,19,3,2,'gym'),('Depot',43,32,8,3,'commercial')]
 for name,x,y,w,h,style in old_buildings:facade(name,x,y,w,h,style,3 if name=='Depot' else 1)
 # Active facades have a real room; no ornamental extra doors.
 specs=[
 ('FairFoodHall',5,7,2,1,'booth','Drinks for the midway.','food'),
 ('FairGardenStand',15,11,2,1,'booth','Seeds and berry stock.','garden'),
 ('FairAgricultureHall',17,4,5,3,'commercial','Crop and water exhibits.','agriculture'),
 ('FairExhibitionHall',4,14,6,3,'commercial','A voluntary show match.','battle0'),
 ('FairCraftHouse',10,14,2,2,'house','Handmade seed ribbons.','craft'),
 ('FairQuizPavilion',21,9,2,2,'house','Three fair questions.','quiz'),
 ('FairInfoHouse',5,3,2,2,'house','Public fair wayfinding.','info'),
 ('FairBreederBarn',18,14,4,3,'commercial','Care before competition.','battle1'),
 ('FairBerrySnack',8,9,2,1,'booth','Berries for the midway.','berrysnack'),
 ('FairFestivalMail',18,8,2,1,'booth','Send a fair postcard.','fairmail'),
 ('CapitolArchive',27,4,3,3,'commercial','Collect three archive stamps.','archive'),
 ('CapitolLibrary',37,3,4,5,'commercial','Reading room and field books.','library'),
 ('CivicMuseum',38,14,4,3,'commercial','Read three river measures.','river'),
 ('SummitMoveStudyHouse',52,4,2,2,'house','A Headbutt field note.','headbutt'),
 ('SummitExhibitionClub',55,9,4,3,'commercial','An optional club match.','battle4'),
 ('SummitPressHouse',48,13,2,2,'house','Local streets in print.','press'),
 ('WestSeventhBooks',13,20,3,3,'commercial','Secondhand field guides.','bookshop'),
 ('WestSeventhBakery',3,25,3,2,'shop','Picnic food for the river.','bakery'),
 ('StreetGardenHouse',14,27,2,2,'house','A pocket garden at the bend.','gardenhouse'),
 ('LowertownExchange',30,19,5,6,'commercial','Three floors of real destinations.','exchange'),
 ('ArchiveAnnex',26,27,4,3,'commercial','A walking archive stop.','annex'),
 ('LowertownCrafts',37,22,2,2,'house','Trade supplies and craftwork.','tools'),
 ('EastAidClinic',56,19,3,2,'clinic','Native free care and PC.','heal'),
 ('EastWorkshop',47,25,4,3,'commercial','An optional workshop match.','battle2'),
 ('CommunityHall',54,23,5,3,'commercial','Club notices and river walkers.','community'),
 ('TransitHeritageHouse',40,27,2,2,'house','Read actual depot notices.','transit'),
 ('FreightOffice',55,31,4,3,'commercial','Local freight records.','freight'),
 ('LowertownCafe',54,27,3,2,'shop','Rest before the river walk.','cafe'),
 ('AnglerHouse',5,42,2,2,'house','A river fishing challenge.','battle5'),
 ('RiversideSupplies',7,38,3,2,'shop','River-walk supplies.','riverstock'),
 ('WestsideCommunity',12,39,5,3,'commercial','A neighborhood gathering room.','westcommunity'),
 ('WestsideArena',25,41,4,3,'commercial','A voluntary training bout.','battle6'),
 ('BoatCoop',33,43,2,2,'house','Boat care on land.','boats'),
 ('SouthLandingInn',45,44,3,2,'shop','Native party rest.','rest'),
 ('SouthLandingSurvey',53,43,4,3,'commercial','A survey team exhibition.','battle7'),
 ]
 contract=dict(schema_version=1,city='MnStPaul',native_module_sha256=plan['native_city_sha256'],old_dimensions_metatiles=[32,24],dimensions_metatiles=[64,48],translation_core=[32,24],arrival_core=[6,48],clinic_constant='MN_ST_PAUL_POKECENTER_1F',default_mart='MART_GOLDENROD_2F_1',anchor_overrides=[],connections=[dict(direction='west',target='MnR04C00R00',old_offset=3,offset=15,town_edge=[[0,48],[0,49]],route_edge=[[55,18],[55,19]])],spawn_overrides=[dict(identifier=271,before=[6,24],after=[6,48])],residents=[],signs=[],doors=[],rooms=[],event_flags=[],programs={},trainers=[],items=[],text_revisions=[])
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
   for suffix,lines in [('Done',['Our work is saved.','Enjoy the city.']),('Need',['Visit the archive,','museum and annex.'] if role=='archive' else ['Read three river','measurement posts.']),('Full',['Your PACK is full.','Return with room.'])]:p[prefix+suffix]=[c('writetext',prefix+suffix+'Text'),c('waitbutton'),c('closetext'),c('end')];p[prefix+suffix+'Text']=text(lines)
   p[prefix+'RewardText']=text(['MAGNET received!','For your records.'] if role=='archive' else ['MYSTIC WATER gift.','For your research.'] if role=='river' else ['A field keepsake!','Enjoy the fair.'])
   # Quiz uses acknowledged instructional records rather than fake yes/no UI.
   prompts={'archive':[['Early rail roads','ran by the river.'],['The CAPITOL stands','on the high bluff.'],['Two banks share','one public city.']], 'river':[['Measure north-bank','water at the rail.'],['The bridge spans','both river banks.'],['Read high marks','from a safe path.']], 'quiz':[['Which berry wakes?','MINT BERRY does.'],['Which ball fits','anglers? LURE BALL.'],['Which party move','can cross water?','SURF, when earned.']]}[role]
   for j,(f,lines) in enumerate(zip(needs,prompts)):
    aid=prefix+'_RECORD_'+str(j+1);ra=actor(aid.removeprefix('MN_SPX_'),[3+4*j,9] if len(rows)==6 else [5,5],lines)
    if len(rows)==4:ra['tile']=[2+j,5]
    if role=='archive' and j==0:ra['tile']=[4,9];ra['approach']=[4,8]
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
 for suffix,lines in [('Wrong',['Use the field book','and visit again.']),('Full',['Your PACK is full.','Return with room.']),('Done',['All answers right!','Enjoy the fair.']),('Reward',['All answers right!','SILVERPOWDER gift.'])]:
  qp[q+suffix+'Text']=text(lines)
  if suffix!='Reward':qp[q+suffix]=[c('writetext',q+suffix+'Text'),c('waitbutton'),c('closetext'),c('end')]
 # Archive discoveries are distributed among three actual entered landmarks.
 ar=next(r for r in contract['rooms'] if r['id']=='MnStPaulCapitolArchive')
 for j,target in [(2,'MnStPaulCivicMuseum'),(3,'MnStPaulArchiveAnnex')]:
  ac=next(v for v in ar['residents'] if v['id']=='MN_SPX_ARCHIVE_RECORD_'+str(j));ar['residents'].remove(ac);ac['tile']=[11,9];dest=next(r for r in contract['rooms'] if r['id']==target);dest['residents'].append(ac)
  for lab in [ac['id'],ac['id']+'Text']:dest['programs'][lab]=ar['programs'].pop(lab)
 # River measurements are three real outdoor signs: both bridge landings and
 # the old fisher's safe waterfront. The host/reward remains in the museum.
 rr=next(r for r in contract['rooms'] if r['id']=='MnStPaulCivicMuseum')
 for j,at in [(1,[33,59]),(2,[87,87]),(3,[67,61])]:
  ac=next(v for v in rr['residents'] if v['id']=='MN_SPX_RIVER_RECORD_'+str(j));rr['residents'].remove(ac)
  contract['signs'].append(dict(id=ac['id'],tile=at,lines=ac['lines']))
  for lab in [ac['id'],ac['id']+'Text']:contract['programs'][lab]=rr['programs'].pop(lab)
  # Background programs have no actor to turn; all other commands preserved.
  contract['programs'][ac['id']]=[v for v in contract['programs'][ac['id']] if v['command']!='faceplayer']
  point(ac['id']+' actual meter sign',[at[0]//2,at[1]//2],'79')
 # Visible ordinary neighborhood residents and useful river circuit records.
 exterior_actors=[('MIDWAY_VISITOR',[23,23],['Food, flowers and','matches are inside.']),('BRIDGE_WALKER',[37,78],['Both bridges cross','to real streets.']),('EAST_HARBOR_WALKER',[83,88],['A safe river walk','ends at the inn.']),('SUMMIT_READER',[103,25],['The city archive','has three records','to find on foot.'])]
 for name,p,lines in exterior_actors:contract['residents'].append(actor(name,p,lines))
 # Small optional discoveries in meaningful side courts, not mandatory paving.
 for j,(p,item) in enumerate([([7,35],'SUPER_POTION'),([45,31],'BERRY_JUICE'),([89,11],'AWAKENING'),([121,33],'ETHER'),([33,57],'LURE_BALL'),([45,91],'PP_UP'),([73,89],'SUPER_REPEL'),([119,91],'MAX_ETHER')],1):contract['items'].append(dict(id='MN_SPX_ITEM_'+str(j),item=item,quantity=1,flag=flag('ITEM_'+str(j)),tile=p))
 # Exterior sign collision belongs to the sign, with ordinary reachable shoulder.
 pass # Signs authored after pedestrian paths below.

 path('Summit fitted west block street',[[42,10],[42,12],[48,12],[48,10]],1)
 path('Workshop west fitted approach',[[42,24],[42,29],[48,29]],1)
 path('Market workshop south working alley',[[31,30],[38,30],[38,29]],1)
 path('Summit offset apartment street',[[48,6],[49,6],[49,7],[54,7],[54,8],[58,8]],1)
 # Indexed old actors and signs keep ordinary connected approaches.
 for name,pts,_ in [('Original clinic gardener loop',[[6,23],[9,23],[9,24],[8,24],[8,25],[10,25]],1),('Original civic guide walk',[[31,8],[33,8],[33,10]],1),('Original bluff walker approach',[[28,12],[30,12]],1),('Original street reader lane',[[24,26],[26,26]],1),('Original river fisher walk',[[31,29],[34,29],[34,30],[31,30]],1),('Actual Depot passenger loop',[[46,35],[49,35],[49,37],[47,37],[47,36]],1)]:path(name,pts,1)
 for a in contract['signs']:
  pp=a['tile'];point(a['id']+' final native record sign',[pp[0]//2,pp[1]//2],'79')
 for a in plan['native_bg_records']:
  pp=a['new']['global_core'];point(a['id']+' relocated sign',[pp[0]//2,pp[1]//2],'79')
 for name,bb in [('North outer wall',[0,0,64,1]),('South outer wall',[0,47,64,48]),('West outer wall',[0,0,1,48]),('East outer wall',[63,0,64,48])]:block(name,bb,'13')
 point('Actual two-lane R04 seam',[0,24],'31');point('Closed north annex visible outer stone cap',[14,0],'13');point('Original closed guard plane',[14,1],'31')
 # Ordinary source-only PC functionality must be native-bound separately; old
 # clinic PC remains exact. No fake exterior doors or unimplemented train UI.
 dossier=copy.deepcopy(old);dossier.update(id='MnStPaul',dimensions_metatiles=[64,48],compiled_tileset='kanto',status='SOURCE_ONLY_HAND_AUTHORED_CITY_EXPANSION; pending native/controller/visual acceptance',expansion_contract='planning/minnesota/towns/MnStPaul.tight.expansion.contract.json',expansion_plan='planning/minnesota/towns/MnStPaul.expansion-plan.json',new_buildings=building_records,new_activities=dict(optional_trainers=8,shops=len(stocks)+2,acknowledged_record_errands=['archive','river'],native_yes_no_quiz=True,source_only=True,rail_travel_added=False),new_rooms=contract['rooms'],source_review=dict(native_gameplay_proved=False,reference_cities=['GoldenrodCity','SaffronCity'],old_room_modules_preserved=True))

 # Full candidate dossier coordinates agree with the executable overrides.
 for d in dossier['doors']:
  move=next(v for v in plan['doors'] if v['id']==d['id']);d['tile']=move['new_active']['global_core'];d['return_tile']=move['new_passive']['global_core'];d['arrival_tile']=d['return_tile']
 for key,source in [('residents','native_objects'),('signs','native_bg_records')]:
  for v in dossier[key]:
   move=next(q for q in plan[source] if q['id']==v['id']);v['tile']=move['new']['global_core']
   revised=next((q for q in contract['text_revisions'] if q['label']==v['id']+'Text'),None)
   if revised:v['lines']=revised['after']
 dossier['one_way_ledges']=[dict(id='CAPITOL_BLUFF_DROP',bounds_core=[72,26,74,28],direction='down',approach=[73,25],landing=[73,28],ordinary_return='west native brick return28,10..17; native05 remains ordinary walking ladder collision')]
 dossier['composition']='Compact fair booth lanes, Capitol low bluff, three-floor Lowertown Exchange, dense Summit blocks, two guarded river crossings and actual unchanged Union Depot story interior.'
 dossier['fairgrounds_port'].update(gate_metatile=[14,1],landing_metatile=[14,3],edge_core=[[28,0],[29,0]],closed_gate_core=[[28,2],[29,2]],status='Northern annex remains unbound and closed by original guard pair; public fair is playable within city source proposal.')
 dossier['depot_binding'].update(door_core=[92,69],safe_return_core=[92,70],contract='Native original courier/manifest/entire room and all indices exact; only city exterior door is relocated. No rail travel added.')
 dossier['viewpoints']=[dict(id='WEST_ARRIVAL',tile=[6,48]),dict(id='FAIR_FOOD_COURT',tile=[17,23]),dict(id='CAPITOL_FRONT',tile=[62,16]),dict(id='BLUFF_DROP',tile=[73,25]),dict(id='GYM_FRONT',tile=[102,42]),dict(id='DEPOT_FRONT',tile=[92,70]),dict(id='EXCHANGE_FRONT',tile=[62,50])]
 dossier['ports'][0].update(gate_metatile=[1,24],landing_metatile=[3,24])
 dossier['source_review']['limits']=['Source-only, no native/controller acceptance','Every original room/program is an unchanged native dependency','Foundational activity transactions require native roundtrip/controller tests','No rail travel or main quest added']
 # Move two voluntary exhibitions into visible public courts; retain exact flags/parties.
 for roomid,tile in [('MnStPaulFairExhibitionHall',[23,35]),('MnStPaulSouthLandingSurvey',[117,91])]:
  r=next(v for v in contract['rooms'] if v['id']==roomid)
  if r['trainers']:trainer=r['trainers'].pop();trainer['tile']=tile;contract['trainers'].append(trainer)
 dossier['new_rooms']=contract['rooms']
 write(OUT/'MnStPaul.tight.layout.json',l);write(OUT/'MnStPaul.tight.expansion.contract.json',contract);write(OUT/'MnStPaul.tight.candidate.json',dossier)
 print('Hand-authored',len(building_records),'new street facades,',len(contract['rooms']),'rooms;',len(layers),'explicit strokes')
if __name__=='__main__':main()
