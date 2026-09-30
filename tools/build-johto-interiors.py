"""Original full-volume domestic, care, retail and civic furnishings.

blender -b --python tools/build-johto-interiors.py -- target/johto-interiors [--skip-preview]
No source pixels, imported models or game data are used by this generator.
All dimensions are authored in meters; runtime fits only verified source drawings.
The saved blend preserves separately named editable mesh objects and materials.
"""
from pathlib import Path
source = Path(__file__).with_name('build-new-bark-models.py').read_text()
exec(compile(source.split('\nmake_house();')[0], 'original_geometric_vocabulary', 'exec'))

for name, color in {
 'enamel':(.80,.86,.83), 'enamel_light':(.95,.96,.87), 'metal':(.30,.39,.42),
 'blue':(.19,.43,.56), 'blue_light':(.39,.64,.71), 'red':(.66,.24,.24),
 'linen':(.96,.88,.72), 'linen_shadow':(.73,.64,.50), 'purple':(.46,.36,.58),
 'screen':(.11,.27,.29), 'screen_light':(.43,.83,.68), 'brass':(.75,.58,.29),
 'rubber':(.12,.17,.18), 'paper':(.98,.93,.78), 'soil':(.21,.16,.12)
}.items():
 m=bpy.data.materials.new(name);m.diffuse_color=(*color,1);m.use_nodes=True
 m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(*color,1)
 m.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.72
 MATS[name]=m

def ball(n,p,s,m):return ico(n,p,s,m,2)
def front(n,x,z,w,h,y,m,d=.025,b=.012):return facade_cube(n,x,z,w,h,y,d,m,bevel=b)
def legs(w,d,h,m='timber_light',r=.065):
 for x in [-w/2,w/2]:
  for y in [-d/2,d/2]:
   rod('Tapered furniture leg',(x,y,.045),(x*.93,y*.93,h),r,m,6)
   cone('Protective foot',(x,y,.045),r*.85,r*.85,.055,'rubber',8)
def knob(x,z,y=-.34):ball('Brass cabinet pull',(x,y,z),(.033,.025,.033),'brass')
def books(y,z,w=1.2,count=8):
 for i in range(count):
  x=-w/2+(i+.5)*w/count;h=.24+.08*((i*3)%4)/3
  cube('Bound reference volume',(x,y,z+h/2),(w/count*.78,.23,h),['red','blue','leaf','linen_shadow','purple'][i%5],.008)
  front('Book spine band',x,z+.065,w/count*.7,.018,y-.125,'brass',.009,.002)
def cabinet_body(w=1.35,d=.62,h=1.65,m='timber_light'):
 cube('Raised cabinet plinth',(0,0,.10),(w,d+.035,.19),'timber',.03)
 for x in [-w/2+.05,w/2-.05]:cube('Cabinet side stile',(x,0,h/2+.1),(.10,d,h),m,.014)
 cube('Cabinet closed rear',(0,d/2-.04,h/2+.1),(w-.12,.08,h),'timber',.008)
 cube('Overhanging cornice',(0,0,h+.13),(w+.06,d+.09,.12),m,.025)
def stool(name='stool',metal=False):
 begin(name);mat='metal' if metal else 'timber_light';legs(.60,.58,.55,mat,.045)
 for y in [-.26,.26]:rod('Low stretcher',(-.27,y,.23),(.27,y,.23),.025,mat)
 cone('Round padded seat',(0,0,.61),.43,.43,.13,'red' if metal else 'timber_gold',12)
 cone('Seat top piping',(0,0,.68),.40,.395,.035,'coral' if metal else 'linen',12)
def chair(name='chair',color='blue',arm=False):
 begin(name);legs(.62,.55,.58,'metal' if arm else 'timber_light',.046)
 cube('Chair seat frame',(0,0,.55),(.82,.73,.13),'metal' if arm else 'timber',.035)
 cube('Upholstered seat',(0,-.025,.65),(.77,.70,.16),color,.065)
 for x in [-.34,.34]:cube('Backrest upright',(x,.29,.97),(.06,.065,.78),'metal' if arm else 'timber_light',.012)
 cube('Padded curved backrest',(0,.285,1.10),(.78,.15,.50),color,.065)
 front('Back cushion stitched seam',0,1.10,.64,.022,.199,'blue_light' if color=='blue' else 'linen_shadow',.009,.004)
 if arm:
  for x in [-.45,.45]:
   rod('Armrest support',(x,.10,.60),(x,.10,.90),.028,'metal')
   cube('Rounded armrest',(x,-.025,.92),(.10,.59,.07),'rubber',.027)
def cushion():
 begin('cushion');cube('Woven square cushion',(0,0,.12),(.91,.91,.23),'purple',.10)
 for x in [-.40,.40]:cube('Cushion stitched piping',(x,0,.165),(.014,.73,.016),'linen_shadow',.004)
 for y in [-.40,.40]:cube('Cushion stitched piping',(0,y,.165),(.73,.014,.016),'linen_shadow',.004)
 ball('Center cushion button',(0,0,.234),(.045,.045,.009),'linen_shadow')
def table(name='dining_table',low=False,round_top=False):
 begin(name);h=.47 if low else .90;w=1.65;d=1.40
 if round_top:
  cone('Flared pedestal foot',(0,0,.08),.46,.24,.15,'metal',12)
  cone('Turned central pedestal',(0,0,h*.49),.09,.07,h*.85,'metal',10)
  cone('Round cafe tabletop',(0,0,h),.88,.88,.12,'linen',20)
  cone('Round table edge',(0,0,h-.035),.895,.895,.065,'timber_light',20)
 else:
  legs(w-.28,d-.28,h-.08,r=.06)
  for y in [-d/2+.09,d/2-.09]:cube('Mortised long table apron',(0,y,h-.15),(w-.15,.07,.19),'timber_light',.008)
  for x in [-w/2+.09,w/2-.09]:cube('Mortised short table apron',(x,0,h-.15),(.07,d-.15,.19),'timber_light',.008)
  cube('Beveled table edge',(0,0,h),(w,d,.13),'timber_gold',.045)
  for i in range(5):cube('Individual tabletop plank',(-w/2+(i+.5)*w/5,0,h+.07),(w/5-.009,d-.05,.025),'timber_light' if i%2 else 'timber_gold',.005)
