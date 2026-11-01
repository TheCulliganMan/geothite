#!/usr/bin/env python3
"""Original faceted Magnet Train station shell, in source-pixel dimensions.

python tools/build-train-station.py OUTPUT --runtime-only
blender -b --threads 2 --python tools/build-train-station.py -- OUTPUT

No source art, game pixels, collision or scripts are imported. The boarding openings remain clear below their continuous curved roof.
Flush plug-door leaves are parked beside the openings in the station pose.
"""
from pathlib import Path
import importlib.util,math,sys
spec=importlib.util.spec_from_file_location('train_common',Path(__file__).with_name('build-cable-club.py'))
common=importlib.util.module_from_spec(spec);spec.loader.exec_module(common)
common.PALETTE.update({
 'train_ivory':(.84,.865,.81,1),'train_crown':(.94,.94,.86,1),
 'train_shadow':(.49,.60,.61,1),'train_band':(.54,.20,.18,1),
 'train_glass':(.055,.16,.21,1),'train_reflection':(.27,.48,.54,1),
 'train_steel':(.39,.46,.46,1),'train_dark':(.10,.15,.17,1),
 'train_marker':(.95,.82,.46,1),
})

def solid(a,label,vertices,faces,mat):
 # Orient every planar convex face away from the volume centroid.
 center=tuple(sum(v[i]for v in vertices)/len(vertices)for i in range(3));out=[]
 for face in faces:
  pts=[vertices[i]for i in face];n=common.normal(*pts[:3]);c=tuple(sum(v[i]for v in pts)/len(pts)for i in range(3))
  if sum(n[i]*(c[i]-center[i])for i in range(3))<0:face=tuple(reversed(face))
  out.append(face)
 a.parts.append((label,mat,vertices,out))

def tube_ring():
 return [(14+12*math.sin(i*math.tau/32),16+15.35*math.cos(i*math.tau/32)) for i in range(32)]

def clip_ring(ring,axis,limit,below=True):
 out=[]
 for a,b in zip(ring,ring[1:]+ring[:1]):
  inside=lambda v:(v[axis]<=limit) if below else (v[axis]>=limit)
  if inside(a):out.append(a)
  if inside(a)!=inside(b):
   t=(limit-a[axis])/(b[axis]-a[axis]);out.append(tuple(a[j]+t*(b[j]-a[j]) for j in range(2)))
 clean=[]
 for v in out:
  if not clean or math.dist(v,clean[-1])>1e-8:clean.append(v)
 if math.dist(clean[0],clean[-1])<1e-8:clean.pop()
 return clean

def loft(a,label,stations,mat,ring=None):
 ring=tube_ring() if ring is None else ring
 for i,((x0,scale0),(x1,scale1))in enumerate(zip(stations,stations[1:])):
  vs=[(x,14+(y-14)*sy,16+(z-16)*sz)for x,(sy,sz)in [(x0,scale0),(x1,scale1)]for y,z in ring]
  n=len(ring);faces=[tuple(range(n)),tuple(range(n,2*n))]+[(j,(j+1)%n,(j+1)%n+n,j+n)for j in range(n)]
  solid(a,label+' folded segment '+str(i+1),vs,faces,mat)

def plate(a,label,x0,x1,z0,z1,y0,y1,mat,corner=.3):a.prism(label,(x0,x1,max(0,z0),min(32,z1)),y0,y1,mat,corner)

def shell_skin(a,label,x0,x1,y0,y1,side,mat,offset=.10):
 # Follow the tube instead of floating flat rectangles in front of its curve.
 def point(x,y,depth):
  stops=[(0,.18,.12),(5,.38,.34),(13,.72,.70),(23,.96,.96),(30,1,1)]
  q=min(x,160-x);sy=sz=1
  for (xa,ya,za),(xb,yb,zb) in zip(stops,stops[1:]):
   if xa<=q<=xb:
    t=(q-xa)/(xb-xa);sy=ya+t*(yb-ya);sz=za+t*(zb-za);break
  z=15.35*sz*math.sqrt(max(0,1-((y-14)/(12*sy))**2))
  return (x,y,16+side*(z+depth))
 for i in range(4):
  lo=y0+(y1-y0)*i/4;hi=y0+(y1-y0)*(i+1)/4
  vs=[point(x,y,d) for d in (offset,offset+.08) for x,y in [(x0,lo),(x1,lo),(x1,hi),(x0,hi)]]
  # Triangulate the nonplanar skins explicitly; all component solids are closed.
  faces=[(0,2,1),(0,3,2),(4,5,6),(4,6,7),(0,1,5),(0,5,4),(1,2,6),(1,6,5),(2,3,7),(2,7,6),(3,0,4),(3,4,7)]
  if side<0:faces=[tuple(reversed(f)) for f in faces]
  a.parts.append((label,mat,vs,faces))

