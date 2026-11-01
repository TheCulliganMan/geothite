"""Original, editable world exterior art; never imports game textures/content.
Run: blender -b --python tools/build-world-exteriors.py -- target/world-exteriors [--skip-preview]
The named Blender parts are the source. Runtime is material-indexed Y-up geometry.
"""
from pathlib import Path
source = Path(__file__).with_name('build-connected-johto-models.py').read_text()
exec(compile(source.split('\nbuilders=')[0], 'exterior_modeling_vocabulary', 'exec'))
# Direct mesh construction bounds peak memory and avoids selection/context
# operators across thousands of independently editable architectural parts.
import bmesh
from mathutils import Matrix
for n,c in {
 'tower_coral':(.64,.18,.13),
 'glass_tower_cyan':(.055,.44,.56),
 'glass_tower_deep':(.025,.16,.29),
 'park_frame':(.065,.075,.085),
 'park_slats_light':(.61,.64,.63),
 'park_slats':(.48,.52,.53),
 'cut_leaf':(.52,.66,.18),
 'cut_leaf_light':(.70,.76,.30),
 'cut_leaf_dark':(.32,.47,.11),
 'cut_bark':(.57,.35,.16),
}.items():
 m=bpy.data.materials.new(n);m.diffuse_color=(*c,1);m.use_nodes=True
 m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(*c,1)
 m.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.82
 MATS[n]=m
def cube(n,loc,dim,mat,bevel=0):
 bm=bmesh.new();bmesh.ops.create_cube(bm,size=1)
 bmesh.ops.scale(bm,vec=Vector(dim),verts=list(bm.verts))
 if bevel:bmesh.ops.bevel(bm,geom=list(bm.edges),offset=min(bevel,min(dim)*.4),segments=1,affect='EDGES')
 bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces))
 me=bpy.data.meshes.new(n);bm.to_mesh(me);bm.free();me.update()
 o=bpy.data.objects.new(n,me);COL.objects.link(o);o.location=loc;me.materials.append(MATS[mat]);return o
DOORS={}; TARGETS={}

def start(name,w,d,side='south'):
 begin(name); TARGETS[name]=(w,d); DOORS[name]=side

def panel(n,x,z,w,h,y,mat='glass',angle=0):
 carve_window_pocket(COL,x,z,w,h,y,angle)
 # Hollow surround seated at the wall plane, with real reveals behind it.
 for side in [-1,1]:
  facade_cube(n+' recessed jamb',x+side*(w+.055)/2,z,.055,h+.11,y+.10,.11,'stone_dark',angle)
  facade_cube(n+' recessed rail',x,z+side*(h+.055)/2,w,.055,y+.10,.11,'stone_dark',angle)
 facade_cube(n+' inset face',x,z,w,h,y+.13,.035,mat,angle)
 facade_cube(n+' reflection',x-w*.20,z+h*.25,w*.32,.055,y+.106,.016,'glass_light',angle)
 for dx in [-w/2,0,w/2]:facade_cube(n+' upright',x+dx,z,.03,h,y+.10,.024,'plaster_light',angle)
 facade_cube(n+' sill',x,z-h/2-.035,w+.14,.055,y+.03,.10,'stone_light',angle)


def medallion(y,z,palette):
 # Shared vocabulary no longer supplies the older civic roundel helper.
 for name,radius,depth,offset,material in [
  ('Medical ball outer ring',.25,.035,0.,'plaster_light'),
  ('Medical ball inset',.19,.025,-.025,palette),
  ('Medical ball button',.065,.025,-.045,'plaster_light')]:
  emblem=cone(name,(0,y+offset,z),radius,radius,depth,material,24)
  emblem.rotation_euler.x=math.pi/2
 facade_cube('Medical ball equator',0,z,.46,.045,y-.05,.025,'plaster_light')

def flat_roof(w,d,z,palette='roof_blue',parapet=True):
 cube('Closed roof slab',(0,0,z),(w,d,.19),palette+'_dark',.025)
 cube('Roof weathering surface',(0,0,z+.105),(w-.20,d-.20,.045),palette)
 if parapet:
  for y in [-d/2+.055,d/2-.055]:cube('Long parapet coping',(0,y,z+.23),(w,.11,.28),palette+'_light',.018)
  for x in [-w/2+.055,w/2-.055]:cube('Side parapet coping',(x,0,z+.23),(.11,d,.28),palette+'_light',.018)
  cube('Roof service hatch',(w*.26,d*.20,z+.25),(.45,.52,.22),'stone_light',.025)
  for y in [d*.20-.15,d*.20,d*.20+.15]:cube('Service hatch louver',(w*.26,y,z+.366),(.34,.035,.02),'stone_dark')

def department_bricks(w,d,h):
 # Reuse the civic kit's closed shallow frusta. The wall behind them remains
 # continuous mortar; the existing window cutter carves both layers together.
 for angle,span,distance,count in [(0,w-.40,d/2-.07,7),(180,w-.40,d/2-.07,7),
                                  (90,d-.40,w/2-.07,4),(-90,d-.40,w/2-.07,4)]:
  for row in range(14):
   step=span/count
   starts=[-span/2]+[-span/2+(i+(.5 if row%2 else 1))*step for i in range(count) if 0<i+(.5 if row%2 else 1)<count]+[span/2]
   for left,right in zip(starts,starts[1:]):
    bottom=.25+row*h/14+.008;top=.25+(row+1)*h/14-.008
    civic_brick('Department running-bond brick',[(left+.008,bottom),(right-.008,bottom),
                (right-.008,top),(left+.008,top)],distance,angle)

def full_shell(w,d,h,wall='plaster_light',floors=1,cols=3,entry_clearance=.6,brick_faces=False):
 foundation(w,d);cube('Complete four-sided wall body',(0,0,.25+h/2),(w-.14,d-.14,h),
                     'plaster_service_mortar' if brick_faces else wall,.025)
 if brick_faces:department_bricks(w,d,h)
 for x in [-w/2+.08,w/2-.08]:
  for y in [-d/2+.08,d/2-.08]:cube('Corner pilaster',(x,y,.25+h/2),(.17,.17,h+.05),'stone_light',.018)
 for floor in range(floors):
  z=.25+(floor+.5)*h/floors
  for angle,span,fy in [(0,w,-d/2+.025),(180,w,-d/2+.025),(90,d,-w/2+.025),(-90,d,-w/2+.025)]:
   count=cols if angle in [0,180] else max(1,round(d/1.35))
   for i in range(count):
    x=(i-(count-1)/2)*span/(count+.35)
    if angle==0 and floor==0 and abs(x)<entry_clearance:continue
    panel('Storey window',x,z,span/(count+.35)*.63,min(.92,h/floors*.58),fy,angle=angle)
  if floor:
   for y in [-d/2-.015,d/2+.015]:cube('Horizontal floor belt',(0,y,.25+floor*h/floors),(w,.10,.095),'plaster')
   for x in [-w/2-.015,w/2+.015]:cube('Side floor belt',(x,0,.25+floor*h/floors),(.10,d,.095),'plaster')

def front_entry(d,w=1.0):
 door(-d/2-.015,z=1.0,w=w,h=1.47,lab=True)
 cube('Entrance threshold',(0,-d/2-.16,.10),(w+.32,.38,.20),'stone_light',.023)

def awning(w,d,z,palette='roof_blue'):
 cube('Entrance canopy',(0,-d/2-.23,z),(w,.54,.12),palette+'_dark',.025)
 for i in range(9):
  x=(i-4)*w/9
  cube('Canopy stripe',(x,-d/2-.26,z+.066),(w/9-.01,.50,.02),palette if i%2 else 'plaster_light')

