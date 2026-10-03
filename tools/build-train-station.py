#!/usr/bin/env python3
"""Original faceted Magnet Train station shell, in source-pixel dimensions.

python tools/build-train-station.py OUTPUT --runtime-only
blender -b --threads 2 --python tools/build-train-station.py -- OUTPUT

No source art, game pixels, collision or scripts are imported. The two boarding
recesses are empty at every height; live source door surfaces belong to runtime.
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

def loft(a,label,stations,mat):
 # Cross-sections follow a hand-folded octagonal coach, never a box extrusion.
 ring=[(2,4),(2,28),(6,31.35),(18,31.35),(24,26),(26,8),(21,.65),(6,.65)]
 for i,((x0,scale0),(x1,scale1))in enumerate(zip(stations,stations[1:])):
  vs=[(x,14+(y-14)*sy,16+(z-16)*sz)for x,(sy,sz)in [(x0,scale0),(x1,scale1)]for y,z in ring]
  n=8;faces=[tuple(range(n)),tuple(range(n,2*n))]+[(j,(j+1)%n,(j+1)%n+n,j+n)for j in range(n)]
  solid(a,label+' folded segment '+str(i+1),vs,faces,mat)

def plate(a,label,x0,x1,z0,z1,y0,y1,mat,corner=.3):a.prism(label,(x0,x1,max(0,z0),min(32,z1)),y0,y1,mat,corner)

def make_train():
 a=common.Asset('magnet_train_shell')
 # The two native 16x16 boarding footprints x32..48 and x112..128,
 # z16..32 are empty in all solids, including roof, frame and undercarriage.
 loft(a,'West streamlined cab',[(0,(.26,.20)),(6,(.60,.58)),(16,(.90,.87)),(25,(1,1)),(31.5,(1,1))],'train_ivory')
 loft(a,'East streamlined cab',[(128.5,(1,1)),(135,(1,1)),(144,(.90,.87)),(154,(.60,.58)),(160,(.26,.20))],'train_ivory')
 loft(a,'Long central passenger coach',[(48.5,(1,1)),(111.5,(1,1))],'train_ivory')
 # Rear half continues behind the boardable recess, with a distinct low roof
 # so the native door surfaces remain visible from the gameplay camera.
 for x in [32,112]:
  plate(a,'Recess rear structural sill',x,x+16,0,15.5,0,3,'train_dark',.5)
  plate(a,'Recess rear ivory coach',x,x+16,0,15.3,3,22,'train_ivory',.7)
  plate(a,'Recess rear chamfered crown',x,x+16,0,15.2,21.5,23.4,'train_crown',.8)
 # Named physical skins, framed windows and service details on each side.
 for side,z0,z1 in [('Platform',31.56,31.68),('Track',.32,.44)]:
  for left,right in [(20,31.2),(48.8,111.2),(128.8,140)]:
   plate(a,side+' recessed lower red belt',left,right,z0,z1,7.0,9.1,'train_band',.12)
   plate(a,side+' brushed lower edge',left,right,z0,z1,4.5,5.1,'train_steel',.08)
  for i,(left,right)in enumerate([(50.8,67.7),(70.0,88.1),(90.4,109.2)]):
   plate(a,side+' window rubber reveal '+str(i),left,right,31.69 if side=='Platform'else .23,31.77 if side=='Platform'else .31,11.4,20.7,'train_dark',.3)
   plate(a,side+' inset blue window '+str(i),left+.55,right-.55,31.78 if side=='Platform'else .11,31.89 if side=='Platform'else .22,12.0,20.1,'train_glass',.25)
   plate(a,side+' folded sky reflection '+str(i),left+1,right-1,31.90 if side=='Platform'else 0,32 if side=='Platform'else .10,18.7,19.35,'train_reflection',.12)
 # Cab windscreens on sloped front side: sealed glass wedge with a wide
 # angled lower edge gives each tapered end its own unmistakable silhouette.
 for side in [-1,1]:
  def sx(x):return x if side<0 else 160-x
  for k,(xa,xb,sa,sb)in enumerate([(6,16,(.60,.58),(.90,.87)),(16,23,(.90,.87),(.97778,.97111))]):
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
 # Boardable recess jambs stay outside the two 16-pixel source apertures.
 for x in [31.15,48.0,111.15,128.0]:
  plate(a,'Ivory boarding jamb '+str(x),x,x+.8,15.8,31.7,3.4,22,'train_shadow',.16)
  plate(a,'Boarding jamb bright edge '+str(x),x+.14,x+.65,31.6,32,4.0,21.5,'train_crown',.10)
 # Quiet folded roof strips, rooftop vents and an original center crest.
 for left,right in [(23.3,31.1),(49,111),(128.9,136.7)]:
  plate(a,'Crown highlight strip',left,right,9,23,24.6,26,'train_crown',.8)
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
 scene['openings']='Runtime x32..48 and x112..128, z16..32 remain completely empty at every height; source door UVs are retained by the world renderer.'
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
