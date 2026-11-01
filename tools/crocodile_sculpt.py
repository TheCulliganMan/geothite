#!/usr/bin/env python3
"""Original Croconaw/Feraligatr anatomical recipes, continuous skins, fitted faces and rooted crocodile limbs."""
import argparse,json,math
from functools import lru_cache
from copy import deepcopy
from bovine_sculpt import curved_tube,smooth
from cubone_sculpt import skin_shading
from sculpt_geometry import fitted_adaptive_contour
def adaptive_patch(name,color,outline,skin,thickness=.008):
 return fitted_adaptive_contour(name,color,outline,None,thickness,surface_at=skin,initial_subdivisions=1)
from pathlib import Path
from sculpt_geometry import part,loft,tube,ellipsoid,translate,fitted_patch,subdivided_part,crease_normals,curved_profiles
from abra_sculpt import polygon_caps
from goldfish_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.30
FORWARD_LEAN={'croconaw':.16,'feraligatr':.22}
BLUE=(.34,.72,.81);CREAM=(.96,.87,.57);RED=(.88,.30,.35)
WHITE=(.98,.97,.87);INK=(.10,.17,.18);ORANGE=(.82,.45,.21)

def between(a,b,t):return tuple(x+(y-x)*t for x,y in zip(a,b))

def posture_point(species,point):
    x,y,z=point;return (x,y,z+FORWARD_LEAN[species]*y)

def anatomy_point(species,point):
    x,y,z=point;return (x,y,z-FORWARD_LEAN[species]*y)

def limb(name,points,radii):
    # Carry the curved surface past each bend and bury its closing ring in
    # the adjoining segment. Flat end disks must not form elbows or knees.
    path,widths=smooth(points,radii,steps=3)
    return crease_normals(tube(name,BLUE,path,widths,24),40.)


def oval(cx,cy,rx,ry,n=20):return [(cx+rx*math.cos(i*math.tau/n),cy+ry*math.sin(i*math.tau/n)) for i in range(n)]


def axial(name,color,sections):
    # Eight-sided transverse rings form a true broad rectangular crocodile jaw.
    v=[]
    for z,w,lo,hi in sections:
        mid=(lo+hi)/2
        v.extend((x,y,z) for x,y in [(-w*.75,lo),(w*.75,lo),(w,mid-.010),(w*.87,hi-.016),(w*.60,hi),(-w*.60,hi),(-w*.87,hi-.016),(-w,mid-.010)])
    f=[tuple(reversed(range(8))),tuple((len(sections)-1)*8+i for i in range(8))]
    for k in range(len(sections)-1):
        for i in range(8):a=k*8+i;b=k*8+(i+1)%8;f.append((a,b,b+8,a+8))
    return part(name,color,v,f) if name.startswith('Mouth /') else subdivided_part(name,color,v,f,2,True)


def mouth_well(adult):
    # A closed cup has a real open aperture, roof, floor and recessed rear.
    # A capped dark extrusion made the old gape a flat rectangular billboard.
    rows=([(.108,.130,.050,.882),(.267,.220,.064,.872),(.443,.210,.062,.872),
           (.442,.205,.057,.872),(.267,.214,.058,.872),(.121,.124,.041,.882)] if adult else
          [(.125,.170,.040,.830),(.263,.215,.050,.837),(.376,.170,.045,.842),
           (.375,.165,.040,.842),(.263,.209,.044,.837),(.137,.164,.032,.830)])
    n=32;v=[(w*math.cos(i*math.tau/n),center+h*math.sin(i*math.tau/n),z)
            for z,w,h,center in rows for i in range(n)]
    faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    for row in range(len(rows)-1):
        for i in range(n):
            a=row*n+i;b=row*n+(i+1)%n;faces.append((a,b,b+n,a+n))
    return crease_normals(part('Mouth / recessed dark interior',(.27,.12,.15),v,faces,True),35.)