def bed(name,color):
 begin(name);legs(1.07,1.87,.33,r=.065)
 cube('Bed side rail',(0,0,.35),(1.25,2.07,.20),'timber_light',.035)
 cube('Layered mattress',(0,0,.55),(1.19,1.97,.27),'linen_shadow',.09)
 cube('Mattress piping',(0,0,.66),(1.20,1.98,.035),'linen',.015)
 cube('Folded duvet',(0,-.30,.72),(1.18,1.36,.21),color,.075)
 cube('Duvet folded cuff',(0,.30,.815),(1.18,.20,.075),'linen',.022)
 for x in [-.40,0,.40]:cube('Duvet woven stripe',(x,-.32,.826),(.03,1.16,.012),'linen_shadow',.003)
 cube('Soft pillow',(0,.73,.80),(.85,.40,.16),'linen',.072)
 for x in [-.58,.58]:cube('Headboard turned post',(x,.99,.65),(.10,.11,1.12),'timber_light',.035)
 cube('Arched headboard center',(0,.98,.97),(1.16,.095,.41),'timber_gold',.065)
 for x in [-.38,-.19,0,.19,.38]:cube('Headboard vertical slat',(x,.98,.70),(.035,.075,.36),'timber',.008)

def plant(name='potted_plant',long=False,flower=False):
 begin(name)
 if long:
  cube('Long planter box',(0,0,.25),(.72,1.65,.49),'plaster',.065)
  cube('Planter lip',(0,0,.46),(.81,1.74,.095),'plaster_light',.025)
  cube('Recessed potting soil',(0,0,.515),(.64,1.57,.035),'soil',.012)
  origins=[(0,-.48),(0,0),(0,.48)]
 else:
  cone('Tapered ceramic pot',(0,0,.27),.25,.36,.50,'roof_red',12)
  cone('Rolled planter lip',(0,0,.52),.37,.37,.10,'roof_red_light',12)
  cone('Dark potting soil',(0,0,.576),.32,.32,.018,'soil',12)
  origins=[(0,0)]
 for cx,cy in origins:
  rod('Living branching stem',(cx,cy,.5),(cx+.025,cy,1.35),.027,'leaf_dark')
  for i in range(9):
   a=i*2.4;z=.78+(i%3)*.23;r=.29 if long else .37
   p=(cx+r*math.cos(a),cy+r*math.sin(a),z)
   rod('Leaf stem',(cx,cy,z-.16),p,.015,'leaf_dark')
   o=ball('Sculpted pointed leaf',p,(.24,.105,.06),['leaf','leaf_light','leaf_gold'][i%3]);o.rotation_euler=(.15,.32,a)
   if flower and i%2==0:
    for j in range(5):
     aa=j*math.tau/5;ball('Flower petal',(p[0]+.06*math.cos(aa),p[1]+.06*math.sin(aa),z+.07),(.055,.043,.035),'pink' if i%3 else 'coral')
    ball('Flower pollen',(p[0],p[1],z+.085),(.032,.032,.025),'yellow')
def bookcase(name='bookcase',lab=False):
 begin(name);cabinet_body(m='enamel' if lab else 'timber_light')
 for z in [.20,.65,1.10,1.55]:cube('Shelving board',(0,-.025,z),(1.20,.62,.055),'enamel_light' if lab else 'timber_gold',.012)
 for z in [.24,.69,1.14]:books(-.16,z)
def cabinet(name='cabinet',drawers=False,clock=False):
 begin(name);cabinet_body(h=1.75 if clock else 1.15)
 if clock:
  front('Clock glazed body',0,1.29,1.06,.93,-.33,'timber_gold',.08,.05)
  o=cone('Clock cream dial',(0,-.392,1.42),.37,.37,.04,'linen',20);o.rotation_euler.x=math.pi/2
  for i in range(12):
   a=i*math.tau/12;front('Clock hour marker',.29*math.sin(a),1.42+.29*math.cos(a),.026,.041,-.42,'timber',.012,.003)
  rod('Clock hour hand',(0,-.45,1.42),(.13,-.45,1.54),.018,'timber')
  rod('Clock minute hand',(0,-.45,1.42),(-.06,-.45,1.68),.012,'timber')
  front('Pendulum recess',0,.57,.24,.49,-.334,'ink',.04)
  rod('Pendulum shaft',(0,-.38,.78),(.04,-.38,.42),.012,'brass');ball('Pendulum bob',(.04,-.38,.43),(.092,.03,.092),'brass')
 elif drawers:
  for z in [.32,.65,.98]:
   front('Inset drawer front',0,z,1.12,.28,-.334,'timber_gold',.07,.018)
   for x in [-.3,.3]:knob(x,z,-.387)
 else:
  for x in [-.29,.29]:
   front('Inset paneled door',x,.69,.53,1.02,-.335,'timber_gold',.07,.017)
   front('Door recessed panel',x,.69,.39,.83,-.377,'timber_light',.018)
   knob(x*.32,.67,-.403)

def display_screen(x=0,y=-.12,z=1.22,w=.9,h=.67):
 cube('Deep monitor shell',(x,y,z),(w,.38,h),'enamel',.065)
 front('Recessed screen bezel',x,z,w*.86,h*.77,y-.21,'rubber',.035,.025)
 front('Curved teal glass screen',x,z,w*.75,h*.65,y-.235,'screen',.023,.032)
 for i in range(3):front('Screen status line',x-w*.13,z+h*.16-i*h*.14,w*(.38-i*.07),h*.025,y-.249,'screen_light',.009,.002)
 for dx in [-w*.26,w*.26]:front('Display lower control',x+dx,z-h*.37,.035,.025,y-.23,'brass',.008,.004)
