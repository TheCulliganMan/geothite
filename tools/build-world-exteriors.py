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

def full_shell(w,d,h,wall='plaster_light',floors=1,cols=3):
 foundation(w,d);cube('Complete four-sided wall body',(0,0,.25+h/2),(w-.14,d-.14,h),wall,.025)
 for x in [-w/2+.08,w/2-.08]:
  for y in [-d/2+.08,d/2-.08]:cube('Corner pilaster',(x,y,.25+h/2),(.17,.17,h+.05),'stone_light',.018)
 for floor in range(floors):
  z=.25+(floor+.5)*h/floors
  for angle,span,fy in [(0,w,-d/2+.025),(180,w,-d/2+.025),(90,d,-w/2+.025),(-90,d,-w/2+.025)]:
   count=cols if angle in [0,180] else max(1,round(d/1.35))
   for i in range(count):
    x=(i-(count-1)/2)*span/(count+.35)
    if angle==0 and floor==0 and abs(x)<.6:continue
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

def civic(name,style,w=5.5,d=3.5):
 start(name,w+.5,d+.6)
 floors=3 if style in ['department','silph','mansion'] else 1
 h={'department':5.8,'silph':7.0,'mansion':4.8,'station':2.6,'gym':2.7,'power':3.2}.get(style,2.35)
 full_shell(w,d,h,wall='plaster' if style in ['mansion'] else 'plaster_light',floors=floors,cols=4 if w>5 else 3)
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
 town_roof(3.93,3.94,2.35,3.38,'roof_blue',gable_material='plaster_light')
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


def pagoda(name,burnt=False):
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
 start('lighthouse',4,3.2)
 cone('Octagonal lighthouse plinth',(0,0,.20),1.7,1.7,.4,'stone',12)
 cone('Tapered white lighthouse shaft',(0,0,3.88),1.45,1.10,7.2,'plaster_light',12)
 for z,r in [(1.0,1.43),(2.6,1.34),(4.3,1.26),(6.0,1.18)]:
  cone('Lighthouse masonry belt',(0,0,z),r+.025,r+.025,.12,'plaster',12)
  for a in [0,90,180,270]:panel('Lighthouse slit window',0,z+.68,.36,.67,-r-.01,angle=a)
 cone('Lantern balcony floor',(0,0,7.62),1.7,1.7,.22,'stone_light',16)
 for i in range(16):
  a=i*math.tau/16;rod('Lantern balcony railing',(math.cos(a)*1.55,math.sin(a)*1.55,7.74),(math.cos(a)*1.55,math.sin(a)*1.55,8.18),.035,'timber',6)
 cone('Lantern handrail',(0,0,8.20),1.61,1.61,.07,'timber',16)
 cone('Lantern glass drum',(0,0,8.46),.95,.95,1.46,'glass',12)
 for i in range(8):
  a=i*math.tau/8;rod('Lantern iron glazing bar',(math.cos(a)*.96,math.sin(a)*.96,7.73),(math.cos(a)*.96,math.sin(a)*.96,9.19),.055,'roof_blue_dark',6)
 ico('Warm lantern lens',(0,0,8.48),(.55,.55,.63),'yellow',2)
 cone('Lantern conical cap',(0,0,9.44),1.18,.14,.65,'roof_blue_dark',12)
 rod('Lightning spike',(0,0,9.7),(0,0,10.3),.045,'yellow')
 front_entry(2.91,.71)


def radio_tower():
 start('radio_tower',4,4);full_shell(3.5,3.25,5.35,floors=3,cols=3);front_entry(3.25,.95)
 for z in [2.10,3.70,5.52]:cube('Broadcast tower projecting floor',(0,0,z),(3.76,3.5,.17),'roof_blue_dark',.025)
 flat_roof(3.80,3.55,5.6,'roof_blue')
 # Triangulated antenna mast, visibly open rather than a solid spike.
 for x,y in [(-.45,-.3),(.45,-.3),(0,.48)]:rod('Triangular radio mast',(x,y,5.75),(x*.3,y*.3,8.9),.045,'stone_dark')
 for level in range(5):
  z=5.9+level*.58;s=1-(z-5.75)/4.5
  pts=[(-.45*s,-.3*s),(.45*s,-.3*s),(0,.48*s)]
  for i in range(3):
   a=pts[i];b=pts[(i+1)%3];rod('Antenna cross brace',(a[0],a[1],z),(b[0]*.88,b[1]*.88,z+.52),.028,'stone_light')
 rod('Broadcast aerial',(0,0,8.7),(0,0,9.65),.035,'stone_dark')
 ico('Aerial warning lamp',(0,0,9.69),(.10,.10,.12),'coral',1)
 facade_cube('Radio station sign',0,2.04,1.7,.30,-1.72,.12,'violet_dark')