def fin(name,outline,width):
    # Ear-clipped YZ silhouette preserves serrations without overlapping fans.
    projected=[(z,y,0.) for y,z in outline];caps=polygon_caps(projected);n=len(outline)
    v=[(-width/2,y,z) for y,z in outline]+[(width/2,y,z) for y,z in outline]
    area=sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(projected,projected[1:]+projected[:1]));ids=list(range(n))
    if area<0:ids.reverse()
    f=caps+[tuple(j+n for j in reversed(t)) for t in caps]+[(a,a+n,b+n,b) for a,b in zip(ids,ids[1:]+ids[:1])]
    return part(name,RED,v,f)


def plate(name,color,center,radii,skin,back=False):
    # Fit each complete beveled face, including its interior, to occupied skin.
    x,y,z=center;w,h,d=radii;direction=-1. if back else 1.
    outline=[(-.7,-1.),(.7,-1.),(1.,-.7),(1.,.7),(.7,1.),(-.7,1.),(-1.,.7),(-1.,-.7)]
    faces=[tuple(reversed(range(8))),tuple(16+i for i in range(8))]
    for k in range(2):
        for i in range(8):a=k*8+i;b=k*8+(i+1)%8;faces.append((a,b,b+8,a+8))
    for attempt in range(12):
        try:
            vertices=[(x+a*w*scale,y+b*h*scale,direction*(skin(y+b*h*scale,x+a*w*scale)+d*depth)) for scale,depth in ((1.,-.25),(1.,0.),(.72,1.)) for a,b in outline]
            candidate=part(name,color,vertices,faces)
            points=[candidate['positions'][i:i+3] for i in range(0,len(candidate['positions']),3)]
            for k in range(0,len(candidate['indices']),3):
                tri=[points[i] for i in candidate['indices'][k:k+3]]
                for weights in ((1/3,1/3,1/3),(.5,.5,0.),(0.,.5,.5),(.5,0.,.5)):
                    q=[sum(v[axis]*weight for v,weight in zip(tri,weights)) for axis in range(2)];skin(q[1],q[0])
            break
        except ValueError:w*=.85;h*=.85
    else:raise ValueError(('armor footprint outside occupied limb',name))
    # An overlapping curved limb can put a silhouette break through a plate.
    # Fit a smaller occupied footprint instead of bridging the break or raising
    # the allowed clearance. Retain the original depth and real outer bevel.
    for attempt in range(5):
        inset=[(x+a*w*.72,y+b*h*.72) for a,b in outline]
        try:
            cap=fitted_adaptive_contour(name,color,inset,None,d,surface_at=skin,initial_subdivisions=1,max_surface_error=.0025)
            break
        except ValueError:
            if attempt==4:raise
            w*=.85;h*=.85
    # Recover the front triangulation; extend its boundary through a real bevel.
    cap_points=[cap['positions'][i:i+3] for i in range(0,len(cap['positions']),3)]
    xy=[];lookup={};front=[]
    for k in range(0,len(cap['indices']),3):
        points=[cap_points[i] for i in cap['indices'][k:k+3]];a,b,c=points
        if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
        face=[]
        for px,py,pz in points:
            key=(px,py)
            if key not in lookup:lookup[key]=len(xy);xy.append(key)
            face.append(lookup[key])
        front.append(tuple(face))
    edges={}
    for face in front:
        for a,b in zip(face,face[1:]+face[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
    boundary=[uses[0] for uses in edges.values() if len(uses)==1];count=len(xy)
    vertices=[(px,py,direction*(skin(py,px)+depth)) for depth in (d,-.25*d) for px,py in xy]
    faces=list(front)+[tuple(i+count for i in reversed(face)) for face in front]
    layers={0:{i:i for edge in boundary for i in edge}}
    for layer in range(1,4):
        factor=1.+(.28/.72)*layer/3;depth=d*(1.-layer/3);layers[layer]={}
        for i in layers[0]:
            px=x+(xy[i][0]-x)*factor;py=y+(xy[i][1]-y)*factor;layers[layer][i]=len(vertices);vertices.append((px,py,direction*(skin(py,px)+depth)))
    # Add the outer back ring separately, then connect it to the inset back cap.
    outer_back={}
    for i in layers[0]:
        px=x+(xy[i][0]-x)/.72;py=y+(xy[i][1]-y)/.72;outer_back[i]=len(vertices);vertices.append((px,py,direction*(skin(py,px)-.25*d)))
    for a,b in boundary:
        for layer in range(3):faces.append((layers[layer][b],layers[layer][a],layers[layer+1][a],layers[layer+1][b]))
        faces.append((layers[3][b],layers[3][a],outer_back[a],outer_back[b]))
        faces.append((a+count,b+count,outer_back[b],outer_back[a]))
    if back:faces=[tuple(reversed(face)) for face in faces]
    return crease_normals(part(name,color,vertices,faces,True),40)



def skin_of(parts,back=False):
    positions=[];indices=[]
    for p in parts:
        offset=len(positions)//3
        positions.extend(v*(-1. if back and i%3==2 else 1.) for i,v in enumerate(p['positions']))
        indices.extend(i+offset for i in p['indices'])
    return surface({'positions':positions,'indices':indices})


def wrapped_hide(body,profiles):
    """Clip the real torso triangles into a closed, thin 360-degree hide.

    Both edges vary around the body. Outer and buried inner surfaces share
    topology, so the asymmetric collar and torn hem wrap through the rear.
    """
    def profile(y):
        for a,b in zip(profiles,profiles[1:]):
            if a[0]-1e-9<=y<=b[0]+1e-9:
                t=(y-a[0])/(b[0]-a[0]);return tuple(a[i]+t*(b[i]-a[i]) for i in range(1,4))
        raise ValueError(('hide outside body profile',y))
    def marked(point):
        x,y,z=point;rx,rz,center=profile(y);angle=math.atan2(x/rx,(z-center)/rz)
        upper=.610-.055*math.sin(angle)+.040*math.cos(angle)-.110*math.exp(-(angle/.24)**4)
        lower=.190+.024*abs(math.sin(angle))+.013*math.sin(7*angle+.5)
        return (*point,y-lower,upper-y)
    def clip(poly,axis):
        out=[]
        for a,b in zip(poly,poly[1:]+poly[:1]):
            inside_a=a[axis]>=0.;inside_b=b[axis]>=0.
            if inside_a:out.append(a)
            if inside_a!=inside_b:
                t=a[axis]/(a[axis]-b[axis]);out.append(tuple(x+t*(y-x) for x,y in zip(a,b)))
        return out
    vertices=[];faces=[];lookup={}
    points=list(zip(*[iter(body['positions'])]*3))
    for k in range(0,len(body['indices']),3):
        tri=[points[i] for i in body['indices'][k:k+3]]
        if max(v[1] for v in tri)<.16 or min(v[1] for v in tri)>.72:continue
        poly=clip(clip([marked(v) for v in tri],3),4)
        if len(poly)<3:continue
        face=[]
        for v in poly:
            key=tuple(round(x,9) for x in v[:3])
            if key not in lookup:lookup[key]=len(vertices);vertices.append(key)
            face.append(lookup[key])
        for i in range(1,len(face)-1):
            if len({face[0],face[i],face[i+1]})==3:faces.append((face[0],face[i],face[i+1]))
    edges={}
    for face in faces:
        for a,b in zip(face,face[1:]+face[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
    count=len(vertices);shell=[]
    for depth in (.006,-.003):
        for x,y,z in vertices:
            center=profile(y)[2];r=math.hypot(x,z-center)
            shell.append((x+depth*x/r,y,z+depth*(z-center)/r))
    caps=faces+[tuple(i+count for i in reversed(f)) for f in faces]
    caps += [(b,a,a+count,b+count) for uses in edges.values() if len(uses)==1 for a,b in uses]
    return part('Belly / fitted broad pale field',CREAM,shell,caps,True)


@lru_cache(maxsize=2)
def raw_sculpture(species):
    if species not in ('croconaw','feraligatr'):raise ValueError(species)
    adult=species=='feraligatr';p=[]
    # Low, broad skulls and substantial necks keep the jaw/eye silhouette
    # continuous. The juvenile has a pear-shaped belly, the adult a wide chest.
    profiles=([(.15,.145,.155,-.015),(.255,.260,.220,-.020),(.405,.288,.235,-.032),(.535,.258,.209,-.031),(.658,.295,.205,-.028),(.774,.355,.215,-.020),(.839,.298,.201,-.015),(.905,.213,.175,.002),(.973,.251,.182,.016),(1.053,.277,.194,.017),(1.117,.248,.170,.003),(1.160,.167,.119,-.010),(1.176,.030,.030,-.016)] if adult else
              [(.117,.13,.133,-.008),(.235,.288,.240,-.024),(.397,.325,.269,-.032),(.568,.296,.242,-.023),(.697,.237,.209,-.012),(.770,.204,.176,.017),(.840,.269,.203,.017),(.976,.288,.205,.012),(1.056,.256,.181,-.006),(1.108,.177,.126,-.009),(1.128,.030,.031,-.013)])
    body_profiles=curved_profiles(profiles,4)
    body=loft('Body / continuous crocodile pelvis belly chest neck and skull',BLUE,body_profiles,48)
    vertices=[]
    for k in range(0,len(body['positions']),3):
        x,y,z=body['positions'][k:k+3]
        if z>0.:
            radius=(x/(.290 if adult else .280))**2+((y-(.840 if adult else .820))/(.110 if adult else .100))**2
            z-=(.170 if adult else .160)*max(0.,1.-radius)**1.1
        vertices.append((x,y,z))
    body=part(body['part'],BLUE,vertices,[tuple(body['indices'][k:k+3]) for k in range(0,len(body['indices']),3)],True)
    p.append(body);surf=surface(body)
    if adult:
        upper=[(-.02,.222,.897,1.092),(.126,.298,.885,1.072),(.283,.284,.888,1.027),(.425,.226,.903,.995),(.476,.144,.923,.975)]
        lower=[(.004,.190,.770,.900),(.155,.285,.680,.800),(.300,.274,.620,.740),(.438,.207,.620,.780),(.475,.123,.660,.790)]
    else:
        upper=[(.002,.218,.824,.987),(.125,.297,.818,.956),(.263,.286,.827,.924),(.355,.225,.847,.910),(.386,.136,.860,.901)]
        lower=[(.004,.195,.690,.833),(.125,.295,.625,.800),(.246,.289,.610,.750),(.352,.216,.630,.770),(.384,.125,.690,.790)]
    # Captured battle idles carry the thick mandible under the muzzle; an
    # extracted model's wide open mouth is not the resting battle expression.
    jaw_raise=.070 if adult else .025
    lower=[(z,w,lo+jaw_raise,hi+jaw_raise) for z,w,lo,hi in lower]
    upper_mesh=axial('Jaw / broad upper snout',BLUE,upper);p.append(upper_mesh);jaw_surface=surface(upper_mesh)
    lower_mesh=axial('Jaw / fitted pale lower shell',CREAM,lower);p.append(lower_mesh)
    # Seat tooth roots inside the actual triangulated jaw, preserving visible tips.
    def along_y(mesh,sign):
        xyz=mesh['positions'];return surface({'positions':[q for i in range(0,len(xyz),3) for q in (xyz[i],xyz[i+2],sign*xyz[i+1])],'indices':mesh['indices']})
    upper_floor=along_y(upper_mesh,-1);upper_roof=along_y(upper_mesh,1)
    lower_floor=along_y(lower_mesh,-1);lower_roof=along_y(lower_mesh,1)
    def tooth_root(x,z,base,upper):
        floor,roof=(upper_floor,upper_roof) if upper else (lower_floor,lower_roof)
        for scale in (1.,.96,.92,.88,.84,.80,.76):
            rx=x*scale
            try:lo=-floor(z,rx);hi=roof(z,rx)
            except ValueError:continue
            if hi-lo<.028:continue
            y=min(hi-.008,max(base+.006,lo+.012)) if upper else max(lo+.008,min(base-.006,hi-.012))
            return (rx,y,z)
        raise ValueError(('no occupied jaw root for tooth',species,x,z))
    p.append(mouth_well(adult))
    if adult:p.append(ellipsoid('Mouth / rounded pink tongue',(.86,.43,.50),(0.,.860,.354),(.125,.023,.074),28,16))
    for s in (-1,1):
        x=s*(.172 if adult else .182);y=1.060 if adult else 1.003
        outline=[(x-s*.070,y-.030),(x-s*.060,y+.015),(x+s*.043,y+.065),(x+s*.073,y-.015),(x+s*.035,y-.036)]
        eye=adaptive_patch('Eye / fitted ivory '+str(s),WHITE,outline,surf,.007);p.append(eye)
        iris=adaptive_patch('Eye / warm iris '+str(s),ORANGE,oval(x+s*.021,y+.002,.020,.028,20),surface(eye),.006);p.append(iris)
        p.append(adaptive_patch('Eye / slit pupil '+str(s),INK,oval(x+s*.023,y+.001,.009,.021,16),surface(iris),.004))
        brow=[(x-s*.079,y+.007),(x+s*.034,y+.074),(x+s*.073,y+.041),(x+s*.076,y+.014),(x+s*.041,y+.048),(x-s*.065,y-.002)]
        p.append(adaptive_patch('Eye / blue brow '+str(s),BLUE,brow,surf,.015))
        p.append(adaptive_patch('Nostril / fitted snout '+str(s),INK,oval(s*.068,.990 if adult else .927,.014,.005,16),jaw_surface,.005))
        if adult:
            p.append(curved_tube('Jaw / pale mouth corner '+str(s),CREAM,[(s*.244,.938,.116),(s*.266,.885,.188),(s*.233,.805,.279)],[.041,.039,.033],16))
        # Independent upper/lower teeth terminate at their real jaw surfaces.
        for tooth,(x,z) in enumerate(((.130,.340),(.196,.246),(.231,.164)) if adult else ((.195,.240),(.245,.130))):
            base=.908 if adult else .812
            p.append(tube('Tooth / upper '+str(s)+' '+str(tooth),WHITE,[tooth_root(s*x,z,base,True),(s*x,base,z),(s*x,base-(.048 if adult else .044),z)],[.022,.022,.0015],10))
        if adult:
            for tooth,(x,z) in enumerate(((.158,.306),(.192,.216))):
                p.append(tube('Tooth / lower '+str(s)+' '+str(tooth),WHITE,[tooth_root(s*x,z,.805,False),(s*x,.805,z),(s*x,.857,z)],[.020,.020,.0015],10))
        shoulder=(s*.303,.794,-.012) if adult else (s*.239,.690,-.006)
        elbow=(s*.400,.630,.060) if adult else (s*.325,.590,.030)
        wrist=(s*.470,.660,.250) if adult else (s*.365,.535,.160)
        p.append(limb('Arm / shaped shoulder elbow '+str(s),[shoulder,between(shoulder,elbow,.5),elbow,between(elbow,wrist,.45)],[.120,.108,.079,.040] if adult else [.078,.070,.050,.029]))
        p.append(limb('Arm / broad forearm '+str(s),[between(shoulder,elbow,.65),elbow,between(elbow,wrist,.5),wrist],[.045,.076,.100,.066] if adult else [.032,.050,.065,.043]))
        p.append(ellipsoid('Hand / broad palm '+str(s),BLUE,wrist,(.101,.066,.077) if adult else (.066,.045,.052),24,16))
        for finger in range(3):
            x=wrist[0]+(finger-1)*(.057 if adult else .035);y=wrist[1]
            end=(x,y-(.058 if adult else .026),wrist[2]+.100)
            finger_points=[(x,y+.024,wrist[2]+.021),(x,y+.006,wrist[2]+.085),end];finger_radii=[.030,.028,.017] if adult else [.021,.019,.010]
            if not adult:
                finger_points=[(x,y+.013,wrist[2]+.015),(x+s*.012,y-.012,wrist[2]+.040),(x+s*.018,y-.035,wrist[2]+.036),(x+s*.019,y-.043,wrist[2]+.035)]
                finger_radii=[.020,.020,.012,.0015]
            p.append(curved_tube('Hand / curved finger '+str(s)+' '+str(finger),BLUE,finger_points,finger_radii,16))
            if adult:p.append(tube('Hand / white claw '+str(s)+' '+str(finger),WHITE,[end,(x,y-.108,wrist[2]+.122)],[.020,.0015],12))
        hip=(s*.220,.330,-.033) if adult else (s*.225,.276,-.011)
        knee=(s*.335,.250,.054) if adult else (s*.309,.193,.071)
        ankle=(s*.323,.074,.089) if adult else (s*.292,.066,.092)
        p.append(limb('Leg / broad bent thigh '+str(s),[hip,between(hip,knee,.5),knee,between(knee,ankle,.55)],[.145,.153,.105,.041] if adult else [.133,.143,.098,.040]))
        hock_root=between(hip,knee,.65) if adult else between(ankle,knee,1.3)
        p.append(limb('Leg / planted hock '+str(s),[hock_root,knee,ankle],[.067,.091,.061] if adult else [.060,.085,.058]))
        p.append(ellipsoid('Foot / raised broad instep '+str(s),BLUE,(ankle[0],.048,.147),(.123,.042,.130) if adult else (.109,.042,.119),28,16))
        for toe in range(3):
            x=ankle[0]+(toe-1)*.065;z=.270 if adult else .252
            toe_points=[(x,.041,.157),(x+(toe-1)*.010,.026,z)];toe_radii=[.031,.021]
            if not adult:toe_points.append((x+(toe-1)*.010,.026,z+.010));toe_radii.append(.0015)
            p.append((tube if adult else curved_tube)('Foot / splayed toe '+str(s)+' '+str(toe),BLUE,toe_points,toe_radii,16))
            if adult:p.append(tube('Foot / white claw '+str(s)+' '+str(toe),WHITE,[(x+(toe-1)*.010,.024,z-.010),(x+(toe-1)*.016,.017,z+.046)],[.022,.0015],10))
        if adult:
            # Knee armor spans the joined thigh/hock envelope. Both surfaces
            # share one continuous leg weight field through the bend.
            for index,(x,y,z,rx,ry,rz) in enumerate(((.315,.770,.040,.045,.022,.016),(.335,.735,.100,.036,.016,.017),(.430,.646,.130,.045,.023,.017),(.466,.660,.230,.046,.023,.014),(.332,.235,.141,.090,.043,.018))):
                owners=[q for q in p if q['part'].startswith(('Leg / broad','Leg / planted') if index==4 else ('Arm / shaped','Arm / broad')) and q['part'].endswith(' '+str(s))]
                p.append(plate('Armor / limb plate '+str(s)+' '+str(index),(.31,.70,.77),(s*x,y,z),(rx,ry,rz),skin_of(owners)))
    if adult:
        # Two convex halves form the characteristic shallow V plate.
        for s in (-1,1):p.append(adaptive_patch('Belly / pale V half '+str(s),CREAM,[(0.,.477),(s*.226,.625),(s*.187,.352),(0.,.308)],surf,.006))
    else:
        p.append(wrapped_hide(body,body_profiles))
        islands=[([(-.225,.499),(-.184,.515),(-.149,.461),(-.181,.424),(-.224,.447)]),
                 ([(.183,.557),(.240,.578),(.265,.538),(.244,.491),(.193,.485)]),
                 ([(.094,.288),(.135,.326),(.172,.272),(.158,.241),(.119,.241)]),
                 ([(-.154,.298),(-.102,.335),(-.063,.284),(-.088,.246),(-.133,.257)])]
        for index,outline in enumerate(islands):
            p.append(adaptive_patch('Belly / blue island '+str(index),BLUE,outline,surf,.014))
    tailpoints=[(0.,.253,-.160),(.018,.190,-.384),(.040,.150,-.600),(.060,.125,-.820),(.065,.120,-.940)] if adult else [(0,.214,-.179),(.014,.170,-.371),(.025,.125,-.530),(.030,.095,-.700)]
    p.append(curved_tube('Tail / heavy tapered counterbalance',BLUE,tailpoints,[.140,.160,.120,.055,.002] if adult else [.120,.125,.075,.002],22))
    if adult:
        p.append(fin('Crest / broad jagged cranial fin',[(1.070,.088),(1.344,.083),(1.281,-.001),(1.395,-.088),(1.086,-.164)],.039))
        p.append(fin('Crest / rear jagged cranial fin',[(1.062,-.148),(1.281,-.172),(1.232,-.216),(1.352,-.287),(.934,-.259)],.045))
        p.append(fin('Crest / dorsal blade',[(.756,-.165),(.913,-.250),(.822,-.335),(.582,-.263)],.047))
        p.append(fin('Crest / tail blade',[(.160,-.550),(.340,-.590),(.280,-.640),(.390,-.720),(.220,-.750),(.270,-.850),(.130,-.900),(.130,-.550)],.038))
        for index,(y,z,w) in enumerate(((.684,-.203,.121),(.536,-.244,.119),(.376,-.233,.107))):
            p.append(plate('Armor / dorsal plate '+str(index),(.30,.68,.76),(0.,y,z),(w,.057,.010),skin_of([body],True),True))
    else:
        p.append(fin('Crest / three-point cranial sail',[(1.060,.060),(1.161,.128),(1.102,.011),(1.375,-.038),(1.173,-.105),(1.301,-.190),(1.000,-.168)],.042))
        p.append(fin('Crest / dorsal sail',[(.650,-.164),(.844,-.248),(.704,-.332),(.481,-.239)],.039))
        p.append(fin('Crest / tail diamond',[(.130,-.530),(.280,-.570),(.310,-.620),(.115,-.680)],.028))
    # The fin sits on the actual laterally curving counterbalance, not x=0.
    from bovine_sculpt import smooth
    path=smooth(tailpoints,[1.]*len(tailpoints))[0]
    def tail_x(z):
        for a,b in zip(path,path[1:]):
            if b[2]<=z<=a[2]:
                t=(z-a[2])/(b[2]-a[2]);return a[0]+t*(b[0]-a[0])
        raise ValueError(('tail crest outside centerline',z))
    for i,q in enumerate(p):
        if q['part'].startswith('Crest / tail'):
            vertices=[(q['positions'][k]+tail_x(q['positions'][k+2]),q['positions'][k+1],q['positions'][k+2]) for k in range(0,len(q['positions']),3)]
            p[i]=part(q['part'],q['base_color'][:3],vertices,[tuple(q['indices'][k:k+3]) for k in range(0,len(q['indices']),3)])
    # Broad organic surfaces blend their normals; authored crest folds stay crisp.
    p=[q if q['part'].startswith('Crest /') else crease_normals(q) for q in p]
    for i,q in enumerate(p):
        if q['part'].startswith(('Eye /','Belly /')) and q['part']!='Belly / fitted broad pale field':p[i]=skin_shading(q,body)
    # A forward-loaded stance keeps the feet flat and the muzzle level. This
    # invertible affine lean applies equally to skin, paint and buried joins;
    # the arms above are authored with lowered elbows and forward wrists.
    lean=FORWARD_LEAN[species]
    for q in p:
        q['positions']=[v for k in range(0,len(q['positions']),3) for v in posture_point(species,q['positions'][k:k+3])]
        normals=[]
        for k in range(0,len(q['normals']),3):
            x,y,z=q['normals'][k:k+3];n=(x,y-lean*z,z);length=math.sqrt(sum(v*v for v in n));normals.extend(v/length for v in n)
        q['normals']=normals
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def sculpture(species,normalize=True):
    model=deepcopy(raw_sculpture(species));p=model['primitives']
    if not normalize:return model
    floor=min(v for q in p for v in q['positions'][1::3]);height=max(v for q in p for v in q['positions'][1::3])-floor
    for q in p:
        q['positions']=[round((v-(floor if i%3==1 else 0.))*HEIGHT/height,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
        # Python versions differ in the sign of exact Catmull-Clark zero sums.
        q['positions']=[0. if v==0. else v for v in q['positions']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species');a=a.parse_args()
    for n in ('croconaw','feraligatr'):
        from crocodile_glb import export_crocodile
        blob=export_crocodile(n);(a.out/(n+'.glb')).write_bytes(blob);(a.out/(n+'.mesh.json')).unlink(missing_ok=True);print(n,len(blob),'bytes')
if __name__=='__main__':main()
