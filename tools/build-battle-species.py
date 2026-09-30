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
A=argparse.ArgumentParser();A.add_argument('out',type=Path);A.add_argument('--only',default='');A.add_argument('--preview',action='store_true');A.add_argument('--batch-size',type=int,default=12)
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
 sculpt_loft('Narrow neck',[(0,.0,.65,.081,.082),(0,-.01,.79,.10,.092)],c,12)
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
  pts=[(s*.048,-.326,.874),(s*.155,-.342,.833),(s*.273,-.331,.747),(s*.348,-.293,.674)]
  # Oval cross-section keeps moustache tapered and ribbonlike, not wire whiskers.
  o=tube('Long swept moustache',pts,[.041,.037,.024,.002],'kadabra_moustache')
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
 big=n=='TYPHLOSION'
 if big:
  sph('Upright cream torso',(0,0,.48),(.27,.23,.45),'cream');sph('Blue dorsal mantle',(0,.10,.60),(.25,.18,.40),'navy');sph('Sloped head',(0,-.065,.91),(.22,.20,.18),'navy');sph('Long cream muzzle',(0,-.21,.85),(.19,.19,.10),'cream');feet('cream',.18);arms('cream',.60,.24);eyes(.94,-.257,.12,.029);sph('Dark nose',(0,-.385,.86),(.042,.022,.022),'ink')
  for x in (-.20,-.1,0,.10,.20):flame('Neck eruption',(x,.13,.86),.47)
 else:
  quad('cream',(.18,.32,.23),(.16,-.30,.66),False);sph('Long blue dorsal mantle',(0,.09,.47),(.18,.31,.16),'navy');sph('Narrow blue head',(0,-.30,.69),(.16,.16,.13),'navy');sph('Pointed cream snout',(0,-.43,.62),(.12,.14,.065),'cream');eyes(.72,-.441,.086,.027);sph('Dark nose',(0,-.562,.63),(.030,.02,.02),'ink');flame('Neck eruption',(0,-.03,.62),.26);flame('Rump eruption',(0,.31,.43),.36)
@register('CROCONAW FERALIGATR',1.30)
def water_starter(n):
 big=n=='FERALIGATR';sph('Heavy crocodile torso',(0,.03,.46),(.31,.24,.36),'blue');sph('Broad jaw head',(0,-.08,.87),(.29,.23,.22),'blue');sph('Squared snout',(0,-.26,.84),(.28,.19,.12),'blue');sph('Lower jaw',(0,-.25,.74),(.26,.18,.055),'cream');feet('blue',.22,s=(.145,.20,.09));arms('blue',.60,.30);eyes(.94,-.263,.17,.043)
 for s in (-1,1):
  for dx in (0,.07):rod('Visible tooth',(s*(.13+dx),-.39,.775),(s*(.13+dx),-.39,.83),.022,'white',.001)
  sph('Nostril',(s*.09,-.43,.88),(.023,.012,.016),'navy')
 tube('Powerful tail',[(0,.20,.30),(0,.47,.26),(0,.66,.37)], [.18,.13,.03],'blue')
 for y,z in [(.05,1.04),(.23,.78),(.31,.53),(.49,.36)]:fin('Red dorsal spike',[(-.07,y,z),(.0,y+.12,z+.23),(.07,y+.14,z-.03)],'red',.06)
 if big:
  sph('Cream belly plate',(0,-.206,.41),(.20,.055,.21),'cream')
  for s in (-1,1):sph('Muscular crocodile shoulder',(s*.29,0,.65),(.14,.13,.17),'blue');leaf('Jaw side crest',(s*.20,-.06,.97),(s*.34,.03,1.18),.06,'blue')
 else:
  sph('Primitive yellow belly',(0,-.211,.39),(.255,.050,.25),'cream')
  for x,z in [(-.11,.44),(.10,.29),(.14,.52)]:fin('Jagged belly marking',[(x-.05,-.264,z),(x+.04,-.264,z+.04),(x+.06,-.264,z-.08)],'blue',.008)
@register('FURRET',1.15)
def furret(n):
 tube('Long curved torso',[(0,.29,.17),(0,.18,.35),(0,0,.55),(0,-.03,.80)],[.17,.19,.17,.145],'cream');sph('Ferret head',(0,-.04,.87),(.19,.16,.17),'cream');eyes(.91,-.186,.10,.03);sph('Nose',(0,-.20,.85),(.028,.020,.02),'rose')
 tube('Ringed plush tail',[(0,.31,.16),(.16,.48,.18),(.23,.66,.30),(.18,.77,.48)],[.16,.14,.12,.04],'brown')
 tube('Lower warm brown body band',[(0,.215,.30),(0,.165,.40)],[.194,.197],'brown')
 tube('Upper warm brown body band',[(0,.055,.50),(0,-.009,.60)],[.184,.179],'brown')
 tube('First cream tail ring',[(.065,.375,.161),(.13,.442,.169)],[.159,.150],'cream')
 tube('Second cream tail ring',[(.188,.54,.215),(.215,.611,.263)],[.140,.130],'cream')
 tube('Third cream tail ring',[(.225,.678,.325),(.205,.728,.397)],[.114,.083],'cream')
 for s in (-1,1):sph('Brown ear',(s*.13,-.01,1.0),(.063,.055,.085),'brown');sph('Small arm',(s*.16,-.07,.54),(.055,.09,.11),'brown');sph('Hind foot',(s*.16,.12,.07),(.09,.13,.07),'brown');rod('Connected hind leg',(s*.10,.20,.26),(s*.16,.12,.10),.055,'brown')
@register('NOCTOWL',1.20)
def noctowl(n):
 sph('Tawny owl body',(0,.02,.47),(.28,.23,.43),'brown');sph('Chest bib',(0,-.20,.47),(.21,.045,.31),'tan');sph('Owl head',(0,-.025,.92),(.27,.22,.22),'tan')
 for s in (-1,1):
  sph('Facial disk',(s*.125,-.205,.92),(.12,.045,.14),'cream');sph('Eye',(s*.125,-.25,.96),(.051,.02,.055),'coral');sph('Pupil',(s*.125,-.267,.96),(.017,.008,.040),'ink');leaf('Long horn eyebrow',(s*.02,-.17,1.02),(s*.25,-.07,1.25),.10,'cream');sph('Folded wing',(s*.27,.04,.51),(.085,.21,.33),'brown')
  for row in range(4):fin('Dark chest chevron',[(s*.07,-.252,.38+row*.10),(s*.14,-.246,.42+row*.10),(s*.11,-.258,.34+row*.10)],'brown',.01)
  rod('Owl leg',(s*.13,0,.15),(s*.13,-.04,.035),.025,'coral')
  for x in (-.03,0,.03):rod('Clawed toe',(s*.13,-.03,.03),(s*.13+x,-.15,.025),.012,'coral')
 rod('Hooked beak',(0,-.22,.89),(0,-.32,.80),.045,'ink',.005)
@register('CATERPIE WEEDLE',.80)
def caterpillars(n):
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
 green=n=='METAPOD';c='green' if green else 'yellow'
 ico('Angular chrysalis',(0,0,.44),(.20,.18,.40),c,2)
 if green:
  fin('Ridged bent profile',[(-.05,-.13,.11),(-.18,-.13,.36),(-.13,-.13,.56),(.05,-.13,.84),(.07,-.13,.55),(.24,-.13,.39),(.08,-.13,.33)],'leaf',.12)
  eyes(.54,-.22,.066,.021);tube('Shell crease',[(-.14,-.19,.33),(0,-.235,.29),(.15,-.17,.35)],.012,'darkgreen')
 else:
  eyes(.63,-.145,.075,.029);fin('Folded forearm plate',[(-.18,-.13,.45),(0,-.205,.32),(.18,-.13,.45),(0,-.195,.51)],'gold',.018)
  tube('Segment shell seam',[(-.16,-.08,.27),(0,-.175,.24),(.16,-.08,.27)],.012,'gold')
@register('BUTTERFREE BEEDRILL',1.04)
def wing_insects(n):
 bee=n=='BEEDRILL';c='yellow' if bee else 'purple';sph('Thorax',(0,0,.52),(.105,.11,.19),c);sph('Head',(0,-.025,.77),(.15,.13,.14),c);eyes(.79,-.144,.09,.045,'red');sph('Abdomen',(0,.09,.28),(.13,.14,.24),c)
 for s in (-1,1):
  tube('Antenna',[(s*.06,0,.85),(s*.13,.015,1.03),(s*.16,.01,1.07)],.011,'ink')
  if bee:
   for z in (.18,.31):sph('Black abdomen band',(0,.09,z),(.134,.14,.035),'ink')
   rod('Arm',(s*.10,-.02,.57),(s*.28,-.06,.46),.032,'yellow');rod('Long forearm drill',(s*.28,-.06,.46),(s*.42,-.20,.17),.085,'white',.001)
   rod('Rear sting',(0,.12,.09),(0,.18,-.11),.06,'white',.001)
   wing=[(s*.08,.04,.61),(s*.37,.04,.98),(s*.60,.08,.99),(s*.43,.08,.61)]
  else:wing=[(s*.08,.035,.65),(s*.34,.015,1.12),(s*.68,.04,1.00),(s*.62,.03,.70),(s*.44,.04,.49),(s*.10,.03,.49)]
  fin('Dark wing perimeter',wing,'ink',.022);cen=sum((Vector(p) for p in wing),Vector())/len(wing);fin('Ivory upper wing membrane',[tuple(cen+(Vector(p)-cen)*.90+Vector((0,-.028,0))) for p in wing],'white',.008)
  if not bee:
   low=[(s*.1,.05,.54),(s*.50,.04,.55),(s*.55,.02,.32),(s*.28,.02,.17),(s*.12,.04,.35)];fin('Lower wing perimeter',low,'ink',.020);cen=sum((Vector(p) for p in low),Vector())/len(low);fin('Pale lower wing membrane',[tuple(cen+(Vector(p)-cen)*.87+Vector((0,-.025,0))) for p in low],'blue',.008)
  for dz in (.1,.25):tube('Wing vein',[(s*.12,-.015,.57),(s*.34,-.02,.63+dz),(s*.51,-.018,.69+dz)],.009,'ink')
  for z in (.4,.51):tube('Insect leg',[(s*.08,0,z),(s*.20,-.07,z-.12),(s*.21,-.19,z-.14)],.018,c)
@register('LEDYBA LEDIAN',.95)
def ladybirds(n):
 adult=n=='LEDIAN';sph('Red shell',(0,.09,.42),(.25,.14,.28),'red');sph('Cream abdomen',(0,-.08,.40),(.21,.13,.25),'cream');sph('Round head',(0,-.055,.75),(.19,.17,.19),'red');eyes(.80,-.211,.105,.048);mouth(-.207,.69)
 for s in (-1,1):
  tube('Black antenna',[(s*.09,-.02,.91),(s*.14,0,1.08),(s*.17,-.02,1.13)],.022,'ink')
  for z in (.30,.46,.59):rod('Jointed limb',(s*.19,0,z),(s*.32,-.04,z-.08),.027,'ink');sph('Glove hand',(s*.32,-.055,z-.08),(.065,.060,.07),'white')
  for z in (.30,.52):sph('Shell black spot',(s*.135,.209,z),(.075,.018,.069),'ink')
  if adult:fin('Translucent flight wing',[(s*.15,.10,.56),(s*.50,.11,.82),(s*.47,.10,.42),(s*.22,.08,.31)],'white',.018)
@register('SPINARAK ARIADOS',.90)
def spiders(n):
 big=n=='ARIADOS';c='red' if big else 'green';sph('Spider abdomen',(0,.18,.35),(.25,.31,.22),c);sph('Cephalothorax',(0,-.15,.29),(.22,.18,.17),c);eyes(.34,-.313,.12,.035,'ink');rod('Forehead horn',(0,-.11,.42),(0,-.10,.62),.065,'white',.002)
 for s in (-1,1):
  for i in range(3):
   a=(s*.16,-.11+i*.15,.31);b=(s*(.39+.035*i),-.12+i*.20,.44);d=(s*(.52+.025*i),-.27+i*.25,.045);tube('Banded spider leg',[a,b,d],[.033,.040,.024],'yellow');rod('Dark leg band',tuple(Vector(b)*.7+Vector(d)*.3),tuple(Vector(b)*.52+Vector(d)*.48),.040,'purple' if big else 'ink')
  rod('Mandible',(s*.075,-.28,.23),(s*.10,-.41,.14),.030,'white',.003)
  sph('Abdominal false eye',(s*.11,.10,.54),(.040,.055,.015),'ink')
 tube('Abdominal smile',[(-.10,.30,.535),(0,.35,.54),(.10,.30,.535)],.012,'ink')
 if big:
  for s in (-1,1):tube('High hind spur',[(s*.16,.28,.45),(s*.28,.41,.72),(s*.24,.48,.96)],[.032,.026,.008],'yellow')
@register('ZUBAT GOLBAT CROBAT',1.00)
def bats(n):
 big=n!='ZUBAT';purple=n=='CROBAT';c='purple' if purple else 'blue';sph('Bat torso',(0,0,.46),(.15,.13,.24),c);sph('Head',(0,-.02,.66),(.20,.16,.19),c)
 if n=='GOLBAT':sph('Vast open mouth',(0,-.157,.58),(.165,.030,.225),'ink')
 else:sph('Open mouth',(0,-.168,.60),(.10,.028,.083),'ink')
 if big:eyes(.73,-.165,.11,.033,'yellow',True)
 for s in (-1,1):
  leaf('Pointed ear',(s*.13,0,.74),(s*.27,.025,.95),.075,c)
  wing=[(s*.13,.02,.65),(s*.42,.02,.84),(s*.81,.03,1.0),(s*.64,.015,.56),(s*.57,.01,.29),(s*.42,.01,.46),(s*.29,0,.29)];fin('Bat wing membrane',wing,'lilac',.020);tube('Wing leading bone',[wing[0],wing[1],wing[2]],.025,c)
  for p in wing[3:]:rod('Wing finger',(s*.34,.00,.64),p,.012,c)
  for z in ([.73,.42] if n=='GOLBAT' else [.64]):rod('White fang',(s*.072,-.19,z),(s*.065,-.198,z-.065),.023,'white',.001)
  if purple:fin('Second wing pair',[(s*.11,.035,.43),(s*.48,.035,.27),(s*.56,.025,.09),(s*.27,.03,.19)],c,.03)
  else:rod('Tiny foot',(s*.07,0,.25),(s*.09,-.05,.11),.021,c,.010)
@register('MAREEP FLAAFFY AMPHAROS',1.15)
def sheep(n):
 final=n=='AMPHAROS';mid=n=='FLAAFFY';c='yellow' if final else ('pink' if mid else 'navy')
 if final:
  sph('Tall beacon body',(0,0,.44),(.23,.19,.35),c);sph('Cream belly',(0,-.169,.4),(.14,.035,.22),'cream');rod('Long neck',(0,0,.65),(0,-.02,.97),.115,c);sph('Head',(0,-.03,1.03),(.16,.16,.16),c);feet(c);arms(c,.61,.20);eyes(1.05,-.172,.075,.032);sph('Forehead red beacon',(0,-.166,1.15),(.036,.018,.044),'red')
 else:
  if mid:
   sph('Upright pink sheep body',(0,0,.35),(.23,.18,.28),c);sph('Pink sheep head',(0,-.13,.80),(.17,.16,.17),c);feet(c,.16);arms(c,.43,.22);eyes(.84,-.278,.085,.028)
  else:quad(c,(.26,.29,.25),(.16,-.29,.62),False);eyes(.65,-.428,.085,.028)
  for i in range(12):a=i*math.tau/12;sph('Soft fleece curl',(.23*math.cos(a),.07+.23*math.sin(a),.48 if not mid else .55),(.105,.10,.13),'white' if mid else 'cream')
  for x,y in [(-.12,-.08),(0,.10),(.12,-.08)]:sph('Crown wool',(x,y,.97 if mid else .66),(.12,.12,.12),'white' if mid else 'cream')
 for s in (-1,1):
  z=1.10 if final else (.89 if mid else .69);rod('Long black ear',(s*.13,-.01 if final else -.22,z),(s*.28,.02 if final else -.21,z+.07),.045,'ink',.025)
  rod('Yellow ear band',(s*.20,.005 if final else -.215,z+.033),(s*.24,.015 if final else -.212,z+.053),.047,'yellow')
 tube('Segmented tail',[(0,.18,.25),(0,.41,.25),(.08,.57,.43)],[.052,.045,.03],c);sph('Tail beacon',(.08,.57,.43),(.09,.08,.09),'red' if final else ('blue' if mid else 'orange'))
 for y in (.31,.42):rod('Black tail band',(0,y,.26),(0,y+.05,.26),.053,'ink')
