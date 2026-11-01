"""Original papercraft lighthouse keeper's tea table, cot and low red stool.

blender -b --threads 2 --python tools/build-lighthouse-chamber.py -- OUTPUT
python3 tools/build-lighthouse-chamber.py OUTPUT --runtime-only

The same named closed components create the compact runtime meshes and editable
Blender objects. No textures, game pixels, collision data or copied art enter
this generator. Model coordinates are normalized inside each source footprint;
Y is up and +Z is south. Gaps around furniture remain actual empty space.
"""
from pathlib import Path
import hashlib, json, math, sys
sys.path.insert(0,str(Path(__file__).resolve().parent))
from model_asset_storage import validate_model
PALETTE={
 'walnut':(.26,.155,.085,1), 'wood_cut':(.40,.27,.14,1),
 'wood_highlight':(.53,.37,.19,1), 'dark_joinery':(.17,.105,.065,1),
 'brass':(.58,.44,.22,1), 'brass_shadow':(.36,.27,.12,1),
 'linen':(.73,.72,.61,1), 'linen_edge':(.62,.63,.53,1),
 'sea_glaze':(.27,.43,.43,1), 'glaze_light':(.48,.63,.58,1),
 'tea':(.16,.10,.055,1), 'terracotta':(.60,.22,.15,1),
 'cushion_edge':(.42,.15,.10,1), 'sage_blanket':(.40,.51,.44,1),
}
DIMS={'tea_table':(32,17,48),'keeper_cot':(16,12,32),'red_stool':(16,6,16)}
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def cross(a,b):return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def normal(a,b,c):
 n=cross(sub(b,a),sub(c,a));d=math.sqrt(sum(v*v for v in n));assert d>1e-10
 return tuple(v/d for v in n)
class Asset:
 def __init__(self,name):self.name=name;self.parts=[]
 def part(self,label,material,vertices,faces):
  # Component topology is retained in Blender, with no destructive joining.
  self.parts.append((label,material,vertices,faces))
 def rings(self,label,material,rings):
  n=len(rings[0]);assert all(len(r)==n for r in rings);v=[p for r in rings for p in r];f=[]
  # Rings run clockwise viewed from above; lower cap winds downward.
  f.append(tuple(reversed(range(n))))
  for j in range(len(rings)-1):
   for i in range(n):k=(i+1)%n;f.append((j*n+i,j*n+k,(j+1)*n+k,(j+1)*n+i))
  f.append(tuple((len(rings)-1)*n+i for i in range(n)))
  self.part(label,material,v,f)
 def box(self,label,material,b,bevel=.35):
  x0,x1,y0,y1,z0,z1=b;bevel=min(bevel,(x1-x0)/5,(y1-y0)/3,(z1-z0)/5)
  def ring(y,inset):
   a,b=x0+inset,x1-inset;c,d=z0+inset,z1-inset;s=bevel
   return [(a+s,y,c),(a,y,c+s),(a,y,d-s),(a+s,y,d),(b-s,y,d),(b,y,d-s),(b,y,c+s),(b-s,y,c)]
  self.rings(label,material,[ring(y0,bevel*.45),ring(y0+bevel,0),ring(y1-bevel,0),ring(y1,bevel*.45)])
 def lathe(self,label,material,cx,cz,profile,n=12):
  # Positive-radius profile bottom-to-top, giving a sealed faceted solid.
  self.rings(label,material,[[(cx+r*math.cos(-2*math.pi*i/n),y,cz+r*math.sin(-2*math.pi*i/n)) for i in range(n)] for y,r in profile])
 def tube(self,label,material,points,radii,n=8):
  # x/y path at constant z; closed end caps avoid open spout silhouettes.
  rings=[]
  for j,((x,y,z),r) in enumerate(zip(points,radii)):
   delta=sub(points[min(j+1,len(points)-1)],points[max(0,j-1)]);length=math.hypot(delta[0],delta[1]);ux,uy=-delta[1]/length,delta[0]/length
   rings.append([(x+ux*r*math.cos(2*math.pi*i/n),y+uy*r*math.cos(2*math.pi*i/n),z+r*math.sin(2*math.pi*i/n)) for i in range(n)])
  self.rings(label,material,rings)
 def handle(self,label,material,cx,cy,cz,rx,ry,tube,n=12,sides=6):
  vertices=[];faces=[]
  for i in range(n):
   a=2*math.pi*i/n
   for j in range(sides):
    b=2*math.pi*j/sides;vertices.append((cx+(rx+tube*math.cos(b))*math.cos(a),cy+(ry+tube*math.cos(b))*math.sin(a),cz+tube*math.sin(b)))
  for i in range(n):
   for j in range(sides):faces.append((i*sides+j,((i+1)%n)*sides+j,((i+1)%n)*sides+(j+1)%sides,i*sides+(j+1)%sides))
  self.part(label,material,vertices,faces)

