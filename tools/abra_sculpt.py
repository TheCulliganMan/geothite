#!/usr/bin/env python3
"""Original closed Abra/Alakazam sculpture, canonical JSON only, +Y up/+Z front."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,tube,ellipsoid,translate,fitted_patch
from skin_glb import COORDINATES

HEIGHT=1.12
YELLOW=(.96,.79,.28);ARMOR=(.37,.32,.30);CREAM=(.99,.89,.52)
INK=(.13,.105,.07);WHITE=(.96,.96,.88);METAL=(.77,.79,.77)


def surface(mesh):
    ps=[mesh['positions'][i:i+3] for i in range(0,len(mesh['positions']),3)]
    ts=[[ps[j] for j in mesh['indices'][i:i+3]] for i in range(0,len(mesh['indices']),3)]
    def at(y,x):
        zs=[]
        for a,b,c in ts:
            den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
            if abs(den)<1e-12:continue
            u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den
            v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den
            if min(u,v,1-u-v)>=-1e-6:zs.append(u*a[2]+v*b[2]+(1-u-v)*c[2])
        if not zs:raise ValueError(('marking outside actual surface',x,y))
        return max(zs)
    return at


def skull(adult):
    # Transverse octagonal sections taper from broad brow to true fox muzzle.
    sections=([(-.11,.159,.692,.902),(.057,.224,.685,.936),(.183,.165,.660,.864),(.325,.064,.603,.744),(.386,.013,.627,.657)] if adult else
              [(-.07,.136,.608,.847),(.054,.222,.619,.893),(.19,.179,.624,.840),(.293,.080,.583,.713),(.333,.015,.599,.626)])
    vs=[]
    for z,w,low,high in sections:
        mid=(low+high)/2
        vs.extend([(x,y,z) for x,y in [(-w*.58,low),(w*.58,low),(w,mid-.023),(w*.79,high-.015),(w*.41,high),(-w*.41,high),(-w*.79,high-.015),(-w,mid-.023)]])
    fs=[tuple(reversed(range(8))),tuple((len(sections)-1)*8+i for i in range(8))]
    for k in range(len(sections)-1):
        for i in range(8):a=k*8+i;b=k*8+(i+1)%8;fs.append((a,b,b+8,a+8))
    return part('Head / joined fox cranium brow and tapered muzzle',YELLOW,vs,fs)


def ear(side,adult):
    y=.854 if adult else .822
    tip=(side*(.337 if adult else .282),1.158 if adult else 1.059,-.083)
    v=[(side*.096,y,-.061),(side*.190,y-.053,.034),(side*.164,y+.066,.069),tip,(side*.157,y+.067,-.066)]
    p=part('Ear / pointed closed fox '+str(side),YELLOW,v,[(0,1,2),(0,2,4),(0,4,3),(4,2,3),(2,1,3),(1,0,3)])
    # Separate inset triangular plane, shallow closed solid along ear's front.
    a=(side*.158,y+.040,.070);b=(side*.200,y+.028,.041);c=(side*(.292 if adult else .250),1.071 if adult else .986,-.026)
    q=[a,b,c]+[(x,y,z-.006) for x,y,z in (a,b,c)]
    return p,part('Ear / inset ochre '+str(side),(.72,.57,.21),q,[(0,1,2),(5,4,3),(0,3,4,1),(1,4,5,2),(2,5,3,0)])


def polygon_caps(vertices):
    """Ear clipping preserves concave moustache serrations without fan overlap."""
    xy=[v[:2] for v in vertices];active=list(range(len(xy)))
    area=sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(xy,xy[1:]+xy[:1]))
    if area<0:active.reverse()
    def orient(a,b,c):
        a,b,c=xy[a],xy[b],xy[c]
        return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    faces=[]
    while len(active)>3:
        for j,b in enumerate(active):
            a,c=active[j-1],active[(j+1)%len(active)]
            if orient(a,b,c)<=1e-12:continue
            if any(min(orient(a,b,q),orient(b,c,q),orient(c,a,q))>=-1e-12 for q in active if q not in (a,b,c)):continue
            faces.append((a,b,c));active.remove(b);break
        else:raise ValueError('invalid polygon')
    faces.append(tuple(active));return faces


def spoon(side,x,y,z):
    p=[tube('Spoon / held tapered stem '+str(side),METAL,[(x,y,z),(x+side*.148,y+.007,z+.021)],[.012,.007],12)]
    # A thin oval bowl, with shallow recessed darker interior facing +Z.
    p.append(ellipsoid('Spoon / oval bowl '+str(side),METAL,(x+side*.209,y+.008,z+.019),(.068,.038,.014),20,10))
    p.append(ellipsoid('Spoon / recessed bowl surface '+str(side),(.59,.63,.61),(x+side*.209,y+.008,z+.031),(.051,.025,.003),20,8))
    return p


def sculpture(species):
    if species not in ('abra','alakazam'):raise ValueError(species)
    adult=species=='alakazam';p=[]
    bodyprofiles=([(.253,.095,.081,0.),(.323,.169,.107,0.),(.423,.118,.086,0.),(.511,.133,.091,-.004),(.586,.179,.103,-.004),(.643,.133,.097,-.005),(.692,.059,.062,-.004)] if adult else
                  [(.163,.115,.117,0.),(.232,.150,.138,.009),(.337,.143,.127,.013),(.444,.106,.093,.008),(.539,.143,.103,0.),(.622,.064,.070,0.)])
    body=loft('Body / continuous pelvis abdomen chest neck',YELLOW,bodyprofiles,24);p.append(body)
    # An actual shaped armor shell embraces the chest, not a brown waist ball.
    cuirass=([(.445,.126,.090,0.),(.482,.158,.112,0.),(.555,.201,.119,-.004),(.621,.176,.110,-.003),(.656,.101,.084,-.003)] if adult else
             [(.454,.111,.096,.009),(.497,.143,.114,.006),(.559,.151,.114,.001),(.598,.112,.091,0.)])
    p.append(loft('Armor / shaped chest cuirass',ARMOR,cuirass,20))
    head=skull(adult);p.append(head);surf=surface(head)
    for s in (-1,1):
        p.extend(ear(s,adult))
        if adult:
            outline=[(s*.074,.749),(s*.163,.798),(s*.147,.742),(s*.087,.724)]
            p.append(fitted_patch('Eye / fitted focused ivory '+str(s),WHITE,outline,surf,.008))
            p.append(fitted_patch('Eye / narrow dark pupil '+str(s),INK,[(s*.117-.005,.746),(s*.117+.005,.746),(s*.117+.005,.772),(s*.117-.005,.772)],surf,.017))
            # Broad sweeping moustache blades with pointed serrated ends.
            v=[(s*x,y,z) for x,y,z in [( .023,.665,.359),(.114,.635,.364),(.214,.655,.331),(.299,.739,.254),(.413,.789,.233),(.375,.753,.244),(.429,.757,.244),(.384,.719,.251),(.423,.721,.249),(.354,.696,.263),(.373,.683,.271),(.291,.687,.287),(.188,.620,.353),(.089,.617,.373)]]
            n=len(v);vs=v+[(x,y,z-.013) for x,y,z in v]
            caps=polygon_caps(v);fs=caps+[tuple(j+n for j in reversed(f)) for f in caps]
            # Side winding follows the same outline orientation as its caps.
            edges=list(range(n))
            if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(v,v[1:]+v[:1]))<0:edges.reverse()
            fs += [(a,a+n,b+n,b) for a,b in zip(edges,edges[1:]+edges[:1])]
            p.append(part('Moustache / sweeping serrated blade '+str(s),CREAM,vs,fs))
        else:
            p.append(fitted_patch('Eye / closed meditation crease '+str(s),INK,[(s*.061,.700),(s*.065,.692),(s*.148,.759),(s*.143,.767)],surf,.005))
        # Clavicle armor, articulated upper arm and broad angular forearm.
        shoulder=(s*(.210 if adult else .163),.592 if adult else .518,-.006)
        p.append(ellipsoid('Armor / shoulder shell '+str(s),ARMOR,shoulder,(.095,.071,.09) if adult else (.092,.055,.08),16,10))
        elbow=(s*.302,.510,.019) if adult else (s*.213,.357,.028)
        wrist=(s*.413,.524,.186) if adult else (s*.129,.275,.157)
        p.append(tube('Arm / tapered upper '+str(s),YELLOW,[shoulder,elbow],[.045,.033],14))
        p.append(tube('Arm / broad tapered forearm '+str(s),ARMOR if adult else YELLOW,[elbow,((elbow[0]+wrist[0])/2,(elbow[1]+wrist[1])/2,(elbow[2]+wrist[2])/2),wrist],[.048,.068 if adult else .043,.038],16))
        p.append(ellipsoid('Hand / palm '+str(s),YELLOW,wrist,(.073,.045,.053) if adult else (.061,.04,.041),16,8))
        for j in range(3):
            x=wrist[0]+(j-1)*.041
            a=(x,wrist[1]+.013,wrist[2]+.012);b=(x,wrist[1]-.027,wrist[2]+.080);c=(x,wrist[1]-.075,wrist[2]+.063)
            p.append(tube('Hand / curled finger '+str(s)+' '+str(j),YELLOW,[a,b,c],[.021,.021,.012],12))
        # Angular crouched thigh with a separate narrow knee and shin.
        hip=(s*.114,.279,-.024) if adult else (s*.087,.204,-.024)
        knee=(s*.240,.272,.087) if adult else (s*.220,.139,.100)
        ankle=(s*.203,.070,.062) if adult else (s*.270,.063,.058)
        p.append(tube('Leg / shaped bent haunch '+str(s),YELLOW,[hip,((hip[0]+knee[0])/2,(hip[1]+knee[1])/2,(hip[2]+knee[2])/2),knee],[.086,.091 if adult else .081,.060],16))
        if adult:p.append(ellipsoid('Armor / fitted knee '+str(s),ARMOR,(knee[0],knee[1],knee[2]+.025),(.069,.066,.062),14,8))
        p.append(tube('Leg / narrow hock '+str(s),YELLOW,[knee,ankle],[.050,.035],14))
        p.append(ellipsoid('Foot / raised instep '+str(s),YELLOW,(ankle[0],.040,.124),(.077,.035,.109),16,8))
        for j in range(3):
            x=ankle[0]+(j-1)*.050;z=.205+(.017 if j==1 else 0.)
            toe=tube('Foot / splayed toe '+str(s)+' '+str(j),YELLOW,[(x,.044,.128),(x+(j-1)*.015,.026,z)],[.030,.019],12);p.append(toe)
            p.append(tube('Foot / pale claw '+str(s)+' '+str(j),WHITE if adult else CREAM,[(x+(j-1)*.015,.027,z-.011),(x+(j-1)*.020,.017,z+.043)],[.020,.002],10))
        if adult:p.extend(spoon(s,wrist[0],wrist[1]-.041,wrist[2]+.061))
    if not adult:
        p.append(tube('Tail / heavy upward psychic curl',YELLOW,[(0,.179,-.100),(.012,.178,-.258),(.107,.227,-.420),(.192,.348,-.507),(.201,.507,-.537),(.163,.629,-.508)],[.077,.103,.093,.074,.039,.003],20))
        p.append(tube('Tail / dark saddle band',ARMOR,[(.193,.364,-.511),(.200,.401,-.526)],[.073,.064],20))
    floor=min(v for q in p for v in q['positions'][1::3]);height=max(v for q in p for v in q['positions'][1::3])-floor
    for q in p:
        q['positions']=[round((v-(floor if i%3==1 else 0.))*HEIGHT/height,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species');a=a.parse_args()
    for n in ('abra','alakazam'):
        m=sculpture(n);(a.out/(n+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(n,len(m['primitives']),sum(len(q['indices'])//3 for q in m['primitives']))
if __name__=='__main__':main()
