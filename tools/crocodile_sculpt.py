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
BLUE=(.26,.67,.76);CREAM=(.96,.86,.53);RED=(.88,.30,.35)
WHITE=(.98,.97,.87);INK=(.10,.17,.18);ORANGE=(.82,.45,.21)

def between(a,b,t):return tuple(x+(y-x)*t for x,y in zip(a,b))

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
    inset=[(x+a*w*.72,y+b*h*.72) for a,b in outline]
    cap=fitted_adaptive_contour(name,color,inset,None,d,surface_at=skin,initial_subdivisions=1,max_surface_error=.0025)
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


@lru_cache(maxsize=2)
def raw_sculpture(species):
    if species not in ('croconaw','feraligatr'):raise ValueError(species)
    adult=species=='feraligatr';p=[]
    profiles=([(.15,.115,.136,-.015),(.255,.233,.197,-.020),(.405,.264,.210,-.032),(.535,.221,.185,-.031),(.658,.274,.185,-.028),(.774,.323,.191,-.020),(.839,.244,.174,-.015),(.905,.137,.136,.002),(.973,.197,.158,.016),(1.053,.225,.173,.017),(1.117,.201,.149,.003),(1.139,.107,.099,-.010),(1.145,.016,.022,-.016)] if adult else
              [(.117,.11,.123,-.008),(.235,.266,.217,-.024),(.397,.305,.253,-.032),(.568,.278,.229,-.023),(.697,.220,.186,-.012),(.770,.155,.140,.017),(.840,.247,.188,.017),(.976,.253,.188,.012),(1.056,.206,.151,-.006),(1.087,.105,.095,-.009),(1.093,.020,.023,-.013)])
    body=loft('Body / continuous crocodile pelvis belly chest neck and skull',BLUE,curved_profiles(profiles,4),48);p.append(body);surf=surface(body)
    if adult:
        upper=[(-.02,.177,.901,1.077),(.126,.246,.889,1.054),(.283,.226,.895,1.006),(.425,.156,.903,.968),(.445,.104,.915,.948)]
        lower=[(.031,.12,.793,.839),(.185,.207,.758,.808),(.328,.187,.773,.822),(.431,.117,.805,.844),(.444,.066,.819,.840)]
    else:
        upper=[(.002,.191,.806,.955),(.125,.264,.804,.930),(.263,.257,.810,.888),(.339,.184,.829,.877),(.356,.109,.839,.863)]
        lower=[(.039,.147,.702,.814),(.125,.269,.668,.813),(.246,.267,.698,.817),(.329,.185,.748,.830),(.352,.097,.791,.838)]
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
    p.append(axial('Mouth / recessed dark interior',(.27,.12,.15),[(.108,.11,.819,.898),(.267,.19,.811,.902),(.381,.126,.832,.900)] if adult else [(.125,.15,.808,.817),(.263,.198,.814,.829),(.350,.084,.835,.843)]))
    for s in (-1,1):
        x=s*(.135 if adult else .153);y=1.037 if adult else .960
        outline=[(x-s*.053,y-.023),(x-s*.034,y+.041),(x+s*.009,y+.057),(x+s*.056,y+.005),(x+s*.053,y-.026)]
        p.append(adaptive_patch('Eye / fitted ivory '+str(s),WHITE,outline,surf,.007))
        p.append(adaptive_patch('Eye / warm iris '+str(s),ORANGE,oval(x,y+.009,.016,.031,16),surf,.016))
        p.append(adaptive_patch('Eye / slit pupil '+str(s),INK,oval(x,y+.008,.007,.023,12),surf,.022))
        p.append(adaptive_patch('Nostril / fitted snout '+str(s),INK,oval(s*.057,.973 if adult else .894,.011,.004,12),jaw_surface,.005))
        if adult:
            p.append(curved_tube('Jaw / pale mouth corner '+str(s),CREAM,[(s*.216,.973,.145),(s*.232,.875,.211),(s*.185,.791,.288)],[.028,.027,.024],14))
        # Independent upper/lower teeth terminate at their real jaw surfaces.
        for tooth,(x,z) in enumerate(((.130,.340),(.196,.246),(.231,.164)) if adult else ((.195,.240),(.245,.130))):
            base=.908 if adult else .812
            p.append(tube('Tooth / upper '+str(s)+' '+str(tooth),WHITE,[tooth_root(s*x,z,base,True),(s*x,base,z),(s*x,base-(.048 if adult else .044),z)],[.022,.022,.0015],10))
        if adult:
            for tooth,(x,z) in enumerate(((.158,.306),(.192,.216))):
                p.append(tube('Tooth / lower '+str(s)+' '+str(tooth),WHITE,[tooth_root(s*x,z,.807,False),(s*x,.807,z),(s*x,.851,z)],[.020,.020,.0015],10))
        shoulder=(s*.283,.773,-.012) if adult else (s*.230,.651,-.006)
        elbow=(s*.416,.625,.040) if adult else (s*.330,.615,.020)
        wrist=(s*.456,.487,.159) if adult else (s*.389,.667,.123)
        p.append(limb('Arm / shaped shoulder elbow '+str(s),[shoulder,between(shoulder,elbow,.5),elbow,between(elbow,wrist,.45)],[.104,.098,.066,.034] if adult else [.071,.066,.049,.027]))
        p.append(limb('Arm / broad forearm '+str(s),[between(shoulder,elbow,.65),elbow,between(elbow,wrist,.5),wrist],[.037,.062,.091,.052] if adult else [.028,.045,.064,.038]))
        p.append(ellipsoid('Hand / broad palm '+str(s),BLUE,wrist,(.085,.056,.061) if adult else (.061,.039,.048),24,16))
        for finger in range(3):
            x=wrist[0]+(finger-1)*(.047 if adult else .034);y=wrist[1]
            end=(x,y-(.058 if adult else .026),wrist[2]+.082)
            finger_points=[(x,y+.024,wrist[2]+.021),(x,y+.006,wrist[2]+.078),end];finger_radii=[.025,.024,.013] if adult else [.019,.018,.009]
            if not adult:finger_points.append((x,y-.034,wrist[2]+.084));finger_radii.append(.0015)
            p.append(curved_tube('Hand / curved finger '+str(s)+' '+str(finger),BLUE,finger_points,finger_radii,16))
            if adult:p.append(tube('Hand / white claw '+str(s)+' '+str(finger),WHITE,[end,(x,y-.100,wrist[2]+.093)],[.016,.0015],10))
        hip=(s*.180,.310,-.033) if adult else (s*.182,.265,-.011)
        knee=(s*.282,.246,.054) if adult else (s*.268,.193,.071)
        ankle=(s*.271,.068,.089) if adult else (s*.232,.062,.078)
        p.append(limb('Leg / broad bent thigh '+str(s),[hip,between(hip,knee,.5),knee,between(knee,ankle,.55)],[.119,.133,.085,.033] if adult else [.112,.122,.077,.028]))
        hock_root=between(hip,knee,.65) if adult else between(ankle,knee,1.3)
        p.append(limb('Leg / planted hock '+str(s),[hock_root,knee,ankle],[.055,.074,.050] if adult else [.048,.062,.043]))
        p.append(ellipsoid('Foot / raised broad instep '+str(s),BLUE,(ankle[0],.048,.134),(.103,.042,.111) if adult else (.09,.042,.096),28,16))
        for toe in range(3):
            x=ankle[0]+(toe-1)*.057;z=.242 if adult else .227
            toe_points=[(x,.041,.157),(x+(toe-1)*.010,.026,z)];toe_radii=[.031,.021]
            if not adult:toe_points.append((x+(toe-1)*.010,.026,z+.010));toe_radii.append(.0015)
            p.append((tube if adult else curved_tube)('Foot / splayed toe '+str(s)+' '+str(toe),BLUE,toe_points,toe_radii,16))
            if adult:p.append(tube('Foot / white claw '+str(s)+' '+str(toe),WHITE,[(x+(toe-1)*.010,.024,z-.010),(x+(toe-1)*.016,.017,z+.046)],[.022,.0015],10))
        if adult:
            # Knee armor spans the joined thigh/hock envelope. Both surfaces
            # share one continuous leg weight field through the bend.
            for index,(x,y,z,rx,ry,rz) in enumerate(((.312,.752,.092,.061,.046,.016),(.362,.721,.109,.060,.034,.017),(.430,.603,.143,.063,.042,.017),(.450,.535,.207,.060,.031,.014),(.273,.235,.141,.078,.038,.018))):
                prefix='Arm / shaped' if index<2 else 'Arm / broad' if index<4 else 'Leg / planted'
                owners=[q for q in p if q['part'].startswith(('Leg / broad','Leg / planted') if index==4 else (prefix,)) and q['part'].endswith(' '+str(s))]
                p.append(plate('Armor / limb plate '+str(s)+' '+str(index),(.31,.70,.77),(s*x,y,z),(rx,ry,rz),skin_of(owners)))
    if adult:
        # Two convex halves form the characteristic shallow V plate.
        for s in (-1,1):p.append(adaptive_patch('Belly / pale V half '+str(s),CREAM,[(0.,.477),(s*.226,.625),(s*.187,.352),(0.,.308)],surf,.006))
    else:
        p.append(adaptive_patch('Belly / fitted broad pale field',CREAM,oval(0,.425,.234,.218,28),surf,.006))
        for index,(x,y,rx,ry) in enumerate(((-.10,.473,.043,.061),(.093,.562,.042,.064),(.096,.300,.039,.030),(-.111,.294,.035,.055))):
            p.append(adaptive_patch('Belly / blue island '+str(index),BLUE,oval(x,y,rx,ry,16),surf,.014))
    tailpoints=[(0.,.253,-.160),(.018,.220,-.384),(.078,.230,-.578),(.120,.302,-.752),(.127,.380,-.891)] if adult else [(0,.214,-.179),(.021,.220,-.371),(.102,.278,-.545),(.147,.350,-.689)]
    p.append(curved_tube('Tail / heavy tapered counterbalance',BLUE,tailpoints,[.126,.145,.110,.064,.004] if adult else [.117,.122,.071,.004],22))
    if adult:
        p.append(fin('Crest / broad jagged cranial fin',[(1.070,.088),(1.344,.083),(1.281,-.001),(1.395,-.088),(1.086,-.164)],.039))
        p.append(fin('Crest / rear jagged cranial fin',[(1.062,-.148),(1.281,-.172),(1.232,-.216),(1.352,-.287),(.934,-.259)],.045))
        p.append(fin('Crest / dorsal blade',[(.756,-.165),(.913,-.250),(.822,-.335),(.582,-.263)],.047))
        p.append(fin('Crest / tail blade',[(.282,-.552),(.455,-.606),(.389,-.710),(.333,-.794)],.038))
        for index,(y,z,w) in enumerate(((.684,-.203,.121),(.536,-.244,.119),(.376,-.233,.107))):
            p.append(plate('Armor / dorsal plate '+str(index),(.30,.68,.76),(0.,y,z),(w,.057,.010),skin_of([body],True),True))
    else:
        p.append(fin('Crest / three-point cranial sail',[(1.060,.060),(1.161,.128),(1.102,.011),(1.375,-.038),(1.173,-.105),(1.301,-.190),(1.000,-.168)],.042))
        p.append(fin('Crest / dorsal sail',[(.650,-.164),(.844,-.248),(.704,-.332),(.481,-.239)],.039))
        p.append(fin('Crest / tail diamond',[(.303,-.547),(.449,-.578),(.471,-.632),(.382,-.676)],.028))
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
        if q['part'].startswith(('Eye /','Belly /')):p[i]=skin_shading(q,body)
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
