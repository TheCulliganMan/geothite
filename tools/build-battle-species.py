#!/usr/bin/env python3
"""Author original, editable species-specific battle geometry with Blender 4.3+.

blender -b -t 2 --python tools/build-battle-species.py -- /tmp/battle-species
Every collection has named anatomy, a floor root and outward solid surfaces.
No pack, image, ROM, external mesh or extracted game data is read by this file.
Export convention: +Y up, front +Z. Blender convention: +Z up, front -Y.
"""
import argparse, json, math, sys
from pathlib import Path
import bpy, bmesh
from mathutils import Vector, Matrix
A=argparse.ArgumentParser();A.add_argument('out',type=Path);A.add_argument('--only',default='');A.add_argument('--preview',action='store_true');A.add_argument('--review-kadabra-source',action='store_true',help='Build detailed Kadabra authoring source and candidate JSON for explicit geometry review');A.add_argument('--batch-size',type=int,default=12)
args=A.parse_args(sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else [])
OUT=args.out.resolve();OUT.mkdir(parents=True,exist_ok=True)
COL=None;MATS={};BUILDERS={};HEIGHTS={}
P={'ink':(.10,.12,.18),'black':(.15,.17,.21),'white':(.97,.96,.89),'cream':(.95,.85,.62),'red':(.86,.20,.23),'coral':(.95,.40,.38),'pink':(.95,.62,.69),'rose':(.73,.32,.44),'yellow':(.98,.80,.23),'gold':(.81,.57,.20),'orange':(.93,.43,.18),'brown':(.43,.27,.15),'tan':(.71,.53,.32),'green':(.31,.61,.33),'leaf':(.50,.74,.34),'darkgreen':(.15,.35,.28),'teal':(.20,.56,.55),'blue':(.35,.61,.82),'navy':(.18,.33,.50),'purple':(.49,.33,.65),'lilac':(.67,.55,.79),'silver':(.69,.76,.79),'stone':(.52,.55,.51),'darkstone':(.32,.36,.37)}
def lin(v):return v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4
def mat(c):
 if c not in MATS:
  m=bpy.data.materials.new(c);m.diffuse_color=tuple(lin(v) for v in P[c])+(1,);m.use_nodes=True;n=m.node_tree.nodes['Principled BSDF'];n.inputs['Base Color'].default_value=m.diffuse_color;n.inputs['Roughness'].default_value=.73;n.inputs['Metallic'].default_value=.23 if c=='silver' else 0;MATS[c]=m
 return MATS[c]
def obj(name,me,p=(0,0,0),color='cream',smooth=True):
 o=bpy.data.objects.new(name,me);COL.objects.link(o);o.location=p;me.materials.append(mat(color))
 for f in me.polygons:f.use_smooth=smooth
 return o
def sph(n,p,s,c='cream',seg=12,rings=8):
 bm=bmesh.new();bmesh.ops.create_uvsphere(bm,u_segments=seg,v_segments=rings,radius=1);bmesh.ops.scale(bm,vec=s,verts=bm.verts);me=bpy.data.meshes.new(n);bm.to_mesh(me);bm.free();return obj(n,me,p,c)
def ico(n,p,s,c,sub=1):
 bm=bmesh.new();bmesh.ops.create_icosphere(bm,subdivisions=sub,radius=1);bmesh.ops.scale(bm,vec=s,verts=bm.verts);me=bpy.data.meshes.new(n);bm.to_mesh(me);bm.free();return obj(n,me,p,c,False)
def rod(n,a,b,r,c,r2=None,seg=10):
 a,b=Vector(a),Vector(b);bm=bmesh.new();bmesh.ops.create_cone(bm,cap_ends=True,cap_tris=False,segments=seg,radius1=r,radius2=r if r2 is None else r2,depth=(b-a).length);me=bpy.data.meshes.new(n);bm.to_mesh(me);bm.free();o=obj(n,me,(a+b)/2,c);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o
def tube(n,pts,r,c):
 # A capped, parallel-transported round section, not a camera-facing strip.
 color=c;ps=[Vector(p) for p in pts]
 if 3<=len(ps)<=20 and n not in ('Curled crown tuft','Curled forehead lock') and any(word in n.lower() for word in ('tail','serpent','neck','vine','antenna','trunk','tentacle','whisker','moustache','hair','curl','ribbon','horn','stamen','ear')):
  radii=list(r) if isinstance(r,(list,tuple)) else [r]*len(ps);smooth=[];rr=[]
  for i in range(len(ps)-1):
   a,b,c,d=ps[max(i-1,0)],ps[i],ps[i+1],ps[min(i+2,len(ps)-1)]
   for j in range(4):
    t=j/4;smooth.append(.5*((2*b)+(-a+c)*t+(2*a-5*b+4*c-d)*t*t+(-a+3*b-3*c+d)*t*t*t));rr.append(radii[i]*(1-t)+radii[i+1]*t)
  ps=smooth+[ps[-1]];r=rr+[radii[-1]]
 vs=[];side=None;seg=10
 for i,p in enumerate(ps):
  t=(ps[min(i+1,len(ps)-1)]-ps[max(i-1,0)]).normalized()
  if side is None:
   seed=Vector((0,0,1)) if abs(t.z)<.85 else Vector((1,0,0));side=t.cross(seed).normalized()
  u=(side-t*side.dot(t)).normalized();v=t.cross(u).normalized();side=u
  rr=r[i] if isinstance(r,(list,tuple)) else r
  for j in range(seg):vs.append(p+rr*(u*math.cos(j*math.tau/seg)+v*math.sin(j*math.tau/seg)))
 fs=[tuple(reversed(range(seg))),tuple((len(ps)-1)*seg+j for j in range(seg))]
 for i in range(len(ps)-1):
  for j in range(seg):a=i*seg+j;b=i*seg+(j+1)%seg;fs.append((a,b,b+seg,a+seg))
 return poly(n,vs,fs,color,True)
def poly(n,vs,fs,c,smooth=False):
 me=bpy.data.meshes.new(n);me.from_pydata(vs,[],fs);me.update();bm=bmesh.new();bm.from_mesh(me);bmesh.ops.recalc_face_normals(bm,faces=bm.faces);bm.to_mesh(me);bm.free();return obj(n,me,color=c,smooth=smooth)
def fin(n,pts,c,thick=.035):
 # Authored closed cross-sections with a raised central ridge.
 pts=[Vector(p) for p in pts];center=sum(pts,Vector())/len(pts)
 normal=sum(((pts[i]-center).cross(pts[(i+1)%len(pts)]-center) for i in range(len(pts))),Vector())
 if normal.length_squared<1e-10:raise ValueError('Collinear outlined anatomy: '+n)
 normal.normalize();vs=pts+[center+normal*thick,center-normal*thick];k=len(pts);fs=[]
 for i in range(k):fs.extend([(i,(i+1)%k,k),((i+1)%k,i,k+1)])
 return poly(n,vs,fs,c)
def leaf(n,a,b,w,c='leaf'):
 a,b=Vector(a),Vector(b);mid=(a+b)*.5;u=(b-a).cross(Vector((0,1,.05))).normalized()*w
 return fin(n,[a,mid+u,b,mid-u],c,.035)
def ring(n,p,r,minor,c,rot=(math.pi/2,0,0)):
 vs=[];fs=[];M=Matrix.Translation(p)@__import__('mathutils').Euler(rot).to_matrix().to_4x4()
 for i in range(20):
  a=i*math.tau/20
  for j in range(6):b=j*math.tau/6;vs.append(M@Vector(((r+minor*math.cos(b))*math.cos(a),(r+minor*math.cos(b))*math.sin(a),minor*math.sin(b))))
 for i in range(20):
  for j in range(6):fs.append((i*6+j,i*6+(j+1)%6,((i+1)%20)*6+(j+1)%6,((i+1)%20)*6+j))
 return poly(n,vs,fs,c,True)
def eyes(z,y,spread=.13,size=.040,color='ink',white=True):
 for s in (-1,1):
  if white:sph('Eye sclera',(s*spread,y+.008,z),(size*1.6,size*.6,size*1.9),'white')
  sph('Eye iris',(s*spread,y-.015,z),(size,size*.45,size*1.3),color)
  sph('Eye catchlight',(s*spread-size*.25,y-.035,z+size*.38),(size*.24,size*.2,size*.25),'white',8,6)
def mouth(y,z,w=.10):tube('Mouth seam',[(-w,y,z+.014),(0,y-.015,z-.015),(w,y,z+.014)],.009,'ink')
def feet(c='cream',x=.17,y=-.06,z=.07,s=(.10,.16,.07)):
 for d in (-1,1):
  rod('Short lower leg',(d*x*.78,y+.03,z+s[2]+.16),(d*x,y,z+s[2]*.5),min(s[0]*.60,.065),c)
  sph('Left foot' if d<0 else 'Right foot',(d*x,y,z),s,c)
def arms(c='cream',z=.52,x=.26):
 for s in (-1,1):rod('Upper arm',(s*x,0,z),(s*(x+.13),-.09,z-.13),.06,c);sph('Hand',(s*(x+.13),-.11,z-.14),(.085,.075,.08),c)
def quad(c='tan',body=(.27,.39,.25),head=(.20,-.34,.58),tail=True):
 sph('Torso',(0,.06,.33),body,c);sph('Head',(0,head[1],head[2]),(head[0],head[0]*.9,head[0]),c)
 for x in (-.18,.18):
  for y in (-.20,.26):rod('Leg',(x,y,.33),(x,y-.02,.08),.065,c);sph('Paw',(x,y-.04,.065),(.085,.12,.065),c)
 if tail:tube('Tail',[(0,.35,.39),(0,.54,.47),(.07,.62,.62)],.055,c)
def spikes(n,base,amount,radius,height,c):
 for i in range(amount):a=i*math.tau/amount;rod(n,(base[0]+radius*math.cos(a),base[1]+radius*math.sin(a),base[2]),(base[0]+radius*1.3*math.cos(a),base[1]+radius*1.3*math.sin(a),base[2]+height),.05,c,.002)
def flame(n,p,h=.35):
 x,y,z=p
 for i,(dx,dy,sz) in enumerate([(-.09,0,.70),(.07,.02,.86),(0,0,1)]):
  fin(n+' outer',[(x+dx-.09,y+dy,z),(x+dx-.07,y+dy,z+h*.45),(x+dx+.035,y+dy,z+h*sz),(x+dx+.08,y+dy,z+h*.26)],'orange',.05)
  fin(n+' core',[(x+dx-.045,y-.055,z),(x+dx,y-.055,z+h*sz*.68),(x+dx+.045,y-.055,z+.035)],'yellow',.022)
def spiral(n,p,r,c='ink',turns=2,th=.01):
 x,y,z=p;pts=[]
 for i in range(49):a=i/48*math.tau*turns;q=min(r*.45,th*2.8)+(r-min(r*.45,th*2.8))*i/48;pts.append((x+q*math.cos(a),y,z+q*math.sin(a)))
 tube(n,pts,th,c)
def register(names,height=1):
 def deco(fn):
  for name in names.split():BUILDERS[name.lower()]=lambda name=name:fn(name);HEIGHTS[name.lower()]=height
  return fn
 return deco


# Individually art-directed hero silhouettes. Shared-family builders below remain
# unchanged; the helpers here are used only by Gengar, Kadabra and Raticate.
def sculpt_join(n,objects,voxel=.018,ratio=.46):
 bpy.ops.object.select_all(action='DESELECT')
 for o in objects:o.select_set(True)
 bpy.context.view_layer.objects.active=objects[0];bpy.ops.object.join();o=bpy.context.object;o.name=n
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 mod=o.modifiers.new('Continuous clay anatomy','REMESH');mod.mode='VOXEL';mod.voxel_size=voxel;mod.use_smooth_shade=True;bpy.ops.object.modifier_apply(modifier=mod.name)
 mod=o.modifiers.new('Sculpt transition smoothing','SMOOTH');mod.factor=.55;mod.iterations=3;bpy.ops.object.modifier_apply(modifier=mod.name)
 mod=o.modifiers.new('Bounded sculpture topology','DECIMATE');mod.ratio=ratio;bpy.ops.object.modifier_apply(modifier=mod.name)
 bm=bmesh.new();bm.from_mesh(o.data);bmesh.ops.remove_doubles(bm,verts=list(bm.verts),dist=.000001);bmesh.ops.dissolve_degenerate(bm,edges=list(bm.edges),dist=.000001)
 unseen=set(bm.faces)
 while unseen:
  group=set();queue=[unseen.pop()]
  while queue:
   f=queue.pop();group.add(f)
   for e in f.edges:
    for neighbor in e.link_faces:
     if neighbor in unseen:unseen.remove(neighbor);queue.append(neighbor)
  if len(group)<8:bmesh.ops.delete(bm,geom=list(group),context='FACES')
 bmesh.ops.recalc_face_normals(bm,faces=bm.faces);bm.to_mesh(o.data);bm.free()
 for f in o.data.polygons:f.use_smooth=True
 o.data.update()
 for f in o.data.polygons:
  if f.normal.dot(sum((o.data.vertices[i].normal for i in f.vertices),Vector()))<.05:f.use_smooth=False
 o.select_set(False);return o

def canonicalize_hero_mesh(o):
 # Stable topology before simplification avoids allocator-dependent BMesh
 # iteration ordering changing the decimator's equally weighted choices.
 me=o.data;coordinates=[tuple(round(v,7) for v in vert.co) for vert in me.vertices]
 ordered=sorted(set(coordinates));lookup={v:i for i,v in enumerate(ordered)};remap=[lookup[v] for v in coordinates];faces=[]
 for f in me.polygons:
  indices=tuple(remap[i] for i in f.vertices)
  if len(set(indices))<3:continue
  indices=min(indices[i:]+indices[:i] for i in range(len(indices)))
  faces.append((indices,f.use_smooth))
 faces.sort();replacement=bpy.data.meshes.new(me.name+' stable topology');replacement.from_pydata(ordered,[],[f[0] for f in faces]);replacement.update()
 for material in me.materials:replacement.materials.append(material)
 for f,source in zip(replacement.polygons,faces):f.use_smooth=source[1]
 o.data=replacement

def finish_hero_normals(objects):
 # Small, sharply bent finger tips retain face normals when averaging across
 # the bend would point inward. The rest of each sculpture stays smoothly lit.
 for o in objects:
  if o.type!='MESH':continue
  canonicalize_hero_mesh(o)
  me=o.data;bm=bmesh.new();bm.from_mesh(me);bmesh.ops.triangulate(bm,faces=list(bm.faces),quad_method='FIXED',ngon_method='EAR_CLIP');bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces))
  if bm.calc_volume(signed=True)<0:bmesh.ops.reverse_faces(bm,faces=list(bm.faces))
  bm.to_mesh(me);bm.free();me.update()
  for attempt in range(3):
   bad=[]
   for f in me.polygons:
    if not f.use_smooth:continue
    n=sum((me.corner_normals[i].vector for i in f.loop_indices),Vector())
    if f.normal.dot(n)<.000001:bad.append(f)
   if not bad:break
   for f in bad:f.use_smooth=False
   me.update()

def sculpt_runtime_lod(name,objects):
 # Keep the editable detailed sculpt beside the bounded game geometry. These
 # hidden source collections are never exported or counted as runtime parts.
 high=bpy.data.collections.new(name+' / detailed authoring source');bpy.context.scene.collection.children.link(high)
 high.hide_render=True;high.hide_viewport=True
 root=bpy.data.objects.new(name+' / detailed source root',None);high.objects.link(root);root['authoring_only']=True
 for o in objects:
  duplicate=o.copy();duplicate.data=o.data.copy();duplicate.name=o.name+' / detailed source';high.objects.link(duplicate);duplicate.parent=root
  o.data.calc_loop_triangles();count=len(o.data.loop_triangles)
  if count<=160 or 'crescent smile' in o.name or 'spoon bowl' in o.name:continue
  ratio=.30 if 'continuous' in o.name else (.35 if 'muscular hind leg' in o.name else .42)
  bpy.context.view_layer.objects.active=o
  modifier=o.modifiers.new('Game-resolution sculpture LOD','DECIMATE');modifier.ratio=ratio;modifier.use_collapse_triangulate=True;bpy.ops.object.modifier_apply(modifier=modifier.name)
 finish_hero_normals(objects)
 bpy.context.view_layer.update();points=[o.matrix_world@v.co for o in objects for v in o.data.vertices]
 low=Vector(tuple(min(p[a] for p in points) for a in range(3)));high=Vector(tuple(max(p[a] for p in points) for a in range(3)))
 scale=HEIGHTS[name]/(high.z-low.z);center=Vector(((low.x+high.x)*.5,(low.y+high.y)*.5,low.z))
 for o in objects:o.location=(o.location-center)*scale;o.scale*=scale


