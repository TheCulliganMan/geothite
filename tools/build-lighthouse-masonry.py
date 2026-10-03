"""Original low-poly lighthouse ashlar and recessed windows, no source artwork.

blender -b --threads 2 --python tools/build-lighthouse-masonry.py -- OUTPUT
python3 tools/build-lighthouse-masonry.py OUTPUT --runtime-only

Three reusable unit volumes share an exact outer seam. Each primitive belongs
only to one cardinal side, the cap or underside. The renderer omits shared
sides, so every complete source drawing supplies its own corners and junctions.
"""
from pathlib import Path
import hashlib, json, math, sys
sys.path.insert(0,str(Path(__file__).resolve().parent))
from model_asset_storage import validate_model, stored_model_size

PALETTE = {
    'warm_limestone': (.56, .53, .43, 1.),
    'chalk_highlight': (.71, .68, .56, 1.),
    'cool_ashlar': (.48, .50, .45, 1.),
    'cut_stone': (.63, .60, .49, 1.),
    'recess_mortar': (.32, .34, .31, 1.),
    'window_reveal': (.27, .31, .30, 1.),
    'deep_sea_glass': (.19, .38, .43, 1.),
    'glass_highlight': (.32, .54, .58, 1.),
    'aged_bronze': (.48, .41, .25, 1.),
}
ASSETS = ['ashlar', 'window_single', 'window_double']


