"""Original special-environment extension, built only from geometric primitives.

blender -b --python tools/build-special-environments.py -- target/special-environments
The editable scene retains named objects and collections. Runtime JSON is Y-up,
front +Z. All source units are arbitrary; runtime fits exact verified source plots.
"""
import bpy, math, json, sys
from pathlib import Path
from mathutils import Vector
args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
out=Path(next((a for a in args if not a.startswith('--')),'target/special-environments'));out.mkdir(parents=True,exist_ok=True)
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


begin('ruins_frieze')
box('Frieze closed stone core',(0,.01,.50),(2.0,.33,1.0),'sandstone',.035)
box('Deep inscription recess',(0,-.17,.50),(1.96,.025,.61),'stone_dark',.015)
for z in [.11,.89]:box('Chamfered sandstone cornice',(0,-.045,z),(2.0,.46,.18),'sand_light',.035)
for z in [.23,.77]:box('Narrow carved frieze molding',(0,-.20,z),(2.0,.055,.045),'sandstone',.009)

begin('league_wall')
box('Closed ceremonial wall core',(0,0,.50),(2.0,.38,1.0),'stone',.025)
for i in range(4):
 x=-.75+i*.5
 box('Inset polished wall panel',(x,-.205,.53),(.47,.07,.65),'dark_rock',.045)
 for dx in [-.20,.20]:box('Panel brass inlay',(x+dx,-.252,.53),(.025,.018,.49),'brass',.005)
for z in [.08,.92]:box('Continuous chamfered wall molding',(0,-.04,z),(2.0,.49,.16),'stone_light',.035)

begin('passage_wall')
box('Closed concrete wall',(0,0,.5),(2.0,.40,1.0),'stone',.02)
box('Dark utility panel',(0,-.225,.53),(1.91,.055,.63),'dark_rock',.025)
for x in [-.77,0,.77]:box('Wall joining rib',(x,-.275,.52),(.055,.04,.68),'metal_light',.006)
for z in [.07,.91]:box('Steel wall edge cap',(0,-.01,z),(2.0,.48,.14),'metal_light',.02)

begin('champion_dragon')
box('Broad stepped statue base',(0,0,.11),(1.20,.80,.22),'stone',.07)
box('Carved lower plinth',(0,0,.30),(.94,.63,.17),'stone_light',.045)
cone('Faceted tapering pedestal',(0,.02,.57),.34,.28,.40,'stone',8)
ico('Coiled dragon haunch',(0,.03,.95),(.43,.30,.42),'stone_light',1)
rod('Long rising dragon neck',(0,-.06,1.14),(-.08,-.11,1.86),.16,'stone_light',7)
ico('Dragon angular skull',(-.09,-.13,1.92),(.25,.22,.22),'stone_light',1)
ico('Long dragon muzzle',(-.08,-.35,1.91),(.16,.25,.11),'stone',1)
for sign in [-1,1]:
 rod('Swept stone horn',(sign*.15,-.03,2.01),(sign*.25,.02,2.30),.06,'sand_light',5)
 ico('Recessed dragon eye',(sign*.15,-.31,1.99),(.035,.022,.028),'stone_dark',1)
 rod('Carved foreleg',(sign*.22,-.12,1.13),(sign*.26,-.23,.78),.10,'stone',6)
 poly('Angular folded dragon wing',[(sign*.14,.10,1.18),(sign*.66,.14,1.74),(sign*.54,.18,1.05),(sign*.22,.26,.90),(sign*.31,.22,1.36)],[(0,1,4),(1,2,4),(2,3,4),(3,0,4),(3,2,1,0)],'stone')
 rod('Wing raised rib',(sign*.16,.092,1.19),(sign*.65,.133,1.73),.035,'stone_light',5)
for i in range(3):cone('Dragon dorsal carved spine',(0,.23,1.20+i*.17),.07,0,.17,'sand_light',4)
rod('Curved tail lower',(0,.13,.77),(.40,.21,.56),.11,'stone_light',6)
rod('Curved tail tip',(.40,.21,.56),(.50,.14,.90),.065,'stone_light',5)

