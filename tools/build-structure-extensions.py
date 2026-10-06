"""Original source-aware mountain, ice shelf, approach, and Kanto architecture.
Run Blender --background --python tools/build-structure-extensions.py -- target/structure-extensions.
No game artwork or content-pack data is loaded by this authoring source.
"""
from pathlib import Path
source = Path(__file__).with_name('build-world-exteriors.py').read_text()
exec(compile(source.split('\nbuilders=')[0], 'world_architecture_vocabulary', 'exec'))
for n,c in {'ice_stone':(.31,.56,.64),'ice_light':(.60,.79,.79),'ice_dark':(.19,.37,.45),'ice_snow':(.78,.88,.83),'highland':(.52,.51,.37),'highland_light':(.65,.62,.44),'highland_dark':(.34,.36,.28),'lavender':(.50,.44,.56),'lavender_light':(.72,.66,.70),'lavender_dark':(.30,.28,.39)}.items():
 m=bpy.data.materials.new(n);m.diffuse_color=(*c,1);m.use_nodes=True;m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(*c,1);m.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.84;MATS[n]=m

original_flat_roof=flat_roof
def flat_roof(w,d,z,palette='roof_blue',parapet=True):
 before=set(COL.objects)
 original_flat_roof(w,d,z,palette,parapet)
 # Butt the side copings into the long courses; coincident corner cubes
 # otherwise cause black coplanar seams in both raster and ray-traced views.
 for o in set(COL.objects)-before:
  if o.name.startswith('Side parapet coping'):o.scale.y*=(d-.22)/d

def prism(name,polygon,bottom,top,mat):
 v=[(x,y,z) for z in [bottom,top] for x,y in polygon];n=len(polygon)
 return mesh(name,v,[tuple(reversed(range(n))),tuple(range(n,2*n))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)],mat)

def rocks_between(a,b,mat='highland',height=1.85,count=8):
 # Hand-designed sedimentary facets repeat at a restrained, coherent scale.
 a=Vector(a);b=Vector(b)
 for row in range(3):
  for j in range(count):
   t=(j+.5)/count;p=a.lerp(b,t);span=(b-a).length/count
   o=ico('Chiseled stratified rock course',(p.x,p.y,.30+row*.58),(span*.62,.29,.34),mat if (j+row)%3 else mat+'_light',1)
   o.rotation_euler.z=math.atan2(b.y-a.y,b.x-a.x)