def assets():
 a=Asset('tea_table')
 for x in [3.4,28.6]:
  for z in [3.8,43.6]:
   a.box(f'Tapered walnut leg {x} {z}','walnut',(x-1.15,x+1.15,0,9.3,z-1.15,z+1.15),.22)
   a.box(f'Brass foot sleeve {x} {z}','brass_shadow',(x-1.18,x+1.18,.35,1.1,z-1.18,z+1.18),.15)
 for x in [3.4,28.6]:a.box(f'Long recessed apron {x}','walnut',(x-.65,x+.65,6.4,9.5,4,43),.2)
 for z in [4.2,43.3]:a.box(f'End recessed apron {z}','walnut',(4,28,6.4,9.5,z-.6,z+.6),.2)
 a.box('Heavy bevelled walnut rim','wood_highlight',(1.1,30.9,9.15,10.4,1.3,46.7),.48)
 a.box('Inset long tabletop','wood_cut',(2.15,29.85,10.4,10.64,2.35,45.65),.11)
 # Wide quiet linen field, matching the long cloth-covered source surface.
 a.box('Linen tea runner','linen_edge',(5.1,26.9,10.65,10.77,4,44),.03)
 a.box('Runner broad face','linen',(5.6,26.4,10.77,10.81,4.6,43.4),.01)
 # A cup with a dark liquid inset, cream rim and an actual loop handle.
 a.lathe('Cup saucer','glaze_light',15,12,[(10.82,3.0),(11.0,3.5),(11.22,3.25)],12)
 a.lathe('Cup faceted bowl','sea_glaze',15,12,[(11.2,1.8),(11.5,2.0),(13.3,2.4),(13.55,2.35)],10)
 a.lathe('Cup tea inset','tea',15,12,[(13.51,2.01),(13.56,2.01)],10)
 a.handle('Cup loop handle','glaze_light',17.7,12.45,12,1.15,.9,.30,10,6)
 # Teapot, not a machine: shoulder, fitted lid, button, long pouring spout,
 # and open handle remain separately editable and readable from above.
 a.lathe('Teapot foot','brass_shadow',16,29,[(10.82,2.7),(11.1,3.0)],12)
 a.lathe('Teapot faceted body','sea_glaze',16,29,[(11.05,2.75),(11.8,3.6),(13.8,3.8),(15.05,2.6),(15.25,2.0)],12)
 a.lathe('Teapot fitted lid','glaze_light',16,29,[(15.22,2.5),(15.5,2.45),(15.95,1.2)],12)
 a.lathe('Teapot lid knob','brass',16,29,[(15.92,.50),(16.5,.62),(16.85,.32)],8)
 a.tube('Teapot rising pouring spout','sea_glaze',[(13.2,12.2,29),(10.7,12.6,29),(9.4,14.3,29),(8.6,14.8,29)],[1.15,1.05,.65,.55])
 a.handle('Teapot open handle','glaze_light',20.25,13.6,29,2.05,1.9,.47,12,6)
 table=a
 a=Asset('keeper_cot')
 for x in [1.6,14.4]:
  for z in [1.4,30.6]:
   a.box(f'Cot raised rail post {x} {z}','walnut',(x-.60,x+.60,0,11.6 if z<10 else 8.7,z-.60,z+.60),.18)
   a.box(f'Cot post cap {x} {z}','wood_highlight',(x-.70,x+.70,11.6 if z<10 else 8.7,12 if z<10 else 9.1,z-.70,z+.70),.14)
 for x in [1.6,14.4]:a.box(f'Cot long side rail {x}','wood_cut',(x-.5,x+.5,3.2,5.1,1.6,30.4),.2)
 for z,y in [(1.4,8.4),(1.4,10.7),(30.6,7.4)]:a.box(f'Cot end cross rail {z} {y}','wood_highlight',(2.1,13.9,y,y+.85,z-.45,z+.45),.18)
 for z in [5,11,17,23,28]:a.box(f'Support slat {z}','dark_joinery',(2.0,14.0,3.5,4.0,z-.65,z+.65),.1)
 a.box('Ivory mattress welt','linen_edge',(2.15,13.85,4.0,5.6,2.5,29.5),.4)
 a.box('Soft folded mattress face','linen',(2.3,13.7,5.5,6.25,2.6,29.3),.35)
 a.box('Folded sage blanket','sage_blanket',(2.25,13.75,6.25,6.60,12.0,28.8),.15)
 a.box('Blanket folded hem','linen_edge',(2.2,13.8,6.6,6.9,11.5,13.3),.13)
 a.box('Cream pillow seam','linen_edge',(3.4,12.6,6.25,6.7,3.5,9.4),.2)
 a.box('Cream pillow broad facet','linen',(3.6,12.4,6.6,7.6,3.7,9.2),.46)
 cot=a
 a=Asset('red_stool')
 for x in [4.2,11.8]:
  for z in [4.2,11.8]:a.box(f'Stubby stool leg {x} {z}','walnut',(x-.8,x+.8,0,4.3,z-.8,z+.8),.2)
 for z in [4.2,11.8]:a.box(f'Stool low stretcher {z}','wood_cut',(4.1,11.9,1.3,2.0,z-.30,z+.30),.13)
 a.lathe('Round wood seat base','wood_highlight',8,8,[(3.85,5.65),(4.2,6.35),(4.6,6.30)],12)
 a.lathe('Red cushion piped edge','cushion_edge',8,8,[(4.6,6.15),(5.0,6.50),(5.35,6.35)],12)
 a.lathe('Red cushion broad facets','terracotta',8,8,[(5.32,6.30),(5.75,5.80),(6.0,4.9)],12)
 return [table,cot,a]