begin('ship_bulkhead')
box('Ivory steel bulkhead',(0,.01,.51),(2.0,.22,1.02),'cloth_light',.025)
for x in [-.95,.95]:box('Hull vertical joining rib',(x,-.13,.51),(.10,.12,1.04),'metal_light',.018)
box('Bulkhead upper cap',(0,-.025,.98),(2.0,.34,.13),'metal_light',.025)
box('Marine timber kickboard',(0,-.105,.10),(2.0,.12,.20),'wood',.015)
for x in [-.72,-.24,.24,.72]:ico('Bulkhead lower rivet',(x,-.178,.27),(.023,.014,.023),'steel',1)

begin('ship_door')
box('Recessed door leaf',(0,.06,.71),(.81,.15,1.42),'blue',.10)
for x in [-.49,.49]:box('Rounded steel doorway jamb',(x,0,.79),(.17,.27,1.59),'metal_light',.055)
box('Rounded steel door lintel',(0,0,1.55),(1.08,.27,.17),'metal_light',.05)
box('Flush doorway sill',(0,-.055,.035),(1.08,.39,.07),'steel',.015)
o=cone('Round door glass',(0,0,0),.22,.22,.025,'glass',12);o.rotation_euler.x=math.pi/2;o.location=(0,-.022,1.12)
o=ring('Brass door window surround',(0,0,0),.27,.205,.045,'brass',12);o.rotation_euler.x=math.pi/2;o.location=(0,-.061,1.12)
rod('Dogged door handle',(.20,-.058,.63),(.20,-.058,.85),.035,'brass',6)

begin('harbor_bollard')
box('Chamfered quay foundation',(0,0,.09),(1.04,.73,.18),'stone',.06)
cone('Weathered polygonal mooring post',(0,0,.45),.42,.40,.63,'stone_light',10)
ico('Rounded mooring crown',(0,0,.78),(.41,.32,.18),'stone_light',2)
box('Dark embedded cleat',(0,-.285,.49),(.58,.12,.12),'steel',.045)
for x in [-.34,.34]:ico('Mooring anchor bolt',(x,-.08,.185),(.043,.043,.021),'steel',1)

begin('dock_railing')
for x in [-.92,.92]:
 cone('Rounded quay rail stanchion',(x,0,.47),.067,.062,.91,'metal_light',8)
 cone('Rail anchored foot',(x,0,.055),.135,.115,.11,'steel',8)
for z in [.32,.84]:rod('Continuous round harbor rail',(-1,0,z),(1,0,z),.057,'metal_light',8)
for x in [-.45,0,.45]:rod('Vertical harbor infill',(x,0,.34),(x,0,.83),.03,'steel',6)

begin('hall_of_fame_terminal')
box('Recording console pedestal',(0,.03,.54),(1.08,.59,1.08),'metal_light',.07)
box('Angled main console',(0,-.03,1.19),(1.11,.65,.40),'cloth_light',.07)
box('Dark display bezel',(0,-.383,1.30),(.81,.05,.47),'steel',.045)
box('Blue recording display',(0,-.416,1.30),(.65,.026,.31),'glass',.02)
for i in range(3):box('Display readout line',(-.09,-.433,1.39-i*.075),(.36-i*.05,.014,.018),'ice_light',.003)
for x in [-.32,-.12,.12,.32]:ico('Console tactile key',(x,-.365,1.02),(.05,.024,.038),'red' if x==.32 else 'blue',1)
box('Console lower inset',(0,-.279,.44),(.73,.027,.55),'steel',.045)
for x in [-.24,-.12,0,.12,.24]:box('Recording console ventilation',(x,-.296,.44),(.04,.014,.35),'ink',.004)

begin('puzzle_dais')
box('Stone puzzle base',(0,0,.11),(1.26,.86,.22),'sandstone',.065)
box('Inset puzzle pedestal',(0,.08,.42),(1.05,.56,.46),'sandstone',.055)
box('Stone puzzle display rim',(0,-.02,.78),(1.24,.85,.26),'sand_light',.07)
box('Dark undecorated puzzle inset',(0,-.10,.925),(.96,.59,.045),'stone_dark',.04)
for x in [-.53,.53]:ico('Carved panel corner boss',(x,-.37,.923),(.044,.044,.030),'sandstone',1)

