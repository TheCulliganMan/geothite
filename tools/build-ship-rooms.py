"""Original editable Fast Ship tea furnishings, captain desk and joined bulkheads.

python3 tools/build-ship-rooms.py OUTPUT --runtime-only
blender -b --threads 2 --python tools/build-ship-rooms.py -- OUTPUT [--skip-preview]
Shared lighthouse components are authored geometry, never copied source pixels.
"""
from pathlib import Path
import importlib.util,sys,json,hashlib,math,os
helper=Path(__file__).with_name('build-lighthouse-chamber.py')
if not helper.exists():helper=Path(os.environ.get('GEOTHITE_ART_TOOLS','tools'))/'build-lighthouse-chamber.py'
spec=importlib.util.spec_from_file_location('chamber',helper);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
Asset=m.Asset
base_document=m.document
def document(asset):
 d=base_document(asset);d["format"]="geothite-ship-rooms-v1";return d
m.document=document
m.PALETTE.update({'hull_ivory':(.60,.63,.60,1),'hull_trim':(.73,.75,.69,1),'hull_shadow':(.32,.38,.37,1),'navy_panel':(.22,.30,.32,1),'paper':(.82,.81,.68,1),'paper_edge':(.65,.64,.52,1),'book_cover':(.28,.22,.13,1),'book_ink':(.31,.35,.32,1),'bulkhead_cap':(.27,.36,.39,1)})
DIMS={'tea_table_short':(32,14,32),'tea_table_long':(32,17,64),'tea_table_mess':(32,17,80),'captains_desk':(48,14,32),'lower_bulkhead_u':(192,16,128),'captains_chair':(16,24,24)}
m.DIMS.update(DIMS)

def table(name,length):
 original=m.assets()[0];a=Asset(name)
 for label,mat,vertices,faces in original.parts:
  if label.startswith(('Cup','Teapot')):continue
  v=[(x,y,z*length/48) for x,y,z in vertices]
  a.part(label,mat,v,faces)
 for label,mat,vertices,faces in original.parts:
  if label.startswith('Cup'):
   for j,z in enumerate([11] if length==32 else [11,43]):
    a.part(label+f' {j+1}',mat,[(x,y,zz+z-12) for x,y,zz in vertices],faces)
  if label.startswith('Teapot') and length>32:
   for j,z in enumerate([28] if length==64 else [28,62]):
    a.part(label+f' {j+1}',mat,[(x,y,zz+z-29) for x,y,zz in vertices],faces)
 return a