def modern_gym():
 start('modern_gym',5.95,3.90)
 w,d,h=5.45,3.30,2.70
 foundation(w,d)
 cube('Closed modern Gym masonry body',(0,0,.25+h/2),(w-.14,d-.14,h),'plaster_service_mortar',.025)
 department_bricks(w,d,h)
 # Two paired front windows flank the entry, matching the native civic face.
 for x in [-1.88,1.88]:panel('Paired Gym front glazing',x,1.62,.91,.77,-d/2+.025)
 for angle,span,face,count in [(180,w,-d/2+.025,3),(90,d,-w/2+.025,2),(-90,d,-w/2+.025,2)]:
  for i in range(count):panel('Gym side and rear glazing',(i-(count-1)/2)*span/(count+.35),1.62,.82,.77,face,angle=angle)
 for x in [-w/2+.08,w/2-.08]:
  for y in [-d/2+.08,d/2-.08]:cube('Slim Gym masonry corner pier',(x,y,1.60),(.15,.17,2.75),'stone_light',.018)
 for y in [-d/2+.025,d/2-.025]:cube('Gym recessed upper wall band',(0,y,2.76),(w-.16,.07,.10),'plaster',.008)
 front_entry(d,1.1)
 # A pale fascia and actual dark lettering replace generic gold badge lumps.
 facade_cube('Gym sign pale fascia',0,2.48,1.46,.39,-d/2-.09,.085,'cream',bevel=.014)
 source_service_letters('GYM',-d/2-.151,2.48,.235,'ink')
 flat_roof(w+.30,d+.30,h+.30,'roof_blue')
 for o in list(COL.objects):
  if o.name.startswith(('Long parapet coping','Side parapet coping')):bpy.data.objects.remove(o,do_unlink=True)
 outer=[(-2.875,-1.80),(2.875,-1.80),(2.875,1.80),(-2.875,1.80)]
 inner=[(-2.765,-1.69),(2.765,-1.69),(2.765,1.69),(-2.765,1.69)]
 vs=[(x,y,z) for z in [3.09,3.37] for ring in [outer,inner] for x,y in ring];fs=[]
 for i in range(4):
  j=(i+1)%4;fs.extend([(i,j,4+j,4+i),(8+j,8+i,12+i,12+j),(i,8+i,8+j,j),(4+j,12+j,12+i,4+i)])
 mesh('Continuous mitered Gym roof coping',vs,fs,'roof_blue_light')


def civic(name,style,w=5.5,d=3.5):
 start(name,w+.5,d+.6)
 floors=3 if style in ['department','silph','mansion'] else 1
 h={'department':5.8,'silph':7.0,'mansion':4.8,'station':2.6,'gym':2.7,'power':3.2}.get(style,2.35)
 full_shell(w,d,h,wall='plaster' if style in ['mansion'] else 'plaster_light',floors=floors,cols=4 if w>5 else 3,brick_faces=name=='modern_department')
 front_entry(d,1.1 if w>4 else .8)
 palette='roof_red' if style=='center' else 'roof_teal' if style in ['station','mansion'] else 'roof_blue'
 if style in ['house','mansion']:
  roof(w+.40,d+.35,h+.28,h+1.35,palette,5)
 else:flat_roof(w+.3,d+.3,h+.3,palette)
 if name.startswith('kanto_'):
  # Kanto has a different civic vocabulary: broad curved service eaves,
  # recessed tiled entrances and pale stone side fins, not Johto city slabs.
  if style=='center':
   z=h+.66;half=w*.48;dep=d*.50
   vs=[]
   for yy in [-dep,dep]:
    for i in range(13):
     a=i*math.pi/12;vs.append((math.cos(a)*half,yy,z+math.sin(a)*.64))
   faces=[tuple(range(12,-1,-1)),tuple(range(13,26))]+[(i,i+1,i+14,i+13) for i in range(12)]+[(0,13,25,12)]
   mesh('Kanto curved ceramic service roof',vs,faces,'roof_red')
  elif style in ['gym','station']:
   for i in range(3):cube('Kanto civic stepped parapet',(0,0,h+.52+i*.19),(w+.20-i*.55,d+.16-i*.38,.19),'roof_blue_light' if i==2 else 'roof_blue_dark',.035)
  elif style=='mart':
   for x in [-w*.37,w*.37]:cube('Kanto shop deep side wing',(x,-d*.2,1.48),(.32,d*.76,2.5),'plaster',.04)
  for x in [-w/2+.19,w/2-.19]:
   cube('Kanto limestone facade fin',(x,-d/2-.07,h/2+.20),(.20,.27,h),'stone_light',.025)
 if style=='center':
  awning(2.0,d,1.99,palette);medallion(-d/2-.22,2.37,palette)
 elif style=='mart':
  awning(w-.3,d,1.98,palette)
  facade_cube('Retail blue signboard',0,2.42,1.35,.38,-d/2-.14,.11,palette+'_dark',bevel=.025)
  for x in [-.38,-.19,0,.19,.38]:facade_cube('Retail plaque raised marks',x,2.42,.085,.20,-d/2-.21,.035,'plaster_light')
 elif style=='gym':
  for x in [-w*.31,w*.31]:
   cube('Gym entrance buttress',(x,-d/2-.10,1.48),(.35,.28,2.45),'stone_light',.025)
   cone('Gym stone capital',(x,-d/2-.10,2.73),.30,.24,.15,'stone',8)
  facade_cube('Gym carved crest',0,2.65,1.2,.45,-d/2-.14,.12,'roof_blue_dark',bevel=.04)
  for x in [-.32,.32]:ico('Gym badge metal',(x,-d/2-.23,2.66),(.19,.05,.18),'yellow',1)
 elif style=='station':
  awning(w-.35,d,2.18,'roof_teal')
  for x in [-w*.37,w*.37]:cube('Station canopy column',(x,-d/2-.31,1.08),(.11,.11,2.16),'stone_dark')
  o=cone('Station clock rim',(0,-d/2-.11,2.70),.28,.28,.085,'timber',20);o.rotation_euler.x=math.pi/2
  o=cone('Station clock face',(0,-d/2-.17,2.70),.23,.23,.015,'cream',20);o.rotation_euler.x=math.pi/2
  facade_cube('Clock minute hand',0,2.76,.035,.15,-d/2-.19,.02,'ink')
  facade_cube('Clock hour hand',.045,2.70,.12,.035,-d/2-.19,.02,'ink')
 elif style=='department':
  awning(w-.22,d,1.92,'roof_red');
  for floor in range(1,4):
   facade_cube('Department vertical corner rib',-w*.42,.25+h/2,.14,h,-d/2-.08,.13,'roof_red_dark')
  facade_cube('Department entry lettering',0,2.24,2,.31,-d/2-.15,.12,'roof_red_dark')
  if name=='modern_department':source_service_letters('MART',-d/2-.24,2.24,.22,'cream')
  else:
   for x in [-.60,-.30,0,.30,.60]:facade_cube('Department raised letter',x,2.24,.13,.17,-d/2-.23,.025,'cream')
 elif style=='silph':
  for x in [-w*.29,0,w*.29]:cube('Corporate full-height curtain rib',(x,-d/2-.07,h*.57),(.12,.12,h*.73),'roof_blue_light')
  cube('Silph rooftop penthouse',(0,0,h+.67),(w*.56,d*.62,.62),'glass_dark',.035)
  facade_cube('Silph company plaque',0,2.13,1.9,.36,-d/2-.14,.12,'roof_blue_dark')
  for x in [-.52,-.26,0,.26,.52]:facade_cube('Silph raised lettering',x,2.13,.12,.19,-d/2-.215,.025,'cream')
 elif style=='mansion':
  for x in [-w*.25,w*.25]:
   cube('Mansion balcony floor',(x,-d/2-.22,3.15),(1.33,.50,.14),'stone_light')
   for dx in [-.55,-.28,0,.28,.55]:cube('Mansion balcony spindle',(x+dx,-d/2-.46,3.42),(.045,.045,.46),'timber')
   cube('Mansion balcony handrail',(x,-d/2-.46,3.68),(1.3,.065,.065),'timber')
 elif style=='power':
  for x in [-w*.30,w*.30]:
   cone('Power station flue',(x,d*.2,h+1.1),.25,.21,1.6,'stone_dark',10)
   cone('Flue collar',(x,d*.2,h+1.9),.30,.30,.10,'stone_light',10)
  for i in range(4):cube('Power roof cooling vent',(-.7+i*.45,0,h+.65),(.25,.8,.45),'stone')
 elif style=='arcade':
  awning(w-.3,d,2.0,'roof_red')
  if name=='modern_arcade':
   # One shared Game Corner frontage: a recessed enamel sign, solid raised
   # lettering and restrained jewel lamps instead of seven anonymous dots.
   facade_cube('Game Corner closed marquee frame',0,2.54,w-.5,.55,-d/2-.16,.15,'violet_light',bevel=.035)
   facade_cube('Game Corner recessed enamel field',0,2.54,w-.64,.41,-d/2-.242,.028,'violet_dark',bevel=.018)
   source_service_letters('GAME',-d/2-.276,2.54,.32,'cream')
   for side in [-1,1]:
    for i in range(3):
     x=side*(1.40+i*.37)
     lamp=cone('Game Corner lamp socket',(x,-d/2-.274,2.54),.087,.087,.035,'timber_gold',12)
     lamp.rotation_euler.x=math.pi/2
     ico('Game Corner jewel lamp',(x,-d/2-.300,2.54),(.062,.036,.062),['yellow','coral','glass_light'][i],2)
    facade_cube('Game Corner marquee end bracket',side*(w-.60)/2,2.37,.09,.25,-d/2-.15,.18,'violet_dark',bevel=.012)
  else:
   facade_cube('Arcade illuminated marquee',0,2.54,w-.5,.42,-d/2-.16,.14,'violet_dark',bevel=.04)
   for i in range(7):ico('Arcade colored marquee bulb',((i-3)*.41,-d/2-.255,2.54),(.11,.07,.11),['yellow','coral','glass_light'][i%3],1)