def make_train():
 a=common.Asset('magnet_train_shell')
 # One rounded envelope, with a continuous roof across both open portals.
 # These are presentation solids only: source door cells and warps stay live.
 loft(a,'West streamlined cab',[(0,(.18,.12)),(5,(.38,.34)),(13,(.72,.70)),(23,(.96,.96)),(30,(1,1))],'train_ivory')
 loft(a,'East streamlined cab',[(130,(1,1)),(137,(.96,.96)),(147,(.72,.70)),(155,(.38,.34)),(160,(.18,.12))],'train_ivory')
 lower=clip_ring(tube_ring(),0,21)
 roof=clip_ring(tube_ring(),0,21,False)
 loft(a,'Continuous tubular crown',[(30,(1,1)),(130,(1,1))],'train_crown',roof)
 for left,right in [(30,32),(48,112),(128,130)]:
  loft(a,'Long central passenger coach',[(left,(1,1)),(right,(1,1))],'train_ivory',lower)
 for x in [32,112]:
  loft(a,'Rear boarding envelope',[(x,(1,1)),(x+16,(1,1))],'train_ivory',clip_ring(lower,1,16))
  # Open double plug doors: each leaf slides sideways on the outer shell,
  # leaving the native 16-pixel portal clear rather than filling it with jambs.
  for leaf,(left,right) in enumerate([(x-7.8,x-.5),(x+16.5,x+23.8)]):
   shell_skin(a,'Sliding door leaf '+str(x)+' '+str(leaf),left,right,4,20.5,1,'train_ivory',.25)
   shell_skin(a,'Sliding door glass '+str(x)+' '+str(leaf),left+.8,right-.8,12,18,1,'train_glass',.38)
   shell_skin(a,'Sliding door seam '+str(x)+' '+str(leaf),left,left+.18,4.5,20,1,'train_dark',.38)
   shell_skin(a,'Sliding door belt '+str(x)+' '+str(leaf),left,right,7,9.1,1,'train_band',.38)
  plate(a,'Door sliding track '+str(x),x-8,x+24,28.5,28.7,21,21.35,'train_steel',.04)
 # Named physical skins, framed windows and service details on each side.
 for side,direction in [('Platform',1),('Track',-1)]:
  for left,right in [(20,31.2),(48.8,111.2),(128.8,140)]:
   shell_skin(a,side+' recessed lower red belt',left,right,7,9.1,direction,'train_band')
   shell_skin(a,side+' brushed lower edge',left,right,4.5,5.1,direction,'train_steel')
  for i,(left,right)in enumerate([(50.8,67.7),(70.0,88.1),(90.4,109.2)]):
   shell_skin(a,side+' window rubber reveal '+str(i),left,right,11.4,20.7,direction,'train_dark',.14)
   shell_skin(a,side+' inset blue window '+str(i),left+.55,right-.55,12,20.1,direction,'train_glass',.25)
   shell_skin(a,side+' folded sky reflection '+str(i),left+1,right-1,18.7,19.35,direction,'train_reflection',.36)
 # Cab windscreens on sloped front side: sealed glass wedge with a wide
 # angled lower edge gives each tapered end its own unmistakable silhouette.
 for side in [-1,1]:
  def sx(x):return x if side<0 else 160-x
  for k,(xa,xb,sa,sb)in enumerate([(6,13,(.4225,.385),(.72,.70)),(13,23,(.72,.70),(.96,.96))]):
   face=[(sx(x),14+(y-14)*sy,16+15.35*sz+.28)for x,sy,sz,y in [(xa,*sa,12),(xb,*sb,12),(xb,*sb,18),(xa,*sa,18)]]
   vs=face+[(x,y,z+.22)for x,y,z in face]
   solid(a,('West'if side<0 else'East')+' angled cab windshield '+str(k),vs,[(0,1,2,3),(4,7,6,5),(0,4,5,1),(1,5,6,2),(2,6,7,3),(3,7,4,0)],'train_glass')
  # End marker lenses and flush service access are small closed parts.
  x0,x1=sorted((sx(24.0),sx(28.0)))
  plate(a,('West'if side<0 else'East')+' amber cab marker',x0,x1,31.69,32,7.4,8.5,'train_marker',.25)
  x0,x1=sorted((sx(24.4),sx(29.5)))
  plate(a,('West'if side<0 else'East')+' service inset',x0,x1,31.7,32,9.5,12,'train_shadow',.25)
 # Four discrete suspended magnetic shoes; no conventional train wheels and
 # no platform slab. They are wholly outside either boarding corridor.
 for x in [17,53,99,136]:
  plate(a,'Suspended magnetic shoe '+str(x),x,x+7,6,27,0,3.3,'train_dark',.8)
  plate(a,'Silver guide shoe face '+str(x),x+.7,x+6.3,27,28,1,3.8,'train_steel',.22)
 # Thin seals sit outside the open portal; no full-depth upright pillars.
 for x in [31.65,48.0,111.65,128.0]:
  shell_skin(a,'Ivory boarding jamb '+str(x),x,x+.3,4,21,1,'train_shadow',.10)
 # Quiet folded roof strips, rooftop vents and an original center crest.
 for x in [58,91]:
  plate(a,'Low rooftop ventilation plinth '+str(x),x,x+11,9.5,17.5,25.4,26,'train_shadow',.6)
  for j in range(4):plate(a,'Roof vent folded blade '+str(x)+' '+str(j),x+1+j*2.5,x+2.5+j*2.5,10.5,16.5,25.95,26.2,'train_steel',.18)
 return a

