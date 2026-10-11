#!/usr/bin/env python3
"""Distinct closed Drowzee/Hypno anatomy, no imported assets or duplicate catalog."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,tube,ellipsoid,translate,fitted_patch,fitted_contour,curved_profiles,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from skin_glb import COORDINATES
HEIGHT=1.12
YELLOW=(.95,.77,.26);BROWN=(.39,.34,.30);IVORY=(.96,.95,.89)
INK=(.15,.12,.07);METAL=(.70,.72,.70)


def colored_body(profiles):
    # One anatomical solid, two welded material groups at its wavy transition.
    n=64;v=[];seam_y=profiles[3][0];profiles=curved_profiles(profiles,4);seam=12
    for k,(y,w,d,z) in enumerate(profiles):
        for i in range(n):
            a=math.tau*i/n;v.append((w*math.sin(a),y+.025*math.cos(a*6)*math.exp(-((y-seam_y)/.10)**2),z+d*math.cos(a)))
    groups=[[],[]]
    groups[0].append(tuple(reversed(range(n))));groups[1].append(tuple((len(profiles)-1)*n+i for i in range(n)))
    for k in range(len(profiles)-1):
        for i in range(n):
            a=k*n+i;b=k*n+(i+1)%n;groups[0 if k<seam else 1].append((a,b,b+n,a+n))
    # Build the whole surface first so shading shares the material boundary.
    whole=part('Body / continuous tapir hips belly neck and head',YELLOW,v,groups[0]+groups[1],True)
    bottom_indices=sum(len(f)-2 for f in groups[0])*3
    out=[]
    from chikorita_sculpt import linear
    for name,color,start,end in [('Body / continuous lower tapir brown',BROWN,0,bottom_indices),('Body / continuous upper tapir yellow',YELLOW,bottom_indices,len(whole['indices']))]:
        # Parts share seam edges as material regions; neither is a separate solid.
        p={k:v for k,v in whole.items()};p['part']=name;p['base_color']=linear(color)
        ids=whole['indices'][start:end];used=sorted(set(ids));remap={old:new for new,old in enumerate(used)}
        for key in ('positions','normals'):p[key]=[c for old in used for c in whole[key][old*3:old*3+3]]
        p['indices']=[remap[i] for i in ids];out.append(p)
    return out,whole


def ruff():
    # Closed soft annulus: a rounded shoulder drape and fine scalloped rim.
    n=128;v=[]
    for ring in range(5):
        for i in range(n):
            a=math.tau*i/n;front=(1+math.cos(a))/2;wave=.020*math.cos(32*a)
            if ring==0:w=.119;d=.098;y=.684
            elif ring==1:w=.242;d=.158;y=.614-front*.023
            elif ring==2:w=.281+wave;d=.180+wave*.65;y=.588-front*.027
            elif ring==3:w=.244;d=.161;y=.570-front*.025
            else:w=.119;d=.098;y=.627
            v.append((w*math.sin(a),y,d*math.cos(a)))
    fs=[]
    for k in range(5):
        for i in range(n):a=k*n+i;b=k*n+(i+1)%n;c=((k+1)%5)*n+(i+1)%n;d=((k+1)%5)*n+i;fs.append((a,b,c,d))
    return crease_normals(part('Ruff / closed serrated ivory collar',IVORY,v,fs,True),45)


def ring(name,color,center,radius,width):
    v=[];fs=[];segments=64;cross=12
    for i in range(segments):
        a=math.tau*i/segments
        for j in range(cross):
            b=math.tau*j/cross;r=radius+width*math.cos(b)
            v.append((center[0]+r*math.cos(a),center[1]+r*math.sin(a),center[2]+width*math.sin(b)))
    for i in range(segments):
        for j in range(cross):fs.append((i*cross+j,((i+1)%segments)*cross+j,((i+1)%segments)*cross+(j+1)%cross,i*cross+(j+1)%cross))
    return part(name,color,v,fs,True)


def sculpture(species):
    if species not in ('drowzee','hypno'):raise ValueError(species)
    hypno=species=='hypno';p=[]
    profiles=([(.108,.192,.155,-.013),(.184,.292,.216,-.025),(.337,.304,.245,-.009),(.460,.283,.225,.005),(.575,.247,.193,.005),(.666,.198,.163,.004),(.736,.169,.145,.023),(.810,.235,.188,.027),(.903,.241,.197,.010),(.980,.199,.164,-.005),(1.028,.112,.101,-.010),(1.042,.020,.025,-.013)] if not hypno else
              [(.233,.098,.090,-.015),(.312,.159,.123,-.016),(.399,.141,.109,-.020),(.490,.150,.112,-.012),(.579,.195,.123,-.003),(.649,.168,.114,0.),(.704,.083,.079,0.),(.755,.174,.147,.023),(.837,.196,.166,.034),(.929,.172,.147,.021),(.980,.113,.109,.012),(.999,.020,.023,.006)])
    if hypno:
        body=loft('Body / continuous yellow pelvis waist chest neck and head',YELLOW,curved_profiles(profiles,4),64);p.append(body)
    else:
        regions,body=colored_body(profiles);p.extend(regions)
    surf=surface(body)
    # The nose is a broad tapered facial volume, not a thin dangling trunk.
    nosepoints=[(0,.859,.158),(0,.828,.249),(0,.773,.303),(0,.731,.302)] if not hypno else [(0,.821,.158),(0,.779,.243),(0,.719,.313),(0,.702,.326)]
    p.append(curved_tube('Nose / short curved tapir muzzle' if not hypno else 'Nose / long tapered hypnotist muzzle',YELLOW,nosepoints,[.085,.076,.053,.031] if not hypno else [.069,.063,.041,.004],28))
    for s in (-1,1):
        eye_y=.912 if not hypno else .864;x=s*(.129 if not hypno else .109)
        xy=[(-.054,-.013),(-.047,.016),(.032,.025),(.060,.009),(.050,-.013)] if hypno else [(-.055,-.012),(-.050,.014),(-.030,.029),(0,.034),(.029,.029),(.053,.012),(.055,-.012),(.020,-.006),(-.019,-.006)]
        border=fitted_contour('Eye border / fitted dark sleepy socket '+str(s),INK,[(x+s*a,eye_y+b) for a,b in xy],surf,.005,subdivisions=3);p.append(border)
        eye=fitted_contour('Eye / fitted half-lidded ivory '+str(s),IVORY,[(x+s*a*.90,eye_y+b*.86) for a,b in xy],surface(border),.003,subdivisions=3);p.append(eye)
        p.append(fitted_patch('Eye / narrow pupil '+str(s),INK,oval(x,eye_y+(.008 if hypno else .010),.0035,.009 if hypno else .011,24),surface(eye),.002))
        if hypno:
            v=[(s*.107,.942,.044),(s*.179,.930,.030),(s*.171,1.199,-.008),(s*.106,1.055,.039),(s*.134,1.039,-.065)]
            ear=part('Ear / pointed hypnotist '+str(s),YELLOW,v,[(0,1,2,3),(0,4,1),(1,4,2),(2,4,3),(3,4,0)]);p.append(ear)
            p.append(fitted_contour('Ear / fitted brown inner '+str(s),BROWN,[(s*.131,.992),(s*.168,.984),(s*.161,1.138)],surface(ear),.003,subdivisions=3))
        else:
            ear=ellipsoid('Ear / rounded tapir '+str(s),YELLOW,(s*.207,.964,-.009),(.067,.090,.055),32,22);p.append(ear)
            p.append(fitted_contour('Ear / recessed brown inner '+str(s),BROWN,oval(s*.219,.977,.038,.049,32),surface(ear),.004,subdivisions=3))
        # Subtle fitted cheek smile creases below the nose.
        y=.766 if not hypno else .746
        outline=[(s*.075,y),(s*.115,y+.006),(s*.133,y+.024),(s*.129,y+.027),(s*.110,y+.012),(s*.076,y+.005)]
        # A small convex wedge instead of a large mouth plane across the muzzle.
        p.append(fitted_contour('Cheek / quiet smile '+str(s),INK,[outline[0],outline[1],outline[2],outline[3],outline[4]],surf,.003,subdivisions=3))
        shoulder=(s*.195,.631,-.003) if hypno else (s*.221,.618,-.003)
        elbow=(s*.326,.619,.008) if hypno else (s*.307,.503,.015)
        wrist=(s*.352,.795,.089) if hypno else (s*.347,.468,.120)
        p.append(curved_tube('Arm / shoulder to elbow '+str(s),YELLOW,[shoulder,((shoulder[0]+elbow[0])/2,(shoulder[1]+elbow[1])/2,0.),elbow],[.073,.070,.046] if hypno else [.089,.073,.057],28))
        p.append(curved_tube('Arm / shaped forearm '+str(s),YELLOW,[elbow,((elbow[0]+wrist[0])/2,(elbow[1]+wrist[1])/2,(elbow[2]+wrist[2])/2),wrist],[.046,.058,.050] if hypno else [.057,.069,.056],28))
        # Slightly cover tube rims so coincident rings cannot shimmer in shading.
        for name,center,r in [('rounded shoulder',shoulder,.078 if hypno else .095),('rounded elbow',elbow,.051 if hypno else .063)]:
            p.append(ellipsoid('Arm joint / '+name+' '+str(s),YELLOW,center,(r,r,r),24,16))
        p.append(ellipsoid('Hand / broad palm '+str(s),YELLOW,wrist,(.073,.055,.054),32,22))
        for finger in range(3):
            x=wrist[0]+(finger-1)*.037;y=wrist[1]+.030
            if hypno:points=[(x,y,wrist[2]),(x,y+.034,wrist[2]+.054),(x,y-.018,wrist[2]+.094)]
            else:points=[(x,y,wrist[2]+.015),(x,y+.038,wrist[2]+.064),(x,y+.027,wrist[2]+.092)]
            p.append(curved_tube('Hand / curved finger '+str(s)+' '+str(finger),YELLOW,points,[.022,.021,.010],24))
        p.append(curved_tube('Hand / rooted curved thumb '+str(s),YELLOW,[(wrist[0]+s*.045,wrist[1]-.017,wrist[2]+.009),(wrist[0]+s*.072,wrist[1]+.006,wrist[2]+.047),(wrist[0]+s*.057,wrist[1]+.004,wrist[2]+.077)],[.026,.022,.014],24))
        hip=(s*.121,.292,-.010) if hypno else (s*.164,.199,-.009)
        knee=(s*.229,.208,.035) if hypno else (s*.223,.105,.027)
        ankle=(s*.193,.065,.084) if hypno else (s*.233,.063,.066)
        legcolor=YELLOW if hypno else BROWN
        p.append(curved_tube('Leg / tapered thigh '+str(s),legcolor,[hip,((hip[0]+knee[0])/2,(hip[1]+knee[1])/2,(hip[2]+knee[2])/2),knee],[.083,.091,.060] if hypno else [.102,.094,.073],28))
        p.append(curved_tube('Leg / planted shin '+str(s),legcolor,[knee,ankle],[.060,.047] if hypno else [.073,.059],28))
        r=.067 if hypno else .080
        p.append(ellipsoid('Leg joint / rounded knee '+str(s),legcolor,knee,(r,r,r),24,16))
        p.append(ellipsoid('Foot / broad raised heel '+str(s),legcolor,(ankle[0],.045,.108),(.093,.040,.089),32,22))
        for toe in range(2 if hypno else 1):
            x=ankle[0]+(toe-.5)*.065 if hypno else ankle[0]
            p.append(ellipsoid('Foot / yellow rounded toe '+str(s)+' '+str(toe),YELLOW,(x,.029,.186),(.052,.023,.052) if hypno else (.074,.023,.044),32,22))
    if hypno:
        p.append(ruff())
        # Thin hanging cord anchored to the closed right hand; no cosmetic pose loop.
        anchor=(.354,.812,.125);end=(.356,.421,.156)
        p.append(curved_tube('Pendulum / hanging dark cord',INK,[anchor,(.348,.649,.164),end],[.0024,.0024,.0024],16))
        p.append(ring('Pendulum / open metal ring',METAL,(end[0],end[1]-.052,end[2]),.052,.010))
        p.append(ring('Pendulum / dark inner rim',(.34,.35,.32),(end[0],end[1]-.052,end[2]+.003),.043,.0025))
    for i,q in enumerate(p):
        if q['part'].startswith(('Eye border /','Eye /','Cheek /','Ear / fitted','Ear / recessed')):p[i]=crease_normals(q,35)
    floor=min(v for q in p for v in q['positions'][1::3]);height=max(v for q in p for v in q['positions'][1::3])-floor
    for q in p:
        q['positions']=[round((v-(floor if i%3==1 else 0.))*HEIGHT/height,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species');a=a.parse_args()
    for n in ('drowzee','hypno'):
        m=sculpture(n);(a.out/(n+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(n,len(m['primitives']),sum(len(q['indices'])//3 for q in m['primitives']))
if __name__=='__main__':main()