def townhouse(name,variant):
 w=2.75 if variant=='compact' else 3.75;d=2.55
 start(name,w+.45,d+.6,side='none' if variant=='blank' else 'south')
 full_shell(w,d,2.05,wall='plaster',cols=3)
 if variant=='pitched':roof(w+.34,d+.31,2.29,3.26,'roof_red',5)
 else:
  flat_roof(w+.3,d+.25,2.29,'roof_blue')
  for y in [-.8,-.4,0,.4,.8]:cube('Corrugated rooftop rib',(0,y,2.42),(w-.23,.035,.025),'roof_blue_light')
 if variant!='blank':front_entry(d,.69)
 if variant=='compact':awning(1.45,d,1.85,'roof_red')
 if variant=='pitched':
  cube('House chimney',(-w*.30,.45,2.98),(.33,.39,1.0),'plaster_shadow',.02)
  cube('House chimney coping',(-w*.30,.45,3.51),(.44,.49,.13),'stone_light',.02)


def east_gate():
 start('east_gate',4,4,'east');foundation(3.52,3.48)
 # Open transverse passage; no solid wall blocks the east/west entrance.
 for y,depth in [(-1.61,.24),(.61,2.16)]:
  cube('Gate side wing',(0,y,1.24),(3.50,depth,2.04),'plaster_light',.025)
  # Three low stone courses seat each wing; the passage remains open.
  for row in range(3):
   for col in range(7):
    x=-1.44+col*.48
    cube('Gatehouse dressed plinth',(x,(-1.74 if y<0 else 1.74),.36+row*.18),(.466,.10,.166),'stone_light' if (col+row)%4 else 'stone',.012)
  for x in [-1.76,1.76]:
   for row in range(3):
    for part in range(max(1,round(depth/.48))):
     span=depth/max(1,round(depth/.48))
     cube('Gatehouse plinth return',(x,y-depth/2+span*(part+.5),.36+row*.18),(.10,span-.012,.166),'stone_light',.012)
 for angle in [0,180]:panel('Gate wing window',0,1.35,1.1,.74,-1.73,angle=angle)
 for x in [-1.71,1.71]:
  cube('Passage lintel',(x,-.985,2.11),(.17,1.45,.38),'timber')
  for y in [-1.535,-.435]:cube('Passage jamb',(x,y,1.08),(.17,.16,2.03),'timber_light')
 cube('Passage recessed paving',(0,-.985,.07),(3.89,1.12,.14),'stone_light',.015)
 # A fitted doorway may widen below; deck, shingles and gable truss must
 # remain one rigid assembly so the backing cannot pierce the roof courses.
 roof_before=set(COL.objects)
 town_roof(3.93,3.94,2.35,3.38,'roof_blue',gable_material='plaster_light')
 for o in set(COL.objects)-roof_before:o['rigid_roof']=True
 # Exposed structural posts and real eave brackets frame the open passage.
 for x in [-1.76,1.76]:
  for y in [-1.74,1.74]:
   cube('Gatehouse corner timber',(x,y,1.30),(.14,.14,2.10),'timber',.015)
  for y in [-1.535,-.435]:
   beam('Passage carved knee brace',(x,y,1.78),(x,y+(.30 if y>-1. else -.30),2.18),.10,.12,'timber_light')
 for x in [-1.94,1.94]:
  cube('Gatehouse gable tie',(x,0,2.38),(.12,3.46,.14),'timber',.012)
  cube('Gatehouse king post',(x,0,2.84),(.12,.13,.90),'timber_light',.012)
  for side in [-1,1]:
   beam('Gatehouse gable brace',(x,side*1.30,2.43),(x,0,3.20),.09,.11,'timber_light')
 for x in [-1.44,-.96,-.48,0,.48,.96,1.44]:
  for y in [-1.80,1.80]:cube('Gatehouse exposed rafter tail',(x,y,2.30),(.09,.28,.12),'timber_light',.008)
 for y in [-1.85,1.85]:
  cube('Gatehouse eave fascia',(0,y,2.30),(3.88,.12,.18),'timber',.018)
 # East-facing low threshold ends on the actual side doorway seam.
 cube('East entry threshold',(1.88,-.985,.08),(.24,1.24,.16),'stone_light',.015)
 facade_cube('Gate side plaque',-.985,2.13,.93,.26,-1.80,.09,'roof_blue_dark',90)
 for o in COL.objects:
  if o.name.startswith(('Gatehouse gable','Gatehouse king post','Gatehouse exposed rafter','Gatehouse eave fascia')):
   o['rigid_roof']=True


def pagoda_hip_roof(width,depth,eave,ridge):
 # Closed hip backing with gently lifted eaves and separately closed ceramic
 # courses. A short ridge and four hip caps replace the stacked house gables.
 x,d=width/2,depth/2;r=max((width-depth)*.34,width*.12)
 lip=eave+.14
 top=[(-x,-d,lip),(x,-d,lip),(x,d,lip),(-x,d,lip),(-r,0,ridge),(r,0,ridge)]
 vs=top+[(a,b,eave-.08) for a,b,_ in top[:4]]
 fs=[(0,1,5,4),(1,2,5),(2,3,4,5),(3,0,4),(6,9,8,7)]
 fs += [(i,6+i,6+(i+1)%4,(i+1)%4) for i in range(4)]
 mesh('Pagoda closed hipped roof backing',vs,fs,'violet_dark')
 for side,(a,b,ia,ib) in enumerate([(0,1,4,5),(1,2,5,5),(2,3,5,4),(3,0,4,4)]):
  outer0,outer1,inner0,inner1=[Vector(top[i]) for i in [a,b,ia,ib]]
  columns=6 if side%2==0 else 3
  def point(u,t):
   p=(outer0+(outer1-outer0)*u)*(1-t)+(inner0+(inner1-inner0)*u)*t
   p.z=eave+(ridge-eave)*t+.14*(1-t)**4+.026
   return tuple(p)
  for row in range(3):
   t0=row/3;t1=min((row+1)/3,.97)
   for col in range(columns):
    u0=(col+.018)/columns;u1=(col+1-.018)/columns
    p=[point(u0,t0),point(u1,t0),point(u1,t1),point(u0,t1)]
    vs=p+[(a,b,z-.035) for a,b,z in p]
    faces=[(0,1,2,3),(7,6,5,4)]+[(i,4+i,4+(i+1)%4,(i+1)%4) for i in range(4)]
    mesh('Pagoda closed ceramic hip course',vs,faces,['violet','violet_warm','violet','violet_light'][(col+row*3+side)%4])
 for a,b in [(0,4),(1,5),(2,5),(3,4)]:
  start=Vector(top[a])+Vector((0,0,.035));end=Vector(top[b])+Vector((0,0,.035))
  rod('Pagoda ceramic hip cap',start,end,.045,'violet_light',6)
 cube('Pagoda short ceramic ridge',(0,0,ridge+.04),(r*2+.12,.18,.12),'violet_light',.025)
 for y in [-d,d]:cube('Pagoda deep eave fascia',(0,y,eave+.005),(width,.10,.14),'timber')
 for xx in [-x,x]:cube('Pagoda return fascia',(xx,0,eave+.005),(.10,depth,.14),'timber')

def pagoda_window(x,z,y,w=.46,h=.64,angle=0):
 # Closed cedar shutters sit in their projecting frame. Their opaque backing
 # seals the infill, so no redundant hidden window cutter is needed.
 f=lambda n,dx,dz,ww,hh,off,dd,mat:facade_cube(n,x+dx,z+dz,ww,hh,y+off,dd,mat,angle)
 f('Pagoda recessed timber shutter',0,0,w,h,-.025,.035,'timber')
 for dx in [-w/2,w/2]:f('Pagoda shutter jamb',dx,0,.052,h+.10,-.065,.10,'timber_light')
 f('Pagoda shutter head rail',0,h/2,w,.055,-.075,.12,'timber_light')
 for dx in [-w*.19,w*.19]:f('Pagoda narrow lattice upright',dx,0,.024,h-.06,-.06,.035,'timber_gold')
 f('Pagoda shutter sill',0,-h/2-.035,w+.15,.065,-.085,.15,'timber_light')

