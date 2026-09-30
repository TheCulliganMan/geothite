"""Original connected-Johto environment kit, no imported game artwork.
blender -b --python tools/build-connected-johto-models.py -- target/connected-johto-assets [--skip-preview]
Uses the New Bark kit's authored modeling vocabulary. Source Z-up/front -Y;
exports +Y-up/front +Z with a floor-centered origin and explicit threshold.
"""
from pathlib import Path
# Load reusable geometric authoring helpers without generating the original kit.
source = Path(__file__).with_name('build-new-bark-models.py').read_text()
exec(compile(source.split('\nmake_house();')[0], 'new_bark_modeling_vocabulary', 'exec'))
from mathutils import Matrix
for n,c in {'roof_blue':(.18,.34,.55),'roof_blue_light':(.31,.48,.69),'roof_blue_dark':(.11,.22,.37),'roof_blue_warm':(.23,.40,.61),'violet':(.38,.29,.45),'violet_light':(.53,.44,.59),'violet_dark':(.24,.19,.32),'violet_warm':(.44,.34,.49)}.items():
 m=bpy.data.materials.new(n);m.diffuse_color=(*c,1);m.use_nodes=True
 m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(*c,1)
 m.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.86
 MATS[n]=m

def beam(n,a,b,w,d,mat):
 a=Vector(a);b=Vector(b);o=cube(n,(a+b)/2,(w,d,(b-a).length),mat,.012)
 o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o

def medallion(y,z,palette='roof_red'):
 # A carved circular healing emblem, assembled from original solid geometry.
 o=cone('Round service insignia',(0,y,z),.265,.265,.075,'plaster_light',20);o.rotation_euler.x=math.pi/2
 o=cone('Inset service insignia',(0,y-.045,z),.214,.214,.026,palette,20);o.rotation_euler.x=math.pi/2
 facade_cube('Insignia horizontal band',0,z,.43,.060,y-.065,.024,'plaster_light')
 o=cone('Insignia round center',(0,y-.083,z),.087,.087,.027,'plaster_light',14);o.rotation_euler.x=math.pi/2
 o=cone('Insignia center inset',(0,y-.101,z),.040,.040,.015,palette,12);o.rotation_euler.x=math.pi/2

def service_building(kind):
 begin(kind);palette='roof_red' if kind=='pokecenter' else 'roof_blue'
 foundation(3.67,3.37)
 cube('Cream civic masonry',(0,0,1.35),(3.5,3.21,2.22),'plaster_light',.045)
 building_trim(3.51,3.22,1.37,2.14)
 for y in [-1.614,1.614]:
  cube('Terracotta or blue base course',(0,y,.54),(3.5,.065,.20),palette)
  cube('Pale upper string course',(0,y,2.34),(3.61,.11,.14),'plaster')
 for x in [-1.765,1.765]:cube('Side colored base course',(x,0,.54),(.065,3.20,.20),palette)
 for xx in [-1.751,1.751]:mesh('Pale civic gable',[(xx,-1.605,2.42),(xx,1.605,2.42),(xx,0,3.28)],[(0,1,2)] if xx<0 else [(2,1,0)],'plaster')
 roof(3.96,3.70,2.46,3.35,palette,6)
 fy=-1.615;door(fy-.03,z=1.16,w=1.02,h=1.75,lab=True)
 for x in [-1.13,1.13]:window(x,1.34,fy,w=.68,h=1.0)
 for a in [-90,90]:
  for x in [-.85,.85]:window(x,1.44,-1.763,w=.75,h=.93,angle=a)
 for x in [-1.08,0,1.08]:window(x,1.42,fy,w=.73,h=.88,angle=180)
 cube('Entrance landing',(0,-1.955,.11),(1.37,.55,.22),'stone_light',.025)
 cube('Threshold front edge',(0,-2.241,.092),(1.20,.026,.09),'stone')
 if kind=='pokecenter':
  cube('Red entry shelter',(0,-1.87,2.16),(1.40,.66,.15),palette,.04)
  cube('Entry shelter cream edge',(0,-2.186,2.135),(1.44,.06,.09),'plaster_light',.014)
  # Small raised badge sits on a broad readable panel instead of tiny raster text.
  facade_cube('Center badge mounting',0,2.58,.82,.57,fy-.14,.14,palette+'_dark',bevel=.035)
  medallion(fy-.24,2.60,palette)
  for x in [-.70,.70]:
   facade_cube('Warm entrance lamp',x,1.87,.095,.16,fy-.095,.12,'yellow',bevel=.015)
 else:
  # The shop's striped awning, recessed glazing and hanging plaque distinguish it.
  for i in range(9):
   x=-1.62+i*.405
   o=cube('Tailored striped shop awning',(x,-1.86,2.105),(.402,.58,.068),palette if i%2==0 else 'plaster_light',.008);o.rotation_euler.x=.14
   cube('Scalloped awning valance',(x,-2.139,2.034),(.402,.035,.17),palette if i%2==0 else 'plaster_light',.022)
  facade_cube('Shop signboard',0,2.64,1.17,.36,fy-.12,.12,palette+'_dark',bevel=.025)
  # Clear geometric M, large enough to read at the actual camera.
  for a,b in [((-.26,-1.81,2.51),(-.26,-1.81,2.76)),((-.26,-1.81,2.76),(0,-1.81,2.58)),((0,-1.81,2.58),(.26,-1.81,2.76)),((.26,-1.81,2.76),(.26,-1.81,2.51))]:beam('Carved shop letter',a,b,.055,.035,'cream')
 # A restrained roof vent gives each roof a readable scale.
 cube('Roof vent base',(.99,.65,2.98),(.44,.43,.24),'plaster_shadow',.018)
 cube('Roof vent cap',(.99,.65,3.14),(.53,.52,.11),palette+'_dark',.022)


