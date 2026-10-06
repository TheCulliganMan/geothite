"""Original closed papercraft slatted partitions and Ecruteak theater scenery.

python tools/build-traditional-room.py OUTPUT --runtime-only
blender -b --threads 2 --python tools/build-traditional-room.py -- OUTPUT
No pack pixels, source textures, scripts, collision or game meshes are imported.
Every part is a named editable manifold mesh. Runtime Y-up, +Z south.
"""
from pathlib import Path
import hashlib,json,math,sys
sys.path.insert(0,str(Path(__file__).resolve().parent))
from model_asset_storage import validate_model
PALETTE={'walnut':(.25,.15,.09,1),'oak':(.48,.32,.17,1),'oak_cut':(.63,.47,.26,1),'edge':(.33,.225,.135,1),'paper':(.78,.77,.61,1),'paper_light':(.85,.82,.66,1),'reed':(.57,.57,.34,1),'reed_light':(.61,.61,.37,1),'ink':(.245,.32,.22,1),'plum':(.62,.37,.29,1),'brass':(.61,.49,.25,1)}
DIMS={'slatted_rail_bay':(8,16,2.5),'theater_platform':(192,8,80),'theater_backdrop':(192,16,4)}
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def cross(a,b):return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def normal(a,b,c):
 n=cross(sub(b,a),sub(c,a));d=math.sqrt(sum(v*v for v in n));assert d>1e-12
 return tuple(v/d for v in n)
class Asset:
 def __init__(self,name):self.name=name;self.parts=[]
 def part(self,label,material,vertices,faces):self.parts.append((label,material,vertices,faces))
 def rings(self,label,material,rings):
  n=len(rings[0]);assert all(len(r)==n for r in rings);v=[p for r in rings for p in r];f=[tuple(reversed(range(n)))]
  for j in range(len(rings)-1):
   for i in range(n):k=(i+1)%n;f.append((j*n+i,j*n+k,(j+1)*n+k,(j+1)*n+i))
  f.append(tuple((len(rings)-1)*n+i for i in range(n)));self.part(label,material,v,f)
 def box(self,label,material,b,bevel=.18):
  x0,x1,y0,y1,z0,z1=b
  if not bevel:
   self.rings(label,material,[[(x0,y,z0),(x0,y,z1),(x1,y,z1),(x1,y,z0)] for y in [y0,y1]]);return
  bevel=min(bevel,(x1-x0)/5,(y1-y0)/3,(z1-z0)/5)
  def ring(y,inset):
   a,b=x0+inset,x1-inset;c,d=z0+inset,z1-inset;s=bevel
   return [(a+s,y,c),(a,y,c+s),(a,y,d-s),(a+s,y,d),(b-s,y,d),(b,y,d-s),(b,y,c+s),(b-s,y,c)]
  self.rings(label,material,[ring(y0,bevel*.45),ring(y0+bevel,0),ring(y1-bevel,0),ring(y1,bevel*.45)])
 def folded_leaf(self,label,mat,cx,cy,cz,w,h):
  v=[(cx-w,cy,cz),(cx,cy+h,cz),(cx+w,cy,cz),(cx,cy-h,cz),(cx,cy,cz+.18),(cx,cy,cz-.09)]
  f=[(0,4,1),(1,4,2),(2,4,3),(3,4,0),(0,1,5),(1,2,5),(2,3,5),(3,0,5)];self.part(label,mat,v,f)