def tin_tower():
 start('tin_tower',5.9,4.5)
 for i in range(8):
  w=5.25-i*(2.56/7);d=3.8-i*(1.60/7)
  z=.25 if i==0 else 1.85+(i-1)*1.02
  h=1.25 if i==0 else .70
  cube('Pagoda plaster infill',(0,0,z+h/2),(w,d,h),'plaster_light',.015)
  for x in [-w/2+.04,w/2-.04]:
   for y in [-d/2+.04,d/2-.04]:cube('Pagoda cedar corner upright',(x,y,z+h/2),(.13,.13,h),'timber_light')
  for y in [-d/2-.02,d/2+.02]:cube('Pagoda cedar floor header',(0,y,z+h-.06),(w,.075,.11),'timber')
  for x in [-w/2-.02,w/2+.02]:cube('Pagoda cedar return header',(x,0,z+h-.06),(.075,d,.11),'timber')
  if i==0:cube('Pagoda foundation sill',(0,0,.35),(w+.07,d+.07,.14),'plaster_light',.016)
  for angle,span,fy in [(0,w,-d/2-.01),(180,w,-d/2-.01),(90,d,-w/2-.01),(-90,d,-w/2-.01)]:
   for x in ([-span*.29,span*.29] if angle==0 else [0]):
    pagoda_window(x,z+(.74 if i==0 else .38),fy,w=.46 if i==0 else .36,h=.64 if i==0 else .40,angle=angle)
  eave=z+(1.26 if i==0 else .71);ridge=z+(2.0 if i==0 else 1.20)
  for sx in [-1,1]:
   for sy in [-1,1]:
    x=sx*(w/2-.06);y=sy*(d/2-.06)
    cube('Pagoda bracket block',(x,y,eave-.13),(.20,.20,.12),'timber_gold')
    rod('Pagoda diagonal eave brace',(x,y,eave-.30),(sx*(w/2+.15),sy*(d/2+.12),eave+.07),.045,'timber_light',4)
  pagoda_hip_roof(w+.50,d+.48,eave,ridge)
 fy=-3.8/2-.015
 facade_cube('Pagoda cedar doorway frame',0,.92,1.10,1.52,fy,.16,'timber',bevel=.018)
 for side in [-1,1]:
  facade_cube('Pagoda solid cedar door leaf',side*.218,.92,.425,1.34,fy-.11,.065,'timber_light',bevel=.012)
  for dz in [-.43,0,.43]:facade_cube('Pagoda door cross rail',side*.218,.92+dz,.37,.055,fy-.15,.035,'timber_gold')
  for dx in [-.12,0,.12]:facade_cube('Pagoda door upright',side*.218+dx,.92,.022,1.25,fy-.154,.025,'timber')
  facade_cube('Pagoda bronze door pull',side*.09,.91,.025,.16,fy-.18,.05,'yellow',bevel=.006)
 cube('Pagoda stone entry threshold',(0,fy-.16,.10),(1.17,.38,.20),'stone_light',.023)
 rod('Pagoda bronze finial',(0,0,9.1),(0,0,10.10),.055,'yellow',10)
 for z,r in [(9.25,.23),(9.48,.18),(9.7,.13)]:cone('Pagoda finial disk',(0,0,z),r,r,.065,'yellow',12)

def pagoda(name,burnt=False):
 if not burnt:return tin_tower()
 start(name,5.9 if not burnt else 4,4.5 if not burnt else 4)
 tiers=5 if not burnt else 1
 for i in range(tiers):
  w=(5.25 if not burnt else 3.58)-i*.64;d=(3.8 if not burnt else 3.5)-i*.40;z=.25+i*1.73
  cube('Pagoda storey',(0,0,z+.62),(w,d,1.25),'plaster_shadow' if burnt else 'plaster_light',.023)
  building_trim(w,d,z+.62,1.25)
  for angle,span,fy in [(0,w,-d/2-.01),(180,w,-d/2-.01),(90,d,-w/2-.01),(-90,d,-w/2-.01)]:
   for x in [-span*.29,span*.29]:window(x,z+.74,fy,w=.52,h=.62,angle=angle)
  roof(w+.50,d+.48,z+1.26,z+2.0,'roof_red' if burnt else 'violet',4)
  if not burnt:
   for x in [-w*.49,w*.49]:
    for y in [-d*.49,d*.49]:rod('Raised eave corner',(x,y,z+1.32),(x*1.10,y*1.10,z+1.56),.065,'violet_light')
 front_entry(3.8 if not burnt else 3.5,.85)
 if burnt:
  # Missing central roof courses reveal charcoal rafters and a dark broken
  # decking panel, rather than painting scorch marks onto a complete roof.
  for ob in list(COL.objects):
   if ob.name.startswith('Roof closed gable'):
    bpy.data.objects.remove(ob,do_unlink=True)
   elif ob.name.startswith('Individual roof course'):
    center=sum((v.co for v in ob.data.vertices),Vector())/len(ob.data.vertices)
    if abs(center.x)<1.04 and center.y>-.65:bpy.data.objects.remove(ob,do_unlink=True)
  cube('Charred exposed roof decking',(0,.30,1.55),(2.15,1.25,.075),'timber')
  for i in range(6):
   x=-1.45+i*.54
   rod('Charred exposed roof timber',(x,-.90,2.1),(x,.40,2.99-(i%3)*.20),.10,'timber')
  for x in [-1.62,1.62]:cube('Burned structural scar',(x,-1.76,1.10),(.16,.05,1.55),'timber')
 else:
  rod('Pagoda bronze finial',(0,0,9.1),(0,0,10.10),.055,'yellow',10)
  for z,r in [(9.25,.23),(9.48,.18),(9.7,.13)]:cone('Pagoda finial disk',(0,0,z),r,r,.065,'yellow',12)


def lighthouse():
 start('lighthouse',4,4)
 cone('Circular lighthouse plinth',(0,0,.20),1.7,1.7,.4,'stone',32)
 shaft=cone('Tapered white lighthouse shaft',(0,0,3.88),1.45,1.10,7.2,'plaster_light',32)
 shaft['radial_normals']=True
 for z,r in [(1.0,1.43),(2.6,1.34),(4.3,1.26),(6.0,1.18)]:
  cone('Lighthouse masonry belt',(0,0,z),r+.025,r+.025,.12,'plaster',32)
  for a in [0,90,180,270]:
   # The entry occupies this lowest front bay; a slit here cuts through
   # its lintel and leaves a second pane protruding above the door frame.
   if z==1.0 and a==0:continue
   panel('Lighthouse slit window',0,z+.68,.36,.67,-r-.01,angle=a)
 cone('Lantern balcony floor',(0,0,7.62),1.7,1.7,.22,'stone_light',32)
 for i in range(16):
  a=i*math.tau/16;rod('Lantern balcony railing',(math.cos(a)*1.55,math.sin(a)*1.55,7.74),(math.cos(a)*1.55,math.sin(a)*1.55,8.18),.035,'timber',6)
 # Closed rail segments form an open ring; a capped cone here seals the
 # entire balcony annulus with a floating timber disk.
 for i in range(16):
  a=i*math.tau/16;b=(i+1)*math.tau/16
  rod('Lantern continuous handrail',(math.cos(a)*1.55,math.sin(a)*1.55,8.20),(math.cos(b)*1.55,math.sin(b)*1.55,8.20),.035,'timber',6)
 # Clear panes are expressed by open framed bays and narrow glass glints.
 # An opaque closed drum hid the actual beacon in the solid terrain domain.
 def lantern_ring(name,lo,hi,outer,inner,material):
  vertices=[(math.cos(i*math.tau/16)*r,math.sin(i*math.tau/16)*r,z)
            for z in [lo,hi] for r in [outer,inner] for i in range(16)]
  faces=[]
  for i in range(16):
   j=(i+1)%16
   faces.extend([(i,j,16+j,16+i),(32+j,32+i,48+i,48+j),
                 (i,32+i,32+j,j),(16+j,48+j,48+i,16+i)])
  mesh(name,vertices,faces,material)
 lantern_ring('Lantern lower glazed weather ring',7.73,7.88,.95,.88,'glass')
 lantern_ring('Lantern upper glazed weather ring',9.04,9.19,.95,.88,'glass')
 for i in range(8):
  a=(i+.30)*math.tau/8
  rod('Lantern narrow pane reflection',(math.cos(a)*.94,math.sin(a)*.94,8.10),
      (math.cos(a+.08)*.94,math.sin(a+.08)*.94,8.73),.012,'glass_light',4)
 for i in range(8):
  a=i*math.tau/8;rod('Lantern iron glazing bar',(math.cos(a)*.96,math.sin(a)*.96,7.73),(math.cos(a)*.96,math.sin(a)*.96,9.19),.055,'roof_blue_dark',6)
 # One closed ribbed optic, with a visible pedestal rather than a buried orb.
 cone('Lantern lens pedestal',(0,0,7.86),.28,.24,.25,'roof_blue_dark',12)
 profile=[(7.85,.14),(7.96,.34)]
 for i in range(9):
  z=8.02+i*.105
  profile.extend([(z,.44),(z+.025,.55),(z+.065,.55),(z+.085,.44)])
 profile.extend([(9.02,.34),(9.11,.14)])
 vertices=[(math.cos(i*math.tau/16)*r,math.sin(i*math.tau/16)*r,z)
           for z,r in profile for i in range(16)]
 faces=[tuple(range(15,-1,-1)),tuple((len(profile)-1)*16+i for i in range(16))]
 for row in range(len(profile)-1):
  for i in range(16):
   j=(i+1)%16;faces.append((row*16+i,row*16+j,(row+1)*16+j,(row+1)*16+i))
 mesh('Warm stepped Fresnel lens',vertices,faces,'yellow')
 cone('Lantern conical cap',(0,0,9.44),1.18,.14,.65,'roof_blue_dark',32)
 rod('Lightning spike',(0,0,9.7),(0,0,10.3),.045,'yellow')
 front_entry(2.91,.71)


