"""Original connected-Johto environment kit, no imported game artwork.
blender -b --python tools/build-connected-johto-models.py -- target/connected-johto-assets [--skip-preview]
Uses the New Bark kit's authored modeling vocabulary. Source Z-up/front -Y;
exports +Y-up/front +Z with a floor-centered origin and explicit threshold.
"""
from pathlib import Path
# Load reusable geometric authoring helpers without generating the original kit.
source = Path(__file__).with_name('build-new-bark-models.py').read_text()
assert '\nbuilders=' in source,'New Bark authoring vocabulary boundary is missing'
exec(compile(source.split('\nbuilders=',1)[0], 'new_bark_modeling_vocabulary', 'exec'))
from mathutils import Matrix
for n,c in {'roof_blue':(.18,.34,.55),'roof_blue_light':(.31,.48,.69),'roof_blue_dark':(.11,.22,.37),'roof_blue_warm':(.23,.40,.61),'violet':(.38,.29,.45),'violet_light':(.53,.44,.59),'violet_dark':(.24,.19,.32),'violet_warm':(.44,.34,.49),'plaster_service_wall':(.98,.91,.73),'plaster_service_mortar':(.88,.86,.77),'medical_red':(.62,.13,.11),'medical_white':(.94,.92,.84),'shop_sign_blue':(.12,.28,.46)}.items():
 m=bpy.data.materials.new(n);m.diffuse_color=(*c,1);m.use_nodes=True
 m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(*c,1)
 m.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.86
 MATS[n]=m

def beam(n,a,b,w,d,mat):
 a=Vector(a);b=Vector(b);o=cube(n,(a+b)/2,(w,d,(b-a).length),mat,.012)
 o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o

def source_service_letters(text,y,z,height,material='ink'):
 # Original solid stroke lettering for storefront and landmark signs.
 # Each stroke is a closed prism; no font/texture dependency.
 glyphs={
  'M':[((0,0),(0,1)),((0,1),(.375,.48)),((.375,.48),(.75,1)),((.75,1),(.75,0))],
  'A':[((0,0),(.375,1)),((.375,1),(.75,0)),((.18,.42),(.57,.42))],
  'R':[((0,0),(0,1)),((0,1),(.75,1)),((.75,1),(.75,.55)),((.75,.55),(0,.55)),((.30,.55),(.75,0))],
  'T':[((0,1),(.75,1)),((.375,1),(.375,0))],
  'P':[((0,0),(0,1)),((0,1),(.75,1)),((.75,1),(.75,.55)),((.75,.55),(0,.55))],
  'O':[((0,0),(0,1)),((0,1),(.75,1)),((.75,1),(.75,0)),((.75,0),(0,0))],
  'D':[((0,0),(0,1)),((0,1),(.5,1)),((.5,1),(.75,.75)),((.75,.75),(.75,.25)),((.75,.25),(.5,0)),((.5,0),(0,0))],
  'I':[((0,1),(.75,1)),((.375,1),(.375,0)),((0,0),(.75,0))],
  'K':[((0,0),(0,1)),((0,.45),(.75,1)),((0,.45),(.75,0))],
  'G':[((.75,1),(0,1)),((0,1),(0,0)),((0,0),(.75,0)),((.75,0),(.75,.5)),((.75,.5),(.40,.5))],
  'E':[((0,0),(0,1)),((0,1),(.75,1)),((0,.5),(.60,.5)),((0,0),(.75,0))],
  'Y':[((0,1),(.375,.5)),((.75,1),(.375,.5)),((.375,.5),(.375,0))],
  'é':[((0,.38),(.75,.38)),((.75,.38),(.75,.72)),((.75,.72),(0,.72)),((0,.72),(0,0)),((0,0),(.75,0)),((.28,.93),(.59,1.20))],
 }
 advance=.98;left=-((len(text)-1)*advance+.75)*height/2
 for index,char in enumerate(text):
  for a,b in glyphs[char]:
   a=Vector((left+(index*advance+a[0])*height,y,z+(a[1]-.5)*height))
   b=Vector((left+(index*advance+b[0])*height,y,z+(b[1]-.5)*height))
   o=cube('Source service letter '+char,(a+b)/2,(height*.105,.025,(b-a).length),material)
   o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler()

