"""Original modular New Bark art kit. Run: blender -b --python tools/build-new-bark-models.py -- target/new-bark-assets --skip-preview.
Blender source is Z-up, front -Y; exports are glTF-standard Y-up, front +Z.
No imported sprites, textures, or Nintendo models. All meshes authored here.
"""
import bpy, math, random, json, sys
from pathlib import Path
from mathutils import Vector
args = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
P = Path(next((arg for arg in args if not arg.startswith('--')), 'target/new-bark-assets'))
P.mkdir(parents=True, exist_ok=True)
random.seed(2017)
bpy.ops.object.select_all(action='SELECT'); bpy.ops.object.delete(use_global=False)
for m in list(bpy.data.materials): bpy.data.materials.remove(m)
scene=bpy.context.scene
COLORS={
 'plaster':(0.87,0.79,0.59), 'plaster_light':(0.98,0.91,0.73), 'plaster_shadow':(0.70,0.60,0.41),
 'timber':(0.28,0.18,0.14), 'timber_light':(0.48,0.31,0.21), 'timber_gold':(0.64,0.43,0.26),
 'roof_red':(0.62,0.20,0.15), 'roof_red_light':(0.76,0.29,0.20), 'roof_red_dark':(0.43,0.14,0.12),
 'roof_red_warm':(0.69,0.25,0.16), 'roof_teal':(0.16,0.40,0.39), 'roof_teal_light':(0.24,0.51,0.46),
 'roof_teal_dark':(0.12,0.28,0.29), 'roof_teal_warm':(0.22,0.44,0.39),
 'stone':(0.43,0.47,0.41), 'stone_light':(0.62,0.65,0.54), 'stone_dark':(0.32,0.35,0.31),
 'glass':(0.27,0.52,0.58), 'glass_light':(0.63,0.82,0.79), 'glass_dark':(0.17,0.34,0.40),
 'leaf':(0.18,0.38,0.23), 'leaf_light':(0.35,0.53,0.29), 'leaf_dark':(0.11,0.28,0.20), 'leaf_gold':(0.48,0.61,0.33),
 'coral':(0.89,0.34,0.25), 'pink':(0.94,0.61,0.60), 'yellow':(0.97,0.73,0.29),
 'cream':(0.97,0.88,0.67), 'ink':(0.12,0.17,0.20), 'skin':(0.87,0.64,0.43),
 'skin_light':(0.98,0.75,0.51), 'trouser':(0.20,0.32,0.38), 'pack':(0.68,0.48,0.24),
 'white':(0.91,0.91,0.80)
}
MATS={}
for n,c in COLORS.items():
 m=bpy.data.materials.new(n); m.diffuse_color=(*c,1); m.use_nodes=True
 bs=m.node_tree.nodes.get('Principled BSDF'); bs.inputs['Base Color'].default_value=(*c,1); bs.inputs['Roughness'].default_value=.91
 MATS[n]=m
COL=None
ASSETS={}
def begin(name):
 global COL
 COL=bpy.data.collections.new(name);scene.collection.children.link(COL);ASSETS[name]=COL
 return COL

def move(o,name,mat):
 o.name=name
 for c in list(o.users_collection):c.objects.unlink(o)
 COL.objects.link(o)
 if mat:o.data.materials.append(MATS[mat])
 return o

def cube(n,loc,dim,mat,bevel=0):
 bpy.ops.mesh.primitive_cube_add(size=1,location=loc)
 o=move(bpy.context.object,n,mat);o.dimensions=dim;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 if bevel:
  b=o.modifiers.new('Hand-cut edges','BEVEL');b.width=min(bevel,min(dim)*.4);b.segments=1
  bpy.ops.object.modifier_apply(modifier=b.name)
 return o

def mesh(n,verts,faces,mat):
 me=bpy.data.meshes.new(n);me.from_pydata(verts,[],faces);me.update()
 o=bpy.data.objects.new(n,me);COL.objects.link(o);me.materials.append(MATS[mat]);return o

def ico(n,loc,scale,mat,sub=1):
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=sub,radius=1,location=loc);o=move(bpy.context.object,n,mat);o.scale=scale;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);return o