def radio_tower():
 start('radio_tower',4,4)
 foundation(3.5,3.25)
 cube('Closed charcoal broadcast body',(0,0,3.675),(3.36,3.11,6.85),'plaster_light',.025)
 # Four narrow paired window banks and louvres follow the native front.
 def louvres(x,z,w,y,angle=0):
  facade_cube('Recessed broadcast vent',x,z,w,.30,y,.045,'roof_blue_dark',angle)
  for dz in [-.085,0,.085]:
   facade_cube('Broadcast aluminium louvre',x,z+dz,w-.065,.035,y-.027,.045,'stone_light',angle,bevel=0)
 for z in [2.65,3.85,5.05,6.25]:
  for x in [-1.26,-.42,.42,1.26]:
   panel('Four-bank broadcast glazing',x,z,.57,.64,-1.60)
   louvres(x,z-.60,.61,-1.62)
 for angle,span,fy,count in [(180,3.5,-1.60,3),(90,3.25,-1.725,2),(-90,3.25,-1.725,2)]:
  for z in [1.17,2.65,3.85,5.05,6.25]:
   for i in range(count):
    x=(i-(count-1)/2)*span/(count+.35)
    panel('Broadcast side and rear glazing',x,z,.62,.64,fy,angle=angle)
    if z>2:louvres(x,z-.60,.66,fy-.02,angle)
 # Metal piers and layered spandrels give the charcoal mass a fine rhythm.
 for x in [-1.70,-.84,0,.84,1.70]:
  cube('Broadcast front vertical fin',(x,-1.615,3.71),(.065,.14,6.76),'stone_light')
 for x in [-1.70,1.70]:
  for y in [-1.57,1.57]:cube('Broadcast corner pier',(x,y,3.67),(.14,.16,6.86),'stone_light')
 for z in [2.05,3.25,4.45,5.65,7.02]:
  cube('Broadcast tower projecting floor',(0,0,z),(3.76,3.5,.17),'roof_blue_dark',.025)
  for y in [-1.755,1.755]:cube('Fine broadcast spandrel reveal',(0,y,z+.045),(3.62,.035,.035),'stone_dark',.005)
 # The source entrance is left of centre. Author it there so the whole
 # building stays evenly fitted while the native door anchor remains exact.
 entry=set(COL.objects);front_entry(3.25,1.04)
 for o in set(COL.objects)-entry:o.location.x-=.45
 for z in [.70,1.15,1.60]:louvres(1.03,z,.79,-1.62)
 flat_roof(3.80,3.55,7.1,'roof_blue')
 # Replace intersecting coping end caps with one closed mitered ring.
 for o in list(COL.objects):
  if o.name.startswith(('Long parapet coping','Side parapet coping')):bpy.data.objects.remove(o,do_unlink=True)
 outer=[(-1.9,-1.775),(1.9,-1.775),(1.9,1.775),(-1.9,1.775)]
 inner=[(-1.79,-1.665),(1.79,-1.665),(1.79,1.665),(-1.79,1.665)]
 vs=[(x,y,z) for z in [7.19,7.47] for ring in [outer,inner] for x,y in ring];faces=[]
 for i in range(4):
  j=(i+1)%4;faces.extend([(i,j,4+j,4+i),(8+j,8+i,12+i,12+j),(i,8+i,8+j,j),(4+j,12+j,12+i,4+i)])
 mesh('Continuous broadcast roof coping',vs,faces,'roof_blue_light')
 # Tapered, open steel mast sits toward the roof front for a readable skyline.
 mast_before=set(COL.objects)
 cy=-.67
 for x,y in [(-.45,-.3),(.45,-.3),(0,.48)]:
  cone('Mast foot mounting plate',(x,y+cy,5.76),.10,.10,.07,'stone_dark',8)
  rod('Triangular radio mast',(x,y+cy,5.75),(x*.3,y*.3+cy,8.9),.045,'stone_dark')
 for level in range(6):
  z=5.9+level*.55;scale=1-(z-5.75)/4.5
  pts=[(-.45*scale,-.3*scale+cy),(.45*scale,-.3*scale+cy),(0,.48*scale+cy)]
  for i in range(3):
   a=pts[i];b=pts[(i+1)%3]
   rod('Mast horizontal triangle',(a[0],a[1],z),(b[0],b[1],z),.022,'stone_light')
   if level<5:
    next_scale=1-(z+.55-5.75)/4.5
    ratio=next_scale/scale
    rod('Mast alternating diagonal',(a[0],a[1],z),(b[0]*ratio,cy+(b[1]-cy)*ratio,z+.55),.025,'stone_light')
 for z,w in [(8.27,.95),(8.88,.66)]:
  rod('Broadcast dipole crossarm',(-w/2,cy,z),(w/2,cy,z),.025,'stone_dark')
  for x in [-w/2,w/2]:rod('Broadcast paired dipole',(x,cy-.21,z),(x,cy+.21,z),.022,'stone_light')
 rod('Broadcast aerial',(0,cy,8.7),(0,cy,9.65),.035,'stone_dark')
 ico('Aerial warning lamp',(0,cy,9.69),(.10,.10,.12),'coral',1)
 # Allocate more of the fixed silhouette to the four authored upper floors.
 # Transform the entire closed mast, including plate thickness and beacon.
 mast_fit=(Matrix.Translation(Vector((.95,-.63,7.25)))
           @ Matrix.Diagonal((1.,1.,2.559999/4.06,1.))
           @ Matrix.Translation(Vector((0,0,-5.75))))
 for obj in set(COL.objects)-mast_before:obj.matrix_world=mast_fit @ obj.matrix_world
 facade_cube('Radio station sign',-.45,2.04,1.7,.30,-1.72,.12,'violet_dark')
 letters=set(COL.objects);source_service_letters('RADIO',-1.805,2.04,.19,'cream')
 for o in set(COL.objects)-letters:o.location.x-=.45


