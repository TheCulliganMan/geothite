"""Original faceted dungeon art, authored from geometric primitives, never sprites.

blender -b --python tools/build-dungeon-models.py -- target/dungeon-assets
The editable scene retains named objects and collections. Runtime JSON is Y-up,
front +Z. All source units are arbitrary; runtime fits exact verified source plots.
"""
import bpy, math, json, sys
from pathlib import Path
from mathutils import Vector
args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
out=Path(next((a for a in args if not a.startswith('--')),'target/dungeon-kit'));out.mkdir(parents=True,exist_ok=True)
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
scene=bpy.context.scene
PALETTE={'stone':(.43,.43,.36),'stone_light':(.63,.63,.52),'stone_dark':(.28,.30,.28),'sandstone':(.64,.60,.42),'sand_light':(.79,.74,.52),'dark_rock':(.29,.34,.37),'dark_light':(.42,.47,.48),'ice':(.29,.64,.76),'ice_light':(.66,.89,.93),'ice_dark':(.15,.40,.59),'wood':(.39,.24,.13),'wood_light':(.60,.39,.20),'wood_dark':(.21,.14,.11),'steel':(.30,.39,.41),'metal_light':(.53,.62,.59),'brass':(.75,.59,.29),'red':(.70,.22,.19),'cloth':(.73,.76,.66),'cloth_light':(.91,.91,.77),'blue':(.28,.42,.52),'glass':(.19,.49,.60),'ink':(.11,.16,.17),'leaf':(.24,.40,.22)}
M={}
for name,rgb in PALETTE.items():
 m=bpy.data.materials.new(name);m.diffuse_color=(*rgb,1);m.use_nodes=True;bs=m.node_tree.nodes.get('Principled BSDF');bs.inputs['Base Color'].default_value=(*rgb,1);bs.inputs['Roughness'].default_value=.87;M[name]=m
assets={};COL=None
def begin(name):
 global COL
 COL=bpy.data.collections.new(name);scene.collection.children.link(COL);assets[name]=COL

def assign(o,n,mat):
 o.name=n
 for c in list(o.users_collection):c.objects.unlink(o)
 COL.objects.link(o);o.data.materials.append(M[mat]);return o

def box(n,p,s,mat,bevel=.02):
 bpy.ops.mesh.primitive_cube_add(size=1,location=p);o=assign(bpy.context.object,n,mat);o.dimensions=s;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 if bevel:
  mod=o.modifiers.new('Hand-cut bevel','BEVEL');mod.width=min(bevel,min(s)*.25);mod.segments=1;bpy.ops.object.modifier_apply(modifier=mod.name)
 return o

def cone(n,p,r1,r2,h,mat,verts=8):
 bpy.ops.mesh.primitive_cone_add(vertices=verts,radius1=r1,radius2=r2,depth=h,location=p);return assign(bpy.context.object,n,mat)

def ico(n,p,s,mat,sub=1):
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=sub,radius=1,location=p);o=assign(bpy.context.object,n,mat);o.scale=s;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);return o

def rod(n,a,b,r,mat,verts=6):
 a,b=Vector(a),Vector(b);o=cone(n,(a+b)*.5,r,r,(b-a).length,mat,verts);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o

def poly(n,verts,faces,mat):
 me=bpy.data.meshes.new(n);me.from_pydata(verts,[],faces);me.update();o=bpy.data.objects.new(n,me);COL.objects.link(o);me.materials.append(M[mat]);return o

def ring(n,p,outer,inner,h,mat,verts=12):
 x,y,z=p;vs=[]
 for zz,r in [(z-h/2,outer),(z+h/2,outer),(z-h/2,inner),(z+h/2,inner)]:
  vs.extend((x+r*math.cos(i*math.tau/verts),y+r*math.sin(i*math.tau/verts),zz) for i in range(verts))
 fs=[]
 for i in range(verts):
  j=(i+1)%verts;fs += [(i,j,j+verts,i+verts),(i+verts,j+verts,j+3*verts,i+3*verts),(i+2*verts,i+3*verts,j+3*verts,j+2*verts),(i,j,j+2*verts,i+2*verts)]
 return poly(n,vs,fs,mat)