def cone(n,loc,r1,r2,depth,mat,vertices=8):
 bpy.ops.mesh.primitive_cone_add(vertices=vertices,radius1=r1,radius2=r2,depth=depth,location=loc);return move(bpy.context.object,n,mat)

def rod(n,a,b,r,mat,vertices=6):
 a=Vector(a);b=Vector(b);o=cone(n,(a+b)/2,r,r*.80,(b-a).length,mat,vertices);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o

# Local facade widgets face -Y, rotated about building center by optional angle.
def facade_cube(n,x,z,w,h,y,depth,mat,angle=0,bevel=0):
 a=math.radians(angle);o=cube(n,(x*math.cos(a)-y*math.sin(a),x*math.sin(a)+y*math.cos(a),z),(w,depth,h),mat,bevel);o.rotation_euler.z=a;return o

def window(x,z,y,w=.68,h=.73,angle=0,shutters=False):
 f=lambda n,dx,dz,ww,hh,off,dd,mat,bevel=0:facade_cube(n,x+dx,z+dz,ww,hh,y+off,dd,mat,angle,bevel)
 f('Recessed window surround',0,0,w+.15,h+.15,-.015,.13,'timber',.015)
 f('Painted window border',0,0,w+.09,h+.09,-.09,.035,'plaster_light')
 f('Cool glass',0,0,w-.07,h-.07,-.115,.025,'glass')
 f('Glass upper reflection',-.12*w,.19*h,w*.46,.06,-.132,.014,'glass_light')
 f('Glass lower reflection',.19*w,-.19*h,w*.21,.035,-.133,.014,'glass_light')
 f('Window vertical bar',0,0,.055,h-.05,-.143,.04,'plaster_light')
 f('Window horizontal bar',0,0,w-.05,.052,-.144,.04,'plaster_light')
 f('Window ledge',0,-h/2-.055,w+.28,.10,-.10,.32,'timber_light',.018)
 if shutters:
  for s in [-1,1]:
   f('Open wooden shutter',s*(w/2+.17),0,.17,h+.08,-.045,.045,'timber_light')
   for zz in [-.2,0,.2]:f('Shutter crosspiece',s*(w/2+.17),zz,.18,.025,-.075,.025,'timber_gold')

def door(y,z=1.02,w=.82,h=1.55,lab=False):
 facade_cube('Deep door frame',0,z,w+.23,h+.16,y,.16,'timber',bevel=.018)
 facade_cube('Cream inner door jamb',0,z,w+.13,h+.08,y-.105,.045,'plaster_light')
 facade_cube('Door leaf',0,z,w,h,y-.135,.04,'roof_teal_dark' if lab else 'timber_light',bevel=.017)
 if lab:
  for dx in [-.23,.23]:facade_cube('Lab door glazing',dx,z+.25,.37,.69,y-.162,.022,'glass')
  facade_cube('Door center seam',0,z,.045,h,y-.177,.02,'plaster_light')
  for dx in [-.08,.08]:facade_cube('Brass door grip',dx,z-.14,.035,.23,y-.191,.06,'yellow',bevel=.01)
 else:
  facade_cube('Door upper inset',0,z+.27,w-.22,.50,y-.162,.025,'timber')
  facade_cube('Door upper glass',0,z+.30,w-.32,.33,y-.177,.025,'glass')
  facade_cube('Door lower panel',0,z-.40,w-.24,.42,y-.162,.025,'timber_gold',bevel=.01)
  facade_cube('Brass door knob',w*.31,z-.1,.07,.07,y-.184,.07,'yellow',bevel=.023)