def civic_brick(name,outline,distance,angle):
 # Closed shallow frustum: its four sloping sides supply restrained arrises
 # without a bevel modifier or extra triangles on every course.
 center=(sum(p[0] for p in outline)/len(outline),sum(p[1] for p in outline)/len(outline))
 front=[]
 for u,z in outline:
  dx,dz=center[0]-u,center[1]-z;length=math.hypot(dx,dz)
  inset=min(.004,length*.10)/max(length,.000001)
  front.append((u+dx*inset,z+dz*inset))
 n=len(outline)
 vs=[(u,-distance-.001,z) for u,z in outline]+[(u,-distance-.023,z) for u,z in front]
 faces=[tuple(range(n-1,-1,-1)),tuple(range(n,2*n))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
 o=mesh(name,vs,faces,'plaster_service_wall');o.rotation_euler.z=math.radians(angle)
 return o

def civic_bricks():
 # Eight smaller running-bond courses retain the exact wall coverage. Every
 # brick is a closed solid above the existing recessed live-painted mortar.
 for angle,span,distance,count in [(0,3.46,1.605,7),(180,3.46,1.605,7),(90,3.17,1.75,6),(-90,3.17,1.75,6)]:
  step=span/count
  for row in range(8):
   starts=[-span/2+i*step for i in range(count+1)] if row%2==0 else [-span/2]+[-span/2+(i+.5)*step for i in range(count)]+[span/2]
   for left,right in zip(starts,starts[1:]):
    left+=.009;right-=.009;bottom=.665+row*.195+.007;top=.665+(row+1)*.195-.007
    civic_brick('Fine civic running-bond brick',[(left,bottom),(right,bottom),(right,top),(left,top)],distance,angle)

def civic_gables():
 # Clip closed brick volumes to the original roof end-wall triangle. The
 # unchanged source-painted gable remains a continuous mortar backing.
 eave,ridge,half=2.46,3.35,1.85;slope=(ridge-eave)/half;step=2*half/7
 def clip(poly,sign):
  result=[]
  for a,b in zip(poly,poly[1:]+poly[:1]):
   da=ridge-.035-a[1]-sign*slope*a[0];db=ridge-.035-b[1]-sign*slope*b[0]
   if da>=0:result.append(a)
   if (da>=0)!=(db>=0):
    t=da/(da-db);result.append((a[0]+t*(b[0]-a[0]),a[1]+t*(b[1]-a[1])))
  return result
 for angle in [-90,90]:
  for row in range(5):
   starts=[-half+i*step for i in range(8)] if row%2==0 else [-half]+[-half+(i+.5)*step for i in range(7)]+[half]
   for left,right in zip(starts,starts[1:]):
    left+=.009;right-=.009;bottom=eave+row*.195+.007;top=eave+(row+1)*.195-.007
    poly=clip(clip([(left,bottom),(right,bottom),(right,top),(left,top)],1),-1)
    if len(poly)<3 or max(p[1] for p in poly)-min(p[1] for p in poly)<.025:continue
    civic_brick('Clipped civic gable brick',poly,1.98,angle)

def service_building(kind):
 begin(kind);palette='roof_red' if kind=='pokecenter' else 'roof_blue'
 foundation(3.67,3.37)
 # This wall has a distinct role: live source brick colors must not repaint
 # the same model's pale window borders, jambs or fascia.
 cube('Recessed civic mortar backing',(0,0,1.35),(3.5,3.21,2.22),'plaster_service_mortar',.045)
 civic_bricks()
 building_trim(3.51,3.22,1.37,2.14)
 for y in [-1.614,1.614]:
  cube('Warm service base course',(0,y,.54),(3.5,.065,.20),'timber_gold')
  cube('Pale upper string course',(0,y,2.34),(3.61,.11,.14),'plaster')
 for x in [-1.765,1.765]:cube('Side warm base course',(x,0,.54),(.065,3.20,.20),'timber_gold')
 for xx in [-1.751,1.751]:mesh('Source-painted civic gable',[(xx,-1.605,2.42),(xx,1.605,2.42),(xx,0,3.28)],[(2,1,0)] if xx<0 else [(0,1,2)],'plaster_service_wall')
 town_roof(3.96,3.70,2.46,3.35,palette,gable_material='plaster_service_mortar')
 civic_gables()
 fy=-1.615;door(fy-.03,z=1.16,w=1.02,h=1.75,lab=True)
 for x in [-1.13,1.13]:window(x,1.34,fy,w=.68,h=1.0)
 for a in [-90,90]:
  for x in [-.85,.85]:window(x,1.44,-1.763,w=.75,h=.93,angle=a)
 for x in [-1.08,0,1.08]:window(x,1.42,fy,w=.73,h=.88,angle=180)
 cube('Entrance landing',(0,-1.955,.11),(1.37,.55,.22),'stone_light',.025)
 cube('Threshold front edge',(0,-2.241,.092),(1.20,.026,.09),'stone')
 if kind=='pokecenter':
  cube('Red entry shelter',(0,-1.87,2.16),(1.40,.66,.15),'medical_red',.04)
  cube('Entry shelter cream edge',(0,-2.186,2.135),(1.44,.06,.09),'plaster_light',.014)
  # The approved render uses the medical ball emblem, separate from roof paint.
  facade_cube('Medical enamel plaque',0,2.62,.84,.64,fy-.14,.14,'medical_red',bevel=.035)
  for name,radius,depth,offset,material in [
      ('Medical ball outer ring',.225,.025,.225,'medical_white'),
      ('Medical ball inset',.174,.030,.244,'medical_red'),
      ('Medical ball central button',.065,.038,.270,'medical_white')]:
   emblem=cone(name,(0,fy-offset,2.62),radius,radius,depth,material,32)
   emblem.rotation_euler.x=math.pi/2
  facade_cube('Medical ball equator',0,2.62,.76,.045,fy-.267,.025,'medical_white',bevel=.008)
  for x in [-.70,.70]:
   facade_cube('Warm entrance lamp',x,1.87,.095,.16,fy-.095,.12,'yellow',bevel=.015)
 else:
  # The shop's striped awning, recessed glazing and hanging plaque distinguish it.
  for i in range(9):
   x=-1.62+i*.405
   o=cube('Tailored striped shop awning',(x,-1.86,2.105),(.402,.58,.068),palette if i%2==0 else 'plaster_light',.008);o.rotation_euler.x=.14
   cube('Scalloped awning valance',(x,-2.139,2.034),(.402,.035,.17),palette if i%2==0 else 'plaster_light',.022)
  facade_cube('Source Mart signboard',0,2.64,1.17,.36,fy-.12,.12,'shop_sign_blue',bevel=.025)
  source_service_letters('MART',fy-.21,2.64,.25,'medical_white')
 # A restrained roof vent gives each roof a readable scale.
 cube('Roof vent base',(.99,.65,2.98),(.44,.43,.24),'plaster_shadow',.018)
 cube('Roof vent cap',(.99,.65,3.14),(.53,.52,.11),'stone_chimney_cap',.022)


def make_gate():
 # One north/south route-gate kit, with the source's offset door and right
 # paired window. Shared live source paint supplies the roof and pink masonry.
 begin('route_gate');foundation(3.67,3.37)
 cube('Recessed gate mortar backing',(0,0,1.35),(3.5,3.21,2.22),'plaster_service_mortar',.045)
 civic_bricks();building_trim(3.51,3.22,1.37,2.14)
 for y in [-1.614,1.614]:
  cube('Gate ochre base course',(0,y,.54),(3.5,.065,.20),'timber_gold')
  cube('Gate pale cornice',(0,y,2.34),(3.61,.11,.14),'plaster',.018)
 for x in [-1.765,1.765]:cube('Gate side base course',(x,0,.54),(.065,3.20,.20),'timber_gold')
 town_roof(3.96,3.70,2.46,3.35,'roof_blue',gable_material='plaster_service_mortar')
 civic_gables()
 fy=-1.615
 door(fy-.03,z=1.16,w=1.02,h=1.75,lab=True)
 # The fitted native doorway remains at column three; the entrance is the
 # physical anchor and the right window is one double-light opening.
 window(1.10,1.54,fy,w=.97,h=.82)
 for a in [-90,90]:window(0,1.48,-1.763,w=.96,h=.86,angle=a)
 for x in [-1.08,1.08]:window(x,1.42,fy,w=.73,h=.88,angle=180)
 cube('Gate entrance landing',(0,-1.955,.11),(1.37,.55,.22),'stone_light',.025)
 cube('Gate threshold nosing',(0,-2.241,.092),(1.20,.026,.09),'stone',.012)
 # Restrained timber brackets make the eave readable from the game camera.
 for x in [-1.58,1.58]:
  beam('Gate eave corbel',(x,-1.63,2.10),(x,-1.82,2.39),.09,.09,'timber')


def make_traditional(kind):
 begin(kind);wide=kind=='violet_gym';w=5.5 if wide else 3.50;d=3.2
 foundation(w+.14,d+.12)
 cube('Warm traditional plaster',(0,0,1.24),(w,d,1.99),'plaster_light',.025)
 building_trim(w,d,1.29,1.92)
 for y in [-d/2-.025,d/2+.025]:
  for x in [-w*.4,-w*.2,0,w*.2,w*.4]:cube('Exposed timber rhythm',(x,y,1.28),(.09,.095,1.85),'timber_light')
 for xx in [-w/2,w/2]:mesh('Traditional pale gable',[(xx,-d/2,2.23),(xx,d/2,2.23),(xx,0,3.0)],[(2,1,0)] if xx<0 else [(0,1,2)],'plaster_light')
 if wide:
  town_roof(w+.48,d+.51,2.25,3.08,'violet',gable_material='plaster_light',columns=15)
 else:
  roof(w+.48,d+.51,2.25,3.08,'violet',6,gable_material='plaster_light')
 fy=-d/2-.035;door(fy,z=1.03,w=.88,h=1.56)
 for x in [-w*.32,w*.32]:
  window(x,1.28,fy,w=.79 if wide else .60,h=.78)
  for dx in [-.22,-.11,.11,.22]:facade_cube('Traditional window lattice',x+dx,1.28,.025,.66,fy-.20,.04,'timber_light')
 for a in [-90,90]:window(0,1.27,-w/2-.02,w=.9,h=.76,angle=a)
 cube('Traditional stone threshold',(0,-1.925,.10),(1.28,.69,.20),'stone_light',.024)
 if wide:
  # A boxed lintel projects beyond the deep fascia, keeping the whole native
  # name legible from the production elevated camera while retaining its back.
  facade_cube('Source Gym nameboard',0,2.07,1.32,.37,fy-.21,.32,'cream',bevel=.027)
  source_service_letters('GYM',fy-.39,2.07,.25)


def pagoda_roof(width,depth,eave,ridge,palette):
 # Four curved hips share closed corners. Source paint still owns every tile.
 x,y=width/2,depth/2
 outer=[(-x,-y),(x,-y),(x,y),(-x,y)]
 inner=[(a*.18,b*.18) for a,b in outer]
 def point(side,u,t,lift=0):
  a,b=outer[side],outer[(side+1)%4];c,d=inner[side],inner[(side+1)%4]
  px=(a[0]*(1-u)+b[0]*u)*(1-t)+(c[0]*(1-u)+d[0]*u)*t
  py=(a[1]*(1-u)+b[1]*u)*(1-t)+(c[1]*(1-u)+d[1]*u)*t
  pz=eave+(ridge-eave)*t**1.45+.10*(1-t)**6+.10*abs(2*u-1)**4*(1-t)**3
  return (px,py,pz+lift)
 def shell(name,rings,material):
  count=len(rings[0]);vs=[p for ring in rings for p in ring]
  fs=[tuple(reversed(range(count))),tuple(range(count,2*count))]
  fs.extend((i,(i+1)%count,(i+1)%count+count,i+count) for i in range(count))
  mesh(name,vs,fs,material)
 for side in range(4):
  # The continuous dark timber deck closes every gap without an open skirt.
  for row in range(5):
   for column in range(2):
    u0,u1=column/2,(column+1)/2;t0,t1=row/5,(row+1)/5
    ps=[point(side,u,t) for u,t in [(u0,t0),(u1,t0),(u1,t1),(u0,t1)]]
    shell('Pagoda continuous closed deck',[[(*p[:2],p[2]-.15) for p in ps],
      [(*p[:2],p[2]-.07) for p in ps]],palette+'_dark')
  for row in range(5):
   cuts=[i/6 for i in range(7)] if row%2==0 else [0.]+[(i+.5)/6 for i in range(6)]+[1.]
   for column,(left,right) in enumerate(zip(cuts,cuts[1:])):
    u0=left+.003;u1=right-.003
    t0=row/5;t1=min(1.,(row+1.10)/5)
    rings=[]
    for t in [t0,t1]:
     top=[point(side,u0+(u1-u0)*j/3,t,.018+row*.012+.032*math.sin(math.pi*j/3)) for j in range(4)]
     rings.append(top+[point(side,u1,t,-.035+row*.012),point(side,u0,t,-.035+row*.012)])
    material=palette+('_light' if (column*17+row*7+side*3)%11==0
      else '_warm' if (column*17+row*7+side*3)%11==7 else '')
    shell('Pagoda closed curved ceramic course',rings,material)
  for segment in range(6):
   u0,u1=segment/6,(segment+1)/6
   ps=[point(side,u,t) for u,t in [(u0,0),(u1,0),(u1,.025),(u0,.025)]]
   shell('Pagoda swept timber fascia',[[(*p[:2],p[2]-.14) for p in ps],
      [(*p[:2],p[2]-.025) for p in ps]],'timber_light')
  a,b=outer[side]
  beam('Pagoda exposed corner bracket',(a*.84,b*.84,eave-.22),
    (a*.96,b*.96,eave+.06),.075,.075,'timber')
 cube('Pagoda closed upper cap',(0,0,ridge+.038),(width*.18,depth*.18,.12),palette+'_light',.015)
 COL['roof_courses']='closed_prism_v1'

def make_tower():
 begin('sprout_tower');foundation(3.60,6.00)
 old_roofs=set();tiers=[(3.48,5.72,.25),(2.92,4.60,2.02),(2.32,3.42,3.59)]
 # Three diminishing timber-and-plaster storeys, with hand-laid violet roofs.
 for level,(w,d,z) in enumerate(tiers):
  h=1.55
  cube('Tower pale storey',(0,0,z+h/2),(w,d,h),'plaster_light',.025)
  for x in [-w/2+.04,w/2-.04]:
   for y in [-d/2+.04,d/2-.04]:cube('Tower oak corner post',(x,y,z+h/2),(.12,.12,h),'timber_light')
  for y in [-d/2-.03,d/2+.03]:
   cube('Tower oak storey beam',(0,y,z+h-.05),(w,.12,.12),'timber')
   for x in [-w*.31,w*.31]:
    facade_cube('Tower narrow window',x,z+.91,.42,.68,-d/2-.05,.09,'timber',0 if y<0 else 180)
    for dx in [-.12,0,.12]:facade_cube('Tower window lath',x+dx,z+.91,.035,.60,-d/2-.115,.022,'plaster',0 if y<0 else 180)
  old=set(COL.objects);roof(w+.46,d+.44,z+h,z+h+.66,'violet',5)
  old_roofs.update(set(COL.objects)-old)
  # Tiered roof courses stay solid and have modeled rims all the way around.
 fy=-2.865;door(fy,z=1.06,w=.84,h=1.58)
 cube('Tower threshold',(0,-3.13,.11),(1.23,.54,.22),'stone_light',.025)
 facade_cube('Tower entry plaque',0,1.97,.64,.26,fy-.10,.10,'timber',bevel=.018)
 for x in [-.17,0,.17]:facade_cube('Tower brass plaque marks',x,1.97,.055,.14,fy-.16,.018,'yellow')
 rod('Tower finial',(0,0,5.89),(0,0,6.39),.07,'timber',8)
 for z,r in [(6.03,.16),(6.17,.13),(6.30,.095)]:cone('Tower finial ring',(0,0,z),r,r,.045,'yellow',10)
 # Preserve the original source-fit datum before replacing the old roof parts.
 # Walls, windows, thresholds and door anchor consequently replay bit-for-bit.
 bpy.context.view_layer.update()
 points=[o.matrix_world@v.co for o in COL.objects for v in o.data.vertices]
 xx=max(abs(v.x) for v in points);lo=min(v.y for v in points);hi=max(v.y for v in points)
 COL['native_fit']=[xx,lo,hi]
 for o in old_roofs:
  data=o.data;bpy.data.objects.remove(o,do_unlink=True)
  if data.users==0:bpy.data.meshes.remove(data)
 dx=xx-(tiers[0][0]+.46)/2;dy=hi-(tiers[0][1]+.44)/2
 for w,d,z in tiers:pagoda_roof(w+.46+2*dx,d+.44+2*dy,z+1.55,z+2.21,'violet')


def make_grass():
 begin('grass')
 for i in range(11):
  a=i*2.399963;r=.10+.25*((i*7)%11)/10;x=math.cos(a)*r;y=math.sin(a)*r
  h=.29+.30*((i*3)%11)/10;lean=.10+.07*(i%3);width=.045+.012*(i%2)
  dx=math.cos(a)*lean;dy=math.sin(a)*lean;px=-math.sin(a)*width;py=math.cos(a)*width
  mesh('Folded meadow blade',[(x-px,y-py,0),(x+px,y+py,0),(x+dx*.40+px*.55,y+dy*.40+py*.55,h*.58),(x+dx,y+dy,h),(x+dx*.40-px*.55,y+dy*.40-py*.55,h*.58),(x+dx*.42,y+dy*.42,h*.64)],[(0,1,5),(1,2,5),(2,3,5),(3,4,5),(4,0,5),(4,3,2,1,0)],['leaf','leaf_light','leaf_gold'][i%3])

builders={'pokecenter':lambda:service_building('pokecenter'),'mart':lambda:service_building('mart'),'route_gate':make_gate,'traditional_house':lambda:make_traditional('traditional_house'),'violet_gym':lambda:make_traditional('violet_gym'),'sprout_tower':make_tower,'grass':make_grass}
only=next((a.split('=',1)[1].split(',') for a in args if a.startswith('--only=')),None)
for name,build in builders.items():
 if only is None or name in only:build()
TARGETS={'pokecenter':(4,4),'mart':(4,4),'route_gate':(4,4),'traditional_house':(4,3),'violet_gym':(6,3),'sprout_tower':(4,8)}
STATS={}
for name,col in list(ASSETS.items()):
 bpy.context.view_layer.update();objects=[o for o in col.objects if o.type=='MESH']
 for ob in objects:
  bpy.ops.object.select_all(action='DESELECT');ob.select_set(True);bpy.context.view_layer.objects.active=ob
  bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
 if name in TARGETS:
  # Symmetric x edges preserve the explicit centered entrance; all features,
  # including shelter and threshold, are accounted for in the exact plot fit.
  xx=max(abs(v.co.x) for o in objects for v in o.data.vertices)
  ys=[v.co.y for o in objects for v in o.data.vertices];lo,hi=min(ys),max(ys);cy=(lo+hi)/2
  if col.get('native_fit'):
   xx,lo,hi=col['native_fit'];cy=(lo+hi)/2
  tw,td=TARGETS[name]
  for o in objects:
   for v in o.data.vertices:v.co.x*=tw/(2*xx);v.co.y=(v.co.y-cy)*td/(hi-lo)
   o.data.update()
 primitives=[];minv=[float('inf')]*3;maxv=[-float('inf')]*3;tri_count=0
 # Export each material with indexed flat-shaded corners. Editable source parts
 # are retained in the .blend, unlike the compact combined runtime meshes.
 groups={}
 for o in objects:
  bpy.ops.object.select_all(action='DESELECT');o.select_set(True);bpy.context.view_layer.objects.active=o
  bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT');bpy.ops.mesh.normals_make_consistent(inside=False);bpy.ops.object.mode_set(mode='OBJECT')
  if o.name.startswith(('Source-painted civic gable','Source-painted roof gable','Traditional pale gable')):
   # An isolated open triangle has no closed-volume "outside" for Blender's
   # consistency operator. Resolve this authored end wall explicitly.
   import bmesh
   bm=bmesh.new();bm.from_mesh(o.data);bm.normal_update()
   for face in bm.faces:
    if face.normal.x*face.calc_center_median().x<0:face.normal_flip()
   bm.to_mesh(o.data);bm.free();o.data.update()
  me=o.data;me.calc_loop_triangles()
  mat=me.materials[0];group=groups.setdefault(mat.name,{'name':name+'__'+mat.name,'material':mat.name,'base_color':[round(v,5) for v in mat.diffuse_color],'positions':[],'normals':[],'indices':[],'lookup':{}})
  for tri in me.loop_triangles:
   if tri.normal.length_squared<.9:continue
   for vi in tri.vertices:
    v=me.vertices[vi].co;n=tri.normal
    pos=(round(v.x,6),round(v.z,6),round(-v.y,6));norm=(round(n.x,6),round(n.z,6),round(-n.y,6));key=pos+norm
    for k in range(3):minv[k]=min(minv[k],pos[k]);maxv[k]=max(maxv[k],pos[k])
    if key not in group['lookup']:
     group['lookup'][key]=len(group['positions'])//3;group['positions'].extend(pos);group['normals'].extend(norm)
    group['indices'].append(group['lookup'][key])
 for group in groups.values():group.pop('lookup');primitives.append(group);tri_count+=len(group['indices'])//3
 data={'name':name,'coordinate_system':'right-handed; +Y up; front +Z','origin':'floor-center','bounds':{'min':minv,'max':maxv},'dimensions':[round(maxv[i]-minv[i],6) for i in range(3)],'triangle_count':tri_count,'primitive_count':len(primitives),'primitives':primitives}
 if col.get('roof_courses'):data['roof_courses']=col['roof_courses']
 if name in TARGETS:data['door_anchor']=[0,0,maxv[2]]
 (P/f'{name}.mesh.json').write_text(json.dumps(data,separators=(',',':')))
 bpy.ops.object.select_all(action='DESELECT')
 for o in objects:o.select_set(True)
 bpy.context.view_layer.objects.active=objects[0]
 bpy.ops.export_scene.gltf(filepath=str(P/f'{name}.glb'),use_selection=True,export_format='GLB',export_yup=True,export_apply=True,export_materials='EXPORT',export_animations=False)
 STATS[name]={k:v for k,v in data.items() if k!='primitives'}
 # Closed window pockets retain every ceramic course and add reveal faces.
 assert tri_count<(13000 if name=='sprout_tower' else 12000 if col.get('roof_courses') else 7500),(name,tri_count)
(P/'asset-stats.json').write_text(json.dumps(STATS,indent=2))
# Inspection lineup is excluded from every runtime export.
for name,(x,y) in {'pokecenter':(-6,3),'mart':(0,3),'route_gate':(6,3),'traditional_house':(-6,-3),'violet_gym':(0,-3),'sprout_tower':(7,-4),'grass':(-3,-5)}.items():
 if name in ASSETS:
  for o in ASSETS[name].objects:o.location+=Vector((x,y,0))
begin('PRESENTATION only');cube('Warm studio floor',(0,0,-.12),(200,200,.18),'cream')
world=bpy.data.worlds.new('Daylit model workshop');scene.world=world;world.use_nodes=True
world.node_tree.nodes['Background'].inputs['Color'].default_value=(.30,.39,.48,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.5
for name,loc,energy,size in [('Key',(-9,-13,18),2300,10),('Fill',(10,-3,12),1700,9),('Rim',(1,12,15),1900,8)]:
 bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name=name;o.data.energy=energy;o.data.shape='DISK';o.data.size=size;o.rotation_euler=(Vector((0,0,1.6))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(19,-28,23));cam=bpy.context.object;cam.rotation_euler=(Vector((.4,-.4,1.6))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=25;scene.camera=cam
scene.render.engine='CYCLES';scene.cycles.samples=32;scene.cycles.use_denoising=False
scene.render.resolution_x=1600;scene.render.resolution_y=1100;scene.render.resolution_percentage=100
scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.filepath=str(P/'connected-johto-kit.png')
scene['Asset source']='Original individually authored geometry, no imported textures or game artwork; editable source parts remain in named collections.'
bpy.ops.wm.save_as_mainfile(filepath=str(P/'connected-johto-editable.blend'))
if '--skip-preview' not in args:bpy.ops.render.render(write_still=True)
print('FINAL_STATS='+json.dumps(STATS))