def sculpt_loft(n,rings,c,seg=16):
 # Rings are (x,y,z,width,depth), so each shape has an authored profile.
 vs=[]
 for x,y,z,w,d in rings:
  for j in range(seg):a=j*math.tau/seg;vs.append((x+w*math.cos(a),y+d*math.sin(a),z))
 fs=[tuple(reversed(range(seg))),tuple((len(rings)-1)*seg+j for j in range(seg))]
 for i in range(len(rings)-1):
  for j in range(seg):a=i*seg+j;b=i*seg+(j+1)%seg;fs.append((a,b,b+seg,a+seg))
 return poly(n,vs,fs,c,True)

def sculpt_patch(n,outline,surface,c,depth=.009):
 # Closed, gently raised surface conforming to an anatomical volume.
 # Each radial band follows the surface, not a flat camera-facing polygon.
 k=len(outline);cx=sum(p[0] for p in outline)/k;cz=sum(p[1] for p in outline)/k
 vs=[]
 for off in (0,depth):
  vs.append((cx,surface(cx,cz)+off,cz))
  for t in (.35,.7,1):
   for x,z in outline:
    px=cx+(x-cx)*t;pz=cz+(z-cz)*t;vs.append((px,surface(px,pz)+off,pz))
 layer=1+k*3;fs=[]
 for base,flip in ((0,False),(layer,True)):
  for j in range(k):
   f=(base,base+1+j,base+1+(j+1)%k);fs.append(tuple(reversed(f)) if flip else f)
  for ring in range(2):
   for j in range(k):
    a=base+1+ring*k+j;b=base+1+ring*k+(j+1)%k;f=(a,b,b+k,a+k);fs.append(tuple(reversed(f)) if flip else f)
 for j in range(k):a=1+2*k+j;b=1+2*k+(j+1)%k;fs.append((a,b,b+layer,a+layer))
 return poly(n,vs,fs,c,False)

def sculpt_box(n,p,size,c,bevel=.007):
 bpy.ops.mesh.primitive_cube_add(size=1,location=p);o=bpy.context.object
 for coll in list(o.users_collection):coll.objects.unlink(o)
 COL.objects.link(o);o.name=n;o.dimensions=size;bpy.context.view_layer.objects.active=o;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);o.data.materials.append(mat(c))
 mod=o.modifiers.new('Soft enamel edge','BEVEL');mod.width=bevel;mod.segments=2;bpy.ops.object.modifier_apply(modifier=mod.name);o.select_set(False);return o

def sculpt_gengar():
 P['gengar_violet']=(.39,.29,.57);P['gengar_eye']=(.92,.28,.31);P['gengar_mouth']=(.16,.10,.24)
 c='gengar_violet';clay=[sph('Broad pear-shaped core',(0,0,.47),(.38,.285,.375),c,40,24)]
 for s in (-1,1):
  clay.append(sculpt_loft('Pointed cranial ear',[(s*.235,.01,.70,.125,.11),(s*.28,.025,.86,.105,.08),(s*.35,.04,1.02,.008,.006)],c,12))
  clay.append(sph('Compact connected thigh',(s*.22,-.025,.19),(.145,.15,.16),c,20,12))
  clay.append(sph('Broad planted foot',(s*.24,-.13,.075),(.13,.18,.075),c,20,12))
  clay.append(tube('Short reaching arm',[(s*.30,0,.49),(s*.40,-.08,.43),(s*.47,-.15,.39)],[.095,.089,.082],c))
 for i,(x,y,z,tx,ty,tz,r) in enumerate([(-.18,.16,.73,-.19,.22,.89,.09),(0,.15,.77,.01,.22,.94,.085),(.17,.16,.73,.22,.22,.87,.09),(-.25,.20,.56,-.41,.31,.66,.12),(.25,.20,.56,.41,.31,.66,.12),(-.14,.245,.40,-.20,.39,.47,.10),(.14,.245,.40,.21,.39,.48,.10),(0,.22,.25,0,.39,.24,.105)]):clay.append(rod('Layered dorsal spine',(x,y,z),(tx,ty,tz),r,c,.004,12))
 sculpt_join('Gengar / continuous body ears limbs and dorsal spines',clay,.014,.28)
 for s in (-1,1):
  for j in (-1,0,1):
   a=(s*(.442+.018*(j+1)),-.178-.029*j,.39+.043*j);b=(s*(.51+.020*(j+1)),-.23-.020*j,.37+.054*j)
   tube('Three tapered hand digits',[a,b,(b[0]+s*.017,b[1]-.016,b[2]+.016)],[.043,.028,.003],c)
  for j in (-1,0,1):rod('Subtle foot digit',(s*.24+j*.045,-.24,.072),(s*.24+j*.053,-.30,.060),.032,c,.010,10)
 surface=lambda x,z:-.285*math.sqrt(max(.035,1-(x/.38)**2-((z-.47)/.375)**2))-.011
 outline=[]
 for i in range(21):u=-1+i/10;outline.append((.303*u,.46+.108*u*u))
 for i in range(20,-1,-1):u=-1+i/10;outline.append((.303*u,.318+.25*u*u))
 sculpt_patch('Recessed broad crescent smile',outline,lambda x,z:surface(x,z)-.001,'gengar_mouth',.011)
 # Separate continuous white tooth field follows the smile; fine gaps end at
 # the exact crescent boundary, never hang as comb-like projecting rods.
 for j in range(7):
  u0=-.96+j*1.92/7+.010;u1=-.96+(j+1)*1.92/7-.010;tooth=[]
  for i in range(5):u=u0+(u1-u0)*i/4;tooth.append((.303*u,.452+.108*u*u))
  for i in range(4,-1,-1):u=u0+(u1-u0)*i/4;tooth.append((.303*u,.327+.239*u*u))
  sculpt_patch('Curved ivory grin tooth',tooth,lambda x,z:surface(x,z)-.006,'white',.005)
 for s in (-1,1):
  eye=[(s*.047,.607),(s*.266,.719),(s*.276,.674),(s*.265,.628),(s*.233,.602),(s*.176,.590),(s*.111,.593)]
  sculpt_patch('Inset angular crimson eye',eye,lambda x,z:surface(x,z)-.005,'gengar_eye',.008)
  x=s*.187;z=.640
  # Thin slit pupil and a restrained highlight keep the eyes inset.
  sculpt_patch('Vertical black eye slit',[(x-.008,z+.030),(x+.008,z+.030),(x+.006,z-.021),(x-.005,z-.021)],lambda x,z:surface(x,z)-.014,'gengar_mouth',.005)
  brow=[(s*.029,.604),(s*.066,.638),(s*.270,.743),(s*.266,.719)]
  sculpt_patch('Heavy expressive upper eyelid',brow,lambda x,z:surface(x,z)-.009,c,.010)

def sculpt_kadabra():
 P['kadabra_yellow']=(.94,.74,.22);P['kadabra_armor']=(.38,.30,.26);P['kadabra_moustache']=(.99,.87,.39)
 c='kadabra_yellow';armor='kadabra_armor'
 sculpt_loft('Tapered abdomen',[(0,.02,.24,.13,.12),(0,.015,.33,.18,.145),(0,.012,.48,.155,.12),(0,0,.62,.11,.085)],c,20)
 sculpt_loft('Angular chest cuirass',[(0,.005,.43,.135,.108),(0,.006,.48,.205,.16),(0,.03,.60,.228,.145),(0,.025,.70,.155,.12)],armor,12)
 sculpt_loft('Narrow neck',[(0,.0,.65,.081,.082),(0,-.01,.875,.10,.092)],c,12)
 # Deliberately wedge-shaped fox skull, projecting tapered muzzle and tall ears.
 # Cross-sections run from the occiput to the tapered projecting muzzle.
 vs=[];rings=[(.11,.155,.85,1.00),(-.10,.209,.839,1.034),(-.245,.090,.854,.957),(-.365,.023,.871,.902)]
 for y,w,low,high in rings:
  mid=(low+high)*.5
  for x,z in [(-w*.64,low),(w*.64,low),(w,mid-.025),(w*.82,high-.017),(w*.45,high),(-w*.45,high),(-w*.82,high-.017),(-w,mid-.025)]:vs.append((x,y,z))
 fs=[tuple(reversed(range(8))),tuple(24+j for j in range(8))]
 for i in range(3):
  for j in range(8):a=i*8+j;b=i*8+(j+1)%8;fs.append((a,b,b+8,a+8))
 head=poly('Sculpted fox skull and projecting tapered muzzle',vs,fs,c,False)
 bpy.context.view_layer.objects.active=head;mod=head.modifiers.new('Softened fox skull planes','BEVEL');mod.width=.012;mod.segments=3;bpy.ops.object.modifier_apply(modifier=mod.name)
 for s in (-1,1):
  poly('Long tapered fox ear',[(s*.105,.03,.975),(s*.232,.04,.94),(s*.294,.075,1.275),(s*.171,-.016,1.095),(s*.219,.123,1.081)],[(0,1,2,3),(0,4,1),(1,4,2),(2,4,3),(3,4,0)],c)
  fin('Recessed ochre ear plane',[(s*.164,.014,1.039),(s*.224,.015,1.034),(s*.269,.055,1.209)],'gold',.007)
  # Raised three-dimensional triangular eye sockets follow the sloped cheeks.
  ey=[(s*.063,.949),(s*.174,.973),(s*.153,.926),(s*.098,.928)]
  surf=lambda x,z:-.247+.73*(abs(x)-.09)
  sculpt_patch('Narrow focused white eye',ey,surf,'white',.012)
  px=s*.111
  sculpt_patch('Focused slit pupil',[(px-.007,.949),(px+.008,.953),(px+.008,.920),(px-.006,.920)],lambda x,z:surf(x,z)-.009,'ink',.006)
  sculpt_patch('Angular brow plane',[(s*.056,.952),(s*.170,.981),(s*.177,.973),(s*.064,.948)],lambda x,z:surf(x,z)-.004,c,.006)
  # Broad flat upper moustache sweeps are thick crescent volumes.
  pts=[(s*.018,-.16,.902),(s*.048,-.326,.874),(s*.155,-.342,.833),(s*.273,-.331,.747),(s*.348,-.293,.674)]
  # Oval cross-section keeps moustache tapered and ribbonlike, not wire whiskers.
  o=tube('Long swept moustache',pts,[.030,.041,.037,.024,.002],'kadabra_moustache')
  for v in o.data.vertices:v.co.y=(v.co.y+.33)*.40-.33
  # Shoulder pads, narrow upper arm, wider forearms and articulated hands.
  o=sculpt_loft('Sculpted shoulder armor',[(s*.210,.012,.531,.068,.076),(s*.216,.013,.593,.116,.123),(s*.210,.014,.684,.130,.125),(s*.191,.014,.757,.074,.084)],armor,16)
  elbow=(s*.34,-.045,.57 if s<0 else .68);wrist=(s*.39,-.18,.68 if s<0 else .89)
  tube('Bent upper arm',[(s*.225,-.015,.635),elbow],[.050,.043],c)
  tube('Sculpted tapered forearm',[elbow,((elbow[0]+wrist[0])*.5,-.105,(elbow[2]+wrist[2])*.5),wrist],[.051,.069,.037],c)
  palm=sph('Three-fingered hand palm',wrist,(.066,.050,.068),c,16,10)
  for j in (-1,0,1):
   x=wrist[0]+j*.043;z=wrist[2]+.028
   tube('Curled hand finger',[(x,-.19,z),(x+s*.010,-.232,z+.085),(x+s*.007,-.258,z+.048)],[.023,.020,.012],c)
   rod('Pale finger claw',(x+s*.007,-.258,z+.048),(x+s*.006,-.274,z+.019),.014,'white',.001,10)
  # Heavy bent thighs and digitigrade lower legs ground the pose.
  thigh=(s*.21,.02,.29);knee=(s*.29,-.10,.26);ankle=(s*.245,-.045,.075)
  sculpt_join('Bent muscular hind leg',[sph('Haunch',thigh,(.14,.13,.14),c,20,12),tube('Bent shin',[knee,(s*.27,-.065,.16),ankle],[.067,.065,.04],c)],.014,.55)
  sph('Splayed hind foot',(s*.245,-.105,.054),(.098,.115,.05),c,16,10)
  for j in (-1,0,1):
   a=(s*.245+j*.046,-.153,.050);b=(s*.245+j*.058,-.237,.040)
   rod('Three long hind toes',a,b,.032,c,.016,10);rod('Ivory toe claw',b,(b[0],b[1]-.046,.026),.019,'white',.001,10)
 # Tail has the characteristic substantial golden curl and dark saddle bands.
 tube('Heavy curved psychic tail',[(0,.125,.30),(0,.33,.23),(-.05,.48,.26),(-.10,.56,.46),(-.07,.50,.66)],[.11,.145,.13,.095,.018],c)
 tube('Broad tail saddle band',[(0,.26,.241),(-.015,.36,.230)],[.149,.145],armor)
 # Three pink abdomen waves belong to Kadabra alone.
 for dx in (-.058,0,.058):tube('Three wavy abdomen markings',[(dx,-.141,.422),(dx-.016,-.153,.390),(dx+.015,-.151,.350),(dx+.002,-.133,.315)],.011,'rose')
 # Five-point forehead star; all vertices project onto the forehead plane.
 star=[]
 for i in range(10):a=math.pi/2+i*math.pi/5;r=.029 if i%2==0 else .013;star.append((r*math.cos(a),.996+r*math.sin(a)))
 sculpt_patch('Five-point red forehead star',star,lambda x,z:-.11-(1.034-z)*1.753-.007,'red',.006)
 # A true concave bowl with a thick rim, held above the right hand.
 tube('Held silver spoon handle',[(.387,-.228,.803),(.394,-.245,.927),(.40,-.25,1.035)],[.013,.012,.010],'silver')
 seg=24;vs=[];fs=[]
 for layer in (0,1):
  for ring in range(5):
   r=ring/4
   for j in range(seg):a=j*math.tau/seg;vs.append((.400+.052*r*math.cos(a),-.246+.022*(1-r*r)+layer*.007,1.082+.076*r*math.sin(a)))
 for layer in (0,1):
  for ring in range(4):
   for j in range(seg):a=layer*5*seg+ring*seg+j;b=layer*5*seg+ring*seg+(j+1)%seg;fs.append((a,b,b+seg,a+seg))
 for j in range(seg):a=4*seg+j;b=4*seg+(j+1)%seg;fs.append((a,b,b+5*seg,a+5*seg))
 poly('Hollow silver spoon bowl',vs,fs,'silver',True)