def mesa():
 start('highland_mesa',12,8,'none')
 poly=[(-5.8,-3.4),(-5.3,-4),(5.3,-4),(5.8,-3.4),(6,2.8),(5.25,4),(-5.25,4),(-6,2.8)]
 prism('Closed chamfered highland core',poly,0,1.86,'highland_dark')
 prism('Continuous level sandstone cap',[(x*.985,y*.975) for x,y in poly],1.82,2.0,'highland_light')
 for a,b in zip(poly,poly[1:]+poly[:1]):rocks_between(a,b,count=max(2,round((Vector(b)-Vector(a)).length/.82)))
 for i in range(10):
  x=-4.5+(i%5)*2.1;y=-2.5+(i//5)*4.6
  prism('Inlaid mineral vein',[(x,y),(x+.9,y+.15),(x+1.15,y+.23),(x+.18,y+.10)],1.999,2.001,'highland')

def cave():
 start('diglett_cave',16,8,'south')
 # Actual passage is left of center. The aperture is made of separate piers
 # and a stone arch, with a deep interior. No hidden solid crosses the opening.
 poly=[(-7.8,-3.4),(-7.2,-4),(7.2,-4),(7.8,-3.4),(8,3.1),(7.1,4),(-7.1,4),(-8,3.1)]
 prism('Cave rear plateau',[(x,y) for x,y in [(-7.2,-4),(7.2,-4),(7.8,-3.4),(8,1.05),(-8,1.05),(-7.8,-3.4)]],0,1.88,'highland_dark')
 prism('Level cave cap',poly,1.82,2.0,'highland_light')
 # Keep the door at x=0 in export; runtime's source-specific fitting places
 # this complete two-cell opening at native x=5 within the sixteen-cell plot.
 for a,b in [(-8,-1),(1,8)]:
  cube('Solid cave facade wing',((a+b)/2,2.54,.88),(b-a,2.91,1.76),'highland_dark',.055)
  rocks_between((a,4.02),(b,4.02),count=max(2,round(b-a)))
 for s in [-1,1]:
  for i in range(3):ico('Carved cave opening jamb',(s*(1.05-i*.04),3.70,.28+i*.51),(.25,.34,.30),'highland_light',1)
 for i in range(5):
  a=(i+.5)*math.pi/5
  ico('Faceted arch voussoir',(math.cos(a)*1.10,3.71,1.27+math.sin(a)*.40),(.32,.38,.26),'highland',1)
 cube('Tunnel recessed darkness',(0,1.16,.77),(1.85,.055,1.54),'ink')
 cube('Native level cave threshold',(0,3.20,.012),(1.98,1.60,.024),'highland_light')
 for x in [-7.8,7.8]:rocks_between((x,-3.15),(x,2.8),count=7)
 rocks_between((-7.2,-3.8),(7.2,-3.8),count=14)
 # Source Z-up front -Y: this object was described in native south +Y.
 bpy.context.view_layer.update()
 for o in ASSETS['diglett_cave'].objects:o.matrix_world=Matrix.Diagonal((1,-1,1,1)) @ o.matrix_world

def forecourt():
 start('tower_forecourt',8,2,'none')
 # Boardwalk top is deliberately y-up zero at runtime; negative substrate
 # provides physical thickness without raising the player walking support.
 cube('Closed timber landing',(0,0,-.20),(8,2,.32),'timber',.035)
 for i in range(16):cube('Individual beveled landing plank',(0,-.9375+i*.125,-.027),(7.90,.115,.054),'timber_gold' if i%3 else 'timber_light',.007)
 for x in [-3.78,3.78,-1.95,.05]:
  for y in ([-.82,.82] if abs(x)>3 else [-.82]):
   cube('Cut stone railing pier',(x,y,.31),(.20,.22,.62),'stone_light',.025)
   cube('Beveled pier cap',(x,y,.66),(.31,.32,.12),'plaster_light',.032)
 for a,b in [(-3.77,-2.03),(.12,3.77)]:
  for z in [.27,.52]:cube('Front rail with exact entrance gap',((a+b)/2,-.82,z),(b-a,.09,.085),'timber',.013)
 for x in [-3.78,3.78]:
  for z in [.27,.52]:cube('Side handrail',(x,0,z),(.09,1.64,.085),'timber',.012)
 for i in range(3):cube('Flush stone entry inlay',(-1,-.74+i*.20,.004),(1.92,.17,.008),'stone_light',.003)

def lavender_tower():
 start('lavender_radio_tower',6,4.4)
 full_shell(5.65,3.75,6.15,'plaster_light',4,4)
 for z in [1.74,3.23,4.72,6.3]:
  for y in [-1.93,1.93]:cube('Lavender limestone storey belt',(0,y,z),(5.84,.17,.16),'lavender_light',.028)
  for x in [-2.88,2.88]:cube('Side limestone storey belt',(x,0,z),(.17,3.92,.16),'lavender_light',.028)
 front_entry(3.75,1.0);flat_roof(5.94,4.10,6.48,'roof_teal')
 for x in [-2.60,2.60]:cube('Continuous lavender corner fin',(x,-1.95,3.28),(.23,.20,6.12),'lavender',.035)
 cube('Broadcast entry overhang',(0,-2.05,2.04),(1.75,.55,.16),'lavender_dark',.035)
 facade_cube('Broadcast station insignia panel',0,2.40,1.32,.41,-2.02,.12,'lavender',bevel=.03)
 for x in [-.38,-.19,0,.19,.38]:facade_cube('Broadcast waveform relief',x,2.40,.058,.12+.18*(1-abs(x)/.5),-2.10,.03,'cream')
 # Low service antenna keeps the native four-storey silhouette readable.
 rod('Roof aerial mast',(1.43,.71,6.72),(1.43,.71,7.60),.035,'stone_dark',8)
 for z,w in [(7.08,.62),(7.38,.93)]:rod('Aerial crossarm',(1.43-w/2,.71,z),(1.43+w/2,.71,z),.025,'stone_light',6)

def kanto_gate():
 start('kanto_route_gate',4.6,4.2)
 full_shell(4.1,3.45,2.48,'plaster_light',1,3)
 flat_roof(4.52,3.87,2.75,'roof_blue')
 front_entry(3.45,.98)
 cube('Gate canopy',(0,-1.86,2.13),(1.48,.63,.17),'roof_blue_dark',.03)
 facade_cube('Route gateway crest panel',0,2.46,1.18,.36,-1.85,.13,'roof_blue',bevel=.025)
 for x in [-.32,0,.32]:facade_cube('Directional wayfinding chevrons',x,2.46,.13,.17,-1.935,.024,'cream')
 # Northern entry uses the same modest civic geometry on the opposite face.
 for x in [-.50,.50]:facade_cube('Rear passage stone jamb',x,1.02,.13,1.64,-1.84,.10,'stone_light',angle=180)
 facade_cube('Rear passage inset',0,1.02,.87,1.61,-1.79,.07,'ink',angle=180)

# Ice shelf parts are true connected, authored corner / run / notch / stair
# modules. Their logical 4x4 footprints are preserved, not normalized by the
# varying ornament bounds; continuous caps and border rocks meet precisely.
ICE_NAMES={4:'northwest',5:'north',6:'northeast',9:'interior',12:'southwest',13:'south',14:'southeast',16:'notch_east',17:'notch_west',18:'stair_right',58:'stair_corner',62:'stair_left'}
def ice_run(a,b):
 a=Vector(a);b=Vector(b);n=max(1,round((b-a).length/.63))
 for row in range(3):
  for i in range(n):
   p=a.lerp(b,(i+.5)/n);o=ico('Faceted icy border strata',(p.x,p.y,.35+row*.54),((b-a).length/n*.61,.23,.33),'ice_light' if (i+row)%3==0 else 'ice_stone',1);o.rotation_euler.z=math.atan2(b.y-a.y,b.x-a.x)

 for i in range(n):
  p=a.lerp(b,(i+.5)/n)
  o=ico('Chamfered frosted shelf coping',(p.x,p.y,1.995),((b-a).length/n*.57,.25,.105),'ice_light',1);o.rotation_euler.z=math.atan2(b.y-a.y,b.x-a.x)

def shelf(kind):
 name='ice_shelf_'+ICE_NAMES[kind];start(name,4,4,'none')
 # Coordinates are x east, y south. Flat cap covers the authoritative first
 # two source courses; face / stairs occupy the lower two source courses.
 front=kind in [12,13,14,18,58,62]
 cap=[(-2,-2),(2,-2),(2,2),(-2,2)]
 if kind==18:cap=[(-2,-2),(2,-2),(2,0),(0,0),(0,2),(-2,2)]
 if kind in [58,62]:cap=[(-2,-2),(2,-2),(2,2),(0,2),(0,0),(-2,0)]
 if kind==16:cap=[(-2,-2),(2,-2),(2,0),(0,0),(0,2),(-2,2)]
 if kind==17:cap=[(-2,-2),(2,-2),(2,2),(0,2),(0,0),(-2,0)]
 prism('Closed connected ice shelf cap',cap,1.78,2.0,'ice_snow')
 # No hidden rectangular hull across a notch or an entry.
 if kind in [4,5,6]:ice_run((-1.80,-1.82),(1.80,-1.82))
 if kind in [4,12]:ice_run((-1.80,-1.70),(-1.80,1.64 if front else 1.98))
 if kind in [6,14,58]:ice_run((1.80,-1.70),(1.80,1.65 if front else 1.98))
 if kind in [12,13,14]:ice_run((-1.70,1.58),(1.70,1.58))
 if kind==16:ice_run((.25,.25),(1.76,1.76))
 if kind==17:ice_run((-1.76,1.76),(-.25,.25))
 if kind==18:ice_run((-1.76,1.58),(-.25,1.58))
 if kind==62:ice_run((.25,1.58),(1.76,1.58))
 if kind in [18,58,62]:
  x=1 if kind==18 else -1
  for i in range(6):
   yy=.166+i/3;h=2*(1-i/6)
   cube('Broad source-aligned stone stair tread',(x,yy,h/2), (1.92,.334,h),'ice_light' if i%2==0 else 'ice_stone',.012)
  for xx in [x-.94,x+.94]:rod('Stair stone nosing edge',(xx,.01,1.99),(xx,1.98,.02),.035,'ice_dark',6)
 # Restrained mineral seams in the cap are actual thin recessed-style parts.
 for j in range(2):
  y=-1.1+j*.72
  prism('Ice cap mineral seam',[(-.85,y),(.12,y+.08),(.50,y+.04),(-.43,y-.025)],1.999,2.001,'ice_light')
 # Native south is source-model front (-Y).
 bpy.context.view_layer.update()
 for o in ASSETS[name].objects:o.matrix_world=Matrix.Diagonal((1,-1,1,1)) @ o.matrix_world

mesa();cave();forecourt();lavender_tower();kanto_gate()
for k in ICE_NAMES:shelf(k)
# Export Y-up geometry, preserving logical grid bounds for the shelf modules.
STATS={}
for name,col in list(ASSETS.items()):
 bpy.context.view_layer.update();objects=[o for o in col.objects if o.type=='MESH']
 for o in objects:o.data.transform(o.matrix_world);o.matrix_world=Matrix.Identity(4)
 if name.startswith('ice_shelf_'):
  for o in objects:
   for v in o.data.vertices:
    v.co.x=max(-2,min(2,v.co.x));v.co.y=max(-2,min(2,v.co.y))
 groups={};mins=[float('inf')]*3;maxs=[-float('inf')]*3
 for o in objects:
  bm=bmesh.new();bm.from_mesh(o.data);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(o.data);bm.free();o.data.update();me=o.data;me.calc_loop_triangles();mat=me.materials[0]
  g=groups.setdefault(mat.name,{'name':name+'__'+mat.name,'material':mat.name,'base_color':[round(v,5) for v in mat.diffuse_color],'positions':[],'normals':[],'indices':[],'lookup':{}})
  for tr in me.loop_triangles:
   if tr.normal.length_squared<.9:continue
   # Grid-edge clipping can collapse a tiny faceted sliver after six-decimal
   # export. Exclude only zero-area triangles, retaining the rest unchanged.
   pp=[Vector(tuple(round(c,6) for c in me.vertices[vi].co)) for vi in tr.vertices]
   if (pp[1]-pp[0]).cross(pp[2]-pp[0]).length<1e-9:continue
   for vi in tr.vertices:
    v=me.vertices[vi].co;n=tr.normal;pos=(round(v.x,6),round(v.z,6),round(-v.y,6));normal=(round(n.x,6),round(n.z,6),round(-n.y,6));key=pos+normal
    for j in range(3):mins[j]=min(mins[j],pos[j]);maxs[j]=max(maxs[j],pos[j])
    if key not in g['lookup']:g['lookup'][key]=len(g['positions'])//3;g['positions'].extend(pos);g['normals'].extend(normal)
    g['indices'].append(g['lookup'][key])
 primitives=[]
 for g in groups.values():g.pop('lookup');primitives.append(g)
 data={'name':name,'coordinate_system':'right-handed; +Y up; front +Z','bounds':{'min':mins,'max':maxs},'primitives':primitives}
 if DOORS[name]=='south':data['door_anchor']=[0,0,maxs[2]];data['door_face']='south'
 (P/f'{name}.mesh.json').write_text(json.dumps(data,separators=(',',':')))
 STATS[name]={'parts':len(objects),'triangles':sum(len(g['indices'])//3 for g in primitives),'bounds':data['bounds']}
 # Curated, compact workshop presentation in separated labeled collections.
 i=list(ASSETS).index(name);offset=Vector(((i%5)*19,(i//5)*14,0))
 for o in objects:o.location+=offset
begin('PRESENTATION only');cube('Workshop floor',(35,21,-.38),(120,110,.40),'cream')
world=bpy.data.worlds.new('Structure daylight');scene.world=world;world.use_nodes=True;world.node_tree.nodes['Background'].inputs['Strength'].default_value=.65
for name,loc,energy,size in [('Key',(10,-30,45),21000,30),('Fill',(70,25,35),15000,35)]:
 bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name=name;o.data.energy=energy;o.data.size=size;o.rotation_euler=(Vector((35,20,0))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(77,-71,86));cam=bpy.context.object;cam.rotation_euler=(Vector((36,21,0))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=95;scene.camera=cam
scene.render.engine='CYCLES';scene.cycles.samples=20;scene.cycles.use_denoising=False;scene.render.resolution_x=2000;scene.render.resolution_y=1500;scene.render.resolution_percentage=100;scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.filepath=str(P/'structure-extensions-preview.png')
scene['Source']='Original editable semantic structures. No imported game artwork.'
bpy.ops.wm.save_as_mainfile(filepath=str(P/'structure-extensions.blend'),compress=True)
(P/'structure-stats.json').write_text(json.dumps(STATS,indent=2))
if '--skip-preview' not in args:bpy.ops.render.render(write_still=True)
print('STRUCTURE_STATS='+json.dumps(STATS))