def captain():
 a=Asset('captains_desk')
 for x in [3.0,45.0]:
  for z in [3.0,29.0]:
   a.box(f'Tapered writing desk leg {x} {z}','walnut',(x-1.3,x+1.3,0,10,z-1.25,z+1.25),.25)
   a.box(f'Brass deck fastener {x} {z}','brass_shadow',(x-1.4,x+1.4,.1,.8,z-1.35,z+1.35),.14)
 for x in [3.0,45.0]:a.box(f'Recessed desk side apron {x}','wood_cut',(x-.7,x+.7,7.0,10.2,3,29),.2)
 for z in [3,29]:a.box(f'Desk transverse apron {z}','walnut',(3,45,7.0,10.0,z-.65,z+.65),.2)
 a.box('Broad bevelled writing surface rim','wood_highlight',(1.1,46.9,9.8,11.0,1.3,30.7),.45)
 a.box('Inset olive writing blotter','sage_blanket',(3,45,11.0,11.14,3.2,28.8),.04)
 a.box('Open logbook leather cover','book_cover',(16.0,32.0,11.16,11.65,9.6,22.4),.22)
 # Raised central binding and broad sloping page leaves, distinct from machines.
 for label,x0,x1,tilt in [('Port',16.5,23.85,False),('Starboard',24.15,31.5,True)]:
  z0,z1=10.2,21.8;low,high=12.2,13.65
  top0,top1=(high,low) if tilt else (low,high)
  vertices=[(x0,11.6,z0),(x1,11.6,z0),(x1,11.6,z1),(x0,11.6,z1),(x0,top0,z0),(x1,top1,z0),(x1,top1,z1),(x0,top0,z1)]
  faces=[(0,1,2,3),(4,7,6,5),(0,4,5,1),(1,5,6,2),(2,6,7,3),(3,7,4,0)]
  a.part(label+' thick logbook leaves','paper_edge',vertices,faces)
  # A thin upper paper wedge is an independent editable sheet.
  vv=[(x,y+.055 if i>=4 else y+.48,z) for i,(x,y,z) in enumerate(vertices)]
  a.part(label+' open cream page','paper',vv,faces)
  for j in range(4):
   xa,xb=x0+.8,x1-.85;za=12.3+j*2.0;zb=za+.16
   ya=top0+(top1-top0)*(xa-x0)/(x1-x0)+.12;yb=top0+(top1-top0)*(xb-x0)/(x1-x0)+.12
   vertices=[(xa,ya,za),(xb,yb,za),(xb,yb,zb),(xa,ya,zb),(xa,ya+.04,za),(xb,yb+.04,za),(xb,yb+.04,zb),(xa,ya+.04,zb)]
   a.part(label+f' ruled journal mark {j}','book_ink',vertices,faces)
 a.box('Book central fabric binding','linen_edge',(23.87,24.13,11.6,14.0,10.0,22.0),.06)
 return a

def chair():
 a=Asset('captains_chair')
 # A compact high-backed wood chair behind the captain's unchanged south-edge
 # foot anchor. Its seat is a real volume; no actor relocation or raised support.
 for x in [1.8,14.2]:
  for z in [2.4,20.5]:
   height=23.1 if z<10 else 7.2
   a.box(f'Chair turned square post {x} {z}','walnut',(x-.7,x+.7,0,height,z-.7,z+.7),.2)
   a.box(f'Chair brass foot ferrule {x} {z}','brass_shadow',(x-.74,x+.74,.1,.75,z-.74,z+.74),.16)
  a.box(f'Chair long seat rail {x}','wood_cut',(x-.6,x+.6,4.5,6.5,3.0,20.3),.22)
 a.box('Chair back upper crest','wood_highlight',(1.1,14.9,22.6,24.0,1.5,3.7),.36)
 a.box('Chair back lower crosspiece','wood_cut',(2.1,13.9,9.0,10.5,1.7,3.8),.22)
 a.box('Chair navy back cushion seam','hull_shadow',(3.0,13.0,10.4,22.5,2.2,4.2),.42)
 a.box('Chair faceted navy back cushion','navy_panel',(3.45,12.55,10.8,22.05,3.9,4.8),.35)
 for z in [4.1,18.2]:a.box(f'Chair seat end apron {z}','walnut',(2.2,13.8,4.5,6.6,z-.55,z+.55),.2)
 a.box('Chair supported wood seat','wood_highlight',(2.1,13.9,6.4,7.0,4.1,18.6),.25)
 a.box('Chair navy seat cushion seam','hull_shadow',(2.55,13.45,7.0,7.55,4.7,18.0),.25)
 a.box('Chair navy cushion broad facets','navy_panel',(2.8,13.2,7.45,8.1,5.0,17.6),.3)
 for x in [1.8,14.2]:a.box(f'Chair low side stretcher {x}','wood_cut',(x-.34,x+.34,1.8,2.5,3.1,19.8),.16)
 return a