def rock(name,mat,light):
 begin(name)
 # A deliberately asymmetric ring mesh with broad chisel-cut facets.
 rings=[(0,.76),(.13,.98),(.48,.90),(.77,.54)]
 vs=[]
 for j,(z,r) in enumerate(rings):
  for i in range(9):
   a=i*math.tau/9;rr=r*(1+.07*math.sin(i*2.7+j*.6));vs.append((rr*math.cos(a),rr*.76*math.sin(a),z+(0 if j==0 else .035*math.cos(i*1.8))))
 vs.append((-.12,.09,.88));fs=[tuple(reversed(range(9)))]
 for j in range(3):
  for i in range(9):
   k=(i+1)%9;fs.extend([(j*9+i,j*9+k,(j+1)*9+k), (j*9+i,(j+1)*9+k,(j+1)*9+i)])
 for i in range(9):fs.append((27+i,27+(i+1)%9,36))
 o=poly('Chisel-cut full-volume boulder',vs,fs,mat);o.data.materials.append(M[light])
 for f in o.data.polygons:
  if f.index%7==2 or f.center.z>.72:f.material_index=1
 # Small embedded strata are physical wedges, not marks cut from pixels.
 for i in range(2):
  a=-1.1+i*.6;ico('Exposed mineral facet',(.61*math.cos(a),.64*math.sin(a),.23+i*.12),(.17,.09,.10),light)

rock('cave_boulder','stone','stone_light');rock('dark_boulder','dark_rock','dark_light');rock('ice_boulder','ice','ice_light')
begin('ice_mass')
# A complete glacial monolith with a level traversable crown and beveled rims.
vs=[(-1,-.7,0),(1,-.7,0),(1,.7,0),(-1,.7,0),(-.93,-.62,.93),(.93,-.62,.93),(.93,.62,.93),(-.93,.62,.93),(-.82,-.53,1),(.82,-.53,1),(.82,.53,1),(-.82,.53,1)]
fs=[(3,2,1,0),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(4,5,9,8),(5,6,10,9),(6,7,11,10),(7,4,8,11),(8,9,10,11)]
poly('Sealed ice crown',vs,fs,'ice')
for x in [-.68,-.29,.22,.65]:
 poly('Glacial vertical fissure',[(x,-.704,.12),(x+.055,-.704,.21),(x-.02,-.665,.86),(x-.10,-.665,.72)],[(0,1,2),(0,2,3)],'ice_dark')
for x in [-.52,.13,.59]:poly('Ice crown facet',[(x,-.49,1.002),(x+.16,-.25,1.002),(x+.07,.34,1.002)],[(0,1,2)],'ice_light')

def guardian(name,stone,highlight,tall=False):
 begin(name)
 box('Chamfered stone plinth',(0,0,.10),(1.12,.78,.20),stone,.07)
 box('Plinth inset step',(0,0,.25),(.91,.65,.12),highlight,.035)
 cone('Tapered statue pedestal',(0,.035,.61),.38,.29,.64,stone,8)
 ico('Faceted guardian body',(0,0,1.11),(.41,.32,.49),stone,1)
 ico('Guardian head',(0,-.045,1.55),(.34,.29,.29),highlight,1)
 ico('Projecting squared muzzle',(0,-.27,1.45),(.22,.18,.13),stone,1)
 for s in [-1,1]:
  cone('Raised carved ear',(s*.22,-.02,1.79),.11,.025,.29,stone,5)
  ico('Folded shoulder',(s*.33,.015,1.21),(.17,.24,.24),highlight,1)
  rod('Foreleg carving',(s*.23,-.19,1.20),(s*.23,-.25,.75),.11,stone)
  ico('Incised dark eye',(s*.15,-.283,1.59),(.040,.023,.036),'stone_dark',1)
 box('Small dedication inset',(0,-.338,.60),(.27,.02,.23),'stone_dark',.025)
 box('Dedication stone relief',(0,-.352,.60),(.16,.016,.14),highlight,.02)

guardian('tower_guardian','stone','stone_light');guardian('alph_guardian','sandstone','sand_light')
begin('league_podium')
box('Podium chamfered stone foot',(0,0,.10),(1.0,.75,.20),'stone',.06)
box('Tall tapered podium body',(0,.01,.68),(.70,.55,1.08),'stone_light',.06)
box('Inset dedication plaque',(0,-.281,.67),(.39,.022,.52),'stone_dark',.018)
for x in [-.10,.10]:
 for z in [.51,.63,.75,.87]:box('Dedication relief',(x,-.297,z),(.10,.015,.035),'stone_light',.004)
