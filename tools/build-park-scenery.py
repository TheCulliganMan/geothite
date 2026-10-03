#!/usr/bin/env python3
"""Original editable low-poly park bins, pedestal fountains and pond rims.

blender -b --threads 2 --python tools/build-park-scenery.py -- target/park-scenery
No source pack, source images, collision data or animation frames are imported.
"""
import bpy,bmesh,json,math,sys
from pathlib import Path
from mathutils import Vector
args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
out=Path(next((a for a in args if not a.startswith('--')),'target/park-scenery'));out.mkdir(parents=True,exist_ok=True)
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
for m in list(bpy.data.materials):bpy.data.materials.remove(m)
scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=20;scene.cycles.use_denoising=False
scene.render.resolution_x=1440;scene.render.resolution_y=800;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX'
def material(name,c):
 m=bpy.data.materials.new(name);m.diffuse_color=(*c,1);m.use_nodes=True;bs=m.node_tree.nodes['Principled BSDF'];bs.inputs['Base Color'].default_value=(*c,1);bs.inputs['Roughness'].default_value=.78;return m
stone=material('Limestone | warm dressed faces',(.57,.61,.53));edge=material('Limestone | pale beveled edge',(.77,.79,.65));base=material('Limestone | dark wet footing',(.31,.39,.34));iron=material('Bin | blue green cast metal',(.24,.34,.32));ironlight=material('Bin | pale rim enamel',(.53,.63,.57));dark=material('Recess | deep cast shadow',(.075,.12,.11));brass=material('Tap | aged bronze',(.49,.47,.24));water=material('Pedestal basin | still ceramic inset',(.18,.36,.40))
assets=[];stats={}
def begin(name):
 global objects,collection
 objects=[];collection=bpy.data.collections.new('ASSET | '+name);scene.collection.children.link(collection)
def mesh(name,verts,faces,m):
 data=bpy.data.meshes.new(name);data.from_pydata([(x,-z,y)for x,y,z in verts],[],faces);data.update();o=bpy.data.objects.new(name,data);collection.objects.link(o);data.materials.append(m);objects.append(o);return o
def lathe(name,profile,m,segments=12,zscale=1):
 # A closed cross-section swept around the vertical axis; profile reversals
 # form genuine hollow walls with underside, inner wall and rim surfaces.
 verts=[(point[0]*math.cos(2*math.pi*k/segments),point[1],(point[2] if len(point)>2 else zscale*point[0])*math.sin(2*math.pi*k/segments))for point in profile for k in range(segments)]
 faces=[]
 for j in range(len(profile)):
  jj=(j+1)%len(profile)
  for k in range(segments):kk=(k+1)%segments;faces.append((j*segments+k,j*segments+kk,jj*segments+kk,jj*segments+k))
 return mesh(name,verts,faces,m)
def cylinder(name,r,y0,y1,m,segments=12,zscale=1):
 verts=[(r*math.cos(2*math.pi*k/segments),y,zscale*r*math.sin(2*math.pi*k/segments))for y in [y0,y1]for k in range(segments)]
 faces=[tuple(reversed(range(segments))),tuple(range(segments,segments*2))]
 faces.extend((k,(k+1)%segments,(k+1)%segments+segments,k+segments)for k in range(segments))
 return mesh(name,verts,faces,m)
def box(name,c,s,m):
 bpy.ops.mesh.primitive_cube_add(size=1,location=(c[0],-c[2],c[1]));o=bpy.context.object;o.scale=(s[0],s[2],s[1]);bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 for col in list(o.users_collection):col.objects.unlink(o)
 collection.objects.link(o);o.name=name;o.data.materials.append(m);objects.append(o);return o
def finish(name,metadata):
 groups={};lo=[float('inf')]*3;hi=[-float('inf')]*3;vol=0;tris=0
 for o in objects:
  bm=bmesh.new();bm.from_mesh(o.data);assert all(e.is_manifold for e in bm.edges),o.name;bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));v=bm.calc_volume(signed=True);assert v>1e-5,(o.name,v);vol+=v;bm.to_mesh(o.data);bm.free();o.data.calc_loop_triangles();tris+=len(o.data.loop_triangles)
  m=o.data.materials[0];g=groups.setdefault(m.name,dict(name=m.name,base_color=[round(v,6)for v in m.diffuse_color],positions=[],normals=[],indices=[],lookup={}))
  for t in o.data.loop_triangles:
   n=o.matrix_world.to_3x3()@t.normal;n.normalize();n=(round(n.x,6),round(n.z,6),round(-n.y,6))
   for index in t.vertices:
    p=o.matrix_world@o.data.vertices[index].co;p=(round(p.x,6),round(p.z,6),round(-p.y,6))
    for a in range(3):lo[a]=min(lo[a],p[a]);hi[a]=max(hi[a],p[a])
    key=p+n
    if key not in g['lookup']:g['lookup'][key]=len(g['positions'])//3;g['positions'].extend(p);g['normals'].extend(n)
    g['indices'].append(g['lookup'][key])
  o['Authorship']='Original faceted solid; no imported game art';o['Function']=metadata
 for g in groups.values():del g['lookup']
 doc=dict(name=name,coordinate_system='right-handed; +Y up; front +Z',bounds=dict(min=lo,max=hi),primitives=list(groups.values()))
 (out/(name+'.mesh.json')).write_text(json.dumps(doc,separators=(',',':'))+'\n');stats[name]=dict(triangles=tris,closed_components=len(objects),materials=len(groups),volume=vol,bounds=doc['bounds']);assets.append((name,list(objects)))