def roof(width,depth,eave,ridge,palette,rows):
 # Closed gable with rich fascia; independent roof panels overlap like handmade clay courses.
 x=width/2;d=depth/2
 mesh('Roof closed gable', [(-x,-d,eave),(x,-d,eave),(x,0,ridge),(-x,0,ridge),(-x,d,eave),(x,d,eave)],[(0,1,2,3),(3,2,5,4),(0,4,5,1),(0,3,4),(1,5,2)],palette+'_dark')
 for side in [-1,1]:
  for r in range(rows):
   t0=r/rows;t1=(r+1)/rows
   y0=side*d*(1-t0);y1=side*d*(1-t1)
   z0=eave+(ridge-eave)*t0+.015;z1=eave+(ridge-eave)*t1+.028
   count=5 if width<5 else 7
   for j in range(count):
    xx0=-x+j*width/count+.007;xx1=-x+(j+1)*width/count-.007
    # wedge strip is thin, genuinely modeled on both slopes
    vs=[(xx0,y0,z0),(xx1,y0,z0),(xx1,y1,z1),(xx0,y1,z1),(xx0,y0,z0-.048),(xx1,y0,z0-.048),(xx1,y1,z1-.038),(xx0,y1,z1-.038)]
    faces=[(0,1,2,3),(7,6,5,4),(4,5,1,0),(1,5,6,2),(2,6,7,3),(3,7,4,0)]
    if side>0:faces=[tuple(reversed(f)) for f in faces]
    color=[palette,palette,palette+'_light',palette+'_warm'][(j+2*r+(1 if side>0 else 0))%4]
    mesh('Individual roof course',vs,faces,color)
  cube('Eave shadow board',(0,side*(d-.025),eave-.025),(width,.12,.14),'timber')
 # gable bargeboards as rectangular beams along slope
 for xx in [-x+.015,x-.015]:
  for side in [-1,1]:
   a=Vector((xx,side*d,eave-.015));b=Vector((xx,0,ridge+.015));o=cube('Gable edge timber',(a+b)/2,(.095,.115,(b-a).length),'timber_light');o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler()
 cube('Ridge cap',(0,0,ridge+.038),(width,.18,.12),palette+'_light',.032)

def foundation(w,d):
 cube('Solid stone plinth',(0,0,.13),(w,d,.26),'stone',.035)
 # substantial corner footings and front blocks give the stonework a crafted rhythm
 for x in [-w/2+.12,w/2-.12]:
  for y in [-d/2+.12,d/2-.12]:cube('Dressed corner stone',(x,y,.18),(.28,.28,.28),'stone_light',.025)
 for x in [-w*.27,w*.27]:
  for y in [-d/2-.002,d/2+.002]:cube('Plinth block accent',(x,y,.135),(.68,.03,.15),'stone_light')

def building_trim(w,d,z,h):
 for x in [-w/2+.04,w/2-.04]:
  for y in [-d/2+.04,d/2-.04]:cube('Timber corner upright',(x,y,z),(.13,.13,h),'timber_light')
 cube('Foundation sill',(0,0,.35),(w+.07,d+.07,.14),'plaster_light',.016)
 for y in [-d/2-.02,d/2+.02]:cube('Long timber header',(0,y,z+h/2-.06),(w,.075,.11),'timber')
 for x in [-w/2-.02,w/2+.02]:cube('Side timber header',(x,0,z+h/2-.06),(.075,d,.11),'timber')

def make_house():
 begin('house');foundation(3.68,2.65)
 cube('Warm plaster walls',(0,0,1.22),(3.52,2.49,1.93),'plaster',.026)
 building_trim(3.53,2.5,1.27,1.83)
 # Side gables fill the volume behind roof, visible from every angle.
 for xx in [-1.761,1.761]:
  mesh('Cream gable end',[(xx,-1.245,2.17),(xx,1.245,2.17),(xx,0,3.17)],[(0,1,2)] if xx<0 else [(2,1,0)],'plaster_light')
 roof(3.935,2.93,2.18,3.29,'roof_red',6)
 fy=-1.252;door(fy-.016,z=1.02,w=.69,h=1.51)
 for x in [-1.13,1.13]:window(x,1.30,fy,w=.64,h=.70,shutters=True)
 # side walls: local y=-halfwidth rotated +/-90
 for a in [-90,90]:window(0,1.29,-1.765,w=.85,h=.72,angle=a)
 for x in [-.92,.92]:window(x,1.30,-1.252,w=.7,h=.7,angle=180)
 for a in [-90,90]:
  facade_cube('Gable attic vent',0,2.50,.38,.27,-1.77,.06,'timber',a)
  for zz in [2.44,2.51,2.58]:facade_cube('Vent louvers',0,zz,.30,.025,-1.806,.025,'plaster_shadow',a)
 cube('Doorstep',(0,-1.365,.105),(1.04,.27,.21),'stone_light',.025)
 # squared masonry chimney behind ridge
 cube('Chimney stack',(-1.08,.48,2.95),(.39,.42,1.17),'plaster_shadow',.018)
 for z in [2.69,2.94,3.19,3.43]:cube('Chimney mortar course',(-1.08,.48,z),(.414,.444,.04),'plaster_light')
 cube('Chimney upper cap',(-1.08,.48,3.56),(.53,.56,.13),'roof_red_dark',.025)
 cube('Chimney dark opening',(-1.08,.48,3.632),(.29,.32,.02),'ink')
 # brass lantern near entry
 facade_cube('Lantern backing',.51,1.37,.09,.29,fy-.025,.10,'timber')
 facade_cube('Lantern glow',.51,1.43,.13,.18,fy-.11,.12,'yellow',bevel=.016)
 facade_cube('Lantern cap',.51,1.545,.19,.06,fy-.11,.17,'timber')