def battle_tower():
 start('battle_tower',10,6)
 def tower_roof(w,d,z):
  before=set(COL.objects)
  flat_roof(w,d,z,'roof_blue')
  for obj in list(set(COL.objects)-before):
   if obj.name.startswith(('Long parapet coping','Side parapet coping')):
    bpy.data.objects.remove(obj,do_unlink=True)
  # One closed annular coping: mitered corners without intersecting end caps.
  outer=[(-w/2,-d/2),(w/2,-d/2),(w/2,d/2),(-w/2,d/2)]
  inner=[(-w/2+.11,-d/2+.11),(w/2-.11,-d/2+.11),(w/2-.11,d/2-.11),(-w/2+.11,d/2-.11)]
  vs=[(x,y,h) for h in [z+.09,z+.37] for ring in [outer,inner] for x,y in ring]
  faces=[]
  for i in range(4):
   j=(i+1)%4
   faces.extend([(i,j,4+j,4+i),(8+j,8+i,12+i,12+j),(i,8+i,8+j,j),(4+j,12+j,12+i,4+i)])
  mesh('Continuous tournament roof coping',vs,faces,'roof_blue_light')

 foundation(8.7,4.7)
 # Separate wings leave a genuine recessed central entrance and glazing bay.
 for side in [-1,1]:
  cube('Tournament wing masonry',(side*2.94,0,3.70),(2.66,4.56,6.90),'plaster_light',.035)
 cube('Recessed central tower body',(0,.27,3.85),(3.22,4.20,7.2),'roof_blue_dark',.025)
 for side in [-1,1]:
  x=side*3.90
  cube('Coral tournament side pier',(x,.12,3.60),(1.05,4.75,7.20),'tower_coral',.15)
  for z in [1.9,3.5,5.1,6.7]:
   cube('Tournament side pier indigo belt',(x,.12,z),(1.16,4.85,.16),'roof_blue_dark',.025)
  for z in [1.18,2.86,4.54,6.22]:
   panel('Broad tournament wing glazing',side*2.60,z,1.68,1.18,-2.325,'glass_tower_deep')
  for angle in [90,-90]:
   if (angle==90)!=(side==1):continue
   for z in [1.18,2.86,4.54,6.22]:
    for xx in [-1.10,1.10]:panel('Tournament side glazing',xx,z,1.18,1.16,-4.47,'glass',angle)
  for z in [1.18,2.86,4.54,6.22]:
   panel('Tournament rear glazing',side*2.72,z,1.46,1.16,-2.325,'glass',180)
 # Convex, faceted curtain wall: each pane is a closed shallow solid.
 points=[]
 for i in range(9):
  a=math.radians(-60+i*15);points.append((math.sin(a)*1.72,-1.1-math.cos(a)*1.72))
 def bow_strip(name,z0,z1,mat,depth=.075):
  for i,(a,b) in enumerate(zip(points,points[1:])):
   c=(b[0]*.97,b[1]+depth);d=(a[0]*.97,a[1]+depth)
   vs=[(x,y,z) for z in [z0,z1] for x,y in [a,b,c,d]]
   mesh(name,vs,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)],mat)
 for lower,upper in [(2.40,3.71),(3.80,5.11),(5.20,6.51),(6.60,7.93)]:
  bow_strip('Convex cyan tournament curtain wall',lower,upper,'glass_tower_cyan')
 for z in [2.35,3.73,5.13,6.53,7.95]:
  bow_strip('Curved curtain wall floor ribbon',z,z+.065,'stone_light',.11)
 for x,y in points:
  cube('Curtain wall vertical mullion',(x,y-.012,5.16),(.042,.055,5.65),'stone_light',.008)
 # Lower wing roofs frame the taller glazed core instead of one office slab.
 for side in [-1,1]:
  before=set(COL.objects)
  tower_roof(3.15,5.3,7.18)
  for obj in set(COL.objects)-before:obj.location.x+=side*3.10
 tower_roof(3.4,5.3,7.98)
 bow_strip('Raised curved tournament crown',7.98,8.18,'roof_blue_light',.16)
 # Both native entrance slots meet at the centre, underneath one canopy.
 # Carve a real shallow vestibule instead of fitting glass to either side of
 # an opaque central wall. The original podium, plot and threshold stay fixed.
 carve_window_pocket(COL,0,1.12,2.80,1.82,-2.325)
 for x in [-.66,.66]:
  facade_cube('Battle Tower paired sliding glass',x,1.07,1.28,1.62,-2.20,.045,'glass_dark')
  facade_cube('Battle entrance glass reflection',x-.28,1.53,.30,.045,-2.232,.014,'glass_light')
  facade_cube('Battle entrance lower reflection',x+.29,.67,.14,.035,-2.232,.014,'glass')
 for x in [-1.36,0,1.36]:
  facade_cube('Battle entrance recessed jamb',x,1.07,.065,1.75,-2.29,.14,'stone_light',bevel=.008)
 for z in [.24,1.90]:
  facade_cube('Battle entrance recessed rail',0,z,2.78,.07,-2.29,.14,'stone_light',bevel=.008)
 for x in [-.12,.12]:
  facade_cube('Battle entrance pull handle',x,.98,.035,.28,-2.31,.065,'stone_light',bevel=.008)
 cube('Battle broad entrance podium',(0,-2.65,.11),(4.0,.70,.22),'stone_light',.035)
 # A broad clipped canopy frames the tournament entry instead of a shop awning.
 outline=[(-3.25,-2.25),(-2.65,-2.95),(2.65,-2.95),(3.25,-2.25),(3.25,-1.95),(-3.25,-1.95)]
 vs=[(x,y,z) for z in [2.12,2.30] for x,y in outline]
 faces=[tuple(range(5,-1,-1)),tuple(range(6,12))]+[(i,(i+1)%6,(i+1)%6+6,i+6) for i in range(6)]
 mesh('Angled tournament entrance canopy',vs,faces,'stone_light')
 for x in [-1.65,1.65]:
  cube('Coral portal pier',(x,-2.40,1.02),(.20,.26,1.96),'tower_coral',.025)
 for x in [-2.67,2.67]:
  facade_cube('Tournament seal recessed panel',x,1.15,.70,.72,-2.45,.12,'tower_coral',bevel=.025)
  for radius,depth,y,mat in [(.255,.035,-2.53,'stone_light'),(.195,.025,-2.56,'roof_blue_dark')]:
   o=cone('Tournament seal solid roundel',(x,y,1.15),radius,radius,depth,mat,16);o.rotation_euler.x=math.pi/2
  facade_cube('Tournament seal crossbar',x,1.15,.37,.06,-2.59,.025,'stone_light')
  o=cone('Tournament seal central boss',(x,-2.61,1.15),.065,.065,.025,'stone_light',12);o.rotation_euler.x=math.pi/2


def barn():
 start('johto_barn',5.2,3.8);full_shell(4.7,3.1,2.25,wall='roof_red_warm',cols=3)
 roof(5.15,3.5,2.54,3.72,'roof_red',5);front_entry(3.1,1.3)
 for x in [-.37,.37]:
  facade_cube('Barn double plank door',x,1.03,.70,1.55,-1.76,.07,'timber_light')
  for dx in [-.25,0,.25]:facade_cube('Barn door plank seam',x+dx,1.03,.025,1.5,-1.805,.025,'timber')
  beam('Barn door diagonal brace',(x-.29,-1.84,.32),(x+.29,-1.84,1.74),.075,.05,'plaster_light')
 facade_cube('Barn hayloft shutter',0,2.54,.74,.53,-1.61,.10,'timber')


def shrine():
 start('forest_shrine',2.7,2.5);foundation(2.3,1.9)
 cube('Shrine dark inner chamber',(0,0,1.13),(1.8,1.56,1.82),'timber',.025)
 for x in [-.93,.93]:
  for y in [-.83,.83]:cube('Shrine cedar pillar',(x,y,1.23),(.18,.18,2.03),'timber_light')
 roof(2.63,2.35,2.20,3.08,'roof_teal',5)
 for x in [-.38,.38]:
  facade_cube('Shrine lattice door',x,1.13,.70,1.4,-.84,.06,'plaster_shadow')
  for dx in [-.25,-.125,0,.125,.25]:facade_cube('Shrine vertical cedar lattice',x+dx,1.13,.036,1.4,-.89,.025,'timber_light')
 cube('Shrine low approach step',(0,-1.02,.12),(1.54,.41,.24),'stone_light',.02)
 rod('Shrine bell rope',(0,-1.13,1.1),(0,-1.13,2.34),.04,'cream')
 ico('Shrine bronze bell',(0,-1.10,2.30),(.18,.18,.20),'yellow',2)


def forest_gate_roof():
 # Preserve the closed deck, ridge and structural edges. The source's small
 # roof courses need overlapping ceramic pieces, rather than seven broad slabs.
 width,depth,eave,ridge=5.95,5.72,3.22,4.48
 roof(width,depth,eave,ridge,'roof_teal',6)
 for o in list(COL.objects):
  if o.name.startswith('Individual roof course'):
   bpy.data.objects.remove(o,do_unlink=True)
 columns,rows=17,12
 half=width/2;run=depth/2;step=width/columns
 for side in [-1,1]:
  for row in range(rows):
   # Alternate half pieces close the verge; staggered seams never form gutters.
   cuts=[-half]+[-half+(j+(0.5 if row%2 else 1))*step
    for j in range(columns) if -half+(j+(0.5 if row%2 else 1))*step<half]+[half]
   t0=max(0.,row/rows-0.012);t1=(row+1)/rows
   y0=side*run*(1-t0);y1=side*run*(1-t1)
   z0=eave+(ridge-eave)*t0+.035;z1=eave+(ridge-eave)*t1+.020
   for column,(left,right) in enumerate(zip(cuts,cuts[1:])):
    # Adjacent closed pieces meet at the verge instead of exposing dark slots.
    # The visible crown is unchanged; a triangular-prism underside eliminates
    # hidden parallel layers and leaves geometry budget for the masonry.
    vs=[]
    for y,z in [(y0,z0),(y1,z1)]:
     for t,lift in [(0,0),(.5,.009),(1,0)]:
      vs.append((left+(right-left)*t,y,z+lift))
    faces=[(0,1,4,3),(1,2,5,4),(0,3,5,2),(0,2,1),(3,4,5)]
    color=['roof_teal','roof_teal_warm','roof_teal','roof_teal_light'][(column+row*3)%4]
    mesh('Forest gate overlapping ceramic shingle',vs,faces,color)