def document(asset):
 dims=DIMS[asset.name];primitives=[]
 for label,mat,vertices,faces in asset.parts:
  p={'part':label,'material':mat,'positions':[],'normals':[],'indices':[],'base_color':PALETTE[mat]}
  for face in faces:
   for i in range(1,len(face)-1):
    tri=[vertices[j] for j in (face[0],face[i],face[i+1])];tri=[tuple(round(v/d,8) for v,d in zip(point,dims)) for point in tri];n=normal(*tri);base=len(p['positions'])//3
    for v in tri:p['positions'].extend(v);p['normals'].extend(round(x,8) for x in n)
    p['indices'].extend((base,base+1,base+2))
  primitives.append(p)
 return {'format':'geothite-lighthouse-chamber-v1','name':asset.name,'dimensions_pixels':dims,'primitives':primitives}
def save_runtime(asset,out):
 raw=(json.dumps(document(asset),separators=(',',':'))+'\n').encode()
 path=out/(asset.name+'.mesh.json');path.write_bytes(raw);validate_model(path)
def build_blender(models,out,skip_preview):
 import bpy
 from mathutils import Vector
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 materials={}
 for name,color in PALETTE.items():
  mat=bpy.data.materials.new(name);mat.diffuse_color=color;mat.use_nodes=True;node=mat.node_tree.nodes.get('Principled BSDF');node.inputs['Base Color'].default_value=color;node.inputs['Roughness'].default_value=.82;materials[name]=mat
 scene=bpy.context.scene;scene['kit']='Lighthouse keeper tea set and cot: original papercraft geometry';scene['coordinates']='Runtime Y-up, +Z south, normalized within original source picture';scene['rebuild']='tools/build-lighthouse-chamber.py'
 for model,origin in zip(models,[(0,0,0),(3.0,0,0),(2.35,-3.05,0)]):
  coll=bpy.data.collections.new('ASSET | '+model.name);scene.collection.children.link(coll);coll['runtime_sha256']=hashlib.sha256((out/(model.name+'.mesh.json')).read_bytes()).hexdigest();coll['dimensions_pixels']=DIMS[model.name]
  for label,mat,vertices,faces in model.parts:
   me=bpy.data.meshes.new(label);me.from_pydata([(x/16,-z/16,y/16) for x,y,z in vertices],[],faces);me.materials.append(materials[mat]);me.update();obj=bpy.data.objects.new(model.name+' | '+label,me);coll.objects.link(obj);obj.location=origin;obj['authored_part']=label
   for poly in me.polygons:poly.use_smooth=False
 # Presentation checker is preview-only. Runtime floor uses guarded exact cells.
 for x in range(7):
  for z in range(6):
   bpy.ops.mesh.primitive_cube_add(size=1,location=(x,-z,-.027));obj=bpy.context.object;obj.name='PREVIEW | quiet checker';obj.scale=(.998,.998,.025)
   name='floor'+str((x+z)%2)
   if name not in materials:
    m=bpy.data.materials.new(name);m.diffuse_color=((.48,.52,.48,1) if (x+z)%2 else (.54,.56,.51,1));m.use_nodes=True;m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=m.diffuse_color;materials[name]=m
   obj.data.materials.append(materials[name])
 scene.render.engine='CYCLES';scene.cycles.samples=24;scene.cycles.use_denoising=False;scene.render.resolution_x=1400;scene.render.resolution_y=1000;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX';scene.world.color=(.15,.15,.15)
 for title,location,power,size in [('Key',(0,-4,8),1000,6),('Fill',(6,3,5),750,5)]:
  bpy.ops.object.light_add(type='AREA',location=location);obj=bpy.context.object;obj.name='PREVIEW | '+title;obj.data.energy=power;obj.data.size=size;obj.rotation_euler=(Vector((2,-1.5,.3))-obj.location).to_track_quat('-Z','Y').to_euler()
 bpy.ops.object.camera_add(location=(7,-10,9));camera=bpy.context.object;scene.camera=camera;camera.data.type='ORTHO';camera.data.ortho_scale=7.2;camera.name='PREVIEW | Camera'
 def aim(pos):camera.location=pos;camera.rotation_euler=(Vector((2.0,-1.5,.25))-camera.location).to_track_quat('-Z','Y').to_euler()
 aim((7,-10,9));bpy.ops.wm.save_as_mainfile(filepath=str(out/'lighthouse-chamber.blend'),compress=False)
 if not skip_preview:
  for name,pos in [('front',(7,-10,9)),('reverse',(-5,6,8))]:aim(pos);scene.render.filepath=str(out/('lighthouse-chamber-'+name+'.png'));bpy.ops.render.render(write_still=True)
def main():
 args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else sys.argv[1:];out=Path(next((a for a in args if not a.startswith('--')),'target/lighthouse-chamber'));out.mkdir(parents=True,exist_ok=True);models=assets()
 for a in models:save_runtime(a,out);print(a.name,len(a.parts),'editable components',sum(len(p['indices'])//3 for p in document(a)['primitives']),'triangles')
 if '--runtime-only' not in args:build_blender(models,out,'--skip-preview' in args)
if __name__=='__main__':main()