def sculpt_raticate():
 P['raticate_fur']=(.64,.42,.20);P['raticate_cream']=(.95,.84,.59);P['raticate_ear']=(.54,.39,.31)
 c='raticate_fur';cream='raticate_cream'
 core=[sph('Pear-shaped rat body',(0,.075,.36),(.34,.305,.335),c,32,20),sph('Broad forward head',(0,-.13,.635),(.295,.235,.25),c,32,20)]
 for s in (-1,1):
  core.append(sph('Powerful low haunch',(s*.245,.07,.245),(.165,.19,.20),c,20,12))
  for i,(x,z,dx,dz) in enumerate([(.235,.77,.095,.075),(.27,.70,.10,.018),(.292,.52,.10,-.035),(.285,.43,.097,-.035)]):
   core.append(rod('Cheek and shoulder fur tuft',(s*x,-.02,z),(s*(x+dx),-.018,z+dz),.064,c,.003,9))
 sculpt_join('Raticate / continuous pear body head and fur silhouette',core,.016,.30)
 # Underside is nested into the body rather than hanging as a separate pouch.
 sph('Broad cream belly',(0,-.185,.315),(.263,.112,.258),cream,28,16)
 for s in (-1,1):
  ear=sph('Flared cupped rat ear',(s*.248,-.065,.823),(.105,.06,.132),c,20,12);ear.rotation_euler[1]=s*.48
  inner=sph('Recessed ear interior',(s*.260,-.116,.830),(.072,.019,.096),'raticate_ear',20,12);inner.rotation_euler[1]=s*.48
  # Low slanting eyes replace generic round protruding buttons.
  outline=[(s*.092,.744),(s*.253,.762),(s*.235,.719),(s*.150,.723)]
  surf=lambda x,z:-.13-.235*math.sqrt(max(.05,1-(x/.299)**2-((z-.635)/.25)**2))-.008
  sculpt_patch('Slanted almond eye',outline,surf,'white',.009)
  x=s*.162
  sculpt_patch('Focused small rat pupil',[(x-.009,.748),(x+.010,.748),(x+.009,.724),(x-.008,.724)],lambda x,z:surf(x,z)-.010,'ink',.006)
  sculpt_patch('Natural upper eyelid',[(s*.083,.747),(s*.249,.774),(s*.253,.762),(s*.092,.743)],lambda x,z:surf(x,z)-.002,c,.006)
 # Cream cheek masses wrap around a real recessed mouth and square incisors.
 sph('Dark open mouth cavity',(0,-.320,.545),(.132,.063,.138),'ink',24,16)
 sph('Lower cream jaw',(0,-.318,.423),(.114,.072,.046),cream,24,12)
 sph('Lower mouth tongue',(0,-.362,.463),(.081,.017,.038),'rose',20,12)
 for s in (-1,1):
  cheek=sph('Cream whisker cheek',(s*.145,-.300,.592),(.151,.101,.105),cream,24,14);cheek.rotation_euler[1]=s*.12
  inc=sculpt_box('Broad upper chisel incisor',(s*.042,-.392,.559),(.076,.045,.125),'white',.008);inc.rotation_euler[0]=-.09
  inc=sculpt_box('Short lower chisel incisor',(s*.037,-.386,.449),(.068,.036,.044),'white',.006)
  # Short reaching forepaws sit high on the belly, with visibly separate claws.
  tube('Short bent forearm',[(s*.28,-.14,.435),(s*.30,-.225,.386),(s*.265,-.284,.411)],[.066,.063,.046],c)
  sph('Small grasping forepaw',(s*.260,-.280,.415),(.065,.048,.046),cream,16,10)
  for j in (-1,0,1):
   x=s*.26+j*.032;tip=(x+s*.01,-.348,.407+.01*j)
   tube('Three curled forepaw digits',[(x,-.302,.433),(x,-.330,.433),tip],[.019,.017,.008],cream)
   rod('Forepaw claw',tip,(tip[0],tip[1]-.018,tip[2]-.018),.009,'white',.001,8)
  sph('Long splayed rat foot',(s*.251,-.04,.053),(.115,.174,.051),cream,20,12)
  for j in (-1,0,1):
   x=s*.251+j*.049;rod('Long rat toe',(x,-.14,.048),(x+j*.011,-.240,.030),.026,cream,.012,10);rod('Rat toe claw',(x+j*.011,-.238,.032),(x+j*.017,-.273,.015),.014,'white',.001,10)
  # Six long tapered whiskers fan from anchored cheek roots.
  for i,(z,dz) in enumerate([(.639,.075),(.611,.005),(.575,-.092)]):
   tube('Tapered facial whisker',[(s*.21,-.373,z),(s*.353,-.375,z+dz*.5),(s*.493,-.330,z+dz)],[.011,.008,.001],cream)
 # Small angular nose stays seated in the cream muzzle.
 poly('Small split rat nose',[(-.040,-.398,.676),(.040,-.398,.676),(0,-.415,.645),(0,-.369,.673)],[(0,1,2),(0,3,1),(1,3,2),(2,3,0)],'raticate_ear',True)
 tube('Nose philtrum',[(0,-.406,.646),(0,-.407,.613)],.006,'brown')
 tube('Long tapered bare rat tail',[(0,.335,.29),(.17,.459,.26),(.365,.505,.36),(.50,.447,.55),(.495,.36,.73),(.433,.30,.86)],[.050,.048,.038,.028,.019,.002],cream)
 for i,(a,b,r) in enumerate([((.131,.438,.263),(.168,.457,.269),.049),((.285,.497,.308),(.314,.501,.327),.044),((.424,.489,.423),(.444,.479,.450),.033)]):rod('Subtle tail segmentation',a,b,r,'tan',r,12)

@register('BAYLEEF MEGANIUM',1.35)
def grass_starter(n):
 big=n=='MEGANIUM';c='leaf' if big else 'yellow';quad(c,(.33,.40,.30),(.21,-.34,.85),True)
 rod('Long dinosaur neck',(0,-.20,.43),(0,-.34,.84),.17,c,.14);eyes(.89,-.53,.10,.035);mouth(-.538,.79,.11)
 for x in (-.18,.18):
  for y in (-.20,.26):
   for d in (-.035,0,.035):sph('Toe claw',(x+d,y-.145,.067),(.025,.040,.025),'white',8,6)
 if big:
  for i in range(8):a=i*math.tau/8;leaf('Broad pink neck flower',(0,-.27,.67),(.36*math.cos(a),-.27+.27*math.sin(a),.61),.13,'pink');sph('Flower gold center',(.18*math.cos(a),-.27+.15*math.sin(a),.69),(.05,.055,.03),'gold')
  for s in (-1,1):tube('Curled head stamen',[(s*.08,-.30,1.02),(s*.13,-.26,1.23),(s*.20,-.26,1.24)],.018,'yellow')
 else:
  for i in range(8):a=i*math.tau/8;leaf('Neck leaf bud',(.17*math.cos(a),-.24+.14*math.sin(a),.56),(.27*math.cos(a),-.24+.24*math.sin(a),.69),.06,'darkgreen')
  leaf('Notched crown leaf',(0,-.31,1.02),(.32,-.18,1.22),.13,'green')
@register('QUILAVA TYPHLOSION',1.25)
def fire_starter(n):
 raise RuntimeError("Fire badgers use dedicated sculptures before Blender batching")
@register('CROCONAW FERALIGATR',1.30)
def sculpted_water_evolutions(n):
 raise RuntimeError('The water evolutions use their dedicated sculpture recipe before Blender batching')
@register('FURRET',1.20)
def furret(n):
 raise ValueError('Furret uses the canonical continuous banded-ferret skin')
@register('NOCTOWL',1.30)
def noctowl(n):
 raise ValueError('Noctowl uses the canonical fitted-faced crowned-owl skin')
@register('CATERPIE WEEDLE',.80)
def caterpillars(n):
 raise ValueError('Caterpie and Weedle use their dedicated connected larva sculptures and skins')
 worm=n=='WEEDLE';c='tan' if worm else 'green'
 for i in range(6):sph('Articulated body segment',(0,.12+i*.12,.14+.03*i),(.12-i*.01,.10,.11-i*.008),c)
 sph('Raised larva head',(0,-.03,.32),(.20,.17,.21),c);eyes(.38,-.184,.11,.042)
 for i in range(4):
  for s in (-1,1):sph('Soft proleg',(s*.10,.13+i*.13,.06),(.05,.06,.05),'pink' if worm else 'yellow')
 if worm:rod('Pale head sting',(0,-.015,.49),(0,.01,.72),.08,'white',.002);rod('Tail sting',(0,.72,.26),(0,.83,.42),.05,'white',.001);sph('Rosy nose',(0,-.19,.29),(.075,.04,.07),'pink')
 else:
  for s in (-1,1):tube('Forked red antenna',[(0,-.04,.49),(s*.08,-.04,.62),(s*.14,-.015,.66)],.023,'red')
  for i in range(3):ring('Yellow side ring',(.111,.20+i*.14,.15),.032,.009,'yellow',(0,math.pi/2,0))
@register('METAPOD KAKUNA',.92)
def cocoons(n):
 raise ValueError('Metapod and Kakuna use their dedicated cocoon sculptures and skins')
@register('SENTRET',1.0)
def ringed_lookout(n):
 raise RuntimeError('Sentret is exported by its dedicated standard-skin recipe before Blender batching')
@register('HOOTHOOT',1.0)
def clock_owl(n):
 raise RuntimeError('Hoothoot is exported by its dedicated standard-skin recipe before Blender batching')
@register('BUTTERFREE BEEDRILL',1.04)
def wing_insects(n):
 pass # Pure wing-insect recipe is dispatched in run().
@register('LEDYBA',.95)
@register('LEDIAN',1.05)
def ladybirds(n):
 raise ValueError('Ladybirds use the canonical distinct six-limb standard skins')
@register('SPINARAK ARIADOS',.90)
def spiders(n):
 raise ValueError('Spinarak and Ariados use the canonical six-limbed spider skins')
@register('ZUBAT GOLBAT CROBAT',1.00)
def bats(n):
 pass # Pure bat recipe is dispatched in run().
@register('MAREEP FLAAFFY',1.15)
def sheep(n):
 pass # Pure sheep recipe is dispatched in run().
@register('WOOPER QUAGSIRE',.98)
def axolotls(n):
 raise ValueError('Wooper and Quagsire use their canonical amphibian skins')
@register('HOPPIP SKIPLOOM JUMPLUFF',.85)
def wind_plants(n):
 pass # Pure wind-plant recipe is dispatched in run().
@register('BELLSPROUT WEEPINBELL VICTREEBEL',1.05)
def pitcher_plants(n):
 raise ValueError('Botanical pitchers use their canonical articulated skins before Blender batching')

@register('IVYSAUR VENUSAUR',1.20)
def bulb_evolutions(n):
 raise ValueError('Ivysaur and Venusaur use bulb_saur_glb before Blender batching')
@register('CHARMELEON CHARIZARD',1.35)
def flame_dragons(n):
 raise RuntimeError("Fire dragons use dedicated sculptures before Blender batching")
@register('WARTORTLE BLASTOISE',1.10)
def turtle_evolutions(n):
 if n in ('WARTORTLE','BLASTOISE'):raise ValueError('Wartortle and Blastoise use their original pure sculpture recipes')
 heavy=n=='BLASTOISE';sph('Brown domed shell',(0,.06,.50),(.33,.24,.38),'brown');sph('Cream shell rim',(0,-.095,.49),(.34,.09,.36),'cream');sph('Plated belly',(0,-.17,.48),(.27,.07,.30),'tan');sph('Blue head',(0,-.08,.91),(.23,.20,.21),'blue');eyes(.96,-.261,.125,.036,'rose');mouth(-.277,.84,.14);feet('blue',.21,s=(.12,.16,.085));arms('blue',.63,.32)
 for z in (.29,.42,.55,.68):tube('Belly plate seam',[(-.23,-.202,z),(0,-.242,z-.015),(.23,-.202,z)],.008,'brown')
 if heavy:
  for s in (-1,1):
   rod('Shoulder cannon',(s*.26,.12,.80),(s*.30,-.24,1.10),.085,'silver',.075);rod('Dark cannon bore',(s*.30,-.241,1.101),(s*.307,-.265,1.125),.056,'ink')
 else:
  for s in (-1,1):leaf('Feathery fin ear',(s*.13,-.03,1.04),(s*.35,.01,1.29),.11,'lilac')
  tube('Long cloud tail',[(0,.26,.31),(0,.48,.42),(.22,.60,.58),(.39,.50,.74),(.36,.35,.81),(.20,.33,.71)],[.10,.09,.075,.055,.04,.015],'lilac')
@register('PIDGEY',.82)
def small_pigeon(n):
 raise ValueError('Pidgey uses its pure sculpture and standard skin before Blender batching')

@register('PIDGEOTTO PIDGEOT FEAROW FARFETCH_D MURKROW',1.10)
def birds(n):
 if n=='PIDGEOTTO':raise ValueError('Pidgeotto uses its pure sculpture and standard skin before Blender batching')
 if n in ('PIDGEOT','FEAROW'):raise ValueError('Pidgeot and Fearow use raptor_bird_glb')
 if n=='FARFETCH_D':raise RuntimeError('Farfetch_d uses its dedicated standard skin recipe before Blender batching')
 if n=='MURKROW':raise RuntimeError('Murkrow uses its dedicated pure sculpture recipe before Blender batching')
 fear=n=='FEAROW';murk=n=='MURKROW';deli=n=='DELIBIRD';c='navy' if murk else ('red' if deli else 'brown');sph('Bird body',(0,.03,.42),(.23,.21,.31),c);sph('Light breast',(0,-.155,.43),(.18,.055,.23),'white' if deli else ('brown' if murk else 'cream'));hz=.97 if fear else .77
 if fear:rod('Long narrow neck',(0,-.025,.54),(0,-.05,.94),.075,c)
 sph('Bird head',(0,-.035,hz),(.18,.17,.17),'white' if deli else c);eyes(hz+.025,-.182,.097,.031);rod('Beak',(0,-.18,hz-.02),(0,-(.58 if fear else .32),hz-.055),.068,'yellow' if murk or deli else 'tan',.002)
 for s in (-1,1):
  rod('Bird shin',(s*.12,0,.20),(s*.12,-.025,.055),.024,'gold')
  for x in (-.045,0,.045):rod('Toe',(s*.12,-.02,.055),(s*.12+x,-.16,.025),.012,'gold')
  if n in ('FEAROW','PIDGEOT','PIDGEOTTO'):wing=[(s*.16,.01,.62),(s*.55,.04,.87),(s*.87,.05,.96),(s*.75,.04,.50),(s*.50,.03,.32),(s*.24,.01,.26)]
  else:wing=[(s*.15,0,.61),(s*.36,.04,.50),(s*.40,.09,.28),(s*.22,.11,.22)]
  fin('Layered wing',wing,c,.055)
  for i in range(4):leaf('Flight feather',(s*(.24+.06*i),.02,.55),(s*(.42+.095*i),.01,.33+.075*i),.045,'tan' if not murk and not deli else c)
 for i in range(3):leaf('Tail feather',(0,.15,.31),((i-1)*.15,.54,.40),.065,'red' if n in ('PIDGEOTTO','PIDGEOT') else c)
 if n in ('PIDGEOTTO','PIDGEOT','SPEAROW','FEAROW'):
  for i in range(3):leaf('Head crest',((i-1)*.05,-.015,hz+.14),((i-1)*.08,.22 if n=='PIDGEOT' else .05,hz+.36),.06,'red')
 if murk:
  for i in (-1,0,1):leaf('Witch-hat crown',(.07*i,-.02,.91),(.13*i,.06,1.13+.06*(i==0)),.09,'navy')
  sph('Red tail tie',(0,.43,.37),(.13,.045,.08),'red')
@register('DELIBIRD',1.10)
def delibird(n):
 pass # Pure feather-sack bird recipe is dispatched in run().
@register('RATICATE',.96)
def raticate(n):
 sculpt_raticate()
@register('RAICHU PICHU',.96)
def electric_mice(n):
 raise ValueError('Raichu and Pichu use their canonical electric mouse sculptures and skins')
@register('SANDSHREW SANDSLASH',1.02)
def sand_mammals(n):
 raise RuntimeError('armored mammals use canonical sand_mammal_glb dispatch')
@register('NIDORAN_F NIDORINA NIDOQUEEN NIDORAN_M NIDORINO NIDOKING',1.12)
def nido(n):
 raise ValueError('All six poison-pin species use their original pure sculptures')
@register('CLEFABLE CLEFFA IGGLYBUFF WIGGLYTUFF',1.00)
def pink_fairies(n):
 raise RuntimeError('all four identities use canonical fairy skin dispatch')