def keyboard(x=0,y=-.33,z=.68,w=.92):
 cube('Keyboard beveled tray',(x,y,z),(w,.37,.065),'enamel',.022)
 for r in range(3):
  for c in range(10):cube('Raised keyboard key',(x+(c-4.5)*w*.075,y+(r-1)*.081,z+.037),(w*.056,.051,.015),'enamel_light' if c%5 else 'blue',.003)
 cube('Keyboard spacebar',(x,y-.145,z+.04),(w*.40,.033,.017),'enamel_light',.003)
def computer(name='computer',lab=False):
 begin(name);legs(1.1,.62,.75,'metal' if lab else 'timber_light',.05)
 cube('Computer desk top',(0,0,.79),(1.38,.95,.12),'enamel' if lab else 'timber_gold',.035)
 cube('Monitor pedestal',(0,.11,.88),(.37,.29,.11),'metal',.025)
 display_screen(y=.10,z=1.27)
 keyboard(y=-.28,z=.88)
 cube('Computer tower',(.44,.20,.42),(.32,.55,.58),'enamel',.025)
 for zz in [.48,.55,.62]:front('Drive bay',.44,zz,.24,.025,-.085,'metal',.01,.003)
 front('Power indicator',.51,.32,.032,.032,-.085,'screen_light',.01,.004)
def television():
 begin('television');cabinet_body(w=1.1,d=.66,h=.53)
 display_screen(y=0,z=1.01,w=1.12,h=.78)
 for s in [-1,1]:rod('Television aerial',(0,.13,1.41),(s*.28,.13,1.75),.013,'metal')
 front('Speaker grille',0,.44,.82,.22,-.35,'timber',.03)
 for x in [-.30,-.15,0,.15,.30]:front('Speaker aperture',x,.44,.022,.16,-.37,'ink',.015,.003)
def radio():
 begin('radio');cube('Wood radio housing',(0,0,.40),(1.12,.53,.73),'timber_gold',.07)
 front('Cloth speaker grille',-.20,.41,.53,.51,-.28,'timber',.04,.027)
 for i in range(7):front('Woven speaker slat',-.43+i*.076,.41,.018,.42,-.309,'linen_shadow',.01,.003)
 front('Frequency dial',.32,.56,.25,.13,-.288,'screen',.03)
 front('Dial pointer',.32,.56,.015,.095,-.31,'coral',.008)
 for zz in [.23,.39]:knob(.32,zz,-.31)
 rod('Radio aerial',(.33,.13,.73),(.54,.14,1.10),.011,'metal')
def console():
 begin('game_console');cube('Game console body',(0,0,.18),(.85,.71,.31),'enamel',.04)
 cube('Cartridge slot',(0,.10,.349),(.50,.055,.022),'rubber',.006)
 cube('Inserted game cartridge',(0,.10,.43),(.43,.11,.19),'blue',.014)
 for x in [-.27,.27]:
  cube('Wired gamepad',(x,-.39,.10),(.35,.18,.15),'rubber',.043)
  cube('Gamepad direction cross',(x-.065,-.399,.18),(.09,.025,.018),'enamel',.005)
  cube('Gamepad direction cross',(x-.065,-.399,.18),(.025,.09,.018),'enamel',.005)
  for yy in [-.025,.025]:ball('Gamepad action button',(x+.085,-.399+yy,.18),(.022,.022,.012),'red')
  rod('Controller cable',(x,-.28,.11),(x,-.22,.17),.012,'rubber')
def refrigerator(name='refrigerator',retail=False):
 begin(name);cube('Enamel refrigerator carcass',(0,0,.91),(1.11,.80,1.80),'enamel',.07)
 for z,h in [(1.48,.53),(.65,1.03)]:
  front('Insulated refrigerator door',0,z,1.04,h,-.432,'blue_light' if retail else 'enamel_light',.12,.045)
  front('Recessed door seal',0,z,.91,h-.11,-.50,'screen' if retail else 'enamel',.018,.025)
  front('Brushed steel door handle',.37,z,.042,h*.60,-.54,'metal',.045,.015)
 if retail:
  for z in [.38,.72,1.12,1.52]:
   front('Glass cabinet shelf',-.05,z,.72,.025,-.515,'enamel_light',.018,.003)
   for x in [-.3,-.08,.14]:front('Visible chilled bottle',x,z+.105,.093,.17,-.521,['coral','yellow','blue'][round((x+.3)*10)%3],.019,.012)
 else:
  for x in [-.24,0,.24]:front('Refrigerator magnet',x,1.49,.075,.08,-.524,['coral','blue','yellow'][round((x+.24)*10)%3],.018,.011)

def kitchen(sink=False):
 begin('kitchen_sink' if sink else 'kitchen_stove');cube('Kitchen carcass',(0,0,.45),(1.30,.80,.88),'enamel',.033)
 cube('Rounded worktop',(0,0,.91),(1.36,.88,.09),'metal',.028)
 front('Inset appliance front',0,.44,1.15,.70,-.418,'enamel_light',.065,.025)
 if sink:
  cube('Recessed basin well',(0,-.015,.963),(.76,.53,.024),'ink',.06)
  cube('Basin interior',(0,-.025,.981),(.61,.38,.024),'metal',.055)
  rod('Chrome tap upright',(.29,.30,.93),(.29,.30,1.22),.035,'enamel_light')
  rod('Chrome tap spout',(.29,.30,1.22),(.29,.03,1.22),.033,'enamel_light')
  knob(0,.44,-.47)
 else:
  for x in [-.32,.32]:
   for y in [-.20,.20]:
    cone('Burner black ring',(x,y,.971),.17,.17,.025,'rubber',12)
    cone('Burner center',(x,y,.985),.11,.11,.015,'metal',10)
  front('Oven window',0,.42,.83,.36,-.461,'screen',.03,.028)
  front('Oven horizontal handle',0,.68,.91,.048,-.50,'metal',.05,.015)
  for x in [-.4,-.13,.13,.4]:knob(x,.80,-.47)

