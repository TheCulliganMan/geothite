#!/usr/bin/env python3
"""Original Pikachu mouse sculpture with continuous tipped ears and bolt tail."""
import math
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import curved_tube,oval,reverse_front
from skarmory_sculpt import blade
from skin_glb import COORDINATES
HEIGHT=1.00
GOLD=(.99,.84,.27);BROWN=(.56,.35,.22);RED=(.90,.25,.24)
INK=(.08,.09,.10);WHITE=(.99,.99,.94)


def ear(side,start,end):
    # Gold and black sections share the exact ring at t=.74. A single frame
    # and profile across the color seam avoids floating black tip caps.
    dx,dy=end[0]-start[0],end[1]-start[1];length=math.hypot(dx,dy)
    across=(-dy/length,dx/length,0.)
    rings=[];n=20
    for t,w,d in ((0.,.043,.024),(.15,.048,.022),(.35,.046,.020),(.55,.040,.018),
                  (.74,.031,.014),(.87,.020,.009),(1.,.0015,.0015)):
        center=tuple(a+(b-a)*t for a,b in zip(start,end))
        rings.append([(center[0]+across[0]*w*math.cos(i*math.tau/n),
                       center[1]+across[1]*w*math.cos(i*math.tau/n),
                       center[2]+d*math.sin(i*math.tau/n)) for i in range(n)])
    def section(label,color,rows):
        v=[p for row in rows for p in row];f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
        for j in range(len(rows)-1):
            for i in range(n):a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
        return part(label+' '+str(side),color,v,f,True)
    return (section('Ear / elongated gold mouse pinna',GOLD,rings[:5]),
            section('Ear tip / continuous black terminal',INK,rings[4:]))


def sculpture():
    body=loft('Body / continuous plump golden mouse pear',GOLD,curved_profiles(
        ((.075,.055,.060,-.015),(.13,.147,.122,-.01),(.25,.207,.150,-.015),
         (.39,.190,.143,-.014),(.49,.135,.114,-.007),(.56,.095,.086,.008),(.58,.022,.027,.012)),4),28)
    head=loft('Head / broad cheeked golden mouse skull',GOLD,curved_profiles(
        ((.435,.035,.046,.045),(.48,.145,.127,.042),(.56,.207,.164,.025),
         (.64,.217,.168,.006),(.71,.174,.142,-.010),(.755,.109,.094,-.020),
         (.77,.025,.026,-.025)),4),32)
    p=[body,head];hs=surface(head)
    for side in (-1,1):
        eye=fitted_patch('Eye / fitted round dark mouse eye '+str(side),INK,oval(side*.096,.661,.032,.039,28),hs,.004);p.append(eye)
        p.append(fitted_patch('Eye glint / fitted white mouse sparkle '+str(side),WHITE,oval(side*.096-.008,.675,.009,.012,20),surface(eye),.0015))
        p.append(fitted_patch('Cheek / fitted round red electric pouch '+str(side),RED,oval(side*.161,.583,.038,.040,28),hs,.004))
        p.extend(ear(side,(-.113,.713,-.034) if side<0 else (.123,.716,-.041),
                          (-.222,1.02,-.040) if side<0 else (.389,.932,-.085)))
        p.append(curved_tube('Arm / short drooping mouse forelimb '+str(side),GOLD,
            [(side*.13,.441,.035),(side*.192,.378,.066),(side*.207,.327,.105)],[.049,.043,.032],18))
        p.append(ellipsoid('Hand / small golden mouse palm '+str(side),GOLD,(side*.207,.318,.12),(.041,.047,.036),20,12))
        for j in range(4):
            x=side*(.180+.017*j);y=.279+(.003 if j in (1,2) else .008)
            p.append(ellipsoid('Finger / rounded mouse digit '+str(side)+' '+str(j),GOLD,(x,y,.134),(.010,.022,.011),12,8))
        p.append(ellipsoid('Thumb / tucked mouse digit '+str(side),GOLD,(side*.169,.316,.141),(.013,.023,.012),12,8))
        p.append(ellipsoid('Haunch / rooted short golden mouse leg '+str(side),GOLD,(side*.109,.116,.008),(.061,.100,.067),22,14))
        foot=ellipsoid('Foot / planted long golden mouse pad '+str(side),GOLD,(side*.114,.027,.065),(.057,.027,.108),24,14);p.append(foot)
        fs=surface(foot)
        for j,dx in enumerate((-.014,.014)):
            x=side*.114+dx
            points=[(x,y,fs(y,x)+.001) for y in (.023,.031,.039)]
            p.append(tube('Toe crease / fitted short mouse groove '+str(side)+' '+str(j),BROWN,points,[.0015]*len(points),8))
    p.append(fitted_patch('Nose / fitted tiny triangular mouse nose',INK,[(-.009,.615),(.009,.615),(0.,.606)],hs,.003))
    smile=[]
    for i in range(25):
        x=-.055+.11*i/24;y=.560+.013*(abs(x)/.055)**2+.007*math.exp(-(x/.013)**2)
        smile.append((x,y,hs(y,x)+.002))
    p.append(tube('Mouth / fitted gentle double mouse smile',INK,smile,[.0022]*len(smile),8))
    rear=reverse_front(body);rs=surface(rear)
    for j,(y,w) in enumerate(((.337,.126),(.426,.122))):
        outline=[(-w,y-.014),(-w*.85,y+.014),(w*.85,y+.014),(w,y-.014),(w*.80,y-.030),(-w*.80,y-.030)]
        p.append(reverse_front(crease_normals(fitted_contour('Back stripe / fitted brown mouse band '+str(j),BROWN,outline,rs,.004,subdivisions=3),55.)))
    p.append(blade('Tail base / rooted brown lightning stem',BROWN,
        [(0.,.208),(.039,.197),(.121,.288),(.088,.322),(.047,.269)],lambda x,y:-.143-.30*x,.019))
    p.append(blade('Tail / broad squared golden lightning bolt',GOLD,
        [(.088,.322),(.121,.288),(.223,.365),(.181,.428),(.291,.476),(.260,.563),
         (.361,.596),(.321,.743),(.564,.805),(.605,.625),(.431,.542),(.472,.400),
         (.326,.405),(.370,.332),(.223,.289),(.190,.266)],lambda x,y:-.143-.30*x,.021))
    # Keep the broad bolt readable without exceeding the mouse's body scale.
    for q in p:
        if q['part'].startswith(('Tail /','Tail base /')):
            q['positions']=[v*.85 if i%3==0 else v for i,v in enumerate(q['positions'])]
            normals=[]
            for i in range(0,len(q['normals']),3):
                x,y,z=q['normals'][i:i+3];n=(x/.85,y,z);length=math.sqrt(sum(v*v for v in n));normals.extend(v/length for v in n)
            q['normals']=normals
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'pikachu','version':1,'coordinate_system':COORDINATES,'primitives':p}