@register('VULPIX NINETALES EEVEE VAPOREON JOLTEON FLAREON ESPEON UMBREON',1.03)
def foxes(n):
 if n=='VAPOREON':raise ValueError('Vaporeon uses its dedicated aquatic-cat sculpture and skin')
 if n=='JOLTEON':raise ValueError('Jolteon uses its dedicated lightning-fox sculpture and skin')
 if n=='FLAREON':raise ValueError('Flareon uses its dedicated flame-fox sculpture and skin')
 if n=='ESPEON':raise ValueError('Espeon uses its dedicated sun-cat sculpture and skin')
 if n=='UMBREON':raise ValueError('Umbreon uses its dedicated moonlight sculpture and skin')
 if n=='VULPIX':raise ValueError('Vulpix uses its dedicated six-tail sculpture and skin')
 if n=='NINETALES':raise ValueError('Ninetales uses its dedicated nine-tail sculpture and skin')
 if n=='EEVEE':raise ValueError('Eevee uses its dedicated evolution-fox sculpture and skin')
 c={'VULPIX':'orange','NINETALES':'cream','EEVEE':'brown','VAPOREON':'blue','JOLTEON':'yellow','FLAREON':'orange','ESPEON':'lilac','UMBREON':'ink'}[n];quad(c,(.25,.30,.23),(.20,-.29,.62),False);eyes(.67,-.47,.105,.034,'red' if n=='UMBREON' else 'ink');sph('Muzzle',(0,-.44,.55),(.14,.09,.063),c);sph('Nose',(0,-.515,.58),(.035,.018,.022),'ink')
 for s in (-1,1):
  leaf('Tall fox ear',(s*.12,-.24,.74),(s*.30,-.20,1.02),.10,c);leaf('Inset ear',(s*.145,-.29,.76),(s*.27,-.245,.96),.06,'rose' if n=='ESPEON' else 'brown')
  if n=='UMBREON':ring('Gold ear marking',(s*.24,-.235,.90),.046,.014,'yellow')
 if n in ('EEVEE','FLAREON','JOLTEON'):
  for i in range(9):a=i*math.tau/9;p=(.20*math.cos(a),-.19+.15*math.sin(a),.48);leaf('Spiky neck ruff',p,(p[0]*1.50,p[1],p[2]-.13),.075,'white' if n=='JOLTEON' else 'cream')
 if n in ('VULPIX','NINETALES'):
  count=9 if n=='NINETALES' else 6
  for i in range(count):a=(i-(count-1)/2)*.27;tube('Individually curled fan tail',[(.06*math.sin(a),.25,.36),(.36*math.sin(a),.47,.45),(.54*math.sin(a),.55,.80),(.54*math.sin(a)+.045,.51,.92)],[.08,.10,.08,.035],c)
  if n=='VULPIX':
   for x in (-.095,0,.095):tube('Curled crown tuft',[(x,-.30,.78),(x,-.22,.87),(x,-.18,.84)],.019,'coral')
 elif n=='VAPOREON':
  tube('Long fish tail',[(0,.25,.32),(0,.48,.30),(.22,.64,.42),(.35,.58,.60)],[.10,.09,.06,.025],c)
  for s in (-1,1):leaf('Split tail fin',(.34,.58,.59),(.34+s*.16,.62,.81),.10,'blue');fin('Ear fin',[(s*.15,-.24,.68),(s*.43,-.18,.81),(s*.35,-.21,.54)],'cream',.024)
  for i in range(10):a=i*math.tau/10;leaf('Frilled collar',(0,-.21,.49),(.28*math.cos(a),-.21+.16*math.sin(a),.48),.07,'cream')
  for y in (.02,.16,.30):leaf('Dark dorsal fin',(0,y,.52),(0,y+.03,.68),.055,'navy')
 elif n=='JOLTEON':
  for i in range(9):a=i*math.tau/9;rod('Electric back quill',(.18*math.cos(a),.23,.40+.10*math.sin(a)),(.37*math.cos(a),.47,.46+.16*math.sin(a)),.075,'yellow',.002)
 elif n=='ESPEON':
  tube('Slender split tail',[(0,.24,.30),(0,.43,.44),(0,.49,.68)],.025,c)
  for s in (-1,1):tube('Tail fork',[(0,.49,.67),(s*.09,.51,.81),(s*.16,.48,.87)],.021,c)
  sph('Forehead ruby',(0,-.46,.78),(.035,.020,.047),'red')
 else:
  tube('Raised plush tail',[(0,.25,.31),(0,.46,.46),(.07,.56,.75)],[.075,.12,.045],c);sph('Tail tip',(.05,.54,.70),(.09,.075,.10),'cream' if n!='UMBREON' else 'yellow')
 if n=='UMBREON':
  ring('Forehead gold ring',(0,-.455,.76),.05,.017,'yellow')
  for s in (-1,1):ring('Shoulder gold ring',(s*.234,-.11,.40),.047,.012,'yellow',(0,math.pi/2,0))

@register('CHINCHOU LANTURN',.95)
def lantern_fish(n):
 pass # Pure rounded angler recipes are dispatched in run().
@register('TOGEPI TOGETIC',.94)
def egg_fairies(n):
 pass # Dedicated pure egg-fairy sculptures are dispatched in run().
@register('NATU XATU',1.04)
def mystic_birds(n):
 pass # Pure distinct psychic-bird recipes are dispatched in run().
@register('MARILL AZUMARILL',1.02)
def sculpted_water_rabbits(n):
 raise RuntimeError('The water rabbits use their dedicated papercraft recipe before Blender batching')
@register('SUNKERN SUNFLORA BELLOSSOM',.95)
def flowers(n):
 raise RuntimeError('The flower species use their dedicated pure sculpture recipes before Blender batching')
@register('AIPOM',1.02)
def aipom(n):
 pass # Pure playful monkey sculpture is dispatched in run().
@register('YANMA',.95)
def yanma(n):
 pass # Pure clear-wing dragonfly sculpture is dispatched in run().
@register('MISDREAVUS',1.02)
def misdreavus(n):
 pass # Pure flowing ghost sculpture is dispatched in run().
@register('WOBBUFFET',1.14)
def wobbuffet(n):
 pass # Pure elastic Wobbuffet sculpture is dispatched in run().
@register('TAUROS',1.28)
def bull(n):
 pass # Pure bovine recipe is dispatched in run().
@register('GIRAFARIG STANTLER',1.28)
def hoofed(n):
 pass # Pure psychic-hoofed recipes are dispatched in run().
@register('PINECO FORRETRESS',.93)
def armored_bugs(n):
 raise ValueError('Pineco and Forretress use their canonical coherent armor skins before Blender batching')
@register('DUNSPARCE',.85)
def dunsparce(n):
 pass # Pure land-snake recipe is dispatched in run().
@register('GLIGAR',1.06)
def gligar(n):
 pass # Pure flying scorpion sculpture is dispatched in run().
@register('SNUBBULL GRANBULL',1.06)
def bulldogs(n):
 pass # Pure distinct bulldog recipes are dispatched in run().
@register('QWILFISH',.91)
def qwilfish(n):
 pass # Pure inflated puffer sculpture is dispatched in run().
@register('SHUCKLE',.91)
def shuckle(n):
 pass # Pure perforated-shell sculpture is dispatched in run().
@register('HERACROSS PINSIR SCYTHER SCIZOR',1.16)
def battle_insects(n):
 raise ValueError('Heracross, Pinsir, Scizor and Scyther use their dedicated sculptures and skins')
@register('SNEASEL',1.11)
def sneasel(n):
 pass # Pure claw-weasel sculpture recipe is dispatched in run().
@register('TEDDIURSA URSARING',1.14)
def bears(n):
 pass # Pure distinct bear recipes are dispatched in run().
@register('SLUGMA MAGCARGO',.98)
def lava_snails(n):
 pass # Pure lava mollusk recipe is dispatched in run().
@register('SWINUB PILOSWINE',.98)
def shaggy_pigs(n):
 pass # Pure fur-pig recipe is dispatched in run().
@register('CORSOLA',.93)
def corsola(n):
 pass # Pure sea sculpture recipe is dispatched in run().
@register('MANTINE',1.02)
def mantine(n):
 pass # Pure sea sculpture recipe is dispatched in run().
@register('SKARMORY',1.21)
def skarmory(n):
 pass # Pure steel-bird sculpture recipe is dispatched in run().
@register('GROWLITHE ARCANINE',1.20)
def dogs(n):
 raise ValueError('Growlithe and Arcanine use original distinct striped-canine sculptures and skins')
@register('HOUNDOUR HOUNDOOM',1.20)
def hellhounds(n):
 pass # Pure canine armor recipe is dispatched in run().
@register('PHANPY DONPHAN',1.02)
def elephants(n):
 pass # Dedicated pure armored and baby elephant recipes are dispatched in run().
@register('SMEARGLE',1.14)
def smeargle(n):
 pass # Pure painter-dog recipe is dispatched in run().
@register('MILTANK',1.12)
def miltank(n):
 pass # Pure bovine recipe is dispatched in run().
@register('LARVITAR PUPITAR TYRANITAR',1.24)
def armored_dinosaurs(n):
 raise RuntimeError('The armored dinosaur family uses dedicated sculptures before Blender batching')
@register('CELEBI',.98)
def celebi(n):
 raise ValueError('Celebi uses its canonical swept-onion fairy sculpture and skin')

@register('EKANS ARBOK DRATINI DRAGONAIR STEELIX',1.18)
def serpents(n):
 raise ValueError('Serpents use their canonical species sculptures and skins')
@register('GLOOM VILEPLUME',.97)
def rafflesia(n):
 raise ValueError('Gloom and Vileplume use their canonical botanical sculptures and skins')
@register('PARAS PARASECT',.91)
def mushroom_crabs(n):
 raise ValueError('Paras and Parasect use their dedicated fungal cap sculptures and skins')
 large=n=='PARASECT';sph('Orange insect body',(0,0,.22),(.27,.23,.18),'orange');eyes(.28,-.211,.13,.05,'white' if large else 'ink')
 for s in (-1,1):
  for i in range(3):tube('Crab leg',[(s*.19,-.07+i*.13,.23),(s*.37,-.13+i*.16,.20),(s*.42,-.24+i*.18,.035)],[.035,.035,.018],'orange')
  rod('Claw arm',(s*.20,-.14,.25),(s*.32,-.28,.33),.05,'orange');sph('Claw',(s*.36,-.30,.37),(.10,.08,.11),'orange')
 caps=[(0,.04,.58,.44)] if large else [(-.15,.07,.44,.16),(.15,.11,.47,.17)]
 for x,y,z,r in caps:
  rod('Mushroom stalk',(x,y,.30),(x,y,z-.035),r*.18,'cream');sph('Umbrella mushroom cap',(x,y,z),(r,r*.80,r*.28),'red')
  for i in range(5):a=i*math.tau/5;sph('Cream cap spot',(x+r*.58*math.cos(a),y+r*.45*math.sin(a),z+r*.23),(r*.13,r*.12,r*.05),'cream')
@register('VENONAT VENOMOTH',.99)
def moths(n):
 if n=='VENOMOTH':raise ValueError('Venomoth uses its dedicated swept moth sculpture and skin')
 if n=='VENONAT':raise ValueError('Venonat uses its dedicated fuzzy insect sculpture and skin')
 adult=n=='VENOMOTH';sph('Furry insect abdomen',(0,0,.32),(.18,.16,.23) if adult else (.27,.22,.28),'lilac' if adult else 'purple');sph('Head',(0,-.025,.62),(.18,.14,.18) if adult else (.27,.20,.23),'lilac' if adult else 'purple')
 for s in (-1,1):
  sph('Large compound eye',(s*.15,-.18,.66),(.105,.07,.12),'red');sph('Eye shine',(s*.17,-.24,.70),(.027,.011,.03),'pink');tube('White antenna',[(s*.08,0,.80),(s*.15,.02,1.00),(s*.19,.02,1.04)],.016,'white');sph('Tiny foot',(s*.12,-.03,.07),(.075,.095,.05),'pink')
  if adult:
   for z,dy in ((.62,0),(.42,.03)):
    pts=[(s*.10,dy,z),(s*.48,dy,z+.31),(s*.67,dy,z+.21),(s*.52,dy,z-.12),(s*.18,dy,z-.17)];fin('Broad moth wing',pts,'lilac',.024)
    for i in range(3):tube('Wing patterned rib',[(s*.18,dy-.03,z),(s*.35,dy-.031,z+.06*i),(s*.54,dy-.03,z+.06+i*.06)],.012,'purple')
  else:
   for i in range(8):a=i*math.tau/8;leaf('Fuzzy body spike',(.23*math.cos(a),.10*math.sin(a),.48),(.34*math.cos(a),.15*math.sin(a),.60),.04,'purple')
 rod('Small mandible',(0,-.18,.55),(0,-.27,.47),.031,'cream',.003)
@register('ENTEI',1.42)
def entei(n):
 raise ValueError('Entei uses the canonical armored volcano-beast skin')
@register('RAIKOU',1.30)
def raikou(n):
 raise ValueError('Raikou uses the canonical armored thunder-beast skin')
@register('SUICUNE',1.60)
def suicune(n):
 raise ValueError('Suicune uses the canonical open-crested aurora-beast skin')
@register('UNOWN',1.0)
def unown(n):
 raise ValueError('Unown uses the canonical A..Z standard glyph skins')
@register('SLOWPOKE',1.0)
def slowpoke(n):
 raise ValueError('Slowpoke uses its canonical sleepy quadruped skin')
@register('DIGLETT',.82)
def diglett(n):
 raise ValueError('Diglett uses its canonical capsule and fixed-soil skin')
@register('DUGTRIO',.98)
def dugtrio(n):
 raise ValueError('Dugtrio uses its canonical three-capsule fixed-soil skin')
@register('MEOWTH PERSIAN',1.01)
def cats(n):
 raise ValueError('Meowth and Persian use their canonical fitted-face and articulated feline skins')
@register('PSYDUCK GOLDUCK',1.03)
def sculpted_ducks(n):
 raise RuntimeError('The duck family is exported by its dedicated sculpture recipe before Blender batching')
@register('MANKEY PRIMEAPE',1.02)
def pig_monkeys(n):
 raise ValueError('Mankey and Primeape use their canonical rooted fur and articulated fist skins')
@register('POLIWHIRL POLIWRATH POLITOED',1.07)
def frogs(n):
 raise ValueError('The three spiral frogs use their original pure sculptures and canonical skins')
@register('TENTACOOL TENTACRUEL',1.13)
def jellyfish(n):
 raise ValueError('Tentacool and Tentacruel use their canonical flared bells and grounded tentacle skins')
@register('GRAVELER GOLEM RHYHORN RHYDON',1.11)
def rocks(n):
 raise ValueError('Stone bipeds and armored rhinoceroses use their original pure sculpture recipes')
@register('PONYTA RAPIDASH',1.22)
def fire_horses(n):
 raise ValueError('Ponyta and Rapidash use their original pure fire-horse sculpture recipes')
@register('SLOWBRO SLOWKING',1.15)
def slow_evolutions(n):
 raise ValueError('Slowbro and Slowking use their original pure shell-tailed and royal sculpture recipes')
@register('MAGNEMITE MAGNETON',.96)
def magnets(n):
 pass # Pure mechanical sculpture dispatch in run().
@register('DODUO DODRIO',1.22)
def multi_birds(n):
 pass # Pure feathered runner-bird sculpture dispatch in run().
@register('SEEL DEWGONG',1.03)
def seals(n):
 raise ValueError('Seel and Dewgong use their dedicated curved seal sculptures and articulated fan skins')
@register('GRIMER MUK DITTO',.96)
def slimes(n):
 raise ValueError('Grimer, Muk and Ditto use their dedicated continuous slime sculptures and soft appendage skins')
@register('SHELLDER CLOYSTER',1.05)
def bivalves(n):
 raise ValueError('Shellder and Cloyster use their dedicated hollow bivalve sculptures and hinged-valve skins')
@register('GASTLY HAUNTER GENGAR',1.05)
def ghosts(n):
 if n=='GENGAR':sculpt_gengar();return
 raise ValueError('Gastly and Haunter use their dedicated ghost sculptures and skins')
@register('KRABBY KINGLER',.97)
def crabs(n):
 raise ValueError('Krabby and Kingler use their dedicated planted crab sculptures and closing-jaw skins')
@register('ONIX',1.72)
def onix(n):
 raise ValueError('Onix shares the canonical rock-chain skin with its world actor')
@register('VOLTORB',.82)
def voltorb(n):
 raise ValueError('Voltorb shares its canonical orb skin with the world icon')
@register('ELECTRODE',.94)
def electrode(n):
 raise ValueError('Electrode uses its canonical coherent shell skin before Blender')
@register('EXEGGCUTE EXEGGUTOR',1.22)
def eggs_palms(n):
 raise ValueError('Exeggcute and Exeggutor use psychic_palm_glb before Blender batching')
