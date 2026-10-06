#!/usr/bin/env python3
"""Author Spearow as a small, alert folded-paper bird with a real glTF skin.

The only geometry artifact is the canonical GLB. This deterministic standard-
library recipe is the editable source: no copied Pidgeotto anatomy, Blender
scene, extracted game data, or secondary neutral mesh. +Y up, front +Z.
"""
import argparse
import hashlib
import math
from pathlib import Path

from skin_glb import COORDINATES, Glb, envelope, packed, quantized_weights, quaternion_xyz, smoothstep

FILE = 'spearow.glb'
MAX_BYTES = 250000
HEIGHT = .75
CLIPS = (('idle', 2.8, 73), ('attack', 1., 65), ('hit', 1., 65))
JOINTS = (
    ('root', None, (0., 0., 0.)),
    ('torso', 0, (0., .165, -.020)),
    ('head', 1, (0., .435, .075)),
    ('lower_beak', 2, (0., .529, .257)),
    ('wing_left', 1, (-.167, .438, .012)),
    ('flight_feathers_left', 4, (-.260, .321, -.052)),
    ('wing_right', 1, (.167, .438, .012)),
    ('flight_feathers_right', 6, (.260, .321, -.052)),
    ('tail_fan', 1, (0., .226, -.154)),
    ('crown', 2, (0., .638, .064)),
)
# Colors are chosen in sRGB, then stored as the linear factors expected by the
# runtime. The broad coral and ivory areas must survive a distant battle view.
PALETTE = {
    'russet': (.53, .31, .14), 'crown_ridge': (.67, .42, .20),
    'cream': (.96, .85, .61), 'cream_shadow': (.81, .67, .43),
    'coral': (.90, .29, .31), 'coral_light': (.99, .45, .45),
    'wine_fold': (.65, .18, .24), 'pink_beak': (.97, .52, .55),
    'pink_feet': (.87, .39, .42), 'beak_shadow': (.74, .29, .35),
    'ivory': (.99, .97, .86), 'ink': (.12, .08, .07),
    'tail': (.43, .25, .13), 'claw': (.94, .88, .71),
}


def linear(c):
    return [v/12.92 if v <= .04045 else ((v+.055)/1.055)**2.4 for v in c]+[1.]