def battle_tower():
 start('battle_tower',10,6);full_shell(6.6,4.7,7.2,floors=4,cols=5)
 for x in [-3.68,3.68]:
  cube('Battle Tower curved side pier',(x,.12,3.9),(1.10,4.75,7.8),'stone_light',.20)
  for z in [1.9,3.5,5.1,6.7]:cube('Side pier indigo belt',(x,.12,z),(1.18,4.85,.20),'roof_blue_dark',.035)
 flat_roof(9.35,5.3,7.98,'roof_blue')
 for x in [-1.35,1.35]:
  facade_cube('Battle Tower double glazed entry',x,1.03,1.21,1.60,-2.42,.11,'glass_dark')
  for dx in [-.5,0,.5]:facade_cube('Battle entrance mullion',x+dx,1.03,.055,1.64,-2.5,.05,'plaster_light')
 cube('Battle broad entrance podium',(0,-2.65,.11),(4.0,.70,.22),'stone_light',.035)
 awning(4.4,4.7,2.08,'roof_blue')
 facade_cube('Battle Tower tournament crest',0,7.3,1.22,.55,-2.49,.16,'roof_blue_dark',bevel=.05)
 ico('Tournament golden emblem',(0,-2.6,7.31),(.24,.07,.24),'yellow',2)


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
   v.co.x=(v.co.x-(xmin+xmax)/2)*tw/(xmax-xmin);v.co.y=(v.co.y-(ymin+ymax)/2)*td/(ymax-ymin)
  o.data.update()
 groups={};minv=[float('inf')]*3;maxv=[-float('inf')]*3
 for o in objects:
  bm=bmesh.new();bm.from_mesh(o.data);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(o.data);bm.free();o.data.update()
  me=o.data;me.calc_loop_triangles();mat=me.materials[0]
  g=groups.setdefault(mat.name,{'name':name+'__'+mat.name,'material':mat.name,'base_color':[round(v,5) for v in mat.diffuse_color],'positions':[],'normals':[],'indices':[],'lookup':{}})
  for tri in me.loop_triangles:
   if tri.normal.length_squared<.9:continue
   for vi in tri.vertices:
    v=me.vertices[vi].co;n=tri.normal;pos=(round(v.x,6),round(v.z,6),round(-v.y,6));norm=(round(n.x,6),round(n.z,6),round(-n.y,6));key=pos+norm
    for k in range(3):minv[k]=min(minv[k],pos[k]);maxv[k]=max(maxv[k],pos[k])
    if key not in g['lookup']:g['lookup'][key]=len(g['positions'])//3;g['positions'].extend(pos);g['normals'].extend(norm)
    g['indices'].append(g['lookup'][key])
 primitives=[]
 for g in groups.values():g.pop('lookup');primitives.append(g)
 data={'name':name,'coordinate_system':'right-handed; +Y up; front +Z','bounds':{'min':minv,'max':maxv},'primitives':primitives}
 if DOORS[name]=='south':
  along=0.
  if name=='forest_gate':
   threshold=next(o for o in objects if o.name.startswith('Forest source entry threshold'))
   along=round(sum(v.co.x for v in threshold.data.vertices)/len(threshold.data.vertices),6)
  data['door_anchor']=[along,0,maxv[2]];data['door_face']='south'
 elif DOORS[name]=='east':data['door_anchor']=[maxv[0],0,1. if name=='east_gate' else 0.];data['door_face']='east'
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
