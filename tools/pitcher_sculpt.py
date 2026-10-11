#!/usr/bin/env python3
"""Original botanical sculptures for the Bellsprout evolution family.

Hollow bell and pitcher mouths, rolled lips, curved veined leaves and fitted
pigmentation. Canonical static JSON; no downloaded or extracted geometry.
"""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,tube,ellipsoid,curved_profiles,fitted_patch,crease_normals
from skin_glb import COORDINATES

SPECIES=('bellsprout','weepinbell','victreebel')
HEIGHT=1.05
GOLD=(.96,.79,.24);LIP=(.88,.39,.48);GREEN=(.27,.57,.25)
LEAF=(.39,.68,.27);VEIN=(.60,.77,.32);STEM=(.52,.35,.19)
INK=(.065,.055,.065);WHITE=(.98,.97,.87)


def leaf(name,start,end,width,color=LEAF,bend=.045):
    # Cross-section loft: pointed outline, raised midrib and gently cupped edges.
    a=start;b=end;axis=[b[i]-a[i] for i in range(3)]
    length=math.hypot(axis[0],axis[1]);side=(-axis[1]/length,axis[0]/length,0.)
    verts=[];faces=[];rows=15;segments=12
    centers=[]
    for j in range(rows):
        t=j/(rows-1);bulge=math.sin(math.pi*(.012+.976*t))
        center=tuple(a[i]+axis[i]*t+(bend*math.sin(math.pi*t) if i==2 else 0.) for i in range(3));centers.append(center)
        for k in range(segments):
            angle=k*math.tau/segments;s=math.sin(angle);c=math.cos(angle)
            verts.append((center[0]+side[0]*width*bulge*s,
                          center[1]+side[1]*width*bulge*s,
                          center[2]+.018*bulge*c-.055*bulge*s*s))
    faces=[tuple(reversed(range(segments))),tuple((rows-1)*segments+k for k in range(segments))]
    for j in range(rows-1):
        for k in range(segments):
            a=j*segments+k;b=j*segments+(k+1)%segments
            faces.append((a,b,b+segments,a+segments))
    result=[crease_normals(part(name+' / cupped blade',color,verts,faces,True))]
    vein=[(x,y,z+.019*math.sin(math.pi*(.012+.976*j/(rows-1)))) for j,(x,y,z) in enumerate(centers)]
    result.append(tube(name+' / raised midrib',VEIN,vein,[.002+.003*math.sin(math.pi*j/(rows-1)) for j in range(rows)],8))
    return result


def funnel(name,color,profiles,center,up=False,thickness=.013,tilt=0.,segments=32):
    # Annular wall follows both outer and inner contours. The opening has real
    # depth, with no opaque sphere crossing its visible mouth.
    v=[];f=[]
    for inside in (False,True):
        for distance,w,h in profiles:
            for k in range(segments):
                a=k*math.tau/segments;x=(w-(thickness if inside else 0.))*math.sin(a)
                q=(h-(thickness if inside else 0.))*math.cos(a)
                v.append((center[0]+x,center[1]+(distance if up else q+tilt*(profiles[-1][0]-distance)),center[2]+(-q if up else distance)))
    count=len(profiles)*segments
    for j in range(len(profiles)-1):
        for k in range(segments):
            a=j*segments+k;b=j*segments+(k+1)%segments
            f.extend(((a,b,b+segments,a+segments),(a+count,a+segments+count,b+segments+count,b+count)))
    for j in (0,len(profiles)-1):
        for k in range(segments):
            a=j*segments+k;b=j*segments+(k+1)%segments
            f.append((a,a+count,b+count,b) if j==0 else (b,b+count,a+count,a))
    return crease_normals(part(name,color,v,f,True))


def rolled_lip(name,center,rx,ry,minor,up=False):
    v=[];f=[];segments=32;sides=10
    for i in range(segments):
        a=i*math.tau/segments
        for j in range(sides):
            b=j*math.tau/sides;x=(rx+minor*math.cos(b))*math.sin(a)
            q=(ry+minor*math.cos(b))*math.cos(a);depth=minor*math.sin(b)
            v.append((center[0]+x,center[1]+(depth if up else q),center[2]+(-q if up else depth)))
    for i in range(segments):
        for j in range(sides):f.append((i*sides+j,i*sides+(j+1)%sides,((i+1)%segments)*sides+(j+1)%sides,((i+1)%segments)*sides+j))
    return part(name,LIP,v,f,True)


def surface(profiles,y,x):
    for (a,w,d,z),(b,ww,dd,zz) in zip(profiles,profiles[1:]):
        if a<=y<=b:
            t=(y-a)/(b-a);w+=(ww-w)*t;d+=(dd-d)*t;z+=(zz-z)*t
            return z+d*math.sqrt(max(0.,1.-(x/w)**2))
    raise ValueError('marking outside pitcher')


def sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported pitcher species')
    p=[]
    if species=='bellsprout':
        stem=[(0,.055,0),(-.025,.18,-.008),(.025,.34,-.025),(.014,.49,-.02),(-.014,.74,-.04),(0,.87,-.065)]
        p.append(tube('Stem / sinuous continuous stalk',STEM,stem,[.026,.024,.027,.026,.024,.025],16))
        for s in (-1,1):
            root=[(0,.076,0),(s*.08,.045,.024),(s*.15,.031,.074),(s*.22,.018,.12)]
            p.append(tube('Root / spreading toe '+str(s),STEM,root,[.025,.026,.021,.009],12))
            p.extend(leaf('Leaf / expressive arm '+str(s),(.01,.42,-.02),(s*.40,.59,.035),.115))
        p.append(funnel('Head / flared hollow trumpet',GOLD,((-.115,.044,.051),(-.045,.088,.102),(.035,.129,.153),(.13,.170,.177),(.235,.196,.163)),(0,.79,0),tilt=.5))
        p.append(rolled_lip('Mouth / rolled coral bell rim',(0,.79,.239),.195,.162,.022))
        p.append(ellipsoid('Mouth / recessed throat',INK,(0,.79,.125),(.142,.128,.008)))
        for s in (-1,1):
            p.append(ellipsoid('Eye / small black oval '+str(s),INK,(s*.075,.997,.145),(.015,.022,.010),12,8))
            p.append(ellipsoid('Eye / tiny catchlight '+str(s),WHITE,(s*.075-.004,1.004,.154),(.004,.006,.002),8,6))
    else:
        mature=species=='victreebel'
        profiles=((.02,.018,.023,-.018),(.09,.091,.098,-.008),(.23,.17,.155,0),(.41,.229,.196,0),(.60,.251,.21,-.014),(.76,.236,.198,-.025),(.775,.21,.177,-.019)) if mature else ((.02,.023,.025,0),(.10,.13,.117,0),(.24,.226,.197,-.012),(.43,.271,.228,-.02),(.60,.25,.208,-.035),(.71,.209,.163,-.024),(.79,.153,.107,-.028),(.82,.036,.032,-.03))
        body=curved_profiles(profiles,3)
        p.append(loft('Body / tapered hanging pitcher' if not mature else 'Body / tall continuous pitcher',GOLD,body,32))
        for s in (-1,1):
            p.extend(leaf('Leaf / lateral blade '+str(s),(s*.18,.40,-.012),(s*(.58 if mature else .49),.65 if mature else .60,.034),.135))
        if mature:
            # Mouth opens upward into a deep bowl; the hood grows from its back.
            p.append(funnel('Mouth / hollow upward bowl',GOLD,((.755,.231,.193),(.86,.228,.190),(.925,.236,.191)),(0,0,-.02),True))
            p.append(rolled_lip('Mouth / heavy rolled pitcher rim',(0,.923,-.02),.236,.191,.031,True))
            p.append(funnel('Mouth / dark fitted cavity lining',(.12,.15,.065),((.780,.215,.178),(.902,.221,.175)),(0,0,-.02),True,thickness=.004,segments=24))
            p.append(ellipsoid('Mouth / deep shaded bowl',INK,(0,.815,-.02),(.191,.010,.16)))
            p.extend(leaf('Hood / overhanging crown',(0,.91,-.17),(.10,1.30,-.025),.255,GREEN,.13))
            vine=[(0,.85,-.15),(.11,1.0,-.23),(.26,1.13,-.28),(.39,1.09,-.25),(.43,.97,-.19)]
            p.append(tube('Vine / long curved tendril',STEM,vine,[.024,.022,.020,.016,.007],14))
            ey=.726;spread=.139
            for s in (-1,1):p.append(tube('Fang / inward hooked ivory tooth '+str(s),WHITE,[(s*.148,.91,.125),(s*.127,.859,.112),(s*.109,.825,.091)],[.025,.019,.002],12))
        else:
            p.append(funnel('Mouth / flared forward funnel',GOLD,((.144,.094,.088),(.23,.134,.116),(.318,.17,.141)),(0,.545,0)))
            p.append(rolled_lip('Mouth / rolled forward rim',(0,.545,.32),.17,.141,.023))
            p.append(ellipsoid('Mouth / recessed dark throat',INK,(0,.545,.235),(.13,.111,.006)))
            p.append(tube('Hook / curved rear attachment',STEM,[(0,.74,-.085),(.035,.86,-.16),(.09,.93,-.17),(.14,.91,-.15)],[.025,.023,.018,.008],14))
            ey=.735;spread=.094
        surf=lambda y,x:surface(body,y,x)
        for s in (-1,1):
            x=s*spread;z=surf(ey,x)
            if mature:p.append(ellipsoid('Eye / narrow ivory sclera '+str(s),WHITE,(x,ey,z+.005),(.050,.027,.011),16,8))
            p.append(ellipsoid('Eye / fitted black oval '+str(s),INK,(x,ey,z+.015),(.014,.022,.006),12,8))
            p.append(ellipsoid('Eye / restrained glint '+str(s),WHITE,(x-.004,ey+.007,z+.020),(.004,.006,.002),8,6))
        for i,(x,y,rx,ry) in enumerate(((-.10,.23,.042,.024),(.11,.32,.032,.049),(-.137,.46,.025,.038),(.12,.53,.037,.026),(-.075,.62,.025,.021))):
            outline=[(x+rx*math.cos(k*math.tau/16),y+ry*math.sin(k*math.tau/16)) for k in range(16)]
            p.append(fitted_patch('Pigment / fitted irregular green fleck '+str(i),GREEN,outline,surf,.003))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}


def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for species in SPECIES:(args.output/(species+'.mesh.json')).write_text(json.dumps(sculpture(species),separators=(',',':')))

if __name__=='__main__':main()