@register('WOOPER QUAGSIRE',.98)
def axolotls(n):
 big=n=='QUAGSIRE';sph('Soft aquatic torso',(0,0,.34),(.28,.21,.31),'blue');sph('Wide rounded head',(0,-.02,.68),(.31,.22,.23),'blue');eyes(.71,-.228,.16,.024,white=False);mouth(-.241,.59,.17);feet('blue',.17);tube('Paddle tail',[(0,.15,.25),(0,.41,.19),(0,.60,.32)],[.14,.13,.04],'blue')
 if big:arms('blue',.43,.27);sph('Lavender dorsal ridge',(0,.216,.44),(.11,.025,.25),'lilac')
 else:
  for s in (-1,1):
   rod('External gill stalk',(s*.26,0,.68),(s*.45,0,.69),.022,'rose')
   for x in (.35,.43):rod('Gill branch',(s*x,0,.60),(s*x,0,.78),.015,'rose')
  for z in (.22,.30,.38):sph('Belly blue bars',(0,-.204,z),(.10,.015,.018),'navy')
@register('HOPPIP SKIPLOOM JUMPLUFF',.85)
def wind_plants(n):
 c={'HOPPIP':'pink','SKIPLOOM':'green','JUMPLUFF':'blue'}[n];sph('Round floating body',(0,0,.36),(.23,.20,.25),c);eyes(.42,-.19,.10,.03);mouth(-.206,.32,.075);feet(c,.12,s=(.065,.09,.045));arms(c,.36,.20)
 if n=='HOPPIP':
  for s in (-1,1):
   leaf('Broad serrated head leaf',(0,0,.56),(s*.34,.04,.90),.13,'green');leaf('Side leaf tooth',(s*.16,.015,.70),(s*.32,.04,.71),.06,'green')
  for s in (-1,1):leaf('Pointed ears',(s*.16,0,.50),(s*.29,0,.64),.07,c)
 elif n=='SKIPLOOM':
  rod('Flower stalk',(0,0,.56),(0,0,.71),.03,'green')
  for i in range(6):a=i*math.tau/6;sph('Flower petal',(.19*math.cos(a),.19*math.sin(a),.72),(.11,.11,.035),'yellow')
  sph('Flower center',(0,0,.745),(.12,.12,.055),'gold')
 else:
  for p in [(-.35,0,.47),(.35,0,.47),(0,0,.78)]:
   for i in range(7):a=i*math.tau/7;sph('Cotton seed tuft',(p[0]+.045*math.cos(a),p[1]+.045*math.sin(a),p[2]),(.115,.11,.12),'cream')
@register('BELLSPROUT WEEPINBELL VICTREEBEL',1.05)
def pitcher_plants(n):
 stage=['BELLSPROUT','WEEPINBELL','VICTREEBEL'].index(n)
 if stage==0:
  tube('Flexible stem',[(0,.03,.08),(.04,0,.35),(0,0,.68)],.038,'brown');sph('Bell head',(0,-.03,.79),(.20,.17,.22),'yellow');ring('Round pink lips',(0,-.19,.78),.115,.025,'pink');sph('Dark throat',(0,-.19,.78),(.095,.02,.095),'ink');eyes(.87,-.15,.12,.025,white=False)
  for s in (-1,1):leaf('Stem leaf',(0,0,.36),(s*.33,-.02,.59),.13,'leaf');tube('Root foot',[(0,0,.1),(s*.11,0,.055),(s*.18,-.10,.025)],.028,'brown')
 else:
  sph('Hanging pitcher',(0,0,.48),(.27,.23,.37),'yellow');ring('Rolled red pitcher lip',(0,-.22,.61),.16,.039,'pink');sph('Pitcher throat',(0,-.226,.61),(.142,.016,.143),'ink');eyes(.78,-.16,.125,.024,white=False)
  for s in (-1,1):leaf('Broad side leaf',(s*.17,0,.38),(s*.52,.01,.64),.16,'leaf')
  for x,z in [(-.13,.31),(.13,.36),(-.10,.47)]:sph('Dark green pitcher spot',(x,-.212,z),(.05,.015,.037),'green')
  if stage==2:leaf('Pitcher hood',(0,.02,.79),(.14,.02,1.15),.29,'green');tube('Long hooked vine',[(0,.17,.68),(.20,.27,.91),(.37,.25,1.06),(.47,.20,.99)],.024,'brown');rod('Lower fang',(-.09,-.255,.70),(-.075,-.26,.59),.027,'white',.002);rod('Lower fang',(.09,-.255,.70),(.075,-.26,.59),.027,'white',.002)
  else:tube('Back hook',[(0,.17,.55),(0,.32,.70),(.05,.37,.77)],.025,'brown')

@register('IVYSAUR VENUSAUR',1.20)
def bulb_evolutions(n):
 giant=n=='VENUSAUR';quad('teal',(.35,.40,.28),(.28,-.35,.52),False);eyes(.57,-.595,.17,.043,'red');mouth(-.616,.44,.16)
 for s in (-1,1):leaf('Triangular saur ear',(s*.17,-.29,.66),(s*.25,-.21,.85),.11,'teal')
 for x,y,z in [(-.23,-.46,.46),(.24,-.43,.47),(-.26,.02,.57),(.22,.15,.53)]:fin('Dark hide spot',[(x-.045,y,z),(x+.045,y,z+.035),(x+.026,y-.02,z-.05)],'darkgreen',.01)
 for i in range(6):a=i*math.tau/6;leaf('Broad back foliage',(0,.10,.57),(.53*math.cos(a),.10+.43*math.sin(a),.62),.18,'green')
 if giant:
  rod('Woody flower trunk',(0,.10,.57),(0,.10,.92),.12,'brown',.09)
  for i in range(7):a=i*math.tau/7;leaf('Large red flower petal',(0,.10,.99),(.57*math.cos(a),.10+.47*math.sin(a),.91),.22,'coral')
  sph('Raised golden flower center',(0,.10,1.05),(.18,.17,.10),'gold')
  for i in range(8):a=i*math.tau/8;sph('Flower cream speckle',(.33*math.cos(a),.10+.28*math.sin(a),1.00),(.036,.036,.014),'cream')
 else:sph('Closed rose bulb',(0,.12,.91),(.22,.19,.27),'rose');leaf('Bud seam',(0,-.06,.68),(0,.1,1.18),.015,'pink')
 for x in (-.20,.20):
  for y in (-.20,.26):
   for d in (-.04,0,.04):rod('White toe',(x+d,y-.10,.08),(x+d,y-.18,.025),.021,'white',.004)
@register('CHARMELEON CHARIZARD',1.35)
def flame_dragons(n):
 winged=n=='CHARIZARD';c='orange' if winged else 'red';sph('Dragon pear torso',(0,0,.42),(.27,.22,.35),c);sph('Cream belly',(0,-.193,.40),(.19,.043,.25),'cream');rod('Neck',(0,-.015,.64),(0,-.06,.86),.12,c);sph('Angular head',(0,-.07,.91),(.19,.19,.18),c);sph('Projecting snout',(0,-.235,.87),(.17,.14,.08),c);eyes(.94,-.228,.115,.030,'teal');feet(c,.19,s=(.13,.18,.075));arms(c,.62,.24)
 for s in (-1,1):
  rod('Horn',(s*.11,.00,1.01),(s*.14,.06,1.23 if winged else 1.13),.066,c,.002)
  for d in (-.05,0,.05):rod('Foot claw',(s*.19+d,-.16,.075),(s*.19+d,-.24,.04),.024,'white',.001)
  if winged:
   pts=[(s*.16,.14,.68),(s*.44,.16,1.07),(s*.69,.20,1.29),(s*.75,.25,.61),(s*.48,.20,.68),(s*.28,.17,.48)];fin('Teal wing membrane',pts,'teal',.025);tube('Orange wing spar',pts[:3],.045,c)
   for p in pts[3:]:rod('Wing rib',(s*.44,.16,1.07),p,.018,c)
 tube('Tapered flame tail',[(0,.16,.25),(0,.44,.23),(.18,.63,.31),(.33,.66,.56)],[.13,.10,.07,.035],c);flame('Tail flame',(.33,.66,.54),.35)
@register('WARTORTLE BLASTOISE',1.10)
def turtle_evolutions(n):
 heavy=n=='BLASTOISE';sph('Brown domed shell',(0,.06,.50),(.33,.24,.38),'brown');sph('Cream shell rim',(0,-.095,.49),(.34,.09,.36),'cream');sph('Plated belly',(0,-.17,.48),(.27,.07,.30),'tan');sph('Blue head',(0,-.08,.91),(.23,.20,.21),'blue');eyes(.96,-.261,.125,.036,'rose');mouth(-.277,.84,.14);feet('blue',.21,s=(.12,.16,.085));arms('blue',.63,.32)
 for z in (.29,.42,.55,.68):tube('Belly plate seam',[(-.23,-.202,z),(0,-.242,z-.015),(.23,-.202,z)],.008,'brown')
 if heavy:
  for s in (-1,1):
   rod('Shoulder cannon',(s*.26,.12,.80),(s*.30,-.24,1.10),.085,'silver',.075);rod('Dark cannon bore',(s*.30,-.241,1.101),(s*.307,-.265,1.125),.056,'ink')
 else:
  for s in (-1,1):leaf('Feathery fin ear',(s*.13,-.03,1.04),(s*.35,.01,1.29),.11,'lilac')
  tube('Long cloud tail',[(0,.26,.31),(0,.48,.42),(.22,.60,.58),(.39,.50,.74),(.36,.35,.81),(.20,.33,.71)],[.10,.09,.075,.055,.04,.015],'lilac')
@register('PIDGEOTTO PIDGEOT SPEAROW FEAROW FARFETCH_D MURKROW DELIBIRD',1.10)
def birds(n):
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
 if n=='FARFETCH_D':
  rod('Leek stem',(.26,-.15,.18),(.33,-.18,.77),.040,'cream')
  for i in (-1,0,1):leaf('Leek leaf',(.33,-.18,.65),(.33+i*.11,-.16,1.08),.055,'leaf')
 if murk:
  for i in (-1,0,1):leaf('Witch-hat crown',(.07*i,-.02,.91),(.13*i,.06,1.13+.06*(i==0)),.09,'navy')
  sph('Red tail tie',(0,.43,.37),(.13,.045,.08),'red')
 if deli:sph('Gift sack',(0,.30,.49),(.26,.22,.32),'cream');tube('Sack neck',[(0,.35,.74),(.11,.28,.88),(.21,.15,.73)],.06,'cream')
@register('RATICATE',.96)
def raticate(n):
 sculpt_raticate()
@register('RAICHU PICHU',.96)
def electric_mice(n):
 baby=n=='PICHU';c='yellow' if baby else 'orange';sph('Mouse body',(0,0,.33),(.20,.16,.26),c);sph('Cream belly',(0,-.143,.30),(.13,.032,.18),'cream');sph('Wide head',(0,-.02,.64),(.24,.18,.22),c);eyes(.67,-.186,.12,.032);feet(c,.14);arms(c,.36,.20);mouth(-.20,.565,.07)
 for s in (-1,1):
  if baby:fin('Black edged huge ear',[(s*.12,.0,.78),(s*.40,.01,1.00),(s*.39,-.005,.77),(s*.22,-.015,.66)],'ink',.035);fin('Yellow ear inner',[(s*.20,-.04,.79),(s*.35,-.03,.91),(s*.31,-.04,.79)],'yellow',.01)
  else:fin('Curling ear',[(s*.12,.02,.78),(s*.26,.025,.98),(s*.43,.03,1.10),(s*.41,.02,.87),(s*.32,.015,.80),(s*.36,.02,.97),(s*.25,.02,.87)],'brown',.035);leaf('Golden inner ear',(s*.18,-.045,.83),(s*.35,-.04,.96),.05,'yellow')
  sph('Electric cheek',(s*.18,-.177,.59),(.052,.019,.052),'pink' if baby else 'yellow')
 if baby:fin('Short dark bolt tail',[(0,.14,.22),(.22,.22,.26),(.30,.21,.41),(.19,.2,.45),(.11,.2,.32)],'ink',.035)
 else:tube('Whip tail',[(0,.13,.19),(0,.41,.16),(.32,.51,.29),(.44,.41,.55)],.018,'brown');fin('Lightning tail blade',[(.41,.41,.50),(.54,.4,.74),(.51,.4,.88),(.39,.4,.72),(.45,.4,.71),(.34,.4,.53)],'yellow',.030)
@register('SANDSHREW SANDSLASH',1.02)
def sand_mammals(n):
 spiny=n=='SANDSLASH';sph('Armor body',(0,.025,.39),(.28,.24,.32),'gold');sph('Cream belly',(0,-.19,.37),(.22,.04,.26),'cream');sph('Head',(0,-.03,.71),(.22,.18,.21),'gold');eyes(.75,-.198,.12,.034);sph('Pointed muzzle',(0,-.19,.64),(.12,.12,.07),'cream');feet('gold',.20);arms('gold',.46,.26)
 for s in (-1,1):leaf('Ear',(s*.13,0,.84),(s*.25,.01,.99),.08,'gold')
 if spiny:
  for row in range(3):
   for i in range(7):a=(i/6-.5)*2.4;rod('Long layered back quill',(.22*math.sin(a),.14+.12*math.cos(a),.28+row*.18),(.43*math.sin(a),.29+.19*math.cos(a),.39+row*.25),.10,'brown',.004)
 else:
  for z in (.24,.38,.52):tube('Armor horizontal seam',[(-.22,-.15,z),(0,-.204,z),(.22,-.15,z)],.009,'brown')
  for x in (-.12,.12):tube('Armor vertical seam',[(x,-.16,.19),(x,-.22,.36),(x,-.15,.61)],.009,'brown')
 for s in (-1,1):
  for i in (-1,0,1):rod('Long digging claw',(s*.39+i*.026,-.15,.32),(s*.42+i*.03,-.25,.18),.026,'white',.002)
@register('NIDORAN_F NIDORINA NIDOQUEEN NIDORAN_M NIDORINO NIDOKING',1.12)
def nido(n):
 female=n in ('NIDORAN_F','NIDORINA','NIDOQUEEN');big=n in ('NIDOQUEEN','NIDOKING');mid=n in ('NIDORINA','NIDORINO');c='blue' if female else 'purple';quad(c,(.35,.36,.29) if mid else (.29,.30,.26),(.27,-.32,.66) if mid else (.23,-.29,.62),False) if not big else None
 if big:sph('Heavy armored torso',(0,0,.47),(.32,.23,.39),c);sph('Belly armor',(0,-.21,.43),(.23,.055,.29),'cream');sph('Head',(0,-.04,.91),(.27,.22,.23),c);feet(c,.23,s=(.14,.20,.08));arms(c,.63,.30)
 z=.97 if big else .70;y=-.235 if big else (-.54 if mid else -.49);eyes(z,y,.15,.035);sph('Blunt snout',(0,y+.02,z-.09),(.19,.10,.09),c)
 for s in (-1,1):
  p=(s*.17,0 if big else -.22,z+.06);tip=(s*.38,.055 if big else -.15,z+.36 if not female else z+(.20 if mid else .29));leaf('Huge pointed ear',p,tip,.11,c);leaf('Dark inner ear',tuple(Vector(p)+Vector((0,-.038,.03))),tuple(Vector(tip)*.91+Vector(p)*.09+Vector((0,-.04,0))),.066,'teal' if female else 'rose')
  for zz in (.3,.47):sph('Flank spots',(s*.281,.06,zz),(.022,.057,.048),'navy' if female else 'rose')
 horn=.13 if female else (.25 if big or mid else .19);rod('Forehead horn',(0,y+.07,z+.11),(0,y-.08,z+.11+horn),.055,c,.002)
 for j in range(4):rod('Spinal spike',(0,.13+j*.085,.78-j*.10 if big else .55-j*.055),(0,.18+j*.10,.94-j*.14 if big else .74-j*.07),.065,c,.002)
 tube('Thick pointed tail',[(0,.21,.27),(0,.48,.29),(0,.66,.43)],[.13,.08,.006],c)
@register('CLEFABLE CLEFFA IGGLYBUFF WIGGLYTUFF',1.00)
def pink_fairies(n):
 puff=n in ('IGGLYBUFF','WIGGLYTUFF');baby=n in ('CLEFFA','IGGLYBUFF');sph('Round fairy body',(0,0,.38),(.29,.22,.33),'pink');eyes(.49,-.213,.14,.046 if puff else .025,'blue' if puff else 'ink',puff);feet('pink',.19,s=(.11,.13,.06));arms('pink',.38,.27);mouth(-.234,.36,.07)
 for s in (-1,1):
  if n=='WIGGLYTUFF':o=sph('Long rabbit ear',(s*.17,0,.88),(.086,.065,.29),'pink');o.rotation_euler[1]=s*.20;sph('Deep inner ear',(s*.18,-.055,.88),(.047,.020,.20),'ink')
  elif not puff:leaf('Pointed fairy ear',(s*.17,0,.60),(s*.31,.015,.84 if not baby else .76),.09,'pink');leaf('Brown ear tip',(s*.265,-.01,.74),(s*.31,-.005,.84 if not baby else .77),.052,'brown')
  if n=='CLEFABLE':fin('Small back fairy wing',[(s*.18,.18,.49),(s*.42,.20,.72),(s*.38,.20,.36),(s*.23,.18,.28)],'pink',.04)
 if puff:sph('Forehead curl',(0,-.08,.68),(.09,.095,.072),'pink')
 else:tube('Curled forehead lock',[(-.07,-.12,.65),(0,-.14,.72),(.07,-.14,.69),(.03,-.15,.63)],.024,'pink');spiral('Curled tail',(0,.245,.29),.15,'pink',1.2,.014)