def assets():
 a=Asset('slatted_rail_bay')
 # Beam ends meet precisely at each 8px repeat; slats leave genuine openings.
 a.box('Continuous top handrail','oak_cut',(0,8,14.1,16,0,2.5),0)
 a.box('Continuous mid crossrail','oak',(0,8,6.1,7.6,.25,2.25),0)
 a.box('Square vertical slat','oak',(3.0,5.0,0,14.2,.38,2.12),.18)
 a.box('Slat broad inset face','oak_cut',(3.35,4.65,1.3,5.5,2.06,2.28),.10)
 a.box('Rear slat inset face','edge',(3.35,4.65,1.3,5.5,.22,.44),.10)
 a.box('Upper mortise peg front','brass',(3.7,4.3,12.25,12.85,2.11,2.32),.09)
 rail=a
 a=Asset('theater_platform')
 # Exactly the native 8px top. Last source course remains raised at the same
 # height, including native FLOOR access columns: no collision/support edits.
 a.box('Closed stage foundation','walnut',(0,192,0,7.80,0,79),0)
 a.box('Continuous level stage top','reed',(0,192,7.80,8,0,80),0)
 # Quiet reed weave and seams are material subdivisions at the identical datum.
 # All top faces coincide only at their edges, not with an extra full top skin.
 # The separate top is replaced below by adjacent sealed strips to avoid z-fight.
 a.parts.pop()
 for r in range(10):
  z=r*8
  for c in range(12):
   x=c*16
   a.box(f'Level reed mat {r:02d} {c:02d}','reed_light' if (r+c)%2 else 'reed',(x,x+16,7.80,8,z,z+8),0)
   # A seam inset on the rear/front mat edge is an inlaid closed band. Its top
   # is exactly 8px; no raised pads are introduced by the finish.
 # Front fascia is joined walnut; access columns have three short horizontal
 # tread/nosing reliefs instead of the vertical decorative studs elsewhere.
 for x in range(24):
  lo=x*8;hi=lo+8
  if x in [2,3,20,21]:
   for y in [1.0,3.3,5.6]:a.box(f'Access tread relief {x:02d} {y}','oak_cut',(lo,hi,y,y+.62,79.25,80),.07)
  else:
   a.box(f'Fascia inset panel {x:02d}','edge',(lo+.45,hi-.45,.55,6.95,79.35,79.68),.09)
   a.box(f'Fascia short vertical key {x:02d}','oak_cut',(lo+3.30,lo+4.70,1.2,5.8,79.60,80),.13)
 for start,end in [(0,16),(32,160),(176,192)]:
  a.box(f'Fascia upper trim {start}','oak_cut',(start,end,7.02,7.60,79.20,80),0)
  a.box(f'Fascia lower trim {start}','oak',(start,end,.10,.62,79.20,80),0)
 stage=a
 a=Asset('theater_backdrop')
 # Twelve shoji-like paper bays correspond to twelve 16px rear screen motifs.
 # Flowers are independently folded relief, not textures or copied pixel art.
 a.box('Continuous rear sill','oak',(0,192,0,2.0,0,4),0)
 a.box('Continuous top lintel','oak_cut',(0,192,14.65,16,0,4),0)
 for bay in range(12):
  x=bay*16
  a.box(f'Paper screen {bay:02d}','paper_light' if bay%2 else 'paper',(x+.65,x+15.35,2,14.68,1.4,2.6),.10)
  for q in [x,x+15.1]:a.box(f'Screen stile {bay:02d} {q}','oak',(q,q+.9,1.70,14.85,.75,3.20),0)
  a.box(f'Screen foot rail {bay:02d}','oak_cut',(x+.75,x+15.25,2.0,2.75,.78,3.20),0)
  a.box(f'Screen lower lattice {bay:02d}','edge',(x+.75,x+15.25,4.10,4.65,2.65,3.08),0)
  # Leaf/fan motif remains restrained against the warm paper.
  stem=x+(6.3 if bay%2 else 9.7)
  a.box(f'Botanical stem {bay:02d}','ink',(stem-.16,stem+.16,2.80,7.6,2.63,2.93),.05)
  a.folded_leaf(f'Folded left leaf {bay:02d}','ink',stem-1.25,5.3,2.83,1.5,.8)
  a.folded_leaf(f'Folded right leaf {bay:02d}','ink',stem+1.35,6.7,2.83,1.6,.9)
  a.folded_leaf(f'Plum petal {bay:02d}','plum',stem,8.0,2.85,.78,1.0)
 return [rail,stage,a]
