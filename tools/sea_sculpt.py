#!/usr/bin/env python3
"""Original branching Corsola and curved, two-tone Mantine sculptures."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,translate,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from spider_sculpt import dorsal_patch,upper_skin
from skin_glb import COORDINATES
from bovine_sculpt import curved_tube,oval
from wing_insect_sculpt import contour
from skarmory_sculpt import blade
SPECIES=('corsola','mantine');HEIGHTS={'corsola':.93,'mantine':1.02}
PINK=(.95,.55,.65);WHITE=(.87,.95,.95);BLUE=(.29,.43,.53);BELLY=(.84,.81,.88);INK=(.10,.12,.15)

def face(p,body,y,spread,eye_size):
    skin=surface(body)
    for side in (-1,1):
        x=side*spread
        eye=ellipsoid('Eye / glossy oval '+str(side),INK,(x,y,skin(y,x)+.006),(eye_size,eye_size*1.5,.012),20,14);p.append(eye)
        p.append(ellipsoid('Eye / white glint '+str(side),WHITE,(x-.003,y+eye_size*.55,surface(eye)(y+eye_size*.55,x-.003)+.001),(.0045,.006,.003),12,8))
    points=[]
    for i in range(13):
        x=(i/12-.5)*spread*1.5;yy=y-.065+.034*(x/(spread*.75))**2
        points.append((x,yy,skin(yy,x)+.002))
    p.append(tube('Mouth / fitted curved smile',INK,points,[.0025]*len(points),10))

def corsola():
    body=loft('Body / rounded pink coral crown',PINK,curved_profiles(((.15,.16,.14,0),(.20,.25,.21,0),(.31,.30,.235,0),(.40,.27,.21,-.01),(.47,.19,.15,-.01),(.49,.02,.02,-.01)),4),40)
    p=[body,loft('Belly / white lower bowl',WHITE,curved_profiles(((.07,.03,.035,0),(.085,.16,.14,0),(.13,.25,.205,0),(.20,.265,.22,0)),4),40)]
    for s in (-1,1):
        for j,z in enumerate((-.13,.14)):
            p.append(ellipsoid(f'Foot / white grounded pad {s} {j}',WHITE,(s*.16,.065,z),(.067,.065,.070),20,12))
        p.append(tube('Side arm / pink drooping branch '+str(s),PINK,[(s*.24,.24,.09),(s*.34,.18,.14),(s*.365,.14,.16)],[.082,.065,.042],20))
        p.append(ellipsoid('Side arm / rounded tip '+str(s),PINK,(s*.365,.14,.16),(.042,)*3,18,12))
    paths=[[( -.15,.40,.00),(-.24,.56,.01),(-.34,.72,.00),(-.41,.82,-.01)],[(.14,.40,-.015),(.24,.58,-.025),(.28,.78,-.04),(.30,.94,-.055)],[(0,.42,-.12),(-.03,.57,-.18),(-.08,.71,-.22)],[(.18,.37,-.10),(.34,.48,-.17),(.44,.61,-.20)],[(-.19,.35,-.10),(-.34,.44,-.17),(-.43,.57,-.22)]]
    for i,path in enumerate(paths):
        radii=[.10,.077,.050,.038] if len(path)==4 else [.087,.066,.039]
        p.append(tube('Coral / rounded primary branch '+str(i),PINK,path,radii,24))
        p.append(ellipsoid('Coral / smooth primary tip '+str(i),PINK,path[-1],(radii[-1],)*3,20,12))
        root=path[-2];s=-1 if root[0]<0 else 1;end=(root[0]+s*.13,root[1]+.014,root[2]+.018)
        p.append(tube('Coral / thick lateral fork '+str(i),PINK,[root,((root[0]+end[0])/2,root[1]+.018,end[2]),end],[radii[-2]*.92,.043,.032],20))
        p.append(ellipsoid('Coral / rounded fork tip '+str(i),PINK,end,(.032,)*3,18,12))
    skin=surface(body)
    for s in (-1,1):
        x=s*.22;y=.28
        p.append(fitted_patch('Belly fleck / fitted white diamond '+str(s),WHITE,[(x-.017,y),(x,y+.022),(x+.017,y),(x,y-.017)],skin,.003))
    face(p,body,.33,.115,.016)
    return p

def wing_rows():
    return curved_profiles(((.07,.038,.29,-.04),(.23,.055,.34,-.06),(.43,.050,.32,-.08),(.65,.037,.27,-.055),(.83,.023,.20,-.00),(.95,.012,.095,.055),(.99,.003,.009,.09)),4)
def wing(side,upper):
    rows=wing_rows()
    v=[];n=19
    for x,t,d,z in rows:
        y=.38+.30*(x/.99)**1.8
        for i in range(n):
            a=math.pi*i/(n-1);zz=z+d*math.cos(a)
            v.append((side*x,y+(1 if upper else -1)*t*math.sin(a),zz))
    f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    for j in range(len(rows)-1):
        for i in range(n-1):a=j*n+i;f.append((a,a+1,a+n+1,a+n))
        a=j*n;f.append((a,a+n,a+2*n-1,a+n-1))
    return crease_normals(part(('Wing / curved blue back ' if upper else 'Wing / pale curved underside ')+str(side),BLUE if upper else BELLY,v,f),35.)

def underside(p):
    q=dict(p)
    for field in ('positions','normals'):q[field]=[v for i in range(0,len(p[field]),3) for v in (p[field][i],p[field][i+2],-p[field][i+1])]
    return q

def from_underside(p):
    q=dict(p)
    for field in ('positions','normals'):q[field]=[v for i in range(0,len(p[field]),3) for v in (p[field][i],-p[field][i+2],p[field][i+1])]
    return q

def wing_border(side,wing):
    rows=wing_rows()
    def leading(x):
        for a,b in zip(rows,rows[1:]):
            if a[0]<=x<=b[0]:
                t=(x-a[0])/(b[0]-a[0]);return a[2]+a[3]+t*(b[2]+b[3]-a[2]-a[3])
        raise ValueError('wing border outside fin')
    xs=[.13+i*.045 for i in range(19)]
    xy=[(side*x,leading(x)-.008) for x in xs]
    xy += [(side*x,leading(x)-(.035 if i%2==0 else .078)) for i,x in reversed(list(enumerate(xs)))]
    return from_underside(fitted_contour('Wing border / fitted blue zigzag underside '+str(side),BLUE,xy,surface(underside(wing)),.006,subdivisions=3))

def mantine():
    body=loft('Body / broad pale ray belly',BELLY,curved_profiles(((.26,.05,.08,-.02),(.28,.18,.25,-.01),(.34,.25,.34,0),(.44,.245,.33,0),(.52,.19,.24,.035),(.565,.03,.035,.04)),4),40)
    p=[body]
    outline=[(.17*math.cos(i*math.tau/24),-.07+.20*math.sin(i*math.tau/24)) for i in range(24)]
    p.append(dorsal_patch('Back / fitted blue dorsal mantle',BLUE,outline,upper_skin(body)))
    for side in (-1,1):
        upper=wing(side,True);lower=wing(side,False);p.extend((upper,lower,wing_border(side,lower)))
        fin=loft('Cephalic fin / rounded curved blue leaf paddle '+str(side),BLUE,curved_profiles(((.48,.027,.016,.25),(.60,.037,.018,.33),(.73,.033,.017,.35),(.82,.021,.013,.31),(.845,.003,.004,.29)),4),32)
        fin['positions']=[v+side*(.145+.105*((fin['positions'][i//3*3+1]-.48)/.365)) if i%3==0 else v for i,v in enumerate(fin['positions'])];p.append(fin)
    p.append(curved_tube('Tail / long flexible taper',BLUE,[(0,.37,-.28),(.05,.33,-.47),(.18,.36,-.65),(.27,.40,-.84)],[.045,.040,.026,.012],28))
    tailxy=contour([((.17,-.65),(.26,-.76),(.41,-.95),(.68,-.91)),((.68,-.91),(.49,-.97),(.34,-1.14),(.22,-1.05)),((.22,-1.05),(.17,-.94),(.23,-.80),(.17,-.65))],14)
    tailxy=[(-x,z) for x,z in tailxy]
    leaf=blade('Tail leaf / swept pale terminal ray fin',BELLY,tailxy,lambda x,z:.36+.15*(-z-.65),.017)
    mantle=blade('Tail leaf back / seated blue upper fin',BLUE,tailxy,lambda x,z:.36+.15*(-z-.65)+.013,.005)
    for q in (leaf,mantle):
        # Proper rotation maps the authored cap into the upward-facing tail.
        for field in ('positions','normals'):q[field]=[v for i in range(0,len(q[field]),3) for v in (-q[field][i],q[field][i+2],q[field][i+1])]
        p.append(q)
    face(p,body,.46,.11,.020)
    skin=surface(body)
    for side in (-1,1):
        for j in range(2):
            yy=.34-j*.025;x=side*.17
            points=[(x,yy,skin(yy,x)+.003),(side*.145,yy-.011,skin(yy-.011,side*.145)+.003)]
            p.append(tube(f'Gill / fitted dark slit {side} {j}',INK,points,[.004]*len(points),10))
    green=(.57,.74,.67);fin_green=(.43,.63,.57)
    fish=ellipsoid('Remora / attached sea-green body',green,(.43,.36,.03),(.049,.080,.112),32,22);p.append(fish)
    nose=ellipsoid('Remora nose / rounded forward sucker muzzle',green,(.43,.385,.135),(.033,.036,.036),24,16);p.append(nose)
    p.append(fitted_patch('Remora mouth / fitted small dark sucker opening',INK,oval(.43,.385,.015,.019,28),surface(nose),.003))
    p.append(curved_tube('Remora / slender trailing tail',green,[(.43,.34,-.045),(.44,.30,-.13),(.46,.29,-.18)],[.023,.019,.010],20))
    fork=contour([((.455,.298),(.465,.304),(.485,.322),(.504,.326)),((.504,.326),(.494,.295),(.481,.287),(.485,.266)),((.485,.266),(.470,.279),(.458,.279),(.455,.298))],9)
    p.append(blade('Remora tail / swept two-lobed trailing fin',fin_green,fork,lambda x,y:-.185,.009))
    p.append(ellipsoid('Remora / dorsal attachment fin',fin_green,(.43,.427,.00),(.016,.037,.062),24,16))
    for side in (-1,1):
        x=.43+side*.030;eye=fitted_patch('Remora eye / fitted tiny dark oval '+str(side),INK,oval(x,.400,.007,.009,24),surface(fish),.003);p.append(eye)
        p.append(fitted_patch('Remora glint / fitted ivory sparkle '+str(side),WHITE,oval(x-.001,.404,.002,.003,16),surface(eye),.001))
    return p

def sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported sea sculpture')
    p=corsola() if species=='corsola' else mantine()
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