def add(a, b): return tuple(x+y for x, y in zip(a, b))
def sub(a, b): return tuple(x-y for x, y in zip(a, b))
def mul(a, k): return tuple(x*k for x in a)
def dot(a, b): return sum(x*y for x, y in zip(a, b))
def cross(a, b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
def unit(a): return mul(a, 1/math.sqrt(dot(a, a)))
def mean(vs): return tuple(sum(p[k] for p in vs)/len(vs) for k in range(3))


class Sculpture:
    def __init__(self): self.parts = {}

    def mesh(self, name, color, vertices, faces, smooth=False, outward=None):
        """Build a closed volume, or a fitted face patch with a known normal."""
        center = mean(vertices)
        triangles = []
        for face in faces:
            for j in range(1, len(face)-1):
                tri = [face[0], face[j], face[j+1]]
                a, b, c = (vertices[i] for i in tri)
                normal = cross(sub(b, a), sub(c, a))
                if dot(normal, normal) < 1e-18: continue
                facing = outward if outward is not None else sub(mean([a, b, c]), center)
                if dot(normal, facing) < 0: tri[1], tri[2] = tri[2], tri[1]
                triangles.append(tri)
        norms = [(0., 0., 0.) for _ in vertices]
        for tri in triangles:
            a, b, c = (vertices[i] for i in tri)
            n = cross(sub(b, a), sub(c, a))
            for i in tri: norms[i] = add(norms[i], n)
        part = self.parts.setdefault(name, {'part': name, 'color': color,
                    'positions': [], 'normals': [], 'indices': [], 'base_color': linear(PALETTE[color])})
        assert part['color'] == color
        if smooth:
            base = len(part['positions'])//3
            part['positions'].extend(v for p in vertices for v in p)
            part['normals'].extend(v for n in norms for v in unit(n))
            part['indices'].extend(base+i for tri in triangles for i in tri)
        else:
            for tri in triangles:
                a, b, c = (vertices[i] for i in tri)
                n = unit(cross(sub(b, a), sub(c, a)))
                base = len(part['positions'])//3
                part['positions'].extend(v for p in (a, b, c) for v in p)
                part['normals'].extend(n*3)
                part['indices'].extend((base, base+1, base+2))

    def loft(self, name, color, profiles, segments=24):
        # Each ring stores height, half-width, fore/aft radius, center Z.
        vertices = [(rx*math.sin(math.tau*i/segments), y,
                     z+rz*math.cos(math.tau*i/segments))
                    for y, rx, rz, z in profiles for i in range(segments)]
        faces = [tuple(reversed(range(segments))), tuple((len(profiles)-1)*segments+i for i in range(segments))]
        for ring in range(len(profiles)-1):
            for i in range(segments):
                a, b = ring*segments+i, ring*segments+(i+1)%segments
                faces.extend(((a, b, b+segments), (a, b+segments, a+segments)))
        self.mesh(name, color, vertices, faces, True)

    def fold(self, name, color, outline, ridge, depth=.008):
        # A solid paper panel with a deliberately raised center crease. Flat
        # face normals belong to the fold; the body uses smooth normal fields.
        center = mean(outline)
        normal = unit(sub(ridge, center))
        vertices = list(outline)+[ridge, sub(center, mul(normal, depth))]
        count = len(outline)
        faces = [(i, (i+1)%count, count) for i in range(count)]
        faces += [((i+1)%count, i, count+1) for i in range(count)]
        self.mesh(name, color, vertices, faces)

    def tube(self, name, color, points, radii, segments=8):
        vertices, faces = [], []
        for k, point in enumerate(points):
            tangent = unit(sub(points[min(k+1, len(points)-1)], points[max(0,k-1)]))
            seed = (0., 1., 0.) if abs(tangent[1]) < .85 else (1., 0., 0.)
            u = unit(cross(tangent, seed)); v = cross(tangent, u)
            vertices.extend(add(point, mul(add(mul(u, math.cos(i*math.tau/segments)),
                          mul(v, math.sin(i*math.tau/segments))), radii[k])) for i in range(segments))
        faces += [tuple(reversed(range(segments))), tuple((len(points)-1)*segments+i for i in range(segments))]
        for k in range(len(points)-1):
            for i in range(segments):
                a, b = k*segments+i, k*segments+(i+1)%segments
                faces.append((a,b,b+segments,a+segments))
        self.mesh(name, color, vertices, faces, True)


def sculpture():
    s = Sculpture()
    # Low, plump body and compact neck. The silhouette is deliberately not a
    # thin songbird: Spearow has a large head, broad chest, and short tail fan.
    s.loft('Torso / warm buff continuous breast and belly', 'cream', [
        (.125,.045,.045,-.035),(.152,.099,.085,-.025),(.190,.145,.114,-.018),
        (.245,.176,.143,-.006),(.300,.185,.156,.007),(.355,.176,.153,.023),
        (.405,.148,.128,.044),(.449,.108,.091,.064),(.465,.062,.050,.077)])
    # Curved back mantle leaves a broad light chest; this fitted rear patch has
    # no independent animation and shares its supporting body's weight field.
    profiles = [(.17,.116,.103,-.021),(.245,.179,.146,-.006),(.30,.188,.159,.007),
                (.355,.179,.156,.023),(.407,.151,.132,.044),(.45,.111,.095,.064)]
    vs=[]
    for y,rx,rz,z in profiles:
        for i in range(13):
            angle=math.pi*.5+math.pi*i/12
            vs.append((rx*math.sin(angle),y,z+rz*math.cos(angle)))
    fs=[(r*13+i,r*13+i+1,(r+1)*13+i+1,(r+1)*13+i) for r in range(len(profiles)-1) for i in range(12)]
    s.mesh('Torso / fitted chestnut back mantle','russet',vs,fs,True,outward=(0,0,-1))
    # A broad wedge-shaped head with a short forehead and pointed feathered
    # lower cheek. Its compact top leaves room for Spearow's brown crown.
    head_profiles=[
        (.401,.024,.037,.135),(.426,.067,.085,.124),(.460,.116,.123,.119),
        (.500,.153,.154,.123),(.545,.164,.160,.132),(.585,.147,.145,.120),
        (.623,.115,.123,.108),(.653,.077,.092,.076),(.668,.034,.040,.054)]
    s.loft('Head / chestnut cheek and forehead','russet',head_profiles)
    # Short backward-swept brown paper spikes, never the red Pidgeotto crest.
    for i,(x,tip_y,tip_z) in enumerate(((-.105,.720,-.022),(-.040,.750,-.006),(.040,.731,-.014),(.112,.702,-.028))):
        a=(x*.72,.627,.121); b=(x,tip_y,tip_z)
        outline=[(a[0]-.034,a[1],a[2]),(a[0]-.031,a[1]+.043,a[2]-.025),
                 b,(a[0]+.034,a[1]+.037,a[2]-.049),(a[0]+.034,a[1]-.008,a[2]-.015)]
        s.fold('Crown / four swept chestnut paper points','russet',outline,(x,.676,.079),.013)
        s.fold('Crown / warm folded highlights','crown_ridge',
               [(x-.011,.661,.092),b,(x+.016,.663,.055)],(x+.002,.682,.061),.002)
    s.fold('Head / swept cheek and throat feather points','russet',
        [(-.101,.455,.223),(-.075,.402,.189),(-.050,.414,.201),
         (-.010,.347,.179),(.020,.401,.190),(.060,.376,.183),
         (.075,.425,.206),(.101,.455,.223)],(0.,.421,.217),.005)
    for side,label in ((-1,'left'),(1,'right')):
        # Rear cheek points continue the head silhouette past the nape.
        for y,z in ((.574,-.015),(.492,.008)):
            s.fold('Head / swept cheek and throat feather points','russet',
                [(side*.120,y+.030,z+.064),(side*.193,y+.030,z-.046),
                 (side*.157,y-.034,z+.007),(side*.120,y-.070,z+.071)],
                 (side*.167,y,z+.025),.010)
        # Both eyes wrap the head. Large ivory negative space plus a squared
        # forward pupil and a heavy slanted brow read in the actual battle view.
        def eye_point(phi,y,offset=.002):
            # Fit the supporting authored loft itself, not an approximate
            # sphere. A tiny paper thickness avoids floating eye-sticker edges.
            for a,b in zip(head_profiles,head_profiles[1:]):
                if a[0]<=y<=b[0]:
                    t=(y-a[0])/(b[0]-a[0]);rx,rz,z=[x+(q-x)*t for x,q in zip(a[1:],b[1:])]
                    return (side*(rx+offset)*math.sin(phi),y,z+(rz+offset)*math.cos(phi))
            raise ValueError('eye outside the supporting cheek loft')
        # Subdivide around the cheek curvature so a flat chord cannot bury
        # the center of the white eye beneath its supporting head surface.
        ev=[]
        for row in range(3):
            t=row/2
            for col in range(7):
                u=col/6
                phi=(.50+.56*u)*(1-t)+(.53+.56*u)*t
                y=(.595+.019*u)*(1-t)+(.559+.011*u)*t
                ev.append(eye_point(phi,y,.0035))
        ef=[(r*7+i,r*7+i+1,(r+1)*7+i+1,(r+1)*7+i) for r in range(2) for i in range(6)]
        s.mesh(f'Eye {label} / angular ivory sclera','ivory',ev,ef,True,outward=(side,0,1))
        pv=[]
        for row in range(3):
            t=row/2
            for col in range(4):
                u=col/3
                phi=(.58+.17*u)*(1-t)+(.60+.17*u)*t
                y=(.594+.006*u)*(1-t)+(.561+.003*u)*t
                pv.append(eye_point(phi,y,.0055))
        pf=[(r*4+i,r*4+i+1,(r+1)*4+i+1,(r+1)*4+i) for r in range(2) for i in range(3)]
        s.mesh(f'Eye {label} / forward square black pupil','ink',pv,pf,True,outward=(side,0,1))
        s.mesh('Head / sharp russet brow rims','russet',
               [eye_point(a,y,.007) for a,y in ((.47,.604),(1.10,.628),(1.08,.611),(.50,.589))],
               [(0,1,2,3)],outward=(side,0,1))
        # Wing covert: broad coral surface, not a collection of thin rods.
        pts=[(side*.171,.441,.040),(side*.239,.464,-.024),(side*.314,.427,-.101),
             (side*.345,.336,-.151),(side*.300,.235,-.086),(side*.213,.275,.053)]
        s.fold(f'Wing {label} / broad coral upper coverts','coral',pts,(side*.338,.359,-.028),.014)
        # Ivory upper patches are paper shapes tucked into the leading edge.
        for i in range(3):
            y=.442-i*.021; z=.011-i*.036; x=side*(.259+i*.027)
            s.fold(f'Wing {label} / ivory covert tips','ivory',
                [(x,y+.016,z+.020),(x+side*.034,y-.003,z-.016),
                 (x+side*.025,y-.053,z-.020),(x-side*.014,y-.031,z+.025)],
                 (x+side*.031,y-.017,z+.005),.003)
        for i in range(4):
            x=side*(.239+i*.021); z=.043-i*.050
            top=(x,.368-i*.006,z)
            tip=(side*(.274+i*.019),.182+i*.010,z-.067)
            outline=[(top[0]-side*.010,top[1],top[2]+.038),
                (top[0]+side*.031,top[1]-.010,top[2]-.039),
                (tip[0]+side*.017,tip[1]+.016,tip[2]-.034),
                (tip[0],tip[1],tip[2]-.009),
                (tip[0]-side*.012,tip[1]+.017,tip[2]+.026),
                (top[0]-side*.012,top[1]-.048,top[2]+.033)]
            s.fold(f'Wing {label} / four overlapping flight feathers','coral_light',outline,
                   (side*(.320+i*.016),.290-i*.002,z-.020),.009)
            s.fold(f'Wing {label} / flight feather crease shadows','wine_fold',
                [top,(side*(.299+i*.020),.278,z-.030),tip],
                (side*(.304+i*.020),.291,z-.023),.001)
        # Bent, planted pink ankles and three forward toes plus a rear hallux.
        x=side*.105
        s.tube('Feet / planted rose ankles','pink_feet',[(x,.187,-.028),(x,.106,-.009),(x,.047,.025)],[.026,.019,.021])
        for i in (-1,0,1):
            end=(x+i*.052,.013,.178-abs(i)*.023)
            pts=[(x,.039,.027),(x+i*.025,.025,.085),end]
            s.tube('Feet / six splayed forward toes','pink_feet',pts,[.016,.014,.008],8)
            # Ivory terminal claws contact y=0 exactly; the nail is a short
            # solid tapered wedge and cannot sink when upper joints animate.
            tip=(end[0]+i*.014,0.,end[2]+.031)
            s.fold('Feet / ivory grounded toe claws','claw',
               [(end[0]-.009,.020,end[2]-.010),(end[0]+.009,.020,end[2]-.010),tip],
               (end[0],.025,end[2]+.004),.003)
        s.tube('Feet / two backward gripping toes','pink_feet',
               [(x,.039,.018),(x+side*.014,.025,-.037),(x+side*.022,.014,-.083)],[.014,.012,.004],8)
    # Pink wedge beak with a real lower-beak hinge and a thin dark mouth line.
    s.mesh('Beak / short rose upper wedge','pink_beak',
        [(-.076,.561,.273),(.076,.561,.273),(-.067,.526,.281),(.067,.526,.281),
         (0.,.552,.425),(0.,.589,.307)],[(0,5,1),(0,2,4),(1,4,3),(0,4,5),(1,5,4),(2,3,4),(0,1,3,2)])
    s.mesh('Beak / lower hinged rose wedge','beak_shadow',
        [(-.059,.521,.278),(.059,.521,.278),(0.,.490,.287),(0.,.525,.410)],
        [(0,1,2),(0,2,3),(1,3,2),(0,3,1)])
    s.mesh('Beak / inset mouth seam','ink',
        [(-.059,.524,.284),(.059,.524,.284),(0.,.529,.411)],[(0,1,2)],outward=(0,1,1))
    # Three short, squared chestnut tail feathers sit as a horizontal fan.
    for i in (-1,0,1):
        x=i*.079
        outline=[(i*.023-.023,.246,-.147),(i*.023+.023,.246,-.147),
                 (x+.045,.225,-.376+abs(i)*.028),(x+.025,.194,-.425+abs(i)*.026),
                 (x-.035,.197,-.419+abs(i)*.026),(x-.048,.222,-.369+abs(i)*.028)]
        s.fold('Tail / three squared chestnut fan feathers','tail',outline,(x,.263,-.285),.011)
        s.fold('Tail / warm raised fold ridges','crown_ridge',
               [(i*.02,.251,-.17),(x+.012,.264,-.291),(x+.021,.211,-.411+abs(i)*.026),
                (x-.004,.226,-.357)],(x+.007,.268,-.293),.002)
    return {'name':'spearow','version':1,'coordinate_system':COORDINATES,'primitives':list(s.parts.values())}


def skin_weights(part, position):
    x,y,z=position
    if part.startswith('Feet /'): return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Crown /'): return [9,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Beak / lower'): return [3,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Head /','Eye ','Beak /')): return [2,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Tail /'): return [8,0,0,0],[1.,0.,0.,0.]
    if part.startswith('Wing '):
        shoulder,wrist=(4,5) if 'left' in part else (6,7)
        # Each layered flight-feather assembly is rigid paper. Its color
        # crease must use exactly the same hinge, avoiding thin-fold inversion.
        joint=wrist if 'flight feather' in part else shoulder
        return [joint,0,0,0],[1.,0.,0.,0.]
    # Lower belly is planted with the foot-root. The chest can settle forward
    # without detaching from the stationary ankle attachments.
    upper=smoothstep(.150,.255,y)
    return [0,1 if upper else 0,0,0],[1.-upper,upper,0.,0.]


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.: raise ValueError('invalid clip or time')
    if t in (0.,1.): return [(0.,0.,0.)]*len(JOINTS)
    a,b=math.sin(math.tau*t),math.sin(2*math.tau*t)
    pulse=.5-.5*math.cos(math.tau*t)
    if clip=='idle':
        # Quick avian glance, chest breath, alternating folded-wing adjustment,
        # and a delayed fan twitch. The stance stays alert rather than flapping.
        glance=envelope(t,((0.,0.),(.16,0.),(.28,1.),(.42,.85),(.57,0.),(.73,-.48),(.90,0.),(1.,0.)))
        return [(0,0,0),(.017*a,0,.008*b),(-.040*pulse,.105*glance,-.018*glance),
                (.022*pulse,0,0),(-.038*pulse,-.018*a,-.048*pulse),(.025*a,0,-.029*pulse),
                (-.028*pulse,.015*a,.037*pulse),(-.018*b,0,.023*pulse),
                (.025*a,.040*b,0),(.022*b,0,-.016*a)]
    if clip=='attack':
        peck=envelope(t,((0.,0.),(.19,-.40),(.39,1.),(.55,.52),(.77,.07),(1.,0.)))
        spread=envelope(t,((0.,0.),(.17,.18),(.34,1.),(.58,.74),(.84,.10),(1.,0.)))
        snap=envelope(t,((0.,0.),(.23,.75),(.40,1.),(.52,.10),(.74,0.),(1.,0.)))
        return [(0,0,0),(.115*peck,0,0),(.230*peck,.021*peck,0),(.11*snap,0,0),
                (-.13*spread,-.13*spread,-.30*spread),(.08*spread,-.035*spread,-.10*spread),
                (-.11*spread,.13*spread,.28*spread),(.065*spread,.035*spread,.09*spread),
                (-.11*peck,.018*spread,0),(-.105*peck,0,0)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.38,.42),(.62,-.13),(.83,.035),(1.,0.)))
    lag=envelope(t,((0.,0.),(.23,1.),(.44,.47),(.71,-.10),(1.,0.)))
    return [(0,0,0),(-.10*recoil,.020*recoil,.012*recoil),(-.17*recoil,-.04*recoil,-.024*recoil),
            (.065*lag,0,0),(.11*recoil,-.04*lag,-.17*recoil),(-.09*lag,0,-.065*lag),
            (.09*recoil,.04*lag,.13*recoil),(-.07*lag,0,.05*lag),
            (.09*lag,-.04*lag,0),(.095*lag,0,.023*lag)]