def shelf(name='retail_shelf',gift=False):
 begin(name);cabinet_body(w=1.35,d=.79,h=1.7,m='timber_light' if gift else 'enamel')
 for iz,z in enumerate([.25,.75,1.24]):
  cube('Merchandise shelf',(0,-.04,z),(1.23,.81,.075),'timber_gold' if gift else 'enamel_light',.018)
  for i in range(5):
   x=(i-2)*.23;m=['red','blue','leaf','yellow','purple'][(i+iz)%5]
   if (i+iz)%2:
    cube('Wrapped stock carton',(x,-.16,z+.20),(.18,.29,.31),m,.013)
    front('Carton cream label',x,z+.20,.11,.10,-.316,'linen',.008,.003)
   else:
    cone('Stock bottle',(x,-.16,z+.19),.075,.069,.29,m,8)
    cone('Bottle cream cap',(x,-.16,z+.35),.044,.044,.05,'linen',8)
  cube('Shelf price rail',(0,-.45,z+.025),(1.20,.05,.09),'blue' if not gift else 'timber',.012)
  for x in [-.4,0,.4]:front('Small shelf label',x,z+.022,.15,.037,-.48,'linen',.009,.003)

def counter(name='reception_counter',lab=False):
 begin(name);cube('Inset counter toe kick',(0,.03,.10),(1.77,.76,.19),'metal' if lab else 'timber',.028)
 cube('Solid service counter',(0,0,.55),(1.90,.92,.90),'enamel' if lab else 'plaster',.035)
 for x in [-.62,0,.62]:front('Counter recessed panel',x,.52,.53,.62,-.481,'blue' if lab else 'roof_red',.045,.025)
 cube('Polished service countertop',(0,0,1.035),(2.02,1.02,.15),'enamel_light',.045)
 cube('Counter privacy lip',(0,.42,1.15),(2.0,.08,.13),'metal' if lab else 'timber_light',.022)
def healing_machine(name='healing_machine'):
 begin(name);cube('Medical machine foot',(0,0,.10),(1.84,.82,.20),'metal',.04)
 cube('Rounded medical enclosure',(0,.04,.96),(1.75,.69,1.77),'enamel',.08)
 front('Scarlet medical header',0,1.64,1.65,.31,-.338,'red',.09,.04)
 front('Medical emblem horizontal',0,1.65,.27,.075,-.399,'enamel_light',.03)
 front('Medical emblem vertical',0,1.65,.075,.27,-.399,'enamel_light',.03)
 front('Dark central status display',0,1.22,.68,.39,-.35,'screen',.06,.03)
 for i in range(3):front('Medical readout band',0,1.29-i*.08,.44-i*.07,.023,-.391,'screen_light',.012,.003)
 cube('Ball treatment tray',(0,-.43,.75),(1.59,.59,.16),'red',.045)
 for x in [-.49,0,.49]:
  for y in [-.57,-.31]:
   cone('Recessed treatment socket',(x,y,.838),.14,.14,.027,'rubber',12)
   ball('Pale treatment capsule',(x,y,.881),(.087,.087,.072),'enamel_light')
 for x in [-.63,.63]:
  front('Machine vent surround',x,.36,.27,.29,-.335,'metal',.036)
  for z in [.27,.35,.43]:front('Machine vent slot',x,z,.19,.018,-.36,'rubber',.014,.003)
def healing_console():
 begin('healing_console');cube('Clinical console base',(0,0,.42),(1.18,.82,.82),'enamel',.065)
 cube('Scarlet console rim',(0,0,.87),(1.29,.91,.14),'red',.043)
 for x in [-.37,0,.37]:
  cone('Medical console socket',(x,.04,.956),.13,.13,.022,'metal',12)
  ball('Treatment light',(x,.04,.978),(.070,.070,.018),'screen_light')
 front('Console status screen',0,.57,.52,.20,-.447,'screen',.025)
 for x in [-.19,0,.19]:front('Medical status lamp',x,.58,.06,.08,-.47,'screen_light',.012,.012)

def partition(name='clinical_partition',wood=False):
 begin(name);mat='timber_light' if wood else 'metal'
 for y in [-.84,.84]:
  cube('Partition footing',(0,y,.055),(.38,.24,.11),mat,.022)
  cube('Partition upright',(0,y,.86),(.095,.095,1.64),mat,.018)
 cube('Partition lower panel',(0,0,.55),(.09,1.64,.94),'plaster' if wood else 'blue',.025)
 cube('Partition translucent upper',(0,0,1.33),(.075,1.64,.54),'linen' if wood else 'blue_light',.025)
 cube('Partition top rail',(0,0,1.65),(.115,1.79,.09),mat,.025)

def lab_machine():
 begin('lab_restoration_machine');cube('Research chassis',(0,0,.37),(1.73,1.11,.72),'enamel',.065)
 cube('Laboratory blue band',(0,0,.72),(1.82,1.18,.13),'blue',.035)
 for x in [-.53,.53]:
  cone('Specimen chamber socket',(x,0,.85),.29,.29,.16,'metal',16)
  cone('Tinted specimen chamber',(x,0,1.16),.245,.245,.49,'glass',16)
  cone('Chamber cap',(x,0,1.46),.29,.26,.10,'enamel_light',16)
  rod('Specimen chamber light',(x-.14,-.19,.96),(x-.14,-.19,1.33),.020,'glass_light')
 display_screen(y=.37,z=1.15,w=.47,h=.45)
 front('Research control panel',0,.41,.91,.28,-.58,'metal',.035)
 for i in range(5):knob((i-2)*.16,.42,-.622)
def lab_table():
 begin('lab_display_table');legs(2.10,1.02,.8,'metal',.047)
 cube('Research display worktop',(0,0,.85),(2.35,1.25,.15),'enamel',.038)
 for x in [-.74,0,.74]:
  cone('Specimen stand',(x,0,.95),.24,.21,.08,'blue',12)
  ball('Research specimen',(x,0,1.10),(.15,.15,.17),'enamel_light')
  cube('Specimen information plaque',(x,-.34,.958),(.37,.15,.024),'metal',.008)