def make_lab():
 begin('lab');foundation(5.62,3.61)
 cube('Laboratory plaster walls',(0,0,1.31),(5.48,3.47,2.1),'plaster_light',.025)
 building_trim(5.49,3.48,1.37,2.03)
 # lower warm wainscoting wraps all walls
 for y in [-1.742,1.742]:cube('Lab lower plaster band',(0,y,.62),(5.45,.045,.30),'plaster')
 for x in [-2.747,2.747]:cube('Lab side plaster band',(x,0,.62),(.045,3.42,.30),'plaster')
 for xx in [-2.741,2.741]:mesh('Lab gable plaster',[(xx,-1.735,2.35),(xx,1.735,2.35),(xx,0,3.40)],[(0,1,2)] if xx<0 else [(2,1,0)],'plaster')
 roof(5.935,3.93,2.36,3.49,'roof_teal',6)
 fy=-1.745;door(fy-.015,z=1.10,w=1.06,h=1.67,lab=True)
 for x in [-1.82,1.82]:window(x,1.38,fy,w=1.17,h=.84)
 for a in [-90,90]:
  for x in [-.82,.82]:window(x,1.4,-2.748,w=.77,h=.85,angle=a)
 for x in [-1.8,0,1.8]:window(x,1.43,-1.745,w=1.02,h=.85,angle=180)
 cube('Broad lab entry step',(0,-1.857,.115),(1.51,.285,.23),'stone_light',.025)
 # Distinctive carved research badge above front doorway, no raster art.
 facade_cube('Lab insignia dark plaque',0,2.065,.80,.29,fy-.075,.09,'roof_teal_dark',bevel=.025)
 for x,z,s in [(-.22,2.06,.083),(0,2.075,.097),(.22,2.06,.083)]:
  o=ico('Research insignia medallion',(x,fy-.141,z),(s,.024,s),'yellow',2)
 facade_cube('Plaque small underline',0,1.976,.47,.02,fy-.145,.025,'plaster_light')
 # skylight on rear roof pitch, modeled frame and pane
 o=cube('Lab roof skylight frame',(1.43,.75,3.11),(1.02,.76,.085),'timber');o.rotation_euler.x=math.radians(-29.47)
 o=cube('Lab roof skylight glass',(1.43,.734,3.16),(.89,.61,.027),'glass');o.rotation_euler.x=math.radians(-29.47)
 # side attic vents
 for a in [-90,90]:
  facade_cube('Laboratory attic vent',0,2.77,.67,.27,-2.75,.065,'roof_teal_dark',a)
  for zz in [2.68,2.75,2.82,2.89]:facade_cube('Vent cream louvers',0,zz,.56,.023,-2.788,.022,'plaster_light',a)