def geometry_digest(model):
    digest=hashlib.sha256()
    for part in model['primitives']:
        for key,kind in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):
            digest.update(packed(part[key],kind))
    return digest.hexdigest()


def export_spearow(model=None):
    model=sculpture() if model is None else model
    g=Glb(); g.doc['asset']['generator']='Geothite folded-paper Spearow sculpture and skin v1'
    nodes,inverse=g.doc['nodes'],[]
    for name,parent,pivot in JOINTS:
        origin=JOINTS[parent][2] if parent is not None else (0.,0.,0.)
        nodes.append({'name':'spearow/'+name,'translation':list(sub(pivot,origin))})
        if parent is not None: nodes[parent].setdefault('children',[]).append(len(nodes)-1)
        x,y,z=pivot
        inverse.extend([1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.])
    nodes.append({'name':'spearow','mesh':0,'skin':0})
    g.doc['scenes']=[{'name':'spearow','nodes':[0,len(JOINTS)]}]
    g.doc['skins']=[{'name':'spearow','skeleton':0,'joints':list(range(len(JOINTS))),
                     'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
    colors,primitives={},[]
    for part in model['primitives']:
        color=part['color']
        if color not in colors:
            colors[color]=len(colors)
            g.doc['materials'].append({'name':'spearow/'+color,'pbrMetallicRoughness':{
                'baseColorFactor':part['base_color'],'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
        ids,weights=[],[]
        for i in range(0,len(part['positions']),3):
            joints,ws=skin_weights(part['part'],part['positions'][i:i+3]); ws=quantized_weights(ws)
            ids.extend(j if w else 0 for j,w in zip(joints,ws));weights.extend(ws)
        primitives.append({'attributes':{
            'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
            'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),
            'JOINTS_0':g.accessor(ids,'VEC4',5121,34962),
            'WEIGHTS_0':g.accessor(weights,'VEC4',5121,34962,normalized=True)},
            'indices':g.accessor(part['indices'],'SCALAR',5123,34963),
            'material':colors[color],'mode':4,'extras':{'part':part['part']}})
    g.doc['meshes']=[{'name':'spearow','primitives':primitives}]
    for clip,duration,count in CLIPS:
        times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True)
        channels,samplers=[],[]
        for joint in range(1,len(JOINTS)):
            rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
            channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
            samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
        g.doc['animations'].append({'name':'spearow.'+clip,'channels':channels,'samplers':samplers,
            'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle',
                'purpose':'Spearow alert glance, folded-wing counterbalance, short peck and startled recoil; game-owned cues'}})
    g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'spearow',
        'source_sha256':geometry_digest(model),'reference_span':HEIGHT,
        'animation_contract':'identity root and grounded toes; rotation-only presentation clips; source battle cues own timing'}
    assert len(g.doc['accessors'])<=256
    return g.bytes()


def read_spearow(path):
    # The recipe is canonical. Readers reject edited storage or accidental clip
    # changes instead of pretending an unknown GLB matches this sculpture.
    with Path(path).open('rb') as source:
        blob=source.read(MAX_BYTES+1)
    if len(blob)>MAX_BYTES or blob!=export_spearow(): raise ValueError('noncanonical Spearow GLB')
    return sculpture()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/spearow.glb'))
    args=parser.parse_args(); model=sculpture();blob=export_spearow(model)
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; {len(model["primitives"])} anatomy parts; '+
          f'{sum(len(p["indices"])//3 for p in model["primitives"])} triangles; neutral {geometry_digest(model)}')


if __name__=='__main__': main()