def triangulate(poly):
 # Ear clipping for the nonconvex U cap; all components remain closed.
 def cross2(a,b,c):return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
 ids=list(range(len(poly)));area=sum(poly[i][0]*poly[(i+1)%len(poly)][1]-poly[(i+1)%len(poly)][0]*poly[i][1] for i in ids);sgn=1 if area>0 else -1;out=[]
 while len(ids)>3:
  for k,b in enumerate(ids):
   a,c=ids[k-1],ids[(k+1)%len(ids)]
   if cross2(poly[a],poly[b],poly[c])*sgn<=1e-8:continue
   def inside(q):return all(cross2(poly[u],poly[v],poly[q])*sgn>=-1e-8 for u,v in [(a,b),(b,c),(c,a)])
   if any(inside(q) for q in ids if q not in [a,b,c]):continue
   out.append((a,b,c));ids.pop(k);break
  else:raise ValueError('invalid cap')
 out.append(tuple(ids));return out

def u_part(a,label,mat,y0,y1,recess=False):
 # The source corner narrows to one tile, while the transverse facade folds
 # into a three-pixel deep physical seam. No solid bridges the open north.
 poly=[(0,0),(0,128),(192,128),(192,0),(176,0),(176,112),(184,112),(184,125),(8,125),(8,112),(16,112),(16,0)]
 if recess:poly=[(1.25,0),(1.25,127.15),(190.75,127.15),(190.75,0),(177.35,0),(177.35,112),(185.35,112),(185.35,125.85),(6.65,125.85),(6.65,112),(14.65,112),(14.65,0)]
 n=len(poly);v=[(x,y,z) for y in [y0,y1] for x,z in poly];f=[]
 for tri in triangulate(poly):f.append(tuple(reversed(tri)));f.append(tuple(i+n for i in tri))
 for i in range(n):j=(i+1)%n;f.append((i,j,j+n,i+n))
 a.part(label,mat,v,f)

def bulkhead():
 a=Asset('lower_bulkhead_u');u_part(a,'Continuous joined recessed steel room shell','hull_ivory',2.8,14.8,True);u_part(a,'Continuous deck skirting','hull_shadow',0,2.8);u_part(a,'Continuous ivory top rail','hull_trim',14.8,15.2);u_part(a,'Continuous cool steel crown','bulkhead_cap',15.2,16,True)
 # A cool inset crown leaves the thin ivory shoulder visible around the
 # broad wall cap, keeping it distinct from the warm floor. The native outer
 # footprint and 16px visual height are unchanged.
 # Recessed enamel panel courses along both full-depth side limbs. Vertical
 # stiles and foot bands create a joined nautical wall, not tile-sized cubes.
 for side in ['port','starboard']:
  x=14.65 if side=='port' else 176.0
  for k in range(4):
   z0=k*28+1.0;z1=(k+1)*28-1.0
   a.box(side+f' enamel inner panel {k}','navy_panel',((x if side=="port" else x+.03),(x+1.32 if side=="port" else x+1.35),2.8,12.8,z0,z1),.25)
   for z in [z0+1.7,z1-1.7]:
    a.box(side+f' inner brass rivet {k} {z}','brass',((15.90 if side=='port' else 176.0),(16.0 if side=='port' else 176.1),11.1,11.7,z-.32,z+.32),.12)
  xo=.1 if side=='port' else 190.75
  for k in range(4):a.box(side+f' outer recessed panel {k}','hull_shadow',(xo,xo+1.15,3.1,12.7,k*28+1.1,(k+1)*28-1.1),.23)
 # The front transverse course follows the exact narrow footprint. Broad
 # panels have physical front, rear and top surfaces, with closed ends.
 for k in range(6):
  x0=9+k*29;x1=min(x0+26,183)
  a.box(f'Transverse interior enamel panel {k}','navy_panel',(x0,x1,3.0,12.8,125.05,125.86),.2)
  a.box(f'Transverse outer inset panel {k}','hull_shadow',(x0,x1,3.0,12.8,127.14,127.95),.2)
 return a

