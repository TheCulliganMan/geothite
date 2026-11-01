#!/usr/bin/env python3
"""Original layered Pineco and rocky, hollow-port Forretress sculptures."""
import argparse,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,crease_normals
from abra_sculpt import surface,polygon_caps
from bovine_sculpt import oval,curved_tube,reverse_front
from skin_glb import COORDINATES
SPECIES=('pineco','forretress');HEIGHT=.93
TEAL=(.39,.59,.59);NAVY=(.22,.29,.38);DARK=(.12,.17,.22);WHITE=(.97,.96,.90);RED=(.71,.25,.25);RUBY=(.86,.25,.28);STONE=(.84,.79,.81);CHIP=(.57,.52,.56)

def plate(row,j,y,root,tip,width,count=6,phase=0):
    a=j*math.tau/count+phase;c=math.cos(a);s=math.sin(a)
    xy=[(-width*.60,root),(width*.60,root),(width,tip-.05),(width*.76,tip+.01),(0,tip+.027),(-width*.76,tip+.01),(-width,tip-.05)]
    def layer(name,color,lower,upper):
        v=[]
        for offset in (lower,upper):
            for x,r in xy:
                yy=y+.075*(tip-r)/(tip-root)+offset
                v.append((x*c+r*s,yy,r*c-x*s))
        n=len(xy);caps=polygon_caps(xy);f=[tuple(reversed(q)) for q in caps]+[tuple(i+n for i in q) for q in caps]
        f.extend((i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n))
        return part(name,color,v,f)
    return [layer(f'Plate / navy underside {row} {j}',NAVY,-.047,0),layer(f'Plate / teal upper armor {row} {j}',TEAL,0,.022)]

def pineco():
    body=loft('Body / tapered dark pine-cone core',NAVY,curved_profiles(((.018,.025,.025,0),(.11,.16,.14,0),(.25,.22,.19,0),(.43,.23,.20,0),(.59,.18,.16,0),(.69,.075,.08,0),(.724,.02,.025,0)),4),30)
    p=[body];hs=surface(body)
    for row,(y,root,tip,w) in enumerate(((.095,.07,.22,.087),(.23,.13,.29,.118),(.415,.13,.33,.13))):
        for j in range(6):p.extend(plate(row,j,y,root,tip,w,phase=0 if row%2==0 else math.pi/6))
    # Tall upper whorl leaves the eyes visible between its two front plates.
    for j in range(5):
        a=(j+.5)*math.tau/5;c=math.cos(a);s=math.sin(a)
        xy=[(-.045,.13),(.045,.13),(.115,.235),(.12,.29),(.055,.325),(-.02,.30)]
        def upper_layer(name,color,low,high):
            v=[]
            for off in (low,high):
                for x,r in xy:v.append((x*c+r*s,.64+(r-.13)*1.10+off,r*c-x*s))
            n=len(xy);caps=polygon_caps(xy);f=[tuple(reversed(q)) for q in caps]+[tuple(i+n for i in q) for q in caps]
            f.extend((i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n));return part(name,color,v,f)
        p.extend((upper_layer('Crown / navy tall plate '+str(j),NAVY,-.025,0),upper_layer('Crown / teal tall plate '+str(j),TEAL,0,.023)))
    p.append(curved_tube('Stem / tapered teal crown spike',TEAL,[(0,.66,-.015),(0,.79,-.012),(0,.92,-.004)],[.082,.051,.0015],24))
    for side in (-1,1):
        outline=[(side*.022,.613),(side*.135,.641),(side*.15,.572),(side*.025,.571)]
        eye=fitted_patch('Eye / fitted ivory angry pine-cone eye '+str(side),WHITE,outline,hs,.008);p.append(eye)
        iris=fitted_patch('Eye / narrow fitted red iris '+str(side),RUBY,oval(side*.078,.602,.018,.018,20),surface(eye),.003);p.append(iris)
        p.append(fitted_patch('Eye / dark vertical pupil '+str(side),DARK,oval(side*.078,.602,.006,.014,14),surface(iris),.002))
    return p

def rock_shell(upper):
    n=48;rows=12;v=[]
    for inner in (False,True):
        for j in range(rows+1):
            t=j/rows
            for k in range(n):
                a=k*math.tau/n;r=.37*math.sqrt(max(.00018,1-t*t))
                r*=1+.035*math.cos(7*a+.7*t)+.018*math.cos(13*a-.5*t)
                y=.59+.285*t+(.037*math.cos(6*a)+.014*math.cos(11*a))*(1-t)**2
                if inner:r=max(.002,r-.036*(1-.75*t));y-=.022
                # Mirror the lower hemisphere around the equatorial center.
                if not upper:y=.88-y
                v.append((r*math.sin(a),y,r*.88*math.cos(a)))
    count=(rows+1)*n;f=[]
    for j in range(rows):
        for k in range(n):
            a=j*n+k;b=j*n+(k+1)%n;f.extend(((a,b,b+n,a+n),(a+count+n,b+count+n,b+count,a+count)))
    for k in range(n):
        a=k;b=(k+1)%n;f.append((b,a,a+count,b+count))
    f.append(tuple(rows*n+k for k in range(n)))
    f.append(tuple(reversed([count+rows*n+k for k in range(n)])))
    return crease_normals(part('Shell / rocky '+('upper' if upper else 'lower')+' hemisphere',STONE,v,f),38.)