cone('Round podium cornice',(0,0,1.22),.47,.42,.16,'stone_light',10)
ico('Faceted league orb',(0,0,1.60),(.47,.42,.43),'stone_dark',2)
ring('Orb equatorial stone band',(0,0,1.60),.469,.419,.08,'stone_light',14)
ico('Orb front seal',(0,-.426,1.59),(.105,.035,.10),'stone_light',1)
begin('stone_tablet')
box('Footed tablet base',(0,0,.09),(1.02,.67,.18),'sandstone',.065)
box('Beveled upright tablet',(0,.045,.74),(.83,.25,1.23),'sandstone',.12)
box('Inset inscription field',(0,-.094,.82),(.63,.028,.87),'stone_dark',.04)
# Original abstract stone inscription, deliberately no copied game glyphs.
for i in range(3):
 z=.56+i*.25
 ring_obj=ring('Carved oval glyph',(0,0,0),.085,.046,.025,'sand_light',8);ring_obj.rotation_euler.x=math.pi/2;ring_obj.location=(.14 if i%2 else -.12,-.12,z)
 rod('Incised tablet stroke',(-.20,-.12,z+.07),(.18,-.12,z+.07),.014,'sand_light',5)
begin('gym_plaque')
box('Stone marker foot',(0,0,.08),(.98,.48,.16),'stone',.045)
box('Timber plaque panel',(0,0,.56),(.91,.17,.77),'wood',.06)
box('Inset brass plaque',(0,-.10,.59),(.71,.035,.51),'brass',.025)
for i,w in enumerate([.46,.55,.35]):box('Embossed plaque line',(0,-.127,.71-i*.115),(w,.016,.028),'wood_dark',.005)
begin('warehouse_crate')
box('Inner wood casing',(0,0,.54),(1.09,.86,1.03),'wood_dark',.025)
for i in range(5):
 x=-.44+i*.22
 box('Separate lid plank',(x,0,1.075),(.207,.94,.07),'wood_light',.014)
 for side in [-1,1]:box('Front and back planks',(x,side*.457,.55),(.204,.055,.96),'wood',.01)
for s in [-1,1]:
 for y in [-.463,.463]:
  box('Crate edge batten',(s*.46,y,.55),(.13,.08,1.09),'wood_light',.012)
 for z in [.13,.95]:box('Crate side batten',(s*.568,0,z),(.075,.98,.12),'wood_light',.012)
for y in [-.51,.51]:
 rod('Diagonal shipping brace',(-.40,y,.20),(.40,y,.89),.055,'wood_light',4)
 for x in [-.46,.46]:
  for z in [.11,.96]:ico('Iron batten nail',(x,y,z),(.025,.016,.025),'steel',1)
begin('ship_barrel')
for i in range(12):
 a=i*math.tau/12;half=math.pi/12*.96
 vs=[]
 for z,r in [(0,.43),(.12,.47),(.51,.54),(.89,.47),(1.0,.43)]:
  for aa in [a-half,a+half]:vs.append((r*math.cos(aa),r*math.sin(aa),z))
 fs=[(j*2,j*2+1,j*2+3,j*2+2) for j in range(4)];fs += [(0,8,9,1)]
 poly('Individual barrel stave',vs,fs,'wood' if i%3 else 'wood_light')
for z,r in [(.15,.483),(.82,.492)]:ring('Iron barrel hoop',(0,0,z),r,r-.033,.11,'steel',12)
cone('Recessed barrel lid',(0,0,.976),.418,.418,.035,'wood_dark',12)
for x in [-.25,0,.25]:box('Barrel lid boards',(x,0,.999),(.235,.62 if x else .82,.04),'wood_light',.006)
cone('Barrel bung',(.17,-.08,1.029),.063,.057,.033,'wood',8)
begin('ship_stool')
cone('Round padded seat',(0,0,.75),.50,.49,.16,'blue',12);cone('Oak seat rim',(0,0,.65),.50,.50,.075,'wood_light',12)
for s in [-1,1]:
 for t in [-1,1]:rod('Splayed stool leg',(s*.29,t*.29,.62),(s*.36,t*.36,0),.061,'wood')
for z in [.27]:
 for s in [-1,1]:rod('Stool stretcher',(-.32,s*.32,z),(.32,s*.32,z),.027,'wood_light')