@register('VULPIX NINETALES EEVEE VAPOREON JOLTEON FLAREON ESPEON UMBREON',1.03)
def foxes(n):
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
 adult=n=='LANTURN';sph('Streamlined fish body',(0,0,.37),(.24,.36,.23),'blue' if adult else 'navy');sph('Yellow face',(0,-.28,.38),(.18,.085,.15),'yellow');eyes(.42,-.347,.11,.043);mouth(-.371,.32,.075)
 for s in (-1,1):
  leaf('Side swimming fin',(s*.17,0,.38),(s*.37,.07,.34),.095,'blue');leaf('Forked caudal fin',(0,.32,.38),(s*.19,.55,.52),.10,'yellow' if adult else 'blue')
  if adult:
   h=.96 if s<0 else .78;tube('Branched beacon stalk',[(0,-.03,.56),(0,-.01,.82),(s*.15,-.02,h),(s*.24,-.06,h-.04)],.015,'navy');sph('Glowing lure',(s*.24,-.06,h-.05),(.066,.055,.072),'yellow')
  else:tube('Flexible light stalk',[(s*.10,-.02,.53),(s*.19,-.02,.85),(s*.30,-.04,.92),(s*.36,-.08,.74)],[.015,.013,.012,.012],'navy');sph('Glowing lure',(s*.36,-.08,.73),(.08,.07,.095),'yellow')
 if adult:leaf('Dorsal fin',(0,.12,.52),(0,.20,.77),.11,'blue')
@register('TOGEPI TOGETIC',.94)
def egg_fairies(n):
 wing=n=='TOGETIC';sph('Egg body',(0,0,.34),(.25,.21,.30),'white' if wing else 'cream');feet('cream',.14,s=(.095,.12,.055));arms('white' if wing else 'cream',.40,.22)
 if wing:rod('Long slender neck',(0,-.01,.55),(0,-.035,.77),.07,'white');sph('Small head',(0,-.035,.83),(.16,.14,.16),'white')
 else:sph('Cream head emerging',(0,-.02,.62),(.16,.13,.16),'cream')
 hz=.87 if wing else .65;eyes(hz,-.157,.075,.020,white=False);mouth(-.178,hz-.075,.052)
 for i in range(5):a=(i-2)*.52;leaf('Crown point',(.10*math.sin(a),-.015,hz+.06),(.23*math.sin(a),-.005,hz+.24-.06*abs(i-2)),.068,'white' if wing else 'cream')
 for i,(x,z,col) in enumerate([(-.13,.42,'red'),(.09,.30,'blue'),(.08,.49,'blue'),(-.10,.20,'red')]):fin('Shell geometric triangle',[(x-.045,-.205,z-.025),(x+.04,-.205,z-.02),(x,-.214,z+.05)],col,.010)
 if wing:
  for s in (-1,1):
   for i in range(3):leaf('White angel wing feather',(s*.19,.10,.51),(s*(.45+.035*i),.13,.69-.12*i),.065,'white')
 else:
  for i in range(7):a=i*math.tau/7;leaf('Broken shell rim',(.18*math.cos(a),.16*math.sin(a),.51),(.18*math.cos(a),.16*math.sin(a),.64),.044,'white')
@register('NATU XATU',1.04)
def mystic_birds(n):
 tall=n=='XATU';sph('Green bird body',(0,0,.50 if tall else .28),(.23,.18,.45 if tall else .24),'green');sph('Green head',(0,-.01,.93 if tall else .49),(.21,.18,.19),'green');hz=.96 if tall else .52;eyes(hz,-.18,.12,.045);rod('Short beak',(0,-.18,hz-.065),(0,-.30,hz-.08),.047,'yellow',.003);feet('red',.13,s=(.08,.13,.04))
 if tall:
  sph('White ceremonial chest',(0,-.167,.47),(.18,.033,.29),'white')
  for z in (.30,.45,.60):fin('Chest red chevron',[(-.15,-.204,z+.04),(0,-.211,z-.04),(.15,-.204,z+.04),(0,-.211,z+.012)],'red',.008)
 for s in (-1,1):
  sph('Folded patterned wing',(s*.22,.035,.53 if tall else .30),(.065,.18,.28 if tall else .15),'white' if tall else 'red')
  if tall:
   for z in (.34,.51,.68):fin('Red wing zigzag',[(s*.19,-.08,z),(s*.29,-.065,z+.07),(s*.28,-.06,z-.035)],'red',.015)
   leaf('Long red head streamer',(s*.10,.08,1.05),(s*.12,.42,.72),.065,'red')
  else:leaf('Red crown feather',(0,.01,.63),(s*.08,.12,.78),.045,'red')
@register('MARILL AZUMARILL',1.02)
def water_rabbits(n):
 rabbit=n=='AZUMARILL';sph('Blue round body',(0,0,.37),(.30,.24,.33),'blue');sph('White belly',(0,-.213,.29),(.235,.05,.22),'white');eyes(.49,-.222,.13,.028,white=False);mouth(-.249,.39,.065);feet('blue',.20,s=(.11,.13,.07));arms('blue',.36,.29)
 for s in (-1,1):
  if rabbit:
   o=sph('Long rabbit ear',(s*.16,0,.86),(.075,.062,.25),'blue');o.rotation_euler[1]=s*.25;sph('Red ear inner',(s*.18,-.05,.89),(.039,.018,.17),'red')
  else:sph('Round mouse ear',(s*.24,0,.67),(.12,.055,.12),'blue');sph('Red ear inner',(s*.24,-.05,.67),(.078,.015,.077),'red')
 tube('Zigzag black tail',[(.23,.12,.32),(.42,.18,.29),(.37,.23,.47),(.56,.25,.50)],.016,'ink');sph('Tail buoy',(.60,.25,.54),(.10,.10,.10),'blue')
 if rabbit:
  for x,z,r in [(-.18,.38,.027),(-.07,.43,.035),(.09,.42,.029),(.21,.35,.02)]:sph('White belly spots',(x,-.224,z),(r,.011,r),'white')
@register('SUNKERN SUNFLORA BELLOSSOM',.95)
def flowers(n):
 if n=='SUNKERN':
  sph('Seed body',(0,0,.32),(.20,.17,.27),'yellow');eyes(.38,-.17,.10,.03);mouth(-.185,.29,.06)
  for x in (-.10,0,.10):tube('Seed stripe',[(x,-.105,.15),(x,-.15,.21),(x,-.159,.27)],.006,'brown')
  for s in (-1,1):leaf('Two sprout leaves',(0,0,.55),(s*.24,.015,.79),.12,'green')
 elif n=='SUNFLORA':
  rod('Green stem body',(0,0,.11),(0,0,.67),.075,'green');sph('Smiling flower face',(0,-.03,.82),(.21,.09,.21),'yellow');eyes(.85,-.12,.09,.022,white=False);mouth(-.131,.74,.10);feet('green',.13,s=(.13,.15,.045))
  for i in range(10):a=i*math.tau/10;sph('Golden ray petal',(.28*math.cos(a),0,.82+.28*math.sin(a)),(.105,.05,.11),'gold')
  for s in (-1,1):leaf('Leaf arm',(0,0,.37),(s*.32,-.02,.48),.13,'leaf')
 else:
  sph('Small green body',(0,0,.35),(.18,.15,.24),'green');sph('Round face',(0,-.02,.65),(.20,.17,.19),'green');eyes(.68,-.183,.11,.031);mouth(-.198,.585,.075);arms('green',.39,.17)
  for i in range(9):a=i*math.tau/9;leaf('Layered leaf skirt',(0,0,.34),(.32*math.cos(a),.25*math.sin(a),.08),.10,'leaf' if i%2 else 'green')
  for s in (-1,1):
   for i in range(5):a=i*math.tau/5;sph('Red crown flower petal',(s*.17+.083*math.cos(a),-.02,.82+.08*math.sin(a)),(.061,.035,.058),'coral')
   sph('Flower yellow center',(s*.17,-.057,.82),(.04,.02,.039),'yellow')
@register('AIPOM',1.02)
def aipom(n):
 sph('Monkey body',(0,0,.32),(.18,.15,.24),'purple');sph('Large monkey head',(0,-.025,.62),(.25,.20,.22),'purple');sph('Cream face',(0,-.193,.61),(.205,.04,.16),'cream');eyes(.67,-.235,.11,.028);mouth(-.243,.535,.12);feet('cream',.14);arms('purple',.35,.18)
 for s in (-1,1):sph('Round ear',(s*.25,-.02,.66),(.092,.05,.11),'purple');sph('Ear center',(s*.25,-.065,.66),(.06,.014,.071),'cream')
 tube('Prehensile tail',[(0,.13,.20),(0,.38,.38),(.22,.50,.71),(.40,.38,.89)],[.055,.047,.04,.035],'purple');sph('Tail hand palm',(.43,.35,.94),(.14,.09,.14),'cream')
 for i in range(3):sph('Tail hand finger',(.32+i*.10,.34,1.08),(.038,.05,.11),'cream')
 for s in (-1,0,1):leaf('Head hair tuft',(s*.05,-.005,.81),(s*.09,.025,.95),.04,'purple')
@register('YANMA',.95)
def yanma(n):
 tube('Long red segmented abdomen',[(0,.43,.34),(0,.21,.40),(0,0,.45),(0,-.18,.49)],[.045,.07,.09,.10],'red');sph('Dragonfly head',(0,-.27,.56),(.23,.12,.13),'red')
 for s in (-1,1):
  sph('Huge compound eye',(s*.15,-.31,.60),(.115,.10,.125),'green');sph('Eye highlight',(s*.18,-.39,.65),(.029,.015,.035),'white')
  for y in (-.05,.18):
   fin('Long dragonfly wing',[(s*.065,y,.48),(s*.56,y-.16,.65),(s*.71,y-.12,.61),(s*.60,y+.04,.52)],'white',.014)
   fin('Red wingtip',[(s*.53,y-.15,.64),(s*.71,y-.12,.61),(s*.60,y+.04,.52),(s*.49,y,.54)],'red',.016)
  for y in (-.13,.03,.17):tube('Thin insect leg',[(s*.07,y,.42),(s*.17,y-.06,.30),(s*.22,y-.10,.26)],.012,'ink')
 for y in (.17,.29,.39):ring('Abdomen cream band',(0,y,.40 if y<.2 else .35),.063,.008,'cream')
@register('MISDREAVUS',1.02)
def misdreavus(n):
 sph('Floating ghost head',(0,-.01,.63),(.28,.22,.25),'teal');sph('Tapered ghost body',(0,.0,.37),(.20,.16,.22),'teal');eyes(.68,-.212,.14,.044,'red');mouth(-.231,.55,.10)
 for i in range(7):a=i*math.tau/7;tube('Flowing spectral hair',[(.20*math.cos(a),.12*math.sin(a),.77),(.34*math.cos(a),.20*math.sin(a),.70),(.42*math.cos(a),.25*math.sin(a),.56)],[.07,.075,.005],'teal');sph('Pink hair tip',(.40*math.cos(a),.24*math.sin(a),.58),(.048,.04,.045),'rose')
 for i in range(7):a=i*math.tau/7;sph('Red necklace orb',(.20*math.cos(a),.16*math.sin(a),.36),(.055,.05,.055),'red')
 for i in range(5):a=i*math.tau/5;leaf('Ghost skirt wisp',(0,0,.27),(.27*math.cos(a),.17*math.sin(a),.10),.09,'teal')
@register('WOBBUFFET',1.14)
def wobbuffet(n):
 sph('Tall elastic body',(0,0,.55),(.27,.19,.48),'blue');feet('blue',.17,s=(.12,.17,.06));arms('blue',.60,.26);tube('Black secret tail',[(0,.13,.25),(0,.38,.21),(.19,.48,.30)],[.07,.08,.10],'ink')
 for s in (-1,1):
  tube('Squeezed eyelid',[(s*.05,-.186,.79),(s*.10,-.20,.83),(s*.17,-.177,.78)],.012,'ink')
  sph('Tail eye',(.19+s*.047,.399,.32),(.028,.013,.037),'white');sph('Tail pupil',(.19+s*.047,.383,.32),(.010,.006,.016),'ink')
 tube('Long zigzag mouth',[(-.12,-.185,.64),(-.05,-.201,.59),(0,-.205,.63),(.05,-.201,.59),(.12,-.185,.64)],.012,'ink')
@register('GIRAFARIG STANTLER TAUROS',1.28)
def hoofed(n):
 giraffe=n=='GIRAFARIG';bull=n=='TAUROS';c='yellow' if giraffe else 'tan';quad(c,(.27,.37,.26),(.18,-.35,.96 if giraffe else .72),False);rod('Long neck',(0,-.20,.41),(0,-.34,.91 if giraffe else .67),.11,c);z=.99 if giraffe else .75;eyes(z,-.513,.09,.03);sph('Broad muzzle',(0,-.47,z-.10),(.17,.12,.09),'cream' if giraffe else 'brown')
 for s in (-1,1):
  leaf('Side ear',(s*.11,-.31,z+.06),(s*.30,-.25,z+.17),.07,c)
  for y in (-.20,.26):sph('Dark hoof',(s*.18,y-.015,.055),(.072,.10,.06),'brown')
  if bull:tube('Curved bull horn',[(s*.13,-.25,z+.12),(s*.28,-.23,z+.24),(s*.33,-.21,z+.39)],[.045,.035,.003],'cream')
  elif giraffe:rod('Ossicone',(s*.08,-.28,z+.13),(s*.10,-.26,z+.30),.021,'brown');sph('Horn tip',(s*.10,-.26,z+.31),(.041,.035,.045),'brown')
  else:
   tube('Branching antler',[(s*.12,-.25,z+.13),(s*.24,-.23,z+.36),(s*.36,-.21,z+.51)],[.031,.025,.006],'cream');rod('Antler fork',(s*.24,-.23,z+.36),(s*.13,-.20,z+.53),.025,'cream',.004);sph('Dark antler orb',(s*.26,-.223,z+.39),(.062,.028,.062),'brown')
 if giraffe:
  sph('Dark rear half',(0,.21,.34),(.275,.24,.26),'brown')
  for s in (-1,1):
   for y,z in [(-.02,.45),(-.17,.36),(.08,.53)]:sph('Giraffe spot',(s*.255,y,z),(.016,.053,.047),'brown')
  tube('Rear snake neck',[(0,.39,.36),(0,.53,.55),(0,.60,.72)],.073,'brown');sph('Sentient tail head',(0,.61,.77),(.15,.13,.14),'ink')
  for s in (-1,1):sph('Tail white eye',(s*.07,.485,.81),(.035,.015,.04),'white');sph('Tail pupil',(s*.07,.469,.81),(.012,.006,.025),'ink')
 else:
  for i in range(3 if bull else 1):tube('Tail',[(0,.36,.37),((i-1)*.15,.54,.50),((i-1)*.22,.61,.67)],[.025,.024,.018],'brown');sph('Tail tuft',((i-1)*.22,.61,.67),(.07,.055,.09),'brown')
  if bull:
   for i in range(7):a=i*math.tau/7;sph('Bull shoulder mane',(.21*math.cos(a),-.18+.12*math.sin(a),.61),(.12,.11,.13),'brown')
@register('PINECO FORRETRESS',.93)
def armored_bugs(n):
 pine=n=='PINECO';c='teal' if pine else 'rose';sph('Armored body',(0,0,.42),(.31,.25,.34),c)
 if pine:
  for row in range(4):
   for i in range(7):a=(i+(row%2)*.5)*math.tau/7;leaf('Overlapping cone plate',(.20*math.cos(a),.18*math.sin(a),.25+row*.14),(.36*math.cos(a),.29*math.sin(a),.16+row*.14),.12,'navy' if row%2 else 'teal')
  rod('Stem spike',(0,0,.69),(0,.02,.88),.07,'teal',.006);eyes(.53,-.268,.10,.033)
 else:
  sph('Dark equatorial opening',(0,-.12,.42),(.29,.16,.12),'ink');eyes(.45,-.277,.12,.034)
  for s in (-1,1):
   for z in (.25,.57):rod('Shell cannon port',(s*.22,0,z),(s*.42,-.06,z),.072,'silver');rod('Cannon bore',(s*.415,-.06,z),(s*.432,-.065,z),.045,'ink')
  for i in range(7):a=i*math.tau/7;ico('Armored shell plates',(.22*math.cos(a),.19*math.sin(a),.66),(.15,.12,.12),'rose',1)