def annulus(name,color,skin,cx,cy,outer,inner):
    n=36;v=[]
    for offset in (.004,-.002):
        for radius in (inner,outer):
            for k in range(n):
                a=k*math.tau/n;x=cx+radius*math.cos(a);y=cy+radius*math.sin(a);v.append((x,y,skin(y,x)+offset))
    f=[]
    for k in range(n):
        a=k;b=(k+1)%n;f.extend(((a,b,b+n,a+n),(a+2*n+n,b+2*n+n,b+2*n,a+2*n),(b,a,a+2*n,b+2*n),(a+n,b+n,b+3*n,a+3*n)))
    return crease_normals(part(name,color,v,f),35.)

def port(side,j):
    # A real annular cone, with its rear recessed into the red core.
    n=32;v=[];origin=(side*.24,.455 if j==0 else .333,.12 if j==0 else -.13)
    axis=(side*.96,.04,.277 if j==0 else -.277);u=(0,1,0);w=(-axis[2],0,axis[0])
    for inner in (False,True):
        for d,r in ((0,.079),(.04,.083),(.235,.046)):
            if inner:r-=.018
            for k in range(n):
                a=k*math.tau/n;v.append(tuple(origin[h]+axis[h]*d+r*(u[h]*math.cos(a)+w[h]*math.sin(a)) for h in range(3)))
    f=[];count=3*n
    for jrow in range(2):
        for k in range(n):
            a=jrow*n+k;b=jrow*n+(k+1)%n;f.extend(((a,b,b+n,a+n),(a+count+n,b+count+n,b+count,a+count)))
    for k in range(n):
        a=k;b=(k+1)%n;f.extend(((b,a,a+count,b+count),(a+2*n,b+2*n,b+count+2*n,a+count+2*n)))
    barrel=crease_normals(part(f'Port / hollow tapered red barrel {side} {j}',RED,v,f),35.)
    center=tuple(origin[h]+axis[h]*.090 for h in range(3))
    recess=ellipsoid(f'Port recess / deep dark bore {side} {j}',(.18,.09,.10),center,(.017,.062,.062),20,14)
    return [barrel,recess]

def belt():
    n=40;v=[]
    for factor in (1.012,.99):
        for y in (.428,.455):
            r=math.sqrt(1-((y-.44)/.33)**2)
            for k in range(n):
                a=k*math.tau/n;v.append((.365*r*factor*math.sin(a),y,.325*r*factor*math.cos(a)))
    f=[]
    for k in range(n):
        a=k;b=(k+1)%n;f.extend(((a,b,b+n,a+n),(a+3*n,b+3*n,b+2*n,a+2*n),(b,a,a+2*n,b+2*n),(a+n,b+n,b+3*n,a+3*n)))
    return crease_normals(part('Belt / continuous dark equatorial seam',DARK,v,f),35.)

def forretress():
    core=ellipsoid('Core / continuous exposed red bug body',RED,(0,.44,0),(.365,.330,.325),36,22)
    p=[core,rock_shell(True),rock_shell(False)];hs=surface(core)
    p.append(belt())
    for side in (-1,1):
        x=side*.119;y=.44
        eye=fitted_patch('Eye / broad dark round socket '+str(side),DARK,oval(x,y,.082,.082,32),hs,.013);p.append(eye)
        p.append(annulus('Eye / ivory annular pupil rim '+str(side),WHITE,surface(eye),x,y,.047,.033))
        for j in range(2):p.extend(port(side,j))
    for upper in (True,False):
        shell=p[1] if upper else p[2]
        for rear in (False,True):
            skin=surface(reverse_front(shell) if rear else shell)
            for j,(x,y,w,h) in enumerate(((-.17,.71,.032,.019),(.105,.79,.024,.012),(.225,.64,.024,.022),(0,.835,.021,.009))):
                if not upper:y=.88-y
                outline=[(x-w,y),(x-w*.35,y+h),(x+w*.60,y+h*.65),(x+w,y-h*.20),(x+w*.15,y-h), (x-w*.55,y-h*.55)]
                dark=fitted_patch(f'Chip / fitted stone depression {upper} {rear} {j}',CHIP,outline,skin,.003)
                inset=[(x+(xx-x)*.78,y+(yy-y)*.72) for xx,yy in outline]
                face=fitted_patch(f'Chip / inset pale stone face {upper} {rear} {j}',STONE,inset,surface(dark),.0015)
                p.extend((reverse_front(dark),reverse_front(face)) if rear else (dark,face))
    return p

def sculpture(species,normalize=True):
    if species not in SPECIES:raise ValueError('unsupported armored bagworm')
    p=pineco() if species=='pineco' else forretress()
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        if normalize:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    from armored_bug_glb import export_armored_bug
    for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_armored_bug(s))
if __name__=='__main__':main()