def make_gate():
 begin('route_gate');foundation(3.72,3.15)
 cube('Gatehouse ochre plaster',(0,0,1.45),(3.54,2.98,2.48),'plaster',.035)
 building_trim(3.56,3.0,1.45,2.42)
 roof(3.96,3.64,2.72,3.50,'roof_teal',6)
 fy=-1.50
 facade_cube('Deep gateway recess',0,1.25,1.38,2.05,fy-.02,.15,'timber',bevel=.022)
 facade_cube('Gateway blue interior',0,1.25,1.15,1.91,fy-.105,.02,'roof_teal_dark')
 for x in [-.65,.65]:facade_cube('Gate opening carved post',x,1.25,.13,2.18,fy-.135,.13,'plaster_light',bevel=.015)
 facade_cube('Gate opening lintel',0,2.29,1.43,.15,fy-.13,.14,'plaster_light',bevel=.017)
 cube('Gate threshold',(0,-1.84,.10),(1.52,.72,.20),'stone_light',.023)
 for x in [-1.30,1.30]:window(x,1.51,fy,w=.46,h=.82)
 for a in [-90,90]:window(0,1.48,-1.783,w=1.16,h=.88,angle=a)
 for x in [-1.1,1.1]:window(x,1.54,fy,w=.68,h=.80,angle=180)
 facade_cube('Route wayfinding board',0,2.51,.76,.24,fy-.13,.09,'roof_teal_dark',bevel=.016)
 for x in [-.23,0,.23]:facade_cube('Route board studs',x,2.51,.072,.072,fy-.19,.028,'yellow',bevel=.009)


def make_traditional(kind):
 begin(kind);wide=kind=='violet_gym';w=5.5 if wide else 3.50;d=3.2
 foundation(w+.14,d+.12)
 cube('Warm traditional plaster',(0,0,1.24),(w,d,1.99),'plaster_light',.025)
 building_trim(w,d,1.29,1.92)
 for y in [-d/2-.025,d/2+.025]:
  for x in [-w*.4,-w*.2,0,w*.2,w*.4]:cube('Exposed timber rhythm',(x,y,1.28),(.09,.095,1.85),'timber_light')
 for xx in [-w/2,w/2]:mesh('Traditional pale gable',[(xx,-d/2,2.23),(xx,d/2,2.23),(xx,0,3.0)],[(0,1,2)] if xx<0 else [(2,1,0)],'plaster')
 roof(w+.48,d+.51,2.25,3.08,'violet',6)
 fy=-d/2-.035;door(fy,z=1.03,w=.88,h=1.56)
 for x in [-w*.32,w*.32]:
  window(x,1.28,fy,w=.79 if wide else .60,h=.78)
  for dx in [-.22,-.11,.11,.22]:facade_cube('Traditional window lattice',x+dx,1.28,.025,.66,fy-.20,.04,'timber_light')
 for a in [-90,90]:window(0,1.27,-w/2-.02,w=.9,h=.76,angle=a)
 cube('Traditional stone threshold',(0,-1.925,.10),(1.28,.69,.20),'stone_light',.024)
 if wide:
  facade_cube('Gym nameboard',0,2.07,1.32,.37,fy-.11,.12,'violet_dark',bevel=.027)
  for x in [-.41,0,.41]:
   facade_cube('Gym plaque gold lettering',x,2.07,.14,.19,fy-.19,.025,'yellow',bevel=.012)
  for s in [-1,1]:
   # Original wing-like wooden crests support the gym's bird theme.
   for i in range(3):beam('Wing crest',(s*.22,fy-.17,2.52+i*.04),(s*(.48+i*.17),fy-.17,2.68+i*.12),.07,.04,'plaster_light')


def make_tower():
 begin('sprout_tower');foundation(3.60,6.00)
 # Three diminishing timber-and-plaster storeys, with hand-laid violet roofs.
 for level,(w,d,z) in enumerate([(3.48,5.72,.25),(2.92,4.60,2.02),(2.32,3.42,3.59)]):
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
  # Tiered roof courses stay solid and have modeled rims all the way around.
 fy=-2.865;door(fy,z=1.06,w=.84,h=1.58)
 cube('Tower threshold',(0,-3.13,.11),(1.23,.54,.22),'stone_light',.025)
 facade_cube('Tower entry plaque',0,1.97,.64,.26,fy-.10,.10,'timber',bevel=.018)
 for x in [-.17,0,.17]:facade_cube('Tower brass plaque marks',x,1.97,.055,.14,fy-.16,.018,'yellow')
 rod('Tower finial',(0,0,5.89),(0,0,6.39),.07,'timber',8)
 for z,r in [(6.03,.16),(6.17,.13),(6.30,.095)]:cone('Tower finial ring',(0,0,z),r,r,.045,'yellow',10)


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
 if name in TARGETS:data['door_anchor']=[0,0,maxv[2]]
 (P/f'{name}.mesh.json').write_text(json.dumps(data,separators=(',',':')))
 bpy.ops.object.select_all(action='DESELECT')
 for o in objects:o.select_set(True)
 bpy.context.view_layer.objects.active=objects[0]
 bpy.ops.export_scene.gltf(filepath=str(P/f'{name}.glb'),use_selection=True,export_format='GLB',export_yup=True,export_apply=True,export_materials='EXPORT',export_animations=False)
 STATS[name]={k:v for k,v in data.items() if k!='primitives'}
 assert tri_count<6500,(name,tri_count)
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
