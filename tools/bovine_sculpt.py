#!/usr/bin/env python3
"""Original Miltank milk-cow and Tauros three-tailed bull sculptures."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,crease_normals,translate
from abra_sculpt import surface
from wing_insect_sculpt import membrane
from skin_glb import COORDINATES
SPECIES=('miltank','tauros');HEIGHTS={'miltank':1.12,'tauros':1.28}
PINK=(.91,.59,.65);HOOD=(.19,.22,.25);MILK=(.94,.89,.60);WHITE=(.97,.97,.94)
GOLD=(.73,.54,.33);MANE=(.43,.41,.40);HORN=(.63,.68,.76);HOOF=(.29,.34,.43)
BROWN=(.39,.30,.25);BLUE=(.26,.52,.70);INK=(.07,.10,.13);TONGUE=(.75,.43,.46)

def oval(x,y,w,h,n=20):return [(x+w*math.cos(k*math.tau/n),y+h*math.sin(k*math.tau/n)) for k in range(n)]

def smooth(points,radii,steps=4):
    p=[];r=[]
    for i in range(len(points)-1):
        a=points[max(i-1,0)];b=points[i];c=points[i+1];d=points[min(i+2,len(points)-1)]
        for j in range(steps):
            t=j/steps;p.append(tuple(.5*(2*b[k]+(-a[k]+c[k])*t+(2*a[k]-5*b[k]+4*c[k]-d[k])*t*t+(-a[k]+3*b[k]-3*c[k]+d[k])*t*t*t) for k in range(3)));r.append(radii[i]*(1-t)+radii[i+1]*t)
    return p+[points[-1]],r+[radii[-1]]

def curved_tube(name,color,points,radii,segments=18):
    pp,rr=smooth(points,radii);return crease_normals(tube(name,color,pp,rr,segments),35.)

def reverse_front(p):
    p=dict(p)
    for field in ('positions','normals'):p[field]=[(-v if i%3!=1 else v) for i,v in enumerate(p[field])]
    return p

def eye(p,skin,x,y,w,h,blue=False):
    side=-1 if x<0 else 1
    outline=oval(x,y,w,h,24) if blue else [(x-side*w,y+.008),(x+side*w,y+.026),(x+side*w,y-.025),(x-side*w*.7,y-.018)]
    border=fitted_patch('Eye / fitted dark socket '+str(side),INK,outline,skin,.004);p.append(border)
    center=(sum(a for a,b in outline)/len(outline),sum(b for a,b in outline)/len(outline))
    inset=[(center[0]+(a-center[0])*.83,center[1]+(b-center[1])*.82) for a,b in outline]
    white=fitted_patch('Eye / fitted ivory white '+str(side),WHITE,inset,surface(border),.0025);p.append(white)
    if blue:
        iris=fitted_patch('Eye / fitted blue iris '+str(side),BLUE,oval(x,y-.004,w*.40,h*.61),surface(white),.002);p.append(iris)
        pupil=fitted_patch('Eye / dark vertical pupil '+str(side),INK,oval(x,y-.004,w*.18,h*.49,14),surface(iris),.001);p.append(pupil)
        p.append(fitted_patch('Eye / white glint '+str(side),WHITE,oval(x-.0015,y+.008,.002,.0055,12),surface(pupil),.001))
    else:p.append(fitted_patch('Eye / dark vertical pupil '+str(side),INK,oval(x,y-.002,.0038,.012,14),surface(white),.001))

def miltank():
    body=loft('Body / continuous plump pink milk-cow pear',PINK,curved_profiles(((.115,.045,.050,0),(.17,.20,.165,-.005),(.30,.284,.22,-.015),(.43,.28,.22,-.015),(.57,.235,.18,-.010),(.66,.145,.12,.0),(.705,.04,.04,0)),4),36)
    head=ellipsoid('Head / rounded dark cow hood',HOOD,(0,.82,.015),(.205,.19,.18),32,20)
    p=[body,head];hs=surface(head)
    face=fitted_patch('Face / fitted pink cow mask',PINK,[(-.14,.89),(-.10,.97),(0,.982),(.10,.97),(.14,.89),(.14,.77),(.09,.71),(-.09,.71),(-.14,.77)],hs,.020);p.append(face);fs=surface(face)
    muzzle=ellipsoid('Muzzle / short rounded pink snout',PINK,(0,.806,fs(.806,0)+.018),(.112,.052,.042),28,16);p.append(muzzle)
    p.append(fitted_patch('Nose / single dark oval cow nose',HOOD,oval(.025,.836,.031,.017),surface(muzzle),.004))
    for side in (-1,1):
        eye(p,fs,side*.080,.899,.034,.056,True)
        outline=[(side*.14,.90),(side*.22,1.045),(side*.34,1.09),(side*.31,.97),(side*.19,.875)]
        p.append(membrane('Ear / broad dark cow pinna '+str(side),HOOD,outline,1,root_z=.018,thickness=.027))
        p.append(membrane('Ear / pale inset '+str(side),MILK,outline,1,root_z=.018,inset=.64,offset=.028,thickness=.004))
        p.append(curved_tube('Horn / short ivory cow horn '+str(side),WHITE,[(side*.094,.963,.005),(side*.108,1.014,.014),(side*.123,1.050,.025)],[.025,.018,.001],18))
        p.append(curved_tube('Arm / raised pink forelimb '+str(side),PINK,[(side*.17,.55,-.005),(side*.27,.59,.01),(side*.33,.685,.035)],[.079,.065,.051],18))
        hand=ellipsoid('Hand / dark cloven hoof '+str(side),HOOD,(side*.335,.695,.037),(.055,.044,.055),20,14);p.append(hand)
        ss=surface(hand);points=[(side*.335,y,ss(y,side*.335)+.001) for y in (.712,.70,.683)]
        p.append(tube('Hand / fitted cloven groove '+str(side),HOOF,points,[.0018]*3,8))
        p.append(ellipsoid('Leg / pink supporting haunch '+str(side),PINK,(side*.17,.155,.035),(.097,.117,.114),24,16))
        foot=ellipsoid('Foot / grounded dark cow hoof '+str(side),HOOD,(side*.178,.047,.070),(.087,.047,.11),24,16);p.append(foot)
        ss=surface(foot);points=[(side*.178,y,ss(y,side*.178)+.001) for y in (.061,.046,.026)]
        p.append(tube('Foot / fitted cloven groove '+str(side),HOOF,points,[.002]*3,8))
    udder=ellipsoid('Udder / broad cream belly bowl',MILK,(0,.365,.231),(.225,.226,.069),32,20);p.append(udder);us=surface(udder)
    for j,(x,y) in enumerate(((-.09,.435),(.087,.458),(-.070,.257),(.105,.292))):
        z=us(y,x)
        p.append(curved_tube('Teat / rounded raised pink teat '+str(j),PINK,[(x,y,z-.006),(x+.004,y+.025,z+.023),(x+.006,y+.047,z+.037),(x+.007,y+.055,z+.037)],[.024,.021,.013,.0015],18))
    mouth=fitted_patch('Mouth / fitted cheerful open grin',INK,[(-.055,.773),(.079,.782),(.044,.723),(-.028,.722)],fs,.007);p.append(mouth)
    p.append(fitted_patch('Mouth / fitted soft pink tongue',TONGUE,[(-.023,.745),(.036,.747),(.025,.731),(-.014,.729)],surface(mouth),.002))
    rear=reverse_front(body);rs=surface(rear)
    for j,(x,y,w,h) in enumerate(((-.105,.49,.056,.060),(.105,.375,.06,.077),(0,.565,.060,.046))):
        p.append(reverse_front(fitted_patch('Spot / fitted dark rear cow marking '+str(j),HOOD,oval(x,y,w,h,24),rs,.003)))
    tail=[(0,.22,-.143),(.07,.16,-.30),(.18,.16,-.43),(.26,.27,-.455)]
    p.append(curved_tube('Tail / thin curved cream whip',MILK,tail,[.020,.017,.015,.012],14))
    p.append(ellipsoid('Tail / rounded dark terminal ball',HOOD,tail[-1],(.052,.049,.052),24,16))
    return p

def tauros():
    rows=curved_profiles(((-.36,.025,.03,.43),(-.28,.17,.18,.45),(-.10,.205,.20,.46),(.11,.185,.20,.47),(.23,.13,.15,.49),(.29,.03,.04,.52)),4)
    n=32;v=[]
    for z,w,h,y in rows:
        for k in range(n):a=k*math.tau/n;v.append((w*math.sin(a),y+h*math.cos(a),z))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+k for k in range(n))]
    for j in range(len(rows)-1):
        for k in range(n):a=j*n+k;b=j*n+(k+1)%n;f.append((a,b,b+n,a+n))
    body=crease_normals(part('Body / continuous heavy golden bull trunk',GOLD,v,f),40.)
    rows=curved_profiles(((.255,.025,.04,.245),(.32,.20,.19,.24),(.46,.255,.215,.23),(.62,.265,.20,.21),(.79,.22,.15,.18),(.85,.15,.105,.17),(.89,.018,.025,.16)),4);v=[];n=40
    for y,w,d,z in rows:
        for k in range(n):
            a=k*math.tau/n;fade=max(0.,1-(y-.255)/.30);ripple=1+.024*math.cos(14*a)*fade
            v.append((w*math.sin(a)*ripple,y+.016*math.cos(14*a)*fade,z+d*math.cos(a)*ripple))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+k for k in range(n))]
    for j in range(len(rows)-1):
        for k in range(n):a=j*n+k;b=j*n+(k+1)%n;f.append((a,b,b+n,a+n))
    mane=crease_normals(part('Mane / continuous shaggy gray shoulder ruff',MANE,v,f),40.)
    head=loft('Head / tapered golden bull forehead and lowered muzzle',GOLD,curved_profiles(((.545,.025,.028,.560),(.585,.093,.101,.495),(.675,.136,.135,.403),(.79,.134,.117,.36),(.855,.093,.075,.34),(.881,.014,.018,.32)),4),32)
    p=[body,mane,head];hs=surface(head);ms=surface(mane)
    for side in (-1,1):
        eye(p,hs,side*.090,.709,.026,.026)
        p.append(curved_tube('Horn / swept gray bull crescent '+str(side),HORN,[(side*.105,.781,.352),(side*.22,.813,.359),(side*.295,.904,.36),(side*.297,1.009,.37),(side*.263,1.073,.386)],[.042,.052,.048,.025,.0015],22))
        for j,x in enumerate((.174,.216)):
            points=[]
            for yy in (.40,.49,.60,.68):
                xx=side*(x-.018*(yy-.4));points.append((xx,yy,ms(yy,xx)+.002))
            p.append(tube(f'Mane crease / fitted shaggy strand {side} {j}',(.33,.31,.30),points,[.002,.0025,.002,.001],8))
        for row in range(2):
            controls=[(side*.060,.51,.17),(side*.14,.43,.18),(side*.145,.27,.22),(side*.15,.055,.26)] if row==0 else [(side*.06,.48,-.235),(side*.145,.42,-.235),(side*.16,.27,-.28),(side*.17,.15,-.22),(side*.18,.055,-.215)]
            radii=[.036,.063,.049,.038] if row==0 else [.035,.076,.063,.046,.034]
            rows=curved_profiles([(q[1],r,q[0],q[2]) for q,r in reversed(list(zip(controls,radii)))],4)
            p.append(crease_normals(tube(f'Leg / curved golden bovine support {side} {row}',GOLD,[(x,y,z) for y,r,x,z in rows],[r for y,r,x,z in rows],18),35.))
            px=controls[-1][0];pz=controls[-1][2]+.015
            hoof=ellipsoid(f'Hoof / grounded blue-gray cloven pad {side} {row}',HOOF,(px,.045,pz),(.067,.045,.088),24,16);p.append(hoof)
            ss=surface(hoof);points=[(px,y,ss(y,px)+.001) for y in (.060,.044,.025)]
            p.append(tube(f'Hoof groove / fitted cleft {side} {row}',INK,points,[.002]*3,8))
    for j,y in enumerate((.819,.765,.702)):
        border=fitted_patch('Rivet / dark forehead rim '+str(j),BROWN,oval(0,y,.024,.030),hs,.003);p.append(border)
        p.append(fitted_patch('Rivet / gray forehead stud '+str(j),HORN,oval(0,y,.019,.024),surface(border),.002))
    nose=ellipsoid('Nose / flat gray bovine muzzle cap',MANE,(0,.583,hs(.583,0)+.003),(.064,.027,.017),24,14);p.append(nose);ns=surface(nose)
    for side in (-1,1):p.append(fitted_patch('Nostril / fitted dark oval '+str(side),INK,oval(side*.028,.584,.010,.006,14),ns,.002))
    paths=[[(0,.49,-.285),(-.09,.65,-.43),(-.25,.89,-.61),(-.29,1.08,-.61),(-.16,1.13,-.58)],[(.018,.49,-.29),(.12,.63,-.47),(.32,.67,-.60),(.39,.80,-.63),(.32,.92,-.65)],[(-.01,.48,-.28),(-.02,.55,-.47),(.04,.45,-.65),(.14,.41,-.68),(.24,.49,-.65)]]
    for j,points in enumerate(paths):
        p.append(curved_tube('Tail / separate curved brown whip '+str(j),BROWN,points,[.018,.014,.012,.011,.009],14))
        x,y,z=points[-1];outline=[(x+a,y+b) for a,b in ((0,-.025),(-.045,.012),(-.070,.067),(-.025,.045),(-.019,.10),(.005,.056),(.038,.078),(.032,.024),(.008,-.025))]
        root=z+.06*abs(x)-.025*(y-.70)**2
        p.append(membrane('Tail / pointed blue-gray tuft '+str(j),HOOF,outline,1,root_z=root,thickness=.015))
    return p

def sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported bovine')
    p=miltank() if species=='miltank' else tauros()
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