# A genuine open-top cylinder, its dark bottom seen through a heavy rolled rim.
begin('litter_bin')
cylinder('Foot | inset octagonal sole',5.25,0,.8,dark,12)
lathe('Body | tapered thick cast shell',[(5.05,.65),(5.70,1.0),(6.05,9.65),(5.75,10.4),(4.95,10.4),(4.65,1.1)],iron)
lathe('Lip | pale rolled open rim',[(5.65,9.7),(6.45,10.3),(6.45,11.1),(5.65,11.8),(4.8,11.8),(4.45,11.2),(4.45,10.45),(4.95,10.1)],ironlight)
cylinder('Interior | visible dark bottom',4.7,1.0,1.18,dark,12)
lathe('Foot | protective band',[(5.30,.60),(5.90,1.05),(5.93,1.80),(5.20,1.75)],base)
for angle in [math.pi/6,5*math.pi/6,3*math.pi/2]:
 o=box('Body | vertical cast seam',(5.66*math.cos(angle),5.0,5.66*math.sin(angle)),(.22,6.7,.18),ironlight);o.rotation_euler[2]=-angle
finish('litter_bin','One complete 2x2 park litter-bin drawing; adjacent plaza remains flat')
# The broad source fixture has a tiered masonry body and central central spout.
begin('pedestal_fountain')
cylinder('Plinth | twelve-sided foundation',10.2,0,.65,base,12,.62)
lathe('Plinth | chamfered lower course',[(10.2,.5),(10.2,1.1),(9.25,2.1),(6.7,2.1),(6.65,.6)],edge,12,.62)
lathe('Pedestal | recessed stone drum',[(7.1,1.7),(7.7,2.4),(7.7,6.1),(8.2,6.8),(6.4,6.8),(6.4,1.7)],stone,12,.64)
lathe('Bowl | thick faceted basin',[(8.1,6.1),(9.25,7.2),(9.25,8.1),(8.20,9.2),(6.5,9.2),(5.8,8.1),(5.1,7.15),(4.7,6.8)],edge,12,.70)
cylinder('Bowl | dark glazed inner well',5.8,7.06,7.24,water,12,.70)
cylinder('Tap | central mounting collar',1.45,7.24,8.1,brass,8,.78)
cylinder('Tap | short central bubbler',.72,8.1,9.8,brass,8,.8)
lathe('Tap | projecting nozzle lip',[(.7,9.4),(1.1,9.8),(1.1,10.05),(.7,10.3),(.40,10.3),(.4,9.4)],ironlight,8,.8)
finish('pedestal_fountain','Static 3x2 source stone pedestal fountain; no source inscription or animation is invented')
# Only masonry is authored for the pond fixture. The open center receives the
# existing live scene-atlas cap at y=5, over the unchanged water base at y=0.
begin('pond_basin')
lathe('Foot | submerged bevel',[(6.6,0),(7.04,.55),(7.04,1.25),(6.72,1.6),(5.50,1.6,3.8),(5.50,0,3.8)],base,24,.7386363636)
lathe('Wall | faceted limestone bowl',[(6.7,1.0),(7.04,1.7),(7.04,4.7),(6.82,5.2),(5.72,5.2,3.9),(5.55,4.75,3.8),(5.55,1.0,3.8)],stone,24,.7386363636)
# The inner aperture is 5.92x4px at its widest, exactly the unchanged live cap.
lathe('Rim | pale dressed coping',[(6.8,4.65),(7.04,5.05),(7.04,5.6),(6.65,6.0),(5.96,6.0,4.04),(5.92,5.30,4.0),(5.92,5.0,4.0),(6.45,4.65)],edge,24,.7386363636)
finish('pond_basin','Exact joined 3f+33 pond masonry only; runtime keeps the cap, spray cadence and source water')
(out/'asset-stats.json').write_text(json.dumps(stats,indent=2)+'\n')
# Separate preview copies leave all editable source components at their origin.
presentation=bpy.data.collections.new('PRESENTATION | non-exported');scene.collection.children.link(presentation)
for offset,(name,objs) in zip([-25,0,28],assets):
 for o in objs:o.hide_render=True
 for o in objs:
  c=o.copy();c.data=o.data;c.location.x+=offset;c.hide_render=False;presentation.objects.link(c)
bpy.ops.mesh.primitive_plane_add(size=200,location=(0,0,-.08));bpy.context.object.data.materials.append(material('Presentation | linen ground',(.42,.46,.40)))
world=bpy.data.worlds.new('Park workshop daylight');scene.world=world;world.use_nodes=True;world.node_tree.nodes['Background'].inputs['Color'].default_value=(.6,.66,.72,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.65
for name,pos,power,size in [('Key',(-24,-25,42),16000,30),('Fill',(32,20,30),10000,24)]:
 bpy.ops.object.light_add(type='AREA',location=pos);o=bpy.context.object;o.name=name;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector((0,0,5))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(24,-75,53));cam=bpy.context.object;cam.data.type='ORTHO';cam.data.ortho_scale=77;cam.rotation_euler=(Vector((0,0,5))-cam.location).to_track_quat('-Z','Y').to_euler();scene.camera=cam
scene['Read me']='Three original full-volume fixtures. Pond preview deliberately leaves live center empty. Runtime supplies real source animation, source water and unchanged datum.'
bpy.ops.wm.save_as_mainfile(filepath=str(out/'park-scenery.blend'),compress=False)
if '--skip-preview' not in args:
 scene.render.filepath=str(out/'park-front.png');bpy.ops.render.render(write_still=True)
 cam.location=(-25,73,44);cam.rotation_euler=(Vector((0,0,5))-cam.location).to_track_quat('-Z','Y').to_euler();scene.render.filepath=str(out/'park-rear.png');bpy.ops.render.render(write_still=True)
print('PARK_SCENERY='+json.dumps(stats))