def make_tree():
 begin('tree')
 cone('Broad tree trunk',(0,0,.8),.20,.145,1.6,'timber_light',7)
 for i in range(5):
  a=i*math.tau/5+.2
  rod('Spreading roots',(.03*math.cos(a),.03*math.sin(a),.19),(.42*math.cos(a),.37*math.sin(a),.065),.08,'timber',5)
 for a in [.2,2.3,4.5]:rod('Forked bough',(0,0,1.12),(.43*math.cos(a),.4*math.sin(a),2.13),.10,'timber_light',6)
 lobes=[((0,0,2.35),(1.08,.93,.95)),((-.68,-.02,2.0),(.62,.66,.61)),((.64,.02,2.12),(.63,.66,.66)),((-.16,-.63,2.15),(.70,.61,.65)),((.14,.54,2.29),(.76,.62,.65)),((-.15,.06,2.96),(.64,.57,.56))]
 for i,(loc,scale) in enumerate(lobes):
  o=ico('Sculpted broadleaf crown',loc,scale,'leaf',2)
  for name in ['leaf_light','leaf_dark','leaf_gold']:o.data.materials.append(MATS[name])
  for v in o.data.vertices:v.co*=random.uniform(.95,1.045)
  o.data.update()
  for poly in o.data.polygons:
   n=poly.normal; r=random.random()
   poly.material_index=3 if n.z>.45 and r>.68 else 1 if n.z>.1 and r>.23 else 2 if n.z<-.35 else 0
 # grounded leaf tufts, deliberately kept within root footprint
 for i in range(4):
  a=i*1.6
  o=ico('Root foliage',(.28*math.cos(a),.25*math.sin(a),.095),(.19,.13,.09),'leaf',1)


def petal(n,center,angle,r,mat):
 cx,cy,cz=center;ca=math.cos(angle);sa=math.sin(angle)
 verts=[(cx,cy,cz-.015),(cx+ca*r*.60-sa*r*.37,cy+sa*r*.60+ca*r*.37,cz+.015),(cx+ca*r,cy+sa*r,cz+.035),(cx+ca*r*.60+sa*r*.37,cy+sa*r*.60-ca*r*.37,cz+.015),(cx+ca*r*.50,cy+sa*r*.50,cz+.060)]
 mesh(n,verts,[(0,1,4),(1,2,4),(2,3,4),(3,0,4),(3,2,1,0)],mat)

def make_flowers():
 begin('flowers')
 for i,(x,y,h) in enumerate([(-.23,-.09,.33),(.11,-.17,.40),(.24,.15,.32),(-.11,.2,.45),(0,.035,.49)]):
  rod('Flower stem',(x,y,0),(x+.018,y,h),.014,'leaf_dark',5)
  for s,z in [(-1,h*.30),(1,h*.56)]:
   mesh('Folded flower leaf',[(x,y,z),(x+s*.13,y-.035,z+.09),(x+s*.14,y+.024,z+.065),(x+s*.03,y+.035,z+.012)],[(0,1,2),(0,2,3)],'leaf_light')
  for j in range(5):petal('Sculpted flower petal',(x+.018,y,h),j*math.tau/5+i,.112,'pink' if i%2 else 'coral')
  ico('Golden flower center',(x+.018,y,h+.034),(.047,.047,.026),'yellow',1)