begin('ship_rack')
for x in [-.46,.46]:
 for y in [-.25,.25]:box('Rack timber upright',(x,y,.90),(.095,.10,1.8),'wood',.014)
for z in [.13,.63,1.13,1.70]:box('Storage shelf',(0,0,z),(1,.68,.10),'wood_light',.015)
for z in [.40,.90,1.41]:
 for x in [-.28,.14]:
  box('Folded linen',(x,0,z),(.31,.42,.23),'cloth',.03)
  box('Linen fold edge',(x,-.22,z-.055),(.29,.015,.025),'cloth_light',.003)
for s in [-1,1]:rod('Rack rear diagonal',(-.40,.29,.20),(.40,.29,1.61),.022,'wood_dark')
begin('ship_bunk')
for x in [-.46,.46]:
 for y in [-.64,.64]:box('Berth rounded leg',(x,y,.20),(.12,.12,.40),'wood',.025)
box('Berth timber frame',(0,0,.32),(1.1,1.47,.16),'wood_light',.035)
box('Rounded mattress',(0,0,.49),(.98,1.36,.23),'cloth',.08)
box('Folded marine blanket',(0,-.21,.635),(1.0,.89,.055),'blue',.025)
for x in [-.28,.26]:box('Blanket woven stripe',(x,-.21,.666),(.065,.86,.018),'cloth_light',.004)
box('Soft pillow',(0,.45,.67),(.70,.39,.18),'cloth_light',.065)
box('Berth headboard',(0,.73,.56),(1.12,.11,.66),'wood',.075)
begin('porthole_bulkhead')
box('Hull wall panel',(0,.08,.90),(2,.20,1.8),'cloth',.025)
for x in [-.93,.93]:box('Steel bulkhead rib',(x,-.02,.90),(.14,.18,1.82),'metal_light',.02)
box('Timber skirting',(0,-.055,.12),(2,.18,.24),'wood',.025)
# Window drum faces forward; real hollow ring and recessed glass.
o=cone('Porthole recessed glass',(0,0,0),.38,.38,.055,'glass',16);o.rotation_euler.x=math.pi/2;o.location=(0,-.059,1.08)
o=ring('Raised brass porthole ring',(0,0,0),.45,.35,.075,'brass',16);o.rotation_euler.x=math.pi/2;o.location=(0,-.12,1.08)
for i in range(8):
 a=i*math.tau/8;ico('Porthole rivet',(.401*math.cos(a),-.169,1.08+.401*math.sin(a)),(.026,.02,.026),'metal_light',1)
begin('timber_column')
box('Stone column shoe',(0,0,.11),(.62,.57,.22),'stone',.05)
box('Chamfered tower column',(0,0,1.00),(.38,.37,1.78),'wood',.055)
box('Capital collar',(0,0,1.72),(.53,.50,.18),'wood_light',.04)
box('Mortise capital',(0,0,1.91),(.73,.58,.19),'wood',.05)
for y in [-.191,.191]:box('Long carved timber bevel',(0,y,1.0),(.20,.018,1.25),'wood_light',.008)
begin('timber_wall')
box('Tower wall closed core',(0,0,.80),(2.0,.17,1.60),'wood_dark',.012)
for i in range(8):
 x=-.87+i*.25
 box('Individual vertical wall board',(x,-.106,.82),(.235,.095,1.29),'wood' if i%3 else 'wood_light',.012)
for z in [.11,1.49]:box('Heavy wall crossbeam',(0,-.045,z),(2.0,.29,.22),'wood_light',.03)
for x in [-.93,.93]:box('Wall edge tenon',(x,-.055,.80),(.14,.25,1.61),'wood',.025)
for x in [-.64,0,.64]:
 for z in [.11,1.49]:ico('Iron joinery peg',(x,-.205,z),(.023,.016,.023),'steel',1)