def arcade(name='arcade_machine',paired=False):
 begin(name)
 for x in [-.55,.55] if paired else [0]:
  cube('Arcade kickplate',(x,0,.12),(.91,.71,.24),'rubber',.035)
  cube('Arcade tapered cabinet',(x,.035,.80),(.94,.75,1.35),'purple',.045)
  front('Recessed game screen bezel',x,1.24,.76,.62,-.37,'rubber',.05,.028)
  front('Emerald game display',x,1.25,.63,.49,-.407,'screen',.025,.021)
  for i in range(3):
   front('Slot reel pale panel',x+(i-1)*.185,1.25,.14,.32,-.426,'linen',.017,.008)
   front('Geometric reel symbol',x+(i-1)*.185,1.25,.08,.12,-.438,['red','blue','leaf'][i],.016,.015)
  cube('Angled control shelf',(x,-.34,.90),(.93,.40,.12),'blue',.028)
  rod('Joystick',(x-.24,-.39,.98),(x-.24,-.39,1.09),.020,'metal')
  ball('Joystick grip',(x-.24,-.39,1.10),(.047,.047,.047),'red')
  for dx in [.07,.23]:cone('Arcade action button',(x+dx,-.42,.979),.044,.044,.03,'yellow',10)
  front('Coin return',x,.47,.13,.18,-.367,'metal',.022)
  front('Coin slot',x,.65,.034,.14,-.37,'rubber',.024)
  front('Lit arcade crown',x,1.61,.87,.20,-.374,'yellow',.07,.025)
  for dx in [-.25,0,.25]:front('Arcade crown inlay',x+dx,1.61,.085,.095,-.42,'purple',.02,.02)
def station_bench():
 begin('station_bench')
 for x in [-.60,.60]:
  cube('Station seat support',(x,0,.31),(.09,.11,.58),'metal',.015)
  cube('Bench anchored foot',(x,0,.04),(.18,.72,.08),'metal',.016)
 for i in range(4):cube('Bench wooden seat slat',(0,-.31+i*.18,.62),(1.60,.14,.10),'timber_gold',.025)
 for x in [-.66,.66]:cube('Bench backrest frame',(x,.31,.87),(.07,.07,1.10),'metal',.014)
 for z in [.91,1.11,1.31]:cube('Bench rounded back slat',(0,.31,z),(1.61,.095,.15),'timber_light',.030)
 for x in [-.79,.79]:cube('Bench armrest',(x,-.02,.89),(.075,.65,.065),'metal',.025)
def turnstile():
 begin('station_turnstile')
 for x in [-.52,.52]:
  cube('Turnstile steel pedestal',(x,0,.49),(.31,.95,.94),'metal',.055)
  cube('Turnstile enamel top',(x,0,.98),(.34,.98,.12),'enamel',.034)
  cube('Ticket reader',(x,-.28,1.06),(.20,.23,.07),'blue',.017)
  cube('Ticket reader slot',(x,-.28,1.105),(.14,.035,.015),'rubber',.003)
  front('Turnstile direction lamp',x,.68,.13,.13,-.50,'screen_light',.025,.04)
 for s in [-1,1]:rod('Turnstile rotating arm',(s*.38,0,.69),(s*.045,-.15,.69),.040,'enamel_light')
def terminal():
 begin('gate_terminal');cube('Information terminal foot',(0,0,.07),(.80,.68,.14),'metal',.04)
 cube('Information terminal column',(0,.10,.54),(.36,.33,.91),'blue',.045)
 display_screen(y=0,z=1.17,w=.86,h=.74)
 cube('Terminal control shelf',(0,-.30,.82),(.73,.40,.09),'enamel',.022)
 for x in [-.22,0,.22]:cone('Terminal large key',(x,-.31,.88),.06,.06,.035,'blue',10)
def broadcast():
 begin('broadcast_console');cube('Broadcast desk base',(0,0,.42),(1.62,.85,.80),'metal',.04)
 cube('Audio mixer top',(0,0,.87),(1.78,1.03,.12),'blue',.04)
 for x in [-.58,-.29,0,.29,.58]:
  cube('Mixer fader track',(x,-.13,.939),(.021,.42,.009),'rubber',.002)
  cube('Mixer fader cap',(x,-.25+(x+.58)*.2,.958),(.09,.064,.039),'enamel_light',.009)
  for y in [.18,.32]:cone('Mixer rotary control',(x,y,.969),.038,.034,.065,'metal',10)
 display_screen(x=.38,y=.32,z=1.20,w=.61,h=.43)
 rod('Broadcast microphone stand',(-.54,.27,.93),(-.54,.27,1.33),.018,'metal')
 rod('Microphone boom',(-.54,.27,1.33),(-.37,-.02,1.45),.017,'metal')
 ball('Broadcast microphone',(-.35,-.06,1.45),(.055,.11,.058),'rubber')
def vending():
 begin('vending_machine');cube('Vending cabinet',(0,0,1.02),(1.22,.79,2.0),'blue',.065)
 front('Vending glass bay',-.14,1.26,.80,1.20,-.425,'screen',.06,.035)
 for z in [.82,1.22,1.62]:
  for x in [-.38,-.12,.13]:
   front('Chilled beverage body',x,z,.13,.22,-.466,['coral','yellow','leaf'][int((x+.4)*10)%3],.022,.024)
   front('Drink can label',x,z,.10,.055,-.482,'linen',.009,.003)
  front('Vending product rail',-.13,z-.14,.71,.032,-.47,'enamel',.02)
 for z in [1.2,1.36,1.52]:knob(.45,z,-.45)
 front('Vending collection hatch',0,.32,.78,.27,-.43,'rubber',.07,.024)
 front('Collection flap highlight',0,.40,.64,.02,-.48,'metal',.016)