def forest_gate_shell(closed):
 w,d,h=5.48,5.25,2.95
 if 'plaster_forest_mortar' not in MATS:
  m=MATS['plaster_light'].copy();m.name='plaster_forest_mortar';MATS[m.name]=m
 foundation(w,d)
 cube('Complete forest masonry body',(0,0,.25+h/2),(w-.14,d-.14,h),'plaster_forest_mortar',.025)
 for x in [-w/2+.08,w/2-.08]:
  for y in [-d/2+.08,d/2-.08]:
   cube('Forest masonry corner pier',(x,y,.25+h/2),(.17,.17,h+.05),'plaster_light',.018)
 for angle,span,face in [(0,w-.34,-(d-.14)/2),(180,w-.34,-(d-.14)/2),
   (90,d-.34,-(w-.14)/2),(-90,d-.34,-(w-.14)/2)]:
  step=span/8
  for row in range(12):
   cuts=[-span/2]+[-span/2+(j+(.5 if row%2 else 1))*step
    for j in range(8) if -span/2+(j+(.5 if row%2 else 1))*step<span/2]+[span/2]
   for column,(left,right) in enumerate(zip(cuts,cuts[1:])):
    z=.43+(row+.5)*2.71/12
    depth=.024+.002*((row+column)%3)
    segments=[(left,right)]
    if not closed and angle==0 and z<1.87:
     # Cut actual half bricks against the jamb, rather than omitting whole
     # columns and leaving broad unpainted mortar stripes beside the door.
     segments=[(left,min(right,-2.10)),(max(left,-.90),right)]
    for a,b in segments:
     if b-a>.02:
      facade_cube('Forest running-bond masonry',(a+b)/2,z,b-a-.014,
       2.71/12-.014,face-.005,depth,'plaster',angle)
  window_span=w if angle in [0,180] else d
  for x in [-window_span/3,0,window_span/3]:
   z=2.48;width=1.04;height=.57;y=face-.052
   facade_cube('Forest recessed window frame',x,z,width+.12,height+.12,y,.10,'stone_dark',angle)
   pane=(width-.10-.035*3)/4
   for part in range(4):
    dx=-width/2+.05+pane/2+part*(pane+.035)
    facade_cube('Forest yellow window pane',x+dx,z,pane,height-.10,y-.058,.016,'glass',angle)
   facade_cube('Forest dressed window sill',x,z-height/2-.065,width+.20,.07,y-.014,.18,'plaster_light',angle)

def forest_gate_entry():
 # The source has one left-side door with a horizontal upper light, not
 # centered laboratory double doors or a striped entrance canopy.
 for name,parent in [('forest_gate_door','timber'),('forest_gate_door_frame','timber_light'),
   ('forest_gate_door_panel','timber')]:
  if name not in MATS:
   m=MATS[parent].copy();m.name=name;MATS[name]=m
 x=-1.5
 for dx in [-.52,.52]:
  cube('Forest source door jamb',(x+dx,-2.73,1.005),(.09,.14,1.61),'forest_gate_door_frame')
 cube('Forest source door header',(x,-2.73,1.765),(1.13,.14,.09),'forest_gate_door_frame')
 cube('Forest source single door',(x,-2.74,.96),(.95,.03,1.52),'forest_gate_door')
 cube('Forest source upper light recess',(x,-2.76,1.34),(.86,.018,.40),'stone_dark')
 cube('Forest source horizontal door light',(x,-2.772,1.34),(.78,.01,.30),'glass')
 cube('Forest source lower door panel',(x,-2.76,.50),(.75,.015,.38),'forest_gate_door_panel')
 cube('Forest source entry threshold',(x,-2.755,.10),(1.30,.30,.20),'stone_light',.014)

def forest_gate(closed=False):
 name='forest_gate_closed' if closed else 'forest_gate'
 start(name,6.0,6.0,'none' if closed else 'south')
 forest_gate_shell(closed)
 forest_gate_roof()
 building_trim(5.50,5.27,1.63,2.70)
 for o in list(COL.objects):
  if o.name.startswith('Timber corner upright'):bpy.data.objects.remove(o,do_unlink=True)
 if not closed:
  forest_gate_entry()

def nature(name,kind):
 start(name,2,{'sign':.32,'bench':1.05,'fence':.30}.get(kind,2),'none')
 if kind in ['canopy','cut','hedge']:
  if kind=='canopy':
   cone('Tapered tree trunk',(0,0,.72),.18,.11,1.44,'timber_light',7)
   for a in [0,2.1,4.2]:rod('Visible trunk root',(0,0,.22),(math.cos(a)*.52,math.sin(a)*.52,.045),.09,'timber',6)
  if kind=='canopy':
   for i,(x,y,z,s) in enumerate([(-.52,0,1.78,.72),(.52,0,1.86,.75),(0,.46,2.14,.80),(0,-.44,2.13,.83),(0,0,2.58,.72)]):
    rod('Bough',(0,0,.95),(x,y,z),.075,'timber_light');ico('Faceted broadleaf crown',(x,y,z),(s,s*.86,s),['leaf','leaf_light','leaf_dark'][i%3],2)
  elif kind=='cut':
   # A small, visibly slender sapling: the open fork and three pointed leaf
   # tufts distinguish the CUT obstacle from rounded grove/fruit trees.
   cone('Cuttable exposed slender trunk',(0,0,.65),.14,.085,1.30,'cut_bark',7)
   for a in [0,2.1,4.2]:
    rod('Cuttable short root',(0,0,.13),(math.cos(a)*.29,math.sin(a)*.29,.035),.055,'cut_bark',6)
   for i,(x,y,z) in enumerate([(-.38,0,1.55),(.38,.04,1.61),(0,-.12,1.98)]):
    rod('Cuttable exposed fork',(0,0,.83),(x,y,z-.12),.065,'cut_bark',6)
    ico('Cuttable pointed leaf tuft',(x,y,z),(.43,.39,.52),['cut_leaf_dark','cut_leaf','cut_leaf_light'][i],1)
  else:
   cube('Clipped hedge dense center',(0,0,.51),(1.65,1.50,.9),'leaf_dark',.20)
   for x in [-.52,0,.52]:
    for y in [-.42,.42]:ico('Rounded hedge foliage',(x,y,.79),(.45,.43,.37),'leaf' if x else 'leaf_light',1)
 elif kind=='boulder':
  ico('Stratified shore boulder',(0,0,.70),(.94,.78,.83),'stone',2)
  ico('Boulder granite cap',(-.1,.03,1.27),(.64,.61,.27),'stone_light',1)
  for x,y in [(-.55,-.48),(.56,.27),(.32,-.56)]:ico('Low boulder foot',(x,y,.22),(.32,.28,.24),'stone_dark',1)
 elif kind=='sign':
  for x in [-.47,.47]:cube('Sign post',(x,0,.73),(.14,.15,1.46),'timber_light',.02)
  # A compact notice panel, matching the native square sign rather than a
  # long bench-like backrest. Keep its existing posts, inset and lettering.
  # Scale the authored panel, including its bevels, exactly as the runtime
  # source mesh was resized. Regeneration must retain that edge treatment.
  board=cube('Beveled signboard',(0,0,1.32),(1.56,.21,.72),'timber',.07);board.scale.z=5/3
  enamel=cube('Recessed sign enamel',(0,-.118,1.32),(1.33,.03,.52),'cream',.035);enamel.scale.z=5/3
  for z,w in [(1.5533333,.89),(1.3033333,1.02),(1.0533333,.68)]:
   text=cube('Raised sign text line',(-.04,-.142,z),(w,.017,.035),'timber_light');text.scale.z=5/3
  for x in [-.61,.61]:
   fixing=ico('Sign brass fixing',(x,-.147,1.32),(.033,.024,.033),'yellow',1);fixing.scale.z=5/3
 elif kind=='bench':
  for x in [-.70,.70]:
   for y in [-.38,.36]:cube('Bench cast leg',(x,y,.37),(.12,.13,.73),'park_frame',.015)
   rod('Bench back support',(x,.35,.4),(x,.47,1.37),.052,'park_frame',6)
  for y in [-.36,-.12,.12,.36]:cube('Bench seat plank',(0,y,.74),(1.88,.17,.11),'park_slats_light',.017)
  for z in [1.00,1.22,1.44]:cube('Bench back plank',(0,.44,z),(1.92,.10,.15),'park_slats',.019)
  for x in [-.82,.82]:cube('Bench armrest',(x,-.02,1.03),(.11,.84,.09),'park_frame',.02)
 elif kind=='fence':
  for x in [0]:
   cube('Buried square fence post',(x,0,.66),(.38,.28,1.32),'timber_light',.024)
   cube('Worn flat end grain',(x,0,1.319),(.34,.24,.012),'timber_gold',.004)
   # Narrow longitudinal checks follow the grain, rather than crosswise stripes.
   for dx,z,h in [(-.045,.62,.72),(.039,.91,.46)]:
    cube('Post weathering check',(x+dx,-.141,z),(.008,.006,h),'timber',.002)
  for z in [.37,.88]:
   cube('Tenoned horizontal rail',(0,.025,z),(2,.14,.19),'timber_gold',.018)
   for x in [0]:
    # Flush dowels secure the two rail mortises without floating metal discs.
    peg=cone('Rail joint timber peg',(x,-.146,z),.022,.022,.012,'timber',8)
    peg.rotation_euler.x=math.pi/2
   for x,length,dz in [(-.43,.49,.033),(.39,.61,-.037)]:
    cube('Restrained rail grain',(x,-.048,z+dz),(length,.004,.006),'timber_light',.001)