def make_trainer():
 begin('trainer')
 # 1.60 tall, 0.70 wide including hands. Front is -Y in this source.
 for s in [-1,1]:
  cube('Hiking shoe',(s*.118,-.045,.071),(.19,.31,.142),'ink',.026)
  cube('Cream boot sole',(s*.118,-.048,.024),(.195,.315,.047),'cream',.009)
  cube('Trouser leg',(s*.114,.005,.305),(.17,.19,.36),'trouser',.026)
 cube('Warm jacket',(0,0,.756),(.435,.276,.544),'coral',.045)
 cube('Jacket hem',(0,-.005,.522),(.448,.285,.09),'roof_red_dark',.015)
 facade_cube('Jacket zipper',0,.756,.025,.35,-.15,.018,'cream')
 for s in [-1,1]:
  o=cube('Short jacket sleeve',(s*.263,0,.82),(.14,.255,.29),'coral',.022);o.rotation_euler.y=s*math.radians(-7)
  cube('Trainer hand',(s*.278,-.015,.624),(.144,.17,.18),'skin_light',.03)
  facade_cube('Front jacket pocket',s*.125,.655,.105,.105,-.151,.018,'roof_red_warm',bevel=.01)
 cube('Neck',(0,0,1.06),(.17,.17,.15),'skin',.025)
 cube('Trainer head',(0,-.012,1.269),(.345,.305,.365),'skin_light',.041)
 # dark hair around sides/back, little front fringe
 cube('Hair back',(0,.113,1.304),(.359,.082,.271),'timber',.025)
 for s in [-1,1]:
  cube('Hair side',(s*.160,.025,1.333),(.040,.19,.18),'timber',.012)
  cube('Ear',(s*.181,-.015,1.251),(.060,.09,.105),'skin',.018)
  cube('Bright eye',(s*.082,-.172,1.28),(.036,.018,.047),'ink',.005)
 cube('Nose',(0,-.183,1.217),(.055,.049,.055),'skin',.012)
 cube('Cap crown',(0,-.003,1.50),(.402,.349,.20),'coral',.043)
 cube('Cap band',(0,-.003,1.420),(.414,.353,.049),'roof_red_dark',.011)
 cube('Cream cap brim',(0,-.209,1.422),(.400,.177,.052),'cream',.016)
 facade_cube('Cap stitched emblem',0,1.51,.075,.066,-.180,.020,'cream',bevel=.014)
 cube('Canvas backpack',(0,.204,.80),(.330,.157,.42),'pack',.035)
 cube('Pack flap',(0,.216,.967),(.349,.164,.103),'yellow',.020)
 cube('Pack back pocket',(0,.294,.74),(.206,.035,.174),'timber_gold',.014)
 for s in [-1,1]:
  cube('Shoulder strap',(s*.154,.014,1.017),(.058,.279,.055),'pack',.012)
  facade_cube('Front pack strap',s*.154,.917,.050,.18,-.146,.026,'pack')


def make_npc(kind):
 begin(kind)
 for side in [-1,1]:
  cube('NPC walking shoe',(side*.11,-.055,.075),(.185,.30,.15),'timber' if kind=='teacher' else 'ink',.025)
  cube('NPC ankle',(side*.11,0,.275),(.14,.17,.31),'skin' if kind=='teacher' else 'trouser',.021)
 if kind=='teacher':
  cone('Teacher pleated skirt',(0,0,.535),.267,.172,.51,'roof_red_dark',8)
  cube('Teacher cardigan',(0,0,.873),(.427,.27,.34),'roof_red_warm',.035)
  facade_cube('Teacher cream collar',0,1.037,.22,.054,-.135,.042,'cream',bevel=.012)
  for z in [.77,.85,.93]:facade_cube('Cardigan button',0,z,.031,.031,-.150,.027,'yellow',bevel=.009)
 else:
  cube('Scientist trousers',(0,0,.432),(.37,.235,.30),'trouser',.025)
  cube('Scientist white coat',(0,0,.824),(.439,.288,.585),'white',.031)
  facade_cube('Scientist blue shirt',0,.998,.151,.19,-.156,.020,'roof_teal')
  for side in [-1,1]:
   facade_cube('White coat lapel',side*.104,.972,.084,.219,-.173,.022,'cream')
   facade_cube('Coat patch pocket',side*.12,.748,.11,.135,-.164,.025,'plaster_light',bevel=.009)
  facade_cube('Coat seam',0,.698,.023,.32,-.164,.021,'stone_light')
 for side in [-1,1]:
  cube('NPC sleeve',(side*.268,0,.851),(.141,.231,.342),'roof_red_warm' if kind=='teacher' else 'white',.021)
  cube('NPC hand',(side*.279,-.011,.624),(.142,.17,.18),'skin_light',.026)
 cube('NPC neck',(0,0,1.077),(.166,.16,.138),'skin',.021)
 cube('NPC head',(0,-.014,1.278),(.349,.31,.37),'skin_light',.039)
 cube('NPC hair cap',(0,.017,1.461),(.383,.325,.171),'timber' if kind=='teacher' else 'stone_light',.04)
 cube('NPC hair back',(0,.116,1.339),(.373,.087,.294),'timber' if kind=='teacher' else 'stone_light',.019)
 for side in [-1,1]:
  cube('NPC ear',(side*.182,-.013,1.273),(.057,.090,.106),'skin',.018)
  cube('NPC eye',(side*.083,-.176,1.292),(.034,.018,.044),'ink',.004)
 cube('NPC nose',(0,-.189,1.241),(.056,.044,.060),'skin',.011)
 if kind=='teacher':
  ico('Teacher hair bun',(0,.116,1.541),(.123,.103,.060),'timber',2)
  for side in [-1,1]:
   cube('Teacher hair side',(side*.169,.007,1.375),(.058,.191,.236),'timber',.018)
   ico('Teacher earring',(side*.197,-.063,1.240),(.017,.018,.028),'yellow',1)
 else:
  # Glasses rims and bridge are modeled, avoiding texture billboards.
  for side in [-1,1]:
   for dz in [-.046,.046]:facade_cube('Glasses horizontal rim',side*.089,1.297+dz,.127,.018,-.191,.018,'ink')
   for dx in [-.063,.063]:facade_cube('Glasses vertical rim',side*.089+dx,1.297,.017,.095,-.191,.018,'ink')
  facade_cube('Glasses bridge',0,1.314,.061,.016,-.191,.020,'ink')
  cube('Scientist top hair',(0,.031,1.57),(.257,.245,.06),'stone_light',.025)

