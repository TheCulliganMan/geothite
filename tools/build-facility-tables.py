#!/usr/bin/env python3
"""Original folded-paper facility tables and square-backed chairs.

python tools/build-facility-tables.py OUTPUT --runtime-only
blender -b --threads 2 --python tools/build-facility-tables.py -- OUTPUT
Runtime triangles and editable Blender objects share the same original parts.
No game art, source textures, or content pack are imported into the asset.
"""
from pathlib import Path
import hashlib,json,math,sys
sys.path.insert(0,str(Path(__file__).resolve().parent))
from model_asset_storage import validate_model
PALETTE={
 'oak':(.56,.43,.20,1),'grain':(.62,.49,.25,1),'edge':(.79,.66,.37,1),
 'shadow':(.27,.23,.15,1),'rail':(.40,.32,.18,1),'foot':(.17,.20,.18,1),
 'linen':(.83,.82,.64,1),'paper':(.95,.93,.80,1),'binding':(.23,.30,.29,1),
 'ink':(.18,.24,.22,1),'brass':(.64,.56,.34,1),
}
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def cross(a,b):return(a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def normal(a,b,c):
 n=cross(sub(b,a),sub(c,a));l=math.sqrt(sum(v*v for v in n));assert l>1e-10;return tuple(v/l for v in n)
class Asset:
 def __init__(self,name):self.name=name;self.parts=[]
 def prism(self,label,rect,y0,y1,mat,corner=.15):
  x0,x1,z0,z1=rect;c=min(corner,(x1-x0)*.2,(z1-z0)*.2)
  assert x0<x1 and z0<z1 and y0<y1
  ring=[(x0+c,z0),(x1-c,z0),(x1,z0+c),(x1,z1-c),(x1-c,z1),(x0+c,z1),(x0,z1-c),(x0,z0+c)]
  vs=[(x,y,z)for y in(y0,y1)for x,z in ring];faces=[tuple(range(8)),tuple(range(15,7,-1))]
  for i in range(8):j=(i+1)%8;faces.append((i,i+8,j+8,j))
  self.parts.append((label,mat,vs,faces))

def table(a,width,depth,drawers=False):
 # Four tapered-looking folded posts and recessed aprons keep the underside
 # open from every side. Nothing extends beyond the true drawing footprint.
 for x in[1.15,width-3.15]:
  for z in[1.15,depth-3.15]:
   a.prism('Square folded oak leg',(x,x+2,z,z+2),.35,8.25,'rail',.23)
   a.prism('Dark ferrule',(x-.1,x+2.1,z-.1,z+2.1),0,.6,'foot',.18)
   a.prism('Leg light outer facet',(x+.2,x+.52,z+.18,z+1.82),.75,7.8,'grain',.04)
 for z in[1.1,depth-2.1]:a.prism('Recessed full-width apron',(2,width-2,z,z+1),5.1,8.35,'rail',.15)
 for x in[1.1,width-2.1]:a.prism('Recessed side apron',(x,x+1,2,depth-2),5.1,8.35,'rail',.15)
 a.prism('Complete folded tabletop',(0,width,0,depth),8.1,9,'edge',.4)
 a.prism('Inset ochre writing surface',(.65,width-.65,.65,depth-.65),9,9.12,'oak',.25)
 # A quiet joint on the long table is original physical joinery, not a source
 # image pasted over the top. Small face seams remain under the beveled cap.
 if width>32:a.prism('Inlaid center timber joint',(width/2-.055,width/2+.055,.9,depth-.9),9.12,9.135,'grain',.025)
 if drawers:
  for x0,x1 in[(1.7,15.5),(16.5,width-1.7)]:
   a.prism('Shallow drawer box',(x0,x1,depth-6.4,depth-1.9),5.4,8.05,'shadow',.15)
   a.prism('Recessed drawer front',(x0+.2,x1-.2,depth-.98,depth-.68),5.8,7.8,'grain',.15)
   cx=(x0+x1)/2
   a.prism('Drawer slim brass grip',(cx-1.5,cx+1.5,depth-.67,depth-.36),6.6,6.97,'brass',.06)
def papers(a,x,z):
 # The book and open cup are independent original closed objects.
 # Only the small original source ink face is sampled live by the renderer.
 a.prism('Document folio dark cover',(x,x+7,z,z+7),9.13,9.46,'binding',.10)
 a.prism('Folio cream page edges',(x+.42,x+6.6,z+.3,z+6.65),9.46,9.71,'linen',.08)
 a.prism('Top writing leaf',(x+.55,x+6.5,z+.36,z+6.52),9.71,9.82,'paper',.04)
 a.prism('Folio folded spine',(x+.1,x+.45,z+.18,z+6.82),9.46,9.88,'rail',.05)
 cx=x+11.5;cz=z+3.5
 # A faceted open cup follows the small round white source silhouette. The
 # joined inner/outer wall and bottom are one closed printable paper shell.
 n=10;vs=[]
 for radius,y in[(1.62,9.16),(2.25,12.15),(1.91,12.15),(1.33,9.58)]:
  vs.extend((cx+radius*math.cos(i*math.tau/n),y,cz+radius*math.sin(i*math.tau/n))for i in range(n))
 faces=[]
 for ring in range(3):
  for i in range(n):
   j=(i+1)%n;faces.append((ring*n+i,(ring+1)*n+i,(ring+1)*n+j,ring*n+j))
 # The inner and outer bases are individually capped; the cavity stays open.
 faces.extend([tuple(range(n)),tuple(range(4*n-1,3*n-1,-1))])
 a.parts.append(('Faceted cup with true open interior','paper',vs,faces))
 # A quiet liquid inset sits below the open rim; no false cube fills the cup.
 a.prism('Recessed tea surface',(cx-1.0,cx+1.0,cz-1.0,cz+1.0),10.7,10.76,'rail',.8)
 # Open handle: top and bottom returns join one outer strap.
 a.prism('Cup handle upper return',(cx+1.85,cx+3.18,cz-.40,cz+.40),11.45,11.87,'linen',.13)
 a.prism('Cup handle lower return',(cx+1.55,cx+3.18,cz-.40,cz+.40),9.93,10.35,'linen',.13)
 a.prism('Cup handle open outer strap',(cx+2.83,cx+3.28,cz-.40,cz+.40),10.16,11.65,'paper',.13)

def square():
 a=Asset('facility_square_document_table');table(a,32,32);papers(a,8.5,18);return a
def meeting():
 a=Asset('facility_meeting_table');table(a,48,32);return a
def side():
 a=Asset('facility_document_side_desk');table(a,32,18,True);papers(a,8.5,8);return a
def chair():
 a=Asset('facility_square_back_chair')
 # Source actors stand at (8,16). This complete seat stays within x=2..14,
 # z=0..8, leaving their real feet and every yaw of their rig untouched.
 for x in[0.7,9.7]:
  for z in[.7,5.7]:
   a.prism('Square chair leg',(x,x+1.6,z,z+1.6),.25,5.8,'rail',.15)
   a.prism('Chair leg ferrule',(x-.08,x+1.68,z-.08,z+1.68),0,.4,'foot',.12)
 a.prism('Under-seat open-side front rail',(.9,11.1,6.1,6.8),3.5,5.6,'shadow',.12)
 a.prism('Folded square seat',(0,12,0,10),5.35,6.25,'edge',.35)
 a.prism('Inset quiet linen seat',(.65,11.35,1.4,9.35),6.25,6.58,'oak',.32)
 for x in[.7,9.9]:a.prism('Backrest oak upright',(x,x+1.4,.25,1.55),5.6,11.0,'rail',.15)
 a.prism('Square chair crest rail',(.1,11.9,0,1.85),10.05,11.4,'edge',.3)
 a.prism('Complete solid rear backrest',(.65,11.35,.15,1.4),7.5,10.2,'rail',.20)
 a.prism('Pale recessed backrest front',(1.6,10.4,1.41,1.69),8.0,9.78,'linen',.16)
 a.prism('Finished rear backrest inset',(1.6,10.4,.0,.14),8.0,9.78,'grain',.12)
 a.parts=[(label,mat,[(x,y,z*.8)for x,y,z in vs],faces)for label,mat,vs,faces in a.parts]
 return a
def assets():return[square(),meeting(),side(),chair()]
def document(a):
 ps=[]
 for label,mat,vs,faces in a.parts:
  p=dict(name=label,base_color=PALETTE[mat],positions=[],normals=[],indices=[])
  for f in faces:
   for k in range(1,len(f)-1):
    points=[vs[f[0]],vs[f[k]],vs[f[k+1]]];n=normal(*points)
    for v in points:p['indices'].append(len(p['positions'])//3);p['positions'].extend(v);p['normals'].extend(n)
  ps.append(p)
 coords=[p['positions'][i:i+3]for p in ps for i in range(0,len(p['positions']),3)]
 return dict(name=a.name,coordinate_system='right-handed; +Y up; front +Z',origin='northwest-floor',bounds=dict(min=[min(v[i]for v in coords)for i in range(3)],max=[max(v[i]for v in coords)for i in range(3)]),triangle_count=sum(len(p['indices'])//3 for p in ps),primitives=ps)
def save_runtime(a,out):
 raw=(json.dumps(document(a),separators=(',',':'))+'\n').encode()
 path=out/(a.name+'.mesh.json');path.write_bytes(raw);validate_model(path)
def build_blender(aa,out,skip_preview):
 import bpy
 from mathutils import Vector
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 for col in list(bpy.data.collections):bpy.data.collections.remove(col)
 scene=bpy.context.scene;mats={}
 for name,color in PALETTE.items():
  m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True;b=m.node_tree.nodes['Principled BSDF'];b.inputs['Base Color'].default_value=color;b.inputs['Roughness'].default_value=.80;mats[name]=m
 gallery=bpy.data.collections.new('PREVIEW | Facility tables and chairs');scene.collection.children.link(gallery)
 for index,a in enumerate(aa):
  col=bpy.data.collections.new('ASSET | '+a.name);scene.collection.children.link(col)
  for label,mat,vs,faces in a.parts:
   me=bpy.data.meshes.new(label);me.from_pydata([(x/16,-z/16,y/16)for x,y,z in vs],[],faces);me.update();me.materials.append(mats[mat]);ob=bpy.data.objects.new(label,me);col.objects.link(ob)
   display=ob.copy();display.data=me;gallery.objects.link(display);display.location=[(0,0,0),(3.5,0,0),(0,-3.0,0),(3.5,-3.0,0)][index]
  col.hide_render=True;col.hide_viewport=True;col['runtime_sha256']=hashlib.sha256((out/(a.name+'.mesh.json')).read_bytes()).hexdigest()
 center=(3,-2,.4);distance=8.5
 bpy.ops.mesh.primitive_plane_add(size=100,location=(3,-2,-.014));bpy.context.object.name='PREVIEW | Neutral studio floor';m=bpy.data.materials.new('Neutral warm grey');m.diffuse_color=(.23,.27,.25,1);bpy.context.object.data.materials.append(m)
 scene.world.color=(.22,.24,.23)
 for name,offset,power,size in[('Key',(-5,-8,12),1600,9),('Fill',(10,8,7),1100,9)]:
  bpy.ops.object.light_add(type='AREA',location=tuple(c+o for c,o in zip(center,offset)));o=bpy.context.object;o.name='PREVIEW | '+name;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector(center)-o.location).to_track_quat('-Z','Y').to_euler()
 bpy.ops.object.camera_add(location=(center[0]+distance,center[1]-distance,center[2]+distance*.8));camera=bpy.context.object;camera.name='PREVIEW | Camera';camera.data.type='ORTHO';camera.data.ortho_scale=distance;scene.camera=camera;camera.rotation_euler=(Vector(center)-camera.location).to_track_quat('-Z','Y').to_euler()
 scene.render.engine='CYCLES';scene.cycles.samples=8;scene.cycles.use_denoising=False;scene.render.resolution_x=1200;scene.render.resolution_y=800;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX';scene['provenance']='Original geometric sculpture; no imported game art';scene['rebuild']='tools/build-facility-tables.py'
 bpy.ops.wm.save_as_mainfile(filepath=str(out/'facility-tables.blend'),compress=False)
 if not skip_preview:
  for view,offset in [('front',(distance,-distance,distance*.8)),('rear',(-distance,distance,distance*.8)),('side',(distance,0,distance*.45))]:
   camera.location=tuple(c+o for c,o in zip(center,offset));camera.rotation_euler=(Vector(center)-camera.location).to_track_quat('-Z','Y').to_euler();scene.render.filepath=str(out/('facility-tables-'+view+'.png'));bpy.ops.render.render(write_still=True)
def main():
 args=sys.argv[sys.argv.index('--')+1:]if '--'in sys.argv else sys.argv[1:];out=Path(next((a for a in args if not a.startswith('--')),'target/facility-tables'));out.mkdir(parents=True,exist_ok=True);aa=assets()
 for a in aa:save_runtime(a,out);print(a.name,document(a)['triangle_count'],'triangles',document(a)['bounds'])
 if '--runtime-only'not in args:build_blender(aa,out,'--skip-preview'in args)
if __name__=='__main__':main()