builders={
'forest_gate':forest_gate,'forest_gate_closed':lambda:forest_gate(True),'east_gate':east_gate,'tin_tower':lambda:pagoda('tin_tower'),'burned_tower':lambda:pagoda('burned_tower',True),'lighthouse':lighthouse,'johto_barn':barn,'forest_shrine':shrine,'radio_tower':radio_tower,'battle_tower':battle_tower,
'modern_house':lambda:townhouse('modern_house','flat'),'modern_blank':lambda:townhouse('modern_blank','blank'),'modern_shop':lambda:townhouse('modern_shop','compact'),
'kanto_house':lambda:townhouse('kanto_house','pitched'),'kanto_flat_house':lambda:townhouse('kanto_flat_house','flat'),'kanto_blank':lambda:townhouse('kanto_blank','blank')}
for prefix in ['modern','kanto']:
 for style in ['center','mart','gym','station','department','arcade']:
  name=prefix+'_'+style;builders[name]=lambda n=name,s=style,p=prefix:civic(n,s,w=5.45 if p=='modern' else 5.9,d=3.3 if p=='modern' else 3.65)
builders['modern_gym']=modern_gym
for name,style,w,d in [('silph','silph',5.6,3.8),('kanto_mansion','mansion',5.3,3.8),('power_plant','power',7.6,4.3),('modern_daycare','house',4.4,3.6)]:builders[name]=lambda n=name,s=style,w=w,d=d:civic(n,s,w,d)
for name,kind in [('kanto_canopy','canopy'),('cut_tree','cut'),('park_hedge','hedge'),('shore_boulder','boulder'),('route_sign','sign'),('park_bench','bench'),('timber_fence','fence')]:builders[name]=lambda n=name,k=kind:nature(n,k)
only=next((a.split('=',1)[1].split(',') for a in args if a.startswith('--only=')),None)
for name,build in builders.items():
 if only is None or name in only:build()

# Export original geometry, grouped by named material; never reads a game pack.
STATS={}
for name,col in list(ASSETS.items()):
 bpy.context.view_layer.update();objects=[o for o in col.objects if o.type=='MESH']
 for o in objects:
  o.data.transform(o.matrix_world);o.matrix_world=Matrix.Identity(4)
 xs=[v.co.x for o in objects for v in o.data.vertices];ys=[v.co.y for o in objects for v in o.data.vertices]
 xmin,xmax=min(xs),max(xs);ymin,ymax=min(ys),max(ys);tw,td=TARGETS[name]
 for o in objects:
  for v in o.data.vertices:
   if name=='lighthouse':
    # The doorway projects beyond the round plinth. Never normalize its
    # asymmetric whole-object bounds independently and squash the cylinder.
    radial_scale=tw/(xmax-xmin)
    v.co.x*=radial_scale;v.co.y*=radial_scale
   else:
    v.co.x=(v.co.x-(xmin+xmax)/2)*tw/(xmax-xmin);v.co.y=(v.co.y-(ymin+ymax)/2)*td/(ymax-ymin)
  o.data.update()
 groups={};minv=[float('inf')]*3;maxv=[-float('inf')]*3
 for o in objects:
  bm=bmesh.new();bm.from_mesh(o.data);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(o.data);bm.free();o.data.update()
  me=o.data;me.calc_loop_triangles();mat=me.materials[0]
  rigid=bool(o.get('rigid_roof',False))
  g=groups.setdefault((mat.name,rigid),{'name':name+'__'+mat.name+('__roof' if rigid else ''),'material':mat.name,'base_color':[round(v,5) for v in mat.diffuse_color],'positions':[],'normals':[],'indices':[],'lookup':{}})
  if rigid:g['rigid_roof']=True
  for tri in me.loop_triangles:
   if tri.normal.length_squared<.9:continue
   # Clipped window pockets contain tiny slivers. Quantization can reverse
   # their signed area; preserve outward winding in the exported coordinates.
   order=list(tri.vertices)
   rounded=[Vector(tuple(round(c,6) for c in me.vertices[vi].co)) for vi in order]
   if (rounded[1]-rounded[0]).cross(rounded[2]-rounded[0]).dot(tri.normal)<0:
    order.reverse()
   for vi in order:
    v=me.vertices[vi].co;n=tri.normal
    if o.get('radial_normals',False) and abs(n.z)<.25:
     radial=Vector((v.x,v.y,0.));radius=radial.length
     expected=(1.45-(v.z-.28)*(.35/7.2))*radial_scale
     # Smooth only the original curved exterior. Recessed backs, horizontal
     # caps and cut window jambs keep their authored hard face normals.
     if radius>0 and abs(radius-expected)<.015 and Vector((n.x,n.y,0.)).normalized().dot(radial.normalized())>.985:
      n=Vector((v.x/radius,v.y/radius,.35*radial_scale/7.2)).normalized()
    pos=(round(v.x,6),round(v.z,6),round(-v.y,6));norm=(round(n.x,6),round(n.z,6),round(-n.y,6));key=pos+norm
    for k in range(3):minv[k]=min(minv[k],pos[k]);maxv[k]=max(maxv[k],pos[k])
    if key not in g['lookup']:g['lookup'][key]=len(g['positions'])//3;g['positions'].extend(pos);g['normals'].extend(norm)
    g['indices'].append(g['lookup'][key])
 primitives=[]
 for g in groups.values():g.pop('lookup');primitives.append(g)
 data={'name':name,'coordinate_system':'right-handed; +Y up; front +Z','bounds':{'min':minv,'max':maxv},'primitives':primitives}
 if DOORS[name]=='south':
  along=0.
  if name in ['forest_gate','radio_tower']:
   threshold=next(o for o in objects if o.name.startswith('Forest source entry threshold' if name=='forest_gate' else 'Entrance threshold'))
   along=round(sum(v.co.x for v in threshold.data.vertices)/len(threshold.data.vertices),6)
  data['door_anchor']=[along,0,maxv[2]];data['door_face']='south'
 elif DOORS[name]=='east':data['door_anchor']=[maxv[0],0,1. if name=='east_gate' else 0.];data['door_face']='east'
 if name=='east_gate':
  # Clear passage edges come from the inner jamb faces after plot fitting.
  spans=[(-max(v.co.y for v in o.data.vertices),-min(v.co.y for v in o.data.vertices))
      for o in objects if o.name.startswith('Passage jamb')]
  middle=sum((a+b)/2 for a,b in spans)/len(spans)
  data['door_aperture']=[max(b for a,b in spans if (a+b)/2<middle),
      min(a for a,b in spans if (a+b)/2>middle)]
 (P/f'{name}.mesh.json').write_text(json.dumps(data,separators=(',',':')))
 triangles=sum(len(g['indices'])//3 for g in primitives)
 # Five storeys of closed window pockets add real reveal surfaces.
 assert triangles<(13000 if name=='tin_tower' else 12000),(name,triangles)
 STATS[name]={'triangles':triangles,'parts':len(objects),'bounds':data['bounds'],'door_face':DOORS[name]}
# Editable collection lineup, after exports so presentation offsets never enter runtime.
for i,(name,col) in enumerate(ASSETS.items()):
 for o in col.objects:o.location+=Vector(((i%7)*12-36,(i//7)*13,0))
begin('PRESENTATION only');cube('Neutral model workshop floor',(0,30,-.16),(160,160,.25),'cream')
world=bpy.data.worlds.new('Exterior kit day');scene.world=world;world.use_nodes=True
world.node_tree.nodes['Background'].inputs['Color'].default_value=(.36,.42,.48,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.65
for name,loc,energy,size in [('Key',(-40,-20,60),35000,35),('Fill',(40,20,50),25000,30)]:
 bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name=name;o.data.energy=energy;o.data.size=size;o.rotation_euler=(Vector((0,25,0))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(60,-75,100));cam=bpy.context.object;cam.rotation_euler=(Vector((0,30,2))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=110;scene.camera=cam
scene.render.engine='CYCLES';scene.cycles.samples=16;scene.render.resolution_x=2200;scene.render.resolution_y=1600;scene.render.resolution_percentage=100;scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.filepath=str(P/'world-exteriors-preview.png')
scene['Source']='Original named modeling parts; no game assets or textures imported.'
bpy.ops.wm.save_as_mainfile(filepath=str(P/('world-exteriors-'+(only[0] if only else 'all')+'.blend')),compress=True)
(P/('asset-stats-'+(only[0] if only else 'all')+'.json')).write_text(json.dumps(STATS,indent=2))
if '--skip-preview' not in args:bpy.ops.render.render(write_still=True)
print('WORLD_EXTERIORS='+json.dumps(STATS))