make_house();make_lab();make_tree();make_flowers();make_trainer();make_npc('teacher');make_npc('scientist')
# Export each model as a compact primitive per material. Unapplied camera/light geometry never enters a model.
def export_asset(name,col):
 bpy.ops.object.select_all(action='DESELECT')
 # Gather a separate object per material, including multi-material crown meshes.
 for obj in list(col.objects):
  if obj.type!='MESH':continue
  if len(obj.data.materials)>1:
   bpy.context.view_layer.objects.active=obj;obj.select_set(True)
   bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT');bpy.ops.mesh.separate(type='MATERIAL');bpy.ops.object.mode_set(mode='OBJECT');bpy.ops.object.select_all(action='DESELECT')
 groups={}
 for obj in list(col.objects):
  if obj.type=='MESH' and len(obj.data.materials):groups.setdefault(obj.data.materials[0].name,[]).append(obj)
 for mat_name,objects in groups.items():
  bpy.ops.object.select_all(action='DESELECT')
  for obj in objects:obj.select_set(True)
  bpy.context.view_layer.objects.active=objects[0];bpy.ops.object.join();obj=bpy.context.object;obj.name=f'{name}__{mat_name}'
  bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
  scene.cursor.location=(0,0,0);bpy.ops.object.origin_set(type='ORIGIN_CURSOR')
  # Recompute consistent face winding, particularly the two roof slope shells.
  bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT');bpy.ops.mesh.normals_make_consistent(inside=False);bpy.ops.object.mode_set(mode='OBJECT')
 objects=[o for o in col.objects if o.type=='MESH']
 if name in ['house','lab']:
  # Tiny ledges and edge timbers are included in the exact occupied footprint.
  target=(4,3) if name=='house' else (6,4)
  ext=[max(abs(v.co[i]) for obj in objects for v in obj.data.vertices) for i in [0,1]]
  for obj in objects:
   for v in obj.data.vertices:
    v.co.x*=target[0]/(2*ext[0]);v.co.y*=target[1]/(2*ext[1])
 if name=='flowers':
  for obj in objects:
   for v in obj.data.vertices:v.co.z=max(0,v.co.z)
 primitives=[];minv=[float('inf')]*3;maxv=[-float('inf')]*3;tri_count=0
 for obj in sorted(objects,key=lambda o:o.name):
  me=obj.data;me.calc_loop_triangles();positions=[];normals=[];indices=[];mapping={}
  for tri in me.loop_triangles:
   if tri.normal.length_squared<.9:continue
   for vid in tri.vertices:
    v=obj.matrix_world@me.vertices[vid].co;n=tri.normal
    pos=(round(v.x,6),round(v.z,6),round(-v.y,6));norm=(round(n.x,6),round(n.z,6),round(-n.y,6))
    for i in range(3):minv[i]=min(minv[i],pos[i]);maxv[i]=max(maxv[i],pos[i])
    key=pos+norm
    if key not in mapping:
     mapping[key]=len(positions)//3;positions.extend(pos);normals.extend(norm)
    indices.append(mapping[key])
  color=list(obj.data.materials[0].diffuse_color)
  primitives.append({'name':obj.name,'material':obj.data.materials[0].name,'base_color':[round(v,5) for v in color], 'positions':positions,'normals':normals,'indices':indices})
  tri_count+=len(indices)//3
 data={'name':name,'coordinate_system':'right-handed; +Y up; front +Z','origin':'floor-center','bounds':{'min':minv,'max':maxv},'dimensions':[round(maxv[i]-minv[i],6) for i in range(3)],'triangle_count':tri_count,'primitive_count':len(primitives),'primitives':primitives}
 if name in ['house','lab']:data['door_anchor']=[0,0,maxv[2]]
 (P/f'{name}.mesh.json').write_text(json.dumps(data,separators=(',',':')))
 bpy.ops.object.select_all(action='DESELECT')
 for obj in objects:obj.select_set(True)
 bpy.context.view_layer.objects.active=objects[0]
 bpy.ops.export_scene.gltf(filepath=str(P/f'{name}.glb'),use_selection=True,export_format='GLB',export_yup=True,export_apply=True,export_materials='EXPORT',export_animations=False)
 return {k:v for k,v in data.items() if k!='primitives'}