def wall(name='wall_panel',windowed=False,doorway=False,radio_inset=False):
 begin(name)
 if doorway:
  for x in [-.88,.88]:cube('Door jamb',(x,0,.92),(.18,.24,1.84),'plaster',.025)
  cube('Door lintel',(0,0,1.83),(1.93,.28,.16),'plaster_light',.027)
  for x in [-.48,.48]:
   cube('Open sliding door leaf',(x*1.7,.075,.88),(.28,.075,1.60),'blue',.02)
   front('Door leaf glass',x*1.7,1.13,.20,.76,-.012,'glass',.028,.012)
  cube('Recessed threshold',(0,0,.022),(1.78,.29,.044),'metal',.012)
 else:
  cube('Full-volume plaster panel',(0,0,.87),(1.95,.18,1.74),'plaster_light',.013)
  cube('Wall timber baseboard',(0,-.11,.09),(1.95,.06,.15),'timber_light',.008)
  cube('Wall upper crown',(0,-.025,1.73),(2.0,.25,.095),'timber_light',.012)
  if windowed:window(0,1.0,-.105,w=1.38,h=.86)
  if radio_inset:
   front('Built-in wall radio cabinet',0,.79,1.21,.62,-.18,'timber_gold',.16,.034)
   front('Built-in radio cloth',-.19,.79,.57,.44,-.281,'timber',.035,.02)
   for zz in [.68,.88]:knob(.39,zz,-.30)
def open_book():
 begin('open_book');cube('Reading lectern base',(0,0,.09),(.80,.66,.18),'timber_light',.024)
 for s in [-1,1]:
  o=cube('Open bound cover',(s*.23,0,.22),(.48,.69,.049),'red',.012);o.rotation_euler.y=s*.14
  o=cube('Thick cream page block',(s*.225,-.007,.264),(.43,.64,.045),'paper',.01);o.rotation_euler.y=s*.14
  for y in [-.20,-.08,.04,.16]:cube('Printed page line',(s*.24,y,.300),(.29,.012,.006),'linen_shadow',.002)
 cube('Book fabric bookmark',(.14,-.26,.304),(.036,.32,.009),'blue',.002)
def flower_stand():
 plant('flower_stand',flower=True)
 # Lift the ceramic arrangement onto an open timber stand, keeping modeled legs visible.
 for o in COL.objects:o.location.z+=.40
 legs(.47,.47,.40,r=.035);cube('Plant stand top',(0,0,.41),(.73,.72,.07),'timber_light',.023)
def memorial():
 begin('memorial');cube('Memorial stone footing',(0,0,.075),(.78,.53,.15),'stone',.025)
 cube('Memorial carved stone',(0,.05,.52),(.62,.19,.83),'stone_light',.06)
 front('Inset memorial plaque',0,.56,.42,.37,-.066,'stone_dark',.028,.028)
 for z in [.46,.56,.65]:front('Carved memorial line',0,z,.27,.018,-.086,'stone_light',.013,.003)

stool();stool('arcade_stool',True);chair();chair('link_seat','blue',True);cushion()
table();table('low_table',True);table('cafe_table',round_top=True)
for name,color in [('bed_red','red'),('bed_blue','blue'),('bed_green','leaf'),('bed_pink','pink')]:bed(name,color)
plant();plant('long_planter',True);flower_stand();bookcase();bookcase('lab_bookcase',True)
cabinet();cabinet('drawer_cabinet',True);cabinet('pendulum_clock',clock=True)
computer();computer('lab_workstation',True);television();radio();console()
begin('keyboard');keyboard(z=.06)
refrigerator();refrigerator('retail_refrigerator',True);kitchen();kitchen(True)
shelf();shelf('gift_shelf',True);counter();counter('lab_counter',True)
healing_machine();healing_console();partition();partition('timber_partition',True)
lab_machine();lab_table();arcade();arcade('arcade_pair',True);station_bench();turnstile();terminal();broadcast();vending()
wall();wall('window_wall',True);wall('gate_door_frame',doorway=True);wall('radio_wall',radio_inset=True)
open_book();memorial()

# Second-pass art: source-distinct bedroom designs and architectural details.
def decorated_bed(name,style):
 bed(name, 'pink' if style=='pink' else 'linen' if style=='feathery' else 'yellow' if style=='pikachu' else 'blue')
 if style=='feathery':
  for x in [-.57,.57]:
   for y in [-.85,-.62,-.39,-.16,.07]:
    ball('Feathery duvet tuft',(x,y,.71),(.075,.07,.045),'linen')
  for x in [-.34,0,.34]:
   for y in [-.69,-.35,.01]:ball('Quilt tuft',(x,y,.833),(.035,.035,.012),'linen_shadow')
 elif style=='polkadot':
  for x in [-.40,0,.40]:
   for y in [-.72,-.32,.08]:cone('Woven polkadot',(x,y,.835),.085,.085,.012,'linen',12)
 elif style=='pikachu':
  # Original geometric woven mascot face, not a sampled game sprite.
  ball('Embroidered yellow face',(0,-.32,.84),(.38,.32,.018),'yellow')
  for s in [-1,1]:
   o=cube('Embroidered pointed ear',(s*.25,.045,.841),(.10,.34,.016),'yellow',.038);o.rotation_euler.z=s*-.28
   o=cube('Dark ear tip',(s*.29,.19,.852),(.105,.13,.012),'timber',.034);o.rotation_euler.z=s*-.28
   ball('Embroidered round eye',(s*.13,-.25,.863),(.035,.045,.011),'ink')
   ball('Embroidered rosy cheek',(s*.25,-.39,.863),(.064,.049,.011),'red')
  ball('Embroidered nose',(0,-.35,.865),(.027,.022,.010),'ink')
  for s in [-1,1]:rod('Embroidered smile',(0,-.43,.866),(s*.076,-.46,.866),.010,'timber')