def document(asset):
 dims=DIMS[asset.name];primitives=[]
 for label,mat,vertices,faces in asset.parts:
  p={'part':label,'material':mat,'positions':[],'normals':[],'indices':[],'base_color':PALETTE[mat]}
  for face in faces:
   for i in range(1,len(face)-1):
    tri=[vertices[j] for j in (face[0],face[i],face[i+1])];tri=[tuple(round(v/d,8) for v,d in zip(point,dims))for point in tri];n=normal(*tri);base=len(p['positions'])//3
    for v in tri:p['positions'].extend(v);p['normals'].extend(round(x,8)for x in n)
    p['indices'].extend((base,base+1,base+2))
  primitives.append(p)
 return {'format':'geothite-traditional-room-v1','name':asset.name,'dimensions_pixels':dims,'primitives':primitives}
def save_runtime(a,out):
 raw=(json.dumps(document(a),separators=(',',':'))+'\n').encode()
 path=out/(a.name+'.mesh.json');path.write_bytes(raw);validate_model(path)
def build_blender(models,out,skip):
 import bpy
 from mathutils import Vector
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 mats={}
 for name,color in PALETTE.items():
  m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True;m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=color;m.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.88;mats[name]=m
 scene=bpy.context.scene;scene['Provenance']='Original closed papercraft geometry; no source art or pack data imported';scene['rebuild']='tools/build-traditional-room.py';scene['NativeStageHeight']=8
 for model in models:
  coll=bpy.data.collections.new('ASSET | '+model.name);scene.collection.children.link(coll);coll['runtime_sha256']=hashlib.sha256((out/(model.name+'.mesh.json')).read_bytes()).hexdigest();coll['dimensions_pixels']=DIMS[model.name]
  for label,mat,vertices,faces in model.parts:
   me=bpy.data.meshes.new(label);me.from_pydata([(x/16,-z/16,y/16)for x,y,z in vertices],[],faces);me.materials.append(mats[mat]);me.update();o=bpy.data.objects.new(model.name+' | '+label,me);coll.objects.link(o);o['authored_part']=label
   o.location=(0,0,.5)if model.name=='theater_backdrop'else(0,0,0)
   if model.name=='slatted_rail_bay':o.location=(0,-6.1,0)
 # Linked preview rail repetitions stay outside ASSET collections.
 preview=bpy.data.collections.new('PREVIEW | Joined rails');scene.collection.children.link(preview)
 for j in range(1,8):
  for obj in bpy.data.collections['ASSET | slatted_rail_bay'].objects:
   o=obj.copy();o.data=obj.data;preview.objects.link(o);o.location.x+=j*.5
 scene.render.engine='CYCLES';scene.cycles.samples=20;scene.cycles.use_denoising=False;scene.render.threads_mode='FIXED';scene.render.threads=2;scene.render.resolution_x=1500;scene.render.resolution_y=1000;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX';scene.world.color=(.20,.20,.20)
 for pos,power,size in [((2,-6,12),2100,8),((11,3,8),1400,7)]:
  bpy.ops.object.light_add(type='AREA',location=pos);o=bpy.context.object;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector((6,-2,.4))-o.location).to_track_quat('-Z','Y').to_euler()
 bpy.ops.object.camera_add(location=(16,-17,14));cam=bpy.context.object;scene.camera=cam;cam.data.type='ORTHO';cam.data.ortho_scale=15.8
 def aim(pos):cam.location=pos;cam.rotation_euler=(Vector((6,-2.6,.35))-cam.location).to_track_quat('-Z','Y').to_euler()
 aim((16,-17,14));bpy.ops.wm.save_as_mainfile(filepath=str(out/'traditional-room-scenery.blend'),compress=False)
 if not skip:
  for suffix,pos in [('front',(16,-17,14)),('rear',(-5,11,12))]:aim(pos);scene.render.filepath=str(out/('traditional-room-'+suffix+'.png'));bpy.ops.render.render(write_still=True)
def main():
 args=sys.argv[sys.argv.index('--')+1:]if'--'in sys.argv else sys.argv[1:];out=Path(next((a for a in args if not a.startswith('--')),'target/traditional-room'));out.mkdir(parents=True,exist_ok=True);models=assets()
 for a in models:save_runtime(a,out);print(a.name,len(a.parts),'closed editable parts')
 if '--runtime-only'not in args:build_blender(models,out,'--skip-preview'in args)
if __name__=='__main__':main()