@register('CUBONE MAROWAK',1.06)
def skull_dinosaurs(n):
 raise ValueError('Cubone and Marowak use their original separate skull-and-bone sculpture recipes')
@register('LICKITUNG',1.06)
def lickitung(n):
 raise ValueError('Lickitung uses its original pure stocky tongue-bearer sculpture recipe')
@register('KOFFING WEEZING',1.05)
def gas_balls(n):
 raise ValueError('Koffing and Weezing use their original pure cratered gas-body recipes')
@register('CHANSEY BLISSEY',1.12)
def egg_nurses(n):
 raise ValueError('Chansey and Blissey use their original separate egg-nurse sculpture recipes')
@register('TANGELA',1.00)
def tangela(n):
 raise ValueError('Tangela uses its original pure rounded vine-tangle sculpture recipe')
@register('KANGASKHAN',1.27)
def kangaskhan(n):
 raise ValueError('Kangaskhan uses its original pure armored parent and open-pouch baby sculpture recipe')

@register('ABRA ALAKAZAM',1.12)
def sculpted_fox_psychics(n):
 raise RuntimeError('Abra and Alakazam use their dedicated sculpture recipe before Blender batching')
@register('DROWZEE HYPNO',1.12)
def sculpted_tapirs(n):
 raise RuntimeError('Drowzee and Hypno use their dedicated sculpture recipe before Blender batching')
@register('KADABRA',1.12)
def sculpted_kadabra(n):
 sculpt_kadabra()
@register('TYROGUE HITMONLEE HITMONCHAN HITMONTOP',1.18)
def martial_artists(n):
 raise ValueError('All four martial fighters use their original pure sculptures')
@register('HORSEA SEADRA KINGDRA',1.07)
def seahorses(n):
 raise RuntimeError('Seahorses use their pure sculpture recipes')
@register('GOLDEEN SEAKING MAGIKARP',1.00)
def fishes(n):
 raise ValueError('Goldeen, Seaking and Magikarp use goldfish_sculpt')
@register('REMORAID OCTILLERY',1.00)
def jet_fish(n):
 pass # Pure jet-fish sculpture recipe is dispatched in run().
@register('STARYU',1.0)
def staryu(n):
 raise ValueError('Staryu uses the canonical pure radial starfish skin')
@register('STARMIE',1.03)
def starmie(n):
 raise ValueError('Starmie uses its dedicated dual-star sculpture and skin')
@register('MR__MIME JYNX SMOOCHUM ELECTABUZZ ELEKID MAGMAR MAGBY',1.17)
def expressive_humanoids(n):
 raise ValueError('Mr Mime, the ice dancers, electric pair and fire pair use their original pure sculptures')
@register('PORYGON PORYGON2',1.05)
def virtual_ducks(n):
 pass # Dedicated angular and rounded virtual-duck recipes are dispatched in run().
@register('OMANYTE OMASTAR KABUTO KABUTOPS',1.08)
def fossils(n):
 raise ValueError('Fossils use their dedicated nautilus and scythe sculptures/skins before Blender batching')
@register('AERODACTYL DRAGONITE',1.30)
def wing_dragons(n):
 raise ValueError('Aerodactyl and Dragonite use their original pure sculpture recipes')
@register('ARTICUNO ZAPDOS MOLTRES',1.28)
def legendary_birds(n):
 raise ValueError('Articuno, Zapdos and Moltres use their original pure sculpture recipes')
@register('HO_OH',1.45)
def rainbow_phoenix(n):
 raise ValueError('Ho-Oh uses its original pure sculpture recipe')
@register('LUGIA',1.45)
def sea_guardian(n):
 raise ValueError('Lugia uses its original pure sculpture recipe')
@register('LAPRAS',1.50)
def swimming_transport(n):
 raise ValueError('Lapras uses its canonical articulated sculpture recipe')
@register('MEW MEWTWO',1.26)
def psychic_cats(n):
 raise ValueError('Mew and Mewtwo use their canonical psychic-cat sculptures and skins')

@register('MACHOP MACHOKE MACHAMP', 1.18)
def sculpted_fighting_family(n):
 raise RuntimeError('The fighting family is exported by its dedicated sculpture recipe before Blender batching')

@register('AMPHAROS', 1.35)
def sculpted_ampharos(n):
 raise RuntimeError('Ampharos is exported by its connected sculpture recipe before Blender batching')

# Spearow has its own compact canonical GLB recipe. It is emitted before
# Blender batching and never recreated as a generic bird or duplicate .blend.
@register('SPEAROW', .75)
def papercraft_spearow(n):
 raise RuntimeError('Spearow is exported by its canonical GLB recipe before Blender batching')

# All 220 complementary exact species have explicit authored builders.
HEIGHTS.update({
 'mew':.78,'mewtwo':1.45,'dragonite':1.40,'aerodactyl':1.30,
 'quilava':.96,'typhlosion':1.38,'croconaw':1.06,'feraligatr':1.48,
 'bayleef':1.15,'meganium':1.48,'furret':1.20,'noctowl':1.20,
 'caterpie':.62,'weedle':.58,'kakuna':.80,'metapod':.80,
 'mareep':.85,'flaaffy':1.0,'ampharos':1.35,'wooper':.70,'quagsire':1.13,
 'pichu':.62,'cleffa':.60,'igglybuff':.58,'togepi':.67,'tyrogue':.80,
 'smoochum':.66,'magby':.70,'elekid':.70,'vulpix':.85,'ninetales':1.30,
 'eevee':.80,'espeon':1.0,'umbreon':1.05,'flareon':.97,'jolteon':1.05,'vaporeon':1.0,
 'nidoran_f':.72,'nidoran_m':.72,'nidorina':1.0,'nidorino':1.0,'nidoqueen':1.40,'nidoking':1.40,
 'larvitar':.85,'pupitar':1.1,'tyranitar':1.55,'magikarp':.90,'goldeen':.80,'seaking':1.0,
 'chansey':1.05,'blissey':1.25,'gloom':.85,'vileplume':1.12,
 'machop':.85,'machoke':1.15,'machamp':1.45,'murkrow':.78,'delibird':.95,
 'pidgeotto':1.0,'pidgeot':1.35,'fearow':1.23,'spearow':.75,
 'magnemite':.65,'magneton':1.15,'houndour':.90,'houndoom':1.20,'growlithe':.90,'arcanine':1.30,
 'omanyte':.72,'omastar':1.02,'kabuto':.60,'kabutops':1.20,
})

def converted(v):return [round(v.x,6),round(v.z,6),round(-v.y,6)]
def export(name,objects):
 bpy.context.view_layer.update();coords=[o.matrix_world@v.co for o in objects for v in o.data.vertices];lo=Vector([min(p[a] for p in coords) for a in range(3)]);hi=Vector([max(p[a] for p in coords) for a in range(3)]);scale=min(HEIGHTS[name]/(hi.z-lo.z),2.25/(hi.x-lo.x),2.25/(hi.y-lo.y));center=Vector(((lo.x+hi.x)/2,(lo.y+hi.y)/2,lo.z))
 for o in objects:o.location=(o.location-center)*scale;o.scale*=scale
 bpy.context.view_layer.update()
 if name in ('gengar','kadabra','raticate'):sculpt_runtime_lod(name,objects)
 bpy.context.view_layer.update();prims=[]
 for o in objects:
  me=o.data;me.calc_loop_triangles();pos=[];nor=[];ind=[];lookup={};normal_matrix=o.matrix_world.to_3x3().inverted().transposed()
  for t in me.loop_triangles:
   ps=[o.matrix_world@me.vertices[me.loops[li].vertex_index].co for li in t.loops]
   if (ps[1]-ps[0]).cross(ps[2]-ps[0]).length_squared<1e-18:continue
   for li in t.loops:
    p=converted(o.matrix_world@me.vertices[me.loops[li].vertex_index].co);n=converted((normal_matrix@me.corner_normals[li].vector).normalized());k=tuple(p+n)
    if k not in lookup:lookup[k]=len(pos)//3;pos.extend(p);nor.extend(n)
    ind.append(lookup[k])
  if ind:prims.append({'part':o.name,'positions':pos,'normals':nor,'indices':ind,'base_color':[round(v,6) for v in me.materials[0].diffuse_color]})
 if name in ('gengar','kadabra','raticate'):
  for p in prims:
   values=[tuple(p['positions'][i:i+3]+p['normals'][i:i+3]) for i in range(0,len(p['positions']),3)];unique=sorted(set(values));lookup={v:i for i,v in enumerate(unique)};remap=[lookup[v] for v in values];tri=[]
   for i in range(0,len(p['indices']),3):
    t=tuple(remap[j] for j in p['indices'][i:i+3]);tri.append(min(t,t[1:]+t[:1],t[2:]+t[:2]))
   p['positions']=[v for row in unique for v in row[:3]];p['normals']=[v for row in unique for v in row[3:]];p['indices']=[i for t in sorted(tri) for i in t]
 data={'name':name,'version':1,'coordinate_system':'+Y up; front +Z; floor-centered root','primitives':prims}
 if name=='kadabra':
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from kadabra_glb import fit_tail_band
  data=fit_tail_band(data)
  prims=data['primitives']
 if name=='gengar':
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from gengar_glb import export_gengar
  if (OUT/'gengar.mesh.json').exists():raise ValueError('remove the reviewed obsolete Gengar JSON before exporting its canonical GLB')
  (OUT/'gengar.glb').write_bytes(export_gengar(data))
 else:(OUT/(name+'.mesh.json')).write_text(json.dumps(data,separators=(',',':')))
 print(f'{name}: {len(prims)} named parts, {sum(len(p["indices"])//3 for p in prims)} triangles',flush=True)
def reset():
 for o in list(bpy.data.objects):bpy.data.objects.remove(o,do_unlink=True)
 for c in list(bpy.data.collections):bpy.data.collections.remove(c)
 for me in list(bpy.data.meshes):
  if not me.users:bpy.data.meshes.remove(me)