@register('DUNSPARCE',.85)
def dunsparce(n):
 sph('Long yellow body',(0,.05,.30),(.27,.42,.23),'yellow');sph('Wide head',(0,-.30,.34),(.29,.22,.20),'yellow');sph('Blue belly',(0,-.405,.29),(.24,.06,.12),'blue')
 for s in (-1,1):
  tube('Closed eye',[(s*.05,-.50,.40),(s*.12,-.513,.42),(s*.20,-.482,.41)],.012,'ink');rod('Small fang',(s*.13,-.485,.29),(s*.14,-.49,.20),.025,'white',.002)
  for i in range(3):leaf('Tiny white wing',(s*.22,.12,.43),(s*(.44+.03*i),.10+.10*i,.58),.06,'white')
 for y in (.00,.14,.28):sph('Blue dorsal stripe',(0,y,.513),(.21,.036,.018),'blue')
 rod('Drill tail',(0,.41,.30),(0,.76,.39),.15,'gold',.004)
@register('GLIGAR',1.06)
def gligar(n):
 sph('Scorpion body',(0,0,.43),(.21,.17,.24),'lilac');sph('Broad head',(0,-.02,.72),(.27,.20,.21),'lilac');eyes(.77,-.197,.13,.036,'ink');mouth(-.222,.63,.14)
 for s in (-1,1):
  leaf('Pointed ear',(s*.17,0,.85),(s*.25,0,1.09),.095,'lilac');rod('Fang',(s*.095,-.22,.63),(s*.08,-.226,.53),.026,'white',.002)
  fin('Gliding membrane',[(s*.15,.05,.64),(s*.50,.08,.59),(s*.42,.11,.21),(s*.16,.12,.25)],'purple',.022);rod('Claw arm',(s*.22,0,.59),(s*.44,-.04,.75),.065,'lilac');sph('Pincer base',(s*.47,-.05,.84),(.13,.10,.13),'lilac')
  for d in (-1,1):leaf('Pincer finger',(s*.47+d*.035,-.08,.85),(s*.47+d*.11,-.09,1.0),.045,'lilac')
  rod('Foot',(s*.14,0,.25),(s*.21,-.07,.08),.035,'lilac')
 tube('Curved scorpion tail',[(0,.15,.28),(0,.40,.25),(.16,.56,.42),(.25,.48,.70)],[.06,.055,.045,.03],'lilac');sph('Stinger bulb',(.25,.48,.73),(.105,.08,.12),'lilac');rod('Stinger tip',(.25,.45,.81),(.24,.39,.96),.055,'purple',.002)
@register('SNUBBULL GRANBULL',1.06)
def bulldogs(n):
 big=n=='GRANBULL';c='purple' if big else 'pink';sph('Bulldog torso',(0,0,.35),(.25,.21,.30),c);sph('Broad bulldog head',(0,-.02,.70),(.31,.22,.23),c);sph('Heavy cream muzzle',(0,-.22,.60),(.24,.08,.12),'cream' if big else 'pink');eyes(.76,-.215,.16,.033);sph('Nose',(0,-.302,.66),(.07,.025,.037),'ink');feet(c,.19);arms(c,.43,.25)
 for s in (-1,1):
  o=sph('Folded ear',(s*.25,0,.84),(.09,.07,.17),c);o.rotation_euler[1]=-s*.6;rod('Lower tusk',(s*.15,-.27,.54),(s*.16,-.28,.72),.047,'white',.003)
  if not big:sph('Blue arm cuff',(s*.35,-.10,.29),(.087,.073,.035),'blue')
 if not big:
  for i in range(7):a=i*math.tau/7;leaf('Dresslike frill',(0,0,.23),(.31*math.cos(a),.25*math.sin(a),.10),.12,'pink')
  sph('Dark ear marking',(-.28,-.01,.80),(.087,.071,.081),'navy')
 else:sph('Black neck collar',(0,0,.515),(.235,.19,.046),'ink')
@register('QWILFISH',.91)
def qwilfish(n):
 sph('Inflated puffer',(0,0,.40),(.30,.28,.29),'teal');sph('Pale lower half',(0,-.035,.28),(.28,.255,.16),'cream');eyes(.48,-.256,.15,.04);sph('Round pink lips',(0,-.30,.38),(.09,.025,.065),'pink')
 for i in range(15):a=i*math.tau/15;rod('Puffer spine',(.27*math.cos(a),.04,.40+.25*math.sin(a)),(.40*math.cos(a),.04,.40+.38*math.sin(a)),.033,'teal',.001)
 for s in (-1,1):leaf('Tail lobe',(0,.27,.39),(s*.18,.57,.54),.09,'teal')
 for x,z in [(-.13,.29),(.11,.26),(0,.17)]:sph('Belly spot',(x,-.252,z),(.025,.01,.020),'teal')
@register('SHUCKLE',.91)
def shuckle(n):
 sph('Red shell',(0,0,.29),(.34,.29,.24),'red')
 for p in [(-.20,-.12,.42),(.20,-.12,.42),(0,.15,.49)]:sph('Cream shell ring',p,(.10,.08,.04),'cream');sph('Dark opening',tuple(Vector(p)+Vector((0,-.01,.025))),(.062,.052,.018),'brown')
 for s in (-1,1):
  for y in (-.17,.16):tube('Long soft leg',[(s*.23,y,.26),(s*.40,y-.03,.15),(s*.49,y-.09,.06)],[.065,.058,.045],'yellow')
 tube('Long flexible neck',[(0,-.19,.39),(0,-.21,.65),(.01,-.24,.84)],[.07,.055,.06],'yellow');sph('Rounded head',(0,-.25,.90),(.13,.11,.10),'yellow');eyes(.91,-.35,.063,.013,white=False)
@register('HERACROSS PINSIR SCYTHER SCIZOR',1.16)
def battle_insects(n):
 hera=n=='HERACROSS';pins=n=='PINSIR';sciz=n=='SCIZOR';c='navy' if hera else ('tan' if pins else ('red' if sciz else 'green'));sph('Armored abdomen',(0,.02,.35),(.22,.18,.27),c);sph('Thorax',(0,0,.59),(.17,.15,.19),c);sph('Head',(0,-.025,.84),(.20,.16,.18),c);eyes(.88,-.17,.11,.036,'yellow' if sciz else 'ink');feet(c,.17,s=(.10,.16,.06))
 for s in (-1,1):
  rod('Upper leg',(s*.12,0,.23),(s*.21,-.035,.11),.055,c);rod('Upper arm',(s*.14,0,.64),(s*.31,-.045,.54),.05,c)
  if sciz:
   rod('Forearm',(s*.31,-.045,.54),(s*.42,-.15,.69),.06,c);sph('Large pincer',(s*.46,-.15,.76),(.17,.13,.16),c);sph('Pincer eye spot',(s*.48,-.266,.80),(.057,.016,.058),'yellow');sph('Pincer black center',(s*.48,-.283,.80),(.025,.006,.027),'ink');fin('Claw split',[(s*.39,-.27,.70),(s*.61,-.26,.75),(s*.48,-.275,.66)],'ink',.007)
  elif n=='SCYTHER':fin('Long scythe blade',[(s*.31,-.05,.56),(s*.53,-.10,.79),(s*.66,-.12,.42),(s*.59,-.14,.18),(s*.48,-.12,.48)],'white',.035)
  else:rod('Forearm',(s*.31,-.045,.54),(s*.39,-.16,.40),.07,c);sph('Hand',(s*.39,-.16,.40),(.09,.08,.085),c)
  if n in ('SCYTHER','SCIZOR'):fin('Insect flight wing',[(s*.10,.16,.69),(s*.45,.22,.84),(s*.45,.23,.49),(s*.21,.20,.37)],'white',.017)
 if hera:
  tube('Long rhinoceros horn',[(0,-.01,.98),(0,.02,1.18),(0,-.04,1.38)],[.09,.065,.023],c)
  for s in (-1,1):rod('Horn fork',(0,-.02,1.30),(s*.12,-.05,1.40),.041,c,.006)
 elif pins:
  for s in (-1,1):
   tube('Large serrated head pincer',[(s*.13,0,.97),(s*.24,.005,1.22),(s*.18,-.025,1.38)],[.075,.075,.014],'silver')
   for z in (1.13,1.24):rod('Pincer inner tooth',(s*.23,0,z),(s*.10,-.02,z-.035),.025,'silver',.003)
  for i in range(5):rod('Vertical barred mouth',((i-2)*.025,-.177,.69),((i-2)*.025,-.18,.81),.016,'cream',seg=6)
 else:
  for i in (-1,0,1):leaf('Sharp head crest',(i*.08,0,.96),(i*.14,.02,1.14),.057,c)
@register('SNEASEL',1.11)
def sneasel(n):
 sph('Slim dark body',(0,0,.36),(.18,.14,.26),'navy');sph('Sharp cat head',(0,-.02,.70),(.23,.18,.21),'navy');eyes(.75,-.18,.12,.032,'red');feet('navy',.14);arms('navy',.43,.19);mouth(-.195,.60,.11)
 for s in (-1,1):
  leaf('Pointed dark ear',(s*.13,0,.84),(s*.25,.01,1.07),.07,'navy')
  for i in (-1,1):rod('Long hand claw',(s*.32+i*.025,-.13,.28),(s*.40+i*.025,-.23,.12),.035,'white',.002)
 leaf('Single rose ear plume',(.15,0,.91),(.38,.05,1.27),.105,'rose');sph('Gold forehead gem',(0,-.175,.84),(.028,.015,.04),'gold')
 for i in range(3):leaf('Three rose tail feathers',(0,.11,.26),((i-1)*.18,.44,.31+.09*(i==1)),.067,'rose')
@register('TEDDIURSA URSARING',1.14)
def bears(n):
 big=n=='URSARING';c='brown' if big else 'orange';sph('Bear torso',(0,0,.39),(.27,.22,.33),c);sph('Bear head',(0,-.01,.81),(.26,.21,.23),c);sph('Cream muzzle',(0,-.202,.72),(.16,.066,.10),'cream');eyes(.86,-.19,.12,.029);sph('Nose',(0,-.267,.76),(.045,.019,.027),'ink');feet(c,.19,s=(.115,.16,.07));arms(c,.49,.28)
 for s in (-1,1):sph('Round bear ear',(s*.19,0,.98),(.09,.06,.093),c);sph('Ear center',(s*.19,-.055,.98),(.055,.017,.059),'cream')
 if big:ring('Large belly ring',(0,-.216,.42),.147,.028,'cream')
 else:
  tube('Forehead crescent',[(-.07,-.184,.96),(-.055,-.20,.91),(0,-.213,.88),(.06,-.20,.90),(.09,-.18,.95)],.026,'cream')
@register('SLUGMA MAGCARGO',.98)
def lava_snails(n):
 shelled=n=='MAGCARGO';sph('Lava slug base',(0,.07,.16),(.33,.37,.15),'red');sph('Raised lava neck',(0,-.20,.35),(.18,.19,.29),'red')
 for s in (-1,1):rod('Eye stalk',(s*.10,-.20,.54),(s*.13,-.17,.77),.055,'red');sph('Yellow eye',(s*.13,-.209,.76),(.038,.025,.047),'yellow');sph('Black iris',(s*.13,-.232,.76),(.014,.008,.023),'ink')
 mouth(-.387,.35,.10)
 for x,y,z in [(-.20,-.1,.24),(.23,.10,.23),(-.1,.30,.27),(.16,-.22,.18)]:sph('Glowing lava bubble',(x,y,z),(.045,.043,.04),'orange')
 if shelled:
  sph('Coiled stone shell',(0,.15,.53),(.30,.23,.31),'darkstone');spiral('Shell spiral',(0,-.095,.54),.25,'stone',1.7,.015);rod('Shell chimney',(.15,.17,.75),(.22,.20,.96),.09,'darkstone',.065);sph('Hot chimney hole',(.22,.20,.97),(.048,.046,.015),'orange')
@register('SWINUB PILOSWINE',.98)
def shaggy_pigs(n):
 big=n=='PILOSWINE';sph('Shaggy oval body',(0,.035,.32),(.34,.37,.29 if not big else .43),'brown');sph('Pink pig snout',(0,-.325,.23),(.115,.045,.074),'pink')
 for s in (-1,1):sph('Nostril',(s*.039,-.367,.238),(.012,.006,.021),'brown')
 for x in (-.21,.21):
  for y in (-.21,.20):sph('Stubby foot',(x,y,.06),(.085,.11,.06),'tan')
 if big:
  for i in range(11):a=i*math.tau/11;leaf('Long hanging fur lock',(.20*math.cos(a),.21*math.sin(a),.55),(.36*math.cos(a),.37*math.sin(a),.08),.12,'tan')
  for s in (-1,1):tube('Long curved white tusk',[(s*.15,-.27,.18),(s*.23,-.34,.28),(s*.22,-.36,.44)],[.06,.037,.002],'cream')
 else:
  for x in (-.19,0,.19):tube('Dark dorsal stripe',[(x,-.20,.48),(x,0,.58),(x,.25,.45)],.024,'darkstone')
  for s in (-1,1):tube('Sleepy eye',[(s*.10,-.319,.31),(s*.16,-.308,.32)],.012,'ink')
@register('CORSOLA',.93)
def corsola(n):
 sph('Coral core',(0,0,.29),(.28,.23,.25),'pink');sph('White lower body',(0,-.02,.14),(.26,.22,.13),'white');eyes(.34,-.223,.12,.025,white=False);mouth(-.239,.25,.07);feet('pink',.19,s=(.10,.13,.05))
 for i in range(7):a=i*math.tau/7;p=(.20*math.cos(a),.15*math.sin(a),.42);end=(.38*math.cos(a),.28*math.sin(a),.79 if i%2 else .64);tube('Coral antler',[p,tuple((Vector(p)+Vector(end))*.5),end],[.07,.06,.04],'pink');sph('Rounded coral tip',end,(.044,.044,.044),'pink');q=tuple(Vector(end)+Vector((.10*math.cos(a+1),.08*math.sin(a+1),-.12)));rod('Coral branch',tuple(Vector(p)*.4+Vector(end)*.6),q,.04,'pink');sph('Coral branch tip',q,(.045,.045,.045),'pink')
@register('MANTINE',1.02)
def mantine(n):
 sph('Ray body',(0,0,.39),(.23,.34,.13),'navy');sph('Cream belly',(0,-.04,.305),(.21,.29,.04),'cream');eyes(.42,-.30,.12,.024,white=False);mouth(-.325,.345,.08)
 for s in (-1,1):
  fin('Broad manta wing',[(s*.13,-.24,.40),(s*.47,-.24,.56),(s*.81,-.13,.75),(s*.68,.18,.45),(s*.22,.31,.34)],'navy',.045);tube('Curled cephalic fin',[(s*.15,-.25,.42),(s*.23,-.41,.48),(s*.22,-.48,.40)],.039,'navy')
 tube('Thin manta tail',[(0,.27,.36),(0,.59,.27),(0,.80,.32)],.025,'navy');sph('Companion fish',(.27,.00,.275),(.045,.12,.042),'blue')
@register('SKARMORY',1.21)
def skarmory(n):
 sph('Armored avian body',(0,0,.49),(.22,.20,.31),'silver');rod('Long metal neck',(0,-.03,.68),(0,-.08,.98),.07,'silver');sph('Angular metal head',(0,-.105,1.04),(.16,.15,.13),'silver');rod('Long sharp beak',(0,-.20,1.02),(0,-.44,1.025),.054,'silver',.002);eyes(1.08,-.20,.083,.027,'yellow')
 for s in (-1,1):
  rod('Thin metal shin',(s*.13,0,.25),(s*.17,-.035,.08),.027,'silver')
  for dx in (-.03,0,.03):rod('Metal claw',(s*.17,-.035,.08),(s*.17+dx,-.18,.035),.014,'silver',.004)
  fin('Red inner wing',[(s*.13,.04,.64),(s*.68,.10,.95),(s*.79,.12,.67),(s*.29,.1,.42)],'red',.024)
  for i in range(5):leaf('Individual steel wing blade',(s*(.22+i*.10),.025,.70+i*.035),(s*(.42+i*.10),.01,.47+i*.026),.055,'silver')
 leaf('Steel head crest',(0,.0,1.12),(0,.12,1.37),.07,'silver');leaf('Spear tail',(0,.16,.36),(0,.65,.30),.11,'silver')