def build_blender(a,out,skip_preview):
 import bpy
 from mathutils import Vector
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 scene=bpy.context.scene;mats={}
 for name,color in common.PALETTE.items():
  m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True;m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=color;m.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.72;mats[name]=m
 col=bpy.data.collections.new('ASSET | Original Magnet Train shell');scene.collection.children.link(col)
 for label,mat,vs,faces in a.parts:
  me=bpy.data.meshes.new(label);me.from_pydata([(x/16,-z/16,y/16)for x,y,z in vs],[],faces);me.update();me.materials.append(mats[mat]);ob=bpy.data.objects.new(label,me);col.objects.link(ob)
 scene['provenance']='Original authored closed geometry and colors. No source pixels, ROM, exported content, scripts or imported models.'
 scene['rebuild']='tools/build-train-station.py'
 scene['openings']='Runtime x32..48 and x112..128 remain open below y21; continuous curved roof and sideways sliding leaves preserve the tubular silhouette.'
 bpy.ops.mesh.primitive_plane_add(size=200,location=(5,-1,-.03));o=bpy.context.object;o.name='PREVIEW | Neutral studio floor';m=bpy.data.materials.new('Preview floor');m.diffuse_color=(.15,.20,.23,1);o.data.materials.append(m)
 scene.world.color=(.20,.23,.26)
 for name,loc,power,size in [('Key',(1,-6,10),1500,8),('Fill',(9,5,7),1000,7)]:
  bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name='PREVIEW | '+name;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector((5,-1,.7))-o.location).to_track_quat('-Z','Y').to_euler()
 bpy.ops.object.camera_add(location=(13,-13,9));cam=bpy.context.object;cam.name='PREVIEW | Camera';cam.data.type='ORTHO';cam.data.ortho_scale=12;scene.camera=cam
 cam.rotation_euler=(Vector((5,-1,.7))-cam.location).to_track_quat('-Z','Y').to_euler()
 scene.render.engine='CYCLES';scene.cycles.samples=24;scene.cycles.use_denoising=False;scene.render.threads_mode='FIXED';scene.render.threads=2;scene.render.resolution_x=1400;scene.render.resolution_y=650;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX'
 bpy.ops.wm.save_as_mainfile(filepath=str(out/'train-station.blend'),compress=False)
 if not skip_preview:
  for name,pos in [('front',(12,-13,9)),('rear',(-2,11,8)),('boarding',(5,-8,8))]:
   cam.location=pos;cam.rotation_euler=(Vector((5,-1,.7))-cam.location).to_track_quat('-Z','Y').to_euler();scene.render.filepath=str(out/('train-'+name+'.png'));bpy.ops.render.render(write_still=True)

def main():
 args=sys.argv[sys.argv.index('--')+1:]if '--'in sys.argv else sys.argv[1:];out=Path(next((x for x in args if not x.startswith('--')),'target/train-station'));out.mkdir(parents=True,exist_ok=True)
 a=make_train();common.save_runtime(a,out);print(a.name,common.document(a)['triangle_count'],'triangles',len(a.parts),'named parts',common.document(a)['bounds'])
 if '--runtime-only'not in args:build_blender(a,out,'--skip-preview'in args)
if __name__=='__main__':main()