def sub(a,b): return tuple(x-y for x,y in zip(a,b))
def cross(a,b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
def normal(a,b,c):
    n=cross(sub(b,a),sub(c,a)); d=math.sqrt(sum(v*v for v in n))
    assert d>1e-10
    return tuple(v/d for v in n)
def rotate(v, turns):
    x,y,z=v
    for _ in range(turns): x,z=1-z,x
    return (x,y,z)
def side_point(u,y,z,side): return rotate((u,y,z),(side-2)%4)


class Asset:
    def __init__(self,name): self.name=name; self.faces=[]
    def face(self, points, material, side, label):
        # Caller supplies outward winding. The source retains these polygons.
        self.faces.append((tuple(tuple(round(v,8) for v in p) for p in points),material,side,label))
    def quad(self,a,b,c,d,material,side,label): self.face([a,b,c,d],material,side,label)
    def panel(self,x0,x1,y0,y1,z0,z1,material,side,label,bevel=True):
        """Connected inset ashlar face; exact border stays on the unit shell."""
        def depth(x,y): return 1. if x in (0.,1.) or y in (0.,1.) else z0
        outer=[side_point(x0,y0,depth(x0,y0),side),side_point(x1,y0,depth(x1,y0),side),side_point(x1,y1,depth(x1,y1),side),side_point(x0,y1,depth(x0,y1),side)]
        if not bevel:
            self.face(outer,material,side,label);return
        inset=min(.013,(x1-x0)*.12,(y1-y0)*.13)
        inner=[side_point(x0+inset,y0+inset,z1,side),side_point(x1-inset,y0+inset,z1,side),side_point(x1-inset,y1-inset,z1,side),side_point(x0+inset,y1-inset,z1,side)]
        for i in range(4):
            j=(i+1)%4
            self.face([outer[i],outer[j],inner[j],inner[i]],material,side,label+' | dressed bevel')
        self.face(inner,material,side,label+' | broad face')


def subtract_rect(rect,hole):
    x0,x1,y0,y1=rect;a,b,c,d=hole
    ix0,ix1=max(x0,a),min(x1,b);iy0,iy1=max(y0,c),min(y1,d)
    if ix0>=ix1 or iy0>=iy1:return [rect]
    return [r for r in [(x0,ix0,y0,y1),(ix1,x1,y0,y1),(ix0,ix1,y0,iy0),(ix0,ix1,iy1,y1)] if r[1]-r[0]>1e-7 and r[3]-r[2]>1e-7]


def wall(asset,side,windows):
    holes=[(x-.145,x+.145,.32,.815) for x in windows]
    bands=[(0.,.11,False),(.11,.37,True),(.37,.63,True),(.63,.89,True),(.89,1.,False)]
    for row,(y0,y1,stone) in enumerate(bands):
        cuts=[0.,.25,.625,1.] if row%2 else [0.,.375,.75,1.]
        if not stone:cuts=[0.,.5,1.]
        cuts=[.035+.93*x for x in cuts]
        for col,(x0,x1) in enumerate(zip(cuts,cuts[1:])):
            rectangles=[(x0,x1,y0,y1)]
            for hole in holes:rectangles=[piece for r in rectangles for piece in subtract_rect(r,hole)]
            for piece in rectangles:
                material=(['warm_limestone','cut_stone','cool_ashlar'][(row+col)%3] if stone else 'chalk_highlight')
                asset.panel(*piece,.976,.997,material,side,f'{side} | course {row+1} stone {col+1}')
    # Fixed-width quoins keep every staggered course on the identical seam.
    for x0,x1 in [(0.,.035),(.965,1.)]:
        for y0,y1,_ in bands:
            asset.panel(x0,x1,y0,y1,.976,.997,'cut_stone',side,f'{side} | continuous edge quoin',False)
    for number,(x0,x1,y0,y1) in enumerate(holes):
        # Four sloping reveals produce a real depth change. Glazing is recessed
        # inside the wall, never pasted onto the stone front. The bronze bars
        # divide that opening into four opaque panes with physical bevels.
        q=[side_point(x0,y0,.976,side),side_point(x1,y0,.976,side),side_point(x1,y1,.976,side),side_point(x0,y1,.976,side)]
        r=[side_point(x0+.025,y0+.027,.785,side),side_point(x1-.025,y0+.027,.785,side),side_point(x1-.025,y1-.027,.785,side),side_point(x0+.025,y1-.027,.785,side)]
        for i in range(4):
            j=(i+1)%4;asset.face([q[i],q[j],r[j],r[i]],'window_reveal' if i in (1,3) else 'chalk_highlight',side,f'{side} | window {number+1} stone reveal')
        a,b=x0+.025,x1-.025;c,d=y0+.027,y1-.027;mx=(a+b)/2;my=(c+d)/2;bar=.014
        # A conforming 3x3 subdivision seals the aperture without floating bars.
        xx=[a,mx-bar/2,mx+bar/2,b];yy=[c,my-bar/2,my+bar/2,d]
        for iy in range(3):
            for ix in range(3):
                mat='aged_bronze' if ix==1 or iy==1 else ('glass_highlight' if ix==0 and iy==2 else 'deep_sea_glass')
                z=.801 if ix==1 or iy==1 else .785
                asset.panel(xx[ix],xx[ix+1],yy[iy],yy[iy+1],.785,z,mat,side,f'{side} | window {number+1} '+('bronze mullion' if ix==1 or iy==1 else 'recessed glazing'),ix==1 or iy==1)


def make_assets():
    assets=[]
    for name in ASSETS:
        a=Asset(name)
        for side in range(4):wall(a,side,([.25] if name=='window_single' else [.25,.75]) if side==2 and name!='ashlar' else [])
        # Closed crown/underside have cardinal boundaries identical to the wall.
        a.face([(0,1,0),(0,1,1),(1,1,1),(1,1,0)],'chalk_highlight',4,'Crown | continuous broad dressed stone')
        a.face([(0,0,0),(1,0,0),(1,0,1),(0,0,1)],'recess_mortar',5,'Underside | sealed original footprint')
        assets.append(a)
    return assets


def conform_faces(asset):
    """Split only collinear polygon edges so edited reveals remain watertight."""
    points=list(dict.fromkeys(p for face,*_ in asset.faces for p in face))
    result=[]
    for face,mat,side,label in asset.faces:
        ring=[]
        for a,b in zip(face,face[1:]+face[:1]):
            d=sub(b,a);length2=sum(v*v for v in d);on=[]
            for p in points:
                t=sum(v*w for v,w in zip(sub(p,a),d))/length2
                if -1e-7<=t<1-1e-7 and sum((p[i]-a[i]-t*d[i])**2 for i in range(3))<1e-14:on.append((t,p))
            ring.extend(p for _,p in sorted(on))
        result.append((ring,mat,side,label))
    return result


def triangles(face):
    # Convex ear clipping retains every subdivided boundary edge while avoiding
    # a costly new centre vertex on each dressed stone and window reveal.
    ring=list(face);result=[]
    while len(ring)>3:
        for i,b in enumerate(ring):
            a=ring[i-1];c=ring[(i+1)%len(ring)]
            n=cross(sub(b,a),sub(c,a))
            if sum(v*v for v in n)>1e-16:
                # Do not eat an ear whose diagonal contains another boundary
                # point. This preserves each small matching seam triangle.
                ac=sub(c,a);length2=sum(v*v for v in ac)
                blocked=False
                for j,p in enumerate(ring):
                    if j in (i,(i-1)%len(ring),(i+1)%len(ring)):continue
                    t=sum(v*w for v,w in zip(sub(p,a),ac))/length2
                    if 1e-7<t<1-1e-7 and sum((p[k]-a[k]-t*ac[k])**2 for k in range(3))<1e-14:blocked=True;break
                if blocked:continue
                result.append((a,b,c));ring.pop(i);break
        else:raise AssertionError('degenerate authoring polygon')
    result.append(tuple(ring));return result


def document(asset):
    groups={}
    for face,mat,side,label in conform_faces(asset):
        p=groups.setdefault((side,mat),{'side':side,'material':mat,'positions':[],'normals':[],'indices':[],'base_color':PALETTE[mat]})
        for tri in triangles(face):
            n=normal(*tri);base=len(p['positions'])//3
            for v in tri:p['positions'].extend(round(f,7) for f in v);p['normals'].extend(round(f,7) for f in n)
            p['indices'].extend([base,base+1,base+2])
    return {'format':'geothite-lighthouse-masonry-v1','name':asset.name,'bounds':[0,0,0,1,1,1],'primitives':list(groups.values())}


def save_runtime(asset,output):
    raw=(json.dumps(document(asset),separators=(',',':'))+'\n').encode()
    path=output/(asset.name+'.mesh.json');path.write_bytes(raw);validate_model(path)


def build_blender(assets,output,skip_preview):
    import bpy
    from mathutils import Vector
    bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
    for coll in list(bpy.data.collections):
        if coll.name!='Collection':bpy.data.collections.remove(coll)
    scene=bpy.context.scene;materials={}
    for name,color in PALETTE.items():
        m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True
        bs=m.node_tree.nodes['Principled BSDF'];bs.inputs['Base Color'].default_value=color;bs.inputs['Roughness'].default_value=.77
        materials[name]=m
    collections={}
    for a in assets:
        coll=bpy.data.collections.new('ASSET | '+a.name);scene.collection.children.link(coll);collections[a.name]=coll
        groups={}
        for face,mat,side,label in conform_faces(a):groups.setdefault(label,[]).append((face,mat,side))
        for label,faces in groups.items():
            vs=[];fs=[];slots=[]
            for face,mat,side in faces:
                fs.append(tuple(range(len(vs),len(vs)+len(face))));vs.extend((x,-z,y) for x,y,z in face);slots.append(list(materials).index(mat))
            me=bpy.data.meshes.new(a.name+' | '+label);me.from_pydata(vs,[],fs);me.update()
            for m in materials.values():me.materials.append(m)
            for poly,slot in zip(me.polygons,slots):poly.material_index=slot;poly.use_smooth=False
            obj=bpy.data.objects.new(a.name+' | '+label,me);coll.objects.link(obj);obj['side']=faces[0][2];obj['authored_part']=label
        coll['runtime_sha256']=hashlib.sha256((output/(a.name+'.mesh.json')).read_bytes()).hexdigest()
        coll['source_contract']='A whole 4x4 exact lighthouse drawing; no navigation, collision, actor or warp edits'
    scene['kit']='Three original unit shells, dressed ashlar, deep window reveals and bronze mullions'
    scene['runtime_coordinate_system']='Y-up; unit bounds [0,1]; south is +Z; rotations preserve original footprint'
    scene['rebuild']='tools/build-lighthouse-masonry.py'
    # Editable prototypes remain in place but do not clutter the presentation.
    for coll in collections.values():coll.hide_render=True;coll.hide_viewport=True
    gallery=bpy.data.collections.new('PREVIEW | Joined architecture and preserved opening');scene.collection.children.link(gallery)
    def instance(name,x,y,turn=0):
        for src in collections[name].objects:
            ob=src.copy();ob.data=src.data;gallery.objects.link(ob)
            # Preview composes the same authored shell meshes. Full shared sides
            # are omitted exactly as runtime does via each part's side property.
            ob.location=(x,y,0);ob.rotation_euler[2]=-turn*math.pi/2
            if turn==1:ob.location.x+=1
            elif turn==2:ob.location.x+=1;ob.location.y-=1
            elif turn==3:ob.location.y-=1
            if turn>=2:
                ob.scale.x=-1
                if turn==2:ob.location.x-=1
                else:ob.location.y+=1
        return
    # Three named prototypes plus a low T-junction and two courses with a full
    # one-block opening. No opaque plane fills that architectural opening.
    layout=[('ashlar',0,0,0),('window_single',1,0,0),('window_double',2,0,0),('ashlar',3,0,0),
            ('ashlar',0,-1,0),('ashlar',3,-1,0),('ashlar',0,-2,0),('ashlar',3,-2,0),
            ('ashlar',0,-3,2),('window_single',2,-3,2),('ashlar',3,-3,0),
            ('ashlar',5,-1,0),('ashlar',6,-1,0),('ashlar',7,-1,0),('ashlar',6,-2,0)]
    occupied={(x,y) for _,x,y,_ in layout}
    for name,x,y,turn in layout:
        start=set(gallery.objects);instance(name,x,y,turn)
        for ob in set(gallery.objects)-start:
            side=ob['side']
            if side<4:
                world=(((4-side)%4 if turn>=2 else side)+turn)%4;dx,dy=[(0,1),(1,0),(0,-1),(-1,0)][world]
                if (x+dx,y+dy) in occupied:ob.hide_render=True
    bpy.ops.mesh.primitive_plane_add(size=200,location=(3,-1,-.015));floor=bpy.context.object;floor.name='PREVIEW | neutral floor'
    m=bpy.data.materials.new('Preview floor');m.diffuse_color=(.13,.16,.18,1);m.use_nodes=True;m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=m.diffuse_color;floor.data.materials.append(m)
    scene.world.color=(.15,.15,.15)
    scene.render.engine='CYCLES';scene.cycles.samples=32;scene.cycles.use_denoising=False
    scene.render.resolution_x=1600;scene.render.resolution_y=1000;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX'
    for label,power,size,location in [('Key',1900,8,(1,-5,9)),('Fill',1100,7,(7,3,7))]:
        bpy.ops.object.light_add(type='AREA',location=location);o=bpy.context.object;o.name='PREVIEW | '+label;o.data.energy=power;o.data.shape='DISK';o.data.size=size;o.rotation_euler=(Vector((3,-1,0.5))-o.location).to_track_quat('-Z','Y').to_euler()
    bpy.ops.object.camera_add(location=(10,-12,10));camera=bpy.context.object;camera.name='PREVIEW | Camera';camera.data.type='ORTHO';camera.data.ortho_scale=11.4;scene.camera=camera
    def camera_at(pos,target):camera.location=pos;camera.rotation_euler=(Vector(target)-camera.location).to_track_quat('-Z','Y').to_euler()
    camera_at((10,-12,10),(3.7,-1.45,.3))
    bpy.ops.wm.save_as_mainfile(filepath=str(output/'lighthouse-masonry.blend'),compress=False)
    if not skip_preview:
        for name,pos in [('front',(10,-12,10)),('rear',(-4,8,5.6))]:
            camera_at(pos,(3.7,-1.45,.3));scene.render.filepath=str(output/('lighthouse-masonry-'+name+'.png'));bpy.ops.render.render(write_still=True)
    print('Blender source and previews:',output)


def main():
    args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else sys.argv[1:]
    output=Path(next((a for a in args if not a.startswith('--')),'target/lighthouse-masonry'));output.mkdir(parents=True,exist_ok=True)
    assets=make_assets()
    for a in assets:save_runtime(a,output)
    if '--runtime-only' not in args:build_blender(assets,output,'--skip-preview' in args)
    for a in assets:
        d=document(a);print(a.name, sum(len(p['indices'])//3 for p in d['primitives']),'triangles', stored_model_size(output/(a.name+'.mesh.json')),'bytes')

if __name__=='__main__':main()