@register('HOUNDOUR HOUNDOOM GROWLITHE ARCANINE',1.20)
def dogs(n):
 dark=n.startswith('HOUND');large=n in ('HOUNDOOM','ARCANINE');c='ink' if dark else 'orange';quad(c,(.26,.34,.26),(.20,-.32,.73 if large else .65),False);z=.76 if large else .68;eyes(z,-.49,.105,.028);sph('Canine muzzle',(0,-.46,z-.10),(.16,.12,.075),'orange' if dark else 'cream');sph('Nose',(0,-.574,z-.065),(.04,.020,.025),'ink')
 for s in (-1,1):
  leaf('Pointed dog ear',(s*.12,-.27,z+.08),(s*.27,-.23,z+.30),.09,c)
  if dark and large:tube('Curled ram horn',[(s*.15,-.18,z+.09),(s*.27,-.07,z+.27),(s*.30,-.20,z+.43),(s*.23,-.29,z+.48)],[.055,.052,.033,.003],'cream')
  for y in (-.20,.26):
   if dark:sph('Bone ankle cuff',(s*.18,y,.15),(.07,.085,.025),'cream')
   else:
    for i in (-1,0,1):leaf('Leg fur tuft',(s*.18+i*.03,y,.19),(s*.20+i*.04,y-.03,.08),.035,'cream')
 if dark:
  for y in (0,.15,.29):tube('Riblike bone band',[(-.22,y,.43),(0,y,.60),(.22,y,.43)],.027,'cream')
  tube('Devil tail',[(0,.32,.40),(0,.56,.43),(.09,.65,.69)],[.027,.025,.015],'ink');fin('Spade tail tip',[(.09,.65,.64),(.01,.65,.79),(.10,.65,.92),(.20,.65,.78)],'ink',.025);sph('Bone throat charm',(0,-.325,.52),(.055,.035,.05),'cream')
 else:
  for i in range(9):a=i*math.tau/9;leaf('Cream neck mane',(0,-.19,.62),(.33*math.cos(a),-.19+.18*math.sin(a),.47),.12,'cream')
  for s in (-1,1):
   for y in (-.04,.14,.29):fin('Black tiger stripe',[(s*.253,y-.04,.52),(s*.272,y+.04,.48),(s*.28,y-.02,.32)],'ink',.008)
  tube('Fluffy curled tail',[(0,.33,.39),(0,.54,.60),(.08,.56,.80),(.15,.46,.83)],[.075,.11,.105,.03],'cream')
@register('PHANPY DONPHAN',1.02)
def elephants(n):
 heavy=n=='DONPHAN';c='stone' if heavy else 'blue';quad(c,(.34,.38,.29),(.26,-.29,.54),False);eyes(.62,-.509,.15,.03);tube('Curved trunk',[(0,-.49,.54),(0,-.55,.35),(0,-.57,.12),(.06,-.61,.10)],[.09,.083,.06,.04],c)
 for s in (-1,1):
  sph('Large elephant ear',(s*.28,-.17,.61),(.13,.075,.20),c);sph('Warm inner ear',(s*.33,-.226,.62),(.068,.025,.12),'orange' if not heavy else 'darkstone')
  if heavy:tube('Long ivory tusk',[(s*.17,-.41,.40),(s*.26,-.54,.36),(s*.30,-.61,.52)],[.06,.042,.001],'cream')
 if heavy:
  for y in (-.29,-.10,.10,.29):sph('Heavy segmented tire armor',(0,y,.71),(.24,.102,.075),'darkstone')
 else:
  for s in (-1,1):sph('Orange nose marking',(s*.12,-.526,.62),(.046,.016,.023),'orange')
@register('SMEARGLE',1.14)
def smeargle(n):
 sph('Dog artist body',(0,0,.38),(.18,.15,.27),'cream');sph('Round head',(0,-.02,.72),(.22,.18,.20),'cream');sph('Long muzzle',(0,-.20,.66),(.17,.11,.075),'cream');eyes(.76,-.18,.11,.022,white=False);sph('Nose',(0,-.30,.69),(.045,.02,.025),'brown');feet('brown',.14);arms('cream',.43,.18)
 for s in (-1,1):sph('Floppy ear',(s*.20,.01,.69),(.065,.07,.19),'brown')
 sph('Painter cap brim',(0,-.005,.91),(.25,.18,.055),'brown');sph('Soft beret',(0,.005,.96),(.19,.15,.08),'cream');rod('Beret tip',(0,0,1.01),(0,0,1.075),.03,'cream')
 tube('Paintbrush tail',[(0,.10,.20),(0,.34,.24),(.31,.40,.44),(.40,.30,.78)],[.047,.045,.033,.03],'brown');sph('Green paint tip',(.40,.30,.84),(.065,.06,.15),'green')
@register('MILTANK',1.12)
def miltank(n):
 sph('Plump pink cow body',(0,0,.40),(.30,.24,.34),'pink');sph('Cream belly',(0,-.20,.36),(.23,.06,.24),'cream');sph('Dark head',(0,-.02,.80),(.24,.20,.22),'ink');sph('Pink muzzle',(0,-.20,.71),(.20,.085,.12),'pink');eyes(.86,-.198,.13,.035);feet('ink',.21);arms('pink',.52,.29)
 for s in (-1,1):
  leaf('Cow ear',(s*.17,0,.91),(s*.36,.005,.93),.085,'pink');rod('Horn',(s*.14,.00,.96),(s*.20,.02,1.16),.052,'cream',.002);sph('Nostril',(s*.065,-.281,.74),(.019,.008,.014),'ink');sph('Black body spot',(s*.255,-.045,.45),(.05,.04,.07),'ink')
 sph('Udder',(0,-.218,.22),(.13,.06,.09),'pink')
 for s in (-1,1):rod('Udder teat',(s*.055,-.245,.20),(s*.055,-.26,.14),.025,'pink')
 tube('Cow tail',[(0,.19,.25),(0,.41,.31),(.10,.50,.52)],.024,'pink');sph('Black tail ball',(.10,.50,.55),(.08,.065,.095),'ink')
@register('LARVITAR PUPITAR TYRANITAR',1.24)
def armored_dinosaurs(n):
 if n=='PUPITAR':
  ico('Blue armor cocoon',(0,0,.51),(.30,.21,.45),'blue',2);eyes(.70,-.179,.11,.034,'red')
  for s in (-1,1):leaf('Lateral armored spike',(s*.20,0,.53),(s*.47,.02,.65),.11,'blue')
  for x in (-.10,.10):rod('Crown spike',(x,0,.86),(x*1.7,.01,1.16),.068,'blue',.002)
  for z in (.24,.38):tube('Cocoon plate seam',[(-.22,-.12,z),(0,-.218,z-.08),(.22,-.12,z)],.013,'navy')
  return
 big=n=='TYRANITAR';c='green' if big else 'leaf';sph('Armored dinosaur body',(0,0,.44),(.31,.24,.36),c);sph('Angular head',(0,-.06,.87),(.24,.20,.23),c);sph('Blunt snout',(0,-.23,.80),(.19,.10,.075),c);eyes(.93,-.226,.12,.032,'red');feet(c,.22,s=(.13,.19,.08));arms(c,.58,.29)
 fin('Chest diamond',[(-.14,-.225,.47),(0,-.252,.66),(.14,-.225,.47),(0,-.25,.28)],'navy' if big else 'red',.015)
 for s in (-1,1):
  for z in (.26,.42,.60):fin('Dark armor vent',[(s*.19,-.18,z),(s*.26,-.15,z+.035),(s*.23,-.20,z-.06)],'ink',.01)
  if big:
   for z in (.40,.65,.87):rod('Side armor spike',(s*.26,.07,z),(s*.43,.12,z+.12),.08,c,.002)
 rod('Forehead spike',(0,-.01,1.05),(0,.03,1.39 if not big else 1.24),.10,c,.002)
 tube('Heavy dinosaur tail',[(0,.19,.24),(0,.48,.22),(0,.67,.39)],[.17,.12,.01],c)
 for y,z in [(.10,1.0),(.22,.78),(.30,.55),(.47,.32)]:rod('Back armored spike',(0,y,z),(0,y+.13,z+.18),.09,c,.002)
@register('CELEBI',.98)
def celebi(n):
 sph('Small sprite body',(0,0,.37),(.12,.11,.20),'leaf');sph('Oversized onion head',(0,-.01,.73),(.25,.19,.25),'leaf');eyes(.75,-.20,.12,.067,'blue');mouth(-.208,.61,.065);feet('green',.095,s=(.07,.10,.04));arms('leaf',.42,.12)
 for s in (-1,1):
  tube('Long antenna',[(s*.13,0,.92),(s*.17,.035,1.14),(s*.25,.08,1.19)],.018,'green');fin('Gossamer wing',[(s*.08,.10,.48),(s*.32,.12,.79),(s*.39,.15,.62),(s*.16,.13,.36)],'white',.015)
 leaf('Onion crown point',(0,.03,.93),(0,.14,1.19),.13,'green')

@register('EKANS ARBOK DRATINI DRAGONAIR STEELIX',1.18)
def serpents(n):
 metal=n=='STEELIX';dragon=n in ('DRATINI','DRAGONAIR');c='silver' if metal else ('blue' if dragon else 'purple');pts=[(-.27,.23,.16),(-.34,-.06,.14),(-.05,-.22,.13),(.29,-.06,.17),(.29,.20,.33),(0,.20,.45),(0,.12,.73),(0,-.02,1.04)]
 if metal:
  for i,p in enumerate(pts[:-1]):ico('Segmented steel rock',p,(.17-i*.007,.16-i*.007,.16-i*.006),'silver',1)
  for i in range(len(pts)-1):rod('Connected steel vertebra',pts[i],pts[i+1],.064,'darkstone')
  sph('Heavy metal jaw',(0,-.035,1.03),(.29,.23,.18),'silver');sph('Square jaw',(0,-.145,.935),(.26,.17,.075),'silver')
  for s in (-1,1):
   for i in (1,3,5):rod('Steel body spike',(s*.11,pts[i][1],pts[i][2]),(s*.37,pts[i][1],pts[i][2]+.08),.09,'silver',.002)
  eyes(1.08,-.24,.16,.038);mouth(-.32,.97,.18)
  for s in (-1,1):rod('Steel head prong',(s*.14,.04,1.16),(s*.20,.09,1.38),.07,'silver',.002)
  return
 tube('Coiled serpent',pts,[.05,.08,.10,.11,.10,.09,.095,.095],c);sph('Serpent head',(0,-.055,1.08),(.19,.16,.15),c);eyes(1.12,-.199,.10,.03);mouth(-.221,1.025,.08)
 if n=='ARBOK':
  fin('Expanded cobra hood',[(-.09,.08,.69),(-.39,.09,.85),(-.41,.10,1.17),(-.21,.10,1.31),(.21,.10,1.31),(.41,.10,1.17),(.39,.09,.85),(.09,.08,.69)],'purple',.11)
  for s in (-1,1):sph('Cobra false eye',(s*.23,-.035,1.07),(.095,.019,.080),'yellow');sph('False eye dark center',(s*.23,-.055,1.07),(.036,.010,.040),'ink')
  tube('Hood threat mouth',[(-.22,-.052,.87),(0,-.075,.82),(.22,-.052,.87)],.022,'ink')
 elif n=='EKANS':
  for z in (.56,.69,.82,.95):sph('Cream neck band',(0,.082,z),(.091,.089,.022),'cream')
 elif dragon:
  for s in (-1,1):leaf('White ear fin',(s*.10,-.035,1.17),(s*.32,.02,1.34),.11,'white')
  sph('White muzzle',(0,-.194,1.04),(.12,.055,.068),'white');rod('Forehead horn',(0,-.07,1.20),(0,-.055,1.36),.043,'white',.003)
  if n=='DRAGONAIR':sph('Neck blue pearl',(0,-.028,.83),(.093,.07,.083),'navy');sph('Tail blue pearl',(-.28,.23,.16),(.092,.09,.09),'navy')
@register('GLOOM VILEPLUME',.97)
def rafflesia(n):
 full=n=='VILEPLUME';sph('Dark plant body',(0,0,.28),(.23,.19,.23),'navy');feet('navy',.15);arms('navy',.30,.22);eyes(.36,-.18,.11,.025,white=False);mouth(-.201,.255,.065)
 if full:
  rod('Flower stem',(0,0,.45),(0,0,.66),.085,'navy')
  for i in range(5):a=i*math.tau/5;sph('Enormous red flower petal',(.25*math.cos(a),.23*math.sin(a),.70),(.25,.23,.085),'red')
  sph('Raised flower core',(0,0,.76),(.14,.14,.07),'cream')
  for i in range(9):a=i*math.tau/9;sph('Flower pale spot',(.36*math.cos(a),.33*math.sin(a),.777),(.040,.039,.014),'cream')
 else:
  for i in range(4):a=i*math.tau/4;sph('Brown bulb lobe',(.14*math.cos(a),.12*math.sin(a),.59),(.15,.14,.18),'orange');leaf('Drooping orange petal',(0,0,.61),(.37*math.cos(a),.29*math.sin(a),.40),.13,'orange')
  for s in (-1,1):sph('Drool drop',(s*.07,-.202,.19),(.034,.022,.09),'white')
@register('PARAS PARASECT',.91)
def mushroom_crabs(n):
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
@register('DUGTRIO',.98)
def dugtrio(n):
 for j,(x,y,h) in enumerate([(-.18,.08,.48),(.15,.12,.74),(.0,-.14,.34)]):
  sph('Separate mole body',(x,y,h*.5),(.17,.16,h*.5),'brown');sph('Mole nose',(x,y-.158,h-.13),(.061,.03,.035),'pink')
  for s in (-1,1):sph('Mole eye',(x+s*.051,y-.153,h-.05),(.011,.010,.026),'ink')
 for i in range(11):a=i*math.tau/11;ico('Loose soil stone',(.31*math.cos(a),.24*math.sin(a),.035),(.10,.09,.05),'tan',1)
@register('MEOWTH PERSIAN',1.01)
def cats(n):
 quadcat=n=='PERSIAN'
 if quadcat:quad('cream',(.24,.32,.21),(.19,-.28,.63),False)
 else:sph('Slim cat body',(0,0,.31),(.16,.13,.23),'cream');sph('Large cat head',(0,-.015,.64),(.24,.18,.21),'cream');feet('brown',.14);arms('cream',.39,.17)
 eyes(.68,-.45 if quadcat else -.188,.115,.028);y=-.40 if quadcat else -.14;hz=.64
 for s in (-1,1):
  leaf('Dark pointed cat ear',(s*.13,y+.12,.78),(s*.27,y+.14,1.0),.095,'brown');leaf('Pink ear inset',(s*.16,y+.085,.82),(s*.24,y+.095,.94),.05,'pink');sph('Whisker muzzle',(s*.07,y-.08,.60),(.082,.04,.063),'cream')
  for z in (.62,.69):tube('Whisker',[(s*.15,y-.035,z),(s*.34,y-.06,z+.035)],.008,'brown')
 sph('Small nose',(0,y-.115,.64),(.027,.014,.021),'pink');tube('Curled cat tail',[(0,.20,.23),(0,.43,.31),(.18,.54,.53),(.28,.48,.66),(.19,.43,.66)],[.039,.034,.025,.015,.009],'brown' if not quadcat else 'cream')
 if quadcat:sph('Forehead ruby',(0,-.427,.80),(.036,.018,.041),'red')
 else:sph('Gold forehead coin',(0,-.178,.82),(.064,.019,.11),'gold');tube('Coin engraved line',[(0,-.20,.74),(0,-.201,.90)],.009,'brown')
@register('PSYDUCK GOLDUCK',1.03)
def ducks(n):
 blue=n=='GOLDUCK';c='blue' if blue else 'yellow';sph('Duck torso',(0,0,.34),(.25,.20,.29),c);sph('Duck head',(0,-.02,.71),(.25,.21,.23),c);sph('Flat duck bill',(0,-.26,.63),(.19,.16,.060),'cream' if not blue else 'blue');eyes(.77,-.207,.13,.040);feet('cream',.17,s=(.14,.19,.045));arms(c,.42,.24)
 if blue:
  for i in (-1,0,1):rod('Pointed head crest',(i*.09,0,.88),(i*.16,.04,1.17-.07*abs(i)),.065,'blue',.002)
  sph('Forehead ruby',(0,-.194,.90),(.035,.020,.046),'red');tube('Swimmer tail',[(0,.15,.20),(0,.44,.20),(0,.57,.34)],[.11,.07,.01],'blue')
 else:
  for i in (-1,0,1):tube('Three black head hairs',[(i*.04,.01,.90),(i*.075,.055,1.03)],.009,'ink')
@register('MANKEY PRIMEAPE',1.02)
def pig_monkeys(n):
 angry=n=='PRIMEAPE';sph('Round furry body',(0,0,.43),(.29,.22,.30),'cream');sph('Pig snout',(0,-.225,.40),(.105,.045,.075),'pink');eyes(.54,-.207,.13,.027);feet('brown',.22,s=(.10,.15,.065))
 for s in (-1,1):
  sph('Nostril',(s*.037,-.267,.42),(.012,.007,.015),'brown');leaf('Small ear',(s*.20,0,.64),(s*.33,.035,.78),.09,'cream');tube('Raised arm',[(s*.23,0,.53),(s*.38,-.045,.60),(s*.42,-.07,.78)],[.055,.052,.047],'cream');sph('Clenched hand',(s*.42,-.08,.80),(.105,.09,.095),'brown')
  tube('Angry brow',[(s*.06,-.218,.57),(s*.17,-.195,.63)],.013,'brown')
 for i in range(12):a=i*math.tau/12;leaf('Fur silhouette point',(.23*math.cos(a),.02,.44+.25*math.sin(a)),(.35*math.cos(a),.025,.44+.34*math.sin(a)),.055,'cream')
 if not angry:tube('Long monkey tail',[(0,.15,.26),(0,.42,.27),(.18,.53,.47),(.28,.44,.56)],.034,'brown')
 else:
  for s in (-1,1):sph('Dark wrist cuff',(s*.42,-.07,.73),(.073,.072,.036),'ink')