begin('harbor_ferry')
# The docking side faces north; bow is west. The separate bow/stern stations
# build a smooth original hull silhouette rather than extruding sprite pixels.
sections=[(-3.6,.04),(-3.32,.67),(-2.65,.90),(2.64,.90),(3.16,.72),(3.36,.35)]
verts=[]
for z,width in [(0,.82),(.30,.98),(.79,1.0),(1.02,.96)]:
 for x,r in sections:verts.extend([(x,-r*width,z),(x,r*width,z)])
faces=[];N=len(sections)*2
for j in range(3):
 for i in range(len(sections)-1):
  a=j*N+i*2;b=a+2;faces += [(a,b,b+N,a+N),(a+1,a+1+N,b+1+N,b+1)]
 faces += [(j*N,j*N+N,j*N+N+1,j*N+1),(j*N+N-2,j*N+N-1,j*N+2*N-1,j*N+2*N-2)]
faces += [tuple(reversed([2*i for i in range(len(sections))]+[2*i+1 for i in reversed(range(len(sections)))])),tuple([3*N+2*i for i in range(len(sections))]+[3*N+2*i+1 for i in reversed(range(len(sections)))])]
poly('Faceted tapered ferry hull',verts,faces,'cloth_light')
box('Marine blue waterline strake',(-.05,0,.28),(5.89,1.69,.19),'blue',.12)
box('Inset teak deck',(-.04,0,1.03),(5.71,1.54,.085),'wood_light',.12)
box('Long passenger cabin',(.18,.0,1.34),(3.92,1.24,.58),'cloth',.12)
box('Overhanging passenger roof',(.18,0,1.66),(4.18,1.40,.14),'cloth_light',.07)
box('Raised bridge house',(-1.29,0,1.88),(1.25,1.13,.44),'cloth_light',.09)
box('Bridge weather roof',(-1.30,0,2.13),(1.48,1.30,.12),'metal_light',.055)
for side in [-1,1]:
 for x in [-1.21,-.68,-.15,.38,.91,1.44]:box('Passenger window',(x,side*.638,1.39),(.36,.028,.22),'glass',.055)
 for x in [-1.65,-1.27,-.89]:box('Bridge glazing',(x,side*.574,1.92),(.26,.025,.20),'glass',.04)
 for x in [-2.58,-2.12,2.40,2.82]:
  rod('Fore and aft rail stanchion',(x,side*.69,1.06),(x,side*.69,1.40),.027,'metal_light',6)
 for a,b in [(-2.80,-1.89),(2.19,2.96)]:rod('Guard rail open cabin-side gangway',(a,side*.69,1.38),(b,side*.69,1.38),.029,'metal_light',6)
cone('Tapered red funnel',(.83,0,1.99),.30,.26,.59,'red',10)
cone('Funnel dark rim',(.83,0,2.30),.29,.29,.12,'steel',10)
rod('Ship foremast',(-2.37,0,1.08),(-2.37,0,2.04),.037,'metal_light',7)
rod('Ship stern mast',(2.64,0,1.04),(2.64,0,1.69),.032,'metal_light',7)
# North-side recessed entry: the real gangway remains open at the authored
# docking seam; rail sections deliberately stop around it.
box('Recessed passenger entry',(-.61,.636,1.35),(.49,.024,.46),'ink',.025)
box('Gangway threshold',(-.61,.82,1.065),(.72,.50,.085),'metal_light',.035)

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
scene.render.engine='CYCLES';scene.cycles.samples=24;scene.cycles.use_denoising=False;scene.render.resolution_x=1600;scene.render.resolution_y=1200;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.render.film_transparent=True;scene.render.filepath=str(out/'special-environments.png');scene.view_settings.view_transform='AgX'
scene['Provenance']='Original hand-authored geometric kit. No pack art, textures, pixels, scripts, ROM data or game models imported.'
bpy.ops.wm.save_as_mainfile(filepath=str(out/'special-environments.blend'),compress=False)
if '--skip-preview' not in args:bpy.ops.render.render(write_still=True)
print('SPECIAL_ENVIRONMENT_STATS='+json.dumps(stats))