def run():
 global COL
 names=list(BUILDERS)
 if args.only:names=[n for n in names if n in args.only.lower().split(',')]
 for species in ('larvitar','pupitar'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from rock_pupa_glb import export_rock_pupa
  (OUT/(species+'.glb')).write_bytes(export_rock_pupa(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': original horned larva or segmented pupa skin',flush=True)
 if 'tyranitar' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from tyranitar_glb import export_tyranitar
  (OUT/'tyranitar.glb').write_bytes(export_tyranitar())
  (OUT/'tyranitar.mesh.json').unlink(missing_ok=True)
  names.remove('tyranitar')
  print('tyranitar: original armored skull, rooted spines and heavy tail skin',flush=True)
 for species in ('quilava','typhlosion'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from fire_badger_glb import export_fire_badger
  (OUT/(species+'.glb')).write_bytes(export_fire_badger(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': original shared-coat fire badger anatomy and skin',flush=True)
 for species in ('charmeleon','charizard'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from fire_dragon_glb import export_fire_dragon
  (OUT/(species+'.glb')).write_bytes(export_fire_dragon(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': original rooted fire dragon anatomy and skin',flush=True)
 for species in ('omanyte','omastar'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from nautilus_glb import export_nautilus
  (OUT/(species+'.glb')).write_bytes(export_nautilus(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': original ribbed nautilus shell and ten rooted tentacles',flush=True)
 if 'farfetch_d' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from leek_duck_glb import export_leek_duck
  (OUT/'farfetch_d.glb').write_bytes(export_leek_duck('farfetch_d'))
  (OUT/'farfetch_d.mesh.json').unlink(missing_ok=True)
  names.remove('farfetch_d')
  print('farfetch_d: original stern gray-brown duck with wing-held leek and webbed feet',flush=True)
 for species in ('doduo','dodrio'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from dodo_glb import export_dodo
  (OUT/(species+'.glb')).write_bytes(export_dodo(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': canonical feathered runner with independent curved necks, coherent heads, hinged bills and planted claws',flush=True)
 for species in ('magnemite','magneton'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from magnet_glb import export_magnet
  (OUT/(species+'.glb')).write_bytes(export_magnet(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': canonical articulated steel orbs, hinged open magnets and independently rotating screws',flush=True)
 for species in ('ivysaur','venusaur'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from bulb_saur_glb import export_bulb_saur
  (OUT/(species+'.glb')).write_bytes(export_bulb_saur(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': original articulated growing or mature seed saur, complete quadruped and rooted botanical crown',flush=True)
 for species in ('exeggcute','exeggutor'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from psychic_palm_glb import export_psychic_palm
  (OUT/(species+'.glb')).write_bytes(export_psychic_palm(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': canonical grounded psychic eggs or independently bending coconut palm',flush=True)
 for species in ('kabuto','kabutops'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from fossil_scythe_glb import export_fossil_scythe
  (OUT/(species+'.glb')).write_bytes(export_fossil_scythe(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': canonical grounded shell, coherent gaze and articulated claws/scythes',flush=True)
 for species in ('pineco','forretress'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from armored_bug_glb import export_armored_bug
  (OUT/(species+'.glb')).write_bytes(export_armored_bug(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': canonical coherent armor with grounded core/shell and articulated whorls/ports',flush=True)
 if 'electrode' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from voltorb_glb import export_electrode
  (OUT/'electrode.glb').write_bytes(export_electrode())
  (OUT/'electrode.mesh.json').unlink(missing_ok=True)
  names.remove('electrode')
  print('electrode: canonical grounded explosive orb skin with coherent fitted expression',flush=True)
 for species in ('krabby','kingler'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from crab_glb import export_crab
  (OUT/(species+'.glb')).write_bytes(export_crab(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': canonical planted crab with independent arms, wrists and closing pincer jaws',flush=True)
 for species in ('shellder','cloyster'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from bivalve_glb import export_bivalve
  (OUT/(species+'.glb')).write_bytes(export_bivalve(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': canonical hollow bivalve with hinged valves, coherent gaze and supported tongue folds',flush=True)
 for species in ('grimer','muk','ditto'):
  if species not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from slime_glb import export_slime
  (OUT/(species+'.glb')).write_bytes(export_slime(species))
  (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
  names.remove(species)
  print(species+': canonical continuous soft body with planted puddle and independent appendages',flush=True)
 if 'dewgong' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from seal_glb import export_dewgong
  (OUT/'dewgong.glb').write_bytes(export_dewgong())
  (OUT/'dewgong.mesh.json').unlink(missing_ok=True)
  names.remove('dewgong')
  print('dewgong: canonical curved seal with independent flippers, two tail carriers and paired fans',flush=True)
 if 'seel' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from seal_glb import export_seel
  (OUT/'seel.glb').write_bytes(export_seel())
  (OUT/'seel.mesh.json').unlink(missing_ok=True)
  names.remove('seel')
  print('seel: canonical reclining seal with level paddles, articulated tongue and paired tail fans',flush=True)
 for fairy in ('clefable','wigglytuff'):
  if fairy in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from fairy_pair_glb import export_fairy_pair
   (OUT/(fairy+'.glb')).write_bytes(export_fairy_pair(fairy))
   (OUT/(fairy+'.mesh.json')).unlink(missing_ok=True)
   names.remove(fairy)
   print(fairy+': original articulated fairy with supported fitted surfaces and complete feet',flush=True)
 for baby in ('pichu','cleffa','igglybuff'):
  if baby in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from baby_glb import export_baby
   (OUT/(baby+'.glb')).write_bytes(export_baby(baby))
   (OUT/(baby+'.mesh.json')).unlink(missing_ok=True)
   names.remove(baby)
   print(baby+': canonical baby skin with fitted face and planted feet',flush=True)
 if 'corsola' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from corsola_glb import export_corsola
  (OUT/'corsola.glb').write_bytes(export_corsola())
  (OUT/'corsola.mesh.json').unlink(missing_ok=True)
  names.remove('corsola')
  print('corsola: canonical coherent coral forks, arms and four planted pads',flush=True)
 if 'mantine' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from mantine_glb import export_mantine
  (OUT/'mantine.glb').write_bytes(export_mantine())
  (OUT/'mantine.mesh.json').unlink(missing_ok=True)
  names.remove('mantine')
  print('mantine: canonical shared wing seams and attached articulated Remora',flush=True)
 if 'porygon' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from porygon_glb import export_porygon
  (OUT/'porygon.glb').write_bytes(export_porygon())
  (OUT/'porygon.mesh.json').unlink(missing_ok=True)
  names.remove('porygon')
  print('porygon: canonical rigid angular panels, scanning head and grounded paddle swivels',flush=True)
 if 'porygon2' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from porygon2_glb import export_porygon2
  (OUT/'porygon2.glb').write_bytes(export_porygon2())
  (OUT/'porygon2.mesh.json').unlink(missing_ok=True)
  names.remove('porygon2')
  print('porygon2: canonical curious gaze, flexible rear tail and grounded cyan paddles',flush=True)
 for small_nido in ('nidoran_f','nidoran_m'):
  if small_nido in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from nidoran_glb import export_nidoran
   (OUT/(small_nido+'.glb')).write_bytes(export_nidoran(small_nido))
   (OUT/(small_nido+'.mesh.json')).unlink(missing_ok=True)
   names.remove(small_nido)
   print(small_nido+': original distinct poison-pin skin, fitted gaze, articulated ears and four supported paws',flush=True)
 if 'nidorino' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from nido_middle_glb import export_nidorino
  (OUT/'nidorino.glb').write_bytes(export_nidorino())
  (OUT/'nidorino.mesh.json').unlink(missing_ok=True)
  names.remove('nidorino')
  print('nidorino: original horned quadruped skin, articulated ears/jaw/horn and four supported paws',flush=True)
 if 'steelix' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from steelix_glb import export_steelix
  (OUT/'steelix.glb').write_bytes(export_steelix())
  (OUT/'steelix.mesh.json').unlink(missing_ok=True)
  names.remove('steelix')
  print('steelix: canonical original steel chain skin, coherent armor and independently hinged jaw',flush=True)
 if 'magmar' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from magmar_glb import export_magmar
  (OUT/'magmar.glb').write_bytes(export_magmar())
  (OUT/'magmar.mesh.json').unlink(missing_ok=True)
  names.remove('magmar')
  print('magmar: canonical articulated twin fire crown, open bill, flame belly and supported claws',flush=True)
 if 'magby' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from magby_glb import export_magby
  (OUT/'magby.glb').write_bytes(export_magby())
  (OUT/'magby.mesh.json').unlink(missing_ok=True)
  names.remove('magby')
  print('magby: canonical planted live-coal skin with coherent crown, open bill and clawed arms',flush=True)
 if 'elekid' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from elekid_glb import export_elekid
  (OUT/'elekid.glb').write_bytes(export_elekid())
  (OUT/'elekid.mesh.json').unlink(missing_ok=True)
  names.remove('elekid')
  print('elekid: canonical planted electric egg with open plug blades and coherent striped arms',flush=True)
 for electric_species in ('electabuzz',):
  if electric_species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from electabuzz_glb import export_electabuzz
   (OUT/(electric_species+'.glb')).write_bytes(export_electabuzz())
   (OUT/(electric_species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(electric_species)
   print(electric_species+': canonical asymmetric electric anatomy with coherent stripe wraps and articulated antennae/tail',flush=True)
 if 'smoochum' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from smoochum_glb import export_smoochum
  (OUT/'smoochum.glb').write_bytes(export_smoochum())
  (OUT/'smoochum.mesh.json').unlink(missing_ok=True)
  names.remove('smoochum')
  print('smoochum: canonical planted kiss creature with fitted gaze, recessed lips and coherent bob',flush=True)
 for species in ('meowth','persian'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from feline_glb import export_feline
   (OUT/(species+'.glb')).write_bytes(export_feline(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical feline anatomy, coherent fitted face/whiskers/charm, articulated curled tail and planted paws',flush=True)
 for species in ('mankey','primeape'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from pig_monkey_glb import export_pig_monkey
   (OUT/(species+'.glb')).write_bytes(export_pig_monkey(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical rooted fur, angry gaze, clenched fists and supporting paws',flush=True)
 if 'jynx' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from jynx_glb import export_jynx
  (OUT/'jynx.glb').write_bytes(export_jynx())
  (OUT/'jynx.mesh.json').unlink(missing_ok=True)
  names.remove('jynx')
  print('jynx: canonical grounded gown, fitted gaze, articulated hair and independent five-digit hands',flush=True)
 if 'tyrogue' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from tyrogue_glb import export_tyrogue
  (OUT/'tyrogue.glb').write_bytes(export_tyrogue())
  (OUT/'tyrogue.mesh.json').unlink(missing_ok=True)
  names.remove('tyrogue')
  print('tyrogue: canonical planted scuffle fighter with continuous crown and articulated wrapped wrists',flush=True)
 if 'hitmontop' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from hitmontop_glb import export_hitmontop
  (OUT/'hitmontop.glb').write_bytes(export_hitmontop())
  (OUT/'hitmontop.mesh.json').unlink(missing_ok=True)
  names.remove('hitmontop')
  print('hitmontop: original handstand fighter with blue spiked feet and ball tail',flush=True)
 if 'hitmonchan' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from hitmonchan_glb import export_hitmonchan
  (OUT/'hitmonchan.glb').write_bytes(export_hitmonchan())
  (OUT/'hitmonchan.mesh.json').unlink(missing_ok=True)
  names.remove('hitmonchan')
  print('hitmonchan: original rounded crown skull, open pleated kit and planted boots in the local pack palette',flush=True)
 for species in ('remoraid','octillery'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from jet_fish_glb import export_jet_fish
   (OUT/(species+'.glb')).write_bytes(export_jet_fish(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical articulated fins or grounded tentacle curls and hollow siphon',flush=True)
 for species in ('swinub','piloswine'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from fur_pig_glb import export_fur_pig
   (OUT/(species+'.glb')).write_bytes(export_fur_pig(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical planted fur pig with coherent coat and sniffing head',flush=True)
 if 'sneasel' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from sneasel_glb import export_sneasel
  (OUT/'sneasel.glb').write_bytes(export_sneasel())
  (OUT/'sneasel.mesh.json').unlink(missing_ok=True)
  names.remove('sneasel')
  print('sneasel: canonical planted toe talons and coherent feathered claw strike',flush=True)
 if 'skarmory' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from skarmory_glb import export_skarmory
  (OUT/'skarmory.glb').write_bytes(export_skarmory())
  (OUT/'skarmory.mesh.json').unlink(missing_ok=True)
  names.remove('skarmory')
  print('skarmory: canonical articulated metal wings and opening lower bill with planted talons',flush=True)
 if 'smeargle' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from smeargle_glb import export_smeargle
  (OUT/'smeargle.glb').write_bytes(export_smeargle())
  (OUT/'smeargle.mesh.json').unlink(missing_ok=True)
  names.remove('smeargle')
  print('smeargle: canonical held brush-tail stroke with continuous banded arms and planted paws',flush=True)
 if 'hitmonlee' in names:





  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from hitmonlee_glb import export_hitmonlee
  (OUT/'hitmonlee.glb').write_bytes(export_hitmonlee())
  (OUT/'hitmonlee.mesh.json').unlink(missing_ok=True)
  names.remove('hitmonlee')
  print('hitmonlee: original headless fighter with continuous ribbed springs and three-finger hands',flush=True)
 for sand_mammal in ('sandshrew','sandslash'):
  if sand_mammal in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from sand_mammal_glb import export_sand_mammal
   (OUT/(sand_mammal+'.glb')).write_bytes(export_sand_mammal(sand_mammal))
   (OUT/(sand_mammal+'.mesh.json')).unlink(missing_ok=True)
   names.remove(sand_mammal)
   print(sand_mammal+': original armored digging mammal sculpture and anatomical skin',flush=True)
 for spiral_frog in ('poliwhirl','poliwrath','politoed'):
  if spiral_frog in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from poli_frog_glb import export_poli_frog
   (OUT/(spiral_frog+'.glb')).write_bytes(export_poli_frog(spiral_frog))
   (OUT/(spiral_frog+'.mesh.json')).unlink(missing_ok=True)
   names.remove(spiral_frog)
   print(spiral_frog+': original distinct spiral frog sculpture and anatomical skin',flush=True)
 if 'nidoking' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from nido_royal_glb import export_nidoking
  (OUT/'nidoking.glb').write_bytes(export_nidoking())
  (OUT/'nidoking.mesh.json').unlink(missing_ok=True)
  names.remove('nidoking')
  print('nidoking: original armored biped with grounded feet and articulated royal anatomy',flush=True)
 if 'nidoqueen' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from nido_royal_glb import export_nidoqueen
  (OUT/'nidoqueen.glb').write_bytes(export_nidoqueen())
  (OUT/'nidoqueen.mesh.json').unlink(missing_ok=True)
  names.remove('nidoqueen')
  print('nidoqueen: original armored biped with grounded feet and articulated royal anatomy',flush=True)
 if 'nidorina' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from nido_middle_glb import export_nidorina
  (OUT/'nidorina.glb').write_bytes(export_nidorina())
  (OUT/'nidorina.mesh.json').unlink(missing_ok=True)
  names.remove('nidorina')
  print('nidorina: original hornless biped skin, articulated forearms/wrists and two supported paws',flush=True)
 for species in ('wartortle','blastoise'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from turtle_glb import export_turtle
   (OUT/(species+'.glb')).write_bytes(export_turtle(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical turtle shell, fitted details and articulated feather ears or hollow cannons',flush=True)
 if 'vaporeon' in names:
  from vaporeon_glb import export_vaporeon
  (OUT/'vaporeon.glb').write_bytes(export_vaporeon())
  (OUT/'vaporeon.mesh.json').unlink(missing_ok=True)
  names.remove('vaporeon')
  print('vaporeon: canonical aquatic cat, pleated frill, fan fins and curling split-fin tail',flush=True)
 for fungal_crab in ('paras','parasect'):
  if fungal_crab in names:
   from mushroom_crab_glb import export_mushroom_crab
   (OUT/(fungal_crab+'.glb')).write_bytes(export_mushroom_crab(fungal_crab))
   (OUT/(fungal_crab+'.mesh.json')).unlink(missing_ok=True)
   names.remove(fungal_crab)
   print(fungal_crab+': canonical spotted fungal caps, four planted walking legs and tapered foreclaws',flush=True)
 if 'venomoth' in names:
  from venomoth_glb import export_venomoth
  (OUT/'venomoth.glb').write_bytes(export_venomoth())
  (OUT/'venomoth.mesh.json').unlink(missing_ok=True)
  names.remove('venomoth')
  print('venomoth: canonical swept moth wings, coherent veins, pointed antennae and striped abdomen',flush=True)
 if 'venonat' in names:
  from venonat_glb import export_venonat
  (OUT/'venonat.glb').write_bytes(export_venonat())
  (OUT/'venonat.mesh.json').unlink(missing_ok=True)
  names.remove('venonat')
  print('venonat: canonical fuzzy insect, compound lenses, ivory feelers and planted broad feet',flush=True)
 if 'ninetales' in names:
  from ninetales_glb import export_ninetales
  (OUT/'ninetales.glb').write_bytes(export_ninetales())
  (OUT/'ninetales.mesh.json').unlink(missing_ok=True)
  names.remove('ninetales')
  print('ninetales: canonical elegant fox, cream mane and nine independently flowing golden-tipped plumes',flush=True)
 if 'vulpix' in names:
  from vulpix_glb import export_vulpix
  (OUT/'vulpix.glb').write_bytes(export_vulpix())
  (OUT/'vulpix.mesh.json').unlink(missing_ok=True)
  names.remove('vulpix')
  print('vulpix: canonical fox, curled forelock, planted paws and six independent orange plumes',flush=True)
 if 'umbreon' in names:
  from umbreon_glb import export_umbreon
  (OUT/'umbreon.glb').write_bytes(export_umbreon())
  (OUT/'umbreon.mesh.json').unlink(missing_ok=True)
  names.remove('umbreon')
  print('umbreon: canonical moonlight companion, fitted hollow gold rings and seamless banded ears/tail',flush=True)
 if 'espeon' in names:
  from espeon_glb import export_espeon
  (OUT/'espeon.glb').write_bytes(export_espeon())
  (OUT/'espeon.mesh.json').unlink(missing_ok=True)
  names.remove('espeon')
  print('espeon: canonical sun cat, wide ears, ruby gem and independent forked tail tips',flush=True)
 if 'flareon' in names:
  from flareon_glb import export_flareon
  (OUT/'flareon.glb').write_bytes(export_flareon())
  (OUT/'flareon.mesh.json').unlink(missing_ok=True)
  names.remove('flareon')
  print('flareon: canonical flame fox, full cream collar, forehead tuft and broad curled plume',flush=True)
 if 'jolteon' in names:
  from jolteon_glb import export_jolteon
  (OUT/'jolteon.glb').write_bytes(export_jolteon())
  (OUT/'jolteon.mesh.json').unlink(missing_ok=True)
  names.remove('jolteon')
  print('jolteon: canonical lightning fox, sharp ears, white needle collar and planted paws',flush=True)
 if 'eevee' in names:
  from eevee_glb import export_eevee
  (OUT/'eevee.glb').write_bytes(export_eevee())
  (OUT/'eevee.mesh.json').unlink(missing_ok=True)
  names.remove('eevee')
  print('eevee: canonical evolution fox, fluffy cream ruff, four planted paws and flexible brush',flush=True)
 if 'haunter' in names:
  from haunter_glb import export_haunter
  (OUT/'haunter.glb').write_bytes(export_haunter())
  (OUT/'haunter.mesh.json').unlink(missing_ok=True)
  names.remove('haunter')
  print('haunter: canonical spiked ghost head and detached six-claw spectral hands',flush=True)
 if 'gastly' in names:
  from gastly_glb import export_gastly
  (OUT/'gastly.glb').write_bytes(export_gastly())
  (OUT/'gastly.mesh.json').unlink(missing_ok=True)
  names.remove('gastly')
  print('gastly: canonical dark orb, broad white gaze, two fangs and irregular vapor',flush=True)
 if 'starmie' in names:
  from starmie_glb import export_starmie
  (OUT/'starmie.glb').write_bytes(export_starmie())
  (OUT/'starmie.mesh.json').unlink(missing_ok=True)
  names.remove('starmie')
  print('starmie: canonical dual purple stars and eight-spoke rose core',flush=True)
 if 'scyther' in names:
  from scyther_glb import export_scyther
  (OUT/'scyther.glb').write_bytes(export_scyther())
  (OUT/'scyther.mesh.json').unlink(missing_ok=True)
  names.remove('scyther')
  print('scyther: canonical green mantis with curved scythes and fitted wing membranes',flush=True)
 if 'scizor' in names:
  from scizor_glb import export_scizor
  (OUT/'scizor.glb').write_bytes(export_scizor())
  (OUT/'scizor.mesh.json').unlink(missing_ok=True)
  names.remove('scizor')
  print('scizor: canonical red steel mantis with hinged pincers and four veined wing leaves',flush=True)
 if 'pinsir' in names:
  from pinsir_glb import export_pinsir
  (OUT/'pinsir.glb').write_bytes(export_pinsir())
  (OUT/'pinsir.mesh.json').unlink(missing_ok=True)
  names.remove('pinsir')
  print('pinsir: canonical stag beetle with spiked pincers, barred mouth and planted claws',flush=True)
 if 'heracross' in names:
  from heracross_glb import export_heracross
  (OUT/'heracross.glb').write_bytes(export_heracross())
  (OUT/'heracross.mesh.json').unlink(missing_ok=True)
  names.remove('heracross')
  print('heracross: canonical single-horn beetle with armored forearms and planted claws',flush=True)
 if 'mewtwo' in names:
  from mewtwo_glb import export_mewtwo
  (OUT/'mewtwo.glb').write_bytes(export_mewtwo())
  (OUT/'mewtwo.mesh.json').unlink(missing_ok=True)
  names.remove('mewtwo')
  print('mewtwo: canonical genetic psychic cat with planted pads, fitted gaze and heavy tail',flush=True)
 if 'mew' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from mew_glb import export_mew
  (OUT/'mew.glb').write_bytes(export_mew())
  (OUT/'mew.mesh.json').unlink(missing_ok=True)
  names.remove('mew')
  print('mew: canonical soft psychic cat with long paws, fitted gaze and looping tail',flush=True)
 if 'celebi' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from celebi_glb import export_celebi
  (OUT/'celebi.glb').write_bytes(export_celebi())
  (OUT/'celebi.mesh.json').unlink(missing_ok=True)
  names.remove('celebi')
  print('celebi: canonical onion fairy with coherent gaze, blue-tipped antennae and wing leaves',flush=True)
 if 'lapras' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from lapras_glb import export_lapras
  (OUT/'lapras.glb').write_bytes(export_lapras())
  (OUT/'lapras.mesh.json').unlink(missing_ok=True)
  names.remove('lapras')
  print('lapras: canonical seated shell, fitted almond gaze and articulated swimming body',flush=True)
 if 'ho_oh' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from ho_oh_glb import export_ho_oh
  (OUT/'ho_oh.glb').write_bytes(export_ho_oh())
  (OUT/'ho_oh.mesh.json').unlink(missing_ok=True)
  names.remove('ho_oh')
  print('ho_oh: original rainbow phoenix with tricolor flight vanes and golden tail fan',flush=True)
 if 'lugia' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from lugia_glb import export_lugia
  (OUT/'lugia.glb').write_bytes(export_lugia())
  (OUT/'lugia.mesh.json').unlink(missing_ok=True)
  names.remove('lugia')
  print('lugia: canonical five-digit hand wings, open mandible and planted pads',flush=True)
 if 'moltres' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from moltres_glb import export_moltres
  (OUT/'moltres.glb').write_bytes(export_moltres())
  (OUT/'moltres.mesh.json').unlink(missing_ok=True)
  names.remove('moltres')
  print('moltres: canonical gold wing strokes and coherent flame-bank articulation',flush=True)
 if 'zapdos' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from zapdos_glb import export_zapdos
  (OUT/'zapdos.glb').write_bytes(export_zapdos())
  (OUT/'zapdos.mesh.json').unlink(missing_ok=True)
  names.remove('zapdos')
  print('zapdos: canonical planted talons and coherent articulated thunder fans',flush=True)
 if 'articuno' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from articuno_glb import export_articuno
  (OUT/'articuno.glb').write_bytes(export_articuno())
  (OUT/'articuno.mesh.json').unlink(missing_ok=True)
  names.remove('articuno')
  print('articuno: canonical feather banks and articulated curling streamer',flush=True)
 if 'pidgey' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from pidgey_glb import export_pidgey
  (OUT/'pidgey.glb').write_bytes(export_pidgey())
  (OUT/'pidgey.mesh.json').unlink(missing_ok=True)
  names.remove('pidgey')
  print('pidgey: original small bird with folded wings and fitted facial landmarks',flush=True)
 if 'pidgeotto' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from pidgeotto_skin_glb import export_pidgeotto_skin
  (OUT/'pidgeotto.glb').write_bytes(export_pidgeotto_skin('pidgeotto'))
  (OUT/'pidgeotto.mesh.json').unlink(missing_ok=True)
  names.remove('pidgeotto')
  print('pidgeotto: original grounded folded-wing sculpture and full-body standard skin',flush=True)
 for bird in ('pidgeot','fearow'):
  if bird in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from raptor_bird_glb import export_raptor_bird
   (OUT/(bird+'.glb')).write_bytes(export_raptor_bird(bird))
   (OUT/(bird+'.mesh.json')).unlink(missing_ok=True)
   names.remove(bird)
   print(bird+': original individual crest, bill and layered upper/lower wing feathers',flush=True)
 for fish in ('goldeen','seaking','magikarp'):
  if fish in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from goldfish_glb import export_goldfish
   (OUT/(fish+'.glb')).write_bytes(export_goldfish(fish))
   (OUT/(fish+'.mesh.json')).unlink(missing_ok=True)
   names.remove(fish)
   print(fish+': original flowing fish with supported markings and recessed oral cavity',flush=True)
 for dragon in ('dratini','dragonair','dragonite'):
  if dragon in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from dragon_family_glb import export_dragon_family
   (OUT/(dragon+'.glb')).write_bytes(export_dragon_family(dragon))
   (OUT/(dragon+'.mesh.json')).unlink(missing_ok=True)
   names.remove(dragon)
   print(dragon+': original continuous dragon family sculpture',flush=True)
 if 'aerodactyl' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from aerodactyl_glb import export_aerodactyl
  (OUT/'aerodactyl.glb').write_bytes(export_aerodactyl())
  (OUT/'aerodactyl.mesh.json').unlink(missing_ok=True)
  names.remove('aerodactyl')
  print('aerodactyl: original open fossil jaw, supported scalloped wings and arrow tail',flush=True)
 if 'cubone' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from bone_keeper_glb import export_bone_keeper
  (OUT/'cubone.glb').write_bytes(export_bone_keeper('cubone'))
  (OUT/'cubone.mesh.json').unlink(missing_ok=True)
  names.remove('cubone')
  print('cubone: articulated compact bone carrier, recessed skull, planted feet and wrist-held club',flush=True)
 for gas_species in ('koffing','weezing'):
  if gas_species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from gas_pair_glb import export_gas
   (OUT/(gas_species+'.glb')).write_bytes(export_gas(gas_species))
   (OUT/(gas_species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(gas_species)
   print(gas_species+': canonical cratered gas orbs, articulated Weezing links and independent vapor billows',flush=True)
 if 'kangaskhan' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from kangaskhan_glb import export_kangaskhan
  (OUT/'kangaskhan.glb').write_bytes(export_kangaskhan())
  (OUT/'kangaskhan.mesh.json').unlink(missing_ok=True)
  names.remove('kangaskhan')
  print('kangaskhan: canonical armored mother and independently articulated seated baby in open pouch',flush=True)
 if 'tangela' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from tangela_glb import export_tangela
  (OUT/'tangela.glb').write_bytes(export_tangela())
  (OUT/'tangela.mesh.json').unlink(missing_ok=True)
  names.remove('tangela')
  print('tangela: canonical rooted vine skin with coherent gaze and planted red shoes',flush=True)
 for fire_canine_name in ('growlithe','arcanine'):
  if fire_canine_name not in names:continue
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from fire_canine_glb import export_fire_canine
  (OUT/(fire_canine_name+'.glb')).write_bytes(export_fire_canine(fire_canine_name))
  (OUT/(fire_canine_name+'.mesh.json')).unlink(missing_ok=True)
  names.remove(fire_canine_name)
  print(fire_canine_name+': original striped canine, fitted markings, four grounded paws and articulated cream fur',flush=True)
 if 'blissey' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from egg_nurse_glb import export_blissey
  (OUT/'blissey.glb').write_bytes(export_blissey())
  (OUT/'blissey.mesh.json').unlink(missing_ok=True)
  names.remove('blissey')
  print('blissey: original pink and white egg nurse, curling locks and rooted white frills',flush=True)
 if 'chansey' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from egg_nurse_glb import export_chansey
  (OUT/'chansey.glb').write_bytes(export_chansey())
  (OUT/'chansey.mesh.json').unlink(missing_ok=True)
  names.remove('chansey')
  print('chansey: original soft pink egg bearer, rounded ribbons and supported egg in open pink pouch',flush=True)
 if 'lickitung' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from lickitung_glb import export_lickitung
  (OUT/'lickitung.glb').write_bytes(export_lickitung())
  (OUT/'lickitung.mesh.json').unlink(missing_ok=True)
  names.remove('lickitung')
  print('lickitung: original stocky tongue bearer, supported broad tongue and cream rings',flush=True)
 if 'marowak' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from bone_keeper_glb import export_bone_keeper
  (OUT/'marowak.glb').write_bytes(export_bone_keeper('marowak'))
  (OUT/'marowak.mesh.json').unlink(missing_ok=True)
  names.remove('marowak')
  print('marowak: articulated athletic bone keeper, recessed skull, planted feet and raised wrist-held club',flush=True)
 if 'mr__mime' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from mr_mime_glb import export_mr_mime
  (OUT/'mr__mime.glb').write_bytes(export_mr_mime())
  (OUT/'mr__mime.mesh.json').unlink(missing_ok=True)
  names.remove('mr__mime')
  print('mr__mime: original cream barrier mime, five-digit gloves and curled blue shoes',flush=True)
 if 'rapidash' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from fire_horse_glb import export_rapidash
  (OUT/'rapidash.glb').write_bytes(export_rapidash())
  (OUT/'rapidash.mesh.json').unlink(missing_ok=True)
  names.remove('rapidash')
  print('rapidash: canonical fire unicorn skin, coherent layered plumes, raised foreleg and articulated open jaw',flush=True)
 if 'ponyta' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from fire_horse_glb import export_ponyta
  (OUT/'ponyta.glb').write_bytes(export_ponyta())
  (OUT/'ponyta.mesh.json').unlink(missing_ok=True)
  names.remove('ponyta')
  print('ponyta: canonical slender fire pony skin, planted hooves and independently articulated layered plumes',flush=True)
 if 'slowking' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from slow_pair_glb import export_slowking
  (OUT/'slowking.glb').write_bytes(export_slowking())
  (OUT/'slowking.mesh.json').unlink(missing_ok=True)
  names.remove('slowking')
  print('slowking: original horned spiral crown, flared royal collar and clawed pink biped',flush=True)
 if 'slowbro' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from slow_pair_glb import export_slowbro
  (OUT/'slowbro.glb').write_bytes(export_slowbro())
  (OUT/'slowbro.mesh.json').unlink(missing_ok=True)
  names.remove('slowbro')
  print('slowbro: original pink shell-tailed biped with fitted gaze, open jaws and rooted claws',flush=True)
 if 'golem' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from stone_pair_glb import export_stone_pair
  (OUT/'golem.glb').write_bytes(export_stone_pair('golem'))
  (OUT/'golem.mesh.json').unlink(missing_ok=True)
  names.remove('golem')
  print('golem: original fitted polygon stone shell, open reptile jaw and rooted claws',flush=True)
 if 'graveler' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from stone_pair_glb import export_stone_pair
  (OUT/'graveler.glb').write_bytes(export_stone_pair('graveler'))
  (OUT/'graveler.mesh.json').unlink(missing_ok=True)
  names.remove('graveler')
  print('graveler: original four-armed boulder, fitted rock face and rooted stone digits',flush=True)
 if 'rhydon' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from rhino_glb import export_rhydon
  (OUT/'rhydon.glb').write_bytes(export_rhydon())
  (OUT/'rhydon.mesh.json').unlink(missing_ok=True)
  names.remove('rhydon')
  print('rhydon: canonical planted drill biped with independent jaw, arms, ears and banded tail',flush=True)
 if 'rhyhorn' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from rhino_glb import export_rhyhorn
  (OUT/'rhyhorn.glb').write_bytes(export_rhyhorn())
  (OUT/'rhyhorn.mesh.json').unlink(missing_ok=True)
  names.remove('rhyhorn')
  print('rhyhorn: canonical planted armored quadruped with red gaze and articulated jaw and ears',flush=True)
 for species in ('phanpy','donphan'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from elephant_glb import export_elephant
   (OUT/(species+'.glb')).write_bytes(export_elephant(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical planted elephant with articulated trunk and coherent fitted details',flush=True)
 for species in ('togepi','togetic'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from egg_fairy_glb import export_egg_fairy
   (OUT/(species+'.glb')).write_bytes(export_egg_fairy(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical planted egg fairy with coherent shell rings and articulated head, arms and wings',flush=True)
 for species in ('natu','xatu'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from psychic_bird_glb import export_psychic_bird
   (OUT/(species+'.glb')).write_bytes(export_psychic_bird(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical planted psychic bird with coherent wing markings and articulated plumage',flush=True)
 if 'yanma' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from yanma_glb import export_yanma
  (OUT/'yanma.glb').write_bytes(export_yanma())
  (OUT/'yanma.mesh.json').unlink(missing_ok=True)
  names.remove('yanma')
  print('yanma: canonical four-wing skin with planted forked toes and articulated segmented abdomen',flush=True)
 if 'shuckle' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from shuckle_glb import export_shuckle
  (OUT/'shuckle.glb').write_bytes(export_shuckle())
  (OUT/'shuckle.mesh.json').unlink(missing_ok=True)
  names.remove('shuckle')
  print('shuckle: standard rigid perforated shell, planted feet and soft neck/limb skin',flush=True)
 if 'dunsparce' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from dunsparce_glb import export_dunsparce
  (OUT/'dunsparce.glb').write_bytes(export_dunsparce())
  (OUT/'dunsparce.mesh.json').unlink(missing_ok=True)
  names.remove('dunsparce')
  print('dunsparce: standard grounded land snake with coherent wings and tilted spiral drill',flush=True)
 if 'gligar' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from gligar_glb import export_gligar
  (OUT/'gligar.glb').write_bytes(export_gligar())
  (OUT/'gligar.mesh.json').unlink(missing_ok=True)
  names.remove('gligar')
  print('gligar: canonical coupled pincer/sail skin with articulated ears, tongue and venom tail',flush=True)
 if 'bellossom' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from bellossom_glb import export_bellossom
  (OUT/'bellossom.glb').write_bytes(export_bellossom())
  (OUT/'bellossom.mesh.json').unlink(missing_ok=True)
  names.remove('bellossom')
  print('bellossom: standard layered dancer skin with grounded leaf tips and coherent crown flowers',flush=True)
 for species in ('sunkern','sunflora'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from sunflower_glb import export_sunflower
   (OUT/(species+'.glb')).write_bytes(export_sunflower(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical grounded seedling or articulated ray-petal sunflower',flush=True)
 if 'qwilfish' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from qwilfish_glb import export_qwilfish
  (OUT/'qwilfish.glb').write_bytes(export_qwilfish())
  (OUT/'qwilfish.mesh.json').unlink(missing_ok=True)
  names.remove('qwilfish')
  print('qwilfish: standard coherent puffer skin with pectoral fins and articulated spotted tail',flush=True)
 if 'misdreavus' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from misdreavus_glb import export_misdreavus
  (OUT/'misdreavus.glb').write_bytes(export_misdreavus())
  (OUT/'misdreavus.mesh.json').unlink(missing_ok=True)
  names.remove('misdreavus')
  print('misdreavus: canonical six swept locks and curls with coherent spectral gaze and necklace',flush=True)
 if 'wobbuffet' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from wobbuffet_glb import export_wobbuffet
  (OUT/'wobbuffet.glb').write_bytes(export_wobbuffet())
  (OUT/'wobbuffet.mesh.json').unlink(missing_ok=True)
  names.remove('wobbuffet')
  print('wobbuffet: standard planted elastic salute skin and articulated watchful tail',flush=True)
 if 'murkrow' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from murkrow_glb import export_murkrow
  (OUT/'murkrow.glb').write_bytes(export_murkrow())
  (OUT/'murkrow.mesh.json').unlink(missing_ok=True)
  names.remove('murkrow')
  print('murkrow: canonical planted witch-crow skin with coherent folded wings and articulated feather hat and broom tail',flush=True)
 for species in ('teddiursa','ursaring'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from bear_glb import export_bear
   (OUT/(species+'.glb')).write_bytes(export_bear(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical planted bear skin with coherent lunar markings and honey-paw contact',flush=True)
 if 'girafarig' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from girafarig_glb import export_girafarig
  (OUT/'girafarig.glb').write_bytes(export_girafarig())
  (OUT/'girafarig.mesh.json').unlink(missing_ok=True)
  names.remove('girafarig')
  print('girafarig: canonical planted giraffe with fitted neck spots and independent second head',flush=True)
 if 'stantler' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from stantler_glb import export_stantler
  (OUT/'stantler.glb').write_bytes(export_stantler())
  (OUT/'stantler.mesh.json').unlink(missing_ok=True)
  names.remove('stantler')
  print('stantler: canonical planted deer with coherent antler loops and independent curled tail',flush=True)
 if 'delibird' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from delibird_glb import export_delibird
  (OUT/'delibird.glb').write_bytes(export_delibird())
  (OUT/'delibird.mesh.json').unlink(missing_ok=True)
  names.remove('delibird')
  print('delibird: canonical planted delivery bird with coherent held wing and sack',flush=True)
 for species in ('miltank','tauros'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from bovine_glb import export_bovine
   (OUT/(species+'.glb')).write_bytes(export_bovine(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical planted bovine skin with coherent fitted markings and separate tail chains',flush=True)
 for species in ('houndour','houndoom'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from hellhound_glb import export_hellhound
   (OUT/(species+'.glb')).write_bytes(export_hellhound(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical bone-armored canine skin with fixed paws and segmented tail',flush=True)
 for species in ('snubbull','granbull'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from bulldog_glb import export_bulldog
   (OUT/(species+'.glb')).write_bytes(export_bulldog(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical planted bulldog skin with coherent folded ears, spots, jaws and tusks',flush=True)
 if 'aipom' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from aipom_glb import export_aipom
  (OUT/'aipom.glb').write_bytes(export_aipom())
  (OUT/'aipom.mesh.json').unlink(missing_ok=True)
  names.remove('aipom')
  print('aipom: articulated prehensile tail hand and fitted smooth face',flush=True)
 if 'hoppip' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from hoppip_glb import export_hoppip
  (OUT/'hoppip.glb').write_bytes(export_hoppip())
  (OUT/'hoppip.mesh.json').unlink(missing_ok=True)
  names.remove('hoppip')
  print('hoppip: canonical grounded wind-plant skin and coherent serrated leaves',flush=True)
 if 'skiploom' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from skiploom_glb import export_skiploom
  (OUT/'skiploom.glb').write_bytes(export_skiploom())
  (OUT/'skiploom.mesh.json').unlink(missing_ok=True)
  names.remove('skiploom')
  print('skiploom: canonical leaf-ear and radial flower skin',flush=True)
 if 'jumpluff' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from jumpluff_glb import export_jumpluff
  (OUT/'jumpluff.glb').write_bytes(export_jumpluff())
  (OUT/'jumpluff.mesh.json').unlink(missing_ok=True)
  names.remove('jumpluff')
  print('jumpluff: canonical three-head skin with all 84 bristles preserved',flush=True)
 for species in ('mareep','flaaffy','ampharos'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from electric_sheep_glb import export_electric_sheep
   (OUT/(species+'.glb')).write_bytes(export_electric_sheep(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical planted electric sheep with coherent fleece, bands and beacon',flush=True)
 for species in ('spinarak','ariados'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from spider_glb import export_spider
   (OUT/(species+'.glb')).write_bytes(export_spider(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical six-limbed spider skin and fitted carapace markings',flush=True)
 for species in ('chinchou','lanturn'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from angler_glb import export_angler
   (OUT/(species+'.glb')).write_bytes(export_angler(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical swimming fins and flexible connected beacons',flush=True)
 for species in ('slugma','magcargo'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from lava_glb import export_lava
   (OUT/(species+'.glb')).write_bytes(export_lava(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical coherent molten neck, eye stalks and rigid vented shell',flush=True)
 for species in ('zubat','golbat','crobat'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from bat_glb import export_bat
   (OUT/(species+'.glb')).write_bytes(export_bat(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': scalloped bat wings and fitted anatomical details',flush=True)
 if 'wooper' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from wooper_glb import export_wooper
  (OUT/'wooper.glb').write_bytes(export_wooper())
  (OUT/'wooper.mesh.json').unlink(missing_ok=True)
  names.remove('wooper')
  print('wooper: canonical axolotl skin with grounded pads and articulated gills',flush=True)
 if 'quagsire' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from quagsire_glb import export_quagsire
  (OUT/'quagsire.glb').write_bytes(export_quagsire())
  (OUT/'quagsire.mesh.json').unlink(missing_ok=True)
  names.remove('quagsire')
  print('quagsire: canonical padded-foot amphibian skin and coherent dorsal ridge',flush=True)
 if 'entei' in names:
  from entei_glb import export_entei
  (OUT/'entei.glb').write_bytes(export_entei())
  (OUT/'entei.mesh.json').unlink(missing_ok=True)
  names.remove('entei')
  print('entei: canonical armored volcano-beast skin',flush=True)
 if 'raikou' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from raikou_glb import export_raikou
  (OUT/'raikou.glb').write_bytes(export_raikou())
  (OUT/'raikou.mesh.json').unlink(missing_ok=True)
  names.remove('raikou')
  print('raikou: canonical armored thunder-beast skin',flush=True)
 for species in ('ledyba','ledian'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from ladybird_glb import export_ladybird
   (OUT/(species+'.glb')).write_bytes(export_ladybird(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical distinct six-limb ladybird skin',flush=True)
 if 'noctowl' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from noctowl_glb import export_noctowl
  (OUT/'noctowl.glb').write_bytes(export_noctowl())
  (OUT/'noctowl.mesh.json').unlink(missing_ok=True)
  names.remove('noctowl')
  print('noctowl: canonical fitted-faced crowned-owl skin',flush=True)
 if 'furret' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from furret_glb import export_furret
  (OUT/'furret.glb').write_bytes(export_furret())
  (OUT/'furret.mesh.json').unlink(missing_ok=True)
  names.remove('furret')
  print('furret: canonical continuous banded-ferret skin',flush=True)
 if 'suicune' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from suicune_glb import export_suicune
  (OUT/'suicune.glb').write_bytes(export_suicune())
  (OUT/'suicune.mesh.json').unlink(missing_ok=True)
  names.remove('suicune')
  print('suicune: canonical open-crested aurora-beast skin',flush=True)
 if 'unown' in names:
  from unown_glb import export_unown
  for letter in 'abcdefghijklmnopqrstuvwxyz':(OUT/('unown_'+letter+'.glb')).write_bytes(export_unown(letter))
  (OUT/'unown.mesh.json').unlink(missing_ok=True)
  names.remove('unown')
  print('unown: canonical A..Z glyph skins',flush=True)
 if 'slowpoke' in names:
  from slowpoke_glb import export_slowpoke
  (OUT/'slowpoke.glb').write_bytes(export_slowpoke())
  (OUT/'slowpoke.mesh.json').unlink(missing_ok=True)
  names.remove('slowpoke')
  print('slowpoke: canonical sleepy quadruped skin',flush=True)
 for species in ('tentacool','tentacruel'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from tentacle_glb import export_tentacle
   (OUT/(species+'.glb')).write_bytes(export_tentacle(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': flared jelly bell, fitted red lobes and grounded articulated tendrils',flush=True)
 if 'dugtrio' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from dugtrio_glb import export_dugtrio
  (OUT/'dugtrio.glb').write_bytes(export_dugtrio())
  (OUT/'dugtrio.mesh.json').unlink(missing_ok=True)
  names.remove('dugtrio')
  print('dugtrio: three independently emerging capsules with fixed shared soil',flush=True)
 if 'diglett' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from diglett_glb import export_diglett
  (OUT/'diglett.glb').write_bytes(export_diglett())
  (OUT/'diglett.mesh.json').unlink(missing_ok=True)
  names.remove('diglett')
  print('diglett: canonical emerging capsule with fixed soil and coherent face',flush=True)
 if 'onix' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from onix_glb import export_onix
  (OUT/'onix.glb').write_bytes(export_onix())
  (OUT/'onix.mesh.json').unlink(missing_ok=True)
  names.remove('onix')
  print('onix: canonical connected rock-chain skin with rooted supports',flush=True)
 if 'voltorb' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from voltorb_glb import export_voltorb
  (OUT/'voltorb.glb').write_bytes(export_voltorb())
  (OUT/'voltorb.mesh.json').unlink(missing_ok=True)
  names.remove('voltorb')
  print('voltorb: canonical coherent shell and grounded orb skin',flush=True)
 if 'staryu' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from staryu_glb import export_staryu
  (OUT/'staryu.glb').write_bytes(export_staryu())
  (OUT/'staryu.mesh.json').unlink(missing_ok=True)
  names.remove('staryu')
  print('staryu: canonical five-arm body and radial skin',flush=True)
 if 'sentret' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from sentret_glb import export_sentret
  (OUT/'sentret.glb').write_bytes(export_sentret())
  (OUT/'sentret.mesh.json').unlink(missing_ok=True)
  names.remove('sentret')
  print('sentret: canonical tail-supported lookout with fitted belly ring',flush=True)
 if 'hoothoot' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from hoothoot_glb import export_hoothoot
  (OUT/'hoothoot.glb').write_bytes(export_hoothoot())
  (OUT/'hoothoot.mesh.json').unlink(missing_ok=True)
  names.remove('hoothoot')
  print('hoothoot: canonical round owl with clock brows and tucked second foot',flush=True)
 for larva in ('caterpie','weedle'):
  if larva in names:
   from larva_glb import export_larva
   (OUT/(larva+'.glb')).write_bytes(export_larva(larva))
   (OUT/(larva+'.mesh.json')).unlink(missing_ok=True)
   names.remove(larva)
   print(larva+': canonical connected segmented body, coherent gaze and articulated tips',flush=True)
 for species in ('metapod','kakuna','butterfree','beedrill'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   if species in ('metapod','kakuna'):
    from cocoon_glb import export_cocoon
    (OUT/(species+'.glb')).write_bytes(export_cocoon(species))
    (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   else:
    from wing_insect_glb import export_insect
    (OUT/(species+'.glb')).write_bytes(export_insect(species))
    (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': distinct insect sculpture and fitted anatomical details',flush=True)
 if 'arbok' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from arbok_glb import export_arbok
  (OUT/'arbok.glb').write_bytes(export_arbok())
  (OUT/'arbok.mesh.json').unlink(missing_ok=True)
  names.remove('arbok')
  print('arbok: canonical grounded cobra, fitted threat hood and articulated fanged jaw',flush=True)
 if 'ekans' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from ekans_glb import export_ekans
  (OUT/'ekans.glb').write_bytes(export_ekans())
  (OUT/'ekans.mesh.json').unlink(missing_ok=True)
  names.remove('ekans')
  print('ekans: canonical grounded coil, broad collar, fitted snake gaze and raised rattle',flush=True)
 if 'raichu' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from raichu_glb import export_raichu
  (OUT/'raichu.glb').write_bytes(export_raichu())
  (OUT/'raichu.mesh.json').unlink(missing_ok=True)
  names.remove('raichu')
  print('raichu: canonical cream-bellied electric mouse with coherent curled ears and a thin bolt-tipped tail',flush=True)
 for species in ('gloom','vileplume'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from rafflesia_glb import export_rafflesia
   (OUT/(species+'.glb')).write_bytes(export_rafflesia(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical rooted botanical skin and independently hinged flower',flush=True)
 for species in ('bellsprout','weepinbell','victreebel'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from pitcher_glb import export_pitcher
   (OUT/(species+'.glb')).write_bytes(export_pitcher(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical grounded botanical skin with fitted gaze, veined leaves and hollow mouths',flush=True)
 for species in ('marill','azumarill'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from water_rabbit_glb import export_water_rabbit
   (OUT/(species+'.glb')).write_bytes(export_water_rabbit(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical planted water rabbit with welded belly and articulated buoy',flush=True)
 for species in ('croconaw','feraligatr'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from crocodile_glb import export_crocodile
   (OUT/(species+'.glb')).write_bytes(export_crocodile(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': continuous crocodile skin with hinged jaw, curved limbs and fitted armor',flush=True)
 for species in ('horsea','seadra','kingdra'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from seahorse_glb import export_seahorse
   (OUT/(species+'.glb')).write_bytes(export_seahorse(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': original articulated seahorse with hollow snout, fins and coiled tail',flush=True)
 for species in ('drowzee','hypno'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from tapir_glb import export_tapir
   (OUT/(species+'.glb')).write_bytes(export_tapir(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': articulated tapirs, planted feet and held pendulum',flush=True)
 for species in ('kadabra',):
  if species in names and not args.review_kadabra_source:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from kadabra_glb import export_kadabra
   (OUT/(species+'.glb')).write_bytes(export_kadabra())
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': reviewed canonical retopology, extended neck and articulated psychic props',flush=True)
 for species in ('abra','alakazam'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from psychic_fox_glb import export_psychic
   (OUT/(species+'.glb')).write_bytes(export_psychic(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': articulated psychic foxes, planted claws and held spoons',flush=True)
 for species in ('psyduck','golduck'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from duck_glb import export_duck
   (OUT/(species+'.glb')).write_bytes(export_duck(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': fitted duck anatomy with real web surfaces',flush=True)
 for species in ('machop','machoke','machamp'):
  if species in names:
   sys.path.insert(0,str(Path(__file__).resolve().parent))
   from fighting_glb import export_fighting
   (OUT/(species+'.glb')).write_bytes(export_fighting(species))
   (OUT/(species+'.mesh.json')).unlink(missing_ok=True)
   names.remove(species)
   print(species+': canonical fighting skin with planted toes, fitted red gaze and independent arm chains',flush=True)
 if 'meganium' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from meganium_glb import export_meganium
  if (OUT/'meganium.mesh.json').exists():raise ValueError('remove the reviewed obsolete Meganium JSON before exporting its canonical GLB')
  (OUT/'meganium.glb').write_bytes(export_meganium())
  names.remove('meganium')
  print('meganium: canonical mature paper sculpture and three anatomical clips',flush=True)
 if 'bayleef' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from bayleef_glb import export_bayleef
  if (OUT/'bayleef.mesh.json').exists():raise ValueError('remove the reviewed obsolete Bayleef JSON before exporting its canonical GLB')
  (OUT/'bayleef.glb').write_bytes(export_bayleef())
  names.remove('bayleef')
  print('bayleef: canonical folded-paper sculpture and three anatomical clips',flush=True)
 if 'spearow' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from spearow_glb import export_spearow
  if (OUT/'spearow.mesh.json').exists():raise ValueError('remove the obsolete Spearow JSON before exporting its canonical GLB')
  (OUT/'spearow.glb').write_bytes(export_spearow())
  names.remove('spearow')
  print('spearow: canonical folded-paper sculpture and three anatomical clips',flush=True)
 if 'raticate' in names:
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from raticate_glb import read_raticate,export_raticate
  if (OUT/'raticate.mesh.json').exists():raise ValueError('remove obsolete Raticate JSON before re-exporting its canonical GLB')
  canonical=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/raticate.glb'
  (OUT/'raticate.glb').write_bytes(export_raticate(read_raticate(canonical)))
  names.remove('raticate')
  print('raticate: preserved canonical pear sculpture and eight-joint skin',flush=True)
 for start in range(0,len(names),args.batch_size):
  reset();scene=bpy.context.scene;roots=[]
  for name in names[start:start+args.batch_size]:
   COL=bpy.data.collections.new(name);scene.collection.children.link(COL);BUILDERS[name]();objects=list(COL.objects)
   if name in ('gengar','kadabra','raticate'):finish_hero_normals(objects)
   export(name,objects);root=bpy.data.objects.new(name+' / floor root',None);COL.objects.link(root);root['species']=name.upper();root['authorship']='Original procedural anatomy; no game textures or copied meshes';root.empty_display_size=.07
   for o in objects:o.parent=root
   roots.append(root)
  for i,root in enumerate(roots):root.location=((i%4-1.5)*2.0,i//4*2.0,0)
  camera_data=bpy.data.cameras.new('Review camera');cam=bpy.data.objects.new('Review camera',camera_data);scene.collection.objects.link(cam);cam.location=(3,-8,7);target=Vector((0,2,.4));cam.rotation_euler=(target-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=9.5;scene.camera=cam
  scene.render.engine='BLENDER_WORKBENCH';scene.display.shading.light='STUDIO';scene.display.shading.color_type='MATERIAL';scene.display.shading.show_shadows=True;scene.display.shading.show_cavity=True;scene.display.shading.cavity_type='BOTH';scene.display.shading.background_type='WORLD';scene.world.color=(.63,.67,.7);scene.display.render_aa='16';scene.view_settings.view_transform='Standard';scene.render.resolution_x=1600;scene.render.resolution_y=1300;scene.render.resolution_percentage=100
  scene['authorship']='Original editable named species anatomy. No extracted art, ROM, imported models or runtime state.';batch=start//args.batch_size+1;suffix=(names[0]+'-' if args.only else '')+f'{batch:02d}'
  bpy.ops.wm.save_as_mainfile(filepath=str(OUT/f'battle-species-{suffix}.blend'),compress=True)
  if args.preview:scene.render.filepath=str(OUT/f'battle-species-{suffix}.png');bpy.ops.render.render(write_still=True)
  print(f'Completed editable source batch {suffix}',flush=True)
run()