@register('POLIWHIRL POLIWRATH POLITOED',1.07)
def frogs(n):
 toad=n=='POLITOED';c='green' if toad else 'blue';sph('Broad frog body',(0,0,.41),(.32,.24,.34),c);sph('Round belly',(0,-.215,.36),(.25,.052,.25),'yellow' if toad else 'white');spiral('Belly spiral',(0,-.276,.36),.19,'green' if toad else 'ink',2.2,.009);feet(c,.23,s=(.14,.16,.07));arms(c,.46,.31)
 for s in (-1,1):sph('Raised eye socket',(s*.17,-.005,.74),(.10,.12,.105),c);sph('White eye',(s*.17,-.104,.75),(.058,.029,.063),'white');sph('Pupil',(s*.17,-.131,.76),(.025,.009,.041),'ink')
 if not toad:
  for s in (-1,1):sph('White punching glove',(s*.44,-.12,.31),(.11,.105,.11),'white')
 if n=='POLIWRATH':
  for s in (-1,1):tube('Severe brow',[(s*.06,-.118,.79),(s*.23,-.087,.83)],.025,'blue')
 if toad:tube('Curled head hair',[(0,0,.74),(0,.025,.93),(.10,.02,.96),(.13,0,.89),(.07,-.005,.85)],.016,'green');sph('Pink cheek',(-.24,-.163,.56),(.059,.025,.067),'pink');sph('Pink cheek',(.24,-.163,.56),(.059,.025,.067),'pink');mouth(-.24,.59,.13)
@register('TENTACOOL TENTACRUEL',1.13)
def jellyfish(n):
 many=n=='TENTACRUEL';sph('Blue jelly bell',(0,0,.72),(.31,.25,.25),'blue');sph('Dark lower mantle',(0,-.015,.56),(.25,.22,.12),'navy')
 for s in (-1,1):sph('Large red bell gem',(s*.18,-.13,.80),(.10,.08,.14),'red');sph('Gem highlight',(s*.19,-.195,.87),(.027,.015,.035),'coral')
 sph('Central blue gem',(0,-.226,.69),(.06,.028,.075),'blue');eyes(.56,-.21,.11,.031)
 for i in range(10 if many else 2):a=i*math.tau/(10 if many else 2);x=.15*math.cos(a);y=.12*math.sin(a);tube('Long waving tentacle',[(x,y,.52),(x*1.6,y*1.5,.32),(x*2.1,y*1.7,.13),(x*2.3-.06,y*1.7-.07,.05)],[.04,.033,.024,.014],'navy' if many else 'cream')
 if many:
  for s in (-1,1):rod('Side head beak',(s*.24,-.06,.60),(s*.43,-.10,.42),.09,'cream',.006)
@register('GRAVELER GOLEM RHYHORN RHYDON',1.11)
def rocks(n):
 rhino=n.startswith('RHY');golem=n=='GOLEM';biped=n!='RHYHORN';c='stone'
 if not biped:quad(c,(.37,.41,.26),(.26,-.33,.44),False);eyes(.52,-.565,.14,.032)
 else:
  ico('Faceted rock torso',(0,0,.43),(.35,.28,.35),'darkstone' if golem else c,2);sph('Rock head',(0,-.07,.74),(.24,.22,.20),c);eyes(.79,-.275,.13,.036);feet(c,.24,s=(.13,.18,.075))
  for s in (-1,1):
   for z in ([.59,.39] if n=='GRAVELER' else [.53]):rod('Heavy arm',(s*.25,0,z),(s*.43,-.05,z-.09),.09,c);ico('Stone fist',(s*.47,-.10,z-.12),(.13,.12,.12),c,1)
 if golem:
  for row in range(3):
   for i in range(7):a=i*math.tau/7;ico('Interlocking shell plate',(.29*math.cos(a),.23*math.sin(a),.25+row*.15),(.14,.11,.11),'brown',1)
 if rhino:
  y=-.27 if biped else -.53;z=.81 if biped else .48;rod('Rhinoceros drill horn',(0,y,z),(0,y-.24,z+.23),.09,'cream',.001)
  for s in (-1,1):leaf('Pointed armored ear',(s*.15,y+.16,z+.09),(s*.31,y+.21,z+.30),.10,c)
  for i in range(4):ico('Dorsal armor ridge',(0,.02+i*.12,.73 if not biped else .65-i*.07),(.15,.13,.11),'darkstone',1)
  if biped:sph('Cream plated belly',(0,-.234,.37),(.22,.06,.26),'cream');tube('Armored tail',[(0,.19,.25),(0,.44,.22),(0,.58,.33)],[.12,.075,.005],c)
@register('PONYTA RAPIDASH',1.22)
def fire_horses(n):
 unicorn=n=='RAPIDASH';quad('cream',(.23,.37,.22),(.145,-.34,.79),False);rod('Long horse neck',(0,-.23,.43),(0,-.34,.81),.095,'cream');sph('Long equine muzzle',(0,-.47,.74),(.12,.16,.075),'cream');eyes(.84,-.454,.079,.024)
 for s in (-1,1):leaf('Horse ear',(s*.08,-.30,.90),(s*.13,-.27,1.07),.047,'cream')
 for y,z in [(-.26,.87),(-.13,.72),(.03,.65),(.21,.59)]:flame('Flaming mane',(0,y,z),.25)
 for s in (-1,1):
  for y in (-.20,.26):flame('Flaming ankle',(s*.18,y,.08),.22)
 tube('Flame tail core',[(0,.31,.43),(0,.55,.46),(0,.70,.65)],.035,'orange');flame('Flowing tail fire',(0,.54,.43),.42)
 if unicorn:rod('Unicorn horn',(0,-.39,.91),(0,-.48,1.24),.047,'cream',.001)
@register('SLOWBRO SLOWKING',1.15)
def slow_evolutions(n):
 king=n=='SLOWKING';sph('Chubby pink body',(0,0,.42),(.30,.24,.35),'pink');sph('Cream belly',(0,-.218,.38),(.21,.050,.24),'cream');sph('Slow head',(0,-.035,.82),(.27,.23,.21),'pink');sph('Wide cream muzzle',(0,-.25,.73),(.25,.12,.09),'cream');eyes(.87,-.245,.15,.040);feet('pink',.22,s=(.12,.18,.065));arms('pink',.53,.30);mouth(-.369,.71,.13)
 for s in (-1,1):sph('Curled ear',(s*.20,.03,.98),(.075,.06,.075),'pink')
 tube('Thick pink tail',[(0,.16,.26),(0,.44,.25),(.15,.63,.40)],[.14,.13,.10],'pink')
 if king:
  sph('Crown shell',(0,.015,1.12),(.25,.19,.17),'cream')
  for s in (-1,1):rod('Crown prong',(s*.15,0,1.23),(s*.23,.02,1.45),.07,'cream',.001)
  sph('Crown ruby',(0,-.173,1.17),(.067,.025,.08),'red')
  for i in range(8):a=i*math.tau/8;leaf('Royal neck ruff',(0,-.015,.66),(.33*math.cos(a),.22*math.sin(a),.62),.10,'white' if i%2 else 'red')
 else:
  sph('Tail biting spiral shell',(.16,.64,.47),(.24,.20,.24),'stone');spiral('Shell coil',(.16,.43,.47),.18,'darkstone',1.5,.026)
  for i in range(5):a=i*math.tau/5;rod('Shell spike',(.16+.15*math.cos(a),.64,.47+.15*math.sin(a)),(.16+.30*math.cos(a),.65,.47+.30*math.sin(a)),.067,'stone',.002)
@register('MAGNEMITE MAGNETON',.96)
def magnets(n):
 centers=[(0,0,.42)] if n=='MAGNEMITE' else [(0,0,.70),(-.27,-.01,.23),(.27,.01,.23)]
 for x,y,z in centers:
  sph('Steel sphere',(x,y,z),(.18,.16,.17),'silver');sph('Single white eye',(x,y-.152,z+.02),(.084,.026,.085),'white');sph('Black eye pupil',(x,y-.178,z+.02),(.027,.010,.035),'ink');rod('Top screw',(x,y,z+.14),(x,y,z+.25),.038,'silver');rod('Screw head',(x-.06,y,z+.25),(x+.06,y,z+.25),.023,'silver')
  for s in (-1,1):
   pts=[(x+s*.16,y,z),(x+s*.28,y,z),(x+s*.28,y,z+.12)];tube('U magnet upper half',pts,.037,'silver');tube('U magnet lower half',[(x+s*.16,y,z),(x+s*.28,y,z),(x+s*.28,y,z-.12)],.037,'silver');rod('Red magnet pole',(x+s*.28,y,z+.10),(x+s*.28,y,z+.17),.04,'red');rod('Blue magnet pole',(x+s*.28,y,z-.10),(x+s*.28,y,z-.17),.04,'blue')
@register('DODUO DODRIO',1.22)
def multi_birds(n):
 triple=n=='DODRIO';sph('Feathery brown body',(0,.03,.38),(.27,.24,.26),'brown');count=3 if triple else 2
 for i in range(count):x=(i-(count-1)/2)*.30;z=.98+(.12 if i==1 else 0);tube('Separate long neck',[(x*.35,.01,.48),(x*.8,0,.74),(x,0,z)],[.04,.032,.028],'brown');sph('Bird head',(x,0,z),(.14,.12,.14),'brown');rod('Pointed beak',(x,-.10,z),(x,-.34,z-.01),.052,'tan',.003)
 for i in range(count):
  x=(i-(count-1)/2)*.30;z=.98+(.12 if i==1 else 0)
  for s in (-1,1):sph('Eye',(x+s*.07,-.105,z+.038),(.026,.013,.031),'white');sph('Eye pupil',(x+s*.07,-.118,z+.038),(.011,.007,.018),'ink')
  if triple:
   for s in (-1,1):leaf('Black head plume',(x+s*.045,0,z+.10),(x+s*.07,.03,z+.30),.03,'ink')
 for s in (-1,1):
  tube('Long bird leg',[(s*.12,0,.26),(s*.16,.02,.15),(s*.15,-.02,.03)],[.025,.021,.017],'tan')
  for dx in (-.04,0,.04):rod('Toe',(s*.15,-.02,.03),(s*.15+dx,-.17,.025),.01,'tan')
 for i in range(4):leaf('Tail plume',(0,.15,.37),((i-1.5)*.10,.47,.69),.075,'red' if triple else 'brown')
@register('SEEL DEWGONG',1.03)
def seals(n):
 slim=n=='DEWGONG';sph('Seal body',(0,.12,.30),(.28,.43,.25),'white');sph('Raised seal neck',(0,-.21,.50),(.20,.21,.26),'white');sph('Seal head',(0,-.28,.68),(.22,.19,.20),'white');eyes(.73,-.451,.11,.028,white=False);sph('Cream muzzle',(0,-.45,.61),(.16,.09,.08),'cream');sph('Nose',(0,-.526,.66),(.038,.017,.024),'ink');rod('Single head horn',(0,-.23,.85),(0,-.19,1.03),.06,'white',.001)
 for s in (-1,1):
  leaf('Front seal flipper',(s*.19,-.09,.33),(s*.48,-.28,.11),.13,'white');leaf('Split tail fluke',(0,.43,.24),(s*.22,.65,.48),.14,'white')
  if not slim:rod('Small seal tusk',(s*.07,-.50,.60),(s*.065,-.51,.51),.025,'white',.002)
 if not slim:sph('Playful tongue',(0,-.535,.57),(.065,.025,.067),'pink')
@register('GRIMER MUK DITTO',.96)
def slimes(n):
 ditto=n=='DITTO';c='lilac' if ditto else 'purple';sph('Viscous base',(0,0,.16),(.37,.30,.15),c);sph('Lumpy central mass',(0,0,.42),(.28,.24,.30),c)
 for i in range(9):a=i*math.tau/9;sph('Pooled irregular lobe',(.29*math.cos(a),.23*math.sin(a),.12),(.13,.11,.10),c)
 if ditto:
  for s in (-1,1):leaf('Slime ear mound',(s*.16,0,.59),(s*.25,.01,.79),.09,c)
  eyes(.50,-.232,.12,.018,white=False);mouth(-.254,.40,.105)
 else:
  eyes(.56,-.222,.14,.038);sph('Open sludge mouth',(0,-.241,.43),(.14,.022,.085),'ink')
  for s in (-1,1):tube('Rising sludge arm',[(s*.22,0,.39),(s*.43,-.04,.53),(s*.48,-.08,.75 if n=='GRIMER' else .58)],[.085,.075,.06],c);sph('Sludge hand',(s*.48,-.08,.76 if n=='GRIMER' else .60),(.12,.085,.08),c)
  for z in (.24,.36,.64):tube('Viscous fold',[(-.22,-.16,z),(0,-.259,z-.035),(.22,-.16,z)],.026,'rose')
@register('SHELLDER CLOYSTER',1.05)
def bivalves(n):
 spiky=n=='CLOYSTER';sph('Black mollusk',(0,0,.38),(.22,.15,.21),'ink');eyes(.45,-.142,.10,.037);c='purple' if spiky else 'lilac'
 for s in (-1,1):
  o=sph('Open shell valve',(s*.25,.015,.46),(.15,.25,.36),c);o.rotation_euler[1]=s*.35
  for z in (.24,.39,.56,.69):tube('Shell growth ridge',[(s*.20,-.17,z),(s*.30,-.10,z+.03),(s*.37,.015,z)],.013,'cream' if not spiky else 'purple')
  if spiky:
   for z in (.24,.47,.69):rod('Long shell spike',(s*.32,.02,z),(s*.61,.045,z+.15),.08,c,.002)
 if spiky:rod('Tall shell crown',(0,.11,.70),(0,.12,1.18),.13,c,.002);mouth(-.17,.35,.09)
 else:sph('Protruding tongue',(0,-.18,.19),(.095,.23,.045),'pink')
@register('GASTLY HAUNTER GENGAR',1.05)
def ghosts(n):
 if n=='GENGAR':sculpt_gengar();return
 gaseous=n=='GASTLY';hands=n=='HAUNTER';sph('Ghost body',(0,0,.49),(.30,.25,.30),'ink' if gaseous else 'purple');eyes(.58,-.229,.15,.044,'red');sph('Wide ghost grin',(0,-.246,.38),(.22,.03,.085),'white')
 for i in range(-3,4):tube('Tooth gap',[(i*.055,-.278,.32),(i*.055,-.278,.44)],.006,'purple')
 for s in (-1,1):leaf('Angry eye brow',(s*.04,-.24,.64),(s*.25,-.19,.72),.048,'ink' if gaseous else 'purple')
 if gaseous:
  for i in range(13):a=i*math.tau/13;sph('Vapor cloud lobe',(.34*math.cos(a),.03,.49+.32*math.sin(a)),(.105,.14,.105),'lilac' if i%3==0 else 'purple')
 else:
  for i in range(8):a=i*math.tau/8;leaf('Ghost silhouette spike',(.23*math.cos(a),.045,.49+.22*math.sin(a)),(.41*math.cos(a),.075,.49+.41*math.sin(a)),.08,'purple')
  if hands:
   for s in (-1,1):
    sph('Detached ghost hand',(s*.43,-.04,.27),(.13,.08,.07),'purple')
    for i in (-1,0,1):rod('Claw finger',(s*.43+i*.06,-.07,.28),(s*.46+i*.07,-.19,.37),.037,'purple',.002)
  else:feet('purple',.21);arms('purple',.44,.29)
@register('KRABBY KINGLER',.97)
def crabs(n):
 king=n=='KINGLER';sph('Crab carapace',(0,0,.31),(.27,.23,.18),'orange');sph('Cream underside',(0,-.11,.23),(.23,.14,.075),'cream')
 for s in (-1,1):
  rod('Eye stalk',(s*.12,-.09,.41),(s*.15,-.13,.57),.024,'orange');sph('Eye',(s*.15,-.13,.59),(.042,.038,.055),'white');sph('Pupil',(s*.15,-.169,.60),(.013,.009,.032),'ink')
  for i in range(3):tube('Jointed leg',[(s*.20,-.03+i*.10,.28),(s*.39,-.06+i*.16,.27),(s*.45,-.16+i*.16,.04)],[.032,.026,.018],'orange')
  large=king and s<0;r=.19 if large else .105;rod('Pincer arm',(s*.20,-.13,.32),(s*.40,-.20,.48),.043,'orange');sph('Large claw palm',(s*.44,-.23,.55),(r,r*.7,r),'orange')
  for d in (-1,1):leaf('Open claw finger',(s*.44+d*r*.35,-.25,.61),(s*.44+d*r*.70,-.27,.82 if large else .74),r*.36,'orange')
 if king:
  for i in (-1,0,1):rod('Crown point',(i*.08,.02,.46),(i*.14,.02,.69),.049,'orange',.002)