begin('warning_beacon')
box('Anchored beacon plinth',(0,0,.12),(.75,.65,.24),'steel',.035)
cone('Steel warning mast',(0,0,.66),.15,.11,.94,'metal_light',8)
box('Beacon bracket',(0,0,1.08),(.62,.43,.14),'steel',.025)
cone('Red faceted warning lens',(0,0,1.37),.26,.17,.48,'red',10)
cone('Beacon protective cap',(0,0,1.63),.24,.17,.09,'steel',10)
for a in [0,math.pi/2,math.pi,3*math.pi/2]:rod('Beacon protective cage',(.27*math.cos(a),.27*math.sin(a),1.14),(.21*math.cos(a),.21*math.sin(a),1.6),.018,'steel',5)
begin('gym_bin')
cone('Tapered brushed metal bin',(0,0,.46),.38,.46,.89,'metal_light',10)
ring('Raised bin rim',(0,0,.90),.475,.40,.08,'steel',10)
cone('Dark open bin interior',(0,0,.88),.395,.395,.022,'ink',10)
for a in [i*math.tau/10 for i in range(10)]:rod('Bin pressed rib',(.365*math.cos(a),.365*math.sin(a),.08),(.444*math.cos(a),.444*math.sin(a),.82),.014,'steel',5)

# Non-destructive export preserves individual authored objects in .blend.
def export(name,col):
 groups={};deps=bpy.context.evaluated_depsgraph_get()
 for obj in col.objects:
  if obj.type!='MESH':continue
  obj_eval=obj.evaluated_get(deps);me=obj_eval.to_mesh();me.calc_loop_triangles();normal_matrix=obj.matrix_world.to_3x3().inverted().transposed()
  for tri in me.loop_triangles:
   mat=obj.data.materials[tri.material_index];g=groups.setdefault(mat.name,{'name':mat.name,'base_color':[round(v,5) for v in mat.diffuse_color],'positions':[],'normals':[],'indices':[]})
   normal=(normal_matrix@tri.normal).normalized()
   if normal.length_squared<.9:continue
   for vid in tri.vertices:
    v=obj.matrix_world@me.vertices[vid].co;g['indices'].append(len(g['positions'])//3);g['positions'].extend(round(c,6) for c in (v.x,v.z,-v.y));g['normals'].extend(round(c,6) for c in (normal.x,normal.z,-normal.y))
  obj_eval.to_mesh_clear()
 prim=[groups[n] for n in sorted(groups)];allp=[p['positions'] for p in prim];mins=[min(v[i] for p in allp for v in zip(*[iter(p)]*3)) for i in range(3)];maxs=[max(v[i] for p in allp for v in zip(*[iter(p)]*3)) for i in range(3)]
 data={'name':name,'coordinate_system':'right-handed; +Y up; front +Z','origin':'floor-center','bounds':{'min':mins,'max':maxs},'triangle_count':sum(len(p['indices'])//3 for p in prim),'primitives':prim}
 (out/(name+'.mesh.json')).write_text(json.dumps(data,separators=(',',':'))+'\n')
 return {'name':name,'triangles':data['triangle_count'],'dimensions':[round(b-a,3) for a,b in zip(mins,maxs)]}
stats=[export(name,col) for name,col in assets.items()]
# Editable source is the unjoined, named-object scene plus an inspectable contact sheet.
for i,(name,col) in enumerate(assets.items()):
 for obj in col.objects:obj.location+=Vector(((i%5)*3.0-6,(i//5)*3.1-4.6,0))
world=bpy.data.worlds.new('Dungeon studio');scene.world=world;world.use_nodes=True;world.node_tree.nodes['Background'].inputs['Color'].default_value=(.17,.23,.27,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.65
for p,energy,size in [((-6,-8,14),1900,10),((8,4,10),1700,8)]:
 bpy.ops.object.light_add(type='AREA',location=p);o=bpy.context.object;o.data.energy=energy;o.data.size=size;o.rotation_euler=(Vector((0,0,.6))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(14,-23,21));cam=bpy.context.object;cam.rotation_euler=(Vector((0,.4,.4))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=20;scene.camera=cam
scene.render.engine='CYCLES';scene.cycles.samples=24;scene.cycles.use_denoising=False;scene.render.resolution_x=1600;scene.render.resolution_y=1200;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.render.film_transparent=True;scene.render.filepath=str(out/'dungeon-kit.png');scene.view_settings.view_transform='AgX'
scene['Provenance']='Original hand-authored geometric kit. No pack art, textures, pixels, scripts, ROM data or game models imported.'
bpy.ops.wm.save_as_mainfile(filepath=str(out/'dungeon-kit.blend'),compress=False)
if '--skip-preview' not in args:bpy.ops.render.render(write_still=True)
print('DUNGEON_STATS='+json.dumps(stats))