def ornamental_plant(name,style):
 begin(name)
 cone('Glazed decorative plant pot',(0,0,.30),.27,.36,.56,'blue' if style=='tropic' else 'roof_red',12)
 cone('Rolled decorative pot lip',(0,0,.59),.385,.385,.10,'blue_light' if style=='tropic' else 'roof_red_light',12)
 cone('Recessed rich soil',(0,0,.65),.33,.33,.018,'soil',12)
 if style=='magna':
  for i in range(11):
   a=i*2.4;z=.89+(i%3)*.24
   p=(.36*math.cos(a),.31*math.sin(a),z)
   rod('Magnaplant fork',(0,0,.61),p,.027,'leaf_dark')
   o=ball('Magnaplant broad leaf',p,(.25,.14,.062),'leaf_light' if i%2 else 'leaf');o.rotation_euler.z=a
 elif style=='tropic':
  rod('Tropicplant slender trunk',(0,0,.6),(0,0,1.39),.05,'timber_light')
  for i in range(8):
   a=i*math.tau/8
   for j in range(4):
    r=.10+j*.15;z=1.48-.10*j
    p=(r*math.cos(a),r*math.sin(a),z)
    o=ball('Long drooping tropical frond',p,(.23,.10,.047),'leaf_light' if i%2 else 'leaf');o.rotation_euler.z=a;o.rotation_euler.y=.22
 else:
  cone('Jumboplant woody trunk',(0,0,.98),.078,.055,.72,'timber_light',8)
  for i,(r,z) in enumerate([(.51,1.11),(.43,1.43),(.31,1.72)]):
   for j in range(7):
    a=j*math.tau/7+i*.6
    ball('Dense tiered broadleaf crown',(r*.54*math.cos(a),r*.54*math.sin(a),z),(r*.65,r*.59,.25),'leaf_light' if (i+j)%3 else 'leaf')

def architectural_panel(name,style):
 begin(name)
 modern=style=='clinical';timber=style=='traditional'
 cube('Solid architectural backing',(0,.06,1.0),(2.0,.15,2.0),'enamel_light' if modern else 'plaster_light',.015)
 for z in [.08,1.96]:cube('Continuous wall molding',(0,-.04,z),(2.03,.24,.12),'metal' if modern else 'timber_light',.012)
 if style=='domestic':
  for x in [-.82,-.41,0,.41,.82]:
   cube('Wallpaper raised vertical weave',(x,-.026,1.06),(.029,.022,1.66),'linen_shadow',.004)
  cube('Domestic dado rail',(0,-.06,.55),(2.0,.11,.07),'timber_gold',.009)
  for x in [-.73,-.24,.24,.73]:front('Wainscot inset',x,.30,.42,.32,-.047,'plaster',.028,.013)
 elif timber:
  for x in [-.94,0,.94]:cube('Exposed timber wall post',(x,-.045,1.01),(.11,.21,1.88),'timber_light',.014)
  cube('Traditional horizontal timber',(0,-.04,.62),(2.0,.18,.12),'timber_light',.015)
 elif modern:
  cube('Clinical wall dado',(0,-.025,.41),(1.98,.025,.62),'blue_light',.007)
  cube('Clinical aluminum chair rail',(0,-.065,.74),(2.0,.067,.075),'metal',.009)
 elif style=='acoustic':
  for x in [-.74,-.25,.25,.74]:
   for z in [.4,.9,1.4]:
    cube('Acoustic wall tile',(x,-.065,z),(.45,.13,.44),'linen_shadow',.028)
    for dx in [-.10,.1]:front('Acoustic perforation',x+dx,z,.034,.20,-.142,'timber',.012,.01)

def picture_frame():
 begin('picture_frame')
 cube('Picture backing board',(0,.025,.74),(1.54,.10,1.49),'timber',.022)
 # Recessed empty aperture receives the exact live source artwork at runtime.
 for x in [-.78,.78]:cube('Mitered frame side',(x,-.052,.75),(.105,.16,1.57),'timber_gold',.025)
 for z in [.02,1.49]:cube('Mitered frame top or bottom',(0,-.052,z),(1.64,.16,.105),'timber_gold',.025)
 for x in [-.708,.708]:cube('Frame inner brass trim',(x,-.095,.75),(.02,.025,1.41),'brass',.004)
 for z in [.071,1.429]:cube('Frame inner brass trim',(0,-.095,z),(1.435,.025,.02),'brass',.004)

def stair_flight():
 begin('stair_flight')
 # Ascending +X flight. Closed tread/riser shells with open central walkway.
 for i in range(8):
  x=-.875+i*.25;h=(i+1)*.125
  cube('Solid individual stair tread',(x,0,h/2),(.25,1.29,h),'stone',.008)
  cube('Light stair nosing',(x-.096,0,h+.013),(.045,1.32,.027),'stone_light',.006)
  for y in [-.66,.66]:cube('Carved stair side stringer',(x,y,h/2),(.25,.095,h+.12),'plaster',.014)
 for y in [-.69,.69]:
  for x in [-.79,-.25,.29,.83]:
   z=(x+1)/2+.48
   rod('Stair rail upright',(x,y,max(.08,z-.44)),(x,y,z),.028,'metal')
  rod('Continuous polished handrail',(-.95,y,.45),(.96,y,1.40),.037,'timber_light')

def office_phone():
 begin('office_phone');cube('Angled office telephone base',(0,0,.14),(.76,.55,.25),'blue',.055)
 for x in [-.25,.25]:cube('Telephone cradle',(x,.10,.29),(.11,.21,.14),'metal',.023)
 cube('Telephone handset',(0,.10,.40),(.73,.15,.13),'rubber',.045)
 for x in [-.28,.28]:ball('Telephone earpiece',(x,.10,.37),(.13,.13,.08),'rubber')
 for i in range(3):
  for j in range(3):cube('Telephone dial key',((j-1)*.10,-.13+(i-1)*.076,.283),(.065,.049,.022),'enamel',.009)

decorated_bed('bed_feathery','feathery');decorated_bed('bed_polkadot','polkadot');decorated_bed('bed_pikachu','pikachu')
for style in ['magna','tropic','jumbo']:ornamental_plant('plant_'+style,style)
for style in ['domestic','traditional','clinical','acoustic']:architectural_panel('wall_'+style,style)
picture_frame();stair_flight();office_phone()