@register('ELECTRODE',.94)
def electrode(n):
 for upper,color in ((True,'white'),(False,'red')):
  seg=20;rings=8;vs=[(0,0,.39+(.38 if upper else -.38))]
  for i in range(1,rings+1):
   t=i/rings*math.pi/2
   for j in range(seg):a=j/seg*math.tau;vs.append((.38*math.sin(t)*math.cos(a),.34*math.sin(t)*math.sin(a),.39+(.38 if upper else -.38)*math.cos(t)))
  fs=[(0,1+j,1+(j+1)%seg) for j in range(seg)]
  for i in range(rings-1):
   for j in range(seg):a=1+i*seg+j;b=1+i*seg+(j+1)%seg;fs.append((a,b,b+seg,a+seg))
  fs.append(tuple(1+(rings-1)*seg+j for j in range(seg)))
  poly('White upper hemisphere' if upper else 'Red lower hemisphere',vs,fs,color,True)
 eyes(.50,-.31,.16,.032,white=False);sph('Black smile surround',(0,-.315,.29),(.24,.035,.087),'ink');sph('Toothy grin',(0,-.345,.31),(.21,.010,.047),'white')
 for i in range(-3,4):tube('Tooth line',[(i*.051,-.357,.274),(i*.051,-.357,.35)],.006,'ink')
@register('EXEGGCUTE EXEGGUTOR',1.22)
def eggs_palms(n):
 if n=='EXEGGCUTE':
  for i,(x,y,h) in enumerate([(-.24,0,.25),(0,-.10,.22),(.25,0,.25),(-.20,.24,.24),(.04,.21,.33),(.26,.25,.20)]):
   sph('Separate pink egg',(x,y,h),(.15,.13,h),'pink')
   for s in (-1,1):sph('Egg eye',(x+s*.05,y-.12,h+.07),(.015,.010,.026),'ink')
   tube('Egg frown',[(x-.035,y-.137,h-.015),(x,y-.144,h+.002),(x+.035,y-.137,h-.015)],.006,'brown')
   if i==4:fin('Cracked egg opening',[(x-.08,y-.02,h*1.91),(x-.04,y-.1,h*1.80),(x+.02,y-.09,h*1.94),(x+.09,y,h*1.86)],'white',.015)
 else:
  rod('Palm trunk',(0,0,.08),(0,0,.88),.18,'tan',.13);feet('tan',.17,s=(.13,.18,.07))
  for z in (.23,.41,.59):ring('Trunk growth ring',(0,0,z),.175-(z-.23)*.08,.015,'brown',(0,0,0))
  for x,y,z in [(-.20,-.03,.88),(.20,-.03,.92),(0,.15,.96)]:
   sph('Coconut head',(x,y,z),(.17,.14,.19),'yellow')
   for s in (-1,1):sph('Coconut eye',(x+s*.052,y-.13,z+.055),(.018,.009,.028),'ink')
   tube('Coconut smile',[(x-.06,y-.14,z-.04),(x,y-.15,z-.07),(x+.06,y-.14,z-.04)],.008,'brown')
  for i in range(9):a=i*math.tau/9;leaf('Long palm frond',(0,0,1.12),(.64*math.cos(a),.53*math.sin(a),1.05),.14,'green')
@register('CUBONE MAROWAK',1.06)
def skull_dinosaurs(n):
 mature=n=='MAROWAK';sph('Brown body',(0,0,.36),(.23,.19,.28),'brown');sph('Cream belly',(0,-.17,.34),(.16,.04,.20),'cream');sph('Large skull helmet',(0,-.03,.76),(.27,.21,.23),'cream');sph('Projecting skull snout',(0,-.235,.69),(.20,.15,.095),'cream');feet('brown',.17);arms('brown',.42,.23)
 for s in (-1,1):
  sph('Deep skull eye socket',(s*.14,-.209,.81),(.075,.029,.083),'brown');sph('Eye in socket',(s*.14,-.236,.81),(.025,.010,.035),'ink');rod('Skull horn',(s*.16,.02,.92),(s*.23,.09,1.11 if mature else 1.04),.063,'cream',.002);sph('Skull nostril',(s*.065,-.379,.73),(.021,.01,.016),'brown')
 rod('Bone club',(.37,-.12,.21),(.44,-.15,.71),.038,'cream')
 for s in (-1,1):sph('Bone club end',(.44+s*.045,-.15,.73),(.059,.052,.059),'cream')
 tube('Pointed brown tail',[(0,.16,.21),(0,.36,.24),(0,.47,.39)],[.09,.065,.002],'brown')
@register('LICKITUNG',1.06)
def lickitung(n):
 sph('Big pink body',(0,0,.40),(.32,.24,.33),'pink');sph('Cream belly',(0,-.215,.36),(.23,.05,.23),'cream');sph('Head',(0,-.035,.79),(.26,.22,.22),'pink');eyes(.88,-.22,.13,.027,white=False);sph('Wide mouth',(0,-.237,.72),(.17,.03,.07),'rose');tube('Long curled tongue',[(0,-.27,.72),(0,-.49,.65),(.05,-.70,.66),(.13,-.74,.72)],[.075,.073,.070,.045],'coral');feet('pink',.21);arms('pink',.50,.31);tube('Heavy tail',[(0,.17,.24),(0,.41,.29),(0,.56,.52)],[.15,.12,.055],'pink')
 for z in (.25,.35,.45):tube('Belly stripe',[(-.16,-.252,z),(0,-.27,z-.02),(.16,-.252,z)],.011,'pink')
@register('KOFFING WEEZING',1.05)
def gas_balls(n):
 pair=n=='WEEZING';balls=[(-.12,0,.47,.31),(.32,.07,.71,.20)] if pair else [(0,0,.43,.31)]
 for x,y,z,r in balls:
  sph('Poison gas sphere',(x,y,z),(r,r*.90,r),'purple')
  for i in range(8):a=i*math.tau/8;p=(x+r*.87*math.cos(a),y+.02,z+r*.87*math.sin(a));end=(x+r*1.20*math.cos(a),y+.02,z+r*1.20*math.sin(a));rod('Gas vent',p,end,r*.16,'purple',r*.12)
  for s in (-1,1):sph('Eye',(x+s*r*.42,y-r*.86,z+r*.22),(r*.16,r*.08,r*.16),'white');sph('Pupil',(x+s*r*.42,y-r*.945,z+r*.22),(r*.061,r*.029,r*.08),'ink')
  tube('Crooked smile',[(x-r*.42,y-r*.90,z-r*.08),(x,y-r*.96,z-r*.23),(x+r*.42,y-r*.90,z-r*.06)],r*.035,'ink');sph('Skull marking',(x,y-r*.82,z-r*.45),(r*.14,r*.018,r*.11),'cream')
  for s in (-1,1):rod('Crossbones',(x-s*r*.19,y-r*.8,z-r*.55),(x+s*r*.19,y-r*.8,z-r*.70),r*.026,'cream')
 if pair:tube('Connecting gas stalk',[(-.01,.07,.65),(.19,.08,.80),(.24,.08,.78)],.045,'purple')
@register('CHANSEY BLISSEY',1.12)
def egg_nurses(n):
 evolved=n=='BLISSEY';sph('Pink egg body',(0,0,.46),(.34,.25,.40),'pink');eyes(.68,-.214,.13,.024,white=False);mouth(-.249,.57,.065);feet('pink',.22);arms('pink',.47,.33);sph('Cream pouch',(0,-.228,.32),(.18,.057,.14),'cream');sph('Carried white egg',(0,-.278,.36),(.085,.064,.11),'white')
 for s in (-1,1):
  for i in range(3):leaf('Side head curl',(s*.25,0,.67),(s*.41,.02,.79-i*.10),.07,'pink')
 if evolved:
  for i in range(11):a=i*math.tau/11;sph('Soft white skirt scallop',(.29*math.cos(a),.23*math.sin(a),.26),(.08,.07,.07),'white')
  for x in (-.12,0,.12):sph('Fluffy crown curl',(x,0,.85),(.10,.09,.09),'pink')
@register('TANGELA',1.00)
def tangela(n):
 sph('Hidden dark body',(0,0,.38),(.24,.19,.27),'ink');feet('red',.17,s=(.13,.17,.08))
 for i in range(19):a=i*math.tau/19;x=.22*math.cos(a);z=.40+.22*math.sin(a);tube('Curling blue vine',[(x,.02,z),(x+.09*math.sin(a),-.18,z+.12),(x-.04,-.25,z+.04),(x-.11,-.22,z-.05)],.031,'blue')
 for s in (-1,1):sph('White eye',(s*.095,-.26,.48),(.055,.029,.072),'white');sph('Pupil',(s*.095,-.289,.48),(.022,.010,.034),'ink')
@register('KANGASKHAN',1.27)
def kangaskhan(n):
 sph('Large kangaroo torso',(0,0,.45),(.31,.23,.35),'brown');sph('Cream belly pouch',(0,-.19,.35),(.24,.08,.24),'cream');sph('Head',(0,-.03,.88),(.23,.19,.20),'brown');sph('Short muzzle',(0,-.19,.82),(.16,.10,.08),'cream');eyes(.93,-.208,.12,.031);feet('brown',.22,s=(.13,.22,.085));arms('brown',.60,.29)
 for s in (-1,1):leaf('Long ear',(s*.13,0,1.0),(s*.27,.025,1.23),.083,'brown');rod('Small skull horn',(s*.10,-.12,1.01),(s*.11,-.13,1.17),.04,'cream',.002)
 tube('Balancing tail',[(0,.18,.28),(0,.47,.24),(0,.72,.34)],[.17,.11,.015],'brown');sph('Baby body',(0,-.246,.36),(.10,.055,.11),'purple');sph('Baby head',(0,-.26,.50),(.12,.08,.11),'purple')
 for s in (-1,1):sph('Baby ear',(s*.09,-.23,.58),(.046,.028,.055),'purple');sph('Baby eye',(s*.04,-.335,.52),(.012,.007,.019),'ink');sph('Baby hand',(s*.11,-.28,.38),(.035,.025,.043),'purple')

@register('ABRA KADABRA ALAKAZAM DROWZEE HYPNO',1.12)
def psychic_humanoids(n):
 if n=='KADABRA':sculpt_kadabra();return
 tapir=n in ('DROWZEE','HYPNO');abra=n=='ABRA';final=n in ('ALAKAZAM','HYPNO');sph('Psychic torso',(0,0,.42),(.24,.19,.29),'yellow');sph('Brown waist',(0,0,.25),(.24,.18,.15),'brown');sph('Psychic head',(0,-.035,.80),(.23,.19,.20),'yellow');feet('yellow',.19,s=(.13,.19,.075));arms('yellow',.51,.24)
 if tapir:
  tube('Tapir nose',[(0,-.17,.80),(0,-.27,.74),(0,-.29,.60)],[.073,.061,.04],'yellow');eyes(.88,-.202,.13,.030)
  for s in (-1,1):sph('Round tapir ear',(s*.17,0,.96),(.08,.06,.09),'yellow')
  if final:
   for i in range(8):a=i*math.tau/8;leaf('White neck ruff',(0,-.01,.62),(.28*math.cos(a),.18*math.sin(a),.55),.09,'white')
   tube('Pendulum chain',[(.36,-.12,.44),(.39,-.14,.24),(.41,-.14,.13)],.006,'silver');ring('Hypnotic pendulum',(.41,-.14,.10),.053,.009,'silver')
 else:
  sph('Foxlike snout',(0,-.20,.72),(.14,.10,.075),'yellow')
  for s in (-1,1):
   leaf('Long angular ear',(s*.13,0,.92),(s*.29,.05,1.21),.11,'yellow');sph('Brown shoulder armor',(s*.22,.01,.57),(.11,.10,.08),'brown')
   if abra:tube('Closed meditation eye',[(s*.06,-.206,.84),(s*.16,-.175,.86)],.010,'brown')
   else:sph('White eye',(s*.12,-.206,.86),(.047,.018,.027),'white');sph('Eye pupil',(s*.12,-.224,.86),(.012,.006,.023),'ink');tube('Long curled moustache',[(s*.07,-.26,.72),(s*.25,-.29,.73),(s*.36,-.28,.64)],[.022,.021,.003],'cream')
  tube('Long psychic tail',[(0,.15,.25),(0,.40,.27),(.12,.55,.44),(.18,.56,.65)],[.10,.085,.063,.012],'yellow')
  if not abra:
   for s in ((-1,1) if final else (1,)):
    rod('Held spoon stem',(s*.37,-.13,.32),(s*.39,-.14,.60),.012,'silver');sph('Spoon bowl',(s*.40,-.14,.65),(.055,.021,.077),'silver')
   fin('Forehead red star',[(0,-.207,1.0),(-.025,-.213,.94),(-.07,-.202,.94),(-.033,-.215,.90),(-.05,-.21,.85),(0,-.223,.885),(.05,-.21,.85),(.033,-.215,.90),(.07,-.202,.94),(.025,-.213,.94)],'red',.006)
@register('MACHOP MACHOKE MACHAMP TYROGUE HITMONLEE HITMONCHAN HITMONTOP',1.18)
def martial_artists(n):
 gray=n.startswith('MACH');lee=n=='HITMONLEE';chan=n=='HITMONCHAN';top=n=='HITMONTOP';baby=n=='TYROGUE';c='stone' if gray else ('lilac' if baby else 'tan');sph('Athletic torso',(0,0,.62 if lee else .48),(.23,.17,.28),c);feet(c,.18,s=(.10,.17,.065))
 for s in (-1,1):
  tube('Powerful leg',[(s*.12,0,.47 if lee else .27),(s*.17,-.02,.27 if lee else .16),(s*.18,-.035,.08)],[.06,.055,.05] if lee else [.08,.072,.064],c)
  for z in ([.67,.43] if n=='MACHAMP' else [.60]):
   tube('Bent muscular arm',[(s*.20,0,z),(s*.34,-.035,z-.09),(s*.38,-.16,z+.05)],[.08,.085,.065],c);sph('Boxing glove' if chan else 'Clenched fist',(s*.39,-.18,z+.09),(.10,.085,.10),'red' if chan else c)
 if lee:
  sph('Oval headless upper body',(0,-.02,.73),(.24,.18,.22),'tan');eyes(.78,-.191,.12,.035)
  for s in (-1,1):
   for z in (.12,.20,.28,.36,.43):ring('Spring leg band',(s*.17,-.01,z),.068,.010,'brown',(0,0,0))
 else:
  sph('Fighter head',(0,-.02,.86),(.20,.17,.19),c);eyes(.91,-.179,.11,.028);mouth(-.196,.79,.09)
  if gray:
   for i in (-1,0,1):leaf('Three cranial ridges',(i*.075,0,1.00),(i*.09,.065,1.22-.04*abs(i)),.035,'brown')
  elif top:ico('Conical spinning head cap',(0,.015,1.04),(.24,.19,.18),'brown',1);rod('Head spin point',(0,.015,1.16),(0,.015,1.35),.07,'brown',.001)
  elif baby:
   for i in (-1,0,1):leaf('Three scalp spikes',(i*.06,0,1.0),(i*.085,0,1.18),.04,'lilac')
  else:
   for i in range(5):leaf('Crownlike hair',(i*.05-.10,0,1.0),(i*.075-.15,.035,1.17),.038,'tan')
 if n in ('MACHOKE','MACHAMP'):
  sph('Black briefs',(0,0,.26),(.235,.18,.12),'ink');ring('Championship belt',(0,0,.35),.207,.029,'gold',(0,0,0));sph('Gold buckle',(0,-.193,.35),(.065,.026,.057),'gold')
 if chan:
  for i in range(8):a=i*math.tau/8;leaf('Purple tunic hem',(0,0,.35),(.28*math.cos(a),.20*math.sin(a),.21),.075,'purple')
 if baby:sph('Brown shorts',(0,0,.25),(.22,.16,.12),'brown')
@register('HORSEA SEADRA KINGDRA',1.07)
def seahorses(n):
 stage=['HORSEA','SEADRA','KINGDRA'].index(n);c='blue';sph('Curved seahorse body',(0,0,.41),(.17,.14,.23),c);sph('Cream belly',(0,-.118,.39),(.115,.034,.17),'cream');tube('Long curved neck',[(0,.01,.50),(0,-.015,.70),(0,-.04,.84)],.078,c);sph('Seahorse head',(0,-.05,.85),(.18,.15,.15),c);rod('Long tubular snout',(0,-.15,.81),(0,-.41 if stage==2 else -.32,.80),.063,c,.045);sph('Dark snout opening',(0,-.421 if stage==2 else -.332,.80),(.032,.010,.032),'navy');eyes(.91,-.179,.10,.027)
 tube('Curled prehensile tail',[(0,0,.22),(0,-.08,.09),(.12,-.13,.07),(.23,-.09,.14),(.19,-.05,.23),(.12,-.06,.22)],[.065,.055,.045,.035,.025,.014],c)
 for s in (-1,1):
  fin('Seahorse side fin',[(s*.11,.05,.51),(s*.36,.07,.72),(s*.34,.08,.40),(s*.16,.06,.33)],'cream' if stage==0 else 'blue',.02)
  if stage:
   for z in (.34,.48,.62):rod('Body needle',(s*.13,.02,z),(s*.30,.025,z+.08),.044,c,.002)
  leaf('Ear crest',(s*.11,.01,.95),(s*.25,.07,1.12 if stage else 1.01),.065,c)
 if stage==2:tube('Long crown horn',[(0,.02,.98),(0,.09,1.18),(.03,.14,1.28)],[.06,.037,.004],c)