def assets():return [table('tea_table_short',32),table('tea_table_long',64),table('tea_table_mess',80),captain(),chair(),bulkhead()]
def build_blender(models,out,skip):
 import bpy
 from mathutils import Vector
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 scene=bpy.context.scene;scene['kit']='Fast Ship original table lengths, captain logbook desk and source-shaped U bulkhead';scene['rebuild']='tools/build-ship-rooms.py';scene['coordinates']='Runtime Y-up, +Z south. All structural and furniture parts remain editable.'
 mats={}
 for name,color in m.PALETTE.items():
  mat=bpy.data.materials.new(name);mat.diffuse_color=color;mat.use_nodes=True;node=mat.node_tree.nodes.get('Principled BSDF');node.inputs['Base Color'].default_value=color;node.inputs['Roughness'].default_value=.85;mats[name]=mat
 positions=[(0,0,0),(2.5,0,0),(5,0,0),(7.5,0,0),(10.8,0,0),(0,-6,0)]
 for model,origin in zip(models,positions):
  coll=bpy.data.collections.new('ASSET | '+model.name);scene.collection.children.link(coll);coll['runtime_sha256']=hashlib.sha256((out/(model.name+'.mesh.json')).read_bytes()).hexdigest();coll['dimensions_pixels']=DIMS[model.name]
  for label,mat,vertices,faces in model.parts:
   me=bpy.data.meshes.new(label);me.from_pydata([(x/16,-z/16,y/16) for x,y,z in vertices],[],faces);me.materials.append(mats[mat]);me.update();obj=bpy.data.objects.new(model.name+' | '+label,me);coll.objects.link(obj);obj.location=origin;obj['authored_part']=label
   for poly in me.polygons:poly.use_smooth=False
 scene.render.engine='CYCLES';scene.cycles.samples=16;scene.cycles.use_denoising=False;scene.render.resolution_x=1600;scene.render.resolution_y=1100;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX';scene.world.color=(.17,.17,.17)
 for title,location,power,size in [('Key',(0,-8,12),1400,8),('Fill',(10,3,8),1100,7)]:
  bpy.ops.object.light_add(type='AREA',location=location);o=bpy.context.object;o.name='PREVIEW | '+title;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector((5,-7,0))-o.location).to_track_quat('-Z','Y').to_euler()
 bpy.ops.object.camera_add(location=(15,-22,20));camera=bpy.context.object;scene.camera=camera;camera.data.type='ORTHO';camera.data.ortho_scale=20;camera.name='PREVIEW | Camera'
 def aim(pos):camera.location=pos;camera.rotation_euler=(Vector((5.5,-6.7,.4))-camera.location).to_track_quat('-Z','Y').to_euler()
 aim((15,-22,20))
 proof=[]
 for model in models:
  coll=bpy.data.collections['ASSET | '+model.name]
  assert len(coll.objects)==len(model.parts)
  assert coll['runtime_sha256']==hashlib.sha256((out/(model.name+'.mesh.json')).read_bytes()).hexdigest()
  proof.append({'asset':model.name,'editable_mesh_objects':len(coll.objects),'runtime_sha256':coll['runtime_sha256']})
 (out/'blender-source-check.json').write_text(json.dumps(proof,indent=2)+'\n')
 bpy.ops.wm.save_as_mainfile(filepath=str(out/'ship-rooms.blend'),compress=False)
 if not skip:
  for name,pos in [('front',(15,-22,20)),('reverse',(-8,9,17))]:aim(pos);scene.render.filepath=str(out/('ship-rooms-'+name+'.png'));bpy.ops.render.render(write_still=True)
def main():
 args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else sys.argv[1:];out=Path(next((a for a in args if not a.startswith('--')),'target/ship-rooms'));out.mkdir(parents=True,exist_ok=True);models=assets()
 for a in models:m.save_runtime(a,out);print(a.name,len(a.parts),'closed editable parts',sum(len(p['indices'])//3 for p in m.document(a)['primitives']),'triangles')
 if '--runtime-only' not in args:build_blender(models,out,'--skip-preview' in args)
if __name__=='__main__':main()