# Individually authored civic equipment refinements.
def broadcast_rack():
 begin('broadcast_rack')
 cube('Broadcast equipment toe kick',(0,0,.10),(1.13,.73,.20),'rubber',.027)
 for x in [-.53,.53]:cube('Rack aluminum upright',(x,0,1.0),(.10,.69,1.82),'metal',.016)
 cube('Rack closed back',(0,.29,1.0),(.98,.09,1.75),'metal',.013)
 for z in [.27,.68,1.09,1.50,1.91]:cube('Individual rack mounting rail',(0,-.02,z),(1.15,.70,.070),'enamel',.012)
 for z in [.45,.87,1.29,1.72]:
  front('Rack instrument fascia',0,z,1.01,.29,-.355,'rubber',.066,.013)
  for x in [-.44,.44]:
   rod('Equipment pull handle',(x,-.417,z-.067),(x,-.417,z+.067),.019,'metal')
  if z>1.5:
   front('Oscilloscope glass',-.18,z,.47,.20,-.394,'screen',.026,.012)
   for i in range(4):front('Meter trace',-.34+i*.095,z+(.035 if i%2 else -.017),.06,.012,-.413,'screen_light',.013,.002)
   for xx in [.20,.34]:knob(xx,z,-.406)
  elif z>1.0:
   for xx in [-.25,.25]:
    o=cone('Tape transport reel',(xx,-.405,z),.103,.103,.027,'enamel',12);o.rotation_euler.x=math.pi/2
    for j in range(3):
     a=j*math.tau/3;front('Reel opening',xx+.055*math.sin(a),z+.055*math.cos(a),.022,.029,-.428,'rubber',.009,.005)
   front('Tape transport status',0,z,.11,.055,-.411,'coral',.018,.007)
  else:
   for i in range(6):
    xx=-.31+i*.12;front('Rack ventilation slit',xx,z,.023,.15,-.395,'metal',.012,.004)
   front('Rack power light',.39,z,.028,.034,-.412,'screen_light',.009,.009)

def tower_reception():
 counter('tower_reception',True)
 display_screen(x=.49,y=.21,z=1.35,w=.57,h=.43)
 cube('Reception monitor pedestal',(.49,.24,1.14),(.20,.23,.085),'metal',.016)
 keyboard(x=.39,y=-.10,z=1.14,w=.68)
 cube('Visitor writing pad',(-.53,-.15,1.117),(.48,.34,.024),'paper',.009)
 rod('Tethered visitor pen',(-.53,-.18,1.14),(-.36,-.05,1.14),.009,'blue')
 front('Reception inset service panel',-.62,.53,.45,.45,-.517,'blue_light',.013,.025)
 for zz in [.41,.54,.67]:front('Service insignia',-.62,zz,.26,.022,-.53,'enamel_light',.015,.003)

broadcast_rack();tower_reception()

# Export by material without joining source objects, retaining a fully editable kit.
def export(name,col):
 groups={}
 for obj in col.objects:
  if obj.type!='MESH':continue
  me=obj.data;me.calc_loop_triangles();normal_matrix=obj.matrix_world.to_3x3().inverted().transposed()
  for tri in me.loop_triangles:
   if tri.normal.length_squared<.9:continue
   mat=me.materials[tri.material_index]
   group=groups.setdefault(mat.name,{'positions':[],'normals':[],'indices':[],'base_color':[round(v,5) for v in mat.diffuse_color], 'material':mat.name,'lookup':{}})
   n=(normal_matrix@tri.normal).normalized();norm=(round(n.x,6),round(n.z,6),round(-n.y,6))
   for vid in tri.vertices:
    v=obj.matrix_world@me.vertices[vid].co;pos=(round(v.x,6),round(v.z,6),round(-v.y,6));key=pos+norm
    if key not in group['lookup']:
     group['lookup'][key]=len(group['positions'])//3;group['positions'].extend(pos);group['normals'].extend(norm)
    group['indices'].append(group['lookup'][key])
 primitives=[]
 for key,g in sorted(groups.items()):del g['lookup'];primitives.append(g)
 coords=[g['positions'] for g in primitives]
 lo=[min(p[i] for g in coords for p in zip(*[iter(g)]*3)) for i in range(3)]
 hi=[max(p[i] for g in coords for p in zip(*[iter(g)]*3)) for i in range(3)]
 data={'name':name,'coordinate_system':'right-handed; +Y up; front +Z','origin':'floor-center','bounds':{'min':lo,'max':hi},'triangle_count':sum(len(g['indices'])//3 for g in primitives),'primitives':primitives}
 (P/f'{name}.mesh.json').write_text(json.dumps(data,separators=(',',':')))
 return {k:v for k,v in data.items() if k!='primitives'}
bpy.context.view_layer.update()
stats={name:export(name,col) for name,col in ASSETS.items()}
# Overview display offsets exist only in this editable file, never in runtime exports.
for i,(name,col) in enumerate(ASSETS.items()):
 offset=Vector(((i%8-3.5)*3.0,(i//8-4)*3.0,0))
 for obj in col.objects:obj.location+=offset
 col['runtime_asset']=name+'.mesh.json';col['authoring_origin']=list(offset)
begin('PRESENTATION only');cube('Neutral studio floor',(0,0,-.12),(200,200,.20),'linen')
world=bpy.data.worlds.new('Interior atelier');scene.world=world;world.use_nodes=True
world.node_tree.nodes['Background'].inputs['Color'].default_value=(.25,.32,.39,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.55
for name,loc,energy,size,color in [('Key',(-10,-13,19),3300,12,(1,.88,.71)),('Fill',(12,-2,13),2200,10,(.73,.86,1)),('Rim',(0,12,15),2600,10,(1,.95,.85))]:
 bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name=name;o.data.energy=energy;o.data.size=size;o.data.color=color;o.rotation_euler=(Vector((0,0,.8))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(17,-24,26));cam=bpy.context.object;cam.rotation_euler=(Vector((0,0,.7))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=35;scene.camera=cam
scene.render.engine='CYCLES';scene.cycles.samples=24;scene.cycles.use_denoising=False
scene.render.resolution_x=1800;scene.render.resolution_y=1500;scene.render.resolution_percentage=100
scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.filepath=str(P/'interiors-kit.png')
scene['Asset source']='Original authored full-volume furnishings. No imported artwork or source pixels. Runtime coordinates +Y up / +Z front. Source objects stay named and individually editable.'
bpy.ops.wm.save_as_mainfile(filepath=str(P/'johto-interiors.blend'),compress=True)
if '--skip-preview' not in args:bpy.ops.render.render(write_still=True)
print('INTERIOR_ASSETS='+json.dumps(stats))