@register('GOLDEEN SEAKING MAGIKARP REMORAID OCTILLERY',1.00)
def fishes(n):
 octo=n=='OCTILLERY';carp=n=='MAGIKARP';gun=n=='REMORAID';c='red' if octo else ('blue' if gun else ('orange' if n=='SEAKING' else ('red' if carp else 'white')))
 if octo:
  sph('Round octopus mantle',(0,.025,.53),(.27,.25,.30),'red');eyes(.58,-.223,.14,.035);rod('Siphon mouth',(0,-.21,.46),(0,-.36,.43),.079,'red',.07);sph('Dark siphon opening',(0,-.373,.43),(.048,.014,.046),'rose')
  for i in range(8):a=i*math.tau/8;pts=[(.16*math.cos(a),.16*math.sin(a),.31),(.30*math.cos(a),.30*math.sin(a),.12),(.47*math.cos(a),.47*math.sin(a),.07),(.49*math.cos(a),.49*math.sin(a),.19)];tube('Curled octopus arm',pts,[.061,.047,.031,.016],'red');sph('Yellow suction cup',(.34*math.cos(a),.29*math.sin(a),.10),(.040,.039,.022),'yellow')
  for x,y in [(-.10,-.04),(.09,.06)]:sph('Yellow head patch',(x,y,.804),(.063,.07,.023),'yellow')
  return
 sph('Fish body',(0,0,.40),(.22,.34,.23),c);eyes(.45,-.308,.115,.043);ring('Round fish mouth',(0,-.356,.36),.055,.015,'cream');sph('Mouth opening',(0,-.357,.36),(.041,.01,.041),'ink')
 for s in (-1,1):
  fin('Fanlike side fin',[(s*.17,-.05,.41),(s*.40,.00,.54),(s*.41,.16,.25),(s*.20,.13,.31)],'white' if not gun else 'blue',.022)
  fin('Split caudal fin',[(0,.27,.41),(s*.23,.58,.67),(s*.29,.58,.23),(0,.39,.36)],'white' if not gun else 'blue',.021)
  if carp:tube('Long whisker',[(s*.08,-.33,.36),(s*.21,-.40,.28),(s*.24,-.34,.16)],.012,'yellow')
 fin('Upright dorsal fin',[(0,-.10,.60),(0,.03,.84),(0,.22,.74),(0,.27,.55)],'yellow' if carp else 'white',.020)
 if n in ('GOLDEEN','SEAKING'):
  rod('Forehead horn',(0,-.23,.56),(0,-.34,.85),.052,'cream',.002)
  for x,y,z in [(-.13,-.12,.55),(.14,.08,.51),(.19,.04,.38)]:sph('Contrasting fish marking',(x,y,z),(.06,.10,.04),'orange' if n=='GOLDEEN' else 'ink')
 if gun:
  for y in (-.08,.11):sph('Dark lateral bar',(.205,y,.42),(.014,.028,.105),'navy')
@register('STARMIE',1.03)
def starmie(n):
 for layer in (0,1):
  pts=[]
  for i in range(10):a=i*math.tau/10+layer*math.pi/5;r=.48 if i%2==0 else .19;pts.append((r*math.sin(a),layer*.08,.51+r*math.cos(a)))
  fin('Five armed star layer',pts,'purple',.065)
 ring('Golden jewel mount',(0,-.078,.51),.17,.035,'gold');ico('Red central gemstone',(0,-.102,.51),(.145,.055,.145),'red',1)
@register('MR__MIME JYNX SMOOCHUM ELECTABUZZ ELEKID MAGMAR MAGBY',1.17)
def expressive_humanoids(n):
 mime=n=='MR__MIME';ice=n in ('JYNX','SMOOCHUM');electric=n in ('ELECTABUZZ','ELEKID');baby=n in ('SMOOCHUM','ELEKID','MAGBY');c='purple' if ice else ('yellow' if electric else ('red' if not mime else 'pink'));sph('Torso',(0,0,.40),(.23,.19,.28),c);sph('Head',(0,-.025,.81),(.24,.20,.22),c);eyes(.87,-.20,.125,.04,'ink');feet('blue' if mime else c,.18,s=(.12,.17,.06));arms(c,.50,.24)
 if mime:
  for s in (-1,1):
   sph('Red shoulder joint',(s*.24,0,.58),(.076,.07,.074),'red');sph('White open palm',(s*.38,-.15,.40),(.105,.055,.095),'white')
   for i in range(4):rod('Spread gloved finger',(s*.38+(i-1.5)*.045,-.15,.44),(s*.38+(i-1.5)*.056,-.17,.57),.018,'white')
   leaf('Blue head horn',(s*.15,0,.96),(s*.41,.05,1.12),.085,'blue');sph('Rosy cheek',(s*.18,-.176,.74),(.051,.025,.05),'red')
  sph('Red chest sphere',(0,-.181,.46),(.079,.029,.087),'red');mouth(-.217,.725,.09)
 elif ice:
  sph('Blonde hair cap',(0,.015,.96),(.27,.20,.12),'yellow')
  for s in (-1,1):sph('Long blonde hair',(s*.21,.04,.69),(.10,.15,.31 if not baby else .17),'yellow')
  sph('Full pink lips',(0,-.232,.72),(.105,.030,.041),'pink')
  if not baby:
   for i in range(10):a=i*math.tau/10;leaf('Long red gown panel',(0,0,.41),(.35*math.cos(a),.29*math.sin(a),.03),.11,'red')
 elif electric:
  for s in (-1,1):
   if baby:rod('Plug prong',(s*.13,0,.97),(s*.15,.015,1.22),.060,'yellow');rod('Plug black end',(s*.15,.015,1.17),(s*.15,.015,1.25),.062,'ink')
   else:tube('Head antenna',[(s*.13,0,.98),(s*.18,0,1.15)],.023,'ink');sph('Antenna bead',(s*.18,0,1.17),(.04,.038,.047),'yellow')
   for z in (.30,.48):fin('Black body stripe',[(s*.04,-.185,z),(s*.20,-.15,z+.04),(s*.17,-.176,z-.045)],'ink',.010)
  fin('Chest lightning mark',[(-.06,-.189,.61),(.04,-.2,.61),(0,-.208,.51),(.08,-.206,.53),(-.03,-.21,.40),(-.01,-.204,.52),(-.09,-.196,.52)],'ink',.008)
  if not baby:tube('Long striped tail',[(0,.14,.24),(0,.42,.29),(.18,.54,.47)],.048,'yellow')
 else:
  sph('Ducklike yellow bill',(0,-.237,.73),(.16,.09,.07),'yellow');sph('Yellow belly',(0,-.172,.37),(.15,.04,.20),'yellow')
  if baby:
   for x,z in [(-.12,1.0),(0,1.07),(.12,1.0)]:sph('Rounded head bump',(x,0,z),(.086,.076,.085),'red')
  else:
   for s in (-1,1):flame('Twin crown flames',(s*.12,0,.94),.28);flame('Shoulder flame',(s*.27,.03,.62),.20)
  tube('Fire tail',[(0,.15,.22),(0,.39,.26),(0,.54,.46)],.057,'red');flame('Tail flame',(0,.54,.44),.24)
@register('PORYGON PORYGON2',1.05)
def virtual_ducks(n):
 smooth=n=='PORYGON2';fn=sph if smooth else ico;fn('Red geometric body',(0,.02,.36),(.25,.26,.23),'red');fn('Long neck',(0,-.09,.58),(.13,.14,.22),'red');fn('Red geometric head',(0,-.13,.80),(.23,.19,.18),'red');fn('Long blue bill',(0,-.36,.76),(.15,.22,.08),'blue')
 for s in (-1,1):
  fn('Paddle foot',(s*.21,-.03,.12),(.15,.21,.06),'blue');sph('White side eye',(s*.18,-.244,.83),(.065,.039,.077),'white');sph('Black pupil',(s*.185,-.280,.83),(.024,.011,.039),'ink')
 fn('Upright blue tail',(0,.32,.42),(.12,.17,.26),'blue')
@register('OMANYTE OMASTAR KABUTO KABUTOPS',1.08)
def fossils(n):
 spiral_shell=n.startswith('OMA');mature=n in ('OMASTAR','KABUTOPS')
 if spiral_shell:
  sph('Blue mollusk',(0,-.10,.31),(.23,.22,.18),'blue');sph('Cream spiral shell',(0,.06,.55),(.29,.25,.31),'cream');spiral('Raised shell coil',(0,-.20,.57),.24,'brown',1.8,.015);eyes(.38,-.30,.13,.045)
  for i in range(8):a=i*math.tau/8;tube('Blue tentacle',[(.16*math.cos(a),-.11+.13*math.sin(a),.23),(.29*math.cos(a),-.11+.23*math.sin(a),.12),(.38*math.cos(a),-.11+.31*math.sin(a),.11)],[.046,.037,.023],'blue')
  if mature:
   for i in range(7):a=i*math.tau/7;rod('Shell thorn',(.23*math.cos(a),.05,.56+.23*math.sin(a)),(.38*math.cos(a),.055,.56+.39*math.sin(a)),.058,'cream',.002)
 else:
  if not mature:
   sph('Domed brown shell',(0,0,.26),(.35,.29,.22),'brown');sph('Black underside',(0,-.04,.13),(.31,.26,.08),'ink')
   for s in (-1,1):
    sph('Hidden red eye',(s*.15,-.26,.16),(.044,.023,.038),'red')
    for y in (-.14,.12):rod('Small claw leg',(s*.24,y,.16),(s*.36,y-.08,.02),.039,'cream',.006)
  else:
   sph('Brown torso',(0,0,.42),(.20,.15,.26),'brown');sph('Broad head',(0,-.03,.85),(.32,.18,.14),'brown');feet('brown',.16)
   for s in (-1,1):
    leaf('Swept head rim',(s*.12,-.01,.86),(s*.46,.03,1.05),.105,'brown');sph('Eye',(s*.17,-.179,.84),(.039,.014,.031),'white');sph('Pupil',(s*.17,-.193,.84),(.012,.007,.019),'ink');rod('Scythe arm',(s*.16,0,.60),(s*.31,-.05,.52),.044,'brown');fin('Long fossil scythe',[(s*.31,-.05,.55),(s*.48,-.08,.86),(s*.59,-.12,.53),(s*.48,-.14,.27),(s*.44,-.10,.54)],'cream',.04)
   for z in (.31,.43,.54):tube('Cream rib plate',[(-.14,-.12,z),(0,-.166,z-.024),(.14,-.12,z)],.023,'cream')
@register('AERODACTYL DRAGONITE',1.30)
def wing_dragons(n):
 aero=n=='AERODACTYL';c='stone' if aero else 'orange';sph('Dragon body',(0,0,.43),(.24,.21,.34),c);sph('Pale belly',(0,-.183,.42),(.17,.045,.25),'cream');rod('Dragon neck',(0,0,.66),(0,-.05,.90),.11,c);sph('Dragon head',(0,-.065,1.0),(.22,.18,.18),c);sph('Long muzzle',(0,-.25,.945),(.16,.16 if aero else .10,.085),c);eyes(1.04,-.224,.115,.032);feet(c,.18,s=(.12,.20,.07));arms(c,.60,.24);tube('Thick dragon tail',[(0,.15,.25),(0,.42,.25),(.16,.58,.37),(.23,.63,.56)],[.13,.09,.05,.006],c)
 for s in (-1,1):
  pts=[(s*.16,.07,.67),(s*.48,.10,.92),(s*.78,.10,1.22),(s*.80,.14,.51),(s*.53,.12,.60),(s*.30,.09,.40)] if aero else [(s*.17,.10,.75),(s*.40,.15,1.0),(s*.59,.20,.91),(s*.61,.21,.60),(s*.30,.13,.48)]
  fin('Stretched wing membrane',pts,'rose' if aero else 'teal',.025);tube('Wing leading bones',pts[:3],.028,c)
  for p in pts[3:]:rod('Wing supporting rib',pts[1],p,.014,c)
  if aero:leaf('Skull horn',(s*.10,0,1.12),(s*.25,.03,1.34),.07,c);rod('Fang',(s*.07,-.36,.925),(s*.07,-.37,.84),.023,'white',.001)
  else:tube('Curled antenna',[(s*.10,0,1.13),(s*.16,.035,1.36),(s*.22,.055,1.35)],.021,'cream')
@register('ARTICUNO ZAPDOS MOLTRES',1.28)
def legendary_birds(n):
 ice=n=='ARTICUNO';fire=n=='MOLTRES';c='blue' if ice else ('yellow' if not fire else 'orange');sph('Legendary bird body',(0,0,.45),(.24,.20,.30),c);sph('Pale breast',(0,-.177,.45),(.19,.046,.22),'white' if ice else 'yellow');rod('Neck',(0,0,.66),(0,-.02,.85),.08,c);sph('Bird head',(0,-.04,.93),(.18,.15,.17),c);eyes(.97,-.18,.095,.029);rod('Sharp beak',(0,-.18,.91),(0,-.40,.88),.052,'gold',.001)
 for s in (-1,1):
  rod('Bird leg',(s*.12,0,.25),(s*.17,-.04,.055),.024,'brown')
  for dx in (-.04,0,.04):rod('Talon',(s*.17,-.04,.055),(s*.17+dx,-.19,.02),.012,'brown')
  fin('Long spread wing',[(s*.16,.025,.66),(s*.55,.05,.96),(s*.85,.08,1.08),(s*.69,.06,.54),(s*.25,.01,.35)],c,.045)
  for i in range(6):leaf('Long distinct flight feather',(s*(.26+i*.08),.014,.69+i*.035),(s*(.46+i*.065),-.005,.29+i*.10),.055,'navy' if not fire and not ice and i%2 else c)
  if fire:
   for i in range(3):flame('Wing flame',(s*(.37+i*.16),.04,.62+i*.10),.28)
 if ice:
  for s in (-1,0,1):leaf('Ice crown plume',(s*.05,0,1.04),(s*.14,.045,1.30),.055,'blue')
  for s in (-1,1):tube('Long trailing ribbon tail',[(s*.055,.14,.31),(s*.13,.42,.22),(s*.21,.65,.16),(s*.31,.89,.35)],[.06,.055,.045,.015],'blue')
 elif fire:flame('Crown flame',(0,0,1.03),.40);flame('Long tail flames',(0,.35,.26),.59)
 else:
  for i in range(5):leaf('Spiky lightning crest',((i-2)*.03,0,1.04),((i-2)*.12,.04,1.32-.07*abs(i-2)),.045,'yellow')
  for i in range(4):leaf('Lightning tail feather',(0,.12,.33),((i-1.5)*.16,.61,.28),.10,'yellow' if i%2 else 'ink')
@register('MEW MEWTWO',1.26)
def psychic_cats(n):
 large=n=='MEWTWO';c='lilac' if large else 'pink';sph('Slender torso',(0,0,.45),(.20,.15,.27),c);rod('Slender neck',(0,0,.66),(0,-.015,.82),.07,c);sph('Large feline head',(0,-.025,.92),(.23,.18,.21),c);eyes(.98,-.184,.12,.05,'purple' if large else 'blue');sph('Short muzzle',(0,-.19,.85),(.105,.07,.048),c);feet(c,.18,s=(.12,.22 if large else .17,.07));arms(c,.56,.20)
 for s in (-1,1):
  leaf('Pointed cat ear',(s*.13,0,1.07),(s*.23,.03,1.28),.084,c)
  if large:sph('Powerful thigh',(s*.18,.015,.28),(.145,.17,.22),c)
  for i in (-1,0,1):sph('Rounded finger',(s*.33+i*.027,-.16,.39),(.020,.028,.033),c)
 tube('Long curved psychic tail',[(0,.14,.28),(0,.40,.29),(.24,.63,.52),(.43,.48,.89),(.39,.27,1.14)],[.115 if large else .04,.10 if large else .035,.065 if large else .025,.05 if large else .018,.03 if large else .014],'purple' if large else 'pink')
 if large:tube('Back neck tube',[(0,.11,.73),(0,.24,.85),(0,.13,1.03)],.036,'lilac')

# All 220 complementary exact species have explicit authored builders.
HEIGHTS.update({
 'mew':.78,'mewtwo':1.45,'dragonite':1.40,'aerodactyl':1.30,
 'quilava':.96,'typhlosion':1.38,'croconaw':1.06,'feraligatr':1.48,
 'bayleef':1.15,'meganium':1.48,'furret':1.20,'noctowl':1.20,
 'caterpie':.62,'weedle':.58,'kakuna':.80,'metapod':.80,
 'mareep':.85,'flaaffy':1.0,'ampharos':1.35,'wooper':.70,'quagsire':1.13,
 'pichu':.62,'cleffa':.60,'igglybuff':.58,'togepi':.67,'tyrogue':.80,
 'smoochum':.66,'magby':.70,'elekid':.70,'vulpix':.84,'ninetales':1.30,
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
 data={'name':name,'version':1,'coordinate_system':'+Y up; front +Z; floor-centered root','primitives':prims};(OUT/(name+'.mesh.json')).write_text(json.dumps(data,separators=(',',':')))
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