STATS={name:export_asset(name,col) for name,col in ASSETS.items()}
(P/'asset-stats.json').write_text(json.dumps(STATS,indent=2))
# Beautiful overview only in the source .blend, laid out as a real art kit.
# Model exports above stay centered; montage offsets are applied after export.
offsets={'house':(-3.40,1.10,0),'lab':(3.00,1.65,0),'tree':(-5.0,-3.1,0),'flowers':(-1.65,-3.4,0),'trainer':(.3,-3.4,0),'teacher':(1.4,-3.4,0),'scientist':(2.5,-3.4,0)}
for name,col in ASSETS.items():
 for obj in col.objects:obj.location+=Vector(offsets[name])
begin('PRESENTATION only')
cube('Warm paper studio',(0,0,-.12),(200,200,.18),'cream')
world=bpy.data.worlds.new('Warm atelier');scene.world=world;world.use_nodes=True
world.node_tree.nodes['Background'].inputs['Color'].default_value=(.25,.35,.42,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.5
for name,loc,energy,size,color in [('Key',(-7,-10,15),1900,9,(1,.84,.64)),('Fill',(8,-2,10),1000,8,(.70,.84,1)),('Rim',(1,10,14),1700,7,(1,.95,.78))]:
 bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name=name;o.data.energy=energy;o.data.shape='DISK';o.data.size=size;o.data.color=color;o.rotation_euler=(Vector((0,0,1))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(13,-21,18));cam=bpy.context.object;cam.name='Art kit camera';cam.rotation_euler=(Vector((-.1,.15,1))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=17;scene.camera=cam
scene.render.engine='CYCLES';scene.cycles.samples=128;scene.cycles.use_denoising=False
scene.render.resolution_x=1600;scene.render.resolution_y=1100;scene.render.resolution_percentage=100
scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.filepath=str(P/'new-bark-asset-kit.png')
scene['Asset source note']='Original reusable 3D art. Seven separate GLBs and mesh JSON files. No textures, sprites, or ground baked into models.'
scene['Source coordinates']='Z-up, front -Y. Export files Y-up, front +Z, floor-centered. Overview transforms exist only in this presentation blend.'
for screen in bpy.data.screens:
 for area in screen.areas:
  if area.type=='VIEW_3D':area.spaces.active.region_3d.view_perspective='CAMERA'
bpy.ops.wm.save_as_mainfile(filepath=str(P/'new-bark-modular-assets.blend'))
if '--skip-preview' not in args:
 bpy.ops.render.render(write_still=True)
print('FINAL_STATS='+json.dumps(STATS))
